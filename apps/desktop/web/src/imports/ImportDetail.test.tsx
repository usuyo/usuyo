import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { ConfirmImportRequest, ImportEntry, ImportView } from "../api/endpoints";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./ImportPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const ID = "0193f2b0-0001-7f3a-8b4c-5d6e7f8a9b0c";

function view(over: Partial<ImportView> = {}): ImportView {
  return {
    id: ID,
    kind: "delivered",
    title: "一批链接",
    createdAt: "2026-11-11T09:00:00.000Z",
    state: "ready",
    pages: 0,
    preview: { new: 3, stored: 1, trashed: 2 },
    taken: 0,
    notStored: 0,
    skipped: 0,
    ...over,
  };
}

const entry = (seq: number, standing: ImportEntry["standing"], title: string): ImportEntry => ({
  seq,
  url: `https://example.org/${seq}`,
  title,
  standing,
  taken: false,
});

const ENTRIES: ImportEntry[] = [
  entry(1, "new", "一"),
  entry(2, "new", "二"),
  entry(3, "new", "三"),
  entry(4, "trashed", "删过的四"),
  entry(5, "trashed", "删过的五"),
  entry(6, "stored", "存着的六"),
];

function serve(v: ImportView) {
  const confirmed: unknown[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const url = new URL(input);
      const path = url.pathname;
      if (path.endsWith("/confirm")) {
        const body: unknown = JSON.parse(typeof init?.body === "string" ? init.body : "null");
        confirmed.push(body);
        return Promise.resolve(Response.json({ ...v, state: "taking" }));
      }
      if (path.endsWith("/entries")) {
        const standing = url.searchParams.get("standing");
        return Promise.resolve(Response.json(ENTRIES.filter((e) => e.standing === standing)));
      }
      if (path.endsWith(`/imports/${ID}`)) return Promise.resolve(Response.json(v));
      if (path.endsWith("/imports")) return Promise.resolve(Response.json([v]));
      if (path.endsWith("/plugins")) return Promise.resolve(Response.json({ plugins: [], notLoaded: [] }));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(Response.json([]));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: [`/import?open=${ID}`] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return confirmed;
}

it("takes new and trashed entries by default, then whole groups or single entries can be left out", async () => {
  const confirmed = serve(view());
  expect(await screen.findByRole("button", { name: "开始收下 5 条" })).toBeTruthy();
  expect(await screen.findByText("删过的四")).toBeTruthy();

  // 回收站那组一下全不收
  const trashed = screen.getByRole("radiogroup", { name: "你删过的：整组" });
  fireEvent.click(within(trashed).getByRole("radio", { name: "全不收" }));
  expect(await screen.findByRole("button", { name: "开始收下 3 条" })).toBeTruthy();

  // 新的那组展开，去掉「二」
  const newGroup = screen.getByRole("radiogroup", { name: "新的：整组" }).closest("section");
  if (!newGroup) throw new Error("no section");
  fireEvent.click(within(newGroup).getByRole("button", { name: "逐条挑" }));
  fireEvent.click(within(newGroup).getByRole("checkbox", { name: "收下 二" }));
  expect(await screen.findByRole("button", { name: "开始收下 2 条" })).toBeTruthy();
  expect(within(newGroup).getByText("收 2 / 3 条")).toBeTruthy();

  fireEvent.click(screen.getByRole("button", { name: "开始收下 2 条" }));
  const sent: ConfirmImportRequest = { new: true, trashed: false, except: [2] };
  await waitFor(() => expect(confirmed).toEqual([sent]));
});

it("pressing a whole-group choice drops that group's single-entry changes", async () => {
  serve(view());
  const newGroup = (await screen.findByRole("radiogroup", { name: "新的：整组" })).closest("section");
  if (!newGroup) throw new Error("no section");
  fireEvent.click(within(newGroup).getByRole("button", { name: "逐条挑" }));
  fireEvent.click(await within(newGroup).findByRole("checkbox", { name: "收下 一" }));
  expect(await screen.findByRole("button", { name: "开始收下 4 条" })).toBeTruthy();
  fireEvent.click(within(newGroup).getByRole("radio", { name: "全收" }));
  expect(await screen.findByRole("button", { name: "开始收下 5 条" })).toBeTruthy();
});

it("words the discard button for a finished import as removing the record", async () => {
  serve(view({ state: "done", taken: 3 }));
  fireEvent.click(await screen.findByRole("button", { name: "去掉这条记录" }));
  expect(screen.getByText("去掉这条导入记录？收下的条目都留着。")).toBeTruthy();
  expect(screen.queryByRole("button", { name: "不要这次导入了" })).toBeNull();
});

it("keeps the cancel wording before everything is taken", async () => {
  serve(view({ state: "taking", taken: 1 }));
  expect(await screen.findByRole("button", { name: "不要这次导入了" })).toBeTruthy();
});
