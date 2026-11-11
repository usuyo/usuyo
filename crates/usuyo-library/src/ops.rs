//! op 的应用和撤销（SPEC §1.5.1、§1.6）。每条 op 改完记一行；撤销写成新的一批，每条带 `undoes`。

use std::collections::HashSet;

use serde_json::{Value, json};
use usuyo_contract::{
    CardMeta, DraftMeta, Extra, Hunk, JournalEntry, Manifest, NotesMeta, Op, OpTarget, SetChange,
    TopicMeta,
};

use crate::citation::{Citation, labels};
use crate::doc::{Doc, Paragraphs, Splice, apply_hunks, revert_splices};
use crate::fsx::{self, read_opt};
use crate::hash::hash_tag;
use crate::highlights::{self, Highlight};
use crate::id::{ItemId, block_id, is_block_id, uuid7};
use crate::journal::{Batch, Step, target};
use crate::link::{Link, column_label, layer_needs_snapshot};
use crate::trash::PurgeReport;
use crate::writing::{DocKind, Titled, check_subdir, subdir_of, to_value};
use crate::{Error, Library, Result, time};

/// 做了一条 op。
#[derive(Debug, Clone)]
pub struct Applied {
    pub entry: JournalEntry,
    /// 目标的 id；新建主题、卡片、成稿时是新 id。
    pub id: String,
    /// `purge` 的结果，调用方据此清本机索引等。
    pub purged: Option<PurgeReport>,
}

/// 撤销一批的结果。
#[derive(Debug, Clone, Default)]
pub struct UndoReport {
    pub batch: String,
    pub undone: Vec<String>,
    /// （记录 id，为什么没撤）：「之后改过」「已经撤过」「撤不了」。
    pub skipped: Vec<(String, String)>,
}

/// 集合增减，回实际加了、去了的。
fn apply_set(list: &mut Vec<String>, change: &SetChange) -> SetChange {
    let mut done = SetChange::default();
    for a in &change.add {
        if !list.contains(a) {
            list.push(a.clone());
            done.add.push(a.clone());
        }
    }
    for r in &change.remove {
        if let Some(i) = list.iter().position(|x| x == r) {
            list.remove(i);
            done.remove.push(r.clone());
        }
    }
    list.sort();
    done
}

fn invert_set(change: &Value) -> Result<SetChange> {
    let c: SetChange =
        serde_json::from_value(change.clone()).map_err(|e| Error::InvalidArgs(e.to_string()))?;
    Ok(SetChange {
        add: c.remove,
        remove: c.add,
    })
}

#[derive(Clone, Copy)]
enum ManifestSet {
    UserTags,
    Topics,
    NotSame,
    Dismissed,
}

impl ManifestSet {
    fn list(self, m: &mut Manifest) -> &mut Vec<String> {
        match self {
            ManifestSet::UserTags => &mut m.tags.user,
            ManifestSet::Topics => &mut m.topics,
            ManifestSet::NotSame => &mut m.not_same_as,
            ManifestSet::Dismissed => &mut m.tags.dismissed,
        }
    }

    fn op(self) -> &'static str {
        match self {
            ManifestSet::UserTags => "set-tags",
            ManifestSet::Topics => "set-topics",
            ManifestSet::NotSame => "set-not-same",
            ManifestSet::Dismissed => "dismiss-suggestion",
        }
    }

    fn from_op(op: &str) -> Option<ManifestSet> {
        match op {
            "set-tags" => Some(ManifestSet::UserTags),
            "set-topics" => Some(ManifestSet::Topics),
            "set-not-same" => Some(ManifestSet::NotSame),
            "dismiss-suggestion" => Some(ManifestSet::Dismissed),
            _ => None,
        }
    }
}

type Skip = String;

impl Library {
    /// 做一条 op、记一行。`add-item` 走 [`Library::add_item`]。
    pub fn apply(&mut self, batch: &Batch, target: &OpTarget, op: Op) -> Result<Applied> {
        self.writable()?;
        if let Op::Purge {} = op {
            let report = self.purge(batch, &target.id)?;
            let entry = self
                .journal_entries()?
                .into_iter()
                .rev()
                .find(|e| e.batch == batch.id && e.op == "purge")
                .ok_or_else(|| Error::NotFound("purge entry".into()))?;
            return Ok(Applied {
                entry,
                id: target.id.clone(),
                purged: Some(report),
            });
        }
        let (id, step) = self.run(target, op)?;
        let entry = self.record(batch, step, None)?;
        Ok(Applied {
            entry,
            id,
            purged: None,
        })
    }

