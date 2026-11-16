//! 登录窗口：每个站点一个窗口、自己一份网页数据，和主窗口、别的站点、你日常的浏览器都分开。
//! 上面一条是我们自己的页面（`login-bar.html`，说这是哪个站点、放「登录好了」），下面是站点的网页。
//! 你在网页里自己登录；声明的 cookie 都出现了（没声明名字的等你点「登录好了」），
//! 壳读出这个站点的 cookie（含 HttpOnly），自己交给内核 `PUT /sites/{site}/login`，关窗口。
//! cookie 的值只经过壳和内核：不进前端、不写日志、不落盘。
//!
//! 访客身份（插件清单的 `visitors`，不是账号）用同一套窗口，不登录：打开清单写的那一页，
//! 等声明的 cookie 都出现（没声明名字的等名字不再变），连同这个窗口的 User-Agent（macOS 上是壳给窗口定的那个，
//! 别的系统上是上面那条自己的页面报来的 webview 自己的）交给内核 `PUT /visitors/{site}`；插件请求时用同一个 UA。
//! 看不见的访客窗口到点没取到，把页面里有的 cookie 名字（不含值）交给内核 `POST /visitors/{site}/missed`。
//! 访客身份的网页数据是 `visitor:<站点>` 那一份，和同一主机的账号分开。
//! 内核要一份访客身份时（`GET /visitors/wanted`），壳自己开一个看不见的访客窗口去取，取不到一分钟后关掉。
//!
//! 在站点窗口里读的方法（`browser-profile` 站点）也用这里的窗口和网页数据，见 `reading.rs`。

use crate::words::{self, Lang};
use std::collections::HashMap;
#[cfg(not(target_vendor = "apple"))]
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use serde_json::{Value, json};
use tauri::webview::{Cookie, WebviewBuilder, cookie::SameSite};
use tauri::window::WindowBuilder;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, Url, Webview, WebviewUrl, Window, WindowEvent,
};

use crate::daemon::Daemon;
use crate::site_cookies::{self, Jar, SiteSpec, VisitorSpec};

/// 上面那条我们自己的页面有多高。
const BAR_HEIGHT: f64 = 52.0;
const POLL: Duration = Duration::from_secs(1);
/// 访客 cookie 连着这么多次没变就算页面生成完了。
const VISITOR_SETTLED_POLLS: u32 = 3;
const API_TIMEOUT: Duration = Duration::from_secs(10);
/// 看不见的访客窗口最多开这么久；取不到就关掉，内核到点照常再试。
const HIDDEN_VISITOR_LIMIT: Duration = Duration::from_secs(60);
/// 内核要的访客身份，隔多久问一次；同一个站点取过以后，这么久里不再自己去取。
const WANTED_POLL: Duration = Duration::from_secs(2);
const WANTED_RETRY: Duration = Duration::from_secs(120);

/// 窗口在等什么。
#[derive(Clone)]
enum Goal {
    Login(SiteSpec),
    /// `hidden`：内核要的，窗口不显示，过了 [`HIDDEN_VISITOR_LIMIT`] 还没取到就关掉。
    Visitor {
        spec: VisitorSpec,
        hidden: bool,
    },
}

struct Session {
    window: String,
    /// 你点了「登录好了」。
    pressed: Arc<AtomicBool>,
    /// 上面那条自己的页面报来的 User-Agent（`navigator.userAgent`）。
    agent: Option<String>,
}

pub struct Logins {
    pub(crate) daemon: Arc<Daemon>,
    /// Windows、Linux 上每个站点的网页数据目录放在这下面（macOS 用数据仓库的标识，不要目录）。
    #[cfg(not(target_vendor = "apple"))]
    data_root: PathBuf,
    /// 网页数据名（站点、`visitor:<站点>` 或 `profile:<插件>:<站点>`）→ 开着的窗口。
    open: Mutex<HashMap<String, Session>>,
    next: AtomicU32,
}

pub(crate) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// 访客身份那个窗口的网页数据名，和同一主机的登录窗口（就叫站点名）分开。
pub fn visitor_store(site: &str) -> String {
    format!("visitor:{site}")
}

