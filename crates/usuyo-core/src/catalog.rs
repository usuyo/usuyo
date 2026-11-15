//! 订阅源和插件目录（协议 §8.4）：一个订阅源是一个放着 `index.json` 的 GitHub 仓库，宿主从它默认分支的
//! raw 地址读（`https://raw.githubusercontent.com/<owner>/<repo>/HEAD/index.json`）。
//!
//! 订阅源记在按机器的 `settings.json` 的 `pluginSources`（地址的数组）；没写时是 [`EngineConfig`] 给的默认那几个
//! （守护进程给 `usuyo-fetch/index`）。每个订阅源读到的索引连同 ETag 存在 `cache/sources/` 下，启动时先用它，
//! 守护进程起来时刷新一次，之后每 6 小时一次，也可以手动刷新。读索引和下载模型走同一条路（按机器的
//! `network.proxy`，只连公网，只走 https），4 MiB 封顶。
//!
//! 开发开关 `USUYO_DEV_LOOPBACK_SOURCES=1`（[`EngineConfig::dev_loopback_sources`]）打开时，订阅源和安装包
//! 只连本机回环地址、可以是明文 http：对着本机起的假订阅源测试用，这时连不上 GitHub。
//!
//! [`EngineConfig`]: crate::EngineConfig
//! [`EngineConfig::dev_loopback_sources`]: crate::EngineConfig

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use url::Url;

use crate::engine::Engine;
use crate::models::{ModelError, Net, Small};
use crate::view::{Event, PluginSourceView};
use crate::words::{self, Lang, Said};
use crate::{Error, Result};

/// 索引最多多大。
const INDEX_LIMIT: u64 = 4 << 20;
/// 一个安装包最多多大。
pub(crate) const ASSET_LIMIT: u64 = 256 << 20;
/// 隔多久自己刷新一次。
const REFRESH_EVERY: Duration = Duration::from_secs(6 * 3600);
/// 按机器 `settings.json` 里的键。
const SETTINGS_KEY: &str = "pluginSources";

/// 四个发版目标里这台电脑是哪一个。
pub(crate) fn this_target() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Some("macos-arm64"),
        ("macos", "x86_64") => Some("macos-x86_64"),
        ("linux", "x86_64") => Some("linux-x86_64"),
        ("windows", "x86_64") => Some("windows-x86_64"),
        _ => None,
    }
}

// —— index.json（usuyo-fetch/index 的 schema/index.schema.json）——

/// 一种语言一个字符串：`zh-Hans` 总在，`en` 可能没有。
pub(crate) type Names = BTreeMap<String, String>;

