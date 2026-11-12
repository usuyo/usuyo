import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { FetchMethod, FetchPlugin, PluginsView, SiteLogin, VisitorIdentity } from "../api/endpoints";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./PluginsPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

/** 内核今天回的插件还没有 `methods`；有了就照给。 */
type WireView = Omit<PluginsView, "plugins"> & { plugins: (Omit<FetchPlugin, "methods"> & { methods?: FetchMethod[] })[] };

function serve(view: WireView, sites: SiteLogin[], visitors: VisitorIdentity[] = []) {
  const switched: unknown[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL) => {
      const path = new URL(input).pathname;
      if (path.includes("/methods/")) {
        switched.push(path);
        return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
      }
      if (path.endsWith("/plugins")) return Promise.resolve(Response.json(view));
      if (path.endsWith("/sites")) return Promise.resolve(Response.json(sites));
      if (path.endsWith("/visitors")) return Promise.resolve(Response.json(visitors));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(new Response("nope", { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/plugins"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return switched;
}

const bili: Omit<FetchPlugin, "methods"> = {
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
  pauses: [{ scope: "bilibili", byYou: false, until: "2099-01-01T15:00:00.000Z", code: "blocked", hint: "B站挡住了这次请求" }],
};

it("says how plugins get installed today when none are", async () => {
  serve({ plugins: [], notLoaded: [] }, []);
  expect(await screen.findByText("just dev-plugins")).toBeTruthy();
  expect(screen.queryByText("这一页还没做。")).toBeNull();
});

it("shows labels, the login state and a platform's pause with both actions", async () => {
  serve({ plugins: [bili], notLoaded: [] }, [
    { site: "www.bilibili.com", name: "B站", loginUrl: "https://passport.bilibili.com/login", cookies: ["SESSDATA"], domains: [], login: "missing", plugins: [bili.id] },
  ]);
  // 内核今天不给取法：照插件自己的档位补成一种。
  expect(await screen.findByText("要在 usuyo 里登录你的B站账号")).toBeTruthy();
  expect(screen.getByText("用到：你在 usuyo 里登录的B站（还没登录）")).toBeTruthy();
  expect(screen.getByText("用你的登录调平台的接口")).toBeTruthy();
  expect(screen.getByText(/B站挡住了这次请求 · .* 自己再试/)).toBeTruthy();
  expect(screen.getByRole("button", { name: "现在再试" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "先别试" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "登录" })).toBeTruthy();
});

it("shows a visitor identity from the daemon's list, apart from logins", async () => {
  const web: Omit<FetchPlugin, "methods"> = {
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
  };
  serve({ plugins: [web], notLoaded: [] }, [], [
    { site: "www.example.com", name: "例子", url: "https://www.example.com/", cookies: [], domains: [], state: "present", plugins: [web.id] },
  ]);
  expect(await screen.findByText("有一份访客身份")).toBeTruthy();
  expect(screen.getByRole("button", { name: "换一份" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "去掉" })).toBeTruthy();
  expect(screen.getByText(/开一次例子首页/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "登录" })).toBeNull();
});

it("lists a plugin's methods as switches, marks the unbuilt ones, and says so when the daemon cannot switch them yet", async () => {
  const m = (over: Partial<FetchMethod> & Pick<FetchMethod, "id" | "name">): FetchMethod => ({
    access: "public-web",
    account: { kind: "none" },
    gets: { fetch: ["正文"] },
    built: true,
    enabled: true,
    pauses: [],
    ...over,
  });
  const switched = serve(
    {
      plugins: [
        {
          ...bili,
          pauses: [],
          methods: [
            m({ id: "public", name: "公开接口" }),
            m({ id: "lists", name: "收藏夹", access: "session-api", account: { kind: "in-usuyo", sites: ["www.bilibili.com"] }, enabled: false, riskNote: { text: "只列，不改。" } }),
            m({ id: "hook", name: "客户端注入", access: "client-hook", account: { kind: "own-phone", where: "B站" }, built: false, enabled: false }),
          ],
        },
      ],
      notLoaded: [],
    },
    [{ site: "www.bilibili.com", name: "B站", cookies: [], domains: [], login: "present", plugins: [bili.id] }],
  );
  const methods = await screen.findByRole("list", { name: "B站的取法" });
  expect(within(methods).getByRole("switch", { name: "公开接口：开着" })).toBeTruthy();
  expect(within(methods).getByText("记下的：只列，不改。")).toBeTruthy();
  expect(screen.getByText("还没做好")).toBeTruthy();
  expect(screen.queryByRole("switch", { name: /客户端注入/ })).toBeNull();

  fireEvent.click(within(methods).getByRole("switch", { name: "收藏夹：关着" }));
  expect(await screen.findByText(/内核还不能分开开关取法/)).toBeTruthy();
  expect(switched).toEqual(["/api/plugins/usuyo-fetch/bilibili/methods/lists"]);
});
