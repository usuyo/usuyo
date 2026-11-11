//! 回收站的 `entry.json`（SPEC §1.7）。

use serde::{Deserialize, Serialize};

use crate::Extra;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashEntry {
    pub id: String,
    /// `item` `topic` `card` `draft` `notes` `content`
    pub kind: String,
    /// 对象的 id（条目 id，或主题、卡片、成稿的 UUIDv7）。
    pub target: String,
    /// 原来从库根起的相对路径。
    pub path: String,
    /// 对象以内的相对路径，文件就放在回收站项以内同样的路径下。
    pub files: Vec<String>,
    /// `delete` `keep-text` `merge` `undo` `conflict`
    pub reason: String,
    pub trashed_at: String,
    #[serde(flatten)]
    pub extra: Extra,
}
