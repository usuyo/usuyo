//! 收下和后台取内容。
//!
//! 收下：认身份 → 马上建条目（只有链接）→ 排一个取内容的任务。
//! 任务：按插件顺序 fetch（平台插件在前、通用网页在后）；能换下一个的错就换，
//! 暂停照插件报的记下；都没取到、又有能重试的，按退避再排；取到了交给资料库当快照。

use std::collections::BTreeSet;
use std::time::Duration;

use serde_json::Value;
use usuyo_contract::FetchRecord;
use usuyo_library::{
    AddOutcome, Capture, ItemId, NewOrigin, NewSnapshot, Whereabouts, hash_staged,
    normalize_web_url,
};
use usuyo_plugin::protocol::{
    CallFailure, Canonicalized, ErrorCode, FetchOutput, MAX_PAUSE, OutputKind, Pause,
};
use usuyo_plugin::{
    CallOptions, FetchInput, FetchPlugin, FetchRequest, FetchedSnapshot, PluginError,
};

use crate::engine::{Engine, Via};
use crate::jobs::FetchJob;
use crate::pace;
use crate::view::{Event, TakeResponse};
use crate::{Error, Result};

/// 后台任务每轮最多睡多久；有新任务时会被叫醒。
const IDLE: Duration = Duration::from_secs(30);

/// 同一条里换方法、补缺时，两次调用之间最多等多久的间隔；再长就留到下一轮。
const PACE_BETWEEN_MAX: Duration = Duration::from_secs(60);

pub(crate) fn now_ms() -> i64 {
    jiff::Timestamp::now().as_millisecond()
}

impl Engine {
    /// 收下一条链接（或带链接的分享文字）。马上回；内容在后台取。
    pub async fn take(&self, input: &str, kind: &str, via: Via) -> Result<TakeResponse> {
        if kind != "link" {
            return Err(Error::UnsupportedInput(format!(
                "taking a `{kind}` is not supported yet"
            )));
        }
        let url = crate::share_text::link_in(input).ok_or_else(|| {
            Error::UnsupportedInput("no http(s) link in the input".into())
                .hinted("这段文字里没有能收下的链接")
        })?;
        // 带用户名、密码的链接不收：它们会原样进标题、身份和 `content.md`。
        if !url.username().is_empty() || url.password().is_some() {
            let more = if crate::share_text::link_count(input) > 1 {
                "（只看第一条链接）"
            } else {
                ""
            };
            return Err(Error::UnsupportedInput(
                "links with a user name or password are not taken".into(),
            )
            .hinted(format!("带用户名或密码的链接不收{more}")));
        }
        let (identity, fetch_url) = self.identify(&url).await?;
        let this = self.clone();
        tokio::task::spawn_blocking(move || this.add(identity, fetch_url, via))
            .await
            .map_err(|e| Error::Task(e.to_string()))?
    }

    /// 身份：插件 canonicalize 的第一个命中；都不认，或因联网、超时失败，就照 `web:` 自己算（SPEC §0.4）。
    pub(crate) async fn identify(
        &self,
        url: &url::Url,
    ) -> Result<(usuyo_contract::Identity, String)> {
        let host = &self.inner.host;
        for plugin in host.candidates(url) {
            match host
                .canonicalize(plugin, url, &CallOptions::default())
                .await
            {
                Ok(Canonicalized::Match {
                    identity,
                    resolved_url,
                }) => {
                    let identity = serde_json::from_value(serde_json::to_value(&identity)?)?;
                    return Ok((identity, resolved_url.unwrap_or_else(|| url.to_string())));
                }
                Ok(Canonicalized::NoMatch) => {}
                Err(PluginError::Call { failure, .. })
                    if matches!(failure.code, ErrorCode::Network | ErrorCode::Timeout) =>
                {
                    break;
                }
                Err(e) => tracing::warn!(plugin = %plugin.id(), error = %e, "canonicalize failed"),
            }
        }
        let identity = usuyo_contract::Identity {
            platform: "web".into(),
            content_id: None,
            url: Some(normalize_web_url(url.as_str())?),
            extra: Default::default(),
        };
        Ok((identity, url.to_string()))
    }

