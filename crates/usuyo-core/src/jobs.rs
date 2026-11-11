//! 本机任务队列 `jobs.db`（SPEC §1.1）：取内容的任务、插件的暂停、平台的节奏、导入，
//! 和上次写的 `content.md` 的哈希。
//! 暂停按 (插件, 范围) 记：平台插件的范围是平台，通用网页插件的是网站的主机名（列名沿用 `platform`）。
//!
//! 一条一个任务（按 `item_id` 去重）；`running` 的在重新打开时放回 `queued`，重启后接着做。
//! 导入的任务带 `import_id`；导入被你停下时它的任务是 `held`，不取。

use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};

use rusqlite::{Connection, OptionalExtension, params};
use usuyo_plugin::protocol::MAX_PAUSE;

use crate::engine::RETRY_DELAY_MAX;
use crate::{Error, Result};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS fetch_jobs (
    item_id    TEXT PRIMARY KEY,
    url        TEXT NOT NULL,
    state      TEXT NOT NULL,
    attempts   INTEGER NOT NULL DEFAULT 0,
    due_at     INTEGER NOT NULL,
    error_code TEXT,
    error_hint TEXT
);
CREATE TABLE IF NOT EXISTS pauses (
    plugin   TEXT NOT NULL,
    platform TEXT NOT NULL,
    until_at INTEGER,
    PRIMARY KEY (plugin, platform)
);
CREATE TABLE IF NOT EXISTS content_hashes (
    item_id TEXT PRIMARY KEY,
    hash    TEXT NOT NULL
);
";

/// 版本 1（feature 002）：任务的优先级和所属导入、暂停的原因和「先别试」、平台节奏、导入。
const SCHEMA_V1: &str = "
ALTER TABLE fetch_jobs ADD COLUMN priority INTEGER NOT NULL DEFAULT 0;
ALTER TABLE fetch_jobs ADD COLUMN import_id TEXT;
ALTER TABLE pauses ADD COLUMN code TEXT;
ALTER TABLE pauses ADD COLUMN hint TEXT;
ALTER TABLE pauses ADD COLUMN held INTEGER NOT NULL DEFAULT 0;
CREATE TABLE pace (
    platform    TEXT PRIMARY KEY,
    last_end_at INTEGER NOT NULL DEFAULT 0,
    day         TEXT NOT NULL DEFAULT '',
    batch_calls INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE imports (
    id           TEXT PRIMARY KEY,
    kind         TEXT NOT NULL,
    plugin       TEXT,
    source       TEXT,
    client       TEXT,
    title        TEXT NOT NULL,
    action       TEXT,
    created_at   INTEGER NOT NULL,
    cursor       TEXT,
    listed       INTEGER NOT NULL DEFAULT 0,
    pages        INTEGER NOT NULL DEFAULT 0,
    attempts     INTEGER NOT NULL DEFAULT 0,
    wait_until   INTEGER,
    stop_code    TEXT,
    stop_hint    TEXT,
    held         INTEGER NOT NULL DEFAULT 0,
    confirmed_at INTEGER,
    also_trashed INTEGER NOT NULL DEFAULT 0,
    skipped      INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE import_entries (
    import_id TEXT NOT NULL,
    seq       INTEGER NOT NULL,
    url       TEXT NOT NULL,
    title     TEXT,
    action    TEXT,
    acted_at  TEXT,
    identity  TEXT,
    item_id   TEXT,
    standing  TEXT NOT NULL,
    added     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (import_id, seq)
);
CREATE INDEX import_entries_item ON import_entries (import_id, item_id);
PRAGMA user_version = 1;
";

/// 手动收下的排在导入前面（协议 §3.1 的队列顺序）。
pub(crate) const PRIORITY_MANUAL: i64 = 0;
pub(crate) const PRIORITY_IMPORT: i64 = 2;

/// 一个取内容的任务。时间都是 Unix 毫秒。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchJob {
    pub item_id: String,
    pub url: String,
    pub state: JobState,
    pub attempts: u32,
    pub due_at: i64,
    pub error_code: Option<String>,
    pub error_hint: Option<String>,
    pub priority: i64,
    pub import_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Queued,
    Running,
    /// 导入被你停下了：不取，接着导入时回到 `queued`。
    Held,
    /// 不再自动重试；原因在 `error_code`、`error_hint`。
    Failed,
}

impl JobState {
    fn as_str(self) -> &'static str {
        match self {
            JobState::Queued => "queued",
            JobState::Running => "running",
            JobState::Held => "held",
            JobState::Failed => "failed",
        }
    }

    fn parse(s: &str) -> JobState {
        match s {
            "running" => JobState::Running,
            "held" => JobState::Held,
            "failed" => JobState::Failed,
            _ => JobState::Queued,
        }
    }
}

