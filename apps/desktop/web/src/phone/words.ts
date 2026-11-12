// 「手机」页上的话，和从插件里挑出用到手机的取法。
import type { Access, AndroidDevice, FetchPlugin, GrantState, MethodBlocker, PhoneClipboardTest, PhoneView } from "../api/endpoints";
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

/** 装好了、「mirroir serve」还没登记：内核给的是登记那一步（`launchctl bootstrap`）。这时系统设置里还没有它。 */
export function needsRegister(ios: PhoneView["ios"]): boolean {
  return ios.driver.state === "present" && (ios.driver.step?.commands.some((c) => c.startsWith("launchctl bootstrap")) ?? false);
}

/** iPhone 那条路还差什么，按要先做的排；都好了是空的。看不出来给没给的授权不算差，试了才知道。 */
export function iosMissing(ios: PhoneView["ios"]): string[] {
  const out: string[] = [];
  if (!ios.system.mirroring) return ["升级到 macOS 15"];
  if (ios.driver.state !== "present") out.push("装上 mirroir");
  if (needsRegister(ios)) out.push("在终端里登记「mirroir serve」");
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
  /** 要哪种手机：清单的 `drives`。 */
  phone: "iphone" | "android";
  built: boolean;
  /** 内核说的：插件这边用得上（做好了、开着、插件自查没挡它）。手机连没连上不在里面，看 `PhoneView`。 */
  usable: boolean;
  /** 用不上的原因；停着（`paused`）也在里面，写到什么时候。 */
  blockers: MethodBlocker[];
  risk?: string;
}

/** 从插件里挑出驱动手机的取法（`drives` 是 `iphone` 或 `android`）；驱动电脑上客户端的留在「获取插件」页。 */
export function phoneMethods(plugins: FetchPlugin[]): PhoneMethod[] {
  return plugins.flatMap((p) =>
    p.methods.flatMap((m) =>
      m.drives === "iphone" || m.drives === "android"
        ? [
            {
              pluginId: p.id,
              platformId: p.platforms[0]?.id ?? p.id,
              platform: p.platforms[0]?.name ?? p.name,
              methodId: m.id,
              name: m.name,
              access: m.access,
              phone: m.drives,
              built: m.built,
              usable: m.usable,
              blockers: m.blockers,
              ...(m.riskNote ? { risk: m.riskNote.text } : {}),
            },
          ]
        : [],
    ),
  );
}

/** 这一种现在能不能用，不能的话差什么。`phone` 是 null 时内核还不给手机那一页。 */
export function methodState(m: PhoneMethod, phone: PhoneView | null): string {
  if (!m.built) return "插件写了，还没做好";
  if (!m.usable) return m.blockers.map((b) => b.hint).join("；");
  if (phone === null) return "插件这边用得上";
  if (m.phone === "android") {
    if (phone.android.adb === null) return "差：装上 adb";
    return phone.android.devices.some((d) => d.state === "device") ? "现在能用" : "差：连好一台安卓手机";
  }
  const missing = iosMissing(phone.ios);
  return missing.length === 0 ? "现在能用" : `差：${missing.join("、")}`;
}
