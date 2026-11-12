import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { ChangesView, FollowPatch, FollowTrialRequest, NewFollowRequest, Draft, DraftPatch, NewDraftRequest, CiteRequest, PagePatch, TopicMatch, TopicPage, SettingsPatch, TransformRequest, ConfirmImportRequest, CreateDeliverTokenRequest, ImportEntry, ImportView, Item, StartImportRequest, TakeRequest } from "./endpoints";
import { ApiError, DaemonUnreachableError, isNotServed } from "./client";
import { call } from "./source";

export const queryKeys = {
  health: ["health"] as const,
  map: ["map"] as const,
  topics: ["topics"] as const,
  item: (id: string) => ["items", id] as const,
  search: (q: string) => ["search", q] as const,
  trash: ["trash"] as const,
  plugins: ["plugins"] as const,
  sites: ["sites"] as const,
  visitors: ["visitors"] as const,
  sources: (pluginId: string) => ["sources", pluginId] as const,
  imports: ["imports"] as const,
  import: (id: string) => ["imports", id] as const,
  importEntries: (id: string, standing?: ImportEntry["standing"]) =>
    standing === undefined ? (["import-entries", id] as const) : (["import-entries", id, standing] as const),
  transforms: ["transforms"] as const,
  settings: ["settings"] as const,
  deliverTokens: ["deliver-tokens"] as const,
  inbox: ["inbox"] as const,
  fileReaders: ["file-readers"] as const,
  topicPage: (id: string) => ["topic-page", id] as const,
  follows: ["follows"] as const,
  changes: ["changes"] as const,
  exportView: ["export"] as const,
  drafts: ["drafts"] as const,
  draft: (id: string) => ["drafts", id] as const,
};

export function useHealth() {
  return useQuery({
    queryKey: queryKeys.health,
    queryFn: () => call((l) => l.health()),
    refetchInterval: 5000,
  });
}

export function useMap() {
  return useQuery({ queryKey: queryKeys.map, queryFn: () => call((l) => l.map()) });
}

export function useTopics() {
  return useQuery({ queryKey: queryKeys.topics, queryFn: () => call((l) => l.topics()) });
}

export function useItem(id: string | null) {
  return useQuery({
    queryKey: queryKeys.item(id ?? ""),
    queryFn: () => call((l) => l.item(id ?? "")),
    enabled: id !== null,
    // 有一面在转时隔一会儿重拉：事件流断了（或样例库没有事件流）也看得到转好的那一面。
    refetchInterval: (q) => (q.state.data?.pending.some((p) => p.state === "now") ? 1500 : false),
    // 404 是这一条已经不在了（例如从回收站清掉了），再试也不会有。
    retry: (count, error) => !(error instanceof DaemonUnreachableError) && !isGone(error) && count < 2,
  });
}

/** 这一条已经不在资料库里了：内核回 404（不是兜底路由的那种）。 */
export function isGone(error: Error | null): boolean {
  return error instanceof ApiError && error.status === 404 && !isNotServed(error);
}

export function useSearch(q: string) {
  const trimmed = q.trim();
  return useQuery({
    queryKey: queryKeys.search(trimmed),
    queryFn: () => call((l) => l.search(trimmed)),
    enabled: trimmed !== "",
    placeholderData: (previous) => previous,
  });
}

export function useTake() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: TakeRequest) => call((l) => l.take(body)),
    // 不等地图重取完：调用处的 onSuccess 要先记下「刚收下」，新的点到了才知道要放落下动画。
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: queryKeys.map }),
  });
}

export function useFollow() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (v: { topicId: string; followed: boolean }) => call((l) => l.follow(v.topicId, v.followed)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.topics }),
  });
}

type ItemChange =
  | { kind: "note"; text: string }
  | { kind: "topic"; topicId: string; member: boolean }
  /** 按名字放进一个主题，还没有这个主题就先建。 */
  | { kind: "new-topic"; name: string }
  | { kind: "trash" }
  | { kind: "restore" }
  | { kind: "restore-media" }
  | { kind: "text-only" }
  | { kind: "refetch" };

