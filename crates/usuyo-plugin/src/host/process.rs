//! 起一个插件进程，看着它跑完（协议 §3.2、§3.4）。
//!
//! 进程自己一个进程组；stdout 有上限，stderr 只留最后几行并拆出 `@usuyo ` 事件；
//! 总时长、无进度时长、`waiting` 暂停三种计时；超时或取消时先 SIGTERM 整组，
//! 宽限期后 SIGKILL 整组。

use std::collections::VecDeque;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time::{Instant, sleep_until, timeout};

use super::{CallLimits, CallOptions};
use crate::protocol::PluginEvent;

/// stderr 一行最多留多少字节，免得没有换行的输出撑爆内存。
const STDERR_LINE_BYTES: usize = 8 * 1024;

/// 要起的进程。环境是全部的环境：起之前清空。
pub(crate) struct Spawn {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub env: Vec<(OsString, OsString)>,
    pub cwd: PathBuf,
}

pub(crate) enum Ending {
    Exited(ExitStatus),
    TimedOut,
    Cancelled,
    StdoutOverflow,
}

pub(crate) struct Finished {
    pub ending: Ending,
    pub stdout: Vec<u8>,
    pub stderr: Vec<String>,
}

pub(crate) async fn run(
    spawn: Spawn,
    total: Duration,
    limits: &CallLimits,
    opts: &CallOptions,
) -> io::Result<Finished> {
    let mut cmd = Command::new(&spawn.program);
    cmd.args(&spawn.args)
        .env_clear()
        .envs(spawn.env.iter().map(|(k, v)| (k, v)))
        .current_dir(&spawn.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    cmd.process_group(0);
    // §7 的沙箱（Seatbelt、bubblewrap、Job Object）接在这里：包住 program 和 args 再起。
    let mut child = cmd.spawn()?;
    // 进程组 id 就是插件的 pid；插件退出被回收以后 `child.id()` 拿不到了，先记下
    let group = child.id();

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("child stdout is not piped"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("child stderr is not piped"))?;
    let mut stdout_task = tokio::spawn(read_capped(stdout, limits.stdout_bytes));
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let mut stderr_task = tokio::spawn(read_stderr(stderr, limits.stderr_lines, event_tx));

    let start = Instant::now();
    let mut total_deadline = start + total;
    let mut idle_deadline: Option<Instant> = None;
    let mut waiting_since: Option<Instant> = None;
    let mut stdout_result: Option<io::Result<(Vec<u8>, bool)>> = None;
    let mut events_open = true;

    let ending = loop {
        let deadline = match waiting_since {
            Some(since) => since + limits.waiting,
            None => idle_deadline.map_or(total_deadline, |idle| idle.min(total_deadline)),
        };
        tokio::select! {
            status = child.wait() => break Ending::Exited(status?),
            joined = &mut stdout_task, if stdout_result.is_none() => {
                let result = joined.map_err(io::Error::other)?;
                let overflow = matches!(result, Ok((_, true)));
                stdout_result = Some(result);
                if overflow {
                    break Ending::StdoutOverflow;
                }
            }
            event = event_rx.recv(), if events_open => match event {
                None => events_open = false,
                Some(event) => {
                    let now = Instant::now();
                    match &event {
                        PluginEvent::Progress { .. } => {
                            // waiting 期间两种计时都停，下一个 progress 恢复
                            if let Some(since) = waiting_since.take() {
                                total_deadline += now - since;
                            }
                            idle_deadline = Some(now + limits.idle);
                        }
                        PluginEvent::Waiting { .. } => {
                            waiting_since.get_or_insert(now);
                        }
                    }
                    if let Some(tx) = &opts.events {
                        // 调用方不再听事件不算错
                        tx.send(event).ok();
                    }
                }
            },
            () = sleep_until(deadline) => break Ending::TimedOut,
            () = cancelled(opts) => break Ending::Cancelled,
        }
    };

    let graceful = !matches!(ending, Ending::Exited(_));
    stop_group(&mut child, group, graceful.then_some(limits.grace)).await;

    let (stdout, overflow) = match stdout_result {
        Some(result) => result?,
        None => match timeout(limits.grace, &mut stdout_task).await {
            Ok(joined) => joined.map_err(io::Error::other)??,
            Err(_) => {
                stdout_task.abort();
                (Vec::new(), false)
            }
        },
    };
    // 插件写超了上限后才退出，也按超限算
    let ending = if overflow {
        Ending::StdoutOverflow
    } else {
        ending
    };
    let stderr = match timeout(limits.grace, &mut stderr_task).await {
        Ok(joined) => joined.map_err(io::Error::other)?,
        Err(_) => {
            stderr_task.abort();
            Vec::new()
        }
    };
    Ok(Finished {
        ending,
        stdout,
        stderr,
    })
}

async fn cancelled(opts: &CallOptions) {
    match &opts.cancel {
        Some(token) => token.cancelled().await,
        None => std::future::pending().await,
    }
}

/// 收尾：要宽限时先 SIGTERM 整组、等一会儿；最后总是 SIGKILL 整组，插件留下的子进程一并结束。
async fn stop_group(child: &mut Child, group: Option<u32>, grace: Option<Duration>) {
    #[cfg(unix)]
    {
        use nix::sys::signal::{Signal, killpg};
        use nix::unistd::Pid;

        let group = group
            .and_then(|pid| i32::try_from(pid).ok())
            .map(Pid::from_raw);
        if let Some(group) = group {
            if let Some(grace) = grace {
                signal_group(killpg(group, Signal::SIGTERM));
                if timeout(grace, child.wait()).await.is_err() {
                    tracing::debug!("plugin ignored SIGTERM for {grace:?}");
                }
            }
            signal_group(killpg(group, Signal::SIGKILL));
        }
    }
    #[cfg(not(unix))]
    {
        // Windows 上终止整个 Job Object 的做法接在这里（§3.4），现在只结束插件本身
        let _ = (group, grace);
        if let Err(e) = child.start_kill() {
            tracing::debug!("kill plugin: {e}");
        }
    }
    if let Err(e) = child.wait().await {
        tracing::warn!("wait for plugin: {e}");
    }
}

#[cfg(unix)]
fn signal_group(result: nix::Result<()>) {
    match result {
        // 进程组已经没有成员
        Ok(()) | Err(nix::errno::Errno::ESRCH) => {}
        Err(e) => tracing::warn!("signal plugin process group: {e}"),
    }
}

/// 读 stdout 到上限；超了就停，第二个值为 `true`。
async fn read_capped(mut reader: impl AsyncRead + Unpin, cap: u64) -> io::Result<(Vec<u8>, bool)> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = reader.read(&mut chunk).await?;
        if n == 0 {
            return Ok((buf, false));
        }
        if (buf.len() + n) as u64 > cap {
            return Ok((buf, true));
        }
        buf.extend_from_slice(&chunk[..n]);
    }
}

