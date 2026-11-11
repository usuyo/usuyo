//! 引用写成脚注（SPEC §1.4.3）：
//! `[^<标签>]: 「原话」[标题 · 栏 位置](usuyo://item/…) · 作者 · 发布日期`。

use crate::id::is_block_id;
use crate::link::{FORGOTTEN, Link};
use crate::text::{md_inline, md_unescape, take_escaped};

/// 解析出来的一条引用；`resolved`（来源还在不在）由调用方按库里的状态填。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    pub label: String,
    pub link: Link,
    pub quote: String,
    pub title: String,
    /// 链接文字里的「栏 位置」。
    pub column: String,
    pub author: Option<String>,
    pub date: Option<String>,
}

impl Citation {
    pub fn item(&self) -> &str {
        &self.link.id
    }

    /// 来源已经删掉（链接是 `usuyo://item/forgotten`）。
    pub fn forgotten(&self) -> bool {
        self.link.id == FORGOTTEN
    }

    /// 脚注定义那一行。
    pub fn render(&self) -> String {
        let mut s = format!(
            "[^{}]: 「{}」[{} · {}]({})",
            self.label,
            md_inline(&self.quote),
            md_inline(&self.title),
            md_inline(&self.column),
            self.link
        );
        for part in [&self.author, &self.date].into_iter().flatten() {
            s.push_str(" · ");
            s.push_str(&md_inline(part));
        }
        s
    }

    /// 只认宿主写的那种脚注；你手写的别的写法不算引用、不改写。
    pub fn parse(line: &str) -> Option<Citation> {
        let rest = line.strip_prefix("[^")?;
        let (label, rest) = rest.split_once("]: 「")?;
        if !is_block_id(label) {
            return None;
        }
        let (quote, rest) = take_escaped(rest, "」[")?;
        let (text, rest) = take_escaped(rest, "](")?;
        let close = rest.find(')')?;
        let link = Link::parse(&rest[..close]).ok()?;
        if link.noun != "item" {
            return None;
        }
        let tail = &rest[close + 1..];
        let (title, column) = text.rsplit_once(" · ")?;
        let mut parts: Vec<&str> = match tail.strip_prefix(" · ") {
            Some(t) => t.split(" · ").collect(),
            None if tail.is_empty() => Vec::new(),
            None => return None,
        };
        let date = match parts.last() {
            Some(d) if is_date(d) => parts.pop().map(str::to_owned),
            _ => None,
        };
        let author = (!parts.is_empty()).then(|| md_unescape(&parts.join(" · ")));
        Some(Citation {
            label: label.to_owned(),
            link,
            quote: md_unescape(quote),
            title: md_unescape(title),
            column: md_unescape(column),
            author,
            date,
        })
    }
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

/// 正文里宿主写的全部引用，按出现的先后。
pub fn citations(body: &str) -> Vec<Citation> {
    body.lines().filter_map(Citation::parse).collect()
}

/// 正文里用过的脚注标签（含你手写的）。
pub fn labels(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(i) = rest.find("[^") {
        rest = &rest[i + 2..];
        if let Some(end) = rest.find(']') {
            out.push(rest[..end].to_owned());
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str = "[^k3f7q]: 「第一段 40 克闷蒸 30 秒」[手冲咖啡：三段式注水 · 口播 00:42](usuyo://item/douyin-4plrcxk343onmyxibmrr672kga/asr@0193f2b0-1d2e-7f3a-8b4c-5d6e7f8a9b0c#t=42,48) · 咖啡日常 · 2026-10-01";

    #[test]
    fn spec_footnote_parses_into_fields() {
        let c = Citation::parse(LINE).unwrap();
        assert_eq!(c.label, "k3f7q");
        assert_eq!(c.item(), "douyin-4plrcxk343onmyxibmrr672kga");
        assert_eq!(c.link.layer.as_deref(), Some("asr"));
        assert_eq!(c.link.fragment.as_deref(), Some("t=42,48"));
        assert_eq!(c.quote, "第一段 40 克闷蒸 30 秒");
        assert_eq!(c.title, "手冲咖啡：三段式注水");
        assert_eq!(c.column, "口播 00:42");
        assert_eq!(c.author.as_deref(), Some("咖啡日常"));
        assert_eq!(c.date.as_deref(), Some("2026-10-01"));
        assert_eq!(c.render(), LINE);
    }

    #[test]
    fn missing_author_and_forgotten_source() {
        let c = Citation::parse("[^abcde]: 「x」[t · 原文](usuyo://item/forgotten) · 2026-10-01")
            .unwrap();
        assert!(c.forgotten());
        assert_eq!(c.author, None);
        assert_eq!(c.date.as_deref(), Some("2026-10-01"));
        assert!(Citation::parse("[^1]: 我自己写的脚注").is_none());
        assert_eq!(labels("a[^x1]b[^abcde]\n[^abcde]: …"), vec!["abcde", "x1"]);
    }
}
