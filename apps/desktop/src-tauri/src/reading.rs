//! 在 usuyo 的站点窗口里读（插件清单里 `delivers: links`、`account: in-usuyo` 的方法）。
//!
//! 窗口和登录窗口一样：上面一条是我们自己的页面（`login-bar.html`，这里多一块面板），下面是站点的网页，
//! 用这个站点（`browser-profile`）自己那份网页数据，和主窗口、访客身份、你日常的浏览器都分开；cookie 不读出来、不交给内核。
//! 下面的网页打开清单里的 `loginUrl`，页面里跑插件的脚本（[`channel::page_script`]）。你在网页里登录；
//! 脚本认出登录了、记下列表里的链接，经只进不出的那一条（`channel.rs`）交给壳。
//!
//! 默认自己往下翻：隔几秒（带一点随机）叫脚本翻一屏；面板上能换成你自己翻、暂停、继续。
//! 脚本说到底了，壳把记下的链接交给内核，落成普通的导入（在导入页确认）；说停下了（验证页、登录墙、
//! 认不出的回答……），壳不再叫它翻，面板上写原因。你点「完成」或关掉窗口时，记下的也照样交过去。
//!
//! 能去哪些地址只照清单的站点和 `domains`；新窗口、下载、别的协议都挡掉。

use std::collections::{BTreeMap, HashMap};
use std::hash::BuildHasher;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::webview::{NewWindowResponse, WebviewBuilder};
use tauri::window::WindowBuilder;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Url, Webview, WebviewUrl};

use crate::channel::{self, Allowed, Counts, Noted, Report, Said};
use crate::login::{self, Logins, lock};
use crate::words::{self, Lang};

/// 上面那条（带面板）有多高。
const BAR_HEIGHT: f64 = 132.0;
/// 自己往下翻时，两屏之间隔多久：`STEP_MIN` 再加至多 `STEP_JITTER` 的随机。
const STEP_MIN: Duration = Duration::from_millis(4500);
const STEP_JITTER: Duration = Duration::from_millis(3000);
const TICK: Duration = Duration::from_millis(250);
/// 面板最多这么快刷新一次。
const PANEL_EVERY: Duration = Duration::from_millis(300);
/// 一个窗口一回最多记这么多条；再多的不记，面板上写着。
const LINKS_MAX: usize = 20_000;

/// 内核 `GET /plugins/{id}/methods/{method}/window` 给的。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowSpec {
    pub plugin: String,
    pub plugin_name: String,
    pub method: String,
    pub method_name: String,
    pub site: String,
    pub site_name: String,
    pub login_url: String,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub guide: Option<String>,
    pub script: String,
}

/// 面板上的按钮。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Control {
    /// 自己往下翻。
    Auto,
    /// 你自己翻。
    Manual,
    Pause,
    Resume,
    /// 交出记下的。
    Finish,
}

enum Msg {
    Report(Value),
    Control(Control),
}

/// 开着的站点窗口：窗口标签 → 给它那个线程的信。
#[derive(Default)]
pub struct Reads {
    windows: Mutex<HashMap<String, Sender<Msg>>>,
}

impl Reads {
    fn send(&self, window: &str, msg: Msg) {
        if let Some(tx) = lock(&self.windows).get(window) {
            let _ = tx.send(msg);
        }
    }

    /// 站点网页交来的报告（`-site` 那一块）。
    pub fn report(&self, webview: &str, body: &Value) {
        if let Some(window) = webview.strip_suffix("-site") {
            self.send(window, Msg::Report(body.clone()));
        }
    }

    /// 面板上的按钮（`-bar` 那一块）。
    pub fn control(&self, webview: &str, c: Control) {
        if let Some(window) = webview.strip_suffix("-bar") {
            self.send(window, Msg::Control(c));
        }
    }
}

/// 读到哪了。
#[derive(Debug, Clone, PartialEq, Eq)]
enum Phase {
    /// 还没认出登录。
    SigningIn,
    Reading,
    /// 停下了，`reason` 是脚本给的词（或壳自己的 `left`）。
    Stopped(String),
    Handing,
    Handed {
        imports: u32,
        noted: u32,
        dropped: u32,
    },
    /// 交给内核没成；可以再点「完成」。
    Failed,
}

