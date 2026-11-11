//! 协议里反复出现的小值：时间、时长、大小、相对路径、各种 id。
//! 每个都在反序列化时校验一次，之后按类型信任。

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use jiff::Timestamp;
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};

macro_rules! string_schema {
    ($ty:ty, $name:literal, $schema:tt) => {
        impl JsonSchema for $ty {
            fn schema_name() -> Cow<'static, str> {
                $name.into()
            }
            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!($schema)
            }
        }
    };
}

/// RFC 3339 带偏移的时间（协议 §13 第 30 项）。保留原文，宿主要另存原样的偏移。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Rfc3339 {
    text: String,
    at: Timestamp,
}

impl Rfc3339 {
    /// 用 UTC 写出，例如插件填 `evidence.fetchedAt`。
    pub fn from_timestamp(at: Timestamp) -> Self {
        Self {
            text: at.to_string(),
            at,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn timestamp(&self) -> Timestamp {
        self.at
    }
}

impl TryFrom<String> for Rfc3339 {
    type Error = String;
    fn try_from(text: String) -> Result<Self, String> {
        // 只认 `T` 分隔、带偏移的写法；jiff 还认空格分隔和方括号时区，这里收紧
        let bytes = text.as_bytes();
        if bytes.len() < 20 || !matches!(bytes[10], b'T' | b't') || text.contains('[') {
            return Err(format!("`{text}` is not an RFC 3339 time with offset"));
        }
        let at = Timestamp::from_str(&text)
            .map_err(|e| format!("`{text}` is not an RFC 3339 time with offset: {e}"))?;
        Ok(Self { text, at })
    }
}

impl From<Rfc3339> for String {
    fn from(v: Rfc3339) -> String {
        v.text
    }
}

impl fmt::Display for Rfc3339 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

string_schema!(Rfc3339, "Rfc3339", {"type": "string", "format": "date-time"});

/// 清单里的时长：`8s`、`10m`、`2h`、`300ms`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DurationSpec(Duration);

impl DurationSpec {
    pub fn get(self) -> Duration {
        self.0
    }

    pub(crate) fn at_most(self, max: Duration) -> Self {
        Self(self.0.min(max))
    }
}

impl TryFrom<String> for DurationSpec {
    type Error = String;
    fn try_from(text: String) -> Result<Self, String> {
        let (num, unit) = split_number(&text)
            .ok_or_else(|| format!("`{text}` is not a duration like `8s` or `10m`"))?;
        let d = match unit {
            "ms" => Duration::from_millis(num),
            "s" => Duration::from_secs(num),
            "m" => Duration::from_secs(num.saturating_mul(60)),
            "h" => Duration::from_secs(num.saturating_mul(3600)),
            _ => return Err(format!("`{text}`: unit must be ms, s, m or h")),
        };
        Ok(Self(d))
    }
}

impl From<DurationSpec> for String {
    fn from(v: DurationSpec) -> String {
        format!("{}ms", v.0.as_millis())
    }
}

string_schema!(DurationSpec, "Duration", {"type": "string", "pattern": "^[0-9]+(ms|s|m|h)$"});

/// 清单里的大小：`512MiB`、`1GiB`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ByteSize(u64);

impl ByteSize {
    pub fn bytes(self) -> u64 {
        self.0
    }
}

impl TryFrom<String> for ByteSize {
    type Error = String;
    fn try_from(text: String) -> Result<Self, String> {
        let (num, unit) =
            split_number(&text).ok_or_else(|| format!("`{text}` is not a size like `512MiB`"))?;
        let shift = match unit {
            "B" => 0,
            "KiB" => 10,
            "MiB" => 20,
            "GiB" => 30,
            "TiB" => 40,
            _ => return Err(format!("`{text}`: unit must be B, KiB, MiB, GiB or TiB")),
        };
        num.checked_mul(1u64 << shift)
            .map(Self)
            .ok_or_else(|| format!("`{text}` is too large"))
    }
}

impl From<ByteSize> for String {
    fn from(v: ByteSize) -> String {
        format!("{}B", v.0)
    }
}

string_schema!(ByteSize, "ByteSize", {"type": "string", "pattern": "^[0-9]+(B|KiB|MiB|GiB|TiB)$"});

fn split_number(text: &str) -> Option<(u64, &str)> {
    let end = text.find(|c: char| !c.is_ascii_digit())?;
    if end == 0 {
        return None;
    }
    Some((text[..end].parse().ok()?, &text[end..]))
}

/// 输出目录里的相对路径，`/` 分隔，不含 `..`、空段、反斜杠，不以 `/` 开头（协议 §3.6）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RelPath(String);

impl RelPath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RelPath {
    type Error = String;
    fn try_from(text: String) -> Result<Self, String> {
        let bad = text.is_empty()
            || text.contains('\\')
            || text.chars().any(char::is_control)
            || text
                .split('/')
                .any(|seg| seg.is_empty() || seg == "." || seg == "..");
        if bad {
            return Err(format!(
                "`{text}` is not a relative path inside the output directory"
            ));
        }
        Ok(Self(text))
    }
}

impl TryFrom<&str> for RelPath {
    type Error = String;
    fn try_from(text: &str) -> Result<Self, String> {
        Self::try_from(text.to_owned())
    }
}

impl From<RelPath> for String {
    fn from(v: RelPath) -> String {
        v.0
    }
}

impl fmt::Display for RelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

string_schema!(RelPath, "RelPath", {"type": "string", "pattern": "^(?!/)(?!.*(^|/)\\.{1,2}(/|$))(?!.*//)[^\\\\]+$"});

/// 秒数：有限、不小于 0。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct Seconds(f64);

impl Seconds {
    pub fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for Seconds {
    type Error = String;
    fn try_from(v: f64) -> Result<Self, String> {
        if v.is_finite() && v >= 0.0 {
            Ok(Self(v))
        } else {
            Err(format!("{v} is not a number of seconds"))
        }
    }
}

impl From<Seconds> for f64 {
    fn from(v: Seconds) -> f64 {
        v.0
    }
}

string_schema!(Seconds, "Seconds", {"type": "number", "minimum": 0});

/// fetch 输出 `media[]` 的 `group`：同一个字符串表示同一段时间线拆开的几个文件（协议 §3.7）。
/// 1–64 个字节，只有字母、数字和 `. _ : -`。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct MediaGroup(String);

impl MediaGroup {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for MediaGroup {
    type Error = String;
    fn try_from(text: String) -> Result<Self, String> {
        let ok = !text.is_empty()
            && text.len() <= 64
            && text
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b));
        if ok {
            Ok(Self(text))
        } else {
            Err(format!("`{text}` is not 1-64 letters, digits or `._:-`"))
        }
    }
}

impl TryFrom<&str> for MediaGroup {
    type Error = String;
    fn try_from(text: &str) -> Result<Self, String> {
        Self::try_from(text.to_owned())
    }
}

impl From<MediaGroup> for String {
    fn from(v: MediaGroup) -> String {
        v.0
    }
}

string_schema!(MediaGroup, "MediaGroup", {"type": "string", "pattern": "^[A-Za-z0-9._:-]{1,64}$"});

/// 插件 id：GitHub 的 `owner/repo`，全小写；`builtin/` 留给预装插件（协议 §2.2）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PluginId(String);

impl PluginId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn owner(&self) -> &str {
        self.0.split_once('/').map_or("", |(o, _)| o)
    }

    pub fn repo(&self) -> &str {
        self.0.split_once('/').map_or("", |(_, r)| r)
    }

    pub fn is_builtin(&self) -> bool {
        self.owner() == "builtin"
    }
}

impl TryFrom<String> for PluginId {
    type Error = String;
    fn try_from(text: String) -> Result<Self, String> {
        let ok = text.split_once('/').is_some_and(|(owner, repo)| {
            let owner_ok = !owner.is_empty()
                && owner.len() <= 39
                && owner
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                && !owner.starts_with('-');
            let repo_ok = !repo.is_empty()
                && repo.len() <= 100
                && repo.bytes().all(|b| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_' | b'.')
                })
                && !repo.starts_with('.');
            owner_ok && repo_ok
        });
        if !ok {
            return Err(format!(
                "`{text}` is not a lowercase GitHub `owner/repo` id"
            ));
        }
        Ok(Self(text))
    }
}

