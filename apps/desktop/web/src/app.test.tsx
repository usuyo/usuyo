import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { createMemoryHistory } from "@tanstack/react-router";
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { createAppRouter } from "./router";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

it("shows one calm line when the daemon is not running", async () => {
  vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new TypeError("Failed to fetch")));
  // jsdom 没有 scrollTo，路由换页时会调它。
  vi.stubGlobal("scrollTo", vi.fn());
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );

  expect((await screen.findAllByText("内核没在运行")).length).toBeGreaterThan(0);
  expect(screen.queryByText(/error/i)).toBeNull();
});
