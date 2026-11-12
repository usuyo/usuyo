//! 本机任务队列 `jobs.db`（SPEC §1.1）：取内容的任务、插件的暂停、平台的节奏、导入、转换的任务，
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

/// 版本 1：任务的优先级和所属导入、暂停的原因和「先别试」、平台节奏、导入（SPEC §1.1、插件协议 §3.1）。
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

/// 版本 2：转换的任务（SPEC §1.1）。一条一个 `job` 一行；`trigger`、`redo_of` 是写 `run.json` 时要的来历。
const SCHEMA_V2: &str = "
CREATE TABLE transform_jobs (
    item_id    TEXT NOT NULL,
    job        TEXT NOT NULL,
    plugin     TEXT,
    options    TEXT,
    state      TEXT NOT NULL,
    wait       TEXT,
    priority   INTEGER NOT NULL,
    due_at     INTEGER NOT NULL,
    attempts   INTEGER NOT NULL DEFAULT 0,
    error_code TEXT,
    error_hint TEXT,
    trigger    TEXT NOT NULL DEFAULT 'auto',
    redo_of    TEXT,
    PRIMARY KEY (item_id, job)
);
PRAGMA user_version = 2;
";

/// 版本 3：获取插件里有几种方法（协议 §2.3.1）。暂停按 (插件, 方法, 范围) 记，「先别试」的方法和范围都是 `*`；
/// 导入记它的来源属于哪种方法；每种方法上一次调用的结果（成没成、错误码和一句话）。
/// 旧的暂停只是一时的，不搬。
const SCHEMA_V3: &str = "
DROP TABLE pauses;
CREATE TABLE pauses (
    plugin   TEXT NOT NULL,
    method   TEXT NOT NULL,
    platform TEXT NOT NULL,
    until_at INTEGER,
    code     TEXT,
    hint     TEXT,
    held     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (plugin, method, platform)
);
ALTER TABLE imports ADD COLUMN method TEXT;
CREATE TABLE method_calls (
    plugin TEXT NOT NULL,
    method TEXT NOT NULL,
    at     INTEGER NOT NULL,
    code   TEXT,
    hint   TEXT,
    PRIMARY KEY (plugin, method)
);
PRAGMA user_version = 3;
";

/// 版本 4：从文件导入（协议 §3.9）。导入记交进来的文件名（副本在按机器目录的 `imports/<导入 id>/` 下）；
/// 清单里的一条可以只有插件的 `ref`、没有链接（`url` 记空串），宿主回到那个插件 `fetch <ref>`。
const SCHEMA_V4: &str = "
ALTER TABLE imports ADD COLUMN file_name TEXT;
ALTER TABLE import_entries ADD COLUMN ref TEXT;
PRAGMA user_version = 4;
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
    /// 停的是哪种方法；`*` 是你点的「先别试」，整个插件。
    pub method: String,
    pub scope: String,
    pub until: Option<i64>,
    pub code: Option<String>,
    pub hint: Option<String>,
    pub held: bool,
}

/// 一种方法上一次调用：什么时候结束、成没成（`code` 空是成了）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodCall {
    pub method: String,
    pub at: i64,
    pub code: Option<String>,
    pub hint: Option<String>,
}

/// 一个平台的节奏记账：上次调用什么时候结束，今天（本地日期）批量调用了几次。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaceRow {
    pub last_end_at: i64,
    pub day: String,
    pub batch_calls: u32,
}

/// 一次导入。`kind`：`source`（插件的 `list`）、`delivered`（交进来的一批链接）或 `file`（插件读交进来的文件）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRow {
    pub id: String,
    pub kind: String,
    pub plugin: Option<String>,
    /// `source` 类：来源属于插件的哪种方法。
    pub method: Option<String>,
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
    /// `file` 类：交进来的文件名，只当显示的字（副本叫 `input` 加允许的扩展名，见 `file_imports`）。
    pub file_name: Option<String>,
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
    /// 插件给的 `ref`（从文件导入）：轮到时回到这个插件 `fetch <ref>`。只有它、没有链接时 `url` 是空串。
    pub reference: Option<String>,
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
    listed, pages, attempts, wait_until, stop_code, stop_hint, held, confirmed_at, also_trashed, skipped,
    method, file_name";
