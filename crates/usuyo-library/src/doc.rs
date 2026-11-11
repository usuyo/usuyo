//! 你写的 `.md`：front matter 加正文；正文按段（空行分开）改（SPEC §1.5.1 的 `hunks`）。

use std::path::Path;

use serde::{Deserialize, Serialize};
use usuyo_contract::{FrontMatter, Hunk};

use crate::fsx::{TextStyle, decode, encode};
use crate::{Result, frontmatter};

/// 一份带 front matter 的 Markdown。
#[derive(Debug, Clone, PartialEq)]
pub struct Doc<M> {
    pub meta: M,
    pub body: String,
    pub style: TextStyle,
}

impl<M: FrontMatter> Doc<M> {
    pub fn new(meta: M, body: String) -> Doc<M> {
        Doc {
            meta,
            body,
            style: TextStyle::default(),
        }
    }

    pub fn parse(path: &Path, bytes: &[u8]) -> Result<Doc<M>> {
        let (text, style) = decode(path, bytes)?;
        let (fm, body) = frontmatter::split(path, &text)?;
        let meta = frontmatter::parse_meta(path, fm.as_deref())?;
        Ok(Doc { meta, body, style })
    }

    pub fn render(&self) -> Result<Vec<u8>> {
        let lines = frontmatter::render_meta(&self.meta)?;
        Ok(encode(&frontmatter::join(&lines, &self.body), self.style))
    }
}

/// 正文拆成段：按 `\n\n` 拆、去掉最后一个换行；拼回来逐字节一样。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paragraphs {
    pub items: Vec<String>,
    trailing_newline: bool,
}

impl Paragraphs {
    pub fn split(body: &str) -> Paragraphs {
        if body.is_empty() {
            return Paragraphs {
                items: Vec::new(),
                trailing_newline: true,
            };
        }
        let (core, trailing_newline) = match body.strip_suffix('\n') {
            Some(c) => (c, true),
            None => (body, false),
        };
        Paragraphs {
            items: core.split("\n\n").map(str::to_owned).collect(),
            trailing_newline,
        }
    }

    pub fn join(&self) -> String {
        if self.items.is_empty() {
            return String::new();
        }
        let mut s = self.items.join("\n\n");
        if self.trailing_newline {
            s.push('\n');
        }
        s
    }

    /// 连续等于 `seq` 的唯一一处的起点；找不到或不唯一回 `None`。
    fn find_unique(&self, seq: &[String]) -> Option<usize> {
        if seq.is_empty() || seq.len() > self.items.len() {
            return None;
        }
        let mut found = None;
        for i in 0..=self.items.len() - seq.len() {
            if self.items[i..i + seq.len()] == *seq {
                if found.is_some() {
                    return None;
                }
                found = Some(i);
            }
        }
        found
    }
}

/// 应用到哪一处：撤销时照位置原样反做。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Splice {
    pub at: usize,
    pub before: Vec<String>,
    pub after: Vec<String>,
}

fn paras(s: &str) -> Vec<String> {
    if s.is_empty() {
        Vec::new()
    } else {
        s.split("\n\n").map(str::to_owned).collect()
    }
}

/// 照 hunks 改正文；有一个对不上就整个不改，回对不上的 hunk id。
pub fn apply_hunks(body: &str, hunks: &[Hunk]) -> Result<(String, Vec<Splice>), String> {
    let mut p = Paragraphs::split(body);
    let mut splices = Vec::with_capacity(hunks.len());
    for h in hunks {
        let after = paras(&h.after);
        let at = if h.before.is_empty() {
            match &h.anchor {
                None => p.items.len(),
                Some(a) => p
                    .find_unique(&paras(a))
                    .map(|i| i + paras(a).len())
                    .ok_or(h.id.clone())?,
            }
        } else {
            p.find_unique(&paras(&h.before)).ok_or(h.id.clone())?
        };
        let before = paras(&h.before);
        p.items.splice(at..at + before.len(), after.clone());
        splices.push(Splice { at, before, after });
    }
    Ok((p.join(), splices))
}

/// 反做：倒着把 `after` 换回 `before`。`exact` 时照位置，否则按内容找唯一一处。
pub fn revert_splices(body: &str, splices: &[Splice], exact: bool) -> Option<String> {
    let mut p = Paragraphs::split(body);
    for s in splices.iter().rev() {
        let at = if exact {
            let end = s.at + s.after.len();
            (end <= p.items.len() && p.items[s.at..end] == *s.after).then_some(s.at)?
        } else if s.after.is_empty() {
            // 插入被反做成删除时内容找不到位置；纯删除的反做只在照位置时可行。
            return None;
        } else {
            p.find_unique(&s.after)?
        };
        p.items.splice(at..at + s.after.len(), s.before.clone());
    }
    Some(p.join())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hunk(before: &str, after: &str, anchor: Option<&str>) -> Hunk {
        Hunk {
            id: "h".into(),
            before: before.into(),
            after: after.into(),
            anchor: anchor.map(Into::into),
        }
    }

    #[test]
    fn paragraphs_round_trip_exactly() {
        for body in [
            "",
            "\n",
            "a",
            "a\n",
            "a\n\nb\n",
            "a\n\n\n\nb",
            "a\nb\n\n\nc\n",
        ] {
            assert_eq!(Paragraphs::split(body).join(), body, "{body:?}");
        }
    }

    #[test]
    fn hunks_replace_insert_and_revert() {
        let body = "一\n\n二\n\n三\n";
        let (out, sp) = apply_hunks(
            body,
            &[
                hunk("二", "二改", None),
                hunk("", "插入", Some("一")),
                hunk("", "末尾", None),
            ],
        )
        .unwrap();
        assert_eq!(out, "一\n\n插入\n\n二改\n\n三\n\n末尾\n");
        assert_eq!(revert_splices(&out, &sp, true).unwrap(), body);
        assert!(apply_hunks(body, &[hunk("没有", "x", None)]).is_err());
        assert!(apply_hunks("a\n\na\n", &[hunk("a", "b", None)]).is_err());
        let (deleted, sp) = apply_hunks(body, &[hunk("二", "", None)]).unwrap();
        assert_eq!(deleted, "一\n\n三\n");
        assert_eq!(revert_splices(&deleted, &sp, true).unwrap(), body);
        assert_eq!(
            apply_hunks("", &[hunk("", "第一段", None)]).unwrap().0,
            "第一段\n"
        );
    }
}
