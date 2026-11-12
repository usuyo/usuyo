//! 条目文件夹（SPEC §1.2、§1.3）：收下、加快照、manifest 重算、`content.md`。

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use usuyo_contract::{
    Completeness, Extra, FetchRecord, FileHash, Identity, ItemState, MachinePicked, Manifest,
    Matched, Origin, Snapshot, TagMark, Tags,
};

use crate::fsx::{self, list_files, read_opt, write_json};
use crate::hash::hash_tag;
use crate::id::{ItemId, uuid7};
use crate::journal::{Batch, Step, target};
use crate::text::{angle_url, markdown_is_inert, md_inline, plain_to_markdown, rel_path_fits};
use crate::{Error, Library, Result, time};

/// 条目文件夹以内的相对路径不超过的 UTF-16 单位。
pub(crate) const ITEM_PATH_MAX: usize = 140;
/// 从库根起的相对路径不超过的 UTF-16 单位。
pub(crate) const ROOT_PATH_MAX: usize = 200;

/// 文字类的输出：只留文字以后还算数的。
const TEXT_OUTPUTS: &[&str] = &["text", "html", "subtitle", "meta", "comments"];

/// 一栏的内容在哪。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerSource {
    Text(String),
    File(PathBuf),
}

/// 读出来的一条。
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: ItemId,
    pub manifest: Manifest,
}

/// 收下一次（SPEC §1.2.1 的 `origins[]`），`id`、`at`、`device` 由宿主填。
#[derive(Debug, Clone, Default)]
pub struct NewOrigin {
    pub entry: String,
    pub taken_by: String,
    pub machine_picked: Option<MachinePicked>,
    pub matched: Vec<Matched>,
    pub subscription: Option<String>,
    pub import_batch: Option<String>,
    pub action: Option<String>,
    pub acted_at: Option<String>,
    pub acted_at_original: Option<String>,
}

/// 插件交回来的一次获取。`staged` 是 [`Library::staging_dir`] 给的文件夹，整个改名进 `raw/<snapshotId>/`。
#[derive(Debug, Clone)]
pub struct NewSnapshot {
    pub plugin: String,
    pub plugin_version: Option<String>,
    pub input_url: Option<String>,
    pub resolved_url: Option<String>,
    pub access: String,
    pub site: Option<String>,
    pub upstream: Option<Value>,
    /// 这次的 `--want`；`missing` = `want` − 实际拿到的。
    pub want: Vec<String>,
    pub missing_reason: Option<String>,
    pub text_origin: Option<String>,
    pub staged: PathBuf,
    /// `staged` 里每个文件的哈希，由 [`hash_staged`] 在拿资料库的锁之前算好。
    pub files: Vec<FileHash>,
    pub fetch: FetchRecord,
}

/// 暂存目录里每个文件（`usuyo.fetch.json` 除外）的路径、大小、哈希。大文件也不整个读进内存；
/// 不用拿资料库的锁，收下前先算，拿锁的时候只核对路径和大小。
pub fn hash_staged(staged: &Path) -> Result<Vec<FileHash>> {
    let mut files = Vec::new();
    for rel in list_files(staged)? {
        if rel == "usuyo.fetch.json" {
            continue;
        }
        let (size, sha256) = crate::hash::sha256_file(&staged.join(&rel))?;
        files.push(FileHash {
            path: rel,
            size,
            sha256,
        });
    }
    Ok(files)
}

