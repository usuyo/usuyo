//! usuyo 资料库的数据契约：磁盘上各记录的 serde 类型（SPEC §1.9）。
//!
//! 每个对象都带 `extra`：读到不认识的字段原样留着，写回时照写（SPEC §0.2）。
//! 时间字段是字符串，格式（UTC 毫秒 `Z`）由写方保证，schema 校验。

mod changeset;
mod fetch;
mod frontmatter;
mod journal;
mod library_file;
mod manifest;
mod op;
mod trash;

use std::collections::BTreeMap;

pub use changeset::{ChangeOp, Changeset, DecisionState, ProposalState};
pub use fetch::{FetchRecord, FileHash, MediaFile};
pub use frontmatter::{CardMeta, DraftMeta, FrontMatter, NotesMeta, TopicMeta};
pub use journal::{JournalEntry, Redactable, Redacted, Target};
pub use library_file::{DeviceRecord, LibraryFile, Settings};
pub use manifest::{
    Author, Completeness, Identity, ItemState, MachinePicked, Manifest, Matched, Origin, Snapshot,
    TagMark, Tags, TextOnly,
};
pub use op::{Hunk, Op, OpTarget, SetChange};
pub use trash::TrashEntry;

/// 不认识的字段，按键名排序存着。
pub type Extra = BTreeMap<String, serde_json::Value>;

/// 本 crate 的错误。
#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("unknown op `{0}`")]
    UnknownOp(String),
    #[error("bad args for op `{op}`: {source}")]
    BadArgs {
        op: String,
        #[source]
        source: serde_json::Error,
    },
}

/// schema 文件内容，按名字取；名字照 `urn:usuyo:schema:<名字>:1`。
pub const SCHEMAS: &[(&str, &str)] = &[
    ("manifest", include_str!("../schema/manifest.schema.json")),
    (
        "journal-entry",
        include_str!("../schema/journal-entry.schema.json"),
    ),
    ("changeset", include_str!("../schema/changeset.schema.json")),
    ("topic", include_str!("../schema/topic.schema.json")),
    ("card", include_str!("../schema/card.schema.json")),
    ("draft", include_str!("../schema/draft.schema.json")),
    ("notes", include_str!("../schema/notes.schema.json")),
];

pub(crate) fn is_false(b: &bool) -> bool {
    !*b
}
