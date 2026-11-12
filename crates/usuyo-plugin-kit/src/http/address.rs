//! 连出去之前的地址检查：链接和平台回答里的地址都不可信，只连公网地址，
//! 不让它们摸到本机、路由器、内网设备和云元数据。

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};

use url::{Host, Url};

use super::proxy::Proxy;

/// 查域名：给主机名和端口，回所有解析出的地址。测试换成固定的表。
pub type Lookup = fn(&str, u16) -> io::Result<Vec<SocketAddr>>;

/// 地址策略：怎么查域名、哪些地址算公网。
/// 程序里只用 `Policy::strict`；测试自己拼一个值，让本机测试服务算「公网」。
#[derive(Debug, Clone, Copy)]
pub struct Policy {
    pub lookup: Lookup,
    pub public: fn(IpAddr) -> bool,
    /// 登录信息能不能走明文 http。出厂的策略不许；连本地测试服务器的测试才开。
    pub cookies_over_http: bool,
    /// 宿主查到这台机器开着假 IP 模式的代理：域名查出来全在假 IP 段时照常放行。
    pub fake_ip_names: bool,
    /// 经这个代理连出去（宿主经 `USUYO_PROXY` 交来的）；`None` 是直连。地址检查换成代理那一套（见 `proxy` 模块）。
    pub proxy: Option<&'static Proxy>,
}

impl Policy {
    pub fn strict() -> Self {
        Policy {
            lookup: system_lookup,
            public: is_public,
            cookies_over_http: false,
            fake_ip_names: false,
            proxy: None,
        }
    }

    /// 插件实际运行时用的：严格策略，加上宿主经 `USUYO_NETWORK` 说明的网络环境和经 `USUYO_PROXY`
    /// 交来的代理。是不是假 IP 环境由宿主拿固定的公网域名去查，不看要取的那个域名自己的回答。
    pub fn from_host() -> Self {
        Policy {
            fake_ip_names: std::env::var("USUYO_NETWORK").is_ok_and(|v| v == "fake-ip"),
            proxy: Proxy::from_env(),
            ..Self::strict()
        }
    }
}

/// 不是公网的 IPv4 段：(网络, 前缀长度)。
const V4_BLOCKED: [(u32, u32); 15] = [
    (0x0000_0000, 8),  // 0.0.0.0/8 本网络，含未指定地址
    (0x0a00_0000, 8),  // 10.0.0.0/8 内网
    (0x6440_0000, 10), // 100.64.0.0/10 运营商 NAT
    (0x7f00_0000, 8),  // 127.0.0.0/8 回环
    (0xa9fe_0000, 16), // 169.254.0.0/16 链路本地，含云元数据
    (0xac10_0000, 12), // 172.16.0.0/12 内网
    (0xc000_0000, 24), // 192.0.0.0/24 协议保留
    (0xc000_0200, 24), // 192.0.2.0/24 文档
    (0xc058_6300, 24), // 192.88.99.0/24 6to4 中继
    (0xc0a8_0000, 16), // 192.168.0.0/16 内网
    (0xc612_0000, 15), // 198.18.0.0/15 基准测试
    (0xc633_6400, 24), // 198.51.100.0/24 文档
    (0xcb00_7100, 24), // 203.0.113.0/24 文档
    (0xe000_0000, 4),  // 224.0.0.0/4 组播
    (0xf000_0000, 4),  // 240.0.0.0/4 保留，含 255.255.255.255 广播
];

/// 2000::/3 全球单播里仍然不放行的段。
const V6_BLOCKED: [(u128, u32); 4] = [
    (0x2001_0000 << 96, 23), // 2001::/23 协议保留，含 Teredo
    (0x2001_0db8 << 96, 32), // 2001:db8::/32 文档
    (0x2002 << 112, 16),     // 2002::/16 6to4
    (0x3fff << 112, 20),     // 3fff::/20 文档
];

fn in_v4(bits: u32, (net, len): (u32, u32)) -> bool {
    bits & (u32::MAX << (32 - len)) == net
}

fn in_v6(bits: u128, (net, len): (u128, u32)) -> bool {
    bits & (u128::MAX << (128 - len)) == net
}

/// 是不是可以连的公网地址。IPv6 只放行 2000::/3 里的全球单播；
/// IPv4 映射（::ffff:0:0/96）和 NAT64（64:ff9b::/96）按里面的 IPv4 判断。
fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4_public(v4),
        IpAddr::V6(v6) => v6_public(v6),
    }
}

fn v4_public(ip: Ipv4Addr) -> bool {
    let bits = u32::from(ip);
    !V4_BLOCKED.iter().any(|&range| in_v4(bits, range))
}

