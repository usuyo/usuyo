//! 登录窗口里读到的 cookie：哪些属于这个站点、什么时候算登录好了、每个站点的网页数据放哪。
//! 规则和内核一致（`usuyo-plugin` 的 `belongs`）：内核收到以后还会再筛一遍。
//!
//! 值是秘密：`Debug` 只写名字和域名。

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// `GET /sites` 里的一项，只取登录窗口要的。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteSpec {
    pub site: String,
    pub name: String,
    #[serde(default)]
    pub login_url: Option<String>,
    /// 登录后才有的 cookie 名；空的要等你点「登录好了」。
    #[serde(default)]
    pub cookies: Vec<String>,
    #[serde(default)]
    pub domains: Vec<String>,
}

/// 交给内核的一个 cookie：`StorageState` 里的形状（endpoints.ts 的 `StoredCookie`）。
#[derive(Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Jar {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    /// Unix 秒；会话 cookie 是 -1。
    pub expires: f64,
    pub http_only: bool,
    pub secure: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub same_site: Option<&'static str>,
}

impl fmt::Debug for Jar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Jar")
            .field("name", &self.name)
            .field("domain", &self.domain)
            .finish_non_exhaustive()
    }
}

impl Jar {
    fn expired_at(&self, now: f64) -> bool {
        self.expires > 0.0 && self.expires <= now
    }
}

/// 这个 cookie 的域名属不属于站点：站点本身和它的上级域名（不含公共后缀），加上 `domains` 和它们的子域。
pub fn belongs(site: &str, domains: &[String], domain: &str) -> bool {
    let d = domain.trim_start_matches('.').to_ascii_lowercase();
    if d.is_empty() {
        return false;
    }
    let host = site.to_ascii_lowercase();
    let public_suffix = psl::suffix_str(&d) == Some(d.as_str());
    if !public_suffix && (host == d || host.ends_with(&format!(".{d}"))) {
        return true;
    }
    domains.iter().any(|x| {
        let x = x.trim_start_matches('.').to_ascii_lowercase();
        !x.is_empty() && (d == x || d.ends_with(&format!(".{x}")))
    })
}

/// 只留属于这个站点、没过期的。
pub fn select(site: &str, domains: &[String], all: Vec<Jar>, now: f64) -> Vec<Jar> {
    all.into_iter()
        .filter(|c| belongs(site, domains, &c.domain) && !c.expired_at(now))
        .collect()
}

/// 声明过的 cookie 名都在（值不空）就算登录好了；没声明名字的站点不自己判断，等你点「登录好了」。
pub fn logged_in(names: &[String], selected: &[Jar]) -> bool {
    !names.is_empty()
        && names
            .iter()
            .all(|n| selected.iter().any(|c| &c.name == n && !c.value.is_empty()))
}

/// 请求头里 `Cookie:` 后面那一行：`a=1; b=2`。同名的只留第一个。
pub fn cookie_line(selected: &[Jar]) -> String {
    let mut seen = Vec::new();
    let mut parts = Vec::new();
    for c in selected {
        if c.value.is_empty() || seen.contains(&c.name.as_str()) {
            continue;
        }
        seen.push(c.name.as_str());
        parts.push(format!("{}={}", c.name, c.value));
    }
    parts.join("; ")
}

/// 每个站点自己一份网页数据：macOS 上是数据仓库的标识，别的系统上是目录名。按名字算，下次还是同一份。
pub fn store_id(store: &str) -> [u8; 16] {
    let digest = Sha256::digest(format!("usuyo-login-store/v1\n{store}").as_bytes());
    let mut id = [0u8; 16];
    id.copy_from_slice(&digest[..16]);
    // 标成 UUID v8，免得哪一层要它像个 UUID。
    id[6] = (id[6] & 0x0f) | 0x80;
    id[8] = (id[8] & 0x3f) | 0x80;
    id
}

