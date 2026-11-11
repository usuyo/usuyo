//! 本机任务队列 `jobs.db`（SPEC §1.1）：取内容的任务、插件的暂停、上次写的 `content.md` 的哈希。
//! 暂停按 (插件, 范围) 记：平台插件的范围是平台，通用网页插件的是网站的主机名（列名沿用 `platform`）。
//!
//! 一条一个任务（按 `item_id` 去重）；`running` 的在重新打开时放回 `queued`，重启后接着做。

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Queued,
    Running,
    /// 不再自动重试；原因在 `error_code`、`error_hint`。
    Failed,
}

impl JobState {
    fn as_str(self) -> &'static str {
        match self {
            JobState::Queued => "queued",
            JobState::Running => "running",
            JobState::Failed => "failed",
        }
    }

    fn parse(s: &str) -> JobState {
        match s {
            "running" => JobState::Running,
            "failed" => JobState::Failed,
            _ => JobState::Queued,
        }
    }
}

pub(crate) struct Jobs {
    conn: Mutex<Connection>,
}

impl Jobs {
    /// 打开（没有就建），把上次没做完的 `running` 放回队列。
    ///
    /// 旧版本存下的暂停没有上限（一个 `Retry-After` 能停到几千年后），也有等「继续」的、
    /// 没有入口能解开的暂停：超过现在的上限的暂停删掉，到点时间远过重试上限的任务马上排。
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
        conn.execute(
            "UPDATE fetch_jobs SET state = 'queued' WHERE state = 'running'",
            [],
        )?;
        let ms = |d: std::time::Duration| i64::try_from(d.as_millis()).unwrap_or(i64::MAX);
        conn.execute(
            "DELETE FROM pauses WHERE until_at IS NULL OR until_at > ?1",
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

    /// 排一个取内容的任务。已经在排或在跑的不动；失败过的重新开始。
    pub fn enqueue(&self, item_id: &str, url: &str, now: i64) -> Result<()> {
        self.conn().execute(
            "INSERT INTO fetch_jobs (item_id, url, state, attempts, due_at)
             VALUES (?1, ?2, 'queued', 0, ?3)
             ON CONFLICT (item_id) DO UPDATE SET
                url = excluded.url, state = 'queued', attempts = 0, due_at = excluded.due_at,
                error_code = NULL, error_hint = NULL
             WHERE fetch_jobs.state = 'failed'",
            params![item_id, url, now],
        )?;
        Ok(())
    }

    /// 取一个到点的任务，标成 `running`。
    pub fn claim_due(&self, now: i64) -> Result<Option<FetchJob>> {
        let conn = self.conn();
        let job = conn
            .query_row(
                "SELECT item_id, url, state, attempts, due_at, error_code, error_hint
                 FROM fetch_jobs WHERE state = 'queued' AND due_at <= ?1
                 ORDER BY due_at, item_id LIMIT 1",
                params![now],
                row_to_job,
            )
            .optional()?;
        if let Some(j) = &job {
            conn.execute(
                "UPDATE fetch_jobs SET state = 'running' WHERE item_id = ?1",
                params![j.item_id],
            )?;
        }
        Ok(job.map(|j| FetchJob {
            state: JobState::Running,
            ..j
        }))
    }

    /// 排着的任务里最早的到点时间。
    pub fn next_due(&self) -> Result<Option<i64>> {
        Ok(self.conn().query_row(
            "SELECT MIN(due_at) FROM fetch_jobs WHERE state = 'queued'",
            [],
            |r| r.get(0),
        )?)
    }

    pub fn job(&self, item_id: &str) -> Result<Option<FetchJob>> {
        Ok(self
            .conn()
            .query_row(
                "SELECT item_id, url, state, attempts, due_at, error_code, error_hint
                 FROM fetch_jobs WHERE item_id = ?1",
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

    /// 这个插件在这个范围上暂停到 `until`（Unix 毫秒）。
    pub fn pause(&self, plugin: &str, scope: &str, until: i64) -> Result<()> {
        self.conn().execute(
            "INSERT INTO pauses (plugin, platform, until_at) VALUES (?1, ?2, ?3)
             ON CONFLICT (plugin, platform) DO UPDATE SET until_at = excluded.until_at",
            params![plugin, scope, until],
        )?;
        Ok(())
    }

    /// 这个插件在这个范围上暂停到什么时候；到点的暂停顺手删掉。
    pub fn paused(&self, plugin: &str, scope: &str, now: i64) -> Result<Option<i64>> {
        let conn = self.conn();
        let row: Option<Option<i64>> = conn
            .query_row(
                "SELECT until_at FROM pauses WHERE plugin = ?1 AND platform = ?2",
                params![plugin, scope],
                |r| r.get(0),
            )
            .optional()?;
        match row {
            None => Ok(None),
            Some(Some(t)) if t > now => Ok(Some(t)),
            Some(_) => {
                conn.execute(
                    "DELETE FROM pauses WHERE plugin = ?1 AND platform = ?2",
                    params![plugin, scope],
                )?;
                Ok(None)
            }
        }
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

fn row_to_job(r: &rusqlite::Row<'_>) -> rusqlite::Result<FetchJob> {
    Ok(FetchJob {
        item_id: r.get(0)?,
        url: r.get(1)?,
        state: JobState::parse(&r.get::<_, String>(2)?),
        attempts: r.get(3)?,
        due_at: r.get(4)?,
        error_code: r.get(5)?,
        error_hint: r.get(6)?,
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
        assert_eq!(jobs.claim_due(5).unwrap(), None);
        let j = jobs.claim_due(10).unwrap().unwrap();
        assert_eq!(j.url, "https://a.test/", "排着的不被第二次覆盖");
        jobs.retry("web-a", 50, "network", None).unwrap();
        assert_eq!(jobs.next_due().unwrap(), Some(50));
        let j = jobs.claim_due(50).unwrap().unwrap();
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
    fn pauses_expire() {
        let dir = tempfile::tempdir().unwrap();
        let jobs = Jobs::open(&dir.path().join("jobs.db"), 0).unwrap();
        jobs.pause("a/b", "a.test", 100).unwrap();
        assert_eq!(jobs.paused("a/b", "a.test", 50).unwrap(), Some(100));
        assert_eq!(jobs.paused("a/b", "a.test", 100).unwrap(), None);
    }
}