    fn run(&mut self, t: &OpTarget, op: Op) -> Result<(String, Step)> {
        let kind = t.kind.as_str();
        let item = || ItemId::parse(&t.id);
        let step = match op {
            Op::AddItem { .. } => {
                return Err(Error::InvalidArgs(
                    "add-item goes through Library::add_item".into(),
                ));
            }
            Op::Purge {} => {
                return Err(Error::InvalidArgs(
                    "purge goes through Library::purge".into(),
                ));
            }
            Op::SetTags(c) => self.set_manifest(&item()?, ManifestSet::UserTags, &c)?,
            Op::SetNotSame(c) => self.set_manifest(&item()?, ManifestSet::NotSame, &c)?,
            Op::DismissSuggestion { tag } => self.set_manifest(
                &item()?,
                ManifestSet::Dismissed,
                &SetChange {
                    add: vec![tag],
                    remove: vec![],
                },
            )?,
            Op::SetTopics(c) if kind == "card" => {
                self.edit_written::<CardMeta>(&t.id, "set-topics", |d| {
                    to_value(&apply_set(&mut d.meta.topics, &c))
                })?
            }
            Op::SetTopics(c) => self.set_manifest(&item()?, ManifestSet::Topics, &c)?,
            Op::SetPrivate { value } => self.edit_manifest(&item()?, "set-private", |m| {
                let before = m.private;
                m.private = value;
                Ok(json!({"before": before, "after": value}))
            })?,
            Op::SetFollowed { value } => {
                self.edit_written::<TopicMeta>(&t.id, "set-followed", |d| {
                    let before = d.meta.followed;
                    d.meta.followed = value;
                    Ok(json!({"before": before, "after": value}))
                })?
            }
            Op::SetSetting { key, value } => self.set_setting(&key, Some(value))?,
            Op::SetTakeaway { text } => self.edit_notes(&item()?, "set-takeaway", |d| {
                let before = d.meta.takeaway.clone();
                d.meta.takeaway = text.clone().filter(|s| !s.is_empty());
                Ok(json!({"before": before, "after": d.meta.takeaway}))
            })?,
            Op::EditNotes { hunks } => {
                self.edit_notes(&item()?, "edit-notes", |d| edit_body(&mut d.body, &hunks))?
            }
            Op::AddHighlight {
                id,
                layer,
                snapshot,
                fragment,
                quote,
            } => {
                let item = item()?;
                let primary = self.item(&item)?.manifest.primary_snapshot;
                let link = cite_link(&item, primary.as_deref(), layer, snapshot, fragment)?;
                if !is_block_id(&id) {
                    return Err(Error::InvalidArgs(format!("bad block id `{id}`")));
                }
                let h = Highlight {
                    label: column_label(
                        link.layer.as_deref().unwrap_or("text"),
                        link.fragment.as_deref(),
                    ),
                    quote,
                    link,
                    id,
                };
                self.edit_highlights(&item, "add-highlight", |list| {
                    if list.iter().any(|x| x.id == h.id) {
                        return Err(Error::Conflict(format!("highlight {} exists", h.id)));
                    }
                    let line = h.render();
                    list.push(h);
                    Ok(json!({"line": line}))
                })?
            }
            Op::RemoveHighlight { id } => {
                self.edit_highlights(&item()?, "remove-highlight", |list| {
                    let i = list
                        .iter()
                        .position(|x| x.id == id)
                        .ok_or_else(|| Error::NotFound(format!("highlight {id}")))?;
                    let h = list.remove(i);
                    Ok(json!({"line": h.render(), "index": i}))
                })?
            }
            Op::CreateTopic {
                title,
                keywords,
                match_,
            } => {
                let meta = TopicMeta {
                    id: new_id(&t.id)?,
                    title: non_empty(title)?,
                    created_at: time::now(),
                    followed: false,
                    keywords,
                    match_,
                    extra: Extra::new(),
                };
                self.create_doc(meta, String::new(), "create-topic")?
            }
            Op::CreateCard { title, body } => {
                let title = if title.trim().is_empty() {
                    first_words(&body)
                } else {
                    title
                };
                let meta = CardMeta {
                    id: new_id(&t.id)?,
                    title: non_empty(title)?,
                    created_at: time::now(),
                    topics: Vec::new(),
                    extra: Extra::new(),
                };
                self.create_doc(meta, body, "create-card")?
            }
            Op::CreateDraft { title, body, kind } => {
                let meta = DraftMeta {
                    id: new_id(&t.id)?,
                    title: non_empty(title)?,
                    created_at: time::now(),
                    kind,
                    from: Vec::new(),
                    extra: Extra::new(),
                };
                self.create_doc(meta, body, "create-draft")?
            }
            Op::EditTopic {
                title,
                keywords,
                match_,
            } => self.edit_written::<TopicMeta>(&t.id, "edit-topic", |d| {
                let mut c = serde_json::Map::new();
                if let Some(title) = title {
                    c.insert(
                        "title".into(),
                        json!({"before": d.meta.title, "after": non_empty(title.clone())?}),
                    );
                    d.meta.title = title;
                }
                if let Some(k) = keywords {
                    c.insert(
                        "keywords".into(),
                        to_value(&replace_set(&mut d.meta.keywords, k))?,
                    );
                }
                if let Some(m) = match_ {
                    c.insert(
                        "match".into(),
                        to_value(&replace_set(&mut d.meta.match_, m))?,
                    );
                }
                Ok(Value::Object(c))
            })?,
            Op::EditTopicPage { hunks } => {
                self.edit_written::<TopicMeta>(&t.id, "edit-topic-page", |d| {
                    edit_body(&mut d.body, &hunks)
                })?
            }
            Op::EditCard { hunks, title } => {
                self.edit_written::<CardMeta>(&t.id, "edit-card", |d| {
                    let mut c = edit_body(&mut d.body, &hunks)?;
                    if let Some(title) = title {
                        c["title"] =
                            json!({"before": d.meta.title, "after": non_empty(title.clone())?});
                        d.meta.title = title;
                    }
                    Ok(c)
                })?
            }
            Op::EditDraft {
                hunks,
                title,
                kind: dkind,
                from,
            } => self.edit_written::<DraftMeta>(&t.id, "edit-draft", |d| {
                let mut c = edit_body(&mut d.body, &hunks)?;
                if let Some(title) = title {
                    c["title"] =
                        json!({"before": d.meta.title, "after": non_empty(title.clone())?});
                    d.meta.title = title;
                }
                if let Some(k) = dkind {
                    c["kind"] = json!({"before": d.meta.kind, "after": k});
                    d.meta.kind = k;
                }
                c["from"] = to_value(&apply_set(&mut d.meta.from, &from))?;
                Ok(c)
            })?,
            Op::AddCitation {
                anchor,
                item,
                layer,
                snapshot,
                fragment,
                quote,
            } => {
                let source = ItemId::parse(&item)?;
                let m = self.item(&source)?.manifest;
                let link = cite_link(
                    &source,
                    m.primary_snapshot.as_deref(),
                    layer,
                    snapshot,
                    fragment,
                )?;
                let cit = Citation {
                    label: String::new(),
                    column: column_label(
                        link.layer.as_deref().unwrap_or("text"),
                        link.fragment.as_deref(),
                    ),
                    link,
                    quote,
                    title: m.title.clone(),
                    author: m.author.as_ref().map(|a| a.name.clone()),
                    date: time::display_date(
                        m.published_at_original.as_deref(),
                        m.published_at.as_deref(),
                    ),
                };
                match DocKind::parse(kind) {
                    Some(DocKind::Topic) => {
                        self.edit_written::<TopicMeta>(&t.id, "add-citation", |d| {
                            insert_citation(&mut d.body, cit, anchor.as_deref())
                        })?
                    }
                    Some(DocKind::Card) => {
                        self.edit_written::<CardMeta>(&t.id, "add-citation", |d| {
                            insert_citation(&mut d.body, cit, anchor.as_deref())
                        })?
                    }
                    Some(DocKind::Draft) => {
                        self.edit_written::<DraftMeta>(&t.id, "add-citation", |d| {
                            insert_citation(&mut d.body, cit, anchor.as_deref())
                        })?
                    }
                    None => return Err(Error::InvalidArgs(format!("cannot cite into `{kind}`"))),
                }
            }
            Op::Move { path } => {
                check_subdir(&path)?;
                match DocKind::parse(kind) {
                    Some(DocKind::Topic) => self.move_doc::<TopicMeta>(&t.id, &path)?,
                    Some(DocKind::Card) => self.move_doc::<CardMeta>(&t.id, &path)?,
                    Some(DocKind::Draft) => self.move_doc::<DraftMeta>(&t.id, &path)?,
                    None => return Err(Error::InvalidArgs(format!("cannot move `{kind}`"))),
                }
            }
            Op::MoveToTrash { reason } => match kind {
                "item" => self.trash_item(&item()?, &reason, None)?,
                "topic" => self.trash_doc::<TopicMeta>(&t.id, &reason, None)?,
                "card" => self.trash_doc::<CardMeta>(&t.id, &reason, None)?,
                "draft" => self.trash_doc::<DraftMeta>(&t.id, &reason, None)?,
                _ => return Err(Error::InvalidArgs(format!("cannot trash `{kind}`"))),
            },
            Op::KeepText {} => self.keep_text(&item()?)?,
            Op::Restore {} => self.restore_step(&t.id, None)?,
            Op::MergeItems { into } => self.merge_items(&item()?, &ItemId::parse(&into)?)?,
        };
        Ok((
            step.targets
                .first()
                .and_then(|x| x.id.value().cloned())
                .unwrap_or_else(|| t.id.clone()),
            step,
        ))
    }