/// 读 stderr：留最后 `keep` 行，`@usuyo ` 开头的行解析成事件发出去。读错了就停在那里。
async fn read_stderr(
    mut reader: impl AsyncRead + Unpin,
    keep: usize,
    events: mpsc::UnboundedSender<PluginEvent>,
) -> Vec<String> {
    let mut lines = VecDeque::with_capacity(keep.min(1024));
    let mut line = Vec::new();
    let mut chunk = [0u8; 8192];
    let finish = |line: &mut Vec<u8>, lines: &mut VecDeque<String>| {
        let text = String::from_utf8_lossy(line)
            .trim_end_matches('\r')
            .to_owned();
        line.clear();
        if let Some(json) = text.strip_prefix(PluginEvent::STDERR_PREFIX) {
            match serde_json::from_str::<PluginEvent>(json) {
                // 接收端只在调用结束后才关
                Ok(event) => drop(events.send(event)),
                Err(e) => tracing::debug!("unparsed plugin event: {e}"),
            }
        }
        if keep > 0 {
            if lines.len() == keep {
                lines.pop_front();
            }
            lines.push_back(text);
        }
    };
    loop {
        let n = match reader.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                tracing::debug!("read plugin stderr: {e}");
                break;
            }
        };
        for &b in &chunk[..n] {
            if b == b'\n' {
                finish(&mut line, &mut lines);
            } else if line.len() < STDERR_LINE_BYTES {
                line.push(b);
            }
        }
    }
    if !line.is_empty() {
        finish(&mut line, &mut lines);
    }
    lines.into()
}
