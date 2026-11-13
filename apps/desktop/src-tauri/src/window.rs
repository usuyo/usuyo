//! 主窗口：开的时候照按机器的 `settings.json` 里的 `window.systemTitleBar` 选标题栏。
//! 内核在 `crates/usuyo-core/src/settings.rs` 读写同一项；窗口开在内核起来以前，壳自己读这个文件。
//! 只在 macOS 上去掉系统标题栏：红绿灯浮在页面自己的顶上那一条上。Windows、Linux 一直用系统标题栏。

use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;
use tauri::{App, WebviewWindowBuilder};

/// 按机器的 `settings.json` 里 `window.systemTitleBar`：没有这个文件、没写这一项是 `false`；写坏了报错。
pub fn system_title_bar(machine_root: &Path) -> Result<bool> {
    let path = machine_root.join("settings.json");
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e).with_context(|| format!("cannot read {}", path.display())),
    };
    let map: Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("{} is not JSON", path.display()))?;
    match map.get("window").and_then(|w| w.get("systemTitleBar")) {
        None => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(other) => anyhow::bail!(
            "{}: window.systemTitleBar is {other}, not true or false",
            path.display()
        ),
    }
}

/// 这个窗口开的时候去掉了系统标题栏没有；前端照它留出顶上那一条、把那一条做成能拖窗口的地方。
pub struct Chrome {
    pub hidden_title_bar: bool,
}

/// 照 `tauri.conf.json` 里 `main` 那一份（`create: false`）开主窗口。
pub fn open_main(app: &App, system_title_bar: bool) -> Result<Chrome> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == "main")
        .context("tauri.conf.json has no `main` window")?
        .clone();
    let builder = WebviewWindowBuilder::from_config(app.handle(), &config)?;
    #[cfg(target_os = "macos")]
    let builder = if system_title_bar {
        builder
    } else {
        builder
            .title_bar_style(tauri::TitleBarStyle::Overlay)
            .hidden_title(true)
    };
    builder.build()?;
    Ok(Chrome {
        hidden_title_bar: cfg!(target_os = "macos") && !system_title_bar,
    })
}

#[cfg(test)]
mod tests {
    use super::system_title_bar;

    #[test]
    fn title_bar_setting_is_read_from_settings_json() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!system_title_bar(dir.path()).unwrap());
        let file = dir.path().join("settings.json");
        std::fs::write(&file, r#"{"network": {"proxy": "direct"}}"#).unwrap();
        assert!(!system_title_bar(dir.path()).unwrap());
        std::fs::write(&file, r#"{"window": {"systemTitleBar": true}}"#).unwrap();
        assert!(system_title_bar(dir.path()).unwrap());
        std::fs::write(&file, r#"{"window": {"systemTitleBar": "yes"}}"#).unwrap();
        assert!(system_title_bar(dir.path()).is_err());
        std::fs::write(&file, "not json").unwrap();
        assert!(system_title_bar(dir.path()).is_err());
    }
}