/// 一个插件在一个范围上的暂停。`until` 为空是你点了「先别试」，到你点「现在再试」为止。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PauseRow {
    pub scope: String,
    pub until: Option<i64>,
    pub code: Option<String>,
    pub hint: Option<String>,
    pub held: bool,
}

/// 一个平台的节奏记账：上次调用什么时候结束，今天（本地日期）批量调用了几次。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaceRow {
    pub last_end_at: i64,
    pub day: String,
    pub batch_calls: u32,
}

/// 一次导入。`kind`：`source`（插件的 `list`）或 `delivered`（交进来的一批链接）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRow {
    pub id: String,
    pub kind: String,
    pub plugin: Option<String>,
    pub source: Option<String>,
    pub client: Option<String>,
    pub title: String,
    pub action: Option<String>,
    pub created_at: i64,
    pub cursor: Option<String>,
    pub listed: bool,
    pub pages: u32,
    pub attempts: u32,
    pub wait_until: Option<i64>,
    pub stop_code: Option<String>,
    pub stop_hint: Option<String>,
    pub held: bool,
    pub confirmed_at: Option<i64>,
    pub also_trashed: bool,
    /// 轮到它时已经进了回收站（又没让拿回来）、或者私密的，没收。
    pub skipped: u32,
}

/// 导入里的一条。`standing`：`unresolved`（还没认出身份）、`new`、`stored`、`trashed`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryRow {
    pub seq: i64,
    pub url: String,
    pub title: Option<String>,
    pub action: Option<String>,
    pub acted_at: Option<String>,
    pub identity: Option<String>,
    pub item_id: Option<String>,
    pub standing: String,
    pub added: bool,
}

pub(crate) struct Jobs {
    conn: Mutex<Connection>,
}

fn ms(d: std::time::Duration) -> i64 {
    i64::try_from(d.as_millis()).unwrap_or(i64::MAX)
}

const JOB_COLUMNS: &str =
    "item_id, url, state, attempts, due_at, error_code, error_hint, priority, import_id";
const IMPORT_COLUMNS: &str = "id, kind, plugin, source, client, title, action, created_at, cursor,
    listed, pages, attempts, wait_until, stop_code, stop_hint, held, confirmed_at, also_trashed, skipped";
const ENTRY_COLUMNS: &str = "seq, url, title, action, acted_at, identity, item_id, standing, added";

