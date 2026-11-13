//! 获取插件里的方法（协议 §2.3.1）：开关、这台机器上用不用得上、取一条链接时挑哪几种，
//! 每种上一次调用的结果和失败日志。
//!
//! 开关记在按机器的 `settings.json`（`plugins.<插件 id>.methods.<方法 id>.enabled`），没写的照默认。
//! 插件自己报的本机检查（`health`）缓存一阵，不为每次取内容、每次看插件页都起一个进程。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError, RwLock};
use std::time::Duration;

use serde_json::{Map, Value};
use usuyo_plugin::protocol::{
    Account, CallFailure, ErrorCode, FetchHealth, FetchInputKind, Login, Method,
};
use usuyo_plugin::{CallOptions, DeclaredSite, DeclaredVisitor, FetchPlugin, LoginState};

use crate::engine::Engine;
use crate::fetch::now_ms;
use crate::view::{Blocker, Event, PluginView};
use crate::{Error, Result};

/// 插件的本机检查多久问一次。
const HEALTH_TTL: Duration = Duration::from_secs(10 * 60);

/// 失败日志多大时换一份：现在这份改名成 `fetch-failures.1.jsonl`（盖掉更早的那份），重新开始。
const FAILURE_LOG_MAX: u64 = 1 << 20;

/// 上次问插件的本机检查：什么时候、回了什么（调用失败时是一句原因）。
#[derive(Debug, Clone)]
pub(crate) struct HealthSeen {
    pub at: i64,
    pub result: std::result::Result<FetchHealth, String>,
}

pub(crate) struct Methods {
    /// 写了的开关：(插件 id, 方法 id) → 开着没有。没写的照 [`Method::on_by_default`]。
    switches: RwLock<BTreeMap<(String, String), bool>>,
    /// 插件 id → 上次的本机检查。
    health: Mutex<BTreeMap<String, HealthSeen>>,
    /// `logs/fetch-failures.jsonl`。
    failure_log: PathBuf,
}

impl Methods {
    /// 从按机器的 `settings.json` 读开关；写坏了（不是真假）的报错。
    pub(crate) fn load(machine_root: &Path) -> Result<Methods> {
        let map = crate::settings::read(machine_root)?;
        let mut switches = BTreeMap::new();
        if let Some(Value::Object(plugins)) = map.get("plugins") {
            for (plugin, conf) in plugins {
                let Some(Value::Object(methods)) = conf.get("methods") else {
                    continue;
                };
                for (method, m) in methods {
                    match m.get("enabled") {
                        None => {}
                        Some(Value::Bool(b)) => {
                            switches.insert((plugin.clone(), method.clone()), *b);
                        }
                        Some(other) => {
                            return Err(Error::InvalidArgs(format!(
                                "settings.json: plugins.{plugin}.methods.{method}.enabled is {other}, not true or false"
                            )));
                        }
                    }
                }
            }
        }
        Ok(Methods {
            switches: RwLock::new(switches),
            health: Mutex::new(BTreeMap::new()),
            failure_log: machine_root.join("logs").join("fetch-failures.jsonl"),
        })
    }

    /// 你拨过的开关；没拨过是 `None`。
    fn switch(&self, plugin: &FetchPlugin, m: &Method) -> Option<bool> {
        self.switches
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&(plugin.id().to_string(), m.id.clone()))
            .copied()
    }

    pub(crate) fn failure_log(&self) -> &Path {
        &self.failure_log
    }

    /// 开关的样子：拨过的照你拨的，没拨过照方法本身的那一层。没做的总是关着。
    pub(crate) fn enabled(&self, plugin: &FetchPlugin, m: &Method) -> bool {
        m.built && self.switch(plugin, m).unwrap_or_else(|| m.on_by_default())
    }

    /// 在这个平台上开着没有：拨过的开关一种方法一个，各平台都照它；没拨过照这个平台上的那一层
    /// （`perPlatform` 抬到 `private-api` 的平台默认关着）。
    pub(crate) fn enabled_on(&self, plugin: &FetchPlugin, m: &Method, platform: &str) -> bool {
        m.built
            && self
                .switch(plugin, m)
                .unwrap_or_else(|| m.on_by_default_on(platform, &plugin.spec.platforms))
    }

    pub(crate) fn health_seen(&self, plugin: &str) -> Option<HealthSeen> {
        self.health
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(plugin)
            .cloned()
    }

    /// 下次用到时重新问（「现在再试」以后）。
    pub(crate) fn forget_health(&self, plugin: &str) {
        self.health
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(plugin);
    }

    /// 记一行失败日志。写不进只记日志：取内容的结果不因它变。
    fn log_failure(&self, line: &FailureLine<'_>) {
        if let Err(e) = append_failure(&self.failure_log, line) {
            tracing::warn!(path = %self.failure_log.display(), error = %e, "fetch failure log not written");
        }
    }
}

