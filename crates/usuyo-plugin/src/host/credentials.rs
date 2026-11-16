//! 凭据和密钥（协议 §2.11、§6）：存在钥匙串里，每次调用只把这个能力声明过的站点的交出去。
//!
//! - 值只在 [`Vault`] 里：键 `site:<站点>` 是那个站点的 storageState，`visitor:<站点>` 是那个站点的访客身份
//!   （不是账号，和同一主机的账号分开），`secret:<插件>:<设置>` 是一个密钥设置。
//! - 按机器的目录里另有一份索引 `logins.json`：站点和访客身份的 cookie 名、域名、过期时间和存下的时间，
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
    Cookie, Credential, CredentialSite, Credentials, Login, Method, OriginState, PluginId, Site,
    StorageState, Visitor,
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
    /// 登录状态从哪来；没写的是登录窗口（这一项以前存下的）。
    #[serde(default)]
    source: LoginSource,
    /// 访客身份：取它的那个窗口的 User-Agent（不是机密，记在索引里）；以前存下的没有。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    user_agent: Option<String>,
}

/// 一个 `cookies` 站点的登录状态从哪来（协议 §6）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoginSource {
    /// 在 usuyo 的登录窗口里登录。
    #[default]
    LoginWindow,
    /// 你选的一个 cookies.txt（Netscape 格式）文件。
    CookieFile,
}

impl LoginSource {
    /// 交给插件的凭据文件里 `sites[].source` 写的。
    fn as_str(self) -> &'static str {
        match self {
            LoginSource::LoginWindow => "login-window",
            LoginSource::CookieFile => "cookie-file",
        }
    }
}

/// cookies.txt 读不了：第几行（从 1 数）、哪一栏不对。不带那一行的字，那里面是 cookie 的值。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("line {line}: {field} is not valid in a Netscape cookies.txt")]
pub struct CookiesTxtError {
    pub line: usize,
    pub field: &'static str,
}

/// 读一份 Netscape 格式的 cookies.txt：空行和 `#` 开头的是注释，`#HttpOnly_` 开头的是 HttpOnly 的 cookie；
/// 每行七栏用 Tab 分开：域名、含子域（`TRUE`/`FALSE`）、路径、只走 https（`TRUE`/`FALSE`）、过期（Unix 秒，`0` 是会话）、名字、值。
pub fn read_cookies_txt(text: &str) -> Result<StorageState, CookiesTxtError> {
    let mut cookies = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let bad = |field| CookiesTxtError { line: i + 1, field };
        let (line, http_only) = match line.strip_prefix("#HttpOnly_") {
            Some(rest) => (rest, true),
            None if line.trim().is_empty() || line.starts_with('#') => continue,
            None => (line, false),
        };
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 7 {
            return Err(bad("the number of fields"));
        }
        let flag = |v: &str, field| match v.to_ascii_uppercase().as_str() {
            "TRUE" => Ok(true),
            "FALSE" => Ok(false),
            _ => Err(bad(field)),
        };
        let host = f[0].trim_start_matches('.');
        if host.is_empty() || host.contains(['/', ' ', ':']) {
            return Err(bad("domain"));
        }
        let domain = if flag(f[1], "include subdomains")? {
            format!(".{host}")
        } else {
            host.to_owned()
        };
        if !f[2].starts_with('/') {
            return Err(bad("path"));
        }
        let secure = flag(f[3], "secure")?;
        let expires: i64 = f[4].parse().map_err(|_| bad("expires"))?;
        if f[5].is_empty() {
            return Err(bad("name"));
        }
        cookies.push(Cookie {
            name: f[5].to_owned(),
            value: f[6].to_owned(),
            domain,
            path: f[2].to_owned(),
            expires: if expires > 0 { expires as f64 } else { -1.0 },
            http_only,
            secure,
            same_site: None,
        });
    }
    Ok(StorageState {
        cookies,
        origins: Vec::new(),
    })
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct Index {
    #[serde(default)]
    sites: BTreeMap<String, SiteRecord>,
    /// 访客身份，按站点；和 `sites` 是两回事。
    #[serde(default)]
    visitors: BTreeMap<String, SiteRecord>,
    /// 插件 id → 填了值的密钥设置。
    #[serde(default)]
    secrets: BTreeMap<String, BTreeSet<String>>,
    /// `browser-profile` 站点：在 usuyo 的站点窗口里登录过。网页数据在壳那份数据仓库里，
    /// 这里只记时间和上次用它的方法，cookie 不拷到钥匙串、不进这份索引。
    #[serde(default)]
    profiles: BTreeMap<String, ProfileRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileRecord {
    signed_in_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_use: Option<ProfileUse>,
}

/// 上次用这个站点窗口读的那一回：什么时候、哪个插件的哪种方法、记下了多少条链接。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUse {
    pub at: String,
    pub plugin: String,
    pub method: String,
    pub noted: u32,
}

