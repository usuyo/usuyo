import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { request } from "./client";
import {
  paths,
  type Health,
  type Item,
  type ItemSummary,
  type SearchHit,
  type TakeRequest,
  type TakeResponse,
} from "./endpoints";

export const queryKeys = {
  health: ["health"] as const,
  items: ["items"] as const,
  item: (id: string) => ["items", id] as const,
  search: (q: string) => ["search", q] as const,
};

export function useHealth() {
  return useQuery({
    queryKey: queryKeys.health,
    queryFn: () => request<Health>(paths.health),
    refetchInterval: 5000,
  });
}

export function useItems() {
  return useQuery({
    queryKey: queryKeys.items,
    queryFn: () => request<ItemSummary[]>(paths.items),
  });
}

export function useItem(id: string) {
  return useQuery({
    queryKey: queryKeys.item(id),
    queryFn: () => request<Item>(paths.item(id)),
  });
}

export function useSearch(q: string) {
  return useQuery({
    queryKey: queryKeys.search(q),
    queryFn: () => request<SearchHit[]>(paths.search, { query: { q } }),
    enabled: q.trim() !== "",
  });
}

export function useTake() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: TakeRequest) =>
      request<TakeResponse>(paths.take, { method: "POST", body }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.items }),
  });
}
