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
  // —— 主题页（BUILD 004、005）。内核今天（11-12）还没有这几条。——
  /** GET `TopicPage`：主题自己的那页字、成员、命中方式、机器挑的线头、几个角度。 */
  topic: (id: string) => `/topics/${encodeURIComponent(id)}`,
  /** PUT `PagePatch`：改这一页的字（段的顺序、文字、每段带着的引用），回整个 `TopicPage`。宿主照 SPEC §1.4 写回 Markdown。 */
  topicPage: (id: string) => `/topics/${encodeURIComponent(id)}/page`,
  /** POST `CiteRequest`：把一段原话引用进这一页，接在 `after` 那段后面或页尾，回整个 `TopicPage`。 */
  topicCite: (id: string) => `/topics/${encodeURIComponent(id)}/cite`,
  /** PUT `TopicMatch`：怎么算命中（关键词；相似度要你自己打开），回整个 `TopicPage`。 */
  topicMatch: (id: string) => `/topics/${encodeURIComponent(id)}/match`,
  /** POST：这个线头不要了，204；不留记录，之后不再拿它给你看。 */
  topicThreadDismiss: (id: string, threadId: string) => `/topics/${encodeURIComponent(id)}/threads/${encodeURIComponent(threadId)}/dismiss`,
  /** POST `{question}`：加一个角度（让模型按这个问题读这一主题的段），回整个 `TopicPage`。 */
  topicAngles: (id: string) => `/topics/${encodeURIComponent(id)}/angles`,
  /** POST：这个角度再读一次，新的一次和旧的并排，回整个 `TopicPage`；没选模型回 `model-required`。 */
  topicAngleRun: (id: string, angleId: string) => `/topics/${encodeURIComponent(id)}/angles/${encodeURIComponent(angleId)}/run`,

  // —— 成稿（BUILD 005）。内核今天（11-12）还没有这几条。——
  /** GET `DraftSummary[]`（改过的在前）；POST `NewDraftRequest` 写一篇新的，回 `Draft`。 */
  drafts: "/drafts",
  /** GET `Draft`；PATCH `DraftPatch` 改标题和是哪种，回 `Draft`。标题就是文件名，在访达里改名也是改标题。 */
  draft: (id: string) => `/drafts/${encodeURIComponent(id)}`,
  /** PUT `PagePatch`：改正文，回 `Draft`。 */
  draftPage: (id: string) => `/drafts/${encodeURIComponent(id)}/page`,
  /** POST `CiteRequest`：把一段原话引进来，出处跟着它，回 `Draft`。 */
  draftCite: (id: string) => `/drafts/${encodeURIComponent(id)}/cite`,

  // —— 跟着收（BUILD 009）。内核今天（11-12）还没有这几条。——
  /** GET `FollowsView`；POST `NewFollowRequest` 跟着收一个新的，回 `FollowView`。 */
  follows: "/follows",
  /** PATCH `FollowPatch`（停下、接着、改节奏、改收什么），回 `FollowView`；DELETE 不再跟着收，204，收下的条目都留着。 */
  follow: (id: string) => `/follows/${encodeURIComponent(id)}`,
  /** POST `FollowTrialRequest`：照现在的内容试列一次，回 `FollowTrial`，什么都不收、不记。会调插件，按节奏可能要等一会儿。 */
  followTrials: "/follow-trials",
  /** PATCH `{powerOnly}`：只在接着电源时跑，回 `FollowsView`。 */
  followSettings: "/follow-settings",

  // —— 改动记录与改动提议（BUILD 006）。内核今天（11-12）还没有这几条。——
  /** GET `ChangesView`：一批一批，新的在前。 */
  changes: "/changes",
  /** POST `DecideRequest`：接受这一批提议里的几处（不给 `ops` 是全部），回这一批。 */
  changeAccept: (id: string) => `/changes/${encodeURIComponent(id)}/accept`,
  /** POST `DecideRequest`：不要这一批提议里的几处（不给 `ops` 是全部），资料库不变，回这一批。 */
  changeDecline: (id: string) => `/changes/${encodeURIComponent(id)}/decline`,
  /** POST：整批撤销，回这一批；之后能重做。 */
  changeUndo: (id: string) => `/changes/${encodeURIComponent(id)}/undo`,
  /** POST：撤销过的这一批再做一次，回这一批。 */
  changeRedo: (id: string) => `/changes/${encodeURIComponent(id)}/redo`,

  // —— 导出到 Obsidian（BUILD 007）。内核今天（11-12）还没有这几条。——
  /** GET `ExportView`：导到哪、会写出什么、一篇笔记的样子、导出记录。 */
  exportView: "/export",
  /** PUT `{folder}`：换导出的文件夹（按机器存，不跟着同步），回 `ExportView`。文件夹不在或写不了回 `invalid-args`。 */
  exportTarget: "/export/target",
  /** POST：现在导出。先读回上次导出以后在那边改的，合进来，再写出；回这一次的 `ExportRun`。只在你点的时候跑。 */
  exportRuns: "/export/runs",

  /** 事件流（SSE）：`event:` 是 `LibraryEvent["type"]` 或 `PlatformEvent["type"]`，`data:` 是事件的 JSON。要带令牌，用 fetch 读。 */
  events: "/events",

  // —— 设置（「设置」页）——
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
  /**
   * GET `PluginSource[]`，`?method=<取法 id>`（必给），`&link=<链接>`（只给 `gets.link` 有的取法）：这种取法现在能列的来源。
   * 会调插件，按平台的节奏可能要等几秒到一分钟；这种取法不能列、这台机器上用不上、要等太久回 `unsupported-input`。
   */
  pluginSources: (id: string) => `/plugins/${pluginPath(id)}/sources`,
  /** POST：「现在再试」，去掉这个插件的暂停。 */
  pluginTryNow: (id: string) => `/plugins/${pluginPath(id)}/try-now`,
  /** PUT `MethodSwitchRequest`：开或关插件的一种取法，回整个插件。没做好的取法回 `unsupported-input`。 */
  pluginMethod: (id: string, method: string) => `/plugins/${pluginPath(id)}/methods/${encodeURIComponent(method)}`,
  /** POST：「先别试」，到你点「现在再试」为止哪儿都不调它。 */
  pluginHold: (id: string) => `/plugins/${pluginPath(id)}/hold`,
  /** PUT `{value}` 填一个密钥设置（协议 §2.11），DELETE 去掉。值从不回出来。 */
  pluginSecret: (id: string, key: string) => `/plugins/${pluginPath(id)}/secrets/${encodeURIComponent(key)}`,
  /** GET：装着的插件会用到的账号站点和登录状态。登录窗口照它开。 */
  sites: "/sites",
  /** PUT `StorageState`：壳的登录窗口读出 cookie 以后交到这里（存进钥匙串）；DELETE 忘掉这个站点的登录。 */
  siteLogin: (site: string) => `/sites/${encodeURIComponent(site)}/login`,
  // —— 转换插件和模型（feature 003 §8.1）——
  /**
   * POST `TransformRequest`，回 202 `{queued: true}`。`{jobs: "pending"}`：这一条还没做好的转换（没转出来的也算）都挪到最前，
   * 在电池上、`transforms.auto` 关着时也转；打开条目页时调一次，「再试」也是它。`{job, plugin?, options?}`：再转一次或换一个插件转，
   * 原来那份留着。这一样正在转回 `conflict`；插件做不了它回 `invalid-args`；只留文字以后回 `unsupported-input`。
   */
  itemTransform: (id: string) => `/items/${encodeURIComponent(id)}/transform`,
  /** GET `TransformsView`：装了哪些转换插件、各做哪几层、模型和状态。 */
  transforms: "/transforms",
  /** POST：下这个模型（按机器目录，不进资料库），回 `ModelDownload`：已经有了 200 `present`，开始下或在下 202 `downloading`。状态变了发 `transforms-updated`。 */
  modelDownload: (pluginId: string, name: string) => `/transforms/${pluginPath(pluginId)}/models/${encodeURIComponent(name)}/download`,
  /** POST：删掉这个模型；之后要它的转换等着模型。 */
  modelRemove: (pluginId: string, name: string) => `/transforms/${pluginPath(pluginId)}/models/${encodeURIComponent(name)}/remove`,

  // —— 手机（「手机」页，docs/platforms/phone.md §4）。内核今天（11-12）还没有这几条。——
  /** GET `PhoneView`：这台电脑上 iPhone 镜像那条路和安卓 adb 那条路各自怎样。用到手机的取法不在这里，在 `paths.plugins`。 */
  phone: "/phone",
  /** POST：装 mirroir（官方 release 的 `darwin-arm64` 包，钉版本、对 `SHA256SUMS`，放按机器目录，再跑 `install-agent`），马上回 `PhoneView`（`installing`）；装好或没装上发 `phone-updated`。 */
  phoneDriverInstall: "/phone/ios/driver/install",
  /** POST：换成 `PhoneDriver.available` 那一版，同上。 */
  phoneDriverUpdate: "/phone/ios/driver/update",
  /** POST：停掉「mirroir serve」、卸掉 mirroir 和它的常驻项，回 `PhoneView`。系统设置里给它的授权由系统留着。 */
  phoneDriverRemove: "/phone/ios/driver/remove",
  /** POST `{pane: SettingsPane}`：打开系统设置的那一栏（或 iPhone 镜像这个 App），204。 */
  phoneOpenSettings: "/phone/open-settings",
  /** POST：看 Mac 剪贴板的 `changeCount` 变不变（`PhoneClipboardTest.waitSec` 秒），马上回 `PhoneView`（`waiting`）；有结果发 `phone-updated`。不读、不存剪贴板里的字。 */
  phoneClipboardTest: "/phone/ios/clipboard-test",
  /** POST：演练模式读一次镜像的这一屏，回 `PhoneDryRun`。什么都不点；读到的字和截图不存、不进日志。 */
  phoneDryRun: "/phone/ios/dry-run",

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

  // —— 别的工具导出的文件（「导入」页的另一半，协议 §3.9）——
  /** GET `FileReader[]`：能读文件的方法（做了的、`list` 写了 `files` 的），照插件和清单的顺序。 */
  fileReaders: "/file-readers",
  /**
   * POST 文件本身，`?reader=<FileReader.id>&name=<文件名>`（`#` 写成 `%23`），最多 16 GiB，超了 413 带 `hint`：
   * 内核存一份副本，原文件不动，马上列第一页，回 `ImportView`（`kind: "file"`，`state: "listing"` 或 `ready`）；
   * 之后和别的导入一样看预览、确认。插件读不了这个文件回 400 `invalid-args`，`hint` 是插件的那句话，什么都不留；
   * 那种方法关着或用不上回 422 `unsupported-input`。
   */
  fileImports: "/file-imports",
} as const;