/// 装着的插件声明过的一个登录站点，合起来看。
#[derive(Debug, Clone)]
pub struct DeclaredSite {
    /// 第一个声明它的插件写的那份；`domains` 是所有插件声明的合起来。
    pub site: Site,
    pub plugins: Vec<PluginId>,
    pub login: LoginState,
    /// 上次存下登录的时间（RFC 3339）；`browser-profile` 站点是在站点窗口里登录好的时间。
    pub captured_at: Option<String>,
    /// `browser-profile` 站点上次用站点窗口读的那一回。
    pub last_use: Option<ProfileUse>,
    /// `cookies` 站点存着的登录状态从哪来；没存着时没有。
    pub source: Option<LoginSource>,
    /// 存着的 cookie 的名字（不含值），给诊断看；`browser-profile` 站点的 cookie 不读出来，总是空的。
    pub cookie_names: Vec<String>,
}

/// 装着的插件声明过的一个访客身份，合起来看。
#[derive(Debug, Clone)]
pub struct DeclaredVisitor {
    /// 第一个声明它的插件写的那份；`domains`、`cookies` 是所有插件声明的合起来。
    pub visitor: Visitor,
    pub plugins: Vec<PluginId>,
    pub state: LoginState,
    /// 上次存下的时间（RFC 3339）。
    pub captured_at: Option<String>,
    /// 最早过期的那个 cookie 什么时候过期（Unix 秒）；都是会话 cookie 时没有。
    pub expires: Option<f64>,
    /// 存着的 cookie 的名字（不含值），给诊断看。
    pub cookie_names: Vec<String>,
    /// 取它的那个窗口的 User-Agent。
    pub user_agent: Option<String>,
}

/// 一个站点在装着的插件里只能有一种凭据形式（`cookies` 或 `browser-profile`）：按给定的顺序，
/// 后面的插件和前面已留下的插件对同一站点声明了不同形式，就不加载，原因里写明两边。
pub fn refuse_credential_conflicts(
    fetch: Vec<FetchPlugin>,
) -> (Vec<FetchPlugin>, Vec<super::Rejected>) {
    let mut kept: Vec<FetchPlugin> = Vec::new();
    let mut refused = Vec::new();
    for p in fetch {
        let conflict = p
            .spec
            .sites
            .iter()
            .filter(|s| s.credential != Credential::App)
            .find_map(|s| {
                kept.iter().find_map(|q| {
                    q.spec
                        .sites
                        .iter()
                        .find(|t| {
                            t.site == s.site
                                && t.credential != Credential::App
                                && t.credential != s.credential
                        })
                        .map(|t| (s, q.id(), t.credential))
                })
            });
        match conflict {
            None => kept.push(p),
            Some((s, other, other_kind)) => {
                let message = format!(
                    "site `{}` is `{}` here but `{}` in {other}: one site has one credential type across installed plugins",
                    s.site,
                    credential_name(s.credential),
                    credential_name(other_kind),
                );
                refused.push(super::Rejected {
                    dir: p.dir.clone(),
                    error: PluginError::Manifest {
                        field: "sites".into(),
                        message: format!("{} not loaded: {message}", p.id()),
                    },
                });
            }
        }
    }
    (kept, refused)
}

