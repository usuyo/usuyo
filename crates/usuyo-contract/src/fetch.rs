//! `raw/<snapshotId>/usuyo.fetch.json`：fetch 输出去掉 `credentialsUpdate`、加 `files`（SPEC §1.2.2）。
//!
//! 只列资料库要读的字段，都可缺；其余原样留在 `extra`。fetch 输出本身的 schema 归插件协议。

use serde::{Deserialize, Serialize};

use crate::{Author, Extra, Identity};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<Identity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at_original: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// `plain`（默认）或 `markdown`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_format: Option<String>,
    #[serde(default)]
    pub images: Vec<String>,
    #[serde(default)]
    pub media: Vec<MediaFile>,
    #[serde(default)]
    pub files: Vec<FileHash>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaFile {
    pub path: String,
    /// `video` `audio` `subtitle` `other`
    pub kind: String,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileHash {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}
