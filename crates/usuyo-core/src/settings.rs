//! 设置页的那几项：按机器的 `settings.json` 里的 `network.proxy`、`transforms.auto`、`window.systemTitleBar`、`language`，资料库的 `trashDays`，
//! 时钟守卫拦下的回收站项。按机器的 `settings.json` 只经这里的 [`read`]、[`write`]（方法的开关在 `methods`），
//! 别的键原样留着；改了马上生效。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};
use usuyo_contract::{Op, OpTarget};
use usuyo_plugin::ProxySetting;

use crate::engine::{Engine, Via};
use crate::trash::ClockGuard;
use crate::view::{NetworkView, SettingsView, TransformsSetting, TrashHeld, WindowSetting};
use crate::words::Lang;
use crate::{Error, Result};

fn settings_path(machine_root: &Path) -> PathBuf {
    machine_root.join("settings.json")
}

pub(crate) fn read(machine_root: &Path) -> Result<Map<String, Value>> {
    let path = settings_path(machine_root);
    match std::fs::read(&path) {
        Ok(b) => serde_json::from_slice(&b)
            .map_err(|e| Error::InvalidArgs(format!("{}: {e}", path.display()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Map::new()),
        Err(e) => Err(Error::io(&path, e)),
    }
}

/// 按机器的 `settings.json` 只经这里写：只有自己能读（代理地址可能带密码），先写临时文件再换名。
pub(crate) fn write(machine_root: &Path, map: &Map<String, Value>) -> Result<()> {
    let path = settings_path(machine_root);
    let mut bytes = serde_json::to_vec_pretty(map)?;
    bytes.push(b'\n');
    crate::write_private(&path, &bytes).map_err(|e| Error::io(&path, e))
}

/// 记下当前资料库；别的键原样留着。调用方：命令行的 `init`、第一次打开。
pub fn remember_library(machine_root: &Path, library: &Path) -> Result<()> {
    let mut map = read(machine_root)?;
    map.insert(
        "library".into(),
        Value::String(library.to_string_lossy().into_owned()),
    );
    write(machine_root, &map)
}

/// `transforms.auto`（feature 003 §4.4）：没写是开着；写了不是真假的报错。
pub fn transforms_auto(machine_root: &Path) -> Result<bool> {
    match read(machine_root)?
        .get("transforms")
        .and_then(|t| t.get("auto"))
    {
        None => Ok(true),
        Some(Value::Bool(b)) => Ok(*b),
        Some(other) => Err(Error::InvalidArgs(format!(
            "settings.json: transforms.auto is {other}, not true or false"
        ))),
    }
}

/// `window.systemTitleBar`：桌面壳开窗口时用不用系统标题栏。没写是不用；写了不是真假的报错。
/// 壳不链内核，开窗口时自己读同一个文件的这一项（`apps/desktop/src-tauri/src/window.rs`）。
fn system_title_bar(machine_root: &Path) -> Result<bool> {
    match read(machine_root)?
        .get("window")
        .and_then(|w| w.get("systemTitleBar"))
    {
        None => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(other) => Err(Error::InvalidArgs(format!(
            "settings.json: window.systemTitleBar is {other}, not true or false"
        ))),
    }
}

/// 设置里选的语言：`system`（没写时）跟随系统，或者定下一种。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize)]
pub enum Language {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "zh-Hans")]
    ZhHans,
    #[serde(rename = "en")]
    En,
}

impl Language {
    /// 这一种在这台机器上是哪种语言；`system` 照 `system` 给的。
    pub fn resolve(self, system: Lang) -> Lang {
        match self {
            Language::System => system,
            Language::ZhHans => Lang::ZhHans,
            Language::En => Lang::En,
        }
    }
}

/// `language`：没写是 `system`；写了别的报错。
pub fn language(machine_root: &Path) -> Result<Language> {
    match read(machine_root)?.get("language") {
        None => Ok(Language::System),
        Some(v) => serde_json::from_value(v.clone()).map_err(|_| {
            Error::InvalidArgs(format!(
                "settings.json: language is {v}, not \"system\", \"zh-Hans\" or \"en\""
            ))
        }),
    }
}

/// `network.proxy`：`system`（没写时）、`direct` 或一个代理地址；写坏了报错，错误里不带地址（可能有密码）。
pub fn proxy_setting(machine_root: &Path) -> Result<ProxySetting> {
    match read(machine_root)?
        .get("network")
        .and_then(|n| n.get("proxy"))
    {
        None => Ok(ProxySetting::System),
        Some(Value::String(text)) => ProxySetting::parse(text)
            .map_err(|why| Error::InvalidArgs(format!("settings.json: network.proxy: {why}"))),
        Some(_) => Err(Error::InvalidArgs(
            "settings.json: network.proxy is not a string".into(),
        )),
    }
}