fn credential_name(c: Credential) -> &'static str {
    match c {
        Credential::Cookies => "credential: cookies",
        Credential::BrowserProfile => "credential: browser-profile",
        Credential::App => "credential: app",
    }
}

/// `Index::profiles` 的键：站点窗口的网页数据一个插件一份。
fn profile_key(plugin: &PluginId, site: &str) -> String {
    profile_key_of(plugin.as_str(), site)
}

fn profile_key_of(plugin: &str, site: &str) -> String {
    format!("{plugin}:{site}")
}

fn site_key(site: &str) -> String {
    format!("site:{site}")
}

fn visitor_key(site: &str) -> String {
    format!("visitor:{site}")
}

/// 一份 cookie 属于谁：主机、额外的域名、算数的 cookie 名。账号站点和访客身份都按它判断。
pub(crate) trait CookieScope {
    fn host(&self) -> &str;
    fn domains(&self) -> &[String];
    fn names(&self) -> &[String];
}

impl CookieScope for Site {
    fn host(&self) -> &str {
        &self.site
    }
    fn domains(&self) -> &[String] {
        &self.domains
    }
    fn names(&self) -> &[String] {
        &self.cookies
    }
}

impl CookieScope for Visitor {
    fn host(&self) -> &str {
        &self.site
    }
    fn domains(&self) -> &[String] {
        &self.domains
    }
    fn names(&self) -> &[String] {
        &self.cookies
    }
}

fn secret_key(plugin: &PluginId, key: &str) -> String {
    format!("secret:{plugin}:{key}")
}

pub(crate) fn now_secs() -> f64 {
    jiff::Timestamp::now().as_second() as f64
}

/// 一个 cookie 的域名属不属于这个站点（协议 §6）：站点本身和它的上级域名（不含公共后缀），
/// 加上 `domains` 显式列出的及其子域。
pub(crate) fn belongs(site: &impl CookieScope, domain: &str) -> bool {
    let d = domain.trim_start_matches('.').to_ascii_lowercase();
    if d.is_empty() {
        return false;
    }
    let host = site.host().to_ascii_lowercase();
    let public_suffix = psl::suffix_str(&d) == Some(d.as_str());
    if !public_suffix && (host == d || host.ends_with(&format!(".{d}"))) {
        return true;
    }
    site.domains().iter().any(|x| {
        let x = x.trim_start_matches('.').to_ascii_lowercase();
        d == x || d.ends_with(&format!(".{x}"))
    })
}

/// 只留属于这个站点、没过期的 cookie 和 localStorage。
pub(crate) fn restrict(site: &impl CookieScope, state: &StorageState, now: f64) -> StorageState {
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

/// 算数的那几个 cookie：声明了名字就是那几个（缺一个就是 `None`），没声明就是全部。
fn counted<'r>(site: &impl CookieScope, r: &'r SiteRecord) -> Option<Vec<&'r CookieName>> {
    if site.names().is_empty() {
        return Some(r.cookies.iter().collect());
    }
    site.names()
        .iter()
        .map(|name| r.cookies.iter().find(|c| &c.name == name))
        .collect()
}

fn login_of(site: &impl CookieScope, record: Option<&SiteRecord>, now: f64) -> LoginState {
    let Some(r) = record else {
        return LoginState::Missing;
    };
    let Some(found) = counted(site, r) else {
        return LoginState::Missing;
    };
    let lapsed = |c: &&CookieName| c.expires > 0.0 && c.expires <= now;
    // 写了名字的：那几个里有一个过期就算过期。没写名字的：交给插件的是没过期的那些（`restrict`），
    // 所以还有一个没过期就算登录着，全过期才算过期。
    let expired = if site.names().is_empty() {
        found.iter().all(lapsed)
    } else {
        found.iter().any(lapsed)
    };
    if found.is_empty() {
        LoginState::Missing
    } else if expired {
        LoginState::Expired
    } else {
        LoginState::Present
    }
}