/// 站点窗口里读（`browser-profile`）那个窗口的网页数据名：一个插件在一个站点一份，
/// 和账号登录窗口（站点名）、访客窗口、同一站点别的插件的都分开。从不读出 cookie 交给插件。
pub fn profile_store(plugin: &str, site: &str) -> String {
    format!("profile:{plugin}:{site}")
}

/// macOS 上 Safari 的 UA：系统和 WebKit 两段 Safari 自己就是冻住的这两个数，只有 `Version/` 跟着 Safari 变。
fn safari_agent(version: &str) -> String {
    format!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/{version} Safari/605.1.15"
    )
}

/// 这台电脑上 Safari 的版本（`CFBundleShortVersionString`），只认数字和点。
#[cfg(target_vendor = "apple")]
fn safari_version() -> Option<String> {
    let path = "/Applications/Safari.app/Contents/Info.plist";
    let v = match plist::Value::from_file(path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("usuyo: cannot read the Safari version from {path}: {e}");
            return None;
        }
    };
    let version = v
        .as_dictionary()
        .and_then(|d| d.get("CFBundleShortVersionString"))
        .and_then(plist::Value::as_string)?;
    let ok = !version.is_empty() && version.chars().all(|c| c.is_ascii_digit() || c == '.');
    ok.then(|| version.to_owned())
}

/// 登录窗口、访客窗口、站点窗口（上面那条和下面的网页一样）用的 UA。macOS 上 WKWebView 自己的那个
/// 没有 `Version/… Safari/…` 一截，站点常当它是 App 里的网页，所以照这台电脑的 Safari 写完整的；
/// 读不出 Safari 的版本、别的系统上，用 webview 自己的（`None`）。访客身份随上面那条报来的 UA 交给内核，
/// 两块用的是同一个，插件拿到的就是网页实际用的那个。
pub(crate) fn window_agent() -> Option<&'static str> {
    static AGENT: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    AGENT
        .get_or_init(|| {
            #[cfg(target_vendor = "apple")]
            {
                safari_version().map(|v| safari_agent(&v))
            }
            #[cfg(not(target_vendor = "apple"))]
            {
                None
            }
        })
        .as_deref()
}

/// 换上 [`window_agent`]（有的话）。
pub(crate) fn with_agent(builder: WebviewBuilder<tauri::Wry>) -> WebviewBuilder<tauri::Wry> {
    match window_agent() {
        Some(ua) => builder.user_agent(ua),
        None => builder,
    }
}

/// 只开网页地址；`http` 只给本机（开发时的测试页）。
pub(crate) fn page_url(raw: &str) -> Result<Url> {
    let url = Url::parse(raw).with_context(|| format!("bad login url {raw}"))?;
    let local = matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
    match url.scheme() {
        "https" => Ok(url),
        "http" if local => Ok(url),
        _ => bail!("login url must be https: {raw}"),
    }
}

fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64())
}

/// 网页给的 cookie 换成交给内核的形状。没有域名的读不出属于谁，不要。
fn jar(c: &Cookie<'static>) -> Option<Jar> {
    Some(Jar {
        name: c.name().to_owned(),
        value: c.value().to_owned(),
        domain: c.domain()?.to_owned(),
        path: c.path().unwrap_or("/").to_owned(),
        expires: c
            .expires_datetime()
            .map_or(-1.0, |t| t.unix_timestamp() as f64),
        http_only: c.http_only().unwrap_or(false),
        secure: c.secure().unwrap_or(false),
        same_site: c.same_site().map(|s| match s {
            SameSite::Strict => "Strict",
            SameSite::Lax => "Lax",
            SameSite::None => "None",
        }),
    })
}

/// 访客 cookie 什么时候算生成完了：有了，而且名字连着几次没变。
#[derive(Default)]
struct Settle {
    names: Vec<String>,
    same: u32,
}

impl Settle {
    fn see(&mut self, selected: &[Jar]) -> bool {
        let mut names: Vec<String> = selected.iter().map(|c| c.name.clone()).collect();
        names.sort();
        if names.is_empty() || names != self.names {
            self.names = names;
            self.same = 0;
            return false;
        }
        self.same += 1;
        self.same >= VISITOR_SETTLED_POLLS
    }
}

