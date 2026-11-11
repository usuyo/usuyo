import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { ConfirmImportRequest, CreateDeliverTokenRequest, ImportEntry, ImportView, Item, StartImportRequest, TakeRequest } from "./endpoints";
import { call } from "./source";

export const queryKeys = {
  health: ["health"] as const,
  map: ["map"] as const,
  topics: ["topics"] as const,
  item: (id: string) => ["items", id] as const,
  search: (q: string) => ["search", q] as const,
  plugins: ["plugins"] as const,
  sites: ["sites"] as const,
  sources: (pluginId: string) => ["sources", pluginId] as const,
  imports: ["imports"] as const,
  import: (id: string) => ["imports", id] as const,
  importEntries: (id: string, standing?: ImportEntry["standing"]) =>
    standing === undefined ? (["import-entries", id] as const) : (["import-entries", id, standing] as const),
  deliverTokens: ["deliver-tokens"] as const,
  inbox: ["inbox"] as const,
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
  });
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
      if (change.kind === "new-topic") void queryClient.invalidateQueries({ queryKey: queryKeys.topics });
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

type PluginChange = { kind: "try-now" } | { kind: "hold" } | { kind: "forget-secret"; key: string };

export function usePluginChange(pluginId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (change: PluginChange) =>
      call((s) => {
        switch (change.kind) {
          case "try-now":
            return s.tryNow(pluginId);
          case "hold":
            return s.hold(pluginId);
          default:
            return s.forgetSecret(pluginId, change.key);
        }
      }),
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
