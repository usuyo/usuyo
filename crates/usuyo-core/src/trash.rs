//! 回收站到期（SPEC §1.7）：守护进程定时把到期的项移除，也供「立即清除」一项。

use std::path::PathBuf;
use std::time::Duration;

use jiff::{SignedDuration, Timestamp};
use usuyo_contract::Manifest;
use usuyo_library::{Batch, ItemId, expires_at, time};

use crate::engine::{Engine, Via};
use crate::view::{Event, TrashEntryView, TrashReason, TrashView, Unreadable};
use crate::{Error, Result};

/// 到期检查的间隔：回收站按天算，不用勤看。
const EXPIRY_EVERY: Duration = Duration::from_secs(6 * 3600);

/// 本机时间比上次记下的最大时间晚了超过 `trashDays`，多半是时钟拨错了（会把回收站整个移除）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockGuard {
    /// 照 SPEC：晚太多就先不移除。
    Check,
    /// 你点了「照时间移除」。
    Override,
}

/// 一轮到期检查做了什么。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Expiry {
    /// 整个离开资料库、已清干净的。
    pub purged: usize,
    /// 只移除了本机文件的（`keep-text`、`merge`、`conflict`、`notes`、`content`）。
    pub files_removed: usize,
    /// 这一轮没处理的项：回收站 id 和错误码。
    pub skipped: Vec<Unreadable>,
    /// 到期了但因为时钟差太多留着的项数。
    pub held: usize,
}

impl Engine {
    /// 移除所有到期的项。`now` 由调用方给（测试里是假的）。
    /// 一项出错就跳过、记进 `skipped`，别的照常。
    pub fn expire_trash(&self, now: &str, guard: ClockGuard) -> Result<Expiry> {
        let now = time::parse(now)?;
        let mut report = Expiry::default();
        let (due, mark, held) = self.due_trash(now, &mut report.skipped)?;
        if guard == ClockGuard::Check && held {
            report.held = due.len();
            return Ok(report);
        }
        let batch = Batch::new("host", "app")?;
        for tid in due {
            match self.purge_entry(&tid, &batch) {
                Ok(true) => report.purged += 1,
                Ok(false) => report.files_removed += 1,
                // 并进别的条目的那一项，随那一条一起移除了。
                Err(Error::Library(usuyo_library::Error::NotFound(_))) => {}
                Err(err) => report.skipped.push(skipped(&tid, &err)),
            }
        }
        self.remember_clock(&self.lib(), mark.map_or(now, |m| m.max(now)))?;
        Ok(report)
    }

    /// 到期的项、记下的最大时间，和时钟守卫拦不拦：本机时间比记下的最大时间晚了超过 `trashDays`。
    /// 读不了的项记进 `skipped`。
    fn due_trash(
        &self,
        now: Timestamp,
        skipped_out: &mut Vec<Unreadable>,
    ) -> Result<(Vec<String>, Option<Timestamp>, bool)> {
        let lib = self.lib();
        let days = lib.settings().trash_days;
        let mut due = Vec::new();
        for e in lib.trash_entries()? {
            match expiry_of(&e.trashed_at, days) {
                Ok(Some(at)) if at <= now => due.push(e.id),
                Ok(_) => {}
                Err(err) => skipped_out.push(skipped(&e.id, &err.into())),
            }
        }
        let mark = self.clock_mark(&lib)?;
        let held = !due.is_empty()
            && match (days, mark) {
                (Some(days), Some(mark)) => {
                    now.duration_since(mark) > SignedDuration::from_hours(i64::from(days) * 24)
                }
                _ => false,
            };
        Ok((due, mark, held))
    }

    /// 时钟守卫现在拦着几项、记下的最大时间是什么时候；没拦时 `None`。设置页照它给「照时间移除」。
    pub(crate) fn trash_hold(&self, now: &str) -> Result<Option<(usize, Timestamp)>> {
        let (due, mark, held) = self.due_trash(time::parse(now)?, &mut Vec::new())?;
        Ok(match (held, mark) {
            (true, Some(m)) => Some((due.len(), m)),
            _ => None,
        })
    }