const ENTRY_COLUMNS: &str =
    "seq, url, title, action, acted_at, identity, item_id, standing, added, ref";

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
        if version < 2 {
            conn.execute_batch(&format!("BEGIN; {SCHEMA_V2} COMMIT;"))?;
        }
        if version < 3 {
            conn.execute_batch(&format!("BEGIN; {SCHEMA_V3} COMMIT;"))?;
        }
        if version < 4 {
            conn.execute_batch(&format!("BEGIN; {SCHEMA_V4} COMMIT;"))?;
        }
        conn.execute(
            "UPDATE fetch_jobs SET state = 'queued' WHERE state = 'running'",
            [],
        )?;
        conn.execute(
            "UPDATE transform_jobs SET state = 'queued' WHERE state = 'running'",
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

    /// 这个插件的这种方法在这个范围上暂停到 `until`（Unix 毫秒），记下原因。你点的「先别试」不被它盖掉。
    pub fn pause(
        &self,
        plugin: &str,
        method: &str,
        scope: &str,
        until: i64,
        code: &str,
        hint: Option<&str>,
    ) -> Result<()> {
        self.conn().execute(
            "INSERT INTO pauses (plugin, method, platform, until_at, code, hint, held)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)
             ON CONFLICT (plugin, method, platform) DO UPDATE SET
                until_at = excluded.until_at, code = excluded.code, hint = excluded.hint
             WHERE pauses.held = 0",
            params![plugin, method, scope, until, code, hint],
        )?;
        Ok(())
    }

    /// 你点了「先别试」：这个插件的哪种方法在哪儿都不试（方法和范围记成 `*`），到你点「现在再试」为止。
    pub fn hold(&self, plugin: &str) -> Result<()> {
        self.conn().execute(
            "INSERT INTO pauses (plugin, method, platform, until_at, code, hint, held)
             VALUES (?1, '*', '*', NULL, NULL, NULL, 1)
             ON CONFLICT (plugin, method, platform) DO UPDATE SET
                until_at = NULL, code = NULL, hint = NULL, held = 1",
            params![plugin],
        )?;
        Ok(())
    }

    /// 「现在再试」：这个插件所有方法的暂停全去掉。
    pub fn clear_pauses(&self, plugin: &str) -> Result<()> {
        self.conn()
            .execute("DELETE FROM pauses WHERE plugin = ?1", params![plugin])?;
        Ok(())
    }

    /// 这种方法在这个范围上停着没有（你点的「先别试」哪儿都算）；到点的暂停顺手删掉。
    pub fn paused(
        &self,
        plugin: &str,
        method: &str,
        scope: &str,
        now: i64,
    ) -> Result<Option<PauseRow>> {
        let conn = self.conn();
        let held = conn
            .query_row(
                "SELECT method, platform, until_at, code, hint, held FROM pauses
                 WHERE plugin = ?1 AND method = '*' AND platform = '*' AND held = 1",
                params![plugin],
                row_to_pause,
            )
            .optional()?;
        if held.is_some() {
            return Ok(held);
        }
        let row = conn
            .query_row(
                "SELECT method, platform, until_at, code, hint, held FROM pauses
                 WHERE plugin = ?1 AND method = ?2 AND platform = ?3",
                params![plugin, method, scope],
                row_to_pause,
            )
            .optional()?;
        match row {
            Some(p) if p.held || p.until.is_some_and(|t| t > now) => Ok(Some(p)),
            Some(_) => {
                conn.execute(
                    "DELETE FROM pauses WHERE plugin = ?1 AND method = ?2 AND platform = ?3",
                    params![plugin, method, scope],
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
            "SELECT method, platform, until_at, code, hint, held FROM pauses
             WHERE plugin = ?1 AND (held = 1 OR until_at > ?2) ORDER BY method, platform",
        )?;
        let rows = stmt.query_map(params![plugin, now], row_to_pause)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    // —— 每种方法上一次调用 ——

    /// 记下这种方法这一次调用的结果，盖掉上一次的；`code` 空是成了。
    pub fn record_method_call(
        &self,
        plugin: &str,
        method: &str,
        at: i64,
        code: Option<&str>,
        hint: Option<&str>,
    ) -> Result<()> {
        self.conn().execute(
            "INSERT INTO method_calls (plugin, method, at, code, hint) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT (plugin, method) DO UPDATE SET
                at = excluded.at, code = excluded.code, hint = excluded.hint",
            params![plugin, method, at, code, hint],
        )?;
        Ok(())
    }

    /// 这个插件每种方法上一次调用的结果。
    pub fn method_calls(&self, plugin: &str) -> Result<Vec<MethodCall>> {
        let conn = self.conn();
        let mut stmt =
            conn.prepare("SELECT method, at, code, hint FROM method_calls WHERE plugin = ?1")?;
        let rows = stmt.query_map(params![plugin], |r| {
            Ok(MethodCall {
                method: r.get(0)?,
                at: r.get(1)?,
                code: r.get(2)?,
                hint: r.get(3)?,
            })
        })?;
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
            "INSERT INTO imports (id, kind, plugin, method, source, client, title, action, created_at, listed,
                file_name)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                i.id,
                i.kind,
                i.plugin,
                i.method,
                i.source,
                i.client,
                i.title,
                i.action,
                i.created_at,
                i.listed,
                i.file_name
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
                (import_id, seq, url, title, action, acted_at, identity, item_id, standing, added, ref)
             VALUES (?1, (SELECT COALESCE(MAX(seq), 0) + 1 FROM import_entries WHERE import_id = ?1),
                ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9)",
            params![
                import_id,
                e.url,
                e.title,
                e.action,
                e.acted_at,
                e.identity,
                e.item_id,
                e.standing,
                e.reference
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

    /// 清单里有没有带这个 `ref` 的了（没有身份的那几条按它去重）。
    pub fn has_entry_with_ref(&self, import_id: &str, reference: &str) -> Result<bool> {
        Ok(self
            .conn()
            .query_row(
                "SELECT 1 FROM import_entries WHERE import_id = ?1 AND ref = ?2",
                params![import_id, reference],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
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

    // —— 转换 ——

    /// 排一个转换任务。已经有这一行时：先后取两者里靠前的（靠前了就换上这次的 `trigger`、到点时间不晚于这次）；
    /// `revive` 时失败过的重新开始（快照新来了，输入可能变了）；排着、等着、在跑的状态不动。
    pub fn schedule_transform(
        &self,
        item_id: &str,
        job: TransformJobKind,
        priority: i64,
        trigger: &str,
        revive: bool,
        now: i64,
    ) -> Result<()> {
        self.conn().execute(
            "INSERT INTO transform_jobs (item_id, job, state, priority, due_at, trigger)
             VALUES (?1, ?2, 'queued', ?3, ?4, ?5)
             ON CONFLICT (item_id, job) DO UPDATE SET
                trigger = CASE WHEN excluded.priority < transform_jobs.priority
                    AND transform_jobs.trigger != 'redo' THEN excluded.trigger
                    ELSE transform_jobs.trigger END,
                due_at = CASE
                    WHEN transform_jobs.state = 'failed' AND ?6 THEN excluded.due_at
                    WHEN excluded.priority < transform_jobs.priority
                        THEN MIN(transform_jobs.due_at, excluded.due_at)
                    ELSE transform_jobs.due_at END,
                priority = MIN(transform_jobs.priority, excluded.priority),
                attempts = CASE WHEN transform_jobs.state = 'failed' AND ?6 THEN 0
                    ELSE transform_jobs.attempts END,
                error_code = CASE WHEN transform_jobs.state = 'failed' AND ?6 THEN NULL
                    ELSE transform_jobs.error_code END,
                error_hint = CASE WHEN transform_jobs.state = 'failed' AND ?6 THEN NULL
                    ELSE transform_jobs.error_hint END,
                state = CASE WHEN transform_jobs.state = 'failed' AND ?6 THEN 'queued'
                    ELSE transform_jobs.state END",
            params![item_id, job.as_str(), priority, now, trigger, revive],
        )?;
        Ok(())
    }

    /// 启动时补排：已经有这一行（含失败过的）就不动。
    pub fn backfill_transform(
        &self,
        item_id: &str,
        job: TransformJobKind,
        priority: i64,
        now: i64,
    ) -> Result<()> {
        self.conn().execute(
            "INSERT INTO transform_jobs (item_id, job, state, priority, due_at, trigger)
             VALUES (?1, ?2, 'queued', ?3, ?4, 'auto')
             ON CONFLICT (item_id, job) DO NOTHING",
            params![item_id, job.as_str(), priority, now],
        )?;
        Ok(())
    }

    /// 「再转一次」「换一个转」：换上给的插件和参数，挪到最前。这一行正在跑时什么都不改，回 `false`。
    pub fn redo_transform(&self, redo: &TransformRedo<'_>, now: i64) -> Result<bool> {
        let n = self.conn().execute(
            "INSERT INTO transform_jobs
                (item_id, job, plugin, options, state, priority, due_at, trigger, redo_of)
             VALUES (?1, ?2, ?3, ?4, 'queued', 0, ?5, 'redo', ?6)
             ON CONFLICT (item_id, job) DO UPDATE SET
                plugin = excluded.plugin, options = excluded.options, state = 'queued', wait = NULL,
                priority = 0, due_at = excluded.due_at, attempts = 0, error_code = NULL,
                error_hint = NULL, trigger = 'redo', redo_of = excluded.redo_of
             WHERE transform_jobs.state != 'running'",
            params![
                redo.item_id,
                redo.job.as_str(),
                redo.plugin,
                redo.options,
                now,
                redo.redo_of
            ],
        )?;
        Ok(n == 1)
    }

    /// 接没接电源变了：在电池上，自动的（不是点名的）排着的改成等电源；接着电，或者被点了名，就回到排着。
    pub fn gate_power(&self, on_mains: bool) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE transform_jobs SET state = 'queued', wait = NULL
             WHERE state = 'waiting' AND wait = 'power' AND (?1 OR priority = 0)",
            params![on_mains],
        )?;
        if !on_mains {
            conn.execute(
                "UPDATE transform_jobs SET state = 'waiting', wait = 'power'
                 WHERE state = 'queued' AND priority > 0",
                [],
            )?;
        }
        Ok(())
    }

    /// 到点、排着的里面最先轮到的：点名的在前，再按到点时间、条目、一条里的先后。
    pub fn next_transform(&self, now: i64) -> Result<Option<TransformJob>> {
        Ok(self
            .conn()
            .query_row(
                &format!(
                    "SELECT {TRANSFORM_COLUMNS} FROM transform_jobs
                     WHERE state = 'queued' AND due_at <= ?1
                     ORDER BY priority, due_at, item_id, {JOB_ORDER} LIMIT 1"
                ),
                params![now],
                row_to_transform,
            )
            .optional()?)
    }

    /// 标成 `running`；不在排了回 `false`。
    pub fn claim_transform(&self, item_id: &str, job: TransformJobKind) -> Result<bool> {
        let n = self.conn().execute(
            "UPDATE transform_jobs SET state = 'running'
             WHERE item_id = ?1 AND job = ?2 AND state = 'queued'",
            params![item_id, job.as_str()],
        )?;
        Ok(n == 1)
    }

    /// 排着、还没到点的里面最早的到点时间。
    pub fn next_transform_due_after(&self, now: i64) -> Result<Option<i64>> {
        Ok(self.conn().query_row(
            "SELECT MIN(due_at) FROM transform_jobs WHERE state = 'queued' AND due_at > ?1",
            params![now],
            |r| r.get(0),
        )?)
    }

    /// 做完了，或者不用做了：这一行删掉。
    pub fn finish_transform(&self, item_id: &str, job: TransformJobKind) -> Result<()> {
        self.conn().execute(
            "DELETE FROM transform_jobs WHERE item_id = ?1 AND job = ?2",
            params![item_id, job.as_str()],
        )?;
        Ok(())
    }

    /// 这一条的转换任务全删掉（进回收站、只留文字、清掉）。
    pub fn drop_transforms(&self, item_id: &str) -> Result<()> {
        self.conn().execute(
            "DELETE FROM transform_jobs WHERE item_id = ?1",
            params![item_id],
        )?;
        Ok(())
    }

    /// 过会儿再试：次数加一。
    pub fn retry_transform(
        &self,
        item_id: &str,
        job: TransformJobKind,
        due_at: i64,
        code: &str,
        hint: Option<&str>,
    ) -> Result<()> {
        self.conn().execute(
            "UPDATE transform_jobs SET state = 'queued', attempts = attempts + 1, due_at = ?3,
                error_code = ?4, error_hint = ?5 WHERE item_id = ?1 AND job = ?2",
            params![item_id, job.as_str(), due_at, code, hint],
        )?;
        Ok(())
    }

    /// 不再自动试，原因留着给条目页那一面写。
    pub fn fail_transform(
        &self,
        item_id: &str,
        job: TransformJobKind,
        code: &str,
        hint: Option<&str>,
    ) -> Result<()> {
        self.conn().execute(
            "UPDATE transform_jobs SET state = 'failed', error_code = ?3, error_hint = ?4
             WHERE item_id = ?1 AND job = ?2",
            params![item_id, job.as_str(), code, hint],
        )?;
        Ok(())
    }

    /// 等这个模型：不算一次失败，次数不变。
    pub fn wait_for_model(&self, item_id: &str, job: TransformJobKind, model: &str) -> Result<()> {
        self.conn().execute(
            "UPDATE transform_jobs SET state = 'waiting', wait = ?3
             WHERE item_id = ?1 AND job = ?2",
            params![item_id, job.as_str(), format!("model:{model}")],
        )?;
        Ok(())
    }

    /// 等着的模型的名字，去重。
    pub fn waited_models(&self) -> Result<Vec<String>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT substr(wait, 7) FROM transform_jobs
             WHERE state = 'waiting' AND wait LIKE 'model:%' ORDER BY 1",
        )?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// 模型到了：等它的回到排着，马上能做。回这几行的条目 id（去重），它们的条目页要重拉。
    pub fn model_arrived(&self, model: &str, now: i64) -> Result<Vec<String>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "UPDATE transform_jobs SET state = 'queued', wait = NULL, due_at = ?2
             WHERE state = 'waiting' AND wait = ?1 RETURNING item_id",
        )?;
        let rows = stmt.query_map(params![format!("model:{model}"), now], |r| r.get(0))?;
        let mut ids: Vec<String> = rows.collect::<rusqlite::Result<_>>()?;
        ids.sort();
        ids.dedup();
        Ok(ids)
    }

    /// 缺插件、缺程序（`dependency-missing`）停下的：重新排上，启动时看装了没有。
    pub fn revive_missing(&self, now: i64) -> Result<()> {
        self.conn().execute(
            "UPDATE transform_jobs SET state = 'queued', attempts = 0, due_at = ?1,
                error_code = NULL, error_hint = NULL
             WHERE state = 'failed' AND error_code = 'dependency-missing'",
            params![now],
        )?;
        Ok(())
    }

    /// `transforms.auto` 关掉了：自动排的（不是点名的）、没在跑的都删掉；回删了哪几条。
    pub fn drop_automatic(&self) -> Result<Vec<String>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "DELETE FROM transform_jobs WHERE priority > 0 AND state != 'running' RETURNING item_id",
        )?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        let mut ids: Vec<String> = rows.collect::<rusqlite::Result<_>>()?;
        ids.sort();
        ids.dedup();
        Ok(ids)
    }

    /// 一条的转换任务，按一条里的先后。
    pub fn transforms_of(&self, item_id: &str) -> Result<Vec<TransformJob>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(&format!(
            "SELECT {TRANSFORM_COLUMNS} FROM transform_jobs WHERE item_id = ?1 ORDER BY {JOB_ORDER}"
        ))?;
        let rows = stmt.query_map(params![item_id], row_to_transform)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

/// 一条里五种转换任务（feature 003 §4.4）：能播的、口播、关键帧、图里的字、关键帧上的字。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransformJobKind {
    Play,
    Asr,
    Keyframes,
    Ocr,
    Frames,
}

