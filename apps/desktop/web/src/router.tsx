import { createRootRoute, createRoute, createRouter } from "@tanstack/react-router";
import { Layout } from "./components/Layout";
import { ItemPage } from "./routes/ItemPage";
import { ListPage } from "./routes/ListPage";
import { SearchPage } from "./routes/SearchPage";
import { TakePage } from "./routes/TakePage";

const rootRoute = createRootRoute({ component: Layout });

const takeRoute = createRoute({ getParentRoute: () => rootRoute, path: "/", component: TakePage });
const listRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/items",
  component: ListPage,
});
const itemRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/items/$id",
  component: ItemPage,
});
const searchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/search",
  validateSearch: (search: Record<string, unknown>): { q: string } => ({
    q: typeof search.q === "string" ? search.q : "",
  }),
  component: SearchPage,
});

const routeTree = rootRoute.addChildren([takeRoute, listRoute, itemRoute, searchRoute]);

export function createAppRouter(history?: Parameters<typeof createRouter>[0]["history"]) {
  return createRouter({ routeTree, history });
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
