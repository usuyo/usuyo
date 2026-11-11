//! 你写的那一层（SPEC §1.4）：主题、卡片、成稿是 `<标题>.md`，条目的 `notes.md`、`highlights.md`。

use std::fs;

use serde::Serialize;
use usuyo_contract::{CardMeta, DraftMeta, FrontMatter, NotesMeta, TopicMeta};

use crate::doc::Doc;
use crate::fsx::{self, decode, read_opt};
use crate::hash::hash_tag;
use crate::highlights::{self, Highlight};
use crate::id::ItemId;
use crate::item::ROOT_PATH_MAX;
use crate::text::{fold, rel_path_fits, sanitize};
use crate::{Error, Library, Result};

/// 主题、卡片、成稿。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocKind {
    Topic,
    Card,
    Draft,
}

impl DocKind {
    pub fn dir(self) -> &'static str {
        match self {
            DocKind::Topic => "topics",
            DocKind::Card => "cards",
            DocKind::Draft => "drafts",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            DocKind::Topic => "topic",
            DocKind::Card => "card",
            DocKind::Draft => "draft",
        }
    }

    pub fn parse(s: &str) -> Option<DocKind> {
        match s {
            "topic" => Some(DocKind::Topic),
            "card" => Some(DocKind::Card),
            "draft" => Some(DocKind::Draft),
            _ => None,
        }
    }
}

/// 有 `id` 和 `title` 的 front matter。
pub trait Titled: FrontMatter + Clone {
    const KIND: DocKind;
    fn id(&self) -> &str;
    fn title(&self) -> &str;
    fn set_title(&mut self, t: String);
}

macro_rules! titled {
    ($t:ty, $k:expr) => {
        impl Titled for $t {
            const KIND: DocKind = $k;
            fn id(&self) -> &str {
                &self.id
            }
            fn title(&self) -> &str {
                &self.title
            }
            fn set_title(&mut self, t: String) {
                self.title = t;
            }
        }
    };
}
titled!(TopicMeta, DocKind::Topic);
titled!(CardMeta, DocKind::Card);
titled!(DraftMeta, DocKind::Draft);

/// 一份主题、卡片或成稿，`path` 从库根起。
#[derive(Debug, Clone, PartialEq)]
pub struct Written<M> {
    pub path: String,
    pub doc: Doc<M>,
}

/// 扫一种文件夹的结果；读不了的不收、列出来（SPEC §1.4「不收」「解析不了的」）。
#[derive(Debug, Clone)]
pub struct Scan<M> {
    pub docs: Vec<Written<M>>,
    pub unreadable: Vec<(String, String)>,
}

/// 记一句和笔记：`notes.md`。
pub type Notes = Doc<NotesMeta>;

impl Library {
    /// 扫 `topics/`、`cards/` 或 `drafts/`，子文件夹都算。
    pub fn scan<M: Titled>(&self) -> Result<Scan<M>> {
        let base = self.path(M::KIND.dir());
        let mut scan = Scan {
            docs: Vec::new(),
            unreadable: Vec::new(),
        };
        for rel in fsx::list_files(&base)? {
            if !rel.ends_with(".md") || rel.split('/').any(|p| p.starts_with('.')) {
                continue;
            }
            let path = base.join(&rel);
            let rel_root = format!("{}/{rel}", M::KIND.dir());
            match Doc::<M>::parse(&path, &fsx::read(&path)?) {
                Ok(doc) if crate::id::is_uuid7(doc.meta.id()) => scan.docs.push(Written {
                    path: rel_root,
                    doc,
                }),
                Ok(_) => scan
                    .unreadable
                    .push((rel_root, "id is not a UUIDv7".into())),
                Err(e) => scan.unreadable.push((rel_root, e.to_string())),
            }
        }
        Ok(scan)
    }

    /// 按 id 找；两份同一个 id 时回 `conflict`。
    pub fn find<M: Titled>(&self, id: &str) -> Result<Written<M>> {
        let mut hits = self
            .scan::<M>()?
            .docs
            .into_iter()
            .filter(|w| w.doc.meta.id() == id);
        let first = hits
            .next()
            .ok_or_else(|| Error::NotFound(format!("{} {id}", M::KIND.name())))?;
        if hits.next().is_some() {
            return Err(Error::Conflict(format!(
                "two files with {} id {id}",
                M::KIND.name()
            )));
        }
        Ok(first)
    }