    fn edit_manifest(
        &mut self,
        id: &ItemId,
        op: &'static str,
        f: impl FnOnce(&mut Manifest) -> Result<Value>,
    ) -> Result<Step> {
        let mut item = self.item(id)?;
        let change = f(&mut item.manifest)?;
        let (before, after) = self.save_manifest(&self.item_dir(id), &mut item.manifest)?;
        Ok(Step {
            op,
            targets: vec![target(
                "item",
                id.as_str(),
                &format!("{}/manifest.json", id.rel_dir()),
                before,
                Some(after),
            )],
            change,
        })
    }

    fn set_manifest(&mut self, id: &ItemId, which: ManifestSet, c: &SetChange) -> Result<Step> {
        self.edit_manifest(id, which.op(), |m| to_value(&apply_set(which.list(m), c)))
    }

    fn set_setting(&mut self, key: &str, value: Option<Value>) -> Result<Step> {
        let path = self.path(".usuyo/settings.json");
        let raw = fsx::read(&path)?;
        let mut v: serde_json::Map<String, Value> = fsx::parse_json(&path, &raw)?;
        let before = v.get(key).cloned();
        match &value {
            Some(x) => v.insert(key.to_owned(), x.clone()),
            None => v.remove(key),
        };
        let settings = serde_json::from_value(Value::Object(v))
            .map_err(|e| Error::InvalidArgs(e.to_string()))?;
        self.save_settings(settings)?;
        let after = fsx::read(&path)?;
        Ok(Step {
            op: "set-setting",
            targets: vec![target(
                "library",
                "settings",
                ".usuyo/settings.json",
                Some(hash_tag(&raw)),
                Some(hash_tag(&after)),
            )],
            change: json!({
                "key": key,
                "before": before,
                "existed": before.is_some(),
                "after": value,
            }),
        })
    }

    fn edit_notes(
        &mut self,
        id: &ItemId,
        op: &'static str,
        f: impl FnOnce(&mut Doc<NotesMeta>) -> Result<Value>,
    ) -> Result<Step> {
        self.item(id)?;
        let path = self.item_dir(id).join("notes.md");
        let raw = read_opt(&path)?;
        let (mut doc, existed) = self.notes(id)?;
        let mut change = f(&mut doc)?;
        change["created"] = Value::Bool(!existed);
        let bytes = doc.render()?;
        fsx::atomic_write(&path, &bytes)?;
        Ok(Step {
            op,
            targets: vec![target(
                "item",
                id.as_str(),
                &format!("{}/notes.md", id.rel_dir()),
                raw.map(|b| hash_tag(&b)),
                Some(hash_tag(&bytes)),
            )],
            change,
        })
    }

    fn edit_highlights(
        &mut self,
        id: &ItemId,
        op: &'static str,
        f: impl FnOnce(&mut Vec<Highlight>) -> Result<Value>,
    ) -> Result<Step> {
        self.item(id)?;
        let path = self.item_dir(id).join("highlights.md");
        let raw = read_opt(&path)?;
        let mut list = self.highlights(id)?;
        let mut change = f(&mut list)?;
        change["created"] = Value::Bool(raw.is_none());
        let bytes = highlights::render_file(&list);
        fsx::atomic_write(&path, bytes.as_bytes())?;
        Ok(Step {
            op,
            targets: vec![target(
                "item",
                id.as_str(),
                &format!("{}/highlights.md", id.rel_dir()),
                raw.map(|b| hash_tag(&b)),
                Some(hash_tag(bytes.as_bytes())),
            )],
            change,
        })
    }

    fn create_doc<M: Titled>(&mut self, meta: M, body: String, op: &'static str) -> Result<Step> {
        if self.find::<M>(meta.id()).is_ok() {
            return Err(Error::Conflict(format!(
                "{} {} exists",
                M::KIND.name(),
                meta.id()
            )));
        }
        let doc = Doc::new(meta, body);
        let to = self.place("", &doc.meta, None)?;
        let (_, after) = self.write_doc(None, &to, &doc)?;
        Ok(Step {
            op,
            targets: vec![target(
                M::KIND.name(),
                doc.meta.id(),
                &to,
                None,
                Some(after),
            )],
            change: json!({"path": to}),
        })
    }

    /// 改一份主题、卡片或成稿；标题变了文件名跟着改（同一个子文件夹）。
    fn edit_written<M: Titled>(
        &mut self,
        id: &str,
        op: &'static str,
        f: impl FnOnce(&mut Doc<M>) -> Result<Value>,
    ) -> Result<Step> {
        let mut w = self.find::<M>(id)?;
        let mut change = f(&mut w.doc)?;
        let sub = subdir_of(M::KIND, &w.path);
        let expected = format!("{}.md", crate::text::sanitize(w.doc.meta.title()));
        let current_name = w.path.rsplit('/').next().unwrap_or_default();
        let to = if current_name == expected {
            w.path.clone()
        } else {
            self.place(&sub, &w.doc.meta, Some(&w.path))?
        };
        let (before, after) = self.write_doc(Some(&w.path), &to, &w.doc)?;
        if to != w.path {
            change["moved"] = json!({"from": w.path, "to": to});
        }
        Ok(Step {
            op,
            targets: vec![target(M::KIND.name(), id, &to, before, Some(after))],
            change,
        })
    }

