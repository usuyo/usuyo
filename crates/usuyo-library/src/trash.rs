//! 回收站（SPEC §1.7）：`.usuyo/trash/<trashId>/entry.json` 加文件；移除时清干净，不留删除记录。

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::{Value, json};
use usuyo_contract::{Changeset, Extra, ItemState, JournalEntry, Manifest, Redactable, TrashEntry};

use crate::fsx::{self, list_files, read_opt, write_json};
use crate::hash::hash_tag;
use crate::id::{ItemId, uuid7};
use crate::journal::{Batch, Step, target};
use crate::link::FORGOTTEN;
use crate::segments::all_segment_files;
use crate::writing::{DocKind, Titled, to_value};
use crate::{Error, Library, Result, time};

/// 移除以后调用方还要清的本机部分（索引、`sync.db` 底本、`backups/`、`exports/`、`imports/`）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PurgeReport {
    /// 被移除的对象和它的 `aliases`；空的表示只移除了本机文件、不用清干净。
    pub forgotten: Vec<String>,
    /// 链接换成 `usuyo://item/forgotten` 的文件。
    pub rewritten: Vec<String>,
}

/// 到期时间；`trash_days` 为 `None`（「一直」）不到期。
pub fn expires_at(trashed_at: &str, trash_days: Option<u32>) -> Result<Option<String>> {
    trash_days
        .map(|d| time::add_days(trashed_at, d))
        .transpose()
}

/// 只在本机、到期只删本机文件的回收站项。
fn local_only(e: &TrashEntry) -> bool {
    matches!(e.reason.as_str(), "keep-text" | "merge" | "conflict")
        || matches!(e.kind.as_str(), "notes" | "content")
}

impl Library {
    fn trash_path(&self, tid: &str) -> Result<PathBuf> {
        if !crate::id::is_uuid7(tid) {
            return Err(Error::InvalidArgs(format!("bad trash id `{tid}`")));
        }
        Ok(self.path(".usuyo/trash").join(tid))
    }

    /// 回收站里的全部项。
    pub fn trash_entries(&self) -> Result<Vec<TrashEntry>> {
        let mut out = Vec::new();
        for dir in crate::item::read_dirs(&self.path(".usuyo/trash"))? {
            let p = dir.join("entry.json");
            if let Some(b) = read_opt(&p)? {
                out.push(fsx::parse_json(&p, &b)?);
            }
        }
        out.sort_by(|a: &TrashEntry, b| (&a.trashed_at, &a.id).cmp(&(&b.trashed_at, &b.id)));
        Ok(out)
    }

    pub fn trash_entry(&self, tid: &str) -> Result<TrashEntry> {
        let p = self.trash_path(tid)?.join("entry.json");
        let b = read_opt(&p)?.ok_or_else(|| Error::NotFound(format!("trash {tid}")))?;
        fsx::parse_json(&p, &b)
    }

    /// 删掉或撤销收下进回收站的这一条。
    pub(crate) fn find_trashed_item(&self, id: &ItemId) -> Result<Option<TrashEntry>> {
        Ok(self.trash_entries()?.into_iter().find(|e| {
            e.kind == "item"
                && e.target == id.as_str()
                && matches!(e.reason.as_str(), "delete" | "undo")
        }))
    }

    /// 整条在回收站里的这一条：回收站项、它的 manifest 和它现在所在的文件夹。
    pub fn trashed_item(&self, id: &ItemId) -> Result<Option<(TrashEntry, Manifest, PathBuf)>> {
        match self.find_trashed_item(id)? {
            Some(e) => {
                let m = self.trashed_manifest(&e)?;
                let dir = self.trash_path(&e.id)?;
                Ok(Some((e, m, dir)))
            }
            None => Ok(None),
        }
    }

    pub(crate) fn trashed_manifest(&self, e: &TrashEntry) -> Result<Manifest> {
        fsx::read_json(&self.trash_path(&e.id)?.join("manifest.json"))
    }

    fn write_entry(&self, e: &TrashEntry) -> Result<()> {
        write_json(&self.trash_path(&e.id)?.join("entry.json"), e)
    }

    /// 到期的项（`now` 是 usuyo 写法的时间）。
    pub fn expired(&self, now: &str) -> Result<Vec<TrashEntry>> {
        let days = self.settings.trash_days;
        let mut out = Vec::new();
        for e in self.trash_entries()? {
            if let Some(at) = expires_at(&e.trashed_at, days)?
                && at.as_str() <= now
            {
                out.push(e);
            }
        }
        Ok(out)
    }