fn v6_public(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return v4_public(v4);
    }
    let bits = u128::from(ip);
    if in_v6(bits, (0x0064_ff9b << 96, 96)) {
        let o = ip.octets();
        return v4_public(Ipv4Addr::new(o[12], o[13], o[14], o[15]));
    }
    in_v6(bits, (1 << 125, 3)) && !V6_BLOCKED.iter().any(|&range| in_v6(bits, range))
}

/// 代理工具「假 IP」模式用的段：开着这种代理时，系统给每个域名都回这一段里的地址，
/// 真正的地址由代理工具自己去查，这里看不到。
const FAKE_IP: (u32, u32) = (0xc612_0000, 15);

fn is_fake_ip(ip: IpAddr) -> bool {
    matches!(ip, IpAddr::V4(v4) if in_v4(u32::from(v4), FAKE_IP))
}

/// 一看就是本机或内网的名字：没有点的单段名，和这些后缀。
/// 只在假 IP 环境下用：平时由解析出的地址把关，这时地址看不到，先按名字挡一道。
fn is_local_name(name: &str) -> bool {
    const SUFFIXES: [&str; 9] = [
        "localhost",
        "local",
        "internal",
        "lan",
        "home",
        "home.arpa",
        "corp",
        "intranet",
        "private",
    ];
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    !name.contains('.')
        || SUFFIXES
            .iter()
            .any(|s| name.strip_suffix(s).is_some_and(|rest| rest.ends_with('.')))
}

#[derive(Debug)]
pub(crate) enum Refusal {
    /// 不是 http(s)、带账号密码、没有主机。
    Invalid(String),
    /// 地址不是公网。
    NotPublic(String),
    /// 域名查不到。
    Lookup(io::Error),
}

/// 一条链接能不能连，能连就回检查过的地址，调用方只连这些地址。
/// `private_ok` 只放开地址段这一条，协议和账号密码照样检查。
pub(crate) fn admit(
    url: &Url,
    private_ok: bool,
    policy: &Policy,
) -> Result<Vec<SocketAddr>, Refusal> {
    let refuse = |why: String| Err(Refusal::Invalid(why));
    if !matches!(url.scheme(), "http" | "https") {
        return refuse(format!("scheme `{}` is not http(s)", url.scheme()));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return refuse("URL carries credentials".into());
    }
    let Some(port) = url.port_or_known_default() else {
        return refuse("no port".into());
    };
    if policy.proxy.is_some() {
        if url.scheme() == "https" {
            return admit_proxied(url, private_ok, port, policy);
        }
        // 经代理的明文 http 照直连的规矩查（本机要查得到、全是公网），只是连的是代理；
        // 一看就是本机、内网的名字先挡掉。
        if let Some(Host::Domain(name)) = url.host()
            && !private_ok
            && is_local_name(name)
        {
            return Err(Refusal::NotPublic(format!(
                "`{name}` looks like a local name"
            )));
        }
    }
    // 数字地址由 URL 类型解析，十进制、十六进制、八进制的写法都已经变成 IP
    let (addrs, name) = match url.host() {
        Some(Host::Ipv4(ip)) => (vec![SocketAddr::new(ip.into(), port)], None),
        Some(Host::Ipv6(ip)) => (vec![SocketAddr::new(ip.into(), port)], None),
        Some(Host::Domain(name)) => (
            (policy.lookup)(name, port).map_err(Refusal::Lookup)?,
            Some(name),
        ),
        None => return refuse("no host".into()),
    };
    if addrs.is_empty() {
        return Err(Refusal::Lookup(io::Error::new(
            io::ErrorKind::NotFound,
            "no address",
        )));
    }
    // 宿主说这台机器开着假 IP 模式的代理时，公网的域名查出来全在假 IP 段，只能放行。
    // 代价是这时看不到真正的地址：一个真指向内网的域名拦不住，要靠代理工具自己的规则；
    // 能做的是按名字先挡掉一看就是本机、内网的。链接里直接写这一段的数字地址照样拒绝。
    let proxied = policy.fake_ip_names
        && name.is_some_and(|n| !is_local_name(n))
        && addrs.iter().all(|a| is_fake_ip(a.ip()));
    if !private_ok
        && !proxied
        && let Some(bad) = addrs.iter().find(|a| !(policy.public)(a.ip()))
    {
        let host = url.host_str().unwrap_or_default();
        return Err(Refusal::NotPublic(format!(
            "`{host}` is not a public address ({})",
            bad.ip()
        )));
    }
    Ok(addrs)
}