/// 要线程去做的事。
#[derive(Debug, PartialEq, Eq)]
enum Effect {
    SignedIn,
    HandOver,
    /// 叫页面里的脚本接着读（停下以后点了「继续」）。
    Resume,
}

/// 一个站点窗口的状态：只有数和记下的链接，没有 cookie。
struct Session {
    phase: Phase,
    auto: bool,
    paused: bool,
    signed_in: bool,
    links: BTreeMap<String, Noted>,
    /// 超过 [`LINKS_MAX`] 没记的。
    over: u32,
    counts: Counts,
    diagnosis: String,
    /// 点了「继续」、脚本还没报它接着读了：路上那几份还写着停下的报告不算。
    resuming: bool,
}

impl Session {
    fn new() -> Session {
        Session {
            phase: Phase::SigningIn,
            auto: true,
            paused: false,
            signed_in: false,
            links: BTreeMap::new(),
            over: 0,
            counts: Counts::default(),
            diagnosis: String::new(),
            resuming: false,
        }
    }

    fn handed(&self) -> bool {
        matches!(self.phase, Phase::Handing | Phase::Handed { .. })
    }

    fn apply(&mut self, r: Report) -> Vec<Effect> {
        let mut out = Vec::new();
        if r.signed_in && !self.signed_in {
            self.signed_in = true;
            out.push(Effect::SignedIn);
            if self.phase == Phase::SigningIn {
                self.phase = Phase::Reading;
            }
        }
        if !self.handed() {
            for l in r.links {
                if self.links.contains_key(&l.url) {
                    continue;
                }
                if self.links.len() >= LINKS_MAX {
                    self.over = self.over.saturating_add(1);
                    continue;
                }
                self.links.insert(l.url.clone(), l);
            }
        }
        if let Some(c) = r.counts {
            self.counts = c;
        }
        if let Some(d) = r.diagnosis {
            self.diagnosis = d;
        }
        match r.said {
            Some(Said::Reading) => self.resuming = false,
            Some(Said::Stopped) if self.resuming => {}
            Some(Said::End) if !self.handed() => {
                out.push(Effect::HandOver);
            }
            Some(Said::Stopped) if matches!(self.phase, Phase::Reading | Phase::SigningIn) => {
                self.phase = Phase::Stopped(r.reason.unwrap_or_else(|| "unknown".into()));
            }
            _ => {}
        }
        out
    }

    fn control(&mut self, c: Control) -> Vec<Effect> {
        match c {
            Control::Auto => self.auto = true,
            Control::Manual => self.auto = false,
            Control::Pause => self.paused = true,
            Control::Resume => {
                self.paused = false;
                if matches!(self.phase, Phase::Stopped(_)) {
                    self.phase = if self.signed_in {
                        Phase::Reading
                    } else {
                        Phase::SigningIn
                    };
                    self.resuming = true;
                    return vec![Effect::Resume];
                }
            }
            Control::Finish if !self.handed() => return vec![Effect::HandOver],
            Control::Finish => {}
        }
        Vec::new()
    }

    /// 该不该叫脚本往下翻一屏。
    fn steps(&self) -> bool {
        self.phase == Phase::Reading && self.auto && !self.paused
    }

    fn noted(&self, action: &str) -> usize {
        self.links.values().filter(|l| l.action == action).count()
    }

    /// 按来源分好，交给内核的样子。
    fn sources(&self) -> Value {
        let mut by: BTreeMap<&str, Vec<Value>> = BTreeMap::new();
        for l in self.links.values() {
            let mut link = json!({ "url": l.url });
            if let Some(t) = &l.title {
                link["title"] = json!(t);
            }
            by.entry(l.action.as_str()).or_default().push(link);
        }
        let sources: Vec<Value> = by
            .into_iter()
            .map(|(action, links)| json!({ "action": action, "links": links }))
            .collect();
        json!({ "sources": sources })
    }

    /// 给面板的：都是数和固定的词，加上脚本给的形状诊断（不含内容）。
    fn view(&self) -> Value {
        let (phase, reason, handed) = match &self.phase {
            Phase::SigningIn => ("signing-in", None, None),
            Phase::Reading => ("reading", None, None),
            Phase::Stopped(r) => ("stopped", Some(r.as_str()), None),
            Phase::Handing => ("handing", None, None),
            Phase::Handed {
                imports,
                noted,
                dropped,
            } => (
                "handed",
                None,
                Some(json!({ "imports": imports, "noted": noted, "dropped": dropped })),
            ),
            Phase::Failed => ("failed", None, None),
        };
        json!({
            "phase": phase,
            "reason": reason,
            "handed": handed,
            "auto": self.auto,
            "paused": self.paused,
            "requests": self.counts.requests,
            "recognised": self.counts.recognised,
            "noted": { "favorite": self.noted("favorite"), "like": self.noted("like"), "all": self.links.len() },
            "skipped": self.counts.skipped,
            "over": self.over,
            "diagnosis": self.diagnosis,
        })
    }
}