impl Logins {
    pub fn new(daemon: Arc<Daemon>) -> Logins {
        Logins {
            #[cfg(not(target_vendor = "apple"))]
            data_root: daemon.machine_dir().join("webviews"),
            daemon,
            open: Mutex::new(HashMap::new()),
            next: AtomicU32::new(1),
        }
    }

    /// 带令牌找内核。回 2xx 的 JSON（204 是 `Null`）；别的状态说清楚是哪一条。
    pub(crate) fn api(
        &self,
        method: &str,
        segments: &[&str],
        body: Option<Value>,
    ) -> Result<Value> {
        let c = self.daemon.connection().map_err(|e| anyhow!(e))?;
        let mut url = Url::parse(&c.base)?;
        url.path_segments_mut()
            .map_err(|()| anyhow!("daemon base cannot take a path"))?
            .extend(segments);
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(API_TIMEOUT))
            .http_status_as_error(false)
            .proxy(None)
            .build()
            .into();
        let auth = format!("Bearer {}", c.token);
        let mut res = match (method, body) {
            ("GET", _) => agent
                .get(url.as_str())
                .header("Authorization", &auth)
                .call(),
            ("PUT", Some(b)) => agent
                .put(url.as_str())
                .header("Authorization", &auth)
                .header("Content-Type", "application/json")
                .send(serde_json::to_vec(&b)?),
            ("POST", Some(b)) => agent
                .post(url.as_str())
                .header("Authorization", &auth)
                .header("Content-Type", "application/json")
                .send(serde_json::to_vec(&b)?),
            ("POST", None) => agent
                .post(url.as_str())
                .header("Authorization", &auth)
                .send_empty(),
            ("DELETE", _) => agent
                .delete(url.as_str())
                .header("Authorization", &auth)
                .call(),
            _ => bail!("unsupported daemon call {method}"),
        }
        .with_context(|| format!("{method} {}", url.path()))?;
        let status = res.status().as_u16();
        if !(200..300).contains(&status) {
            // 内核回的错误里没有 cookie 的值（它只回名字和原因）。
            let text = res.body_mut().read_to_string().unwrap_or_default();
            bail!("{method} {} answered {status}: {text}", url.path());
        }
        if status == 204 {
            return Ok(Value::Null);
        }
        Ok(serde_json::from_str(&res.body_mut().read_to_string()?)?)
    }

    fn site_spec(&self, site: &str) -> Result<SiteSpec> {
        let sites: Vec<SiteSpec> = serde_json::from_value(self.api("GET", &["sites"], None)?)?;
        sites
            .into_iter()
            .find(|s| s.site == site)
            .ok_or_else(|| anyhow!("no installed plugin uses {site}"))
    }

    /// 开（或叫到前面）一个站点的登录窗口。
    pub fn open_login(self: &Arc<Self>, app: &AppHandle, site: &str) -> Result<()> {
        if self.focus(app, site) {
            return Ok(());
        }
        let spec = self.site_spec(site)?;
        let url = page_url(
            spec.login_url
                .as_deref()
                .ok_or_else(|| anyhow!("{site} has no login page"))?,
        )?;
        let manual = spec.cookies.is_empty();
        let lang = Lang::of(self.daemon.machine_dir());
        let bar = json!({ "mode": if manual { "manual" } else { "auto" }, "name": spec.name, "lang": lang.tag() });
        let title = words::LOGIN_TITLE.fill(lang, &[("site", &spec.name)]);
        self.open_window(app, site.to_owned(), &title, url, bar, Goal::Login(spec))
    }

    /// 开访客身份的窗口：不登录，取到就关。打开哪一页、等哪些 cookie 由内核照插件清单给。
    /// `hidden`：内核要的（[`Self::watch_wanted`]），窗口不显示，取不到一分钟后关掉。
    pub fn open_visitor(self: &Arc<Self>, app: &AppHandle, site: &str, hidden: bool) -> Result<()> {
        let store = visitor_store(site);
        if hidden && lock(&self.open).contains_key(&store) {
            return Ok(());
        }
        if self.focus(app, &store) {
            return Ok(());
        }
        let visitors: Vec<VisitorSpec> =
            serde_json::from_value(self.api("GET", &["visitors"], None)?)?;
        let v = visitors
            .into_iter()
            .find(|v| v.site == site)
            .ok_or_else(|| anyhow!("no installed plugin uses a visitor identity for {site}"))?;
        let url = page_url(&v.url)?;
        let lang = Lang::of(self.daemon.machine_dir());
        let bar = json!({ "mode": "visitor", "name": v.name, "lang": lang.tag() });
        let title = words::VISITOR_TITLE.fill(lang, &[("site", &v.name)]);
        self.open_window(
            app,
            store,
            &title,
            url,
            bar,
            Goal::Visitor { spec: v, hidden },
        )
    }

    /// 一直问内核要不要访客身份（`GET /visitors/wanted`）；要的话开一个看不见、不登录的访客窗口去取，
    /// 站点拒收了存着的那份时先清掉那份网页数据。同一个站点取过以后两分钟里不再自己去取。
    pub fn watch_wanted(self: &Arc<Self>, app: &AppHandle) {
        let mut tried: HashMap<String, std::time::Instant> = HashMap::new();
        loop {
            std::thread::sleep(WANTED_POLL);
            let wanted = match self.api("GET", &["visitors", "wanted"], None) {
                Ok(v) => v,
                // 内核还没起来、或重启着：下一轮再问
                Err(_) => continue,
            };
            let Some(list) = wanted.as_array() else {
                continue;
            };
            for w in list {
                let (Some(site), replace) = (
                    w.get("site").and_then(Value::as_str),
                    w.get("replace").and_then(Value::as_bool).unwrap_or(false),
                ) else {
                    continue;
                };
                if tried.get(site).is_some_and(|t| t.elapsed() < WANTED_RETRY)
                    || lock(&self.open).contains_key(&visitor_store(site))
                {
                    continue;
                }
                tried.insert(site.to_owned(), std::time::Instant::now());
                if replace
                    && let Err(e) =
                        tauri::async_runtime::block_on(self.clear(app, &visitor_store(site)))
                {
                    eprintln!("usuyo: cannot clear the old visitor identity of {site}: {e:#}");
                    continue;
                }
                if let Err(e) = self.open_visitor(app, site, true) {
                    eprintln!("usuyo: cannot open the visitor window for {site}: {e:#}");
                }
            }
        }
    }

    pub(crate) fn focus(&self, app: &AppHandle, store: &str) -> bool {
        let label = lock(&self.open).get(store).map(|s| s.window.clone());
        match label.and_then(|l| app.get_window(&l)) {
            Some(w) => {
                let _ = w.unminimize();
                let _ = w.set_focus();
                true
            }
            None => false,
        }
    }

    /// 下一个窗口的标签：`login-<n>`，上面那条是 `-bar`，站点的网页是 `-site`。
    pub(crate) fn next_label(&self) -> String {
        format!("login-{}", self.next.fetch_add(1, Ordering::Relaxed))
    }

    /// 站点的网页用这个站点（或 `visitor:<站点>`）自己那份网页数据。
    pub(crate) fn with_store(
        &self,
        builder: WebviewBuilder<tauri::Wry>,
        store: &str,
    ) -> WebviewBuilder<tauri::Wry> {
        #[cfg(target_vendor = "apple")]
        {
            builder.data_store_identifier(site_cookies::store_id(store))
        }
        #[cfg(not(target_vendor = "apple"))]
        {
            builder.data_directory(self.data_root.join(site_cookies::store_dir_name(store)))
        }
    }

    /// 记下这个站点开着的窗口；同一个站点同一时间只有一个。
    pub(crate) fn hold(&self, store: &str, label: &str) -> Arc<AtomicBool> {
        let pressed = Arc::new(AtomicBool::new(false));
        lock(&self.open).insert(
            store.to_owned(),
            Session {
                window: label.to_owned(),
                pressed: Arc::clone(&pressed),
                agent: None,
            },
        );
        pressed
    }

    /// 窗口关了：这个站点不再记着它（换成别的窗口了的不动）。
    pub(crate) fn release(&self, store: &str, label: &str) {
        let mut open = lock(&self.open);
        if open.get(store).is_some_and(|s| s.window == label) {
            open.remove(store);
        }
    }

    fn open_window(
        self: &Arc<Self>,
        app: &AppHandle,
        store: String,
        title: &str,
        url: Url,
        bar: Value,
        goal: Goal,
    ) -> Result<()> {
        let label = self.next_label();
        let hidden = matches!(goal, Goal::Visitor { hidden: true, .. });
        let (width, height) = (480.0, 780.0);
        let window = WindowBuilder::new(app, &label)
            .title(title)
            .inner_size(width, height)
            .min_inner_size(360.0, 480.0)
            .visible(!hidden)
            .focused(!hidden)
            .build()?;
        let bar_view = window.add_child(
            with_agent(
                WebviewBuilder::new(
                    format!("{label}-bar"),
                    WebviewUrl::App("login-bar.html".into()),
                )
                .initialization_script(format!("window.usuyoLogin = {bar};")),
            ),
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(width, BAR_HEIGHT),
        )?;
        let site_label = format!("{label}-site");
        let site_builder = self.with_store(
            with_agent(WebviewBuilder::new(&site_label, WebviewUrl::External(url))),
            &store,
        );
        let site_view = window.add_child(
            site_builder,
            LogicalPosition::new(0.0, BAR_HEIGHT),
            LogicalSize::new(width, height - BAR_HEIGHT),
        )?;
        let closed = follow_size(&window, &bar_view, &site_view, BAR_HEIGHT);
        let pressed = self.hold(&store, &label);
        let me = Arc::clone(self);
        std::thread::spawn(move || {
            me.watch(&bar_view, &site_view, &goal, &pressed, &closed, &store);
            me.release(&store, &label);
        });
        Ok(())
    }

    /// 窗口开着时每秒看一次 cookie（不在主线程上读：Windows 上在主线程读会卡死）。
    fn watch(
        &self,
        bar: &Webview,
        site: &Webview,
        goal: &Goal,
        pressed: &AtomicBool,
        closed: &AtomicBool,
        store: &str,
    ) {
        let window = site.window();
        let mut settle = Settle::default();
        // 访客页面上一次有的 cookie 名字：到点没取到时交给内核看
        let mut seen: Vec<String> = Vec::new();
        let mut failed_at: Option<std::time::Instant> = None;
        let opened = std::time::Instant::now();
        while !closed.load(Ordering::SeqCst) {
            std::thread::sleep(POLL);
            if closed.load(Ordering::SeqCst) {
                return;
            }
            if matches!(goal, Goal::Visitor { hidden: true, .. })
                && opened.elapsed() > HIDDEN_VISITOR_LIMIT
            {
                eprintln!("usuyo: the hidden visitor window got no identity in time; closing it");
                if let Goal::Visitor { spec, .. } = goal
                    && let Err(e) = self.api(
                        "POST",
                        &["visitors", &spec.site, "missed"],
                        Some(json!({ "cookieNames": seen, "userAgent": self.agent_of(store) })),
                    )
                {
                    eprintln!(
                        "usuyo: cannot tell the daemon what the visitor window lacked: {e:#}"
                    );
                }
                let _ = window.destroy();
                return;
            }
            // 交给内核没成：过一会儿再试，不一直打。
            if failed_at.is_some_and(|t| t.elapsed() < Duration::from_secs(10))
                && !pressed.load(Ordering::SeqCst)
            {
                continue;
            }
            let all: Vec<Jar> = match site.cookies() {
                Ok(v) => v.iter().filter_map(jar).collect(),
                Err(e) => {
                    eprintln!("usuyo: cannot read the login window's cookies: {e}");
                    continue;
                }
            };
            let result = match goal {
                Goal::Login(spec) => {
                    let selected = site_cookies::select(&spec.site, &spec.domains, all, now_secs());
                    let ready = if spec.cookies.is_empty() {
                        if !pressed.swap(false, Ordering::SeqCst) {
                            continue;
                        }
                        if selected.is_empty() {
                            say(bar, words::NOT_SIGNED_IN_YET.pick(self.lang()));
                            continue;
                        }
                        true
                    } else {
                        pressed.store(false, Ordering::SeqCst);
                        site_cookies::logged_in(&spec.cookies, &selected)
                    };
                    if !ready {
                        continue;
                    }
                    say(bar, words::SENDING_LOGIN.pick(self.lang()));
                    self.api(
                        "PUT",
                        &["sites", &spec.site, "login"],
                        Some(json!({ "cookies": selected })),
                    )
                }
                Goal::Visitor { spec: v, .. } => {
                    let selected = site_cookies::select(&v.site, &v.domains, all, now_secs());
                    seen = selected.iter().map(|c| c.name.clone()).collect();
                    let ready = if v.cookies.is_empty() {
                        settle.see(&selected)
                    } else {
                        site_cookies::logged_in(&v.cookies, &selected)
                    };
                    if !ready {
                        continue;
                    }
                    let agent = self.agent_of(store);
                    if agent.is_none() {
                        eprintln!(
                            "usuyo: the visitor window's bar has not told its user agent; saving the visitor identity without it"
                        );
                    }
                    self.api(
                        "PUT",
                        &["visitors", &v.site],
                        Some(visitor_body(&selected, agent.as_deref())),
                    )
                }
            };
            match result {
                Ok(_) => {
                    let _ = window.close();
                    return;
                }
                Err(e) => {
                    eprintln!("usuyo: cannot hand the login to the daemon: {e:#}");
                    say(bar, words::SEND_FAILED.pick(self.lang()));
                    failed_at = Some(std::time::Instant::now());
                }
            }
        }
    }

    /// 上面那条自己的页面报来的 User-Agent：和下面的网页同一个引擎、同一个 UA。
    pub fn agent(&self, bar_label: &str, user_agent: String) {
        let Some(window) = bar_label.strip_suffix("-bar") else {
            return;
        };
        if let Some(s) = lock(&self.open).values_mut().find(|s| s.window == window) {
            s.agent = Some(user_agent);
        }
    }

    /// 这个窗口用的 UA：壳自己定了的（[`window_agent`]）就是它；没定时用上面那条报来的 webview 自己的。
    fn agent_of(&self, store: &str) -> Option<String> {
        window_agent()
            .map(str::to_owned)
            .or_else(|| lock(&self.open).get(store).and_then(|s| s.agent.clone()))
    }

    /// 上面那条里的「登录好了」。
    pub fn pressed(&self, bar_label: &str) {
        let window = bar_label.strip_suffix("-bar").unwrap_or(bar_label);
        if let Some(s) = lock(&self.open).values().find(|s| s.window == window) {
            s.pressed.store(true, Ordering::SeqCst);
        }
    }

    /// 退出登录以后：关掉这个站点开着的窗口，清掉它那份网页数据，下次登录从头来。
    /// 访客身份换一份时也用它，`site` 写 `visitor:<站点>`（`visitor_store`）；
    /// 站点窗口里读的方法用 `profile:<插件>:<站点>`（`profile_store`）。
    pub async fn clear(&self, app: &AppHandle, site: &str) -> Result<()> {
        let label = lock(&self.open).remove(site).map(|s| s.window);
        if let Some(w) = label.and_then(|l| app.get_window(&l)) {
            w.destroy()?;
            // 等网页进程放手，再删它的数据。
            tauri::async_runtime::spawn_blocking(|| std::thread::sleep(Duration::from_millis(500)))
                .await?;
        }
        #[cfg(target_vendor = "apple")]
        {
            let id = site_cookies::store_id(site);
            if app.fetch_data_store_identifiers().await?.contains(&id) {
                app.remove_data_store(id).await?;
            }
        }
        #[cfg(not(target_vendor = "apple"))]
        {
            let dir = self.data_root.join(site_cookies::store_dir_name(site));
            match std::fs::remove_dir_all(&dir) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).with_context(|| format!("cannot clear {}", dir.display())),
            }
        }
        Ok(())
    }
}

