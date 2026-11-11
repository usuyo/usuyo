// 内核接口的路径和类型，都在这一个文件里。接口还没定稿，改动只落在这里；Rust 那边照这里的形状做。
// 层名和位置照 docs/SPEC.md §1.2 的引用：asr、ocr、text、image、video、comments；时间是秒，图片用 imageKey。

export const DEFAULT_API_BASE = "http://127.0.0.1:47615";

/** 开发服务器上代理到内核的路径：vite.config.ts 在服务端带上令牌，前端走同源。 */
export const DEV_PROXY_PATH = "/api";

// 除了 CORS 预检，每个请求都带 `Authorization: Bearer <令牌>`；令牌由桌面壳给（src/shell/）。
// 带了 `Origin` 的还要在内核的名单里，名单里只有桌面壳的 webview。
// `<img>`、`<video>` 带不了令牌，媒体地址里自带 `k`。
export const paths = {
  health: "/health",
  map: "/map",
  /** GET 列出全部；POST 按名字建一个，同名的已经有了就回那个。 */
  topics: "/topics",
  topicFollow: (id: string) => `/topics/${encodeURIComponent(id)}/follow`,
  item: (id: string) => `/items/${encodeURIComponent(id)}`,
  itemNote: (id: string) => `/items/${encodeURIComponent(id)}/note`,
  itemTopics: (id: string) => `/items/${encodeURIComponent(id)}/topics`,
  itemTrash: (id: string) => `/items/${encodeURIComponent(id)}/trash`,
  /** 整条从回收站拿回来。 */
  itemRestore: (id: string) => `/items/${encodeURIComponent(id)}/restore`,
  /** 只留文字以后，把移进回收站的视频和图拿回来。 */
  itemRestoreMedia: (id: string) => `/items/${encodeURIComponent(id)}/restore-media`,
  itemTextOnly: (id: string) => `/items/${encodeURIComponent(id)}/text-only`,
  itemRefetch: (id: string) => `/items/${encodeURIComponent(id)}/refetch`,
  search: "/search",
  take: "/items",
  /** 事件流（SSE）：`event:` 是 `LibraryEvent["type"]` 或 `PlatformEvent["type"]`，`data:` 是事件的 JSON。要带令牌，用 fetch 读。 */
  events: "/events",

  // —— 获取插件（「获取插件」页）——
  /** GET：装着的获取插件、各自的标签和节奏、停着没有；还有没加载的和原因。 */
  plugins: "/plugins",
  /** GET：这个插件现在能列的来源（会调插件，按平台的节奏可能要等几秒到一分钟）。 */
  pluginSources: (id: string) => `/plugins/${pluginPath(id)}/sources`,
  /** POST：「现在再试」，去掉这个插件的暂停。 */
  pluginTryNow: (id: string) => `/plugins/${pluginPath(id)}/try-now`,
  /** POST：「先别试」，到你点「现在再试」为止哪儿都不调它。 */
  pluginHold: (id: string) => `/plugins/${pluginPath(id)}/hold`,
  /** PUT `{value}` 填一个密钥设置（协议 §2.11），DELETE 去掉。值从不回出来。 */
  pluginSecret: (id: string, key: string) => `/plugins/${pluginPath(id)}/secrets/${encodeURIComponent(key)}`,
  /** GET：装着的插件会用到的账号站点和登录状态。登录窗口照它开。 */
  sites: "/sites",
  /** PUT `StorageState`：壳的登录窗口读出 cookie 以后交到这里（存进钥匙串）；DELETE 忘掉这个站点的登录。 */
  siteLogin: (site: string) => `/sites/${encodeURIComponent(site)}/login`,
  /** GET：装着的插件会用到的访客身份（不是账号）和有没有存着。访客窗口照它开；不出现在 `paths.sites` 里。 */
  visitors: "/visitors",
  /** PUT `StorageState`：壳的访客窗口读出 cookie 以后交到这里（存进钥匙串，和同一主机的账号分开）；DELETE 去掉。 */
  visitor: (site: string) => `/visitors/${encodeURIComponent(site)}`,

  // —— 导入（「导入」页）——
  /** GET 全部（新的在前）；POST `StartImportRequest` 从一个插件的来源开始列。 */
  imports: "/imports",
  /** GET 一次导入；DELETE 不要了（清单和还没收的去掉，已经收下的留着）。 */
  import: (id: string) => `/imports/${encodeURIComponent(id)}`,
  /** GET 预览里的条目，`?standing=new|stored|trashed|unresolved&offset=&limit=`。 */
  importEntries: (id: string) => `/imports/${encodeURIComponent(id)}/entries`,
  /** POST `ConfirmImportRequest`：看过了，按节奏去收你挑的。 */
  importConfirm: (id: string) => `/imports/${encodeURIComponent(id)}/confirm`,
  /** POST：先停下。 */
  importHold: (id: string) => `/imports/${encodeURIComponent(id)}/hold`,
  /** POST：接着导入（停下的、等你处理的都从这里继续）。 */
  importResume: (id: string) => `/imports/${encodeURIComponent(id)}/resume`,

  // —— 交进来的：投递令牌、收件文件夹 ——
  /** GET 列出投递端（不带令牌）；POST `CreateDeliverTokenRequest` 发一个，令牌只在回的那一次出现。 */
  deliverTokens: "/deliver-tokens",
  /** DELETE 撤销，马上生效。 */
  deliverToken: (id: string) => `/deliver-tokens/${encodeURIComponent(id)}`,
  /** GET `{path}`：收件文件夹在哪。手机经云盘往里放 `.txt`（一行一个链接或整段分享文字），收完移到 `.usuyo-done/`。 */
  inbox: "/inbox",
} as const;

