//! 你写的 `.md` 的 front matter 对象（SPEC §1.4）。
//!
//! 写法（YAML 子集、键顺序）归资料库那一层；这里只给字段和键的顺序。

use serde::{Deserialize, Serialize};

use crate::Extra;

/// 有 front matter 的对象：`KEYS` 是写出时认识的键的顺序，其余按键名排在后面。
pub trait FrontMatter: Serialize + for<'de> Deserialize<'de> {
    const KEYS: &'static [&'static str];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicMeta {
    pub id: String,
    pub title: String,
    pub created_at: String,
    #[serde(default)]
    pub followed: bool,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default, rename = "match")]
    pub match_: Vec<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl FrontMatter for TopicMeta {
    const KEYS: &'static [&'static str] =
        &["id", "title", "createdAt", "followed", "keywords", "match"];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardMeta {
    pub id: String,
    pub title: String,
    pub created_at: String,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl FrontMatter for CardMeta {
    const KEYS: &'static [&'static str] = &["id", "title", "createdAt", "topics"];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftMeta {
    pub id: String,
    pub title: String,
    pub created_at: String,
    /// `article` `post` `script` `other`
    pub kind: String,
    #[serde(default)]
    pub from: Vec<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl FrontMatter for DraftMeta {
    const KEYS: &'static [&'static str] = &["id", "title", "createdAt", "kind", "from"];
}

/// `notes.md` 头部；没有记一句时整个 front matter 不写。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NotesMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub takeaway: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl FrontMatter for NotesMeta {
    const KEYS: &'static [&'static str] = &["takeaway"];
}
