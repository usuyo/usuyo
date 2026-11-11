//! 登录窗口：每个站点一个窗口、自己一份网页数据，和主窗口、别的站点、你日常的浏览器都分开。
//! 上面一条是我们自己的页面（`login-bar.html`，说这是哪个站点、放「登录好了」），下面是站点的网页。
//! 你在网页里自己登录；声明的 cookie 都出现了（没声明名字的等你点「登录好了」），
//! 壳读出这个站点的 cookie（含 HttpOnly），自己交给内核 `PUT /sites/{site}/login`，关窗口。
//! cookie 的值只经过壳和内核：不进前端、不写日志、不落盘。
//!
//! 小红书的访客身份用同一套窗口，不登录：打开首页，等页面自己生成的 cookie 稳定下来，
//! 整行交给内核存成插件的密钥设置。

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
use crate::site_cookies::{self, Jar, SiteSpec};

/// 上面那条我们自己的页面有多高。
const BAR_HEIGHT: f64 = 52.0;
const POLL: Duration = Duration::from_secs(1);
/// 访客 cookie 连着这么多次没变就算页面生成完了。
const VISITOR_SETTLED_POLLS: u32 = 3;
const API_TIMEOUT: Duration = Duration::from_secs(10);

/// 不登录、只取访客身份的插件。插件清单还写不出「不是账号的 cookie」（协议 §13 待定），先列在壳里。
struct Visitor {
    plugin: &'static str,
    key: &'static str,
    name: &'static str,
    url: &'static str,
}

const VISITORS: &[Visitor] = &[Visitor {
    plugin: "usuyo-fetch/xiaohongshu",
    key: "visitor-cookie",
    name: "小红书",
    url: "https://www.xiaohongshu.com/",
}];

/// 窗口在等什么。
#[derive(Clone)]
enum Goal {
    Login(SiteSpec),
    Visitor {
        plugin: String,
        key: String,
        host: String,
    },
}

struct Session {
    window: String,
    /// 你点了「登录好了」。
    pressed: Arc<AtomicBool>,
}