impl From<PluginId> for String {
    fn from(v: PluginId) -> String {
        v.0
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

string_schema!(PluginId, "PluginId", {"type": "string", "pattern": "^[a-z0-9][a-z0-9-]{0,38}/[a-z0-9_-][a-z0-9._-]{0,99}$"});

/// 平台 id，例如 `web`（协议 §2.3）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PlatformId(pub(crate) String);

impl PlatformId {
    /// 通用网页的身份只带链接（协议 §13 第 29 项）。
    pub const WEB: &'static str = "web";
    /// 没有平台身份的条目（协议 §3.7）。
    pub const FILE: &'static str = "file";

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PlatformId {
    type Error = String;
    fn try_from(text: String) -> Result<Self, String> {
        let ok = text
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
            && text
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if !ok {
            return Err(format!("`{text}` is not a platform id like `douyin`"));
        }
        Ok(Self(text))
    }
}

impl TryFrom<&str> for PlatformId {
    type Error = String;
    fn try_from(text: &str) -> Result<Self, String> {
        Self::try_from(text.to_owned())
    }
}

impl From<PlatformId> for String {
    fn from(v: PlatformId) -> String {
        v.0
    }
}

impl fmt::Display for PlatformId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

string_schema!(PlatformId, "PlatformId", {"type": "string", "pattern": "^[a-z0-9][a-z0-9-]*$"});

/// 插件交来的链接最长多少字节。
const URL_BYTES: usize = 4096;

/// 能解析的 http(s) 绝对链接；原文保留，不做规范化（规范化归资料库，SPEC §0.4）。
/// 原文会存下来，所以也不许超长、不许带控制字符（解析时会被悄悄去掉的换行、制表也算）。
pub(crate) fn check_http_url(text: &str) -> Result<url::Url, String> {
    if text.len() > URL_BYTES {
        return Err(format!("a URL is longer than {URL_BYTES} bytes"));
    }
    if text.chars().any(char::is_control) {
        return Err("a URL has control characters".into());
    }
    let url = url::Url::parse(text).map_err(|e| format!("`{text}` is not a URL: {e}"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(format!("`{text}` is not an http(s) URL"));
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_and_sizes() {
        assert_eq!(
            DurationSpec::try_from("10m".to_owned()).unwrap().get(),
            Duration::from_secs(600)
        );
        assert!(DurationSpec::try_from("10".to_owned()).is_err());
        assert!(DurationSpec::try_from("1d".to_owned()).is_err());
        assert_eq!(
            ByteSize::try_from("1GiB".to_owned()).unwrap().bytes(),
            1 << 30
        );
        assert!(ByteSize::try_from("1GB".to_owned()).is_err());
    }

    #[test]
    fn times_need_offset() {
        assert!(Rfc3339::try_from("2026-10-31T22:14:00+08:00".to_owned()).is_ok());
        assert!(Rfc3339::try_from("2026-10-31T22:14:00".to_owned()).is_err());
        assert!(Rfc3339::try_from("2026-10-31 22:14:00Z".to_owned()).is_err());
        assert!(Rfc3339::try_from("1793000000".to_owned()).is_err());
    }

    #[test]
    fn rel_paths() {
        assert!(RelPath::try_from("images/001.jpg").is_ok());
        for bad in ["", "/etc/passwd", "a/../b", "..", "a//b", "a\\b", "./a"] {
            assert!(RelPath::try_from(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn plugin_ids() {
        assert!(PluginId::try_from("someone/usuyo-fetch-douyin".to_owned()).is_ok());
        assert!(
            PluginId::try_from("builtin/web".to_owned())
                .unwrap()
                .is_builtin()
        );
        assert!(PluginId::try_from("Someone/x".to_owned()).is_err());
        assert!(PluginId::try_from("someone".to_owned()).is_err());
        assert!(PluginId::try_from("a/b/c".to_owned()).is_err());
    }
}
