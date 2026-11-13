import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, onTestFinished, vi } from "vitest";
import type { FetchMethod, FetchPlugin, PhoneDryRun, PhoneView, PluginsView } from "../api/endpoints";
import { createMockPhone, type PhoneMock } from "../mock/phone";
import { method as mockMethod } from "../mock/platforms";
import { setLanguageChoice } from "../i18n";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./PhonePage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const sample = (kind: PhoneMock) => createMockPhone(kind).phone();

const method = (m: Partial<FetchMethod> & Pick<FetchMethod, "id" | "name">): FetchMethod =>
  mockMethod({ access: "vision-ui", account: { kind: "own-phone", where: "小红书" }, gets: { list: ["收藏"], delivers: "links" }, ...m });

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
    // 是哪种手机看 `drives`，不看 id。
    method({ id: "mirror", name: "iPhone", drives: "iphone", riskNote: { text: "镜像没有公开的识别接口。", source: { label: "某个 issue", url: "https://example.com/1" } } }),
    method({ id: "usb", name: "安卓", drives: "android", built: false }),
    method({ id: "pc", name: "电脑客户端", access: "client-hook", account: { kind: "own-app", where: "小红书" }, drives: "desktop-app" }),
  ],
};

/** `phone` 是 null 时内核没有这几条（兜底路由的 404）。记下发出去的动作和它们带的身子。 */
const SEEN: PhoneDryRun = { read: 31, match: { flow: "小红书 · 收藏", step: 4, expect: { matched: 2, total: 2 }, targets: [{ label: "分享", found: true }, { label: "复制链接", found: false }] } };

/** 读过几次 `GET /phone`。 */
const reads: string[] = [];

function serve(phone: PhoneView | null, plugins: PluginsView = { plugins: [xhs], notLoaded: [], failureLog: "" }, dryRun: PhoneDryRun = SEEN) {
  const sent: string[] = [];
  reads.length = 0;
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      if (init?.method === "POST") sent.push(`POST ${path}${typeof init.body === "string" ? ` ${init.body}` : ""}`);
      else if (path.endsWith("/phone")) reads.push(path);
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
  expect(await screen.findByText(/此版本的 usuyo 尚未提供手机功能/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "安装" })).toBeNull();
  expect(screen.queryByText(/mirroir v/)).toBeNull();
  const list = await screen.findByRole("list", { name: "使用手机的获取方式" });
  expect(within(list).getAllByText("插件可用")).toHaveLength(1);
  expect(within(list).getByText("尚未提供")).toBeTruthy();
});

it("offers to install the driver when nothing is installed, saying where it comes from", async () => {
  const sent = serve(await sample("phone-none"));
  expect(await screen.findByText("尚未安装 mirroir。")).toBeTruthy();
  expect(screen.getByText(/下载 v0\.47\.0（约 .+），与官方 SHA256SUMS 核对一致后才放入这台电脑上 usuyo 的文件夹。安装后，需要你在「终端」中完成让它保持运行的步骤。/)).toBeTruthy();
  expect(screen.getByText("iPhone 镜像未打开，或尚未在这台 Mac 上设置。")).toBeTruthy();
  expect(screen.getAllByText("安装 mirroir 后再允许。")).toHaveLength(2);
  expect(screen.getByText("未找到 adb。")).toBeTruthy();
  expect(screen.getByText("brew install android-platform-tools")).toBeTruthy();
  expect(screen.queryByRole("button", { name: "读取屏幕" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "安装" }));
  await waitFor(() => expect(sent).toEqual(["POST /api/phone/ios/driver/install"]));
  fireEvent.click(screen.getByRole("button", { name: "打开 iPhone 镜像" }));
  await waitFor(() => expect(sent).toContain('POST /api/phone/open-settings {"pane":"iphone-mirroring"}'));
});

it("while installing says so and offers nothing to press on the driver row", async () => {
  const none = await sample("phone-none");
  serve({ ...none, ios: { ...none.ios, driver: { ...none.ios.driver, state: "installing" } } });
  expect(await screen.findByText("正在安装 mirroir v0.47.0。")).toBeTruthy();
  expect(screen.queryByRole("button", { name: "安装" })).toBeNull();
  expect(screen.queryByRole("button", { name: "卸载" })).toBeNull();
});