/**
 * 投递接口（协议 §5.1）：浏览器扩展、油猴脚本、手机用自己的投递令牌调，壳和页面不用。
 * 投递令牌只能进这几条，读不了资料库；带 `Origin` 的只认 `chrome-extension://` 一类。
 * 发令牌以前先握手：`POST /handshake {nonce, client: <令牌 . 前面的 id>}`，证明用整个令牌算。
 */
export const deliverPaths = {
  ping: "/v1/ping",
  /** POST `DeliverRequest` → 202 `DeliverResponse`。 */
  deliver: "/v1/deliver",
  /** POST `BatchRequest` → 202 `{id}`：进导入的预览。 */
  batches: "/v1/batches",
  /** POST 一个装着已取到内容的文件，`?reader=&name=`，最多 256 MiB → 202 `{id}`；只认 `extension` 类令牌。 */
  fileImports: "/v1/file-imports",
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
  /** 插件的暂停、站点的登录、访客身份、密钥设置、设置里的 `network` 变了。重新拉 `paths.plugins`、`paths.sites`、`paths.visitors`、`paths.settings`。 */
  | { type: "plugins-updated" }
  /** 模型开始下、下好了、没下成、删掉了，或设置里的 `transforms.auto` 变了。重新拉 `paths.transforms`、`paths.settings`。 */
  | { type: "transforms-updated" }
  /** 手机那一页的事变了：mirroir 装好、没装上、卸掉，授权、镜像连上断开，剪贴板试出了结果，adb 的设备来去。重新拉 `paths.phone`。 */
  | { type: "phone-updated" };

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
  /** 机器的：做这一份的转换插件 id（`owner/repo`）；宿主自己判的没有。 */
  plugin?: string;
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
 * `remark` 只写量得出来的明显的事（003 §6），例如「没听到人说话」；作者写的字和评论没有。
 */
export type Facet =
  | { layer: "asr"; key: string; source: FacetSource; segments: Segment[]; remark?: string }
  | { layer: "ocr"; key: string; source: FacetSource; segments: Segment[]; remark?: string }
  /** 画面上的字：关键帧上认出的字，连着几帧一样的一行合成一段，`endSec` 写到最后出现的那一帧。 */
  | { layer: "frames"; key: string; source: FacetSource; segments: Segment[]; remark?: string }
  | { layer: "text"; key: string; segments: Segment[] }
  | { layer: "image"; key: string; images: ImageRef[]; remark?: string }
  | {
      layer: "video";
      key: string;
      snapshotId: string;
      durationSec: number;
      /** 有合起来的 `play.mp4` 时指它（带 `v=<那次 play 的 runId>`），没有指原样的画面；没存下视频文件时省略，只剩关键帧。 */
      url?: string;
      /** 最新一次 `keyframes` 的帧，地址是 `/media/<itemId>/frame/<runId>/<毫秒>`。 */
      frames: Frame[];
      remark?: string;
    }
  | { layer: "comments"; key: string; comments: Comment[] };

export type FacetLayer = Facet["layer"];

/** 转出文字的三层，和转换任务同名。 */
export type TransformLayer = "asr" | "ocr" | "frames";

/** 转换任务：三层文字，加画面的两样（`play` 合成能播的一份，`keyframes` 取关键帧）。 */
export type TransformJob = TransformLayer | "play" | "keyframes";

/**
 * 还没转出来的面，只在条目页那一面写一句（003 §4.4）：排着接着电（`later`）、排着在电池上（`on-power`）、
 * 在转（`now`）、等模型（`needs-model`，带要下的模型）、没转出来（`failed`，带原因）。没有计数、没有百分比。
 * `video` 是画面的 `play`、`keyframes`。同一面有几样在等时只给一行。
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

// —— 你写的那一层：主题页、成稿共用 ——

/** 一段能引用的原话：哪一条、哪一层、哪一处。引用按这几样找回，不认哪一次转录（重新识别不断）。 */
export interface SegmentRef {
  itemId: string;
  title: string;
  platform: Platform;
  author?: string;
  layer: FacetLayer;
  position?: Position;
  text: string;
}

/**
 * 页里的一处引用（SPEC §1.4.3 的脚注）。原话、标题、作者、日期是插入时抄下的，之后不跟着刷新。
 * `found`：照原样找回了（`same`）；那一秒的文字变了，`now` 是现在的（`changed`，两句并排看）；
 * 文字类找不到了，显示存下的原话（`missing`）；被引的条目清掉了，`itemId` 是 null，写「来源已删掉」（`gone`）。
 */
export interface Citation {
  label: string;
  itemId: string | null;
  layer: FacetLayer;
  position?: Position;
  quote: string;
  title: string;
  author?: string;
  /** 发布日期，`YYYY-MM-DD`。 */
  date: string;
  found: "same" | "changed" | "missing" | "gone";
  now?: string;
}

/** 页里的一段：你写的字，段末带着的几处引用（`Citation.label`）。只放引用、不写字的段 `text` 是空的。 */
export interface PageBlock {
  id: string;
  kind: "text" | "heading";
  text: string;
  cites: string[];
}

/** 你写的一页：普通 Markdown 文件，不开 usuyo 也能读。 */
export interface WrittenPage {
  /** 资料库里的路径，例如 `topics/咖啡.md`。 */
  file: string;
  blocks: PageBlock[];
  citations: Citation[];
}

/** 改一页：整页的段交回来；新段不带 `id`，宿主起。没列出的引用从页尾去掉。 */
export interface PagePatch {
  blocks: (Omit<PageBlock, "id"> & { id?: string })[];
}

export interface CiteRequest {
  segment: Pick<SegmentRef, "itemId" | "layer" | "position" | "text">;
  /** 接在哪一段后面；不给接在页尾。 */
  after?: string;
}

/** 怎么算命中：关键词按字面看标题层和内容层；相似度按和成员相近，要你自己打开（SPEC §1.4.1）。 */
export interface TopicMatch {
  keywords: string[];
  similarity: boolean;
}

/** 机器拼出来的一组相近的段：标「机器挑的」，只摘原话，不起名、不写总结；没有「看过」。 */
export interface Thread {
  id: string;
  segments: SegmentRef[];
}

/** 一个角度：让模型按一个问题去读这一主题的段。每读一次留一份，并排放；引用由机器另外核对到原话。 */
export interface Angle {
  id: string;
  question: string;
  runs: AngleRun[];
}

export interface AngleRun {
  id: string;
  at: string;
  /** 读的模型，例如「Qwen3 8B · 本机」。 */
  model: string;
  /** 模型写的几点，每点带它引的原话；`checked` 是核对到了原话。 */
  points: { text: string; cites: { segment: SegmentRef; checked: boolean }[] }[];
}

export interface TopicPage extends Topic {
  page: WrittenPage;
  match: TopicMatch;
  /** 你放进来的条目。命中不自动变成成员。 */
  members: ItemSummary[];
  /** 按关键词命中、还没放进来的（机器挑的），带命中的词。 */
  hits: { item: ItemSummary; words: string[] }[];
  threads: Thread[];
  angles: Angle[];
}

/** 成稿是哪一种（SPEC §1.4.2 的 `kind`）。 */
export type DraftKind = "article" | "post" | "script" | "other";

export interface DraftSummary {
  id: string;
  title: string;
  kind: DraftKind;
  createdAt: string;
  updatedAt: string;
}

/** 一篇成稿：你自己的东西，每处引用都带着出处；`from` 是用到的主题（引过它们页里的段）。 */
export interface Draft extends DraftSummary {
  page: WrittenPage;
  from: { id: string; name: string }[];
}

export interface NewDraftRequest {
  title: string;
  kind: DraftKind;
}

export interface DraftPatch {
  title?: string;
  kind?: DraftKind;
}

// —— 跟着收 ——

/** 跟着什么：你在平台上的一个收藏夹或列表、一个创作者的主页、一个订阅源（RSS、Atom、JSON Feed），或者关键词。 */
export type FollowKind = "list" | "creator" | "feed" | "keywords";

/**
 * 收什么：`all` 新出现的都收（你自己的收藏夹只能这样，不逐条看）；
 * 不是 `all` 的只收命中的：`topics` 是关注着的主题（照它们的关键词），`words` 是另加的词。按词命中的都标「机器挑的」。
 */
export type FollowTake = { all: true } | { all: false; topics: string[]; words: string[] };

/** 这台电脑上能跟着收的：哪个插件的哪种取法能列这一种，没做好的也列出来。 */
export interface FollowOption {
  kind: FollowKind;
  pluginId: string;
  pluginName: string;
  methodName: string;
  built: boolean;
  /** `list` 的：能跟着的收藏夹和列表。 */
  sources?: PluginSource[];
  /** 插件声明的最快节奏，小时。 */
  minHours: number;
}

export interface FollowView {
  id: string;
  kind: FollowKind;
  title: string;
  pluginId: string;
  pluginName: string;
  /** `list` 的：`PluginSource.source`。 */
  source?: string;
  /** `creator`、`feed` 的地址；`keywords` 没有。 */
  url?: string;
  take: FollowTake;
  /** 多久看一次，小时。 */
  everyHours: number;
  /** 你停下了的是 false。 */
  on: boolean;
  lastAt?: string;
  nextAt?: string;
  /** 为什么停着：平台限频（带 `until`，到点自己再试）、要重新登录、插件不能用（等你动手）。只停这一个。 */
  waiting?: ImportWaiting;
}

/** 跟着收最近收下的一条。`picked` 有的是按词命中收下的，标「机器挑的」，一直留着这个来历。 */
export interface FollowTaken {
  itemId: string;
  title: string;
  platform: Platform;
  takenAt: string;
  followId: string;
  picked?: { words: string[]; topicIds: string[] };
}

export interface FollowsView {
  follows: FollowView[];
  /** 最近收下的，新的在前。 */
  recent: FollowTaken[];
  options: FollowOption[];
  /** 只在接着电源时跑（默认开）。 */
  powerOnly: boolean;
}

export interface FollowTrialRequest {
  kind: FollowKind;
  pluginId: string;
  source?: string;
  url?: string;
  /** `keywords` 的词。 */
  words?: string[];
  take: FollowTake;
}

/**
 * 试列一次的结果：现在列得到的里头，照「收什么」会收下哪些。
 * `firstRun`：`record` 是第一次只记下现在有的、不收，以后新出现的才收（创作者、订阅源、关键词）；`take` 是新加进去的直接收（你自己的收藏夹）。
 */
export interface FollowTrial {
  title: string;
  firstRun: "record" | "take";
  /** 这一次列到了几条。 */
  listed: number;
  would: { title: string; url: string; actedAt?: string; picked?: string[] }[];
  /** 列到了、但不会收的：库里已经存着的、在回收站里的、没命中的。 */
  left: { stored: number; trashed: number; unmatched: number };
}

export type NewFollowRequest = FollowTrialRequest & { title?: string; everyHours: number };

export interface FollowPatch {
  on?: boolean;
  everyHours?: number;
  take?: FollowTake;
}

// —— 改动记录与改动提议 ——

/** 谁做的：你、哪个 Agent、导出读回、同步来的；都带是哪台电脑。 */
export type ChangeActor =
  | { kind: "you"; device: string }
  | { kind: "agent"; name: string; device: string }
  | { kind: "readback"; target: string; device: string }
  | { kind: "sync"; device: string };

/** 一批里的一处：动了什么、怎么动的，文字类的带改之前和改之后。提议里的每一处有自己的决定。 */
export interface ChangeOp {
  id: string;
  target: { kind: "item" | "topic" | "card" | "draft"; id: string; title: string };
  op: "add-item" | "set-note" | "edit-page" | "add-citation" | "add-to-topic" | "keep-text" | "restore";
  before?: string;
  after?: string;
  /** 只在提议里有：还没定、接受了、不要了。 */
  decision?: "open" | "accepted" | "declined";
}

/**
 * 一批改动。`applied` 生效了；`proposed` 是提议（Agent 改已有内容），等你一处处或整批定；
 * `declined` 是整批都不要了，资料库没变；`undone` 撤销了，能重做。提议不作废，也不计数、不提醒。
 */
export interface ChangeBatch {
  id: string;
  at: string;
  by: ChangeActor;
  state: "applied" | "proposed" | "declined" | "undone";
  /** 一句来历，例如「经 MCP」「导出前从 Obsidian 读回」。 */
  via: string;
  ops: ChangeOp[];
  /** 撤销会连带的事，撤之前问你一次，例如「其中 1 条你记过一句，一起撤掉」。 */
  undoNote?: string;
}

export interface ChangesView {
  batches: ChangeBatch[];
}

export interface DecideRequest {
  ops?: string[];
}

// —— 导出 ——

/** 一次导出：先读回、再写出。数字只说这一次做了什么。 */
export interface ExportRun {
  id: string;
  at: string;
  state: "done" | "failed";
  /** 没成的原因。 */
  hint?: string;
  /** 写出了几篇笔记。 */
  written: number;
  /** 读回合进来的：哪篇、改了什么；记在改动记录里的那一批。 */
  readBack: { file: string; title: string; what: string; batchId?: string }[];
  /** 两边都改过同一处：留下 usuyo 这份，Obsidian 那段进回收站，在那里能「换上这段」。 */
  both: { file: string; title: string; kept: string; theirs: string }[];
  /** 这次没读也没写的，和为什么；在那边改好，下次照常读回。 */
  left: { file: string; reason: string }[];
}

export interface ExportView {
  /** 导出插件，例如「Obsidian」。 */
  target: { pluginId: string; name: string };
  /** 导到哪个文件夹；还没选是 null。按机器记，不跟着同步。 */
  folder: string | null;
  /** 这一次会写出的：条目、主题页、成稿各几篇；私密的不导出。 */
  will: { items: number; topics: number; drafts: number; privateLeft: number };
  /** 一篇条目笔记的样子：分隔线以上是你的（读回），以下每次导出照 usuyo 重写。 */
  sample: { file: string; yours: string; ours: string };
  /** 导出记录，新的在前。 */
  runs: ExportRun[];
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
  /** 属于哪种取法（`FetchMethod.id`），开始导入时一起交回。 */
  method: string;
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
  /** 停的是哪种取法（`FetchMethod.id`）；`*` 是你点的「先别试」，整个插件。 */
  method: string;
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
  /** 清单里写死的来源，各带自己的取法，登录前就能显示；以 `paths.pluginSources` 为准。 */
  sources: PluginSource[];
  /** 做好了的取法会用到哪些站点的账号，登录状态见 `paths.sites`。 */
  sites: string[];
  /** 会用到哪些站点的访客身份（不是账号），有没有见 `paths.visitors`。 */
  visitors: string[];
  settings: PluginSetting[];
  pacing: PluginPacing[];
  /** 所有取法的暂停。 */
  pauses: PluginPause[];
  /** 清单里的每一种取法，照清单的顺序，没做好的也在（协议 §2.3.1）。 */
  methods: FetchMethod[];
}

/**
 * 账号用在哪：`none` 不用；`visitor` 用访客身份（不是账号，`sites` 是 `paths.visitors` 的站点）；
 * `own-browser` 账号留在你自己的浏览器里，由脚本把链接交进来；`own-phone` 账号留在手机的 App 里；
 * `own-app` 账号留在这台电脑上的客户端里；`in-usuyo` 你在 usuyo 的窗口里登录（`sites` 是 `paths.sites` 的站点）；
 * `developer` 用你自己申请的开发者凭据（平台的官方 API）。`where` 是账号所在的地方给人看的名字：`visitor` 是访客身份的名字，
 * `in-usuyo`、`own-browser`、`own-phone`、`own-app` 是方法列的站点的名字，没列站点时是插件的名字；`none`、`developer` 没有。
 */
export interface MethodAccount {
  kind: "none" | "visitor" | "own-browser" | "own-phone" | "own-app" | "in-usuyo" | "developer";
  sites?: string[];
  where?: string;
}

/** 取一条时拿得到的几样（协议的输出词表）；给人看的话在 `plugins/words.ts`。 */
export type OutputKind = "text" | "html" | "images" | "media:video" | "media:audio" | "subtitle" | "meta" | "comments";

/**
 * 这种取法这台机器上为什么用不上，一项一句：`not-built`、`off`、`login-missing`、`login-expired`、`visitor-missing`、
 * `visitor-expired`、`check-failed`（插件的本机检查没过）、`health-failed`（插件没能自查）、`paused`（停着：被限流、被拦，
 * `hint` 写到几点；或你点的「先别试」，写到你点「现在再试」为止）。`hint` 是内核写好的中文。
 */
export interface MethodBlocker {
  code: string;
  hint: string;
}

/** 这种取法上一次调用：`ok` 为假时带错误码和一句话。 */
export interface MethodCall {
  at: string;
  ok: boolean;
  code?: string;
  hint?: string;
}

/**
 * 一种取法在某个平台上的层级和方法本身不同（清单的 `perPlatform`）。`platform` 是 `FetchPlugin.platforms` 里的 id；
 * `enabled`：没拨过开关时照那一层的默认（`private-api`、`client-hook` 关着），拨过就和方法的 `enabled` 一样。
 */
export interface PlatformTier {
  platform: string;
  access: Access;
  login: LoginNeed;
  enabled: boolean;
}

/** 一种取法。`built: false` 是插件声明了、还没做好的：照样列出来，标明，开不了。 */
export interface FetchMethod {
  id: string;
  /** 例如「访客接口」「浏览器脚本」「官方归档」。 */
  name: string;
  access: Access;
  login: LoginNeed;
  account: MethodAccount;
  /** `foreground`：调用时你看得见（弹菜单、动剪贴板、占手机屏幕这一类）；只用来排程，不拦。 */
  interaction: "background" | "foreground";
  /** 驱动哪一样：`iphone`（经 iPhone 镜像）、`android`（经 adb），`desktop-app`（这台电脑上一个客户端的窗口）。 */
  drives?: "iphone" | "android" | "desktop-app";
  /** 层级和方法本身不同的平台；没有这样的平台时不给。开关只有一个，拨过以后各平台都照它。 */
  perPlatform?: PlatformTier[];
  /**
   * 取得到什么（没做好的也给，说的是做好以后）：`fetch` 是取一条时拿得到的几样，`list` 是清单里写死的来源名（能列、没有写死的是空的），
   * `link` 是来源从一条链接开始时链接框上的那句，`delivers: "links"` 是由别处把链接交进来。
   */
  gets: { fetch?: OutputKind[]; list?: string[]; link?: string; delivers?: "links" };
  built: boolean;
  /** 开关：你拨过的照你拨的，没拨过照方法本身的那一层。 */
  enabled: boolean;
  /** 这台机器上现在用得上：做好了、开着、要的登录和访客身份都在、本机检查没挡它、没停着。 */
  usable: boolean;
  blockers: MethodBlocker[];
  lastCall?: MethodCall;
  /** 这种取法的暂停，加上你点的「先别试」。 */
  pauses: PluginPause[];
  /** 记下来的一句风险，带出处。只是信息，不拦。 */
  riskNote?: { text: string; source: { label: string; url: string } };
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
  /** `source`：插件列出来的；`delivered`：交进来的一批链接；`file`：别的工具导出的文件读出来的。 */
  kind: "source" | "delivered" | "file";
  title: string;
  /** `file` 的：交来的文件名（洗过的，只当显示的字）和读它的插件的名字。 */
  file?: { name: string; reader: string };
  pluginId?: string;
  /** 来源属于插件的哪种取法；交进来的没有。 */
  method?: string;
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
  /** 从文件导入时只有插件的 `ref`、没有链接的那几条没有。 */
  url?: string;
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
  /** `PluginSource.method`。 */
  method: string;
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

/**
 * 一种能读别的工具导出的文件的方法（`GET /file-readers`）：做了的、`list` 写了 `files` 的。
 * 名字、读什么、文件长什么样都取自插件清单，应用里不写死任何平台。
 */
export interface FileReader {
  /** `<插件 id>#<方法 id>`，交文件时原样给回；同一种方法也在插件的 `methods` 里，别处靠它不重复列。 */
  id: string;
  /** 方法的名字，例如「官方归档」。 */
  name: string;
  /** 插件的名字。 */
  plugin: string;
  /** 清单的 `list.reads`：读哪个工具的哪种导出。 */
  reads: string;
  /** 清单的 `list.files`，例如 `twitter-*.zip`；只拿来提示和挑文件，内核不按它拒。 */
  patterns: string[];
  /** 清单 `list.sources` 的名字；没写死来源时是空的。 */
  lists: string[];
  /** 方法的 `risk.text`。只是信息。 */
  note?: string;
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

export type TransformRequest = { jobs: "pending" } | { job: TransformJob; plugin?: string; options?: Record<string, unknown> };

/** `POST paths.modelDownload` 的回答。 */
export interface ModelDownload {
  state: "present" | "downloading";
}

/** 一个模型：`size` 是字节；`state` 照内核 `ModelState`，`failed` 带一句原因。 */
export interface TransformModel {
  name: string;
  size: number;
  license: string;
  state: "absent" | "downloading" | "present" | "failed";
  hint?: string;
}

/**
 * 一个转换插件：做哪几样，自带的程序，模型，能不能跑。
 * `health` 是插件自己报的：`ok`，或 `unavailable` 带 `hint`（没过的检查；声明的模型还没下时只写「要先下载模型」）。
 */
export interface TransformPlugin {
  id: string;
  name: string;
  version: string;
  jobs: TransformJob[];
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

// —— 手机 ——

/** 系统给没给：`unknown` 是读不出来（例如「mirroir serve」还没跑过一次）。 */
export type GrantState = "given" | "missing" | "unknown";

/** `phoneOpenSettings` 能打开的：系统设置里两项授权那一栏，和 iPhone 镜像这个 App（不是设置）。 */
export type SettingsPane = "screen-recording" | "accessibility" | "iphone-mirroring";

/** 手机的驱动 mirroir（mirroir-mcp，Apache-2.0）在这台电脑上的样子。 */
export interface PhoneDriver {
  state: "absent" | "installing" | "present" | "failed";
  /** 装着的版本，例如 `0.47.0`。 */
  version?: string;
  /** usuyo 钉的那一版：没装时是要装的，装着的旧了是能换的；装着的就是它时省略。`size` 是字节，`source` 是官方 release 的地址。 */
  available?: { version: string; size: number; source: string };
  /** 没装上的原因，一句话（`failed`）。 */
  hint?: string;
}

/** 试一下通用剪贴板：在等、到了、`waitSec` 秒里没变。拷的字从不回出来。 */
export interface PhoneClipboardTest {
  state: "waiting" | "arrived" | "unchanged";
  /** 开始试的时间，RFC 3339。 */
  at: string;
  waitSec: number;
}

export interface PhoneIos {
  /** macOS 的版本，例如 `15.6.1`；`mirroring` 是有没有 iPhone 镜像（要 macOS 15）。 */
  system: { macos: string; mirroring: boolean };
  driver: PhoneDriver;
  /** 授权给的是 `to`（「mirroir serve」），不是 usuyo。驱动没装时两项都是 `unknown`。 */
  grants: { to: string; screen: GrantState; accessibility: GrantState };
  /** iPhone 镜像：没设置过、手机不在（解锁着或不在旁边）、连着。 */
  mirroring: "not-set-up" | "away" | "connected";
  /** 最近一次试剪贴板；没试过省略。 */
  clipboard?: PhoneClipboardTest;
}

/** `adb devices` 的一台：要在手机上点允许 USB 调试、连好了、连着但不回话。 */
export interface AndroidDevice {
  serial: string;
  /** 型号，例如 `Pixel 8`；没授权前读不到。 */
  model?: string;
  state: "unauthorized" | "device" | "offline";
}

export interface PhoneAndroid {
  /** 找到的 adb；没找到是 null。 */
  adb: { path: string; version: string } | null;
  devices: AndroidDevice[];
}

export interface PhoneView {
  ios: PhoneIos;
  android: PhoneAndroid;
}

/**
 * 演练一次：读这一屏，说它像哪个流程的第几步，什么都没点。`read` 是读到几段字；认不出时没有 `match`。
 * `expect` 是这一步该有的字对上了几段，`targets` 是这一步要点的每个字找没找到，`cards` 是列表里读到几张卡片。
 */
export interface PhoneDryRun {
  read: number;
  match?: {
    /** 给人看的流程名，例如「小红书 · 收藏」。 */
    flow: string;
    /** 第几步，从 1 数。 */
    step: number;
    expect: { matched: number; total: number };
    targets: { label: string; found: boolean }[];
    cards?: number;
  };
}

// —— 设置 ——

/**
 * 插件出门走哪条路（宿主的本机代理替插件连出去，协议 §7）：`system` 跟系统代理，和浏览器同一个出口；
 * `direct` 直连；`custom` 用 `proxy` 里填的（`http://`、`socks5://` 或 `socks5h://`，有主机、没有路径）。
 * 读回来的 `proxy` 不带密码；交回同一个不带密码的地址时，内核留着存着的密码。
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