/// 按语言挑一个：有这种语言的用它，没有用 `zh-Hans`，再没有用第一个。
pub(crate) fn pick(names: &Names, lang: Lang) -> Option<String> {
    names
        .get(lang.tag())
        .or_else(|| names.get("zh-Hans"))
        .or_else(|| names.values().next())
        .cloned()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Index {
    pub format: u32,
    pub generated_at: String,
    #[serde(default)]
    pub platforms: BTreeMap<String, Names>,
    pub plugins: Vec<IndexPlugin>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IndexPlugin {
    pub id: String,
    pub kind: String,
    pub name: Names,
    #[serde(default)]
    pub description: Names,
    #[serde(default)]
    pub platforms: Vec<String>,
    pub repository: String,
    pub license: String,
    #[serde(default)]
    pub methods: Vec<IndexMethod>,
    pub release: Option<IndexRelease>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IndexMethod {
    pub id: String,
    pub name: Names,
    #[serde(default = "yes")]
    pub built: bool,
    pub access: String,
    pub login: String,
    pub account: String,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IndexRelease {
    pub version: semver::Version,
    pub tag: String,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    pub assets: Vec<IndexAsset>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IndexAsset {
    pub target: String,
    pub name: String,
    pub url: Url,
    pub size: u64,
    pub sha256: String,
}

impl IndexRelease {
    /// 这台电脑用的包。
    pub fn asset(&self) -> Option<&IndexAsset> {
        let target = this_target()?;
        self.assets.iter().find(|a| a.target == target)
    }
}

fn plain_id(id: &str) -> bool {
    let part = |s: &str| {
        !s.is_empty()
            && s.len() <= 100
            && s != "."
            && s != ".."
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_.".contains(&b))
    };
    matches!(id.split_once('/'), Some((o, r)) if part(o) && part(r))
}

/// 索引读进来时查一次：格式、id、每个包的地址、大小和 sha256。之后照信。
pub(crate) fn parse_index(bytes: &[u8], loopback: bool) -> std::result::Result<Index, String> {
    let index: Index = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if index.format != 1 {
        return Err(format!("format {} is not 1", index.format));
    }
    let mut seen = std::collections::BTreeSet::new();
    for p in &index.plugins {
        if !plain_id(&p.id) || p.id.starts_with("builtin/") {
            return Err(format!("plugin id `{}` is not owner/repo", p.id));
        }
        if !seen.insert(p.id.as_str()) {
            return Err(format!("plugin `{}` is listed twice", p.id));
        }
        if p.name.is_empty() {
            return Err(format!("plugin `{}` has no name", p.id));
        }
        let Some(r) = &p.release else { continue };
        for a in &r.assets {
            let hex = a.sha256.len() == 64
                && a.sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
            if !hex {
                return Err(format!("asset `{}` of `{}` has no sha256", a.name, p.id));
            }
            if a.size == 0 || a.size > ASSET_LIMIT {
                return Err(format!(
                    "asset `{}` of `{}` has size {}",
                    a.name, p.id, a.size
                ));
            }
            if a.name.is_empty() || a.name.contains(['/', '\\']) || a.name.starts_with('.') {
                return Err(format!(
                    "asset name `{}` of `{}` is not plain",
                    a.name, p.id
                ));
            }
            let scheme_ok = a.url.scheme() == "https" || (loopback && a.url.scheme() == "http");
            if !scheme_ok {
                return Err(format!("asset `{}` of `{}` is not https", a.name, p.id));
            }
        }
    }
    Ok(index)
}

// —— 订阅源 ——

/// 一个订阅源的地址：记下来的写法和读索引的地址。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Address {
    pub text: String,
    pub index_url: Url,
}

/// 认出用户给的地址：`owner/repo`、`github.com/owner/repo`、`https://github.com/owner/repo(.git)`；
/// 开发开关打开时还认本机回环的 `http://127.0.0.1:<端口>/<路径>`（索引在 `<路径>/index.json`）。
pub(crate) fn parse_address(input: &str, loopback: bool) -> Option<Address> {
    let text = input.trim();
    if loopback && text.starts_with("http://") {
        let url = Url::parse(text).ok()?;
        let local = match url.host()? {
            url::Host::Ipv4(ip) => ip.is_loopback(),
            url::Host::Ipv6(ip) => ip.is_loopback(),
            url::Host::Domain(d) => d == "localhost",
        };
        if !local || url.query().is_some() || !url.username().is_empty() {
            return None;
        }
        let base = text.trim_end_matches('/').to_owned();
        let index_url = Url::parse(&format!("{base}/index.json")).ok()?;
        return Some(Address {
            text: base,
            index_url,
        });
    }
    let rest = text
        .strip_prefix("https://")
        .or_else(|| text.strip_prefix("http://"))
        .unwrap_or(text);
    let rest = rest
        .strip_prefix("www.github.com/")
        .or_else(|| rest.strip_prefix("github.com/"))
        .unwrap_or(if text.contains("://") { "" } else { rest });
    let rest = rest.trim_end_matches('/');
    let rest = rest
        .strip_suffix(".git")
        .unwrap_or(rest)
        .to_ascii_lowercase();
    if !plain_id(&rest) {
        return None;
    }
    let index_url = Url::parse(&format!(
        "https://raw.githubusercontent.com/{rest}/HEAD/index.json"
    ))
    .ok()?;
    Some(Address {
        text: rest,
        index_url,
    })
}

/// 读到的新索引：解析好的、ETag、原样的字节（写缓存用）。
type Fresh = (Index, Option<String>, Vec<u8>);

/// 一个订阅源现在的样子。
struct Source {
    address: Address,
    index: Option<Arc<Index>>,
    etag: Option<String>,
    refreshed_at: Option<String>,
    refreshing: bool,
    error: Option<Said>,
}

/// 存在 `cache/sources/<地址的哈希>.json` 里的：上次读到的索引和它的 ETag。
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Cached {
    address: String,
    etag: Option<String>,
    refreshed_at: String,
    index: Value,
}

/// 引擎里订阅源那一路的东西。
pub(crate) struct Catalog {
    net: Net,
    pub loopback: bool,
    defaults: Vec<String>,
    cache_dir: PathBuf,
    sources: Mutex<Vec<Source>>,
    /// 改订阅源的设置时一次一个。
    edit: tokio::sync::Mutex<()>,
}

fn cache_name(address: &str) -> String {
    let h = Sha256::digest(address.as_bytes());
    format!("{}.json", &crate::unpack::hex(&h)[..16])
}

impl Catalog {
    pub fn new(
        machine_root: &Path,
        net: Net,
        loopback: bool,
        defaults: Vec<String>,
    ) -> Result<Catalog> {
        let catalog = Catalog {
            net,
            loopback,
            defaults,
            cache_dir: machine_root.join("cache/sources"),
            sources: Mutex::new(Vec::new()),
            edit: tokio::sync::Mutex::new(()),
        };
        let list = catalog.configured(machine_root)?;
        let sources = list
            .into_iter()
            .map(|address| catalog.cached_source(address))
            .collect();
        *catalog.lock() = sources;
        Ok(catalog)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Source>> {
        self.sources.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn net(&self) -> &Net {
        &self.net
    }

    /// 设置里的订阅源；没写时是默认那几个。写坏了报错。
    fn configured(&self, machine_root: &Path) -> Result<Vec<Address>> {
        let texts: Vec<String> = match crate::settings::read(machine_root)?.get(SETTINGS_KEY) {
            None => self.defaults.clone(),
            Some(v) => serde_json::from_value(v.clone()).map_err(|_| {
                Error::InvalidArgs(format!(
                    "settings.json: {SETTINGS_KEY} is not an array of addresses"
                ))
            })?,
        };
        texts
            .iter()
            .map(|t| {
                parse_address(t, self.loopback).ok_or_else(|| {
                    Error::InvalidArgs(format!(
                        "settings.json: {SETTINGS_KEY} has an address that is not usable: {t}"
                    ))
                })
            })
            .collect()
    }

    /// 读缓存里的索引；没有或读不了就是还没读到过（读不了写一行日志）。
    fn cached_source(&self, address: Address) -> Source {
        let path = self.cache_dir.join(cache_name(&address.text));
        let mut source = Source {
            address,
            index: None,
            etag: None,
            refreshed_at: None,
            refreshing: false,
            error: None,
        };
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return source,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "source cache not readable");
                return source;
            }
        };
        let cached: Cached = match serde_json::from_slice(&bytes) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "source cache not readable");
                return source;
            }
        };
        if cached.address != source.address.text {
            return source;
        }
        match serde_json::to_vec(&cached.index)
            .map_err(|e| e.to_string())
            .and_then(|b| parse_index(&b, self.loopback))
        {
            Ok(index) => {
                source.index = Some(Arc::new(index));
                source.etag = cached.etag;
                source.refreshed_at = Some(cached.refreshed_at);
            }
            Err(e) => tracing::warn!(path = %path.display(), error = %e, "cached index dropped"),
        }
        source
    }

    fn write_cache(&self, address: &str, etag: Option<&str>, at: &str, raw: &[u8]) -> Result<()> {
        std::fs::create_dir_all(&self.cache_dir).map_err(|e| Error::io(&self.cache_dir, e))?;
        let index: Value = serde_json::from_slice(raw)?;
        let cached = Cached {
            address: address.to_owned(),
            etag: etag.map(str::to_owned),
            refreshed_at: at.to_owned(),
            index,
        };
        let path = self.cache_dir.join(cache_name(address));
        let bytes = serde_json::to_vec(&cached)?;
        crate::write_private(&path, &bytes).map_err(|e| Error::io(&path, e))
    }

    /// 所有订阅源的索引，照订阅源的顺序。
    pub fn indexes(&self) -> Vec<(String, Arc<Index>)> {
        self.lock()
            .iter()
            .filter_map(|s| s.index.clone().map(|i| (s.address.text.clone(), i)))
            .collect()
    }

    pub fn views(&self, lang: Lang) -> Vec<PluginSourceView> {
        self.lock()
            .iter()
            .map(|s| PluginSourceView {
                address: s.address.text.clone(),
                index_url: s.address.index_url.to_string(),
                default: self.defaults.contains(&s.address.text),
                refreshed_at: s.refreshed_at.clone(),
                plugins: s.index.as_ref().map(|i| i.plugins.len()),
                refreshing: s.refreshing,
                error: s.error.as_ref().map(|e| e.pick(lang).to_owned()),
            })
            .collect()
    }

    fn addresses(&self) -> Vec<String> {
        self.lock().iter().map(|s| s.address.text.clone()).collect()
    }
}

