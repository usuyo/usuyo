//! 自动取访客身份（协议 §2.5.1）：取一条要访客身份而它没有、过期了或被站点拒收时，内核记下「要一份」，
//! 这条先等着；桌面壳问 `GET /visitors/wanted`，开一个看不见、不登录的窗口取一份交回来，等着的几条马上再试，
//! 每条只多等这一回。没有壳（只开着内核）或取不到时，到时间照常再试一次，这时才在条目上说要访客身份。
//!
//! 一个站点同一时间只要一份；取不到以后歇一阵（[`COOL_DOWN_MS`]）不再要，免得一批几百条反复开窗口。

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::Serialize;
use usuyo_plugin::protocol::ErrorCode;

use crate::Result;
use crate::engine::Engine;
use crate::fetch::now_ms;
use crate::view::Event;
use crate::words;

/// 要了一份以后多久没交来就算这回取不到。
pub(crate) const COOL_DOWN_MS: i64 = 10 * 60 * 1000;

/// 一个在要的访客身份（`GET /visitors/wanted`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WantedVisitor {
    pub site: String,
    /// 站点拒收了存着的那一份：先清掉访客窗口的网页数据再取。
    pub replace: bool,
}

#[derive(Debug, Default)]
struct State {
    /// 站点 → 什么时候要的、要不要换一份。
    asked: BTreeMap<String, (i64, bool)>,
    /// 站点 → 上一回取不到的时间。
    failed: BTreeMap<String, i64>,
    /// 站点 → 等着它的条目。
    parked: BTreeMap<String, BTreeSet<String>>,
    /// 已经为访客身份等过一回的条目。
    waited: HashSet<String>,
}

#[derive(Debug, Default)]
pub(crate) struct Wanted(Mutex<State>);

impl Wanted {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// 这条要不要为 `site` 的访客身份等一回：没等过、这个站点不在歇着。要等的话记下，回 `true`。
    pub(crate) fn park(
        &self,
        site: &str,
        item: &str,
        replace: bool,
        now: i64,
        wait_ms: i64,
    ) -> bool {
        let mut s = self.lock();
        if s.waited.contains(item) {
            return false;
        }
        if s.failed.get(site).is_some_and(|at| now - at < COOL_DOWN_MS) {
            return false;
        }
        // 上一回要的过了时间还没交来：这一回算取不到，歇一阵
        if let Some((at, _)) = s.asked.get(site)
            && now - at >= wait_ms
        {
            s.asked.remove(site);
            s.failed.insert(site.to_owned(), now);
            return false;
        }
        s.waited.insert(item.to_owned());
        let asked = s.asked.entry(site.to_owned()).or_insert((now, replace));
        asked.1 |= replace;
        s.parked
            .entry(site.to_owned())
            .or_default()
            .insert(item.to_owned());
        true
    }

    /// 等过一回又回来、访客身份还是没有：这个站点歇一阵。
    pub(crate) fn gave_up(&self, site: &str, item: &str, now: i64) {
        let mut s = self.lock();
        if s.waited.contains(item) && s.asked.remove(site).is_some() {
            s.failed.insert(site.to_owned(), now);
        }
        if let Some(p) = s.parked.get_mut(site) {
            p.remove(item);
        }
    }

    /// 在要的；过了 `wait_ms` 的不再给壳。
    pub(crate) fn asked(&self, now: i64, wait_ms: i64) -> Vec<WantedVisitor> {
        self.lock()
            .asked
            .iter()
            .filter(|(_, (at, _))| now - at < wait_ms)
            .map(|(site, (_, replace))| WantedVisitor {
                site: site.clone(),
                replace: *replace,
            })
            .collect()
    }

    /// 交来了一份：不再要，等着它的条目回给调用方马上再排。
    pub(crate) fn arrived(&self, site: &str) -> Vec<String> {
        let mut s = self.lock();
        s.asked.remove(site);
        s.failed.remove(site);
        s.parked
            .remove(site)
            .map(|p| p.into_iter().collect())
            .unwrap_or_default()
    }