    /// 条目整个进回收站：manifest 记 `trashed`，文件夹改名进去。`reuse` 是撤销「拿回来」时原样放回的
    /// 那一项（同一个 `trashId`、原来的 `trashedAt`）。
    pub(crate) fn trash_item(
        &mut self,
        id: &ItemId,
        reason: &str,
        reuse: Option<(&str, &str)>,
    ) -> Result<Step> {
        let mut item = self.item(id)?;
        let dir = self.item_dir(id);
        let state_before = json!({"state": item.manifest.state, "stateAt": item.manifest.state_at});
        let (tid, at) = match reuse {
            Some((t, a)) => (t.to_owned(), a.to_owned()),
            None => (uuid7(), time::now()),
        };
        item.manifest.state = ItemState::Trashed;
        item.manifest.state_at = Some(at.clone());
        let (before, after) = self.save_manifest(&dir, &mut item.manifest)?;
        let files = list_files(&dir)?;
        let tdir = self.trash_path(&tid)?;
        if tdir.exists() {
            return Err(Error::Conflict(format!("trash {tid} already exists")));
        }
        fsx::move_path(&dir, &tdir)?;
        fsx::prune_empty_dirs(dir.parent().unwrap_or(&self.root), &self.path("items"));
        self.write_entry(&TrashEntry {
            id: tid.clone(),
            kind: "item".into(),
            target: id.to_string(),
            path: id.rel_dir(),
            files,
            reason: reason.into(),
            trashed_at: at,
            extra: Extra::new(),
        })?;
        Ok(Step {
            op: "move-to-trash",
            targets: vec![target(
                "item",
                id.as_str(),
                &format!("{}/manifest.json", id.rel_dir()),
                before,
                Some(after),
            )],
            change: json!({"trashId": tid, "reason": reason, "before": state_before}),
        })
    }

    /// 主题、卡片、成稿进回收站，存成 `body.md`。
    pub(crate) fn trash_doc<M: Titled>(
        &mut self,
        id: &str,
        reason: &str,
        reuse: Option<(&str, &str)>,
    ) -> Result<Step> {
        let w = self.find::<M>(id)?;
        let bytes = fsx::read(&self.path(&w.path))?;
        let (tid, at) = match reuse {
            Some((t, a)) => (t.to_owned(), a.to_owned()),
            None => (uuid7(), time::now()),
        };
        let tdir = self.trash_path(&tid)?;
        fsx::move_path(&self.path(&w.path), &tdir.join("body.md"))?;
        self.write_entry(&TrashEntry {
            id: tid.clone(),
            kind: M::KIND.name().into(),
            target: id.into(),
            path: w.path.clone(),
            files: vec!["body.md".into()],
            reason: reason.into(),
            trashed_at: at,
            extra: Extra::new(),
        })?;
        Ok(Step {
            op: "move-to-trash",
            targets: vec![target(
                M::KIND.name(),
                id,
                &w.path,
                Some(hash_tag(&bytes)),
                None,
            )],
            change: json!({"trashId": tid, "reason": reason}),
        })
    }

    /// 条目里的一个文件进回收站（`notes`、`content`），只在本机。
    pub(crate) fn trash_local_file(
        &mut self,
        id: &ItemId,
        file: &str,
        kind: &str,
    ) -> Result<TrashEntry> {
        let tid = uuid7();
        let tdir = self.trash_path(&tid)?;
        fsx::move_path(&self.item_dir(id).join(file), &tdir.join(file))?;
        let e = TrashEntry {
            id: tid,
            kind: kind.into(),
            target: id.to_string(),
            path: format!("{}/{file}", id.rel_dir()),
            files: vec![file.into()],
            reason: "conflict".into(),
            trashed_at: time::now(),
            extra: Extra::new(),
        };
        self.write_entry(&e)?;
        Ok(e)
    }

