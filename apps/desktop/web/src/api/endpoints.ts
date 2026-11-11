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
  /** 事件流（SSE）：`event:` 是 `LibraryEvent["type"]`，`data:` 是事件的 JSON。要带令牌，用 fetch 读。 */
  events: "/events",
} as const;

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
  | { type: "topic-updated"; id: string };

/** 平台的显示名由获取插件给，内核不认识具体平台。 */
export interface Platform {
  /** 插件给的 id，例如 `bilibili`。 */
  id: string;
  /** 显示名，例如「B站」。 */
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
