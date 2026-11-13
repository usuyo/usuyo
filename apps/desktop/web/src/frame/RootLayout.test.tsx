import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { createAppRouter } from "../router";

const { titleBarHidden } = vi.hoisted(() => ({ titleBarHidden: vi.fn<() => Promise<boolean>>() }));
vi.mock("../shell", async (importOriginal) => ({ ...(await importOriginal<typeof import("../shell")>()), titleBarHidden }));

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  titleBarHidden.mockReset();
});

function show() {
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL) => {
      const path = new URL(input).pathname;
      if (path.endsWith("/map")) return Promise.resolve(Response.json({ regions: [], points: [], unreadable: [] }));
      if (path.endsWith("/topics")) return Promise.resolve(Response.json([]));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(Response.json({ code: "not-found" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/"] }));
  const { container } = render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return container;
}

it("leaves a draggable band with nothing in it at the top when the window opened without the system title bar", async () => {
  titleBarHidden.mockResolvedValue(true);
  const container = show();
  await waitFor(() => expect(container.querySelector("[data-tauri-drag-region]")).not.toBeNull());
  const band = container.querySelector("[data-tauri-drag-region]");
  expect(band?.querySelectorAll("button, a, input").length).toBe(0);
  expect(await screen.findByText(/尚未添加任何内容/)).toBeTruthy();
});

it("adds no band when the window has its system title bar", async () => {
  titleBarHidden.mockResolvedValue(false);
  const container = show();
  expect(await screen.findByText(/尚未添加任何内容/)).toBeTruthy();
  expect(container.querySelector("[data-tauri-drag-region]")).toBeNull();
});