    /// 只留文字：`raw/` 里的图片、视频、音频和 `derived/keyframes/`、`derived/audio/` 进回收站。
    /// 缩略图 `derived/thumb.jpg` 归调用方先生成。
    pub(crate) fn keep_text(&mut self, id: &ItemId) -> Result<Step> {
        let mut item = self.item(id)?;
        let dir = self.item_dir(id);
        let mut moved = BTreeSet::new();
        for s in &item.manifest.snapshots {
            let path = dir.join("raw").join(&s.id).join("usuyo.fetch.json");
            if let Some(b) = read_opt(&path)? {
                let rec = crate::item::parse_fetch_record(&path, &b)?;
                let media = rec
                    .media
                    .iter()
                    .filter(|m| matches!(m.kind.as_str(), "video" | "audio"))
                    .map(|m| m.path.clone());
                for p in rec.images.iter().cloned().chain(media) {
                    let rel = format!("raw/{}/{p}", s.id);
                    if dir.join(&rel).is_file() {
                        moved.insert(rel);
                    }
                }
            }
        }
        for sub in ["derived/keyframes", "derived/audio"] {
            for f in list_files(&dir.join(sub))? {
                moved.insert(format!("{sub}/{f}"));
            }
        }
        let tid = uuid7();
        let tdir = self.trash_path(&tid)?;
        for rel in &moved {
            fsx::move_path(&dir.join(rel), &tdir.join(rel))?;
        }
        let at = time::now();
        let state_before = json!({"state": item.manifest.state, "stateAt": item.manifest.state_at});
        item.manifest.state = ItemState::TextOnly;
        item.manifest.state_at = Some(at.clone());
        item.manifest.text_only = Some(usuyo_contract::TextOnly {
            at: at.clone(),
            removed: moved.iter().cloned().collect(),
            extra: Extra::new(),
        });
        let (before, after) = self.save_manifest(&dir, &mut item.manifest)?;
        self.write_entry(&TrashEntry {
            id: tid.clone(),
            kind: "item".into(),
            target: id.to_string(),
            path: id.rel_dir(),
            files: moved.into_iter().collect(),
            reason: "keep-text".into(),
            trashed_at: at,
            extra: Extra::new(),
        })?;
        Ok(Step {
            op: "keep-text",
            targets: vec![target(
                "item",
                id.as_str(),
                &format!("{}/manifest.json", id.rel_dir()),
                before,
                Some(after),
            )],
            change: json!({"trashId": tid, "before": state_before}),
        })
    }

