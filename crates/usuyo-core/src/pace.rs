//! 平台的节奏（协议 §3.1）：同一平台所有获取插件共用一组上限，取装着的插件里最严的。
//!
//! - `interval`、`intervalMax`：上一次调用结束到下一次开始隔多久，每次在这两者之间随机取一个，手动收下的也守。
//!   随机数在一次调用结束时抽一回、记进 `jobs.db`，等的时候不再抽；没有插件写 `intervalMax` 时上限是 `interval` 的 1.5 倍。
//! - `daily`：批量（导入）的调用一天最多几次，按本地日期数；手动收下的不计。
//! - `batch: false`：这个平台不做导入。
//! - `blocked`：插件回 `blocked` 以后这种方法暂停多久，或者整个平台停下等你说接着。
//! - 同一平台同一时间只跑一个调用：每个平台一把锁。
//!
//! 计入节奏的是 `fetch`、`list` 的一页和 `sources`；`canonicalize` 不计。记账在 `jobs.db`，重启后照旧。

use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, Hasher};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use usuyo_plugin::FetchPlugin;
use usuyo_plugin::protocol::{MAX_PAUSE, OnBlocked};

use crate::Result;
use crate::jobs::Jobs;

/// 没有插件写 `intervalMax` 时，间隔的上限是 `interval` 的几倍（分子、分母）。
const DEFAULT_SPREAD: (u32, u32) = (3, 2);

/// 插件回 `blocked` 以后。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AfterBlocked {
    /// 这种方法在这个平台上暂停这么久，到点自己接着。
    Pause(Duration),
    /// 整个平台停下，等你点「继续获取」。
    Stop,
}

/// 一个平台合起来的节奏。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Pacing {
    pub batch: bool,
    pub interval: Duration,
    /// 间隔的上限，不小于 `interval`。
    pub interval_max: Duration,
    pub daily: Option<u32>,
    pub blocked: AfterBlocked,
}

impl Default for Pacing {
    fn default() -> Self {
        Pacing {
            batch: true,
            interval: Duration::ZERO,
            interval_max: Duration::ZERO,
            daily: None,
            blocked: AfterBlocked::Pause(MAX_PAUSE),
        }
    }
}

impl Pacing {
    /// 抽到 `draw`（0 到 1）时这一回的间隔：`interval` 到 `interval_max` 之间。
    pub fn gap(&self, draw: f64) -> Duration {
        let spread = self.interval_max.saturating_sub(self.interval);
        self.interval + spread.mul_f64(draw.clamp(0.0, 1.0))
    }
}

/// 装着的插件声明的节奏，按平台取最严的：间隔和上限取长的，被拦以后停下比暂停严、暂停取长的。没声明的平台不等。
pub(crate) fn combine(plugins: &[FetchPlugin]) -> BTreeMap<String, Pacing> {
    let mut out: BTreeMap<String, Pacing> = BTreeMap::new();
    let mut declared_max: BTreeMap<String, Duration> = BTreeMap::new();
    let mut declared_blocked: BTreeMap<String, AfterBlocked> = BTreeMap::new();
    for p in plugins {
        for (platform, decl) in &p.spec.pacing {
            let key = platform.as_str().to_owned();
            if let Some(m) = decl.interval_max {
                let e = declared_max.entry(key.clone()).or_default();
                *e = (*e).max(m.get());
            }
            if let Some(b) = decl.blocked {
                let b = match b {
                    OnBlocked::Stop => AfterBlocked::Stop,
                    OnBlocked::Resume(d) => AfterBlocked::Pause(d.get()),
                };
                let e = declared_blocked.entry(key.clone()).or_insert(b);
                *e = stricter(*e, b);
            }
            let e = out.entry(key).or_default();
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
    for (platform, e) in &mut out {
        let (num, den) = DEFAULT_SPREAD;
        let spread = e.interval * num / den;
        e.interval_max = declared_max
            .get(platform)
            .copied()
            .filter(|m| *m >= e.interval)
            .unwrap_or(spread);
        if let Some(b) = declared_blocked.get(platform) {
            e.blocked = *b;
        }
    }
    out
}

fn stricter(a: AfterBlocked, b: AfterBlocked) -> AfterBlocked {
    match (a, b) {
        (AfterBlocked::Stop, _) | (_, AfterBlocked::Stop) => AfterBlocked::Stop,
        (AfterBlocked::Pause(x), AfterBlocked::Pause(y)) => AfterBlocked::Pause(x.max(y)),
    }
}

/// 抽一个 0 到 1 之间的数，给一回的间隔用。用标准库每次新给的哈希种子（取自系统的随机数），不为它另加依赖；
/// 测试在 [`crate::EngineConfig`] 里换成固定的。
pub(crate) fn random_draw() -> f64 {
    let bits = std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish();
    (bits >> 11) as f64 / (1u64 << 53) as f64
}

/// 本地日期，`daily` 按它数。
/// 给人看的本地时间，两种语言各照各的写法：「11月12日 15:30」「Nov 12, 3:30 PM」。
pub(crate) fn local_time(ms: i64) -> crate::words::Said {
    crate::words::Said::by(|lang| {
        let format = match lang {
            crate::words::Lang::ZhHans => "%-m月%-d日 %H:%M",
            crate::words::Lang::En => "%b %-d, %-I:%M %p",
        };
        jiff::Timestamp::from_millisecond(ms)
            .map(|t| {
                t.to_zoned(jiff::tz::TimeZone::system())
                    .strftime(format)
                    .to_string()
            })
            .unwrap_or_default()
    })
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
    let gap = i64::try_from(p.gap(row.draw).as_millis()).unwrap_or(i64::MAX);
    let mut at = row.last_end_at.saturating_add(gap);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gap_stays_between_interval_and_its_upper_bound() {
        let p = Pacing {
            interval: Duration::from_secs(15),
            interval_max: Duration::from_secs(25),
            ..Pacing::default()
        };
        assert_eq!(p.gap(0.0), Duration::from_secs(15));
        assert_eq!(p.gap(0.5), Duration::from_secs(20));
        assert_eq!(p.gap(1.0), Duration::from_secs(25));
        // 抽到范围外的数也落在范围里
        assert_eq!(p.gap(-1.0), Duration::from_secs(15));
        assert_eq!(p.gap(7.0), Duration::from_secs(25));
        for _ in 0..100 {
            let d = random_draw();
            assert!((0.0..1.0).contains(&d), "{d}");
        }
    }
}
