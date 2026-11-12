import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { SettingsView } from "../api/endpoints";
import { createAppRouter } from "../router";
import { heldText, proxyProblem, trashText } from "./words";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./SettingsPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const VIEW: SettingsView = {
  library: "/Users/someone/Documents/usuyo",
  trashDays: 30,
  trashHeld: { count: 3, since: "2026-08-02T12:00:00.000Z" },
  network: { mode: "system", fakeIp: false },
  transforms: { auto: true },
  logs: "/Users/someone/Library/Application Support/usuyo/logs/daemon.log",
};

function serve(settings: SettingsView | null) {
  const sent: { path: string; method: string; body: unknown }[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      const method = init?.method ?? "GET";
      if (method !== "GET") sent.push({ path, method, body: typeof init?.body === "string" ? JSON.parse(init.body) : null });
      if (path.endsWith("/settings") && settings) {
        return Promise.resolve(Response.json(method === "PATCH" ? { ...settings, trashDays: 90 } : settings));
      }
      if (path.endsWith("/release-held")) return Promise.resolve(Response.json({ removed: 3 }));
      if (path.endsWith("/map")) return Promise.resolve(Response.json({ regions: [], points: [], unreadable: [{ id: "019a", reason: "manifest.json 读不了" }] }));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/settings"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return sent;
}

describe("settings wording", () => {
  it("only takes proxies that start with http, https or socks5 and name a host", () => {
    expect(proxyProblem("http://127.0.0.1:7890")).toBeNull();
    expect(proxyProblem("socks5://localhost:1080")).toBeNull();
    expect(proxyProblem("127.0.0.1:7890")).not.toBeNull();
    expect(proxyProblem("")).toBe("写一个代理地址");
  });

  it("says how long the trash keeps things, and why the clearing was held", () => {
    expect(trashText(null)).toContain("一直留在回收站里");
    expect(trashText(90)).toContain("放 90 天");
    const held = heldText({ count: 3, since: "2026-08-02T12:00:00.000Z" }, 30);
    expect(held).toContain("到期的 3 项先留着没清");
    expect(held).toContain("超过了回收站留的 30 天");
    expect(held).not.toMatch(/清空|进度/);
  });
});

it("changes the trash days at once and releases the held items after one confirming press", async () => {
  const sent = serve(VIEW);
  fireEvent.click(await screen.findByRole("radio", { name: "90 天" }));
  await waitFor(() => expect(sent).toContainEqual({ path: "/api/settings", method: "PATCH", body: { trashDays: 90 } }));

  fireEvent.click(screen.getByRole("button", { name: "照时间清掉" }));
  fireEvent.click(screen.getByRole("button", { name: "清掉这 3 项，找不回来" }));
  await waitFor(() => expect(sent.some((s) => s.path === "/api/trash/release-held")).toBe(true));
  expect(screen.getByText("manifest.json 读不了")).toBeTruthy();
});

it("keeps appearance working and says one sentence per section when the daemon has no settings yet", async () => {
  serve(null);
  expect(await screen.findByText(/内核还不能改出口/)).toBeTruthy();
  expect(screen.getByText(/内核还没有设置接口/)).toBeTruthy();
  expect(screen.getByRole("radio", { name: "深色" })).toBeTruthy();
  // 诊断里读不了的条目来自地图，今天的内核就给。
  expect(await screen.findByText("manifest.json 读不了")).toBeTruthy();
  expect(screen.getByText(/内核还不说日志在哪/)).toBeTruthy();
});
