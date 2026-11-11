//! 凭据和密钥（协议 §2.11、§6）：存在钥匙串里，每次调用只把这个能力声明过的站点的交出去。
//!
//! - 值只在 [`Vault`] 里：键 `site:<站点>` 是那个站点的 storageState，`secret:<插件>:<设置>` 是一个密钥设置。
//! - 按机器的目录里另有一份索引 `logins.json`：站点的 cookie 名、域名、过期时间和登录的时间，
//!   哪些密钥设置填了。没有值，看登录状态不用碰钥匙串。
//! - 交给插件的文件放在按机器目录的 `run/` 下、这次调用自己的 0700 目录里，文件 0600，
//!   进程一结束整个目录删掉；不放在资料库的暂存目录里。

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::PluginError;
use crate::protocol::{
    CapabilityUse, Cookie, Credential, CredentialSite, Credentials, Login, OriginState, PluginId,
    Site, StorageState,
};

use super::FetchPlugin;

/// 存秘密的地方：系统钥匙串，没有钥匙串服务的机器上是一个 0600 文件。调用会阻塞，可能弹窗。
pub trait Vault: Send + Sync + std::fmt::Debug {
    fn read(&self, key: &str) -> Result<Option<String>, PluginError>;
    fn write(&self, key: &str, value: &str) -> Result<(), PluginError>;
    /// 没有这个键不算错。
    fn remove(&self, key: &str) -> Result<(), PluginError>;
}

/// 一个站点现在登录着没有（协议 §3.10，宿主按 `sites.cookies` 自己算）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoginState {
    Missing,
    Present,
    Expired,
}

