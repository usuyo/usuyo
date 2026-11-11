//! 获取插件各子命令的输出：canonicalize（§3.5）、fetch（§3.7）、sources 和 list（§3.8）。

use std::borrow::Cow;
use std::fmt;

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use super::identity::Identity;
use super::manifest::Access;
use super::values::{MediaGroup, RelPath, Rfc3339, check_http_url};

/// `outputs` 的固定词表（协议 §2.3；`comments` 照 §13 第 31 项）。
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub enum OutputKind {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "images")]
    Images,
    #[serde(rename = "media:video")]
    Video,
    #[serde(rename = "media:audio")]
    Audio,
    #[serde(rename = "subtitle")]
    Subtitle,
    #[serde(rename = "meta")]
    Meta,
    #[serde(rename = "comments")]
    Comments,
}

impl OutputKind {
    pub fn as_str(self) -> &'static str {
        match self {
            OutputKind::Text => "text",
            OutputKind::Html => "html",
            OutputKind::Images => "images",
            OutputKind::Video => "media:video",
            OutputKind::Audio => "media:audio",
            OutputKind::Subtitle => "subtitle",
            OutputKind::Meta => "meta",
            OutputKind::Comments => "comments",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [
            OutputKind::Text,
            OutputKind::Html,
            OutputKind::Images,
            OutputKind::Video,
            OutputKind::Audio,
            OutputKind::Subtitle,
            OutputKind::Meta,
            OutputKind::Comments,
        ]
        .into_iter()
        .find(|k| k.as_str() == text)
    }
}

impl fmt::Display for OutputKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 条目种类（沿用 v0 的词表）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ItemKind {
    Article,
    Post,
    Image,
    Video,
    File,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Completeness {
    Link,
    Partial,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TextFormat {
    #[default]
    Plain,
    Markdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MediaKind {
    Video,
    Audio,
    Subtitle,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub path: RelPath,
    pub kind: MediaKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_sec: Option<f64>,
    /// 同一个值表示同一段时间线拆开的几个文件，例如分开存的画面和音轨（协议 §13 第 38 项）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<MediaGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct UpstreamVersion {
    pub name: String,
    pub version: String,
}

/// 这次调用实际怎么拿到的。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub fetched_at: Rfc3339,
    /// 这次实际用到的那一层。
    pub access: Access,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream: Option<UpstreamVersion>,
    /// 只在 HTTP 类请求时填。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
}

/// fetch 的输出（协议 §3.7）。路径都相对 `--out`；数组总是出现。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FetchOutput {
    /// 省略等于 `{"platform": "file"}`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<Identity>,
    pub kind: ItemKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<Rfc3339>,
    /// 平台原文的文字。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_format: Option<TextFormat>,
    /// 原始 HTML 文件。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html: Option<RelPath>,
    pub images: Vec<RelPath>,
    pub media: Vec<Media>,
    pub platform_tags: Vec<String>,
    /// 固定是 `comments.json`（协议 §13 第 31 项）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comments: Option<RelPath>,
    pub completeness: Completeness,
    pub missing: Vec<OutputKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing_reason: Option<String>,
    pub evidence: Evidence,
    /// 调用期间平台换了会话 cookie 时交回（§6，`storageState` 的形状；凭据还没接，原样带着）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credentials_update: Option<serde_json::Value>,
}

impl FetchOutput {
    pub const COMMENTS_FILE: &'static str = "comments.json";

