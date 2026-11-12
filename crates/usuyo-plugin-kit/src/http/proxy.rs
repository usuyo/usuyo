//! 经代理连出去：宿主照这台机器的设置（`network.proxy`）定好路线，经 `USUYO_PROXY` 交给插件。
//!
//! 信的是：宿主给的代理本身（只有它可以是本机地址），和它替 https 目标按名字查到的地址，因为 TLS 要对上
//! 这个名字的证书，内网服务出示不了；不信的是：要取的地址和明文 http 目标在代理那边会被解析成什么，
//! 所以明文 http 在本机查、只许公网答案，问代理要的是查过的数字地址，名字放在 `Host` 里。
//!
//! 名单、逐跳核对（每一跳照它自己的协议套这两条规矩，https 跳到 http 就按 http 查）、大小上限、超时、
//! cookie 只发给声明的主机和 https 都不变。两种协议都先挡掉一看就是本机、内网的名字和内网、保留段的数字地址；
//! https 的名字本机查出来是内网地址的也挡（本机解析器被改过、回的是错的公网地址，由代理去连，证书兜底）。
//! 证书和主机名的校验一直开着，插件没有办法关掉。

use std::fmt;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Mutex, OnceLock, PoisonError};

use url::Url;

use super::Error;

/// 代理说的是哪种话。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// HTTP 代理：对每个目标发 `CONNECT host:port`。
    Http,
    /// SOCKS5，主机名交给代理去查。
    Socks5,
}

/// 一个代理：`http://[用户:密码@]主机:端口` 或 `socks5://…`（`socks5h://` 同义）。
/// `Debug`、`Display` 只写协议、主机和端口，不写用户名和密码。
pub struct Proxy {
    kind: Kind,
    host: String,
    port: u16,
    user: Option<String>,
    password: Option<String>,
    /// `USUYO_PROXY` 写坏了时为 `Some`：每次请求都回这个错，不悄悄直连。
    broken: Option<String>,
}

impl Proxy {
    /// 读一个代理地址。只认 `http`、`socks5`、`socks5h`，要有主机，不许有路径、查询串。
    pub fn parse(text: &str) -> Result<Proxy, Error> {
        let bad = |why: &str| Error::Invalid(format!("proxy address: {why}"));
        let url = Url::parse(text.trim()).map_err(|_| bad("not a URL"))?;
        let kind = match url.scheme() {
            "http" => Kind::Http,
            "socks5" | "socks5h" => Kind::Socks5,
            _ => return Err(bad("the scheme is not http, socks5 or socks5h")),
        };
        let host = url
            .host_str()
            .filter(|h| !h.is_empty())
            .ok_or_else(|| bad("no host"))?;
        if !matches!(url.path(), "" | "/") || url.query().is_some() || url.fragment().is_some() {
            return Err(bad("a path, query or fragment is not allowed"));
        }
        let port = url
            .port()
            .unwrap_or(if kind == Kind::Http { 80 } else { 1080 });
        let decode = |s: &str| {
            percent_decode(s).ok_or_else(|| bad("the user name or password is not valid UTF-8"))
        };
        let user = (!url.username().is_empty())
            .then(|| decode(url.username()))
            .transpose()?;
        let password = url.password().map(decode).transpose()?;
        Ok(Proxy {
            kind,
            host: host
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_owned(),
            port,
            user,
            password,
            broken: None,
        })
    }

    /// 同 [`Proxy::parse`]，同一个地址只留一份、一直在（[`super::Policy`] 是 `Copy`，只能放引用）。
    /// 不同的地址最多是这台机器换过的几个代理。
    pub fn intern(text: &str) -> Result<&'static Proxy, Error> {
        static ALL: Mutex<Vec<(String, &'static Proxy)>> = Mutex::new(Vec::new());
        let mut all = ALL.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((_, p)) = all.iter().find(|(t, _)| t == text) {
            return Ok(p);
        }
        let p: &'static Proxy = Box::leak(Box::new(Proxy::parse(text)?));
        all.push((text.to_owned(), p));
        Ok(p)
    }

