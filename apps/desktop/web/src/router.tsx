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

export interface PluginsSearch {
  /** `text`：开着「变成文字」那一栏；不给是第一个平台。 */
  show?: "text";
  /** 开着哪个平台（`Platform.id`）；「手机」页的取法链过来用。 */
  platform?: string;
}

const pluginsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/plugins",
  validateSearch: (search: Record<string, unknown>): PluginsSearch =>
    search.show === "text" ? { show: "text" } : typeof search.platform === "string" && search.platform !== "" ? { platform: search.platform } : {},
  component: lazyRouteComponent(() => import("./plugins/PluginsPage"), "PluginsPage"),
});

export interface ImportSearch {
  /** 开着的那一次导入。 */
  open?: string;
  /** `file`：从「别的工具导出的文件」那一半开始。 */
  from?: "file";
}

const importRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/import",
  validateSearch: (search: Record<string, unknown>): ImportSearch => ({
    ...(typeof search.open === "string" && search.open !== "" ? { open: search.open } : {}),
    ...(search.from === "file" ? { from: "file" as const } : {}),
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

const routeTree = rootRoute.addChildren([homeRoute, searchRoute, itemRoute, topicRoute, draftsRoute, draftRoute, followRoute, changesRoute, exportRoute, takeRoute, pluginsRoute, phoneRoute, importRoute, trashRoute, settingsRoute, laterRoute]);

export function createAppRouter(history?: Parameters<typeof createRouter>[0]["history"]) {
  return createRouter({ routeTree, history });
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