it("on a Mac without iPhone Mirroring says what it needs instead of offering the dry run", async () => {
  const none = await sample("phone-none");
  serve({ ...none, ios: { ...none.ios, system: { macos: "14.7", mirroring: false } } });
  expect(await screen.findByText("macOS 14.7，不支持 iPhone 镜像。需要 macOS 15 或更高版本。")).toBeTruthy();
  expect(screen.getByText("这台 Mac 不支持 iPhone 镜像。")).toBeTruthy();
  expect(screen.getByText("需要先升级到 macOS 15。")).toBeTruthy();
});

it("waits for the clipboard without a button, then says what to check when nothing came", async () => {
  const ready = await sample("phone-ready");
  const at = new Date().toISOString();
  serve({ ...ready, ios: { ...ready.ios, clipboard: { state: "waiting", at, waitSec: 10 } } });
  expect(await screen.findByText("正在等待：在 iPhone 上拷贝任意文字。")).toBeTruthy();
  expect(screen.queryByRole("button", { name: /测试/ })).toBeNull();
  cleanup();
  serve({ ...ready, ios: { ...ready.ios, clipboard: { state: "unchanged", at, waitSec: 10 } } });
  expect(await screen.findByText("10 秒内这台 Mac 的剪贴板没有变化。")).toBeTruthy();
  expect(screen.getByText(/都打开了「接力」，登录了同一个 Apple 账户/)).toBeTruthy();
  expect(screen.getByRole("button", { name: "再次测试" })).toBeTruthy();
});

it("says plainly when a dry run recognises nothing", async () => {
  serve(await sample("phone-ready"), undefined, { read: 23 });
  fireEvent.click(await screen.findByRole("button", { name: "读取屏幕" }));
  const result = await screen.findByRole("list", { name: "读取结果" });
  expect(within(result).getByText("无法识别此屏幕：读取到 23 段文字，与任何流程的步骤都不匹配。")).toBeTruthy();
  expect(within(result).getByText("没有点按任何内容。")).toBeTruthy();
});

it("shows missing grants with a way to the right settings pane, and the remove action with what it leaves", async () => {
  const grants = await sample("phone-grants");
  const sent = serve({ ...grants, ios: { ...grants.ios, grants: { ...grants.ios.grants, accessibility: "missing" } } });
  expect(await screen.findByText("已安装 mirroir v0.47.0。")).toBeTruthy();
  expect(screen.getAllByText(/未允许。在系统设置中找到「mirroir serve」/)).toHaveLength(2);
  expect(screen.getByText("iPhone 未连接：iPhone 已解锁，或不在附近。")).toBeTruthy();
  expect(screen.getByText(/卸载只删除 usuyo 放在这台电脑上的副本：正在运行的「mirroir serve」不会停止，它的登录项和证书也会保留。/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "更新" })).toBeNull();
  const [screenPane, accessibilityPane, more] = screen.getAllByRole("button", { name: "打开系统设置" });
  expect(more).toBeUndefined();
  if (!screenPane || !accessibilityPane) throw new Error("两项授权各有一个打开系统设置");
  fireEvent.click(screenPane);
  await waitFor(() => expect(sent).toContain('POST /api/phone/open-settings {"pane":"screen-recording"}'));
  fireEvent.click(accessibilityPane);
  await waitFor(() => expect(sent).toContain('POST /api/phone/open-settings {"pane":"accessibility"}'));
  fireEvent.click(screen.getByRole("button", { name: "卸载" }));
  await waitFor(() => expect(sent).toContain("POST /api/phone/ios/driver/remove"));
});