/// 失败日志的一行（§2.3.1）。链接只记主机。
#[derive(serde::Serialize)]
struct FailureLine<'a> {
    at: String,
    plugin: &'a str,
    method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    host: Option<&'a str>,
    code: &'a str,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<&'a str>,
}

fn append_failure(path: &Path, line: &FailureLine<'_>) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    if std::fs::metadata(path).is_ok_and(|m| m.len() > FAILURE_LOG_MAX) {
        std::fs::rename(path, path.with_extension("1.jsonl"))?;
    }
    let mut opts = std::fs::OpenOptions::new();
    opts.append(true).create(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
    let mut f = opts.open(path)?;
    let mut bytes = serde_json::to_vec(line)?;
    bytes.push(b'\n');
    f.write_all(&bytes)
}

/// 一种方法这台机器上为什么用不上；空的是用得上。名字取自清单，不写平台名。
pub(crate) fn blockers(
    m: &Method,
    enabled: bool,
    health: Option<&HealthSeen>,
    sites: &[DeclaredSite],
    visitors: &[DeclaredVisitor],
) -> Vec<Blocker> {
    let mut out = Vec::new();
    let mut add = |code: &str, hint: String| {
        out.push(Blocker {
            code: code.into(),
            hint,
        });
    };
    if !m.built {
        add("not-built", "这种方法还没做".into());
        return out;
    }
    if !enabled {
        add("off", "你关掉了这种方法".into());
    }
    if m.login == Login::Required {
        for name in &m.sites {
            let Some(d) = sites.iter().find(|d| &d.site.site == name) else {
                continue;
            };
            match d.login {
                LoginState::Present => {}
                LoginState::Missing => add(
                    "login-missing",
                    format!("要先在 usuyo 里登录{}", d.site.name),
                ),
                LoginState::Expired => add(
                    "login-expired",
                    format!("{}的登录过期了，要重新登录", d.site.name),
                ),
            }
        }
    }
    for name in &m.visitors {
        let Some(v) = visitors.iter().find(|v| &v.visitor.site == name) else {
            continue;
        };
        match v.state {
            LoginState::Present => {}
            LoginState::Missing => add(
                "visitor-missing",
                format!("要先开一次{}首页，取一份访客身份", v.visitor.name),
            ),
            LoginState::Expired => add(
                "visitor-expired",
                format!("{}的访客身份过期了，要再取一份", v.visitor.name),
            ),
        }
    }
    match health.map(|h| &h.result) {
        None => {}
        Some(Err(why)) => add("health-failed", format!("插件没能自查：{why}")),
        Some(Ok(h)) => {
            for c in h.checks.iter().filter(|c| !c.ok) {
                if c.methods.is_empty() || c.methods.iter().any(|x| x == &m.id) {
                    add(
                        "check-failed",
                        c.message
                            .clone()
                            .unwrap_or_else(|| format!("没过：{}", c.name)),
                    );
                }
            }
        }
    }
    out
}

/// 取一条链接时挑出来的一种方法。
#[derive(Debug, Clone)]
pub(crate) struct Candidate {
    pub plugin: FetchPlugin,
    pub method: Method,
}

impl Engine {
    /// 这个插件的本机检查：缓存过期了（或没问过）才问。调用失败记成一句原因。
    pub(crate) async fn fetch_health(&self, plugin: &FetchPlugin) -> HealthSeen {
        let methods = &self.inner.methods;
        let now = now_ms();
        if let Some(seen) = methods.health_seen(plugin.id().as_str())
            && now.saturating_sub(seen.at) < HEALTH_TTL.as_millis() as i64
        {
            return seen;
        }
        let result = self
            .inner
            .host
            .health(plugin, &CallOptions::default())
            .await
            .map_err(|e| match e {
                usuyo_plugin::PluginError::Call { failure, .. } => {
                    failure.hint.unwrap_or(failure.message)
                }
                other => other.to_string(),
            });
        let seen = HealthSeen {
            at: now_ms(),
            result,
        };
        methods
            .health
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(plugin.id().to_string(), seen.clone());
        seen
    }