/**
 * 投递接口（协议 §5.1）：浏览器扩展、油猴脚本、手机用自己的投递令牌调，壳和页面不用。
 * 投递令牌只能进这三条，读不了资料库；带 `Origin` 的只认 `chrome-extension://` 一类。
 * 发令牌以前先握手：`POST /handshake {nonce, client: <令牌 . 前面的 id>}`，证明用整个令牌算。
 */
export const deliverPaths = {
  ping: "/v1/ping",
  /** POST `DeliverRequest` → 202 `DeliverResponse`。 */
  deliver: "/v1/deliver",
  /** POST `BatchRequest` → 202 `{id}`：进导入的预览。 */
  batches: "/v1/batches",
} as const;

/** 插件 id 是 `owner/repo`：两段分别编码，中间的 `/` 留着。 */
function pluginPath(id: string): string {
  return id.split("/").map(encodeURIComponent).join("/");
}

export interface Health {
  status: string;
  version?: string;
  /** 资料库在哪个文件夹。 */
  library?: string;
}

/** 内核推的事件。收到以后重新拉对应的数据就是最新的；落后丢掉的不补。 */
export type LibraryEvent =
  | { type: "item-added"; id: string }
  | { type: "item-updated"; id: string }
  | { type: "item-fetch-failed"; id: string; code: string; hint?: string; retry: boolean }
  | { type: "item-trashed"; id: string }
  /** 回收站里的到期了、或你点了「现在就清掉」：这一条从资料库里没了，再拉是 404。 */
  | { type: "item-removed"; id: string }
  | { type: "topic-updated"; id: string };

/** 获取插件、导入的事件，和 `LibraryEvent` 走同一条事件流。 */
export type PlatformEvent =
  /** 一次导入变了：列了一页、收下了一条、停下了。重新拉 `paths.import(id)`。 */
  | { type: "import-updated"; id: string }
  /** 插件的暂停、站点的登录、访客身份变了。重新拉 `paths.plugins`、`paths.sites`、`paths.visitors`。 */
  | { type: "plugins-updated" };

/** 平台的显示名由获取插件给，内核不认识具体平台。 */
export interface Platform {
  /** 插件给的 id，例如 `web`。 */
  id: string;
  /** 显示名，例如「网页」。 */
  name: string;
}

/** 媒体里的一处：一个时间点（秒，可带结束）或一张图。 */
export type Position =
  | { kind: "time"; sec: number; endSec?: number; snapshotId: string }
  | { kind: "image"; imageKey: string };

