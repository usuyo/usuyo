import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { createMemoryHistory } from "@tanstack/react-router";
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { createAppRouter } from "./router";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

it("shows one calm line when the daemon is not running", async () => {
  vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new TypeError("Failed to fetch")));
  // jsdom 没有 scrollTo，路由换页时会调它。
  vi.stubGlobal("scrollTo", vi.fn());
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );

  expect((await screen.findAllByText("usuyo 服务未运行。重新打开 usuyo 后再试。")).length).toBeGreaterThan(0);
  expect(screen.queryByText(/error/i)).toBeNull();
});

it("opens on the map even when the library is empty, with one quiet line instead of a first-run page", async () => {
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL) => {
      const path = new URL(input).pathname;
      if (path.endsWith("/map")) return Promise.resolve(Response.json({ regions: [], points: [], unreadable: [] }));
      if (path.endsWith("/topics")) return Promise.resolve(Response.json([]));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(Response.json({ code: "not-found" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  const line = await screen.findByText(/尚未添加任何内容/);
  expect(line.querySelector("a")?.getAttribute("href")).toBe("/plugins");
  expect(screen.getByRole("searchbox", { name: "搜索" })).toBeTruthy();
});
