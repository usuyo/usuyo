import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { FetchMethod, FetchPlugin, PhoneDryRun, PhoneView, PluginsView } from "../api/endpoints";
import { createMockPhone, type PhoneMock } from "../mock/phone";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./PhonePage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const sample = (kind: PhoneMock) => createMockPhone(kind).phone();

const method = (m: Partial<FetchMethod> & Pick<FetchMethod, "id" | "name">): FetchMethod => ({
  access: "vision-ui",
  account: { kind: "own-phone", where: "小红书" },
  gets: { list: ["收藏"], delivers: "links" },
  built: true,
  enabled: true,
  pauses: [],
  ...m,
});

const xhs: FetchPlugin = {
  id: "usuyo-fetch/xiaohongshu",
  name: "小红书",
  version: "0.1.0",
  platforms: [{ id: "xiaohongshu", name: "小红书" }],
  access: "public-web",
  login: "none",
  canList: false,
  sources: [],
  sites: [],
  visitors: [],
  settings: [],
  pacing: [],
  pauses: [],
  methods: [
    method({ id: "note-page", name: "笔记页", access: "public-web", account: { kind: "none" } }),
    method({ id: "iphone", name: "iPhone", riskNote: { text: "镜像没有公开的识别接口。" } }),
    method({ id: "android", name: "安卓", built: false }),
  ],
};

/** `phone` 是 null 时内核没有这几条（兜底路由的 404）。记下发出去的动作和它们带的身子。 */
const SEEN: PhoneDryRun = { read: 31, match: { flow: "小红书 · 收藏", step: 4, expect: { matched: 2, total: 2 }, targets: [{ label: "分享", found: true }, { label: "复制链接", found: false }] } };

function serve(phone: PhoneView | null, plugins: PluginsView = { plugins: [xhs], notLoaded: [] }, dryRun: PhoneDryRun = SEEN) {
  const sent: string[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      if (init?.method === "POST") sent.push(`POST ${path}${typeof init.body === "string" ? ` ${init.body}` : ""}`);
      if (path.endsWith("/plugins")) return Promise.resolve(Response.json(plugins));
      if (path.endsWith("/events")) return new Promise(() => {});
      if (phone && path.endsWith("/phone/ios/dry-run")) {
        return Promise.resolve(Response.json(dryRun));
      }
      if (phone && path.endsWith("/phone/open-settings")) return Promise.resolve(new Response(null, { status: 204 }));
      if (phone && path.includes("/phone")) return Promise.resolve(Response.json(phone));
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/phone"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return sent;
}

it("says one calm sentence when the daemon does not look after phones yet, and still lists the methods", async () => {
  serve(null);
  expect(await screen.findByText(/这台电脑上的 usuyo 内核还不管手机/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "装上" })).toBeNull();
  expect(screen.queryByText(/mirroir v/)).toBeNull();
  const list = await screen.findByRole("list", { name: "用到手机的取法" });
  expect(within(list).getAllByText("开着")).toHaveLength(1);
});

it("offers to install the driver when nothing is installed, saying where it comes from", async () => {
  const sent = serve(await sample("phone-none"));
  expect(await screen.findByText("还没装 mirroir。")).toBeTruthy();
  expect(screen.getByText(/取 v0\.47\.0，约 12 MB，和官方的 SHA256SUMS 对上了/)).toBeTruthy();
  expect(screen.getAllByText("装上 mirroir 以后再给。")).toHaveLength(2);
  expect(screen.getByText("没找到 adb。")).toBeTruthy();
  expect(screen.getByText("brew install android-platform-tools")).toBeTruthy();
  expect(screen.queryByRole("button", { name: "看一眼，不点" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "装上" }));
  await waitFor(() => expect(sent).toEqual(["POST /api/phone/ios/driver/install"]));
  fireEvent.click(screen.getByRole("button", { name: "打开 iPhone 镜像" }));
  await waitFor(() => expect(sent).toContain('POST /api/phone/open-settings {"pane":"iphone-mirroring"}'));
});

it("while installing says so and offers nothing to press on the driver row", async () => {
  const none = await sample("phone-none");
  serve({ ...none, ios: { ...none.ios, driver: { ...none.ios.driver, state: "installing" } } });
  expect(await screen.findByText("在装 mirroir v0.47.0。")).toBeTruthy();
  expect(screen.queryByRole("button", { name: "装上" })).toBeNull();
  expect(screen.queryByRole("button", { name: "卸掉" })).toBeNull();
});

it("on a Mac without iPhone Mirroring says what it needs instead of offering the dry run", async () => {
  const none = await sample("phone-none");
  serve({ ...none, ios: { ...none.ios, system: { macos: "14.7", mirroring: false } } });
  expect(await screen.findByText("macOS 14.7，没有 iPhone 镜像：要 macOS 15 或更新的。")).toBeTruthy();
  expect(screen.getByText("这台 Mac 用不了 iPhone 镜像。")).toBeTruthy();
  expect(screen.getByText("先要升级到 macOS 15。")).toBeTruthy();
});

it("waits for the clipboard without a button, then says what to check when nothing came", async () => {
  const ready = await sample("phone-ready");
  const at = new Date().toISOString();
  serve({ ...ready, ios: { ...ready.ios, clipboard: { state: "waiting", at, waitSec: 10 } } });
  expect(await screen.findByText("在等：去 iPhone 上随便拷贝一段字。")).toBeTruthy();
  expect(screen.queryByRole("button", { name: /试一下/ })).toBeNull();
  cleanup();
  serve({ ...ready, ios: { ...ready.ios, clipboard: { state: "unchanged", at, waitSec: 10 } } });
  expect(await screen.findByText("10 秒里这台 Mac 的剪贴板没有变。")).toBeTruthy();
  expect(screen.getByText(/都开着「接力」，登的是同一个 Apple 账户/)).toBeTruthy();
  expect(screen.getByRole("button", { name: "再试一下" })).toBeTruthy();
});

