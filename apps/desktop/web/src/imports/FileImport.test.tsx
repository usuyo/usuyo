import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { FileReader, ImportView } from "../api/endpoints";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./ImportPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const READER: FileReader = {
  id: "someone/site#archive",
  name: "官方归档",
  plugin: "某站",
  reads: "某站设置里给的 zip",
  patterns: ["site-*.zip"],
  lists: ["喜欢"],
};

const VIEW: ImportView = {
  id: "0193f2b0-0009-7f3a-8b4c-5d6e7f8a9b0c",
  kind: "file",
  title: "喜欢",
  file: { name: "site-2026.zip", reader: "某站 · 官方归档" },
  createdAt: "2026-11-12T08:00:00.000Z",
  state: "ready",
  pages: 0,
  preview: { new: 2, stored: 0, trashed: 0 },
  taken: 0,
  notStored: 0,
  skipped: 0,
};

function serve(readers: FileReader[] | null) {
  const posted: { url: URL; body: unknown }[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const url = new URL(input);
      const path = url.pathname;
      if (init?.method === "POST") posted.push({ url, body: init.body });
      if (readers && path.endsWith("/file-readers")) return Promise.resolve(Response.json(readers));
      if (readers && path.endsWith("/file-imports")) return Promise.resolve(Response.json(VIEW));
      if (path.endsWith(`/imports/${VIEW.id}`)) return Promise.resolve(Response.json(VIEW));
      if (path.endsWith("/imports")) return Promise.resolve(Response.json([]));
      if (path.endsWith("/plugins")) return Promise.resolve(Response.json({ plugins: [], notLoaded: [] }));
      if (path.endsWith("/events")) return new Promise(() => {});
      if (path.endsWith("/entries")) return Promise.resolve(Response.json([]));
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/import?from=file"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return { posted, router };
}

it("hands a picked file to the chosen reader, then shows what it lists like any import", async () => {
  const { posted, router } = serve([READER]);
  await screen.findByText("某站设置里给的 zip");
  const input = document.querySelector<HTMLInputElement>('input[type="file"]');
  if (!input) throw new Error("no file input");
  expect(input.accept).toBe(".zip");
  const file = new File(["PK"], "site-2026.zip", { type: "application/zip" });
  fireEvent.change(input, { target: { files: [file] } });
  fireEvent.click(await screen.findByRole("button", { name: "用官方归档读一遍" }));
  await waitFor(() => expect(posted).toHaveLength(1));
  expect(posted[0]?.url.searchParams.get("reader")).toBe(READER.id);
  expect(posted[0]?.url.searchParams.get("name")).toBe("site-2026.zip");
  expect(posted[0]?.body).toBe(file);
  await waitFor(() => expect(router.state.location.search).toEqual({ open: VIEW.id }));
  expect(await screen.findByText("读完了：2 条")).toBeTruthy();
});

it("says one calm sentence when the daemon reads no files yet", async () => {
  serve(null);
  expect(await screen.findByText(/这台电脑上的 usuyo 内核还读不了别的工具导出的文件/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "选一个文件" })).toBeNull();
});
