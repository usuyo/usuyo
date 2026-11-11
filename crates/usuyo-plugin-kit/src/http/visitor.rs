//! 宿主交来的访客身份（协议 §2.5.1、§6）：`USUYO_VISITOR` 指的文件，和 `USUYO_CREDENTIALS` 一个形状。
//! 访客身份不是账号；值只进 [`Cookie`]，不进日志、不进错误信息。

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use usuyo_plugin::protocol::Credentials;

use super::{Cookie, Hosts};

/// 文件最大多少：一个站点的访客 cookie 远小于这个数。
const FILE_BYTES: u64 = 1 << 20;

/// `USUYO_VISITOR` 指的文件读不了、形状不对，或者里面一个能放进 Cookie 请求头的都没有。消息里没有值。
#[derive(Debug, thiserror::Error)]
#[error("visitor identity: {0}")]
pub struct VisitorError(String);

impl Cookie {
    /// 这次调用的访客身份，拼成一份只发给 `hosts`、只走 https 的 Cookie（和账号的 cookie 一样限定）。
    /// 宿主没交（能力没列访客身份，或者还没存）时是 `None`。过期的跳过；放不进 Cookie 请求头的跳过，
    /// 在 stderr 记一行名字（不记值）。
    pub fn visitor<I, S>(hosts: I) -> Result<Option<Cookie>, VisitorError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let Some(path) = std::env::var_os("USUYO_VISITOR") else {
            return Ok(None);
        };
        let mut bytes = Vec::new();
        std::fs::File::open(&path)
            .and_then(|f| f.take(FILE_BYTES + 1).read_to_end(&mut bytes))
            .map_err(|e| VisitorError(format!("cannot read the file: {e}")))?;
        if bytes.len() as u64 > FILE_BYTES {
            return Err(VisitorError("the file is too large".into()));
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0.0, |d| d.as_secs_f64());
        header(&bytes, now).map(|line| {
            Some(Cookie {
                value: line,
                hosts: Hosts::only(hosts),
            })
        })
    }
}

/// 文件里没过期、放得进请求头的 cookie 拼成 `name=value; name=value`。
fn header(bytes: &[u8], now: f64) -> Result<String, VisitorError> {
    let file = Credentials::from_json(bytes).map_err(VisitorError)?;
    let mut pairs = Vec::new();
    for c in &file.storage_state.cookies {
        if c.expired_at(now) {
            continue;
        }
        if !sendable(&c.name, &c.value) {
            eprintln!(
                "visitor identity: skipped cookie `{}`: its name or value cannot go in a Cookie header",
                c.name.escape_debug()
            );
            continue;
        }
        pairs.push(format!("{}={}", c.name, c.value));
    }
    if pairs.is_empty() {
        return Err(VisitorError("no unexpired cookies in the file".into()));
    }
    Ok(pairs.join("; "))
}

/// RFC 6265 的 cookie-name 和 cookie-value：没有空白、控制字符、分号、逗号、引号、反斜杠。
fn sendable(name: &str, value: &str) -> bool {
    let token = |b: u8| b.is_ascii_graphic() && !b"()<>@,;:\\\"/[]?={}".contains(&b);
    let octet = |b: u8| b.is_ascii_graphic() && !b"\",;\\".contains(&b);
    !name.is_empty() && name.bytes().all(token) && value.bytes().all(octet)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(cookies: &str) -> Vec<u8> {
        format!(r#"{{"sites": [{{"site": "www.site.test", "source": "visitor-window"}}], "storageState": {{"cookies": [{cookies}]}}}}"#)
            .into_bytes()
    }

    #[test]
    fn joins_unexpired_cookies() {
        let f = file(
            r#"{"name": "a1", "value": "x", "domain": ".site.test"},
               {"name": "old", "value": "y", "domain": ".site.test", "expires": 10},
               {"name": "webId", "value": "z", "domain": ".site.test", "expires": 5000}"#,
        );
        assert_eq!(header(&f, 100.0).unwrap(), "a1=x; webId=z");
    }

    #[test]
    fn skips_what_cannot_be_a_header_and_never_quotes_values() {
        let bad = r#"{"name": "a1", "value": "sec;ret\r\nHost: evil", "domain": ".site.test"}"#;
        let f = file(&format!(
            r#"{bad}, {{"name": "webId", "value": "z", "domain": ".site.test"}}"#
        ));
        assert_eq!(header(&f, 0.0).unwrap(), "webId=z");
        let err = header(&file(bad), 0.0).unwrap_err().to_string();
        assert!(!err.contains("sec"), "{err}");
        let err = header(br#"{"sites": "very-secret"}"#, 0.0)
            .unwrap_err()
            .to_string();
        assert!(!err.contains("very-secret"), "{err}");
        assert!(header(&file(""), 0.0).is_err());
    }
}
