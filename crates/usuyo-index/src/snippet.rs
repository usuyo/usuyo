use std::ops::Range;

use crate::analyzer::{Mode, tokenize};
use crate::query::Marks;

/// 前文最多带几个字，整段最多几个字。
const LEAD_CHARS: usize = 30;
const MAX_CHARS: usize = 160;
const ELLIPSIS: &str = "…";

/// 命中附近的一小段原文。`highlights` 是 `text` 里的字节范围，落在字符边界上。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    pub text: String,
    pub highlights: Vec<Range<usize>>,
}

pub(crate) fn make(text: &str, marks: &Marks) -> Snippet {
    let mut ranges: Vec<Range<usize>> = Vec::new();
    for (mode, set) in [
        (Mode::Words, &marks.words),
        (Mode::Grams, &marks.grams),
        (Mode::Pinyin, &marks.pinyin),
    ] {
        if set.is_empty() {
            continue;
        }
        ranges.extend(
            tokenize(mode, text)
                .into_iter()
                .filter(|(t, _)| set.contains(t))
                .map(|(_, r)| r),
        );
    }
    let ranges = merge(ranges);

    let first = ranges.first().map_or(0, |r| r.start);
    let start = text[..first]
        .char_indices()
        .rev()
        .nth(LEAD_CHARS - 1)
        .map_or(0, |(i, _)| i);
    let end = text[start..]
        .char_indices()
        .nth(MAX_CHARS)
        .map_or(text.len(), |(i, _)| start + i);

    let lead = if start > 0 { ELLIPSIS } else { "" };
    let tail = if end < text.len() { ELLIPSIS } else { "" };
    let highlights = ranges
        .into_iter()
        .filter(|r| r.start < end)
        .map(|r| (r.start - start + lead.len())..(r.end.min(end) - start + lead.len()))
        .collect();
    Snippet {
        text: format!("{lead}{}{tail}", text[start..end].trim_end()),
        highlights,
    }
}

/// 排序并合并重叠或相邻的范围。
fn merge(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.sort_by_key(|r| (r.start, r.end));
    let mut out: Vec<Range<usize>> = Vec::new();
    for r in ranges {
        match out.last_mut() {
            Some(last) if r.start <= last.end => last.end = last.end.max(r.end),
            _ => out.push(r),
        }
    }
    out
}
