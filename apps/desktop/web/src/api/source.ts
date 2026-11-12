import { DaemonUnreachableError, request, upload } from "./client";
import {
  paths,
  type ConfirmImportRequest,
  type CreateDeliverTokenRequest,
  type DeliverToken,
  type FetchPlugin,
  type FileReader,
  type ImportEntry,
  type ImportView,
  type NewDeliverToken,
  type PluginSource,
  type PluginsView,
  type SiteLogin,
  type StartImportRequest,
  type Health,
  type Item,
  type ModelDownload,
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
  type CiteRequest,
  type PagePatch,
  type TopicMatch,
  type TopicPage,
  type Draft,
  type DraftPatch,
  type DraftSummary,
  type NewDraftRequest,
  type FollowPatch,
  type FollowTrial,
  type FollowTrialRequest,
  type FollowView,
  type FollowsView,
  type NewFollowRequest,
  type ChangeBatch,
  type ChangesView,
  type ExportRun,
  type ExportView,
  type PhoneDryRun,
  type PhoneView,
  type SettingsPane,
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
  /** 一种取法现在能列的来源；`link` 只给 `gets.link` 有的取法。 */
  sources(pluginId: string, method: string, link?: string): Promise<PluginSource[]>;
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
  /** 能读别的工具导出的文件的方法。 */
  fileReaders(): Promise<FileReader[]>;
  /** 交一个文件给一种读文件的方法读，边交边报交了多少字节；回这一次导入。 */
  importFile(readerId: string, file: File, onSent: (bytes: number) => void, signal: AbortSignal): Promise<ImportView>;
}

/** 转换插件和模型（feature 003 §8.1）。 */
export interface Transforms {
  transforms(): Promise<TransformsView>;
  transform(itemId: string, body: TransformRequest): Promise<void>;
  downloadModel(pluginId: string, name: string): Promise<ModelDownload>;
  removeModel(pluginId: string, name: string): Promise<void>;
}

/** 设置页的读写。外观不在这里，在窗口自己的存储里（src/store/ui.ts）。 */
export interface Settings {
  settings(): Promise<SettingsView>;
  changeSettings(patch: SettingsPatch): Promise<SettingsView>;
  releaseHeld(): Promise<{ removed: number }>;
}

/** 主题页的读写（BUILD 004、005）。成员的加减走 `Library.setTopic`。 */
export interface Topics {
  topicPage(id: string): Promise<TopicPage>;
  editTopicPage(id: string, patch: PagePatch): Promise<TopicPage>;
  citeInTopic(id: string, body: CiteRequest): Promise<TopicPage>;
  setTopicMatch(id: string, match: TopicMatch): Promise<TopicPage>;
  dismissThread(id: string, threadId: string): Promise<void>;
  addAngle(id: string, question: string): Promise<TopicPage>;
  runAngle(id: string, angleId: string): Promise<TopicPage>;
}

/** 成稿的读写（BUILD 005）。 */
export interface Drafts {
  drafts(): Promise<DraftSummary[]>;
  newDraft(body: NewDraftRequest): Promise<Draft>;
  draft(id: string): Promise<Draft>;
  changeDraft(id: string, patch: DraftPatch): Promise<Draft>;
  editDraftPage(id: string, patch: PagePatch): Promise<Draft>;
  citeInDraft(id: string, body: CiteRequest): Promise<Draft>;
}

/** 跟着收的读写（BUILD 009）。 */
export interface Follows {
  follows(): Promise<FollowsView>;
  newFollow(body: NewFollowRequest): Promise<FollowView>;
  changeFollow(id: string, patch: FollowPatch): Promise<FollowView>;
  removeFollow(id: string): Promise<void>;
  trialFollow(body: FollowTrialRequest): Promise<FollowTrial>;
  setFollowPower(powerOnly: boolean): Promise<FollowsView>;
}

/** 改动记录与改动提议的读写（BUILD 006）。 */
export interface Changes {
  changes(): Promise<ChangesView>;
  acceptChange(id: string, ops?: string[]): Promise<ChangeBatch>;
  declineChange(id: string, ops?: string[]): Promise<ChangeBatch>;
  undoChange(id: string): Promise<ChangeBatch>;
  redoChange(id: string): Promise<ChangeBatch>;
}

/** 导出到 Obsidian 的读写（BUILD 007）。 */
export interface Exports {
  exportView(): Promise<ExportView>;
  setExportFolder(folder: string): Promise<ExportView>;
  exportNow(): Promise<ExportRun>;
}

/** 「手机」页的读写（docs/platforms/phone.md §4）。用到手机的取法走 `Platforms.plugins`。 */
export interface Phone {
  phone(): Promise<PhoneView>;
  installDriver(): Promise<PhoneView>;
  updateDriver(): Promise<PhoneView>;
  removeDriver(): Promise<PhoneView>;
  openSettings(pane: SettingsPane): Promise<void>;
  testClipboard(): Promise<PhoneView>;
  dryRun(): Promise<PhoneDryRun>;
  lookForAndroid(): Promise<PhoneView>;
}