/// 读索引没成时的一句话。
fn source_hint(e: &ModelError) -> Said {
    match e {
        ModelError::Status(404) => words::SOURCE_NOT_FOUND.said(),
        ModelError::Refused(_) | ModelError::NotHttps(_) => words::SOURCE_REFUSED.said(),
        ModelError::TooLarge(_) => words::SOURCE_TOO_LARGE.said(),
        _ => words::SOURCE_NETWORK.said(),
    }
}

fn hinted(source: Error, hint: Said) -> Error {
    Error::Hinted {
        source: Box::new(source),
        hint,
    }
}

impl Engine {
    pub(crate) fn catalog(&self) -> &Catalog {
        &self.inner.catalog
    }

    /// 订阅源，照设置里的顺序。
    pub fn source_list(&self) -> Vec<PluginSourceView> {
        self.catalog().views(self.lang())
    }

    /// 刷新一个订阅源（`address`）或全部（`None`），等它们读完。读不成的记在那个订阅源上，不当作这次调用失败。
    pub async fn refresh_plugin_sources(
        &self,
        address: Option<&str>,
    ) -> Result<Vec<PluginSourceView>> {
        let targets: Vec<String> = match address {
            Some(a) => {
                let a = parse_address(a, self.catalog().loopback)
                    .ok_or_else(|| {
                        hinted(
                            Error::InvalidArgs(format!("source address `{a}`")),
                            words::SOURCE_ADDRESS.said(),
                        )
                    })?
                    .text;
                if !self.catalog().addresses().contains(&a) {
                    return Err(hinted(
                        Error::NotFound(format!("source {a}")),
                        words::SOURCE_UNKNOWN.said(),
                    ));
                }
                vec![a]
            }
            None => self.catalog().addresses(),
        };
        for a in targets {
            self.refresh_source(&a).await;
        }
        Ok(self.source_list())
    }

