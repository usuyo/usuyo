import { QueryClient } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";
import { applyEvent, isDaemonEvent } from "./events";
import { queryKeys } from "./hooks";

describe("daemon events", () => {
  it("accepts the platform events, including plugins-updated without an id", () => {
    expect(isDaemonEvent({ type: "plugins-updated" })).toBe(true);
    expect(isDaemonEvent({ type: "import-updated", id: "a" })).toBe(true);
    expect(isDaemonEvent({ type: "import-updated" })).toBe(false);
    expect(isDaemonEvent({ type: "something-else", id: "a" })).toBe(false);
  });

  it("refreshes plugins and sites, or the one import, when told", () => {
    const qc = new QueryClient();
    const spy = vi.spyOn(qc, "invalidateQueries");
    applyEvent(qc, { type: "plugins-updated" });
    applyEvent(qc, { type: "import-updated", id: "x" });
    const keys = spy.mock.calls.map((c) => c[0]?.queryKey);
    expect(keys).toEqual([queryKeys.plugins, queryKeys.sites, queryKeys.imports, queryKeys.import("x"), queryKeys.importEntries("x")]);
  });
});