    /// 从回收站拿回来。`state` 给定时照它写回条目的 `state`、`stateAt`（撤销进回收站时原样）。
    pub(crate) fn restore_step(&mut self, tid: &str, state: Option<&Value>) -> Result<Step> {
        let e = self.trash_entry(tid)?;
        let tdir = self.trash_path(tid)?;
        let entry_value = to_value(&e)?;
        let set_state = |m: &mut Manifest| -> Result<Value> {
            let prev = json!({"state": m.state, "stateAt": m.state_at});
            match state {
                Some(s) => {
                    m.state = serde_json::from_value(s["state"].clone())
                        .map_err(|e| Error::InvalidArgs(e.to_string()))?;
                    m.state_at = s["stateAt"].as_str().map(str::to_owned);
                }
                None => {
                    m.state = if m.text_only.is_some() {
                        ItemState::TextOnly
                    } else {
                        ItemState::Normal
                    };
                    m.state_at = Some(time::now());
                }
            }
            Ok(prev)
        };
        let (targets, prev) = match (e.kind.as_str(), e.reason.as_str()) {
            ("item", "keep-text") => {
                let id = ItemId::parse(&e.target)?;
                let dir = self.item_dir(&id);
                let mut item = self.item(&id)?;
                for rel in &e.files {
                    fsx::move_path(&tdir.join(rel), &dir.join(rel))?;
                }
                let prev = set_state(&mut item.manifest)?;
                if state.is_none() {
                    item.manifest.state = ItemState::Normal;
                }
                item.manifest.text_only = None;
                let (b, a) = self.save_manifest(&dir, &mut item.manifest)?;
                (
                    vec![target(
                        "item",
                        id.as_str(),
                        &format!("{}/manifest.json", id.rel_dir()),
                        b,
                        Some(a),
                    )],
                    prev,
                )
            }
            ("item", _) => {
                let id = ItemId::parse(&e.target)?;
                let dir = self.item_dir(&id);
                if dir.exists() {
                    return Err(Error::Conflict(format!("{id} is already in the library")));
                }
                fsx::remove_file(&tdir.join("entry.json"))?;
                fsx::move_path(&tdir, &dir)?;
                let mut item = self.item(&id)?;
                let prev = if e.reason == "merge" {
                    Value::Null
                } else {
                    set_state(&mut item.manifest)?
                };
                let (b, a) = if e.reason == "merge" {
                    (None, hash_tag(&fsx::read(&dir.join("manifest.json"))?))
                } else {
                    self.save_manifest(&dir, &mut item.manifest)?
                };
                (
                    vec![target(
                        "item",
                        id.as_str(),
                        &format!("{}/manifest.json", id.rel_dir()),
                        b,
                        Some(a),
                    )],
                    prev,
                )
            }
            (k, _) if DocKind::parse(k).is_some() => {
                let kind = DocKind::parse(k).ok_or_else(|| Error::InvalidArgs(k.into()))?;
                let to = match kind {
                    DocKind::Topic => self.restore_doc_path::<usuyo_contract::TopicMeta>(&e)?,
                    DocKind::Card => self.restore_doc_path::<usuyo_contract::CardMeta>(&e)?,
                    DocKind::Draft => self.restore_doc_path::<usuyo_contract::DraftMeta>(&e)?,
                };
                let bytes = fsx::read(&tdir.join("body.md"))?;
                fsx::move_path(&tdir.join("body.md"), &self.path(&to))?;
                (
                    vec![target(k, &e.target, &to, None, Some(hash_tag(&bytes)))],
                    Value::Null,
                )
            }
            ("notes" | "content", _) => {
                let dest = self.path(&e.path);
                if dest.exists() {
                    return Err(Error::Conflict(format!("{} exists", e.path)));
                }
                let file = e
                    .files
                    .first()
                    .ok_or_else(|| Error::malformed(&tdir, "no files"))?;
                let bytes = fsx::read(&tdir.join(file))?;
                fsx::move_path(&tdir.join(file), &dest)?;
                (
                    vec![target(
                        "item",
                        &e.target,
                        &e.path,
                        None,
                        Some(hash_tag(&bytes)),
                    )],
                    Value::Null,
                )
            }
            (k, _) => return Err(Error::InvalidArgs(format!("unknown trash kind `{k}`"))),
        };
        fsx::remove_dir(&tdir)?;
        Ok(Step {
            op: "restore",
            targets,
            change: json!({"trashId": tid, "entry": entry_value, "before": prev}),
        })
    }

    fn restore_doc_path<M: Titled>(&self, e: &TrashEntry) -> Result<String> {
        let tdir = self.trash_path(&e.id)?;
        let doc =
            crate::doc::Doc::<M>::parse(&tdir.join("body.md"), &fsx::read(&tdir.join("body.md"))?)?;
        let sub = crate::writing::subdir_of(M::KIND, &e.path);
        self.place(&sub, &doc.meta, None)
    }

    /// 拿回来并记 `restore`。
    pub fn restore_entry(
        &mut self,
        batch: &Batch,
        tid: &str,
        state: Option<&Value>,
    ) -> Result<JournalEntry> {
        self.writable()?;
        let step = self.restore_step(tid, state)?;
        self.record(batch, step, None)
    }

    /// 立即清除或到期移除。对象整个离开资料库的（条目 `delete`、`undo`；主题、卡片、成稿，
    /// `conflict` 的除外）清干净：改动记录脱敏、提议里它的 op 删掉、发生过的事删行、
    /// 别的条目和你写的字里的引用换成 `usuyo://item/forgotten`。撤不了。
    pub fn purge(&mut self, batch: &Batch, tid: &str) -> Result<PurgeReport> {
        self.writable()?;
        let e = self.trash_entry(tid)?;
        let tdir = self.trash_path(tid)?;
        if local_only(&e) {
            fsx::remove_dir(&tdir)?;
            self.record(
                batch,
                Step {
                    op: "purge",
                    targets: vec![target(&e.kind, &e.target, &e.path, None, None)],
                    change: json!({"trashId": tid}),
                },
                None,
            )?;
            return Ok(PurgeReport::default());
        }
        let mut ids = vec![e.target.clone()];
        if e.kind == "item" {
            ids.extend(self.trashed_manifest(&e)?.aliases);
        }
        fsx::remove_dir(&tdir)?;
        let mut report = PurgeReport {
            forgotten: ids.clone(),
            rewritten: Vec::new(),
        };
        // 这一行自己也是脱敏的。
        let redacted = usuyo_contract::Target {
            kind: e.kind.clone(),
            id: Redactable::redacted(),
            path: Redactable::redacted(),
            before: None,
            after: None,
            extra: Extra::new(),
        };
        self.record(
            batch,
            Step {
                op: "purge",
                targets: vec![redacted],
                change: json!({"redacted": true}),
            },
            None,
        )?;
        self.redact_journal(&ids)?;
        self.drop_from_proposals(&ids)?;
        if e.kind == "item" {
            self.drop_facts(&ids)?;
            for other in self.trash_entries()? {
                if other.reason == "merge" && ids.contains(&other.target) {
                    fsx::remove_dir(&self.trash_path(&other.id)?)?;
                }
            }
            self.drop_item_refs(&ids)?;
            report.rewritten = self.forget_links(batch, &ids)?;
        }
        Ok(report)
    }

