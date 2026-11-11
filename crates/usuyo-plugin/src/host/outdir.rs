//! 输出目录的检查（协议 §3.6，名字的规则照 §13 第 32 项和 SPEC §0.6）。
//!
//! 移进资料库之前遍历整个目录，不只看 JSON 引用的：只收普通文件和目录；
//! 符号链接、硬链接、FIFO、套接字、设备文件、不合规的名字、超过大小上限，整次不收。

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use unicode_normalization::{UnicodeNormalization, is_nfc};

use crate::protocol::RelPath;

/// 输出目录里的一个文件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutFile {
    pub path: RelPath,
    pub size: u64,
}

/// 每段名字最多多少字节（SPEC §0.6）。
const NAME_BYTES: usize = 200;
/// 相对输出目录的路径最多多少个 UTF-16 单位：条目文件夹以内 140，减去 `raw/<UUIDv7>/` 的 41。
const PATH_UTF16: usize = 140 - "raw/".len() - 36 - 1;
/// Windows 的保留名，带扩展名也算。
const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// 检查整个目录，返回里面的文件。错误是给日志看的原因，调用方记成 `invalid-output`。
pub(crate) fn inspect(root: &Path, max_bytes: u64) -> Result<Vec<OutFile>, String> {
    let meta = fs::symlink_metadata(root).map_err(|e| format!("output directory: {e}"))?;
    if !meta.file_type().is_dir() {
        return Err(
            "the output directory was replaced by something that is not a directory".into(),
        );
    }
    let mut walk = Walk {
        files: Vec::new(),
        total: 0,
        max_bytes,
    };
    walk.dir(root, "")?;
    walk.files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(walk.files)
}

struct Walk {
    files: Vec<OutFile>,
    total: u64,
    max_bytes: u64,
}

impl Walk {
    fn dir(&mut self, dir: &Path, prefix: &str) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|e| format!("{prefix}: {e}"))?;
        let mut folded = HashSet::new();
        for entry in entries {
            let entry = entry.map_err(|e| format!("{prefix}: {e}"))?;
            let name = entry.file_name();
            let name = name
                .to_str()
                .ok_or_else(|| format!("{prefix}: a name is not valid UTF-8"))?;
            let rel = if prefix.is_empty() {
                name.to_owned()
            } else {
                format!("{prefix}/{name}")
            };
            check_name(name).map_err(|why| format!("`{rel}`: {why}"))?;
            if !folded.insert(name.nfc().collect::<String>().to_lowercase()) {
                return Err(format!("`{rel}`: another name differs only in case"));
            }
            if rel.encode_utf16().count() > PATH_UTF16 {
                return Err(format!(
                    "`{rel}`: path is longer than {PATH_UTF16} UTF-16 units"
                ));
            }
            let meta = fs::symlink_metadata(entry.path()).map_err(|e| format!("`{rel}`: {e}"))?;
            let kind = meta.file_type();
            if kind.is_symlink() {
                return Err(format!("`{rel}`: symbolic links are not accepted"));
            } else if kind.is_dir() {
                self.dir(&entry.path(), &rel)?;
            } else if kind.is_file() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if meta.nlink() > 1 {
                        return Err(format!("`{rel}`: hard links are not accepted"));
                    }
                }
                self.total = self.total.saturating_add(meta.len());
                if self.total > self.max_bytes {
                    return Err(format!("output is larger than {} bytes", self.max_bytes));
                }
                self.files.push(OutFile {
                    path: RelPath::try_from(rel)?,
                    size: meta.len(),
                });
            } else {
                return Err(format!(
                    "`{rel}`: only regular files and directories are accepted"
                ));
            }
        }
        Ok(())
    }
}

fn check_name(name: &str) -> Result<(), &'static str> {
    if name.chars().any(char::is_control) {
        return Err("contains a control character");
    }
    if name.contains(['<', '>', ':', '"', '/', '\\', '|', '?', '*']) {
        return Err("contains a character some systems do not allow");
    }
    if name.starts_with('.') {
        return Err("starts with `.`");
    }
    if name.ends_with([' ', '.']) {
        return Err("ends with a space or `.`");
    }
    if name.len() > NAME_BYTES {
        return Err("is longer than 200 bytes");
    }
    if name.to_lowercase().starts_with("usuyo.") {
        return Err("names starting with `usuyo.` are reserved for the host");
    }
    if !is_nfc(name) {
        return Err("is not in Unicode NFC");
    }
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    if RESERVED.contains(&stem.as_str()) {
        return Err("is a reserved Windows name");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        for ok in [
            "page.html",
            "001.jpg",
            "手冲咖啡.md",
            "a b.txt",
            "console.log",
        ] {
            assert!(check_name(ok).is_ok(), "{ok}");
        }
        for bad in [
            ".hidden",
            "a:b",
            "trailing.",
            "trailing ",
            "usuyo.fetch.json",
            "Usuyo.x",
            "CON",
            "nul.txt",
            "a\u{7}b",
            "e\u{301}.txt",
        ] {
            assert!(check_name(bad).is_err(), "{bad}");
        }
        assert!(check_name(&"a".repeat(201)).is_err());
    }
}
