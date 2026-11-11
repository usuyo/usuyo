//! 一条现在存着什么：原文、评论、图、视频、记一句和笔记。索引和条目页都从这里读，两边一致。
//!
//! 这些字大多来自平台，是不可信输入：只当纯文字往外给，不拼进 HTML。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;
use usuyo_contract::{FetchRecord, Manifest};
use usuyo_library::{Item, LayerSource, Library, Notes};

use crate::{Error, Result};

/// 一条评论。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CommentText {
    pub author: String,
    pub text: String,
    pub likes: Option<u64>,
}

#[derive(Debug, Clone)]
pub(crate) struct ImageFile {
    /// `imageKey`：图片字节 SHA-256 的前 16 位十六进制（SPEC §1.2.2）。
    pub key: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct VideoFile {
    pub snapshot: String,
    pub duration_sec: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Content {
    /// 原文按段分好，带主快照 id。
    pub text: Option<(Option<String>, Vec<String>)>,
    pub comments: Option<(String, Vec<CommentText>)>,
    pub images: Vec<ImageFile>,
    pub video: Option<VideoFile>,
    pub takeaway: Option<String>,
    pub notes: String,
    /// 主快照的 `missingReason`：插件说有些没存下来，哪怕每一类都拿到了一些（例如十张图下了八张）。
    pub partial: Option<String>,
}

/// 读一条在 `items/` 里的条目存着的内容。
pub(crate) fn read(lib: &Library, item: &Item) -> Result<Content> {
    let id = &item.id;
    let m = &item.manifest;
    let dir = lib.root().join(id.rel_dir());
    let mut c = Content::default();
    let (notes, _) = lib.notes(id)?;
    c.takeaway = notes.meta.takeaway.filter(|t| !t.trim().is_empty());
    c.notes = notes.body;

    let Some(primary) = m.primary_snapshot.clone() else {
        return Ok(c);
    };
    let text = match lib.resolve_layer(id, "text", None) {
        Ok(LayerSource::Text(t)) => Some(t),
        Ok(LayerSource::File(p)) => {
            Some(std::fs::read_to_string(&p).map_err(|e| Error::io(&p, e))?)
        }
        Err(usuyo_library::Error::NotFound(_)) => None,
        Err(e) => return Err(e.into()),
    };
    if let Some(t) = text {
        let paragraphs = paragraphs(&t);
        if !paragraphs.is_empty() {
            c.text = Some((Some(primary.clone()), paragraphs));
        }
    }

    let rec = lib.fetch_record(id, &primary)?;
    let raw = dir.join("raw").join(&primary);
    if rec.extra.get("comments").and_then(Value::as_str).is_some() {
        let path = raw.join("comments.json");
        let bytes = std::fs::read(&path).map_err(|e| Error::io(&path, e))?;
        c.comments = Some((primary.clone(), parse_comments(&bytes)));
    }
    c.images = images(&rec, &raw);
    c.partial = rec
        .extra
        .get("missingReason")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|r| !r.is_empty())
        .map(str::to_owned);
    if let Some(v) = rec.media.iter().find(|m| m.kind == "video")
        && raw.join(&v.path).is_file()
    {
        c.video = Some(VideoFile {
            snapshot: primary.clone(),
            duration_sec: v.extra.get("durationSec").and_then(Value::as_f64),
        });
    }
    Ok(c)
}

/// 读整条在回收站里的条目：只读文字（原文、评论、记一句）。图和视频不经媒体地址给，在回收站里时不列。
pub(crate) fn read_trashed(dir: &Path, m: &Manifest) -> Result<Content> {
    let mut c = Content::default();
    let notes_path = dir.join("notes.md");
    if let Some(b) = read_opt(&notes_path)? {
        let notes = Notes::parse(&notes_path, &b)?;
        c.takeaway = notes.meta.takeaway.filter(|t| !t.trim().is_empty());
        c.notes = notes.body;
    }
    let Some(primary) = m.primary_snapshot.clone() else {
        return Ok(c);
    };
    let raw = dir.join("raw").join(&primary);
    let rec_path = raw.join("usuyo.fetch.json");
    let Some(b) = read_opt(&rec_path)? else {
        return Ok(c);
    };
    let rec: FetchRecord = serde_json::from_slice(&b)?;
    let text = match rec.text.clone().filter(|t| !t.is_empty()) {
        Some(t) => Some(t),
        None => read_opt(&dir.join("derived/readable.md"))?
            .map(|b| String::from_utf8_lossy(&b).into_owned()),
    };
    if let Some(t) = text {
        let paragraphs = paragraphs(&t);
        if !paragraphs.is_empty() {
            c.text = Some((Some(primary.clone()), paragraphs));
        }
    }
    if rec.extra.get("comments").and_then(Value::as_str).is_some()
        && let Some(b) = read_opt(&raw.join("comments.json"))?
    {
        c.comments = Some((primary, parse_comments(&b)));
    }
    Ok(c)
}

fn read_opt(path: &Path) -> Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::io(path, e)),
    }
}

fn images(rec: &FetchRecord, raw: &Path) -> Vec<ImageFile> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for path in &rec.images {
        let Some(f) = rec.files.iter().find(|f| &f.path == path) else {
            continue;
        };
        let Some(key) = f.sha256.get(..16) else {
            continue;
        };
        let full = raw.join(&f.path);
        // 只留文字以后图在回收站里，不在这里了。
        if full.is_file() && seen.insert(key.to_owned()) {
            out.push(ImageFile {
                key: key.to_owned(),
                path: full,
            });
        }
    }
    out
}

/// 按空行分段，去掉首尾空白，空段不要。
pub(crate) fn paragraphs(text: &str) -> Vec<String> {
    text.replace("\r\n", "\n")
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect()
}

/// `comments.json` 的格式归 plugin-kit（还没定）：认数组或 `{comments: [...]}`，
/// 每项认 `author`（字符串或 `{name}`）、`text`/`content`、`likes`/`likeCount`。认不出的项跳过。
pub(crate) fn parse_comments(bytes: &[u8]) -> Vec<CommentText> {
    let Ok(v) = serde_json::from_slice::<Value>(bytes) else {
        return Vec::new();
    };
    let list = match &v {
        Value::Array(a) => a.as_slice(),
        Value::Object(o) => o
            .get("comments")
            .and_then(Value::as_array)
            .map_or(&[][..], Vec::as_slice),
        _ => &[],
    };
    list.iter()
        .filter_map(|c| {
            let text = c
                .get("text")
                .or_else(|| c.get("content"))
                .and_then(Value::as_str)?;
            let author = match c.get("author") {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Object(o)) => o
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                _ => String::new(),
            };
            let likes = c
                .get("likes")
                .or_else(|| c.get("likeCount"))
                .and_then(Value::as_u64);
            Some(CommentText {
                author,
                text: text.to_owned(),
                likes,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_paragraphs() {
        assert_eq!(paragraphs("a\r\n\r\n\n b \n\nc\nd\n"), ["a", "b", "c\nd"]);
        assert!(paragraphs("  \n\n ").is_empty());
    }

    #[test]
    fn reads_comment_shapes() {
        let a = parse_comments(br#"[{"author":"x","text":"hi","likes":3},{"nope":1}]"#);
        assert_eq!(
            a,
            [CommentText {
                author: "x".into(),
                text: "hi".into(),
                likes: Some(3)
            }]
        );
        let b = parse_comments(br#"{"comments":[{"author":{"name":"y"},"content":"yo"}]}"#);
        assert_eq!(b[0].author, "y");
        assert!(parse_comments(b"<html>").is_empty());
    }
}
