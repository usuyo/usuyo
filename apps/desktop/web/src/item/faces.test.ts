import { describe, expect, it } from "vitest";
import type { Segment } from "../api/endpoints";
import { alignRows, currentRow, pendingText, sizeText, sourceText } from "./faces";
import { keepText } from "./format";

const at = (sec: number, text: string): Segment => ({ text, position: { kind: "time", sec, snapshotId: "s" } });

describe("faces wording", () => {
  it("says where each copy came from", () => {
    expect(sourceText("asr", { by: "platform", lang: "zh" })).toBe("平台给的字幕");
    expect(sourceText("asr", { by: "machine", tool: "SenseVoice", at: "2026-11-10T12:00:00.000Z" })).toBe("机器转的 · SenseVoice · 11 月 10 日");
    expect(sourceText("frames", { by: "machine", tool: "Vision" })).toBe("机器认的 · Vision");
  });

  it("writes one plain sentence per pending state, never a percentage", () => {
    expect(pendingText({ layer: "asr", state: "later" }, "")).toBe("口播还没转出来，会慢慢转。");
    expect(pendingText({ layer: "ocr", state: "on-power" }, "")).toBe("图里的字还没转出来，接上电源时会转。");
    expect(pendingText({ layer: "frames", state: "now" }, "")).toBe("在转画面上的字。");
    const model = { plugin: "builtin/asr-sherpa", name: "sv", size: 163_002_883, license: "x" };
    expect(pendingText({ layer: "asr", state: "needs-model", model }, "SenseVoice")).toBe("转口播要先下载 SenseVoice 模型，约 163 MB，只下一次。");
    expect(pendingText({ layer: "asr", state: "needs-model", model }, "SenseVoice", true)).toBe("在下载 SenseVoice 模型，约 163 MB。下好了这一条会接着转。");
    expect(pendingText({ layer: "asr", state: "failed", hint: "音轨读不出来" }, "")).toBe("口播没转出来：音轨读不出来");
  });

  it("writes sizes and how long the trash keeps things", () => {
    expect(sizeText(16_000_000)).toBe("16 MB");
    expect(sizeText(912_000_000)).toBe("912 MB");
    expect(sizeText(1_400_000_000)).toBe("1.4 GB");
    expect(keepText(30)).toBe("30 天内能恢复");
    expect(keepText(7)).toBe("7 天内能恢复");
    expect(keepText(null)).toBe("一直能恢复");
    expect(keepText(undefined)).toBe("能恢复");
  });
});

describe("aligning faces by time", () => {
  it("puts what is said and what is on screen at the same moment in one row", () => {
    const speech = [at(0, "大家好"), at(12, "先说比例"), at(102, "92 度")];
    const screen = [at(13, "1:15"), at(100, "90–96°C"), at(210, "50g · 70g")];
    const rows = alignRows([speech, screen]);
    expect(rows.map((r) => r.sec)).toEqual([0, 12, 100, 210]);
    expect(rows[1]?.cells.map((c) => c.map((s) => s.text))).toEqual([["先说比例"], ["1:15"]]);
    expect(rows[2]?.cells.map((c) => c.map((s) => s.text))).toEqual([["92 度"], ["90–96°C"]]);
    expect(rows[3]?.cells[0]).toEqual([]);
  });

  it("lights the last row that has started", () => {
    const rows = alignRows([[at(0, "a"), at(10, "b"), at(20, "c")]]);
    expect(currentRow(rows, 0)).toBe(0);
    expect(currentRow(rows, 15)).toBe(1);
    expect(currentRow(rows, 99)).toBe(2);
  });
});
