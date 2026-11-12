//! 起 ffmpeg、ffprobe 子进程并看着它跑完：参数是向量（不经 shell），超时、取消、卡住时的心跳都在这里。
//!
//! 子进程的 stdin 是空的、环境变量清空。stdout 要么整个收下（ffprobe 的 JSON，有上限），
//! 要么按 ffmpeg 的 `-progress` 一行行读 `out_time_us`；stderr 是不可信的文字：一行最多 8 KiB，
//! 只留调用方要的几种行和最后几行（出错时给日志看）。超时和取消都杀掉子进程再等它收尾，不留孤儿。

use std::collections::VecDeque;
use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/// 隔多久查一次子进程、取消标志、超时。
const POLL: Duration = Duration::from_millis(25);
/// 没有新进度时，每隔多久补发一次心跳（宿主 120 秒没有 `progress` 算卡住）。
const HEARTBEAT: Duration = Duration::from_secs(10);
/// stdout 整个收下时的上限。
const MAX_CAPTURE: u64 = 4 << 20;
/// stderr 一行最多读多少字节；更长的拆成几行。
const MAX_LINE: u64 = 8 << 10;
/// 出错时带给日志的 stderr 最后几行。
const TAIL_LINES: usize = 12;
/// 调用方要留的行最多多少条。
const MAX_KEPT: usize = 200_000;

/// 调用方给的：取消标志和进度口子。
pub trait Supervisor: Sync {
    fn is_cancelled(&self) -> bool;
    fn progress(&self, done: f64, total: f64, unit: &str);
}

/// 不取消、不报进度：启动时问 `-version` 用。
pub struct Unsupervised;

impl Supervisor for Unsupervised {
    fn is_cancelled(&self) -> bool {
        false
    }
    fn progress(&self, _done: f64, _total: f64, _unit: &str) {}
}

/// stdout 怎么处理。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stdout {
    /// 整个收下（上限 4 MiB）。
    Capture,
    /// ffmpeg 的 `-progress pipe:1`：读 `out_time_us`，转成进度。
    FfmpegProgress,
}

pub struct Spec<'a> {
    pub program: &'a Path,
    pub args: Vec<OsString>,
    pub timeout: Duration,
    pub stdout: Stdout,
    /// 进度的总量和单位（秒数用 `sec`）；`FfmpegProgress` 才用。
    pub total: f64,
    pub unit: &'static str,
}

pub struct Finished {
    pub stdout: Vec<u8>,
    /// stderr 里 `keep` 认下的行，按出现的顺序。
    pub kept: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("start {program}: {source}")]
    Spawn { program: PathBuf, source: io::Error },
    #[error("wait for {program}: {source}")]
    Wait { program: PathBuf, source: io::Error },
    #[error("cancelled")]
    Cancelled,
    #[error("{program} ran longer than {limit:?} and was stopped")]
    Timeout { program: PathBuf, limit: Duration },
    #[error("{program} exited with {status}: {tail}")]
    Exited {
        program: PathBuf,
        status: String,
        tail: String,
    },
    #[error("{program} wrote more output than the limit")]
    TooMuchOutput { program: PathBuf },
    #[error("read the output of {program}: {source}")]
    Read { program: PathBuf, source: io::Error },
}

/// 跑完一条命令。`keep` 决定 stderr 的哪些行留给调用方。
pub fn run(
    spec: &Spec<'_>,
    sup: &dyn Supervisor,
    keep: fn(&str) -> bool,
) -> Result<Finished, ExecError> {
    if sup.is_cancelled() {
        return Err(ExecError::Cancelled);
    }
    let program = spec.program.to_owned();
    let mut child = Command::new(spec.program)
        .args(&spec.args)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| ExecError::Spawn {
            program: program.clone(),
            source,
        })?;
    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let done = AtomicU64::new(0f64.to_bits());

    let (status, out, err) = thread::scope(|scope| {
        let out = scope.spawn(|| read_stdout(stdout, spec, sup, &done));
        let err = scope.spawn(|| read_stderr(stderr, keep));
        let status = wait(&mut child, spec, sup, &done);
        // 子进程退出或被杀以后管道关闭，两个读线程跟着结束。
        (status, out.join(), err.join())
    });
    let status = status?;
    let read = |source| ExecError::Read {
        program: program.clone(),
        source,
    };
    let out = out
        .expect("the stdout reader does not panic")
        .map_err(read)?;
    let err = err
        .expect("the stderr reader does not panic")
        .map_err(read)?;
    if out.overflow || err.overflow {
        return Err(ExecError::TooMuchOutput { program });
    }
    if !status.success() {
        return Err(ExecError::Exited {
            program,
            status: status.to_string(),
            tail: err.tail,
        });
    }
    Ok(Finished {
        stdout: out.bytes,
        kept: err.kept,
    })
}

