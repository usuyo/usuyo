//! 日志里不留能用的秘密：网址的查询串（分享链接常带 `xsec_token` 之类、媒体地址带 `k`）
//! 和 `Bearer` 后面的令牌，写出去以前换成 `…`。平台的链接、插件报的错都会进日志。

use std::io::{self, Write};

use tracing_subscriber::fmt::MakeWriter;

/// 每条日志先攒在自己这里，整条洗过再写到 stderr。
pub struct Redacted;

impl<'a> MakeWriter<'a> for Redacted {
    type Writer = Line;

    fn make_writer(&'a self) -> Line {
        Line(Vec::new())
    }
}

pub struct Line(Vec<u8>);

impl Write for Line {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Line {
    fn drop(&mut self) {
        let text = redact(&String::from_utf8_lossy(&self.0));
        // 日志写不出去也没有别处可报。
        let _ = io::stderr().write_all(text.as_bytes());
    }
}

/// 网址到哪儿算完：空白、引号、括号。
fn url_end(c: char) -> bool {
    c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | ')' | ']' | '`' | '\u{1b}')
}

pub fn redact(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    loop {
        let url = rest.find("://");
        let bearer = rest.find("Bearer ");
        match (url, bearer) {
            (Some(u), b) if b.is_none_or(|b| u < b) => {
                let start = u + 3;
                let len = rest[start..].find(url_end).unwrap_or(rest.len() - start);
                let whole = &rest[start..start + len];
                out.push_str(&rest[..start]);
                match whole.find(['?', '#']) {
                    Some(q) => {
                        out.push_str(&whole[..q + 1]);
                        out.push('…');
                    }
                    None => out.push_str(whole),
                }
                rest = &rest[start + len..];
            }
            (_, Some(b)) => {
                let start = b + "Bearer ".len();
                let len = rest[start..].find(url_end).unwrap_or(rest.len() - start);
                out.push_str(&rest[..start]);
                out.push('…');
                rest = &rest[start + len..];
            }
            _ => {
                out.push_str(rest);
                return out;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::redact;

    #[test]
    fn strips_queries_and_bearer_tokens() {
        assert_eq!(
            redact(
                "fetch failed url=https://www.xiaohongshu.com/explore/abc?xsec_token=SECRET&x=1 next"
            ),
            "fetch failed url=https://www.xiaohongshu.com/explore/abc?… next"
        );
        assert_eq!(
            redact(r#"error="GET http://127.0.0.1:47615/media/a/image/1?k=SECRET""#),
            r#"error="GET http://127.0.0.1:47615/media/a/image/1?…""#
        );
        assert_eq!(
            redact("Authorization: Bearer SECRET\n"),
            "Authorization: Bearer …\n"
        );
        assert_eq!(
            redact("see https://a.test/x#frag"),
            "see https://a.test/x#…"
        );
        assert_eq!(
            redact("listening on http://127.0.0.1:47615\n"),
            "listening on http://127.0.0.1:47615\n"
        );
        assert_eq!(
            redact("闷蒸 https://例子.test/路径?q=秘密 完"),
            "闷蒸 https://例子.test/路径?… 完"
        );
        assert!(!redact("a https://x.test/?t=SECRET b Bearer SECRET").contains("SECRET"));
    }
}