/** 一层文字里的一段，带它在原样里的位置。 */
export interface Segment {
  text: string;
  position?: Position;
}

export interface ImageRef {
  /** 资料库里图片的 key；`cover` 是封面。 */
  key: string;
  /** daemon 经 localhost 提供的地址。 */
  url: string;
  width: number;
  height: number;
}

export interface Frame {
  sec: number;
  url: string;
}

export interface Comment {
  author: string;
  text: string;
  likes?: number;
}

/**
 * 一条收下的东西有几个侧面，有哪个给哪个，前端都摆出来，不挑哪个要紧。
 * `remark` 只写显而易见的事，例如「画面一直是同一张图」。
 */
export type Facet =
  | { layer: "asr"; by: "machine" | "platform"; segments: Segment[]; remark?: string }
  | { layer: "ocr"; segments: Segment[]; remark?: string }
  | { layer: "text"; segments: Segment[]; remark?: string }
  | { layer: "image"; images: ImageRef[]; remark?: string }
  | {
      layer: "video";
      snapshotId: string;
      durationSec: number;
      /** 没存下视频文件时省略，只剩画面缩略图。 */
      url?: string;
      frames: Frame[];
      cover?: ImageRef;
      quality?: string;
      remark?: string;
    }
  | { layer: "comments"; comments: Comment[]; remark?: string };

export type FacetLayer = Facet["layer"];

/** 存下了多少：都存好了、存下了一部分（缺的写在 `missing`）、只有链接、只留文字（媒体在回收站）。 */
export type Stored = "complete" | "partial" | "link" | "text-only";

export interface ItemSummary {
  id: string;
  title: string;
  platform: Platform;
  author?: string;
  /** RFC 3339，UTC。 */
  takenAt: string;
  topicIds: string[];
}

export interface Item extends ItemSummary {
  /** 原帖链接。 */
  url: string;
  stored: Stored;
  /** 没存下来的部分和原因，例如「视频没存下来：平台没提供原文件」。 */
  missing?: string;
  note?: string;
  facets: Facet[];
  /** 在回收站里时有；RFC 3339。 */
  trashedAt?: string;
}

export interface Topic {
  id: string;
  name: string;
  followed: boolean;
}

/** 地图上的位置都是 0–1，按地图区域的宽高算；位置由内容相近决定，不会自己挪。 */
export interface MapRegion {
  topicId: string;
  x: number;
  y: number;
  /** 半径，按地图宽度算的比例。 */
  r: number;
  count: number;
}

export interface MapPoint {
  itemId: string;
  x: number;
  y: number;
  title: string;
  platform: Platform;
  topicIds: string[];
  takenAt: string;
}

export interface MapView {
  regions: MapRegion[];
  points: MapPoint[];
}

/** 搜到的地方：哪一层、哪一处，和命中那句话；`match` 是命中词在 `snippet` 里的 [起, 止)。 */
export interface SearchHit {
  itemId: string;
  title: string;
  platform: Platform;
  layer: FacetLayer | "title" | "author" | "note";
  position?: Position;
  snippet: string;
  match: [number, number];
}

export interface TakeRequest {
  /** 一条分享链接、一段分享文字，或拖进来的文件路径。 */
  input: string;
  kind: "link" | "file";
}

export interface TakeResponse {
  id: string;
  /** 以前收下过：补了一次快照，原来存的不动。 */
  seen: boolean;
  title: string;
  platform: Platform;
  takenAt: string;
}

export interface NoteRequest {
  text: string;
}

export interface TopicMembershipRequest {
  topicId: string;
  member: boolean;
}

export interface CreateTopicRequest {
  name: string;
}

export interface FollowRequest {
  followed: boolean;
}

// —— 获取插件 ——

/** 七层取法（协议 §2.4），从低到高。只是给你挑的标签，不拦什么。 */
export type Access =
  | "official-export"
  | "public-web"
  | "vision-ui"
  | "browser-session"
  | "session-api"
  | "private-api"
  | "client-hook";

