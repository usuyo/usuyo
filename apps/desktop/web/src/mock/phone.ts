// 样例：「手机」页。地址栏带 ?mock=phone-<样子> 才有；不带时和内核今天一样，这几条接口不在。
import { ApiError } from "../api/client";
import type { AndroidDevice, PhoneDryRun, PhoneStep, PhoneView } from "../api/endpoints";
import type { Phone } from "../api/source";
import { said } from "./said";

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/** 几种样子：什么都没装；装了、授权没给；还没登记「mirroir serve」；换新没成；都好了；没装上；安卓手机在等你点允许。 */
export const PHONE_MOCKS = ["phone-none", "phone-grants", "phone-register", "phone-update-failed", "phone-ready", "phone-failed", "phone-android"] as const;
export type PhoneMock = (typeof PHONE_MOCKS)[number];

const RELEASE = { version: "0.47.0", size: 2_518_002, source: "https://github.com/jfarcand/mirroir-mcp/releases/tag/v0.47.0" };
const TO = "mirroir serve";
const BINARY = "'/Users/you/Library/Application Support/usuyo/drivers/mirroir-mcp/mirroir-mcp'";
const ADB = "/opt/homebrew/bin/adb";

// 和内核 `driver.rs`、`adb.rs` 里的几步同样的话；按当前语言给，所以每次现做。
const REGISTER = (): PhoneStep => ({
  text: said(
    "还需要你在「终端」中完成一步：让 mirroir 以「mirroir serve」保持运行。它会在你的登录钥匙串中添加一张仅用于自身签名的证书（钥匙串询问 codesign 能否使用时，选择「始终允许」），并登记为登录时启动的后台项目。这两项会更改这台 Mac 的设置，因此 usuyo 不代你完成。",
    "One step is left for you to do in Terminal: keep mirroir running as “mirroir serve”. It adds a certificate used only for its own signing to your login keychain (choose “Always Allow” when Keychain asks whether codesign can use it) and registers a background item that starts at login. Both change this Mac's settings, so usuyo doesn't do them for you.",
  ),
  commands: [`${BINARY} install-agent`, "launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/dev.mirroir.serve.plist"],
});
const STOP = (): PhoneStep => ({
  text: said(
    "「mirroir serve」仍在运行，它使用自己的副本。在「终端」中停止它；之后可以在访达中删除 ~/Library/LaunchAgents/dev.mirroir.serve.plist 和 ~/.mirroir-mcp/mirroir serve.app，并在「钥匙串访问」中删除证书「mirroir serve local signing」。系统设置中授予它的权限由系统保留。",
    "“mirroir serve” is still running from its own copy. Stop it in Terminal. Afterward you can delete ~/Library/LaunchAgents/dev.mirroir.serve.plist and ~/.mirroir-mcp/mirroir serve.app in Finder, and the certificate “mirroir serve local signing” in Keychain Access. The permissions granted in System Settings are kept by the system.",
  ),
  commands: ["launchctl bootout gui/$(id -u)/dev.mirroir.serve"],
});
const ADB_STOP = (): PhoneStep => ({
  text: said(
    "adb 服务在这台电脑的后台运行，会一直保留，usuyo 不会停止它。要停止它，在「终端」中运行：",
    "The adb service runs in the background on this computer and stays running; usuyo doesn't stop it. To stop it, run this in Terminal:",
  ),
  commands: [`'${ADB}' kill-server`],
});

/** 内核报来的序列号已经缩短过。 */
const FOUND: AndroidDevice[] = [
  { serial: "R58N…3K2A", state: "unauthorized" },
  { serial: "2B1F…9A4D", model: "Pixel 8", state: "device" },
];

