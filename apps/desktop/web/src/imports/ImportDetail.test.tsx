import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { ConfirmImportRequest, ImportEntry, ImportView } from "../api/endpoints";
import { setLanguageChoice } from "../i18n";
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
  expect(await screen.findByRole("button", { name: "添加 5 条" })).toBeTruthy();
  expect(await screen.findByText("删过的四")).toBeTruthy();

  // 回收站那组一下全不收
  const trashed = screen.getByRole("radiogroup", { name: "你删除过的：整组" });
  fireEvent.click(within(trashed).getByRole("radio", { name: "全部不添加" }));
  expect(await screen.findByRole("button", { name: "添加 3 条" })).toBeTruthy();

  // 新的那组展开，去掉「二」
  const newGroup = screen.getByRole("radiogroup", { name: "新内容：整组" }).closest("section");
  if (!newGroup) throw new Error("no section");
  fireEvent.click(within(newGroup).getByRole("button", { name: "逐条选择" }));
  fireEvent.click(within(newGroup).getByRole("checkbox", { name: "添加 二" }));
  expect(await screen.findByRole("button", { name: "添加 2 条" })).toBeTruthy();
  expect(within(newGroup).getByText("添加 2/3 条")).toBeTruthy();

  fireEvent.click(screen.getByRole("button", { name: "添加 2 条" }));
  const sent: ConfirmImportRequest = { new: true, trashed: false, except: [2] };
  await waitFor(() => expect(confirmed).toEqual([sent]));
});

it("pressing a whole-group choice drops that group's single-entry changes", async () => {
  serve(view());
  const newGroup = (await screen.findByRole("radiogroup", { name: "新内容：整组" })).closest("section");
  if (!newGroup) throw new Error("no section");
  fireEvent.click(within(newGroup).getByRole("button", { name: "逐条选择" }));
  fireEvent.click(await within(newGroup).findByRole("checkbox", { name: "添加 一" }));
  expect(await screen.findByRole("button", { name: "添加 4 条" })).toBeTruthy();
  fireEvent.click(within(newGroup).getByRole("radio", { name: "全部添加" }));
  expect(await screen.findByRole("button", { name: "添加 5 条" })).toBeTruthy();
});

it("words the discard button for a finished import as removing the record", async () => {
  serve(view({ state: "done", taken: 3 }));
  fireEvent.click(await screen.findByRole("button", { name: "移除记录" }));
  expect(screen.getByText("移除这条导入记录？已添加的条目会保留。")).toBeTruthy();
  expect(screen.queryByRole("button", { name: "放弃导入" })).toBeNull();
});

it("keeps the cancel wording before everything is taken", async () => {
  serve(view({ state: "taking", taken: 1 }));
  expect(await screen.findByRole("button", { name: "放弃导入" })).toBeTruthy();
});

it("counts in whole English sentences, singular and plural", async () => {
  setLanguageChoice("en");
  serve(view({ preview: { new: 1, stored: 1, trashed: 0 } }));
  expect(await screen.findByRole("button", { name: "Add 1 Item" })).toBeTruthy();
  expect(screen.getByText("1 new item will be added. 1 saved item won't change.")).toBeTruthy();
  cleanup();
  serve(view());
  expect(await screen.findByRole("button", { name: "Add 5 Items" })).toBeTruthy();
  expect(screen.getByText("New items to add: 3. Items to restore from the Trash: 2. 1 saved item won't change.")).toBeTruthy();
});
