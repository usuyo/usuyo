//! 内核守护进程：找已经在跑的（`daemon.json` 加握手），没有就把打包带的 `usuyo daemon` 起起来。
//! 按机器的目录和命令行一样：`USUYO_HOME`，没有就是系统的位置。
//!
//! 令牌只交给握手核得上的那个（[`usuyo_api::handshake`]）：端口上是别的程序时，不把它当内核，
//! 也不在占着的端口上再起一个，直接告诉用户。内核重启过（令牌换了），前端再要时重新找、重新核对。

use std::fs::{File, OpenOptions};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;

/// 窗口的 CSP 只放行这个端口（tauri.conf.json），和内核的默认端口一致。
pub const PORT: u16 = 47615;
const START_TIMEOUT: Duration = Duration::from_secs(20);
const STOP_TIMEOUT: Duration = Duration::from_secs(5);
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Serialize)]
pub struct Connection {
    pub base: String,
    pub token: String,
    #[serde(skip)]
    port: u16,
}

/// 给用户看的一句话；别的错只进日志。
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct Plain(String);

pub struct Daemon {
    /// 按机器的目录。
    root: PathBuf,
    plugins: PathBuf,
    port: u16,
    /// 第一次找的结果；之后每次要连接时更新。
    ready: Mutex<Option<Result<Connection, String>>>,
    changed: Condvar,
    /// 同一时间只有一个在找或起内核。
    finding: Mutex<()>,
    /// 壳自己起的那个；退出时停掉。
    owned: Mutex<Option<Child>>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Daemon {
    pub fn new(plugins: PathBuf) -> Result<Daemon> {
        Ok(Daemon::at(machine_root()?, plugins, port()))
    }

    /// 按机器的目录：登录窗口在 Windows、Linux 上的网页数据也放这下面。
    #[cfg(not(target_vendor = "apple"))]
    pub fn machine_dir(&self) -> &Path {
        &self.root
    }

    fn at(root: PathBuf, plugins: PathBuf, port: u16) -> Daemon {
        Daemon {
            root,
            plugins,
            port,
            ready: Mutex::new(None),
            changed: Condvar::new(),
            finding: Mutex::new(()),
            owned: Mutex::new(None),
        }
    }

    /// 打开窗口时找一次。
    pub fn start(&self) {
        let result = {
            let _turn = lock(&self.finding);
            self.connect_plain()
        };
        *lock(&self.ready) = Some(result);
        self.changed.notify_all();
    }

    /// 前端要连接：等第一次找完；手上那个还核得上就用它，核不上（内核重启过、退了）就重新找。
    pub fn connection(&self) -> Result<Connection, String> {
        self.wait_first()?;
        let _turn = lock(&self.finding);
        let current = lock(&self.ready).clone();
        if let Some(Ok(c)) = current
            && usuyo_api::handshake::prove(&agent(), c.port, &c.token).is_ok()
        {
            return Ok(c);
        }
        let result = self.connect_plain();
        *lock(&self.ready) = Some(result.clone());
        result
    }

    fn wait_first(&self) -> Result<(), String> {
        let deadline = Instant::now() + START_TIMEOUT + Duration::from_secs(5);
        let mut ready = lock(&self.ready);
        while ready.is_none() {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err("usuyo 的后台没有按时启动".into());
            }
            ready = self
                .changed
                .wait_timeout(ready, left)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
        Ok(())
    }

    /// 细节进日志，给前端一句话。
    fn connect_plain(&self) -> Result<Connection, String> {
        self.connect().map_err(|e| {
            eprintln!("usuyo: cannot reach the daemon: {e:#}");
            match e.downcast_ref::<Plain>() {
                Some(p) => p.0.clone(),
                None => "没能连上 usuyo 的后台".into(),
            }
        })
    }

    fn connect(&self) -> Result<Connection> {
        let root = &self.root;
        let mut owned = lock(&self.owned);
        // 自己起的那个还活着就接着等它；不在了（或从没起过）先看端口上是谁。
        let alive = match owned.as_mut() {
            Some(c) => c.try_wait()?.is_none(),
            None => false,
        };
        let mut child = match owned.take() {
            Some(c) if alive => c,
            _ => match find(root, self.port)? {
                Found::Daemon(c) => return Ok(c),
                Found::OtherPort(p) => {
                    return Err(Plain(format!(
                        "已经有一个 usuyo 的后台在端口 {p} 上，窗口只能连端口 {}",
                        self.port
                    ))
                    .into());
                }
                Found::Stranger => return Err(port_taken(self.port).into()),
                Found::Nothing => spawn(root, &self.plugins, self.port)?,
            },
        };
        let started = Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                eprintln!("usuyo: the daemon exited ({status})");
                return Err(Plain(format!(
                    "usuyo 的后台没能启动，详情在 {}",
                    log_path(root).display()
                ))
                .into());
            }
            // 自己的还没写好令牌时，握手核不上是正常的，接着等。
            if let Found::Daemon(c) = find(root, self.port)? {
                *owned = Some(child);
                return Ok(c);
            }
            if started.elapsed() > START_TIMEOUT {
                stop_child(child);
                return Err(Plain("usuyo 的后台没有按时启动".into()).into());
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

/// 调试版可以用 `USUYO_PORT` 换端口（配一个另外的 `USUYO_HOME` 试东西）；
/// 开发时窗口经 Vite 的代理连内核，不受 CSP 的端口限制。发布版只用 47615。
fn port() -> u16 {
    if cfg!(debug_assertions)
        && let Some(p) = std::env::var("USUYO_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
    {
        return p;
    }
    PORT
}

fn port_taken(port: u16) -> Plain {
    Plain(format!("端口 {port} 被别的程序占着，usuyo 的后台没能启动"))
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

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(PROBE_TIMEOUT))
        .http_status_as_error(false)
        .proxy(None)
        .build()
        .into()
}

#[derive(Debug)]
enum Found {
    /// 握手核得上，在窗口能连的端口上。
    Daemon(Connection),
    /// 握手核得上，但在别的端口上。
    OtherPort(u16),
    /// 端口上有程序在听，拿不出证明。
    Stranger,
    Nothing,
}

/// 照 `daemon.json` 和令牌文件找，先握手；令牌只在核得上以后才交出去。
fn find(root: &Path, port: u16) -> Result<Found> {
    if let Some((p, token)) = registered(root)?
        && usuyo_api::handshake::prove(&agent(), p, &token).is_ok()
    {
        if p != port {
            return Ok(Found::OtherPort(p));
        }
        return Ok(Found::Daemon(Connection {
            base: format!("http://127.0.0.1:{p}"),
            token,
            port: p,
        }));
    }
    // 上次没正常退出留下的 daemon.json，或者根本没有：看端口空不空。
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    if TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok() {
        return Ok(Found::Stranger);
    }
    Ok(Found::Nothing)
}

/// `daemon.json` 里的端口和令牌文件里的令牌；缺一个就是没登记。
fn registered(root: &Path) -> Result<Option<(u16, String)>> {
    let Ok(bytes) = std::fs::read(root.join("daemon.json")) else {
        return Ok(None);
    };
    let info: Value = serde_json::from_slice(&bytes).context("bad daemon.json")?;
    let Some(port) = info
        .get("port")
        .and_then(Value::as_u64)
        .and_then(|p| u16::try_from(p).ok())
    else {
        return Ok(None);
    };
    let Ok(token) = std::fs::read_to_string(root.join("api-token")) else {
        return Ok(None);
    };
    Ok(Some((port, token.trim().to_owned())))
}

fn spawn(root: &Path, plugins: &Path, port: u16) -> Result<Child> {
    let exe =
        std::env::current_exe()?.with_file_name(if cfg!(windows) { "usuyo.exe" } else { "usuyo" });
    let out = open_log(root)?;
    Command::new(&exe)
        .arg("--plugins")
        .arg(plugins)
        .arg("daemon")
        .arg("--port")
        .arg(port.to_string())
        .arg("--first-library")
        .arg(first_library()?)
        .stdin(Stdio::null())
        .stdout(out.try_clone()?)
        .stderr(out)
        .spawn()
        .with_context(|| format!("cannot start {}", exe.display()))
}

/// 按机器的目录和 `logs/` 只给自己（0700），日志文件 0600：日志里有资料库的路径和插件报的错。
fn open_log(root: &Path) -> Result<File> {
    let log = log_path(root);
    let dirs = [root, log.parent().unwrap_or(root)];
    for dir in dirs {
        std::fs::create_dir_all(dir).with_context(|| format!("cannot create {}", dir.display()))?;
        #[cfg(unix)]
        std::fs::set_permissions(dir, std::os::unix::fs::PermissionsExt::from_mode(0o700))
            .with_context(|| format!("cannot restrict {}", dir.display()))?;
    }
    let mut opts = OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
    let file = opts
        .open(&log)
        .with_context(|| format!("cannot open {}", log.display()))?;
    // 早就在的日志文件不受 `mode` 管，再设一次。
    #[cfg(unix)]
    file.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
    Ok(file)
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

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::Arc;

    use super::*;

    /// 本机的假服务：每个请求都答 200，请求体由 `answer` 按收到的请求定；记下收到的全部字节。
    fn fake(answer: impl Fn(&str) -> String + Send + 'static) -> (u16, Arc<Mutex<Vec<u8>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::clone(&seen);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut s) = stream else { return };
                s.set_read_timeout(Some(Duration::from_millis(300)))
                    .unwrap();
                let mut req = Vec::new();
                let mut buf = [0u8; 4096];
                while let Ok(n) = s.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    req.extend_from_slice(&buf[..n]);
                    let text = String::from_utf8_lossy(&req).to_lowercase();
                    if let Some((head, body)) = text.split_once("\r\n\r\n") {
                        let len = head
                            .lines()
                            .find_map(|l| l.strip_prefix("content-length:"))
                            .and_then(|v| v.trim().parse::<usize>().ok())
                            .unwrap_or(0);
                        if body.len() >= len {
                            break;
                        }
                    }
                }
                lock(&log).extend_from_slice(&req);
                let body = answer(&String::from_utf8_lossy(&req));
                let _ = write!(
                    s,
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
            }
        });
        (port, seen)
    }