    /// 取到了：这条以后再要访客身份时还能再等一回。
    pub(crate) fn done(&self, item: &str) {
        self.lock().waited.remove(item);
    }
}

impl Engine {
    fn visitor_wait_ms(&self) -> i64 {
        i64::try_from(self.inner.visitor_wait.as_millis()).unwrap_or(i64::MAX)
    }

    /// 这条为 `site` 的访客身份等一回：要一份，任务排到 [`EngineConfig::visitor_wait`](crate::EngineConfig) 以后
    /// （交来了就马上）。这条等过了、或这个站点在歇着时不等，回 `false`。
    pub(crate) fn wait_for_visitor(&self, item: &str, site: &str, replace: bool) -> Result<bool> {
        let now = now_ms();
        let wait = self.visitor_wait_ms();
        if !self
            .inner
            .visitors_wanted
            .park(site, item, replace, now, wait)
        {
            return Ok(false);
        }
        let name = self
            .loaded()
            .host
            .visitors()?
            .into_iter()
            .find(|v| v.visitor.site == site)
            .map_or_else(|| site.to_owned(), |v| v.visitor.name);
        let hint = words::VISITOR_FETCHING.with(&[("site", &name)]).to_stored();
        self.inner.jobs.retry(
            item,
            now.saturating_add(wait),
            ErrorCode::VisitorRequired.as_str(),
            Some(&hint),
        )?;
        tracing::info!(item, site, replace, "waiting for a visitor identity");
        self.emit(Event::ItemUpdated {
            id: item.to_owned(),
        });
        Ok(true)
    }

    /// `GET /visitors/wanted`：在要的访客身份，桌面壳照它开看不见的访客窗口。
    pub fn visitors_wanted(&self) -> Vec<WantedVisitor> {
        self.inner
            .visitors_wanted
            .asked(now_ms(), self.visitor_wait_ms())
    }

    /// 存下了一份访客身份：等着它的条目马上再排。
    pub(crate) fn visitor_arrived(&self, site: &str) -> Result<()> {
        let now = now_ms();
        for item in self.inner.visitors_wanted.arrived(site) {
            self.inner.jobs.due_now(&item, now)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WAIT: i64 = 90_000;

    #[test]
    fn one_ask_per_site_and_each_item_waits_once() {
        let w = Wanted::default();
        assert!(w.park("a.test", "i1", false, 0, WAIT));
        assert!(w.park("a.test", "i2", true, 10, WAIT));
        assert_eq!(
            w.asked(20, WAIT),
            [WantedVisitor {
                site: "a.test".into(),
                replace: true
            }]
        );
        // 同一条不等第二回
        assert!(!w.park("a.test", "i1", false, 30, WAIT));
        let mut back = w.arrived("a.test");
        back.sort();
        assert_eq!(back, ["i1", "i2"]);
        assert!(w.asked(40, WAIT).is_empty());
        w.done("i1");
        assert!(w.park("a.test", "i1", false, 50, WAIT));
    }

    #[test]
    fn a_site_that_never_answers_rests_for_a_while() {
        let w = Wanted::default();
        assert!(w.park("a.test", "i1", false, 0, WAIT));
        // 到点还没交来：给壳的单子里没有它了，新来的也不等
        assert!(w.asked(WAIT, WAIT).is_empty());
        assert!(!w.park("a.test", "i2", false, WAIT, WAIT));
        assert!(!w.park("a.test", "i3", false, WAIT + 1, WAIT));
        // 歇过了再要
        assert!(w.park("a.test", "i3", false, WAIT + COOL_DOWN_MS + 1, WAIT));
    }

    #[test]
    fn an_item_that_comes_back_without_one_gives_the_site_a_rest() {
        let w = Wanted::default();
        assert!(w.park("a.test", "i1", false, 0, WAIT));
        w.gave_up("a.test", "i1", 5);
        assert!(w.asked(6, WAIT).is_empty());
        assert!(!w.park("a.test", "i2", false, 7, WAIT));
    }
}
