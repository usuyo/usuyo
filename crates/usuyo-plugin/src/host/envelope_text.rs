//! 信封里插件给的字，在解码处收一次口：一行的字段截到上限、去掉控制字符。
//! 正文（`text`）是平台原文，照收到的存，不改。
//! 这些字会进 manifest、地图、`jobs.db` 和日志；往里走的代码不再查。

use crate::protocol::{Author, CallFailure, FetchOutput, ListPage, MAX_PAUSE, SourceInfo};

/// 标题、作者、原因、提示这类一行的字段最多多少字节。
const LINE_BYTES: usize = 1024;
/// 平台标签：最多几个，每个最多多少字节。
const TAGS: usize = 64;
const TAG_BYTES: usize = 100;

/// 一行：控制字符（含换行）换成空格，截到 `max` 字节，不切开字符。
fn line(s: &mut String, max: usize) {
    if s.chars().any(char::is_control) {
        *s = s
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect();
    }
    if s.len() > max {
        let mut end = max;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        s.truncate(end);
    }
}

fn opt_line(s: &mut Option<String>) {
    if let Some(s) = s {
        line(s, LINE_BYTES);
    }
}

fn author(a: &mut Option<Author>) {
    if let Some(a) = a {
        line(&mut a.name, LINE_BYTES);
        opt_line(&mut a.platform_id);
        opt_line(&mut a.url);
    }
}

/// 插件报的错。`retryAfter` 也在这里封顶：一个页面给的 `Retry-After` 不能让取内容停很久。
pub(super) fn failure(f: &mut CallFailure) {
    line(&mut f.message, LINE_BYTES);
    opt_line(&mut f.hint);
    opt_line(&mut f.site);
    if let Some(secs) = &mut f.retry_after {
        *secs = (*secs).min(MAX_PAUSE.as_secs());
    }
}

pub(super) fn fetch(o: &mut FetchOutput) {
    opt_line(&mut o.title);
    author(&mut o.author);
    opt_line(&mut o.missing_reason);
    opt_line(&mut o.evidence.site);
    o.platform_tags.truncate(TAGS);
    for t in &mut o.platform_tags {
        line(t, TAG_BYTES);
    }
}

pub(super) fn list(p: &mut ListPage) {
    for item in &mut p.items {
        opt_line(&mut item.title);
        author(&mut item.author);
    }
}

pub(super) fn sources(s: &mut [SourceInfo]) {
    for source in s {
        line(&mut source.title, LINE_BYTES);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn cuts_on_a_char_boundary() {
        let mut s = "标".repeat(400);
        line(&mut s, 1024);
        assert_eq!(s.len(), 1023);
        let mut s = "a\r\nb\u{1b}".to_owned();
        line(&mut s, 1024);
        assert_eq!(s, "a  b ");
    }

    #[test]
    fn retry_after_is_capped() {
        let mut f = CallFailure::new(crate::protocol::ErrorCode::RateLimited, "429");
        f.retry_after = Some(u64::MAX);
        failure(&mut f);
        assert_eq!(f.retry_after, Some(MAX_PAUSE.as_secs()));
        assert!(MAX_PAUSE <= Duration::from_secs(3600));
    }
}
