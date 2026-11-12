// 「手机」页上的话，和从插件里挑出用到手机的取法。
import type { Access, AndroidDevice, FetchPlugin, GrantState, PhoneClipboardTest, PhoneView, PluginPause } from "../api/endpoints";
import { timeText } from "../plugins/words";

export const GRANT_TEXT: Record<GrantState, string> = {
  given: "给了。",
  missing: "还没给。",
  unknown: "看不出来给没给。",
};

export const DEVICE_TEXT: Record<AndroidDevice["state"], string> = {
  unauthorized: "在手机上点「允许 USB 调试」",
  device: "连好了",
  offline: "连着，但不回话：拔下来再插上",
};

/** 序列号长的只留头尾四位。 */
export function shortSerial(serial: string): string {
  return serial.length > 10 ? `${serial.slice(0, 4)}…${serial.slice(-4)}` : serial;
}

/** 剪贴板试的结果，一句话；拷的字从不出现。 */
export function clipboardText(t: PhoneClipboardTest | undefined): string {
  if (t === undefined) return "链接从手机经通用剪贴板回到这台 Mac。";
  switch (t.state) {
    case "waiting":
      return "在等：去 iPhone 上随便拷贝一段字。";
    case "arrived":
      return `${timeText(t.at)} 从 iPhone 拷的到了这台 Mac。`;
    default:
      return `${t.waitSec} 秒里这台 Mac 的剪贴板没有变。`;
  }
}

/** iPhone 那条路还差什么，按要先做的排；都好了是空的。看不出来给没给的授权不算差，试了才知道。 */
export function iosMissing(ios: PhoneView["ios"]): string[] {
  const out: string[] = [];
  if (!ios.system.mirroring) return ["升级到 macOS 15"];
  if (ios.driver.state !== "present") out.push("装上 mirroir");
  if (ios.driver.state === "present" && ios.grants.screen === "missing") out.push("给屏幕录制");
  if (ios.driver.state === "present" && ios.grants.accessibility === "missing") out.push("给辅助功能");
  if (ios.mirroring !== "connected") out.push("连上 iPhone");
  return out;
}

/** 用到手机的一种取法，「手机」页上一行。 */
export interface PhoneMethod {
  pluginId: string;
  /** 插件卡片在「获取插件」页的哪个平台下。 */
  platformId: string;
  platform: string;
  methodId: string;
  name: string;
  access: Access;
  /** 契约里没写是哪种手机：照取法的 id 猜（phone.md §4 叫它们 `iphone`、`android`），不是 `android` 的都算 iPhone 镜像。 */
  phone: "iphone" | "android";
  built: boolean;
  enabled: boolean;
  pauses: PluginPause[];
  risk?: string;
}

/** 从插件里挑出账号在手机上的取法。插件和取法的形状可能会变，用到它们的只有这一个函数。 */
export function phoneMethods(plugins: FetchPlugin[]): PhoneMethod[] {
  return plugins.flatMap((p) =>
    p.methods
      .filter((m) => m.account.kind === "own-phone")
      .map((m) => ({
        pluginId: p.id,
        platformId: p.platforms[0]?.id ?? p.id,
        platform: p.platforms[0]?.name ?? p.name,
        methodId: m.id,
        name: m.name,
        access: m.access,
        phone: m.id === "android" ? ("android" as const) : ("iphone" as const),
        built: m.built,
        enabled: m.enabled,
        pauses: [...p.pauses, ...m.pauses],
        ...(m.riskNote ? { risk: m.riskNote.text } : {}),
      })),
  );
}

/** 这一种现在能不能用，不能的话差什么。`phone` 是 null 时内核还不给手机那一页。 */
export function methodState(m: PhoneMethod, phone: PhoneView | null): string {
  if (!m.built) return "插件写了，还没做好";
  if (!m.enabled) return "关着";
  if (m.pauses.length > 0) return "停着，见插件卡片";
  if (phone === null) return "开着";
  if (m.phone === "android") {
    if (phone.android.adb === null) return "差：装上 adb";
    return phone.android.devices.some((d) => d.state === "device") ? "现在能用" : "差：连好一台安卓手机";
  }
  const missing = iosMissing(phone.ios);
  return missing.length === 0 ? "现在能用" : `差：${missing.join("、")}`;
}
