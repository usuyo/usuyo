//! 真的起两次 `usuyo daemon`（临时的按机器目录、空闲端口）：每次启动换一个令牌，
//! 文件只有自己能读，收尾退出时 `daemon.json` 和令牌文件都去掉。
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use usuyo_api::handshake::{self, ProveError};

const BIN: &str = env!("CARGO_BIN_EXE_usuyo");

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(2)))
        .proxy(None)
        .build()
        .into()
}

fn start(home: &Path, port: u16) -> Child {
    Command::new(BIN)
        .env("USUYO_HOME", home)
        .env_remove("USUYO_LIBRARY")
        .env_remove("USUYO_PLUGINS")
        .args(["daemon", "--port", &port.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

/// 等到令牌文件里的令牌握手核得上，回那个令牌。
fn wait_ready(home: &Path, port: u16, child: &mut Child) -> String {
    let started = Instant::now();
    loop {
        assert!(child.try_wait().unwrap().is_none(), "daemon exited early");
        if let Ok(t) = std::fs::read_to_string(home.join("api-token"))
            && home.join("daemon.json").is_file()
            && handshake::prove(&agent(), port, t.trim()).is_ok()
        {
            return t.trim().to_owned();
        }
        assert!(
            started.elapsed() < Duration::from_secs(30),
            "daemon not ready"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn stop(child: &mut Child) {
    let status = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(status.success());
    let started = Instant::now();
    while child.try_wait().unwrap().is_none() {
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "daemon did not stop"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn mode(p: &Path) -> u32 {
    std::fs::metadata(p).unwrap().permissions().mode() & 0o777
}

#[test]
fn each_start_has_its_own_token() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("machine");
    let init = Command::new(BIN)
        .env("USUYO_HOME", &home)
        .args(["init"])
        .arg(tmp.path().join("lib"))
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");
    let port = free_port();

    let mut first = start(&home, port);
    let one = wait_ready(&home, port, &mut first);
    assert_eq!(mode(&home), 0o700);
    assert_eq!(mode(&home.join("api-token")), 0o600);
    assert_eq!(mode(&home.join("daemon.json")), 0o600);
    stop(&mut first);
    assert!(!home.join("daemon.json").exists());
    assert!(!home.join("api-token").exists());

    let mut second = start(&home, port);
    let two = wait_ready(&home, port, &mut second);
    assert_ne!(one, two);
    // 上一次的令牌对这一次的守护进程不算数：握手核不上，带着它请求也进不去。
    assert!(matches!(
        handshake::prove(&agent(), port, &one),
        Err(ProveError::Stranger { .. })
    ));
    let old = agent()
        .get(format!("http://127.0.0.1:{port}/health"))
        .header("Authorization", format!("Bearer {one}"))
        .call();
    assert!(matches!(old, Err(ureq::Error::StatusCode(401))), "{old:?}");
    stop(&mut second);
}
