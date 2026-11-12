import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { TransformsView } from "../api/endpoints";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./PluginsPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const VIEW: TransformsView = {
  auto: true,
  plugins: [
    {
      id: "builtin/asr-sherpa",
      name: "口播转文字",
      version: "0.1.0",
      jobs: ["asr"],
      models: [{ name: "sense-voice-int8", size: 163_002_883, license: "FunASR", state: "absent" }],
      health: "unavailable",
      hint: "要先下载模型",
    },
    {
      id: "lin/asr-firered",
      name: "FireRedASR2 转口播",
      version: "0.3.1",
      jobs: ["asr"],
      models: [{ name: "firered", size: 912_000_000, license: "Apache-2.0", state: "present" }],
      health: "ok",
    },
    {
      id: "builtin/media",
      name: "音视频",
      version: "0.1.0",
      jobs: ["play", "keyframes"],
      bundled: [{ name: "FFmpeg", license: "LGPL-2.1-or-later" }],
      models: [],
      health: "ok",
    },
  ],
};

function serve(view: TransformsView) {
  const posted: string[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      if (init?.method === "POST") posted.push(path);
      if (path.endsWith("/plugins")) return Promise.resolve(Response.json({ plugins: [], notLoaded: [] }));
      if (path.endsWith("/transforms")) return Promise.resolve(Response.json(view));
      if (path.endsWith("/download")) return Promise.resolve(Response.json({ state: "downloading" }, { status: 202 }));
      if (path.endsWith("/remove")) return Promise.resolve(new Response(null, { status: 204 }));
      if (path.endsWith("/events")) return new Promise(() => {});
      if (path.endsWith("/sites") || path.endsWith("/visitors")) return Promise.resolve(Response.json([]));
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/plugins?show=text"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return posted;
}

it("lists transform plugins with each model's size and licence, downloads one, and removes one after asking", async () => {
  const posted = serve(VIEW);
  fireEvent.click(await screen.findByRole("button", { name: "下载" }));
  expect(screen.getByText("163 MB")).toBeTruthy();
  expect(screen.getByText(/许可：FunASR/)).toBeTruthy();
  await waitFor(() => expect(posted).toContain("/api/transforms/builtin/asr-sherpa/models/sense-voice-int8/download"));

  fireEvent.click(screen.getByRole("button", { name: "删掉" }));
  expect(screen.getByText(/已经转出来的文字都留着/)).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "删掉模型" }));
  await waitFor(() => expect(posted).toContain("/api/transforms/lin/asr-firered/models/firered/remove"));
});

it("shows each plugin by name with its jobs in words, and a missing model as a plain fact", async () => {
  serve(VIEW);
  expect(await screen.findByText("音视频")).toBeTruthy();
  expect(screen.getByText("合成能播的视频、取关键帧")).toBeTruthy();
  expect(screen.getByText("自带 FFmpeg（LGPL-2.1-or-later）")).toBeTruthy();
  expect(screen.getAllByText("口播转成文字")).toHaveLength(2);
  expect(screen.getByText("sense-voice-int8")).toBeTruthy();
  // 只差模型没下：一句平常的话，不标成出了问题。
  expect(screen.getByText("要先下载模型").className).not.toContain("text-warn");
});
