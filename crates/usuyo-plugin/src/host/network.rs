//! 这台机器的网络环境：是不是开着假 IP 模式的代理（Clash、Surge、sing-box 一类）。
//!
//! 开着这种代理时，系统给每个域名都回 198.18.0.0/15 里的地址，插件的「只连公网地址」
//! 检查会把所有网站都拒掉。这里拿固定的公网域名去查一次，结果经 `USUYO_NETWORK` 告诉插件。
//! 不看要取的那个域名自己的回答：那是别人能控制的。

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::{Mutex, mpsc};
use std::time::{Duration, Instant};

/// 拿来查的固定域名：IANA 留作示例用的，不属于任何平台，真实地址是公网的。
const PROBES: [&str; 2] = ["example.com", "example.org"];
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(2);
/// 结果留这么久：代理开了或关了，一分钟内跟上。
const FRESH_FOR: Duration = Duration::from_secs(60);

static LAST: Mutex<Option<(Instant, bool)>> = Mutex::new(None);

fn is_fake_ip(ip: IpAddr) -> bool {
    matches!(ip, IpAddr::V4(v4) if v4.octets()[0] == 198 && v4.octets()[1] & 0xfe == 18)
}

/// 每个固定域名都查到了地址，而且全在假 IP 段，才算。
fn all_fake(answers: &[Vec<SocketAddr>]) -> bool {
    !answers.is_empty()
        && answers
            .iter()
            .all(|found| !found.is_empty() && found.iter().all(|a| is_fake_ip(a.ip())))
}

fn probe() -> bool {
    let (tx, rx) = mpsc::channel();
    for name in PROBES {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let found = (name, 80)
                .to_socket_addrs()
                .map(Vec::from_iter)
                .unwrap_or_default();
            tx.send(found).ok();
        });
    }
    drop(tx);
    let deadline = Instant::now() + LOOKUP_TIMEOUT;
    let mut answers = Vec::new();
    while answers.len() < PROBES.len() {
        match rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            Ok(found) => answers.push(found),
            // 查不到或等不到：按普通网络算，插件照严格的规则来。
            Err(_) => return false,
        }
    }
    all_fake(&answers)
}

/// 会等 DNS，最多两秒；异步代码里放进 `spawn_blocking`。
/// 调用方：获取插件的调用（经 `USUYO_NETWORK` 告诉插件）、内核下载模型（`usuyo-core` 的 `models`）、
/// 设置页（`GET /settings` 的 `fakeIp`）。
pub fn behind_fake_ip_proxy() -> bool {
    let mut last = LAST.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((at, value)) = *last
        && at.elapsed() < FRESH_FOR
    {
        return value;
    }
    let value = probe();
    *last = Some((Instant::now(), value));
    value
}

/// 按机器的 `settings.json` 里的 `network.proxy`：插件和模型下载怎么连出去。
#[derive(Clone, Default, PartialEq, Eq)]
pub enum ProxySetting {
    /// 跟这台机器的系统代理走（默认）：macOS 读 `scutil --proxy`，别的系统暂时读环境变量。
    #[default]
    System,
    /// 不经代理。
    Direct,
    /// 这个代理：`http://…`、`socks5://…`（`socks5h://` 同义）。
    Url(String),
}

impl ProxySetting {
    /// `system`、`direct`，或一个代理地址；地址不合规回原因（原因里不写地址，可能带密码）。
    pub fn parse(text: &str) -> Result<ProxySetting, String> {
        match text.trim() {
            "system" => Ok(ProxySetting::System),
            "direct" => Ok(ProxySetting::Direct),
            other => proxy_url(other).map(ProxySetting::Url),
        }
    }
}

impl std::fmt::Debug for ProxySetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxySetting::System => f.write_str("System"),
            ProxySetting::Direct => f.write_str("Direct"),
            ProxySetting::Url(_) => f.write_str("Url(<redacted>)"),
        }
    }
}

/// 一个代理地址合不合规：`http`、`socks5`、`socks5h`，有主机，没有路径和查询串。回规整后的写法。
fn proxy_url(text: &str) -> Result<String, String> {
    let url = url::Url::parse(text).map_err(|_| "the proxy is not a URL".to_owned())?;
    if !matches!(url.scheme(), "http" | "socks5" | "socks5h") {
        return Err("the proxy scheme is not http, socks5 or socks5h".into());
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err("the proxy has no host".into());
    }
    if !matches!(url.path(), "" | "/") || url.query().is_some() || url.fragment().is_some() {
        return Err("the proxy has a path or query".into());
    }
    Ok(url.as_str().trim_end_matches('/').to_owned())
}

/// 系统代理最多多久读一次。
const PROXY_FRESH_FOR: Duration = Duration::from_secs(60);
/// `scutil --proxy` 最多等多久。
const SCUTIL_TIMEOUT: Duration = Duration::from_secs(2);

static SYSTEM_PROXY: Mutex<Option<(Instant, Option<String>)>> = Mutex::new(None);

/// 照设置定这一次走哪个代理：`None` 是直连。`system` 最多一分钟读一次系统设置；读不出来按直连，记一行日志。
/// 会起子进程、最多等两秒；异步代码里放进 `spawn_blocking`。
/// 调用方：获取插件的调用（经 `USUYO_PROXY` 交给插件）、内核下载模型（`usuyo-core` 的 `models`）。
pub fn resolve_proxy(setting: &ProxySetting) -> Option<String> {
    match setting {
        ProxySetting::Direct => None,
        ProxySetting::Url(u) => Some(u.clone()),
        ProxySetting::System => {
            let mut last = SYSTEM_PROXY
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some((at, value)) = &*last
                && at.elapsed() < PROXY_FRESH_FOR
            {
                return value.clone();
            }
            let value = system_proxy();
            *last = Some((Instant::now(), value.clone()));
            value
        }
    }
}