    /// 回收站页：整条删掉的条目和只留文字移出来的媒体，新的在前。
    /// 主题、卡片、成稿，并掉的（`merge`）和改冲突替下来的（`conflict`）现在没有拿回来的接口，不列；
    /// 只留文字的条目自己也在回收站里时，媒体随那一条，等它拿回来再列。读不了的一项记进 `unreadable`。
    pub fn trash_list(&self) -> Result<TrashView> {
        let lib = self.lib();
        let days = lib.settings().trash_days;
        let mut entries = Vec::new();
        let mut unreadable = Vec::new();
        for e in lib.trash_entries()?.into_iter().rev() {
            let reason = match (e.kind.as_str(), e.reason.as_str()) {
                ("item", "delete") => TrashReason::Delete,
                ("item", "undo") => TrashReason::Undo,
                ("item", "keep-text") => TrashReason::KeepText,
                _ => continue,
            };
            let read = || -> Result<Option<(Manifest, Option<String>)>> {
                let manifest = if reason == TrashReason::KeepText {
                    match lib.item(&ItemId::parse(&e.target)?) {
                        Ok(item) => item.manifest,
                        Err(usuyo_library::Error::NotFound(_)) => return Ok(None),
                        Err(err) => return Err(err.into()),
                    }
                } else {
                    lib.trashed_manifest(&e)?
                };
                Ok(Some((manifest, expires_at(&e.trashed_at, days)?)))
            };
            match read() {
                Ok(Some((m, expires_at))) => {
                    let s = self.summary(&m);
                    entries.push(TrashEntryView {
                        id: e.id,
                        reason,
                        item_id: s.id,
                        title: s.title,
                        platform: s.platform,
                        author: s.author,
                        trashed_at: e.trashed_at,
                        expires_at,
                    });
                }
                Ok(None) => {}
                Err(err) => unreadable.push(skipped(&e.id, &err)),
            }
        }
        Ok(TrashView {
            trash_days: days,
            entries,
            unreadable,
        })
    }

    /// 你在回收站里「立即清除」一项：照到期办，不等 `trashDays`。
    pub fn purge_now(&self, trash_id: &str, via: Via) -> Result<()> {
        self.purge_entry(trash_id, &via.batch()?)?;
        Ok(())
    }

    /// 移除一项、清索引、发事件。回是不是整个离开了资料库。
    fn purge_entry(&self, tid: &str, batch: &Batch) -> Result<bool> {
        let (entry, report) = {
            let mut lib = self.lib();
            let entry = lib.trash_entry(tid)?;
            let report = lib.purge(batch, tid)?;
            (entry, report)
        };
        for id in &report.forgotten {
            self.inner.index.delete(id)?;
            self.stop_transforms(id)?;
        }
        if report.forgotten.is_empty() {
            // 只留文字移出来的媒体清掉了：条目还在，回收站页跟着这个事件重拉。
            if entry.kind == "item" && entry.reason == "keep-text" {
                self.emit(Event::ItemUpdated { id: entry.target });
            }
            return Ok(false);
        }
        self.inner.index.commit()?;
        if entry.kind == "item" {
            self.emit(Event::ItemRemoved { id: entry.target });
        }
        Ok(true)
    }

