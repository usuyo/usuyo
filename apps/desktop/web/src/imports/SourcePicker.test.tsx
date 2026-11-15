import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { FetchPlugin, ImportView, PluginSource } from "../api/endpoints";
import { method } from "../mock/platforms";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./ImportPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const BILI: FetchPlugin = {
  id: "usuyo-fetch/bilibili",
  name: "B站",
  version: "0.1.0",
  platforms: [{ id: "bilibili", name: "B站" }],
  access: "public-web",
  login: "none",
  canList: true,
  sources: [],
  sites: [],
  visitors: [],
  settings: [],
  pacing: [],
  pauses: [],
  methods: [
    method({ id: "view", name: "公开接口", access: "public-web", account: { kind: "none" }, gets: { fetch: ["meta"] } }),
    method({ id: "favorites-public", name: "公开收藏夹", access: "public-web", account: { kind: "none" }, gets: { list: [], link: "收藏夹或个人空间的链接" } }),
    method({
      id: "favorites",
      name: "我的收藏夹",
      access: "session-api",
      account: { kind: "in-usuyo", sites: ["www.bilibili.com"], where: "B站" },
      gets: { list: ["稍后再看"] },
      blockers: [{ code: "login-missing", hint: "要先在 usuyo 里登录B站" }],
    }),
    method({
      id: "userscript",
      name: "浏览器脚本",
      access: "browser-session",
      account: { kind: "own-browser" },
      gets: { delivers: "links" },
      script: { path: "/m/plugins/usuyo-fetch/bilibili/userscript/bili.user.js", present: true },
    }),
    method({ id: "archive", name: "官方归档", access: "official-export", account: { kind: "none" }, gets: { list: ["稍后再看"] }, enabled: false }),
  ],
};

const SITE = {
  site: "www.bilibili.com",
  name: "B站",
  loginUrl: "https://passport.bilibili.com/login",
  cookies: [],
  domains: [],
  login: "missing",
  plugins: [BILI.id],
  credential: "cookies",
};

const SCRIPT = `// ==UserScript==
// @description  在你自己的「收藏」页上，只看页面自己请求回来的列表。
// @match        https://www.bilibili.com/*
// ==/UserScript==
`;

const SOURCE: PluginSource = { source: "bilibili:fav:1", title: "胶片", count: 36, method: "favorites-public" };

const VIEW: ImportView = {
  id: "0193f2b0-0009-7f3a-8b4c-5d6e7f8a9b0c",
  kind: "source",
  title: "胶片",
  pluginId: BILI.id,
  method: "favorites-public",
  source: SOURCE.source,
  createdAt: "2026-11-12T08:00:00.000Z",
  state: "listing",
  pages: 0,
  preview: { new: 0, stored: 0, trashed: 0 },
  taken: 0,
  notStored: 0,
  skipped: 0,
};

/** `sources` 是每次问来源时依次回的；用完了回最后一个。 */
function serve(sources: Response[] = []) {
  const seen: { url: URL; body: unknown }[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const url = new URL(input);
      const path = url.pathname;
      if (path.endsWith("/sources") || init?.method === "POST") seen.push({ url, body: typeof init?.body === "string" ? JSON.parse(init.body) : null });
      if (path.endsWith("/sources")) return Promise.resolve(sources.length > 1 ? (sources.shift() ?? Response.json([SOURCE])) : (sources[0]?.clone() ?? Response.json([SOURCE])));
      if (path.endsWith(`/imports/${VIEW.id}`)) return Promise.resolve(Response.json(VIEW));
      if (path.endsWith("/imports")) return Promise.resolve(Response.json(init?.method === "POST" ? VIEW : []));
      if (path.endsWith("/plugins")) return Promise.resolve(Response.json({ plugins: [BILI], notLoaded: [] }));
      if (path.endsWith("/file-readers")) return Promise.resolve(Response.json([]));
      if (path.endsWith("/sites")) return Promise.resolve(Response.json([SITE]));
      if (path.endsWith("/visitors")) return Promise.resolve(Response.json([]));
      if (path.endsWith("/script")) return Promise.resolve(Response.json({ text: SCRIPT }));
      if (path.endsWith("/deliver-tokens")) return Promise.resolve(Response.json(init?.method === "POST" ? { id: "t1", name: "B站脚本", kind: "extension", createdAt: "2026-11-15T08:00:00.000Z", token: "t1.secret" } : []));
      if (path.endsWith("/events")) return new Promise(() => {});
      if (path.endsWith("/entries")) return Promise.resolve(Response.json([]));
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/import"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return seen;
}

it("groups what can be imported under each platform, gives a list method that starts from a link a link field, asks only after you hand one in, and starts with the method", async () => {
  const seen = serve();
  expect(await screen.findByRole("region", { name: "B站" })).toBeTruthy();
  // 只取单条的不摆；还差登录的那一行就地给「登录」。
  expect(screen.queryByText("公开接口")).toBeNull();
  expect(await screen.findByText(/在 usuyo 中登录B站账号/)).toBeTruthy();
  expect(screen.getByRole("button", { name: "登录" })).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "输入链接" }));
  const field = await screen.findByLabelText("收藏夹或个人空间的链接");
  expect(seen).toEqual([]);
  fireEvent.change(field, { target: { value: "https://space.bilibili.com/1/favlist?fid=2" } });
  fireEvent.click(screen.getByRole("button", { name: "查找" }));
  fireEvent.click(await screen.findByRole("button", { name: "列出" }));
  await waitFor(() => expect(seen.filter((x) => !x.url.pathname.endsWith("/deliver-tokens"))).toHaveLength(2));
  const [ask, start] = seen;
  expect(ask?.url.searchParams.get("method")).toBe("favorites-public");
  expect(ask?.url.searchParams.get("link")).toBe("https://space.bilibili.com/1/favlist?fid=2");
  expect(start?.body).toEqual({ pluginId: BILI.id, method: "favorites-public", source: SOURCE.source, title: "胶片" });
  // 开始以后那一次导入写着是哪种取法列的。
  expect(await screen.findByText(/B站 · 公开收藏夹 · /)).toBeTruthy();
});