/// cookies.txt（Netscape 格式），给读这种文件的工具。
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

/// 一种方法在这次调用的平台上要交凭据的站点（协议 §2.3.1）：`perPlatform` 写了这个平台就照它的 `login`、`sites`，
/// `platforms` 里别的平台照方法本身的；平台不知道、不在 `platforms` 里时照方法所有层合起来（同 [`Method::access_on`]）。
/// `login: none` 的那一层不交。
fn login_sites<'m>(
    m: &'m Method,
    platform: Option<&str>,
    declared: &[crate::protocol::PlatformId],
) -> Vec<&'m String> {
    let own = || -> Vec<&'m String> {
        if m.login == Login::None {
            Vec::new()
        } else {
            m.sites.iter().collect()
        }
    };
    if let Some(p) = platform {
        if let Some((_, u)) = m.per_platform.iter().find(|(x, _)| x.as_str() == p) {
            return if u.login == Login::None {
                Vec::new()
            } else {
                u.sites.iter().collect()
            };
        }
        if declared.iter().any(|x| x.as_str() == p) {
            return own();
        }
    }
    let mut names = own();
    for u in m.per_platform.values().filter(|u| u.login != Login::None) {
        names.extend(u.sites.iter());
    }
    names
}

/// 这次调用交出去的东西：目录随它删掉；`values` 用来把日志里的值洗掉。
pub(crate) struct Handover {
    _dir: Option<tempfile::TempDir>,
    pub env: Vec<(OsString, OsString)>,
    pub values: Vec<String>,
    /// 这次交了哪些站点：`credentialsUpdate` 只往它们写回。
    pub sites: Vec<Site>,
    /// 这次交出去的 cookie 的名字（账号的和访客身份的），不含值。
    pub cookie_names: Vec<String>,
}

impl Handover {
    fn none() -> Handover {
        Handover {
            _dir: None,
            env: Vec::new(),
            values: Vec::new(),
            sites: Vec::new(),
            cookie_names: Vec::new(),
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

    /// 装着的插件声明过的 `credential: cookies` 和 `browser-profile` 站点，合起来，带登录状态。
    /// `browser-profile` 的登录状态照站点窗口记下的：登录好了就是 `present`，退出登录以后没有记录。
    pub fn declared(&self, plugins: &[FetchPlugin]) -> Result<Vec<DeclaredSite>, PluginError> {
        let index = self.read_index()?;
        let now = now_secs();
        let mut out: Vec<DeclaredSite> = Vec::new();
        for p in plugins {
            for s in p
                .spec
                .sites
                .iter()
                .filter(|s| s.credential != Credential::App)
            {
                // 站点窗口的网页数据一个插件一份：browser-profile 站点按 (插件, 站点) 各列一项，不合并。
                let shared = s.credential == Credential::Cookies;
                match out
                    .iter_mut()
                    .find(|d| shared && d.site.site == s.site && d.site.credential == s.credential)
                {
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
                        last_use: None,
                        source: None,
                        cookie_names: Vec::new(),
                    }),
                }
            }
        }
        for d in &mut out {
            if d.site.credential == Credential::BrowserProfile {
                let record = index
                    .profiles
                    .get(&profile_key(&d.plugins[0], &d.site.site));
                d.login = if record.is_some() {
                    LoginState::Present
                } else {
                    LoginState::Missing
                };
                d.captured_at = record.map(|r| r.signed_in_at.clone());
                d.last_use = record.and_then(|r| r.last_use.clone());
                continue;
            }
            let record = index.sites.get(&d.site.site);
            d.login = login_of(&d.site, record, now);
            d.captured_at = record.map(|r| r.captured_at.clone());
            d.source = record.map(|r| r.source);
            d.cookie_names = record.map(names_of).unwrap_or_default();
        }
        Ok(out)
    }

