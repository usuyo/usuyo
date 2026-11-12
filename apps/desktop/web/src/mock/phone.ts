// 样例：「手机」页。地址栏带 ?mock=phone-<样子> 才有；不带时和内核今天一样，这几条接口不在。
import { ApiError } from "../api/client";
import type { PhoneDryRun, PhoneView } from "../api/endpoints";
import type { Phone } from "../api/source";

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/** 几种样子：什么都没装；装了、授权没给；都好了；没装上；安卓手机在等你点允许。 */
export const PHONE_MOCKS = ["phone-none", "phone-grants", "phone-ready", "phone-failed", "phone-android"] as const;
export type PhoneMock = (typeof PHONE_MOCKS)[number];

const RELEASE = { version: "0.47.0", size: 12_400_000, source: "https://github.com/jfarcand/mirroir-mcp/releases/tag/v0.47.0" };
const TO = "mirroir serve";

function start(kind: PhoneMock): PhoneView {
  const system = { macos: "15.6.1", mirroring: true };
  const adb = { path: "/opt/homebrew/bin/adb", version: "36.0.0" };
  switch (kind) {
    case "phone-none":
      return {
        ios: { system, driver: { state: "absent", available: RELEASE }, grants: { to: TO, screen: "unknown", accessibility: "unknown" }, mirroring: "not-set-up" },
        android: { adb: null, devices: [] },
      };
    case "phone-grants":
      return {
        ios: { system, driver: { state: "present", version: "0.46.2", available: RELEASE }, grants: { to: TO, screen: "missing", accessibility: "unknown" }, mirroring: "away" },
        android: { adb, devices: [] },
      };
    case "phone-failed":
      return {
        ios: {
          system,
          driver: { state: "failed", available: RELEASE, hint: "下载下来的包和 SHA256SUMS 对不上，已经丢掉了，没有放进这台电脑" },
          grants: { to: TO, screen: "unknown", accessibility: "unknown" },
          mirroring: "away",
        },
        android: { adb: null, devices: [] },
      };
    case "phone-android":
      return {
        ios: { system, driver: { state: "present", version: "0.47.0" }, grants: { to: TO, screen: "given", accessibility: "given" }, mirroring: "away" },
        android: { adb, devices: [{ serial: "R58N70ABC3K2A", state: "unauthorized" }, { serial: "emulator-5554", model: "sdk_gphone64_arm64", state: "offline" }] },
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
        android: { adb, devices: [{ serial: "2B1F0C7E9A4D", model: "Pixel 8", state: "device" }] },
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
    return { phone: notServed, installDriver: notServed, updateDriver: notServed, removeDriver: notServed, openSettings: notServed, testClipboard: notServed, dryRun: notServed };
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
    // 样例里装得很快；装好以后授权还是原来的样子（系统记着给过「mirroir serve」的）。
    setTimeout(() => set({ driver: { state: "present", version: target.version } }), 2500);
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
      set({ driver: { state: "absent", available: RELEASE }, grants: { ...view.ios.grants, screen: "unknown", accessibility: "unknown" } });
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
    async dryRun() {
      await wait(1500);
      return structuredClone(DRY_RUN);
    },
  };
}
