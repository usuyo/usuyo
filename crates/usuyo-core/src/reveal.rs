//! 在系统的文件管理器里显示一个文件：macOS 的访达选中它，Windows 的资源管理器选中它，Linux 打开它所在的文件夹。

use std::path::Path;
use std::time::Duration;

use tokio::process::Command;

use crate::{Error, Result};

/// 等文件管理器回话最多这么久。
const LIMIT: Duration = Duration::from_secs(10);

fn command(path: &Path) -> Command {
    if cfg!(target_os = "macos") {
        let mut c = Command::new("/usr/bin/open");
        c.arg("-R").arg(path);
        c
    } else if cfg!(windows) {
        let mut select = std::ffi::OsString::from("/select,");
        select.push(path);
        let mut c = Command::new("explorer.exe");
        c.arg(select);
        c
    } else {
        let mut c = Command::new("xdg-open");
        c.arg(path.parent().unwrap_or(path));
        c
    }
}

pub(crate) async fn reveal(path: &Path) -> Result<()> {
    let mut c = command(path);
    c.kill_on_drop(true);
    let status = tokio::time::timeout(LIMIT, c.status())
        .await
        .map_err(|_| {
            Error::Task(format!("revealing {} timed out", path.display()))
                .hinted(crate::words::REVEAL_NO_ANSWER.said())
        })?
        .map_err(|e| Error::io(path, e))?;
    // 资源管理器选中了文件也回 1，它的退出码说明不了什么。
    if status.success() || cfg!(windows) {
        Ok(())
    } else {
        Err(Error::Task(format!(
            "revealing {} exited with {:?}",
            path.display(),
            status.code()
        ))
        .hinted(crate::words::REVEAL_FAILED.said()))
    }
}