function start(kind: PhoneMock): PhoneView {
  const system = { macos: "15.6.1", mirroring: true };
  const stopped = { path: ADB, version: "36.0.0", running: false };
  const running = { path: ADB, version: "36.0.0", running: true };
  switch (kind) {
    case "phone-none":
      return {
        ios: { system, driver: { state: "absent", available: RELEASE }, grants: { to: TO, screen: "unknown", accessibility: "unknown" }, mirroring: "not-set-up" },
        android: { adb: null, devices: [] },
      };
    case "phone-grants":
      return {
        ios: { system, driver: { state: "present", version: "0.47.0" }, grants: { to: TO, screen: "missing", accessibility: "given" }, mirroring: "away" },
        android: { adb: stopped, devices: [] },
      };
    case "phone-register":
      return {
        ios: { system, driver: { state: "present", version: "0.47.0", step: REGISTER() }, grants: { to: TO, screen: "unknown", accessibility: "unknown" }, mirroring: "not-set-up" },
        android: { adb: stopped, devices: [] },
      };
    case "phone-update-failed":
      return {
        ios: {
          system,
          driver: { state: "present", version: "0.46.2", available: RELEASE, hint: said("无法连接到 GitHub。检查设置中的代理。这台电脑上没有放入任何内容。", "Can't connect to GitHub. Check the proxy in Settings. Nothing was put on this computer.") },
          grants: { to: TO, screen: "given", accessibility: "given" },
          mirroring: "away",
        },
        android: { adb: stopped, devices: [] },
      };
    case "phone-failed":
      return {
        ios: {
          system,
          driver: { state: "failed", available: RELEASE, hint: said("下载的软件包与固定的 SHA256 不一致，已丢弃，没有放入这台电脑。", "The downloaded package doesn't match the pinned SHA256. It was discarded and nothing was put on this computer.") },
          grants: { to: TO, screen: "unknown", accessibility: "unknown" },
          mirroring: "away",
        },
        android: { adb: null, devices: [] },
      };
    case "phone-android":
      return {
        ios: { system, driver: { state: "present", version: "0.47.0" }, grants: { to: TO, screen: "given", accessibility: "given" }, mirroring: "away" },
        android: { adb: running, devices: [{ serial: "R58N…3K2A", state: "unauthorized" }, { serial: "emul…5554", model: "sdk gphone64 arm64", state: "offline" }], step: ADB_STOP() },
      };
    default:
      return {
        ios: {
          system,
          driver: { state: "present", version: "0.47.0" },
          grants: { to: TO, screen: "given", accessibility: "given" },
          mirroring: "connected",
          clipboard: { state: "arrived", at: new Date(Date.now() - 20 * 60_000).toISOString(), waitSec: 10 },
        },
        android: { adb: running, devices: [{ serial: "2B1F…9A4D", model: "Pixel 8", state: "device" }], step: ADB_STOP() },
      };
  }
}

const DRY_RUN: PhoneDryRun = {
  read: 31,
  match: {
    flow: "小红书 · 收藏",
    step: 4,
    expect: { matched: 2, total: 2 },
    targets: [
      { label: "分享", found: true },
      { label: "复制链接", found: false },
    ],
  },
};

/** `param` 是地址栏的 `?mock=`；不是手机的几种样子时，每条都回内核兜底路由的 404。 */
export function createMockPhone(param: string | null): Phone {
  const kind = PHONE_MOCKS.find((k) => k === param);
  const notServed = () => Promise.reject(new ApiError(404, "no such route", "not-found"));
  if (kind === undefined) {
    return {
      phone: notServed,
      installDriver: notServed,
      updateDriver: notServed,
      removeDriver: notServed,
      openSettings: notServed,
      testClipboard: notServed,
      dryRun: notServed,
      lookForAndroid: notServed,
    };
  }
  let view = start(kind);
  const copy = (): PhoneView => structuredClone(view);
  const set = (ios: Partial<PhoneView["ios"]>) => {
    view = { ...view, ios: { ...view.ios, ...ios } };
  };
  const install = async () => {
    await wait(80);
    const target = view.ios.driver.available ?? RELEASE;
    set({ driver: { ...view.ios.driver, state: "installing" } });
    // 样例里装得很快；装好以后还没登记「mirroir serve」的，给登记那一步。授权还是原来的样子（系统记着给过「mirroir serve」的）。
    const registered = view.ios.grants.screen !== "unknown" || view.ios.grants.accessibility !== "unknown";
    setTimeout(() => set({ driver: { state: "present", version: target.version, ...(registered ? {} : { step: REGISTER() }) } }), 2500);
    return copy();
  };
  return {
    async phone() {
      await wait(60);
      return copy();
    },
    installDriver: install,
    updateDriver: install,
    async removeDriver() {
      await wait(300);
      // 卸掉不停「mirroir serve」：它还在跑时给停掉它的那一步。
      const serving = view.ios.grants.screen !== "unknown" || view.ios.grants.accessibility !== "unknown";
      set({ driver: { state: "absent", available: RELEASE, ...(serving ? { step: STOP() } : {}) } });
      return copy();
    },
    async openSettings() {
      await wait(60);
    },
    async testClipboard() {
      await wait(60);
      const at = new Date().toISOString();
      set({ clipboard: { state: "waiting", at, waitSec: 10 } });
      // 都好了的那一种试得到；别的几种等满了也没变。
      const arrived = kind === "phone-ready";
      setTimeout(() => set({ clipboard: { state: arrived ? "arrived" : "unchanged", at, waitSec: 10 } }), arrived ? 3000 : 10_000);
      return copy();
    },
    async lookForAndroid() {
      await wait(900);
      const adb = view.android.adb;
      if (adb === null) throw new ApiError(424, "no adb on this computer", "dependency-missing", said("这台电脑上没有找到 adb。", "adb wasn't found on this computer."));
      view = { ...view, android: { adb: { ...adb, running: true }, devices: structuredClone(FOUND), step: ADB_STOP() } };
      return copy();
    },
    async dryRun() {
      await wait(1500);
      return structuredClone(DRY_RUN);
    },
  };
}
