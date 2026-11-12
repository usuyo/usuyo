// 设置页上的话：网络出口、回收站、时钟跳了那一句。
import type { NetworkSetting } from "../api/endpoints";
import { dayText } from "../item/format";

export const NETWORK_TEXT: Record<NetworkSetting["mode"], { name: string; sub: string }> = {
  system: {
    name: "跟系统代理",
    sub: "插件出门和你的浏览器走同一个出口。平台看到的还是你平常上网的样子，不会突然换一个地方来访问，这样最安静。",
  },
  direct: { name: "直连", sub: "不走代理，直接连。系统开着代理时，插件和浏览器就走两条路了。" },
  custom: { name: "自己填一个代理", sub: "只给插件用，不改系统的设置。写 http://、socks5:// 或 socks5h:// 开头的地址。" },
};

const PROXY_SCHEMES = ["http:", "socks5:", "socks5h:"];

/** 填的代理地址像不像样：照内核，只认这三种开头，要有主机，不带路径和查询串；可以带用户名和密码。 */
export function proxyProblem(proxy: string): string | null {
  const p = proxy.trim();
  if (p === "") return "写一个代理地址";
  const u = URL.canParse(p) ? new URL(p) : null;
  const fits = u !== null && PROXY_SCHEMES.includes(u.protocol) && u.hostname !== "" && (u.pathname === "" || u.pathname === "/") && u.search === "" && u.hash === "";
  return fits ? null : "要写成 http://主机:端口 这样，也可以是 socks5:// 或 socks5h://";
}

export const TRASH_CHOICES: { id: "30" | "90" | "forever"; text: string; days: number | null }[] = [
  { id: "30", text: "30 天", days: 30 },
  { id: "90", text: "90 天", days: 90 },
  { id: "forever", text: "一直留着", days: null },
];

export function trashChoiceOf(days: number | null): "30" | "90" | "forever" | undefined {
  return TRASH_CHOICES.find((c) => c.days === days)?.id;
}

/** 回收站那一句：放多久，到期怎么样。 */
export function trashText(days: number | null): string {
  return days === null ? "删掉的和只留文字移出来的图、视频一直留在回收站里，不会自己清除。" : `删掉的和只留文字移出来的图、视频在回收站里放 ${days} 天，到期就彻底清除，不留记录；到期以前随时拿得回来。`;
}

/** 时钟跳了：说清楚发生了什么、为什么没清，只给一个动作。 */
export function heldText(held: { count: number; since: string }, days: number | null): string {
  const span = days === null ? "" : `超过了回收站留的 ${days} 天`;
  return `这台电脑的时间比 ${dayText(held.since)}往后跳了一大截${span ? `，${span}` : ""}。为了不因为时间不对把回收站整个清掉，到期的 ${held.count} 项先留着没清。时间是对的，就照时间清掉它们。`;
}
