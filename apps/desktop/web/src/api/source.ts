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
  type Facet,
  type Health,
  type Item,
  type Pending,
  type SettingsPatch,
  type SettingsView,
  type TransformRequest,
  type TransformsView,
  type MapView,
  type SearchHit,
  type TakeRequest,
  type TakeResponse,
  type Topic,
  type TrashView,
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
  /** 回收站里现在有的。 */
  trashList(): Promise<TrashView>;
  /** 立即清除回收站里的一项（回收站项的 id）。 */
  purge(trashId: string): Promise<void>;
}

/**
 * 「获取插件」「导入」两页要的读写。登录和访客身份的 cookie 不经这里：壳的窗口直接交给内核（src/shell/）。
 */
export interface Platforms {
  plugins(): Promise<PluginsView>;
  sources(pluginId: string): Promise<PluginSource[]>;
  tryNow(pluginId: string): Promise<FetchPlugin>;
  hold(pluginId: string): Promise<FetchPlugin>;
  /** 开或关一种取法。 */
  switchMethod(pluginId: string, methodId: string, enabled: boolean): Promise<FetchPlugin>;
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

/** 转换插件和模型（feature 003 §8.1）。 */
export interface Transforms {
  transforms(): Promise<TransformsView>;
  transform(itemId: string, body: TransformRequest): Promise<void>;
  downloadModel(pluginId: string, name: string): Promise<void>;
  removeModel(pluginId: string, name: string): Promise<void>;
}

/** 设置页的读写。外观不在这里，在窗口自己的存储里（src/store/ui.ts）。 */
export interface Settings {
  settings(): Promise<SettingsView>;
  changeSettings(patch: SettingsPatch): Promise<SettingsView>;
  releaseHeld(): Promise<{ removed: number }>;
}

/** 内核和样例库都实现的全部读写。 */
export type Source = Library & Platforms & Transforms & Settings;

/** 内核今天回的一条：还没有 `pending`，每面还没有 `key`（feature 003 的 WP7、WP8 还没接上）。 */
type KeylessFacet<F> = F extends Facet ? Omit<F, "key"> & { key?: string } : never;
type WireItem = Omit<Item, "pending" | "facets"> & { pending?: Pending[]; facets: KeylessFacet<Facet>[] };

/** 在边界上补齐成 endpoints.ts 的形状：没有 `key` 的按层起一个，没有 `pending` 的当没有在等的。 */
function itemOf(wire: WireItem): Item {
  return {
    ...wire,
    pending: wire.pending ?? [],
    facets: wire.facets.map((f) => ({ ...f, key: f.key ?? (f.layer === "video" ? `video:${f.snapshotId}` : f.layer) })),
  };
}

/** 内核今天回的插件：还没有 `methods`。 */
type WirePlugin = Omit<FetchPlugin, "methods"> & { methods?: FetchPlugin["methods"] };

/** 没有 `methods` 的插件，照它自己的档位和登录要求补成一种取法，开着。 */
function pluginOf(wire: WirePlugin): FetchPlugin {
  if (wire.methods) return { ...wire, methods: wire.methods };
  const kind = wire.login === "none" ? (wire.visitors.length > 0 ? "visitor" : "none") : "in-usuyo";
  return {
    ...wire,
    methods: [
      {
        id: "default",
        name: wire.name,
        access: wire.access,
        account: { kind, ...(kind === "visitor" ? { sites: wire.visitors } : kind === "in-usuyo" ? { sites: wire.sites } : {}) },
        gets: wire.canList ? { list: wire.sources.map((s) => s.title) } : {},
        built: true,
        enabled: true,
        // 停着的写在插件上，这一种不再重复。
        pauses: [],
      },
    ],
  };
}

const itemRequest = (path: string, method?: "POST") => request<WireItem>(path, method ? { method } : {}).then(itemOf);

/** 列来源要调插件，内核最多等一分钟多一点。 */
const SOURCES_TIMEOUT_MS = 75_000;

const daemon: Source = {
  health: () => request<Health>(paths.health),
  map: () => request<MapView>(paths.map),
  topics: () => request<Topic[]>(paths.topics),
  createTopic: (name) => request<Topic>(paths.topics, { method: "POST", body: { name } }),
  follow: (topicId, followed) => request<Topic>(paths.topicFollow(topicId), { method: "POST", body: { followed } }),
  item: (id) => itemRequest(paths.item(id)),
  search: (q) => request<SearchHit[]>(paths.search, { query: { q } }),
  take: (body) => request<TakeResponse>(paths.take, { method: "POST", body }),
  setNote: (id, text) => request<WireItem>(paths.itemNote(id), { method: "POST", body: { text } }).then(itemOf),
  setTopic: (id, topicId, member) =>
    request<WireItem>(paths.itemTopics(id), { method: "POST", body: { topicId, member } }).then(itemOf),
  trash: (id) => itemRequest(paths.itemTrash(id), "POST"),
  restore: (id) => itemRequest(paths.itemRestore(id), "POST"),
  restoreMedia: (id) => itemRequest(paths.itemRestoreMedia(id), "POST"),
  textOnly: (id) => itemRequest(paths.itemTextOnly(id), "POST"),
  refetch: (id) => itemRequest(paths.itemRefetch(id), "POST"),
  trashList: () => request<TrashView>(paths.trash),
  purge: (trashId) => request<void>(paths.trashPurge(trashId), { method: "POST" }),

  plugins: () => request<Omit<PluginsView, "plugins"> & { plugins: WirePlugin[] }>(paths.plugins).then((v) => ({ ...v, plugins: v.plugins.map(pluginOf) })),
  sources: (id) => request<PluginSource[]>(paths.pluginSources(id), { timeoutMs: SOURCES_TIMEOUT_MS }),
  tryNow: (id) => request<WirePlugin>(paths.pluginTryNow(id), { method: "POST" }).then(pluginOf),
  hold: (id) => request<WirePlugin>(paths.pluginHold(id), { method: "POST" }).then(pluginOf),
  switchMethod: (id, method, enabled) => request<WirePlugin>(paths.pluginMethod(id, method), { method: "PUT", body: { enabled } }).then(pluginOf),
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

  settings: () => request<SettingsView>(paths.settings),
  changeSettings: (patch) => request<SettingsView>(paths.settings, { method: "PATCH", body: patch }),
  releaseHeld: () => request<{ removed: number }>(paths.trashReleaseHeld, { method: "POST" }),

  transforms: () => request<TransformsView>(paths.transforms),
  transform: (itemId, body) => request<void>(paths.itemTransform(itemId), { method: "POST", body }).then(() => undefined),
  downloadModel: (pluginId, name) => request<void>(paths.modelDownload(pluginId, name), { method: "POST" }).then(() => undefined),
  removeModel: (pluginId, name) => request<void>(paths.modelRemove(pluginId, name), { method: "POST" }).then(() => undefined),
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
