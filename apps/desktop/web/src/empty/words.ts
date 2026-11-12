// 第一次打开时列出的「把存过的一批拿回来」的几种路子：照插件自己的取法，一样摆，不挑哪个。
import type { FetchPlugin } from "../api/endpoints";
import { accountText, ACCESS_TEXT, getsText } from "../plugins/words";

export interface Way {
  key: string;
  /** 取法的名字，例如「浏览器脚本」。 */
  name: string;
  /** 「列出：书签、喜欢」这类，一样一行。 */
  gets: string[];
  access: string;
  account: string;
  /** 从哪一页开始：开着的、能列或能交链接的去导入页，别的去获取插件页看。 */
  to: "/import" | "/plugins";
}

/** 一个插件的几种：做好了的一种一行；声明了、还没做好的只列名字。 */
export interface WayGroup {
  plugin: string;
  ways: Way[];
  later: string[];
}

/**
 * 能把存过的一批拿回来的取法：能列来源的，或由别处把链接交进来的。只取单条的不算；
 * 读别的工具导出的文件的那几种（`readers` 里的 `<插件 id>#<取法 id>`）另列在文件那一段，这里不重复。
 * 顺序照插件给的，插件里照它自己的取法顺序；`siteName` 把站点换成给人看的名字。
 */
export function waysOf(plugins: FetchPlugin[], readers: Set<string>, siteName: (site: string) => string): WayGroup[] {
  return plugins
    .map((p) => {
      const all = p.methods.filter(
        (m) => ((m.gets.list?.length ?? 0) > 0 || m.gets.delivers === "links") && !readers.has(`${p.id}#${m.id}`),
      );
      return {
        plugin: p.name,
        ways: all
          .filter((m) => m.built)
          .map((m) => ({
            key: `${p.id}#${m.id}`,
            name: m.name,
            gets: getsText({ ...(m.gets.list ? { list: m.gets.list } : {}), ...(m.gets.delivers ? { delivers: m.gets.delivers } : {}) }),
            access: ACCESS_TEXT[m.access],
            account: accountText(m.account, (m.account.sites ?? []).map(siteName)),
            to: m.enabled && m.account.kind !== "own-phone" ? ("/import" as const) : ("/plugins" as const),
          })),
        later: all.filter((m) => !m.built).map((m) => m.name),
      };
    })
    .filter((g) => g.ways.length > 0 || g.later.length > 0);
}
