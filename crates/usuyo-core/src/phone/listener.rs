//! 发令牌之前先认人：「mirroir serve」的端口上听着的，要是你自己（同一个用户）跑的驱动，令牌才发过去。
//! 驱动没在跑时，这台电脑上随便哪个程序都能占住那个端口等令牌。
//!
//! 认法：`lsof` 找出在那个端口上听的进程和它的用户；用户要是这个守护进程的用户（`id -u`），而且
//! 这个进程是 launchd 里登记的「mirroir serve」（`launchctl print gui/<uid>/dev.mirroir.serve` 报的
//! `pid`），或者它的程序就是按机器目录里装着的那一份 `mirroir-mcp`。每一步都有时限；哪一步认不出来都算不是。

use std::path::Path;
use std::time::Duration;

use super::probe::run;

const QUICK: Duration = Duration::from_secs(5);

/// 认人要用的几个程序，和装着的驱动在哪。
pub(crate) struct Check<'a> {
    pub lsof: &'a Path,
    pub launchctl: &'a Path,
    /// 这个守护进程的用户（[`me`]）。
    pub uid: u32,
    pub label: &'a str,
    pub binary: &'a Path,
}

/// 在 `port` 上听的是驱动：是就回 `Ok(())`，不是或认不出时回一句英文原因（进日志，不带令牌）。
pub(crate) async fn is_driver(c: &Check<'_>, port: u16) -> Result<(), String> {
    let uid = c.uid;
    let listen = format!("-iTCP:{port}");
    let out = run(c.lsof, &["-nP", &listen, "-sTCP:LISTEN", "-Fpu"], QUICK)
        .await
        .map_err(|e| e.to_string())?;
    let owners = owners(&String::from_utf8_lossy(&out.stdout));
    let (pid, owner) = match owners.as_slice() {
        [] => return Err(format!("nothing listens on port {port}")),
        [one] => *one,
        _ => return Err(format!("more than one process listens on port {port}")),
    };
    if owner != uid {
        return Err(format!("port {port} is held by user {owner}, not {uid}"));
    }
    if launchd_pid(c, uid).await == Some(pid) {
        return Ok(());
    }
    let pid_arg = pid.to_string();
    let out = run(
        c.lsof,
        &["-nP", "-a", "-p", &pid_arg, "-d", "txt", "-Fn"],
        QUICK,
    )
    .await
    .map_err(|e| e.to_string())?;
    let program = executable(&String::from_utf8_lossy(&out.stdout))
        .ok_or_else(|| format!("no program for process {pid}"))?;
    if same_file(Path::new(&program), c.binary) {
        Ok(())
    } else {
        Err(format!("port {port} is held by {program}, not the driver"))
    }
}

/// 这个守护进程的用户：`id -u`。
pub(crate) async fn me(id: &Path) -> Result<u32, String> {
    let out = run(id, &["-u"], QUICK).await.map_err(|e| e.to_string())?;
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .map_err(|_| "`id -u` did not print a number".to_owned())
}

/// launchd 里登记的那一项现在的 `pid`；没登记、没在跑、读不出来都是 `None`。
async fn launchd_pid(c: &Check<'_>, uid: u32) -> Option<u32> {
    let target = format!("gui/{uid}/{}", c.label);
    let out = run(c.launchctl, &["print", &target], QUICK).await.ok()?;
    if !out.status.success() {
        return None;
    }
    top_level(&String::from_utf8_lossy(&out.stdout), "pid")?
        .parse()
        .ok()
}

/// `lsof -F pu` 的每个进程：`p<pid>` 后面跟 `u<uid>`（`f…` 行跳过）。同一个进程听 IPv4 和 IPv6 时只算一次。
pub(crate) fn owners(out: &str) -> Vec<(u32, u32)> {
    let mut found: Vec<(u32, u32)> = Vec::new();
    let mut pid = None;
    for line in out.lines() {
        if let Some(p) = line.strip_prefix('p') {
            pid = p.parse::<u32>().ok();
        } else if let (Some(u), Some(p)) = (line.strip_prefix('u'), pid)
            && let Ok(u) = u.parse::<u32>()
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
