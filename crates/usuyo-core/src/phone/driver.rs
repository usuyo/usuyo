//! 驱动 mirroir 的安装：照 `data/phone-driver.json` 钉住的那一版，从它的官方 release 取这台机器架构的包，
//! 按钉住的 sha256 核对（那个数抄自同一个 release 的 `SHA256SUMS`），解开放进按机器目录的
//! `drivers/mirroir-mcp/`：一个可执行的 `mirroir-mcp` 加一个 `driver.json`。不碰系统目录，不要更高的权限。
//!
//! 下载、核对、解开有一步不成就什么都不留下：下了一半的包删掉，解开的暂存文件夹删掉，原来那份不动。
//!
//! 让它常驻成「mirroir serve」（`mirroir-mcp install-agent`，再 `launchctl bootstrap`）不在这里做：
//! `install-agent` 会在你的登录钥匙串里放一张自签名的代码签名证书（第一次签名时钥匙串要你点「始终允许」），
//! 再在 `~/Library/LaunchAgents/` 写一个登录时启动的后台项。这两样改的是这台 Mac 的设置，守护进程不自己动，
//! 写成 [`DriverStep`] 由页面交给你。

use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::models::{ModelError, Net, hash_file, redact};
use crate::view::DriverStep;

/// 钉住的版本、包名和 sha256；只有这一份。
const PIN: &str = include_str!("../../data/phone-driver.json");
/// 解开的程序最多多大。
const BINARY_CAP: u64 = 256 << 20;
/// 包里最多几项（目录、元数据算在内）。
const MAX_ENTRIES: usize = 64;
const RECORD: &str = "driver.json";
/// 「mirroir serve」的 launchd 名字（mirroir-mcp `InstallAgentCommand.label`）。
pub(crate) const AGENT: &str = "dev.mirroir.serve";

/// 钉住的那一版。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Release {
    pub name: String,
    pub version: String,
    pub license: String,
    /// 官方 release 的页面。
    pub page: String,
    /// 包里那个程序的名字，也是放进驱动目录后的名字。
    pub binary: String,
    pub assets: Vec<Asset>,
}

/// 一个架构的包。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Asset {
    /// `std::env::consts::ARCH` 的写法，例如 `aarch64`。
    pub arch: String,
    pub name: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
}

impl Release {
    /// 仓库里钉住的那一版。
    pub fn pinned() -> Result<Release, serde_json::Error> {
        serde_json::from_str(PIN)
    }
}

/// `driver.json`：装着的是哪一版、哪个包、它的 sha256。
#[derive(Debug, Serialize, Deserialize)]
struct Record {
    version: String,
    asset: String,
    sha256: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum InstallError {
    #[error(transparent)]
    Download(#[from] ModelError),
    #[error("sha256 is {got}, pinned {expected}")]
    Sha256 { expected: String, got: String },
    #[error("archive: {0}")]
    Archive(String),
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("background task failed: {0}")]
    Task(String),
}

/// 写在「没装上：」后面的一句。
pub(crate) fn hint(e: &InstallError) -> &'static str {
    match e {
        InstallError::Download(ModelError::Status(404)) => {
            "官方 release 上没有这一版的包，什么都没放进这台电脑。"
        }
        InstallError::Download(ModelError::Refused(_)) => {
            "连不到 GitHub（看看设置里的代理），什么都没放进这台电脑。"
        }
        InstallError::Download(
            ModelError::TooLarge(_) | ModelError::Short { .. } | ModelError::Sha256 { .. },
        )
        | InstallError::Sha256 { .. } => {
            "下载下来的包和钉住的 SHA256 对不上，已经丢掉了，没有放进这台电脑。"
        }
        InstallError::Download(ModelError::Io { .. }) | InstallError::Io { .. } => {
            "存不进这台电脑的驱动目录，什么都没留下。"
        }
        InstallError::Archive(_) => {
            "下载下来的包里不是要的那个程序，已经丢掉了，没有放进这台电脑。"
        }
        InstallError::Download(_) | InstallError::Task(_) => {
            "网络不通，或者下载断了，什么都没放进这台电脑。"
        }
    }
}

fn io(path: &Path) -> impl FnOnce(std::io::Error) -> InstallError + '_ {
    move |source| InstallError::Io {
        path: path.to_owned(),
        source,
    }
}

/// 装着的版本：`driver.json` 读得出、程序在，才算装着。
pub(crate) fn installed(dir: &Path, binary: &str) -> Option<String> {
    let path = dir.join(RECORD);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "driver.json not readable");
            return None;
        }
    };
    let record: Record = match serde_json::from_slice(&bytes) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "driver.json is not a driver record");
            return None;
        }
    };
    dir.join(binary).is_file().then_some(record.version)
}

