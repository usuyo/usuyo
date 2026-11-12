//! 这台 Mac 上几个只读的小调用：macOS 的版本、一个 App 开没开、剪贴板的 `changeCount`，
//! 和打开系统设置的一栏。每次都有时限，到点杀掉。

use std::path::Path;
use std::process::{Output, Stdio};
use std::time::Duration;

use super::PhoneError;

const QUICK: Duration = Duration::from_secs(5);
const OPEN: Duration = Duration::from_secs(10);

/// 跑一个程序到它退出，最多 `timeout`；stdin 是空的。
pub(crate) async fn run(
    program: &Path,
    args: &[&str],
    timeout: Duration,
) -> Result<Output, PhoneError> {
    let fail = |reason: String| PhoneError::Program {
        program: program.display().to_string(),
        reason,
    };
    let child = tokio::process::Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| fail(e.to_string()))?;
    tokio::time::timeout(timeout, child.wait_with_output())
        .await
        .map_err(|_| fail(format!("no answer in {timeout:?}")))?
        .map_err(|e| fail(e.to_string()))
}

/// `sw_vers -productVersion`，例如 `15.6.1`。
pub(crate) async fn macos(sw_vers: &Path) -> crate::Result<String> {
    let out = run(sw_vers, &["-productVersion"], QUICK).await?;
    let version = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if !out.status.success() || version.is_empty() {
        return Err(PhoneError::Program {
            program: sw_vers.display().to_string(),
            reason: format!("exit {:?}, no version", out.status.code()),
        }
        .into());
    }
    Ok(version)
}

/// 叫 `name` 的进程在不在（`pgrep -x`：0 是在，1 是不在）。
pub(crate) async fn running(pgrep: &Path, name: &str) -> crate::Result<bool> {
    let out = run(pgrep, &["-x", name], QUICK).await?;
    match out.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        other => Err(PhoneError::Program {
            program: pgrep.display().to_string(),
            reason: format!("exit {other:?}"),
        }
        .into()),
    }
}

/// Mac 剪贴板的 `changeCount`：`argv` 打出一个整数。
pub(crate) async fn change_count(argv: &[String]) -> Result<i64, PhoneError> {
    let (program, args) = argv
        .split_first()
        .ok_or_else(|| PhoneError::Pasteboard("no program".into()))?;
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = run(Path::new(program), &args, QUICK)
        .await
        .map_err(|e| PhoneError::Pasteboard(e.to_string()))?;
    if !out.status.success() {
        return Err(PhoneError::Pasteboard(format!(
            "exit {:?}",
            out.status.code()
        )));
    }
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<i64>()
        .map_err(|_| PhoneError::Pasteboard("the answer is not a number".into()))
}

/// `open <target>`：系统设置的一栏（`x-apple.systempreferences:`）或一个 App 的路径。
pub(crate) async fn open(open: &Path, target: &str) -> Result<(), PhoneError> {
    let out = run(open, &[target], OPEN)
        .await
        .map_err(|e| PhoneError::Open(e.to_string()))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(PhoneError::Open(format!(
            "{target}: exit {:?}",
            out.status.code()
        )))
    }
}