    fn redact_journal(&self, ids: &[String]) -> Result<()> {
        let mentions = |e: &JournalEntry| {
            e.targets.iter().any(|t| {
                t.id.value().is_some_and(|i| ids.contains(i))
                    || t.path
                        .value()
                        .is_some_and(|p| ids.iter().any(|i| p.contains(i.as_str())))
            }) || e
                .change
                .as_ref()
                .is_some_and(|c| ids.iter().any(|i| c.to_string().contains(i.as_str())))
        };
        for path in all_segment_files(&self.path(".usuyo/journal"))? {
            let text = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
            let mut changed = false;
            let mut out = String::with_capacity(text.len());
            for line in text.lines().filter(|l| !l.is_empty()) {
                let mut entry: JournalEntry =
                    serde_json::from_str(line).map_err(|e| Error::Json {
                        path: path.clone(),
                        source: e,
                    })?;
                if mentions(&entry) {
                    for t in &mut entry.targets {
                        t.id = Redactable::redacted();
                        t.path = Redactable::redacted();
                    }
                    entry.change = Some(json!({"redacted": true}));
                    changed = true;
                    out.push_str(&serde_json::to_string(&entry).map_err(|e| Error::Json {
                        path: path.clone(),
                        source: e,
                    })?);
                } else {
                    out.push_str(line);
                }
                out.push('\n');
            }
            if changed {
                fsx::atomic_write(&path, out.as_bytes())?;
            }
        }
        Ok(())
    }

    fn drop_from_proposals(&self, ids: &[String]) -> Result<()> {
        for p in self.proposals()? {
            let mut c: Changeset = p;
            let before = c.ops.len();
            c.ops.retain(|op| {
                let args = op.args.to_string();
                !ids.contains(&op.target.id) && !ids.iter().any(|i| args.contains(i.as_str()))
            });
            if c.ops.len() == before {
                continue;
            }
            let path = self.proposal_path(&c.id)?;
            if c.ops.is_empty() {
                fsx::remove_file(&path)?;
            } else {
                write_json(&path, &c)?;
            }
        }
        Ok(())
    }

    fn drop_facts(&self, ids: &[String]) -> Result<()> {
        for path in all_segment_files(&self.path(".usuyo/facts"))? {
            let text = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
            let mut out = String::with_capacity(text.len());
            let mut changed = false;
            for line in text.lines().filter(|l| !l.is_empty()) {
                let v = crate::segments::line_value(&path, line)?;
                if v["item"]
                    .as_str()
                    .is_some_and(|i| ids.iter().any(|x| x == i))
                {
                    changed = true;
                } else {
                    out.push_str(line);
                    out.push('\n');
                }
            }
            if changed {
                fsx::atomic_write(&path, out.as_bytes())?;
            }
        }
        Ok(())
    }

    /// 别的条目 manifest 的 `sameAs`、`notSameAs`、`aliases` 去掉它。
    fn drop_item_refs(&self, ids: &[String]) -> Result<()> {
        for other in self.item_ids()? {
            let mut item = self.item(&other)?;
            let m = &mut item.manifest;
            let n = m.same_as.len() + m.not_same_as.len() + m.aliases.len();
            for v in [&mut m.same_as, &mut m.not_same_as, &mut m.aliases] {
                v.retain(|x| !ids.contains(x));
            }
            if m.same_as.len() + m.not_same_as.len() + m.aliases.len() != n {
                self.save_manifest(&self.item_dir(&other), &mut item.manifest)?;
            }
        }
        Ok(())
    }