pub struct Logins {
    daemon: Arc<Daemon>,
    /// Windows、Linux 上每个站点的网页数据目录放在这下面（macOS 用数据仓库的标识，不要目录）。
    #[cfg(not(target_vendor = "apple"))]
    data_root: PathBuf,
    /// 站点（或 `visitor:<插件>`）→ 开着的窗口。
    open: Mutex<HashMap<String, Session>>,
    next: AtomicU32,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// 只开网页地址；`http` 只给本机（开发时的测试页）。
fn page_url(raw: &str) -> Result<Url> {
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
    fn api(&self, method: &str, segments: &[&str], body: Option<Value>) -> Result<Value> {
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
        let bar = json!({ "mode": if manual { "manual" } else { "auto" }, "name": spec.name });
        let title = format!("登录{} · usuyo", spec.name);
        self.open_window(app, site.to_owned(), &title, url, bar, Goal::Login(spec))
    }

    /// 开访客身份的窗口：不登录，取到就关。
    pub fn open_visitor(self: &Arc<Self>, app: &AppHandle, plugin: &str) -> Result<()> {
        let v = VISITORS
            .iter()
            .find(|v| v.plugin == plugin)
            .ok_or_else(|| anyhow!("{plugin} has no visitor identity"))?;
        let store = format!("visitor:{plugin}");
        if self.focus(app, &store) {
            return Ok(());
        }
        let url = page_url(v.url)?;
        let host = url.host_str().unwrap_or_default().to_owned();
        let bar = json!({ "mode": "visitor", "name": v.name });
        let title = format!("取{}的访客身份 · usuyo", v.name);
        let goal = Goal::Visitor {
            plugin: v.plugin.to_owned(),
            key: v.key.to_owned(),
            host,
        };
        self.open_window(app, store, &title, url, bar, goal)
    }

    fn focus(&self, app: &AppHandle, store: &str) -> bool {
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

    fn open_window(
        self: &Arc<Self>,
        app: &AppHandle,
        store: String,
        title: &str,
        url: Url,
        bar: Value,
        goal: Goal,
    ) -> Result<()> {
        let n = self.next.fetch_add(1, Ordering::Relaxed);
        let label = format!("login-{n}");
        let (width, height) = (480.0, 780.0);
        let window = WindowBuilder::new(app, &label)
            .title(title)
            .inner_size(width, height)
            .min_inner_size(360.0, 480.0)
            .build()?;
        let bar_view = window.add_child(
            WebviewBuilder::new(
                format!("{label}-bar"),
                WebviewUrl::App("login-bar.html".into()),
            )
            .initialization_script(format!("window.usuyoLogin = {bar};")),
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(width, BAR_HEIGHT),
        )?;
        let site_builder = WebviewBuilder::new(format!("{label}-site"), WebviewUrl::External(url));
        #[cfg(target_vendor = "apple")]
        let site_builder = site_builder.data_store_identifier(site_cookies::store_id(&store));
        #[cfg(not(target_vendor = "apple"))]
        let site_builder =
            site_builder.data_directory(self.data_root.join(site_cookies::store_dir_name(&store)));
        let site_view = window.add_child(
            site_builder,
            LogicalPosition::new(0.0, BAR_HEIGHT),
            LogicalSize::new(width, height - BAR_HEIGHT),
        )?;

        let closed = Arc::new(AtomicBool::new(false));
        {
            let (w, bar_view, site_view, closed) = (
                window.clone(),
                bar_view.clone(),
                site_view.clone(),
                Arc::clone(&closed),
            );
            window.on_window_event(move |e| match e {
                WindowEvent::Resized(size) => {
                    let s = size.to_logical::<f64>(w.scale_factor().unwrap_or(1.0));
                    let _ = bar_view.set_size(LogicalSize::new(s.width, BAR_HEIGHT));
                    let _ = site_view.set_position(LogicalPosition::new(0.0, BAR_HEIGHT));
                    let _ = site_view
                        .set_size(LogicalSize::new(s.width, (s.height - BAR_HEIGHT).max(0.0)));
                }
                WindowEvent::Destroyed => closed.store(true, Ordering::SeqCst),
                _ => {}
            });
        }

        let pressed = Arc::new(AtomicBool::new(false));
        lock(&self.open).insert(
            store.clone(),
            Session {
                window: label.clone(),
                pressed: Arc::clone(&pressed),
            },
        );
        let me = Arc::clone(self);
        std::thread::spawn(move || {
            me.watch(&window, &bar_view, &site_view, &goal, &pressed, &closed);
            let mut open = lock(&me.open);
            if open.get(&store).is_some_and(|s| s.window == label) {
                open.remove(&store);
            }
        });
        Ok(())
    }

    /// 窗口开着时每秒看一次 cookie（不在主线程上读：Windows 上在主线程读会卡死）。
    fn watch(
        &self,
        window: &Window,
        bar: &Webview,
        site: &Webview,
        goal: &Goal,
        pressed: &AtomicBool,
        closed: &AtomicBool,
    ) {
        let mut settle = Settle::default();
        let mut failed_at: Option<std::time::Instant> = None;
        while !closed.load(Ordering::SeqCst) {
            std::thread::sleep(POLL);
            if closed.load(Ordering::SeqCst) {
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
                            say(bar, "还没看到这个站点的登录。登录好了再点一次。");
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
                    say(bar, "看到登录了，正在交给 usuyo……");
                    self.api(
                        "PUT",
                        &["sites", &spec.site, "login"],
                        Some(json!({ "cookies": selected })),
                    )
                }
                Goal::Visitor { plugin, key, host } => {
                    let selected = site_cookies::select(host, &[], all, now_secs());
                    if !settle.see(&selected) {
                        continue;
                    }
                    let mut path: Vec<&str> = vec!["plugins"];
                    path.extend(plugin.split('/'));
                    path.extend(["secrets", key.as_str()]);
                    self.api(
                        "PUT",
                        &path,
                        Some(json!({ "value": site_cookies::cookie_line(&selected) })),
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
                    say(bar, "没能交给 usuyo 的后台，过一会儿自己再试。");
                    failed_at = Some(std::time::Instant::now());
                }
            }
        }
    }

    /// 上面那条里的「登录好了」。
    pub fn pressed(&self, bar_label: &str) {
        let window = bar_label.strip_suffix("-bar").unwrap_or(bar_label);
        if let Some(s) = lock(&self.open).values().find(|s| s.window == window) {
            s.pressed.store(true, Ordering::SeqCst);
        }
    }

    /// 退出登录以后：关掉这个站点开着的窗口，清掉它那份网页数据，下次登录从头来。
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

/// 在上面那条里说一句。
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
    fn only_web_pages_open_and_plain_http_only_on_this_machine() {
        assert!(page_url("https://passport.bilibili.com/login").is_ok());
        assert!(page_url("http://127.0.0.1:8080/login").is_ok());
        assert!(page_url("http://passport.bilibili.com/login").is_err());
        assert!(page_url("file:///etc/passwd").is_err());
        assert!(page_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn the_visitor_table_only_opens_https() {
        for v in VISITORS {
            assert!(page_url(v.url).unwrap().scheme() == "https", "{}", v.plugin);
        }
    }
}