    /// 上次记下的最大时间；还没记过就取改动记录里最晚的一行。
    fn clock_mark(&self, lib: &usuyo_library::Library) -> Result<Option<Timestamp>> {
        let path = mark_path(lib);
        match std::fs::read_to_string(&path) {
            Ok(text) => match time::parse(text.trim()) {
                Ok(ts) => return Ok(Some(ts)),
                Err(err) => {
                    tracing::warn!(error = %err, "clock mark is not a time; using the journal")
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(Error::io(&path, e)),
        }
        Ok(lib
            .journal_entries()?
            .iter()
            .filter_map(|e| time::parse(&e.t).ok())
            .max())
    }

    fn remember_clock(&self, lib: &usuyo_library::Library, at: Timestamp) -> Result<()> {
        let path = mark_path(lib);
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, time::format(at)).map_err(|e| Error::io(&tmp, e))?;
        std::fs::rename(&tmp, &path).map_err(|e| Error::io(&path, e))
    }

    /// 一直跑：起来时检查一次，之后隔几个小时一次。
    pub(crate) async fn run_expiry(&self) {
        loop {
            let this = self.clone();
            let pass = tokio::task::spawn_blocking(move || {
                this.expire_trash(&time::now(), ClockGuard::Check)
            })
            .await
            .map_err(|e| Error::Task(e.to_string()))
            .and_then(|r| r);
            match pass {
                Ok(r) => {
                    if r != Expiry::default() {
                        // 只记数和回收站 id，不记标题和链接。
                        tracing::info!(
                            purged = r.purged,
                            files_removed = r.files_removed,
                            held = r.held,
                            skipped = r.skipped.len(),
                            "trash expiry"
                        );
                    }
                    for s in &r.skipped {
                        tracing::warn!(trash = %s.id, reason = %s.reason, "trash entry skipped");
                    }
                }
                Err(e) => tracing::error!(error = %e, "trash expiry"),
            }
            tokio::time::sleep(EXPIRY_EVERY).await;
        }
    }
}

fn mark_path(lib: &usuyo_library::Library) -> PathBuf {
    lib.machine_dir().join("clock-mark")
}

/// 到期时间；「一直」不到期。
fn expiry_of(trashed_at: &str, days: Option<u32>) -> usuyo_library::Result<Option<Timestamp>> {
    expires_at(trashed_at, days)?
        .map(|at| time::parse(&at))
        .transpose()
}

/// 跳过的一项：只留错误码，错误全文里可能带标题。
fn skipped(tid: &str, e: &Error) -> Unreadable {
    Unreadable {
        id: tid.to_owned(),
        reason: e.code().to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::TempDir;
    use tokio::sync::broadcast::Receiver;
    use usuyo_library::ItemId;

    use super::*;
    use crate::{EngineConfig, Links};

    struct Fixture {
        _tmp: TempDir,
        root: PathBuf,
        engine: Engine,
    }

    impl Fixture {
        /// `trash_days` 写进 settings.json；`None` 是「一直」。
        fn new(trash_days: Option<u32>) -> Fixture {
            let tmp = tempfile::tempdir().unwrap();
            let machine = tmp.path().join("machine");
            let root = Engine::init_library(&tmp.path().join("lib"), &machine).unwrap();
            std::fs::write(
                root.join(".usuyo/settings.json"),
                json!({"trashDays": trash_days}).to_string(),
            )
            .unwrap();
            let engine = Engine::open(EngineConfig::new(&root, &machine)).unwrap();
            Fixture {
                _tmp: tmp,
                root,
                engine,
            }
        }

        /// 收下一条并记一句，搜得到。
        async fn item(&self, path: &str, note: &str) -> String {
            let url = format!("https://example.test/{path}");
            let id = self.engine.take(&url, "link", Via::Cli).await.unwrap().id;
            self.engine
                .set_note(&id, note, Via::Cli, &Links::default())
                .unwrap();
            id
        }

        fn found(&self, q: &str) -> Vec<String> {
            let hits = self.engine.search(q, 10).unwrap();
            hits.into_iter().map(|h| h.item_id).collect()
        }

        fn trash(&self, id: &str) -> usuyo_contract::TrashEntry {
            self.engine.trash(id, Via::Cli, &Links::default()).unwrap();
            self.lib_entries()
                .into_iter()
                .find(|e| e.target == id)
                .unwrap()
        }

        fn lib_entries(&self) -> Vec<usuyo_contract::TrashEntry> {
            self.engine.lib().trash_entries().unwrap()
        }

        /// 手写一项回收站项（到期的时间自己定）。
        fn craft(&self, tid: &str, target: &str, reason: &str, trashed_at: &str, files: &[&str]) {
            let dir = self.root.join(".usuyo/trash").join(tid);
            std::fs::create_dir_all(&dir).unwrap();
            for f in files {
                let p = dir.join(f);
                std::fs::create_dir_all(p.parent().unwrap()).unwrap();
                std::fs::write(p, "x").unwrap();
            }
            let entry = json!({
                "id": tid, "kind": "item", "target": target, "path": "raw/s/images/001.jpg",
                "files": files, "reason": reason, "trashedAt": trashed_at,
            });
            std::fs::write(dir.join("entry.json"), entry.to_string()).unwrap();
        }

        fn journal(&self) -> String {
            let mut out = String::new();
            let mut dirs = vec![self.root.join(".usuyo/journal")];
            while let Some(d) = dirs.pop() {
                for e in std::fs::read_dir(d).unwrap() {
                    let p = e.unwrap().path();
                    if p.is_dir() {
                        dirs.push(p);
                    } else {
                        out.push_str(&std::fs::read_to_string(p).unwrap());
                    }
                }
            }
            out
        }

        fn trash_dir(&self, tid: &str) -> PathBuf {
            self.root.join(".usuyo/trash").join(tid)
        }
    }

    fn after(e: &usuyo_contract::TrashEntry, days: u32) -> String {
        time::add_days(&e.trashed_at, days).unwrap()
    }

    fn removed(rx: &mut Receiver<Event>) -> Vec<String> {
        let mut out = Vec::new();
        while let Ok(e) = rx.try_recv() {
            if let Event::ItemRemoved { id } = e {
                out.push(id);
            }
        }
        out
    }

    const OLD: &str = "2020-01-01T00:00:00.000Z";
    const T1: &str = "0190a000-0000-7000-8000-000000000001";
    const T2: &str = "0190a000-0000-7000-8000-000000000002";
    const T3: &str = "0190a000-0000-7000-8000-000000000003";

    #[tokio::test]
    async fn due_item_is_cleaned_out_and_leaves_the_index() {
        let f = Fixture::new(Some(30));
        let id = f.item("a", "手冲咖啡").await;
        let mut rx = f.engine.subscribe();
        // 删掉进索引的另一条路：没来得及清索引。
        let stale = {
            let lib = f.engine.lib();
            let item = lib.item(&ItemId::parse(&id).unwrap()).unwrap();
            let c = crate::content::read(&lib, &item).unwrap();
            crate::index_doc::index_doc(&item.manifest, &c)
        };
        let e = f.trash(&id);
        f.engine.inner.index.upsert(&stale).unwrap();
        f.engine.inner.index.commit().unwrap();
        assert_eq!(f.found("咖啡"), vec![id.clone()]);
        assert!(f.journal().contains(&id));

        // 第一轮没到期，只记下时间。
        let r = f
            .engine
            .expire_trash(&after(&e, 1), ClockGuard::Check)
            .unwrap();
        assert_eq!(r, Expiry::default());
        let r = f
            .engine
            .expire_trash(&after(&e, 31), ClockGuard::Check)
            .unwrap();
        assert_eq!((r.purged, r.files_removed, r.held), (1, 0, 0));
        assert!(f.lib_entries().is_empty());
        assert!(f.found("咖啡").is_empty());
        assert!(matches!(
            f.engine.item(&id, &Links::default()),
            Err(Error::NotFound(_))
        ));
        assert!(!f.journal().contains(&id), "改动记录里不留它的 id");
        assert_eq!(removed(&mut rx), [id]);
    }

    #[tokio::test]
    async fn not_yet_due_stays() {
        let f = Fixture::new(Some(30));
        let id = f.item("a", "咖啡").await;
        let e = f.trash(&id);
        f.engine
            .expire_trash(&after(&e, 1), ClockGuard::Check)
            .unwrap();
        let r = f
            .engine
            .expire_trash(&after(&e, 29), ClockGuard::Check)
            .unwrap();
        assert_eq!(r, Expiry::default());
        assert_eq!(f.lib_entries().len(), 1);
        assert!(f.trash_dir(&e.id).is_dir());
    }

    #[tokio::test]
    async fn forever_never_expires() {
        let f = Fixture::new(None);
        let id = f.item("a", "咖啡").await;
        let e = f.trash(&id);
        let r = f
            .engine
            .expire_trash(&after(&e, 36_500), ClockGuard::Check)
            .unwrap();
        assert_eq!(r, Expiry::default());
        assert_eq!(f.lib_entries().len(), 1);
    }

    #[tokio::test]
    async fn keep_text_and_merge_only_lose_their_files() {
        let f = Fixture::new(Some(30));
        let id = f.item("a", "咖啡").await;
        f.craft(T1, &id, "keep-text", OLD, &["raw/s/images/001.jpg"]);
        f.craft(T2, &id, "merge", OLD, &["manifest.json"]);
        let r = f
            .engine
            .expire_trash(&time::now(), ClockGuard::Check)
            .unwrap();
        assert_eq!((r.purged, r.files_removed), (0, 2));
        assert!(!f.trash_dir(T1).exists() && !f.trash_dir(T2).exists());
        // 条目还在，索引也没动。
        assert_eq!(f.found("咖啡"), vec![id.clone()]);
        assert!(f.engine.item(&id, &Links::default()).is_ok());
        assert!(f.journal().contains(&id), "本机文件的移除不脱敏改动记录");
    }

    #[tokio::test]
    async fn a_clock_far_ahead_purges_nothing_until_you_say_so() {
        let f = Fixture::new(Some(30));
        let id = f.item("a", "咖啡").await;
        let e = f.trash(&id);
        // 还没记过时间：拿改动记录里最晚的一行当底。
        let jump = after(&e, 3650);
        let r = f.engine.expire_trash(&jump, ClockGuard::Check).unwrap();
        assert_eq!((r.purged, r.held), (0, 1));
        assert_eq!(f.lib_entries().len(), 1);
        assert_eq!(f.found("咖啡").len(), 0, "还在回收站里，索引里本来就没有");
        // 记下了时间以后，拨远了照样留着。
        f.engine
            .expire_trash(&after(&e, 2), ClockGuard::Check)
            .unwrap();
        let r = f.engine.expire_trash(&jump, ClockGuard::Check).unwrap();
        assert_eq!((r.purged, r.held), (0, 1));
        // 设置页照同一个判法看到拦着一项，记下的时间是上次那一轮的。
        let (count, since) = f.engine.trash_hold(&jump).unwrap().unwrap();
        assert_eq!(count, 1);
        assert_eq!(time::format(since), after(&e, 2));
        assert_eq!(f.engine.trash_hold(&after(&e, 2)).unwrap(), None);
        // 你点了「照时间移除」。
        let r = f.engine.expire_trash(&jump, ClockGuard::Override).unwrap();
        assert_eq!((r.purged, r.held), (1, 0));
        assert!(f.lib_entries().is_empty());
        assert_eq!(f.engine.trash_hold(&jump).unwrap(), None);
    }

    #[tokio::test]
    async fn a_bad_entry_is_skipped_and_the_rest_go() {
        let f = Fixture::new(Some(30));
        let id = f.item("a", "咖啡").await;
        // 时间写坏了；整条没有 manifest（清不了）；一项好的。
        f.craft(T1, &id, "delete", "not a time", &[]);
        f.craft(
            T2,
            "web-0190a0000000700080000000000000aa",
            "delete",
            "2019-01-01T00:00:00.000Z",
            &[],
        );
        f.craft(T3, &id, "keep-text", OLD, &["raw/s/images/001.jpg"]);
        let r = f
            .engine
            .expire_trash(&time::now(), ClockGuard::Check)
            .unwrap();
        assert_eq!((r.purged, r.files_removed), (0, 1));
        let ids: Vec<&str> = r.skipped.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&T1) && ids.contains(&T2));
        assert!(r.skipped.iter().all(|s| !s.reason.contains(&id)));
        assert!(!f.trash_dir(T3).exists());
        assert!(f.trash_dir(T1).exists() && f.trash_dir(T2).exists());
    }

