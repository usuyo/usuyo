//! 导入（协议 §3.8、§5.1）：列出你在平台上存过的，或收下交进来的一批链接，先给你看一眼，你确认了才按节奏去收。
//!
//! 一次导入走这几步，都记在 `jobs.db`，重启后接着：
//!
//! 1. **列**（`listing`）：`source` 类按插件的节奏一页页调 `list`；`delivered` 类一条条认身份。
//!    每条按身份分成新的、库里已经有的、在回收站里的；这一步什么都不建。
//! 2. **看一眼**（`ready`）：列完了，等你确认。
//! 3. **收**（`taking`）：你确认以后，你挑的（新的、回收站里的各一个整组的选择，加上逐条改过的）各排一个任务，按节奏一条条来：
//!    轮到时先收下（`origins[].entry: import`，同一批共用 `importBatch`），再取内容。
//! 4. **停下**（`held`）随时可以，接着导入从停的地方继续；**收完**（`done`）以后只剩几条事实。
//!
//! 列的时候被限流、被拦，等到点再列；要登录、插件改版这一类停在那里，等你处理完点「接着导入」。

use std::future::Future;
use std::time::Duration;

use usuyo_library::{ItemId, Whereabouts};
use usuyo_plugin::protocol::{ErrorCode, ListItem, Method, Pause};
use usuyo_plugin::{CallOptions, FetchPlugin, ListRequest, PluginError};

use crate::engine::Engine;
use crate::fetch::{now_ms, platform_of};
use crate::jobs::{EntryRow, ImportRow};
use crate::methods::blockers;
use crate::pace;
use crate::view::{
    Event, ImportEntryView, ImportKind, ImportState, ImportView, PreviewCounts, SourceView, Waiting,
};
use crate::{Error, Result};

/// 确认导入时你挑的：两组各一个整组的选择，`except` 是和自己那组选得不一样的条目（`seq`）。
/// 只交例外，不交整张清单：预览按页看，没翻到的条目跟着整组走。
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportChoice {
    /// 新的整组收不收。
    pub new: bool,
    /// 回收站里的整组收不收（收下就是拿回来）。
    pub trashed: bool,
    #[serde(default)]
    pub except: std::collections::BTreeSet<i64>,
}

/// 导入那一路每轮最多睡多久。
const IDLE: Duration = Duration::from_secs(30);
/// 交进来的一批每轮认多少条身份。
const RESOLVE_CHUNK: u32 = 50;
/// 一批最多交多少条。
pub const BATCH_MAX: usize = 5000;
/// 列一页失败（联网、超时）以后多久再试：`base × 2^次数`，试满就停下等你。
const LIST_RETRY_BASE: Duration = Duration::from_secs(60);
const LIST_RETRY_MAX: u32 = 6;
/// 按节奏等一次调用最多等多久；再长就记下时间，到点再来。
const PACE_WAIT_MAX: Duration = Duration::from_secs(60);

/// 交进来的一条链接（`/v1/batches` 的 `links[]`）。
#[derive(Debug, Clone)]
pub struct DeliveredLink {
    pub url: url::Url,
    pub title: Option<String>,
    /// RFC 3339；在边界上校验过。
    pub acted_at: Option<String>,
}

/// 一件事现在为什么不能做。`until` 为空是等你动手。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Wait {
    pub until: Option<i64>,
    pub code: String,
    pub hint: Option<String>,
}

fn ts(ms: i64) -> String {
    jiff::Timestamp::from_millisecond(ms)
        .map(usuyo_library::time::format)
        .unwrap_or_default()
}

