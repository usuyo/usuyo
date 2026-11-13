import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { Item, TransformsView, TrashView } from "../api/endpoints";
import { setLanguageChoice } from "../i18n";
import { createMockLibrary } from "../mock/library";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./ItemPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const NO_ROUTE = () => Response.json({ code: "not-found", detail: "no such route" }, { status: 404 });

const TRANSFORMS: TransformsView = {
  auto: true,
  plugins: [
    { id: "builtin/media", name: "音视频", version: "0.1.0", jobs: ["play", "keyframes"], models: [], health: "ok" },
    { id: "builtin/ocr-vision", name: "图里的字", version: "0.1.0", jobs: ["ocr", "frames"], models: [], health: "ok" },
    { id: "lin/ocr-ppocr", name: "PP-OCR 认字", version: "0.2.0", jobs: ["ocr", "frames"], models: [], health: "ok" },
  ],
};

/** 照内核的接口回：条目、回收站、主题、转换插件；POST 记下路径和请求体。 */
function serve(item: Item, trash: TrashView, search = "") {
  const posted: { path: string; body: unknown }[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      if (init?.method === "POST") posted.push({ path, body: typeof init.body === "string" ? JSON.parse(init.body) : null });
      if (path.endsWith(`/items/${item.id}/transform`)) return Promise.resolve(Response.json({ queued: true }, { status: 202 }));
      if (path.endsWith(`/items/${item.id}`)) return Promise.resolve(Response.json(item));
      if (path.endsWith("/transforms")) return Promise.resolve(Response.json(TRANSFORMS));
      if (path.endsWith("/trash")) return Promise.resolve(Response.json(trash));
      if (path.endsWith("/topics")) return Promise.resolve(Response.json([]));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(NO_ROUTE());
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: [`/items/${item.id}${search}`] }));
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
  expect(await screen.findByText("视频和封面已从回收站中删除")).toBeTruthy();
  expect(screen.queryByRole("button", { name: /恢复视频和封面/ })).toBeNull();
  expect(screen.getByText("仅保留文字，视频和封面已删除", { exact: false })).toBeTruthy();
});

it("uses the library's trash days instead of a fixed thirty", async () => {
  const item = await createMockLibrary(false).item("w3");
  serve(item, {
    trashDays: 7,
    entries: [{ id: "t1", reason: "keep-text", itemId: "w3", title: item.title, platform: item.platform, trashedAt: "2026-11-01T12:00:00.000Z" }],
  });
  expect(await screen.findByText("7 天内可以恢复。转写、图中文字和画面缩略图仍然保留。")).toBeTruthy();
  expect(screen.getByRole("button", { name: /^恢复视频和封面/ })).toBeTruthy();
  expect(screen.queryByText(/30 天/)).toBeNull();
});

it("speaks English when English is chosen, with plurals and the platform's trash", async () => {
  setLanguageChoice("en");
  const item = await createMockLibrary(false).item("w3");
  serve(item, {
    trashDays: 1,
    entries: [{ id: "t1", reason: "keep-text", itemId: "w3", title: item.title, platform: item.platform, trashedAt: "2026-11-01T12:00:00.000Z" }],
  });
  expect(await screen.findByText("Can be restored for 1 day. The transcript, text in images, and video thumbnails are kept.")).toBeTruthy();
  expect(screen.getByText("The video and cover are in the Trash")).toBeTruthy();
  expect(screen.getByRole("button", { name: /^Restore Video and Cover/ })).toBeTruthy();
  expect(screen.getByRole("button", { name: /Open in Browser/ })).toBeTruthy();
});