    fn move_doc<M: Titled>(&mut self, id: &str, sub: &str) -> Result<Step> {
        let w = self.find::<M>(id)?;
        let to = self.place(sub, &w.doc.meta, Some(&w.path))?;
        let (before, after) = self.write_doc(Some(&w.path), &to, &w.doc)?;
        Ok(Step {
            op: "move",
            targets: vec![target(M::KIND.name(), id, &to, before, Some(after))],
            change: json!({"from": w.path, "to": to}),
        })
    }

    /// A 并进 B（SPEC §1.5.1 的 `merge-items`）。B 改之前的 manifest、笔记、划线存进 `change`，撤销照它原样换回。
    fn merge_items(&mut self, a: &ItemId, b: &ItemId) -> Result<Step> {
        if a == b {
            return Err(Error::InvalidArgs(
                "cannot merge an item into itself".into(),
            ));
        }
        let am = self.item(a)?.manifest;
        let mut bm = self.item(b)?.manifest;
        let (adir, bdir) = (self.item_dir(a), self.item_dir(b));
        let expected = self.content_hash(b)?;
        let read_text = |p: &std::path::Path| -> Result<Option<String>> {
            Ok(match read_opt(p)? {
                Some(bytes) => Some(fsx::decode(p, &bytes)?.0),
                None => None,
            })
        };
        let before = json!({
            "manifest": read_text(&bdir.join("manifest.json"))?,
            "notes": read_text(&bdir.join("notes.md"))?,
            "highlights": read_text(&bdir.join("highlights.md"))?,
        });
        // 先读好 A 的笔记和划线，B 的写好以后再动文件夹。
        let (a_notes, _) = self.notes(a)?;
        let a_lights = self.highlights(a)?;
        let (mut b_notes, _) = self.notes(b)?;
        let mut b_lights = self.highlights(b)?;
        let mut moved = Vec::new();
        for s in &am.snapshots {
            let from = adir.join("raw").join(&s.id);
            if from.exists() {
                fsx::move_path(&from, &bdir.join("raw").join(&s.id))?;
                moved.push(s.id.clone());
            }
        }
        bm.snapshots.extend(am.snapshots.iter().cloned());
        for i in &am.identities {
            if !bm.identities.iter().any(|x| x.key() == i.key()) {
                bm.identities.push(i.clone());
            }
        }
        bm.origins.extend(am.origins.iter().cloned());
        for (dst, src) in [
            (&mut bm.topics, &am.topics),
            (&mut bm.tags.user, &am.tags.user),
            (&mut bm.tags.dismissed, &am.tags.dismissed),
            (&mut bm.aliases, &am.aliases),
        ] {
            dst.extend(src.iter().cloned());
        }
        bm.aliases.push(a.to_string());
        if bm.added_at > am.added_at {
            bm.added_at = am.added_at.clone();
        }
        self.save_manifest(&bdir, &mut bm)?;
        let notes_changed = !a_notes.body.is_empty()
            || (b_notes.meta.takeaway.is_none() && a_notes.meta.takeaway.is_some());
        if notes_changed {
            if !a_notes.body.is_empty() {
                let mut p = Paragraphs::split(&b_notes.body);
                p.items.extend(Paragraphs::split(&a_notes.body).items);
                b_notes.body = p.join();
            }
            if b_notes.meta.takeaway.is_none() {
                b_notes.meta.takeaway = a_notes.meta.takeaway.clone();
            }
            fsx::atomic_write(&bdir.join("notes.md"), &b_notes.render()?)?;
        }
        if !a_lights.is_empty() {
            for mut h in a_lights {
                while b_lights.iter().any(|x| x.id == h.id) {
                    h.id = block_id()?;
                }
                h.link.id = b.to_string();
                b_lights.push(h);
            }
            fsx::atomic_write(
                &bdir.join("highlights.md"),
                highlights::render_file(&b_lights).as_bytes(),
            )?;
        }
        self.write_content(b, Some(&expected))?;
        // A 剩下的（manifest、content.md、derived/、笔记、划线）进回收站。
        let tid = uuid7();
        let files = fsx::list_files(&adir)?;
        let tdir = self.path(".usuyo/trash").join(&tid);
        fsx::move_path(&adir, &tdir)?;
        fsx::write_json(
            &tdir.join("entry.json"),
            &usuyo_contract::TrashEntry {
                id: tid.clone(),
                kind: "item".into(),
                target: a.to_string(),
                path: a.rel_dir(),
                files,
                reason: "merge".into(),
                trashed_at: time::now(),
                extra: Extra::new(),
            },
        )?;
        let bm_after = hash_tag(&fsx::read(&bdir.join("manifest.json"))?);
        Ok(Step {
            op: "merge-items",
            targets: vec![
                target(
                    "item",
                    b.as_str(),
                    &format!("{}/manifest.json", b.rel_dir()),
                    None,
                    Some(bm_after),
                ),
                target("item", a.as_str(), &a.rel_dir(), None, None),
            ],
            change: json!({"from": a.as_str(), "into": b.as_str(), "snapshots": moved, "trashId": tid, "before": before}),
        })
    }

    /// 整批撤销：从新到旧反着做，写成新的一批 `by`，每条带 `undoes`。
    pub fn undo(&mut self, batch_id: &str, by: &Batch) -> Result<UndoReport> {
        self.writable()?;
        let all = self.journal_entries()?;
        let undone: HashSet<String> = all.iter().filter_map(|e| e.undoes.clone()).collect();
        let mut entries: Vec<JournalEntry> =
            all.into_iter().filter(|e| e.batch == batch_id).collect();
        if entries.is_empty() {
            return Err(Error::NotFound(format!("batch {batch_id}")));
        }
        entries.sort_by(|a, b| (&b.device, b.n).cmp(&(&a.device, a.n)));
        let mut report = UndoReport {
            batch: by.id.clone(),
            ..UndoReport::default()
        };
        for e in entries {
            if undone.contains(&e.id) {
                report.skipped.push((e.id.clone(), "already undone".into()));
                continue;
            }
            if e.is_redacted() {
                report
                    .skipped
                    .push((e.id.clone(), "cannot be undone".into()));
                continue;
            }
            match self.invert(&e) {
                Ok(Ok(step)) => {
                    self.record(by, step, Some(&e.id))?;
                    report.undone.push(e.id.clone());
                }
                Ok(Err(why)) => report.skipped.push((e.id.clone(), why)),
                // 目标之后被挪走、删掉或改得对不上：这一条跳过，其余照撤。
                Err(Error::NotFound(w) | Error::Conflict(w)) => {
                    report
                        .skipped
                        .push((e.id.clone(), format!("changed since: {w}")));
                }
                Err(err) => return Err(err),
            }
        }
        Ok(report)
    }

