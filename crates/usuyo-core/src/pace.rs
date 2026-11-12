//! 平台的节奏（协议 §3.1）：同一平台所有获取插件共用一组上限，取装着的插件里最严的。
//!
//! - `interval`：上一次调用结束到下一次开始的最短间隔，手动收下的也守。
//! - `daily`：批量（导入）的调用一天最多几次，按本地日期数；手动收下的不计。
//! - `batch: false`：这个平台不做导入。
//! - 同一平台同一时间只跑一个调用：每个平台一把锁。
//!
//! 计入节奏的是 `fetch`、`list` 的一页和 `sources`；`canonicalize` 不计。记账在 `jobs.db`，重启后照旧。

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use usuyo_plugin::FetchPlugin;

use crate::Result;
use crate::jobs::Jobs;

/// 一个平台合起来的节奏。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Pacing {
    pub batch: bool,
    pub interval: Duration,
    pub daily: Option<u32>,
}

impl Default for Pacing {
    fn default() -> Self {
        Pacing {
            batch: true,
            interval: Duration::ZERO,
            daily: None,
        }
    }
}

/// 装着的插件声明的节奏，按平台取最严的。没声明的平台不等。
pub(crate) fn combine(plugins: &[FetchPlugin]) -> BTreeMap<String, Pacing> {
    let mut out: BTreeMap<String, Pacing> = BTreeMap::new();
    for p in plugins {
        for (platform, decl) in &p.spec.pacing {
            let e = out.entry(platform.as_str().to_owned()).or_default();
            if decl.batch == Some(false) {
                e.batch = false;
            }
            if let Some(i) = decl.interval {
                e.interval = e.interval.max(i.get());
            }
            if let Some(d) = decl.daily {
                e.daily = Some(e.daily.map_or(d, |x| x.min(d)));
            }
        }
    }
    out
}

/// 本地日期，`daily` 按它数。
/// 给人看的本地时间，例如「11 月 12 日 15:30」。
pub(crate) fn local_time(ms: i64) -> String {
    jiff::Timestamp::from_millisecond(ms)
        .map(|t| {
            t.to_zoned(jiff::tz::TimeZone::system())
                .strftime("%-m 月 %-d 日 %H:%M")
                .to_string()
        })
        .unwrap_or_default()
}

pub(crate) fn today() -> String {
    jiff::Zoned::now().date().to_string()
}

/// 明天本地零点（Unix 毫秒）。
fn next_midnight_ms() -> i64 {
    let now = jiff::Zoned::now();
    now.date()
        .tomorrow()
        .ok()
        .and_then(|d| d.to_zoned(now.time_zone().clone()).ok())
        .map_or(i64::MAX, |z| z.timestamp().as_millisecond())
}

/// 现在能不能在这个平台上调一次：能就回 `None`，不能就回最早什么时候能。
pub(crate) fn ready_at(
    pacing: Option<&Pacing>,
    jobs: &Jobs,
    platform: &str,
    batch: bool,
    now: i64,
) -> Result<Option<i64>> {
    let Some(p) = pacing else {
        return Ok(None);
    };
    let row = jobs.pace(platform)?;
    let interval = i64::try_from(p.interval.as_millis()).unwrap_or(i64::MAX);
    let mut at = row.last_end_at.saturating_add(interval);
    if batch
        && let Some(daily) = p.daily
        && row.day == today()
        && row.batch_calls >= daily
    {
        at = at.max(next_midnight_ms());
    }
    Ok((at > now).then_some(at))
}

/// 每个平台一把锁：同一平台同一时间只跑一个调用。
#[derive(Default)]
pub(crate) struct PlatformLocks(Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>);

impl PlatformLocks {
    pub fn get(&self, platform: &str) -> Arc<tokio::sync::Mutex<()>> {
        let mut map = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        Arc::clone(map.entry(platform.to_owned()).or_default())
    }
}
