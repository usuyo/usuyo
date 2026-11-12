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

const itemRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/items/$id",
  component: lazyRouteComponent(() => import("./item/ItemPage"), "ItemPage"),
});

const takeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/take",
  component: lazyRouteComponent(() => import("./capture/CapturePage"), "CapturePage"),
});

export interface PluginsSearch {
  /** `text`：开着「变成文字」那一栏；不给是第一个平台。 */
  show?: "text";
}

const pluginsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/plugins",
  validateSearch: (search: Record<string, unknown>): PluginsSearch => (search.show === "text" ? { show: "text" } : {}),
  component: lazyRouteComponent(() => import("./plugins/PluginsPage"), "PluginsPage"),
});

export interface ImportSearch {
  /** 开着的那一次导入。 */
  open?: string;
}

const importRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/import",
  validateSearch: (search: Record<string, unknown>): ImportSearch =>
    typeof search.open === "string" && search.open !== "" ? { open: search.open } : {},
  component: lazyRouteComponent(() => import("./imports/ImportPage"), "ImportPage"),
});

const trashRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/trash",
  component: lazyRouteComponent(() => import("./trash/TrashPage"), "TrashPage"),
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
    if (params.page === "plugins") throw redirect({ to: "/plugins" });
    if (params.page === "trash") throw redirect({ to: "/trash" });
    if (params.page === "settings") throw redirect({ to: "/settings" });
  },
  component: LaterPage,
});

const routeTree = rootRoute.addChildren([homeRoute, searchRoute, itemRoute, takeRoute, pluginsRoute, importRoute, trashRoute, settingsRoute, laterRoute]);

export function createAppRouter(history?: Parameters<typeof createRouter>[0]["history"]) {
  return createRouter({ routeTree, history });
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