/// 下载、核对、解开，换成这一份。
pub(crate) async fn install(
    net: &Net,
    dir: &Path,
    release: &Release,
    asset: &Asset,
) -> Result<(), InstallError> {
    let parent = dir
        .parent()
        .ok_or_else(|| InstallError::Archive("driver folder has no parent".into()))?
        .to_owned();
    tokio::fs::create_dir_all(&parent)
        .await
        .map_err(io(&parent))?;
    let part = parent.join(format!(".{}.part", asset.name));
    let result = fetch(net, &part, dir, release, asset).await;
    match tokio::fs::remove_file(&part).await {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => tracing::warn!(path = %part.display(), error = %e, "driver download not removed"),
    }
    result
}

async fn fetch(
    net: &Net,
    part: &Path,
    dir: &Path,
    release: &Release,
    asset: &Asset,
) -> Result<(), InstallError> {
    let url = Url::parse(&asset.url).map_err(|_| ModelError::Url(asset.url.clone()))?;
    // 上次断在一半的不接着下：每次从头来。
    match tokio::fs::remove_file(part).await {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(io(part)(e)),
    }
    tracing::info!(from = %redact(&url), "downloading the phone driver");
    net.download(&url, part, asset.size).await?;
    let p = part.to_owned();
    let got = tokio::task::spawn_blocking(move || hash_file(&p))
        .await
        .map_err(|e| InstallError::Task(e.to_string()))??;
    if got != asset.sha256 {
        return Err(InstallError::Sha256 {
            expected: asset.sha256.clone(),
            got,
        });
    }
    let (p, d, r, a) = (
        part.to_owned(),
        dir.to_owned(),
        release.clone(),
        asset.clone(),
    );
    tokio::task::spawn_blocking(move || unpack(&p, &d, &r, &a))
        .await
        .map_err(|e| InstallError::Task(e.to_string()))?
}

/// 解进 `<父目录>/.mirroir-mcp.stage-<UUIDv7>/`，写好 `driver.json`，再整个换成 `dir`。出错时暂存的删掉。
fn unpack(
    archive: &Path,
    dir: &Path,
    release: &Release,
    asset: &Asset,
) -> Result<(), InstallError> {
    let parent = dir.parent().unwrap_or(dir);
    let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("driver");
    let stage = parent.join(format!(".{name}.stage-{}", usuyo_library::uuid7()));
    std::fs::create_dir(&stage).map_err(io(&stage))?;
    let result = unpack_into(archive, &stage, release, asset).and_then(|()| {
        if std::fs::symlink_metadata(dir).is_ok() {
            let aside = parent.join(format!(".{name}.old-{}", usuyo_library::uuid7()));
            std::fs::rename(dir, &aside).map_err(io(dir))?;
            std::fs::remove_dir_all(&aside).map_err(io(&aside))?;
        }
        std::fs::rename(&stage, dir).map_err(io(dir))
    });
    if result.is_err()
        && let Err(e) = std::fs::remove_dir_all(&stage)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(path = %stage.display(), error = %e, "driver stage folder not removed");
    }
    result
}