/// 等子进程结束；取消或超时就杀掉。
fn wait(
    child: &mut Child,
    spec: &Spec<'_>,
    sup: &dyn Supervisor,
    done: &AtomicU64,
) -> Result<ExitStatus, ExecError> {
    let started = Instant::now();
    let mut beat = started;
    let stop = |child: &mut Child| {
        // 已经退出时 kill 会报错，不影响接下来 wait。
        let _ = child.kill();
        let _ = child.wait();
    };
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {}
            Err(source) => {
                stop(child);
                return Err(ExecError::Wait {
                    program: spec.program.to_owned(),
                    source,
                });
            }
        }
        if sup.is_cancelled() {
            stop(child);
            return Err(ExecError::Cancelled);
        }
        if started.elapsed() > spec.timeout {
            stop(child);
            return Err(ExecError::Timeout {
                program: spec.program.to_owned(),
                limit: spec.timeout,
            });
        }
        if beat.elapsed() >= HEARTBEAT {
            let at = f64::from_bits(done.load(Ordering::Relaxed));
            sup.progress(at, spec.total, spec.unit);
            beat = Instant::now();
        }
        thread::sleep(POLL);
    }
}

struct StdoutRead {
    bytes: Vec<u8>,
    overflow: bool,
}

fn read_stdout(
    stdout: ChildStdout,
    spec: &Spec<'_>,
    sup: &dyn Supervisor,
    done: &AtomicU64,
) -> io::Result<StdoutRead> {
    match spec.stdout {
        Stdout::Capture => {
            let mut reader = BufReader::new(stdout);
            let mut bytes = Vec::new();
            (&mut reader)
                .take(MAX_CAPTURE + 1)
                .read_to_end(&mut bytes)?;
            let overflow = bytes.len() as u64 > MAX_CAPTURE;
            // 超出上限的部分读掉扔掉，免得子进程写不出去卡住。
            io::copy(&mut reader, &mut io::sink())?;
            Ok(StdoutRead { bytes, overflow })
        }
        Stdout::FfmpegProgress => {
            for line in read_lines(BufReader::new(stdout)) {
                if let Some(sec) = parse_out_time(&line?) {
                    done.store(sec.to_bits(), Ordering::Relaxed);
                    sup.progress(sec, spec.total, spec.unit);
                }
            }
            Ok(StdoutRead {
                bytes: Vec::new(),
                overflow: false,
            })
        }
    }
}

struct StderrRead {
    kept: Vec<String>,
    tail: String,
    overflow: bool,
}

fn read_stderr(stderr: ChildStderr, keep: fn(&str) -> bool) -> io::Result<StderrRead> {
    let mut kept = Vec::new();
    let mut tail = VecDeque::with_capacity(TAIL_LINES);
    let mut overflow = false;
    for line in read_lines(BufReader::new(stderr)) {
        let line = line?;
        if keep(&line) {
            if kept.len() < MAX_KEPT {
                kept.push(line.clone());
            } else {
                overflow = true;
            }
        }
        if tail.len() == TAIL_LINES {
            tail.pop_front();
        }
        tail.push_back(line);
    }
    let tail = tail
        .into_iter()
        .map(|l| l.chars().filter(|c| !c.is_control()).collect::<String>())
        .collect::<Vec<_>>()
        .join(" | ");
    Ok(StderrRead {
        kept,
        tail,
        overflow,
    })
}

/// 按行读，一行最多 `MAX_LINE` 字节（再长的拆开）；不是 UTF-8 的字节换成 U+FFFD。
fn read_lines<R: BufRead>(mut reader: R) -> impl Iterator<Item = io::Result<String>> {
    let mut buf = Vec::new();
    std::iter::from_fn(move || {
        buf.clear();
        match (&mut reader).take(MAX_LINE).read_until(b'\n', &mut buf) {
            Ok(0) => None,
            Ok(_) => {
                if buf.last() == Some(&b'\n') {
                    buf.pop();
                }
                Some(Ok(String::from_utf8_lossy(&buf).into_owned()))
            }
            Err(e) => Some(Err(e)),
        }
    })
}

/// `-progress` 的一行 `out_time_us=<微秒>` 换成秒；`N/A`、负数、别的行都是 `None`。
pub fn parse_out_time(line: &str) -> Option<f64> {
    let micros: i64 = line.strip_prefix("out_time_us=")?.trim().parse().ok()?;
    (micros >= 0).then(|| micros as f64 / 1e6)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_time_lines() {
        assert_eq!(parse_out_time("out_time_us=7500000"), Some(7.5));
        assert_eq!(parse_out_time("out_time_us=N/A"), None);
        assert_eq!(parse_out_time("out_time_us=-5"), None);
        assert_eq!(parse_out_time("out_time=00:00:07.5"), None);
        assert_eq!(parse_out_time("progress=end"), None);
    }

    #[test]
    fn long_and_binary_lines_are_bounded() {
        let mut data = vec![b'a'; MAX_LINE as usize + 10];
        data.extend_from_slice(b"\nok \xff\n");
        let lines: Vec<String> = read_lines(&data[..]).map(Result::unwrap).collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].len(), MAX_LINE as usize);
        assert_eq!(lines[1], "aaaaaaaaaa");
        assert_eq!(lines[2], "ok \u{fffd}");
    }
}
