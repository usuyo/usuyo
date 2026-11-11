//! 改动提议文件 `.usuyo/changes/<id>.json`（SPEC §1.5.2）：这一层只读写，接受和应用归以后。

use std::path::PathBuf;

use usuyo_contract::Changeset;

use crate::fsx::{self, read_opt, write_json};
use crate::{Error, Library, Result};

impl Library {
    pub(crate) fn proposal_path(&self, id: &str) -> Result<PathBuf> {
        if !crate::id::is_uuid7(id) {
            return Err(Error::InvalidArgs(format!("bad changeset id `{id}`")));
        }
        Ok(self.path(".usuyo/changes").join(format!("{id}.json")))
    }

    /// 全部提议，按 id（即按时间）。
    pub fn proposals(&self) -> Result<Vec<Changeset>> {
        let dir = self.path(".usuyo/changes");
        let mut out = Vec::new();
        for f in fsx::list_files(&dir)? {
            if f.ends_with(".json") && !f.contains('/') {
                out.push(fsx::read_json::<Changeset>(&dir.join(&f))?);
            }
        }
        out.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(out)
    }

    pub fn proposal(&self, id: &str) -> Result<Changeset> {
        let path = self.proposal_path(id)?;
        let b = read_opt(&path)?.ok_or_else(|| Error::NotFound(format!("changeset {id}")))?;
        fsx::parse_json(&path, &b)
    }

    /// 存一个提议。只收能提议的 op（SPEC §1.5.1），`proposer` 要是 `agent:` 或 `model:`。
    pub fn save_proposal(&mut self, c: &Changeset) -> Result<()> {
        self.writable()?;
        if !(c.proposer.starts_with("agent:") || c.proposer.starts_with("model:")) {
            return Err(Error::InvalidArgs(format!("bad proposer `{}`", c.proposer)));
        }
        for op in &c.ops {
            if !op.typed()?.proposable() {
                return Err(Error::InvalidArgs(format!(
                    "`{}` cannot be proposed",
                    op.op
                )));
            }
        }
        write_json(&self.proposal_path(&c.id)?, c)
    }
}
