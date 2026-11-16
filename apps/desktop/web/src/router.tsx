import { createRootRoute, createRoute, createRouter, lazyRouteComponent, redirect } from "@tanstack/react-router";
import { RootLayout } from "./frame/RootLayout";
import { LaterPage } from "./frame/LaterPage";
import { HomePage } from "./home/HomePage";

export interface HomeSearch {
  q?: string;
  /** 右侧面板里开着的条目。 */
  open?: string;
}

const rootRoute = createRootRoute({ component: RootLayout });

const homeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  validateSearch: (search: Record<string, unknown>): HomeSearch => ({
    ...(typeof search.q === "string" && search.q !== "" ? { q: search.q } : {}),
    ...(typeof search.open === "string" && search.open !== "" ? { open: search.open } : {}),
  }),
  component: HomePage,
});

// 搜索就是主页带着搜索词打开。
const searchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/search",
  validateSearch: (search: Record<string, unknown>): { q?: string } =>
    typeof search.q === "string" && search.q !== "" ? { q: search.q } : {},
  beforeLoad: ({ search }) => {
    throw redirect({ to: "/", search: search.q ? { q: search.q } : {} });
  },
});

export interface ItemSearch {
  /** 从引用跳回来：口播、画面上的字的那一秒。 */
  t?: number;
  /** 从引用跳回来：图里的字的那一张图。 */
  img?: string;
}

const itemRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/items/$id",
  validateSearch: (search: Record<string, unknown>): ItemSearch => ({
    ...(typeof search.t === "number" && Number.isFinite(search.t) && search.t >= 0 ? { t: search.t } : {}),
    ...(typeof search.img === "string" && search.img !== "" ? { img: search.img } : {}),
  }),
  component: lazyRouteComponent(() => import("./item/ItemPage"), "ItemPage"),
});

const topicRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/topics/$id",
  component: lazyRouteComponent(() => import("./topic/TopicPage"), "TopicPage"),
});

const draftsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/drafts",
  component: lazyRouteComponent(() => import("./drafts/DraftsPage"), "DraftsPage"),
});

const draftRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/drafts/$id",
  component: lazyRouteComponent(() => import("./drafts/DraftPage"), "DraftPage"),
});

const followRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/follow",
  component: lazyRouteComponent(() => import("./follow/FollowPage"), "FollowPage"),
});

const changesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/changes",
  component: lazyRouteComponent(() => import("./changes/ChangesPage"), "ChangesPage"),
});

const exportRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/export",
  component: lazyRouteComponent(() => import("./export/ExportPage"), "ExportPage"),
});

const takeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/take",
  component: lazyRouteComponent(() => import("./capture/CapturePage"), "CapturePage"),
});

// 获取插件：`/plugins` 是插件目录（订阅源里的和装着的），`/plugins/<owner>/<repo>` 是一个插件的详情，
// `/plugins/text` 是所有转写与识别插件和模型；以前一个平台一页的 `/plugins/<平台 id>` 换到管它的插件的详情。
const pluginsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/plugins",
  component: lazyRouteComponent(() => import("./plugins/PluginsPage"), "PluginsPage"),
});

const pluginsIndexRoute = createRoute({
  getParentRoute: () => pluginsRoute,
  path: "/",
  component: lazyRouteComponent(() => import("./plugins/Browser"), "BrowserRoute"),
});

const pluginsTextRoute = createRoute({
  getParentRoute: () => pluginsRoute,
  path: "text",
  component: lazyRouteComponent(() => import("./plugins/PluginsPage"), "TextRoute"),
});

const platformRoute = createRoute({
  getParentRoute: () => pluginsRoute,
  path: "$platform",
  component: lazyRouteComponent(() => import("./plugins/PluginsPage"), "PlatformRoute"),
});

export interface PluginSearch {
  /** 管几个平台的插件：设置照哪个平台显示。 */
  platform?: string;
}

const pluginDetailRoute = createRoute({
  getParentRoute: () => pluginsRoute,
  path: "$owner/$repo",
  validateSearch: (search: Record<string, unknown>): PluginSearch => (typeof search.platform === "string" && search.platform !== "" ? { platform: search.platform } : {}),
  component: lazyRouteComponent(() => import("./plugins/PluginDetail"), "PluginDetailRoute"),
});

export interface ImportSearch {
  /** 开着的那一次导入。 */
  open?: string;
  /** `file`：从「别的工具导出的文件」开始；`links`：从「粘贴链接」开始。 */
  from?: "file" | "links";
  /** 平台页链过来：先展开这一种能列的方法（`<插件 id>#<方法 id>`）。 */
  pick?: string;
  /** 平台页链过来：先选好这一种读文件的方法（`FileReader.id`）。 */
  reader?: string;
}

const importRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/import",
  validateSearch: (search: Record<string, unknown>): ImportSearch => ({
    ...(typeof search.open === "string" && search.open !== "" ? { open: search.open } : {}),
    ...(search.from === "file" || search.from === "links" ? { from: search.from } : {}),
    ...(typeof search.pick === "string" && search.pick !== "" ? { pick: search.pick } : {}),
    ...(typeof search.reader === "string" && search.reader !== "" ? { reader: search.reader } : {}),
  }),
  component: lazyRouteComponent(() => import("./imports/ImportPage"), "ImportPage"),
});

const trashRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/trash",
  component: lazyRouteComponent(() => import("./trash/TrashPage"), "TrashPage"),
});

const phoneRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/phone",
  component: lazyRouteComponent(() => import("./phone/PhonePage"), "PhonePage"),
});

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/settings",
  component: lazyRouteComponent(() => import("./settings/SettingsPage"), "SettingsPage"),
});

const laterRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/later/$page",
  // 做好了的几页有自己的路由。
  beforeLoad: ({ params }) => {
    if (params.page === "import") throw redirect({ to: "/import" });
    if (params.page === "follow") throw redirect({ to: "/follow" });
    if (params.page === "changes") throw redirect({ to: "/changes" });
    if (params.page === "export") throw redirect({ to: "/export" });
    if (params.page === "plugins") throw redirect({ to: "/plugins" });
    if (params.page === "trash") throw redirect({ to: "/trash" });
    if (params.page === "settings") throw redirect({ to: "/settings" });
  },
  component: LaterPage,
});

const routeTree = rootRoute.addChildren([homeRoute, searchRoute, itemRoute, topicRoute, draftsRoute, draftRoute, followRoute, changesRoute, exportRoute, takeRoute, pluginsRoute.addChildren([pluginsIndexRoute, pluginsTextRoute, platformRoute, pluginDetailRoute]), phoneRoute, importRoute, trashRoute, settingsRoute, laterRoute]);

export function createAppRouter(history?: Parameters<typeof createRouter>[0]["history"]) {
  return createRouter({ routeTree, history });
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