impl Engine {
    /// 从一个插件一种方法的来源开始导入（先列，列完给你看）。
    pub fn start_import(
        &self,
        plugin_id: &str,
        method: &str,
        source: &str,
        title: &str,
    ) -> Result<ImportView> {
        let plugin = self.fetch_plugin(plugin_id)?;
        self.list_method(&plugin, method)?;
        if source.trim().is_empty() || source.len() > 512 {
            return Err(Error::InvalidArgs("source is empty or too long".into()));
        }
        self.batch_allowed(&plugin)?;
        let title = match title.trim() {
            "" => source.to_owned(),
            t => t.to_owned(),
        };
        let row = ImportRow {
            id: usuyo_library::uuid7(),
            kind: "source".into(),
            plugin: Some(plugin_id.to_owned()),
            method: Some(method.to_owned()),
            source: Some(source.to_owned()),
            client: None,
            title,
            action: None,
            created_at: now_ms(),
            cursor: None,
            listed: false,
            pages: 0,
            attempts: 0,
            wait_until: None,
            stop_code: None,
            stop_hint: None,
            held: false,
            confirmed_at: None,
            also_trashed: false,
            skipped: 0,
        };
        self.inner.jobs.insert_import(&row)?;
        self.inner.imports_wake.notify_one();
        self.import(&row.id)
    }

    /// 收下交进来的一批链接（`/v1/batches`）：同样先认身份、给你看一眼。
    pub fn deliver_batch(
        &self,
        name: &str,
        action: Option<&str>,
        links: Vec<DeliveredLink>,
        client: Option<&str>,
    ) -> Result<ImportView> {
        if links.is_empty() || links.len() > BATCH_MAX {
            return Err(Error::InvalidArgs(format!(
                "a batch has 1 to {BATCH_MAX} links"
            )));
        }
        let row = ImportRow {
            id: usuyo_library::uuid7(),
            kind: "delivered".into(),
            plugin: None,
            method: None,
            source: None,
            client: client.map(str::to_owned),
            title: match name.trim() {
                "" => "交进来的一批".to_owned(),
                n => n.chars().take(200).collect(),
            },
            action: action.map(str::to_owned),
            created_at: now_ms(),
            cursor: None,
            listed: false,
            pages: 0,
            attempts: 0,
            wait_until: None,
            stop_code: None,
            stop_hint: None,
            held: false,
            confirmed_at: None,
            also_trashed: false,
            skipped: 0,
        };
        self.inner.jobs.insert_import(&row)?;
        let entries: Vec<EntryRow> = links
            .into_iter()
            .map(|l| EntryRow {
                seq: 0,
                url: l.url.to_string(),
                title: l.title,
                action: action.map(str::to_owned),
                acted_at: l.acted_at,
                identity: None,
                item_id: None,
                standing: "unresolved".into(),
                added: false,
            })
            .collect();
        self.inner.jobs.add_entries(&row.id, &entries)?;
        self.inner.imports_wake.notify_one();
        self.import(&row.id)
    }

    pub fn imports(&self) -> Result<Vec<ImportView>> {
        self.inner
            .jobs
            .imports()?
            .into_iter()
            .map(|r| self.import_view(r))
            .collect()
    }

    pub fn import(&self, id: &str) -> Result<ImportView> {
        let row = self.import_row(id)?;
        self.import_view(row)
    }