/// 经代理的 https：域名由代理去查，这里不回地址（调用方只连代理）。
/// 数字地址照样要是公网的；名字不许一看就是本机、内网的；本机查得到、查出来有内网地址的也不许
/// （假 IP 段在宿主说了是假 IP 环境时不算）。本机查不到不要紧：代理会去查，TLS 证书兜底。
fn admit_proxied(
    url: &Url,
    private_ok: bool,
    port: u16,
    policy: &Policy,
) -> Result<Vec<SocketAddr>, Refusal> {
    let host = url.host_str().unwrap_or_default();
    let not_public = |ip: IpAddr| {
        Err(Refusal::NotPublic(format!(
            "`{host}` is not a public address ({ip})"
        )))
    };
    match url.host() {
        Some(Host::Ipv4(ip)) if !private_ok && !(policy.public)(ip.into()) => not_public(ip.into()),
        Some(Host::Ipv6(ip)) if !private_ok && !(policy.public)(ip.into()) => not_public(ip.into()),
        Some(Host::Ipv4(_) | Host::Ipv6(_)) => Ok(Vec::new()),
        Some(Host::Domain(name)) => {
            if private_ok {
                return Ok(Vec::new());
            }
            if is_local_name(name) {
                return Err(Refusal::NotPublic(format!(
                    "`{name}` looks like a local name"
                )));
            }
            if let Ok(addrs) = (policy.lookup)(name, port)
                && let Some(bad) = addrs.iter().find(|a| {
                    !((policy.public)(a.ip()) || policy.fake_ip_names && is_fake_ip(a.ip()))
                })
            {
                return not_public(bad.ip());
            }
            Ok(Vec::new())
        }
        None => Err(Refusal::Invalid("no host".into())),
    }
}

