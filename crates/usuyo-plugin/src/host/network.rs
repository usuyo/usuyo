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
pub(crate) fn behind_fake_ip_proxy() -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

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