    #[tokio::test]
    async fn the_list_has_whole_items_and_moved_media_newest_first() {
        let f = Fixture::new(Some(30));
        let a = f.item("a", "咖啡").await;
        let b = f.item("b", "茶").await;
        f.craft(T1, &b, "keep-text", OLD, &["raw/s/images/001.jpg"]);
        f.craft(T2, &b, "merge", OLD, &["manifest.json"]);
        let e = f.trash(&a);
        let v = f.engine.trash_list().unwrap();
        assert_eq!(v.trash_days, Some(30));
        assert!(v.unreadable.is_empty());
        let rows: Vec<(&str, TrashReason, &str)> = v
            .entries
            .iter()
            .map(|r| (r.id.as_str(), r.reason, r.item_id.as_str()))
            .collect();
        assert_eq!(
            rows,
            [
                (e.id.as_str(), TrashReason::Delete, a.as_str()),
                (T1, TrashReason::KeepText, b.as_str()),
            ]
        );
        assert_eq!(v.entries[0].trashed_at, e.trashed_at);
        assert_eq!(v.entries[0].expires_at, Some(after(&e, 30)));
        assert_eq!(v.entries[0].title, "https://example.test/a");
        assert_eq!(v.entries[0].platform.id, "web");

        // 条目自己也进了回收站：媒体随那一条，不单列。
        f.trash(&b);
        let v = f.engine.trash_list().unwrap();
        let reasons: Vec<TrashReason> = v.entries.iter().map(|r| r.reason).collect();
        assert_eq!(reasons, [TrashReason::Delete, TrashReason::Delete]);
    }

