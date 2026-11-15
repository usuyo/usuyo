//! 站点窗口往壳交东西的那一条：只进不出。
//!
//! 站点窗口里跑着站点自己的代码，插件的脚本和它在同一个页面里，所以这一条照不可信的来处理：
//! - 只有一条命令 `plugin:usuyo-window|submit`，做成应用自带的插件（`build.rs`），它的权限
//!   `usuyo-window:allow-submit` 只在开窗口时给那一块网页、只给插件清单写的站点（[`grant`]）。
//!   远程页面本来就够不着应用的命令（Tauri 2 只认带 `remote` 的权限），主窗口的那些命令一个也不放开。
//! - 交来的是一份报告：记下的链接和几个计数，形状、条数、长度都有上限（[`Report::parse`]），
//!   不合的整份丢掉；命令什么都不回，页面拿不到 usuyo 的任何东西（令牌、cookie、别的窗口）。
//! - 链接还要合插件的 `match`，由内核在落成导入时再筛一遍。

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;
use tauri::ipc::CapabilityBuilder;
use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime, Url};

/// 应用自带的插件名，和 `build.rs` 里的一致。
pub const PLUGIN: &str = "usuyo-window";
const PERMISSION: &str = "usuyo-window:allow-submit";

/// 一份报告最多多大（序列化以后）。
const REPORT_BYTES: usize = 512 << 10;
/// 一份报告里最多几条链接；脚本记下一页就交一回，一页远少于这个数。
const REPORT_LINKS: usize = 200;
const URL_BYTES: usize = 2048;
const TITLE_CHARS: usize = 200;
const DIAGNOSIS_BYTES: usize = 256 << 10;
const SKIP_KINDS: usize = 16;
/// 能交的来源，和内核 `/v1/batches` 认的 `action` 一样。
const ACTIONS: [&str; 5] = ["favorite", "like", "share", "view", "unknown"];

/// 窗口能去的地方：站点本身，和清单 `domains` 里的（含子域）。
#[derive(Debug, Clone)]
pub struct Allowed {
    site: String,
    domains: Vec<String>,
}

fn loopback(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "[::1]")
}

impl Allowed {
    pub fn new(site: &str, domains: &[String]) -> Allowed {
        Allowed {
            site: site.to_ascii_lowercase(),
            domains: domains
                .iter()
                .map(|d| d.trim_start_matches('.').to_ascii_lowercase())
                .filter(|d| !d.is_empty())
                .collect(),
        }
    }

    fn host(&self, host: &str) -> bool {
        let h = host.to_ascii_lowercase();
        h == self.site
            || self
                .domains
                .iter()
                .any(|d| h == *d || h.ends_with(&format!(".{d}")))
    }

    /// 窗口里的页面（和页面里的小框）能不能打开这一页：https 的站点页面；http 只给本机的测试站点；
    /// 空白页照放。别的协议（`file:`、`data:`、`javascript:`、`blob:`、App 的链接）和别的站点都不放。
    pub fn page(&self, url: &Url) -> bool {
        match url.scheme() {
            "about" => matches!(url.path(), "blank" | "srcdoc"),
            "https" => url.host_str().is_some_and(|h| self.host(h)),
            "http" => url.host_str().is_some_and(|h| loopback(h) && self.host(h)),
            _ => false,
        }
    }

    /// 给 Tauri 权限用的地址式样：站点本身和 `domains`（含子域）；本机测试站点不限端口。
    fn patterns(&self) -> Vec<String> {
        let scheme = if loopback(&self.site) {
            "http"
        } else {
            "https"
        };
        let port = if loopback(&self.site) { ":*" } else { "" };
        let mut out = vec![format!("{scheme}://{}{port}/*", self.site)];
        for d in &self.domains {
            out.push(format!("{scheme}://{d}{port}/*"));
            out.push(format!("{scheme}://*.{d}{port}/*"));
        }
        out.sort();
        out.dedup();
        out
    }
}

/// 交链接的那条命令只给这一块网页、只在这几个站点上用：开窗口时加一条运行时的权限。
/// 网页的标签每个窗口都不一样，窗口关了以后这条权限不会再碰到别的网页。
pub fn grant<R: Runtime, M: Manager<R>>(
    app: &M,
    webview: &str,
    allowed: &Allowed,
) -> tauri::Result<()> {
    let mut cap = CapabilityBuilder::new(format!("reading-{webview}"))
        .local(false)
        .webview(webview)
        .permission(PERMISSION);
    for p in allowed.patterns() {
        cap = cap.remote(p);
    }
    app.add_capability(cap)
}

