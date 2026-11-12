import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { Item, TrashView } from "../api/endpoints";
import { createMockLibrary } from "../mock/library";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./ItemPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const NO_ROUTE = () => Response.json({ code: "not-found", detail: "no such route" }, { status: 404 });

/** 内核今天的样子：条目没有 `pending`、每面没有 `key`，也还没有 /transforms。 */
function serve(item: Item, trash: TrashView, today = false) {
  const posted: string[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      if (init?.method === "POST") posted.push(path);
      if (path.endsWith(`/items/${item.id}`)) {
        if (!today) return Promise.resolve(Response.json(item));
        const { pending: _p, ...rest } = item;
        return Promise.resolve(Response.json({ ...rest, facets: item.facets.map(({ key: _k, ...f }) => f) }));
      }
      if (path.endsWith("/trash")) return Promise.resolve(Response.json(trash));
      if (path.endsWith("/topics")) return Promise.resolve(Response.json([]));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(NO_ROUTE());
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: [`/items/${item.id}`] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return posted;
}

it("says the video and cover were cleared from the trash and offers no way to restore them", async () => {
  const item = await createMockLibrary(false).item("w3");
  expect(item.stored).toBe("text-only");
  serve(item, { trashDays: 30, entries: [] });
  expect(await screen.findByText("视频和封面已经从回收站清掉了")).toBeTruthy();
  expect(screen.queryByRole("button", { name: /恢复视频和封面/ })).toBeNull();
  expect(screen.getByText("只留文字，视频和封面已经清掉了", { exact: false })).toBeTruthy();
});

it("uses the library's trash days instead of a fixed thirty", async () => {
  const item = await createMockLibrary(false).item("w3");
  serve(item, {
    trashDays: 7,
    entries: [{ id: "t1", reason: "keep-text", itemId: "w3", title: item.title, platform: item.platform, trashedAt: "2026-11-01T12:00:00.000Z" }],
  });
  expect(await screen.findByText("7 天内能恢复；口播文字、图里的字和画面缩略图都还在")).toBeTruthy();
  expect(screen.getByRole("button", { name: /恢复视频和封面/ })).toBeTruthy();
  expect(screen.queryByText(/30 天/)).toBeNull();
});

it("opens today's daemon items, which have no pending and no facet keys, without asking it to transform", async () => {
  const item = await createMockLibrary(false).item("c3");
  const posted = serve(item, { trashDays: 30, entries: [] }, true);
  expect(await screen.findByText("耶加雪菲水洗的风味")).toBeTruthy();
  expect(await screen.findByText("水洗处理的耶加雪菲，最常被说到的是花香和柑橘。")).toBeTruthy();
  // 内核还没有转换接口：不出「再转」，也不发转换请求。
  expect(screen.queryByRole("button", { name: /再转一次/ })).toBeNull();
  expect(posted).toEqual([]);
});

it("does not call the media cleared while the whole text-only item sits in the trash", async () => {
  const item = { ...(await createMockLibrary(false).item("w3")), trashedAt: "2026-11-10T12:00:00.000Z" };
  serve(item, {
    trashDays: 30,
    entries: [{ id: "t2", reason: "delete", itemId: "w3", title: item.title, platform: item.platform, trashedAt: "2026-11-10T12:00:00.000Z" }],
  });
  expect(await screen.findByText("视频和封面在回收站里")).toBeTruthy();
  expect(screen.queryByText(/已经从回收站清掉了/)).toBeNull();
});