    /// 一个装着的插件自己声明过的 `browser-profile` 站点；别的回错。
    fn profile_site(
        &self,
        plugins: &[FetchPlugin],
        plugin: &str,
        site: &str,
    ) -> Result<(), PluginError> {
        let declared = plugins.iter().any(|p| {
            p.id().as_str() == plugin
                && p.spec
                    .sites
                    .iter()
                    .any(|s| s.site == site && s.credential == Credential::BrowserProfile)
        });
        if declared {
            Ok(())
        } else {
            Err(PluginError::Credentials {
                target: site.into(),
                message: format!(
                    "plugin {plugin} does not declare this site with `credential: browser-profile`"
                ),
            })
        }
    }

    /// 站点窗口里登录好了：记下时间（已经记着的不改，退出登录才清掉）。
    pub fn profile_signed_in(
        &self,
        plugins: &[FetchPlugin],
        plugin: &str,
        site: &str,
    ) -> Result<(), PluginError> {
        self.profile_site(plugins, plugin, site)?;
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut index = self.read_index()?;
        let key = profile_key_of(plugin, site);
        if index.profiles.contains_key(&key) {
            return Ok(());
        }
        index.profiles.insert(
            key,
            ProfileRecord {
                signed_in_at: jiff::Timestamp::now().to_string(),
                last_use: None,
            },
        );
        self.write_index(&index)
    }

    /// 站点窗口读完一回：记下哪种方法、记下了多少条。还没记登录时一并记上。
    pub fn profile_used(
        &self,
        plugins: &[FetchPlugin],
        site: &str,
        plugin: &str,
        method: &str,
        noted: u32,
    ) -> Result<(), PluginError> {
        self.profile_site(plugins, plugin, site)?;
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut index = self.read_index()?;
        let now = jiff::Timestamp::now().to_string();
        let record = index
            .profiles
            .entry(profile_key_of(plugin, site))
            .or_insert_with(|| ProfileRecord {
                signed_in_at: now.clone(),
                last_use: None,
            });
        record.last_use = Some(ProfileUse {
            at: now,
            plugin: plugin.into(),
            method: method.into(),
            noted,
        });
        self.write_index(&index)
    }

    /// 存下登录窗口交来的、或从 cookies.txt 读出的 cookie：只留属于这个站点（合起所有插件的声明）的，记下从哪来。
    pub fn save(
        &self,
        plugins: &[FetchPlugin],
        site: &str,
        state: &StorageState,
        source: LoginSource,
    ) -> Result<DeclaredSite, PluginError> {
        let declared = self
            .declared(plugins)?
            .into_iter()
            .find(|d| d.site.site == site && d.site.credential == Credential::Cookies)
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
        // 文件里没有登录后才有的那几个（导出时没登录、已过期）：存下也算没登录，不存
        if source == LoginSource::CookieFile
            && login_of(&declared.site, Some(&record(&kept)), now_secs()) != LoginState::Present
        {
            return Err(PluginError::Credentials {
                target: site.into(),
                message: "the file has no signed-in cookies for this site".into(),
            });
        }
        let guard = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.store_site(site, &kept, Some(source))?;
        drop(guard);
        self.declared(plugins)?
            .into_iter()
            .find(|d| d.site.site == site && d.site.credential == Credential::Cookies)
            .ok_or_else(|| PluginError::Vault("site vanished while saving".into()))
    }

    /// 写钥匙串和索引；调用方拿着锁。`source` 空是照存着的那份（插件交回的 `credentialsUpdate`）。
    fn store_site(
        &self,
        site: &str,
        state: &StorageState,
        source: Option<LoginSource>,
    ) -> Result<(), PluginError> {
        let json = serde_json::to_string(state).map_err(|e| PluginError::Vault(e.to_string()))?;
        self.vault()?.write(&site_key(site), &json)?;
        let mut index = self.read_index()?;
        let mut r = record(state);
        r.source = source
            .or_else(|| index.sites.get(site).map(|old| old.source))
            .unwrap_or_default();
        index.sites.insert(site.into(), r);
        self.write_index(&index)
    }