    /// 读一个订阅源的索引：带上次的 ETag；读到新的就换上、写进缓存。
    async fn refresh_source(&self, address: &str) {
        let (url, etag) = {
            let mut sources = self.catalog().lock();
            let Some(s) = sources.iter_mut().find(|s| s.address.text == address) else {
                return;
            };
            if s.refreshing {
                return;
            }
            s.refreshing = true;
            (
                s.address.index_url.clone(),
                s.index.as_ref().and(s.etag.clone()),
            )
        };
        self.emit(Event::CatalogUpdated);
        let got = self
            .catalog()
            .net
            .get_small(&url, INDEX_LIMIT, etag.as_deref())
            .await;
        let at = usuyo_library::time::now();
        let outcome: std::result::Result<Option<Fresh>, Said> = match got {
            Ok(Small::NotModified) => Ok(None),
            Ok(Small::Body { bytes, etag }) => match parse_index(&bytes, self.catalog().loopback) {
                Ok(index) => Ok(Some((index, etag, bytes))),
                Err(reason) => {
                    tracing::warn!(source = address, %reason, "index not usable");
                    Err(words::SOURCE_FORMAT.with(&[("reason", &reason)]))
                }
            },
            Err(e) => {
                tracing::warn!(source = address, error = %e, "index not read");
                Err(source_hint(&e))
            }
        };
        if let Ok(Some((_, etag, bytes))) = &outcome
            && let Err(e) = self
                .catalog()
                .write_cache(address, etag.as_deref(), &at, bytes)
        {
            tracing::warn!(source = address, error = %e, "source cache not written");
        }
        {
            let mut sources = self.catalog().lock();
            if let Some(s) = sources.iter_mut().find(|s| s.address.text == address) {
                s.refreshing = false;
                match outcome {
                    Ok(Some((index, etag, _))) => {
                        s.index = Some(Arc::new(index));
                        s.etag = etag;
                        s.refreshed_at = Some(at);
                        s.error = None;
                    }
                    Ok(None) => {
                        s.refreshed_at = Some(at);
                        s.error = None;
                    }
                    Err(hint) => s.error = Some(hint),
                }
            }
        }
        self.emit(Event::CatalogUpdated);
    }

