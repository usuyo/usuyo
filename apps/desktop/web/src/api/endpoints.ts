// 内核接口的路径和类型，都在这一个文件里。接口还没定稿，改动只落在这里；Rust 那边照这里的形状做。
// 层名和位置照 docs/spec/library.md §1.2 的引用：asr、ocr、text、image、video、comments；时间是秒，图片用 imageKey。

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
  /** GET `TrashView`：现在在回收站里的整条和只留文字移出来的媒体，新的在前。 */
  trash: "/trash",
  /** POST：立即清除回收站里的一项（`TrashEntry.id`，不是条目 id），204，撤不了。 */
  trashPurge: (trashId: string) => `/trash/${encodeURIComponent(trashId)}/purge`,
  itemRefetch: (id: string) => `/items/${encodeURIComponent(id)}/refetch`,
  search: "/search",
  take: "/items",
  /** 事件流（SSE）：`event:` 是 `LibraryEvent["type"]` 或 `PlatformEvent["type"]`，`data:` 是事件的 JSON。要带令牌，用 fetch 读。 */
  events: "/events",

  // —— 设置（「设置」页）。内核今天（11-12）还没有这几条。——
  /** GET `SettingsView`；PATCH `SettingsPatch` 改其中几项，回改好的整份。改了马上生效。 */
  settings: "/settings",
  /**
   * POST：时钟守卫拦下的那几项照时间移除（SPEC §1.7「照时间移除」）。回 `{removed}`，之后 `trashHeld` 没了。
   * 只在 `SettingsView.trashHeld` 有的时候给点。
   */
  trashReleaseHeld: "/trash/release-held",

  // —— 获取插件（「获取插件」页）——
  /** GET：装着的获取插件、各自的标签和节奏、停着没有；还有没加载的和原因。 */
  plugins: "/plugins",
  /** GET：这个插件现在能列的来源（会调插件，按平台的节奏可能要等几秒到一分钟）。 */
  pluginSources: (id: string) => `/plugins/${pluginPath(id)}/sources`,
  /** POST：「现在再试」，去掉这个插件的暂停。 */
  pluginTryNow: (id: string) => `/plugins/${pluginPath(id)}/try-now`,
  /** PUT `MethodSwitchRequest`：开或关插件的一种取法，回整个插件。内核今天（11-12）还没有这条。 */
  pluginMethod: (id: string, method: string) => `/plugins/${pluginPath(id)}/methods/${encodeURIComponent(method)}`,
  /** POST：「先别试」，到你点「现在再试」为止哪儿都不调它。 */
  pluginHold: (id: string) => `/plugins/${pluginPath(id)}/hold`,
  /** PUT `{value}` 填一个密钥设置（协议 §2.11），DELETE 去掉。值从不回出来。 */
  pluginSecret: (id: string, key: string) => `/plugins/${pluginPath(id)}/secrets/${encodeURIComponent(key)}`,
  /** GET：装着的插件会用到的账号站点和登录状态。登录窗口照它开。 */
  sites: "/sites",
  /** PUT `StorageState`：壳的登录窗口读出 cookie 以后交到这里（存进钥匙串）；DELETE 忘掉这个站点的登录。 */
  siteLogin: (site: string) => `/sites/${encodeURIComponent(site)}/login`,
  // —— 转换插件和模型（feature 003 §8.1）。内核今天（11-12）都还没有这几条。——
  /**
   * POST `TransformRequest`：`{jobs: "pending"}` 把这条还没转的挪到最前（打开条目页时调）；
   * `{job, plugin?, options?}` 再转一次或换一个插件转，回 202 `{queued: true}`，原来那份留着。
   * 这一层正在转回 `conflict`；插件做不了这一层回 `invalid-args`。
   */
  itemTransform: (id: string) => `/items/${encodeURIComponent(id)}/transform`,
  /** GET `TransformsView`：装了哪些转换插件、各做哪几层、模型和状态。 */
  transforms: "/transforms",
  /** POST：下这个模型（按机器目录，不进资料库）；已经有了回 200。状态变了发 `transforms-updated`。 */
  modelDownload: (pluginId: string, name: string) => `/transforms/${pluginPath(pluginId)}/models/${encodeURIComponent(name)}/download`,
  /** POST：删掉这个模型；之后要它的转换等着模型。 */
  modelRemove: (pluginId: string, name: string) => `/transforms/${pluginPath(pluginId)}/models/${encodeURIComponent(name)}/remove`,

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
  | { type: "plugins-updated" }
  /** 转换插件或模型的状态变了（下好了、删掉了、下载没成）。重新拉 `paths.transforms`。内核今天还不发。 */
  | { type: "transforms-updated" };