    fn current_hash(&self, e: &JournalEntry) -> Result<Option<String>> {
        let Some(path) = e.targets.first().and_then(|t| t.path.value()) else {
            return Ok(None);
        };
        Ok(read_opt(&self.path(path))?.map(|b| hash_tag(&b)))
    }

    /// 反着做一条；外层错误是读写失败，内层是这条撤不了的原因。
    fn invert(&mut self, e: &JournalEntry) -> Result<std::result::Result<Step, Skip>> {
        let change = e.change.clone().unwrap_or(Value::Null);
        let t = e.targets.first().cloned();
        let tid = t
            .as_ref()
            .and_then(|t| t.id.value().cloned())
            .unwrap_or_default();
        let kind = t.as_ref().map(|t| t.kind.clone()).unwrap_or_default();
        let unchanged = self.current_hash(e)? == t.as_ref().and_then(|t| t.after.clone());
        let skip = |s: &str| Ok(Err(s.to_owned()));
        let path = t
            .as_ref()
            .and_then(|t| t.path.value().cloned())
            .unwrap_or_default();
        let op = static_op(&e.op);
        if change.get("createdFile").is_some() || change.get("removedFile").is_some() {
            if !unchanged {
                return skip("changed since");
            }
            return Ok(Ok(self.flip_file(&kind, &tid, &path, op, &change)?));
        }
        if let Some(body) = change.get("restoreBody").and_then(Value::as_str) {
            if !unchanged {
                return skip("changed since");
            }
            let body = body.to_owned();
            let swap = |d: &mut String| {
                let prev = std::mem::replace(d, body.clone());
                Ok(json!({ "restoreBody": prev }))
            };
            return Ok(Ok(match DocKind::parse(&kind) {
                Some(DocKind::Topic) => {
                    self.edit_written::<TopicMeta>(&tid, op, |d| swap(&mut d.body))?
                }
                Some(DocKind::Card) => {
                    self.edit_written::<CardMeta>(&tid, op, |d| swap(&mut d.body))?
                }
                Some(DocKind::Draft) => {
                    self.edit_written::<DraftMeta>(&tid, op, |d| swap(&mut d.body))?
                }
                None => return skip("cannot be undone"),
            }));
        }
        let step = match e.op.as_str() {
            "purge" => return skip("cannot be undone"),
            op if ManifestSet::from_op(op).is_some() && kind == "item" => {
                let which =
                    ManifestSet::from_op(op).ok_or_else(|| Error::InvalidArgs(op.into()))?;
                let inv = invert_set(&change)?;
                self.set_manifest(&ItemId::parse(&tid)?, which, &inv)?
            }
            "set-topics" => {
                let inv = invert_set(&change)?;
                self.edit_written::<CardMeta>(&tid, "set-topics", |d| {
                    to_value(&apply_set(&mut d.meta.topics, &inv))
                })?
            }
            "set-private" => {
                let id = ItemId::parse(&tid)?;
                if Some(self.item(&id)?.manifest.private) != change["after"].as_bool() {
                    return skip("changed since");
                }
                let v = change["before"].as_bool().unwrap_or(false);
                self.edit_manifest(&id, "set-private", |m| {
                    m.private = v;
                    Ok(json!({"before": !v, "after": v}))
                })?
            }
            "set-followed" => {
                let v = change["before"].as_bool().unwrap_or(false);
                let after = change["after"].as_bool();
                let w = self.find::<TopicMeta>(&tid)?;
                if Some(w.doc.meta.followed) != after {
                    return skip("changed since");
                }
                self.edit_written::<TopicMeta>(&tid, "set-followed", |d| {
                    d.meta.followed = v;
                    Ok(json!({"before": !v, "after": v}))
                })?
            }
            "set-setting" => {
                let key = change["key"].as_str().unwrap_or_default().to_owned();
                let before = (change["existed"] == true).then(|| change["before"].clone());
                self.set_setting(&key, before)?
            }
            "set-takeaway" => {
                let id = ItemId::parse(&tid)?;
                let (doc, _) = self.notes(&id)?;
                if json!(doc.meta.takeaway) != change["after"] {
                    return skip("changed since");
                }
                if change["created"] == true && unchanged {
                    self.remove_item_file(&id, "notes.md", "set-takeaway")?
                } else {
                    let before = change["before"].as_str().map(str::to_owned);
                    self.edit_notes(&id, "set-takeaway", |d| {
                        let prev = d.meta.takeaway.clone();
                        d.meta.takeaway = before.clone();
                        Ok(json!({"before": prev, "after": before}))
                    })?
                }
            }
            "edit-notes" => {
                let id = ItemId::parse(&tid)?;
                if change["created"] == true && unchanged {
                    self.remove_item_file(&id, "notes.md", "edit-notes")?
                } else {
                    let splices = splices_of(&change)?;
                    match self.edit_notes(&id, "edit-notes", |d| {
                        revert_body(&mut d.body, &splices, unchanged)
                    }) {
                        Ok(s) => s,
                        Err(Error::Conflict(_)) => return skip("changed since"),
                        Err(err) => return Err(err),
                    }
                }
            }
            "add-highlight" => {
                let id = ItemId::parse(&tid)?;
                let line = change["line"].as_str().unwrap_or_default().to_owned();
                let Some(h) = Highlight::parse(&line) else {
                    return skip("cannot be undone");
                };
                if change["created"] == true && unchanged {
                    self.remove_item_file(&id, "highlights.md", "remove-highlight")?
                } else {
                    match self.edit_highlights(&id, "remove-highlight", |list| {
                        let i = list
                            .iter()
                            .position(|x| x.id == h.id)
                            .ok_or_else(|| Error::Conflict("gone".into()))?;
                        let h = list.remove(i);
                        Ok(json!({"line": h.render(), "index": i}))
                    }) {
                        Ok(s) => s,
                        Err(Error::Conflict(_)) => return skip("changed since"),
                        Err(err) => return Err(err),
                    }
                }
            }
            "remove-highlight" => {
                let id = ItemId::parse(&tid)?;
                let Some(h) = change["line"].as_str().and_then(Highlight::parse) else {
                    return skip("cannot be undone");
                };
                let index = change["index"].as_u64().unwrap_or(u64::MAX) as usize;
                self.edit_highlights(&id, "add-highlight", |list| {
                    if list.iter().any(|x| x.id == h.id) {
                        return Err(Error::Conflict(format!("highlight {} exists", h.id)));
                    }
                    let line = h.render();
                    list.insert(index.min(list.len()), h);
                    Ok(json!({"line": line}))
                })?
            }
            "create-topic" => self.trash_doc::<TopicMeta>(&tid, "undo", None)?,
            "create-card" => self.trash_doc::<CardMeta>(&tid, "undo", None)?,
            "create-draft" => self.trash_doc::<DraftMeta>(&tid, "undo", None)?,
            "edit-topic" => {
                let ch = change.clone();
                self.edit_written::<TopicMeta>(&tid, "edit-topic", |d| {
                    let mut c = serde_json::Map::new();
                    if let Some(title) = ch["title"]["before"].as_str() {
                        c.insert(
                            "title".into(),
                            json!({"before": d.meta.title, "after": title}),
                        );
                        d.meta.title = title.to_owned();
                    }
                    for key in ["keywords", "match"] {
                        if let Some(set) = ch.get(key) {
                            let inv = invert_set(set)?;
                            let list = if key == "keywords" {
                                &mut d.meta.keywords
                            } else {
                                &mut d.meta.match_
                            };
                            c.insert(key.into(), to_value(&apply_set(list, &inv))?);
                        }
                    }
                    Ok(Value::Object(c))
                })?
            }
            "edit-topic-page" | "edit-card" | "edit-draft" => {
                let splices = splices_of(&change)?;
                let ch = change.clone();
                let op: &'static str = match e.op.as_str() {
                    "edit-topic-page" => "edit-topic-page",
                    "edit-card" => "edit-card",
                    _ => "edit-draft",
                };
                let r = match DocKind::parse(&kind) {
                    Some(DocKind::Topic) => self.edit_written::<TopicMeta>(&tid, op, |d| {
                        revert_body(&mut d.body, &splices, unchanged)
                    }),
                    Some(DocKind::Card) => self.edit_written::<CardMeta>(&tid, op, |d| {
                        let mut c = revert_body(&mut d.body, &splices, unchanged)?;
                        revert_title(&mut d.meta.title, &ch, &mut c);
                        Ok(c)
                    }),
                    Some(DocKind::Draft) => self.edit_written::<DraftMeta>(&tid, op, |d| {
                        let mut c = revert_body(&mut d.body, &splices, unchanged)?;
                        revert_title(&mut d.meta.title, &ch, &mut c);
                        if let Some(k) = ch["kind"]["before"].as_str() {
                            c["kind"] = json!({"before": d.meta.kind, "after": k});
                            d.meta.kind = k.to_owned();
                        }
                        if let Some(from) = ch.get("from") {
                            c["from"] = to_value(&apply_set(&mut d.meta.from, &invert_set(from)?))?;
                        }
                        Ok(c)
                    }),
                    None => return skip("cannot be undone"),
                };
                match r {
                    Ok(s) => s,
                    Err(Error::Conflict(_)) => return skip("changed since"),
                    Err(err) => return Err(err),
                }
            }
            "add-citation" => {
                let label = change["label"].as_str().unwrap_or_default().to_owned();
                let (rp, fp) = (
                    change["refParagraph"] == true,
                    change["footnoteParagraph"] == true,
                );
                let r = match DocKind::parse(&kind) {
                    Some(DocKind::Topic) => {
                        self.edit_written::<TopicMeta>(&tid, "add-citation", |d| {
                            let before = d.body.clone();
                            remove_citation(&mut d.body, &label, rp, fp)?;
                            Ok(json!({ "restoreBody": before }))
                        })
                    }
                    Some(DocKind::Card) => {
                        self.edit_written::<CardMeta>(&tid, "add-citation", |d| {
                            let before = d.body.clone();
                            remove_citation(&mut d.body, &label, rp, fp)?;
                            Ok(json!({ "restoreBody": before }))
                        })
                    }
                    Some(DocKind::Draft) => {
                        self.edit_written::<DraftMeta>(&tid, "add-citation", |d| {
                            let before = d.body.clone();
                            remove_citation(&mut d.body, &label, rp, fp)?;
                            Ok(json!({ "restoreBody": before }))
                        })
                    }
                    None => return skip("cannot be undone"),
                };
                match r {
                    Ok(s) => s,
                    Err(Error::Conflict(_)) => return skip("changed since"),
                    Err(err) => return Err(err),
                }
            }
            "move" => {
                let from = change["from"].as_str().unwrap_or_default();
                let dk = DocKind::parse(&kind).ok_or_else(|| Error::InvalidArgs(kind.clone()))?;
                let sub = subdir_of(dk, from);
                match dk {
                    DocKind::Topic => self.move_doc::<TopicMeta>(&tid, &sub)?,
                    DocKind::Card => self.move_doc::<CardMeta>(&tid, &sub)?,
                    DocKind::Draft => self.move_doc::<DraftMeta>(&tid, &sub)?,
                }
            }
            "move-to-trash" | "keep-text" => {
                let trash_id = change["trashId"].as_str().unwrap_or_default().to_owned();
                if self.trash_entry(&trash_id).is_err() {
                    return skip("no longer in trash");
                }
                let state = change.get("before").filter(|v| !v.is_null()).cloned();
                self.restore_step(&trash_id, state.as_ref())?
            }
            "restore" => {
                let entry: usuyo_contract::TrashEntry =
                    serde_json::from_value(change["entry"].clone())
                        .map_err(|e| Error::InvalidArgs(e.to_string()))?;
                // 原来的到期时间已经过了的，记放回的时间，再留 trashDays。
                let expired =
                    crate::trash::expires_at(&entry.trashed_at, self.settings.trash_days)?
                        .is_some_and(|at| at <= time::now());
                let at = if expired {
                    time::now()
                } else {
                    entry.trashed_at.clone()
                };
                match (entry.kind.as_str(), entry.reason.as_str()) {
                    ("item", "keep-text") => self.keep_text(&ItemId::parse(&entry.target)?)?,
                    ("item", "merge") => return skip("cannot be undone"),
                    ("item", reason) => self.trash_item(
                        &ItemId::parse(&entry.target)?,
                        reason,
                        Some((&entry.id, &at)),
                    )?,
                    ("topic", reason) => {
                        self.trash_doc::<TopicMeta>(&entry.target, reason, Some((&entry.id, &at)))?
                    }
                    ("card", reason) => {
                        self.trash_doc::<CardMeta>(&entry.target, reason, Some((&entry.id, &at)))?
                    }
                    ("draft", reason) => {
                        self.trash_doc::<DraftMeta>(&entry.target, reason, Some((&entry.id, &at)))?
                    }
                    _ => return skip("cannot be undone"),
                }
            }
            "add-item" => {
                let id = ItemId::parse(&tid)?;
                if change["created"] == true {
                    if !self.item_dir(&id).exists() {
                        return skip("no longer in the library");
                    }
                    self.trash_item(&id, "undo", None)?
                } else if let Some(o) = change.get("removedOrigin") {
                    let origin: usuyo_contract::Origin = serde_json::from_value(o.clone())
                        .map_err(|e| Error::InvalidArgs(e.to_string()))?;
                    self.edit_manifest(&id, "add-item", |m| {
                        let oid = origin.id.clone();
                        m.origins.push(origin);
                        Ok(json!({"created": false, "origin": oid}))
                    })?
                } else {
                    let origin = change["origin"].as_str().unwrap_or_default().to_owned();
                    self.edit_manifest(&id, "add-item", |m| {
                        let i = m
                            .origins
                            .iter()
                            .position(|o| o.id == origin)
                            .ok_or_else(|| Error::Conflict(format!("origin {origin} is gone")))?;
                        Ok(json!({ "removedOrigin": to_value(&m.origins.remove(i))? }))
                    })?
                }
            }
            "merge-items" if change.get("unmerged").is_some() => {
                let a = ItemId::parse(change["unmerged"].as_str().unwrap_or_default())?;
                let b = ItemId::parse(change["into"].as_str().unwrap_or_default())?;
                self.merge_items(&a, &b)?
            }
            "merge-items" => match self.unmerge(&change)? {
                Some(s) => s,
                None => return skip("changed since"),
            },
            _ => return skip("cannot be undone"),
        };
        Ok(Ok(step))
    }

    /// 撤销「新建了文件」和「删掉了文件」：删掉的那份内容存在 `change` 里，来回都能做。
    fn flip_file(
        &mut self,
        kind: &str,
        id: &str,
        rel: &str,
        op: &'static str,
        change: &Value,
    ) -> Result<Step> {
        let path = self.path(rel);
        let (change, before, after) = if change.get("createdFile").is_some() {
            let bytes = fsx::read(&path)?;
            let content = String::from_utf8(bytes.clone())
                .map_err(|e| Error::malformed(&path, e.to_string()))?;
            fsx::remove_file(&path)?;
            (
                json!({"removedFile": rel, "content": content}),
                Some(hash_tag(&bytes)),
                None,
            )
        } else {
            if path.exists() {
                return Err(Error::Conflict(format!("{rel} exists")));
            }
            let content = change["content"].as_str().unwrap_or_default();
            fsx::atomic_write(&path, content.as_bytes())?;
            (
                json!({"createdFile": rel}),
                None,
                Some(hash_tag(content.as_bytes())),
            )
        };
        Ok(Step {
            op,
            targets: vec![target(kind, id, rel, before, after)],
            change,
        })
    }

    /// 撤销时删掉这条 op 新建的文件（撤销后和改之前逐字节一样）。
    fn remove_item_file(&mut self, id: &ItemId, file: &str, op: &'static str) -> Result<Step> {
        let path = self.item_dir(id).join(file);
        let bytes = fsx::read(&path)?;
        let content =
            String::from_utf8(bytes.clone()).map_err(|e| Error::malformed(&path, e.to_string()))?;
        let before = Some(hash_tag(&bytes));
        fsx::remove_file(&path)?;
        Ok(Step {
            op,
            targets: vec![target(
                "item",
                id.as_str(),
                &format!("{}/{file}", id.rel_dir()),
                before,
                None,
            )],
            change: json!({"removedFile": file, "content": content}),
        })
    }

    fn unmerge(&mut self, change: &Value) -> Result<Option<Step>> {
        let a = ItemId::parse(change["from"].as_str().unwrap_or_default())?;
        let b = ItemId::parse(change["into"].as_str().unwrap_or_default())?;
        let tid = change["trashId"].as_str().unwrap_or_default().to_owned();
        if self.item_dir(&a).exists() || self.trash_entry(&tid).is_err() {
            return Ok(None);
        }
        let (adir, bdir) = (self.item_dir(&a), self.item_dir(&b));
        let tdir = self.path(".usuyo/trash").join(&tid);
        fsx::remove_file(&tdir.join("entry.json"))?;
        fsx::move_path(&tdir, &adir)?;
        for sid in change["snapshots"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            fsx::move_path(&bdir.join("raw").join(sid), &adir.join("raw").join(sid))?;
        }
        let before = &change["before"];
        let before_manifest = before["manifest"]
            .as_str()
            .ok_or_else(|| Error::InvalidArgs("merge change has no manifest".into()))?
            .as_bytes()
            .to_vec();
        fsx::atomic_write(&bdir.join("manifest.json"), &before_manifest)?;
        for (file, key) in [("notes.md", "notes"), ("highlights.md", "highlights")] {
            match before[key].as_str() {
                Some(text) => fsx::atomic_write(&bdir.join(file), text.as_bytes())?,
                None => fsx::remove_file(&bdir.join(file))?,
            }
        }
        let expected = read_opt(&bdir.join("content.md"))?.map(|b| hash_tag(&b));
        self.write_content(&b, expected.as_deref())?;
        Ok(Some(Step {
            op: "merge-items",
            targets: vec![
                target(
                    "item",
                    b.as_str(),
                    &format!("{}/manifest.json", b.rel_dir()),
                    None,
                    Some(hash_tag(&before_manifest)),
                ),
                target("item", a.as_str(), &a.rel_dir(), None, None),
            ],
            change: json!({"unmerged": a.as_str(), "into": b.as_str()}),
        }))
    }
}