    /// 添加一个订阅源并读一次它的索引。读不成也留着，错误记在它上面。
    pub async fn add_plugin_source(&self, input: &str) -> Result<Vec<PluginSourceView>> {
        let address = parse_address(input, self.catalog().loopback).ok_or_else(|| {
            hinted(
                Error::InvalidArgs(format!("source address `{input}`")),
                words::SOURCE_ADDRESS.said(),
            )
        })?;
        {
            let _edit = self.catalog().edit.lock().await;
            let mut list = self.catalog().addresses();
            if list.contains(&address.text) {
                return Err(hinted(
                    Error::Conflict(format!("source {} exists", address.text)),
                    words::SOURCE_EXISTS.said(),
                ));
            }
            list.push(address.text.clone());
            self.write_sources(&list)?;
            let fresh = self.catalog().cached_source(address.clone());
            self.catalog().lock().push(fresh);
        }
        self.emit(Event::CatalogUpdated);
        self.refresh_source(&address.text).await;
        Ok(self.source_list())
    }

    /// 去掉一个订阅源。从它装的插件留着，只是不再从它更新。
    pub async fn remove_plugin_source(&self, input: &str) -> Result<Vec<PluginSourceView>> {
        let address = parse_address(input, self.catalog().loopback).ok_or_else(|| {
            hinted(
                Error::InvalidArgs(format!("source address `{input}`")),
                words::SOURCE_ADDRESS.said(),
            )
        })?;
        {
            let _edit = self.catalog().edit.lock().await;
            let mut list = self.catalog().addresses();
            if !list.contains(&address.text) {
                return Err(hinted(
                    Error::NotFound(format!("source {}", address.text)),
                    words::SOURCE_UNKNOWN.said(),
                ));
            }
            list.retain(|a| a != &address.text);
            self.write_sources(&list)?;
            self.catalog()
                .lock()
                .retain(|s| s.address.text != address.text);
            let cache = self.catalog().cache_dir.join(cache_name(&address.text));
            match std::fs::remove_file(&cache) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(Error::io(&cache, e)),
            }
        }
        self.emit(Event::CatalogUpdated);
        Ok(self.source_list())
    }

    fn write_sources(&self, list: &[String]) -> Result<()> {
        let _lock = self
            .inner
            .settings_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut map = crate::settings::read(&self.inner.machine_root)?;
        map.insert(SETTINGS_KEY.into(), serde_json::to_value(list)?);
        crate::settings::write(&self.inner.machine_root, &map)
    }

    /// 守护进程里跑：起来时刷新一次，之后每 6 小时一次。
    pub(crate) async fn run_sources(&self) {
        loop {
            if let Err(e) = self.refresh_plugin_sources(None).await {
                tracing::warn!(error = %e, "sources not refreshed");
            }
            tokio::time::sleep(REFRESH_EVERY).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses() {
        let a = parse_address("usuyo-fetch/index", false).unwrap();
        assert_eq!(a.text, "usuyo-fetch/index");
        assert_eq!(
            a.index_url.as_str(),
            "https://raw.githubusercontent.com/usuyo-fetch/index/HEAD/index.json"
        );
        for ok in [
            "https://github.com/Usuyo-Fetch/Index",
            "github.com/usuyo-fetch/index/",
            "https://github.com/usuyo-fetch/index.git",
        ] {
            assert_eq!(
                parse_address(ok, false).unwrap().text,
                "usuyo-fetch/index",
                "{ok}"
            );
        }
        for bad in [
            "",
            "usuyo-fetch",
            "a/b/c",
            "https://gitlab.com/a/b",
            "../x",
            "http://127.0.0.1:8080/s",
        ] {
            assert!(parse_address(bad, false).is_none(), "{bad}");
        }
        let local = parse_address("http://127.0.0.1:8080/s/", true).unwrap();
        assert_eq!(local.text, "http://127.0.0.1:8080/s");
        assert_eq!(
            local.index_url.as_str(),
            "http://127.0.0.1:8080/s/index.json"
        );
        assert!(parse_address("http://example.com/s", true).is_none());
    }

    #[test]
    fn index_checks() {
        let good = br#"{"format":1,"generatedAt":"2026-11-13T00:00:00.000Z","plugins":[
            {"id":"o/a","kind":"fetch","name":{"zh-Hans":"A"},"repository":"https://github.com/o/a","license":"MIT","release":null}]}"#;
        assert_eq!(parse_index(good, false).unwrap().plugins.len(), 1);
        let twice = br#"{"format":1,"generatedAt":"x","plugins":[
            {"id":"o/a","kind":"fetch","name":{"zh-Hans":"A"},"repository":"r","license":"MIT","release":null},
            {"id":"o/a","kind":"fetch","name":{"zh-Hans":"A"},"repository":"r","license":"MIT","release":null}]}"#;
        assert!(parse_index(twice, false).is_err());
        let builtin = br#"{"format":1,"generatedAt":"x","plugins":[
            {"id":"builtin/web","kind":"fetch","name":{"zh-Hans":"A"},"repository":"r","license":"MIT","release":null}]}"#;
        assert!(parse_index(builtin, false).is_err());
        let http = br#"{"format":1,"generatedAt":"x","plugins":[
            {"id":"o/a","kind":"fetch","name":{"zh-Hans":"A"},"repository":"r","license":"MIT","release":
              {"version":"0.1.0","tag":"v0.1.0","assets":[{"target":"linux-x86_64","name":"a.tar.gz","url":"http://127.0.0.1/a.tar.gz","size":3,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}}]}"#;
        assert!(parse_index(http, false).is_err());
        assert!(parse_index(http, true).is_ok());
        assert!(parse_index(br#"{"format":2,"generatedAt":"x","plugins":[]}"#, false).is_err());
    }

    #[test]
    fn names_fall_back_to_chinese() {
        let mut n = Names::new();
        n.insert("zh-Hans".into(), "B站".into());
        assert_eq!(pick(&n, Lang::En).as_deref(), Some("B站"));
        n.insert("en".into(), "Bilibili".into());
        assert_eq!(pick(&n, Lang::En).as_deref(), Some("Bilibili"));
    }
}
