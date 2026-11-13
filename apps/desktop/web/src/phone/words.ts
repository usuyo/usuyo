// 「手机」页上的话，和从插件里挑出用到手机的获取方式。
import type { Access, AndroidDevice, FetchPlugin, GrantState, MethodBlocker, PhoneClipboardTest, PhoneView } from "../api/endpoints";
import { t, type MessageKey } from "../i18n";
import { formatList } from "../i18n/format";
import { timeText } from "../plugins/words";

/** 授权的状态，一句话；没给和看不出来的带上去哪里打开。 */
export function grantText(state: GrantState, to: string): string {
  return state === "given" ? t("phone.grant.given") : t(state === "missing" ? "phone.grant.missing" : "phone.grant.unknown", { to });
}

const DEVICE_TEXT = {
  unauthorized: "phone.devices.unauthorized",
  device: "phone.devices.device",
  offline: "phone.devices.offline",
} as const satisfies Record<AndroidDevice["state"], MessageKey>;

export function deviceText(state: AndroidDevice["state"]): string {
  return t(DEVICE_TEXT[state]);
}

/** 剪贴板测试的结果，一句话；拷贝的字从不出现。 */
export function clipboardText(c: PhoneClipboardTest | undefined): string {
  if (c === undefined) return t("phone.clipboard.idle");
  switch (c.state) {
    case "waiting":
      return t("phone.clipboard.waiting");
    case "arrived":
      return t("phone.clipboard.arrived", { time: timeText(c.at) });
    default:
      return t("phone.clipboard.unchanged", { count: c.waitSec });
  }
}

/** 装好了、「mirroir serve」还没登记：内核给的是登记那一步（`launchctl bootstrap`）。这时系统设置里还没有它。 */
export function needsRegister(ios: PhoneView["ios"]): boolean {
  return ios.driver.state === "present" && (ios.driver.step?.commands.some((c) => c.startsWith("launchctl bootstrap")) ?? false);
}

/** iPhone 那条路还差什么，按要先做的排；都好了是空的。看不出来给没给的授权不算差，试了才知道。 */
export function iosMissing(ios: PhoneView["ios"]): string[] {
  const out: string[] = [];
  if (!ios.system.mirroring) return [t("phone.missing.upgrade")];
  if (ios.driver.state !== "present") out.push(t("phone.missing.install"));
  if (needsRegister(ios)) out.push(t("phone.missing.register"));
  if (ios.driver.state === "present" && ios.grants.screen === "missing") out.push(t("phone.missing.screen"));
  if (ios.driver.state === "present" && ios.grants.accessibility === "missing") out.push(t("phone.missing.accessibility"));
  if (ios.mirroring !== "connected") out.push(t("phone.missing.connect"));
  return out;
}

/** 用到手机的一种获取方式，「手机」页上一行。 */
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

/** 从插件里挑出驱动手机的获取方式（`drives` 是 `iphone` 或 `android`）；驱动电脑上客户端的留在「获取插件」页。 */
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
  if (!m.built) return t("phone.methods.notBuilt");
  // 内核给的原因原样显示。
  if (!m.usable) return m.blockers.map((b) => b.hint).join(t("common.clauseSeparator"));
  if (phone === null) return t("phone.methods.pluginReady");
  if (m.phone === "android") {
    if (phone.android.adb === null) return t("phone.methods.needAdb");
    return phone.android.devices.some((d) => d.state === "device") ? t("phone.methods.usable") : t("phone.methods.needDevice");
  }
  const missing = iosMissing(phone.ios);
  return missing.length === 0 ? t("phone.methods.usable") : t("phone.methods.needs", { list: formatList(missing) });
}