#[cfg(not(target_vendor = "apple"))]
pub fn store_dir_name(store: &str) -> String {
    store_id(store).iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jar(name: &str, value: &str, domain: &str) -> Jar {
        Jar {
            name: name.into(),
            value: value.into(),
            domain: domain.into(),
            path: "/".into(),
            expires: -1.0,
            http_only: false,
            secure: true,
            same_site: None,
        }
    }

    fn names(v: &[Jar]) -> Vec<&str> {
        v.iter().map(|c| c.name.as_str()).collect()
    }

    #[test]
    fn keeps_the_site_its_parents_and_declared_domains_only() {
        let domains = vec![".bilibili.com".to_owned()];
        let all = vec![
            jar("SESSDATA", "s", ".bilibili.com"),
            jar("host", "h", "www.bilibili.com"),
            jar("passport", "p", "passport.bilibili.com"),
            jar("other", "o", ".example.com"),
            jar("tld", "t", ".com"),
            jar("lookalike", "l", "evilbilibili.com"),
        ];
        let kept = select("www.bilibili.com", &domains, all, 0.0);
        assert_eq!(names(&kept), ["SESSDATA", "host", "passport"]);
    }

    #[test]
    fn parent_matching_never_reaches_a_public_suffix() {
        assert!(belongs("yuanbao.tencent.com", &[], ".tencent.com"));
        assert!(belongs("yuanbao.tencent.com", &[], "yuanbao.tencent.com"));
        assert!(!belongs("yuanbao.tencent.com", &[], ".com"));
        assert!(!belongs("www.example.co.uk", &[], "co.uk"));
        assert!(!belongs("yuanbao.tencent.com", &[], "qq.com"));
        assert!(!belongs("yuanbao.tencent.com", &[], ""));
        // 没声明 domains 时，同级的别的子域不算。
        assert!(!belongs("yuanbao.tencent.com", &[], "im.tencent.com"));
    }

    #[test]
    fn expired_cookies_are_dropped() {
        let mut old = jar("SESSDATA", "s", ".bilibili.com");
        old.expires = 100.0;
        let mut fresh = jar("bili_jct", "j", ".bilibili.com");
        fresh.expires = 10_000.0;
        let kept = select("www.bilibili.com", &[], vec![old, fresh], 5_000.0);
        assert_eq!(names(&kept), ["bili_jct"]);
    }

    #[test]
    fn logged_in_needs_every_declared_name_with_a_value() {
        let want = vec!["SESSDATA".to_owned(), "bili_jct".to_owned()];
        let one = vec![jar("SESSDATA", "s", ".bilibili.com")];
        assert!(!logged_in(&want, &one));
        let both = vec![
            jar("SESSDATA", "s", ".bilibili.com"),
            jar("bili_jct", "j", ".bilibili.com"),
        ];
        assert!(logged_in(&want, &both));
        let empty_value = vec![
            jar("SESSDATA", "", ".bilibili.com"),
            jar("bili_jct", "j", ".bilibili.com"),
        ];
        assert!(!logged_in(&want, &empty_value));
        // 没声明名字：窗口不自己判断。
        assert!(!logged_in(&[], &both));
    }

    #[test]
    fn cookie_line_joins_pairs_once() {
        let v = vec![
            jar("a1", "x", ".xiaohongshu.com"),
            jar("webId", "y", ".xiaohongshu.com"),
            jar("a1", "z", "www.xiaohongshu.com"),
            jar("blank", "", ".xiaohongshu.com"),
        ];
        assert_eq!(cookie_line(&v), "a1=x; webId=y");
    }

    #[test]
    fn values_stay_out_of_debug() {
        let c = jar("SESSDATA", "secret-value", ".bilibili.com");
        assert!(!format!("{c:?}").contains("secret-value"));
        let json = serde_json::to_string(&c).unwrap();
        assert!(json.contains(r#""httpOnly":false"#), "{json}");
        assert!(!json.contains("sameSite"), "{json}");
    }

    #[test]
    fn each_site_gets_its_own_stable_store() {
        assert_eq!(store_id("www.bilibili.com"), store_id("www.bilibili.com"));
        assert_ne!(
            store_id("www.bilibili.com"),
            store_id("yuanbao.tencent.com")
        );
        assert_ne!(
            store_id("www.xiaohongshu.com"),
            store_id("visitor:usuyo-fetch/xiaohongshu")
        );
        #[cfg(not(target_vendor = "apple"))]
        assert_eq!(store_dir_name("www.bilibili.com").len(), 32);
    }
}
