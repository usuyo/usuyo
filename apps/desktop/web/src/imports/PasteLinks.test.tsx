import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { ImportEntry, ImportView } from "../api/endpoints";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./ImportPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const VIEW: ImportView = {
  id: "0193f2b0-0021-7f3a-8b4c-5d6e7f8a9b0c",
  kind: "pasted",
  title: "粘贴的链接",
  createdAt: "2026-11-15T08:00:00.000Z",
  state: "ready",
  pages: 0,
  preview: { new: 2, stored: 0, trashed: 0 },
  taken: 0,
  notStored: 0,
  skipped: 0,
};

const ENTRIES: ImportEntry[] = [
  { seq: 1, url: "https://example.org/a", standing: "new", taken: false },
  { seq: 2, url: "https://example.org/b", standing: "new", taken: false },
  { seq: 3, url: "https://", standing: "unrecognized", taken: false },
];

function serve(reply: { status: number; body: unknown } = { status: 200, body: VIEW }) {
  const posted: unknown[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const url = new URL(input);
      const path = url.pathname;
      if (path.endsWith("/imports/links") && init?.method === "POST") {
        const body: unknown = typeof init.body === "string" ? JSON.parse(init.body) : null;
        posted.push(body);
        return Promise.resolve(Response.json(reply.body, { status: reply.status }));
      }
      if (path.endsWith("/entries")) {
        const standing = url.searchParams.get("standing");
        return Promise.resolve(Response.json(ENTRIES.filter((e) => e.standing === standing)));
      }
      if (path.endsWith(`/imports/${VIEW.id}`)) return Promise.resolve(Response.json(VIEW));
      if (path.endsWith("/imports")) return Promise.resolve(Response.json([]));
      if (path.endsWith("/plugins")) return Promise.resolve(Response.json({ plugins: [], notLoaded: [] }));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/import?from=links"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return { posted, router };
}

it("hands the pasted text to the daemon as is, then shows the preview with what it couldn't recognize", async () => {
  const { posted, router } = serve();
  const box = await screen.findByRole("textbox", { name: "链接或分享文字" });
  const text = "https://example.org/a\n看看https://example.org/bhttps://";
  fireEvent.change(box, { target: { value: text } });
  fireEvent.click(screen.getByRole("button", { name: "识别链接" }));
  await waitFor(() => expect(posted).toEqual([{ text }]));
  await waitFor(() => expect(router.state.location.search).toEqual({ open: VIEW.id }));
  expect(await screen.findByText("已识别：2 条")).toBeTruthy();
  const bad = (await screen.findByText("无法识别")).closest("section");
  if (!bad) throw new Error("no unrecognized group");
  expect(await within(bad).findByText("https://")).toBeTruthy();
});

it("says the daemon's sentence when the text has no links", async () => {
  serve({ status: 400, body: { code: "invalid-args", detail: "the text has no links", hint: "没有找到链接。" } });
  const box = await screen.findByRole("textbox", { name: "链接或分享文字" });
  fireEvent.change(box, { target: { value: "没有链接" } });
  fireEvent.click(screen.getByRole("button", { name: "识别链接" }));
  expect(await screen.findByText("没有找到链接。")).toBeTruthy();
});