/** 条目上的改动都回给整条最新的样子，直接放进缓存。 */
export function useItemChange(id: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (change: ItemChange): Promise<Item> =>
      call((l) => {
        switch (change.kind) {
          case "note":
            return l.setNote(id, change.text);
          case "topic":
            return l.setTopic(id, change.topicId, change.member);
          case "new-topic":
            return l.createTopic(change.name).then((t) => l.setTopic(id, t.id, true));
          case "trash":
            return l.trash(id);
          case "restore":
            return l.restore(id);
          case "restore-media":
            return l.restoreMedia(id);
          case "text-only":
            return l.textOnly(id);
          default:
            return l.refetch(id);
        }
      }),
    onSuccess: (item, change) => {
      queryClient.setQueryData(queryKeys.item(id), item);
      if (change.kind !== "note") void queryClient.invalidateQueries({ queryKey: queryKeys.map });
      if (["trash", "restore", "restore-media", "text-only"].includes(change.kind)) {
        void queryClient.invalidateQueries({ queryKey: queryKeys.trash });
      }
      if (change.kind === "new-topic") void queryClient.invalidateQueries({ queryKey: queryKeys.topics });
    },
  });
}

// —— 主题页 ——

export function useTopicPage(id: string) {
  return useQuery({ queryKey: queryKeys.topicPage(id), queryFn: () => call((s) => s.topicPage(id)), retry: retryUnlessMissing });
}

type TopicPageChange =
  | { kind: "page"; patch: PagePatch }
  | { kind: "cite"; body: CiteRequest }
  | { kind: "match"; match: TopicMatch }
  | { kind: "angle"; question: string }
  | { kind: "run"; angleId: string }
  /** 放进或拿出这个主题：走条目上的成员接口，回来再拉一次这一页。 */
  | { kind: "member"; itemId: string; member: boolean };

/** 主题页上的改动都回整页最新的样子，直接放进缓存。 */
export function useTopicPageChange(id: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (change: TopicPageChange): Promise<TopicPage> =>
      call((s) => {
        switch (change.kind) {
          case "page":
            return s.editTopicPage(id, change.patch);
          case "cite":
            return s.citeInTopic(id, change.body);
          case "match":
            return s.setTopicMatch(id, change.match);
          case "angle":
            return s.addAngle(id, change.question);
          case "run":
            return s.runAngle(id, change.angleId);
          default:
            return s.setTopic(change.itemId, id, change.member).then(() => s.topicPage(id));
        }
      }),
    onSuccess: (page, change) => {
      queryClient.setQueryData(queryKeys.topicPage(id), page);
      if (change.kind === "member") {
        void queryClient.invalidateQueries({ queryKey: queryKeys.map });
        void queryClient.invalidateQueries({ queryKey: queryKeys.item(change.itemId) });
      }
    },
  });
}

/** 不要这个线头：不留记录，从这一页上拿掉。 */
export function useDismissThread(id: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (threadId: string) => call((s) => s.dismissThread(id, threadId)),
    onSuccess: (_, threadId) =>
      queryClient.setQueryData(queryKeys.topicPage(id), (page: TopicPage | undefined) =>
        page ? { ...page, threads: page.threads.filter((t) => t.id !== threadId) } : page,
      ),
  });
}

// —— 成稿 ——

export function useDrafts() {
  return useQuery({ queryKey: queryKeys.drafts, queryFn: () => call((s) => s.drafts()), retry: retryUnlessMissing });
}

export function useDraft(id: string) {
  return useQuery({ queryKey: queryKeys.draft(id), queryFn: () => call((s) => s.draft(id)), retry: retryUnlessMissing });
}

export function useNewDraft() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: NewDraftRequest) => call((s) => s.newDraft(body)),
    onSuccess: (draft) => {
      queryClient.setQueryData(queryKeys.draft(draft.id), draft);
      void queryClient.invalidateQueries({ queryKey: queryKeys.drafts, exact: true });
    },
  });
}

type DraftChange = { kind: "meta"; patch: DraftPatch } | { kind: "page"; patch: PagePatch } | { kind: "cite"; body: CiteRequest };

/** 成稿上的改动都回整篇最新的样子。 */
export function useDraftChange(id: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (change: DraftChange): Promise<Draft> =>
      call((s) => (change.kind === "meta" ? s.changeDraft(id, change.patch) : change.kind === "page" ? s.editDraftPage(id, change.patch) : s.citeInDraft(id, change.body))),
    onSuccess: (draft) => {
      queryClient.setQueryData(queryKeys.draft(id), draft);
      void queryClient.invalidateQueries({ queryKey: queryKeys.drafts, exact: true });
    },
  });
}

