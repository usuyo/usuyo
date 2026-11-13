import { describe, expect, it } from "vitest";
import { setLanguageChoice } from "../i18n";
import type { Segment } from "../api/endpoints";
import { alignRows, copyLabel, currentRow, pendingText, sizeText, sourceText, type CopyFacet } from "./faces";
import { imageName, keepText } from "./format";

const at = (sec: number, text: string): Segment => ({ text, position: { kind: "time", sec, snapshotId: "s" } });

describe("faces wording", () => {
  it("says where each copy came from", () => {
    expect(sourceText("asr", { by: "platform", lang: "zh" })).toBe("平台提供的字幕");
    expect(sourceText("asr", { by: "machine", tool: "SenseVoice", at: "2026-11-10T12:00:00.000Z" })).toBe("自动转写 · SenseVoice · 11月10日");
    expect(sourceText("frames", { by: "machine", tool: "Vision" })).toBe("自动识别 · Vision");
  });

  it("tells two copies by the same tool apart, down to the minute on the same day", () => {
    const copy = (runId: string, when: string): CopyFacet => ({ layer: "ocr", key: `ocr:${runId}`, source: { by: "machine", tool: "Vision", runId, at: when }, segments: [] });
    const a = copy("a", new Date(2026, 10, 12, 9, 5).toISOString());
    const b = copy("b", new Date(2026, 10, 12, 10, 30).toISOString());
    const c = copy("c", new Date(2026, 10, 10, 8, 0).toISOString());
    expect(copyLabel(a, [a, b])).toBe("Vision · 11月12日 09:05");
    expect(copyLabel(b, [a, b])).toBe("Vision · 11月12日 10:30");
    expect(copyLabel(c, [a, c])).toBe("Vision · 11月10日");
    expect(copyLabel(a, [a])).toBe("Vision");
  });

  it("names the daemon's hash-keyed images by their place in the item", () => {
    expect(imageName("b8ee896171b6e386", ["b8ee896171b6e386", "c0ffee"])).toBe("图 1");
    expect(imageName("c0ffee", ["b8ee896171b6e386", "c0ffee"])).toBe("图 2");
    expect(imageName("cover")).toBe("封面");
    expect(imageName("p3")).toBe("图 3");
  });

  it("writes one plain sentence per pending state, never a percentage", () => {
    expect(pendingText({ layer: "asr", state: "later" })).toBe("尚未转写。usuyo 会在后台转写。");
    expect(pendingText({ layer: "ocr", state: "on-power" })).toBe("尚未识别图中文字。接通电源后开始识别。");
    expect(pendingText({ layer: "frames", state: "now" })).toBe("正在识别画面中文字…");
    const model = { plugin: "builtin/asr-sherpa", name: "sense-voice-int8", size: 163_002_883, license: "x" };
    expect(pendingText({ layer: "asr", state: "needs-model", model })).toBe("转写需要先下载 sense-voice-int8 模型（163 MB），只需下载一次。");
    expect(pendingText({ layer: "asr", state: "needs-model", model }, true)).toBe("正在下载 sense-voice-int8 模型（163 MB）。下载完成后继续处理此条目。");
    expect(pendingText({ layer: "asr", state: "failed", hint: "音轨读不出来" })).toBe("无法转写：音轨读不出来");
  });

  it("writes sizes and how long the trash keeps things", () => {
    expect(sizeText(16_000_000)).toBe("16 MB");
    expect(sizeText(912_000_000)).toBe("912 MB");
    expect(sizeText(1_400_000_000)).toBe("1.4 GB");
    expect(keepText(30)).toBe("30 天内可以恢复。");
    expect(keepText(7)).toBe("7 天内可以恢复。");
    expect(keepText(null)).toBe("可随时恢复。");
    expect(keepText(undefined)).toBe("可以恢复。");
  });

  it("writes the same sentences in English, with singular and plural", () => {
    setLanguageChoice("en");
    expect(pendingText({ layer: "asr", state: "later" })).toBe("Not transcribed yet. usuyo transcribes it in the background.");
    expect(pendingText({ layer: "ocr", state: "failed" })).toBe("Couldn't recognize text in images: Unknown reason");
    const model = { plugin: "builtin/asr-sherpa", name: "sense-voice-int8", size: 163_002_883, license: "x" };
    expect(pendingText({ layer: "asr", state: "needs-model", model })).toBe("Transcription needs the sense-voice-int8 model (163 MB). It's downloaded only once.");
    expect(keepText(1)).toBe("Can be restored for 1 day.");
    expect(keepText(30)).toBe("Can be restored for 30 days.");
    expect(imageName("c0ffee", ["b8ee896171b6e386", "c0ffee"])).toBe("Image 2");
    expect(sourceText("asr", { by: "machine", tool: "SenseVoice", at: "2026-11-10T12:00:00.000Z" })).toBe("Auto-transcribed · SenseVoice · Nov 10");
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
