import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { ExportView } from "../api/endpoints";
import { createMockExports } from "../mock/exports";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./ExportPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

function serve(view: ExportView | null) {
  const sent: string[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      if (init?.method === "POST" || init?.method === "PUT") sent.push(`${init.method} ${path}`);
      if (view && path.endsWith("/export/runs")) return Promise.resolve(Response.json({ id: "e9", at: new Date().toISOString(), state: "done", written: 48, readBack: [], both: [], left: [] }));
      if (view && path.endsWith("/export")) return Promise.resolve(Response.json(view));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/export"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return sent;
}

it("shows what the last export read back, kept from both sides and left alone", async () => {
  serve(await createMockExports(false).exportView());
  expect(await screen.findByText("两边都有更改")).toBeTruthy();
  expect(screen.getByText("冰手冲：热水 150，冰 100，研磨细一档，冰块用纯净水冻。")).toBeTruthy();
  expect(screen.getByText("开头的属性格式有误，无法读取")).toBeTruthy();
});

it("exports only when you press it", async () => {
  const sent = serve(await createMockExports(false).exportView());
  await screen.findByText("两边都有更改");
  expect(sent).toEqual([]);
  fireEvent.click(screen.getByRole("button", { name: "再次导出" }));
  await waitFor(() => expect(sent).toEqual(["POST /api/export/runs"]));
});

it("cannot export before a folder is chosen", async () => {
  serve(await createMockExports(true).exportView());
  expect((await screen.findByRole("button", { name: "导出" })).hasAttribute("disabled")).toBe(true);
  expect(screen.getByRole("textbox", { name: /导出文件夹/ })).toBeTruthy();
});

it("says one calm sentence when the daemon cannot export yet", async () => {
  serve(null);
  expect(await screen.findByText(/此版本的 usuyo 尚不能导出/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "导出" })).toBeNull();
});
