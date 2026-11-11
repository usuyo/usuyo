use serde::{Deserialize, Serialize};

/// 一条要进索引的条目。调用方从资料库映射而来，索引不读文件。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IndexDoc {
    pub item_id: String,
    pub platform: String,
    pub kind: String,
    pub title: String,
    pub author: String,
    pub tags: Vec<String>,
    pub topics: Vec<String>,
    /// 发布时间，Unix 秒；不知道就留空（按日期筛时不会命中）。
    pub published_at: Option<i64>,
    /// 收下的时间，Unix 秒。
    pub added_at: i64,
    pub private: bool,
    pub layers: Vec<LayerText>,
}

/// 一层文字。`layer` 只放层名（`ocr/<imageKey>` 的 `ocr`），
/// 图片键之类放进段的 `locator`；筛选按层名整串相等。
/// （text、asr、ocr、vision、notes、takeaway、highlights……）。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LayerText {
    pub layer: String,
    pub snapshot_id: Option<String>,
    pub segments: Vec<Segment>,
}

/// 一段文字。`locator` 对索引不透明，搜到时原样还给调用方，用来拼引用。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub text: String,
    pub locator: String,
}