    /// 宿主经 `USUYO_PROXY` 交来的；没有就是直连。写坏了的也返回一个代理，用它的每个请求都报错。
    pub(super) fn from_env() -> Option<&'static Proxy> {
        static FROM_ENV: OnceLock<Option<&'static Proxy>> = OnceLock::new();
        *FROM_ENV.get_or_init(|| {
            let text = std::env::var("USUYO_PROXY").ok()?;
            Some(Proxy::intern(&text).unwrap_or_else(|e| {
                Box::leak(Box::new(Proxy {
                    kind: Kind::Http,
                    host: String::new(),
                    port: 0,
                    user: None,
                    password: None,
                    broken: Some(format!("USUYO_PROXY: {e}")),
                }))
            }))
        })
    }

    /// 能不能用；写坏了的回原因。
    pub(super) fn usable(&self) -> Result<(), Error> {
        match &self.broken {
            Some(why) => Err(Error::Invalid(why.clone())),
            None => Ok(()),
        }
    }

    pub(super) fn host(&self) -> &str {
        &self.host
    }

    /// 代理自己的地址：数字地址直接用，名字用系统 DNS 查（宿主给的，信它）。
    pub(super) fn addrs(&self) -> Result<Vec<SocketAddr>, Error> {
        if let Ok(ip) = self.host.parse::<std::net::IpAddr>() {
            return Ok(vec![SocketAddr::new(ip, self.port)]);
        }
        (self.host.as_str(), self.port)
            .to_socket_addrs()
            .map(Vec::from_iter)
            .map_err(Error::Lookup)
    }

    /// 给 ureq 的设置：目标的名字交给代理去查。
    pub(super) fn to_ureq(&self) -> Result<ureq::Proxy, Error> {
        let protocol = match self.kind {
            Kind::Http => ureq::ProxyProtocol::Http,
            Kind::Socks5 => ureq::ProxyProtocol::Socks5,
        };
        let host = if self.host.contains(':') {
            format!("[{}]", self.host)
        } else {
            self.host.clone()
        };
        let mut b = ureq::Proxy::builder(protocol)
            .host(&host)
            .port(self.port)
            .resolve_target(false);
        if let Some(u) = &self.user {
            b = b.username(u);
        }
        if let Some(p) = &self.password {
            b = b.password(p);
        }
        b.build()
            .map_err(|e| Error::Invalid(self.scrub(&format!("proxy: {e}"))))
    }

    /// 错误里去掉代理的用户名和密码。
    pub(super) fn scrub(&self, text: &str) -> String {
        let mut out = text.to_owned();
        for secret in [&self.password, &self.user].into_iter().flatten() {
            if !secret.is_empty() {
                out = out.replace(secret.as_str(), "<redacted>");
            }
        }
        out
    }

    /// 同 [`Proxy::scrub`]，作用在一个错误上。
    pub(super) fn scrub_error(&self, e: Error) -> Error {
        match e {
            Error::Network(m) => Error::Network(self.scrub(&m)),
            Error::Invalid(m) => Error::Invalid(self.scrub(&m)),
            Error::BadLocation(m) => Error::BadLocation(self.scrub(&m)),
            other => other,
        }
    }
}

impl fmt::Display for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.broken.is_some() {
            return f.write_str("<unusable proxy>");
        }
        let scheme = match self.kind {
            Kind::Http => "http",
            Kind::Socks5 => "socks5",
        };
        write!(f, "{scheme}://{}:{}", self.host, self.port)
    }
}

impl fmt::Debug for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Proxy({self})")
    }
}

/// `%xx` 解码；解出来不是 UTF-8 回 `None`。
fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = s.get(i + 1..i + 3)
            && let Ok(b) = u8::from_str_radix(hex, 16)
        {
            out.push(b);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_never_shows_credentials() {
        let p = Proxy::parse("http://alice:s%40cret@127.0.0.1:7890").unwrap();
        assert_eq!(
            (p.kind, p.host.as_str(), p.port),
            (Kind::Http, "127.0.0.1", 7890)
        );
        assert_eq!(p.password.as_deref(), Some("s@cret"));
        assert_eq!(p.to_string(), "http://127.0.0.1:7890");
        assert!(!format!("{p:?}").contains("cret"));
        assert_eq!(
            p.scrub("auth failed for alice with s@cret"),
            "auth failed for <redacted> with <redacted>"
        );
        let s = Proxy::parse("socks5h://[::1]:1080").unwrap();
        assert_eq!((s.kind, s.host.as_str()), (Kind::Socks5, "::1"));
        for bad in [
            "ftp://x:1",
            "http://",
            "http://x:1/path",
            "http://x:1/?q",
            "not a url",
        ] {
            assert!(Proxy::parse(bad).is_err(), "{bad}");
        }
        let a = Proxy::intern("socks5://127.0.0.1:1").unwrap();
        let b = Proxy::intern("socks5://127.0.0.1:1").unwrap();
        assert!(std::ptr::eq(a, b));
    }
}
