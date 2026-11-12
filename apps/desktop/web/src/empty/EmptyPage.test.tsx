import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { FetchPlugin, MapView } from "../api/endpoints";
import { method } from "../mock/platforms";
import { createAppRouter } from "../router";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const PLUGIN: FetchPlugin = {
  id: "someone/site",
  name: "某站",
  version: "0.1.0",
  platforms: [{ id: "site", name: "某站" }],
  access: "browser-session",
  login: "none",
  canList: false,
  sources: [],
  sites: [],
  visitors: [],
  settings: [],
  pacing: [],
  pauses: [],
  methods: [
    method({ id: "script", name: "浏览器脚本", access: "browser-session", account: { kind: "own-browser", where: "某站" }, gets: { list: ["收藏"], delivers: "links" } }),
  ],
};

function serve() {
  const map: MapView = { regions: [], points: [] };
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      if (path.endsWith("/map")) return Promise.resolve(Response.json(map));
      if (path.endsWith("/topics")) return Promise.resolve(Response.json([]));
      if (path.endsWith("/health")) return Promise.resolve(Response.json({ status: "ok", library: "/Users/someone/Documents/usuyo" }));
      if (path.endsWith("/items") && init?.method === "POST") {
        return Promise.resolve(Response.json({ id: "i1", seen: false, title: "一条", platform: { id: "web", name: "网页" }, takenAt: "2026-11-12T08:00:00.000Z" }));
      }
      if (path.endsWith("/plugins")) return Promise.resolve(Response.json({ plugins: [PLUGIN], notLoaded: [] }));
      if (path.endsWith("/events")) return new Promise(() => {});
      if (path.endsWith("/sites") || path.endsWith("/visitors")) return Promise.resolve(Response.json([]));
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

it("offers the ways to bring a batch in with their labels, and one calm line where the daemon lacks file readers", async () => {
  serve();
  const trigger = await screen.findByRole("button", { name: "存过的一批，也能拿回来" });
  fireEvent.click(trigger);
  expect(await screen.findByText("浏览器脚本")).toBeTruthy();
  expect(screen.getByText("列出：收藏；链接交进来")).toBeTruthy();
  expect(screen.getByText(/在登录着的网页里读/)).toBeTruthy();
  expect(await screen.findByText("这台电脑上的 usuyo 内核还读不了别的工具导出的文件。")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "收起" }));
  expect(screen.queryByText("浏览器脚本")).toBeNull();
  await waitFor(() => expect(document.activeElement).toBe(trigger));
});

it("keeps the fog after the first landing until you go to the map", async () => {
  serve();
  const field = await screen.findByPlaceholderText("粘贴一条分享链接");
  fireEvent.change(field, { target: { value: "https://example.org/a" } });
  fireEvent.keyDown(field, { key: "Enter" });
  expect(await screen.findByRole("button", { name: "去地图看看" })).toBeTruthy();
  expect(screen.getByText("收下的会落进雾里，以后从哪一片进去都行。")).toBeTruthy();
});