    fn add(
        &self,
        identity: usuyo_contract::Identity,
        fetch_url: String,
        via: Via,
    ) -> Result<TakeResponse> {
        let mut lib = self.lib();
        let outcome = lib.add_item(
            &via.batch()?,
            Capture {
                identity: Some(identity),
                title_hint: None,
                origin: NewOrigin {
                    entry: via.entry().into(),
                    taken_by: "user".into(),
                    ..NewOrigin::default()
                },
                snapshot: None,
                // 你自己在窗口和终端里收的，私密的也照收；别的程序交来的跳过（SPEC §1.7 的表）。
                skip_if_private: via.skips_private(),
            },
        )?;
        let id = outcome.id().clone();
        if !matches!(outcome, AddOutcome::Skipped(_)) {
            self.reindex_item(&lib, &id)?;
            self.inner.jobs.enqueue(id.as_str(), &fetch_url, now_ms())?;
            self.inner.wake.notify_one();
        }
        let item = lib.item(&id)?;
        drop(lib);
        self.emit(match outcome {
            AddOutcome::Created(_) => Event::ItemAdded { id: id.to_string() },
            _ => Event::ItemUpdated { id: id.to_string() },
        });
        let s = self.summary(&item.manifest);
        Ok(TakeResponse {
            id: s.id,
            seen: !matches!(outcome, AddOutcome::Created(_)),
            title: s.title,
            platform: s.platform,
            taken_at: s.taken_at,
        })
    }

    /// 一直跑后台的事：取内容、列导入、收件文件夹、回收站到期、转换。有新任务时 [`Engine::take`] 会叫醒取内容那一路。
    pub async fn run_worker(&self) {
        tokio::join!(
            self.run_fetches(),
            self.run_imports(),
            self.run_inbox(),
            self.run_expiry(),
            self.run_transforms()
        );
    }

    async fn run_fetches(&self) {
        loop {
            let blocked = match self.run_ready().await {
                Ok((_, wake)) => wake,
                Err(e) => {
                    tracing::error!(error = %e, "fetch jobs");
                    None
                }
            };
            let now = now_ms();
            let next = match self.inner.jobs.next_due_after(now) {
                Ok(at) => at,
                Err(e) => {
                    tracing::error!(error = %e, "jobs database");
                    None
                }
            };
            let wait = [next, blocked]
                .into_iter()
                .flatten()
                .min()
                .map_or(IDLE, |at| until(at, now).min(IDLE));
            tokio::select! {
                () = self.inner.wake.notified() => {}
                () = tokio::time::sleep(wait) => {}
            }
        }
    }

    /// 把现在能做的任务一个个做完，回做了几个。等节奏、等暂停的留着，后台到点再做。
    pub async fn run_due(&self) -> Result<usize> {
        Ok(self.run_ready().await?.0)
    }

    /// 同 [`Engine::run_due`]，另回还在等的任务最早什么时候能做。
    async fn run_ready(&self) -> Result<(usize, Option<i64>)> {
        let mut n = 0;
        loop {
            let now = now_ms();
            let mut wake: Option<i64> = None;
            let mut picked = None;
            for job in self.inner.jobs.due(now)? {
                if let Some(at) = self.blocked(&job, now)? {
                    if at != i64::MAX {
                        wake = Some(wake.map_or(at, |w| w.min(at)));
                    }
                    continue;
                }
                let platform = platform_of(&job.item_id).to_owned();
                // 这个平台正在跑别的调用（例如导入在列清单）：过一秒再看。
                let Ok(guard) = self.inner.locks.get(&platform).try_lock_owned() else {
                    wake = Some(wake.map_or(now + 1000, |w| w.min(now + 1000)));
                    continue;
                };
                if self.inner.jobs.claim(&job.item_id)? {
                    picked = Some((job, platform, guard));
                    break;
                }
            }
            let Some((job, platform, _guard)) = picked else {
                return Ok((n, wake));
            };
            n += 1;
            match self.run_job(&job, &platform).await {
                Ok(()) => {}
                Err(e) => {
                    // 资料库这边出的错（写不进、条目没了）：不再自动重试，原因记在任务上。
                    tracing::error!(item = %job.item_id, error = %e, "fetch job");
                    self.inner
                        .jobs
                        .fail(&job.item_id, e.code(), Some(&e.to_string()))?;
                    self.emit(Event::ItemFetchFailed {
                        id: job.item_id.clone(),
                        code: e.code().into(),
                        hint: None,
                        retry: false,
                    });
                }
            }
            if let Some(import) = &job.import_id {
                self.drop_copy_if_done(import)?;
                self.emit(Event::ImportUpdated { id: import.clone() });
            }
        }
    }