    /// 取这条链接的方法，照 §2.3.1 排好：平台插件的在前、通用网页的在后；各自里先层级低的，同一层先不带账号的，
    /// 再照插件和清单的顺序。只留做了、开着、这台机器上用得上、有能接链接的 `fetch` 的。
    /// 一种都用不上时回第二项：排在最前的那种为什么用不上，拿来告诉你。
    /// `platform` 是这条链接的平台（条目 id 前面那段），`perPlatform` 照它取层级和默认的开关。
    pub(crate) async fn fetch_candidates(
        &self,
        url: &url::Url,
        platform: &str,
    ) -> Result<(Vec<Candidate>, Option<CallFailure>)> {
        let host = &self.inner.host;
        let sites = host.sites()?;
        let visitors = host.visitors()?;
        let mut ranked = Vec::new();
        for (pi, plugin) in host.candidates(url).into_iter().enumerate() {
            for (mi, m) in plugin.spec.methods.iter().enumerate() {
                let takes_url = m.callable()
                    && m.capabilities
                        .fetch
                        .as_ref()
                        .is_some_and(|f| f.input.contains(&FetchInputKind::Url));
                if takes_url {
                    ranked.push((
                        plugin.spec.is_web(),
                        m.rank_on(platform, &plugin.spec.platforms),
                        pi,
                        mi,
                        plugin,
                        m,
                    ));
                }
            }
        }
        ranked.sort_by_key(|(web, rank, pi, mi, _, _)| (*web, *rank, *pi, *mi));
        let mut out = Vec::new();
        let mut why: Option<CallFailure> = None;
        for (_, _, _, _, plugin, m) in ranked {
            let enabled = self.inner.methods.enabled_on(plugin, m, platform);
            let health = if enabled {
                Some(self.fetch_health(plugin).await)
            } else {
                None
            };
            let blocked = blockers(m, enabled, health.as_ref(), &sites, &visitors);
            match blocked.first() {
                None => out.push(Candidate {
                    plugin: plugin.clone(),
                    method: m.clone(),
                }),
                // 你关掉的不拿来当原因：说一个你能去补的（登录、访客身份、缺的程序）
                Some(b) if why.is_none() && b.code != "off" => {
                    why = Some(blocker_failure(plugin, m, b));
                }
                Some(_) => {}
            }
        }
        Ok((out, why))
    }

    /// 记下这种方法这一次调用的结果；失败的另记一行失败日志。
    pub(crate) fn record_method_call(
        &self,
        plugin: &FetchPlugin,
        method: &str,
        host: Option<&str>,
        result: std::result::Result<(), &CallFailure>,
    ) -> Result<()> {
        let now = now_ms();
        let (code, hint) = match result {
            Ok(()) => (None, None),
            Err(f) => (Some(f.code.as_str()), f.hint.as_deref()),
        };
        self.inner
            .jobs
            .record_method_call(plugin.id().as_str(), method, now, code, hint)?;
        if let Err(f) = result {
            self.inner.methods.log_failure(&FailureLine {
                at: jiff::Timestamp::from_millisecond(now)
                    .map(usuyo_library::time::format)
                    .unwrap_or_default(),
                plugin: plugin.id().as_str(),
                method,
                host,
                code: f.code.as_str(),
                message: &f.message,
                hint: f.hint.as_deref(),
                detail: f.detail.as_deref().map(String::as_str),
            });
        }
        Ok(())
    }

    /// 开或关插件的一种方法，写进按机器的 `settings.json`，马上生效。没做的方法没有开关。
    pub fn set_method(
        &self,
        plugin_id: &str,
        method_id: &str,
        enabled: bool,
    ) -> Result<PluginView> {
        self.switch_method(plugin_id, method_id, Some(enabled))
    }

    /// 去掉拨过的开关：这种方法回到各平台各自那一层的默认（`private-api`、`client-hook` 关着）。
    pub fn clear_method(&self, plugin_id: &str, method_id: &str) -> Result<PluginView> {
        self.switch_method(plugin_id, method_id, None)
    }

    fn switch_method(
        &self,
        plugin_id: &str,
        method_id: &str,
        enabled: Option<bool>,
    ) -> Result<PluginView> {
        let plugin = self
            .inner
            .host
            .plugin(plugin_id)
            .ok_or_else(|| Error::NotFound(format!("plugin {plugin_id}")))?;
        let m = plugin
            .spec
            .method(method_id)
            .ok_or_else(|| Error::NotFound(format!("method {method_id} of {plugin_id}")))?;
        if !m.built {
            return Err(Error::UnsupportedInput(format!(
                "method {method_id} is not built, so it has no switch"
            )));
        }
        {
            let _guard = self
                .inner
                .settings_lock
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let root = &self.inner.machine_root;
            let mut map = crate::settings::read(root)?;
            let key = (plugin_id.to_owned(), method_id.to_owned());
            let mut switches = self
                .inner
                .methods
                .switches
                .write()
                .unwrap_or_else(PoisonError::into_inner);
            match enabled {
                Some(on) => {
                    set_switch(&mut map, plugin_id, method_id, on);
                    crate::settings::write(root, &map)?;
                    switches.insert(key, on);
                }
                None => {
                    clear_switch(&mut map, plugin_id, method_id);
                    crate::settings::write(root, &map)?;
                    switches.remove(&key);
                }
            }
        }
        if enabled != Some(false) {
            self.inner.wake.notify_one();
            self.inner.imports_wake.notify_one();
        }
        self.emit(Event::PluginsUpdated);
        self.plugin_view(plugin, now_ms())
    }
}