/** 平台的显示名由获取插件给，内核不认识具体平台。 */
export interface Platform {
  /** 插件给的 id，例如 `web`。 */
  id: string;
  /** 显示名，例如「网页」。 */
  name: string;
}

/**
 * 媒体里的一处：一个时间点（秒，可带结束），或一张图。`xywh` 是图上（或那一帧上）的框，0–1，按宽高算，原点在左上角；
 * 画面上的字带 `frameUrl`，是认出这一行的那一帧（feature 003 §5.3、§7.1）。
 */
export type Position =
  | { kind: "time"; sec: number; endSec?: number; snapshotId: string; xywh?: [number, number, number, number]; frameUrl?: string }
  | { kind: "image"; imageKey: string; xywh?: [number, number, number, number] };

/** 一层文字里的一段，带它在原样里的位置。 */
export interface Segment {
  text: string;
  position?: Position;
}

/** 一份是从哪来的。 */
export interface FacetSource {
  by: "platform" | "machine";
  /** 机器的：工具和模型的显示名，例如「SenseVoice」「Vision」。 */
  tool?: string;
  runId?: string;
  /** RFC 3339。 */
  at?: string;
  /** 平台字幕、转写的语言。 */
  lang?: string;
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
 * 一条收下的东西有几个侧面，有哪个给哪个，前端都摆出来，不挑哪个要紧（feature 003 §7.1）。
 * 同一层可以有几份（平台的字幕和机器转的，或两次机器转的），都给：平台的在前，机器的按 `source.at` 新的在前。
 * `key` 在一条里唯一：`asr:platform:<snapshotId>:<lang>`、`asr:<runId>`、`ocr:<runId>`、`frames:<runId>`、`video:<snapshotId>`。
 * `remark` 只写量得出来的明显的事（003 §6），例如「没听到人说话」。
 * 内核今天（11-12）还没给 `key`、`source`，也还没有 `asr`、`ocr`、`frames` 三层；`source.ts` 在边界上补 `key`。
 */
export type Facet =
  | { layer: "asr"; key: string; source: FacetSource; segments: Segment[]; remark?: string }
  | { layer: "ocr"; key: string; source: FacetSource; segments: Segment[]; remark?: string }
  /** 画面上的字：关键帧上认出的字，连着几帧一样的一行合成一段，`endSec` 写到最后出现的那一帧。 */
  | { layer: "frames"; key: string; source: FacetSource; segments: Segment[]; remark?: string }
  | { layer: "text"; key: string; segments: Segment[]; remark?: string }
  | { layer: "image"; key: string; images: ImageRef[]; remark?: string }
  | {
      layer: "video";
      key: string;
      snapshotId: string;
      durationSec: number;
      /** 有合起来的 `play.mp4` 时指它，没有指原样的画面；没存下视频文件时省略，只剩关键帧。 */
      url?: string;
      /** 最新一次 `keyframes` 的帧，地址是 `/media/<itemId>/frame/<runId>/<毫秒>`。 */
      frames: Frame[];
      cover?: ImageRef;
      quality?: string;
      remark?: string;
    }
  | { layer: "comments"; key: string; comments: Comment[]; remark?: string };

export type FacetLayer = Facet["layer"];

/** 能再转一次的三层，和转换任务同名。 */
export type TransformLayer = "asr" | "ocr" | "frames";

/**
 * 还没转出来的面，只在条目页那一面写一句（003 §4.4）：排着接着电（`later`）、排着在电池上（`on-power`）、
 * 在转（`now`）、等模型（`needs-model`，带要下的模型）、没转出来（`failed`，带原因）。没有计数、没有百分比。
 * 内核今天（11-12）还不给 `pending`；`source.ts` 在边界上补成空的。
 */
export interface Pending {
  layer: "asr" | "ocr" | "frames" | "video";
  state: "later" | "on-power" | "now" | "needs-model" | "failed";
  /** `plugin` 是转换插件的 id（`owner/repo`），`size` 是字节。 */
  model?: { plugin: string; name: string; size: number; license: string };
  hint?: string;
}

/** 回收站里的一项。`delete` 是你删掉的整条，`undo` 是撤销收下时移进来的整条，拿回来用 `paths.itemRestore`；
 * `keep-text` 是只留文字时移出来的图和视频，条目和文字还在，拿回来用 `paths.itemRestoreMedia`。 */
export interface TrashEntry {
  /** 回收站项的 id，「立即清除」用它。 */
  id: string;
  reason: "delete" | "undo" | "keep-text";
  itemId: string;
  title: string;
  platform: Platform;
  author?: string;
  /** RFC 3339，UTC。 */
  trashedAt: string;
  /** 到这个时间就清除；`trashDays` 是「一直」时没有。到期检查几个小时一次，过了一会儿还在是正常的。 */
  expiresAt?: string;
}

export interface TrashView {
  /** 放多少天；`null` 是「一直」。 */
  trashDays: number | null;
  entries: TrashEntry[];
  /** 读不了、没列出来的回收站项（`id` 是回收站项的）。 */
  unreadable?: { id: string; reason: string }[];
}

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
  /** 还没转出来的面。 */
  pending: Pending[];
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
  /** 读不了、没摆上地图的条目（多半在 usuyo 外改坏了文件）和原因；内核今天就给，都读得了时省略。 */
  unreadable?: { id: string; reason: string }[];
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
  /**
   * 一个平台一个插件，插件里有几种取法，各有自己的档位、账号用在哪、开着没有、停着没有（11-12 定）。
   * 内核今天还不给；`source.ts` 在边界上照插件自己的 `access`、`login` 补成一种。
   */
  methods: FetchMethod[];
}