// —— 跟着收 ——

export function useFollows() {
  return useQuery({ queryKey: queryKeys.follows, queryFn: () => call((s) => s.follows()), retry: retryUnlessMissing });
}

type FollowChange =
  | { kind: "new"; body: NewFollowRequest }
  | { kind: "patch"; id: string; patch: FollowPatch }
  | { kind: "remove"; id: string }
  | { kind: "power"; powerOnly: boolean };

/** 跟着收的改动；改完重拉整页。 */
export function useFollowChange() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (change: FollowChange): Promise<unknown> =>
      call<unknown>((s) => {
        switch (change.kind) {
          case "new":
            return s.newFollow(change.body);
          case "patch":
            return s.changeFollow(change.id, change.patch);
          case "remove":
            return s.removeFollow(change.id);
          default:
            return s.setFollowPower(change.powerOnly);
        }
      }),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: queryKeys.follows }),
  });
}

/** 试列一次：什么都不收，只看会收下哪些。 */
export function useFollowTrial() {
  return useMutation({ mutationFn: (body: FollowTrialRequest) => call((s) => s.trialFollow(body)) });
}

// —— 改动记录 ——

export function useChanges() {
  return useQuery({ queryKey: queryKeys.changes, queryFn: () => call((s) => s.changes()), retry: retryUnlessMissing });
}

type BatchAction = { kind: "accept" | "decline"; ops?: string[] } | { kind: "undo" | "redo" };

/** 定一批提议、撤销、重做：回来的这一批换进列表；资料库变了，地图、条目和主题页都重拉。 */
export function useChangeBatch(id: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (a: BatchAction) =>
      call((s) =>
        a.kind === "accept" ? s.acceptChange(id, a.ops) : a.kind === "decline" ? s.declineChange(id, a.ops) : a.kind === "undo" ? s.undoChange(id) : s.redoChange(id),
      ),
    onSuccess: (batch) => {
      queryClient.setQueryData(queryKeys.changes, (v: ChangesView | undefined) => (v ? { batches: v.batches.map((b) => (b.id === batch.id ? batch : b)) } : v));
      void queryClient.invalidateQueries({ queryKey: queryKeys.map });
      void queryClient.invalidateQueries({ queryKey: ["items"] });
      void queryClient.invalidateQueries({ queryKey: ["topic-page"] });
    },
  });
}

// —— 导出 ——

export function useExportView() {
  return useQuery({ queryKey: queryKeys.exportView, queryFn: () => call((s) => s.exportView()), retry: retryUnlessMissing });
}

export function useExportFolder() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (folder: string) => call((s) => s.setExportFolder(folder)),
    onSuccess: (view) => queryClient.setQueryData(queryKeys.exportView, view),
  });
}

/** 现在导出：读回的改动进了改动记录，两边都改过的那段进了回收站，这几处都重拉。 */
export function useExportNow() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => call((s) => s.exportNow()),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.exportView });
      void queryClient.invalidateQueries({ queryKey: queryKeys.changes });
      void queryClient.invalidateQueries({ queryKey: queryKeys.trash });
    },
  });
}

// —— 回收站 ——

export function useTrash() {
  return useQuery({ queryKey: queryKeys.trash, queryFn: () => call((l) => l.trashList()) });
}

/** 立即清除一项。只留文字移出来的媒体清掉时条目还在，整条清掉时条目没了：两种都重拉回收站和地图。 */
export function usePurge() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (trashId: string) => call((l) => l.purge(trashId)),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.trash });
      void queryClient.invalidateQueries({ queryKey: queryKeys.map });
    },
  });
}

// —— 获取插件 ——

export function usePlugins() {
  return useQuery({ queryKey: queryKeys.plugins, queryFn: () => call((s) => s.plugins()) });
}

export function useSites() {
  return useQuery({ queryKey: queryKeys.sites, queryFn: () => call((s) => s.sites()) });
}