fn new_id(given: &str) -> Result<String> {
    if given.is_empty() {
        Ok(uuid7())
    } else if crate::id::is_uuid7(given) {
        Ok(given.to_owned())
    } else {
        Err(Error::InvalidArgs(format!("bad id `{given}`")))
    }
}

fn non_empty(title: String) -> Result<String> {
    if title.trim().is_empty() {
        Err(Error::InvalidArgs("title is empty".into()))
    } else {
        Ok(title)
    }
}

/// 卡片没写标题时取第一行前 30 个字。
fn first_words(body: &str) -> String {
    use unicode_segmentation::UnicodeSegmentation;
    body.lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().graphemes(true).take(30).collect())
        .unwrap_or_default()
}

/// 列表整个换成 `new`，回增减。
fn replace_set(list: &mut Vec<String>, new: Vec<String>) -> SetChange {
    let c = SetChange {
        add: new.iter().filter(|x| !list.contains(x)).cloned().collect(),
        remove: list.iter().filter(|x| !new.contains(x)).cloned().collect(),
    };
    apply_set(list, &c)
}

/// 引用和划线的链接。`text` 不给快照时带引用时的主快照（SPEC §0.4）。
fn cite_link(
    item: &ItemId,
    primary: Option<&str>,
    layer: String,
    snapshot: Option<String>,
    fragment: Option<String>,
) -> Result<Link> {
    let snapshot = match snapshot {
        None if layer == "text" => primary.map(str::to_owned),
        s => s,
    };
    if layer_needs_snapshot(&layer) && snapshot.is_none() {
        return Err(Error::InvalidArgs(format!(
            "layer `{layer}` needs a snapshot"
        )));
    }
    if let Some(s) = &snapshot
        && !crate::id::is_uuid7(s)
    {
        return Err(Error::InvalidArgs(format!("bad snapshot `{s}`")));
    }
    Ok(Link {
        noun: "item".into(),
        id: item.to_string(),
        layer: Some(layer),
        snapshot,
        fragment,
    })
}