/// 系统 DNS。超时由调用方管：检查放在线程里，等不到就不等了。
fn system_lookup(host: &str, port: u16) -> io::Result<Vec<SocketAddr>> {
    (host, port).to_socket_addrs().map(Vec::from_iter)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn ranges() {
        for blocked in [
            "0.0.0.0",
            "0.1.2.3",
            "10.0.0.1",
            "100.64.0.1",
            "100.127.255.255",
            "127.0.0.1",
            "127.255.255.254",
            "169.254.169.254",
            "172.16.0.1",
            "172.31.255.255",
            "192.0.0.8",
            "192.0.2.1",
            "192.88.99.1",
            "192.168.1.1",
            "198.18.0.1",
            "198.19.255.255",
            "198.51.100.1",
            "203.0.113.1",
            "224.0.0.1",
            "239.255.255.250",
            "240.0.0.1",
            "255.255.255.255",
            "::",
            "::1",
            "::127.0.0.1",
            "::ffff:127.0.0.1",
            "::ffff:10.0.0.1",
            "::ffff:169.254.169.254",
            "64:ff9b::7f00:1",
            "64:ff9b::a9fe:a9fe",
            "64:ff9b:1::1",
            "100::1",
            "fc00::1",
            "fd12:3456::1",
            "fe80::1",
            "fec0::1",
            "ff02::1",
            "2001::1",
            "2001:db8::1",
            "2002:7f00:1::1",
            "3fff::1",
            "5f00::1",
        ] {
            assert!(!is_public(ip(blocked)), "{blocked}");
        }
        for public in [
            "1.1.1.1",
            "8.8.8.8",
            "93.184.216.34",
            "100.63.255.255",
            "100.128.0.0",
            "172.15.255.255",
            "172.32.0.0",
            "192.0.1.1",
            "198.17.255.255",
            "223.255.255.255",
            "::ffff:8.8.8.8",
            "64:ff9b::808:808",
            "2606:4700:4700::1111",
            "2400:cb00::1",
        ] {
            assert!(is_public(ip(public)), "{public}");
        }
    }

    fn table(host: &str, port: u16) -> io::Result<Vec<SocketAddr>> {
        let ips: &[&str] = match host {
            "public.test" => &["93.184.216.34", "2606:4700::1"],
            "intranet.test" => &["192.168.1.10"],
            "mixed.test" => &["93.184.216.34", "10.0.0.1"],
            "mapped.test" => &["::ffff:127.0.0.1"],
            "proxied.test"
            | "router"
            | "nas.local"
            | "nas.local."
            | "db.corp"
            | "metadata.google.internal" => &["198.18.11.245"],
            "half-proxied.test" => &["198.18.11.245", "10.0.0.1"],
            _ => return Err(io::Error::new(io::ErrorKind::NotFound, "no such host")),
        };
        Ok(ips.iter().map(|s| SocketAddr::new(ip(s), port)).collect())
    }

    fn check(input: &str, private_ok: bool) -> Result<Vec<SocketAddr>, Refusal> {
        let policy = Policy {
            lookup: table,
            ..Policy::strict()
        };
        admit(&Url::parse(input).unwrap(), private_ok, &policy)
    }

    fn refused(input: &str) -> bool {
        matches!(
            check(input, false),
            Err(Refusal::Invalid(_) | Refusal::NotPublic(_))
        )
    }

    /// 经代理时每一跳照它自己的协议查：https 的名字本机查不到也行（代理去查，证书兜底），
    /// 明文 http 的本机要查得到、全是公网。https 跳到 http 就落到 http 那一条。
    #[test]
    fn behind_a_proxy_the_rule_follows_the_scheme() {
        let proxy = Proxy::intern("http://127.0.0.1:9").unwrap();
        let policy = Policy {
            lookup: table,
            proxy: Some(proxy),
            ..Policy::strict()
        };
        let check = |u: &str| admit(&Url::parse(u).unwrap(), false, &policy);
        assert_eq!(check("https://nowhere.test/").unwrap(), Vec::new());
        assert!(matches!(
            check("http://nowhere.test/"),
            Err(Refusal::Lookup(_))
        ));
        assert_eq!(check("http://public.test/").unwrap().len(), 2);
        for refused in [
            "https://intranet.test/",
            "http://intranet.test/",
            "https://nas.local/",
            "http://router/",
            "https://192.168.1.1/",
            "http://[::1]/",
        ] {
            assert!(
                matches!(check(refused), Err(Refusal::NotPublic(_))),
                "{refused}"
            );
        }
    }

    #[test]
    fn names_resolve_then_check_every_address() {
        let addrs = check("https://public.test/a", false).unwrap();
        assert_eq!(addrs.len(), 2);
        assert!(addrs.iter().all(|a| a.port() == 443));
        assert!(matches!(
            check("http://intranet.test/", false),
            Err(Refusal::NotPublic(_))
        ));
        assert!(refused("http://mixed.test/"));
        assert!(refused("http://mapped.test/"));
        assert!(matches!(
            check("http://nowhere.test/", false),
            Err(Refusal::Lookup(_))
        ));
    }

    fn check_behind_fake_ip(input: &str) -> Result<Vec<SocketAddr>, Refusal> {
        let policy = Policy {
            lookup: table,
            fake_ip_names: true,
            ..Policy::strict()
        };
        admit(&Url::parse(input).unwrap(), false, &policy)
    }

    #[test]
    fn fake_ip_answers_need_the_host_to_say_so() {
        // 宿主没说是假 IP 环境：这一段照旧拒绝
        assert!(refused("https://proxied.test/"));
        assert_eq!(
            check_behind_fake_ip("https://proxied.test/").unwrap(),
            [SocketAddr::new(ip("198.18.11.245"), 443)]
        );
        for input in [
            "http://half-proxied.test/",
            "http://198.18.11.245/",
            "http://198.18.0.1/",
            "http://router/",
            "http://nas.local/",
            "http://NAS.Local./",
            "http://db.corp/",
            "http://metadata.google.internal/",
            "http://intranet.test/",
        ] {
            assert!(
                matches!(
                    check_behind_fake_ip(input),
                    Err(Refusal::Invalid(_) | Refusal::NotPublic(_))
                ),
                "{input}"
            );
        }
        assert!(!is_local_name("notlocal.example"));
        assert!(!is_local_name("mylan.example.com"));
    }

    #[test]
    fn numeric_spellings_go_through_the_url_parser() {
        for input in [
            "http://127.0.0.1/",
            "http://127.1/",
            "http://2130706433/",
            "http://0x7f000001/",
            "http://0x7f.0.0.1/",
            "http://0177.0.0.1/",
            "http://127.0.0.1./",
            "http://[::1]/",
            "http://[::ffff:127.0.0.1]/",
            "http://[::ffff:7f00:1]/",
            "http://[64:ff9b::a00:1]/",
            "http://169.254.169.254/latest/meta-data/",
            "http://0/",
        ] {
            assert!(refused(input), "{input}");
        }
        assert!(check("http://93.184.216.34:8080/", false).is_ok());
    }

    #[test]
    fn scheme_and_credentials() {
        assert!(refused("ftp://public.test/"));
        assert!(refused("file:///etc/passwd"));
        assert!(refused("http://user@public.test/"));
        assert!(refused("http://user:pw@public.test/"));
        assert!(refused("http://:pw@public.test/"));
    }

    #[test]
    fn private_ok_only_lifts_the_range_check() {
        assert_eq!(
            check("http://intranet.test:8080/", true).unwrap(),
            [SocketAddr::new(ip("192.168.1.10"), 8080)]
        );
        assert!(check("http://127.0.0.1/", true).is_ok());
        assert!(matches!(
            check("http://user:pw@127.0.0.1/", true),
            Err(Refusal::Invalid(_))
        ));
        assert!(matches!(
            check("ftp://127.0.0.1/", true),
            Err(Refusal::Invalid(_))
        ));
    }
}