/** 调插件列来源，可能要等几秒到一分钟；不自己重试，免得多打平台。 */
export function useSources(pluginId: string | null) {
  return useQuery({
    queryKey: queryKeys.sources(pluginId ?? ""),
    queryFn: () => call((s) => s.sources(pluginId ?? "")),
    enabled: pluginId !== null,
    retry: false,
    staleTime: 5 * 60_000,
  });
}

export function useVisitors() {
  return useQuery({ queryKey: queryKeys.visitors, queryFn: () => call((s) => s.visitors()) });
}

type PluginChange = { kind: "try-now" } | { kind: "hold" };

export function usePluginChange(pluginId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (change: PluginChange) =>
      call((s) => {
        switch (change.kind) {
          case "try-now":
            return s.tryNow(pluginId);
          default:
            return s.hold(pluginId);
        }
      }),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: queryKeys.plugins }),
  });
}

/** 开或关一种取法。 */
export function useSwitchMethod(pluginId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (v: { methodId: string; enabled: boolean }) => call((s) => s.switchMethod(pluginId, v.methodId, v.enabled)),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: queryKeys.plugins }),
  });
}

export function useForgetLogin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (site: string) => call((s) => s.forgetLogin(site)),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: queryKeys.sites }),
  });
}

export function useForgetVisitor() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (site: string) => call((s) => s.forgetVisitor(site)),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: queryKeys.visitors }),
  });
}

// —— 导入 ——

/** 在列、在收的时候也隔一会儿重拉：事件流断了（或样例库没有事件流）也能看到它往前走。 */
const moving = (v: ImportView | undefined) => v !== undefined && (v.state === "listing" || v.state === "taking");

export function useImports() {
  return useQuery({
    queryKey: queryKeys.imports,
    queryFn: () => call((s) => s.imports()),
    refetchInterval: (q) => (q.state.data?.some(moving) ? 3000 : false),
  });
}

export function useImport(id: string | null) {
  return useQuery({
    queryKey: queryKeys.import(id ?? ""),
    queryFn: () => call((s) => s.import(id ?? "")),
    enabled: id !== null,
    refetchInterval: (q) => (moving(q.state.data) ? 1500 : false),
  });
}

/** 预览里一组的条目，前 `limit` 条。这次导入还在走（`live`）时也隔一会儿重拉，同上。 */
export function useImportEntries(id: string, standing: ImportEntry["standing"], limit: number, enabled: boolean, live: boolean) {
  return useQuery({
    queryKey: [...queryKeys.importEntries(id, standing), limit],
    queryFn: () => call((s) => s.importEntries(id, standing, 0, limit)),
    enabled,
    placeholderData: (previous) => previous,
    refetchInterval: live ? 2000 : false,
  });
}

/** 收下了、内容没取到的那几条和原因（原因在条目上）。只看前 `limit` 条新的里收下的。 */
export function useNotStored(id: string, taken: number, limit: number) {
  return useQuery({
    queryKey: [...queryKeys.importEntries(id), "not-stored", taken],
    queryFn: () =>
      call(async (s) => {
        const entries = await s.importEntries(id, "new", 0, limit);
        const ids = entries.filter((e) => e.taken && e.itemId !== undefined).map((e) => e.itemId ?? "");
        const out: Item[] = [];
        // 一次问几条，不一下子把内核塞满。
        for (let i = 0; i < ids.length; i += 6) {
          const items = await Promise.all(ids.slice(i, i + 6).map((itemId) => s.item(itemId)));
          out.push(...items.filter((it) => it.missing !== undefined || it.stored === "link"));
        }
        return out;
      }),
  });
}

type ImportChange = { kind: "confirm"; body: ConfirmImportRequest } | { kind: "hold" } | { kind: "resume" } | { kind: "delete" };

/** 能读别的工具导出的文件的导入插件。 */
export function useFileReaders() {
  return useQuery({ queryKey: queryKeys.fileReaders, queryFn: () => call((s) => s.fileReaders()), retry: retryUnlessMissing });
}

/** 交一个文件给导入插件读：回来的导入和别的导入一样，看预览、确认。 */
export function useImportFile() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ reader, file }: { reader: string; file: File }) => call((s) => s.importFile(reader, file)),
    onSuccess: (view) => {
      queryClient.setQueryData(queryKeys.import(view.id), view);
      void queryClient.invalidateQueries({ queryKey: queryKeys.imports });
    },
  });
}

