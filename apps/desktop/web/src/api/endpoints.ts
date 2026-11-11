// 内核接口的路径和类型，都在这一个文件里。接口还没定稿，改动只落在这里。

export const DEFAULT_API_BASE = "http://127.0.0.1:47615";

export const paths = {
  health: "/health",
  items: "/items",
  item: (id: string) => `/items/${encodeURIComponent(id)}`,
  search: "/search",
  take: "/items",
} as const;

export interface Health {
  status: string;
  version?: string;
}

export interface ItemSummary {
  id: string;
  title: string;
  /** 来源的一行说明，例如平台和作者。 */
  source: string;
  takenAt: string;
}

export interface Item extends ItemSummary {
  url?: string;
  text: string;
}

export interface SearchHit {
  id: string;
  title: string;
  source: string;
  snippet: string;
}

export interface TakeRequest {
  url: string;
}

export interface TakeResponse {
  id: string;
}