    /// 文件该放哪：`<种类>/<子文件夹>/<sanitize(title)>.md`。同名（NFC 加大小写折叠）时
    /// `id` 字符串序大的那份加「 2」：对方大就让对方挪（不同步、不记改动）。
    pub(crate) fn place<M: Titled>(
        &self,
        subdir: &str,
        meta: &M,
        current: Option<&str>,
    ) -> Result<String> {
        let dir_rel = if subdir.is_empty() {
            M::KIND.dir().to_owned()
        } else {
            format!("{}/{subdir}", M::KIND.dir())
        };
        let stem = sanitize(meta.title());
        let dir = self.path(&dir_rel);
        let mut holders: Vec<(String, String)> = Vec::new(); // (折叠后的名字, 文件名)
        let entries = match fs::read_dir(&dir) {
            Ok(e) => Some(e),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(Error::io(&dir, e)),
        };
        if let Some(entries) = entries {
            for e in entries {
                let e = e.map_err(|err| Error::io(&dir, err))?;
                if let Some(n) = e.file_name().to_str()
                    && n.ends_with(".md")
                {
                    let rel = format!("{dir_rel}/{n}");
                    if Some(rel.as_str()) != current {
                        holders.push((fold(n), n.to_owned()));
                    }
                }
            }
        }
        let base = format!("{stem}.md");
        let chosen = match holders.iter().find(|(f, _)| *f == fold(&base)).cloned() {
            None => base,
            Some((_, other_name)) => {
                let other_path = dir.join(&other_name);
                // 解析不了的是别人放的文件：名字算占着，它不动。
                let other = Doc::<M>::parse(&other_path, &fsx::read(&other_path)?).ok();
                if other.is_some_and(|o| o.meta.id() > meta.id()) {
                    // 对方 id 大：对方挪到下一个空名字，这份用本名。
                    let free = next_free(&stem, &holders);
                    fsx::move_path(&other_path, &dir.join(&free))?;
                    base
                } else {
                    next_free(&stem, &holders)
                }
            }
        };
        let rel = format!("{dir_rel}/{chosen}");
        if !rel_path_fits(&rel, ROOT_PATH_MAX) {
            return Err(Error::TooLarge(format!(
                "path `{rel}` is longer than {ROOT_PATH_MAX} UTF-16 units"
            )));
        }
        Ok(rel)
    }

    /// 写一份；回改前改后的哈希。`from` 是原路径（改了标题或挪了时不同）。
    pub(crate) fn write_doc<M: Titled>(
        &self,
        from: Option<&str>,
        to: &str,
        doc: &Doc<M>,
    ) -> Result<(Option<String>, String)> {
        let before = match from {
            Some(f) => read_opt(&self.path(f))?.map(|b| hash_tag(&b)),
            None => None,
        };
        let bytes = doc.render()?;
        match from {
            Some(f) if f != to && fold(f) == fold(to) => {
                // 只改大小写的改名分两步，经过临时名：大小写不敏感的盘上两者是同一个文件。
                fsx::atomic_write(&self.path(f), &bytes)?;
                let tmp = self.path(&format!("{f}.usuyo-case-{}", crate::id::uuid7()));
                fsx::move_path(&self.path(f), &tmp)?;
                fsx::move_path(&tmp, &self.path(to))?;
            }
            Some(f) if f != to => {
                fsx::atomic_write(&self.path(to), &bytes)?;
                fsx::remove_file(&self.path(f))?;
                let stop = self.path(M::KIND.dir());
                fsx::prune_empty_dirs(self.path(f).parent().unwrap_or(&stop), &stop);
            }
            _ => fsx::atomic_write(&self.path(to), &bytes)?,
        }
        Ok((before, hash_tag(&bytes)))
    }

    /// 条目的 `notes.md`；没有就是空的。第二项：文件在不在。
    pub fn notes(&self, id: &ItemId) -> Result<(Notes, bool)> {
        let path = self.item_dir(id).join("notes.md");
        match read_opt(&path)? {
            Some(b) => Ok((Doc::parse(&path, &b)?, true)),
            None => Ok((Doc::new(NotesMeta::default(), String::new()), false)),
        }
    }

    /// 条目的划线，按加的先后。
    pub fn highlights(&self, id: &ItemId) -> Result<Vec<Highlight>> {
        let path = self.item_dir(id).join("highlights.md");
        match read_opt(&path)? {
            Some(b) => highlights::parse_file(&path, &decode(&path, &b)?.0),
            None => Ok(Vec::new()),
        }
    }
}

fn next_free(stem: &str, holders: &[(String, String)]) -> String {
    let mut n = 2;
    loop {
        let name = format!("{stem} {n}.md");
        if !holders.iter().any(|(f, _)| *f == fold(&name)) {
            return name;
        }
        n += 1;
    }
}

/// 子文件夹路径要每段都是 `sanitize` 过的样子，不能有 `..`。
pub(crate) fn check_subdir(sub: &str) -> Result<()> {
    if sub.is_empty() {
        return Ok(());
    }
    for part in sub.split('/') {
        if part.is_empty() || part == "." || part == ".." || sanitize(part) != part {
            return Err(Error::InvalidArgs(format!("bad folder `{sub}`")));
        }
    }
    Ok(())
}

/// `topics/a/b/x.md` 的子文件夹 `a/b`。
pub(crate) fn subdir_of(kind: DocKind, rel: &str) -> String {
    let inner = rel
        .strip_prefix(kind.dir())
        .unwrap_or(rel)
        .trim_start_matches('/');
    match inner.rsplit_once('/') {
        Some((d, _)) => d.to_owned(),
        None => String::new(),
    }
}

/// 序列化成 JSON 对象，给改动记录用。
pub(crate) fn to_value<T: Serialize>(v: &T) -> Result<serde_json::Value> {
    serde_json::to_value(v).map_err(|e| Error::InvalidArgs(e.to_string()))
}