fn edit_body(body: &mut String, hunks: &[Hunk]) -> Result<Value> {
    let (new, splices) = apply_hunks(body, hunks)
        .map_err(|h| Error::Conflict(format!("hunk {h} does not match")))?;
    *body = new;
    Ok(json!({"splices": splices}))
}

fn splices_of(change: &Value) -> Result<Vec<Splice>> {
    match change.get("splices") {
        Some(s) => serde_json::from_value(s.clone()).map_err(|e| Error::InvalidArgs(e.to_string())),
        None => Ok(Vec::new()),
    }
}

fn revert_body(body: &mut String, splices: &[Splice], exact: bool) -> Result<Value> {
    let new = revert_splices(body, splices, exact)
        .ok_or_else(|| Error::Conflict("paragraphs changed since".into()))?;
    let inverse: Vec<Splice> = splices
        .iter()
        .rev()
        .map(|s| Splice {
            at: s.at,
            before: s.after.clone(),
            after: s.before.clone(),
        })
        .collect();
    *body = new;
    Ok(json!({"splices": inverse}))
}

fn revert_title(title: &mut String, change: &Value, out: &mut Value) {
    if let Some(t) = change["title"]["before"].as_str() {
        out["title"] = json!({"before": title.clone(), "after": t});
        *title = t.to_owned();
    }
}