    /// 装着的插件声明过的访客身份，合起来，带有没有、什么时候过期。
    pub fn visitors(&self, plugins: &[FetchPlugin]) -> Result<Vec<DeclaredVisitor>, PluginError> {
        let index = self.read_index()?;
        let now = now_secs();
        let mut out: Vec<DeclaredVisitor> = Vec::new();
        for p in plugins {
            for v in &p.spec.visitors {
                match out.iter_mut().find(|d| d.visitor.site == v.site) {
                    Some(d) => {
                        d.plugins.push(p.id().clone());
                        for dom in &v.domains {
                            if !d.visitor.domains.contains(dom) {
                                d.visitor.domains.push(dom.clone());
                            }
                        }
                        for name in &v.cookies {
                            if !d.visitor.cookies.contains(name) {
                                d.visitor.cookies.push(name.clone());
                            }
                        }
                    }
                    None => out.push(DeclaredVisitor {
                        visitor: v.clone(),
                        plugins: vec![p.id().clone()],
                        state: LoginState::Missing,
                        captured_at: None,
                        expires: None,
                        cookie_names: Vec::new(),
                        user_agent: None,
                    }),
                }
            }
        }
        for d in &mut out {
            let record = index.visitors.get(&d.visitor.site);
            d.state = login_of(&d.visitor, record, now);
            d.captured_at = record.map(|r| r.captured_at.clone());
            d.cookie_names = record.map(names_of).unwrap_or_default();
            d.user_agent = record.and_then(|r| r.user_agent.clone());
            d.expires = record
                .and_then(|r| counted(&d.visitor, r))
                .and_then(|found| {
                    found
                        .iter()
                        .map(|c| c.expires)
                        .filter(|t| *t > 0.0)
                        .reduce(f64::min)
                });
        }
        Ok(out)
    }

    /// 存下访客窗口交来的 cookie：只留属于这个访客身份（合起所有插件的声明）的。不碰同一主机的账号。
    /// `user_agent` 是取它的那个窗口的 UA，调用时随访客身份交给插件。
    pub fn save_visitor(
        &self,
        plugins: &[FetchPlugin],
        site: &str,
        state: &StorageState,
        user_agent: Option<&str>,
    ) -> Result<DeclaredVisitor, PluginError> {
        let find = |list: Vec<DeclaredVisitor>| list.into_iter().find(|d| d.visitor.site == site);
        let declared = find(self.visitors(plugins)?).ok_or_else(|| PluginError::Credentials {
            target: format!("visitor:{site}"),
            message: "no installed plugin declares this visitor identity".into(),
        })?;
        // 访客身份只要 cookie，不要 localStorage。
        let kept = StorageState {
            cookies: restrict(&declared.visitor, state, now_secs()).cookies,
            origins: Vec::new(),
        };
        if kept.cookies.is_empty() {
            return Err(PluginError::Credentials {
                target: format!("visitor:{site}"),
                message: "none of the cookies belong to this visitor identity".into(),
            });
        }
        let json = serde_json::to_string(&kept).map_err(|e| PluginError::Vault(e.to_string()))?;
        {
            let _guard = self
                .lock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            self.vault()?.write(&visitor_key(site), &json)?;
            let mut index = self.read_index()?;
            index.visitors.insert(
                site.into(),
                SiteRecord {
                    user_agent: user_agent.map(str::to_owned),
                    ..record(&kept)
                },
            );
            self.write_index(&index)?;
        }
        find(self.visitors(plugins)?)
            .ok_or_else(|| PluginError::Vault("visitor identity vanished while saving".into()))
    }

