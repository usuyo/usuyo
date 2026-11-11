//! 给接口的视图，形状照 `apps/desktop/web/src/api/endpoints.ts`（camelCase、可缺的字段不写）。

use serde::Serialize;

/// 平台的显示名由获取插件给，内核不认识具体平台。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Platform {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImageRef {
    pub key: String,
    pub url: String,
    /// 量不出尺寸（不认识的格式）时写 0，前端按图片自己的大小摆。
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Frame {
    pub sec: f64,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Comment {
    pub author: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub likes: Option<u64>,
}

/// 一条有几个侧面就给几个。口播（asr）和图里的字（ocr）等转换插件接上再有。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    tag = "layer",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum Facet {
    Text {
        segments: Vec<Segment>,
    },
    Image {
        images: Vec<ImageRef>,
    },
    Video {
        snapshot_id: String,
        /// 插件没给时长时是 0。
        duration_sec: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// 关键帧要等转换插件，现在总是空的。
        frames: Vec<Frame>,
    },
    Comments {
        comments: Vec<Comment>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stored {
    Complete,
    /// 取到了，但有的没存下来（见 `missing`）。
    Partial,
    Link,
    TextOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSummary {
    pub id: String,
    pub title: String,
    pub platform: Platform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub taken_at: String,
    pub topic_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemView {
    #[serde(flatten)]
    pub summary: ItemSummary,
    pub url: String,
    pub stored: Stored,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub facets: Vec<Facet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trashed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Topic {
    pub id: String,
    pub name: String,
    pub followed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapRegion {
    pub topic_id: String,
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapPoint {
    pub item_id: String,
    pub x: f64,
    pub y: f64,
    pub title: String,
    pub platform: Platform,
    pub topic_ids: Vec<String>,
    pub taken_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapView {
    pub regions: Vec<MapRegion>,
    pub points: Vec<MapPoint>,
    /// 读不了、没摆上地图的条目。
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unreadable: Vec<Unreadable>,
}

/// 一条读不了的条目（多半在 usuyo 外改坏了文件）：跳过它，别的照常，把它和原因告诉问的人。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unreadable {
    pub id: String,
    pub reason: String,
}

/// 重建索引的结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reindexed {
    pub items: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unreadable: Vec<Unreadable>,
}

/// 命中的那一层；标题、作者、记一句另算。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HitLayer {
    Text,
    Comments,
    Title,
    Author,
    Note,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub item_id: String,
    pub title: String,
    pub platform: Platform,
    pub layer: HitLayer,
    pub snippet: String,
    /// 命中词在 `snippet` 里的 [起, 止)，按 UTF-16 单位（JS 字符串下标）。
    #[serde(rename = "match")]
    pub matched: [usize; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeResponse {
    pub id: String,
    /// 以前收下过：这次补一次快照，原来存的不动。
    pub seen: bool,
    pub title: String,
    pub platform: Platform,
    pub taken_at: String,
}

/// 推给前端的事件；SSE 的事件名就是 `type`。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum Event {
    /// 新收下一条（还只有链接）。
    ItemAdded {
        id: String,
    },
    /// 条目变了：取到了内容、改了记一句、换了主题……
    ItemUpdated {
        id: String,
    },
    /// 这次没取到；`retry` 为真时过会儿自己再试。
    ItemFetchFailed {
        id: String,
        code: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        hint: Option<String>,
        retry: bool,
    },
    /// 进了回收站。
    ItemTrashed {
        id: String,
    },
    TopicUpdated {
        id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn shapes_match_the_frontend_contract() {
        let f = Facet::Video {
            snapshot_id: "s".into(),
            duration_sec: 3.0,
            url: None,
            frames: vec![],
        };
        assert_eq!(
            serde_json::to_value(&f).unwrap(),
            json!({"layer": "video", "snapshotId": "s", "durationSec": 3.0, "frames": []})
        );
        assert_eq!(
            serde_json::to_value(Stored::TextOnly).unwrap(),
            json!("text-only")
        );
        let e = Event::ItemFetchFailed {
            id: "a".into(),
            code: "network".into(),
            hint: None,
            retry: true,
        };
        assert_eq!(
            serde_json::to_value(&e).unwrap()["type"],
            json!("item-fetch-failed")
        );
    }
}