impl TransformJobKind {
    pub const ALL: [TransformJobKind; 5] = [
        TransformJobKind::Play,
        TransformJobKind::Asr,
        TransformJobKind::Keyframes,
        TransformJobKind::Ocr,
        TransformJobKind::Frames,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            TransformJobKind::Play => "play",
            TransformJobKind::Asr => "asr",
            TransformJobKind::Keyframes => "keyframes",
            TransformJobKind::Ocr => "ocr",
            TransformJobKind::Frames => "frames",
        }
    }

    pub fn parse(s: &str) -> Option<TransformJobKind> {
        Self::ALL.into_iter().find(|j| j.as_str() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformState {
    Queued,
    Running,
    /// 等电源或模型，原因在 `wait`。
    Waiting,
    /// 不再自动试；原因在 `error_code`、`error_hint`。
    Failed,
}

impl TransformState {
    fn parse(s: &str) -> TransformState {
        match s {
            "running" => TransformState::Running,
            "waiting" => TransformState::Waiting,
            "failed" => TransformState::Failed,
            _ => TransformState::Queued,
        }
    }
}

/// `transform_jobs` 的一行。时间都是 Unix 毫秒。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformJob {
    pub item_id: String,
    pub job: TransformJobKind,
    /// 点名换插件时有；空是用默认的。
    pub plugin: Option<String>,
    /// 点名时给的参数，JSON 对象的文字。
    pub options: Option<String>,
    pub state: TransformState,
    /// `power` 或 `model:<名字>`。
    pub wait: Option<String>,
    pub priority: i64,
    pub due_at: i64,
    pub attempts: u32,
    pub error_code: Option<String>,
    pub error_hint: Option<String>,
    /// `auto` `open` `redo`
    pub trigger: String,
    pub redo_of: Option<String>,
}

/// 一次「再转一次」要写进 `transform_jobs` 的。
pub struct TransformRedo<'a> {
    pub item_id: &'a str,
    pub job: TransformJobKind,
    pub plugin: Option<&'a str>,
    pub options: Option<&'a str>,
    pub redo_of: Option<&'a str>,
}

