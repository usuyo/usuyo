import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { Draft, SearchHit } from "../api/endpoints";
import { createMockLibrary } from "../mock/library";
import { createMockPages } from "../mock/pages";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./DraftPage";
import "./DraftsPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const HIT: SearchHit = {
  itemId: "c5",
  title: "手冲水温到底差多少",
  platform: { id: "bilibili", name: "B站" },
  layer: "asr",
  position: { kind: "time", sec: 152, snapshotId: "s" },
  snippet: "92 度最平衡，甜感出来了",
  match: [0, 4],
};

function serve(draft: Draft | null, at: string) {
  const sent: { path: string; body: unknown }[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const url = new URL(input);
      const path = url.pathname;
      if ((init?.method ?? "GET") !== "GET") sent.push({ path, body: typeof init?.body === "string" ? JSON.parse(init.body) : null });
      if (draft && path.endsWith(`/drafts/${draft.id}`)) return Promise.resolve(Response.json(draft));
      if (draft && path.endsWith("/cite")) return Promise.resolve(Response.json(draft));
      if (draft && path.endsWith("/drafts")) return Promise.resolve(Response.json([draft]));
      if (path.endsWith("/search")) return Promise.resolve(Response.json(url.searchParams.get("q") ? [HIT] : []));
      if (path.endsWith("/topics")) return Promise.resolve(Response.json([]));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: [at] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return sent;
}

const sample = () => createMockPages(createMockLibrary(false), false).draft("d-pour");

it("lists every quotation's source at the end, in the order they are read", async () => {
  serve(await sample(), "/drafts/d-pour");
  const sources = await screen.findByRole("region", { name: "出处" });
  const rows = within(sources).getAllByRole("listitem");
  expect(rows).toHaveLength(4);
  expect(rows[0]?.textContent).toContain("「粉 20 克，水 300 克，就是 1:15」");
  expect(rows[0]?.textContent).toContain("口播 00:31");
});

it("quotes a sentence found by search into the draft with its position", async () => {
  const sent = serve(await sample(), "/drafts/d-pour");
  fireEvent.change(await screen.findByPlaceholderText("搜口播、图里的字、作者写的"), { target: { value: "92" } });
  fireEvent.click(await screen.findByRole("button", { name: "把「92 度最平衡，甜感出来了」引进来" }));
  await waitFor(() => expect(sent.find((s) => s.path.endsWith("/drafts/d-pour/cite"))?.body).toEqual({ segment: { itemId: "c5", layer: "asr", text: "92 度最平衡，甜感出来了", position: HIT.position } }));
});

it("says one calm sentence on both pages when the daemon has no drafts", async () => {
  serve(null, "/drafts");
  expect(await screen.findByText(/这台电脑上的 usuyo 内核还没有成稿/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "写一篇" })).toBeNull();
});
