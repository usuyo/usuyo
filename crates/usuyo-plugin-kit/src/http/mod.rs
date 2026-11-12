//! 插件发请求的客户端。每一跳（包括跳转去的地址）都先过三道检查再连：
//! 主机在调用方给的名单里；解析出的地址全是公网；连接只连检查过的那些地址，不再查第二次 DNS。
//! 跳转在这里一跳一跳地跟，默认不跟；不读环境变量里的代理，没有 cookie 罐，
//! cookie 只发给调用方指定的主机。

mod address;
mod proxy;
mod visitor;

use std::fmt;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use ureq::config::Config as AgentConfig;
use ureq::http::Uri;
use ureq::unversioned::resolver::{ResolvedSocketAddrs, Resolver};
use ureq::unversioned::transport::{DefaultConnector, NextTimeout};
use url::Url;

use address::Refusal;
pub use address::{Lookup, Policy};
pub use proxy::Proxy;
pub use visitor::VisitorError;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(15);
/// 没有另给 `timeout` 时，一次请求连同它的所有跳转的总时长。
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 不是 http(s)、带账号密码、没有主机。
    #[error("{0}")]
    Invalid(String),
    /// 主机不在名单里；`first` 表示是请求的地址本身，不是跳转目标。
    #[error("`{host}` is not on the plugin's host list")]
    NotAllowed { host: String, first: bool },
    /// 地址不是公网；`first` 同上。
    #[error("{why}")]
    NotPublic { why: String, first: bool },
    #[error("DNS lookup: {0}")]
    Lookup(std::io::Error),
    #[error("more than {0} redirects")]
    TooManyRedirects(u32),
    #[error("bad redirect location {0}")]
    BadLocation(String),
    /// cookie 只能经 [`Request::cookie`] 带上，好限定发给哪些主机。
    #[error("a Cookie header must be attached with Request::cookie")]
    CookieHeader,
    #[error("response is larger than {0} bytes")]
    TooLarge(u64),
    #[error("timed out")]
    Timeout,
    #[error("network: {0}")]
    Network(String),
}

impl Error {
    /// 是不是检查没过、根本没有连出去。
    pub fn is_refusal(&self) -> bool {
        matches!(
            self,
            Error::Invalid(_) | Error::NotAllowed { .. } | Error::NotPublic { .. }
        )
    }
}

fn transport(e: ureq::Error) -> Error {
    match e {
        ureq::Error::Timeout(_) => Error::Timeout,
        ureq::Error::BadUri(why) => Error::Invalid(why),
        ureq::Error::BodyExceedsLimit(limit) => Error::TooLarge(limit),
        other => Error::Network(other.to_string()),
    }
}

/// 能连的主机名单。`.hdslb.com` 这样以点开头的项只匹配它的子域，不含它自己；
/// 别的项按整个主机名匹配。只比主机名，不比端口。
#[derive(Debug, Clone)]
pub struct Hosts(Option<Vec<String>>);

impl Hosts {
    /// 不限主机，只看地址是不是公网。给不分站点的插件（例如通用网页）用。
    pub fn any() -> Self {
        Hosts(None)
    }

    pub fn only<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Hosts(Some(
            names
                .into_iter()
                .map(|n| n.as_ref().to_ascii_lowercase())
                .collect(),
        ))
    }

    fn allows(&self, host: &str) -> bool {
        let Some(names) = &self.0 else {
            return true;
        };
        let host = host.to_ascii_lowercase();
        names.iter().any(|name| {
            if name.starts_with('.') {
                host.len() > name.len() && host.ends_with(name.as_str())
            } else {
                host == *name
            }
        })
    }
}

/// 一份 Cookie 请求头的值，和能收到它的主机。别的主机的那一跳不带它。值不进 `Debug`。
#[derive(Clone)]
pub struct Cookie {
    value: String,
    hosts: Hosts,
}

impl Cookie {
    pub fn new<I, S>(value: impl Into<String>, hosts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Cookie {
            value: value.into(),
            hosts: Hosts::only(hosts),
        }
    }
}

impl fmt::Debug for Cookie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Cookie(<redacted>)")
    }
}

/// 一个插件的客户端：固定的 User-Agent、主机名单和地址策略。
#[derive(Debug, Clone)]
pub struct Client {
    user_agent: String,
    hosts: Hosts,
    policy: Policy,
}

impl Client {
    /// 用严格的地址策略：只连公网地址；宿主说明了网络环境的照它来（见 [`Policy::from_host`]）。
    pub fn new(user_agent: impl Into<String>, hosts: Hosts) -> Self {
        Client {
            user_agent: user_agent.into(),
            hosts,
            policy: Policy::from_host(),
        }
    }

    /// 换地址策略。只有测试需要：让本机测试服务算「公网」。
    pub fn with_policy(self, policy: Policy) -> Self {
        Client { policy, ..self }
    }

    pub fn get(&self, url: &Url) -> Request<'_> {
        self.request(url, None)
    }

    pub fn post(&self, url: &Url, body: impl Into<Vec<u8>>) -> Request<'_> {
        self.request(url, Some(body.into()))
    }

    fn request(&self, url: &Url, body: Option<Vec<u8>>) -> Request<'_> {
        Request {
            client: self,
            url: url.clone(),
            body,
            headers: Vec::new(),
            cookie: None,
            timeout: DEFAULT_TIMEOUT,
            max_redirects: 0,
            private_first_hop: false,
        }
    }

    /// 一跳能不能连，能连就回检查过的地址。查域名最多等 `left` 和 [`LOOKUP_TIMEOUT`] 里短的那个：
    /// 查询放在线程里，等不到就不等了。
    fn admit(
        &self,
        url: &Url,
        private_ok: bool,
        first: bool,
        left: Duration,
    ) -> Result<Vec<SocketAddr>, Error> {
        if let Some(host) = url.host_str()
            && !self.hosts.allows(host)
        {
            return Err(Error::NotAllowed {
                host: host.to_owned(),
                first,
            });
        }
        vet(self.policy, url, private_ok, first, left)
    }
}

