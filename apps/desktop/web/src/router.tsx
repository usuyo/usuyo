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

const laterRoute = createRoute({ getParentRoute: () => rootRoute, path: "/later/$page", component: LaterPage });

const routeTree = rootRoute.addChildren([homeRoute, searchRoute, itemRoute, takeRoute, laterRoute]);

export function createAppRouter(history?: Parameters<typeof createRouter>[0]["history"]) {
  return createRouter({ routeTree, history });
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
