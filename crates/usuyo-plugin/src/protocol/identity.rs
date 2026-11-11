//! 身份（协议 §3.5、SPEC §0.4）。

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use super::values::{PlatformId, check_http_url};

/// 条目的身份。三种形状互斥：平台内容 ID、只有链接的网页、没有身份的文件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identity {
    /// 平台内容：`contentId` 必带，`url` 是规范链接，只用于去重和显示。
    Platform {
        platform: PlatformId,
        content_id: String,
        url: Option<String>,
    },
    /// `platform: web`：不带 `contentId`，`url` 必带（协议 §13 第 29 项）。
    Web { url: String },
    /// `platform: file`：不参与身份层，只靠内容去重。
    File,
}

impl Identity {
    pub fn platform(&self) -> &str {
        match self {
            Identity::Platform { platform, .. } => platform.as_str(),
            Identity::Web { .. } => PlatformId::WEB,
            Identity::File => PlatformId::FILE,
        }
    }
}

/// `contentId` 最长多少字节：它进条目 id 的哈希、manifest 和日志。
const CONTENT_ID_BYTES: usize = 256;

/// 线上的形状：`{platform, contentId?, url?}`。
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "Identity")]
struct WireIdentity {
    platform: PlatformId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    content_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

impl TryFrom<WireIdentity> for Identity {
    type Error = String;
    fn try_from(w: WireIdentity) -> Result<Self, String> {
        if let Some(url) = &w.url {
            check_http_url(url).map_err(|e| format!("identity.url: {e}"))?;
        }
        match w.platform.as_str() {
            PlatformId::WEB => match (w.content_id, w.url) {
                (None, Some(url)) => Ok(Identity::Web { url }),
                (Some(_), _) => Err("a `web` identity has no `contentId`".into()),
                (None, None) => Err("a `web` identity needs `url`".into()),
            },
            PlatformId::FILE => match (w.content_id, w.url) {
                (None, None) => Ok(Identity::File),
                _ => Err("a `file` identity has neither `contentId` nor `url`".into()),
            },
            _ => match w.content_id {
                Some(id) if id.len() > CONTENT_ID_BYTES || id.chars().any(char::is_control) => {
                    Err(format!(
                        "`contentId` is over {CONTENT_ID_BYTES} bytes or has control characters"
                    ))
                }
                Some(id) if !id.is_empty() => Ok(Identity::Platform {
                    platform: w.platform,
                    content_id: id,
                    url: w.url,
                }),
                _ => Err(format!(
                    "a `{}` identity needs a non-empty `contentId`",
                    w.platform
                )),
            },
        }
    }
}

impl From<&Identity> for WireIdentity {
    fn from(id: &Identity) -> Self {
        match id {
            Identity::Platform {
                platform,
                content_id,
                url,
            } => WireIdentity {
                platform: platform.clone(),
                content_id: Some(content_id.clone()),
                url: url.clone(),
            },
            Identity::Web { url } => WireIdentity {
                platform: PlatformId(PlatformId::WEB.into()),
                content_id: None,
                url: Some(url.clone()),
            },
            Identity::File => WireIdentity {
                platform: PlatformId(PlatformId::FILE.into()),
                content_id: None,
                url: None,
            },
        }
    }
}

impl Serialize for Identity {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        WireIdentity::from(self).serialize(s)
    }
}

impl<'de> Deserialize<'de> for Identity {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let wire = WireIdentity::deserialize(d)?;
        Identity::try_from(wire).map_err(serde::de::Error::custom)
    }
}

impl JsonSchema for Identity {
    fn schema_name() -> Cow<'static, str> {
        WireIdentity::schema_name()
    }
    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        WireIdentity::json_schema(generator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Result<Identity, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[test]
    fn three_shapes() {
        assert!(matches!(
            parse(r#"{"platform":"douyin","contentId":"7431234567890123456"}"#).unwrap(),
            Identity::Platform { .. }
        ));
        assert_eq!(
            parse(r#"{"platform":"web","url":"https://example.com/a"}"#).unwrap(),
            Identity::Web {
                url: "https://example.com/a".into()
            }
        );
        assert_eq!(parse(r#"{"platform":"file"}"#).unwrap(), Identity::File);
    }

    #[test]
    fn rejects_mixed_shapes() {
        for bad in [
            r#"{"platform":"douyin"}"#,
            r#"{"platform":"web"}"#,
            r#"{"platform":"web","contentId":"1","url":"https://a.b/"}"#,
            r#"{"platform":"file","url":"https://a.b/"}"#,
            r#"{"platform":"web","url":"file:///etc/passwd"}"#,
            r#"{"platform":"Douyin","contentId":"1"}"#,
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn round_trips() {
        let id = Identity::Web {
            url: "https://example.com/a".into(),
        };
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#"{"platform":"web","url":"https://example.com/a"}"#);
        assert_eq!(parse(&json).unwrap(), id);
    }
}
