// 平台、插件、获取方式给人看的名字随界面语言走：订阅源目录里有各语言的名字，内核照设置的语言挑好了放在 `/catalog`；
// 插件清单只有一种语言的名字（`/plugins`、条目上的平台名），目录里没有的照它。应用里不写平台名。
import type { PluginsView } from "../api/endpoints";
import { useCatalog } from "../api/hooks";

export interface Names {
  platform: (id: string, fallback: string) => string;
  plugin: (id: string, fallback: string) => string;
  method: (pluginId: string, methodId: string, fallback: string) => string;
}

export function useNames(): Names {
  const catalog = useCatalog();
  const plugins = catalog.data?.plugins ?? [];
  const platforms = new Map(plugins.flatMap((p) => p.platforms.map((x) => [x.id, x.name] as const)));
  const pluginNames = new Map(plugins.map((p) => [p.id, p.name] as const));
  const methods = new Map(plugins.flatMap((p) => p.methods.map((m) => [`${p.id}#${m.id}`, m.name] as const)));
  return {
    platform: (id, fallback) => platforms.get(id) ?? fallback,
    plugin: (id, fallback) => pluginNames.get(id) ?? fallback,
    method: (pluginId, methodId, fallback) => methods.get(`${pluginId}#${methodId}`) ?? fallback,
  };
}

/** 条目、搜索结果上的平台名：随界面语言换。 */
export function PlatformName({ platform }: { platform: { id: string; name: string } }) {
  return useNames().platform(platform.id, platform.name);
}

/** 装着的获取插件换上随语言的名字：插件、它管的平台、各获取方式。按平台分页、导入页都照换过的摆。 */
export function localized(view: PluginsView, names: Names): PluginsView {
  return {
    ...view,
    plugins: view.plugins.map((p) => ({
      ...p,
      name: names.plugin(p.id, p.name),
      platforms: p.platforms.map((x) => ({ ...x, name: names.platform(x.id, x.name) })),
      methods: p.methods.map((m) => ({ ...m, name: names.method(p.id, m.id, m.name) })),
    })),
  };
}
