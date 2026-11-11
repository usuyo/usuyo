//! `usuyo://<名词>/<id>[/<层>[@<snapshotId>]][#片段]`（SPEC §0.1、§0.4）。

use std::fmt;

use crate::{Error, Result};

const NOUNS: &[&str] = &["item", "topic", "card", "draft", "changeset"];

/// 删掉的条目被引用时链接换成这个 id（SPEC §1.7「清干净」）。
pub const FORGOTTEN: &str = "forgotten";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub noun: String,
    pub id: String,
    /// 层可以带 `/`：`ocr/<imageKey>`。
    pub layer: Option<String>,
    pub snapshot: Option<String>,
    /// 不带 `#`，原样（已编码）。
    pub fragment: Option<String>,
}

impl Link {
    pub fn item(id: &str) -> Link {
        Link {
            noun: "item".into(),
            id: id.into(),
            layer: None,
            snapshot: None,
            fragment: None,
        }
    }

    pub fn parse(s: &str) -> Result<Link> {
        let bad = |why: &str| Error::InvalidArgs(format!("bad usuyo link `{s}`: {why}"));
        let rest = s.strip_prefix("usuyo://").ok_or_else(|| bad("scheme"))?;
        let (path, fragment) = match rest.split_once('#') {
            Some((p, f)) => (p, Some(f.to_owned())),
            None => (rest, None),
        };
        let mut parts = path.splitn(3, '/');
        let noun = parts.next().unwrap_or_default();
        if !NOUNS.contains(&noun) {
            return Err(bad("noun"));
        }
        let id = parts
            .next()
            .filter(|i| !i.is_empty())
            .ok_or_else(|| bad("id"))?;
        let (layer, snapshot) = match parts.next() {
            None => (None, None),
            Some(l) => match l.rsplit_once('@') {
                Some((l, snap)) if !l.is_empty() && !snap.is_empty() => {
                    (Some(l.to_owned()), Some(snap.to_owned()))
                }
                Some(_) => return Err(bad("layer@snapshot")),
                None if l.is_empty() => return Err(bad("layer")),
                None => (Some(l.to_owned()), None),
            },
        };
        if layer.is_none() && snapshot.is_some() {
            return Err(bad("snapshot without layer"));
        }
        Ok(Link {
            noun: noun.into(),
            id: id.into(),
            layer,
            snapshot,
            fragment,
        })
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "usuyo://{}/{}", self.noun, self.id)?;
        if let Some(l) = &self.layer {
            write!(f, "/{l}")?;
            if let Some(s) = &self.snapshot {
                write!(f, "@{s}")?;
            }
        }
        if let Some(fr) = &self.fragment {
            write!(f, "#{fr}")?;
        }
        Ok(())
    }
}

/// 必带 `@<snapshotId>` 的层：时间类、评论、画面（SPEC §0.4）。
pub fn layer_needs_snapshot(layer: &str) -> bool {
    matches!(layer, "asr" | "video" | "audio" | "comments" | "vision")
}

/// 层的栏名，给链接文字用（SPEC §1.2.3 的表）。
pub fn column_name(layer: &str) -> &'static str {
    match layer.split('/').next().unwrap_or_default() {
        "text" => "原文",
        "ocr" | "image" => "图里的字",
        "asr" | "video" | "audio" => "口播",
        "vision" => "画面",
        "summary" => "AI 摘要",
        "notes" => "你的笔记",
        "comments" => "评论",
        _ => "原文",
    }
}

/// 链接文字里的「栏 位置」：有时间片段就带位置。
pub fn column_label(layer: &str, fragment: Option<&str>) -> String {
    let name = column_name(layer);
    match fragment.and_then(parse_time_fragment) {
        Some((start, _)) => format!("{name} {}", crate::time::format_position(start)),
        None => name.to_owned(),
    }
}

