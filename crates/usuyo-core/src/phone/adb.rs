//! 安卓那条路：找 `adb`（`PATH`，再 Homebrew 和 Android SDK 的位置），读它的版本；adb 的服务已经在跑时
//! 经它的口子读设备。序列号长的只留头尾四位。
//!
//! `adb devices` 会让 adb 起它自己的服务（5037 端口），之后留在后台，所以只有你点「找一下连着的手机」
//! 时才跑它（[`look`]）；看这一页不起任何程序。宿主自己管的 adb 服务（协议 §7）还没做。

use std::ffi::OsStr;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::probe::run;
use crate::view::{AdbFound, AndroidDevice, DeviceState, DriverStep, PhoneAndroid};

const VERSION_TIMEOUT: Duration = Duration::from_secs(5);
/// 第一次要起 adb 的服务，慢一些。
const DEVICES_TIMEOUT: Duration = Duration::from_secs(10);
/// 连 adb 的服务、写请求、读回答，每一步最多这么久。
const SERVER_TIMEOUT: Duration = Duration::from_secs(3);
/// 设备表最多多大。
const LIST_CAP: usize = 64 << 10;

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

/// `GET /phone` 的安卓那一半：找 `adb`、读它的版本（`adb version` 不连服务）；它的服务已经在 `server` 上
/// 跑着时，经那个口子问一次设备（`host:devices-l`）。不跑 `adb devices`，所以看这一页不会起 adb 的服务。
pub(crate) async fn android(
    path_var: Option<&OsStr>,
    dirs: &[PathBuf],
    server: SocketAddr,
) -> crate::Result<PhoneAndroid> {
    let Some(adb) = find(path_var, dirs) else {
        return Ok(PhoneAndroid {
            adb: None,
            devices: Vec::new(),
            step: None,
        });
    };
    let version = read_version(&adb).await?;
    let listed = match tokio::task::spawn_blocking(move || ask_server(server)).await {
        Ok(Ok(listed)) => listed,
        Ok(Err(e)) => {
            tracing::warn!(%server, error = %e, "the adb server port did not answer as adb");
            None
        }
        Err(e) => return Err(crate::Error::Task(e.to_string())),
    };
    Ok(found(&adb, version, listed))
}

/// 「找一下连着的手机」：跑一次 `adb devices -l`。adb 的服务没在跑时它会起来，之后留在后台。
pub(crate) async fn look(
    path_var: Option<&OsStr>,
    dirs: &[PathBuf],
) -> crate::Result<PhoneAndroid> {
    let adb = find(path_var, dirs).ok_or(super::PhoneError::NoAdb)?;
    let version = read_version(&adb).await?;
    let d = run(&adb, &["devices", "-l"], DEVICES_TIMEOUT).await?;
    if !d.status.success() {
        return Err(failed(&adb, d.status.code()));
    }
    Ok(found(
        &adb,
        version,
        Some(devices(&String::from_utf8_lossy(&d.stdout))),
    ))
}

async fn read_version(adb: &Path) -> crate::Result<String> {
    let v = run(adb, &["version"], VERSION_TIMEOUT).await?;
    Ok(version(&String::from_utf8_lossy(&v.stdout)).unwrap_or_default())
}

/// `listed` 是在跑的服务报的设备；`None` 是服务没在跑。
fn found(adb: &Path, version: String, listed: Option<Vec<AndroidDevice>>) -> PhoneAndroid {
    let running = listed.is_some();
    PhoneAndroid {
        adb: Some(AdbFound {
            path: adb.display().to_string(),
            version,
            running,
        }),
        devices: listed.unwrap_or_default(),
        step: running.then(|| stop_step(adb)),
    }
}

/// adb 的服务在后台跑着：怎么停掉它。
pub(crate) fn stop_step(adb: &Path) -> DriverStep {
    DriverStep {
        text:
            "adb 的服务在这台电脑的后台跑着，会一直留着，usuyo 不去停它。要停掉它，在「终端」里跑："
                .into(),
        commands: vec![format!("{} kill-server", super::driver::quoted(adb))],
    }
}

/// 经 adb 服务的口子问一次 `host:devices-l`：请求是四位十六进制的长度加服务名，回答是 `OKAY`、
/// 四位十六进制的长度、设备表，之后服务关掉连接。连不上（没在跑）是 `None`。连、写、读各自限在 `SERVER_TIMEOUT` 里。
fn ask_server(server: SocketAddr) -> std::io::Result<Option<Vec<AndroidDevice>>> {
    use std::io::{Read, Write};
    let mut stream = match std::net::TcpStream::connect_timeout(&server, SERVER_TIMEOUT) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => return Ok(None),
        Err(e) => return Err(e),
    };
    stream.set_read_timeout(Some(SERVER_TIMEOUT))?;
    stream.set_write_timeout(Some(SERVER_TIMEOUT))?;
    let service = "host:devices-l";
    stream.write_all(format!("{:04x}{service}", service.len()).as_bytes())?;
    let mut head = [0u8; 8];
    stream.read_exact(&mut head)?;
    let bad = |what: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, what.to_owned());
    if &head[..4] != b"OKAY" {
        return Err(bad("not OKAY"));
    }
    let len = std::str::from_utf8(&head[4..])
        .ok()
        .and_then(|h| usize::from_str_radix(h, 16).ok())
        .filter(|&n| n <= LIST_CAP)
        .ok_or_else(|| bad("length is not hex or too large"))?;
    let mut list = vec![0u8; len];
    stream.read_exact(&mut list)?;
    Ok(Some(devices(&String::from_utf8_lossy(&list))))
}

fn failed(adb: &Path, code: Option<i32>) -> crate::Error {
    super::PhoneError::Program {
        program: adb.display().to_string(),
        reason: format!("`devices -l` exit {code:?}"),
    }
    .into()
}
