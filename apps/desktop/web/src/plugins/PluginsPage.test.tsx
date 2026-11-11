import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { FetchPlugin, PluginsView, SiteLogin, VisitorIdentity } from "../api/endpoints";
import { createAppRouter } from "../router";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

function serve(view: PluginsView, sites: SiteLogin[], visitors: VisitorIdentity[] = []) {
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL) => {
      const path = new URL(input).pathname;
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
}

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
  expect(await screen.findByText("要登录你的B站账号")).toBeTruthy();
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
