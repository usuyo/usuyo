import { DaemonUnreachableError, request } from "./client";
import {
  paths,
  type Health,
  type Item,
  type MapView,
  type SearchHit,
  type TakeRequest,
  type TakeResponse,
  type Topic,
} from "./endpoints";

/** 页面要的全部读写。真的内核和开发用的样例库都实现它。 */
export interface Library {
  health(): Promise<Health>;
  map(): Promise<MapView>;
  topics(): Promise<Topic[]>;
  follow(topicId: string, followed: boolean): Promise<Topic>;
  item(id: string): Promise<Item>;
  search(q: string): Promise<SearchHit[]>;
  take(body: TakeRequest): Promise<TakeResponse>;
  setNote(id: string, text: string): Promise<Item>;
  setTopic(id: string, topicId: string, member: boolean): Promise<Item>;
  trash(id: string): Promise<Item>;
  restore(id: string): Promise<Item>;
  restoreMedia(id: string): Promise<Item>;
  textOnly(id: string): Promise<Item>;
  refetch(id: string): Promise<Item>;
}

const daemon: Library = {
  health: () => request<Health>(paths.health),
  map: () => request<MapView>(paths.map),
  topics: () => request<Topic[]>(paths.topics),
  follow: (topicId, followed) => request<Topic>(paths.topicFollow(topicId), { method: "POST", body: { followed } }),
  item: (id) => request<Item>(paths.item(id)),
  search: (q) => request<SearchHit[]>(paths.search, { query: { q } }),
  take: (body) => request<TakeResponse>(paths.take, { method: "POST", body }),
  setNote: (id, text) => request<Item>(paths.itemNote(id), { method: "POST", body: { text } }),
  setTopic: (id, topicId, member) => request<Item>(paths.itemTopics(id), { method: "POST", body: { topicId, member } }),
  trash: (id) => request<Item>(paths.itemTrash(id), { method: "POST" }),
  restore: (id) => request<Item>(paths.itemRestore(id), { method: "POST" }),
  restoreMedia: (id) => request<Item>(paths.itemRestoreMedia(id), { method: "POST" }),
  textOnly: (id) => request<Item>(paths.itemTextOnly(id), { method: "POST" }),
  refetch: (id) => request<Item>(paths.itemRefetch(id), { method: "POST" }),
};

// 样例库只在 VITE_USUYO_MOCK=1 或开发服务器里用；生产构建里这两个条件都是常量 false，样例库不进包。
const MOCK_FORCED = import.meta.env.VITE_USUYO_MOCK === "1";
const MOCK_FALLBACK = import.meta.env.MODE === "development";

// 地址栏带 ?mock=empty 时样例库从空的开始；要在路由改写地址之前读。
const MOCK_EMPTY =
  (MOCK_FORCED || MOCK_FALLBACK) && typeof location !== "undefined" && new URLSearchParams(location.search).get("mock") === "empty";

let mockLibrary: Promise<Library> | null = null;
let usingMock = MOCK_FORCED;

function mock(): Promise<Library> {
  mockLibrary ??= import("../mock/library").then((m) => m.createMockLibrary(MOCK_EMPTY));
  return mockLibrary;
}

/** 走内核；开发时连不上内核就换成样例库，换过以后一直用它。 */
export async function call<T>(run: (library: Library) => Promise<T>): Promise<T> {
  if ((MOCK_FORCED || MOCK_FALLBACK) && usingMock) return run(await mock());
  try {
    return await run(daemon);
  } catch (error) {
    if (MOCK_FALLBACK && error instanceof DaemonUnreachableError) {
      if (!usingMock) console.info("usuyo: 内核没在运行，开发服务器改用样例库");
      usingMock = true;
      return run(await mock());
    }
    throw error;
  }
}