/**
 * 账号用在哪：`none` 不用；`visitor` 用访客身份（不是账号，`sites` 是 `paths.visitors` 的站点）；
 * `own-browser` 账号留在你自己的浏览器里，由脚本把链接交进来；`own-phone` 账号留在手机的 App 里；
 * `in-usuyo` 你在 usuyo 的窗口里登录（`sites` 是 `paths.sites` 的站点）。`where` 是给人看的账号名，例如「X」。
 */
export interface MethodAccount {
  kind: "none" | "visitor" | "own-browser" | "own-phone" | "in-usuyo";
  sites?: string[];
  where?: string;
}

/** 一种取法。`built: false` 是插件声明了、还没做好的：照样列出来，标明，开不了。 */
export interface FetchMethod {
  id: string;
  /** 例如「访客接口」「浏览器脚本」「官方归档」。 */
  name: string;
  access: Access;
  account: MethodAccount;
  /** 取得到什么：`fetch` 是单条里有的（「正文」「图」「视频」），`list` 是能列的来源名，`delivers: "links"` 是由别处把链接交进来。 */
  gets: { fetch?: string[]; list?: string[]; delivers?: "links" };
  built: boolean;
  enabled: boolean;
  pauses: PluginPause[];
  /** 记下来的一句风险，带出处。只是信息，不拦。 */
  riskNote?: { text: string; source?: { label: string; url: string } };
}

