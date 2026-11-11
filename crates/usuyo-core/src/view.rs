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

/// 一次导入走到哪了（见 `imports.rs` 的模块说明）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImportState {
    /// 在列（或在认交进来的链接的身份）。
    Listing,
    /// 列完了，等你看一眼、确认。
    Ready,
    /// 你确认了，按节奏一条条收。
    Taking,
    /// 你让它先停下。
    Held,
    /// 收完了。
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImportKind {
    /// 插件列出来的（`list`）。
    Source,
    /// 交进来的一批链接（`/v1/batches`）。
    Delivered,
}

/// 预览里各几条：新的、库里已经有的、在回收站里的。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PreviewCounts {
    pub new: u32,
    pub stored: u32,
    pub trashed: u32,
}

/// 为什么在等、停着。`code`：插件的错误码（`login-required`、`rate-limited`、`blocked`……），
/// 或 `pacing`（等间隔）、`daily-limit`（今天批量的次数用完了）、`held`（你让插件先别试）、
/// `batch-off`（这个平台不做导入）。没有 `until` 的等你动手。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Waiting {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
}

/// 一次导入。只说事实：列了几页、预览里各几条、收下了几条、为什么在等；不说还剩多少。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportView {
    pub id: String,
    pub kind: ImportKind,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// 交进来的那个投递令牌的名字。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,
    pub created_at: String,
    pub state: ImportState,
    pub pages: u32,
    pub preview: PreviewCounts,
    /// 收下了几条。
    pub taken: u32,
    /// 收下了、但内容取不到的（条目页写着原因）。
    pub not_stored: u32,
    /// 轮到时已经进了回收站、或是私密的，没收。
    pub skipped: u32,
    /// 下一步大概什么时候。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting: Option<Waiting>,
}

/// 预览里的一条。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportEntryView {
    pub seq: i64,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acted_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    /// `new`、`stored`、`trashed`、`unresolved`（还没认出身份）。
    pub standing: String,
    /// 已经收下了。
    pub taken: bool,
}

/// 一个获取插件（`/plugins`）。档位和登录要求只是给你挑的标签，不拦什么。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginView {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub version: String,
    pub platforms: Vec<Platform>,
    /// 七层取法里最高的那层（协议 §2.4）。
    pub access: usuyo_plugin::protocol::Access,
    pub login: usuyo_plugin::protocol::Login,
    /// 能列出你存过的（有 `list`）。
    pub can_list: bool,
    /// 清单里写死的来源，登录前就能显示；运行时以 `/plugins/{id}/sources` 为准。
    pub sources: Vec<usuyo_plugin::protocol::SourceInfo>,
    /// 会用到哪些站点的账号；登录状态见 `/sites`。
    pub sites: Vec<String>,
    /// 会用到哪些站点的访客身份（不是账号）；有没有见 `/visitors`。
    pub visitors: Vec<String>,
    pub settings: Vec<SettingView>,
    pub pacing: Vec<PacingView>,
    /// 现在停着的地方和原因。
    pub pauses: Vec<PauseView>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingView {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub value_type: usuyo_plugin::protocol::SettingType,
    pub secret: bool,
    /// 密钥设置填了没有；值从不回出来。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PacingView {
    pub platform: String,
    pub batch: bool,
    pub interval_sec: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PauseView {
    /// 平台，或通用网页插件的网站；`*` 是你点的「先别试」。
    pub scope: String,
    /// 你点了「先别试」。
    pub by_you: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// 没加载的插件和原因。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotLoaded {
    pub dir: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsView {
    pub plugins: Vec<PluginView>,
    pub not_loaded: Vec<NotLoaded>,
}

/// 一个登录站点（`/sites`）：登录窗口照它开、照它判断登录好了没有。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteView {
    pub site: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_url: Option<String>,
    /// 登录后才有的 cookie 名：都在就算登录好了。空的时候有任何没过期的 cookie 就算。
    pub cookies: Vec<String>,
    /// 除了站点本身和它的上级域名，还要读哪些域名的 cookie。
    pub domains: Vec<String>,
    pub login: usuyo_plugin::LoginState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<String>,
    pub plugins: Vec<String>,
}

/// 一个访客身份（`/visitors`）：站点给没登录的访客的 cookie，不是账号。访客窗口照它开、照它判断取到没有。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitorView {
    pub site: String,
    pub name: String,
    /// 访客窗口打开的那一页。
    pub url: String,
    /// 这些名字都有了就算取到；空的时候等名字不再变。
    pub cookies: Vec<String>,
    pub domains: Vec<String>,
    /// `present`、`missing`、`expired`。
    pub state: usuyo_plugin::LoginState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<String>,
    /// 最早过期的那个 cookie 什么时候过期；都是会话 cookie 时没有。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub plugins: Vec<String>,
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
    /// 回收站里的条目到期或被立即清除，从资料库里移除了（已清干净，拉不到了）。
    ItemRemoved {
        id: String,
    },
    TopicUpdated {
        id: String,
    },
    /// 一次导入变了：列了一页、收下了一条、停下了。重新拉 `/imports/{id}`。
    ImportUpdated {
        id: String,
    },
    /// 插件的暂停、站点的登录变了。重新拉 `/plugins`、`/sites`。
    PluginsUpdated,
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
        let removed = Event::ItemRemoved { id: "a".into() };
        assert_eq!(
            serde_json::to_value(&removed).unwrap(),
            json!({"type": "item-removed", "id": "a"})
        );
    }
}
