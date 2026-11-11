//! 凭据文件的形状（协议 §6）：`USUYO_CREDENTIALS` 指的 JSON，`storageState` 照 Playwright。
//!
//! 值是秘密：`Debug` 只写名字和域名，不写值，免得进日志。

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 一个 cookie。`expires` 是 Unix 秒，`-1` 是会话 cookie。
#[derive(Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    #[serde(default = "root_path")]
    pub path: String,
    #[serde(default = "session")]
    pub expires: f64,
    #[serde(default)]
    pub http_only: bool,
    #[serde(default)]
    pub secure: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub same_site: Option<SameSite>,
}

fn root_path() -> String {
    "/".into()
}

fn session() -> f64 {
    -1.0
}

impl Cookie {
    /// 有过期时间、而且已经过了。
    pub fn expired_at(&self, now_secs: f64) -> bool {
        self.expires > 0.0 && self.expires <= now_secs
    }
}

impl fmt::Debug for Cookie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cookie")
            .field("name", &self.name)
            .field("domain", &self.domain)
            .field("path", &self.path)
            .field("expires", &self.expires)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

/// 一个来源的 localStorage。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OriginState {
    pub origin: String,
    #[serde(default)]
    pub local_storage: Vec<StorageEntry>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StorageEntry {
    pub name: String,
    pub value: String,
}

impl fmt::Debug for StorageEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StorageEntry")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

/// Playwright 的 storageState：cookie 加各来源的 localStorage。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StorageState {
    #[serde(default)]
    pub cookies: Vec<Cookie>,
    #[serde(default)]
    pub origins: Vec<OriginState>,
}

/// 凭据文件里的一个站点和它的来源（`login-window`、`cookie-file`、`daily-browser`）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CredentialSite {
    pub site: String,
    pub source: String,
}

/// `USUYO_CREDENTIALS` 指的文件。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Credentials {
    pub sites: Vec<CredentialSite>,
    pub storage_state: StorageState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_shows_values() {
        let c: Cookie = serde_json::from_str(
            r#"{"name":"SESSDATA","value":"very-secret","domain":".bilibili.com"}"#,
        )
        .unwrap();
        assert_eq!((c.path.as_str(), c.expires), ("/", -1.0));
        let s = StorageState {
            cookies: vec![c],
            origins: vec![OriginState {
                origin: "https://www.bilibili.com".into(),
                local_storage: vec![StorageEntry {
                    name: "k".into(),
                    value: "also-secret".into(),
                }],
            }],
        };
        let shown = format!("{s:?}");
        assert!(
            shown.contains("SESSDATA") && !shown.contains("secret"),
            "{shown}"
        );
    }
}
