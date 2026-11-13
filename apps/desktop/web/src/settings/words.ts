// 设置页上的话：网络连接方式、废纸篓保留多久、时钟跳了那一句。
import type { NetworkSetting } from "../api/endpoints";
import { t } from "../i18n";
import { dayText } from "../item/format";

/** 连接方式的名字和说明。 */
export function networkText(mode: NetworkSetting["mode"]): { name: string; sub: string } {
  switch (mode) {
    case "system":
      return { name: t("settings.network.system"), sub: t("settings.network.systemSub") };
    case "direct":
      return { name: t("settings.network.direct"), sub: t("settings.network.directSub") };
    default:
      return { name: t("settings.network.custom"), sub: t("settings.network.customSub") };
  }
}

const PROXY_SCHEMES = ["http:", "socks5:", "socks5h:"];

/** 填的代理地址像不像样：照内核，只认这三种开头，要有主机，不带路径和查询串；可以带用户名和密码。 */
export function proxyProblem(proxy: string): string | null {
  const p = proxy.trim();
  if (p === "") return t("settings.network.proxyEmpty");
  const u = URL.canParse(p) ? new URL(p) : null;
  const fits = u !== null && PROXY_SCHEMES.includes(u.protocol) && u.hostname !== "" && (u.pathname === "" || u.pathname === "/") && u.search === "" && u.hash === "";
  return fits ? null : t("settings.network.proxyBad");
}

export const TRASH_DAYS: { id: "30" | "90" | "forever"; days: number | null }[] = [
  { id: "30", days: 30 },
  { id: "90", days: 90 },
  { id: "forever", days: null },
];

/** 保留多久的几个选项。 */
export function trashChoices(): { id: "30" | "90" | "forever"; text: string }[] {
  return TRASH_DAYS.map((c) => ({ id: c.id, text: c.days === null ? t("settings.storage.forever") : t("settings.storage.days", { count: c.days }) }));
}

export function trashChoiceOf(days: number | null): "30" | "90" | "forever" | undefined {
  return TRASH_DAYS.find((c) => c.days === days)?.id;
}

/** 废纸篓那一句：放多久，到期怎么样。 */
export function trashText(days: number | null): string {
  return days === null ? t("settings.storage.keepsForever") : t("settings.storage.keepsDays", { count: days });
}

/** 时钟跳了：说清楚发生了什么、为什么没删，只给一个动作。 */
export function heldText(held: { count: number; since: string }, days: number | null): string {
  const since = dayText(held.since);
  return days === null ? t("settings.storage.heldForever", { since, count: held.count }) : t("settings.storage.held", { since, days, count: held.count });
}
