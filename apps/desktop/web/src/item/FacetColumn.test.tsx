import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, expect, it, vi } from "vitest";
import type { Item } from "../api/endpoints";
import { createMockLibrary } from "../mock/library";
import { FacetColumn } from "./FacetColumn";
import type { Player } from "./usePlayer";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

function player(): Player {
  return {
    t: 0,
    playing: false,
    image: null,
    still: null,
    region: null,
    peek: null,
    setPeek: vi.fn(),
    canPlay: true,
    video: undefined,
    attachVideo: () => undefined,
    toggle: () => undefined,
    go: vi.fn(),
    seek: vi.fn(),
    showImage: () => undefined,
    onTime: () => undefined,
  };
}

function show(item: Item, p: Player) {
  // 头上的「再转」要问 /transforms；这里的内核还没有这条，菜单不出现。
  vi.stubGlobal("fetch", vi.fn(() => Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }))));
  const wrap = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>{children}</QueryClientProvider>
  );
  render(<FacetColumn item={item} player={p} keep={{ days: 30, gone: false }} />, { wrapper: wrap });
}

it("lines up what is said and what is on screen by time, and jumps the player from either", async () => {
  const p = player();
  show(await createMockLibrary(false).item("c1"), p);
  const table = screen.getByRole("table", { name: "说的和画面上的，按时间对齐" });
  // 01:40 那一行：说的是 92 度，画面上写的是 90–96°C，同一行里。
  const row = within(table).getByRole("button", { name: "从 01:40 播" }).closest("[role=row]");
  if (!(row instanceof HTMLElement)) throw new Error("no row");
  expect(within(row).getByText("水温控制在 92 度左右")).toBeTruthy();
  expect(within(row).getByText("水温 90–96°C，浅烘取高")).toBeTruthy();

  fireEvent.click(within(row).getByRole("button", { name: "从 01:40 播" }));
  expect(p.seek).toHaveBeenCalledWith(100);
  fireEvent.click(within(row).getByRole("button", { name: "看 01:40 画面上的「水温 90–96°C，浅烘取高」" }));
  expect(p.go).toHaveBeenCalledWith(expect.objectContaining({ kind: "time", sec: 100, xywh: expect.any(Array) }));
});

it("keeps both transcripts of one speech, switches between them, and can put them side by side", async () => {
  show(await createMockLibrary(false).item("c1"), player());
  const copies = screen.getByRole("radiogroup", { name: "口播的几份" });
  expect(within(copies).getByRole("radio", { name: "平台字幕" }).getAttribute("aria-checked")).toBe("true");
  expect(screen.getByText("这期复刻粕谷哲的 4:6 手冲，先说比例")).toBeTruthy();

  fireEvent.click(within(copies).getByRole("radio", { name: "SenseVoice" }));
  expect(screen.getByText("这期复刻卡斯哲的四六手冲，先说比例")).toBeTruthy();
  expect(screen.queryByText("这期复刻粕谷哲的 4:6 手冲，先说比例")).toBeNull();

  fireEvent.click(screen.getByRole("button", { name: "两份并排" }));
  expect(screen.getByText("这期复刻卡斯哲的四六手冲，先说比例")).toBeTruthy();
  expect(screen.getByText("这期复刻粕谷哲的 4:6 手冲，先说比例")).toBeTruthy();
});

it("shows where on an image a line was read, and says when nothing was heard", async () => {
  const lib = createMockLibrary(false);
  const p = player();
  show(await lib.item("c2"), p);
  const line = screen.getByRole("button", { name: "在图 1 上框出「水温 93°C」" });
  fireEvent.mouseEnter(line);
  fireEvent.click(line);
  expect(p.go).toHaveBeenCalledWith(expect.objectContaining({ kind: "image", imageKey: "p1", xywh: expect.any(Array) }));
  cleanup();

  show(await lib.item("e7"), player());
  expect(screen.getByText(/没听到人说话/)).toBeTruthy();
  expect(screen.getByText("Rendering → Paint flashing")).toBeTruthy();
});

it("says the one true sentence for a speech that waits for its model, with the model's size and licence", async () => {
  show(await createMockLibrary(false).item("w5"), player());
  expect(screen.getByText("转口播要先下载 sense-voice-int8-2024-07-17 模型，约 163 MB，只下一次。")).toBeTruthy();
  expect(screen.getByRole("button", { name: "下载" })).toBeTruthy();
  expect(screen.getByText("许可：FunASR Model Open Source License")).toBeTruthy();
});
