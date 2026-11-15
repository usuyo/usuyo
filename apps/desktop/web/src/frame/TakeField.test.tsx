import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { Item, TakeResponse } from "../api/endpoints";
import { createAppRouter } from "../router";
import { useUiStore } from "../store/ui";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  useUiStore.getState().clearTake();
});

const TAKEN: TakeResponse = { id: "n1", seen: false, title: "胶片", platform: { id: "web", name: "网页" }, takenAt: "2026-11-15T08:00:00.000Z" };
const ITEM: Item = { ...TAKEN, author: undefined, topicIds: [], url: "https://example.com/a", stored: "link", facets: [], pending: [] };

/** 添加以后取这一条：前几次还在取，`after` 次以后回 `then`。 */
function serve(then: Item, after = 1) {
  let asked = 0;
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      if (path.endsWith("/items") && init?.method === "POST") return Promise.resolve(Response.json(TAKEN));
      if (path.endsWith("/items/n1")) {
        asked += 1;
        return Promise.resolve(Response.json(asked > after ? then : ITEM));
      }
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/later/x"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

async function paste(text: string) {
  const field = await screen.findByPlaceholderText("粘贴链接以添加");
  fireEvent.paste(field, { clipboardData: { getData: () => text } });
}

it("says a pasted link was added and is being fetched, then that it wasn't, with a link to the reason", async () => {
  serve({ ...ITEM, missing: "无法获取内容：这个网页需要登录后才能查看" });
  await paste("https://example.com/a");
  expect(await screen.findByText("已添加，正在获取内容…")).toBeTruthy();
  expect(await screen.findByText("已添加链接，未获取到内容", undefined, { timeout: 4000 })).toBeTruthy();
  expect(screen.getByRole("link", { name: "查看原因" }).getAttribute("href")).toBe("/items/n1");
});

it("says the content arrived and opens the item on the map", async () => {
  serve({ ...ITEM, stored: "complete" }, 0);
  await paste("https://example.com/a");
  expect(await screen.findByText("已添加并获取内容")).toBeTruthy();
  expect(screen.getByRole("link", { name: "查看" }).getAttribute("href")).toBe("/?open=n1");
});

it("lists only the pages that work in the rail, most used first", async () => {
  serve(ITEM);
  await screen.findByPlaceholderText("粘贴链接以添加");
  const nav = screen.getByRole("navigation", { name: "主导航" });
  const pages = [...nav.querySelectorAll("a")].map((a) => a.textContent);
  expect(pages).toEqual(["导入", "获取插件", "手机连接", expect.stringMatching(/^(废纸篓|回收站)$/), "设置"]);
});