/// 一次收下。
#[derive(Debug, Clone)]
pub struct Capture {
    /// `None` 或 `{platform: file}`：没有身份，随机 `file-` id。
    pub identity: Option<Identity>,
    /// 没有快照时的标题（`list` 或投递给的值），再没有就写原链接。
    pub title_hint: Option<String>,
    pub origin: NewOrigin,
    pub snapshot: Option<NewSnapshot>,
    /// Agent、`api` 令牌、终端外的命令行：遇到私密条目跳过（SPEC §1.7 的表）。
    pub skip_if_private: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddOutcome {
    Created(ItemId),
    /// 已在库里，多了一项 `origins`；`restored` 时是从回收站拿回来的。
    Existing {
        id: ItemId,
        restored: bool,
    },
    /// 私密，或跟着收遇到回收站里的：不加 `origins`。
    Skipped(ItemId),
}

impl AddOutcome {
    pub fn id(&self) -> &ItemId {
        match self {
            AddOutcome::Created(id) | AddOutcome::Skipped(id) => id,
            AddOutcome::Existing { id, .. } => id,
        }
    }
}

/// 读一份 `usuyo.fetch.json` 并查过里面的路径：路径来自插件，文件也可能在 usuyo 外被改过，
/// 只许快照文件夹以内的相对路径。
pub(crate) fn parse_fetch_record(path: &Path, bytes: &[u8]) -> Result<FetchRecord> {
    let rec: FetchRecord = fsx::parse_json(path, bytes)?;
    match rec.bad_path() {
        Some(bad) => Err(Error::Malformed {
            path: path.to_owned(),
            reason: format!("path `{bad}` leaves the snapshot folder"),
        }),
        None => Ok(rec),
    }
}

/// 快照实际拿到了哪些输出（`outputs` 词表）。
pub fn outputs_of(fetch: &FetchRecord) -> Vec<String> {
    let mut out = BTreeSet::new();
    if fetch.text.as_deref().is_some_and(|t| !t.is_empty()) {
        out.insert("text");
    }
    if fetch.extra.get("html").is_some_and(|v| v.is_string()) {
        out.insert("html");
    }
    if !fetch.images.is_empty() {
        out.insert("images");
    }
    for m in &fetch.media {
        match m.kind.as_str() {
            "video" => out.insert("media:video"),
            "audio" => out.insert("media:audio"),
            "subtitle" => out.insert("subtitle"),
            _ => false,
        };
    }
    if fetch.title.is_some() || fetch.author.is_some() || fetch.published_at.is_some() {
        out.insert("meta");
    }
    if fetch.extra.get("comments").is_some_and(|v| v.is_string()) {
        out.insert("comments");
    }
    out.into_iter().map(str::to_owned).collect()
}

fn is_text_output(o: &str) -> bool {
    TEXT_OUTPUTS.contains(&o)
}

/// 照快照重算宿主写的字段（SPEC §1.2.1）；`primary` 是主快照的 `usuyo.fetch.json`。
pub fn refresh(m: &mut Manifest, primary: Option<&FetchRecord>) {
    m.primary_snapshot = m
        .snapshots
        .iter()
        .max_by(|a, b| (a.outputs.len(), &a.at, &a.id).cmp(&(b.outputs.len(), &b.at, &b.id)))
        .map(|s| s.id.clone());
    let text_only = m.state == ItemState::TextOnly || m.text_only.is_some();
    let keep = |o: &String| !text_only || is_text_output(o);
    let outputs: BTreeSet<String> = m
        .snapshots
        .iter()
        .flat_map(|s| s.outputs.iter().cloned())
        .filter(keep)
        .collect();
    let missing: BTreeSet<String> = m
        .snapshots
        .iter()
        .flat_map(|s| s.missing.iter().cloned())
        .filter(|x| !outputs.contains(x) && keep(x))
        .collect();
    m.missing_reason = m
        .snapshots
        .iter()
        .rev()
        .find(|s| s.missing.iter().any(|x| missing.contains(x)))
        .and_then(|s| s.missing_reason.clone());
    m.outputs = outputs.into_iter().collect();
    m.missing = missing.into_iter().collect();
    let computed = if m.snapshots.is_empty() {
        Completeness::Link
    } else if m.missing.is_empty() {
        Completeness::Full
    } else {
        Completeness::Partial
    };
    m.completeness = m.completeness.max(computed);
    if let Some(f) = primary {
        if let Some(k) = &f.kind {
            m.kind = k.clone();
        }
        if let Some(t) = &f.title {
            m.title = t.clone();
        }
        m.author = f.author.clone().or(m.author.take());
        m.published_at = f.published_at.clone().or(m.published_at.take());
        m.published_at_original = f
            .published_at_original
            .clone()
            .or(m.published_at_original.take());
    }
    m.sort_arrays();
}

/// `content.md` 的字节（SPEC §1.2.3）。`body` 是主快照的 `text`，没有就是 `derived/readable.md`。
pub fn render_content(
    m: &Manifest,
    primary: Option<&FetchRecord>,
    readable: Option<&str>,
) -> String {
    let mut s = format!("# {}\n\n", md_inline(&m.title));
    let platform = m
        .identities
        .first()
        .map(|i| i.platform.clone())
        .or_else(|| ItemId::parse(&m.id).ok().map(|i| i.platform().to_owned()));
    let date = time::display_date(
        m.published_at_original.as_deref(),
        m.published_at.as_deref(),
    );
    let parts: Vec<String> = [
        m.author.as_ref().map(|a| md_inline(&a.name)),
        date,
        platform,
        (m.kind != "unknown").then(|| m.kind.clone()),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !parts.is_empty() {
        s.push_str(&parts.join(" · "));
        s.push('\n');
    }
    let snap = m
        .primary_snapshot
        .as_ref()
        .and_then(|p| m.snapshots.iter().find(|s| &s.id == p));
    let url = snap
        .and_then(|s| s.resolved_url.clone().or(s.input_url.clone()))
        .or_else(|| m.identities.iter().find_map(|i| i.url.clone()));
    if let Some(u) = url {
        s.push_str(&format!("<{}>\n", angle_url(&u)));
    }
    s.push_str("\n---\n\n");
    let body = match primary
        .and_then(|f| f.text.as_deref().filter(|t| !t.is_empty()).map(|t| (t, f)))
    {
        Some((t, f)) if f.text_format.as_deref() == Some("markdown") && markdown_is_inert(t) => {
            t.to_owned()
        }
        Some((t, _)) => plain_to_markdown(t),
        None => readable.unwrap_or_default().to_owned(),
    };
    if !body.is_empty() {
        s.push_str(&body);
        if !body.ends_with('\n') {
            s.push('\n');
        }
    }
    s
}

/// 主快照的 `usuyo.fetch.json`。
fn primary_fetch(dir: &Path, m: &Manifest) -> Result<Option<FetchRecord>> {
    let Some(p) = &m.primary_snapshot else {
        return Ok(None);
    };
    let path = dir.join("raw").join(p).join("usuyo.fetch.json");
    match read_opt(&path)? {
        Some(b) => Ok(Some(fsx::parse_json(&path, &b)?)),
        None => Ok(None),
    }
}

impl Library {
    pub(crate) fn item_dir(&self, id: &ItemId) -> PathBuf {
        self.path(&id.rel_dir())
    }

    /// 读一条；不在 `items/` 里（在回收站或没有）回 `not-found`。
    pub fn item(&self, id: &ItemId) -> Result<Item> {
        let path = self.item_dir(id).join("manifest.json");
        let bytes = read_opt(&path)?.ok_or_else(|| Error::NotFound(id.to_string()))?;
        let manifest: Manifest = fsx::parse_json(&path, &bytes)?;
        if manifest.id != id.as_str() {
            return Err(Error::malformed(
                &path,
                "manifest id differs from folder name",
            ));
        }
        Ok(Item {
            id: id.clone(),
            manifest,
        })
    }

    /// `items/` 下全部条目 id。
    pub fn item_ids(&self) -> Result<Vec<ItemId>> {
        let mut out = Vec::new();
        let items = self.path("items");
        for platform in read_dirs(&items)? {
            for dir in read_dirs(&platform)? {
                if let Some(id) = dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .and_then(|n| ItemId::parse(n).ok())
                {
                    out.push(id);
                }
            }
        }
        out.sort();
        Ok(out)
    }

    /// 某个快照的 `usuyo.fetch.json`。
    pub fn fetch_record(&self, id: &ItemId, snapshot: &str) -> Result<FetchRecord> {
        if !crate::id::is_uuid7(snapshot) {
            return Err(Error::InvalidArgs(format!("bad snapshot id `{snapshot}`")));
        }
        let path = self
            .item_dir(id)
            .join("raw")
            .join(snapshot)
            .join("usuyo.fetch.json");
        parse_fetch_record(&path, &fsx::read(&path)?)
    }

    /// 写回 manifest：数组排好、按主快照重算。回改前改后的哈希。
    pub(crate) fn save_manifest(
        &self,
        dir: &Path,
        m: &mut Manifest,
    ) -> Result<(Option<String>, String)> {
        let path = dir.join("manifest.json");
        let before = read_opt(&path)?.map(|b| hash_tag(&b));
        let primary = {
            refresh(m, None);
            primary_fetch(dir, m)?
        };
        refresh(m, primary.as_ref());
        let bytes = fsx::to_json_bytes(m)?;
        fsx::atomic_write(&path, &bytes)?;
        Ok((before, hash_tag(&bytes)))
    }

    /// 收下（SPEC §1.2、§1.7 的「再遇到同一身份」）。记 `add-item`，从回收站拿回来的另记 `restore`。
    pub fn add_item(&mut self, batch: &Batch, capture: Capture) -> Result<AddOutcome> {
        self.writable()?;
        let id = match capture.identity.as_ref().filter(|i| i.platform != "file") {
            Some(identity) => ItemId::from_identity(identity)?,
            None => ItemId::random_file()?,
        };
        let id = self.resolve_alias(&id)?.unwrap_or(id);
        let mut restored = false;
        if !self.item_dir(&id).exists()
            && let Some(entry) = self.find_trashed_item(&id)?
        {
            let private = self.trashed_manifest(&entry)?.private;
            if capture.origin.entry == "subscription" || (private && capture.skip_if_private) {
                self.discard_staged(&capture)?;
                return Ok(AddOutcome::Skipped(id));
            }
            self.restore_entry(batch, &entry.id, None)?;
            restored = true;
        }
        if self.item_dir(&id).exists() {
            let mut item = self.item(&id)?;
            if item.manifest.private && capture.skip_if_private {
                self.discard_staged(&capture)?;
                return Ok(AddOutcome::Skipped(id));
            }
            let dir = self.item_dir(&id);
            let expected = self.content_hash(&id)?;
            let origin = self.new_origin(capture.origin);
            let origin_id = origin.id.clone();
            item.manifest.origins.push(origin);
            if let Some(snap) = capture.snapshot {
                self.attach_snapshot(&dir, &mut item.manifest, snap)?;
            }
            let (before, after) = self.save_manifest(&dir, &mut item.manifest)?;
            self.write_content(&id, Some(&expected))?;
            self.record(
                batch,
                Step {
                    op: "add-item",
                    targets: vec![target(
                        "item",
                        id.as_str(),
                        &format!("{}/manifest.json", id.rel_dir()),
                        before,
                        Some(after),
                    )],
                    change: json!({"created": false, "origin": origin_id}),
                },
                None,
            )?;
            return Ok(AddOutcome::Existing { id, restored });
        }
        // 新条目：在 .usuyo/tmp/ 里拼好再改名进去。
        let tmp = self.staging_dir()?;
        let now = time::now();
        let origin = self.new_origin(capture.origin);
        let origin_id = origin.id.clone();
        let title = capture
            .title_hint
            .or_else(|| capture.identity.as_ref().and_then(|i| i.url.clone()))
            .or_else(|| capture.snapshot.as_ref().and_then(|s| s.input_url.clone()))
            .unwrap_or_default();
        let mut m = Manifest {
            id: id.to_string(),
            aliases: Vec::new(),
            identities: capture
                .identity
                .into_iter()
                .filter(|i| i.platform != "file")
                .collect(),
            kind: "unknown".into(),
            title,
            author: None,
            published_at: None,
            published_at_original: None,
            lang: None,
            added_at: now,
            origins: vec![origin],
            snapshots: Vec::new(),
            primary_snapshot: None,
            outputs: Vec::new(),
            missing: Vec::new(),
            missing_reason: None,
            completeness: Completeness::Link,
            tags: Tags::default(),
            topics: Vec::new(),
            same_as: Vec::new(),
            not_same_as: Vec::new(),
            private: false,
            state: ItemState::Normal,
            state_at: None,
            text_only: None,
            extra: Extra::new(),
        };
        for i in &mut m.identities {
            if i.platform == "web"
                && let Some(u) = &i.url
            {
                i.url = Some(crate::id::normalize_web_url(u)?);
            }
        }
        if let Some(snap) = capture.snapshot {
            self.attach_snapshot(&tmp, &mut m, snap)?;
        }
        let (_, after) = self.save_manifest(&tmp, &mut m)?;
        let content = self.content_bytes(&tmp, &m)?;
        fsx::atomic_write_readonly(&tmp.join("content.md"), content.as_bytes())?;
        fsx::move_path(&tmp, &self.item_dir(&id))?;
        self.record(
            batch,
            Step {
                op: "add-item",
                targets: vec![target(
                    "item",
                    id.as_str(),
                    &format!("{}/manifest.json", id.rel_dir()),
                    None,
                    Some(after),
                )],
                change: json!({"created": true, "origin": origin_id}),
            },
            None,
        )?;
        Ok(AddOutcome::Created(id))
    }

    /// 已有条目再取一次（重试、取评论）。和已有快照逐个哈希都一样时不新建，回 `None`。
    pub fn add_snapshot(&mut self, id: &ItemId, snapshot: NewSnapshot) -> Result<Option<String>> {
        self.writable()?;
        let mut item = self.item(id)?;
        let dir = self.item_dir(id);
        let expected = self.content_hash(id)?;
        let sid = self.attach_snapshot(&dir, &mut item.manifest, snapshot)?;
        if sid.is_some() {
            self.save_manifest(&dir, &mut item.manifest)?;
            self.write_content(id, Some(&expected))?;
        }
        Ok(sid)
    }

    fn new_origin(&self, o: NewOrigin) -> Origin {
        Origin {
            id: uuid7(),
            entry: o.entry,
            taken_by: o.taken_by,
            machine_picked: o.machine_picked,
            matched: o.matched,
            subscription: o.subscription,
            import_batch: o.import_batch,
            action: o.action,
            acted_at: o.acted_at,
            acted_at_original: o.acted_at_original,
            at: time::now(),
            device: self.device_id.clone(),
            extra: Extra::new(),
        }
    }

    fn discard_staged(&self, capture: &Capture) -> Result<()> {
        if let Some(s) = &capture.snapshot {
            self.check_staged(&s.staged)?;
            fsx::remove_dir(&s.staged)?;
        }
        Ok(())
    }

    pub(crate) fn check_staged(&self, staged: &Path) -> Result<()> {
        let tmp = self.path(".usuyo/tmp");
        if staged.starts_with(&tmp) && staged != tmp {
            Ok(())
        } else {
            Err(Error::InvalidArgs(format!(
                "{} is not a staging dir of this library",
                staged.display()
            )))
        }
    }

    /// 把暂存的输出目录移进 `<dir>/raw/<snapshotId>/`，写 `usuyo.fetch.json`，记进 manifest。
    fn attach_snapshot(
        &self,
        dir: &Path,
        m: &mut Manifest,
        s: NewSnapshot,
    ) -> Result<Option<String>> {
        self.check_staged(&s.staged)?;
        fsx::reject_symlinks(&s.staged)?;
        if let Some(bad) = s.fetch.bad_path() {
            return Err(Error::InvalidArgs(format!(
                "fetch output path `{bad}` leaves the output folder"
            )));
        }
        // 算哈希以后暂存目录不该再变：路径和大小对不上就不收。
        let mut listed = Vec::new();
        for rel in list_files(&s.staged)? {
            if rel == "usuyo.fetch.json" {
                continue;
            }
            let path = s.staged.join(&rel);
            let size = fs::metadata(&path).map_err(|e| Error::io(&path, e))?.len();
            listed.push((rel, size));
        }
        let hashed: Vec<(String, u64)> = s.files.iter().map(|f| (f.path.clone(), f.size)).collect();
        if listed != hashed {
            return Err(Error::InvalidArgs(
                "staged files changed after they were hashed".into(),
            ));
        }
        let files = s.files;
        // 和已有快照逐个文件哈希都一样：不新建。
        for existing in &m.snapshots {
            let path = dir.join("raw").join(&existing.id).join("usuyo.fetch.json");
            if let Some(b) = read_opt(&path)? {
                let rec = parse_fetch_record(&path, &b)?;
                if !files.is_empty() && rec.files == files {
                    fsx::remove_dir(&s.staged)?;
                    return Ok(None);
                }
            }
        }
        let sid = uuid7();
        for f in &files {
            let rel = format!("raw/{sid}/{}", f.path);
            if !rel_path_fits(&rel, ITEM_PATH_MAX) {
                return Err(Error::TooLarge(format!(
                    "path `{rel}` is longer than {ITEM_PATH_MAX} UTF-16 units"
                )));
            }
        }
        let outputs = outputs_of(&s.fetch);
        let missing: Vec<String> = s
            .want
            .iter()
            .filter(|w| !outputs.contains(w))
            .cloned()
            .collect();
        let completeness = if outputs.is_empty() {
            Completeness::Link
        } else if missing.is_empty() {
            Completeness::Full
        } else {
            Completeness::Partial
        };
        let mut record = s.fetch;
        record.extra.remove("credentialsUpdate");
        record.files = files;
        let at = time::now();
        if let Some(Value::Array(tags)) = record.extra.get("platformTags") {
            for tag in tags.iter().filter_map(Value::as_str) {
                if !m.tags.platform.iter().any(|t| t.tag == tag) {
                    m.tags.platform.push(TagMark {
                        tag: tag.to_owned(),
                        by: s.plugin.clone(),
                        at: at.clone(),
                        confidence: 1.0,
                        extra: Extra::new(),
                    });
                }
            }
        }
        let raw = dir.join("raw").join(&sid);
        fsx::move_path(&s.staged, &raw)?;
        write_json(&raw.join("usuyo.fetch.json"), &record)?;
        m.snapshots.push(Snapshot {
            id: sid.clone(),
            plugin: s.plugin,
            plugin_version: s.plugin_version,
            at,
            input_url: s.input_url,
            resolved_url: s.resolved_url,
            access: s.access,
            site: s.site,
            upstream: s.upstream,
            outputs,
            missing,
            missing_reason: s.missing_reason,
            completeness,
            text_origin: s.text_origin,
            extra: Extra::new(),
        });
        Ok(Some(sid))
    }

    fn content_bytes(&self, dir: &Path, m: &Manifest) -> Result<String> {
        let primary = primary_fetch(dir, m)?;
        let path = dir.join("derived/readable.md");
        let readable = match read_opt(&path)? {
            Some(b) => Some(fsx::decode(&path, &b)?.0),
            None => None,
        };
        Ok(render_content(m, primary.as_ref(), readable.as_deref()))
    }

    /// 照主快照重写 `content.md`（只读文件）。`last_written` 是本机上次写的哈希（`jobs.db` 里存着）：
    /// 现有的和它不一样、也不等于这次要写的，就是外面改过，先进回收站（`kind: content`）。
    /// 回这次写的哈希，调用方存回去。
    pub fn write_content(&mut self, id: &ItemId, last_written: Option<&str>) -> Result<String> {
        self.writable()?;
        let item = self.item(id)?;
        let dir = self.item_dir(id);
        let new = self.content_bytes(&dir, &item.manifest)?;
        let new_hash = hash_tag(new.as_bytes());
        let path = dir.join("content.md");
        if let Some(existing) = read_opt(&path)? {
            let h = hash_tag(&existing);
            if h == new_hash {
                return Ok(new_hash);
            }
            if Some(h.as_str()) != last_written {
                self.trash_local_file(id, "content.md", "content")?;
            }
        }
        fsx::atomic_write_readonly(&path, new.as_bytes())?;
        Ok(new_hash)
    }

    /// 照现在的 manifest 会生成的 `content.md` 的哈希：内部改动前先算好，改完拿它当「上次写的」。
    pub(crate) fn content_hash(&self, id: &ItemId) -> Result<String> {
        let item = self.item(id)?;
        Ok(hash_tag(
            self.content_bytes(&self.item_dir(id), &item.manifest)?
                .as_bytes(),
        ))
    }

    /// 一栏在哪（SPEC §1.2.3 的表）：`text` 回原文的字（主快照的 `text`，没有就是
    /// `derived/readable.md`），其余回文件路径。`snapshot` 照 §0.4 哪些层带。
    ///
    /// 转录的层（feature 003 §7.1）：`asr`、`ocr`、`frames` 回这一层最新一次转录的产物，
    /// `<层>/<runId>` 回指定的那次，`ocr/<imageKey>` 回盖到这张图的最新一次；给了 `snapshot` 时
    /// 只看输入在这个快照里的转录。最新一次是空的（没有人声、没认出字）回 `not-found`，
    /// 不往回找旧的。`video` 有合起来的 `play.mp4` 时给它，没有给原样的画面。
    /// `frame/<runId>/<毫秒>` 回那次 `keyframes` 转录里的一帧。
    /// `layer` 来自调用方（本机接口的 `/media/{id}/{*layer}`），不可信：路径只经过各记录拼，
    /// 拼出来的都要解析掉符号链接以后还在条目文件夹里。
    ///
    /// 调用方：`usuyo-core` 的 `Engine::media`（本机接口给媒体文件，WP8）、条目页（WP7）。
    pub fn resolve_layer(
        &self,
        id: &ItemId,
        layer: &str,
        snapshot: Option<&str>,
    ) -> Result<LayerSource> {
        let item = self.item(id)?;
        let dir = self.item_dir(id);
        let snap = || -> Result<String> {
            let s = snapshot
                .map(str::to_owned)
                .or_else(|| item.manifest.primary_snapshot.clone())
                .ok_or_else(|| Error::NotFound(format!("{id} has no snapshot")))?;
            if item.manifest.snapshots.iter().any(|x| x.id == s) {
                Ok(s)
            } else {
                Err(Error::NotFound(format!("snapshot {s}")))
            }
        };
        let not_found = || Error::NotFound(format!("{layer} of {id}"));
        let existing = |p: PathBuf| -> Result<LayerSource> {
            contained_file(&dir, &p, not_found).map(LayerSource::File)
        };
        let (name, key) = match layer.split_once('/') {
            Some((n, k)) => (n, Some(k)),
            None => (layer, None),
        };
        let image_key = || -> Result<&str> {
            key.filter(|k| is_image_key(k))
                .ok_or_else(|| Error::InvalidArgs(format!("bad image key in `{layer}`")))
        };
        match name {
            "text" => {
                let s = snap()?;
                let rec = self.fetch_record(id, &s)?;
                match rec.text.filter(|t| !t.is_empty()) {
                    Some(t) => Ok(LayerSource::Text(t)),
                    None => existing(dir.join("derived/readable.md")),
                }
            }
            "asr" | "ocr" | "frames" | "frame" => self
                .resolve_run_layer(id, layer, name, key, snapshot)
                .map(LayerSource::File),
            "image" => {
                let k = image_key()?;
                for s in &item.manifest.snapshots {
                    let rec = self.fetch_record(id, &s.id)?;
                    if let Some(f) = rec
                        .files
                        .iter()
                        .find(|f| rec.images.contains(&f.path) && f.sha256.starts_with(k))
                    {
                        return existing(dir.join("raw").join(&s.id).join(&f.path));
                    }
                }
                Err(Error::NotFound(format!("{layer} of {id}")))
            }
            "video" | "audio" => {
                if name == "video"
                    && let Some(play) = self.play_file(id, snapshot)?
                {
                    return Ok(LayerSource::File(play));
                }
                let s = snap()?;
                let rec = self.fetch_record(id, &s)?;
                let m = rec
                    .media
                    .iter()
                    .find(|m| m.kind == name)
                    .ok_or_else(|| Error::NotFound(format!("{layer} of {id}")))?;
                existing(dir.join("raw").join(&s).join(&m.path))
            }
            "vision" => existing(dir.join("derived/vision.md")),
            "summary" => existing(dir.join("derived/summary.md")),
            "notes" => existing(dir.join("notes.md")),
            "comments" => existing(dir.join("raw").join(snap()?).join("comments.json")),
            _ => Err(Error::InvalidArgs(format!("unknown layer `{layer}`"))),
        }
    }

    /// 别的条目 `aliases` 里有这个 id：回那一条。逐个读 manifest，以后由索引代替。
    /// 读不了的条目跳过并记日志，不挡住收下新的。
    fn resolve_alias(&self, id: &ItemId) -> Result<Option<ItemId>> {
        if self.item_dir(id).exists() {
            return Ok(None);
        }
        for other in self.item_ids()? {
            let item = match self.item(&other) {
                Ok(item) => item,
                Err(e) => {
                    tracing::warn!(item = %other, error = %e, "item not readable");
                    continue;
                }
            };
            if item.manifest.aliases.iter().any(|a| a == id.as_str()) {
                return Ok(Some(other));
            }
        }
        Ok(None)
    }

    /// 一批 id 现在各在哪：库里、回收站里、都不在。别名换成它指向的那一条。
    /// 只读；条目的 manifest 和回收站都只扫一遍（导入的预览一次问几百条）。
    pub fn whereabouts(&self, ids: &[ItemId]) -> Result<Vec<Whereabouts>> {
        let mut aliases: Option<std::collections::BTreeMap<String, ItemId>> = None;
        let trash = self.trash_entries()?;
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if self.item_dir(id).exists() {
                out.push(Whereabouts::Stored(id.clone()));
                continue;
            }
            let map = match &mut aliases {
                Some(m) => m,
                None => {
                    let mut m = std::collections::BTreeMap::new();
                    for other in self.item_ids()? {
                        match self.item(&other) {
                            Ok(item) => {
                                for a in item.manifest.aliases {
                                    m.insert(a, other.clone());
                                }
                            }
                            Err(e) => {
                                tracing::warn!(item = %other, error = %e, "item not readable")
                            }
                        }
                    }
                    aliases.insert(m)
                }
            };
            if let Some(target) = map.get(id.as_str()) {
                out.push(Whereabouts::Stored(target.clone()));
            } else if trash.iter().any(|e| {
                e.kind == "item"
                    && e.target == id.as_str()
                    && matches!(e.reason.as_str(), "delete" | "undo")
            }) {
                out.push(Whereabouts::Trashed(id.clone()));
            } else {
                out.push(Whereabouts::Absent(id.clone()));
            }
        }
        Ok(out)
    }
}

/// 一条现在在哪（[`Library::whereabouts`]）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Whereabouts {
    Stored(ItemId),
    Trashed(ItemId),
    Absent(ItemId),
}

/// `imageKey`：图片字节 SHA-256 的前 16 位小写十六进制（SPEC §1.2.2）。
pub(crate) fn is_image_key(k: &str) -> bool {
    k.len() == 16
        && k.bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

/// `p` 解析掉符号链接以后还得是 `root` 里的普通文件，不然回 `not_found()`。
/// 回解析后的路径：之后打开它的不再跟着符号链接走。
pub(crate) fn contained_file(
    root: &Path,
    p: &Path,
    not_found: impl Fn() -> Error,
) -> Result<PathBuf> {
    let real = p.canonicalize().map_err(|_| not_found())?;
    let root = root.canonicalize().map_err(|e| Error::io(root, e))?;
    if real.is_file() && real.starts_with(&root) {
        Ok(real)
    } else {
        Err(not_found())
    }
}

pub(crate) fn read_dirs(dir: &Path) -> Result<Vec<PathBuf>> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(Error::io(dir, e)),
    };
    let mut out = Vec::new();
    for e in entries {
        let p = e.map_err(|e| Error::io(dir, e))?.path();
        if p.is_dir() {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}
