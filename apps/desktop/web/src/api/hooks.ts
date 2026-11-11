import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { Item, TakeRequest } from "./endpoints";
import { call } from "./source";

export const queryKeys = {
  health: ["health"] as const,
  map: ["map"] as const,
  topics: ["topics"] as const,
  item: (id: string) => ["items", id] as const,
  search: (q: string) => ["search", q] as const,
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
    },
  });
}