/// 下一屏什么时候翻：至少 [`STEP_MIN`]，再加一点随机。
fn next_step(now: Instant, n: u64) -> Instant {
    let jitter_ms = u64::try_from(STEP_JITTER.as_millis()).unwrap_or(0);
    let r = std::collections::hash_map::RandomState::new().hash_one(n) % jitter_ms.max(1);
    now + STEP_MIN + Duration::from_millis(r)
}

/// `plugin` 是 `owner/repo`，拆成路径的两段。
fn plugin_path(plugin: &str) -> Result<(&str, &str)> {
    plugin
        .split_once('/')
        .ok_or_else(|| anyhow!("bad plugin id {plugin}"))
}

#[derive(Serialize)]
struct BarSetup<'a> {
    mode: &'static str,
    name: &'a str,
    lang: &'static str,
    plugin: &'a str,
    method: &'a str,
    guide: Option<&'a str>,
}

impl Logins {
    /// 开（或叫到前面）一种方法的站点窗口。
    pub fn open_reading(
        self: &Arc<Self>,
        app: &AppHandle,
        reads: &Arc<Reads>,
        plugin: &str,
        method: &str,
    ) -> Result<()> {
        let (owner, repo) = plugin_path(plugin)?;
        let spec: WindowSpec = serde_json::from_value(self.api(
            "GET",
            &["plugins", owner, repo, "methods", method, "window"],
            None,
        )?)
        .context("the window answer has the wrong shape")?;
        let store = login::profile_store(&spec.plugin, &spec.site);
        if self.focus(app, &store) {
            return Ok(());
        }
        let url = login::page_url(&spec.login_url)?;
        let allowed = Allowed::new(&spec.site, &spec.domains);
        let lang = Lang::of(self.daemon.machine_dir());
        let label = self.next_label();
        let site_label = format!("{label}-site");
        // 先给权限再开网页：第一页一载入脚本就要能交
        channel::grant(app, &site_label, &allowed)?;

        let title = words::LOGIN_TITLE.fill(lang, &[("site", &spec.site_name)]);
        let (width, height) = (1180.0, 860.0);
        let window = WindowBuilder::new(app, &label)
            .title(&title)
            .inner_size(width, height)
            .min_inner_size(720.0, 560.0)
            .build()?;
        let bar = BarSetup {
            mode: "reading",
            name: &spec.site_name,
            lang: lang.tag(),
            plugin: &spec.plugin_name,
            method: &spec.method_name,
            guide: spec.guide.as_deref(),
        };
        let bar_view = window.add_child(
            WebviewBuilder::new(
                format!("{label}-bar"),
                WebviewUrl::App("login-bar.html".into()),
            )
            .initialization_script(format!(
                "window.usuyoLogin = {};",
                serde_json::to_string(&bar)?
            )),
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(width, BAR_HEIGHT),
        )?;
        let nav = allowed.clone();
        let site_builder = WebviewBuilder::new(&site_label, WebviewUrl::External(url))
            .initialization_script(channel::page_script(&allowed, &spec.script, lang.tag()))
            .on_navigation(move |u: &Url| {
                let ok = nav.page(u);
                if !ok {
                    eprintln!(
                        "usuyo: the site window stayed put instead of opening {}://{}",
                        u.scheme(),
                        u.host_str().unwrap_or("")
                    );
                }
                ok
            })
            .on_new_window(|_, _| NewWindowResponse::Deny)
            .on_download(|_, _| false);
        let site_view = window.add_child(
            self.with_store(site_builder, &store),
            LogicalPosition::new(0.0, BAR_HEIGHT),
            LogicalSize::new(width, height - BAR_HEIGHT),
        )?;
        let closed = login::follow_size(&window, &bar_view, &site_view, BAR_HEIGHT);
        let _pressed = self.hold(&store, &label);
        let (tx, rx) = mpsc::channel();
        lock(&reads.windows).insert(label.clone(), tx);

        let me = Arc::clone(self);
        let reads = Arc::clone(reads);
        std::thread::spawn(move || {
            let seat = Seat {
                bar: &bar_view,
                site: &site_view,
                closed: &closed,
                allowed: &allowed,
            };
            me.read(&seat, &spec, &rx);
            lock(&reads.windows).remove(&label);
            me.release(&store, &label);
        });
        Ok(())
    }