export type LoginNeed = "none" | "optional" | "required";

/** 一个能列的来源，例如「稍后再看」。 */
export interface PluginSource {
  /** 插件自己定的字符串，开始导入时原样交回。 */
  source: string;
  title: string;
  action?: "favorite" | "like" | "share" | "view" | "unknown";
  count?: number;
}

export interface PluginSetting {
  key: string;
  label: string;
  type: "string" | "bool" | "int" | "enum";
  secret: boolean;
  /** 只有密钥设置有：填了没有。值从不回出来。 */
  filled?: boolean;
}

/** 平台的节奏：两次调用至少隔多久、每天批量最多几次；`batch: false` 的平台不做导入。 */
export interface PluginPacing {
  platform: string;
  batch: boolean;
  intervalSec: number;
  daily?: number;
}

/** 插件停在哪儿、为什么、到什么时候。 */
export interface PluginPause {
  /** 平台 id，或通用网页插件的网站；`*` 是你点的「先别试」。 */
  scope: string;
  /** 你点了「先别试」：到你点「现在再试」为止。 */
  byYou: boolean;
  /** RFC 3339；没有的等你动手。 */
  until?: string;
  /** 插件报的错误码，例如 `rate-limited`、`blocked`。 */
  code?: string;
  /** 插件给你的一句话。 */
  hint?: string;
}

export interface FetchPlugin {
  id: string;
  name: string;
  description?: string;
  version: string;
  platforms: Platform[];
  access: Access;
  login: LoginNeed;
  /** 能列出你存过的。 */
  canList: boolean;
  /** 清单里写死的来源，登录前就能显示；以 `paths.pluginSources` 为准。 */
  sources: PluginSource[];
  /** 会用到哪些站点的账号，登录状态见 `paths.sites`。 */
  sites: string[];
  /** 会用到哪些站点的访客身份（不是账号），有没有见 `paths.visitors`。 */
  visitors: string[];
  settings: PluginSetting[];
  pacing: PluginPacing[];
  pauses: PluginPause[];
}

export interface PluginsView {
  plugins: FetchPlugin[];
  /** 装了但没加载的：目录和原因（多半是清单不合规）。 */
  notLoaded: { dir: string; reason: string }[];
}

export interface SecretRequest {
  value: string;
}

/** 一个账号站点。登录窗口：只开 `loginUrl`，每个站点自己一份网页数据；`cookies` 里的名字都出现了（没有名字时有任何 cookie）就算登录好了，读出 cookie 交给 `paths.siteLogin`，关窗口。 */
export interface SiteLogin {
  /** 主机名，例如 `www.example.com`。 */
  site: string;
  /** 界面上写「会用到你的 X 账号」的 X。 */
  name: string;
  loginUrl?: string;
  /** 登录后才有的 cookie 名。 */
  cookies: string[];
  /** 除了站点本身和它的上级域名，还要读这些域名（和它们的子域）的 cookie。 */
  domains: string[];
  login: "missing" | "present" | "expired";
  /** 上次存下登录的时间，RFC 3339。 */
  capturedAt?: string;
  /** 用到这个站点的插件 id。 */
  plugins: string[];
}

/**
 * 一个访客身份：站点给任何没登录的访客的 cookie，不是账号。访客窗口：只开 `url`，网页数据和同一主机的账号分开；
 * `cookies` 里的名字都出现了（没有名字时名字不再变）就算取到，读出 cookie 交给 `paths.visitor`，关窗口。
 */
export interface VisitorIdentity {
  site: string;
  /** 界面上写「X 首页」的 X。 */
  name: string;
  url: string;
  cookies: string[];
  domains: string[];
  state: "missing" | "present" | "expired";
  /** 上次存下的时间，RFC 3339。 */
  capturedAt?: string;
  /** 最早过期的那个 cookie 什么时候过期，RFC 3339；都是会话 cookie 时没有。 */
  expiresAt?: string;
  /** 用到它的插件 id。 */
  plugins: string[];
}

