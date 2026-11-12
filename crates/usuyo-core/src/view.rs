//! 给接口的视图，形状照 `apps/desktop/web/src/api/endpoints.ts`（camelCase、可缺的字段不写）。

use serde::Serialize;

/// 平台的显示名由获取插件给，内核不认识具体平台。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Platform {
    pub id: String,
    pub name: String,
}

/// 一层文字里的一段，带它在原样里的位置（转出来的文字有，原文的段没有）。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// 媒体里的一处（feature 003 §5.3、§7.1）：一个时间点（秒，可带结束），或一张图。
/// `xywh` 是图上（或那一帧上）的框，0–1，原点在左上角。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum Position {
    Time {
        sec: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        end_sec: Option<f64>,
        snapshot_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        xywh: Option<[f64; 4]>,
        /// 画面上的字：认出这一行的那一帧。
        #[serde(skip_serializing_if = "Option::is_none")]
        frame_url: Option<String>,
    },
    Image {
        image_key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        xywh: Option<[f64; 4]>,
    },
}

/// 一份是从哪来的。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FacetSource {
    pub by: SourceBy,
    /// 机器的：工具或模型的名字。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// 机器的：做这一份的转换插件 id（`run.json` 的 `by`），两个插件各转一份时靠它分开。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceBy {
    Platform,
    Machine,
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

/// 一条有几个侧面就给几个，同一层有几份就给几份（feature 003 §7.1）。`key` 在一条里唯一：
/// `asr:platform:<snapshotId>:<lang>`、`asr:<runId>`、`ocr:<runId>`、`frames:<runId>`、`video:<snapshotId>`，
/// 原文、图、评论是 `text:<snapshotId>`、`image:<snapshotId>`、`comments:<snapshotId>`。
/// `remark` 只写量得出来的明显的事（003 §6）。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    tag = "layer",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum Facet {
    Asr {
        key: String,
        source: FacetSource,
        segments: Vec<Segment>,
        #[serde(skip_serializing_if = "Option::is_none")]
        remark: Option<String>,
    },
    Ocr {
        key: String,
        source: FacetSource,
        segments: Vec<Segment>,
        #[serde(skip_serializing_if = "Option::is_none")]
        remark: Option<String>,
    },
    /// 关键帧上的字：连着几帧一样的一行合成一段，`endSec` 写到最后出现的那一帧。
    Frames {
        key: String,
        source: FacetSource,
        segments: Vec<Segment>,
        #[serde(skip_serializing_if = "Option::is_none")]
        remark: Option<String>,
    },
    Text {
        key: String,
        segments: Vec<Segment>,
    },
    Image {
        key: String,
        images: Vec<ImageRef>,
        #[serde(skip_serializing_if = "Option::is_none")]
        remark: Option<String>,
    },
    Video {
        key: String,
        snapshot_id: String,
        /// 插件没给时长时是 0。
        duration_sec: f64,
        /// 有合起来的 `play.mp4` 时是它，不然是原样的画面。
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// 最新一次 `keyframes` 的帧。
        frames: Vec<Frame>,
        #[serde(skip_serializing_if = "Option::is_none")]
        remark: Option<String>,
    },
    Comments {
        key: String,
        comments: Vec<Comment>,
    },
}

/// 还没转出来的一面，只在条目页那一面写一句（feature 003 §4.4）。没有计数、没有百分比。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Pending {
    /// `asr` `ocr` `frames` `video`（`play`、`keyframes` 都算画面那一面）。
    pub layer: &'static str,
    pub state: PendingState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<PendingModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PendingState {
    /// 在转。
    Now,
    /// 等你下载模型。
    NeedsModel,
    /// 没转出来，原因在 `hint`。
    Failed,
    /// 排着，在电池上：接上电源时转。
    OnPower,
    /// 排着，会慢慢转。
    Later,
}

/// 要先下载的模型；`plugin` 是转换插件的 id，`size` 是字节。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PendingModel {
    pub plugin: String,
    pub name: String,
    pub size: u64,
    pub license: String,
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
    /// 还没转出来的面。
    pub pending: Vec<Pending>,
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

/// 回收站页上的一项为什么在回收站里。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrashReason {
    /// 你删掉的整条。
    Delete,
    /// 撤销收下时移进来的整条。
    Undo,
    /// 只留文字时移出来的图、视频、音频；条目和文字还在。
    KeepText,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashEntryView {
    /// 回收站项的 id（`trashId`），「立即清除」用它；拿回来用 `item_id`。
    pub id: String,
    pub reason: TrashReason,
    pub item_id: String,
    pub title: String,
    pub platform: Platform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub trashed_at: String,
    /// 到这个时间就清除；`trashDays` 是「一直」时没有。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// 回收站页：现在在回收站里的条目和媒体，新的在前。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashView {
    /// 放多少天；`null` 是「一直」。
    pub trash_days: Option<u32>,
    pub entries: Vec<TrashEntryView>,
    /// 读不了、没列出来的回收站项（id 是 `trashId`）。
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unreadable: Vec<Unreadable>,
}