    /// 占着端口、什么都答 200 的程序。
    fn impostor() -> (u16, Arc<Mutex<Vec<u8>>>) {
        fake(|_| format!(r#"{{"status":"ok","proof":"{}"}}"#, "0".repeat(64)))
    }

    /// 用 `key` 签握手的假内核；`key` 可以中途换（像重启过）。
    fn signing_daemon(key: Arc<Mutex<String>>) -> u16 {
        let port = Arc::new(Mutex::new(0u16));
        let p = Arc::clone(&port);
        let (bound, _) = fake(move |req| {
            let nonce = req
                .split("\"nonce\":\"")
                .nth(1)
                .and_then(|r| r.get(..64))
                .unwrap_or_default()
                .to_owned();
            let proof = usuyo_api::handshake::proof(&lock(&key), *lock(&p), &nonce);
            format!(r#"{{"proof":"{proof}"}}"#)
        });
        *lock(&port) = bound;
        bound
    }

    fn register(root: &Path, port: u16, token: &str) {
        std::fs::write(root.join("api-token"), token).unwrap();
        std::fs::write(
            root.join("daemon.json"),
            format!(r#"{{"port":{port},"pid":1}}"#),
        )
        .unwrap();
    }

    #[test]
    fn taken_port_is_reported_and_nothing_is_started() {
        let root = tempfile::tempdir().unwrap();
        let (port, seen) = impostor();
        register(root.path(), port, "secret-token-for-this-test");
        let d = Daemon::at(root.path().to_owned(), root.path().join("plugins"), port);
        d.start();
        let err = d.connection().unwrap_err();
        assert_eq!(
            err,
            format!("端口 {port} 被别的程序占着，usuyo 的后台没能启动")
        );
        assert!(lock(&d.owned).is_none(), "no second daemon was started");
        assert!(!root.path().join("logs").exists(), "spawn never ran");
        assert!(!String::from_utf8_lossy(&lock(&seen)).contains("secret-token-for-this-test"));
    }

    #[test]
    fn restarted_daemon_hands_out_the_new_token() {
        let root = tempfile::tempdir().unwrap();
        let key = Arc::new(Mutex::new("token-a".to_owned()));
        let port = signing_daemon(Arc::clone(&key));
        register(root.path(), port, "token-a");
        let d = Daemon::at(root.path().to_owned(), root.path().join("plugins"), port);
        d.start();
        assert_eq!(d.connection().unwrap().token, "token-a");
        // 重启：令牌换了，文件也换了。
        *lock(&key) = "token-b".to_owned();
        register(root.path(), port, "token-b");
        assert_eq!(d.connection().unwrap().token, "token-b");
    }

    #[test]
    fn impostor_is_a_stranger_and_never_gets_the_token() {
        let root = tempfile::tempdir().unwrap();
        let (port, seen) = impostor();
        let token = "secret-token-for-this-test";
        register(root.path(), port, token);

        assert!(matches!(find(root.path(), port).unwrap(), Found::Stranger));
        let seen = String::from_utf8_lossy(&lock(&seen)).to_lowercase();
        assert!(seen.contains("post /handshake"), "{seen}");
        assert!(!seen.contains(token), "{seen}");
        assert!(!seen.contains("authorization"), "{seen}");
        assert_eq!(
            port_taken(PORT).to_string(),
            "端口 47615 被别的程序占着，usuyo 的后台没能启动"
        );
    }

    #[test]
    fn free_port_without_daemon_json_is_nothing() {
        let root = tempfile::tempdir().unwrap();
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        assert!(matches!(find(root.path(), port).unwrap(), Found::Nothing));
    }

    #[cfg(unix)]
    #[test]
    fn log_and_machine_dir_are_private() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("machine");
        std::fs::create_dir_all(root.join("logs")).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(log_path(&root), "old\n").unwrap();
        std::fs::set_permissions(log_path(&root), std::fs::Permissions::from_mode(0o644)).unwrap();
        drop(open_log(&root).unwrap());
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&root), 0o700);
        assert_eq!(mode(&root.join("logs")), 0o700);
        assert_eq!(mode(&log_path(&root)), 0o600);
    }
}