/// 索引里的一个 cookie：没有值。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CookieName {
    name: String,
    domain: String,
    expires: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SiteRecord {
    captured_at: String,
    cookies: Vec<CookieName>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct Index {
    #[serde(default)]
    sites: BTreeMap<String, SiteRecord>,
    /// 插件 id → 填了值的密钥设置。
    #[serde(default)]
    secrets: BTreeMap<String, BTreeSet<String>>,
}

/// 装着的插件声明过的一个登录站点，合起来看。
#[derive(Debug, Clone)]
pub struct DeclaredSite {
    /// 第一个声明它的插件写的那份；`domains` 是所有插件声明的合起来。
    pub site: Site,
    pub plugins: Vec<PluginId>,
    pub login: LoginState,
    /// 上次存下登录的时间（RFC 3339）。
    pub captured_at: Option<String>,
}

fn site_key(site: &str) -> String {
    format!("site:{site}")
}

fn secret_key(plugin: &PluginId, key: &str) -> String {
    format!("secret:{plugin}:{key}")
}

pub(crate) fn now_secs() -> f64 {
    jiff::Timestamp::now().as_second() as f64
}

/// 一个 cookie 的域名属不属于这个站点（协议 §6）：站点本身和它的上级域名（不含公共后缀），
/// 加上 `domains` 显式列出的及其子域。
pub(crate) fn belongs(site: &Site, domain: &str) -> bool {
    let d = domain.trim_start_matches('.').to_ascii_lowercase();
    if d.is_empty() {
        return false;
    }
    let host = site.site.to_ascii_lowercase();
    let public_suffix = psl::suffix_str(&d) == Some(d.as_str());
    if !public_suffix && (host == d || host.ends_with(&format!(".{d}"))) {
        return true;
    }
    site.domains.iter().any(|x| {
        let x = x.trim_start_matches('.').to_ascii_lowercase();
        d == x || d.ends_with(&format!(".{x}"))
    })
}

/// 只留属于这个站点、没过期的 cookie 和 localStorage。
pub(crate) fn restrict(site: &Site, state: &StorageState, now: f64) -> StorageState {
    StorageState {
        cookies: state
            .cookies
            .iter()
            .filter(|c| belongs(site, &c.domain) && !c.expired_at(now))
            .cloned()
            .collect(),
        origins: state
            .origins
            .iter()
            .filter(|o| {
                url::Url::parse(&o.origin)
                    .ok()
                    .and_then(|u| u.host_str().map(|h| belongs(site, h)))
                    .unwrap_or(false)
            })
            .cloned()
            .collect(),
    }
}

fn login_of(site: &Site, record: Option<&SiteRecord>, now: f64) -> LoginState {
    let Some(r) = record else {
        return LoginState::Missing;
    };
    let found: Vec<&CookieName> = if site.cookies.is_empty() {
        r.cookies.iter().collect()
    } else {
        let mut found = Vec::new();
        for name in &site.cookies {
            match r.cookies.iter().find(|c| &c.name == name) {
                Some(c) => found.push(c),
                None => return LoginState::Missing,
            }
        }
        found
    };
    if found.is_empty() {
        LoginState::Missing
    } else if found.iter().any(|c| c.expires > 0.0 && c.expires <= now) {
        LoginState::Expired
    } else {
        LoginState::Present
    }
}

/// cookies.txt（Netscape 格式），给 yt-dlp 一类工具。
fn netscape(cookies: &[Cookie]) -> String {
    let mut out = String::from("# Netscape HTTP Cookie File\n");
    for c in cookies {
        let sub = if c.domain.starts_with('.') {
            "TRUE"
        } else {
            "FALSE"
        };
        let expires = if c.expires > 0.0 { c.expires as i64 } else { 0 };
        let prefix = if c.http_only { "#HttpOnly_" } else { "" };
        out.push_str(&format!(
            "{prefix}{}\t{sub}\t{}\t{}\t{expires}\t{}\t{}\n",
            c.domain,
            c.path,
            if c.secure { "TRUE" } else { "FALSE" },
            c.name,
            c.value
        ));
    }
    out
}

/// 这次调用交出去的东西：目录随它删掉；`values` 用来把日志里的值洗掉。
pub(crate) struct Handover {
    _dir: Option<tempfile::TempDir>,
    pub env: Vec<(OsString, OsString)>,
    pub values: Vec<String>,
    /// 这次交了哪些站点：`credentialsUpdate` 只往它们写回。
    pub sites: Vec<Site>,
}

impl Handover {
    fn none() -> Handover {
        Handover {
            _dir: None,
            env: Vec::new(),
            values: Vec::new(),
            sites: Vec::new(),
        }
    }
}

/// 钥匙串、索引和每次调用的凭据目录。没配（[`Logins::none`]）时什么都不交，存凭据报错。
pub(crate) struct Logins {
    store: Option<Store>,
    /// 改索引是读、改、写一整份，同一时间只一个。
    lock: Mutex<()>,
}

struct Store {
    vault: std::sync::Arc<dyn Vault>,
    index_path: PathBuf,
    run_dir: PathBuf,
}

impl Logins {
    pub fn none() -> Logins {
        Logins {
            store: None,
            lock: Mutex::new(()),
        }
    }

    /// `private_dir` 是按机器的目录（不在资料库里）。
    pub fn new(private_dir: &Path, vault: std::sync::Arc<dyn Vault>) -> Logins {
        Logins {
            store: Some(Store {
                vault,
                index_path: private_dir.join("logins.json"),
                run_dir: private_dir.join("run"),
            }),
            lock: Mutex::new(()),
        }
    }

    fn store(&self) -> Result<&Store, PluginError> {
        self.store
            .as_ref()
            .ok_or_else(|| PluginError::Vault("no secret store is configured".into()))
    }

    /// 上次没删掉的凭据目录（进程被杀时会剩下）：开机时清掉。
    pub fn sweep(&self) -> Result<(), PluginError> {
        let Some(store) = &self.store else {
            return Ok(());
        };
        let run_dir = &store.run_dir;
        let entries = match std::fs::read_dir(run_dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(io(run_dir, e)),
        };
        for entry in entries {
            let entry = entry.map_err(|e| io(run_dir, e))?;
            let path = entry.path();
            let ours = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(RUN_PREFIX));
            if ours {
                std::fs::remove_dir_all(&path).map_err(|e| io(&path, e))?;
            }
        }
        Ok(())
    }

    fn vault(&self) -> Result<&dyn Vault, PluginError> {
        Ok(self.store()?.vault.as_ref())
    }

    fn read_index(&self) -> Result<Index, PluginError> {
        let Some(store) = &self.store else {
            return Ok(Index::default());
        };
        let path = &store.index_path;
        match std::fs::read(path) {
            Ok(b) => serde_json::from_slice(&b).map_err(|e| {
                PluginError::Vault(format!("{} is not readable: {e}", path.display()))
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Index::default()),
            Err(e) => Err(io(path, e)),
        }
    }

    fn write_index(&self, index: &Index) -> Result<(), PluginError> {
        let bytes =
            serde_json::to_vec_pretty(index).map_err(|e| PluginError::Vault(e.to_string()))?;
        let path = &self.store()?.index_path;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| io(dir, e))?;
        }
        write_private(path, &bytes)
    }

    /// 装着的插件声明过的 `credential: cookies` 站点，合起来，带登录状态。
    pub fn declared(&self, plugins: &[FetchPlugin]) -> Result<Vec<DeclaredSite>, PluginError> {
        let index = self.read_index()?;
        let now = now_secs();
        let mut out: Vec<DeclaredSite> = Vec::new();
        for p in plugins {
            for s in p
                .spec
                .sites
                .iter()
                .filter(|s| s.credential == Credential::Cookies)
            {
                match out.iter_mut().find(|d| d.site.site == s.site) {
                    Some(d) => {
                        d.plugins.push(p.id().clone());
                        for dom in &s.domains {
                            if !d.site.domains.contains(dom) {
                                d.site.domains.push(dom.clone());
                            }
                        }
                        for name in &s.cookies {
                            if !d.site.cookies.contains(name) {
                                d.site.cookies.push(name.clone());
                            }
                        }
                    }
                    None => out.push(DeclaredSite {
                        site: s.clone(),
                        plugins: vec![p.id().clone()],
                        login: LoginState::Missing,
                        captured_at: None,
                    }),
                }
            }
        }
        for d in &mut out {
            let record = index.sites.get(&d.site.site);
            d.login = login_of(&d.site, record, now);
            d.captured_at = record.map(|r| r.captured_at.clone());
        }
        Ok(out)
    }

    /// 存下登录窗口交来的 cookie：只留属于这个站点（合起所有插件的声明）的。
    pub fn save(
        &self,
        plugins: &[FetchPlugin],
        site: &str,
        state: &StorageState,
    ) -> Result<DeclaredSite, PluginError> {
        let declared = self
            .declared(plugins)?
            .into_iter()
            .find(|d| d.site.site == site)
            .ok_or_else(|| PluginError::Credentials {
                target: site.into(),
                message: "no installed plugin declares this site with `credential: cookies`".into(),
            })?;
        let kept = restrict(&declared.site, state, now_secs());
        if kept.cookies.is_empty() {
            return Err(PluginError::Credentials {
                target: site.into(),
                message: "none of the cookies belong to this site".into(),
            });
        }
        let guard = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.store_site(site, &kept)?;
        drop(guard);
        self.declared(plugins)?
            .into_iter()
            .find(|d| d.site.site == site)
            .ok_or_else(|| PluginError::Vault("site vanished while saving".into()))
    }

    /// 写钥匙串和索引；调用方拿着锁。
    fn store_site(&self, site: &str, state: &StorageState) -> Result<(), PluginError> {
        let json = serde_json::to_string(state).map_err(|e| PluginError::Vault(e.to_string()))?;
        self.vault()?.write(&site_key(site), &json)?;
        let mut index = self.read_index()?;
        index.sites.insert(
            site.into(),
            SiteRecord {
                captured_at: jiff::Timestamp::now().to_string(),
                cookies: state
                    .cookies
                    .iter()
                    .map(|c| CookieName {
                        name: c.name.clone(),
                        domain: c.domain.clone(),
                        expires: c.expires,
                    })
                    .collect(),
            },
        );
        self.write_index(&index)
    }

    pub fn forget(&self, site: &str) -> Result<(), PluginError> {
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.vault()?.remove(&site_key(site))?;
        let mut index = self.read_index()?;
        if index.sites.remove(site).is_some() {
            self.write_index(&index)?;
        }
        Ok(())
    }

    /// 填一个密钥设置；空的就是去掉。
    pub fn set_secret(
        &self,
        plugin: &FetchPlugin,
        key: &str,
        value: &str,
    ) -> Result<(), PluginError> {
        if !plugin
            .header
            .settings
            .iter()
            .any(|s| s.secret && s.key == key)
        {
            return Err(PluginError::Credentials {
                target: format!("{}:{key}", plugin.id()),
                message: "the plugin declares no secret setting with this key".into(),
            });
        }
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let vkey = secret_key(plugin.id(), key);
        let mut index = self.read_index()?;
        let set = index.secrets.entry(plugin.id().to_string()).or_default();
        if value.trim().is_empty() {
            self.vault()?.remove(&vkey)?;
            set.remove(key);
        } else {
            self.vault()?.write(&vkey, value)?;
            set.insert(key.into());
        }
        index.secrets.retain(|_, keys| !keys.is_empty());
        self.write_index(&index)
    }

    /// 填了值的密钥设置。
    pub fn secrets_set(&self, plugin: &PluginId) -> Result<BTreeSet<String>, PluginError> {
        Ok(self
            .read_index()?
            .secrets
            .remove(plugin.as_str())
            .unwrap_or_default())
    }

    /// 准备这次调用要交的：`cap` 是这次用的能力（`canonicalize` 没有）。读钥匙串，会阻塞。
    pub fn handover(
        &self,
        plugin: &FetchPlugin,
        cap: Option<&CapabilityUse>,
    ) -> Result<Handover, PluginError> {
        let index = self.read_index()?;
        let now = now_secs();
        let mut sites = Vec::new();
        if let Some(cap) = cap.filter(|c| c.login != Login::None) {
            let mut names: Vec<&String> = cap.sites.iter().collect();
            for p in cap.per_platform.values().filter(|p| p.login != Login::None) {
                names.extend(p.sites.iter());
            }
            for name in names {
                let Some(site) = plugin.spec.sites.iter().find(|s| &s.site == name) else {
                    continue;
                };
                // 只有 `cookies` 一种；`browser-profile`、`app` 宿主现在什么都不传。
                if site.credential != Credential::Cookies
                    || !index.sites.contains_key(&site.site)
                    || sites
                        .iter()
                        .any(|(s, _): &(Site, StorageState)| s.site == site.site)
                {
                    continue;
                }
                let Some(json) = self.vault()?.read(&site_key(&site.site))? else {
                    continue;
                };
                let state: StorageState = serde_json::from_str(&json).map_err(|e| {
                    PluginError::Vault(format!(
                        "stored login for {} is not readable: {e}",
                        site.site
                    ))
                })?;
                let kept = restrict(site, &state, now);
                if !kept.cookies.is_empty() {
                    sites.push((site.clone(), kept));
                }
            }
        }
        let secret_keys = index.secrets.get(plugin.id().as_str());
        let mut secrets = serde_json::Map::new();
        for s in plugin.header.settings.iter().filter(|s| s.secret) {
            if !secret_keys.is_some_and(|k| k.contains(&s.key)) {
                continue;
            }
            if let Some(v) = self.vault()?.read(&secret_key(plugin.id(), &s.key))? {
                secrets.insert(s.key.clone(), serde_json::Value::String(v));
            }
        }
        if sites.is_empty() && secrets.is_empty() {
            return Ok(Handover::none());
        }

        let run_dir = &self.store()?.run_dir;
        std::fs::create_dir_all(run_dir).map_err(|e| io(run_dir, e))?;
        let dir = tempfile::Builder::new()
            .prefix(RUN_PREFIX)
            .tempdir_in(run_dir)
            .map_err(|e| io(run_dir, e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700))
                .map_err(|e| io(dir.path(), e))?;
        }
        let mut env = Vec::new();
        let mut values = Vec::new();
        if !sites.is_empty() {
            let mut all = StorageState::default();
            for (_, s) in &sites {
                all.cookies.extend(s.cookies.iter().cloned());
                all.origins.extend(s.origins.iter().cloned());
            }
            values.extend(all.cookies.iter().map(|c| c.value.clone()));
            values.extend(
                all.origins
                    .iter()
                    .flat_map(|o: &OriginState| o.local_storage.iter().map(|e| e.value.clone())),
            );
            let file = Credentials {
                sites: sites
                    .iter()
                    .map(|(s, _)| CredentialSite {
                        site: s.site.clone(),
                        source: "login-window".into(),
                    })
                    .collect(),
                storage_state: all.clone(),
            };
            let path = dir.path().join("credentials.json");
            let bytes = serde_json::to_vec(&file).map_err(|e| PluginError::Vault(e.to_string()))?;
            write_private(&path, &bytes)?;
            env.push(("USUYO_CREDENTIALS".into(), path.into()));
            let txt = dir.path().join("cookies.txt");
            write_private(&txt, netscape(&all.cookies).as_bytes())?;
            env.push(("USUYO_COOKIES_TXT".into(), txt.into()));
        }
        if !secrets.is_empty() {
            values.extend(
                secrets
                    .values()
                    .filter_map(|v| v.as_str().map(str::to_owned)),
            );
            let path = dir.path().join("secrets.json");
            let bytes =
                serde_json::to_vec(&secrets).map_err(|e| PluginError::Vault(e.to_string()))?;
            write_private(&path, &bytes)?;
            env.push(("USUYO_SECRETS".into(), path.into()));
        }
        // 太短的值洗起来会误伤正常的字，也猜得出来；不洗。
        values.retain(|v| v.len() >= 6);
        Ok(Handover {
            _dir: Some(dir),
            env,
            values,
            sites: sites.into_iter().map(|(s, _)| s).collect(),
        })
    }

    /// 插件交回的 `credentialsUpdate`（协议 §6）：只认这次交出去的站点的 cookie，按名字、域名、路径换掉旧的。
    pub fn update(&self, sites: &[Site], update: &serde_json::Value) -> Result<(), PluginError> {
        let incoming: StorageState =
            serde_json::from_value(update.clone()).map_err(|e| PluginError::Credentials {
                target: "credentialsUpdate".into(),
                // 只说是哪一类错：serde 的原话会引用值，那可能就是新的 cookie
                message: format!("not a storageState ({:?} error)", e.classify()),
            })?;
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = now_secs();
        for site in sites {
            let fresh = restrict(site, &incoming, now);
            if fresh.cookies.is_empty() {
                continue;
            }
            let mut stored: StorageState = match self.vault()?.read(&site_key(&site.site))? {
                Some(json) => serde_json::from_str(&json).map_err(|e| {
                    PluginError::Vault(format!("stored login is not readable: {e}"))
                })?,
                None => StorageState::default(),
            };
            for c in fresh.cookies {
                stored
                    .cookies
                    .retain(|o| !(o.name == c.name && o.domain == c.domain && o.path == c.path));
                stored.cookies.push(c);
            }
            self.store_site(&site.site, &stored)?;
        }
        Ok(())
    }
}