it("sets up a browser script with one press: a token named after the platform, the script and the token", async () => {
  const seen = serve();
  // 说明里写脚本自己读的那一页。
  expect(await screen.findByText("在你的浏览器中读取已登录的「收藏」页面，把链接发送到 usuyo")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "设置浏览器脚本" }));
  expect(await screen.findByText("t1.secret")).toBeTruthy();
  expect(seen.find((x) => x.url.pathname.endsWith("/deliver-tokens"))?.body).toEqual({ name: "B站脚本", kind: "extension" });
  expect(screen.getByRole("button", { name: "拷贝脚本" })).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "完成" }));
  await waitFor(() => expect(screen.queryByText("t1.secret")).toBeNull());
});

it("shows a turned-off method with its switch instead of an action that can't run", async () => {
  serve();
  expect(await screen.findByRole("switch", { name: "官方归档：已关闭" })).toBeTruthy();
  // 关着的、还差登录的，「选择列表」都按不了。
  for (const b of screen.getAllByRole("button", { name: "选择列表" })) expect(b.hasAttribute("disabled")).toBe(true);
});

const problem = (status: number, code: string, hint?: string) =>
  Response.json({ code, detail: "link: not a link", status, ...(hint ? { hint } : {}) }, { status, headers: { "content-type": "application/problem+json" } });

it("says the daemon's sentence for a link it cannot use, hands share text over as pasted, and asks again for the same link", async () => {
  const seen = serve([problem(400, "invalid-args", "这段文字里没有链接"), problem(422, "unsupported-input", "公开收藏夹现在用不上：在按节奏等"), Response.json([SOURCE])]);
  fireEvent.click(await screen.findByRole("button", { name: "输入链接" }));
  const field = await screen.findByLabelText("收藏夹或个人空间的链接");
  fireEvent.change(field, { target: { value: "ftp://space.bilibili.com/1" } });
  fireEvent.click(screen.getByRole("button", { name: "查找" }));
  expect(await screen.findByText("这段文字里没有链接")).toBeTruthy();
  // 带链接的分享文字照原样交过去，链接由内核取出来。
  const shared = "【胶片】的收藏夹 https://space.bilibili.com/1/favlist?fid=2 快来看看";
  fireEvent.change(field, { target: { value: shared } });
  fireEvent.click(screen.getByRole("button", { name: "查找" }));
  expect(await screen.findByText("公开收藏夹现在用不上：在按节奏等")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "查找" }));
  expect(await screen.findByRole("button", { name: "列出" })).toBeTruthy();
  const asked = seen.filter((x) => x.url.pathname.endsWith("/sources"));
  expect(asked).toHaveLength(3);
  expect(asked[1]?.url.searchParams.get("link")).toBe(shared);
});

it("says its own sentence for a plugin's error that came without a hint, with the sign-in button right there", async () => {
  serve([problem(502, "login-required")]);
  fireEvent.click(await screen.findByRole("button", { name: "输入链接" }));
  fireEvent.change(await screen.findByLabelText("收藏夹或个人空间的链接"), { target: { value: "https://space.bilibili.com/1" } });
  fireEvent.click(screen.getByRole("button", { name: "查找" }));
  const said = await screen.findByText("需要先登录。登录后再试。");
  expect(said.parentElement?.querySelector("button")?.textContent).toBe("登录");
});
