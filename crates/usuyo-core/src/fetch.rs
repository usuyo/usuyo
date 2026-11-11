//! 收下和后台取内容。
//!
//! 收下：认身份 → 马上建条目（只有链接）→ 排一个取内容的任务。
//! 任务：按插件顺序 fetch（平台插件在前、通用网页在后）；能换下一个的错就换，
//! 暂停照插件报的记下；都没取到、又有能重试的，按退避再排；取到了交给资料库当快照。

use std::time::Duration;

use serde_json::Value;
use usuyo_contract::FetchRecord;
use usuyo_library::{AddOutcome, Capture, ItemId, NewOrigin, NewSnapshot, normalize_web_url};
use usuyo_plugin::protocol::{CallFailure, Canonicalized, ErrorCode, FetchOutput, Pause};
use usuyo_plugin::{
    CallOptions, FetchInput, FetchPlugin, FetchRequest, FetchedSnapshot, PluginError,
};

use crate::engine::{Engine, Via};
use crate::jobs::{FetchJob, PausedUntil};
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
        let (identity, fetch_url) = self.identify(&url).await?;
        let this = self.clone();
        tokio::task::spawn_blocking(move || this.add(identity, fetch_url, via))
            .await
            .map_err(|e| Error::Task(e.to_string()))?
    }

    /// 身份：插件 canonicalize 的第一个命中；都不认，或因联网、超时失败，就照 `web:` 自己算（SPEC §0.4）。
    async fn identify(&self, url: &url::Url) -> Result<(usuyo_contract::Identity, String)> {
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
                // 你自己在窗口和终端里收的，私密的也照收（SPEC §1.7 的表）。
                skip_if_private: false,
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

    /// 一直跑后台任务；有新任务时 [`Engine::take`] 会叫醒它。
    pub async fn run_worker(&self) {
        loop {
            if let Err(e) = self.run_due().await {
                tracing::error!(error = %e, "fetch jobs");
            }
            let wait = match self.inner.jobs.next_due() {
                Ok(Some(at)) => {
                    Duration::from_millis(u64::try_from(at - now_ms()).unwrap_or(0)).min(IDLE)
                }
                Ok(None) => IDLE,
                Err(e) => {
                    tracing::error!(error = %e, "jobs database");
                    IDLE
                }
            };
            tokio::select! {
                () = self.inner.wake.notified() => {}
                () = tokio::time::sleep(wait) => {}
            }
        }
    }

    /// 把到点的任务一个个做完，回做了几个。
    pub async fn run_due(&self) -> Result<usize> {
        let mut n = 0;
        while let Some(job) = self.inner.jobs.claim_due(now_ms())? {
            n += 1;
            if let Err(e) = self.run_job(&job).await {
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
        Ok(n)
    }

    async fn run_job(&self, job: &FetchJob) -> Result<()> {
        let id = ItemId::parse(&job.item_id)?;
        let platform = id.platform().to_owned();
        let url = url::Url::parse(&job.url)
            .map_err(|e| Error::InvalidArgs(format!("bad job url `{}`: {e}", job.url)))?;
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
        for plugin in candidates {
            let pid = plugin.id().to_string();
            match self.inner.jobs.paused(&pid, &platform, now_ms())? {
                Some(PausedUntil::At(t)) => {
                    paused_until = Some(paused_until.map_or(t, |p| p.min(t)));
                    continue;
                }
                Some(PausedUntil::Resumed) => continue,
                None => {}
            }
            let request = FetchRequest {
                input: FetchInput::Url(url.clone()),
                want: None,
                input_file: None,
            };
            match self
                .inner
                .host
                .fetch(plugin, request, &CallOptions::default())
                .await
            {
                Ok(snap) => {
                    let this = self.clone();
                    let plugin = plugin.clone();
                    let input_url = job.url.clone();
                    let item = id.clone();
                    tokio::task::spawn_blocking(move || {
                        this.attach(&item, &plugin, input_url, snap)
                    })
                    .await
                    .map_err(|e| Error::Task(e.to_string()))??;
                    self.inner.jobs.finish(id.as_str())?;
                    self.emit(Event::ItemUpdated { id: id.to_string() });
                    return Ok(());
                }
                Err(PluginError::Call { failure, .. }) => {
                    tracing::info!(plugin = %pid, item = %id, failure = %failure, "fetch failed");
                    match failure.pause() {
                        Some(Pause::For(d)) => {
                            let ms = i64::try_from(d.as_millis()).unwrap_or(i64::MAX);
                            let until = now_ms().saturating_add(ms);
                            self.inner
                                .jobs
                                .pause(&pid, &platform, PausedUntil::At(until))?;
                            paused_until = Some(paused_until.map_or(until, |p| p.min(until)));
                        }
                        Some(Pause::UntilResumed) => {
                            self.inner
                                .jobs
                                .pause(&pid, &platform, PausedUntil::Resumed)?;
                        }
                        None => {}
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
        Ok(())
    }

    /// 取到的输出目录交给资料库当一份快照，再重写 `content.md` 和索引。
    fn attach(
        &self,
        id: &ItemId,
        plugin: &FetchPlugin,
        input_url: String,
        snap: FetchedSnapshot,
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