/// 命中的那一层；标题、作者、记一句另算。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HitLayer {
    Text,
    Asr,
    Ocr,
    Frames,
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
    /// 转出来的文字命中时：那一秒、那一张、那一块。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
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
    /// 导入插件读交进来的一个文件（`POST /file-imports`，或浏览器脚本经 `/v1/file-imports` 交的）。
    File,
}

/// `file` 类导入：交进来的文件名和读它的插件的名字。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImportFileView {
    pub name: String,
    pub reader: String,
}

/// 一种能读别的工具导出的文件的方法（`GET /file-readers`）：做了的、`list` 写了 `files` 的。
/// 名字、读什么、文件长什么样都取自清单。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileReaderView {
    /// `<插件 id>#<方法 id>`，交文件时原样给回。
    pub id: String,
    /// 方法的名字。
    pub name: String,
    /// 插件的名字。
    pub plugin: String,
    /// 清单 `list.reads`。
    pub reads: String,
    /// 清单 `list.files`。
    pub patterns: Vec<String>,
    /// 清单 `list.sources` 的名字。
    pub lists: Vec<String>,
    /// 方法的 `risk.text`。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// 预览里各几条：新的、库里已经有的、在回收站里的。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PreviewCounts {
    pub new: u32,
    pub stored: u32,
    pub trashed: u32,
}

/// 为什么在等、停着。`code`：插件的错误码（`login-required`、`visitor-required`、`rate-limited`、`blocked`……），
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
    pub file: Option<ImportFileView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<String>,
    /// 来源属于插件的哪种方法。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
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
    /// 从文件导入时只有插件的 `ref`、没有链接的那几条没有。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
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
    pub sources: Vec<SourceView>,
    /// 会用到哪些站点的账号；登录状态见 `/sites`。
    pub sites: Vec<String>,
    /// 会用到哪些站点的访客身份（不是账号）；有没有见 `/visitors`。
    pub visitors: Vec<String>,
    pub settings: Vec<SettingView>,
    pub pacing: Vec<PacingView>,
    /// 现在停着的地方和原因，所有方法的；你点的「先别试」方法是 `*`。
    pub pauses: Vec<PauseView>,
    /// 清单里的每一种方法，没做的也在（协议 §2.3.1）。
    pub methods: Vec<MethodView>,
}

/// 一个来源，带它属于哪种方法：开始导入时一起交回。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceView {
    #[serde(flatten)]
    pub source: usuyo_plugin::protocol::SourceInfo,
    pub method: String,
}

/// 一种方法（`/plugins` 的 `methods[]`）。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodView {
    pub id: String,
    pub name: String,
    pub access: usuyo_plugin::protocol::Access,
    pub login: usuyo_plugin::protocol::Login,
    pub account: AccountView,
    pub interaction: usuyo_plugin::protocol::Interaction,
    /// 驱动哪一样：`iphone`、`android`、`desktop-app`；不驱动设备的没有。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drives: Option<usuyo_plugin::protocol::Drives>,
    /// 层级和方法本身不同的平台（清单的 `perPlatform`），各自的层级和在这个平台上开没开。
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub per_platform: Vec<PlatformTierView>,
    pub gets: GetsView,
    pub built: bool,
    /// 开关：你拨过的照你拨的，没拨过照方法本身的那一层。
    pub enabled: bool,
    /// 这台机器上现在用得上：做了、开着、要的登录和访客身份都在、本机检查没挡它、没停着。
    pub usable: bool,
    /// 为什么用不上，一项一句。
    pub blockers: Vec<Blocker>,
    /// 上一次调用。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_call: Option<LastCall>,
    /// 这种方法停着的地方，加上你点的「先别试」。
    pub pauses: Vec<PauseView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_note: Option<usuyo_plugin::protocol::Risk>,
}

/// 一种方法在某个平台上的层级（清单的 `perPlatform`）。`enabled`：没拨过开关时照这一层的默认
/// （`private-api`、`client-hook` 关着），拨过就和方法的开关一样。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformTierView {
    pub platform: String,
    pub access: usuyo_plugin::protocol::Access,
    pub login: usuyo_plugin::protocol::Login,
    pub enabled: bool,
}

/// 账号用在哪；`sites` 是登录站点（`in-usuyo`）或访客身份的站点（`visitor`），`where` 是账号所在的地方给人看的名字。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    pub kind: usuyo_plugin::protocol::Account,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sites: Vec<String>,
    #[serde(rename = "where", skip_serializing_if = "Option::is_none")]
    pub where_: Option<String>,
}

/// 取得到什么：`fetch` 是输出的词表（`text`、`images`、`media:video`…），`list` 是清单里写死的来源名，
/// `link` 是来源从一条链接开始时链接框上的那句，`delivers: links` 是由别处把链接交进来。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetsView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetch: Option<Vec<usuyo_plugin::protocol::OutputKind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivers: Option<usuyo_plugin::protocol::Delivers>,
}