/// `PATCH /settings` 的请求：给了的几项才改。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsPatch {
    /// 没给是不改，`null` 是「一直」。
    #[serde(default, deserialize_with = "present")]
    pub trash_days: Option<Option<u32>>,
    pub network: Option<NetworkPatch>,
    pub transforms: Option<TransformsPatch>,
    pub window: Option<WindowPatch>,
    pub language: Option<Language>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkPatch {
    pub mode: NetworkMode,
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkMode {
    System,
    Direct,
    Custom,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformsPatch {
    pub auto: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowPatch {
    pub system_title_bar: bool,
}

/// 写了的字段（含 `null`）是 `Some`。
fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}

/// 回收站能放几天：30、90，或一直（SPEC §1.1）。
const TRASH_DAYS: [u32; 2] = [30, 90];

/// 给界面看的代理地址：去掉密码。
fn shown(url: &str) -> String {
    // 有主机的地址（`ProxySetting` 核过）都去得掉密码；去不掉的不给看。
    let Ok(mut u) = url::Url::parse(url) else {
        return String::new();
    };
    if u.set_password(None).is_err() {
        return String::new();
    }
    u.as_str().trim_end_matches('/').to_owned()
}

/// 读回去的地址不带密码（[`shown`]）：交回来的就是那个、没带密码时，留着存着的密码。
fn keep_password(given: ProxySetting, stored: ProxySetting) -> ProxySetting {
    match (&given, stored) {
        (ProxySetting::Url(g), ProxySetting::Url(s))
            if url::Url::parse(g).is_ok_and(|u| u.password().is_none()) && shown(&s) == *g =>
        {
            ProxySetting::Url(s)
        }
        _ => given,
    }
}

impl Engine {
    /// 设置页。会读一次这台机器的网络环境（假 IP 代理），在阻塞线程上调。
    pub fn settings(&self) -> Result<SettingsView> {
        let root = &self.inner.machine_root;
        let (library, trash_days) = {
            let lib = self.lib();
            (lib.root().to_owned(), lib.settings().trash_days)
        };
        let network = match proxy_setting(root)? {
            ProxySetting::System => ("system", None),
            ProxySetting::Direct => ("direct", None),
            ProxySetting::Url(u) => ("custom", Some(shown(&u))),
        };
        let trash_held = self
            .trash_hold(&usuyo_library::time::now())?
            .map(|(count, since)| TrashHeld {
                count,
                since: usuyo_library::time::format(since),
            });
        Ok(SettingsView {
            library: library.to_string_lossy().into_owned(),
            trash_days,
            trash_held,
            network: NetworkView {
                mode: network.0,
                proxy: network.1,
                fake_ip: usuyo_plugin::behind_fake_ip_proxy(),
            },
            transforms: TransformsSetting {
                auto: transforms_auto(root)?,
            },
            window: WindowSetting {
                system_title_bar: system_title_bar(root)?,
            },
            language: language(root)?,
            logs: root.join("logs/daemon.log").to_string_lossy().into_owned(),
        })
    }

    /// 改设置：先核对给的每一项，再写文件，再换上正在用的值。回改好的整份。
    pub fn patch_settings(&self, patch: SettingsPatch, via: Via) -> Result<SettingsView> {
        if let Some(Some(d)) = patch.trash_days
            && !TRASH_DAYS.contains(&d)
        {
            return Err(Error::InvalidArgs(format!(
                "trashDays is 30, 90 or null, not {d}"
            )));
        }
        let proxy = match &patch.network {
            None => None,
            Some(n) => Some(match (n.mode, n.proxy.as_deref()) {
                (NetworkMode::System, None) => ProxySetting::System,
                (NetworkMode::Direct, None) => ProxySetting::Direct,
                (NetworkMode::Custom, Some(p)) => {
                    let given = ProxySetting::parse(p)
                        .map_err(|why| Error::InvalidArgs(format!("network.proxy: {why}")))?;
                    keep_password(given, proxy_setting(&self.inner.machine_root)?)
                }
                (NetworkMode::Custom, None) => {
                    return Err(Error::InvalidArgs("network.proxy is missing".into()));
                }
                (_, Some(_)) => {
                    return Err(Error::InvalidArgs(
                        "network.proxy is only for mode custom".into(),
                    ));
                }
            }),
        };
        let auto = patch.transforms.map(|t| t.auto);
        let title_bar = patch.window.map(|w| w.system_title_bar);
        let chosen = patch.language;
        // 资料库那一项先改：它没改成，按机器的文件和正在用的值都不动。
        if let Some(days) = patch.trash_days {
            let mut lib = self.lib();
            lib.apply(
                &via.batch()?,
                &OpTarget {
                    kind: "library".into(),
                    id: "settings".into(),
                },
                Op::SetSetting {
                    key: "trashDays".into(),
                    value: days.map_or(Value::Null, Value::from),
                },
            )?;
        }
        if proxy.is_some() || auto.is_some() || title_bar.is_some() || chosen.is_some() {
            let _guard = self
                .inner
                .settings_lock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let root = &self.inner.machine_root;
            let mut map = read(root)?;
            if let Some(p) = &proxy {
                let text = match p {
                    ProxySetting::System => "system".to_owned(),
                    ProxySetting::Direct => "direct".to_owned(),
                    ProxySetting::Url(u) => u.clone(),
                };
                set(&mut map, "network", "proxy", Value::String(text));
            }
            if let Some(a) = auto {
                set(&mut map, "transforms", "auto", Value::Bool(a));
            }
            if let Some(t) = title_bar {
                set(&mut map, "window", "systemTitleBar", Value::Bool(t));
            }
            if let Some(l) = chosen {
                map.insert("language".into(), serde_json::to_value(l)?);
            }
            write(root, &map)?;
        }
        if let Some(p) = proxy {
            self.inner.host.set_proxy(p.clone());
            self.transforms().models.set_route(p);
            self.emit(crate::view::Event::PluginsUpdated);
        }
        if let Some(a) = auto {
            self.set_transforms_auto(a)?;
            self.emit(crate::view::Event::TransformsUpdated);
        }
        if let Some(l) = chosen {
            self.set_language(l.resolve(self.inner.system_language));
        }
        self.settings()
    }

    /// 时钟守卫拦下的那几项照时间移除（SPEC §1.7「照时间移除」）。回移除了几项。
    pub fn release_held_trash(&self) -> Result<usize> {
        let r = self.expire_trash(&usuyo_library::time::now(), ClockGuard::Override)?;
        Ok(r.purged + r.files_removed)
    }
}

/// `map[section][key] = value`；那一节不是对象（没写过、写坏了）时换成只有这一项的对象。
fn set(map: &mut Map<String, Value>, section: &str, key: &str, value: Value) {
    match map.get_mut(section) {
        Some(Value::Object(m)) => {
            m.insert(key.to_owned(), value);
        }
        _ => {
            let mut m = Map::new();
            m.insert(key.to_owned(), value);
            map.insert(section.to_owned(), Value::Object(m));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_tells_absent_from_null() {
        let p: SettingsPatch = serde_json::from_str(r#"{"trashDays": null}"#).unwrap();
        assert_eq!(p.trash_days, Some(None));
        let p: SettingsPatch = serde_json::from_str(r#"{"trashDays": 90}"#).unwrap();
        assert_eq!(p.trash_days, Some(Some(90)));
        let p: SettingsPatch = serde_json::from_str("{}").unwrap();
        assert_eq!(p.trash_days, None);
        assert!(serde_json::from_str::<SettingsPatch>(r#"{"nope": 1}"#).is_err());
    }

    #[test]
    fn system_title_bar_is_off_unless_written() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!system_title_bar(dir.path()).unwrap());
        std::fs::write(
            dir.path().join("settings.json"),
            r#"{"window": {"systemTitleBar": true}}"#,
        )
        .unwrap();
        assert!(system_title_bar(dir.path()).unwrap());
        std::fs::write(
            dir.path().join("settings.json"),
            r#"{"window": {"systemTitleBar": 1}}"#,
        )
        .unwrap();
        assert!(system_title_bar(dir.path()).is_err());
        let p: SettingsPatch =
            serde_json::from_str(r#"{"window": {"systemTitleBar": false}}"#).unwrap();
        assert_eq!(p.window.map(|w| w.system_title_bar), Some(false));
        assert!(
            serde_json::from_str::<SettingsPatch>(r#"{"window": {"titleBar": false}}"#).is_err()
        );
    }

    #[test]
    fn language_is_system_unless_written() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(language(dir.path()).unwrap(), Language::System);
        std::fs::write(dir.path().join("settings.json"), r#"{"language": "en"}"#).unwrap();
        assert_eq!(language(dir.path()).unwrap(), Language::En);
        std::fs::write(dir.path().join("settings.json"), r#"{"language": "zh-CN"}"#).unwrap();
        assert!(language(dir.path()).is_err());
        assert_eq!(Language::System.resolve(Lang::En), Lang::En);
        assert_eq!(Language::ZhHans.resolve(Lang::En), Lang::ZhHans);
        let p: SettingsPatch = serde_json::from_str(r#"{"language": "zh-Hans"}"#).unwrap();
        assert_eq!(p.language, Some(Language::ZhHans));
        assert!(serde_json::from_str::<SettingsPatch>(r#"{"language": "fr"}"#).is_err());
    }

    #[test]
    fn shown_proxy_has_no_password() {
        assert_eq!(
            shown("http://me:secret@127.0.0.1:7890"),
            "http://me@127.0.0.1:7890"
        );
        assert_eq!(shown("socks5://127.0.0.1:7891"), "socks5://127.0.0.1:7891");
    }

    #[test]
    fn a_shown_proxy_sent_back_keeps_its_password() {
        let stored = ProxySetting::Url("http://me:secret@127.0.0.1:7890".into());
        let back = ProxySetting::parse("http://me@127.0.0.1:7890").unwrap();
        assert_eq!(keep_password(back, stored.clone()), stored);
        let other = ProxySetting::parse("http://me@127.0.0.1:7891").unwrap();
        assert_eq!(keep_password(other.clone(), stored), other);
    }
}
