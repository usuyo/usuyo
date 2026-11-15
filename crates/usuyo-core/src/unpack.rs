//! 解开下载来的压缩包（模型、从来源装的插件）：压缩包不可信，在这里查一次。
//!
//! 只建普通文件和文件夹；链接、设备、管道都拒。每一项的路径只许普通的相对路径（没有根、盘符、`.`、`..`），
//! 留给宿主自己的名字（`model.json`、安装记录）不许占顶层；项数和解开后的总字节数有上限。
//! 文件在 unix 上照压缩包里的模式只留执行位：有执行位的是 0755，别的 0644。

use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub(crate) enum UnpackError {
    #[error("archive: {0}")]
    Archive(String),
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

fn archive(msg: impl Into<String>) -> UnpackError {
    UnpackError::Archive(msg.into())
}

fn io(path: &Path) -> impl FnOnce(std::io::Error) -> UnpackError + '_ {
    move |source| UnpackError::Io {
        path: path.to_owned(),
        source,
    }
}

/// 解开的一个文件：相对路径（`/` 分隔）、大小、sha256。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnpackedFile {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

/// 压缩包的格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Format {
    TarGz,
    TarBz2,
    Zip,
}

/// 上限和留给宿主的名字。
#[derive(Debug, Clone, Copy)]
pub(crate) struct Limits {
    pub entries: usize,
    pub bytes: u64,
    /// 顶层不许出现的名字。
    pub reserved: &'static str,
}

/// 解开的文件写到哪、写了多少。
pub(crate) struct Unpacked {
    root: PathBuf,
    total: u64,
    limits: Limits,
    files: Vec<UnpackedFile>,
    entries: usize,
}

impl Unpacked {
    pub fn new(root: &Path, limits: Limits) -> Unpacked {
        Unpacked {
            root: root.to_owned(),
            total: 0,
            limits,
            files: Vec::new(),
            entries: 0,
        }
    }

    /// 解开的文件，按路径排好。
    pub fn into_files(mut self) -> Vec<UnpackedFile> {
        self.files.sort_by(|a, b| a.path.cmp(&b.path));
        self.files
    }

    fn checked(&mut self, rel: &Path) -> Result<PathBuf, UnpackError> {
        self.entries += 1;
        if self.entries > self.limits.entries {
            return Err(archive(format!(
                "more than {} entries",
                self.limits.entries
            )));
        }
        let mut out = PathBuf::new();
        for c in rel.components() {
            match c {
                Component::Normal(part) if part.to_str().is_some() => out.push(part),
                _ => {
                    return Err(archive(format!(
                        "entry `{}` is not a plain relative path",
                        rel.display()
                    )));
                }
            }
        }
        if out.as_os_str().is_empty() || out == Path::new(self.limits.reserved) {
            return Err(archive(format!("entry `{}` is not allowed", rel.display())));
        }
        Ok(out)
    }

    fn dir(&mut self, rel: &Path) -> Result<(), UnpackError> {
        let rel = self.checked(rel)?;
        let path = self.root.join(rel);
        std::fs::create_dir_all(&path).map_err(io(&path))
    }

    /// 写一个文件；`mode` 是压缩包里记的 unix 模式（没有就是 0）。
    pub fn file(
        &mut self,
        rel: &Path,
        mode: u32,
        mut reader: impl Read,
    ) -> Result<(), UnpackError> {
        let rel = self.checked(rel)?;
        let path = self.root.join(&rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(io(parent))?;
        }
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(io(&path))?;
        let mut h = Sha256::new();
        let mut size = 0u64;
        let mut buf = vec![0u8; 1 << 16];
        loop {
            let n = reader
                .read(&mut buf)
                .map_err(|e| archive(format!("reading `{}`: {e}", rel.display())))?;
            if n == 0 {
                break;
            }
            self.total += n as u64;
            if self.total > self.limits.bytes {
                return Err(archive(format!(
                    "unpacks to more than {} bytes",
                    self.limits.bytes
                )));
            }
            f.write_all(&buf[..n]).map_err(io(&path))?;
            h.update(&buf[..n]);
            size += n as u64;
        }
        f.sync_all().map_err(io(&path))?;
        set_mode(&path, mode)?;
        let path_text = rel
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect::<Vec<_>>()
            .join("/");
        self.files.push(UnpackedFile {
            path: path_text,
            size,
            sha256: hex(&h.finalize()),
        });
        Ok(())
    }
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<(), UnpackError> {
    use std::os::unix::fs::PermissionsExt;
    let bits = if mode & 0o111 != 0 { 0o755 } else { 0o644 };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(bits)).map_err(io(path))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> Result<(), UnpackError> {
    Ok(())
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// 把 `archive_path` 照 `format` 解进 `out` 的根。
pub(crate) fn unpack(
    out: &mut Unpacked,
    archive_path: &Path,
    format: Format,
) -> Result<(), UnpackError> {
    let open = || std::fs::File::open(archive_path).map_err(io(archive_path));
    match format {
        Format::TarBz2 => untar(
            out,
            bzip2::read::BzDecoder::new(std::io::BufReader::new(open()?)),
        ),
        Format::TarGz => untar(
            out,
            flate2::read::GzDecoder::new(std::io::BufReader::new(open()?)),
        ),
        Format::Zip => unzip(out, open()?),
    }
}

/// tar：GNU 长名和 pax 头由 `tar` 自己并进下一项。
fn untar(out: &mut Unpacked, reader: impl Read) -> Result<(), UnpackError> {
    use tar::EntryType;
    let mut ar = tar::Archive::new(reader);
    let bad = |e: std::io::Error| archive(format!("tar: {e}"));
    for entry in ar.entries().map_err(bad)? {
        let entry = entry.map_err(bad)?;
        let kind = entry.header().entry_type();
        let mode = entry.header().mode().unwrap_or(0);
        let path = entry.path().map_err(bad)?.into_owned();
        match kind {
            EntryType::Regular | EntryType::Continuous => out.file(&path, mode, entry)?,
            EntryType::Directory => out.dir(&path)?,
            EntryType::XGlobalHeader => {}
            other => {
                return Err(archive(format!(
                    "entry `{}` is a {other:?}, not a file or folder",
                    path.display()
                )));
            }
        }
    }
    Ok(())
}

/// zip：按 unix 模式认出符号链接和特殊文件。
fn unzip(out: &mut Unpacked, file: std::fs::File) -> Result<(), UnpackError> {
    let bad = |e: zip::result::ZipError| archive(format!("zip: {e}"));
    let mut ar = zip::ZipArchive::new(file).map_err(bad)?;
    for i in 0..ar.len() {
        let entry = ar.by_index(i).map_err(bad)?;
        let name = entry.name().map_err(bad)?.into_owned();
        let mode = entry.unix_mode().unwrap_or(0);
        let special = !matches!(mode & 0o170000, 0 | 0o100000 | 0o040000);
        if entry.is_symlink() || special {
            return Err(archive(format!("entry `{name}` is not a file or folder")));
        }
        let path = entry
            .enclosed_name()
            .ok_or_else(|| archive(format!("entry `{name}` is not a plain relative path")))?;
        if entry.is_dir() {
            out.dir(&path)?;
        } else {
            out.file(&path, mode, entry)?;
        }
    }
    Ok(())
}
