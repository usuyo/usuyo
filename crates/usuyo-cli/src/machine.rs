//! 按机器的东西：根目录、`settings.json`（当前资料库、插件设置）、接口令牌、守护进程的地址、开发用的插件。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{Map, Value};

/// 按机器的根目录：`USUYO_HOME`，没有就是系统的位置（SPEC §0.1）。
pub fn root() -> Result<PathBuf> {
    match std::env::var_os("USUYO_HOME") {
        Some(p) => Ok(PathBuf::from(p)),
        None => Ok(usuyo_library::Library::default_machine_root()?),
    }
}

pub fn token_path(root: &Path) -> PathBuf {
    root.join("api-token")
}

pub fn daemon_path(root: &Path) -> PathBuf {
    root.join("daemon.json")
}

fn settings_path(root: &Path) -> PathBuf {
    root.join("settings.json")
}

fn read_settings(root: &Path) -> Result<Map<String, Value>> {
    let path = settings_path(root);
    match std::fs::read(&path) {
        Ok(b) => serde_json::from_slice(&b).with_context(|| format!("bad {}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Map::new()),
        Err(e) => Err(e).with_context(|| format!("cannot read {}", path.display())),
    }
}

/// 记下当前资料库；别的键原样留着。
pub fn remember_library(root: &Path, library: &Path) -> Result<()> {
    let mut s = read_settings(root)?;
    s.insert(
        "library".into(),
        Value::String(library.to_string_lossy().into_owned()),
    );
    std::fs::create_dir_all(root)?;
    let path = settings_path(root);
    let mut bytes = serde_json::to_vec_pretty(&s)?;
    bytes.push(b'\n');
    std::fs::write(&path, bytes).with_context(|| format!("cannot write {}", path.display()))
}

/// `settings.json` 里记着当前资料库没有。
pub fn has_library(root: &Path) -> Result<bool> {
    Ok(read_settings(root)?.contains_key("library"))
}

/// 第一次打开：在 `dir` 建资料库并记成当前的；那里已经有一个（设置丢过）就直接用它。
pub fn first_library(dir: &Path, root: &Path) -> Result<PathBuf> {
    let path = if dir.join(".usuyo/library.json").is_file() {
        dir.to_owned()
    } else {
        usuyo_core::Engine::init_library(dir, root)?
    };
    remember_library(root, &path)?;
    Ok(path)
}

/// 退出时去掉 daemon.json，只在它还登记着自己时。
pub fn forget_daemon(info: &Path, pid: u32) {
    let ours = std::fs::read(info)
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .and_then(|v| v.get("pid").and_then(Value::as_u64))
        == Some(u64::from(pid));
    if ours && let Err(e) = std::fs::remove_file(info) {
        tracing::warn!(path = %info.display(), error = %e, "cannot remove daemon.json");
    }
}

/// 当前资料库：`--library`、`USUYO_LIBRARY`，再没有就是 `usuyo init` 记下的那个。
pub fn library(root: &Path, flag: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = flag {
        return Ok(p);
    }
    read_settings(root)?
        .get("library")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .context("no library yet: run `usuyo init <dir>` or pass --library")
}

/// `settings.json` 里的 `plugins.<id>.settings`（协议 §2.11）。
pub fn plugin_settings(root: &Path) -> Result<BTreeMap<String, Value>> {
    let s = read_settings(root)?;
    let mut out = BTreeMap::new();
    if let Some(Value::Object(plugins)) = s.get("plugins") {
        for (id, v) in plugins {
            if let Some(settings) = v.get("settings") {
                out.insert(id.clone(), settings.clone());
            }
        }
    }
    Ok(out)
}

/// 开发时：把 `cargo build` 出来的 `usuyo-plugin-web`（和本程序在同一个目录）和仓库里的清单
/// 照安装的样子放进 `<root>/dev-plugins/builtin/web/<version>/`。发行版里没有这一步。
#[cfg(debug_assertions)]
pub fn stage_dev_web(root: &Path) -> Result<Option<PathBuf>> {
    const MANIFEST: &str = include_str!("../../../plugins/web/usuyo-plugin.yaml");
    let bin = std::env::current_exe()?.with_file_name("usuyo-plugin-web");
    if !bin.is_file() {
        return Ok(None);
    }
    let version = MANIFEST
        .lines()
        .find_map(|l| l.strip_prefix("version:"))
        .map(str::trim)
        .context("plugins/web manifest has no version")?;
    let plugins = root.join("dev-plugins");
    let dir = plugins.join("builtin/web").join(version);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("usuyo-plugin.yaml"), MANIFEST)?;
    std::fs::copy(&bin, dir.join("usuyo-plugin-web"))
        .with_context(|| format!("cannot copy {}", bin.display()))?;
    Ok(Some(plugins))
}

#[cfg(not(debug_assertions))]
pub fn stage_dev_web(_root: &Path) -> Result<Option<PathBuf>> {
    Ok(None)
}