/// 用不上的一个原因：`code` 是 `not-built`、`off`、`login-missing`、`login-expired`、`visitor-missing`、
/// `visitor-expired`、`check-failed`（插件的本机检查没过）、`health-failed`（插件没能自查）、
/// `paused`（被限流、被拦或你点了「先别试」，`hint` 写到什么时候）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Blocker {
    pub code: String,
    pub hint: String,
}

/// 一种方法上一次调用：`ok` 为假时带错误码和一句话。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastCall {
    pub at: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
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
    /// 停的是哪种方法；`*` 是你点的「先别试」，整个插件。
    pub method: String,
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

/// `GET /transforms`：装着的转换插件，和 `transforms.auto`。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TransformsView {
    pub plugins: Vec<TransformPluginView>,
    pub auto: bool,
}

/// 一个装着的转换插件。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPluginView {
    pub id: String,
    pub name: String,
    pub version: String,
    /// 它能做的 `job`：`play` `asr` `keyframes` `ocr` `frames`。
    pub jobs: Vec<String>,
    /// 清单的 `upstream`：自带或包着的第三方程序和它的许可。
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bundled: Vec<Bundled>,
    pub models: Vec<ModelView>,
    /// 插件自己报的：`ok`，或 `unavailable`（`hint` 是没过的那几项检查）。
    pub health: Health,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Bundled {
    pub name: String,
    pub license: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Health {
    Ok,
    Unavailable,
}

/// 一个模型：名字、下载大小、许可、现在怎样；`failed` 带一句给人看的原因。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelView {
    pub name: String,
    pub size: u64,
    pub license: String,
    #[serde(serialize_with = "model_state")]
    pub state: crate::ModelState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// `absent` `downloading` `present` `failed`；没成的英文原因只进日志。
fn model_state<S: serde::Serializer>(s: &crate::ModelState, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_str(match s {
        crate::ModelState::Absent => "absent",
        crate::ModelState::Downloading => "downloading",
        crate::ModelState::Present => "present",
        crate::ModelState::Failed(_) => "failed",
    })
}

/// 设置页要的内核那边的设置（`GET /settings`）。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub library: String,
    /// 回收站放几天；`null` 是一直。
    pub trash_days: Option<u32>,
    /// 时钟守卫拦下了到期的项；没有拦下时省略。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trash_held: Option<TrashHeld>,
    pub network: NetworkView,
    pub transforms: TransformsSetting,
    /// 内核的日志在哪。
    pub logs: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrashHeld {
    pub count: usize,
    /// 记下的最晚时间，RFC 3339。
    pub since: String,
}

/// `mode` 是 `system`、`direct`、`custom`；`custom` 的 `proxy` 不带密码。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkView {
    pub mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    /// 这台电脑开着假 IP 模式的代理，宿主已经照它调好了。
    pub fake_ip: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TransformsSetting {
    pub auto: bool,
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
    /// 模型在下、下好了、没下成、删掉了。重新拉 `/transforms`。
    TransformsUpdated,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn shapes_match_the_frontend_contract() {
        let f = Facet::Video {
            key: "video:s".into(),
            snapshot_id: "s".into(),
            duration_sec: 3.0,
            url: None,
            frames: vec![],
            remark: None,
        };
        assert_eq!(
            serde_json::to_value(&f).unwrap(),
            json!({"layer": "video", "key": "video:s", "snapshotId": "s", "durationSec": 3.0, "frames": []})
        );
        let a = Facet::Frames {
            key: "frames:r".into(),
            source: FacetSource {
                by: SourceBy::Machine,
                tool: Some("Vision".into()),
                plugin: Some("usuyo/ocr-vision".into()),
                run_id: Some("r".into()),
                at: None,
                lang: None,
            },
            segments: vec![Segment {
                text: "x".into(),
                position: Some(Position::Time {
                    sec: 1.5,
                    end_sec: Some(4.0),
                    snapshot_id: "s".into(),
                    xywh: Some([0.1, 0.2, 0.3, 0.4]),
                    frame_url: None,
                }),
            }],
            remark: None,
        };
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            json!({"layer": "frames", "key": "frames:r",
                   "source": {"by": "machine", "tool": "Vision", "plugin": "usuyo/ocr-vision", "runId": "r"},
                   "segments": [{"text": "x", "position": {"kind": "time", "sec": 1.5, "endSec": 4.0,
                                 "snapshotId": "s", "xywh": [0.1, 0.2, 0.3, 0.4]}}]})
        );
        let p = Pending {
            layer: "asr",
            state: PendingState::NeedsModel,
            model: None,
            hint: None,
        };
        assert_eq!(
            serde_json::to_value(&p).unwrap(),
            json!({"layer": "asr", "state": "needs-model"})
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
