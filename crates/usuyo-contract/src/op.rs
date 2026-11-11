//! op 词表（SPEC §1.5.1），提议和改动记录共用。
//!
//! 线上写成 `{"op": "<名字>", "args": {…}}`；改段落的 op 的 `hunks` 也在 `args` 里。

use serde::{Deserialize, Serialize};

use crate::ContractError;

/// op 作用在谁身上。`kind` 是 `item` `topic` `card` `draft` `trash` `library`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpTarget {
    pub kind: String,
    pub id: String,
}

/// 集合的增减。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SetChange {
    #[serde(default)]
    pub add: Vec<String>,
    #[serde(default)]
    pub remove: Vec<String>,
}

/// 按段改：原文是 `before` 的那几段换成 `after`；`before` 为空时插在等于 `anchor` 的那段后，
/// `anchor` 为 `null` 插在末尾。段之间是空行。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hunk {
    pub id: String,
    pub before: String,
    pub after: String,
    #[serde(default)]
    pub anchor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "kebab-case")]
pub enum Op {
    #[serde(rename_all = "camelCase")]
    AddItem {
        origin: String,
        created: bool,
    },
    SetTags(SetChange),
    SetTopics(SetChange),
    SetTakeaway {
        text: Option<String>,
    },
    EditNotes {
        hunks: Vec<Hunk>,
    },
    AddHighlight {
        id: String,
        layer: String,
        #[serde(default)]
        snapshot: Option<String>,
        #[serde(default)]
        fragment: Option<String>,
        quote: String,
    },
    CreateTopic {
        title: String,
        #[serde(default)]
        keywords: Vec<String>,
        #[serde(default = "default_match", rename = "match")]
        match_: Vec<String>,
    },
    EditTopic {
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        keywords: Option<Vec<String>>,
        #[serde(default, rename = "match")]
        match_: Option<Vec<String>>,
    },
    CreateCard {
        title: String,
        body: String,
    },
    CreateDraft {
        title: String,
        body: String,
        kind: String,
    },
    EditTopicPage {
        hunks: Vec<Hunk>,
    },
    EditCard {
        #[serde(default)]
        hunks: Vec<Hunk>,
        #[serde(default)]
        title: Option<String>,
    },
    EditDraft {
        #[serde(default)]
        hunks: Vec<Hunk>,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        kind: Option<String>,
        #[serde(default)]
        from: SetChange,
    },
    AddCitation {
        #[serde(default)]
        anchor: Option<String>,
        item: String,
        layer: String,
        #[serde(default)]
        snapshot: Option<String>,
        #[serde(default)]
        fragment: Option<String>,
        quote: String,
    },
    RemoveHighlight {
        id: String,
    },
    DismissSuggestion {
        tag: String,
    },
    SetFollowed {
        value: bool,
    },
    SetPrivate {
        value: bool,
    },
    SetSetting {
        key: String,
        value: serde_json::Value,
    },
    Move {
        path: String,
    },
    SetNotSame(SetChange),
    MoveToTrash {
        reason: String,
    },
    KeepText {},
    Restore {},
    MergeItems {
        into: String,
    },
    Purge {},
}

fn default_match() -> Vec<String> {
    vec!["keywords".to_owned()]
}

impl Op {
    /// op 的名字，照 SPEC 的 kebab-case。
    pub fn name(&self) -> &'static str {
        match self {
            Op::AddItem { .. } => "add-item",
            Op::SetTags(_) => "set-tags",
            Op::SetTopics(_) => "set-topics",
            Op::SetTakeaway { .. } => "set-takeaway",
            Op::EditNotes { .. } => "edit-notes",
            Op::AddHighlight { .. } => "add-highlight",
            Op::CreateTopic { .. } => "create-topic",
            Op::EditTopic { .. } => "edit-topic",
            Op::CreateCard { .. } => "create-card",
            Op::CreateDraft { .. } => "create-draft",
            Op::EditTopicPage { .. } => "edit-topic-page",
            Op::EditCard { .. } => "edit-card",
            Op::EditDraft { .. } => "edit-draft",
            Op::AddCitation { .. } => "add-citation",
            Op::RemoveHighlight { .. } => "remove-highlight",
            Op::DismissSuggestion { .. } => "dismiss-suggestion",
            Op::SetFollowed { .. } => "set-followed",
            Op::SetPrivate { .. } => "set-private",
            Op::SetSetting { .. } => "set-setting",
            Op::Move { .. } => "move",
            Op::SetNotSame(_) => "set-not-same",
            Op::MoveToTrash { .. } => "move-to-trash",
            Op::KeepText {} => "keep-text",
            Op::Restore {} => "restore",
            Op::MergeItems { .. } => "merge-items",
            Op::Purge {} => "purge",
        }
    }

    /// Agent 和模型能不能提议它（SPEC §1.5.1「能提议」那一列）。
    pub fn proposable(&self) -> bool {
        matches!(
            self,
            Op::SetTags(_)
                | Op::SetTopics(_)
                | Op::SetTakeaway { .. }
                | Op::EditNotes { .. }
                | Op::AddHighlight { .. }
                | Op::CreateTopic { .. }
                | Op::EditTopic { .. }
                | Op::CreateCard { .. }
                | Op::CreateDraft { .. }
                | Op::EditTopicPage { .. }
                | Op::EditCard { .. }
                | Op::EditDraft { .. }
                | Op::AddCitation { .. }
        )
    }

    /// 从提议文件里的 `op` 和 `args` 拼回来。
    pub fn from_parts(op: &str, args: &serde_json::Value) -> Result<Op, ContractError> {
        let args = if args.is_null() {
            serde_json::Value::Object(Default::default())
        } else {
            args.clone()
        };
        serde_json::from_value(serde_json::json!({"op": op, "args": args})).map_err(|source| {
            if source.to_string().contains("unknown variant") {
                ContractError::UnknownOp(op.to_owned())
            } else {
                ContractError::BadArgs {
                    op: op.to_owned(),
                    source,
                }
            }
        })
    }

    /// 拆成 `op` 和 `args`。
    pub fn to_parts(&self) -> Result<(&'static str, serde_json::Value), ContractError> {
        let mut v = serde_json::to_value(self).map_err(|source| ContractError::BadArgs {
            op: self.name().to_owned(),
            source,
        })?;
        let args = v
            .get_mut("args")
            .map(serde_json::Value::take)
            .unwrap_or_else(|| serde_json::Value::Object(Default::default()));
        Ok((self.name(), args))
    }
}
