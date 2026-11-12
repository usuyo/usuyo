//! `raw/<snapshotId>/usuyo.fetch.json`：fetch 输出去掉 `credentialsUpdate`、加 `files`（SPEC §1.2.2）。
//!
//! 只列资料库要读的字段，都可缺；其余原样留在 `extra`。fetch 输出本身的 schema 归插件协议。

use serde::{Deserialize, Serialize};

use crate::{Author, Extra, Identity, is_plain_relative};

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
    /// 同一个字符串表示同一段时间线拆开的几个文件（例如分开的画面和音轨）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// 字幕、音轨的语言（BCP 47）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl FetchRecord {
    /// 第一个不是快照文件夹以内普通相对路径的路径（图、媒体、`files`）。
    pub fn bad_path(&self) -> Option<&str> {
        self.images
            .iter()
            .map(String::as_str)
            .chain(self.media.iter().map(|m| m.path.as_str()))
            .chain(self.files.iter().map(|f| f.path.as_str()))
            .find(|p| !is_plain_relative(p))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileHash {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_language_and_group_are_fields_and_round_trip() {
        let json = serde_json::json!({
            "path": "media/subtitle.zh.vtt", "kind": "subtitle", "lang": "zh",
            "group": "main", "mime": "text/vtt"
        });
        let m: MediaFile = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(
            (m.lang.as_deref(), m.group.as_deref()),
            (Some("zh"), Some("main"))
        );
        assert!(!m.extra.contains_key("lang"), "不留在 extra 里");
        assert_eq!(serde_json::to_value(&m).unwrap(), json);
    }
}