    /// 窗口开着时的那个线程：收报告和按钮，到点叫脚本翻一屏，刷新面板。
    fn read(&self, seat: &Seat<'_>, spec: &WindowSpec, rx: &Receiver<Msg>) {
        let mut s = Session::new();
        let mut next = Instant::now() + STEP_MIN;
        let mut steps = 0u64;
        let mut dirty = true;
        let mut painted = Instant::now() - PANEL_EVERY;
        loop {
            let effects = match rx.recv_timeout(TICK) {
                Ok(Msg::Report(v)) => match Report::parse(&v, seat.allowed) {
                    Ok(r) => {
                        dirty = true;
                        s.apply(r)
                    }
                    Err(why) => {
                        eprintln!("usuyo: dropped a report from the site window: {why}");
                        Vec::new()
                    }
                },
                Ok(Msg::Control(c)) => {
                    dirty = true;
                    s.control(c)
                }
                Err(RecvTimeoutError::Timeout) => Vec::new(),
                Err(RecvTimeoutError::Disconnected) => return,
            };
            if seat.closed.load(Ordering::SeqCst) {
                // 关窗口时还没交的照样交过去，在导入页确认
                if !s.handed() && !s.links.is_empty() {
                    self.hand_over(&mut s, spec);
                }
                return;
            }
            for e in effects {
                match e {
                    Effect::SignedIn => {
                        if let Err(err) = self.signed_in(spec) {
                            eprintln!("usuyo: cannot record the sign-in: {err:#}");
                        }
                    }
                    Effect::Resume => {
                        let _ = seat.site.eval(channel::RESUME);
                    }
                    Effect::HandOver => {
                        s.phase = Phase::Handing;
                        self.paint(seat.bar, &s);
                        self.hand_over(&mut s, spec);
                        dirty = true;
                    }
                }
            }
            let now = Instant::now();
            if s.steps() && now >= next {
                let _ = seat.site.eval(channel::STEP);
                steps += 1;
                next = next_step(now, steps);
            }
            if !s.steps() {
                next = next.max(now + STEP_MIN);
            }
            if dirty && painted.elapsed() >= PANEL_EVERY {
                self.paint(seat.bar, &s);
                painted = Instant::now();
                dirty = false;
            }
        }
    }

    fn signed_in(&self, spec: &WindowSpec) -> Result<()> {
        let (owner, repo) = plugin_path(&spec.plugin)?;
        self.api(
            "POST",
            &[
                "plugins",
                owner,
                repo,
                "methods",
                &spec.method,
                "window",
                "signed-in",
            ],
            None,
        )?;
        Ok(())
    }

    fn hand_over(&self, s: &mut Session, spec: &WindowSpec) {
        let result = plugin_path(&spec.plugin).and_then(|(owner, repo)| {
            self.api(
                "POST",
                &[
                    "plugins",
                    owner,
                    repo,
                    "methods",
                    &spec.method,
                    "window",
                    "links",
                ],
                Some(s.sources()),
            )
        });
        s.phase = match result {
            Ok(v) => {
                let n = |k: &str| {
                    v.get(k)
                        .and_then(Value::as_u64)
                        .and_then(|x| u32::try_from(x).ok())
                        .unwrap_or(0)
                };
                let imports = v
                    .get("imports")
                    .and_then(Value::as_array)
                    .map_or(0, |a| u32::try_from(a.len()).unwrap_or(u32::MAX));
                Phase::Handed {
                    imports,
                    noted: n("noted"),
                    dropped: n("dropped"),
                }
            }
            Err(e) => {
                eprintln!("usuyo: cannot hand the noted links to the daemon: {e:#}");
                Phase::Failed
            }
        };
    }

    fn paint(&self, bar: &Webview, s: &Session) {
        let view = s.view().to_string();
        let _ = bar.eval(format!("window.usuyoPanel && window.usuyoPanel({view});"));
    }
}