export interface PluginsView {
  plugins: FetchPlugin[];
  /** 装了但没加载的：目录和原因（多半是清单不合规）。 */
  notLoaded: { dir: string; reason: string }[];
}

export interface MethodSwitchRequest {
  enabled: boolean;
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
 * 为什么在等、停着。`code`：插件的错误码（`login-required`、`visitor-required`、`rate-limited`、`blocked`……），
 * 或 `pacing`（等间隔）、`daily-limit`（今天批量的次数用完了）、`held`（你让插件先别试）、`batch-off`（这个平台不做导入）。
 * 没有 `until` 的等你动手（例如重新登录或取到访客身份以后点「接着导入」）。
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

// —— 转换插件和模型（feature 003 §4、§8.1）——

export type TransformRequest = { jobs: "pending" } | { job: TransformLayer; plugin?: string; options?: Record<string, unknown> };

/** 一个模型：`size` 是字节；`state` 照内核 `ModelState`，`failed` 带一句原因。 */
export interface TransformModel {
  name: string;
  /** 给人看的名字，例如「SenseVoice」。 */
  label: string;
  size: number;
  license: string;
  state: "absent" | "downloading" | "present" | "failed";
  hint?: string;
}

/**
 * 一个转换插件：做哪几层（`jobs` 里还可以有 `play`、`keyframes` 这类不出文字的），用哪个工具，模型，能不能跑。
 * `health` 是插件自己报的：`ok`，或 `unavailable`（例如系统版本不够）带 `hint`。
 */
export interface TransformPlugin {
  id: string;
  name: string;
  version: string;
  /** 转成文字时写在那一面头上的工具名，例如「SenseVoice」「Vision」。 */
  tool: string;
  /** 吃什么、出什么，给人看的一句，例如「声音 → 口播文字」。 */
  turns: string;
  jobs: string[];
  /** 自带的第三方程序和它的许可，例如 ffmpeg（LGPL）。 */
  bundled?: { name: string; license: string }[];
  models: TransformModel[];
  health: "ok" | "unavailable";
  hint?: string;
}

export interface TransformsView {
  plugins: TransformPlugin[];
  /** `transforms.auto`：收下以后自己排上转换（只在接着电源时跑）。 */
  auto: boolean;
}

// —— 设置 ——

/**
 * 插件出门走哪条路（宿主的本机代理替插件连出去，协议 §7）：`system` 跟系统代理，和浏览器同一个出口；
 * `direct` 直连；`custom` 用 `proxy` 里填的（`http://`、`https://` 或 `socks5://`）。
 */
export interface NetworkSetting {
  mode: "system" | "direct" | "custom";
  proxy?: string;
}

/** 设置页要的、内核那边的设置。外观只在这台电脑的窗口里，不经内核。 */
export interface SettingsView {
  /** 资料库在哪个文件夹（第一次打开时建在「文稿」里的 `usuyo`）。 */
  library: string;
  /** 回收站放几天：30、90，`null` 是一直（资料库的 `settings.json`）。 */
  trashDays: number | null;
  /**
   * 时钟跳了：这台电脑的时间比记下的最晚时间晚了超过 `trashDays`，到期检查一项也没移除。
   * `count` 是因此留着的项数，`since` 是记下的那个最晚时间。没有拦下时省略。
   */
  trashHeld?: { count: number; since: string };
  network: NetworkSetting & {
    /** 这台电脑开着假 IP 模式的代理（Clash、Surge 一类），宿主已经按它调好了（`host/network.rs`）。 */
    fakeIp: boolean;
  };
  /** `transforms.auto`：收下以后自己转，只在接着电源时跑（feature 003 §4.4）。 */
  transforms: { auto: boolean };
  /** 内核的日志在哪（按机器目录的 `logs/daemon.log`）。 */
  logs: string;
}

export interface SettingsPatch {
  trashDays?: number | null;
  network?: NetworkSetting;
  transforms?: { auto: boolean };
}
