//! id（SPEC §0.4）：条目 id 由身份算，别的用 UUIDv7，块 id 和脚注标签是 5 位 base32。

use std::fmt;

use usuyo_contract::Identity;

use crate::hash::sha256_bytes;
use crate::{Error, Result};

const ALPHABET: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";

/// RFC 4648 base32，小写，不补 `=`。
pub fn base32(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(5) * 8);
    let (mut buf, mut bits) = (0u32, 0u32);
    for &b in bytes {
        buf = (buf << 8) | u32::from(b);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(ALPHABET[((buf >> bits) & 31) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(ALPHABET[((buf << (5 - bits)) & 31) as usize] as char);
    }
    out
}

/// 系统随机源的字节；失败回 `internal`。
pub(crate) fn random_bytes<const N: usize>() -> Result<[u8; N]> {
    let mut b = [0u8; N];
    getrandom::fill(&mut b).map_err(|e| Error::Random(e.to_string()))?;
    Ok(b)
}

/// UUIDv7 字符串。
pub fn uuid7() -> String {
    uuid::Uuid::now_v7().to_string()
}

/// 是不是 UUIDv7 的写法。
pub fn is_uuid7(s: &str) -> bool {
    uuid::Uuid::try_parse(s).is_ok_and(|u| u.get_version_num() == 7 && u.to_string() == s)
}

/// 5 位随机小写 base32：块 id 和脚注标签。
pub fn block_id() -> Result<String> {
    Ok(base32(&random_bytes::<4>()?)[..5].to_owned())
}

pub fn is_block_id(s: &str) -> bool {
    s.len() == 5 && s.bytes().all(|c| ALPHABET.contains(&c))
}

fn is_platform(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty()
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

/// 条目 id：`<platform>-<26 位 base32>`。按最后一个 `-` 拆。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId(String);

impl ItemId {
    /// 校验写法；不合的回 `invalid-args`，所以也挡住了路径穿越。
    pub fn parse(s: &str) -> Result<ItemId> {
        let ok = s.rsplit_once('-').is_some_and(|(p, h)| {
            is_platform(p) && h.len() == 26 && h.bytes().all(|c| ALPHABET.contains(&c))
        });
        if ok {
            Ok(ItemId(s.to_owned()))
        } else {
            Err(Error::InvalidArgs(format!("bad item id `{s}`")))
        }
    }

    /// 由身份算；`file` 和没有身份的另用 [`ItemId::random_file`]。
    pub fn from_identity(identity: &Identity) -> Result<ItemId> {
        let key = identity_key(identity)?;
        let digest = sha256_bytes(key.as_bytes());
        Ok(ItemId(format!(
            "{}-{}",
            identity.platform,
            base32(&digest[..16])
        )))
    }

    /// `file-` 加 26 位随机 base32。
    pub fn random_file() -> Result<ItemId> {
        Ok(ItemId(format!("file-{}", base32(&random_bytes::<16>()?))))
    }

    pub fn platform(&self) -> &str {
        self.0.rsplit_once('-').map_or("", |(p, _)| p)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 从库根起的文件夹：`items/<platform>/<itemId>`。
    pub fn rel_dir(&self) -> String {
        format!("items/{}/{}", self.platform(), self.0)
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// `identityKey`：`<platform>:<contentId>`，web 是 `web:<规范化的 url>`。
/// `file` 和缺字段的身份不算身份，回 `invalid-args`。
pub fn identity_key(identity: &Identity) -> Result<String> {
    if !is_platform(&identity.platform) || identity.platform == "file" {
        return Err(Error::InvalidArgs(format!(
            "`{}` is not an identity platform",
            identity.platform
        )));
    }
    match (&identity.content_id, &identity.url) {
        (Some(c), _) if !c.is_empty() => Ok(format!("{}:{}", identity.platform, c)),
        (_, Some(u)) if identity.platform == "web" => Ok(format!("web:{}", normalize_web_url(u)?)),
        _ => Err(Error::InvalidArgs(format!(
            "identity on `{}` needs contentId",
            identity.platform
        ))),
    }
}

/// web 身份的宿主规范化：scheme、主机小写，去默认端口和 `#` 片段；路径和查询串不动。
pub fn normalize_web_url(url: &str) -> Result<String> {
    let bad = || Error::InvalidArgs(format!("not an http(s) url: `{url}`"));
    let (scheme, rest) = url.split_once("://").ok_or_else(bad)?;
    let scheme = scheme.to_ascii_lowercase();
    let default_port = match scheme.as_str() {
        "http" => "80",
        "https" => "443",
        _ => return Err(bad()),
    };
    let rest = rest.split_once('#').map_or(rest, |(r, _)| r);
    let split = rest.find(['/', '?']).unwrap_or(rest.len());
    let (authority, tail) = rest.split_at(split);
    let (userinfo, hostport) = match authority.rsplit_once('@') {
        Some((u, h)) => (Some(u), h),
        None => (None, authority),
    };
    // IPv6 字面量里有冒号，端口只看 `]` 之后。
    let port_at = match hostport.rfind(']') {
        Some(close) => hostport[close..].find(':').map(|i| close + i),
        None => hostport.rfind(':'),
    };
    let (host, port) = match port_at {
        Some(i) => (&hostport[..i], Some(&hostport[i + 1..])),
        None => (hostport, None),
    };
    if host.is_empty() {
        return Err(bad());
    }
    let mut out = format!("{scheme}://");
    if let Some(u) = userinfo {
        out.push_str(u);
        out.push('@');
    }
    out.push_str(&host.to_ascii_lowercase());
    if let Some(p) = port.filter(|p| !p.is_empty() && *p != default_port) {
        out.push(':');
        out.push_str(p);
    }
    // 空路径和 `/` 是同一个资源（RFC 3986 §6.2.3），统一写成 `/`。
    if !tail.starts_with('/') {
        out.push('/');
    }
    out.push_str(tail);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use usuyo_contract::Extra;

    fn ident(platform: &str, content_id: Option<&str>, url: Option<&str>) -> Identity {
        Identity {
            platform: platform.into(),
            content_id: content_id.map(Into::into),
            url: url.map(Into::into),
            extra: Extra::new(),
        }
    }

    #[test]
    fn spec_example() {
        let id =
            ItemId::from_identity(&ident("douyin", Some("7431234567890123456"), None)).unwrap();
        assert_eq!(id.as_str(), "douyin-4plrcxk343onmyxibmrr672kga");
        assert_eq!(id.platform(), "douyin");
        assert_eq!(
            id.rel_dir(),
            "items/douyin/douyin-4plrcxk343onmyxibmrr672kga"
        );
    }

    #[test]
    fn hyphenated_platform_splits_on_last_dash() {
        let id = ItemId::from_identity(&ident("wechat-mp", Some("AbC"), None)).unwrap();
        assert_eq!(id.platform(), "wechat-mp");
        assert_eq!(ItemId::parse(id.as_str()).unwrap(), id);
    }

    #[test]
    fn base32_rfc4648_vectors() {
        assert_eq!(base32(b"f"), "my");
        assert_eq!(base32(b"foobar"), "mzxw6ytboi");
    }

    #[test]
    fn web_identity_is_normalized() {
        let a = ItemId::from_identity(&ident(
            "web",
            None,
            Some("HTTPS://Example.COM:443/a/B?q=1#frag"),
        ))
        .unwrap();
        let b = ItemId::from_identity(&ident("web", None, Some("https://example.com/a/B?q=1")))
            .unwrap();
        assert_eq!(a, b);
        assert_eq!(
            normalize_web_url("http://u:p@Host.example:8080/x?Y=Z#f").unwrap(),
            "http://u:p@host.example:8080/x?Y=Z"
        );
        assert_eq!(
            normalize_web_url("http://[::1]:80/").unwrap(),
            "http://[::1]/"
        );
        assert_eq!(
            normalize_web_url("https://Example.com").unwrap(),
            "https://example.com/"
        );
        assert_eq!(
            normalize_web_url("https://example.com?q=1").unwrap(),
            "https://example.com/?q=1"
        );
        assert!(normalize_web_url("ftp://example.com/").is_err());
        assert!(normalize_web_url("https:///nohost").is_err());
    }

    #[test]
    fn identities_that_are_not_identities() {
        assert!(identity_key(&ident("file", None, None)).is_err());
        assert!(identity_key(&ident("douyin", None, Some("https://x"))).is_err());
        assert!(identity_key(&ident("../etc", Some("1"), None)).is_err());
    }

    #[test]
    fn random_ids_have_the_right_shape() {
        let f = ItemId::random_file().unwrap();
        assert!(ItemId::parse(f.as_str()).is_ok());
        assert_eq!(f.platform(), "file");
        assert!(is_block_id(&block_id().unwrap()));
        assert!(is_uuid7(&uuid7()));
        assert!(ItemId::parse("douyin-../../x").is_err());
    }
}