it("shows the registration step with each command to copy and what it changes on this Mac, and nothing that runs it", async () => {
  const writeText = vi.fn(() => Promise.resolve());
  const sent = serve(await sample("phone-register"));
  Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
  onTestFinished(() => {
    Reflect.deleteProperty(navigator, "clipboard");
  });
  const box = await screen.findByRole("group", { name: "需要在终端中完成的步骤" });
  expect(within(box).getByText(/还需要你在「终端」中完成一步/)).toBeTruthy();
  const commands = within(box).getAllByText(/install-agent|launchctl bootstrap/);
  expect(commands.map((c) => c.textContent)).toEqual([
    "'/Users/you/Library/Application Support/usuyo/drivers/mirroir-mcp/mirroir-mcp' install-agent",
    "launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/dev.mirroir.serve.plist",
  ]);
  expect(within(box).getByText(/登录钥匙串中一张由 mirroir 生成的签名证书（钥匙串询问时选择「始终允许」），以及一个登录时启动的后台项目/)).toBeTruthy();
  // 只有「拷贝」：没有替你跑的按钮
  expect(within(box).getAllByRole("button").map((b) => b.textContent)).toEqual(["拷贝", "拷贝"]);
  // 还没常驻：不说它常驻着
  expect(screen.queryByText(/它以「mirroir serve」在这台电脑上保持运行/)).toBeNull();
  // 系统设置里还没有它：授权两行指回这一步，不给打开系统设置
  expect(screen.getAllByText("完成「驱动」一行中需要在「终端」中进行的步骤后，系统设置中才会出现「mirroir serve」。")).toHaveLength(2);
  expect(screen.queryByRole("button", { name: "打开系统设置" })).toBeNull();
  expect(screen.getByText("需要先在「终端」中登记「mirroir serve」和连接 iPhone。")).toBeTruthy();
  fireEvent.click(within(box).getByRole("button", { name: "拷贝第 2 条命令" }));
  await waitFor(() => expect(writeText).toHaveBeenCalledWith("launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/dev.mirroir.serve.plist"));
  expect(sent).toEqual([]);
});

it("after removing, shows the step that stops the still-running service, without the keychain sentence", async () => {
  const phone = createMockPhone("phone-grants");
  const removed = await phone.removeDriver();
  serve(removed);
  expect(await screen.findByText("尚未安装 mirroir。")).toBeTruthy();
  const box = screen.getByRole("group", { name: "需要在终端中完成的步骤" });
  expect(within(box).getByText("launchctl bootout gui/$(id -u)/dev.mirroir.serve")).toBeTruthy();
  expect(within(box).queryByText(/签名证书（钥匙串询问时/)).toBeNull();
});

it("keeps the reason when an update failed and the old copy stays, still offering the update", async () => {
  const sent = serve(await sample("phone-update-failed"));
  expect(await screen.findByText(/已安装 mirroir v0\.46\.2。有新版本 v0\.47\.0 可用。/)).toBeTruthy();
  expect(screen.getByText("未能更新，当前安装的版本仍可使用：无法连接到 GitHub。检查设置中的代理。这台电脑上没有放入任何内容。")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "更新" }));
  await waitFor(() => expect(sent).toContain("POST /api/phone/ios/driver/update"));
});

it("when everything is ready, tests the clipboard and reports a dry run without tapping", async () => {
  const sent = serve(await sample("phone-ready"));
  expect(await screen.findByText("iPhone 已连接。")).toBeTruthy();
  expect(screen.getAllByText("已允许。")).toHaveLength(2);
  expect(screen.getByText(/在 iPhone 上拷贝的内容已传到这台 Mac/)).toBeTruthy();
  expect(screen.getByText("已连接")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "再次测试" }));
  await waitFor(() => expect(sent).toEqual(["POST /api/phone/ios/clipboard-test"]));
  fireEvent.click(screen.getByRole("button", { name: "读取屏幕" }));
  const result = await screen.findByRole("list", { name: "读取结果" });
  expect(within(result).getByText("看起来是「小红书 · 收藏」的第 4 步。")).toBeTruthy();
  expect(within(result).getByText("这一步应有的 2 段文字全部匹配。")).toBeTruthy();
  expect(within(result).getByText("没有找到要点按的「复制链接」。")).toBeTruthy();
  expect(within(result).getByText("没有点按任何内容。")).toBeTruthy();
  expect(sent).toContain("POST /api/phone/ios/dry-run");
  expect(within(screen.getByRole("list", { name: "使用手机的获取方式" })).getByText("可用")).toBeTruthy();
});

it("keeps the reason when the install failed and offers to try again", async () => {
  const sent = serve(await sample("phone-failed"));
  expect(await screen.findByText(/无法安装：下载的软件包与固定的 SHA256 不一致/)).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "重新安装" }));
  await waitFor(() => expect(sent).toEqual(["POST /api/phone/ios/driver/install"]));
});