/// 一条记下的链接。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Noted {
    pub action: String,
    pub url: String,
    pub title: Option<String>,
}

/// 脚本报的计数：只是数，给面板看。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counts {
    pub requests: u32,
    pub recognised: u32,
    pub skipped: BTreeMap<String, u32>,
}

/// 脚本说读到哪了。`end`：列表到底了；`stopped`：停下了，`reason` 说为什么（固定的几个词）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Said {
    Reading,
    End,
    Stopped,
}

/// 一份合格的报告。
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub links: Vec<Noted>,
    pub counts: Option<Counts>,
    pub said: Option<Said>,
    pub reason: Option<String>,
    pub signed_in: bool,
    pub diagnosis: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WireReport {
    #[serde(default)]
    links: Vec<WireLink>,
    #[serde(default)]
    counts: Option<WireCounts>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    signed_in: bool,
    #[serde(default)]
    diagnosis: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireLink {
    action: String,
    url: String,
    #[serde(default)]
    title: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireCounts {
    requests: u32,
    recognised: u32,
    #[serde(default)]
    skipped: BTreeMap<String, u32>,
}

/// 固定的几个词：小写字母开头，只有字母，最长 25 个。
fn word(w: &str) -> bool {
    let mut chars = w.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && w.len() <= 25
        && chars.all(|c| c.is_ascii_alphabetic())
}

impl Report {
    /// 照上限查一份报告；不合的整份不要，回一句给日志（不带页面给的内容）。
    pub fn parse(body: &Value, allowed: &Allowed) -> Result<Report, &'static str> {
        let size = serde_json::to_vec(body).map_or(usize::MAX, |b| b.len());
        if size > REPORT_BYTES {
            return Err("report too large");
        }
        let report = body.get("report").ok_or("no report")?;
        let w: WireReport =
            serde_json::from_value(report.clone()).map_err(|_| "report has the wrong shape")?;
        if w.links.len() > REPORT_LINKS {
            return Err("too many links in one report");
        }
        let mut links = Vec::with_capacity(w.links.len());
        for l in w.links {
            if !ACTIONS.contains(&l.action.as_str()) {
                return Err("unknown action");
            }
            if l.url.len() > URL_BYTES {
                return Err("link too long");
            }
            let url = Url::parse(&l.url).map_err(|_| "not a link")?;
            let web = matches!(url.scheme(), "https" | "http");
            if !web || !allowed.page(&url) || !url.username().is_empty() || url.password().is_some()
            {
                return Err("link outside the site");
            }
            let title = l
                .title
                .map(|t| {
                    t.chars()
                        .filter(|c| !c.is_control())
                        .take(TITLE_CHARS)
                        .collect::<String>()
                })
                .filter(|t| !t.trim().is_empty());
            links.push(Noted {
                action: l.action,
                url: url.to_string(),
                title,
            });
        }
        let counts = match w.counts {
            None => None,
            Some(c) => {
                if c.skipped.len() > SKIP_KINDS || !c.skipped.keys().all(|k| word(k)) {
                    return Err("bad skip kinds");
                }
                Some(Counts {
                    requests: c.requests,
                    recognised: c.recognised,
                    skipped: c.skipped,
                })
            }
        };
        let said = match w.state.as_deref() {
            None => None,
            Some("reading") => Some(Said::Reading),
            Some("end") => Some(Said::End),
            Some("stopped") => Some(Said::Stopped),
            Some(_) => return Err("unknown state"),
        };
        if w.reason.as_deref().is_some_and(|r| !word(r)) {
            return Err("bad reason");
        }
        if w.diagnosis
            .as_ref()
            .is_some_and(|d| d.len() > DIAGNOSIS_BYTES)
        {
            return Err("diagnosis too large");
        }
        Ok(Report {
            links,
            counts,
            said,
            reason: w.reason,
            signed_in: w.signed_in,
            diagnosis: w.diagnosis,
        })
    }
}

/// 页面里那条命令。什么都不回：合格的交给读的那个窗口，不合格的只进日志。
#[tauri::command]
fn submit<R: Runtime>(
    webview: tauri::Webview<R>,
    request: tauri::ipc::Request<'_>,
    reads: tauri::State<'_, std::sync::Arc<crate::reading::Reads>>,
) {
    let tauri::ipc::InvokeBody::Json(body) = request.body() else {
        eprintln!("usuyo: a site window sent a report that is not JSON");
        return;
    };
    reads.report(webview.label(), body);
}

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new(PLUGIN)
        .invoke_handler(tauri::generate_handler![submit])
        .build()
}