/// 窗口变大小时，上面那条和下面的网页跟着变；回一个窗口关了以后为真的标记。
pub(crate) fn follow_size(
    window: &Window,
    bar: &Webview,
    site: &Webview,
    bar_height: f64,
) -> Arc<AtomicBool> {
    let closed = Arc::new(AtomicBool::new(false));
    let (w, bar, site, flag) = (
        window.clone(),
        bar.clone(),
        site.clone(),
        Arc::clone(&closed),
    );
    window.on_window_event(move |e| match e {
        WindowEvent::Resized(size) => {
            let s = size.to_logical::<f64>(w.scale_factor().unwrap_or(1.0));
            let _ = bar.set_size(LogicalSize::new(s.width, bar_height));
            let _ = site.set_position(LogicalPosition::new(0.0, bar_height));
            let _ = site.set_size(LogicalSize::new(s.width, (s.height - bar_height).max(0.0)));
        }
        WindowEvent::Destroyed => flag.store(true, Ordering::SeqCst),
        _ => {}
    });
    closed
}

/// 交给内核的访客身份：cookie 和取它的那个窗口的 UA（没报来时不写）。
fn visitor_body(cookies: &[Jar], agent: Option<&str>) -> Value {
    let mut body = json!({ "cookies": cookies });
    if let Some(a) = agent {
        body["userAgent"] = json!(a);
    }
    body
}