/// 一个站点窗口的几块。
struct Seat<'a> {
    bar: &'a Webview,
    site: &'a Webview,
    closed: &'a std::sync::atomic::AtomicBool,
    allowed: &'a Allowed,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(v: Value) -> Report {
        let allowed = Allowed::new("www.example.test", &[]);
        Report::parse(&json!({ "report": v }), &allowed).unwrap()
    }

    fn link(action: &str, n: u32) -> Value {
        json!({"action": action, "url": format!("https://www.example.test/explore/{n:024x}?xsec_token=t{n}"), "title": format!("笔记 {n}")})
    }

    #[test]
    fn reading_starts_once_signed_in_and_hands_over_at_the_end() {
        let mut s = Session::new();
        assert!(!s.steps(), "不在登录前翻");
        assert_eq!(
            s.apply(report(json!({"signedIn": true}))),
            [Effect::SignedIn]
        );
        assert!(s.steps());
        assert!(
            s.apply(report(json!({"signedIn": true}))).is_empty(),
            "登录只记一次"
        );
        s.apply(report(
            json!({"links": [link("favorite", 1), link("favorite", 2), link("favorite", 1)]}),
        ));
        s.apply(report(json!({"links": [link("like", 3)], "counts": {"requests": 9, "recognised": 3, "skipped": {"noToken": 1}}})));
        let v = s.view();
        assert_eq!(v["noted"], json!({"favorite": 2, "like": 1, "all": 3}));
        assert_eq!(v["skipped"]["noToken"], 1);
        assert_eq!(s.apply(report(json!({"state": "end"}))), [Effect::HandOver]);
        let sources = s.sources();
        assert_eq!(sources["sources"].as_array().unwrap().len(), 2);
        assert_eq!(sources["sources"][0]["action"], "favorite");
        assert_eq!(sources["sources"][0]["links"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn a_stop_holds_scrolling_until_you_resume() {
        let mut s = Session::new();
        s.apply(report(json!({"signedIn": true})));
        s.apply(report(json!({"state": "stopped", "reason": "verify"})));
        assert_eq!(s.phase, Phase::Stopped("verify".into()));
        assert!(!s.steps());
        assert_eq!(s.view()["reason"], "verify");
        assert_eq!(s.control(Control::Resume), [Effect::Resume]);
        assert!(s.steps());
        // 路上那份还写着停下：不算；脚本报它接着读了以后，再停下才算
        s.apply(report(json!({"state": "stopped", "reason": "verify"})));
        assert!(s.steps());
        s.apply(report(json!({"state": "reading"})));
        s.apply(report(json!({"state": "stopped", "reason": "verify"})));
        assert!(!s.steps());
    }

    #[test]
    fn manual_and_pause_stop_the_steps_and_finish_hands_over_once() {
        let mut s = Session::new();
        s.apply(report(json!({"signedIn": true})));
        s.control(Control::Manual);
        assert!(!s.steps());
        s.control(Control::Auto);
        s.control(Control::Pause);
        assert!(!s.steps());
        s.control(Control::Resume);
        assert!(s.steps());
        assert_eq!(s.control(Control::Finish), [Effect::HandOver]);
        s.phase = Phase::Handing;
        assert!(s.control(Control::Finish).is_empty());
        // 交出去以后不再记
        s.apply(report(json!({"links": [link("favorite", 9)]})));
        assert!(s.links.is_empty());
    }

    #[test]
    fn steps_are_several_seconds_apart_and_vary() {
        let now = Instant::now();
        let gaps: Vec<Duration> = (0..20).map(|n| next_step(now, n) - now).collect();
        assert!(
            gaps.iter()
                .all(|g| *g >= STEP_MIN && *g < STEP_MIN + STEP_JITTER)
        );
        assert!(gaps.iter().any(|g| *g != gaps[0]), "带一点随机");
    }

    #[test]
    fn the_number_of_links_kept_is_bounded() {
        let mut s = Session::new();
        s.signed_in = true;
        for chunk in 0..(LINKS_MAX / 200 + 1) {
            let links: Vec<Value> = (0..200)
                .map(|i| link("favorite", u32::try_from(chunk * 200 + i).unwrap()))
                .collect();
            s.apply(report(json!({ "links": links })));
        }
        assert_eq!(s.links.len(), LINKS_MAX);
        assert_eq!(s.over, 200);
    }
}
