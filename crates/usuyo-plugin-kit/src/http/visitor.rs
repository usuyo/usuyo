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

/// 宿主交来的访客身份：拼好的 Cookie，和取它的那个窗口的 User-Agent（`sites[].userAgent`，宿主记下了才有）。
/// 站点可能把访客 cookie 和取它时的浏览器对上，所以请求时用同一个 UA 和配它的请求头。
#[derive(Debug, Clone)]
pub struct Visitor {
    pub cookie: Cookie,
    pub user_agent: Option<String>,
}

impl Cookie {
    /// 这次调用的访客身份，拼成一份只发给 `hosts`、只走 https 的 Cookie（和账号的 cookie 一样限定）。
    /// 宿主没交（能力没列访客身份，或者还没存）时是 `None`。过期的跳过；放不进 Cookie 请求头的跳过，
    /// 在 stderr 记一行名字（不记值）。要取它时的 UA 用 [`Visitor::from_env`]。
    pub fn visitor<I, S>(hosts: I) -> Result<Option<Cookie>, VisitorError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Ok(Visitor::from_env(hosts)?.map(|v| v.cookie))
    }
}

impl Visitor {
    /// 和 [`Cookie::visitor`] 一样读 `USUYO_VISITOR`，再带上取它时的 UA。UA 放不进请求头时报错（不带它的值）。
    pub fn from_env<I, S>(hosts: I) -> Result<Option<Visitor>, VisitorError>
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
        let file = Credentials::from_json(&bytes).map_err(VisitorError)?;
        let line = header(&file, now)?;
        Ok(Some(Visitor {
            cookie: Cookie {
                value: line,
                hosts: Hosts::only(hosts),
            },
            user_agent: user_agent(&file)?,
        }))
    }
}

/// 文件里第一个写了 `userAgent` 的站点的 UA：只收可见的 ASCII 和空格。
fn user_agent(file: &Credentials) -> Result<Option<String>, VisitorError> {
    match file.sites.iter().find_map(|s| s.user_agent.as_deref()) {
        None => Ok(None),
        Some(ua) if !ua.is_empty() && ua.bytes().all(|b| (0x20..0x7f).contains(&b)) => {
            Ok(Some(ua.to_owned()))
        }
        Some(_) => Err(VisitorError(
            "the user agent cannot go in a User-Agent header".into(),
        )),
    }
}

/// 文件里没过期、放得进请求头的 cookie 拼成 `name=value; name=value`。
fn header(file: &Credentials, now: f64) -> Result<String, VisitorError> {
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

    fn parsed(bytes: &[u8]) -> Credentials {
        Credentials::from_json(bytes).unwrap()
    }

    #[test]
    fn the_window_user_agent_comes_along() {
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.5 Safari/605.1.15";
        let f = format!(
            r#"{{"sites": [{{"site": "www.site.test", "source": "visitor-window", "userAgent": "{ua}"}}], "storageState": {{"cookies": []}}}}"#
        );
        assert_eq!(
            user_agent(&parsed(f.as_bytes())).unwrap().as_deref(),
            Some(ua)
        );
        // 以前存下的没有 UA
        assert_eq!(user_agent(&parsed(&file(""))).unwrap(), None);
        let bad = r#"{"sites": [{"site": "s", "source": "visitor-window", "userAgent": "x\r\nHost: evil"}], "storageState": {"cookies": []}}"#;
        let err = user_agent(&parsed(bad.as_bytes())).unwrap_err().to_string();
        assert!(!err.contains("evil"), "{err}");
    }

    #[test]
    fn joins_unexpired_cookies() {
        let f = file(
            r#"{"name": "a1", "value": "x", "domain": ".site.test"},
               {"name": "old", "value": "y", "domain": ".site.test", "expires": 10},
               {"name": "webId", "value": "z", "domain": ".site.test", "expires": 5000}"#,
        );
        assert_eq!(header(&parsed(&f), 100.0).unwrap(), "a1=x; webId=z");
    }

    #[test]
    fn skips_what_cannot_be_a_header_and_never_quotes_values() {
        let bad = r#"{"name": "a1", "value": "sec;ret\r\nHost: evil", "domain": ".site.test"}"#;
        let f = file(&format!(
            r#"{bad}, {{"name": "webId", "value": "z", "domain": ".site.test"}}"#
        ));
        assert_eq!(header(&parsed(&f), 0.0).unwrap(), "webId=z");
        let err = header(&parsed(&file(bad)), 0.0).unwrap_err().to_string();
        assert!(!err.contains("sec"), "{err}");
        let err = Credentials::from_json(br#"{"sites": "very-secret"}"#).unwrap_err();
        assert!(!err.contains("very-secret"), "{err}");
        assert!(header(&parsed(&file("")), 0.0).is_err());
    }
}
