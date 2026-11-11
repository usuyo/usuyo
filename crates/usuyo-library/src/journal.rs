//! 改动记录（SPEC §1.6）：一次动作一批，每条 op 一行。

use serde_json::Value;
use usuyo_contract::{Extra, JournalEntry, Redactable, Target};

use crate::segments::{all_segment_files, line_value};
use crate::{Error, Library, Result, id::uuid7, time};

/// 一次动作（接受一次、读回一次、一次 MCP 调用）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch {
    pub id: String,
    /// `user` `external` `agent:<name>` `model:<provider>` `plugin:<id>` `subscription:<id>` `sync` `host`
    pub actor: String,
    /// `app` `cli` `mcp` `api` `extension` `mobile` `import` `subscription` `file` `readback:<插件 id>`
    pub via: String,
    /// 来自哪个提议的哪一项。
    pub change_set: Option<(String, String)>,
}

impl Batch {
    pub fn new(actor: &str, via: &str) -> Result<Batch> {
        if !valid_actor(actor) {
            return Err(Error::InvalidArgs(format!("bad actor `{actor}`")));
        }
        if !valid_via(via) {
            return Err(Error::InvalidArgs(format!("bad via `{via}`")));
        }
        Ok(Batch {
            id: uuid7(),
            actor: actor.to_owned(),
            via: via.to_owned(),
            change_set: None,
        })
    }
}

fn valid_actor(a: &str) -> bool {
    match a.split_once(':') {
        None => matches!(a, "user" | "external" | "sync" | "host"),
        Some(("agent", n)) => {
            !n.is_empty()
                && n.bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        }
        Some(("model" | "plugin", n)) => !n.is_empty(),
        Some(("subscription", n)) => crate::id::is_uuid7(n),
        Some(_) => false,
    }
}

fn valid_via(v: &str) -> bool {
    matches!(
        v,
        "app" | "cli" | "mcp" | "api" | "extension" | "mobile" | "import" | "subscription" | "file"
    ) || v.strip_prefix("readback:").is_some_and(|p| !p.is_empty())
}

/// 改了哪个文件。
pub(crate) fn target(
    kind: &str,
    id: &str,
    path: &str,
    before: Option<String>,
    after: Option<String>,
) -> Target {
    Target {
        kind: kind.to_owned(),
        id: Redactable::Value(id.to_owned()),
        path: Redactable::Value(path.to_owned()),
        before,
        after,
        extra: Extra::new(),
    }
}

/// 一步改动：哪几个文件、反着做要的内容。
#[derive(Debug, Clone)]
pub(crate) struct Step {
    pub op: &'static str,
    pub targets: Vec<Target>,
    pub change: Value,
}

impl Library {
    /// 记一行。
    pub(crate) fn record(
        &mut self,
        batch: &Batch,
        step: Step,
        undoes: Option<&str>,
    ) -> Result<JournalEntry> {
        let device = self.device_id.clone();
        let entry_id = uuid7();
        let value = self.journal.append(|n| {
            let entry = JournalEntry {
                id: entry_id.clone(),
                n,
                t: time::now(),
                device: device.clone(),
                actor: batch.actor.clone(),
                via: Some(batch.via.clone()),
                op: step.op.to_owned(),
                targets: step.targets.clone(),
                change: Some(step.change.clone()),
                change_set: batch.change_set.as_ref().map(|c| c.0.clone()),
                change_op: batch.change_set.as_ref().map(|c| c.1.clone()),
                batch: batch.id.clone(),
                undoes: undoes.map(str::to_owned),
                extra: Extra::new(),
            };
            serde_json::to_value(entry).map_err(|e| Error::InvalidArgs(e.to_string()))
        })?;
        serde_json::from_value(value).map_err(|e| Error::Json {
            path: self.root.join(".usuyo/journal"),
            source: e,
        })
    }

    /// 全部设备的全部改动记录，按设备、段、行的先后。
    pub fn journal_entries(&self) -> Result<Vec<JournalEntry>> {
        let mut out = Vec::new();
        for path in all_segment_files(&self.root.join(".usuyo/journal"))? {
            let text = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
            for line in text.lines().filter(|l| !l.is_empty()) {
                let v = line_value(&path, line)?;
                out.push(serde_json::from_value(v).map_err(|e| Error::Json {
                    path: path.clone(),
                    source: e,
                })?);
            }
        }
        Ok(out)
    }

    /// 一批里的各条，按 `n`。
    pub fn batch_entries(&self, batch_id: &str) -> Result<Vec<JournalEntry>> {
        let mut v: Vec<JournalEntry> = self
            .journal_entries()?
            .into_iter()
            .filter(|e| e.batch == batch_id)
            .collect();
        v.sort_by(|a, b| (&a.device, a.n).cmp(&(&b.device, b.n)));
        Ok(v)
    }

    /// 记一次「打开过」（SPEC §1.4.4）。
    pub fn record_opened(&mut self, item: &crate::ItemId) -> Result<()> {
        self.writable()?;
        let item = item.as_str().to_owned();
        self.facts.append(|n| {
            Ok(serde_json::json!({"n": n, "t": time::now(), "ev": "opened", "item": item}))
        })?;
        Ok(())
    }
}