/// tar.gz 里要正好有一个普通文件叫 `release.binary`（可以带 `./`）；AppleDouble 的 `._*` 和 pax 头跳过，
/// 别的一律拒。程序写成 0755。
fn unpack_into(
    archive: &Path,
    stage: &Path,
    release: &Release,
    asset: &Asset,
) -> Result<(), InstallError> {
    use tar::EntryType;
    let file = std::fs::File::open(archive).map_err(io(archive))?;
    let mut ar = tar::Archive::new(flate2::read::GzDecoder::new(std::io::BufReader::new(file)));
    let bad = |e: std::io::Error| InstallError::Archive(format!("tar: {e}"));
    let target = stage.join(&release.binary);
    let mut found = false;
    for (n, entry) in ar.entries().map_err(bad)?.enumerate() {
        if n >= MAX_ENTRIES {
            return Err(InstallError::Archive(format!(
                "more than {MAX_ENTRIES} entries"
            )));
        }
        let entry = entry.map_err(bad)?;
        let path = entry.path().map_err(bad)?.into_owned();
        let parts: Vec<&str> = path
            .components()
            .filter_map(|c| match c {
                Component::CurDir => None,
                Component::Normal(p) => Some(p.to_str().unwrap_or("\u{fffd}")),
                _ => Some("\u{fffd}"),
            })
            .collect();
        let kind = entry.header().entry_type();
        if matches!(kind, EntryType::XGlobalHeader | EntryType::XHeader)
            || parts.last().is_some_and(|p| p.starts_with("._"))
        {
            continue;
        }
        if kind == EntryType::Directory && parts.is_empty() {
            continue;
        }
        let is_binary = parts.as_slice() == [release.binary.as_str()];
        if !is_binary || !matches!(kind, EntryType::Regular | EntryType::Continuous) || found {
            return Err(InstallError::Archive(format!(
                "unexpected entry `{}` ({kind:?})",
                path.display()
            )));
        }
        found = true;
        write_binary(entry, &target)?;
    }
    if !found {
        return Err(InstallError::Archive(format!(
            "no `{}` in the archive",
            release.binary
        )));
    }
    let record = Record {
        version: release.version.clone(),
        asset: asset.name.clone(),
        sha256: asset.sha256.clone(),
    };
    let path = stage.join(RECORD);
    let mut bytes = serde_json::to_vec_pretty(&record)
        .map_err(|e| InstallError::Archive(format!("driver.json: {e}")))?;
    bytes.push(b'\n');
    let mut f = std::fs::File::create(&path).map_err(io(&path))?;
    f.write_all(&bytes).map_err(io(&path))?;
    f.sync_all().map_err(io(&path))
}

fn write_binary(mut reader: impl Read, target: &Path) -> Result<(), InstallError> {
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)
        .map_err(io(target))?;
    let mut total = 0u64;
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| InstallError::Archive(format!("reading the program: {e}")))?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > BINARY_CAP {
            return Err(InstallError::Archive(format!(
                "the program is larger than {BINARY_CAP} bytes"
            )));
        }
        f.write_all(&buf[..n]).map_err(io(target))?;
    }
    f.sync_all().map_err(io(target))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(target, std::fs::Permissions::from_mode(0o755))
            .map_err(io(target))?;
    }
    Ok(())
}

/// 卸掉：整个驱动目录挪开再删。不在也算卸掉了。
pub(crate) fn remove(dir: &Path) -> crate::Result<()> {
    if std::fs::symlink_metadata(dir).is_err() {
        return Ok(());
    }
    let parent = dir.parent().unwrap_or(dir);
    let aside = parent.join(format!(".mirroir-mcp.removing-{}", usuyo_library::uuid7()));
    std::fs::rename(dir, &aside).map_err(|e| crate::Error::io(dir, e))?;
    std::fs::remove_dir_all(&aside).map_err(|e| crate::Error::io(&aside, e))
}

/// 给 shell 的一个参数：单引号括起来（路径里有「Application Support」的空格）。
pub(crate) fn quoted(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', r"'\''"))
}

/// 装好了、「mirroir serve」还没在跑：要你登记它。
pub(crate) fn register_step(binary: &Path) -> DriverStep {
    DriverStep {
        text: "还差一步要你自己在「终端」里做：让 mirroir 以「mirroir serve」常驻。它会在你的登录钥匙串里放一张只给自己签名用的证书（钥匙串问 codesign 能不能用时选「始终允许」），再登记成登录时启动的后台项；这两样改的是这台 Mac 的设置，所以 usuyo 不替你做。".into(),
        commands: vec![
            format!("{} install-agent", quoted(binary)),
            format!("launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/{AGENT}.plist"),
        ],
    }
}

/// 换了新的一版、在跑的「mirroir serve」还是旧的那份：要你重新登记、重启它。
pub(crate) fn renew_step(binary: &Path) -> DriverStep {
    DriverStep {
        text: "在跑的「mirroir serve」还是旧的那一版：在「终端」里重新登记一次，再重启它；授权跟着证书走，不用重给。".into(),
        commands: vec![
            format!("{} install-agent", quoted(binary)),
            format!("launchctl kickstart -k gui/$(id -u)/{AGENT}"),
        ],
    }
}

/// 卸掉了、「mirroir serve」还在跑（它跑的是自己那份副本）：要你停掉它。
pub(crate) fn stop_step() -> DriverStep {
    DriverStep {
        text: "「mirroir serve」还在跑，它用的是自己那份副本：在「终端」里停掉它；之后可以在访达里删掉 ~/Library/LaunchAgents/dev.mirroir.serve.plist 和 ~/.mirroir-mcp/mirroir serve.app，在「钥匙串访问」里删掉证书「mirroir serve local signing」。系统设置里给它的授权由系统留着。".into(),
        commands: vec![format!("launchctl bootout gui/$(id -u)/{AGENT}")],
    }
}