/// 页面里跑的那一段：先认地址，再把插件的脚本（一个接 `usuyo` 的函数）叫起来。
/// `usuyo.submit(report)` 交一份报告，`usuyo.onStep(fn)` 在该往下翻一屏时叫 `fn`，
/// `usuyo.onResume(fn)` 在停下以后点了「继续」时叫 `fn`。
pub fn page_script(allowed: &Allowed, plugin_script: &str, lang: &str) -> String {
    let hosts = serde_json::json!({ "site": allowed.site, "domains": allowed.domains });
    let lang = serde_json::to_string(lang).unwrap_or_else(|_| "\"zh-Hans\"".into());
    format!(
        r#"(function () {{
  'use strict';
  var hosts = {hosts};
  var h = String(location.hostname).toLowerCase();
  var ok = h === hosts.site || hosts.domains.some(function (d) {{ return h === d || h.slice(-d.length - 1) === '.' + d; }});
  if (!ok || window.top !== window) return;
  var ipc = window.__TAURI_INTERNALS__;
  if (!ipc || typeof ipc.invoke !== 'function') return;
  var invoke = ipc.invoke.bind(ipc);
  var usuyo = Object.freeze({{
    lang: {lang},
    submit: function (report) {{ return invoke('plugin:{PLUGIN}|submit', {{ report: report }}).then(function () {{}}, function () {{}}); }},
    onStep: function (fn) {{ window.addEventListener('usuyo:step', function () {{ fn(); }}); }},
    onResume: function (fn) {{ window.addEventListener('usuyo:resume', function () {{ fn(); }}); }}
  }});
  ({plugin_script})(usuyo);
}})();"#
    )
}

/// 叫页面往下翻一屏（脚本自己定怎么翻）。
pub const STEP: &str = "window.dispatchEvent(new Event('usuyo:step'));";