/// macOS：`scutil --proxy` 里 https 的代理（CONNECT），没有就 SOCKS，再没有就 http 的。
/// 例外列表和自动配置（PAC）不看。
#[cfg(target_os = "macos")]
fn system_proxy() -> Option<String> {
    use std::process::{Command, Stdio};
    let mut child = match Command::new("/usr/sbin/scutil")
        .arg("--proxy")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "cannot run scutil; going out directly");
            return None;
        }
    };
    let deadline = Instant::now() + SCUTIL_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            Ok(None) => {
                // 停不下来的子进程只能丢下；这时按直连。
                let _ = child.kill();
                let _ = child.wait();
                tracing::warn!("scutil --proxy did not answer in time; going out directly");
                return None;
            }
            Err(e) => {
                tracing::warn!(error = %e, "scutil --proxy failed; going out directly");
                return None;
            }
        }
    }
    let mut out = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        use std::io::Read;
        // 输出就几十行，读到上限为止
        let _ = (&mut stdout).take(64 << 10).read_to_string(&mut out);
    }
    scutil_proxy(&out)
}

/// 别的系统暂时读 `HTTPS_PROXY`、`ALL_PROXY`（大小写都认）；不合规的不用，记一行日志。
#[cfg(not(target_os = "macos"))]
fn system_proxy() -> Option<String> {
    for key in ["HTTPS_PROXY", "https_proxy", "ALL_PROXY", "all_proxy"] {
        if let Ok(v) = std::env::var(key)
            && !v.trim().is_empty()
        {
            match proxy_url(v.trim()) {
                Ok(u) => return Some(u),
                Err(why) => tracing::warn!(variable = key, why, "proxy variable not used"),
            }
        }
    }
    None
}

/// 读 `scutil --proxy` 的输出。
#[cfg(any(target_os = "macos", test))]
fn scutil_proxy(text: &str) -> Option<String> {
    let mut keys = std::collections::HashMap::new();
    for line in text.lines() {
        if let Some((k, v)) = line.split_once(" : ") {
            keys.insert(k.trim().to_owned(), v.trim().to_owned());
        }
    }
    let pick = |prefix: &str, scheme: &str| {
        let on = keys
            .get(&format!("{prefix}Enable"))
            .is_some_and(|v| v == "1");
        let host = keys
            .get(&format!("{prefix}Proxy"))
            .filter(|h| !h.is_empty())?;
        let port = keys.get(&format!("{prefix}Port"))?.parse::<u16>().ok()?;
        let host = if host.contains(':') {
            format!("[{host}]")
        } else {
            host.clone()
        };
        on.then(|| format!("{scheme}://{host}:{port}"))
    };
    pick("HTTPS", "http")
        .or_else(|| pick("SOCKS", "socks5"))
        .or_else(|| pick("HTTP", "http"))
        .and_then(|u| proxy_url(&u).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_scutil_and_the_setting() {
        let out = "<dictionary> {
  ExceptionsList : <array> {
    0 : 127.0.0.1
  }
  HTTPEnable : 1
  HTTPPort : 7979
  HTTPProxy : 127.0.0.1
  HTTPSEnable : 1
  HTTPSPort : 7979
  HTTPSProxy : 127.0.0.1
  SOCKSEnable : 1
  SOCKSPort : 7980
  SOCKSProxy : 127.0.0.1
}
";
        assert_eq!(scutil_proxy(out).as_deref(), Some("http://127.0.0.1:7979"));
        let socks_only = out.replace("HTTPSEnable : 1", "HTTPSEnable : 0");
        assert_eq!(
            scutil_proxy(&socks_only).as_deref(),
            Some("socks5://127.0.0.1:7980")
        );
        let none = "<dictionary> {
  HTTPEnable : 0
  HTTPSEnable : 0
}
";
        assert_eq!(scutil_proxy(none), None);

        assert_eq!(ProxySetting::parse("system"), Ok(ProxySetting::System));
        assert_eq!(ProxySetting::parse("direct"), Ok(ProxySetting::Direct));
        assert_eq!(
            ProxySetting::parse("socks5://127.0.0.1:1080"),
            Ok(ProxySetting::Url("socks5://127.0.0.1:1080".into()))
        );
        let with_password = ProxySetting::parse("http://u:secret@127.0.0.1:8080").unwrap();
        assert!(!format!("{with_password:?}").contains("secret"));
        for bad in ["ftp://x:1", "http://", "http://x:1/p", "nope"] {
            assert!(ProxySetting::parse(bad).is_err(), "{bad}");
        }
        assert_eq!(resolve_proxy(&ProxySetting::Direct), None);
    }

    fn addrs(ips: &[&str]) -> Vec<SocketAddr> {
        ips.iter()
            .map(|ip| SocketAddr::new(ip.parse().expect("test ip"), 80))
            .collect()
    }

    #[test]
    fn only_all_fake_answers_count() {
        assert!(all_fake(&[
            addrs(&["198.18.0.5"]),
            addrs(&["198.19.255.254"])
        ]));
        // 一个域名是公网地址、查不到，或混着别的地址：都不算
        assert!(!all_fake(&[
            addrs(&["198.18.0.5"]),
            addrs(&["93.184.216.34"])
        ]));
        assert!(!all_fake(&[addrs(&["198.18.0.5"]), Vec::new()]));
        assert!(!all_fake(&[addrs(&["198.18.0.5", "10.0.0.1"])]));
        assert!(!all_fake(&[addrs(&["198.17.0.1"])]));
        assert!(!all_fake(&[addrs(&["198.20.0.1"])]));
        assert!(!all_fake(&[]));
    }
}
