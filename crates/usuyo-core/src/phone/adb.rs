//! 安卓那条路：找 `adb`（`PATH`，再 Homebrew 和 Android SDK 的位置），读它的版本和
//! `adb devices -l`。序列号长的只留头尾四位。
//!
//! 第一次 `adb devices` 会让 adb 起它自己的服务（5037 端口），之后留着；宿主自己管的 adb 服务
//! （协议 §7）还没做。

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::probe::run;
use crate::view::{AdbFound, AndroidDevice, DeviceState, PhoneAndroid};

const VERSION_TIMEOUT: Duration = Duration::from_secs(5);
/// 第一次要起 adb 的服务，慢一些。
const DEVICES_TIMEOUT: Duration = Duration::from_secs(10);

/// 第一个能执行的 `adb`。
pub(crate) fn find(path_var: Option<&OsStr>, dirs: &[PathBuf]) -> Option<PathBuf> {
    let from_path = path_var
        .map(|p| std::env::split_paths(p).collect::<Vec<_>>())
        .unwrap_or_default();
    from_path
        .iter()
        .chain(dirs)
        .map(|d| d.join("adb"))
        .find(|p| std::fs::metadata(p).is_ok_and(|m| m.is_file() && executable(&m)))
}

#[cfg(unix)]
fn executable(m: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    m.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn executable(_: &std::fs::Metadata) -> bool {
    true
}

/// `adb version`：`Version 36.0.0-13206524` 那一行的版本号；没有那一行时用第一行最后一段。
pub(crate) fn version(out: &str) -> Option<String> {
    out.lines()
        .find_map(|l| l.trim().strip_prefix("Version "))
        .map(|v| v.split('-').next().unwrap_or(v).trim().to_owned())
        .or_else(|| {
            out.lines()
                .next()
                .and_then(|l| l.split_whitespace().last())
                .map(str::to_owned)
        })
        .filter(|v| !v.is_empty())
}

/// 长的序列号只留头尾四位，和页面上的写法一样。
pub(crate) fn short(serial: &str) -> String {
    let chars: Vec<char> = serial.chars().collect();
    if chars.len() > 10 {
        let head: String = chars[..4].iter().collect();
        let tail: String = chars[chars.len() - 4..].iter().collect();
        format!("{head}…{tail}")
    } else {
        serial.to_owned()
    }
}

/// `adb devices -l` 的每一行：`<序列号> <状态> [product:… model:Pixel_8 device:… transport_id:…]`。
/// 三种状态以外的（`recovery`、`sideload`、`no permissions`…）不列。
pub(crate) fn devices(out: &str) -> Vec<AndroidDevice> {
    out.lines()
        .filter(|l| !l.starts_with("List of devices") && !l.starts_with('*'))
        .filter_map(|l| {
            let mut fields = l.split_whitespace();
            let serial = fields.next()?;
            let state = match fields.next()? {
                "device" => DeviceState::Device,
                "unauthorized" => DeviceState::Unauthorized,
                "offline" => DeviceState::Offline,
                _ => return None,
            };
            let model = fields
                .find_map(|f| f.strip_prefix("model:"))
                .map(|m| m.replace('_', " "));
            Some(AndroidDevice {
                serial: short(serial),
                model,
                state,
            })
        })
        .collect()
}

pub(crate) async fn android(
    path_var: Option<&OsStr>,
    dirs: &[PathBuf],
) -> crate::Result<PhoneAndroid> {
    let Some(adb) = find(path_var, dirs) else {
        return Ok(PhoneAndroid {
            adb: None,
            devices: Vec::new(),
        });
    };
    let v = run(&adb, &["version"], VERSION_TIMEOUT).await?;
    let version = version(&String::from_utf8_lossy(&v.stdout)).unwrap_or_default();
    let d = run(&adb, &["devices", "-l"], DEVICES_TIMEOUT).await?;
    if !d.status.success() {
        return Err(failed(&adb, d.status.code()));
    }
    Ok(PhoneAndroid {
        adb: Some(AdbFound {
            path: adb.display().to_string(),
            version,
        }),
        devices: devices(&String::from_utf8_lossy(&d.stdout)),
    })
}

fn failed(adb: &Path, code: Option<i32>) -> crate::Error {
    super::PhoneError::Program {
        program: adb.display().to_string(),
        reason: format!("`devices -l` exit {code:?}"),
    }
    .into()
}
