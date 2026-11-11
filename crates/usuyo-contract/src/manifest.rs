//! `manifest.json` v1（SPEC §1.2.1）。

use serde::{Deserialize, Serialize};

use crate::Extra;

/// 条目的索引卡。数组总是出现；可选字段缺了就不写。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub id: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub identities: Vec<Identity>,
    pub kind: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at_original: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    pub added_at: String,
    #[serde(default)]
    pub origins: Vec<Origin>,
    #[serde(default)]
    pub snapshots: Vec<Snapshot>,
    #[serde(default)]
    pub primary_snapshot: Option<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub missing: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing_reason: Option<String>,
    pub completeness: Completeness,
    #[serde(default)]
    pub tags: Tags,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub same_as: Vec<String>,
    #[serde(default)]
    pub not_same_as: Vec<String>,
    #[serde(default)]
    pub private: bool,
    pub state: ItemState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_only: Option<TextOnly>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 平台身份；`contentId` 是字符串（19 位超过 2^53）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub platform: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 收下一次一项。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Origin {
    pub id: String,
    pub entry: String,
    pub taken_by: String,
    #[serde(default)]
    pub machine_picked: Option<MachinePicked>,
    #[serde(default)]
    pub matched: Vec<Matched>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_batch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acted_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acted_at_original: Option<String>,
    pub at: String,
    pub device: String,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 机器挑的：`by` 是 `keywords` `similarity` `upstream` `agent`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MachinePicked {
    pub by: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Matched {
    pub topic: String,
    pub by: String,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 一次获取的记录；内容在 `raw/<id>/`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub id: String,
    pub plugin: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_version: Option<String>,
    pub at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_url: Option<String>,
    pub access: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream: Option<serde_json::Value>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub missing: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing_reason: Option<String>,
    pub completeness: Completeness,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_origin: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 存下了多少；只升不降。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Completeness {
    Link,
    Partial,
    Full,
}

/// 条目在不在。合并时 `Trashed` > `TextOnly` > `Normal`，按这个顺序排。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ItemState {
    Normal,
    TextOnly,
    Trashed,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Tags {
    #[serde(default)]
    pub platform: Vec<TagMark>,
    #[serde(default)]
    pub engine: Vec<TagMark>,
    #[serde(default)]
    pub user: Vec<String>,
    #[serde(default)]
    pub dismissed: Vec<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagMark {
    pub tag: String,
    pub by: String,
    pub at: String,
    pub confidence: f64,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextOnly {
    pub at: String,
    #[serde(default)]
    pub removed: Vec<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Manifest {
    /// 数组照 SPEC §1.2.1 排好，两台写出一样的字节。
    pub fn sort_arrays(&mut self) {
        self.origins
            .sort_by(|a, b| (&a.at, &a.id).cmp(&(&b.at, &b.id)));
        self.snapshots
            .sort_by(|a, b| (&a.at, &a.id).cmp(&(&b.at, &b.id)));
        self.identities.sort_by_key(Identity::key);
        self.tags.platform.sort_by(|a, b| a.tag.cmp(&b.tag));
        self.tags.engine.sort_by(|a, b| a.tag.cmp(&b.tag));
        for o in &mut self.origins {
            o.matched
                .sort_by(|a, b| (&a.topic, &a.by).cmp(&(&b.topic, &b.by)));
        }
        for v in [
            &mut self.aliases,
            &mut self.outputs,
            &mut self.missing,
            &mut self.tags.user,
            &mut self.tags.dismissed,
            &mut self.topics,
            &mut self.same_as,
            &mut self.not_same_as,
        ] {
            v.sort();
            v.dedup();
        }
        for s in &mut self.snapshots {
            s.outputs.sort();
            s.outputs.dedup();
            s.missing.sort();
            s.missing.dedup();
        }
        if let Some(t) = &mut self.text_only {
            t.removed.sort();
            t.removed.dedup();
        }
    }
}

impl Identity {
    /// `identityKey`（SPEC §0.4）：有 `contentId` 用它，`web` 用 url；其余为空串。
    /// web 的 url 应已规范化，这里不再做。
    pub fn key(&self) -> String {
        match (&self.content_id, &self.url) {
            (Some(c), _) => format!("{}:{}", self.platform, c),
            (None, Some(u)) if self.platform == "web" => format!("web:{u}"),
            _ => String::new(),
        }
    }
}