/** 内核和样例库都实现的全部读写。 */
export type Source = Library & Platforms & Transforms & Settings & Topics & Drafts & Follows & Changes & Exports & Phone;

const itemRequest = (path: string, method?: "POST") => request<Item>(path, method ? { method } : {});

/** 列来源要调插件，内核最多等一分钟多一点。 */
const SOURCES_TIMEOUT_MS = 75_000;

/** 演练要截一屏、认一遍字，比平常的请求慢。 */
const DRY_RUN_TIMEOUT_MS = 30_000;

/** 找连着的安卓手机：内核读 adb 的版本（5 秒）、跑 `adb devices -l`（第一次要起 adb 的服务，最多 10 秒），再读一遍这一页。 */
const ADB_LOOK_TIMEOUT_MS = 25_000;

/** 交文件是把整个文件抄给内核，大的归档要几分钟。 */
const FILE_TIMEOUT_MS = 600_000;

const daemon: Source = {
  health: () => request<Health>(paths.health),
  map: () => request<MapView>(paths.map),
  topics: () => request<Topic[]>(paths.topics),
  createTopic: (name) => request<Topic>(paths.topics, { method: "POST", body: { name } }),
  follow: (topicId, followed) => request<Topic>(paths.topicFollow(topicId), { method: "POST", body: { followed } }),
  item: (id) => itemRequest(paths.item(id)),
  search: (q) => request<SearchHit[]>(paths.search, { query: { q } }),
  take: (body) => request<TakeResponse>(paths.take, { method: "POST", body }),
  setNote: (id, text) => request<Item>(paths.itemNote(id), { method: "POST", body: { text } }),
  setTopic: (id, topicId, member) => request<Item>(paths.itemTopics(id), { method: "POST", body: { topicId, member } }),
  trash: (id) => itemRequest(paths.itemTrash(id), "POST"),
  restore: (id) => itemRequest(paths.itemRestore(id), "POST"),
  restoreMedia: (id) => itemRequest(paths.itemRestoreMedia(id), "POST"),
  textOnly: (id) => itemRequest(paths.itemTextOnly(id), "POST"),
  refetch: (id) => itemRequest(paths.itemRefetch(id), "POST"),
  trashList: () => request<TrashView>(paths.trash),
  purge: (trashId) => request<void>(paths.trashPurge(trashId), { method: "POST" }),

  plugins: () => request<PluginsView>(paths.plugins),
  sources: (id, method, link) =>
    request<PluginSource[]>(paths.pluginSources(id), { query: { method, ...(link === undefined ? {} : { link }) }, timeoutMs: SOURCES_TIMEOUT_MS }),
  tryNow: (id) => request<FetchPlugin>(paths.pluginTryNow(id), { method: "POST" }),
  hold: (id) => request<FetchPlugin>(paths.pluginHold(id), { method: "POST" }),
  switchMethod: (id, method, enabled) => request<FetchPlugin>(paths.pluginMethod(id, method), { method: "PUT", body: { enabled } }),
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
  fileReaders: () => request<FileReader[]>(paths.fileReaders),
  importFile: (reader, file, onSent, signal) => upload<ImportView>(paths.fileImports, file, { reader, name: file.name }, onSent, signal),

  settings: () => request<SettingsView>(paths.settings),
  changeSettings: (patch) => request<SettingsView>(paths.settings, { method: "PATCH", body: patch }),
  releaseHeld: () => request<{ removed: number }>(paths.trashReleaseHeld, { method: "POST" }),

  topicPage: (id) => request<TopicPage>(paths.topic(id)),
  editTopicPage: (id, patch) => request<TopicPage>(paths.topicPage(id), { method: "PUT", body: patch }),
  citeInTopic: (id, body) => request<TopicPage>(paths.topicCite(id), { method: "POST", body }),
  setTopicMatch: (id, match) => request<TopicPage>(paths.topicMatch(id), { method: "PUT", body: match }),
  dismissThread: (id, threadId) => request<void>(paths.topicThreadDismiss(id, threadId), { method: "POST" }).then(() => undefined),
  addAngle: (id, question) => request<TopicPage>(paths.topicAngles(id), { method: "POST", body: { question } }),
  runAngle: (id, angleId) => request<TopicPage>(paths.topicAngleRun(id, angleId), { method: "POST" }),

  drafts: () => request<DraftSummary[]>(paths.drafts),
  newDraft: (body) => request<Draft>(paths.drafts, { method: "POST", body }),
  draft: (id) => request<Draft>(paths.draft(id)),
  changeDraft: (id, patch) => request<Draft>(paths.draft(id), { method: "PATCH", body: patch }),
  editDraftPage: (id, patch) => request<Draft>(paths.draftPage(id), { method: "PUT", body: patch }),
  citeInDraft: (id, body) => request<Draft>(paths.draftCite(id), { method: "POST", body }),

  follows: () => request<FollowsView>(paths.follows),
  newFollow: (body) => request<FollowView>(paths.follows, { method: "POST", body }),
  changeFollow: (id, patch) => request<FollowView>(paths.follow(id), { method: "PATCH", body: patch }),
  removeFollow: (id) => request<void>(paths.follow(id), { method: "DELETE" }).then(() => undefined),
  trialFollow: (body) => request<FollowTrial>(paths.followTrials, { method: "POST", body, timeoutMs: SOURCES_TIMEOUT_MS }),
  setFollowPower: (powerOnly) => request<FollowsView>(paths.followSettings, { method: "PATCH", body: { powerOnly } }),

  changes: () => request<ChangesView>(paths.changes),
  acceptChange: (id, ops) => request<ChangeBatch>(paths.changeAccept(id), { method: "POST", body: ops ? { ops } : {} }),
  declineChange: (id, ops) => request<ChangeBatch>(paths.changeDecline(id), { method: "POST", body: ops ? { ops } : {} }),
  undoChange: (id) => request<ChangeBatch>(paths.changeUndo(id), { method: "POST" }),
  redoChange: (id) => request<ChangeBatch>(paths.changeRedo(id), { method: "POST" }),

  exportView: () => request<ExportView>(paths.exportView),
  setExportFolder: (folder) => request<ExportView>(paths.exportTarget, { method: "PUT", body: { folder } }),
  exportNow: () => request<ExportRun>(paths.exportRuns, { method: "POST", timeoutMs: FILE_TIMEOUT_MS }),

  phone: () => request<PhoneView>(paths.phone),
  installDriver: () => request<PhoneView>(paths.phoneDriverInstall, { method: "POST" }),
  updateDriver: () => request<PhoneView>(paths.phoneDriverUpdate, { method: "POST" }),
  removeDriver: () => request<PhoneView>(paths.phoneDriverRemove, { method: "POST" }),
  openSettings: (pane) => request<void>(paths.phoneOpenSettings, { method: "POST", body: { pane } }).then(() => undefined),
  testClipboard: () => request<PhoneView>(paths.phoneClipboardTest, { method: "POST" }),
  dryRun: () => request<PhoneDryRun>(paths.phoneDryRun, { method: "POST", timeoutMs: DRY_RUN_TIMEOUT_MS }),
  lookForAndroid: () => request<PhoneView>(paths.phoneAndroidDevices, { method: "POST", timeoutMs: ADB_LOOK_TIMEOUT_MS }),

  transforms: () => request<TransformsView>(paths.transforms),
  transform: (itemId, body) => request<void>(paths.itemTransform(itemId), { method: "POST", body }).then(() => undefined),
  downloadModel: (pluginId, name) => request<ModelDownload>(paths.modelDownload(pluginId, name), { method: "POST" }),
  removeModel: (pluginId, name) => request<void>(paths.modelRemove(pluginId, name), { method: "POST" }).then(() => undefined),
};