    /// 这个任务现在不能做：回最早什么时候能（`i64::MAX` 是等你动手）。
    fn blocked(&self, job: &FetchJob, now: i64) -> Result<Option<i64>> {
        Ok(self
            .wait_for(job, now)?
            .map(|w| w.until.unwrap_or(i64::MAX)))
    }

    /// 取一条（协议 §2.3.1）：照排好的方法一种种试。第一种取到的要是还缺几样，往后找声明了那几样的方法，
    /// 只要那几样再取，直到不缺或没有方法了；每次取到的都是一份快照。前一种回 `rate-limited`、`network`
    /// 时照节奏等、不换，回 `cancelled` 停下，别的错换下一种。这个平台的锁已经拿着；每次调用都记进节奏，
    /// 两次调用之间照 `interval` 等。
    async fn run_job(&self, job: &FetchJob, platform: &str) -> Result<()> {
        if let Some(import) = job.import_id.clone() {
            let this = self.clone();
            let j = job.clone();
            let imp = import.clone();
            let go = tokio::task::spawn_blocking(move || this.take_entry(&imp, &j))
                .await
                .map_err(|e| Error::Task(e.to_string()))??;
            if !go {
                return Ok(());
            }
            if let Some(by_ref) = self.ref_fetch(&import, job)? {
                return self.run_ref_job(job, by_ref).await;
            }
        }
        let id = ItemId::parse(&job.item_id)?;
        let url = url::Url::parse(&job.url)
            // 链接可能带分享参数，原因里不写它：这句会进日志和 `jobs.db`
            .map_err(|e| Error::InvalidArgs(format!("bad job url: {e}")))?;
        let batch = job.import_id.is_some();
        let (candidates, unusable) = self.fetch_candidates(&url, platform).await?;

        // 都没取到时条目上写谁的原因：为这条链接写的平台方法里第一个失败的；通用网页的失败只进失败日志，
        // 平台方法一种都没调成（都用不上）时才轮到它。
        let mut first_platform: Option<CallFailure> = None;
        let mut first_web: Option<CallFailure> = None;
        // 停着、这一轮没调的平台方法：调过的都没失败时才拿它当原因
        let mut paused_note: Option<CallFailure> = None;
        let mut retryable = false;
        let mut paused_any = false;
        let mut called = false;
        // 平台方法调过没有；有没有哪一种不是回「这种链接我不取」（被拦、要登录、要访客身份、没找到、停着……）
        let mut platform_called = false;
        let mut platform_refused = false;
        // 取到第一份以后还缺的；`None` 是还没取到。
        let mut missing: Option<BTreeSet<OutputKind>> = None;
        for c in &candidates {
            let (plugin, method) = (&c.plugin, &c.method);
            let pid = plugin.id().to_string();
            let Some(cap) = method.capabilities.fetch.as_ref() else {
                continue;
            };
            let want = match &missing {
                None => None,
                // 补缺的不找通用网页插件：它的快照是另一个身份的网页。
                Some(_) if plugin.spec.is_web() => continue,
                Some(m) => {
                    let w: BTreeSet<OutputKind> = m.intersection(&cap.outputs).copied().collect();
                    if w.is_empty() {
                        continue;
                    }
                    Some(w)
                }
            };
            // 平台插件认了这条链接（条目的平台不是 web）：通用网页插件只在平台方法都说「这种链接我不取」
            // 时才上。平台方法被拦、要登录、没找到、停着的，不再用通用网页去请求同一个地址：
            // 那只会再撞一次校验页，还把校验页存成一份快照。
            if plugin.spec.is_web() && platform != "web" && (!platform_called || platform_refused) {
                continue;
            }
            let scope = pause_scope(plugin, &url, platform);
            if let Some(p) = self.inner.jobs.paused(&pid, &method.id, scope, now_ms())? {
                paused_any = true;
                if !plugin.spec.is_web() {
                    platform_called = true;
                    platform_refused = true;
                }
                let note = if plugin.spec.is_web() {
                    &mut first_web
                } else {
                    &mut paused_note
                };
                if missing.is_none() {
                    note.get_or_insert_with(|| {
                        let mut f = CallFailure::new(
                            ErrorCode::RateLimited,
                            format!("{pid} {} is paused here", method.id),
                        );
                        if p.held {
                            f.hint = Some("你让这个插件先别试了".into());
                        }
                        f
                    });
                }
                continue;
            }
            let first = if plugin.spec.is_web() {
                &mut first_web
            } else {
                &mut first_platform
            };
            if called && !self.pace_between_calls(platform, batch).await? {
                break;
            }
            let request = FetchRequest {
                method: method.id.clone(),
                input: FetchInput::Url(url.clone()),
                want,
                input_file: None,
            };
            called = true;
            if !plugin.spec.is_web() {
                platform_called = true;
            }
            let result = self
                .inner
                .host
                .fetch(plugin, request, &CallOptions::default())
                .await;
            self.inner
                .jobs
                .record_call(platform, now_ms(), &pace::today(), batch)?;
            match result {
                Ok(snap) => {
                    self.record_method_call(plugin, &method.id, url.host_str(), Ok(()))?;
                    let still: BTreeSet<OutputKind> = match &missing {
                        None => snap.output.missing.iter().copied().collect(),
                        Some(m) => m.difference(&snap.obtained).copied().collect(),
                    };
                    self.store_snapshot(&id, plugin, job, snap).await?;
                    if still.is_empty() {
                        missing = Some(still);
                        break;
                    }
                    missing = Some(still);
                }
                Err(PluginError::Call { failure, .. }) => {
                    tracing::info!(plugin = %pid, method = %method.id, item = %id, failure = %failure, "fetch failed");
                    self.record_method_call(plugin, &method.id, url.host_str(), Err(&failure))?;
                    if !plugin.spec.is_web() && failure.code != ErrorCode::UnsupportedInput {
                        platform_refused = true;
                    }
                    // 被拦也按最长的暂停记，到点自己解开；被拦的这一条不因暂停重试，只有限流的到点再试。
                    let (pause, retry_after_pause) = match failure.pause() {
                        Some(Pause::For(d)) => (Some(d), true),
                        Some(Pause::UntilResumed) => (Some(MAX_PAUSE), false),
                        None => (None, false),
                    };
                    if let Some(d) = pause {
                        let ms = i64::try_from(d.as_millis()).unwrap_or(i64::MAX);
                        let until = now_ms().saturating_add(ms);
                        self.inner.jobs.pause(
                            &pid,
                            &method.id,
                            scope,
                            until,
                            failure.code.as_str(),
                            failure.hint.as_deref(),
                        )?;
                        self.emit(Event::PluginsUpdated);
                        if retry_after_pause {
                            paused_any = true;
                        }
                    }
                    let waits = matches!(
                        failure.code,
                        ErrorCode::RateLimited | ErrorCode::Network | ErrorCode::Cancelled
                    );
                    if missing.is_none() {
                        retryable |= failure.code.retryable();
                        first.get_or_insert(*failure);
                    }
                    if waits {
                        break;
                    }
                }
                Err(e @ PluginError::Unsupported { .. }) => {
                    if missing.is_none() {
                        first.get_or_insert_with(|| {
                            CallFailure::new(ErrorCode::UnsupportedInput, e.to_string())
                        });
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        if missing.is_some() {
            self.inner.jobs.finish(id.as_str())?;
            self.emit(Event::ItemUpdated { id: id.to_string() });
            return Ok(());
        }
        let failure = first_platform
            .or(paused_note)
            .or(unusable)
            .or(first_web)
            .unwrap_or_else(|| {
                CallFailure::new(
                    ErrorCode::UnsupportedInput,
                    "no installed plugin can fetch this link",
                )
            });
        let hint = failure
            .hint
            .clone()
            .or_else(|| Some(failure.message.clone()));
        let code = failure.code.as_str();
        let can_retry = retryable || paused_any;
        if can_retry && job.attempts + 1 < self.inner.retry.max_attempts {
            let backoff =
                i64::try_from(self.inner.retry.delay(job.attempts).as_millis()).unwrap_or(i64::MAX);
            // 等到最早有一种方法能再试：都停着时等最早到点的那种；有没停的（例如限流以后还没轮到的下一种），退避以后就试它。
            let mut available_at = i64::MAX;
            for c in &candidates {
                let scope = pause_scope(&c.plugin, &url, platform);
                let at = match self.inner.jobs.paused(
                    c.plugin.id().as_str(),
                    &c.method.id,
                    scope,
                    now_ms(),
                )? {
                    Some(p) => p.until.unwrap_or(i64::MAX),
                    None => 0,
                };
                available_at = available_at.min(at);
            }
            let due = now_ms()
                .saturating_add(backoff)
                .max(if available_at == i64::MAX {
                    0
                } else {
                    available_at
                });
            self.inner
                .jobs
                .retry(id.as_str(), due, code, hint.as_deref())?;
        } else {
            self.inner.jobs.fail(id.as_str(), code, hint.as_deref())?;
        }
        self.emit(Event::ItemFetchFailed {
            id: id.to_string(),
            code: code.into(),
            hint,
            retry: can_retry,
        });
        Ok(())
    }

    /// 这一条要不要回到读文件的那个插件 `fetch <ref>`（协议 §3.9）：清单里记了 `ref`、导入的方法接 `ref`。
    pub(crate) fn ref_fetch(&self, import: &str, job: &FetchJob) -> Result<Option<RefFetch>> {
        let jobs = &self.inner.jobs;
        let (Some(imp), Some(entry)) =
            (jobs.import(import)?, jobs.entry_for(import, &job.item_id)?)
        else {
            return Ok(None);
        };
        let Some(reference) = entry.reference else {
            return Ok(None);
        };
        let plugin = imp
            .plugin
            .as_deref()
            .and_then(|p| self.inner.host.plugin(p))
            .cloned();
        Ok(Some(RefFetch {
            plugin,
            plugin_id: imp.plugin.clone().unwrap_or_default(),
            method: imp.method.clone().unwrap_or_default(),
            reference,
            input: self.import_copy(&imp),
            file_name: imp.file_name.clone().unwrap_or_default(),
        }))
    }

    /// 回到读文件的插件取一条（`fetch <ref>`，`USUYO_INPUT` 指着那份副本）。读的是本机的文件，不碰平台，
    /// 不守节奏；不换别的方法，也不补缺。
    async fn run_ref_job(&self, job: &FetchJob, by: RefFetch) -> Result<()> {
        let id = ItemId::parse(&job.item_id)?;
        let fail = |code: ErrorCode, message: String, hint: String| {
            let mut f = CallFailure::new(code, message);
            f.hint = Some(hint);
            f
        };
        let outcome = match (&by.plugin, &by.input) {
            (None, _) => Err(fail(
                ErrorCode::UnsupportedInput,
                format!("plugin {} is not installed", by.plugin_id),
                "读这个文件的插件不在了".into(),
            )),
            (_, Some(input)) if !input.is_file() => Err(fail(
                ErrorCode::DependencyMissing,
                format!("the copy of file:{} is gone", by.file_name),
                format!("交进来的文件 {} 不在了", by.file_name),
            )),
            (Some(plugin), input) => {
                let request = FetchRequest {
                    method: by.method.clone(),
                    input: FetchInput::Ref(by.reference.clone()),
                    want: None,
                    input_file: input.clone(),
                };
                let result = self
                    .inner
                    .host
                    .fetch(plugin, request, &CallOptions::default())
                    .await;
                match result {
                    Ok(snap) => {
                        self.record_method_call(plugin, &by.method, None, Ok(()))?;
                        self.store_snapshot(&id, plugin, job, snap).await?;
                        Ok(())
                    }
                    Err(PluginError::Call { failure, .. }) => {
                        self.record_method_call(plugin, &by.method, None, Err(&failure))?;
                        Err(*failure)
                    }
                    Err(e @ PluginError::Unsupported { .. }) => {
                        Err(CallFailure::new(ErrorCode::UnsupportedInput, e.to_string()))
                    }
                    Err(e) => return Err(e.into()),
                }
            }
        };
        let failure = match outcome {
            Ok(()) => {
                self.inner.jobs.finish(id.as_str())?;
                self.emit(Event::ItemUpdated { id: id.to_string() });
                return Ok(());
            }
            Err(f) => f,
        };
        let hint = failure
            .hint
            .clone()
            .or_else(|| Some(failure.message.clone()));
        let code = failure.code.as_str();
        let can_retry = failure.code.retryable();
        if can_retry && job.attempts + 1 < self.inner.retry.max_attempts {
            let backoff =
                i64::try_from(self.inner.retry.delay(job.attempts).as_millis()).unwrap_or(i64::MAX);
            self.inner.jobs.retry(
                id.as_str(),
                now_ms().saturating_add(backoff),
                code,
                hint.as_deref(),
            )?;
        } else {
            self.inner.jobs.fail(id.as_str(), code, hint.as_deref())?;
        }
        self.emit(Event::ItemFetchFailed {
            id: id.to_string(),
            code: code.into(),
            hint,
            retry: can_retry,
        });
        Ok(())
    }

    /// 同一条里接着调下一种方法之前照节奏等：要等的超过 [`PACE_BETWEEN_MAX`] 就不等了，回 `false`。
    async fn pace_between_calls(&self, platform: &str, batch: bool) -> Result<bool> {
        let pacing = self.inner.pacing.get(platform);
        let Some(at) = pace::ready_at(pacing, &self.inner.jobs, platform, batch, now_ms())? else {
            return Ok(true);
        };
        let wait = until(at, now_ms());
        if wait > PACE_BETWEEN_MAX {
            return Ok(false);
        }
        tokio::time::sleep(wait).await;
        Ok(true)
    }

    /// 取到的一份：在锁外算好哈希，再交给资料库当快照。
    async fn store_snapshot(
        &self,
        id: &ItemId,
        plugin: &FetchPlugin,
        job: &FetchJob,
        snap: FetchedSnapshot,
    ) -> Result<()> {
        let staged = snap.out_dir();
        let files = tokio::task::spawn_blocking(move || hash_staged(&staged))
            .await
            .map_err(|e| Error::Task(e.to_string()))??;
        let this = self.clone();
        let plugin = plugin.clone();
        // 只有 `ref` 的（从文件导入）没有原链接
        let input_url = (!job.url.is_empty()).then(|| job.url.clone());
        let item = id.clone();
        let imported = job.import_id.is_some();
        tokio::task::spawn_blocking(move || {
            this.attach(&item, &plugin, input_url, snap, files, imported)
        })
        .await
        .map_err(|e| Error::Task(e.to_string()))?
    }

    /// 导入里的一条轮到了：先收下（只有链接，记 `entry: import`），再取内容。
    /// 回要不要接着取：轮到时已进回收站（又没让拿回来）、库里已经存着内容、或者导入删了，就不取。
    fn take_entry(&self, import: &str, job: &FetchJob) -> Result<bool> {
        let jobs = &self.inner.jobs;
        let (Some(imp), Some(entry)) =
            (jobs.import(import)?, jobs.entry_for(import, &job.item_id)?)
        else {
            jobs.finish(&job.item_id)?;
            return Ok(false);
        };
        if entry.added {
            return Ok(true);
        }
        // 从文件导入的、没有身份的（聊天记录一类）照清单里记下的 `file-` id 建
        let identity: Option<usuyo_contract::Identity> = entry
            .identity
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?;
        let id = ItemId::parse(&job.item_id)?;
        let (acted_at, acted_at_original) = match entry.acted_at.as_deref() {
            Some(raw) => {
                let ts = usuyo_library::time::parse(raw)?;
                let original = (!raw.ends_with('Z') && !raw.ends_with('z')).then(|| raw.to_owned());
                (Some(usuyo_library::time::format(ts)), original)
            }
            None => (None, None),
        };
        let mut lib = self.lib();
        if !imp.also_trashed
            && matches!(
                lib.whereabouts(std::slice::from_ref(&id))?.first(),
                Some(Whereabouts::Trashed(_))
            )
        {
            drop(lib);
            jobs.finish(&job.item_id)?;
            jobs.import_skipped(import)?;
            return Ok(false);
        }
        let outcome = lib.add_file_item(
            &Via::Import.batch()?,
            id.clone(),
            Capture {
                identity,
                title_hint: entry.title.clone(),
                origin: NewOrigin {
                    entry: Via::Import.entry().into(),
                    // 你自己平台上的收藏、列表，你确认过才收：不是机器挑的（SPEC §1.2.1）。
                    taken_by: "user".into(),
                    import_batch: Some(import.to_owned()),
                    action: entry.action.clone(),
                    acted_at,
                    acted_at_original,
                    ..NewOrigin::default()
                },
                snapshot: None,
                skip_if_private: false,
            },
        )?;
        let got = outcome.id().clone();
        if matches!(outcome, AddOutcome::Skipped(_)) {
            drop(lib);
            jobs.finish(&job.item_id)?;
            jobs.import_skipped(import)?;
            return Ok(false);
        }
        self.reindex_item(&lib, &got)?;
        let has_content = !lib.item(&got)?.manifest.snapshots.is_empty();
        drop(lib);
        jobs.mark_added(import, &job.item_id)?;
        self.emit(match outcome {
            AddOutcome::Created(_) => Event::ItemAdded {
                id: got.to_string(),
            },
            _ => Event::ItemUpdated {
                id: got.to_string(),
            },
        });
        if got != id || has_content {
            jobs.finish(&job.item_id)?;
            return Ok(false);
        }
        Ok(true)
    }

    /// 取到的输出目录交给资料库当一份快照，再重写 `content.md` 和索引，排上转换。
    /// `imported`：这次取是导入、交进来的一批里的一条，转换排在你手动收的后面。
    fn attach(
        &self,
        id: &ItemId,
        plugin: &FetchPlugin,
        input_url: Option<String>,
        snap: FetchedSnapshot,
        files: Vec<usuyo_contract::FileHash>,
        imported: bool,
    ) -> Result<()> {
        let out = &snap.output;
        let fetch = fetch_record(out)?;
        let missing = out.missing.iter().map(|o| o.as_str().to_owned()).collect();
        let access = serde_json::to_value(out.evidence.access)?
            .as_str()
            .unwrap_or_default()
            .to_owned();
        let upstream = out
            .evidence
            .upstream
            .as_ref()
            .map(serde_json::to_value)
            .transpose()?;
        let new = NewSnapshot {
            plugin: plugin.id().to_string(),
            plugin_version: Some(plugin.header.version.to_string()),
            input_url,
            resolved_url: None,
            access,
            site: out.evidence.site.clone(),
            upstream,
            missing,
            missing_reason: out.missing_reason.clone(),
            text_origin: None,
            staged: snap.out_dir(),
            files,
            fetch,
        };
        let mut lib = self.lib();
        lib.add_snapshot(id, new)?;
        let last = self.inner.jobs.content_hash(id.as_str())?;
        let hash = lib.write_content(id, last.as_deref())?;
        self.inner.jobs.set_content_hash(id.as_str(), &hash)?;
        self.reindex_item(&lib, id)?;
        let by_hand = !imported && Engine::taken_by_hand(&lib.item(id)?.manifest);
        self.schedule_after_snapshot(&lib, id, by_hand)?;
        // 输出目录已经移进 raw/；`snap` 到这里才丢掉，剩下的暂存一起删。
        drop(snap);
        Ok(())
    }
}

/// 回到读文件的插件取的一条：插件、方法、`ref`、副本在哪。
pub(crate) struct RefFetch {
    /// 插件不在了是 `None`。
    pub plugin: Option<FetchPlugin>,
    plugin_id: String,
    pub method: String,
    reference: String,
    input: Option<std::path::PathBuf>,
    file_name: String,
}

/// fetch 输出 → `usuyo.fetch.json`：去掉 `credentialsUpdate`；`publishedAt` 换成 UTC 毫秒，
/// 偏移不是 `Z` 时原样另存 `publishedAtOriginal`（SPEC §0.5）。其余字段原样留在 `extra`。
fn fetch_record(out: &FetchOutput) -> Result<FetchRecord> {
    let mut v = serde_json::to_value(out)?;
    if let Value::Object(o) = &mut v {
        o.remove("credentialsUpdate");
        if let Some(Value::String(raw)) = o.get("publishedAt").cloned() {
            let ts = usuyo_library::time::parse(&raw)?;
            o.insert(
                "publishedAt".into(),
                Value::String(usuyo_library::time::format(ts)),
            );
            if !raw.ends_with('Z') && !raw.ends_with('z') {
                o.insert("publishedAtOriginal".into(), Value::String(raw));
            }
        }
    }
    Ok(serde_json::from_value(v)?)
}

/// 暂停记在哪个范围：通用网页插件的平台是所有网站，按网站暂停，一个站限流不停掉别的站；别的按平台。
fn pause_scope<'a>(plugin: &FetchPlugin, url: &'a url::Url, platform: &'a str) -> &'a str {
    if plugin.spec.is_web() {
        url.host_str().unwrap_or_default()
    } else {
        platform
    }
}

/// 条目 id 前面那段就是平台（SPEC §0.4）。
pub(crate) fn platform_of(item_id: &str) -> &str {
    item_id.rsplit_once('-').map_or("", |(p, _)| p)
}

fn until(at: i64, now: i64) -> Duration {
    Duration::from_millis(u64::try_from(at.saturating_sub(now)).unwrap_or(0))
}