const TRANSFORM_COLUMNS: &str = "item_id, job, plugin, options, state, wait, priority, due_at,
    attempts, error_code, error_hint, trigger, redo_of";
/// 一条里的先后：先合起来能播，再口播、关键帧、图里的字，最后关键帧上的字。
const JOB_ORDER: &str = "CASE job WHEN 'play' THEN 0 WHEN 'asr' THEN 1 WHEN 'keyframes' THEN 2
    WHEN 'ocr' THEN 3 ELSE 4 END";

fn row_to_transform(r: &rusqlite::Row<'_>) -> rusqlite::Result<TransformJob> {
    let job: String = r.get(1)?;
    Ok(TransformJob {
        item_id: r.get(0)?,
        job: TransformJobKind::parse(&job).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                format!("unknown transform job `{job}`").into(),
            )
        })?,
        plugin: r.get(2)?,
        options: r.get(3)?,
        state: TransformState::parse(&r.get::<_, String>(4)?),
        wait: r.get(5)?,
        priority: r.get(6)?,
        due_at: r.get(7)?,
        attempts: r.get(8)?,
        error_code: r.get(9)?,
        error_hint: r.get(10)?,
        trigger: r.get(11)?,
        redo_of: r.get(12)?,
    })
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
        method: r.get(0)?,
        scope: r.get(1)?,
        until: r.get(2)?,
        code: r.get(3)?,
        hint: r.get(4)?,
        held: r.get(5)?,
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
        method: r.get(19)?,
        file_name: r.get(20)?,
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
        reference: r.get(9)?,
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
        jobs.pause("a/b", "guest", "a.test", 100, "rate-limited", Some("慢点"))
            .unwrap();
        let p = jobs.paused("a/b", "guest", "a.test", 50).unwrap().unwrap();
        assert_eq!(
            (p.until, p.code.as_deref(), p.method.as_str()),
            (Some(100), Some("rate-limited"), "guest")
        );
        assert_eq!(
            jobs.paused("a/b", "embed", "a.test", 50).unwrap(),
            None,
            "一种方法停着，别的方法照常"
        );
        assert_eq!(jobs.paused("a/b", "guest", "a.test", 100).unwrap(), None);
        jobs.hold("a/b").unwrap();
        jobs.pause("a/b", "guest", "a.test", 10, "rate-limited", None)
            .unwrap();
        assert!(
            jobs.paused("a/b", "guest", "a.test", 1_000)
                .unwrap()
                .unwrap()
                .held,
            "限流不盖掉你点的"
        );
        assert!(
            jobs.paused("a/b", "embed", "b.test", 1_000)
                .unwrap()
                .unwrap()
                .held,
            "哪种方法、哪儿都算"
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
    fn last_call_per_method_survives_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jobs.db");
        let jobs = Jobs::open(&path, 0).unwrap();
        jobs.record_method_call("a/b", "guest", 5, Some("blocked"), Some("要验证"))
            .unwrap();
        jobs.record_method_call("a/b", "guest", 9, None, None)
            .unwrap();
        jobs.record_method_call("a/b", "embed", 7, Some("network"), None)
            .unwrap();
        drop(jobs);
        let jobs = Jobs::open(&path, 0).unwrap();
        let mut calls = jobs.method_calls("a/b").unwrap();
        calls.sort_by(|x, y| x.method.cmp(&y.method));
        assert_eq!(
            calls,
            vec![
                MethodCall {
                    method: "embed".into(),
                    at: 7,
                    code: Some("network".into()),
                    hint: None
                },
                MethodCall {
                    method: "guest".into(),
                    at: 9,
                    code: None,
                    hint: None
                },
            ]
        );
    }

    #[test]
    fn transform_order_power_models_redo_and_restart() {
        use TransformJobKind::*;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jobs.db");
        let jobs = Jobs::open(&path, 0).unwrap();
        jobs.schedule_transform("web-b", Ocr, 2, "auto", true, 10)
            .unwrap();
        jobs.schedule_transform("web-a", Asr, 1, "auto", true, 20)
            .unwrap();
        jobs.schedule_transform("web-a", Play, 1, "auto", true, 20)
            .unwrap();
        // 手动收的在导入的前面，一条里先合起来能播
        let next = jobs.next_transform(100).unwrap().unwrap();
        assert_eq!((next.item_id.as_str(), next.job), ("web-a", Play));
        // 点名：挪到最前，换上 open
        jobs.schedule_transform("web-b", Ocr, 0, "open", false, 30)
            .unwrap();
        let next = jobs.next_transform(100).unwrap().unwrap();
        assert_eq!(
            (next.item_id.as_str(), next.trigger.as_str()),
            ("web-b", "open")
        );
        // 在电池上：点名的照常，自动的等电源
        jobs.gate_power(false).unwrap();
        let waiting: Vec<_> = jobs
            .transforms_of("web-a")
            .unwrap()
            .into_iter()
            .map(|j| (j.state, j.wait))
            .collect();
        assert!(
            waiting
                .iter()
                .all(|w| *w == (TransformState::Waiting, Some("power".into())))
        );
        assert_eq!(jobs.next_transform(100).unwrap().unwrap().item_id, "web-b");
        jobs.gate_power(true).unwrap();
        assert!(
            jobs.transforms_of("web-a")
                .unwrap()
                .iter()
                .all(|j| j.state == TransformState::Queued)
        );

        // 等模型不算次数，模型到了接着排
        assert!(jobs.claim_transform("web-a", Asr).unwrap());
        jobs.wait_for_model("web-a", Asr, "m").unwrap();
        assert_eq!(jobs.waited_models().unwrap(), vec!["m".to_owned()]);
        jobs.model_arrived("m", 50).unwrap();
        let asr = jobs.transforms_of("web-a").unwrap().remove(1);
        assert_eq!(
            (asr.job, asr.state, asr.attempts),
            (Asr, TransformState::Queued, 0)
        );

        // 正在跑的不能再转一次
        assert!(jobs.claim_transform("web-a", Asr).unwrap());
        let redo = TransformRedo {
            item_id: "web-a",
            job: Asr,
            plugin: Some("x/y"),
            options: Some("{}"),
            redo_of: None,
        };
        assert!(!jobs.redo_transform(&redo, 60).unwrap());
        // 重启：跑着的回到排着
        drop(jobs);
        let jobs = Jobs::open(&path, 70).unwrap();
        assert!(jobs.redo_transform(&redo, 80).unwrap());
        let asr = jobs.transforms_of("web-a").unwrap().remove(1);
        assert_eq!(
            (asr.priority, asr.trigger.as_str(), asr.plugin.as_deref()),
            (0, "redo", Some("x/y"))
        );

        // 失败过的：新快照来了重新开始，打开时不动
        jobs.fail_transform("web-b", Ocr, "internal", None).unwrap();
        jobs.schedule_transform("web-b", Ocr, 0, "open", false, 90)
            .unwrap();
        assert_eq!(
            jobs.transforms_of("web-b").unwrap()[0].state,
            TransformState::Failed
        );
        jobs.schedule_transform("web-b", Ocr, 2, "auto", true, 90)
            .unwrap();
        assert_eq!(
            jobs.transforms_of("web-b").unwrap()[0].state,
            TransformState::Queued
        );
        jobs.drop_transforms("web-b").unwrap();
        assert!(jobs.transforms_of("web-b").unwrap().is_empty());
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