/** 壳交给 `paths.siteLogin`、`paths.visitor` 的：Playwright storageState 的形状（协议 §6）。 */
export interface StorageState {
  cookies: StoredCookie[];
  origins?: { origin: string; localStorage: { name: string; value: string }[] }[];
}

export interface StoredCookie {
  name: string;
  value: string;
  /** 照 cookie 自己的 domain 属性写，带不带前面的点都行。 */
  domain: string;
  path?: string;
  /** Unix 秒；会话 cookie 写 -1 或不写。 */
  expires?: number;
  httpOnly?: boolean;
  secure?: boolean;
  sameSite?: "Strict" | "Lax" | "None";
}

// —— 导入 ——

/** 一次导入走到哪了：在列 → 列完等你看 → 按节奏收 → 收完；随时可以停下。 */
export type ImportState = "listing" | "ready" | "taking" | "held" | "done";

/**
 * 为什么在等、停着。`code`：插件的错误码（`login-required`、`rate-limited`、`blocked`……），
 * 或 `pacing`（等间隔）、`daily-limit`（今天批量的次数用完了）、`held`（你让插件先别试）、`batch-off`（这个平台不做导入）。
 * 没有 `until` 的等你动手（例如重新登录以后点「接着导入」）。
 */
export interface ImportWaiting {
  code: string;
  hint?: string;
  until?: string;
}

/** 一次导入。只说事实：列了几页、预览里各几条、收下了几条、为什么在等；不给还剩多少。 */
export interface ImportView {
  id: string;
  /** `source`：插件列出来的；`delivered`：交进来的一批链接。 */
  kind: "source" | "delivered";
  title: string;
  pluginId?: string;
  source?: string;
  /** 交进来的那个投递端的名字。 */
  client?: string;
  createdAt: string;
  state: ImportState;
  pages: number;
  /** 预览：新的、库里已经存着的、在回收站里的。 */
  preview: { new: number; stored: number; trashed: number };
  /** 收下了几条。 */
  taken: number;
  /** 收下了、但内容取不到的（条目页写着原因）。 */
  notStored: number;
  /** 轮到时已经进了回收站、或是私密的，没收。 */
  skipped: number;
  /** 下一步大概什么时候，RFC 3339。 */
  nextAt?: string;
  waiting?: ImportWaiting;
}

export interface ImportEntry {
  seq: number;
  url: string;
  title?: string;
  action?: string;
  actedAt?: string;
  itemId?: string;
  standing: "new" | "stored" | "trashed" | "unresolved";
  /** 已经收下了。 */
  taken: boolean;
}

export interface StartImportRequest {
  pluginId: string;
  /** `PluginSource.source`。 */
  source: string;
  /** 不给就用来源的名字。 */
  title?: string;
}

/**
 * 确认导入时你挑的：新的、回收站里的各一个整组的选择（回收站里的收下就是拿回来），
 * `except` 是和自己那组选得不一样的条目 `seq`。只交例外：预览按页看，没翻到的条目跟着整组走。
 * 已经存着的、认不出的不收，也不能出现在 `except` 里。
 */
export interface ConfirmImportRequest {
  new: boolean;
  trashed: boolean;
  except?: number[];
}

// —— 投递 ——

export type DeliverClientKind = "extension" | "mobile" | "api";

export interface DeliverToken {
  id: string;
  name: string;
  kind: DeliverClientKind;
  createdAt: string;
}

export interface CreateDeliverTokenRequest {
  name: string;
  kind: DeliverClientKind;
}

/** 只在发的那一次回：`token` 是 `<id>.<秘密>`，抄给扩展或脚本。 */
export interface NewDeliverToken extends DeliverToken {
  token: string;
}

/** `url` 或 `text`（手机分享出来的整段文字）给一个。 */
export interface DeliverRequest {
  url?: string;
  text?: string;
}

export interface DeliverResponse {
  id: string;
  state: "accepted";
  /** 以前收下过。 */
  duplicate: boolean;
}

export interface BatchRequest {
  name?: string;
  action?: "favorite" | "like" | "share" | "view" | "unknown";
  /** 最多 5000 条。 */
  links: { url: string; title?: string; actedAt?: string }[];
}
