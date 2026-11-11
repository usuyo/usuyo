//! 收下和后台取内容。
//!
//! 收下：认身份 → 马上建条目（只有链接）→ 排一个取内容的任务。
//! 任务：按插件顺序 fetch（平台插件在前、通用网页在后）；能换下一个的错就换，
//! 暂停照插件报的记下；都没取到、又有能重试的，按退避再排；取到了交给资料库当快照。

use std::time::Duration;

use serde_json::Value;
use usuyo_contract::FetchRecord;
use usuyo_library::{
    AddOutcome, Capture, ItemId, NewOrigin, NewSnapshot, Whereabouts, hash_staged,
    normalize_web_url,
};
use usuyo_plugin::protocol::{
    CallFailure, Canonicalized, ErrorCode, FetchOutput, MAX_PAUSE, Pause,
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

pub(crate) fn now_ms() -> i64 {
    jiff::Timestamp::now().as_millisecond()
}

/// 分享文字里的第一条 http(s) 链接：到空白或全角标点为止。
pub(crate) fn first_url(input: &str) -> Option<url::Url> {
    let lower = input.to_ascii_lowercase();
    let start = [lower.find("https://"), lower.find("http://")]
        .into_iter()
        .flatten()
        .min()?;
    let rest = &input[start..];
    let end = rest
        .find(|c: char| c.is_whitespace() || (!c.is_ascii() && !c.is_alphanumeric()))
        .unwrap_or(rest.len());
    let u = url::Url::parse(&rest[..end]).ok()?;
    u.host_str().is_some().then_some(u)
}

impl Engine {
    /// 收下一条链接（或带链接的分享文字）。马上回；内容在后台取。
    pub async fn take(&self, input: &str, kind: &str, via: Via) -> Result<TakeResponse> {
        if kind != "link" {
            return Err(Error::UnsupportedInput(format!(
                "taking a `{kind}` is not supported yet"
            )));
        }
        let url = first_url(input)
            .ok_or_else(|| Error::UnsupportedInput("no http(s) link in the input".into()))?;
        // 带用户名、密码的链接不收：它们会原样进标题、身份和 `content.md`。
        if !url.username().is_empty() || url.password().is_some() {
            return Err(Error::UnsupportedInput(
                "links with a user name or password are not taken".into(),
            ));
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

    /// 一直跑后台的事：取内容、列导入、收件文件夹。有新任务时 [`Engine::take`] 会叫醒取内容那一路。
    pub async fn run_worker(&self) {
        tokio::join!(self.run_fetches(), self.run_imports(), self.run_inbox());
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
            let batch = job.import_id.is_some();
            let called = match self.run_job(&job).await {
                Ok(called) => called,
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
                    true
                }
            };
            if called {
                self.inner
                    .jobs
                    .record_call(&platform, now_ms(), &pace::today(), batch)?;
            }
            if let Some(import) = &job.import_id {
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

    /// 回这次有没有调过插件（调过才记进节奏）。
    async fn run_job(&self, job: &FetchJob) -> Result<bool> {
        if let Some(import) = job.import_id.clone() {
            let this = self.clone();
            let j = job.clone();
            let go = tokio::task::spawn_blocking(move || this.take_entry(&import, &j))
                .await
                .map_err(|e| Error::Task(e.to_string()))??;
            if !go {
                return Ok(false);
            }
        }
        let id = ItemId::parse(&job.item_id)?;
        let platform = id.platform().to_owned();
        let url = url::Url::parse(&job.url)
            // 链接可能带分享参数，原因里不写它：这句会进日志和 `jobs.db`
            .map_err(|e| Error::InvalidArgs(format!("bad job url: {e}")))?;
        let candidates: Vec<&FetchPlugin> = self
            .inner
            .host
            .candidates(&url)
            .into_iter()
            .filter(|p| p.spec.capabilities.fetch.is_some())
            .collect();

        let mut last: Option<CallFailure> = None;
        let mut retryable = false;
        let mut paused_until: Option<i64> = None;
        let mut called = false;
        for plugin in candidates {
            let pid = plugin.id().to_string();
            // 通用网页插件的平台是所有网站：按网站暂停，一个站限流不停掉别的站。
            let scope = if plugin.spec.is_web() {
                url.host_str().unwrap_or_default()
            } else {
                platform.as_str()
            };
            if let Some(p) = self.inner.jobs.paused(&pid, scope, now_ms())? {
                if let Some(t) = p.until {
                    paused_until = Some(paused_until.map_or(t, |x| x.min(t)));
                }
                last.get_or_insert_with(|| {
                    let mut f =
                        CallFailure::new(ErrorCode::RateLimited, format!("{pid} is paused here"));
                    if p.held {
                        f.hint = Some("你让这个插件先别试了".into());
                    }
                    f
                });
                continue;
            }
            let request = FetchRequest {
                input: FetchInput::Url(url.clone()),
                want: None,
                input_file: None,
            };
            called = true;
            match self
                .inner
                .host
                .fetch(plugin, request, &CallOptions::default())
                .await
            {
                Ok(snap) => {
                    // 先在锁外算好哈希，大文件也不让别的请求等着资料库
                    let staged = snap.out_dir();
                    let files = tokio::task::spawn_blocking(move || hash_staged(&staged))
                        .await
                        .map_err(|e| Error::Task(e.to_string()))??;
                    let this = self.clone();
                    let plugin = plugin.clone();
                    let input_url = job.url.clone();
                    let item = id.clone();
                    tokio::task::spawn_blocking(move || {
                        this.attach(&item, &plugin, input_url, snap, files)
                    })
                    .await
                    .map_err(|e| Error::Task(e.to_string()))??;
                    self.inner.jobs.finish(id.as_str())?;
                    self.emit(Event::ItemUpdated { id: id.to_string() });
                    return Ok(true);
                }
                Err(PluginError::Call { failure, .. }) => {
                    tracing::info!(plugin = %pid, item = %id, failure = %failure, "fetch failed");
                    // 「等用户点继续」还没有入口：被拦也按最长的暂停记，到点自己解开。
                    // 被拦的这一条不因暂停重试（`blocked` 不可重试），只有限流的到点再试。
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
                            scope,
                            until,
                            failure.code.as_str(),
                            failure.hint.as_deref(),
                        )?;
                        if retry_after_pause {
                            paused_until = Some(paused_until.map_or(until, |p| p.min(until)));
                        }
                    }
                    retryable |= failure.code.retryable();
                    let stop = !failure.code.tries_next();
                    last = Some(*failure);
                    if stop {
                        break;
                    }
                }
                Err(e @ PluginError::Unsupported { .. }) => {
                    last = Some(CallFailure::new(ErrorCode::UnsupportedInput, e.to_string()));
                }
                Err(e) => return Err(e.into()),
            }
        }

        let failure = last.unwrap_or_else(|| {
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
        let can_retry = retryable || paused_until.is_some();
        if can_retry && job.attempts + 1 < self.inner.retry.max_attempts {
            let backoff =
                i64::try_from(self.inner.retry.delay(job.attempts).as_millis()).unwrap_or(i64::MAX);
            let due = now_ms()
                .saturating_add(backoff)
                .max(paused_until.unwrap_or(0));
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
        Ok(called)
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
        let identity: usuyo_contract::Identity = serde_json::from_str(
            entry
                .identity
                .as_deref()
                .ok_or_else(|| Error::InvalidArgs("import entry has no identity".into()))?,
        )?;
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
        let outcome = lib.add_item(
            &Via::Import.batch()?,
            Capture {
                identity: Some(identity),
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

    /// 取到的输出目录交给资料库当一份快照，再重写 `content.md` 和索引。
    fn attach(
        &self,
        id: &ItemId,
        plugin: &FetchPlugin,
        input_url: String,
        snap: FetchedSnapshot,
        files: Vec<usuyo_contract::FileHash>,
    ) -> Result<()> {
        let out = &snap.output;
        let fetch = fetch_record(out)?;
        let want = plugin
            .spec
            .capabilities
            .fetch
            .as_ref()
            .map(|c| c.outputs.iter().map(|o| o.as_str().to_owned()).collect())
            .unwrap_or_default();
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
            input_url: Some(input_url),
            resolved_url: None,
            access,
            site: out.evidence.site.clone(),
            upstream,
            want,
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
        // 输出目录已经移进 raw/；`snap` 到这里才丢掉，剩下的暂存一起删。
        drop(snap);
        Ok(())
    }
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

/// 条目 id 前面那段就是平台（SPEC §0.4）。
pub(crate) fn platform_of(item_id: &str) -> &str {
    item_id.rsplit_once('-').map_or("", |(p, _)| p)
}

fn until(at: i64, now: i64) -> Duration {
    Duration::from_millis(u64::try_from(at.saturating_sub(now)).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_link_in_share_text() {
        let u = first_url("【三段式注水】 https://example.com/a?b=1，复制打开").unwrap();
        assert_eq!(u.as_str(), "https://example.com/a?b=1");
        assert_eq!(
            first_url("看这个HTTP://Example.com/x y").unwrap().as_str(),
            "http://example.com/x"
        );
        assert!(first_url("没有链接").is_none());
        assert!(first_url("https://").is_none());
    }
}
