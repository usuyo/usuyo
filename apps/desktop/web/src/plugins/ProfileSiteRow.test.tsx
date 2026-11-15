import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { SiteLogin } from "../api/endpoints";
import { ProfileSiteRow } from "./ProfileSiteRow";

const { clearLoginData } = vi.hoisted(() => ({ clearLoginData: vi.fn<(site: string) => Promise<void>>() }));
vi.mock("../shell", () => ({ clearLoginData }));
const forget = vi.fn<(input: URL, init?: RequestInit) => Promise<Response>>();

afterEach(() => {
  cleanup();
  clearLoginData.mockReset();
  forget.mockReset();
  vi.unstubAllGlobals();
});

const site: SiteLogin = {
  site: "www.example.test",
  name: "例子",
  loginUrl: "https://www.example.test/login",
  cookies: [],
  domains: [],
  login: "present",
  capturedAt: "2026-11-14T08:00:00.000Z",
  plugins: ["someone/example"],
  credential: "browser-profile",
  lastUse: { at: "2026-11-14T09:00:00.000Z", plugin: "someone/example", method: "in-usuyo", methodName: "在 usuyo 中登录并读取", noted: 312 },
};

function show(s: SiteLogin) {
  render(
    <QueryClientProvider client={new QueryClient()}>
      <ProfileSiteRow site={s} users={["在 usuyo 中登录并读取"]} />
    </QueryClientProvider>,
  );
}

it("says what usuyo holds, that cookies are not copied, and how it was used", () => {
  show(site);
  const text = document.body.textContent ?? "";
  expect(text).toContain("usuyo 在这台电脑上为例子保存了一份单独的网站数据");
  expect(text).toContain("cookie 不会拷贝到钥匙串或其他地方");
  expect(text).toContain("与例子访客身份和你平时使用的浏览器分开");
  expect(text).toContain("登录后的页面中会运行插件附带的脚本");
  expect(text).toContain("在 usuyo 中登录并读取，找到 312 条链接");
  expect(text).toContain("使用它的获取方式：在 usuyo 中登录并读取");
  expect(screen.getByRole("button", { name: "退出登录" })).toBeTruthy();
  cleanup();

  show({ ...site, login: "missing", capturedAt: undefined, lastUse: undefined });
  expect(document.body.textContent).toContain("尚未使用");
  expect(screen.queryByRole("button", { name: "退出登录" })).toBeNull();
});

it("deletes the website data before the daemon forgets, and keeps both when deleting fails", async () => {
  const order: string[] = [];
  clearLoginData.mockImplementation(() => {
    order.push("clear");
    return Promise.resolve();
  });
  forget.mockImplementation(() => {
    order.push("forget");
    return Promise.resolve(new Response(null, { status: 204 }));
  });
  vi.stubGlobal("fetch", forget);
  show(site);
  fireEvent.click(screen.getByRole("button", { name: "退出登录" }));
  await waitFor(() => expect(order).toEqual(["clear", "forget"]));
  expect(clearLoginData).toHaveBeenCalledWith("www.example.test");
  expect(forget.mock.calls[0]?.[1]?.method).toBe("DELETE");
  cleanup();

  order.length = 0;
  clearLoginData.mockRejectedValue("没能删掉");
  show(site);
  fireEvent.click(screen.getByRole("button", { name: "退出登录" }));
  expect(await screen.findByText("没能删掉")).toBeTruthy();
  expect(order).toEqual([]);
});
