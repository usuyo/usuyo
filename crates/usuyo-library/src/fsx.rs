//! 文件读写：原子写（SPEC §0.6）、JSON 的固定写法、BOM 和 CRLF 照读照写。

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::id::{base32, random_bytes};
use crate::{Error, Result};

/// 同目录写 `.<名字>.usuyo-tmp-<随机>`，fsync，改名盖过去。
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    write_via_temp(path, bytes, false)
}

/// 同 [`atomic_write`]，写成只读文件（`content.md`）。
pub fn atomic_write_readonly(path: &Path, bytes: &[u8]) -> Result<()> {
    write_via_temp(path, bytes, true)
}

fn write_via_temp(path: &Path, bytes: &[u8], readonly: bool) -> Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| Error::InvalidArgs(format!("no parent for {}", path.display())))?;
    fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::InvalidArgs(format!("bad file name {}", path.display())))?;
    let tmp = dir.join(format!(
        ".{name}.usuyo-tmp-{}",
        base32(&random_bytes::<5>()?)
    ));
    let write = || -> std::io::Result<()> {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        if readonly {
            let mut perm = f.metadata()?.permissions();
            perm.set_readonly(true);
            fs::set_permissions(&tmp, perm)?;
        }
        Ok(())
    };
    if let Err(e) = write() {
        // 临时文件写一半就扔掉；扔不掉不影响目标文件，只报原来的错。
        let _ = fs::remove_file(&tmp);
        return Err(Error::io(&tmp, e));
    }
    clear_readonly_for_rename(path)?;
    fs::rename(&tmp, path).map_err(|e| Error::io(path, e))?;
    sync_dir(dir);
    Ok(())
}

/// Windows 上只读的目标不能被改名盖过去，先去掉只读。
#[cfg(windows)]
fn clear_readonly_for_rename(path: &Path) -> Result<()> {
    if let Ok(meta) = fs::metadata(path) {
        let mut perm = meta.permissions();
        if perm.readonly() {
            perm.set_readonly(false);
            fs::set_permissions(path, perm).map_err(|e| Error::io(path, e))?;
        }
    }
    Ok(())
}

#[cfg(not(windows))]
fn clear_readonly_for_rename(_path: &Path) -> Result<()> {
    Ok(())
}

/// 改名以后 fsync 目录；Windows 不支持打开目录，跳过。
fn sync_dir(dir: &Path) {
    #[cfg(unix)]
    if let Ok(d) = fs::File::open(dir) {
        // 目录 fsync 失败不改变改名已经生效的事实。
        let _ = d.sync_all();
    }
    #[cfg(not(unix))]
    let _ = dir;
}

/// 文件不在时回 `None`。
pub fn read_opt(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::io(path, e)),
    }
}

pub fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| Error::io(path, e))
}

/// JSON：缩进两格、键顺序照类型、以换行结尾。
pub fn to_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut out = serde_json::to_vec_pretty(value).map_err(|e| Error::Json {
        path: PathBuf::new(),
        source: e,
    })?;
    out.push(b'\n');
    Ok(out)
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    atomic_write(path, &to_json_bytes(value)?)
}

pub fn parse_json<T: DeserializeOwned>(path: &Path, bytes: &[u8]) -> Result<T> {
    let (text, _) = decode(path, bytes)?;
    serde_json::from_str(&text).map_err(|e| Error::Json {
        path: path.to_owned(),
        source: e,
    })
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    parse_json(path, &read(path)?)
}

/// 文件原来的 BOM 和换行，改写时保持原样。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextStyle {
    pub bom: bool,
    pub crlf: bool,
}

/// 读成 LF、不带 BOM 的字符串，记下原来的样子。
pub fn decode(path: &Path, bytes: &[u8]) -> Result<(String, TextStyle)> {
    let (bom, body) = match bytes.strip_prefix(b"\xEF\xBB\xBF") {
        Some(rest) => (true, rest),
        None => (false, bytes),
    };
    let text = std::str::from_utf8(body).map_err(|e| Error::malformed(path, e.to_string()))?;
    let crlf = text.contains("\r\n");
    let text = if crlf {
        text.replace("\r\n", "\n")
    } else {
        text.to_owned()
    };
    Ok((text, TextStyle { bom, crlf }))
}