    /// 输出里实际出现了哪些（协议 §3.1「完整度只升不降」比的就是它）。
    /// `meta` 算标题、作者、发布时间、平台标签里有任何一样。
    pub fn obtained(&self) -> std::collections::BTreeSet<OutputKind> {
        let mut set = std::collections::BTreeSet::new();
        if self.text.as_deref().is_some_and(|t| !t.trim().is_empty()) {
            set.insert(OutputKind::Text);
        }
        if self.html.is_some() {
            set.insert(OutputKind::Html);
        }
        if !self.images.is_empty() {
            set.insert(OutputKind::Images);
        }
        for m in &self.media {
            match m.kind {
                MediaKind::Video => set.insert(OutputKind::Video),
                MediaKind::Audio => set.insert(OutputKind::Audio),
                MediaKind::Subtitle => set.insert(OutputKind::Subtitle),
                MediaKind::Other => false,
            };
        }
        if self.title.is_some()
            || self.author.is_some()
            || self.published_at.is_some()
            || !self.platform_tags.is_empty()
        {
            set.insert(OutputKind::Meta);
        }
        if self.comments.is_some() {
            set.insert(OutputKind::Comments);
        }
        set
    }

    /// 输出里引用的所有文件。
    pub fn referenced_files(&self) -> impl Iterator<Item = &RelPath> {
        self.html
            .iter()
            .chain(&self.images)
            .chain(self.media.iter().map(|m| &m.path))
            .chain(&self.comments)
    }
}

/// canonicalize 的输出（协议 §3.5）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Canonicalized {
    /// 不是这个插件的链接，不是错误。
    NoMatch,
    Match {
        identity: Identity,
        /// 展开短链后的完整链接，保留全部参数；有它时交给 fetch 的是它。
        resolved_url: Option<String>,
    },
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "Canonicalized")]
struct WireCanonicalized {
    #[serde(rename = "match")]
    matched: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    identity: Option<Identity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    resolved_url: Option<String>,
}

impl Serialize for Canonicalized {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let wire = match self {
            Canonicalized::NoMatch => WireCanonicalized {
                matched: false,
                identity: None,
                resolved_url: None,
            },
            Canonicalized::Match {
                identity,
                resolved_url,
            } => WireCanonicalized {
                matched: true,
                identity: Some(identity.clone()),
                resolved_url: resolved_url.clone(),
            },
        };
        wire.serialize(s)
    }
}

impl<'de> Deserialize<'de> for Canonicalized {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let wire = WireCanonicalized::deserialize(d)?;
        match (wire.matched, wire.identity) {
            (false, None) if wire.resolved_url.is_none() => Ok(Canonicalized::NoMatch),
            (false, _) => Err(D::Error::custom(
                "`match: false` carries no `identity` or `resolvedUrl`",
            )),
            (true, None) => Err(D::Error::custom("`match: true` needs `identity`")),
            (true, Some(Identity::File)) => Err(D::Error::custom(
                "canonicalize cannot return a `file` identity",
            )),
            (true, Some(identity)) => {
                if let Some(u) = &wire.resolved_url {
                    check_http_url(u).map_err(|e| D::Error::custom(format!("resolvedUrl: {e}")))?;
                }
                Ok(Canonicalized::Match {
                    identity,
                    resolved_url: wire.resolved_url,
                })
            }
        }
    }
}

impl JsonSchema for Canonicalized {
    fn schema_name() -> Cow<'static, str> {
        WireCanonicalized::schema_name()
    }
    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        WireCanonicalized::json_schema(generator)
    }
}

/// 平台上的动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ListAction {
    Favorite,
    Like,
    Share,
    View,
    Unknown,
}

/// 一个用户能选的列表来源（`sources` 的输出，也用在清单的 `list.sources`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SourceInfo {
    /// 插件自己定，建议 `<platform>:<kind>[:<id>]`；`file:` 开头的留给导入。
    pub source: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<ListAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SourcesOutput {
    pub sources: Vec<SourceInfo>,
}

/// list 的一条。`url` 或 `ref` 至少一个。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// 插件私有、对宿主不透明；宿主回到这个插件调 `fetch <ref>`。
    #[serde(rename = "ref", default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<Identity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<Rfc3339>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<ListAction>,
    /// 拿不到加入时间就留空，不用抓取时间冒充。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acted_at: Option<Rfc3339>,
}

/// list 的一页（协议 §3.8）。`nextCursor: null` 是最后一页。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListPage {
    pub items: Vec<ListItem>,
    pub next_cursor: Option<String>,
    pub evidence: Evidence,
}