fn format_sec(sec: f64) -> String {
    let s = format!("{:.3}", sec);
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// `t=102.5` 或 `t=102.5,110`（Media Fragments 1.0），秒最多 3 位小数。
pub fn time_fragment(start: f64, end: Option<f64>) -> String {
    match end {
        Some(e) => format!("t={},{}", format_sec(start), format_sec(e)),
        None => format!("t={}", format_sec(start)),
    }
}

pub fn parse_time_fragment(fragment: &str) -> Option<(f64, Option<f64>)> {
    let v = fragment.strip_prefix("t=")?;
    let (a, b) = match v.split_once(',') {
        Some((a, b)) => (a, Some(b)),
        None => (v, None),
    };
    let start = a.parse().ok()?;
    let end = match b {
        Some(b) => Some(b.parse().ok()?),
        None => None,
    };
    Some((start, end))
}

/// 原话部分的编码：ASCII 字母数字以外（连同 `-`、`,`、`&`）都百分号编码。
pub fn encode_text_part(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// 原话超过这么多字时链接里写 `开头,结尾`。
const TEXT_DIRECTIVE_MAX: usize = 32;
/// `开头,结尾` 各取几个字。
const TEXT_DIRECTIVE_EDGE: usize = 10;

/// `:~:text=[前缀-,]原话[,-后缀]`（URL Fragment Text Directives）。
pub fn text_fragment(prefix: Option<&str>, quote: &str, suffix: Option<&str>) -> String {
    let mut out = String::from(":~:text=");
    if let Some(p) = prefix {
        out.push_str(&encode_text_part(p));
        out.push_str("-,");
    }
    let chars: Vec<char> = quote.chars().collect();
    if chars.len() > TEXT_DIRECTIVE_MAX {
        let start: String = chars[..TEXT_DIRECTIVE_EDGE].iter().collect();
        let end: String = chars[chars.len() - TEXT_DIRECTIVE_EDGE..].iter().collect();
        out.push_str(&encode_text_part(&start));
        out.push(',');
        out.push_str(&encode_text_part(&end));
    } else {
        out.push_str(&encode_text_part(quote));
    }
    if let Some(s) = suffix {
        out.push_str(",-");
        out.push_str(&encode_text_part(s));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_with_layer_snapshot_fragment() {
        let s = "usuyo://item/douyin-4plrcxk343onmyxibmrr672kga/asr@0193f2b0-1d2e-7f3a-8b4c-5d6e7f8a9b0c#t=42,48";
        let l = Link::parse(s).unwrap();
        assert_eq!(l.layer.as_deref(), Some("asr"));
        assert_eq!(
            l.snapshot.as_deref(),
            Some("0193f2b0-1d2e-7f3a-8b4c-5d6e7f8a9b0c")
        );
        assert_eq!(
            parse_time_fragment(l.fragment.as_deref().unwrap()),
            Some((42.0, Some(48.0)))
        );
        assert_eq!(l.to_string(), s);
        let ocr =
            Link::parse("usuyo://item/x-aaaaaaaaaaaaaaaaaaaaaaaaaa/ocr/0123456789abcdef").unwrap();
        assert_eq!(ocr.layer.as_deref(), Some("ocr/0123456789abcdef"));
        assert_eq!(
            Link::parse("usuyo://topic/abc#^k3f7q")
                .unwrap()
                .fragment
                .as_deref(),
            Some("^k3f7q")
        );
        assert!(Link::parse("usuyo://nope/x").is_err());
        assert!(Link::parse("usuyo://item/").is_err());
    }

    #[test]
    fn spec_text_directive_example() {
        assert_eq!(
            text_fragment(Some("40 克"), "闷蒸 30 秒", None),
            ":~:text=40%20%E5%85%8B-,%E9%97%B7%E8%92%B8%2030%20%E7%A7%92"
        );
        let long = "一".repeat(40);
        assert_eq!(text_fragment(None, &long, None).matches(',').count(), 1);
    }

    #[test]
    fn times_and_labels() {
        assert_eq!(time_fragment(102.5, None), "t=102.5");
        assert_eq!(time_fragment(42.0, Some(48.0)), "t=42,48");
        assert_eq!(column_label("asr", Some("t=42,48")), "口播 00:42");
        assert_eq!(column_label("text", Some(":~:text=a")), "原文");
    }
}