    pub fn forget_visitor(&self, site: &str) -> Result<(), PluginError> {
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.vault()?.remove(&visitor_key(site))?;
        let mut index = self.read_index()?;
        if index.visitors.remove(site).is_some() {
            self.write_index(&index)?;
        }
        Ok(())
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

    /// 忘掉一个插件在站点窗口里的登录记录；别的插件同一站点的不动。
    pub fn forget_profile(&self, plugin: &str, site: &str) -> Result<(), PluginError> {
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut index = self.read_index()?;
        if index
            .profiles
            .remove(&profile_key_of(plugin, site))
            .is_some()
        {
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

    /// 准备这次调用要交的：`method` 是这次用的方法（`canonicalize`、`health` 没有），`platform` 是这次调用的平台。
    /// 凭据只交方法在这个平台上列了的站点（[`login_sites`]），访客身份只交方法列了的。读钥匙串，会阻塞。
    pub fn handover(
        &self,
        plugin: &FetchPlugin,
        method: Option<&Method>,
        platform: Option<&str>,
    ) -> Result<Handover, PluginError> {
        let index = self.read_index()?;
        let now = now_secs();
        let mut sites = Vec::new();
        if let Some(m) = method {
            for name in login_sites(m, platform, &plugin.spec.platforms) {
                let Some(site) = plugin.spec.sites.iter().find(|s| &s.site == name) else {
                    continue;
                };
                // 只有 `cookies` 一种；`browser-profile`、`app` 宿主现在什么都不传。
                if site.credential != Credential::Cookies
                    || !index.sites.contains_key(&site.site)
                    || sites
                        .iter()
                        .any(|(s, _, _): &(Site, StorageState, LoginSource)| s.site == site.site)
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
                    sites.push((site.clone(), kept, index.sites[&site.site].source));
                }
            }
        }
        // 访客身份和 `login` 无关：方法列了、存着，就交。只读 `visitor:` 那份，不碰账号。
        let mut visitors: Vec<(Visitor, StorageState, Option<String>)> = Vec::new();
        for name in method.map(|m| m.visitors.as_slice()).unwrap_or_default() {
            let Some(v) = plugin.spec.visitors.iter().find(|v| &v.site == name) else {
                continue;
            };
            if !index.visitors.contains_key(&v.site)
                || visitors.iter().any(|(x, _, _)| x.site == v.site)
            {
                continue;
            }
            let Some(json) = self.vault()?.read(&visitor_key(&v.site))? else {
                continue;
            };
            let state: StorageState = serde_json::from_str(&json).map_err(|e| {
                PluginError::Vault(format!(
                    "stored visitor identity for {} is not readable ({:?} error)",
                    v.site,
                    e.classify()
                ))
            })?;
            let kept = restrict(v, &state, now);
            if !kept.cookies.is_empty() {
                let ua = index.visitors[&v.site].user_agent.clone();
                visitors.push((v.clone(), kept, ua));
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
        if sites.is_empty() && visitors.is_empty() && secrets.is_empty() {
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
        // 交出去的 cookie 的名字（不含值）：调用失败时记进失败日志
        let mut cookie_names: Vec<String> = sites
            .iter()
            .flat_map(|(_, s, _)| s.cookies.iter())
            .chain(visitors.iter().flat_map(|(_, s, _)| s.cookies.iter()))
            .map(|c| c.name.clone())
            .collect();
        cookie_names.sort();
        cookie_names.dedup();
        if !sites.is_empty() {
            let mut all = StorageState::default();
            for (_, s, _) in &sites {
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
                    .map(|(s, _, from)| CredentialSite {
                        site: s.site.clone(),
                        source: from.as_str().into(),
                        user_agent: None,
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
        if !visitors.is_empty() {
            let mut cookies = Vec::new();
            for (_, s, _) in &visitors {
                cookies.extend(s.cookies.iter().cloned());
            }
            values.extend(cookies.iter().map(|c| c.value.clone()));
            let file = Credentials {
                sites: visitors
                    .iter()
                    .map(|(v, _, ua)| CredentialSite {
                        site: v.site.clone(),
                        source: "visitor-window".into(),
                        user_agent: ua.clone(),
                    })
                    .collect(),
                storage_state: StorageState {
                    cookies,
                    origins: Vec::new(),
                },
            };
            let path = dir.path().join("visitor.json");
            let bytes = serde_json::to_vec(&file).map_err(|e| PluginError::Vault(e.to_string()))?;
            write_private(&path, &bytes)?;
            env.push(("USUYO_VISITOR".into(), path.into()));
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
            sites: sites.into_iter().map(|(s, _, _)| s).collect(),
            cookie_names,
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
            self.store_site(&site.site, &stored, None)?;
        }
        Ok(())
    }
}

const RUN_PREFIX: &str = "usuyo-cred-";

/// 索引里的一条：名字、域名、过期时间，没有值。
fn record(state: &StorageState) -> SiteRecord {
    SiteRecord {
        source: LoginSource::default(),
        user_agent: None,
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
    }
}

/// 索引里记着的 cookie 名字，排好、去掉重复。
fn names_of(r: &SiteRecord) -> Vec<String> {
    let mut names: Vec<String> = r.cookies.iter().map(|c| c.name.clone()).collect();
    names.sort();
    names.dedup();
    names
}

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
        // 原样的，和被 JSON、Debug 转义过的写法（值里有引号、反斜杠、非 ASCII 字符时不一样）
        let json = serde_json::to_string(v).unwrap_or_default();
        let debug = format!("{v:?}");
        let escaped =
            [json.as_str(), debug.as_str()].map(|quoted| quoted.trim_matches('"').to_owned());
        for form in [v.as_str(), escaped[0].as_str(), escaped[1].as_str()] {
            if !form.is_empty() && text.contains(form) {
                *text = text.replace(form, "[redacted]");
            }
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
            source: LoginSource::default(),
            user_agent: None,
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
    fn without_names_one_live_cookie_is_a_login_and_only_all_lapsed_is_expired() {
        let mut unnamed = site(&[]);
        unnamed.cookies.clear();
        let cookie = |name: &str, expires: f64| CookieName {
            name: name.into(),
            domain: ".bilibili.com".into(),
            expires,
        };
        let rec = |cookies: Vec<CookieName>| SiteRecord {
            source: LoginSource::default(),
            user_agent: None,
            captured_at: String::new(),
            cookies,
        };
        // 导出的文件里常有几个很快过期的统计 cookie：它们过期了，交给插件的还有没过期的那些
        let mixed = rec(vec![cookie("session", -1.0), cookie("tracker", 5.0)]);
        assert_eq!(login_of(&unnamed, Some(&mixed), 10.0), LoginState::Present);
        let lapsed = rec(vec![cookie("a", 5.0), cookie("b", 8.0)]);
        assert_eq!(login_of(&unnamed, Some(&lapsed), 10.0), LoginState::Expired);
        // 写了名字的照旧：那几个里有一个过期就算过期
        let named = site(&[]);
        let rec_named = rec(vec![cookie("SESSDATA", 5.0), cookie("other", -1.0)]);
        assert_eq!(
            login_of(&named, Some(&rec_named), 10.0),
            LoginState::Expired
        );
    }

    #[test]
    fn scrub_replaces_values() {
        let mut t = "cookie SESSDATA=abcdef123 failed".to_owned();
        scrub(&mut t, &["abcdef123".into()]);
        assert_eq!(t, "cookie SESSDATA=[redacted] failed");
    }

    #[test]
    fn scrub_matches_escaped_forms() {
        let value = "a\"b\\c\u{4e2d}".to_owned();
        // 解析错误里值是按 Debug 转义出现的，JSON 里是按 JSON 转义出现的
        let mut debug = format!("invalid type: string {value:?}, expected a sequence");
        scrub(&mut debug, std::slice::from_ref(&value));
        assert_eq!(
            debug,
            "invalid type: string \"[redacted]\", expected a sequence"
        );
        let mut json = format!("got {}", serde_json::json!(value));
        scrub(&mut json, std::slice::from_ref(&value));
        assert_eq!(json, "got \"[redacted]\"");
    }
}