it("asks once on open for what is still to come, and offers redo with the default and with another plugin", async () => {
  const item = await createMockLibrary(false).item("c1");
  const posted = serve(item, { trashDays: 30, entries: [] });
  expect(await screen.findByRole("slider")).toBeTruthy();
  await waitFor(() => expect(posted).toContainEqual({ path: `/api/items/${item.id}/transform`, body: { jobs: "pending" } }));
  fireEvent.click(await screen.findByRole("button", { name: "重新处理画面中文字，或改用其他插件" }));
  fireEvent.click(await screen.findByRole("menuitem", { name: /^改用「PP-OCR 认字」识别/ }));
  await waitFor(() => expect(posted).toContainEqual({ path: `/api/items/${item.id}/transform`, body: { job: "frames", plugin: "lin/ocr-ppocr" } }));
  fireEvent.click(screen.getByRole("button", { name: "重新处理视频，或改用其他插件" }));
  fireEvent.click(await screen.findByRole("menuitem", { name: /^重新提取关键帧/ }));
  await waitFor(() => expect(posted).toContainEqual({ path: `/api/items/${item.id}/transform`, body: { job: "keyframes" } }));
  expect(posted.filter((p) => JSON.stringify(p.body) === '{"jobs":"pending"}')).toHaveLength(1);
});

it("redoes the copy on show with the plugin that made it, not the first plugin with that job", async () => {
  const base = await createMockLibrary(false).item("c1");
  // 摆着的那份画面上的字是 PP-OCR 做的：「再转一次」点名用它，排在前面的 Vision 是「换成」。
  const item: Item = {
    ...base,
    facets: base.facets.map((f) => (f.layer === "frames" ? { ...f, source: { ...f.source, plugin: "lin/ocr-ppocr", tool: "PP-OCR" } } : f)),
  };
  const posted = serve(item, { trashDays: 30, entries: [] });
  fireEvent.click(await screen.findByRole("button", { name: "重新处理画面中文字，或改用其他插件" }));
  expect(await screen.findByRole("menuitem", { name: /^改用「图里的字」识别/ })).toBeTruthy();
  fireEvent.click(screen.getByRole("menuitem", { name: /^重新识别/ }));
  await waitFor(() => expect(posted).toContainEqual({ path: `/api/items/${item.id}/transform`, body: { job: "frames", plugin: "lin/ocr-ppocr" } }));
});

it("says a face needs a model with its name, size and licence, and tries again by asking for everything still to come", async () => {
  const base = await createMockLibrary(false).item("c1");
  const model = { plugin: "builtin/asr-sherpa", name: "sense-voice-int8-2024-07-17", size: 163_002_883, license: "FunASR Model Open Source License" };
  const item: Item = { ...base, pending: [{ layer: "asr", state: "needs-model", model }, { layer: "video", state: "failed", hint: "画面读不出来" }] };
  const posted = serve(item, { trashDays: 30, entries: [] });
  expect(await screen.findByText("转写需要先下载 sense-voice-int8-2024-07-17 模型（163 MB），只需下载一次。")).toBeTruthy();
  expect(screen.getByText("许可：FunASR Model Open Source License")).toBeTruthy();
  expect(screen.getByRole("button", { name: "下载" })).toBeTruthy();
  expect(screen.getByText("无法处理视频：画面读不出来")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "重试" }));
  await waitFor(() => expect(posted.filter((p) => JSON.stringify(p.body) === '{"jobs":"pending"}')).toHaveLength(2));
});

it("does not call the media cleared while the whole text-only item sits in the trash", async () => {
  const item = { ...(await createMockLibrary(false).item("w3")), trashedAt: "2026-11-10T12:00:00.000Z" };
  serve(item, {
    trashDays: 30,
    entries: [{ id: "t2", reason: "delete", itemId: "w3", title: item.title, platform: item.platform, trashedAt: "2026-11-10T12:00:00.000Z" }],
  });
  expect(await screen.findByText("视频和封面在回收站中")).toBeTruthy();
  expect(screen.queryByText(/已从回收站中删除/)).toBeNull();
});

it("opens at the second a quotation points back to", async () => {
  const item = await createMockLibrary(false).item("c1");
  serve(item, { trashDays: 30, entries: [] }, "?t=150");
  const slider = await screen.findByRole("slider");
  expect(slider.getAttribute("aria-valuetext")).toMatch(/^02:3/);
});
