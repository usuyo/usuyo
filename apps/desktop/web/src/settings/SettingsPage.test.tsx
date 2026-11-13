import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { SettingsView } from "../api/endpoints";
import { setLanguageChoice } from "../i18n";
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
  language: "zh-Hans",
  logs: "/Users/someone/Library/Application Support/usuyo/logs/daemon.log",
};

function serve(settings: SettingsView) {
  let language = settings.language;
  const sent: { path: string; method: string; body: unknown }[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      const method = init?.method ?? "GET";
      if (method !== "GET") sent.push({ path, method, body: typeof init?.body === "string" ? JSON.parse(init.body) : null });
      if (path.endsWith("/settings")) {
        // 语言照内核记着：改了以后再读也是新的。
        const asked: unknown = typeof init?.body === "string" ? JSON.parse(init.body) : null;
        if (method === "PATCH" && typeof asked === "object" && asked !== null && "language" in asked && (asked.language === "en" || asked.language === "zh-Hans" || asked.language === "system")) {
          language = asked.language;
        }
        return Promise.resolve(Response.json(method === "PATCH" ? { ...settings, trashDays: 90, language } : { ...settings, language }));
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
    expect(proxyProblem("")).toBe("输入代理地址。");
  });

  it("says how long the trash keeps things, and why the clearing was held", () => {
    // 废纸篓叫什么跟系统（jsdom 不是 Mac，是「回收站」）。
    expect(trashText(null)).toMatch(/会一直保留在(废纸篓|回收站)中/);
    expect(trashText(90)).toMatch(/在(废纸篓|回收站)中保留 90 天/);
    const held = heldText({ count: 3, since: "2026-08-02T12:00:00.000Z" }, 30);
    expect(held).toContain("3 个到期的项目暂未删除");
    expect(held).toMatch(/超过了(废纸篓|回收站)保留的 30 天/);
    expect(held).not.toMatch(/清空|进度/);
  });
});

it("changes the trash days at once and releases the held items after one confirming press", async () => {
  const sent = serve(VIEW);
  fireEvent.click(await screen.findByRole("radio", { name: "90 天" }));
  await waitFor(() => expect(sent).toContainEqual({ path: "/api/settings", method: "PATCH", body: { trashDays: 90 } }));

  fireEvent.click(screen.getByRole("button", { name: "按时间删除…" }));
  fireEvent.click(screen.getByRole("button", { name: "删除 3 个项目" }));
  await waitFor(() => expect(sent.some((s) => s.path === "/api/trash/release-held")).toBe(true));
  expect(screen.getByText("manifest.json 读不了")).toBeTruthy();
});

it("shows the stored proxy without its password and sends the same address back without asking for it again", async () => {
  const sent = serve({ ...VIEW, network: { mode: "custom", proxy: "http://me@127.0.0.1:7890", fakeIp: false } });
  const field = await screen.findByRole("textbox", { name: "代理地址" });
  expect(field instanceof HTMLInputElement && field.value).toBe("http://me@127.0.0.1:7890");
  expect(screen.getByRole("button", { name: "使用中" })).toBeTruthy();
  expect(screen.getByText(/此处不显示密码/)).toBeTruthy();
  fireEvent.change(field, { target: { value: "https://127.0.0.1:7890" } });
  expect(screen.getByText(/也可以使用 socks5:\/\/ 或 socks5h:\/\//)).toBeTruthy();
  fireEvent.change(field, { target: { value: "socks5h://127.0.0.1:7891" } });
  fireEvent.click(screen.getByRole("button", { name: "使用" }));
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

it("switches the whole page to English at once and stores the choice with the daemon", async () => {
  const sent = serve(VIEW);
  fireEvent.click(await screen.findByRole("radio", { name: "English" }));
  expect(await screen.findByRole("heading", { level: 1, name: "Settings" })).toBeTruthy();
  expect(screen.getByRole("switch", { name: "Use system title bar" })).toBeTruthy();
  expect(screen.getByText("Storage & Trash", { selector: "h2" })).toBeTruthy();
  await waitFor(() => expect(sent).toContainEqual({ path: "/api/settings", method: "PATCH", body: { language: "en" } }));
  // 内核回了新的设置、页面重新取过以后还是英文。
  await waitFor(() => expect(screen.getByRole("radio", { name: "System" })).toBeTruthy());
  expect(screen.getByRole("heading", { level: 1, name: "Settings" })).toBeTruthy();
  setLanguageChoice("zh-Hans");
});