impl Policy {
    /// 一个地址能不能连：http(s)、不带账号密码，解析出的地址照这个策略都能连（只连公网，假 IP 环境照
    /// [`Policy::fake_ip_names`]）；能连就回查过的地址，调用方只连这些地址。查域名最多等 `left` 和 15 秒里短的那个。
    /// 不跟跳转：自己跟跳转的调用方每一跳问一次。
    ///
    /// 调用方：内核下载模型（`usuyo-core` 的 `models`），它要断流超时和续传，不用 [`Client`]。
    pub fn vet(&self, url: &Url, left: Duration) -> Result<Vec<SocketAddr>, Error> {
        vet(*self, url, false, true, left)
    }
}

/// 查一跳：查询放在线程里，等不到就不等了。
fn vet(
    policy: Policy,
    url: &Url,
    private_ok: bool,
    first: bool,
    left: Duration,
) -> Result<Vec<SocketAddr>, Error> {
    let (tx, rx) = mpsc::sync_channel(1);
    let target = url.clone();
    std::thread::spawn(move || {
        tx.send(address::admit(&target, private_ok, &policy)).ok();
    });
    let vetted = rx.recv_timeout(left.min(LOOKUP_TIMEOUT)).map_err(|_| {
        Error::Lookup(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "DNS lookup timed out",
        ))
    })?;
    vetted.map_err(|r| match r {
        Refusal::Invalid(why) => Error::Invalid(why),
        Refusal::NotPublic(why) => Error::NotPublic { why, first },
        Refusal::Lookup(e) => Error::Lookup(e),
    })
}

pub struct Request<'c> {
    client: &'c Client,
    url: Url,
    /// 有体就是 POST。
    body: Option<Vec<u8>>,
    headers: Vec<(String, String)>,
    cookie: Option<Cookie>,
    timeout: Duration,
    max_redirects: u32,
    private_first_hop: bool,
}

impl Request<'_> {
    /// 请求头。跟跳转时只在和第一跳同一个主机、没有从 https 降到 http 的那几跳全带，
    /// 别的跳只带 `Accept` 和 `Accept-Language`。cookie 不能从这里带，见 [`Request::cookie`]。
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    /// 只在主机属于这份 cookie 的那几跳带上。
    pub fn cookie(mut self, cookie: &Cookie) -> Self {
        self.cookie = Some(cookie.clone());
        self
    }

    /// 连同所有跳转、读完响应体的总时长。
    pub fn timeout(mut self, total: Duration) -> Self {
        self.timeout = total;
        self
    }

    /// 最多跟几次跳转，每一跳都重新检查。不调用就不跟：3xx 原样交回。
    pub fn follow_redirects(mut self, max: u32) -> Self {
        self.max_redirects = max;
        self
    }

    /// 请求的地址本身可以是本机或内网地址；跳转目标照样只连公网。
    /// 只给用户自己在设置里打开的那一种情况用（例如通用网页插件的「允许取内网网页」）。
    pub fn allow_private_first_hop(mut self, allow: bool) -> Self {
        self.private_first_hop = allow;
        self
    }

    pub fn send(self) -> Result<Response, Error> {
        if self
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("cookie"))
        {
            return Err(Error::CookieHeader);
        }
        let proxy = self.client.policy.proxy;
        if let Some(p) = proxy {
            p.usable()?;
        }
        let deadline = Instant::now() + self.timeout;
        let first_url = self.url.clone();
        let hop_safe: Vec<(String, String)> = self
            .headers
            .iter()
            .filter(|(name, _)| {
                HOP_SAFE_HEADERS
                    .iter()
                    .any(|s| name.eq_ignore_ascii_case(s))
            })
            .cloned()
            .collect();
        let mut url = self.url;
        let mut body = self.body;
        for hop in 0..=self.max_redirects {
            let first = hop == 0;
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err(Error::Timeout);
            }
            let addrs = self
                .client
                .admit(&url, self.private_first_hop && first, first, left)?;
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err(Error::Timeout);
            }
            let host = url.host_str().unwrap_or_default();
            // 经代理的明文 http：问代理要的是本机查过的那个数字地址，原来的名字放在 `Host` 里（见 `proxy` 模块）
            let mut wire = url.clone();
            let mut host_header = None;
            let agent = match proxy {
                Some(p) => {
                    if let Some(first) = addrs.first() {
                        host_header = Some(match url.port() {
                            Some(port) => format!("{host}:{port}"),
                            None => host.to_owned(),
                        });
                        wire.set_ip_host(first.ip())
                            .map_err(|()| Error::Invalid("cannot address the vetted IP".into()))?;
                    }
                    proxied_agent(p, left, &self.client.user_agent)?
                }
                None => pinned_agent(host, addrs, left, &self.client.user_agent),
            };
            // 登录信息只走 https：同一个主机跳到明文 http 也不带。
            let encrypted = url.scheme() == "https" || self.client.policy.cookies_over_http;
            let cookie = self
                .cookie
                .as_ref()
                .filter(|c| encrypted && c.hosts.allows(host))
                .map(|c| c.value.as_str());
            let headers = if keeps_caller_headers(&first_url, &url) {
                &self.headers
            } else {
                &hop_safe
            };
            let host_header = host_header.as_deref();
            let response = match &body {
                None => with_headers(agent.get(wire.as_str()), headers, cookie, host_header).call(),
                Some(bytes) => {
                    with_headers(agent.post(wire.as_str()), headers, cookie, host_header)
                        .send(bytes.as_slice())
                }
            }
            .map_err(|e| match proxy {
                Some(p) => p.scrub_error(transport(e)),
                None => transport(e),
            })?;
            let status = response.status().as_u16();
            let redirect = self.max_redirects > 0 && matches!(status, 301 | 302 | 303 | 307 | 308);
            let Some(location) = response.headers().get("location").filter(|_| redirect) else {
                return Ok(Response {
                    url,
                    inner: response,
                });
            };
            let next = location
                .to_str()
                .ok()
                .and_then(|l| url.join(l).ok())
                .ok_or_else(|| Error::BadLocation(format!("{location:?}")))?;
            // 照浏览器：303，和 POST 收到的 301、302，下一跳改成不带体的 GET
            if status == 303 || matches!(status, 301 | 302) {
                body = None;
            }
            url = next;
        }
        Err(Error::TooManyRedirects(self.max_redirects))
    }
}