const RUN_PREFIX: &str = "usuyo-cred-";

fn io(path: &Path, source: std::io::Error) -> PluginError {
    PluginError::Io {
        path: path.to_owned(),
        source,
    }
}

/// 只给自己读写的文件（0600）：先写旁边的临时文件再换名。
fn write_private(path: &Path, bytes: &[u8]) -> Result<(), PluginError> {
    use std::io::Write;
    let tmp = path.with_extension("tmp");
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
    let mut f = opts.open(&tmp).map_err(|e| io(&tmp, e))?;
    #[cfg(unix)]
    f.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))
        .map_err(|e| io(&tmp, e))?;
    f.write_all(bytes).map_err(|e| io(&tmp, e))?;
    std::fs::rename(&tmp, path).map_err(|e| io(path, e))
}

/// 把交出去的值从插件的日志和错误里洗掉。
pub(crate) fn scrub(text: &mut String, values: &[String]) {
    for v in values {
        if text.contains(v.as_str()) {
            *text = text.replace(v.as_str(), "[redacted]");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Credential;

    fn site(domains: &[&str]) -> Site {
        Site {
            site: "www.bilibili.com".into(),
            name: "B站".into(),
            credential: Credential::Cookies,
            login_url: None,
            cookies: vec!["SESSDATA".into()],
            domains: domains.iter().map(|d| (*d).into()).collect(),
        }
    }

    #[test]
    fn cookie_domains_match_the_site_and_its_parents_only() {
        let s = site(&[]);
        assert!(belongs(&s, "www.bilibili.com"));
        assert!(belongs(&s, ".bilibili.com"));
        assert!(!belongs(&s, ".com"), "公共后缀不算");
        assert!(
            !belongs(&s, "api.bilibili.com"),
            "兄弟子域要显式写进 domains"
        );
        assert!(!belongs(&s, "evil-bilibili.com"));
        assert!(!belongs(&s, ".xiaohongshu.com"));
        let wide = site(&[".bilibili.com"]);
        assert!(belongs(&wide, "api.bilibili.com"));
        assert!(!belongs(&wide, "bilibili.com.evil.test"));
    }

    #[test]
    fn login_follows_the_named_cookies() {
        let s = site(&[]);
        let rec = |name: &str, expires: f64| SiteRecord {
            captured_at: String::new(),
            cookies: vec![CookieName {
                name: name.into(),
                domain: ".bilibili.com".into(),
                expires,
            }],
        };
        assert_eq!(login_of(&s, None, 10.0), LoginState::Missing);
        assert_eq!(
            login_of(&s, Some(&rec("other", -1.0)), 10.0),
            LoginState::Missing
        );
        assert_eq!(
            login_of(&s, Some(&rec("SESSDATA", -1.0)), 10.0),
            LoginState::Present
        );
        assert_eq!(
            login_of(&s, Some(&rec("SESSDATA", 5.0)), 10.0),
            LoginState::Expired
        );
        let mut unnamed = s.clone();
        unnamed.cookies.clear();
        assert_eq!(
            login_of(&unnamed, Some(&rec("x", 50.0)), 10.0),
            LoginState::Present
        );
    }

    #[test]
    fn scrub_replaces_values() {
        let mut t = "cookie SESSDATA=abcdef123 failed".to_owned();
        scrub(&mut t, &["abcdef123".into()]);
        assert_eq!(t, "cookie SESSDATA=[redacted] failed");
    }
}