/// 在上面那条里说一句。
impl Logins {
    fn lang(&self) -> Lang {
        Lang::of(self.daemon.machine_dir())
    }
}

fn say(bar: &Webview, text: &str) {
    let text = serde_json::to_string(text).unwrap_or_default();
    let _ = bar.eval(format!("window.usuyoSay && window.usuyoSay({text});"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(names: &[&str]) -> Vec<Jar> {
        names
            .iter()
            .map(|n| Jar {
                name: (*n).into(),
                value: "v".into(),
                domain: ".xiaohongshu.com".into(),
                path: "/".into(),
                expires: -1.0,
                http_only: false,
                secure: true,
                same_site: None,
            })
            .collect()
    }

    #[test]
    fn visitor_cookies_count_once_they_stop_changing() {
        let mut s = Settle::default();
        assert!(!s.see(&[]));
        assert!(!s.see(&named(&["a1"])));
        assert!(!s.see(&named(&["a1", "webId"])));
        assert!(!s.see(&named(&["webId", "a1"])));
        assert!(!s.see(&named(&["a1", "webId"])));
        assert!(s.see(&named(&["a1", "webId"])));
    }

    #[test]
    fn the_window_user_agent_is_safari_s_and_never_a_chrome_one() {
        let ua = safari_agent("26.5");
        assert_eq!(
            ua,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.5 Safari/605.1.15"
        );
        assert!(!ua.contains("Chrome"));
        // 这台电脑上读得出 Safari 的版本时，窗口用的就是这一个
        #[cfg(target_vendor = "apple")]
        if let Some(v) = safari_version() {
            assert_eq!(window_agent(), Some(safari_agent(&v).as_str()));
        }
    }

    #[test]
    fn the_visitor_identity_goes_with_the_window_user_agent() {
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.5 Safari/605.1.15";
        let body = visitor_body(&named(&["a1"]), Some(ua));
        assert_eq!(body["userAgent"], ua);
        assert_eq!(body["cookies"][0]["name"], "a1");
        assert!(
            visitor_body(&named(&["a1"]), None)
                .get("userAgent")
                .is_none()
        );
    }

    #[test]
    fn a_visitor_window_never_shares_data_with_the_login_window_of_its_host() {
        let site = "www.example.test";
        assert_eq!(visitor_store(site), "visitor:www.example.test");
        assert_ne!(
            site_cookies::store_id(&visitor_store(site)),
            site_cookies::store_id(site)
        );
    }

    #[test]
    fn a_profile_window_never_shares_data_with_the_login_window_or_another_plugin() {
        let site = "www.example.test";
        let mine = profile_store("owner/mine", site);
        assert_eq!(mine, "profile:owner/mine:www.example.test");
        let id = |s: &str| site_cookies::store_id(s);
        assert_ne!(id(&mine), id(site));
        assert_ne!(id(&mine), id(&visitor_store(site)));
        assert_ne!(id(&mine), id(&profile_store("owner/other", site)));
    }

    #[test]
    fn only_web_pages_open_and_plain_http_only_on_this_machine() {
        assert!(page_url("https://passport.bilibili.com/login").is_ok());
        assert!(page_url("http://127.0.0.1:8080/login").is_ok());
        assert!(page_url("http://passport.bilibili.com/login").is_err());
        assert!(page_url("file:///etc/passwd").is_err());
        assert!(page_url("javascript:alert(1)").is_err());
    }
}