pub fn encode(text: &str, style: TextStyle) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len() + 3);
    if style.bom {
        out.extend_from_slice(b"\xEF\xBB\xBF");
    }
    if style.crlf {
        out.extend_from_slice(text.replace('\n', "\r\n").as_bytes());
    } else {
        out.extend_from_slice(text.as_bytes());
    }
    out
}

/// 挪文件或文件夹，目标的上级不在就建。
pub fn move_path(from: &Path, to: &Path) -> Result<()> {
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
    }
    fs::rename(from, to).map_err(|e| Error::io(from, e))
}

/// 只有自己能进的文件夹（按机器的目录：令牌、任务队列、日志在里面），没有就建。
pub(crate) fn private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|e| Error::io(path, e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|e| Error::io(path, e))?;
    }
    Ok(())
}

/// 文件夹里有符号链接就报错：插件交来的输出目录不许带，免得指到资料库外面。
pub fn reject_symlinks(root: &Path) -> Result<()> {
    let entries = match fs::read_dir(root) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(Error::io(root, e)),
    };
    for entry in entries {
        let entry = entry.map_err(|e| Error::io(root, e))?;
        let path = entry.path();
        let ft = entry.file_type().map_err(|e| Error::io(&path, e))?;
        if ft.is_symlink() {
            return Err(Error::InvalidArgs(format!(
                "{} is a symlink",
                path.display()
            )));
        }
        if ft.is_dir() {
            reject_symlinks(&path)?;
        }
    }
    Ok(())
}

/// 文件夹里所有文件的相对路径（`/` 分隔），排好序。
pub fn list_files(root: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    walk(root, root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(Error::io(dir, e)),
    };
    for entry in entries {
        let entry = entry.map_err(|e| Error::io(dir, e))?;
        let path = entry.path();
        let ft = entry.file_type().map_err(|e| Error::io(&path, e))?;
        if ft.is_dir() {
            walk(root, &path, out)?;
        } else {
            out.push(rel_string(root, &path)?);
        }
    }
    Ok(())
}

pub fn rel_string(root: &Path, path: &Path) -> Result<String> {
    let rel = path.strip_prefix(root).map_err(|_| {
        Error::InvalidArgs(format!("{} is outside {}", path.display(), root.display()))
    })?;
    let parts: Option<Vec<&str>> = rel.components().map(|c| c.as_os_str().to_str()).collect();
    parts
        .map(|p| p.join("/"))
        .ok_or_else(|| Error::InvalidArgs(format!("non-UTF-8 path {}", path.display())))
}

/// 删文件；不在也算删掉了。
pub fn remove_file(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::io(path, e)),
    }
}

/// 删文件夹；不在也算删掉了。只读文件也删（`content.md`）。
pub fn remove_dir(path: &Path) -> Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::io(path, e)),
    }
}

/// 只删空了的上级文件夹，到 `stop` 为止。
pub fn prune_empty_dirs(mut dir: &Path, stop: &Path) {
    while dir != stop && dir.starts_with(stop) {
        if fs::remove_dir(dir).is_err() {
            break;
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces_and_leaves_no_temp() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.txt");
        atomic_write(&p, b"one").unwrap();
        atomic_write_readonly(&p, b"two").unwrap();
        atomic_write_readonly(&p, b"three").unwrap();
        assert_eq!(fs::read(&p).unwrap(), b"three");
        assert!(fs::metadata(&p).unwrap().permissions().readonly());
        assert_eq!(list_files(dir.path()).unwrap(), vec!["a.txt"]);
    }

    #[test]
    fn bom_and_crlf_survive() {
        let raw = b"\xEF\xBB\xBFa\r\nb\r\n";
        let (text, style) = decode(Path::new("x"), raw).unwrap();
        assert_eq!(text, "a\nb\n");
        assert_eq!(encode(&text, style), raw);
    }
}
