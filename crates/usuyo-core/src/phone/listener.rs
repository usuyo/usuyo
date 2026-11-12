//! 发令牌之前先认人：要带令牌的那一条连接，对面得是你自己（同一个用户）跑的驱动，令牌才发过去。
//! 驱动没在跑时，这台电脑上随便哪个程序都能占住那个端口等令牌；别的用户的程序占住同一个回环地址时，
//! 这个用户的 `lsof` 看不见它。所以认的不是「端口上谁在听」，是「我这一条连接的另一头在谁手里」。
//!
//! 认法：先连上（[`super::serve`]），再用 `lsof -nP -iTCP@<服务地址> -sTCP:ESTABLISHED -Fpun` 找名字是
//! `<服务地址>-><这一条的本机地址>` 的那个套接字在哪个进程手里；对面接下连接以前看不到它，所以短短地等一会儿。
//! 要正好一个进程，用户是这个守护进程的用户（`id -u`），而且这个进程是 launchd 里登记的「mirroir serve」
//! （`launchctl print gui/<uid>/dev.mirroir.serve` 报的 `pid`），或者它的程序就是按机器目录里装着的那一份
//! `mirroir-mcp`。看不见的（别的用户的）算不是。每一步都有时限；哪一步认不出来都算不是。

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::probe::run;

/// 对面接下连接要多久才在 `lsof` 里露出来：最多等这么久，隔一会儿看一次。
const ACCEPT_WAIT: Duration = Duration::from_secs(2);
const ACCEPT_EVERY: Duration = Duration::from_millis(50);

/// 认人要用的几个程序，和装着的驱动在哪。
pub(crate) struct Check {
    pub lsof: PathBuf,
    pub launchctl: PathBuf,
    /// 这个守护进程的用户（[`me`]）。
    pub uid: u32,
    pub label: &'static str,
    pub binary: PathBuf,
    /// 每个程序最多跑多久。
    pub limit: Duration,
}

/// 从 `ours` 连到 `service` 的这一条连接，另一头在驱动手里：是就回 `Ok(())`，不是或认不出时回一句英文原因
/// （进日志，不带令牌）。
pub(crate) async fn peer_is_driver(
    c: &Check,
    service: SocketAddr,
    ours: SocketAddr,
) -> Result<(), String> {
    let uid = c.uid;
    let select = format!("-iTCP@{service}");
    let name = format!("{service}->{ours}");
    let deadline = tokio::time::Instant::now() + ACCEPT_WAIT;
    let peers = loop {
        let out = run(
            &c.lsof,
            &["-nP", &select, "-sTCP:ESTABLISHED", "-Fpun"],
            c.limit,
        )
        .await
        .map_err(|e| e.to_string())?;
        let peers = holders(&String::from_utf8_lossy(&out.stdout), &name);
        if !peers.is_empty() || tokio::time::Instant::now() >= deadline {
            break peers;
        }
        tokio::time::sleep(ACCEPT_EVERY).await;
    };
    let (pid, owner) = match peers.as_slice() {
        [] => {
            return Err(format!(
                "no process this user can see holds the other end of {ours} -> {service}"
            ));
        }
        [one] => *one,
        _ => return Err(format!("more than one process holds {name}")),
    };
    if owner != uid {
        return Err(format!("{service} is held by user {owner}, not {uid}"));
    }
    if launchd_pid(c, uid).await == Some(pid) {
        return Ok(());
    }
    let pid_arg = pid.to_string();
    let out = run(
        &c.lsof,
        &["-nP", "-a", "-p", &pid_arg, "-d", "txt", "-Fn"],
        c.limit,
    )
    .await
    .map_err(|e| e.to_string())?;
    let program = executable(&String::from_utf8_lossy(&out.stdout))
        .ok_or_else(|| format!("no program for process {pid}"))?;
    if same_file(Path::new(&program), &c.binary) {
        Ok(())
    } else {
        Err(format!("{service} is held by {program}, not the driver"))
    }
}

/// 这个守护进程的用户：`id -u`。
pub(crate) async fn me(id: &Path, limit: Duration) -> Result<u32, String> {
    let out = run(id, &["-u"], limit).await.map_err(|e| e.to_string())?;
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .map_err(|_| "`id -u` did not print a number".to_owned())
}

/// launchd 里登记的那一项现在的 `pid`；没登记、没在跑、读不出来都是 `None`。
async fn launchd_pid(c: &Check, uid: u32) -> Option<u32> {
    let target = format!("gui/{uid}/{}", c.label);
    let out = run(&c.launchctl, &["print", &target], c.limit).await.ok()?;
    if !out.status.success() {
        return None;
    }
    top_level(&String::from_utf8_lossy(&out.stdout), "pid")?
        .parse()
        .ok()
}

/// `lsof -F pun` 里拿着名字是 `name` 的套接字的每个进程：`p<pid>` 后面跟 `u<uid>`，再跟它的
/// `f<fd>`、`n<名字>`。同一个进程拿着几个同名的只算一次。
pub(crate) fn holders(out: &str, name: &str) -> Vec<(u32, u32)> {
    let mut found: Vec<(u32, u32)> = Vec::new();
    let (mut pid, mut uid) = (None, None);
    for line in out.lines() {
        if let Some(p) = line.strip_prefix('p') {
            pid = p.parse::<u32>().ok();
            uid = None;
        } else if let Some(u) = line.strip_prefix('u') {
            uid = u.parse::<u32>().ok();
        } else if line.strip_prefix('n') == Some(name)
            && let (Some(p), Some(u)) = (pid, uid)
            && !found.contains(&(p, u))
        {
            found.push((p, u));
        }
    }
    found
}

/// `lsof -d txt -F n` 的第一个 `n` 行：进程的程序本身（后面的是它载入的库）。
pub(crate) fn executable(out: &str) -> Option<String> {
    out.lines()
        .find_map(|l| l.strip_prefix('n'))
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
}

/// `launchctl print` 顶层（只缩进一个制表符）的 `<key> = <value>`。
pub(crate) fn top_level<'a>(out: &'a str, key: &str) -> Option<&'a str> {
    out.lines().find_map(|l| {
        let rest = l.strip_prefix('\t')?;
        if rest.starts_with('\t') {
            return None;
        }
        let (k, v) = rest.split_once(" = ")?;
        (k == key).then(|| v.trim())
    })
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}