// 样例库只在 VITE_USUYO_MOCK=1 或开发服务器里用；生产构建里这两个条件都是常量 false，样例库不进包。
const MOCK_FORCED = import.meta.env.VITE_USUYO_MOCK === "1";
const MOCK_FALLBACK = import.meta.env.MODE === "development";

// 地址栏带 ?mock=empty 时样例库从空的开始；?mock=fresh 是资料库空着、获取插件装好了（第一次打开）；
// ?mock=phone-<样子> 是「手机」页的几种样子（mock/phone.ts），不带时那一页和内核今天一样说一句。要在路由改写地址之前读。
const MOCK_PARAM = (MOCK_FORCED || MOCK_FALLBACK) && typeof location !== "undefined" ? new URLSearchParams(location.search).get("mock") : null;
const MOCK_EMPTY = MOCK_PARAM === "empty" || MOCK_PARAM === "fresh";
const MOCK_NO_PLUGINS = MOCK_PARAM === "empty";

let mockLibrary: Promise<Source> | null = null;
let usingMock = MOCK_FORCED;

/** 已经换成样例库了：事件流不用再连。 */
export function usingMockLibrary(): boolean {
  return usingMock;
}

function mock(): Promise<Source> {
  mockLibrary ??= Promise.all([import("../mock/library"), import("../mock/platforms"), import("../mock/pages"), import("../mock/follows"), import("../mock/changes"), import("../mock/exports"), import("../mock/phone")]).then(([l, p, w, f, c, e, ph]) => {
    const library = l.createMockLibrary(MOCK_EMPTY);
    return { ...library, ...p.createMockPlatforms(MOCK_NO_PLUGINS), ...w.createMockPages(library, MOCK_EMPTY), ...f.createMockFollows(library, MOCK_EMPTY), ...c.createMockChanges(MOCK_EMPTY), ...e.createMockExports(MOCK_EMPTY), ...ph.createMockPhone(MOCK_PARAM) };
  });
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