    /// 你写的字里指向它的链接换成 `usuyo://item/forgotten`，原话和标题留着。记 `actor: host`。
    fn forget_links(&mut self, batch: &Batch, ids: &[String]) -> Result<Vec<String>> {
        let mut files: Vec<String> = Vec::new();
        for kind in [DocKind::Topic, DocKind::Card, DocKind::Draft] {
            for rel in list_files(&self.path(kind.dir()))? {
                if rel.ends_with(".md") {
                    files.push(format!("{}/{rel}", kind.dir()));
                }
            }
        }
        for id in self.item_ids()? {
            for f in ["notes.md", "highlights.md"] {
                if self.item_dir(&id).join(f).exists() {
                    files.push(format!("{}/{f}", id.rel_dir()));
                }
            }
        }
        // 回收站里你写的字也算：拿回来时不该再指向它。
        for e in self.trash_entries()? {
            for f in &e.files {
                if f == "body.md" || f == "notes.md" || f == "highlights.md" {
                    files.push(format!(".usuyo/trash/{}/{f}", e.id));
                }
            }
        }
        let mut targets = Vec::new();
        for rel in files {
            let path = self.path(&rel);
            let bytes = fsx::read(&path)?;
            let Ok(text) = std::str::from_utf8(&bytes) else {
                continue; // 不是 UTF-8 的不是 usuyo 写的，不去碰
            };
            let new = replace_item_links(text, ids);
            if new != text {
                fsx::atomic_write(&path, new.as_bytes())?;
                let kind = rel
                    .split('/')
                    .next()
                    .unwrap_or_default()
                    .trim_end_matches('s');
                targets.push(target(
                    kind,
                    "",
                    &rel,
                    Some(hash_tag(&bytes)),
                    Some(hash_tag(new.as_bytes())),
                ));
            }
        }
        let rewritten: Vec<String> = targets
            .iter()
            .filter_map(|t| t.path.value().cloned())
            .collect();
        if !targets.is_empty() {
            let host = Batch {
                id: batch.id.clone(),
                actor: "host".into(),
                via: batch.via.clone(),
                change_set: None,
            };
            self.record(
                &host,
                Step {
                    op: "purge",
                    targets,
                    change: json!({"redacted": true}),
                },
                None,
            )?;
        }
        Ok(rewritten)
    }
}

/// `usuyo://item/<id>…` 整个链接换成 `usuyo://item/forgotten`。
pub(crate) fn replace_item_links(text: &str, ids: &[String]) -> String {
    const PREFIX: &str = "usuyo://item/";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(PREFIX) {
        out.push_str(&rest[..i]);
        let after = &rest[i + PREFIX.len()..];
        let end = after
            .find(|c: char| c == ')' || c == '>' || c.is_whitespace())
            .unwrap_or(after.len());
        let link = &after[..end];
        let id_end = link.find(['/', '#', '@']).unwrap_or(link.len());
        if ids.iter().any(|x| x == &link[..id_end]) {
            out.push_str(PREFIX);
            out.push_str(FORGOTTEN);
        } else {
            out.push_str(PREFIX);
            out.push_str(link);
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_rewrite_keeps_quote_and_title() {
        let text = "[^k3f7q]: 「原话」[标题 · 口播 00:42](usuyo://item/douyin-4plrcxk343onmyxibmrr672kga/asr@s#t=42) · 作者\n还有 usuyo://item/web-bbbbbbbbbbbbbbbbbbbbbbbbbb 不动\n";
        let out = replace_item_links(text, &["douyin-4plrcxk343onmyxibmrr672kga".into()]);
        assert_eq!(
            out,
            "[^k3f7q]: 「原话」[标题 · 口播 00:42](usuyo://item/forgotten) · 作者\n还有 usuyo://item/web-bbbbbbbbbbbbbbbbbbbbbbbbbb 不动\n"
        );
    }

    #[test]
    fn expiry() {
        assert_eq!(
            expires_at("2026-11-05T03:00:01.123Z", Some(30))
                .unwrap()
                .as_deref(),
            Some("2026-12-05T03:00:01.123Z")
        );
        assert_eq!(expires_at("2026-11-05T03:00:01.123Z", None).unwrap(), None);
    }
}
