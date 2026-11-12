import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { FetchMethod, FetchPlugin, PluginsView, SiteLogin, VisitorIdentity } from "../api/endpoints";
import { method } from "../mock/platforms";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./PluginsPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

function serve(view: PluginsView, sites: SiteLogin[], visitors: VisitorIdentity[] = [], at = "/plugins") {
  const switched: { path: string; body: unknown }[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL, init?: RequestInit) => {
      const path = new URL(input).pathname;
      if (path.includes("/methods/")) {
        const body: unknown = typeof init?.body === "string" ? JSON.parse(init.body) : null;
        switched.push({ path, body });
        const plugin = view.plugins[0];
        return Promise.resolve(Response.json(plugin));
      }
      if (path.endsWith("/plugins")) return Promise.resolve(Response.json(view));
      if (path.endsWith("/sites")) return Promise.resolve(Response.json(sites));
      if (path.endsWith("/visitors")) return Promise.resolve(Response.json(visitors));
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
  return switched;
}

const BLOCKED = { method: "favorites", scope: "bilibili", byYou: false, until: "2099-01-01T15:00:00.000Z", code: "blocked", hint: "B站挡住了这次请求" };

const favorites: FetchMethod = method({
  id: "favorites",
  name: "我的收藏夹",
  access: "session-api",
  login: "required",
  account: { kind: "in-usuyo", sites: ["www.bilibili.com"], where: "B站" },
  gets: { list: ["稍后再看"] },
  blockers: [{ code: "login-missing", hint: "要先在 usuyo 里登录B站" }],
  pauses: [BLOCKED],
});

const bili: FetchPlugin = {
  id: "usuyo-fetch/bilibili",
  name: "B站",
  version: "0.1.0",
  platforms: [{ id: "bilibili", name: "B站" }],
  access: "session-api",
  login: "required",
  canList: true,
  sources: [],
  sites: ["www.bilibili.com"],
  visitors: [],
  settings: [],
  pacing: [{ platform: "bilibili", batch: true, intervalSec: 5, daily: 500 }],
  pauses: [BLOCKED],
  methods: [favorites],
};

it("says how plugins get installed today when none are", async () => {
  serve({ plugins: [], notLoaded: [] }, []);
  expect(await screen.findByText("just dev-plugins")).toBeTruthy();
  expect(screen.queryByText("这一页还没做。")).toBeNull();
});

it("shows a method's labels, the daemon's sentence for what it is missing, its pause, and the login row", async () => {
  serve({ plugins: [bili], notLoaded: [] }, [
    { site: "www.bilibili.com", name: "B站", loginUrl: "https://passport.bilibili.com/login", cookies: ["SESSDATA"], domains: [], login: "missing", plugins: [bili.id] },
  ]);
  expect(await screen.findByText("要在 usuyo 里登录你的B站账号")).toBeTruthy();
  expect(screen.getByText("要先在 usuyo 里登录B站")).toBeTruthy();
  expect(screen.getByText("用你的登录调平台的接口")).toBeTruthy();
  expect(screen.getByText(/B站挡住了这次请求 · .* 自己再试/)).toBeTruthy();
  expect(screen.getByRole("button", { name: "现在再试" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "先别试" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "登录" })).toBeTruthy();
});

it("shows a visitor identity from the daemon's list, apart from logins", async () => {
  const web: FetchPlugin = {
    ...bili,
    id: "someone/example",
    name: "例子",
    platforms: [{ id: "example", name: "例子" }],
    access: "public-web",
    login: "none",
    canList: false,
    sites: [],
    visitors: ["www.example.com"],
    pacing: [],
    pauses: [],
    methods: [method({ id: "page", name: "页面", access: "public-web", account: { kind: "visitor", sites: ["www.example.com"], where: "例子" }, gets: { fetch: ["text"] } })],
  };
  serve({ plugins: [web], notLoaded: [] }, [], [
    { site: "www.example.com", name: "例子", url: "https://www.example.com/", cookies: [], domains: [], state: "present", plugins: [web.id] },
  ]);
  expect(await screen.findByText("有一份访客身份")).toBeTruthy();
  expect(screen.getByRole("button", { name: "换一份" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "去掉" })).toBeTruthy();
  expect(screen.getByText(/开一次例子首页/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "登录" })).toBeNull();
  expect(screen.getByText("现在能用")).toBeTruthy();
});