    #[tokio::test]
    async fn the_list_has_no_clearing_time_when_kept_forever_and_skips_a_bad_entry() {
        let f = Fixture::new(None);
        let id = f.item("a", "咖啡").await;
        f.trash(&id);
        // 条目 id 写坏了的一项。
        f.craft(T1, "not-an-item", "keep-text", OLD, &[]);
        let v = f.engine.trash_list().unwrap();
        assert_eq!(v.trash_days, None);
        assert_eq!(v.entries.len(), 1);
        assert_eq!(v.entries[0].expires_at, None);
        let bad: Vec<&str> = v.unreadable.iter().map(|u| u.id.as_str()).collect();
        assert_eq!(bad, [T1]);
        assert!(
            v.unreadable
                .iter()
                .all(|u| !u.reason.contains("not-an-item"))
        );
    }

    #[tokio::test]
    async fn clearing_a_whole_item_also_clears_its_moved_media() {
        let f = Fixture::new(None);
        let id = f.item("a", "咖啡").await;
        f.craft(T1, &id, "keep-text", OLD, &["raw/s/images/001.jpg"]);
        f.trash(&id);
        let whole = f
            .lib_entries()
            .into_iter()
            .find(|e| e.reason == "delete")
            .unwrap();
        assert!(f.trash_dir(T1).is_dir());
        f.engine.purge_now(&whole.id, Via::App).unwrap();
        assert!(
            !f.trash_dir(T1).exists(),
            "只留文字移出来的媒体随条目一起清掉"
        );
        assert!(f.lib_entries().is_empty());
    }

    #[tokio::test]
    async fn clearing_moved_media_says_the_item_changed() {
        let f = Fixture::new(Some(30));
        let id = f.item("a", "咖啡").await;
        f.craft(T1, &id, "keep-text", OLD, &["raw/s/images/001.jpg"]);
        let mut rx = f.engine.subscribe();
        f.engine.purge_now(T1, Via::App).unwrap();
        let mut updated = Vec::new();
        while let Ok(e) = rx.try_recv() {
            if let Event::ItemUpdated { id } = e {
                updated.push(id);
            }
        }
        assert_eq!(updated, [id]);
    }

    #[tokio::test]
    async fn purge_now_does_not_wait() {
        let f = Fixture::new(Some(30));
        let id = f.item("a", "咖啡").await;
        let mut rx = f.engine.subscribe();
        let e = f.trash(&id);
        f.engine.purge_now(&e.id, Via::App).unwrap();
        assert!(f.lib_entries().is_empty());
        assert!(!f.journal().contains(&id));
        assert_eq!(removed(&mut rx), [id]);
        assert!(matches!(
            f.engine.purge_now(&e.id, Via::App),
            Err(Error::Library(usuyo_library::Error::NotFound(_)))
        ));
    }
}
