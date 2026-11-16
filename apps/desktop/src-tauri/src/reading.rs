//! 在 usuyo 的站点窗口里读（插件清单里 `delivers: links`、`account: in-usuyo` 的方法）。
//!
//! 窗口和登录窗口一样：上面一条是我们自己的页面（`login-bar.html`，这里多一块面板），下面是站点的网页，
//! 用这个站点（`browser-profile`）自己那份网页数据，和主窗口、访客身份、你日常的浏览器都分开；cookie 不读出来、不交给内核。
//! 下面的网页打开清单里的 `loginUrl`，页面里跑插件的脚本（[`channel::page_script`]）。你在网页里登录；
//! 脚本认出登录了、记下列表里的链接，经只进不出的那一条（`channel.rs`）交给壳。
//!
//! 默认自己往下翻：隔几秒（带一点随机）叫脚本翻一屏；面板上能换成你自己翻、暂停、继续。
//! 脚本每读完一个列表（读到底，或者找不到、认不出而跳过）就说一声，壳把这个列表记下的交给内核，
//! 落成普通的导入（在导入页确认）；同一个列表后来又记下的并进那一份。脚本说到底了，壳交出剩下的，不用你点什么。
//! 说停下了（验证页、登录墙、站点的错误码：要你动手的），壳不再叫它翻，交出已经记下的，面板上写原因和你要做什么。
//! 「完成」和关掉窗口照样交出还没交的；交没成的过一会儿自己再交。
//!
//! 能去哪些地址只照清单的站点和 `domains`；新窗口、下载、别的协议都挡掉。

use std::collections::{BTreeMap, BTreeSet, HashMap};
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
/// 交给内核没成：隔这么久自己再交。
const SEND_RETRY: Duration = Duration::from_secs(10);

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
    /// 停下了，`reason` 是脚本给的词：要你动手（验证页、登录墙、站点的错误码……）。
    Stopped(String),
    /// 读完了（脚本说到底了，或你点了「完成」）。之后你自己翻出来的照样交。
    Ended,
}

/// 要线程去做的事。
#[derive(Debug, PartialEq, Eq)]
enum Effect {
    SignedIn,
    /// 叫页面里的脚本接着读（停下以后点了「继续」）。
    Resume,
}

/// 交给内核的一回：请求体，和这回交了哪几条（交成了才算交过）。
#[derive(Debug)]
struct Batch {
    body: Value,
    urls: Vec<String>,
}

