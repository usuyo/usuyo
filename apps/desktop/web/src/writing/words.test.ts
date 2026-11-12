import { describe, expect, it } from "vitest";
import type { Citation, PageBlock } from "../api/endpoints";
import { citeSource, foundText, jumpOf, pageAfter, whereText } from "./words";

const time = { kind: "time", sec: 150, snapshotId: "s" } as const;
const image = { kind: "image", imageKey: "p2" } as const;

describe("where a quotation sits and how to jump back", () => {
  it("names the layer and the second or the picture", () => {
    expect(whereText("asr", time)).toBe("口播 02:30");
    expect(whereText("ocr", image)).toBe("图里的字 · 图 2");
    expect(whereText("text")).toBe("作者写的");
  });
  it("jumps to the second or the picture", () => {
    expect(jumpOf(time)).toEqual({ t: 150 });
    expect(jumpOf(image)).toEqual({ img: "p2" });
    expect(jumpOf(undefined)).toEqual({});
  });
});

const cite = (over: Partial<Citation>): Citation => ({
  label: "a",
  itemId: "c1",
  layer: "asr",
  quote: "粉 20 克",
  title: "比例",
  author: "豆研所",
  date: "2026-10-21",
  found: "same",
  ...over,
});

it("says how a quotation was found back, and nothing when it is the same", () => {
  expect(foundText(cite({}))).toBeNull();
  expect(foundText(cite({ found: "changed", now: "粉二十克" }))).toBe("这一处现在写成：「粉二十克」");
  expect(foundText(cite({ found: "missing" }))).toBe("原文里找不到这句了，这是当时抄下的");
  expect(foundText(cite({ found: "gone", itemId: null }))).toBe("来源已删掉，这是当时抄下的");
});

it("writes the source with the year only when it is not this year", () => {
  const now = new Date("2026-11-12T00:00:00Z");
  expect(citeSource(cite({}), now)).toBe("比例　豆研所　10 月 21 日");
  expect(citeSource(cite({ date: "2025-03-02", author: undefined }), now)).toBe("比例　2025 年 3 月 2 日");
});

describe("the page sent back after one edit", () => {
  const blocks: PageBlock[] = [
    { id: "1", kind: "text", text: "一", cites: ["a"] },
    { id: "2", kind: "text", text: "二", cites: [] },
    { id: "3", kind: "text", text: "", cites: ["b"] },
  ];
  it("changes one block's text and drops a block emptied of words and quotations", () => {
    expect(pageAfter(blocks, { kind: "text", id: "1", text: " 一一 " }).blocks[0]).toEqual({ id: "1", kind: "text", text: "一一", cites: ["a"] });
    expect(pageAfter(blocks, { kind: "text", id: "2", text: "  " }).blocks.map((b) => b.id)).toEqual(["1", "3"]);
  });
  it("adds a new block at the end, but not an empty one", () => {
    expect(pageAfter(blocks, { kind: "text", id: null, text: "四" }).blocks.at(-1)).toEqual({ kind: "text", text: "四", cites: [] });
    expect(pageAfter(blocks, { kind: "text", id: null, text: "" }).blocks).toHaveLength(3);
  });
  it("takes a quotation out of a block, dropping the block when nothing is left", () => {
    expect(pageAfter(blocks, { kind: "uncite", id: "1", label: "a" }).blocks[0]?.cites).toEqual([]);
    expect(pageAfter(blocks, { kind: "uncite", id: "3", label: "b" }).blocks.map((b) => b.id)).toEqual(["1", "2"]);
  });
});