fn is_footnote_block(p: &str) -> bool {
    !p.is_empty() && p.lines().all(|l| l.starts_with("[^"))
}

fn insert_citation(body: &mut String, mut cit: Citation, anchor: Option<&str>) -> Result<Value> {
    let used = labels(body);
    loop {
        let l = block_id()?;
        if !used.contains(&l) {
            cit.label = l;
            break;
        }
    }
    let reference = format!("[^{}]", cit.label);
    let mut p = Paragraphs::split(body);
    let ref_paragraph = match anchor {
        Some(a) => {
            let hits: Vec<usize> = p
                .items
                .iter()
                .enumerate()
                .filter(|(_, x)| x.as_str() == a)
                .map(|(i, _)| i)
                .collect();
            match hits.as_slice() {
                [i] => {
                    p.items[*i].push_str(&reference);
                    false
                }
                _ => return Err(Error::Conflict("anchor paragraph not found once".into())),
            }
        }
        None => {
            p.items.push(reference);
            true
        }
    };
    let line = cit.render();
    let footnote_paragraph = match p.items.last_mut() {
        Some(last) if is_footnote_block(last) && !ref_paragraph => {
            last.push('\n');
            last.push_str(&line);
            false
        }
        _ => {
            p.items.push(line);
            true
        }
    };
    *body = p.join();
    Ok(
        json!({"label": cit.label, "refParagraph": ref_paragraph, "footnoteParagraph": footnote_paragraph}),
    )
}

fn remove_citation(
    body: &mut String,
    label: &str,
    ref_paragraph: bool,
    footnote_paragraph: bool,
) -> Result<Value> {
    let gone = || Error::Conflict("citation changed since".into());
    let reference = format!("[^{label}]");
    let def = format!("[^{label}]: ");
    let mut p = Paragraphs::split(body);
    if footnote_paragraph {
        let i = p
            .items
            .iter()
            .position(|x| x.starts_with(&def) && !x.contains('\n'))
            .ok_or_else(gone)?;
        p.items.remove(i);
    } else {
        let i = p
            .items
            .iter()
            .position(|x| x.lines().any(|l| l.starts_with(&def)))
            .ok_or_else(gone)?;
        let kept: Vec<&str> = p.items[i]
            .lines()
            .filter(|l| !l.starts_with(&def))
            .collect();
        p.items[i] = kept.join("\n");
    }
    if ref_paragraph {
        let i = p
            .items
            .iter()
            .position(|x| *x == reference)
            .ok_or_else(gone)?;
        p.items.remove(i);
    } else {
        let i = p
            .items
            .iter()
            .position(|x| x.ends_with(&reference))
            .ok_or_else(gone)?;
        let n = p.items[i].len() - reference.len();
        p.items[i].truncate(n);
    }
    *body = p.join();
    Ok(json!({"removedCitation": label}))
}

/// 记录里的 op 名字换成静态字符串；不认识的回 `edit-notes`（只在撤销文件增删时用到名字）。
fn static_op(op: &str) -> &'static str {
    const NAMES: &[&str] = &[
        "add-item",
        "set-tags",
        "set-topics",
        "set-takeaway",
        "edit-notes",
        "add-highlight",
        "create-topic",
        "edit-topic",
        "create-card",
        "create-draft",
        "edit-topic-page",
        "edit-card",
        "edit-draft",
        "add-citation",
        "remove-highlight",
        "dismiss-suggestion",
        "set-followed",
        "set-private",
        "set-setting",
        "move",
        "set-not-same",
        "move-to-trash",
        "keep-text",
        "restore",
        "merge-items",
        "purge",
    ];
    NAMES
        .iter()
        .find(|n| **n == op)
        .copied()
        .unwrap_or("edit-notes")
}