/// `plugins.<插件>.methods.<方法>.enabled = enabled`；路上不是对象的一节换成对象，别的键留着。
fn set_switch(map: &mut Map<String, Value>, plugin: &str, method: &str, enabled: bool) {
    let mut node = map;
    for key in ["plugins", plugin, "methods", method] {
        let entry = node
            .entry(key.to_owned())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        let Value::Object(next) = entry else {
            unreachable!("just made an object")
        };
        node = next;
    }
    node.insert("enabled".into(), Value::Bool(enabled));
}

/// 去掉 `plugins.<插件>.methods.<方法>`；那个插件的 `methods` 空了也去掉，别的键留着。
fn clear_switch(map: &mut Map<String, Value>, plugin: &str, method: &str) {
    let Some(Value::Object(plugins)) = map.get_mut("plugins") else {
        return;
    };
    let Some(Value::Object(entry)) = plugins.get_mut(plugin) else {
        return;
    };
    let Some(Value::Object(methods)) = entry.get_mut("methods") else {
        return;
    };
    methods.remove(method);
    if methods.is_empty() {
        entry.remove("methods");
    }
    if entry.is_empty() {
        plugins.remove(plugin);
    }
}

/// 一种都用不上时告诉你的原因：要登录的照 `login-required`，要访客身份的照 `visitor-required`，
/// 别的照「这个链接现在取不了」。
fn blocker_failure(plugin: &FetchPlugin, m: &Method, b: &Blocker) -> CallFailure {
    let code = match b.code.as_str() {
        "login-missing" | "login-expired" => ErrorCode::LoginRequired,
        "visitor-missing" | "visitor-expired" => ErrorCode::VisitorRequired,
        _ => ErrorCode::UnsupportedInput,
    };
    let mut f = CallFailure::new(code, format!("{} {}: {}", plugin.id(), m.id, b.code));
    f.hint = Some(format!("{}：{}", m.name, b.hint));
    if m.account == Account::InUsuyo
        && let Some(site) = m.sites.first()
    {
        f.site = Some(site.as_str().into());
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(message: &str) -> FailureLine<'_> {
        FailureLine {
            at: "2026-11-12T00:00:00.000Z".into(),
            plugin: "a/b",
            method: "guest",
            host: Some("a.test"),
            code: "blocked",
            message,
            hint: None,
            detail: Some("<html>\nverify"),
        }
    }

    #[test]
    fn the_failure_log_is_one_line_a_call_and_bounded() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("logs/fetch-failures.jsonl");
        append_failure(&path, &line("first")).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text.lines().count(), 1);
        let v: Value = serde_json::from_str(text.trim()).unwrap();
        assert_eq!(v["detail"], "<html>\nverify");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        let big = "x".repeat(4096);
        while std::fs::metadata(&path).unwrap().len() <= FAILURE_LOG_MAX {
            append_failure(&path, &line(&big)).unwrap();
        }
        append_failure(&path, &line("after")).unwrap();
        assert!(std::fs::read_to_string(&path).unwrap().lines().count() == 1);
        assert!(path.with_extension("1.jsonl").is_file());
    }

    #[test]
    fn switches_are_read_and_bad_ones_refused() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("settings.json"),
            r#"{"plugins": {"a/b": {"methods": {"guest": {"enabled": false}, "embed": {}}}}}"#,
        )
        .unwrap();
        let m = Methods::load(dir.path()).unwrap();
        let s = m.switches.read().unwrap();
        assert_eq!(s.get(&("a/b".into(), "guest".into())), Some(&false));
        assert_eq!(s.get(&("a/b".into(), "embed".into())), None);
        drop(s);
        std::fs::write(
            dir.path().join("settings.json"),
            r#"{"plugins": {"a/b": {"methods": {"guest": {"enabled": "no"}}}}}"#,
        )
        .unwrap();
        assert!(Methods::load(dir.path()).is_err());
    }

    #[test]
    fn switch_is_written_under_its_plugin_and_keeps_other_keys() {
        let mut map: Map<String, Value> = serde_json::from_str(
            r#"{"plugins": {"a/b": {"settings": {"k": 1}}}, "network": {"proxy": "direct"}}"#,
        )
        .unwrap();
        set_switch(&mut map, "a/b", "guest", false);
        assert_eq!(
            Value::Object(map),
            serde_json::json!({
                "plugins": {"a/b": {"settings": {"k": 1}, "methods": {"guest": {"enabled": false}}}},
                "network": {"proxy": "direct"}
            })
        );
    }
}
