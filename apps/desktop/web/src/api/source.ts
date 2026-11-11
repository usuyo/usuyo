import { DaemonUnreachableError, request } from "./client";
import {
  paths,
  type ConfirmImportRequest,
  type CreateDeliverTokenRequest,
  type DeliverToken,
  type FetchPlugin,
  type ImportEntry,
  type ImportView,
  type NewDeliverToken,
  type PluginSource,
  type PluginsView,
  type SiteLogin,
  type StartImportRequest,
  type Health,
  type Item,
  type MapView,
  type SearchHit,
  type TakeRequest,
  type TakeResponse,
  type Topic,
  type VisitorIdentity,
} from "./endpoints";

/** 页面要的全部读写。真的内核和开发用的样例库都实现它。 */
export interface Library {
  health(): Promise<Health>;
  map(): Promise<MapView>;
  topics(): Promise<Topic[]>;
  /** 按名字建一个主题；同名的已经有了就回那个。 */
  createTopic(name: string): Promise<Topic>;
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

/**
 * 「获取插件」「导入」两页要的读写。登录和访客身份的 cookie 不经这里：壳的窗口直接交给内核（src/shell/）。
 */
export interface Platforms {
  plugins(): Promise<PluginsView>;
  sources(pluginId: string): Promise<PluginSource[]>;
  tryNow(pluginId: string): Promise<FetchPlugin>;
  hold(pluginId: string): Promise<FetchPlugin>;
  sites(): Promise<SiteLogin[]>;
  forgetLogin(site: string): Promise<void>;
  visitors(): Promise<VisitorIdentity[]>;
  forgetVisitor(site: string): Promise<void>;
  imports(): Promise<ImportView[]>;
  startImport(body: StartImportRequest): Promise<ImportView>;
  import(id: string): Promise<ImportView>;
  importEntries(id: string, standing: ImportEntry["standing"], offset: number, limit: number): Promise<ImportEntry[]>;
  confirmImport(id: string, body: ConfirmImportRequest): Promise<ImportView>;
  holdImport(id: string): Promise<ImportView>;
  resumeImport(id: string): Promise<ImportView>;
  deleteImport(id: string): Promise<void>;
  deliverTokens(): Promise<DeliverToken[]>;
  createDeliverToken(body: CreateDeliverTokenRequest): Promise<NewDeliverToken>;
  revokeDeliverToken(id: string): Promise<void>;
  inbox(): Promise<{ path: string }>;
}

/** 内核和样例库都实现的全部读写。 */
export type Source = Library & Platforms;

/** 列来源要调插件，内核最多等一分钟多一点。 */
const SOURCES_TIMEOUT_MS = 75_000;

const daemon: Source = {
  health: () => request<Health>(paths.health),
  map: () => request<MapView>(paths.map),
  topics: () => request<Topic[]>(paths.topics),
  createTopic: (name) => request<Topic>(paths.topics, { method: "POST", body: { name } }),
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

  plugins: () => request<PluginsView>(paths.plugins),
  sources: (id) => request<PluginSource[]>(paths.pluginSources(id), { timeoutMs: SOURCES_TIMEOUT_MS }),
  tryNow: (id) => request<FetchPlugin>(paths.pluginTryNow(id), { method: "POST" }),
  hold: (id) => request<FetchPlugin>(paths.pluginHold(id), { method: "POST" }),
  sites: () => request<SiteLogin[]>(paths.sites),
  forgetLogin: (site) => request<void>(paths.siteLogin(site), { method: "DELETE" }),
  visitors: () => request<VisitorIdentity[]>(paths.visitors),
  forgetVisitor: (site) => request<void>(paths.visitor(site), { method: "DELETE" }),
  imports: () => request<ImportView[]>(paths.imports),
  startImport: (body) => request<ImportView>(paths.imports, { method: "POST", body }),
  import: (id) => request<ImportView>(paths.import(id)),
  importEntries: (id, standing, offset, limit) =>
    request<ImportEntry[]>(paths.importEntries(id), {
      query: { standing, offset: String(offset), limit: String(limit) },
    }),
  confirmImport: (id, body) => request<ImportView>(paths.importConfirm(id), { method: "POST", body }),
  holdImport: (id) => request<ImportView>(paths.importHold(id), { method: "POST" }),
  resumeImport: (id) => request<ImportView>(paths.importResume(id), { method: "POST" }),
  deleteImport: (id) => request<void>(paths.import(id), { method: "DELETE" }),
  deliverTokens: () => request<DeliverToken[]>(paths.deliverTokens),
  createDeliverToken: (body) => request<NewDeliverToken>(paths.deliverTokens, { method: "POST", body }),
  revokeDeliverToken: (id) => request<void>(paths.deliverToken(id), { method: "DELETE" }),
  inbox: () => request<{ path: string }>(paths.inbox),
};

// 样例库只在 VITE_USUYO_MOCK=1 或开发服务器里用；生产构建里这两个条件都是常量 false，样例库不进包。
const MOCK_FORCED = import.meta.env.VITE_USUYO_MOCK === "1";
const MOCK_FALLBACK = import.meta.env.MODE === "development";

// 地址栏带 ?mock=empty 时样例库从空的开始；要在路由改写地址之前读。
const MOCK_EMPTY =
  (MOCK_FORCED || MOCK_FALLBACK) && typeof location !== "undefined" && new URLSearchParams(location.search).get("mock") === "empty";

let mockLibrary: Promise<Source> | null = null;
let usingMock = MOCK_FORCED;

/** 已经换成样例库了：事件流不用再连。 */
export function usingMockLibrary(): boolean {
  return usingMock;
}

function mock(): Promise<Source> {
  mockLibrary ??= Promise.all([import("../mock/library"), import("../mock/platforms")]).then(([l, p]) => ({
    ...l.createMockLibrary(MOCK_EMPTY),
    ...p.createMockPlatforms(MOCK_EMPTY),
  }));
  return mockLibrary;
}

/** 走内核；开发时连不上内核就换成样例库，换过以后一直用它。 */
export async function call<T>(run: (source: Source) => Promise<T>): Promise<T> {
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
