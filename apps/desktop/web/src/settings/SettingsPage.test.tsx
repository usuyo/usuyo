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
  window: { systemTitleBar: false },
  logs: "/Users/someone/Library/Application Support/usuyo/logs/daemon.log",
};

function serve(settings: SettingsView) {
  const sent: { path: string; method: string; body: unknown }[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      const method = init?.method ?? "GET";
      if (method !== "GET") sent.push({ path, method, body: typeof init?.body === "string" ? JSON.parse(init.body) : null });
      if (path.endsWith("/settings")) {
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
  it("only takes the proxies the daemon takes: http, socks5 or socks5h with a host and no path", () => {
    expect(proxyProblem("http://127.0.0.1:7890")).toBeNull();
    expect(proxyProblem("socks5://localhost:1080")).toBeNull();
    expect(proxyProblem("socks5h://localhost:1080")).toBeNull();
    expect(proxyProblem("http://me:secret@127.0.0.1:7890")).toBeNull();
    expect(proxyProblem("http://me@127.0.0.1:7890")).toBeNull();
    expect(proxyProblem("https://127.0.0.1:7890")).not.toBeNull();
    expect(proxyProblem("http://127.0.0.1:7890/path")).not.toBeNull();
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

it("shows the stored proxy without its password and sends the same address back without asking for it again", async () => {
  const sent = serve({ ...VIEW, network: { mode: "custom", proxy: "http://me@127.0.0.1:7890", fakeIp: false } });
  const field = await screen.findByRole("textbox", { name: "代理地址" });
  expect(field instanceof HTMLInputElement && field.value).toBe("http://me@127.0.0.1:7890");
  expect(screen.getByRole("button", { name: "在用这个" })).toBeTruthy();
  expect(screen.getByText(/这里不显示密码/)).toBeTruthy();
  fireEvent.change(field, { target: { value: "https://127.0.0.1:7890" } });
  expect(screen.getByText(/也可以是 socks5:\/\/ 或 socks5h:\/\//)).toBeTruthy();
  fireEvent.change(field, { target: { value: "socks5h://127.0.0.1:7891" } });
  fireEvent.click(screen.getByRole("button", { name: "用这个" }));
  await waitFor(() => expect(sent).toContainEqual({ path: "/api/settings", method: "PATCH", body: { network: { mode: "custom", proxy: "socks5h://127.0.0.1:7891" } } }));
  expect(screen.getByRole("radio", { name: "深色" })).toBeTruthy();
});

it("the window row stores whether to use the system title bar and says it takes effect on the next launch", async () => {
  const sent = serve(VIEW);
  const toggle = await screen.findByRole("switch", { name: "使用系统标题栏" });
  expect(screen.getByText(/更改在下次打开 usuyo 时生效/)).toBeTruthy();
  fireEvent.click(toggle);
  await waitFor(() => expect(sent).toEqual([{ path: "/api/settings", method: "PATCH", body: { window: { systemTitleBar: true } } }]));
});