    /// 预览里的条目；`standing` 是 `new`、`stored`、`trashed`、`unresolved` 之一，不给就是全部。
    pub fn import_entries(
        &self,
        id: &str,
        standing: Option<&str>,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<ImportEntryView>> {
        self.import_row(id)?;
        if let Some(s) = standing
            && !matches!(s, "new" | "stored" | "trashed" | "unresolved")
        {
            return Err(Error::InvalidArgs(format!("no standing `{s}`")));
        }
        Ok(self
            .inner
            .jobs
            .entries(id, standing, offset, limit.clamp(1, 500))?
            .into_iter()
            .map(|e| ImportEntryView {
                seq: e.seq,
                url: e.url,
                title: e.title,
                action: e.action,
                acted_at: e.acted_at,
                item_id: e.item_id,
                standing: e.standing,
                taken: e.added,
            })
            .collect())
    }

    /// 你看过了：按你挑的去收。新的、回收站里的各按整组的选择，`except` 里的条目和它那组反着来；
    /// 回收站里的收下就是拿回来。已经存着的、认不出的不收。
    pub fn confirm_import(&self, id: &str, choice: &ImportChoice) -> Result<ImportView> {
        let row = self.import_row(id)?;
        if !row.listed {
            return Err(Error::InvalidArgs("this import is still listing".into()));
        }
        if row.confirmed_at.is_some() {
            return Err(Error::InvalidArgs(
                "this import is already confirmed".into(),
            ));
        }
        let now = now_ms();
        let mut offset = 0;
        let mut wanted = Vec::new();
        let mut except_seen = 0;
        loop {
            let page = self.inner.jobs.entries(id, None, offset, 500)?;
            if page.is_empty() {
                break;
            }
            offset += 500;
            for e in page {
                let group = match e.standing.as_str() {
                    "new" => choice.new,
                    "trashed" => choice.trashed,
                    _ => continue,
                };
                let flipped = choice.except.contains(&e.seq);
                except_seen += usize::from(flipped);
                if group != flipped {
                    wanted.push(e);
                }
            }
        }
        if except_seen != choice.except.len() {
            return Err(Error::InvalidArgs(
                "`except` names entries that are neither new nor in the trash".into(),
            ));
        }
        // 挑了回收站里的，轮到时就拿回来；一条都没挑，轮到时已经进了回收站的照旧不收。
        let also_trashed = wanted.iter().any(|e| e.standing == "trashed");
        for e in &wanted {
            if let Some(item) = &e.item_id {
                self.inner.jobs.enqueue_import(item, &e.url, id, now)?;
            }
        }
        self.inner.jobs.confirm_import(id, now, also_trashed)?;
        if row.held {
            self.inner.jobs.set_import_held(id, true)?;
        }
        self.inner.wake.notify_one();
        self.emit(Event::ImportUpdated { id: id.to_owned() });
        self.import(id)
    }

    /// 先停下：不再列、不再收，已经收下的不动。
    pub fn hold_import(&self, id: &str) -> Result<ImportView> {
        self.import_row(id)?;
        self.inner.jobs.set_import_held(id, true)?;
        self.emit(Event::ImportUpdated { id: id.to_owned() });
        self.import(id)
    }

    /// 接着导入：停下的从停的地方继续；停在要你处理的原因上（例如重新登录了）也从这里再试。
    pub fn resume_import(&self, id: &str) -> Result<ImportView> {
        self.import_row(id)?;
        self.inner.jobs.set_import_held(id, false)?;
        self.inner.imports_wake.notify_one();
        self.inner.wake.notify_one();
        self.emit(Event::ImportUpdated { id: id.to_owned() });
        self.import(id)
    }

    /// 这次导入不要了：清单和还没收的一起去掉；已经收下的条目留着。
    pub fn delete_import(&self, id: &str) -> Result<()> {
        self.import_row(id)?;
        self.inner.jobs.delete_import(id)?;
        self.emit(Event::ImportUpdated { id: id.to_owned() });
        Ok(())
    }

    /// 一个插件一种方法现在能列的来源（`sources`，计入节奏）。`link` 是你给的一条链接，
    /// 只给声明了 `list.link` 的方法（协议 §2.3.1）。
    pub async fn plugin_sources(
        &self,
        plugin_id: &str,
        method: &str,
        link: Option<&url::Url>,
    ) -> Result<Vec<SourceView>> {
        let plugin = self.fetch_plugin(plugin_id)?;
        let m = self.list_method(&plugin, method)?;
        if link.is_some()
            && m.capabilities
                .list
                .as_ref()
                .is_some_and(|l| l.link.is_none())
        {
            return Err(Error::UnsupportedInput(format!(
                "{plugin_id} {method} does not start from a link"
            )));
        }
        let platform = pace_scope(&plugin);
        let host = &self.inner.host;
        let r = match self
            .paced(
                &platform,
                false,
                PACE_WAIT_MAX,
                host.sources(&plugin, method, link, &CallOptions::default()),
            )
            .await?
        {
            Ok(r) => r,
            Err(at) => {
                return Err(Error::UnsupportedInput(format!(
                    "this platform is pacing calls; try again after {}",
                    ts(at)
                )));
            }
        };
        let link_host = link.and_then(|l| l.host_str());
        let sources = self.after_call(&plugin, method, &platform, link_host, r)?;
        Ok(sources
            .into_iter()
            .map(|source| SourceView {
                source,
                method: method.to_owned(),
            })
            .collect())
    }

    /// 这种方法能列：做了、在插件进程里跑、有 `list`、开着、这台机器上用得上。
    fn list_method(&self, plugin: &FetchPlugin, method: &str) -> Result<Method> {
        let m = plugin
            .spec
            .method(method)
            .filter(|m| m.callable() && m.capabilities.list.is_some())
            .ok_or_else(|| {
                Error::UnsupportedInput(format!(
                    "{} has no method `{method}` that lists what you saved",
                    plugin.id()
                ))
            })?;
        let enabled = self.inner.methods.enabled(plugin, m);
        let health = self.inner.methods.health_seen(plugin.id().as_str());
        let host = &self.inner.host;
        if let Some(b) = blockers(
            m,
            enabled,
            health.as_ref(),
            &host.sites()?,
            &host.visitors()?,
        )
        .into_iter()
        .next()
        {
            return Err(Error::UnsupportedInput(format!("{}：{}", m.name, b.hint)));
        }
        Ok(m.clone())
    }

    /// 记下这次调用；插件报的错要暂停这种方法的，照记下（和取内容时一样）。
    fn after_call<T>(
        &self,
        plugin: &FetchPlugin,
        method: &str,
        scope: &str,
        link_host: Option<&str>,
        r: std::result::Result<T, PluginError>,
    ) -> Result<T> {
        match &r {
            Ok(_) => self.record_method_call(plugin, method, link_host, Ok(()))?,
            Err(PluginError::Call { failure, .. }) => {
                self.record_method_call(plugin, method, link_host, Err(failure))?;
                self.pause_for(plugin, method, scope, failure)?;
            }
            Err(_) => {}
        }
        Ok(r?)
    }

    /// 被限流、被拦：这种方法在这里暂停（被拦也按最长的暂停，到点自己解开）。回暂停到什么时候。
    pub(crate) fn pause_for(
        &self,
        plugin: &FetchPlugin,
        method: &str,
        scope: &str,
        failure: &usuyo_plugin::protocol::CallFailure,
    ) -> Result<Option<i64>> {
        let d = match failure.pause() {
            Some(Pause::For(d)) => d,
            Some(Pause::UntilResumed) => usuyo_plugin::protocol::MAX_PAUSE,
            None => return Ok(None),
        };
        let until = now_ms().saturating_add(i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
        self.inner.jobs.pause(
            plugin.id().as_str(),
            method,
            scope,
            until,
            failure.code.as_str(),
            failure.hint.as_deref(),
        )?;
        self.emit(Event::PluginsUpdated);
        Ok(Some(until))
    }

    /// 在这个平台上按节奏调一次：拿平台的锁，等到间隔够了再调，调完记账。
    /// 要等的比 `max_wait` 长就不调，回 `Err(最早什么时候能)`。
    async fn paced<T>(
        &self,
        platform: &str,
        batch: bool,
        max_wait: Duration,
        call: impl Future<Output = T>,
    ) -> Result<std::result::Result<T, i64>> {
        let lock = self.inner.locks.get(platform);
        let _guard = lock.lock().await;
        let pacing = self.inner.pacing.get(platform);
        if let Some(at) = pace::ready_at(pacing, &self.inner.jobs, platform, batch, now_ms())? {
            let wait = Duration::from_millis(u64::try_from(at - now_ms()).unwrap_or(0));
            if wait > max_wait {
                return Ok(Err(at));
            }
            tokio::time::sleep(wait).await;
        }
        let out = call.await;
        self.inner
            .jobs
            .record_call(platform, now_ms(), &pace::today(), batch)?;
        Ok(Ok(out))
    }

    fn fetch_plugin(&self, id: &str) -> Result<FetchPlugin> {
        self.inner
            .host
            .plugin(id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("plugin {id}")))
    }

    fn import_row(&self, id: &str) -> Result<ImportRow> {
        self.inner
            .jobs
            .import(id)?
            .ok_or_else(|| Error::NotFound(format!("import {id}")))
    }

    fn batch_allowed(&self, plugin: &FetchPlugin) -> Result<()> {
        let off = plugin
            .spec
            .platforms
            .iter()
            .any(|p| self.inner.pacing.get(p.as_str()).is_some_and(|x| !x.batch));
        if off {
            return Err(Error::UnsupportedInput(
                "this platform takes items one at a time (pacing.batch: false)".into(),
            ));
        }
        Ok(())
    }

    fn import_view(&self, r: ImportRow) -> Result<ImportView> {
        let now = now_ms();
        let counts = self.inner.jobs.entry_counts(&r.id)?;
        let (live, failed) = self.inner.jobs.import_jobs(&r.id)?;
        let state = if r.held {
            ImportState::Held
        } else if !r.listed {
            ImportState::Listing
        } else if r.confirmed_at.is_none() {
            ImportState::Ready
        } else if live > 0 {
            ImportState::Taking
        } else {
            ImportState::Done
        };
        let mut waiting = None;
        let mut next_at = None;
        match state {
            ImportState::Listing => {
                if let Some(code) = &r.stop_code {
                    waiting = Some(Waiting {
                        code: code.clone(),
                        hint: r.stop_hint.clone(),
                        until: r.wait_until.map(ts),
                    });
                } else if let Some(w) = self.listing_wait(&r, now)? {
                    next_at = w.until.map(ts);
                    waiting = Some(Waiting {
                        code: w.code,
                        hint: w.hint,
                        until: w.until.map(ts),
                    });
                }
            }
            ImportState::Taking => {
                if let Some(job) = self.inner.jobs.import_first_job(&r.id)? {
                    match self.wait_for(&job, now)? {
                        Some(w) => {
                            next_at = w.until.map(ts);
                            waiting = Some(Waiting {
                                code: w.code,
                                hint: w.hint,
                                until: w.until.map(ts),
                            });
                        }
                        None if job.due_at > now => next_at = Some(ts(job.due_at)),
                        None => {}
                    }
                }
            }
            _ => {}
        }
        Ok(ImportView {
            id: r.id,
            kind: if r.kind == "source" {
                ImportKind::Source
            } else {
                ImportKind::Delivered
            },
            title: r.title,
            plugin_id: r.plugin,
            method: r.method,
            source: r.source,
            client: r.client,
            created_at: ts(r.created_at),
            state,
            pages: r.pages,
            preview: PreviewCounts {
                new: counts.new,
                stored: counts.stored,
                trashed: counts.trashed,
            },
            taken: counts.added,
            not_stored: failed,
            skipped: r.skipped,
            next_at,
            waiting,
        })
    }

    /// 列的这一步现在为什么不能做。
    fn listing_wait(&self, r: &ImportRow, now: i64) -> Result<Option<Wait>> {
        if let Some(at) = r.wait_until.filter(|t| *t > now) {
            return Ok(Some(Wait {
                until: Some(at),
                code: "pacing".into(),
                hint: None,
            }));
        }
        let Some(plugin) = r.plugin.as_deref().and_then(|p| self.inner.host.plugin(p)) else {
            return Ok(None);
        };
        let scope = pace_scope(plugin);
        let method = r.method.as_deref().unwrap_or_default();
        if let Some(p) = self
            .inner
            .jobs
            .paused(plugin.id().as_str(), method, &scope, now)?
        {
            return Ok(Some(pause_wait(&p)));
        }
        let pacing = self.inner.pacing.get(scope.as_str());
        match pace::ready_at(pacing, &self.inner.jobs, &scope, true, now)? {
            Some(at) => Ok(Some(Wait {
                until: Some(at),
                code: pace_code(pacing, &self.inner.jobs, &scope)?,
                hint: None,
            })),
            None => Ok(None),
        }
    }

    // —— 后台：列和认身份 ——

    pub(crate) async fn run_imports(&self) {
        loop {
            let wake = match self.advance_imports().await {
                Ok((_, wake)) => wake,
                Err(e) => {
                    tracing::error!(error = %e, "imports");
                    None
                }
            };
            let now = now_ms();
            let wait = wake.map_or(IDLE, |at| {
                Duration::from_millis(u64::try_from(at - now).unwrap_or(0)).min(IDLE)
            });
            tokio::select! {
                () = self.inner.imports_wake.notified() => {}
                () = tokio::time::sleep(wait) => {}
            }
        }
    }

    /// 每个要列（或要认身份）的导入往前走一步。回走了几步，和还在等的最早什么时候能再走。
    pub async fn advance_imports(&self) -> Result<(usize, Option<i64>)> {
        let mut steps = 0;
        let mut wake: Option<i64> = None;
        let mut note = |at: i64| wake = Some(wake.map_or(at, |w: i64| w.min(at)));
        for r in self.inner.jobs.imports()? {
            if r.held || r.listed || (r.stop_code.is_some() && r.wait_until.is_none()) {
                continue;
            }
            let now = now_ms();
            if let Some(at) = r.wait_until.filter(|t| *t > now) {
                note(at);
                continue;
            }
            let step = if r.kind == "source" {
                self.list_page(&r).await
            } else {
                self.resolve_some(&r).await
            };
            steps += 1;
            // 一个导入出错（资料库写不进一类）停在它自己身上，别的导入照走
            let next = match step {
                Ok(next) => next,
                Err(e) => {
                    tracing::warn!(import = %r.id, error = %e, "import stopped");
                    self.inner
                        .jobs
                        .import_stop(&r.id, e.code(), Some(&e.to_string()))?;
                    None
                }
            };
            self.emit(Event::ImportUpdated { id: r.id.clone() });
            if let Some(at) = next {
                note(at);
            }
        }
        Ok((steps, wake))
    }

    /// 列一页。回下一步最早什么时候能做（还有页要列时）。
    async fn list_page(&self, r: &ImportRow) -> Result<Option<i64>> {
        let jobs = &self.inner.jobs;
        let Some(plugin) = r
            .plugin
            .as_deref()
            .and_then(|p| self.inner.host.plugin(p))
            .cloned()
        else {
            jobs.import_stop(&r.id, "unsupported-input", Some("这个插件不在了"))?;
            return Ok(None);
        };
        let scope = pace_scope(&plugin);
        let now = now_ms();
        let method = r.method.clone().unwrap_or_default();
        if let Some(p) = jobs.paused(plugin.id().as_str(), &method, &scope, now)? {
            // 停着的方法不列；到点（或你点了「现在再试」）再来
            return Ok(p.until);
        }
        let request = ListRequest {
            method: method.clone(),
            source: r.source.clone().unwrap_or_default(),
            cursor: r.cursor.clone(),
            since: None,
            input_file: None,
        };
        let host = &self.inner.host;
        let result = match self
            .paced(
                &scope,
                true,
                PACE_WAIT_MAX,
                host.list(&plugin, request, &CallOptions::default()),
            )
            .await?
        {
            Ok(result) => result,
            Err(at) => {
                jobs.import_wait(
                    &r.id,
                    at,
                    false,
                    &pace_code(self.inner.pacing.get(scope.as_str()), jobs, &scope)?,
                    None,
                )?;
                return Ok(Some(at));
            }
        };
        let page = match result {
            Ok(page) => {
                self.record_method_call(&plugin, &method, None, Ok(()))?;
                page
            }
            Err(PluginError::Call { failure, .. }) => {
                self.record_method_call(&plugin, &method, None, Err(&failure))?;
                // 被限流、被拦：只记在这种方法的暂停上（导入照它显示原因），「现在再试」一去掉就接着列
                if let Some(until) = self.pause_for(&plugin, &method, &scope, &failure)? {
                    return Ok(Some(until));
                }
                let transient = matches!(
                    failure.code,
                    ErrorCode::Network | ErrorCode::Timeout | ErrorCode::Crashed
                );
                if transient && r.attempts + 1 < LIST_RETRY_MAX {
                    let delay = LIST_RETRY_BASE.saturating_mul(2u32.saturating_pow(r.attempts));
                    let at = now_ms()
                        .saturating_add(i64::try_from(delay.as_millis()).unwrap_or(i64::MAX));
                    jobs.import_wait(
                        &r.id,
                        at,
                        true,
                        failure.code.as_str(),
                        failure.hint.as_deref(),
                    )?;
                    return Ok(Some(at));
                }
                jobs.import_stop(
                    &r.id,
                    failure.code.as_str(),
                    Some(failure.hint.as_deref().unwrap_or(&failure.message)),
                )?;
                return Ok(None);
            }
            Err(e) => {
                jobs.import_stop(&r.id, Error::from(e).code(), None)?;
                return Ok(None);
            }
        };
        self.add_listed(&r.id, &plugin, page.items).await?;
        let next = page.next_cursor.filter(|c| !c.is_empty());
        let last = next.is_none();
        jobs.listed_page(&r.id, next.as_deref())?;
        if last {
            return Ok(None);
        }
        let pacing = self.inner.pacing.get(scope.as_str());
        Ok(pace::ready_at(pacing, jobs, &scope, true, now_ms())?.or(Some(now_ms())))
    }

    /// 一页列出来的：认身份（插件给了就用，没给就 canonicalize），分好放进清单。
    async fn add_listed(
        &self,
        import: &str,
        plugin: &FetchPlugin,
        items: Vec<ListItem>,
    ) -> Result<()> {
        let mut ready = Vec::new();
        for item in items {
            let Some(raw) = item.url.as_deref() else {
                // 只有 `ref` 的条目（导入插件）还不支持
                tracing::warn!(plugin = %plugin.id(), "list item without url skipped");
                continue;
            };
            let url = url::Url::parse(raw)
                .map_err(|e| Error::InvalidArgs(format!("list item url: {e}")))?;
            let (identity, fetch_url) = match &item.identity {
                Some(i) if !matches!(i, usuyo_plugin::protocol::Identity::File) => {
                    let identity: usuyo_contract::Identity =
                        serde_json::from_value(serde_json::to_value(i)?)?;
                    (identity, url.to_string())
                }
                _ => self.identify(&url).await?,
            };
            ready.push((
                identity,
                EntryRow {
                    seq: 0,
                    url: fetch_url,
                    title: item.title,
                    action: item
                        .action
                        .map(serde_json::to_value)
                        .transpose()?
                        .and_then(|v| v.as_str().map(str::to_owned)),
                    acted_at: item.acted_at.map(|t| t.as_str().to_owned()),
                    identity: None,
                    item_id: None,
                    standing: String::new(),
                    added: false,
                },
            ));
        }
        self.classify_and_store(import, ready).await
    }

    /// 交进来的一批：认一段身份。回下一步最早什么时候（还有没认的就是马上）。
    async fn resolve_some(&self, r: &ImportRow) -> Result<Option<i64>> {
        let pending = self
            .inner
            .jobs
            .entries(&r.id, Some("unresolved"), 0, RESOLVE_CHUNK)?;
        if pending.is_empty() {
            self.inner.jobs.mark_listed(&r.id)?;
            return Ok(None);
        }
        let mut ready = Vec::new();
        for e in pending {
            // 身份由宿主重新算，不信交来的（协议 §5.1）；认不出的这一条去掉，记日志
            let found = match url::Url::parse(&e.url) {
                Ok(url) => self.identify(&url).await,
                Err(err) => Err(Error::InvalidArgs(format!("batch link: {err}"))),
            };
            match found {
                Ok((identity, fetch_url)) => ready.push((
                    identity,
                    EntryRow {
                        url: fetch_url,
                        ..e
                    },
                )),
                Err(err) => {
                    tracing::warn!(import = %r.id, seq = e.seq, error = %err, "batch link dropped");
                    self.inner.jobs.drop_entry(&r.id, e.seq)?;
                }
            }
        }
        self.classify_and_store(&r.id, ready).await?;
        Ok(Some(now_ms()))
    }

    /// 按身份算出条目 id，问资料库各在哪，去掉清单里重复的，记进清单。
    async fn classify_and_store(
        &self,
        import: &str,
        ready: Vec<(usuyo_contract::Identity, EntryRow)>,
    ) -> Result<()> {
        if ready.is_empty() {
            return Ok(());
        }
        let ids = ready
            .iter()
            .map(|(i, _)| ItemId::from_identity(i).map_err(Error::from))
            .collect::<Result<Vec<_>>>()?;
        let this = self.clone();
        let lookup = ids.clone();
        let places = tokio::task::spawn_blocking(move || this.lib().whereabouts(&lookup))
            .await
            .map_err(|e| Error::Task(e.to_string()))??;
        for (((identity, mut entry), id), place) in ready.into_iter().zip(ids).zip(places) {
            let (item, standing) = match place {
                Whereabouts::Stored(target) => (target, "stored"),
                Whereabouts::Trashed(_) => (id, "trashed"),
                Whereabouts::Absent(_) => (id, "new"),
            };
            // 交进来的一批里已经在清单上的（`seq` 不是 0）就地改，列出来的新加
            if self.inner.jobs.has_entry_for(import, item.as_str())? {
                if entry.seq != 0 {
                    self.inner.jobs.drop_entry(import, entry.seq)?;
                }
                continue;
            }
            entry.identity = Some(serde_json::to_string(&identity)?);
            entry.item_id = Some(item.to_string());
            entry.standing = standing.into();
            if entry.seq == 0 {
                self.inner.jobs.add_entry(import, &entry)?;
            } else {
                self.inner.jobs.resolve_entry(import, &entry)?;
            }
        }
        Ok(())
    }
}

/// 节奏按插件的第一个平台算（一个获取插件的 `list` 只列一个平台的）。
fn pace_scope(plugin: &FetchPlugin) -> String {
    plugin
        .spec
        .platforms
        .first()
        .map(|p| p.as_str().to_owned())
        .unwrap_or_default()
}

/// 等的是间隔，还是今天批量的次数用完了。
fn pace_code(
    pacing: Option<&pace::Pacing>,
    jobs: &crate::jobs::Jobs,
    platform: &str,
) -> Result<String> {
    let row = jobs.pace(platform)?;
    let daily = pacing
        .and_then(|p| p.daily)
        .is_some_and(|d| row.day == pace::today() && row.batch_calls >= d);
    Ok(if daily { "daily-limit" } else { "pacing" }.into())
}

pub(crate) fn pause_wait(p: &crate::jobs::PauseRow) -> Wait {
    if p.held {
        Wait {
            until: None,
            code: "held".into(),
            hint: None,
        }
    } else {
        Wait {
            until: p.until,
            code: p.code.clone().unwrap_or_else(|| "paused".into()),
            hint: p.hint.clone(),
        }
    }
}

impl Engine {
    /// 一个任务现在为什么不能做。节奏人人守（手动收下的只守间隔）；
    /// 导入的任务另外等插件的暂停，不白白耗掉重试次数。
    pub(crate) fn wait_for(&self, job: &crate::jobs::FetchJob, now: i64) -> Result<Option<Wait>> {
        let platform = platform_of(&job.item_id);
        let batch = job.import_id.is_some();
        let pacing = self.inner.pacing.get(platform);
        if batch && pacing.is_some_and(|p| !p.batch) {
            return Ok(Some(Wait {
                until: None,
                code: "batch-off".into(),
                hint: None,
            }));
        }
        if let Some(at) = pace::ready_at(pacing, &self.inner.jobs, platform, batch, now)? {
            return Ok(Some(Wait {
                until: Some(at),
                code: pace_code(pacing, &self.inner.jobs, platform)?,
                hint: None,
            }));
        }
        if !batch {
            return Ok(None);
        }
        let Ok(url) = url::Url::parse(&job.url) else {
            return Ok(None);
        };
        let mut found: Option<Wait> = None;
        let fetching = self
            .inner
            .host
            .candidates(&url)
            .into_iter()
            .flat_map(|plugin| {
                plugin
                    .spec
                    .methods
                    .iter()
                    .filter(|m| m.callable() && m.capabilities.fetch.is_some())
                    .filter(|m| self.inner.methods.enabled(plugin, m))
                    .map(move |m| (plugin, m))
            });
        for (plugin, method) in fetching {
            let scope = if plugin.spec.is_web() {
                url.host_str().unwrap_or_default()
            } else {
                platform
            };
            match self
                .inner
                .jobs
                .paused(plugin.id().as_str(), &method.id, scope, now)?
            {
                None => return Ok(None),
                Some(p) => {
                    let w = pause_wait(&p);
                    // 几个都停着：报最早能再试的那个，等你动手的排最后
                    let sooner = match (&found, w.until) {
                        (None, _) => true,
                        (Some(f), Some(t)) => f.until.is_none_or(|u| t < u),
                        (Some(_), None) => false,
                    };
                    if sooner {
                        found = Some(w);
                    }
                }
            }
        }
        Ok(found)
    }
}
