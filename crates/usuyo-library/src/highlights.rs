//! `highlights.md`（SPEC §1.4.2）：一行一条
//! `- 「<原话>」[<栏 位置>](usuyo://item/<本条 id>/<层>@<snapshotId>#<片段>) ^<块 id>`。

use std::path::Path;

use crate::id::is_block_id;
use crate::link::Link;
use crate::text::{md_inline, md_unescape, take_escaped};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Highlight {
    pub quote: String,
    /// 链接文字「栏 位置」。
    pub label: String,
    pub link: Link,
    pub id: String,
}

impl Highlight {
    pub fn render(&self) -> String {
        format!(
            "- 「{}」[{}]({}) ^{}",
            md_inline(&self.quote),
            md_inline(&self.label),
            self.link,
            self.id
        )
    }

    pub fn parse(line: &str) -> Option<Highlight> {
        let rest = line.strip_prefix("- 「")?;
        let (quote, rest) = take_escaped(rest, "」[")?;
        let (label, rest) = take_escaped(rest, "](")?;
        let (link, id) = rest.rsplit_once(") ^")?;
        if !is_block_id(id) {
            return None;
        }
        let link = Link::parse(link).ok()?;
        if link.noun != "item" || link.layer.is_none() {
            return None;
        }
        Some(Highlight {
            quote: md_unescape(quote),
            label: md_unescape(label),
            link,
            id: id.to_owned(),
        })
    }
}

/// 整个文件；有一行不合写法就算写坏了。
pub fn parse_file(path: &Path, text: &str) -> Result<Vec<Highlight>> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let h = Highlight::parse(line).ok_or_else(|| {
            Error::malformed(path, format!("highlight line {} does not parse", i + 1))
        })?;
        out.push(h);
    }
    Ok(out)
}

pub fn render_file(items: &[Highlight]) -> String {
    let mut s = String::new();
    for h in items {
        s.push_str(&h.render());
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str = "- 「第一段 40 克闷蒸 30 秒」[口播 00:42](usuyo://item/douyin-4plrcxk343onmyxibmrr672kga/asr@0193f2b0-1d2e-7f3a-8b4c-5d6e7f8a9b0c#t=42,48) ^h7k2m";

    #[test]
    fn spec_line_round_trips() {
        let h = Highlight::parse(LINE).unwrap();
        assert_eq!(h.quote, "第一段 40 克闷蒸 30 秒");
        assert_eq!(h.label, "口播 00:42");
        assert_eq!(h.id, "h7k2m");
        assert_eq!(h.render(), LINE);
    }

    #[test]
    fn escaped_quote_with_brackets_round_trips() {
        let h = Highlight {
            quote: "带」[括号](和) \\ 反斜杠".into(),
            label: "原文".into(),
            link: Link::parse("usuyo://item/web-aaaaaaaaaaaaaaaaaaaaaaaaaa/text#:~:text=x")
                .unwrap(),
            id: "abcde".into(),
        };
        assert_eq!(Highlight::parse(&h.render()).unwrap(), h);
        assert!(parse_file(Path::new("h"), "- 不是划线\n").is_err());
        assert!(
            parse_file(Path::new("h"), &format!("{LINE}\n{LINE}\n"))
                .unwrap()
                .len()
                == 2
        );
    }
}