/// 一个站点窗口的状态：只有数和记下的链接，没有 cookie。
struct Session {
    phase: Phase,
    auto: bool,
    paused: bool,
    signed_in: bool,
    links: BTreeMap<String, Noted>,
    /// 已经交给内核的。
    sent: BTreeSet<String>,
    /// 来源（`favorite`……）→ 它落成的那次导入；后面再交就并进去。
    imports: BTreeMap<String, String>,
    /// 脚本说读过了的列表 → 跳过的原因（读到底的是 `None`）。
    finished: BTreeMap<String, Option<String>>,
    /// 内核收下了几条、几条不合插件的 `match`。
    handed_noted: u32,
    dropped: u32,
    /// 上一回交给内核没成；线程过一会儿自己再交。
    failed: bool,
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
            sent: BTreeSet::new(),
            imports: BTreeMap::new(),
            finished: BTreeMap::new(),
            handed_noted: 0,
            dropped: 0,
            failed: false,
            over: 0,
            counts: Counts::default(),
            diagnosis: String::new(),
            resuming: false,
        }
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
        if let Some(c) = r.counts {
            self.counts = c;
        }
        if let Some(d) = r.diagnosis {
            self.diagnosis = d;
        }
        if let Some(f) = r.finished {
            self.finished = f.into_iter().map(|f| (f.action, f.reason)).collect();
        }
        match r.said {
            Some(Said::Reading) => self.resuming = false,
            Some(Said::Stopped) if self.resuming => {}
            Some(Said::End) if self.phase != Phase::Ended => self.phase = Phase::Ended,
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
            // 「完成」：不再翻，记下的都交出去（交没成的也马上再交一回）
            Control::Finish => {
                self.phase = Phase::Ended;
                self.failed = false;
            }
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

    fn unsent(&self) -> usize {
        self.links.len() - self.sent.len()
    }

    /// 现在该交的：读过了的列表里还没交的；停下了、读完了就是全部还没交的。没有就是 `None`。
    /// 前面交过的来源都带上它的导入（没有新链接的也带，内核照它算这一回一共几条），新链接并进那一份。
    fn batch(&self) -> Option<Batch> {
        let all = matches!(self.phase, Phase::Stopped(_) | Phase::Ended);
        let mut by: BTreeMap<&str, Vec<Value>> = BTreeMap::new();
        let mut urls = Vec::new();
        for l in self.links.values() {
            if self.sent.contains(&l.url) || !(all || self.finished.contains_key(&l.action)) {
                continue;
            }
            let mut link = json!({ "url": l.url });
            if let Some(t) = &l.title {
                link["title"] = json!(t);
            }
            by.entry(l.action.as_str()).or_default().push(link);
            urls.push(l.url.clone());
        }
        if urls.is_empty() {
            return None;
        }
        for action in self.imports.keys() {
            by.entry(action.as_str()).or_default();
        }
        let sources: Vec<Value> = by
            .into_iter()
            .map(|(action, links)| {
                let mut src = json!({ "action": action, "links": links });
                if let Some(id) = self.imports.get(action) {
                    src["import"] = json!(id);
                }
                src
            })
            .collect();
        Some(Batch {
            body: json!({ "sources": sources }),
            urls,
        })
    }

    /// 内核收下了这一回：记下交过的、每个来源落成的导入和条数。
    fn delivered(&mut self, urls: Vec<String>, answer: &Value) {
        self.sent.extend(urls);
        let n = |k: &str| {
            answer
                .get(k)
                .and_then(Value::as_u64)
                .and_then(|x| u32::try_from(x).ok())
                .unwrap_or(0)
        };
        self.handed_noted = self.handed_noted.saturating_add(n("noted"));
        self.dropped = self.dropped.saturating_add(n("dropped"));
        for i in answer
            .get("imports")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let (Some(action), Some(id)) = (
                i.get("action").and_then(Value::as_str),
                i.get("id").and_then(Value::as_str),
            ) {
                self.imports.insert(action.to_owned(), id.to_owned());
            }
        }
        self.failed = false;
    }

    /// 窗口标题，跟着读到哪了变。
    fn title(&self, lang: Lang, site: &str) -> String {
        let w = match self.phase {
            Phase::SigningIn => words::LOGIN_TITLE,
            Phase::Reading => words::READING_TITLE,
            Phase::Stopped(_) => words::READING_STOPPED_TITLE,
            Phase::Ended => words::READING_ENDED_TITLE,
        };
        w.fill(lang, &[("site", &site)])
    }

    /// 给面板的：都是数和固定的词，加上脚本给的形状诊断（不含内容）。
    fn view(&self) -> Value {
        let (phase, reason) = match &self.phase {
            Phase::SigningIn => ("signing-in", None),
            Phase::Reading => ("reading", None),
            Phase::Stopped(r) => ("stopped", Some(r.as_str())),
            Phase::Ended => ("ended", None),
        };
        let lists: Vec<Value> = self
            .finished
            .iter()
            .map(|(action, reason)| json!({ "action": action, "reason": reason }))
            .collect();
        json!({
            "phase": phase,
            "reason": reason,
            "handed": { "imports": self.imports.len(), "noted": self.handed_noted, "dropped": self.dropped },
            "unsent": self.unsent(),
            "failed": self.failed,
            "lists": lists,
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
            login::with_agent(
                WebviewBuilder::new(
                    format!("{label}-bar"),
                    WebviewUrl::App("login-bar.html".into()),
                )
                .initialization_script(format!(
                    "window.usuyoLogin = {};",
                    serde_json::to_string(&bar)?
                )),
            ),
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(width, BAR_HEIGHT),
        )?;
        let nav = allowed.clone();
        let site_builder =
            login::with_agent(WebviewBuilder::new(&site_label, WebviewUrl::External(url)))
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

    /// 窗口开着时的那个线程：收报告和按钮，到点叫脚本翻一屏，读完一个列表就交给内核，刷新面板和标题。
    fn read(&self, seat: &Seat<'_>, spec: &WindowSpec, rx: &Receiver<Msg>) {
        let mut s = Session::new();
        let mut next = Instant::now() + STEP_MIN;
        let mut steps = 0u64;
        let mut dirty = true;
        let mut painted = Instant::now() - PANEL_EVERY;
        let mut send_at = Instant::now();
        let mut titled = String::new();
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
                    if c == Control::Finish {
                        send_at = Instant::now();
                    }
                    s.control(c)
                }
                Err(RecvTimeoutError::Timeout) => Vec::new(),
                Err(RecvTimeoutError::Disconnected) => return,
            };
            if seat.closed.load(Ordering::SeqCst) {
                // 关窗口时还没交的照样交过去，在导入页确认
                s.phase = Phase::Ended;
                if let Some(b) = s.batch() {
                    self.hand_over(&mut s, spec, b);
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
                }
            }
            let now = Instant::now();
            if now >= send_at
                && let Some(b) = s.batch()
            {
                self.hand_over(&mut s, spec, b);
                if s.failed {
                    send_at = now + SEND_RETRY;
                }
                dirty = true;
            }
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
                let title = s.title(Lang::of(self.daemon.machine_dir()), &spec.site_name);
                if title != titled {
                    let _ = seat.site.window().set_title(&title);
                    titled = title;
                }
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

    /// 交一回；没成的留着，下一回连同新记下的一起交。
    fn hand_over(&self, s: &mut Session, spec: &WindowSpec, b: Batch) {
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
                Some(b.body),
            )
        });
        match result {
            Ok(v) => s.delivered(b.urls, &v),
            Err(e) => {
                eprintln!("usuyo: cannot hand the noted links to the daemon: {e:#}");
                s.failed = true;
            }
        }
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

    /// 交给内核的那一回里每个来源有几条、带没带导入 id。
    fn sources(b: &Batch) -> Vec<(String, usize, Option<String>)> {
        b.body["sources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                (
                    s["action"].as_str().unwrap().to_owned(),
                    s["links"].as_array().unwrap().len(),
                    s["import"].as_str().map(str::to_owned),
                )
            })
            .collect()
    }

    fn answer(imports: &[(&str, &str)], noted: u32) -> Value {
        let imports: Vec<Value> = imports
            .iter()
            .map(|(a, id)| json!({"id": id, "action": a, "links": 0}))
            .collect();
        json!({"imports": imports, "noted": noted, "dropped": 0})
    }

    #[test]
    fn reading_starts_once_signed_in_and_counts_what_it_noted() {
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
        assert!(s.batch().is_none(), "一个列表都还没读完：不交");
    }

    /// 2026-11-15：读完「收藏」以后找不到下一栏，读到的一直不交、要你点「完成」。现在读完一栏就交这一栏；
    /// 后面的栏跳过了，脚本说到底，剩下的自己交，不用点什么。
    #[test]
    fn each_list_is_handed_over_once_it_is_read_and_the_end_needs_no_click() {
        let mut s = Session::new();
        s.apply(report(json!({"signedIn": true})));
        let favs: Vec<Value> = (0..150).map(|i| link("favorite", i)).collect();
        s.apply(report(json!({ "links": favs })));
        s.apply(report(
            json!({"links": [link("like", 900)], "finished": [{"action": "favorite"}]}),
        ));
        let b = s.batch().expect("favorites are due");
        assert_eq!(sources(&b), [("favorite".into(), 150, None)]);
        s.delivered(b.urls, &answer(&[("favorite", "imp-fav")], 150));
        assert!(s.batch().is_none(), "交过的不再交，没读完的「赞过」不交");
        assert!(s.steps(), "交了「收藏」照样往下读");
        // 「收藏」后来又冒出来一条：并进同一份导入
        s.apply(report(json!({"links": [link("favorite", 500)]})));
        let b = s.batch().unwrap();
        assert_eq!(
            sources(&b),
            [("favorite".into(), 1, Some("imp-fav".into()))]
        );
        s.delivered(b.urls, &answer(&[("favorite", "imp-fav")], 1));
        // 「赞过」找不到，跳过；脚本说到底了：剩下的自己交，带上前面那份导入好算总数
        s.apply(report(json!({"state": "end", "finished": [{"action": "favorite"}, {"action": "like", "reason": "notab"}]})));
        assert_eq!(s.phase, Phase::Ended);
        assert!(!s.steps());
        let b = s.batch().unwrap();
        assert_eq!(
            sources(&b),
            [
                ("favorite".into(), 0, Some("imp-fav".into())),
                ("like".into(), 1, None)
            ]
        );
        s.delivered(
            b.urls,
            &answer(&[("favorite", "imp-fav"), ("like", "imp-like")], 1),
        );
        assert!(s.batch().is_none());
        let v = s.view();
        assert_eq!(v["phase"], "ended");
        assert_eq!(v["unsent"], 0);
        assert_eq!(v["handed"]["noted"], 152);
        assert_eq!(v["lists"][1], json!({"action": "like", "reason": "notab"}));
        assert_eq!(s.title(Lang::ZhHans, "例子"), "已读取例子 · usuyo");
    }

    #[test]
    fn a_stop_holds_scrolling_hands_over_what_was_read_and_waits_for_you() {
        let mut s = Session::new();
        s.apply(report(json!({"signedIn": true})));
        assert_eq!(s.title(Lang::ZhHans, "例子"), "正在读取例子 · usuyo");
        s.apply(report(json!({"links": [link("favorite", 1)]})));
        s.apply(report(json!({"state": "stopped", "reason": "verify"})));
        assert_eq!(s.phase, Phase::Stopped("verify".into()));
        assert!(!s.steps());
        assert_eq!(s.view()["reason"], "verify");
        assert_eq!(
            s.title(Lang::En, "Example"),
            "Reading Stopped · Example · usuyo"
        );
        // 停下了：记下的先交出去
        let b = s.batch().unwrap();
        s.delivered(b.urls, &answer(&[("favorite", "imp-fav")], 1));
        assert_eq!(s.control(Control::Resume), [Effect::Resume]);
        assert!(s.steps());
        // 路上那份还写着停下：不算；脚本报它接着读了以后，再停下才算
        s.apply(report(json!({"state": "stopped", "reason": "verify"})));
        assert!(s.steps());
        s.apply(report(json!({"state": "reading"})));
        s.apply(report(json!({"links": [link("favorite", 2)]})));
        assert!(s.batch().is_none(), "接着读的这一栏没读完：先不交");
        s.apply(report(json!({"state": "stopped", "reason": "verify"})));
        assert!(!s.steps());
        let b = s.batch().unwrap();
        assert_eq!(
            sources(&b),
            [("favorite".into(), 1, Some("imp-fav".into()))]
        );
    }

    #[test]
    fn a_failed_hand_over_keeps_the_links_for_the_next_one() {
        let mut s = Session::new();
        s.apply(report(
            json!({"signedIn": true, "links": [link("favorite", 1)], "state": "end"}),
        ));
        assert!(s.batch().is_some());
        s.failed = true;
        assert_eq!(s.view()["failed"], true);
        s.apply(report(json!({"links": [link("favorite", 2)]})));
        let b = s.batch().unwrap();
        assert_eq!(sources(&b), [("favorite".into(), 2, None)]);
        s.delivered(b.urls, &answer(&[("favorite", "imp")], 2));
        assert_eq!(s.view()["failed"], false);
    }

    #[test]
    fn manual_and_pause_stop_the_steps_and_finish_hands_over_everything() {
        let mut s = Session::new();
        s.apply(report(json!({"signedIn": true})));
        s.control(Control::Manual);
        assert!(!s.steps());
        s.control(Control::Auto);
        s.control(Control::Pause);
        assert!(!s.steps());
        s.control(Control::Resume);
        assert!(s.steps());
        s.apply(report(
            json!({"links": [link("favorite", 9), link("like", 8)]}),
        ));
        assert!(s.batch().is_none());
        s.control(Control::Finish);
        assert!(!s.steps());
        assert_eq!(s.batch().unwrap().urls.len(), 2);
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
