import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { CatalogPlugin, CatalogView } from "../api/endpoints";
import { setLanguageChoice } from "../i18n";
import { createAppRouter } from "../router";
import { filterPlugins, platformsOf, stateOf, useBrowserStore } from "./catalog";
import "./Browser";
import "./PluginDetail";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  useBrowserStore.setState({ view: "cards", query: "", platform: "", state: "all" });
  setLanguageChoice("zh-Hans");
});

const plugin = (id: string, name: string, extra: Partial<CatalogPlugin> = {}): CatalogPlugin => ({
  id,
  kind: "fetch",
  name,
  description: `${name}的说明`,
  platforms: [{ id: id.split("/")[1] ?? id, name }],
  repository: `https://github.com/${id}`,
  license: "MIT",
  methods: [{ id: "page", name: "公开页", built: true, access: "public-web", login: "none", account: "none" }],
  source: "usuyo-fetch/index",
  latest: { version: "0.2.0", publishedAt: "2026-11-12T08:00:00.000Z", asset: { name: "x.tar.gz", size: 2_000_000 } },
  update: false,
  ...extra,
});

const BILI = plugin("usuyo-fetch/bilibili", "B站", { installed: { version: "0.1.0", origin: "source", source: "usuyo-fetch/index", dir: "/m/plugins/usuyo-fetch/bilibili/0.1.0" }, update: true });
const ZHIHU = plugin("usuyo-fetch/zhihu", "知乎");
const TG = plugin("usuyo-fetch/telegram", "Telegram", { latest: undefined, blocked: { code: "no-release", hint: "这个插件还没有发布的版本。" } });

function serve(plugins: CatalogPlugin[], at = "/plugins") {
  let view: CatalogView = { target: "macos-arm64", sources: [{ address: "usuyo-fetch/index", indexUrl: "https://raw.githubusercontent.com/usuyo-fetch/index/HEAD/index.json", default: true, refreshing: false, plugins: plugins.length, refreshedAt: "2026-11-13T08:00:00.000Z" }], plugins };
  const posted: string[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL, init?: RequestInit) => {
      const path = new URL(input).pathname;
      if (path.endsWith("/install") && init?.method === "POST") {
        posted.push(path);
        view = { ...view, plugins: view.plugins.map((p) => (p.id === "usuyo-fetch/zhihu" ? { ...p, installed: { version: "0.2.0", origin: "source", source: "usuyo-fetch/index", dir: "/m/x" } } : p)) };
        return Promise.resolve(Response.json(view.plugins[1], { status: 202 }));
      }
      if (path.endsWith("/plugin-sources") && init?.method === "POST") {
        posted.push(path);
        return Promise.resolve(Response.json({ type: "urn:usuyo:problem:invalid-args", title: "Invalid", status: 400, code: "invalid-args", hint: "不是可用的订阅源地址。" }, { status: 400 }));
      }
      if (path.endsWith("/catalog")) return Promise.resolve(Response.json(view));
      if (path.endsWith("/plugins")) return Promise.resolve(Response.json({ plugins: [], notLoaded: [], failureLog: "" }));
      if (path.endsWith("/sites") || path.endsWith("/visitors") || path.endsWith("/file-readers")) return Promise.resolve(Response.json([]));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(new Response("nope", { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: [at] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return { posted, router };
}

it("filters by text, platform and state", () => {
  const all = [BILI, ZHIHU, TG];
  expect(filterPlugins(all, { query: "知", platform: "", state: "all" }).map((p) => p.name)).toEqual(["知乎"]);
  expect(filterPlugins(all, { query: "", platform: "telegram", state: "all" }).map((p) => p.name)).toEqual(["Telegram"]);
  expect(filterPlugins(all, { query: "", platform: "", state: "update" }).map((p) => p.name)).toEqual(["B站"]);
  expect(filterPlugins(all, { query: "", platform: "", state: "available" }).map((p) => p.name)).toEqual(["知乎", "Telegram"]);
  expect(stateOf(BILI)).toBe("update");
  expect(platformsOf(all).map((p) => p.name)).toEqual(["B站", "Telegram", "知乎"]);
});

it("shows cards or a list, remembers the choice, and searches", async () => {
  serve([BILI, ZHIHU, TG]);
  const card = await screen.findByRole("link", { name: "B站" });
  expect(within(card).getByText("有更新 0.2.0")).toBeTruthy();
  expect(within(screen.getByRole("link", { name: "Telegram" })).getByText("尚未发布")).toBeTruthy();
  fireEvent.click(screen.getByRole("radio", { name: "列表" }));
  expect(useBrowserStore.getState().view).toBe("list");
  expect(JSON.parse(localStorage.getItem("usuyo.plugins.browser") ?? "{}").state.view).toBe("list");
  fireEvent.change(screen.getByRole("searchbox", { name: "搜索插件" }), { target: { value: "知乎" } });
  await waitFor(() => expect(screen.queryByRole("link", { name: "B站" })).toBeNull());
  expect(screen.getByRole("link", { name: "知乎" })).toBeTruthy();
});

it("opens a plugin's detail across the page and installs it", async () => {
  const { posted } = serve([BILI, ZHIHU, TG], "/plugins/usuyo-fetch/zhihu");
  expect(await screen.findByRole("heading", { name: "知乎", level: 1 })).toBeTruthy();
  expect(screen.getByText("x.tar.gz · 2 MB")).toBeTruthy();
  expect(screen.getByText("不登录，读取公开页面 · 不需要账号")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "安装" }));
  await waitFor(() => expect(posted).toHaveLength(1));
  expect(posted[0]).toMatch(/\/catalog\/usuyo-fetch\/zhihu\/install$/);
  expect(await screen.findByText("已安装 0.2.0")).toBeTruthy();
});

it("says why a source address is not usable, in English too", async () => {
  setLanguageChoice("en");
  serve([BILI]);
  const field = await screen.findByRole("textbox", { name: "Source Address" });
  fireEvent.change(field, { target: { value: "nope" } });
  fireEvent.click(screen.getByRole("button", { name: "Add" }));
  expect(await screen.findByText("不是可用的订阅源地址。")).toBeTruthy();
  expect(screen.getByText("Update 0.2.0")).toBeTruthy();
});
