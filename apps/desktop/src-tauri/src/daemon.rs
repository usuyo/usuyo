//! 内核守护进程：找已经在跑的（`daemon.json` 加 `/health`），没有就把打包带的 `usuyo daemon` 起起来。
//! 按机器的目录和命令行一样：`USUYO_HOME`，没有就是系统的位置。

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;

/// 窗口的 CSP 只放行这个端口（tauri.conf.json），和内核的默认端口一致。
const PORT: u16 = 47615;
const START_TIMEOUT: Duration = Duration::from_secs(20);
const STOP_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Serialize)]
pub struct Connection {
    pub base: String,
    pub token: String,
}

#[derive(Default)]
pub struct Daemon {
    ready: Mutex<Option<Result<Connection, String>>>,
    changed: Condvar,
    /// 壳自己起的那个；退出时停掉。
    owned: Mutex<Option<Child>>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Daemon {
    pub fn start(&self, plugins: &Path) {
        let result = self.connect(plugins).map_err(|e| {
            eprintln!("usuyo: cannot reach the daemon: {e:#}");
            format!("{e:#}")
        });
        *lock(&self.ready) = Some(result);
        self.changed.notify_all();
    }

    /// 等 [`Daemon::start`] 有结果。
    pub fn wait(&self) -> Result<Connection, String> {
        let deadline = Instant::now() + START_TIMEOUT + Duration::from_secs(5);
        let mut ready = lock(&self.ready);
        loop {
            if let Some(r) = ready.as_ref() {
                return r.clone();
            }
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err("the daemon did not start in time".into());
            }
            ready = self
                .changed
                .wait_timeout(ready, left)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
    }

    fn connect(&self, plugins: &Path) -> Result<Connection> {
        let root = machine_root()?;
        if let Some(c) = running(&root)? {
            return Ok(c);
        }
        let mut child = spawn(&root, plugins)?;
        let started = Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                bail!(
                    "the daemon exited ({status}); see {}",
                    log_path(&root).display()
                );
            }
            if let Some(c) = running(&root)? {
                *lock(&self.owned) = Some(child);
                return Ok(c);
            }
            if started.elapsed() > START_TIMEOUT {
                stop_child(child);
                bail!("the daemon did not answer within {START_TIMEOUT:?}");
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// 停掉自己起的那个：先请它收尾（SIGTERM），等一会儿还不走再强停。
    pub fn stop(&self) {
        if let Some(child) = lock(&self.owned).take() {
            stop_child(child);
        }
    }
}

fn stop_child(mut child: Child) {
    #[cfg(unix)]
    if let Ok(pid) = i32::try_from(child.id()) {
        use nix::sys::signal::{Signal, kill};
        use nix::unistd::Pid;
        if kill(Pid::from_raw(pid), Signal::SIGTERM).is_ok() {
            let started = Instant::now();
            while started.elapsed() < STOP_TIMEOUT {
                if let Ok(Some(_)) = child.try_wait() {
                    return;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            eprintln!("usuyo: the daemon did not stop in {STOP_TIMEOUT:?}; killing it");
        }
    }
    if let Err(e) = child.kill() {
        eprintln!("usuyo: cannot kill the daemon: {e}");
    }
    // 收掉僵尸进程。
    let _ = child.wait();
}

/// `daemon.json` 在、令牌在、`/health` 带令牌回 200 才算在跑。
fn running(root: &Path) -> Result<Option<Connection>> {
    let Ok(bytes) = std::fs::read(root.join("daemon.json")) else {
        return Ok(None);
    };
    let info: Value = serde_json::from_slice(&bytes).context("bad daemon.json")?;
    let Some(port) = info.get("port").and_then(Value::as_u64) else {
        return Ok(None);
    };
    let Ok(token) = std::fs::read_to_string(root.join("api-token")) else {
        return Ok(None);
    };
    let token = token.trim().to_owned();
    let base = format!("http://127.0.0.1:{port}");
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(2)))
        .proxy(None)
        .build()
        .into();
    let healthy = agent
        .get(format!("{base}/health"))
        .header("Authorization", format!("Bearer {token}"))
        .call()
        .is_ok_and(|r| r.status().is_success());
    if !healthy {
        // 上次没正常退出留下的 daemon.json：照不在跑办。
        return Ok(None);
    }
    if port != u64::from(PORT) {
        bail!("a daemon is running on port {port}; the window can only reach port {PORT}");
    }
    Ok(Some(Connection { base, token }))
}

fn spawn(root: &Path, plugins: &Path) -> Result<Child> {
    let exe =
        std::env::current_exe()?.with_file_name(if cfg!(windows) { "usuyo.exe" } else { "usuyo" });
    let log = log_path(root);
    if let Some(dir) = log.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let out = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log)
        .with_context(|| format!("cannot open {}", log.display()))?;
    Command::new(&exe)
        .arg("--plugins")
        .arg(plugins)
        .arg("daemon")
        .arg("--port")
        .arg(PORT.to_string())
        .arg("--first-library")
        .arg(first_library()?)
        .stdin(Stdio::null())
        .stdout(out.try_clone()?)
        .stderr(out)
        .spawn()
        .with_context(|| format!("cannot start {}", exe.display()))
}

fn log_path(root: &Path) -> PathBuf {
    root.join("logs/daemon.log")
}

fn env_path(k: &str) -> Option<PathBuf> {
    std::env::var_os(k).map(PathBuf::from)
}

/// 同 `usuyo_library::Library::default_machine_root`（SPEC §0.1）；壳不链内核的库，这几行抄一份。
fn machine_root() -> Result<PathBuf> {
    if let Some(p) = env_path("USUYO_HOME") {
        return Ok(p);
    }
    let home = || env_path("HOME").context("HOME is not set");
    if cfg!(target_os = "macos") {
        Ok(home()?.join("Library/Application Support/usuyo"))
    } else if cfg!(windows) {
        Ok(env_path("LOCALAPPDATA")
            .context("LOCALAPPDATA is not set")?
            .join("usuyo"))
    } else {
        match env_path("XDG_STATE_HOME") {
            Some(p) => Ok(p.join("usuyo")),
            None => Ok(home()?.join(".local/state/usuyo")),
        }
    }
}

/// 还没有资料库时建在哪：「文稿」下的 `usuyo`，打开 Finder 就看得到。
/// 换了 `USUYO_HOME`（另一套设置，比如测试）时建在它下面的 `library`，不碰真的「文稿」。
fn first_library() -> Result<PathBuf> {
    if let Some(p) = env_path("USUYO_HOME") {
        return Ok(p.join("library"));
    }
    let home = if cfg!(windows) {
        env_path("USERPROFILE").context("USERPROFILE is not set")?
    } else {
        env_path("HOME").context("HOME is not set")?
    };
    Ok(home.join("Documents/usuyo"))
}
