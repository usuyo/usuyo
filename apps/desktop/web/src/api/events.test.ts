import { QueryClient } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";
import { ApiError } from "./client";
import { applyEvent, isDaemonEvent } from "./events";
import { isGone, queryKeys } from "./hooks";

describe("daemon events", () => {
  it("accepts the platform events, including plugins-updated without an id", () => {
    expect(isDaemonEvent({ type: "plugins-updated" })).toBe(true);
    expect(isDaemonEvent({ type: "import-updated", id: "a" })).toBe(true);
    expect(isDaemonEvent({ type: "import-updated" })).toBe(false);
    expect(isDaemonEvent({ type: "something-else", id: "a" })).toBe(false);
  });

  it("refreshes the transform plugins and models on transforms-updated, which has no id", () => {
    expect(isDaemonEvent({ type: "transforms-updated" })).toBe(true);
    const qc = new QueryClient();
    const spy = vi.spyOn(qc, "invalidateQueries");
    applyEvent(qc, { type: "transforms-updated" });
    expect(spy.mock.calls.map((c) => c[0]?.queryKey)).toEqual([queryKeys.transforms]);
  });

  it("refreshes plugins, sites and visitors, or the one import, when told", () => {
    const qc = new QueryClient();
    const spy = vi.spyOn(qc, "invalidateQueries");
    applyEvent(qc, { type: "plugins-updated" });
    applyEvent(qc, { type: "import-updated", id: "x" });
    const keys = spy.mock.calls.map((c) => c[0]?.queryKey);
    expect(keys).toEqual([queryKeys.plugins, queryKeys.sites, queryKeys.visitors, queryKeys.imports, queryKeys.import("x"), queryKeys.importEntries("x")]);
  });

  it("follows an item removed from the trash: the map, search results, that item and the trash page refresh", () => {
    expect(isDaemonEvent({ type: "item-removed", id: "a" })).toBe(true);
    expect(isDaemonEvent({ type: "item-removed" })).toBe(false);
    const qc = new QueryClient();
    const spy = vi.spyOn(qc, "invalidateQueries");
    applyEvent(qc, { type: "item-removed", id: "a" });
    const keys = spy.mock.calls.map((c) => c[0]?.queryKey);
    expect(keys).toEqual([queryKeys.map, ["search"], queryKeys.item("a"), queryKeys.trash]);
  });

  it("treats a 404 on an item as gone, other errors as errors", () => {
    expect(isGone(new ApiError(404, "not found"))).toBe(true);
    expect(isGone(new ApiError(500, "boom"))).toBe(false);
    // 兜底路由的 404 是内核还没有这条接口，不是这一条不在了。
    expect(isGone(new ApiError(404, "no such route", "not-found"))).toBe(false);
    expect(isGone(null)).toBe(false);
  });
});
