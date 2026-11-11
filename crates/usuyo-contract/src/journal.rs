//! 改动记录的一行（SPEC §1.6）。

use serde::{Deserialize, Serialize};

use crate::Extra;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    pub id: String,
    pub n: u64,
    pub t: String,
    pub device: String,
    pub actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    pub op: String,
    #[serde(default)]
    pub targets: Vec<Target>,
    /// 反着做所需的内容；清干净后是 `{"redacted": true}`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_set: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_op: Option<String>,
    pub batch: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub undoes: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 改了哪个文件；`before`、`after` 是 `sha256:<hex>`，新建的 `before` 为 `null`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub kind: String,
    pub id: Redactable<String>,
    pub path: Redactable<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 值本身，或清干净后的 `{"redacted": true}`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Redactable<T> {
    Value(T),
    Redacted(Redacted),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Redacted {
    pub redacted: bool,
}

impl<T> Redactable<T> {
    pub fn redacted() -> Self {
        Redactable::Redacted(Redacted { redacted: true })
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            Redactable::Value(v) => Some(v),
            Redactable::Redacted(_) => None,
        }
    }
}

impl JournalEntry {
    /// 是否已脱敏。
    pub fn is_redacted(&self) -> bool {
        self.targets
            .iter()
            .any(|t| matches!(t.id, Redactable::Redacted(_)))
    }
}