it("says plainly when a dry run recognises nothing", async () => {
  serve(await sample("phone-ready"), undefined, { read: 23 });
  fireEvent.click(await screen.findByRole("button", { name: "看一眼，不点" }));
  const result = await screen.findByRole("list", { name: "看一眼的结果" });
  expect(within(result).getByText("认不出这一屏：读到 23 段字，对不上哪个流程的哪一步。")).toBeTruthy();
  expect(within(result).getByText("什么都没点。")).toBeTruthy();
});

it("shows missing grants with a way to the right settings pane, and the update and remove actions", async () => {
  const sent = serve(await sample("phone-grants"));
  expect(await screen.findByText(/装着 mirroir v0\.46\.2，有新的 v0\.47\.0/)).toBeTruthy();
  expect(screen.getByText(/还没给。在系统设置里找到「mirroir serve」/)).toBeTruthy();
  expect(screen.getByText(/看不出来给没给。/)).toBeTruthy();
  expect(screen.getByText("iPhone 没连上：手机解锁着，或者不在旁边。")).toBeTruthy();
  expect(screen.getByText("没有连着的安卓手机。")).toBeTruthy();
  const [screenPane, accessibilityPane, more] = screen.getAllByRole("button", { name: "打开系统设置" });
  expect(more).toBeUndefined();
  if (!screenPane || !accessibilityPane) throw new Error("两项授权各有一个打开系统设置");
  fireEvent.click(screenPane);
  await waitFor(() => expect(sent).toContain('POST /api/phone/open-settings {"pane":"screen-recording"}'));
  fireEvent.click(accessibilityPane);
  await waitFor(() => expect(sent).toContain('POST /api/phone/open-settings {"pane":"accessibility"}'));
  fireEvent.click(screen.getByRole("button", { name: "更新" }));
  await waitFor(() => expect(sent).toContain("POST /api/phone/ios/driver/update"));
  fireEvent.click(screen.getByRole("button", { name: "卸掉" }));
  await waitFor(() => expect(sent).toContain("POST /api/phone/ios/driver/remove"));
});

it("when everything is ready, tests the clipboard and reports a dry run without tapping", async () => {
  const sent = serve(await sample("phone-ready"));
  expect(await screen.findByText("iPhone 连着。")).toBeTruthy();
  expect(screen.getAllByText("给了。")).toHaveLength(2);
  expect(screen.getByText(/从 iPhone 拷的到了这台 Mac/)).toBeTruthy();
  expect(screen.getByText("连好了")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "再试一下" }));
  await waitFor(() => expect(sent).toEqual(["POST /api/phone/ios/clipboard-test"]));
  fireEvent.click(screen.getByRole("button", { name: "看一眼，不点" }));
  const result = await screen.findByRole("list", { name: "看一眼的结果" });
  expect(within(result).getByText("像是「小红书 · 收藏」的第 4 步。")).toBeTruthy();
  expect(within(result).getByText("这一步该有的 2 段字都对上了。")).toBeTruthy();
  expect(within(result).getByText("要点的「复制链接」没找到。")).toBeTruthy();
  expect(within(result).getByText("什么都没点。")).toBeTruthy();
  expect(sent).toContain("POST /api/phone/ios/dry-run");
  expect(within(screen.getByRole("list", { name: "用到手机的取法" })).getByText("现在能用")).toBeTruthy();
});

it("keeps the reason when the install failed and offers to try again", async () => {
  const sent = serve(await sample("phone-failed"));
  expect(await screen.findByText(/没装上：下载下来的包和 SHA256SUMS 对不上/)).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "再装一次" }));
  await waitFor(() => expect(sent).toEqual(["POST /api/phone/ios/driver/install"]));
});

it("tells you to accept the USB debugging prompt on an Android phone, with the serial shortened", async () => {
  serve(await sample("phone-android"));
  expect(await screen.findByText("在手机上点「允许 USB 调试」")).toBeTruthy();
  expect(screen.getByText("R58N…3K2A")).toBeTruthy();
  expect(screen.getByText("连着，但不回话：拔下来再插上")).toBeTruthy();
});

it("lists the methods that use a phone, what each still needs, and links to its plugin card", async () => {
  serve(await sample("phone-grants"));
  const list = await screen.findByRole("list", { name: "用到手机的取法" });
  const links = within(list).getAllByRole("link");
  expect(links).toHaveLength(2);
  expect(links[0]?.getAttribute("href")).toBe("/plugins?platform=xiaohongshu");
  expect(within(list).getByText("差：给屏幕录制、连上 iPhone")).toBeTruthy();
  expect(within(list).getByText("插件写了，还没做好")).toBeTruthy();
  expect(within(list).getByText("记下的：镜像没有公开的识别接口。")).toBeTruthy();
  expect(within(list).queryByText("笔记页 · iPhone 镜像")).toBeNull();
});

it("has no sample phone without a phone ?mock= variant", async () => {
  await expect(createMockPhone(null).phone()).rejects.toMatchObject({ status: 404, message: "no such route" });
  await expect(createMockPhone("fresh").installDriver()).rejects.toMatchObject({ status: 404 });
  await expect(createMockPhone("phone-ready").phone()).resolves.toMatchObject({ ios: { mirroring: "connected" } });
});