it("lists every method, built or not, with a switch only on the built ones, the last call, and sends the switch", async () => {
  const switched = serve(
    {
      plugins: [
        {
          ...bili,
          pauses: [],
          methods: [
            method({ id: "public", name: "公开接口", access: "public-web", account: { kind: "none" }, gets: { fetch: ["meta", "media:video"] }, lastCall: { at: "2026-11-12T08:00:00.000Z", ok: true } }),
            method({
              id: "lists",
              name: "收藏夹",
              access: "session-api",
              account: { kind: "in-usuyo", sites: ["www.bilibili.com"], where: "B站" },
              gets: { list: ["稍后再看"] },
              enabled: false,
              lastCall: { at: "2026-11-12T08:00:00.000Z", ok: false, code: "not-found", hint: "这个收藏夹不公开" },
              riskNote: { text: "只列，不改。", source: { label: "某个 issue", url: "https://example.com/1" } },
            }),
            method({ id: "hook", name: "客户端注入", access: "client-hook", account: { kind: "own-app", where: "B站" }, gets: { fetch: ["media:video"] }, built: false, enabled: false }),
          ],
        },
      ],
      notLoaded: [],
    },
    [{ site: "www.bilibili.com", name: "B站", cookies: [], domains: [], login: "present", plugins: [bili.id] }],
  );
  const methods = await screen.findByRole("list", { name: "B站的取法" });
  expect(within(methods).getByRole("switch", { name: "公开接口：开着" })).toBeTruthy();
  expect(within(methods).getByText("取一条：标题和作者、视频")).toBeTruthy();
  expect(within(methods).getByText(/上次 .* 用过，没出错/)).toBeTruthy();
  expect(within(methods).getByText("你关掉了这种方法")).toBeTruthy();
  expect(within(methods).getByText(/上次 .* 没成：这个收藏夹不公开/)).toBeTruthy();
  expect(within(methods).getByText("记下的：只列，不改。")).toBeTruthy();
  expect(within(methods).getByText("还没做好")).toBeTruthy();
  expect(within(methods).getByText("账号留在这台电脑上的B站客户端里")).toBeTruthy();
  expect(within(methods).queryByRole("switch", { name: /客户端注入/ })).toBeNull();

  fireEvent.click(within(methods).getByRole("switch", { name: "收藏夹：关着" }));
  await waitFor(() => expect(switched).toEqual([{ path: "/api/plugins/usuyo-fetch/bilibili/methods/lists", body: { enabled: true } }]));
});

it("shows only what is usable now, or only what is not, across all platforms", async () => {
  const other: FetchPlugin = {
    ...bili,
    id: "someone/example",
    name: "例子",
    platforms: [{ id: "example", name: "例子" }],
    sites: [],
    pauses: [],
    methods: [method({ id: "page", name: "页面", access: "public-web", account: { kind: "none" }, gets: { fetch: ["text"] } })],
  };
  serve({ plugins: [bili, other], notLoaded: [] }, []);
  await screen.findByRole("region", { name: "B站" });
  fireEvent.click(screen.getByRole("radio", { name: "现在能用的" }));
  expect(await screen.findByRole("region", { name: "例子" })).toBeTruthy();
  expect(screen.queryByRole("region", { name: "B站" })).toBeNull();
  fireEvent.click(screen.getByRole("radio", { name: "还用不上的" }));
  expect(await screen.findByRole("region", { name: "B站" })).toBeTruthy();
  expect(screen.queryByRole("region", { name: "例子" })).toBeNull();
});

it("opens the platform named in the address, and a method on a phone points to the phone page", async () => {
  const phone = (id: string, name: string): FetchPlugin => ({
    ...bili,
    id: `usuyo-fetch/${id}`,
    name,
    platforms: [{ id, name }],
    sites: [],
    pacing: [],
    pauses: [],
    methods: [method({ id: "iphone", name: "iPhone", access: "vision-ui", account: { kind: "own-phone", where: name }, gets: { list: ["收藏"], delivers: "links" } })],
  });
  serve({ plugins: [phone("bilibili", "B站"), phone("xiaohongshu", "小红书")], notLoaded: [] }, [], [], "/plugins?platform=xiaohongshu");
  const section = await screen.findByRole("region", { name: "小红书" });
  expect(within(section).getByRole("link", { name: "手机" }).getAttribute("href")).toBe("/phone");
});