/// 跳转里，调用方给的请求头还能不能全带：还在第一跳的主机上，而且没有从 https 降到 http。
/// 别的跳只带 [`HOP_SAFE_HEADERS`]，免得 Referer、令牌一类的头跟着跳到别的主机或明文上。
fn keeps_caller_headers(first: &Url, hop: &Url) -> bool {
    hop.host_str() == first.host_str() && !(first.scheme() == "https" && hop.scheme() == "http")
}

/// 换了主机或降到明文的那一跳也照带的请求头：只说要什么格式，不带来历和凭据。
const HOP_SAFE_HEADERS: [&str; 2] = ["accept", "accept-language"];

/// `host`：经代理连明文 http 时，地址栏里是查过的数字地址，`Host` 写原来的名字。
fn with_headers<B>(
    mut request: ureq::RequestBuilder<B>,
    headers: &[(String, String)],
    cookie: Option<&str>,
    host: Option<&str>,
) -> ureq::RequestBuilder<B> {
    if let Some(host) = host {
        request = request.header("Host", host);
    }
    for (name, value) in headers {
        request = request.header(name, value);
    }
    if let Some(cookie) = cookie {
        request = request.header("Cookie", cookie);
    }
    request
}

/// 一跳用的底层客户端：只连检查过的地址，不自己跟跳转，不走环境变量里的代理。
/// 每跳新建一个，所以上一跳的 `Set-Cookie` 带不到下一跳。
fn pinned_agent(
    host: &str,
    addrs: Vec<SocketAddr>,
    timeout: Duration,
    user_agent: &str,
) -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .timeout_connect(Some(CONNECT_TIMEOUT.min(timeout)))
        .max_redirects(0)
        .user_agent(user_agent)
        .http_status_as_error(false)
        .proxy(None)
        .build();
    let pinned = Pinned {
        host: host.to_owned(),
        addrs,
    };
    ureq::Agent::with_parts(config, DefaultConnector::default(), pinned)
}

/// 经代理的那一跳：底层客户端只认代理的主机，解析出的是代理自己的地址。
fn proxied_agent(
    proxy: &'static Proxy,
    timeout: Duration,
    user_agent: &str,
) -> Result<ureq::Agent, Error> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .timeout_connect(Some(CONNECT_TIMEOUT.min(timeout)))
        .max_redirects(0)
        .user_agent(user_agent)
        .http_status_as_error(false)
        .proxy(Some(proxy.to_ureq()?))
        .build();
    let pinned = Pinned {
        host: proxy.host().to_owned(),
        addrs: proxy.addrs()?,
    };
    Ok(ureq::Agent::with_parts(
        config,
        DefaultConnector::default(),
        pinned,
    ))
}

/// 钉住地址的解析器：只认这一跳的主机，回的就是检查过的那些地址。
#[derive(Debug)]
struct Pinned {
    host: String,
    addrs: Vec<SocketAddr>,
}

impl Resolver for Pinned {
    fn resolve(
        &self,
        uri: &Uri,
        _: &AgentConfig,
        _: NextTimeout,
    ) -> Result<ResolvedSocketAddrs, ureq::Error> {
        if uri.host() != Some(self.host.as_str()) {
            return Err(ureq::Error::HostNotFound);
        }
        // ureq 最多收 16 个地址，多出来的不要
        let mut out = self.empty();
        for addr in &self.addrs {
            if out.try_push(*addr).is_err() {
                break;
            }
        }
        Ok(out)
    }
}

/// 最后一跳的响应。响应体只能带上限读。
#[derive(Debug)]
pub struct Response {
    url: Url,
    inner: ureq::http::Response<ureq::Body>,
}

impl Response {
    pub fn status(&self) -> u16 {
        self.inner.status().as_u16()
    }

    /// 最后一跳的地址。
    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.inner.headers().get(name).and_then(|v| v.to_str().ok())
    }

    /// 读完整个响应体，解压后超过 `limit` 字节回 [`Error::TooLarge`]，读到上限就停。
    pub fn read_to_vec(&mut self, limit: u64) -> Result<Vec<u8>, Error> {
        let mut reader = Capped::new(self.inner.body_mut().with_config().reader(), limit);
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).map_err(|e| {
            let kind = e.kind();
            if kind == std::io::ErrorKind::FileTooLarge {
                return Error::TooLarge(limit);
            }
            // ureq 把读体时的错误包在 io::Error 里，拆出来按发请求时的规则分
            match e.into_inner().map(|inner| inner.downcast::<ureq::Error>()) {
                Some(Ok(ureq)) => transport(*ureq),
                _ if kind == std::io::ErrorKind::TimedOut => Error::Timeout,
                Some(Err(other)) => Error::Network(other.to_string()),
                None => Error::Network(format!("reading the body: {kind}")),
            }
        })?;
        Ok(bytes)
    }

    /// 边读边处理大文件；解压后读出的字节超过 `limit` 时报读错误，错误的类别是
    /// `io::ErrorKind::FileTooLarge`，调用方凭它和网络中断分开。
    pub fn into_reader(self, limit: u64) -> impl Read + Send + 'static {
        Capped::new(self.inner.into_body().into_with_config().reader(), limit)
    }
}

/// 读响应体的唯一途径：按解压后的字节数计，到上限就报错，不再往下读。
/// ureq 自己的上限数的是压缩前的字节，挡不住一小段 gzip 解出几个 GiB，所以这里不用它。
struct Capped<R> {
    inner: R,
    left: u64,
    limit: u64,
}

