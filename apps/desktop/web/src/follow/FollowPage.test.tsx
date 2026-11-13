import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { FollowsView, FollowTrial } from "../api/endpoints";
import { createMockFollows } from "../mock/follows";
import { createMockLibrary } from "../mock/library";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./FollowPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const TRIAL: FollowTrial = { title: "每周胶片简报", firstRun: "record", listed: 5, would: [{ title: "这个月拍完的三卷", url: "https://example.org/1", picked: ["胶片"] }], left: { stored: 0, trashed: 0, unmatched: 4 } };

function serve(view: FollowsView | null) {
  const sent: { path: string; method: string; body: unknown }[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      const method = init?.method ?? "GET";
      if (method !== "GET") sent.push({ path, method, body: typeof init?.body === "string" ? JSON.parse(init.body) : null });
      if (view && path.endsWith("/follow-trials")) return Promise.resolve(Response.json(TRIAL));
      if (view && path.endsWith("/follows") && method === "POST") return Promise.resolve(Response.json(view.follows[0]));
      if (view && path.endsWith("/follows")) return Promise.resolve(Response.json(view));
      if (path.endsWith("/topics")) return Promise.resolve(Response.json([{ id: "film", name: "胶片摄影", followed: true }]));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/follow"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return sent;
}

const sample = () => createMockFollows(createMockLibrary(false), false).follows();

it("lists each follow with what it takes and why it waits, and labels what the machine picked", async () => {
  serve(await sample());
  expect(await screen.findByText("仅添加匹配「胶片摄影」的关键词和「C41」的内容")).toBeTruthy();
  expect(screen.getByText(/此订阅已暂停/)).toBeTruthy();
  const recent = screen.getByRole("region", { name: "最近添加" });
  expect(within(recent).getAllByText("自动选取")).toHaveLength(2);
});

it("tries a feed once before following it, then follows it as tried", async () => {
  const sent = serve(await sample());
  fireEvent.click(await screen.findByRole("button", { name: "新建订阅" }));
  fireEvent.click(screen.getByRole("radio", { name: "订阅源" }));
  fireEvent.change(screen.getByPlaceholderText("RSS、Atom 或 JSON Feed 地址"), { target: { value: "https://example.org/feed.xml" } });
  fireEvent.click(screen.getByRole("button", { name: "试运行" }));
  expect(await screen.findByText(/首次运行只记录现有内容，不添加/)).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "订阅" }));
  await waitFor(() => expect(sent.find((s) => s.path.endsWith("/follows"))?.body).toMatchObject({ kind: "feed", url: "https://example.org/feed.xml", take: { all: false, topics: ["film"] }, everyHours: 24 }));
});

it("says one calm sentence when the daemon cannot follow yet", async () => {
  serve(null);
  expect(await screen.findByText(/此版本的 usuyo 尚不支持订阅/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "新建订阅" })).toBeNull();
});
