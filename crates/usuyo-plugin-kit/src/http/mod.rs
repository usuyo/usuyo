//! 插件发请求的客户端。每一跳（包括跳转去的地址）都先过三道检查再连：
//! 主机在调用方给的名单里；解析出的地址全是公网；连接只连检查过的那些地址，不再查第二次 DNS。
//! 跳转在这里一跳一跳地跟，默认不跟；不读环境变量里的代理，没有 cookie 罐，
//! cookie 只发给调用方指定的主机。

mod address;

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
        let (tx, rx) = mpsc::sync_channel(1);
        let (target, policy) = (url.clone(), self.policy);
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
    /// 每一跳都带的请求头。cookie 不能从这里带，见 [`Request::cookie`]。
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
        let deadline = Instant::now() + self.timeout;
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
            let agent = pinned_agent(host, addrs, left, &self.client.user_agent);
            // 登录信息只走 https：同一个主机跳到明文 http 也不带。
            let encrypted = url.scheme() == "https" || self.client.policy.cookies_over_http;
            let cookie = self
                .cookie
                .as_ref()
                .filter(|c| encrypted && c.hosts.allows(host))
                .map(|c| c.value.as_str());
            let response = match &body {
                None => with_headers(agent.get(url.as_str()), &self.headers, cookie).call(),
                Some(bytes) => with_headers(agent.post(url.as_str()), &self.headers, cookie)
                    .send(bytes.as_slice()),
            }
            .map_err(transport)?;
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

fn with_headers<B>(
    mut request: ureq::RequestBuilder<B>,
    headers: &[(String, String)],
    cookie: Option<&str>,
) -> ureq::RequestBuilder<B> {
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

    /// 读完整个响应体，超过 `limit` 字节回 [`Error::TooLarge`]。
    pub fn read_to_vec(&mut self, limit: u64) -> Result<Vec<u8>, Error> {
        // ureq 读满上限就报错，哪怕正好读完：多给一个字节，自己比
        let bytes = self
            .inner
            .body_mut()
            .with_config()
            .limit(limit.saturating_add(1))
            .read_to_vec()
            .map_err(|e| match e {
                ureq::Error::BodyExceedsLimit(_) => Error::TooLarge(limit),
                other => transport(other),
            })?;
        if bytes.len() as u64 > limit {
            return Err(Error::TooLarge(limit));
        }
        Ok(bytes)
    }

    /// 边读边处理大文件；读出的字节超过 `limit` 时报读错误，错误的类别是
    /// `io::ErrorKind::FileTooLarge`，调用方凭它和网络中断分开。
    pub fn into_reader(self, limit: u64) -> impl Read + Send + 'static {
        Capped {
            inner: self
                .inner
                .into_body()
                .into_with_config()
                .limit(limit.saturating_add(1))
                .reader(),
            left: limit,
            limit,
        }
    }
}

struct Capped<R> {
    inner: R,
    left: u64,
    limit: u64,
}

impl<R: Read> Read for Capped<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
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

    /// 收到的请求：方法、路径、Cookie 头。
    type Seen = Arc<Mutex<Vec<(String, String, Option<String>)>>>;

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
                loop {
                    let mut header = String::new();
                    if reader.read_line(&mut header).unwrap_or(0) == 0 || header == "\r\n" {
                        break;
                    }
                    let lower = header.to_ascii_lowercase();
                    if let Some(v) = lower.strip_prefix("cookie:") {
                        cookie = Some(v.trim().to_owned());
                    }
                    if let Some(v) = lower.strip_prefix("content-length:") {
                        length = v.trim().parse().unwrap_or(0);
                    }
                }
                let mut body = vec![0u8; length];
                reader.read_exact(&mut body).ok();
                log.lock().unwrap().push((method, path.clone(), cookie));
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
}