export function useStartImport() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: StartImportRequest) => call((s) => s.startImport(body)),
    onSuccess: (view) => {
      queryClient.setQueryData(queryKeys.import(view.id), view);
      void queryClient.invalidateQueries({ queryKey: queryKeys.imports });
    },
  });
}

export function useImportChange(id: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (change: ImportChange): Promise<ImportView | undefined> =>
      call((s) => {
        switch (change.kind) {
          case "confirm":
            return s.confirmImport(id, change.body);
          case "hold":
            return s.holdImport(id);
          case "resume":
            return s.resumeImport(id);
          default:
            return s.deleteImport(id).then(() => undefined);
        }
      }),
    onSuccess: (view) => {
      if (view) queryClient.setQueryData(queryKeys.import(id), view);
      else queryClient.removeQueries({ queryKey: queryKeys.import(id) });
      void queryClient.invalidateQueries({ queryKey: queryKeys.imports });
    },
  });
}

// —— 投递令牌 ——

export function useDeliverTokens() {
  return useQuery({ queryKey: queryKeys.deliverTokens, queryFn: () => call((s) => s.deliverTokens()) });
}

export function useInbox() {
  return useQuery({ queryKey: queryKeys.inbox, queryFn: () => call((s) => s.inbox()), staleTime: Infinity });
}

/** 发一个投递令牌。回的令牌只在这一次出现：调用处只放在组件自己的状态里，不进缓存。 */
export function useCreateDeliverToken() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateDeliverTokenRequest) => call((s) => s.createDeliverToken(body)),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: queryKeys.deliverTokens }),
  });
}

export function useRevokeDeliverToken() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => call((s) => s.revokeDeliverToken(id)),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: queryKeys.deliverTokens }),
  });
}

// —— 转换插件和模型 ——

/** 内核没有这条接口（`isNotServed`）时不重试：页面照它说一句。 */
const retryUnlessMissing = (count: number, error: Error) => !(error instanceof DaemonUnreachableError) && !isNotServed(error) && count < 2;

export function useTransforms() {
  return useQuery({
    queryKey: queryKeys.transforms,
    queryFn: () => call((s) => s.transforms()),
    retry: retryUnlessMissing,
    // 有模型在下载时隔一会儿重拉，下好了按钮跟着变。
    refetchInterval: (q) => (q.state.data?.plugins.some((p) => p.models.some((m) => m.state === "downloading")) ? 1500 : false),
  });
}

/** 再转一次、换一个转，或把还没转的挪到最前。转出来以后 `item-updated` 让这一条自己刷新；样例库没有事件流，这里也重拉一次。 */
export function useTransform(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: TransformRequest) => call((s) => s.transform(itemId, body)),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: queryKeys.item(itemId) }),
  });
}

/** 下载或删掉一个模型。等着它的条目跟着变，所以条目也重拉。 */
export function useModelChange() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (v: { kind: "download" | "remove"; pluginId: string; name: string }) =>
      call(async (s) => {
        if (v.kind === "download") await s.downloadModel(v.pluginId, v.name);
        else await s.removeModel(v.pluginId, v.name);
      }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.transforms });
      void queryClient.invalidateQueries({ queryKey: ["items"] });
    },
  });
}

// —— 设置 ——

export function useSettings() {
  return useQuery({ queryKey: queryKeys.settings, queryFn: () => call((s) => s.settings()), retry: retryUnlessMissing });
}

/** 改几项设置。回收站的天数变了，回收站页和条目页上的「几天内能恢复」也跟着变。 */
export function useChangeSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (patch: SettingsPatch) => call((s) => s.changeSettings(patch)),
    onSuccess: (view, patch) => {
      queryClient.setQueryData(queryKeys.settings, view);
      if ("trashDays" in patch) void queryClient.invalidateQueries({ queryKey: queryKeys.trash });
      if (patch.transforms) void queryClient.invalidateQueries({ queryKey: queryKeys.transforms });
    },
  });
}

/** 时钟跳了以后拦下的那几项，照时间移除。 */
export function useReleaseHeld() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => call((s) => s.releaseHeld()),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.settings });
      void queryClient.invalidateQueries({ queryKey: queryKeys.trash });
      void queryClient.invalidateQueries({ queryKey: queryKeys.map });
    },
  });
}