impl Jobs {
    /// 打开（没有就建、旧的升上来），把上次没做完的 `running` 放回队列。
    ///
    /// 旧版本存下的暂停没有上限（一个 `Retry-After` 能停到几千年后），也有不是你点的、没有到点时间的：
    /// 超过现在的上限的、和不是「先别试」又没有到点时间的删掉；到点时间远过重试上限的任务马上排。
    pub fn open(path: &Path, now: i64) -> Result<Jobs> {
        let conn = Connection::open(path)?;
        // 里面有链接（可能带分享参数），只给自己读
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| Error::io(path, e))?;
        }
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch(SCHEMA)?;
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version < 1 {
            conn.execute_batch(&format!("BEGIN; {SCHEMA_V1} COMMIT;"))?;
        }
        conn.execute(
            "UPDATE fetch_jobs SET state = 'queued' WHERE state = 'running'",
            [],
        )?;
        conn.execute(
            "DELETE FROM pauses WHERE (until_at IS NULL AND held = 0) OR until_at > ?1",
            params![now.saturating_add(ms(MAX_PAUSE))],
        )?;
        conn.execute(
            "UPDATE fetch_jobs SET due_at = ?1 WHERE state = 'queued' AND due_at > ?2",
            params![now, now.saturating_add(ms(RETRY_DELAY_MAX))],
        )?;
        Ok(Jobs {
            conn: Mutex::new(conn),
        })
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// 排一个手动收下（或再取一次）的任务。已经在排或在跑的不动；失败过的重新开始，不再算哪次导入的。
    pub fn enqueue(&self, item_id: &str, url: &str, now: i64) -> Result<()> {
        self.conn().execute(
            "INSERT INTO fetch_jobs (item_id, url, state, attempts, due_at, priority)
             VALUES (?1, ?2, 'queued', 0, ?3, ?4)
             ON CONFLICT (item_id) DO UPDATE SET
                url = excluded.url, state = 'queued', attempts = 0, due_at = excluded.due_at,
                error_code = NULL, error_hint = NULL, priority = excluded.priority, import_id = NULL
             WHERE fetch_jobs.state = 'failed'",
            params![item_id, url, now, PRIORITY_MANUAL],
        )?;
        Ok(())
    }

    /// 排一条导入里的。这一条已经有任务（你刚手动收过）就不动，回 `false`。
    pub fn enqueue_import(
        &self,
        item_id: &str,
        url: &str,
        import_id: &str,
        now: i64,
    ) -> Result<bool> {
        let n = self.conn().execute(
            "INSERT INTO fetch_jobs (item_id, url, state, attempts, due_at, priority, import_id)
             VALUES (?1, ?2, 'queued', 0, ?3, ?4, ?5)
             ON CONFLICT (item_id) DO NOTHING",
            params![item_id, url, now, PRIORITY_IMPORT, import_id],
        )?;
        Ok(n == 1)
    }

    /// 到点的任务，按队列顺序（手动在前，再按到点时间）。由调用方挑能跑的去 [`Jobs::claim`]。
    pub fn due(&self, now: i64) -> Result<Vec<FetchJob>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(&format!(
            "SELECT {JOB_COLUMNS} FROM fetch_jobs WHERE state = 'queued' AND due_at <= ?1
             ORDER BY priority, due_at, item_id LIMIT 1000"
        ))?;
        let rows = stmt.query_map(params![now], row_to_job)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// 标成 `running`；被别人拿走了（或者不在排了）回 `false`。
    pub fn claim(&self, item_id: &str) -> Result<bool> {
        let n = self.conn().execute(
            "UPDATE fetch_jobs SET state = 'running' WHERE item_id = ?1 AND state = 'queued'",
            params![item_id],
        )?;
        Ok(n == 1)
    }

    /// 排着、还没到点的任务里最早的到点时间。到点了还排着的是在等节奏或暂停，另算。
    pub fn next_due_after(&self, now: i64) -> Result<Option<i64>> {
        Ok(self.conn().query_row(
            "SELECT MIN(due_at) FROM fetch_jobs WHERE state = 'queued' AND due_at > ?1",
            params![now],
            |r| r.get(0),
        )?)
    }

    pub fn job(&self, item_id: &str) -> Result<Option<FetchJob>> {
        Ok(self
            .conn()
            .query_row(
                &format!("SELECT {JOB_COLUMNS} FROM fetch_jobs WHERE item_id = ?1"),
                params![item_id],
                row_to_job,
            )
            .optional()?)
    }

    /// 做完了：任务删掉。
    pub fn finish(&self, item_id: &str) -> Result<()> {
        self.conn().execute(
            "DELETE FROM fetch_jobs WHERE item_id = ?1",
            params![item_id],
        )?;
        Ok(())
    }

    /// 过会儿再试：次数加一。
    pub fn retry(&self, item_id: &str, due_at: i64, code: &str, hint: Option<&str>) -> Result<()> {
        self.conn().execute(
            "UPDATE fetch_jobs SET state = 'queued', attempts = attempts + 1, due_at = ?2,
                error_code = ?3, error_hint = ?4 WHERE item_id = ?1",
            params![item_id, due_at, code, hint],
        )?;
        Ok(())
    }

    /// 不再试：链接清掉（可能带分享参数，之后没人读它；再取一次时 `enqueue` 重新写）。
    pub fn fail(&self, item_id: &str, code: &str, hint: Option<&str>) -> Result<()> {
        self.conn().execute(
            "UPDATE fetch_jobs SET state = ?2, url = '', error_code = ?3, error_hint = ?4
             WHERE item_id = ?1",
            params![item_id, JobState::Failed.as_str(), code, hint],
        )?;
        Ok(())
    }

    // —— 暂停 ——

    /// 这个插件在这个范围上暂停到 `until`（Unix 毫秒），记下原因。你点的「先别试」不被它盖掉。
    pub fn pause(
        &self,
        plugin: &str,
        scope: &str,
        until: i64,
        code: &str,
        hint: Option<&str>,
    ) -> Result<()> {
        self.conn().execute(
            "INSERT INTO pauses (plugin, platform, until_at, code, hint, held) VALUES (?1, ?2, ?3, ?4, ?5, 0)
             ON CONFLICT (plugin, platform) DO UPDATE SET
                until_at = excluded.until_at, code = excluded.code, hint = excluded.hint
             WHERE pauses.held = 0",
            params![plugin, scope, until, code, hint],
        )?;
        Ok(())
    }

    /// 你点了「先别试」：这个插件在哪儿都不试（范围记成 `*`），到你点「现在再试」为止。
    pub fn hold(&self, plugin: &str) -> Result<()> {
        self.conn().execute(
            "INSERT INTO pauses (plugin, platform, until_at, code, hint, held) VALUES (?1, '*', NULL, NULL, NULL, 1)
             ON CONFLICT (plugin, platform) DO UPDATE SET until_at = NULL, code = NULL, hint = NULL, held = 1",
            params![plugin],
        )?;
        Ok(())
    }

    /// 「现在再试」：这个插件的暂停全去掉。
    pub fn clear_pauses(&self, plugin: &str) -> Result<()> {
        self.conn()
            .execute("DELETE FROM pauses WHERE plugin = ?1", params![plugin])?;
        Ok(())
    }

    /// 这个插件在这个范围上停着没有（你点的「先别试」哪儿都算）；到点的暂停顺手删掉。
    pub fn paused(&self, plugin: &str, scope: &str, now: i64) -> Result<Option<PauseRow>> {
        let conn = self.conn();
        let held = conn
            .query_row(
                "SELECT platform, until_at, code, hint, held FROM pauses
                 WHERE plugin = ?1 AND platform = '*' AND held = 1",
                params![plugin],
                row_to_pause,
            )
            .optional()?;
        if held.is_some() {
            return Ok(held);
        }
        let row = conn
            .query_row(
                "SELECT platform, until_at, code, hint, held FROM pauses WHERE plugin = ?1 AND platform = ?2",
                params![plugin, scope],
                row_to_pause,
            )
            .optional()?;
        match row {
            Some(p) if p.held || p.until.is_some_and(|t| t > now) => Ok(Some(p)),
            Some(_) => {
                conn.execute(
                    "DELETE FROM pauses WHERE plugin = ?1 AND platform = ?2",
                    params![plugin, scope],
                )?;
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// 这个插件现在所有的暂停。
    pub fn pauses_of(&self, plugin: &str, now: i64) -> Result<Vec<PauseRow>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT platform, until_at, code, hint, held FROM pauses
             WHERE plugin = ?1 AND (held = 1 OR until_at > ?2) ORDER BY platform",
        )?;
        let rows = stmt.query_map(params![plugin, now], row_to_pause)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    // —— 节奏 ——

    pub fn pace(&self, platform: &str) -> Result<PaceRow> {
        Ok(self
            .conn()
            .query_row(
                "SELECT last_end_at, day, batch_calls FROM pace WHERE platform = ?1",
                params![platform],
                |r| {
                    Ok(PaceRow {
                        last_end_at: r.get(0)?,
                        day: r.get(1)?,
                        batch_calls: r.get(2)?,
                    })
                },
            )
            .optional()?
            .unwrap_or_default())
    }

    /// 一次计入节奏的调用结束了；`batch` 的算进今天的次数。
    pub fn record_call(&self, platform: &str, end: i64, today: &str, batch: bool) -> Result<()> {
        let add = i64::from(batch);
        self.conn().execute(
            "INSERT INTO pace (platform, last_end_at, day, batch_calls) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (platform) DO UPDATE SET
                last_end_at = excluded.last_end_at,
                batch_calls = CASE WHEN pace.day = excluded.day THEN pace.batch_calls + ?4 ELSE ?4 END,
                day = excluded.day",
            params![platform, end, today, add],
        )?;
        Ok(())
    }

    // —— 导入 ——

    pub fn insert_import(&self, i: &ImportRow) -> Result<()> {
        self.conn().execute(
            "INSERT INTO imports (id, kind, plugin, source, client, title, action, created_at, listed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                i.id, i.kind, i.plugin, i.source, i.client, i.title, i.action, i.created_at, i.listed
            ],
        )?;
        Ok(())
    }

    pub fn import(&self, id: &str) -> Result<Option<ImportRow>> {
        Ok(self
            .conn()
            .query_row(
                &format!("SELECT {IMPORT_COLUMNS} FROM imports WHERE id = ?1"),
                params![id],
                row_to_import,
            )
            .optional()?)
    }

    /// 全部导入，新的在前。
    pub fn imports(&self) -> Result<Vec<ImportRow>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(&format!(
            "SELECT {IMPORT_COLUMNS} FROM imports ORDER BY created_at DESC, id DESC"
        ))?;
        let rows = stmt.query_map([], row_to_import)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// 列完了一页：游标往后，页数加一；`next` 为空就是列完了。
    pub fn listed_page(&self, id: &str, next: Option<&str>) -> Result<()> {
        self.conn().execute(
            "UPDATE imports SET cursor = ?2, listed = ?3, pages = pages + 1, attempts = 0,
                wait_until = NULL, stop_code = NULL, stop_hint = NULL WHERE id = ?1",
            params![id, next, next.is_none()],
        )?;
        Ok(())
    }

    pub fn mark_listed(&self, id: &str) -> Result<()> {
        self.conn().execute(
            "UPDATE imports SET listed = 1, wait_until = NULL WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// 过会儿再列，记下为什么等；`failed` 时次数加一。
    pub fn import_wait(
        &self,
        id: &str,
        until: i64,
        failed: bool,
        code: &str,
        hint: Option<&str>,
    ) -> Result<()> {
        self.conn().execute(
            "UPDATE imports SET wait_until = ?2, attempts = attempts + ?3, stop_code = ?4, stop_hint = ?5
             WHERE id = ?1",
            params![id, until, i64::from(failed), code, hint],
        )?;
        Ok(())
    }

    /// 停在这里，等你处理以后点「接着导入」（例如重新登录）。
    pub fn import_stop(&self, id: &str, code: &str, hint: Option<&str>) -> Result<()> {
        self.conn().execute(
            "UPDATE imports SET stop_code = ?2, stop_hint = ?3, wait_until = NULL WHERE id = ?1",
            params![id, code, hint],
        )?;
        Ok(())
    }

    /// 停下或接着：导入自己和它排着的任务一起。接着时原因清掉、马上再试。
    pub fn set_import_held(&self, id: &str, held: bool) -> Result<()> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        if held {
            tx.execute("UPDATE imports SET held = 1 WHERE id = ?1", params![id])?;
            tx.execute(
                "UPDATE fetch_jobs SET state = 'held' WHERE import_id = ?1 AND state = 'queued'",
                params![id],
            )?;
        } else {
            tx.execute(
                "UPDATE imports SET held = 0, stop_code = NULL, stop_hint = NULL, wait_until = NULL,
                    attempts = 0 WHERE id = ?1",
                params![id],
            )?;
            tx.execute(
                "UPDATE fetch_jobs SET state = 'queued' WHERE import_id = ?1 AND state = 'held'",
                params![id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn confirm_import(&self, id: &str, now: i64, also_trashed: bool) -> Result<()> {
        self.conn().execute(
            "UPDATE imports SET confirmed_at = ?2, also_trashed = ?3 WHERE id = ?1",
            params![id, now, also_trashed],
        )?;
        Ok(())
    }

    pub fn import_skipped(&self, id: &str) -> Result<()> {
        self.conn().execute(
            "UPDATE imports SET skipped = skipped + 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// 整个导入不要了：它的清单和还没取的任务一起删掉；已经收下的条目不动。
    pub fn delete_import(&self, id: &str) -> Result<()> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM fetch_jobs WHERE import_id = ?1 AND state IN ('queued', 'held')",
            params![id],
        )?;
        tx.execute(
            "UPDATE fetch_jobs SET import_id = NULL WHERE import_id = ?1",
            params![id],
        )?;
        tx.execute(
            "DELETE FROM import_entries WHERE import_id = ?1",
            params![id],
        )?;
        tx.execute("DELETE FROM imports WHERE id = ?1", params![id])?;
        tx.commit()?;
        Ok(())
    }

    /// 清单里加一条，顺序号接着排。
    pub fn add_entry(&self, import_id: &str, e: &EntryRow) -> Result<()> {
        self.conn().execute(
            "INSERT INTO import_entries
                (import_id, seq, url, title, action, acted_at, identity, item_id, standing, added)
             VALUES (?1, (SELECT COALESCE(MAX(seq), 0) + 1 FROM import_entries WHERE import_id = ?1),
                ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)",
            params![
                import_id, e.url, e.title, e.action, e.acted_at, e.identity, e.item_id, e.standing
            ],
        )?;
        Ok(())
    }

    /// 一批一起加，一个事务（交进来的一批最多几千条）。
    pub fn add_entries(&self, import_id: &str, entries: &[EntryRow]) -> Result<()> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO import_entries
                    (import_id, seq, url, title, action, acted_at, identity, item_id, standing, added)
                 VALUES (?1, (SELECT COALESCE(MAX(seq), 0) + 1 FROM import_entries WHERE import_id = ?1),
                    ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)",
            )?;
            for e in entries {
                stmt.execute(params![
                    import_id, e.url, e.title, e.action, e.acted_at, e.identity, e.item_id,
                    e.standing
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// 认出了身份：就地写上链接、身份、条目 id 和它现在在哪。
    pub fn resolve_entry(&self, import_id: &str, e: &EntryRow) -> Result<()> {
        self.conn().execute(
            "UPDATE import_entries SET url = ?3, identity = ?4, item_id = ?5, standing = ?6
             WHERE import_id = ?1 AND seq = ?2",
            params![import_id, e.seq, e.url, e.identity, e.item_id, e.standing],
        )?;
        Ok(())
    }

    /// 排着的任务（「现在再试」时挑出这个插件的，马上再排）。
    pub fn queued(&self) -> Result<Vec<FetchJob>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(&format!(
            "SELECT {JOB_COLUMNS} FROM fetch_jobs WHERE state = 'queued'"
        ))?;
        let rows = stmt.query_map([], row_to_job)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn due_now(&self, item_id: &str, now: i64) -> Result<()> {
        self.conn().execute(
            "UPDATE fetch_jobs SET due_at = ?2 WHERE item_id = ?1 AND state = 'queued' AND due_at > ?2",
            params![item_id, now],
        )?;
        Ok(())
    }

    /// 清单里有没有这一条了（去重）。
    pub fn has_entry_for(&self, import_id: &str, item_id: &str) -> Result<bool> {
        Ok(self
            .conn()
            .query_row(
                "SELECT 1 FROM import_entries WHERE import_id = ?1 AND item_id = ?2",
                params![import_id, item_id],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }

    pub fn entries(
        &self,
        import_id: &str,
        standing: Option<&str>,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<EntryRow>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(&format!(
            "SELECT {ENTRY_COLUMNS} FROM import_entries
             WHERE import_id = ?1 AND (?2 IS NULL OR standing = ?2)
             ORDER BY seq LIMIT ?3 OFFSET ?4"
        ))?;
        let rows = stmt.query_map(params![import_id, standing, limit, offset], row_to_entry)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn entry_for(&self, import_id: &str, item_id: &str) -> Result<Option<EntryRow>> {
        Ok(self
            .conn()
            .query_row(
                &format!(
                    "SELECT {ENTRY_COLUMNS} FROM import_entries WHERE import_id = ?1 AND item_id = ?2"
                ),
                params![import_id, item_id],
                row_to_entry,
            )
            .optional()?)
    }

    pub fn drop_entry(&self, import_id: &str, seq: i64) -> Result<()> {
        self.conn().execute(
            "DELETE FROM import_entries WHERE import_id = ?1 AND seq = ?2",
            params![import_id, seq],
        )?;
        Ok(())
    }

    pub fn mark_added(&self, import_id: &str, item_id: &str) -> Result<()> {
        self.conn().execute(
            "UPDATE import_entries SET added = 1 WHERE import_id = ?1 AND item_id = ?2",
            params![import_id, item_id],
        )?;
        Ok(())
    }

    /// 清单里各种 `standing` 各几条，和收下了几条。
    pub fn entry_counts(&self, import_id: &str) -> Result<EntryCounts> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT standing, COUNT(*), SUM(added) FROM import_entries WHERE import_id = ?1 GROUP BY standing",
        )?;
        let mut c = EntryCounts::default();
        let rows = stmt.query_map(params![import_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, u32>(1)?,
                r.get::<_, u32>(2)?,
            ))
        })?;
        for row in rows {
            let (standing, n, added) = row?;
            c.added += added;
            match standing.as_str() {
                "new" => c.new = n,
                "stored" => c.stored = n,
                "trashed" => c.trashed = n,
                _ => c.unresolved = n,
            }
        }
        Ok(c)
    }

    /// 这次导入还有几个任务没做完（排着、在跑、停着），几个取不到。
    pub fn import_jobs(&self, import_id: &str) -> Result<(u32, u32)> {
        Ok(self.conn().query_row(
            "SELECT COALESCE(SUM(state IN ('queued', 'running', 'held')), 0),
                    COALESCE(SUM(state = 'failed'), 0)
             FROM fetch_jobs WHERE import_id = ?1",
            params![import_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?)
    }

    /// 这次导入排着的任务里最先轮到的那个。
    pub fn import_first_job(&self, import_id: &str) -> Result<Option<FetchJob>> {
        Ok(self
            .conn()
            .query_row(
                &format!(
                    "SELECT {JOB_COLUMNS} FROM fetch_jobs WHERE import_id = ?1 AND state = 'queued'
                     ORDER BY due_at, item_id LIMIT 1"
                ),
                params![import_id],
                row_to_job,
            )
            .optional()?)
    }

    pub fn content_hash(&self, item_id: &str) -> Result<Option<String>> {
        Ok(self
            .conn()
            .query_row(
                "SELECT hash FROM content_hashes WHERE item_id = ?1",
                params![item_id],
                |r| r.get(0),
            )
            .optional()?)
    }

    pub fn set_content_hash(&self, item_id: &str, hash: &str) -> Result<()> {
        self.conn().execute(
            "INSERT INTO content_hashes (item_id, hash) VALUES (?1, ?2)
             ON CONFLICT (item_id) DO UPDATE SET hash = excluded.hash",
            params![item_id, hash],
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EntryCounts {
    pub new: u32,
    pub stored: u32,
    pub trashed: u32,
    pub unresolved: u32,
    pub added: u32,
}

fn row_to_job(r: &rusqlite::Row<'_>) -> rusqlite::Result<FetchJob> {
    Ok(FetchJob {
        item_id: r.get(0)?,
        url: r.get(1)?,
        state: JobState::parse(&r.get::<_, String>(2)?),
        attempts: r.get(3)?,
        due_at: r.get(4)?,
        error_code: r.get(5)?,
        error_hint: r.get(6)?,
        priority: r.get(7)?,
        import_id: r.get(8)?,
    })
}

fn row_to_pause(r: &rusqlite::Row<'_>) -> rusqlite::Result<PauseRow> {
    Ok(PauseRow {
        scope: r.get(0)?,
        until: r.get(1)?,
        code: r.get(2)?,
        hint: r.get(3)?,
        held: r.get(4)?,
    })
}

fn row_to_import(r: &rusqlite::Row<'_>) -> rusqlite::Result<ImportRow> {
    Ok(ImportRow {
        id: r.get(0)?,
        kind: r.get(1)?,
        plugin: r.get(2)?,
        source: r.get(3)?,
        client: r.get(4)?,
        title: r.get(5)?,
        action: r.get(6)?,
        created_at: r.get(7)?,
        cursor: r.get(8)?,
        listed: r.get(9)?,
        pages: r.get(10)?,
        attempts: r.get(11)?,
        wait_until: r.get(12)?,
        stop_code: r.get(13)?,
        stop_hint: r.get(14)?,
        held: r.get(15)?,
        confirmed_at: r.get(16)?,
        also_trashed: r.get(17)?,
        skipped: r.get(18)?,
    })
}

fn row_to_entry(r: &rusqlite::Row<'_>) -> rusqlite::Result<EntryRow> {
    Ok(EntryRow {
        seq: r.get(0)?,
        url: r.get(1)?,
        title: r.get(2)?,
        action: r.get(3)?,
        acted_at: r.get(4)?,
        identity: r.get(5)?,
        item_id: r.get(6)?,
        standing: r.get(7)?,
        added: r.get(8)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_retry_and_resume() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jobs.db");
        let jobs = Jobs::open(&path, 0).unwrap();
        jobs.enqueue("web-a", "https://a.test/", 10).unwrap();
        jobs.enqueue("web-a", "https://a.test/other", 10).unwrap();
        assert!(jobs.due(5).unwrap().is_empty());
        let j = jobs.due(10).unwrap().remove(0);
        assert!(jobs.claim(&j.item_id).unwrap());
        assert!(!jobs.claim(&j.item_id).unwrap(), "拿走了的不能再拿");
        assert_eq!(j.url, "https://a.test/", "排着的不被第二次覆盖");
        jobs.retry("web-a", 50, "network", None).unwrap();
        assert_eq!(jobs.next_due_after(10).unwrap(), Some(50));
        let j = jobs.due(50).unwrap().remove(0);
        assert!(jobs.claim(&j.item_id).unwrap());
        assert_eq!(j.attempts, 1);
        drop(jobs);
        // 重启：running 回到 queued
        let jobs = Jobs::open(&path, 50).unwrap();
        assert_eq!(jobs.job("web-a").unwrap().unwrap().state, JobState::Queued);
        jobs.fail("web-a", "not-found", Some("没了")).unwrap();
        jobs.enqueue("web-a", "https://a.test/", 60).unwrap();
        let j = jobs.job("web-a").unwrap().unwrap();
        assert_eq!((j.state, j.attempts), (JobState::Queued, 0));
    }

    #[test]
    fn pauses_expire_but_holds_stay() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jobs.db");
        let jobs = Jobs::open(&path, 0).unwrap();
        jobs.pause("a/b", "a.test", 100, "rate-limited", Some("慢点"))
            .unwrap();
        let p = jobs.paused("a/b", "a.test", 50).unwrap().unwrap();
        assert_eq!(
            (p.until, p.code.as_deref()),
            (Some(100), Some("rate-limited"))
        );
        assert_eq!(jobs.paused("a/b", "a.test", 100).unwrap(), None);
        jobs.hold("a/b").unwrap();
        jobs.pause("a/b", "a.test", 10, "rate-limited", None)
            .unwrap();
        assert!(
            jobs.paused("a/b", "a.test", 1_000).unwrap().unwrap().held,
            "限流不盖掉你点的"
        );
        assert!(
            jobs.paused("a/b", "b.test", 1_000).unwrap().unwrap().held,
            "哪儿都算"
        );
        drop(jobs);
        let jobs = Jobs::open(&path, 0).unwrap();
        assert!(
            jobs.pauses_of("a/b", 0).unwrap().iter().any(|p| p.held),
            "重启后还在"
        );
        jobs.clear_pauses("a/b").unwrap();
        assert!(jobs.pauses_of("a/b", 0).unwrap().is_empty());
    }

    #[test]
    fn pace_counts_batch_calls_per_day() {
        let dir = tempfile::tempdir().unwrap();
        let jobs = Jobs::open(&dir.path().join("jobs.db"), 0).unwrap();
        jobs.record_call("p", 10, "2026-11-11", true).unwrap();
        jobs.record_call("p", 20, "2026-11-11", false).unwrap();
        jobs.record_call("p", 30, "2026-11-11", true).unwrap();
        assert_eq!(
            jobs.pace("p").unwrap(),
            PaceRow {
                last_end_at: 30,
                day: "2026-11-11".into(),
                batch_calls: 2
            }
        );
        jobs.record_call("p", 40, "2026-11-12", true).unwrap();
        assert_eq!(jobs.pace("p").unwrap().batch_calls, 1, "换了一天重新数");
    }
}