/// 叫页面里的脚本从停下的地方接着读。
pub const RESUME: &str = "window.dispatchEvent(new Event('usuyo:resume'));";

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn xhs() -> Allowed {
        Allowed::new("www.example.test", &[".example.test".into()])
    }

    fn url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn the_window_stays_on_the_declared_site() {
        let a = xhs();
        assert!(a.page(&url("https://www.example.test/explore")));
        assert!(a.page(&url("https://edith.example.test/api")));
        assert!(a.page(&url("https://example.test/")));
        assert!(a.page(&url("about:blank")));
        for bad in [
            "http://www.example.test/",
            "https://example.test.evil.test/",
            "https://evilexample.test/",
            "https://other.test/",
            "file:///etc/passwd",
            "data:text/html,x",
            "javascript:alert(1)",
            "blob:https://www.example.test/1",
            "itms-apps://x",
            "about:config",
        ] {
            assert!(!a.page(&url(bad)), "{bad}");
        }
        let local = Allowed::new("127.0.0.1", &[]);
        assert!(local.page(&url("http://127.0.0.1:8123/login")));
        assert!(!local.page(&url("http://localhost:8123/login")));
        assert_eq!(local.patterns(), ["http://127.0.0.1:*/*"]);
        assert_eq!(
            a.patterns(),
            [
                "https://*.example.test/*",
                "https://example.test/*",
                "https://www.example.test/*"
            ]
        );
    }

    fn link(action: &str, u: &str) -> Value {
        json!({"action": action, "url": u, "title": "t"})
    }

    #[test]
    fn a_report_is_bounded_and_checked() {
        let a = xhs();
        let ok = json!({"report": {
            "links": [link("favorite", "https://www.example.test/explore/1?xsec_token=a")],
            "counts": {"requests": 3, "recognised": 1, "skipped": {"noToken": 2}},
            "state": "end", "signedIn": true, "diagnosis": "{}"
        }});
        let r = Report::parse(&ok, &a).unwrap();
        assert_eq!(r.links.len(), 1);
        assert_eq!(r.said, Some(Said::End));
        assert!(r.signed_in);
        assert_eq!(r.counts.unwrap().skipped["noToken"], 2);

        let bad = |report: Value| Report::parse(&json!({ "report": report }), &a).is_err();
        assert!(bad(
            json!({"links": [link("favorite", "https://other.test/explore/1")]})
        ));
        assert!(bad(
            json!({"links": [link("favorite", "javascript:alert(1)")]})
        ));
        assert!(bad(
            json!({"links": [link("steal", "https://www.example.test/x")]})
        ));
        assert!(bad(
            json!({"links": [link("like", &format!("https://www.example.test/{}", "a".repeat(3000)))]})
        ));
        let many: Vec<Value> = (0..201)
            .map(|i| link("like", &format!("https://www.example.test/explore/{i}")))
            .collect();
        assert!(bad(json!({ "links": many })));
        assert!(bad(json!({"state": "exfiltrate"})));
        assert!(bad(json!({"reason": "两个字"})));
        assert!(bad(json!({"token": "x"})), "unknown fields are refused");
        assert!(bad(json!({"diagnosis": "x".repeat(300 << 10)})));
        assert!(Report::parse(&json!({"other": 1}), &a).is_err());
        // 标题截短、去掉控制字符
        let long = json!({"report": {"links": [{"action": "like", "url": "https://www.example.test/n/1", "title": format!("a\u{0}{}", "字".repeat(500))}]}});
        let t = Report::parse(&long, &a).unwrap().links[0]
            .title
            .clone()
            .unwrap();
        assert_eq!(t.chars().count(), TITLE_CHARS);
        assert!(!t.contains('\u{0}'));
    }

    /// 一个应用自己的命令，代表主窗口那些：站点网页够不着它。
    #[tauri::command]
    fn ping() -> &'static str {
        "pong"
    }

    fn invoke(
        webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        url: &str,
    ) -> Result<(), Value> {
        tauri::test::get_ipc_response(
            webview,
            tauri::webview::InvokeRequest {
                cmd: cmd.into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: url.parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(json!({"report": {"links": []}})),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        )
        .map(|_| ())
    }

    /// 交链接的那条只给开窗口时点名的那块网页、只在清单的站点上；应用自己的命令在站点上一条也够不着。
    #[test]
    fn only_the_granted_site_webview_reaches_submit_and_nothing_else() {
        let app = tauri::test::mock_builder()
            .plugin(plugin())
            .manage(std::sync::Arc::new(crate::reading::Reads::default()))
            .invoke_handler(tauri::generate_handler![ping])
            .build(tauri::generate_context!(
                test = true,
                assets = tauri::test::noop_assets()
            ))
            .unwrap();
        let open = |label: &str| {
            tauri::WebviewWindowBuilder::new(&app, label, tauri::WebviewUrl::default())
                .build()
                .unwrap()
        };
        let site = open("login-1-site");
        let other = open("login-2-site");
        grant(&app, "login-1-site", &xhs()).unwrap();
        let submit = "plugin:usuyo-window|submit";

        assert!(invoke(&site, submit, "https://www.example.test/explore").is_ok());
        assert!(invoke(&site, submit, "https://edith.example.test/x").is_ok());
        assert!(invoke(&site, submit, "https://other.test/").is_err());
        assert!(invoke(&site, submit, "http://www.example.test/").is_err());
        assert!(invoke(&other, submit, "https://www.example.test/explore").is_err());
        assert!(invoke(&site, "ping", "https://www.example.test/explore").is_err());
        // 本机的页面（主窗口那种）照旧用得上应用的命令
        assert!(invoke(&other, "ping", "tauri://localhost").is_ok());
    }

    #[test]
    fn the_page_script_checks_the_host_before_calling_the_plugin_script() {
        let s = page_script(&xhs(), "(function (u) { u.submit({}); })", "en");
        assert!(s.contains("plugin:usuyo-window|submit"));
        assert!(s.contains(r#""site":"www.example.test""#));
        assert!(s.contains("window.top !== window"));
        assert!(s.ends_with("})();"));
    }
}