it("tells you to accept the USB debugging prompt on an Android phone, showing the serial as the daemon sent it, and how to stop adb's server", async () => {
  const android = await sample("phone-android");
  // 两台缩短以后一样：都列出来
  const twin = android.android.devices[0];
  if (!twin) throw new Error("样例里有安卓手机");
  serve({ ...android, android: { ...android.android, devices: [...android.android.devices, twin] } });
  expect(await screen.findAllByText("在手机上点按「允许 USB 调试」")).toHaveLength(2);
  expect(screen.getAllByText("R58N…3K2A")).toHaveLength(2);
  expect(screen.getByText("emul…5554")).toBeTruthy();
  expect(screen.getByText("已连接但无响应。拔下后重新插入。")).toBeTruthy();
  const box = screen.getByRole("group", { name: "如何停止 adb 服务" });
  expect(within(box).getByText("'/opt/homebrew/bin/adb' kill-server")).toBeTruthy();
  expect(screen.queryByRole("button", { name: "查找已连接的手机" })).toBeNull();
});

it("when adb's server is not running, lists nothing and looks for phones only when you press it", async () => {
  const sent = serve(await sample("phone-grants"));
  expect(await screen.findByText("尚未查找已连接的 Android 手机。")).toBeTruthy();
  expect(screen.getByText(/adb 会启动自己的服务，之后在这台电脑的后台持续运行，直到你停止它/)).toBeTruthy();
  expect(screen.queryByRole("list", { name: "已连接的 Android 手机" })).toBeNull();
  expect(sent).toEqual([]);
  fireEvent.click(screen.getByRole("button", { name: "查找已连接的手机" }));
  await waitFor(() => expect(sent).toEqual(["POST /api/phone/android/devices"]));
});

it("reads the page again when the window comes back to the front", async () => {
  serve(await sample("phone-grants"));
  await screen.findByText("已安装 mirroir v0.47.0。");
  const before = reads.length;
  window.dispatchEvent(new Event("focus"));
  await waitFor(() => expect(reads.length).toBeGreaterThan(before));
});

it("lists the methods that use a phone, what each still needs, and links to its plugin card", async () => {
  serve(await sample("phone-grants"));
  const list = await screen.findByRole("list", { name: "使用手机的获取方式" });
  const links = within(list).getAllByRole("link");
  expect(links).toHaveLength(2);
  expect(links[0]?.getAttribute("href")).toBe("/plugins/xiaohongshu");
  expect(within(list).getByText("不可用。需要先允许录屏与系统录音和连接 iPhone。")).toBeTruthy();
  expect(within(list).getByText("尚未提供")).toBeTruthy();
  expect(within(list).getByText("已记录：镜像没有公开的识别接口。")).toBeTruthy();
  expect(within(list).queryByText("笔记页 · iPhone 镜像")).toBeNull();
  // 驱动电脑上客户端的留在「获取插件」页。
  expect(within(list).queryByText(/电脑客户端/)).toBeNull();
});

it("has no sample phone without a phone ?mock= variant", async () => {
  await expect(createMockPhone(null).phone()).rejects.toMatchObject({ status: 404, message: "no such route" });
  await expect(createMockPhone("fresh").installDriver()).rejects.toMatchObject({ status: 404 });
  await expect(createMockPhone("phone-ready").phone()).resolves.toMatchObject({ ios: { mirroring: "connected" } });
});

it("speaks English when English is chosen, with plurals and lists", async () => {
  setLanguageChoice("en");
  const ready = await sample("phone-register");
  serve({ ...ready, ios: { ...ready.ios, clipboard: { state: "unchanged", at: new Date().toISOString(), waitSec: 1 } } });
  expect(await screen.findByRole("heading", { name: "Phone" })).toBeTruthy();
  expect(await screen.findByText("This Mac's clipboard didn't change within 1 second.")).toBeTruthy();
  expect(screen.getByText("First, register “mirroir serve” in Terminal and connect iPhone.")).toBeTruthy();
  expect(screen.getByRole("group", { name: "Step to Complete in Terminal" })).toBeTruthy();
  expect(within(await screen.findByRole("list", { name: "Fetch Methods That Use a Phone" })).getByText("Not available yet")).toBeTruthy();
});
