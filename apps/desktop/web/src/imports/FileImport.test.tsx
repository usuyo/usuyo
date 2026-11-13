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
  title: "site-2026.zip",
  file: { name: "site-2026.zip", reader: "某站" },
  createdAt: "2026-11-12T08:00:00.000Z",
  state: "ready",
  pages: 0,
  preview: { new: 2, stored: 0, trashed: 0 },
  taken: 0,
  notStored: 0,
  skipped: 0,
};

/** 交文件走 XMLHttpRequest（要报交了多少）；这里照内核回答，`hold` 为真时停在交了一半，等测试停下它。 */
interface Reply {
  status: number;
  body: unknown;
}

class FakeUpload {
  onprogress: ((e: { loaded: number }) => void) | null = null;
}

function fakeXhr(reply: (url: URL) => Reply, sent: { url: URL; body: unknown; headers: Record<string, string> }[], hold: boolean) {
  return class {
    upload = new FakeUpload();
    status = 0;
    statusText = "";
    responseText = "";
    timeout = 0;
    onload: (() => void) | null = null;
    onerror: (() => void) | null = null;
    ontimeout: (() => void) | null = null;
    onabort: (() => void) | null = null;
    private url = new URL("http://x");
    private headers: Record<string, string> = {};
    open(_method: string, url: URL) {
      this.url = new URL(url);
    }
    setRequestHeader(k: string, v: string) {
      this.headers[k] = v;
    }
    getResponseHeader() {
      return null;
    }
    abort() {
      this.onabort?.();
    }
    send(body: Blob) {
      sent.push({ url: this.url, body, headers: this.headers });
      setTimeout(() => {
        this.upload.onprogress?.({ loaded: Math.floor(body.size / 2) });
        if (hold) return;
        const r = reply(this.url);
        this.status = r.status;
        this.responseText = JSON.stringify(r.body);
        this.onload?.();
      }, 10);
    }
  };
}

function serve(readers: FileReader[], reply: (url: URL) => Reply = () => ({ status: 200, body: VIEW }), hold = false) {
  const sent: { url: URL; body: unknown; headers: Record<string, string> }[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal("XMLHttpRequest", fakeXhr(reply, sent, hold));
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string) => {
      const url = new URL(input);
      const path = url.pathname;
      if (path.endsWith("/file-readers")) return Promise.resolve(Response.json(readers));
      if (path.endsWith(`/imports/${VIEW.id}`)) return Promise.resolve(Response.json(VIEW));
      if (path.endsWith("/imports")) return Promise.resolve(Response.json([]));
      if (path.endsWith("/plugins")) return Promise.resolve(Response.json({ plugins: [], notLoaded: [] }));
      if (path.endsWith("/events")) return new Promise(() => {});
      if (path.endsWith("/entries")) return Promise.resolve(Response.json([{ seq: 1, title: "私信里的一张图", standing: "new", taken: false }]));
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/import?from=file"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return { sent, router };
}

async function pick(name: string, bytes = "PK") {
  await screen.findByText("某站设置里给的 zip");
  const input = document.querySelector<HTMLInputElement>('input[type="file"]');
  if (!input) throw new Error("no file input");
  const file = new File([bytes], name, { type: "application/zip" });
  fireEvent.change(input, { target: { files: [file] } });
  return { input, file };
}

it("hands a picked file straight to the chosen reader, then shows what it lists like any import", async () => {
  const { sent, router } = serve([READER]);
  const { input, file } = await pick("site-2026.zip");
  expect(input.accept).toBe(".zip");
  await waitFor(() => expect(sent).toHaveLength(1));
  // `#` 在查询里写成 `%23`。
  expect(sent[0]?.url.search).toContain("reader=someone%2Fsite%23archive");
  expect(sent[0]?.url.searchParams.get("reader")).toBe(READER.id);
  expect(sent[0]?.url.searchParams.get("name")).toBe("site-2026.zip");
  expect(sent[0]?.body).toBe(file);
  await waitFor(() => expect(router.state.location.search).toEqual({ open: VIEW.id }));
  expect(await screen.findByText("已读取：2 条")).toBeTruthy();
  // 只在文件里、没有链接的那条只写标题。
  expect((await screen.findAllByText("私信里的一张图")).length).toBeGreaterThan(0);
});

it("says how much has gone over while sending, and stops without an error", async () => {
  const { sent } = serve([READER], () => ({ status: 200, body: VIEW }), true);
  await pick("site-2026.zip", "x".repeat(4000));
  expect(await screen.findByText("已发送 2 KB，共 4 KB")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "停止" }));
  expect(await screen.findByText(/已停止，未保留已发送的部分/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "停止" })).toBeNull();
  expect(sent).toHaveLength(1);
});

it("shows the daemon's sentence when the reader cannot read the file", async () => {
  serve([READER], () => ({ status: 400, body: { code: "invalid-args", detail: "plugin: not an archive", hint: "这不像某站的归档：里面没有 data/ 这个文件夹" } }));
  await pick("notes.zip");
  expect(await screen.findByText("这不像某站的归档：里面没有 data/ 这个文件夹")).toBeTruthy();
  expect(screen.getByRole("button", { name: "选择文件…" })).toBeTruthy();
});