impl<R> Capped<R> {
    fn new(inner: R, limit: u64) -> Self {
        Capped {
            inner,
            left: limit,
            limit,
        }
    }
}

impl<R: Read> Read for Capped<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // 最多比剩下的多读一个字节：够看出超了，又不多解压
        let room = usize::try_from(self.left.saturating_add(1))
            .unwrap_or(usize::MAX)
            .min(buf.len());
        let n = self.inner.read(&mut buf[..room])?;
        self.left = self.left.checked_sub(n as u64).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::FileTooLarge,
                Error::TooLarge(self.limit),
            )
        })?;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, BufRead, BufReader, Write};
    use std::net::{IpAddr, TcpListener};
    use std::sync::{Arc, Mutex};

    use super::*;

    const UA: &str = "usuyo-plugin-kit-test";

    /// 收到的请求：方法、路径、Cookie 头、全部请求头（小写、去掉行尾）。
    type Seen = Arc<Mutex<Vec<(String, String, Option<String>, Vec<String>)>>>;

    /// 本机 HTTP 服务。`/to/<地址>` 回 302 到那个地址，`/see-other` 回 303 到 `/end`，
    /// `/big` 回 1 KiB，其余回 200。
    fn serve() -> (u16, Seen) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen: Seen = Arc::default();
        let log = Arc::clone(&seen);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let mut parts = line.split_whitespace();
                let method = parts.next().unwrap_or_default().to_owned();
                let path = parts.next().unwrap_or("/").to_owned();
                let mut cookie = None;
                let mut length = 0usize;
                let mut headers = Vec::new();
                loop {
                    let mut header = String::new();
                    if reader.read_line(&mut header).unwrap_or(0) == 0 || header == "\r\n" {
                        break;
                    }
                    let lower = header.to_ascii_lowercase();
                    headers.push(lower.trim_end().to_owned());
                    if let Some(v) = lower.strip_prefix("cookie:") {
                        cookie = Some(v.trim().to_owned());
                    }
                    if let Some(v) = lower.strip_prefix("content-length:") {
                        length = v.trim().parse().unwrap_or(0);
                    }
                }
                let mut body = vec![0u8; length];
                reader.read_exact(&mut body).ok();
                log.lock()
                    .unwrap()
                    .push((method, path.clone(), cookie, headers));
                let (status, location, body) = if let Some(to) = path.strip_prefix("/to/") {
                    ("302 Found", to.to_owned(), Vec::new())
                } else if path == "/see-other" {
                    ("303 See Other", "/end".to_owned(), Vec::new())
                } else if path == "/big" {
                    ("200 OK", String::new(), vec![b'x'; 1024])
                } else {
                    ("200 OK", String::new(), b"ok".to_vec())
                };
                let location = if location.is_empty() {
                    String::new()
                } else {
                    format!("Location: {location}\r\n")
                };
                let head = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n{location}\r\n",
                    body.len()
                );
                stream.write_all(head.as_bytes()).ok();
                stream.write_all(&body).ok();
            }
        });
        (port, seen)
    }

    fn paths(seen: &Seen) -> Vec<String> {
        seen.lock().unwrap().iter().map(|r| r.1.clone()).collect()
    }

    /// 测试用的策略：只有 127.0.0.1 算公网。`*.site.test`、`other.test` 解析到它，
    /// `inner.site.test` 解析到 127.0.0.2：名单放行，地址不放行。
    fn local_policy() -> Policy {
        fn lookup(host: &str, port: u16) -> io::Result<Vec<SocketAddr>> {
            let ip = match host {
                "inner.site.test" => [127, 0, 0, 2],
                "site.test" | "a.site.test" | "b.site.test" | "other.test" => [127, 0, 0, 1],
                _ => return Err(io::Error::new(io::ErrorKind::NotFound, "no such host")),
            };
            Ok(vec![SocketAddr::from((ip, port))])
        }
        Policy {
            lookup,
            public: |ip| ip == IpAddr::from([127, 0, 0, 1]),
            cookies_over_http: true,
            ..Policy::strict()
        }
    }

    fn url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn host_list_rules() {
        let hosts = Hosts::only(["api.bilibili.com", ".HDSLB.com"]);
        assert!(hosts.allows("api.bilibili.com"));
        assert!(hosts.allows("i0.hdslb.com"));
        assert!(hosts.allows("a.b.hdslb.com"));
        assert!(!hosts.allows("hdslb.com"));
        assert!(!hosts.allows("evilhdslb.com"));
        assert!(!hosts.allows("hdslb.com.evil.test"));
        assert!(!hosts.allows("www.bilibili.com"));
        assert!(Hosts::any().allows("anything.example"));
    }

    /// 名单里的主机跳到名单外的主机、跳到解析成本机的名单内主机、跳到字面的回环地址，都不连。
    #[test]
    fn every_redirect_hop_is_checked() {
        let (port, seen) = serve();
        let client =
            Client::new(UA, Hosts::only(["site.test", ".site.test"])).with_policy(local_policy());
        let get = |path: &str| {
            client
                .get(&url(&format!("http://a.site.test:{port}{path}")))
                .follow_redirects(5)
                .send()
        };

        let ok = get(&format!("/to/http://b.site.test:{port}/end")).unwrap();
        assert_eq!(ok.status(), 200);
        assert_eq!(ok.url().as_str(), format!("http://b.site.test:{port}/end"));

        let err = get(&format!("/to/http://other.test:{port}/x")).unwrap_err();
        assert!(
            matches!(&err, Error::NotAllowed { host, first: false } if host == "other.test"),
            "{err:?}"
        );
        let err = get(&format!("/to/http://inner.site.test:{port}/x")).unwrap_err();
        assert!(
            matches!(err, Error::NotPublic { first: false, .. }),
            "{err:?}"
        );
        let err = get(&format!("/to/http://127.0.0.2:{port}/x")).unwrap_err();
        assert!(
            matches!(err, Error::NotAllowed { first: false, .. }),
            "{err:?}"
        );
        // 不限主机时，回环地址照样被地址策略拒
        let any = Client::new(UA, Hosts::any()).with_policy(local_policy());
        let err = any
            .get(&url(&format!(
                "http://site.test:{port}/to/http://127.0.0.2:{port}/x"
            )))
            .follow_redirects(5)
            .send()
            .unwrap_err();
        assert!(
            matches!(err, Error::NotPublic { first: false, .. }),
            "{err:?}"
        );
        // 被拒的那几跳都没有连出去
        assert!(
            paths(&seen)
                .iter()
                .all(|p| p.starts_with("/to/") || p == "/end")
        );
        assert_eq!(paths(&seen).iter().filter(|p| *p == "/end").count(), 1);
    }

    /// 内网放开只给第一跳：用真的公网判断，第一跳连本机可以，跳回本机就不行。
    #[test]
    fn private_first_hop_does_not_carry_over() {
        let (port, seen) = serve();
        fn lookup(_: &str, port: u16) -> io::Result<Vec<SocketAddr>> {
            Ok(vec![SocketAddr::from(([127, 0, 0, 1], port))])
        }
        let client = Client::new(UA, Hosts::any()).with_policy(Policy {
            lookup,
            ..Policy::strict()
        });
        let start = url(&format!("http://nas.test:{port}/end"));
        let err = client.get(&start).send().unwrap_err();
        assert!(
            matches!(err, Error::NotPublic { first: true, .. }),
            "{err:?}"
        );

        let ok = client.get(&start).allow_private_first_hop(true).send();
        assert_eq!(ok.unwrap().status(), 200);

        let hop = url(&format!(
            "http://nas.test:{port}/to/http://nas.test:{port}/end"
        ));
        let err = client
            .get(&hop)
            .allow_private_first_hop(true)
            .follow_redirects(5)
            .send()
            .unwrap_err();
        assert!(
            matches!(err, Error::NotPublic { first: false, .. }),
            "{err:?}"
        );
        assert_eq!(paths(&seen), ["/end".to_owned(), hop.path().to_owned()]);
    }

    /// 连的是检查过的那个地址：`pinned.test` 在真 DNS 里查不到，能拿到 200 就说明没有再查一次。
    #[test]
    fn connects_to_the_vetted_address() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut host = String::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                    break;
                }
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("host:") {
                    host = v.trim().to_owned();
                }
            }
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .unwrap();
            host
        });
        fn lookup(host: &str, port: u16) -> io::Result<Vec<SocketAddr>> {
            assert_eq!(host, "pinned.test");
            Ok(vec![SocketAddr::from(([127, 0, 0, 1], port))])
        }
        let target = url(&format!("http://pinned.test:{port}/"));
        let client = Client::new(UA, Hosts::any()).with_policy(Policy {
            lookup,
            ..Policy::strict()
        });
        let response = client
            .get(&target)
            .allow_private_first_hop(true)
            .timeout(Duration::from_secs(5))
            .send()
            .unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.url(), &target);
        assert_eq!(server.join().unwrap(), format!("pinned.test:{port}"));
        // 没有放开时，同样的名字因为解析到回环被拒
        assert!(matches!(
            client.get(&target).send(),
            Err(Error::NotPublic { first: true, .. })
        ));
    }

    #[test]
    fn cookie_goes_only_to_its_hosts() {
        let (port, seen) = serve();
        let client = Client::new(UA, Hosts::any()).with_policy(local_policy());
        let cookie = Cookie::new("sid=secret", ["a.site.test"]);
        assert!(!format!("{cookie:?}").contains("secret"));
        let start = url(&format!(
            "http://a.site.test:{port}/to/http://b.site.test:{port}/end"
        ));
        client
            .get(&start)
            .cookie(&cookie)
            .follow_redirects(1)
            .send()
            .unwrap();
        let log = seen.lock().unwrap().clone();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].2.as_deref(), Some("sid=secret"));
        assert_eq!((log[1].1.as_str(), log[1].2.as_deref()), ("/end", None));

        // 自己写的 Cookie 头不发
        let err = client
            .get(&start)
            .header("cookie", "sid=secret")
            .send()
            .unwrap_err();
        assert!(matches!(err, Error::CookieHeader), "{err:?}");
        assert_eq!(seen.lock().unwrap().len(), 2);
    }

    #[test]
    fn cookie_is_not_sent_over_plain_http() {
        let (port, seen) = serve();
        let client = Client::new(UA, Hosts::any()).with_policy(Policy {
            cookies_over_http: false,
            ..local_policy()
        });
        let cookie = Cookie::new("sid=secret", ["a.site.test"]);
        client
            .get(&url(&format!("http://a.site.test:{port}/end")))
            .cookie(&cookie)
            .send()
            .unwrap();
        let log = seen.lock().unwrap().clone();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].2, None);
    }

    #[test]
    fn redirects_are_not_followed_unless_asked() {
        let (port, seen) = serve();
        let client = Client::new(UA, Hosts::only(["site.test"])).with_policy(local_policy());
        let start = url(&format!(
            "http://site.test:{port}/to/http://elsewhere.test/"
        ));
        let response = client.get(&start).send().unwrap();
        assert_eq!(response.status(), 302);
        assert_eq!(response.header("location"), Some("http://elsewhere.test/"));
        assert_eq!(response.url(), &start);

        // 跳转数到上限还在跳就报错
        let chain = url(&format!(
            "http://site.test:{port}/to/http://site.test:{port}/to/http://site.test:{port}/end"
        ));
        assert!(client.get(&chain).follow_redirects(2).send().is_ok());
        assert!(matches!(
            client.get(&chain).follow_redirects(1).send(),
            Err(Error::TooManyRedirects(1))
        ));

        // 303 之后改成 GET
        let posted = url(&format!("http://site.test:{port}/see-other"));
        client
            .post(&posted, b"x=1".to_vec())
            .follow_redirects(1)
            .send()
            .unwrap();
        let log = seen.lock().unwrap().clone();
        let tail: Vec<_> = log[log.len() - 2..]
            .iter()
            .map(|r| (r.0.as_str(), r.1.as_str()))
            .collect();
        assert_eq!(tail, [("POST", "/see-other"), ("GET", "/end")]);
    }

    /// 查域名也算在请求的总时长里：查得慢就按总时长放弃，不等满 15 秒。
    #[test]
    fn slow_lookups_stop_at_the_deadline() {
        fn slow(_: &str, _: u16) -> io::Result<Vec<SocketAddr>> {
            std::thread::sleep(Duration::from_secs(3));
            Err(io::Error::new(io::ErrorKind::NotFound, "too late"))
        }
        let client = Client::new(UA, Hosts::any()).with_policy(Policy {
            lookup: slow,
            ..local_policy()
        });
        let started = Instant::now();
        let err = client
            .get(&url("http://slow.test/"))
            .timeout(Duration::from_millis(200))
            .send()
            .unwrap_err();
        assert!(
            matches!(&err, Error::Lookup(e) if e.kind() == io::ErrorKind::TimedOut),
            "{err:?}"
        );
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn bodies_are_capped() {
        let (port, _) = serve();
        let client = Client::new(UA, Hosts::any()).with_policy(local_policy());
        let big = url(&format!("http://site.test:{port}/big"));
        let mut response = client.get(&big).send().unwrap();
        assert!(matches!(
            response.read_to_vec(100),
            Err(Error::TooLarge(100))
        ));
        let mut response = client.get(&big).send().unwrap();
        assert_eq!(response.read_to_vec(1024).unwrap().len(), 1024);
        let mut reader = client.get(&big).send().unwrap().into_reader(100);
        let err = io::copy(&mut reader, &mut io::sink()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::FileTooLarge);
        let mut reader = client.get(&big).send().unwrap().into_reader(1024);
        assert_eq!(io::copy(&mut reader, &mut io::sink()).unwrap(), 1024);
    }

    /// 回 gzip 压缩的 4 MiB 零：去掉结尾几十字节后停住不再发，连接也不关。
    fn serve_gzip_bomb() -> u16 {
        const BOMB: &[u8] = include_bytes!("testdata/zeros-4mib.gz");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    loop {
                        let mut line = String::new();
                        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                            break;
                        }
                    }
                    let head =
                        "HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nConnection: close\r\n\r\n";
                    stream.write_all(head.as_bytes()).ok();
                    stream.write_all(&BOMB[..BOMB.len() - 64]).ok();
                    stream.flush().ok();
                    std::thread::sleep(Duration::from_secs(10));
                });
            }
        });
        port
    }

    /// 上限按解压后的字节算，读到上限就停：不先把整个压缩包解开，也不等服务器发完。
    #[test]
    fn body_limits_count_decoded_bytes() {
        let port = serve_gzip_bomb();
        let client = Client::new(UA, Hosts::any()).with_policy(local_policy());
        let bomb = url(&format!("http://site.test:{port}/"));
        let limit = 64 << 10;
        let send = || {
            client
                .get(&bomb)
                .timeout(Duration::from_secs(3))
                .send()
                .unwrap()
        };

        let started = Instant::now();
        let err = send().read_to_vec(limit).unwrap_err();
        assert!(matches!(err, Error::TooLarge(l) if l == limit), "{err:?}");
        assert!(started.elapsed() < Duration::from_secs(2));

        let started = Instant::now();
        let err = io::copy(&mut send().into_reader(limit), &mut io::sink()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::FileTooLarge, "{err:?}");
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    /// 调用方给的请求头只跟到同一个主机的那几跳；换了主机只留 Accept 和 Accept-Language。
    #[test]
    fn caller_headers_stay_on_the_first_host() {
        let (port, seen) = serve();
        let client = Client::new(UA, Hosts::any()).with_policy(local_policy());
        let start = url(&format!(
            "http://a.site.test:{port}/to/http://a.site.test:{port}/to/http://other.test:{port}/end"
        ));
        client
            .get(&start)
            .header("X-Secret", "s3cret")
            .header("Referer", "http://a.site.test/page?token=t")
            .header("Accept", "image/*")
            .follow_redirects(2)
            .send()
            .unwrap();
        let log = seen.lock().unwrap().clone();
        assert_eq!(log.len(), 3);
        for hop in &log[..2] {
            assert!(hop.3.iter().any(|h| h == "x-secret: s3cret"), "{hop:?}");
            assert!(hop.3.iter().any(|h| h.starts_with("referer:")), "{hop:?}");
        }
        let last = &log[2];
        assert_eq!(last.1, "/end");
        assert!(
            !last
                .3
                .iter()
                .any(|h| h.starts_with("x-secret") || h.starts_with("referer")),
            "{last:?}"
        );
        assert!(last.3.iter().any(|h| h == "accept: image/*"), "{last:?}");
    }

    /// 本机测试服务只有明文 http，降级那一条按规则本身测。
    #[test]
    fn caller_headers_rule() {
        let first = url("https://a.site.test/x");
        assert!(keeps_caller_headers(&first, &url("https://a.site.test/y")));
        assert!(!keeps_caller_headers(&first, &url("http://a.site.test/y")));
        assert!(!keeps_caller_headers(&first, &url("https://b.site.test/y")));
        let plain = url("http://a.site.test/x");
        assert!(keeps_caller_headers(&plain, &url("https://a.site.test/y")));
        assert!(keeps_caller_headers(&plain, &url("http://a.site.test/y")));
    }

    /// 本机的小代理：`http` 认 `CONNECT`，`socks5` 认不要账号的 SOCKS5。不管被要求连哪里都接到 `target`
    /// 那个端口（一个解析错了、把名字指到本机的代理），记下被要求连的「主机:端口」。`reject` 时 `CONNECT` 回 407。
    fn proxy(kind: &'static str, target: u16, reject: bool) -> (u16, Arc<Mutex<Vec<String>>>) {
        use std::io::Read;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let asked: Arc<Mutex<Vec<String>>> = Arc::default();
        let log = Arc::clone(&asked);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut client) = stream else { continue };
                let log = Arc::clone(&log);
                std::thread::spawn(move || {
                    let wanted = if kind == "http" {
                        let mut reader = BufReader::new(client.try_clone().unwrap());
                        let mut line = String::new();
                        reader.read_line(&mut line).unwrap();
                        loop {
                            let mut h = String::new();
                            if reader.read_line(&mut h).unwrap_or(0) == 0 || h == "\r\n" {
                                break;
                            }
                        }
                        let wanted = line
                            .split_whitespace()
                            .nth(1)
                            .unwrap_or_default()
                            .to_owned();
                        if reject {
                            log.lock().unwrap().push(wanted);
                            let _ = client.write_all(
                                b"HTTP/1.1 407 Proxy Authentication Required\r\nContent-Length: 0\r\n\r\n",
                            );
                            return;
                        }
                        client
                            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                            .unwrap();
                        wanted
                    } else {
                        let mut head = [0u8; 2];
                        client.read_exact(&mut head).unwrap();
                        let mut methods = vec![0u8; head[1] as usize];
                        client.read_exact(&mut methods).unwrap();
                        client.write_all(&[5, 0]).unwrap();
                        let mut req = [0u8; 4];
                        client.read_exact(&mut req).unwrap();
                        let host = match req[3] {
                            1 => {
                                let mut a = [0u8; 4];
                                client.read_exact(&mut a).unwrap();
                                std::net::Ipv4Addr::from(a).to_string()
                            }
                            4 => {
                                let mut a = [0u8; 16];
                                client.read_exact(&mut a).unwrap();
                                std::net::Ipv6Addr::from(a).to_string()
                            }
                            _ => {
                                let mut len = [0u8; 1];
                                client.read_exact(&mut len).unwrap();
                                let mut name = vec![0u8; len[0] as usize];
                                client.read_exact(&mut name).unwrap();
                                String::from_utf8(name).unwrap()
                            }
                        };
                        let mut p = [0u8; 2];
                        client.read_exact(&mut p).unwrap();
                        client.write_all(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 0]).unwrap();
                        format!("{host}:{}", u16::from_be_bytes(p))
                    };
                    log.lock().unwrap().push(wanted);
                    let upstream = std::net::TcpStream::connect(("127.0.0.1", target)).unwrap();
                    let (mut a, mut b) =
                        (client.try_clone().unwrap(), upstream.try_clone().unwrap());
                    std::thread::spawn(move || {
                        let _ = io::copy(&mut a, &mut b);
                    });
                    let (mut c, mut d) = (upstream, client);
                    let _ = io::copy(&mut c, &mut d);
                });
            }
        });
        (port, asked)
    }

    /// 经代理时的测试策略：`public.test` 本机查出来是公网地址，`inner.test` 是内网地址，别的本机查不到。
    fn proxied(proxy: &str) -> Policy {
        fn lookup(host: &str, port: u16) -> io::Result<Vec<SocketAddr>> {
            match host {
                "public.test" | "b.public.test" => {
                    Ok(vec![SocketAddr::from(([93, 184, 216, 34], port))])
                }
                "inner.test" => Ok(vec![SocketAddr::from(([10, 0, 0, 1], port))]),
                _ => Err(io::Error::new(io::ErrorKind::NotFound, "no such host")),
            }
        }
        Policy {
            lookup,
            proxy: Some(Proxy::intern(proxy).unwrap()),
            ..Policy::strict()
        }
    }

    /// 明文 http 经代理：本机查过、是公网的那个数字地址才交给代理，名字只在 `Host` 里；
    /// 代理那边就算把名字指到本机，它也看不到名字。
    #[test]
    fn plain_http_asks_the_proxy_for_the_vetted_address() {
        let (port, seen) = serve();
        for kind in ["http", "socks5"] {
            let (pport, asked) = proxy(kind, port, false);
            let client = Client::new(UA, Hosts::only(["public.test"]))
                .with_policy(proxied(&format!("{kind}://127.0.0.1:{pport}")));
            let ok = client
                .get(&url(&format!("http://public.test:{port}/end")))
                .send()
                .unwrap();
            assert_eq!(ok.status(), 200, "{kind}");
            assert_eq!(ok.url().as_str(), format!("http://public.test:{port}/end"));
            assert_eq!(
                *asked.lock().unwrap(),
                vec![format!("93.184.216.34:{port}")],
                "{kind}: the proxy never sees the name"
            );
        }
        let all = seen.lock().unwrap().clone();
        assert_eq!(all.len(), 2);
        for (_, path, _, headers) in &all {
            assert_eq!(path, "/end");
            let host: Vec<_> = headers.iter().filter(|h| h.starts_with("host:")).collect();
            assert_eq!(host, [&format!("host: public.test:{port}")], "{headers:?}");
        }

        // 本机查不到、或查出来是内网地址的明文 http：不问代理
        let (pport, asked) = proxy("http", port, false);
        let any = Client::new(UA, Hosts::any())
            .with_policy(proxied(&format!("http://127.0.0.1:{pport}")));
        let err = any.get(&url("http://nowhere.test/")).send().unwrap_err();
        assert!(matches!(err, Error::Lookup(_)), "{err:?}");
        let err = any.get(&url("http://inner.test/")).send().unwrap_err();
        assert!(matches!(err, Error::NotPublic { .. }), "{err:?}");
        assert!(asked.lock().unwrap().is_empty());
    }

    /// 一个只出示别的名字的证书（自签的 `other.test`）的本机 TLS 服务。
    fn tls_server() -> u16 {
        let cert = rcgen::generate_simple_self_signed(vec!["other.test".to_owned()]).unwrap();
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(
                vec![cert.cert.der().clone()],
                rustls::pki_types::PrivateKeyDer::Pkcs8(cert.signing_key.serialize_der().into()),
            )
            .unwrap();
        let config = Arc::new(config);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let config = Arc::clone(&config);
                std::thread::spawn(move || {
                    let mut conn = rustls::ServerConnection::new(config).unwrap();
                    // 握手做不完（客户端不认证书）就断了
                    while conn.is_handshaking() {
                        if conn.complete_io(&mut stream).is_err() {
                            break;
                        }
                    }
                });
            }
        });
        port
    }

    /// https 经代理按名字去：代理把隧道接到一个只能出示别的名字的证书的本机服务，请求失败，
    /// 什么都没取到。证书和主机名的校验在这条路上一直开着。
    #[test]
    fn https_by_name_is_bounded_by_the_certificate() {
        let tls = tls_server();
        for kind in ["http", "socks5"] {
            let (pport, asked) = proxy(kind, tls, false);
            let client = Client::new(UA, Hosts::only(["unknown.test"]))
                .with_policy(proxied(&format!("{kind}://127.0.0.1:{pport}")));
            let err = client
                .get(&url(&format!("https://unknown.test:{tls}/")))
                .send()
                .unwrap_err();
            assert!(
                matches!(&err, Error::Network(m) if m.to_ascii_lowercase().contains("certificate")),
                "{kind}: {err:?}"
            );
            assert_eq!(
                *asked.lock().unwrap(),
                vec![format!("unknown.test:{tls}")],
                "{kind}: https goes by name"
            );
        }
    }

    /// 两种协议都先挡：名单外的主机、一看就是本机内网的名字、内网和保留段的数字地址；
    /// https 的名字本机查出来是内网地址的也挡。每一跳照它自己的协议查。
    #[test]
    fn refusals_happen_before_the_proxy_is_asked() {
        let (port, seen) = serve();
        let (pport, asked) = proxy("http", port, false);
        let policy = proxied(&format!("http://127.0.0.1:{pport}"));
        let only =
            Client::new(UA, Hosts::only(["public.test", "b.public.test"])).with_policy(policy);
        let any = Client::new(UA, Hosts::any()).with_policy(policy);
        let err = only.get(&url("http://other.test/")).send().unwrap_err();
        assert!(
            matches!(err, Error::NotAllowed { first: true, .. }),
            "{err:?}"
        );
        for scheme in ["http", "https"] {
            for host in [
                "router",
                "nas.local",
                "10.0.0.1",
                "[::1]",
                "127.0.0.1",
                "inner.test",
            ] {
                let target = format!("{scheme}://{host}/");
                let err = any.get(&url(&target)).send().unwrap_err();
                assert!(
                    matches!(err, Error::NotPublic { first: true, .. }),
                    "{target}: {err:?}"
                );
            }
        }
        // 跳到名单外的主机：第一跳经代理去了，第二跳没问代理
        let err = only
            .get(&url(&format!(
                "http://public.test:{port}/to/http://other.test:{port}/x"
            )))
            .follow_redirects(5)
            .send()
            .unwrap_err();
        assert!(
            matches!(err, Error::NotAllowed { first: false, .. }),
            "{err:?}"
        );
        // 跳到本机查不到的明文 http：照 http 的规矩拒，不问代理
        let err = any
            .get(&url(&format!(
                "http://public.test:{port}/to/http://nowhere.test:{port}/x"
            )))
            .follow_redirects(5)
            .send()
            .unwrap_err();
        assert!(matches!(err, Error::Lookup(_)), "{err:?}");
        assert_eq!(
            *asked.lock().unwrap(),
            vec![format!("93.184.216.34:{port}"); 2]
        );
        assert_eq!(paths(&seen).len(), 2);
    }

    #[test]
    fn proxy_credentials_stay_out_of_errors() {
        let (port, _) = serve();
        let (pport, asked) = proxy("http", port, true);
        let policy = proxied(&format!("http://alice:hunter2pass@127.0.0.1:{pport}"));
        assert!(!format!("{policy:?}").contains("hunter2pass"));
        let client = Client::new(UA, Hosts::any()).with_policy(policy);
        let err = client
            .get(&url(&format!("http://public.test:{port}/end")))
            .send()
            .unwrap_err();
        let text = format!("{err} {err:?}");
        assert!(
            !text.contains("hunter2pass") && !text.contains("alice"),
            "{text}"
        );
        assert_eq!(asked.lock().unwrap().len(), 1);
    }

    #[test]
    fn a_broken_proxy_setting_is_an_error_not_a_direct_connection() {
        let broken = Proxy::parse("ftp://x").unwrap_err();
        assert!(matches!(broken, Error::Invalid(_)));
        assert!(
            Policy::strict().proxy.is_none(),
            "strict never uses a proxy"
        );
    }

    /// 经这台机器的系统代理取 `https://example.com/`（`USUYO_TEST_PROXY` 指向代理，不设就跳过）。
    #[test]
    fn reaches_example_com_through_the_system_proxy() {
        let Ok(proxy) = std::env::var("USUYO_TEST_PROXY") else {
            eprintln!("skipped: USUYO_TEST_PROXY is not set");
            return;
        };
        let policy = Policy {
            proxy: Some(Proxy::intern(&proxy).unwrap()),
            ..Policy::strict()
        };
        let client = Client::new(UA, Hosts::only(["example.com"])).with_policy(policy);
        let mut r = client
            .get(&url("https://example.com/"))
            .timeout(Duration::from_secs(20))
            .send()
            .unwrap();
        assert_eq!(r.status(), 200);
        let body = r.read_to_vec(1 << 20).unwrap();
        assert!(String::from_utf8_lossy(&body).contains("Example Domain"));
    }
}
