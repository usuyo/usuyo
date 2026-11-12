import { expect, it } from "vitest";
import type { SearchHit } from "../api/endpoints";
import { editedText, kindText, quotableOf } from "./words";

it("names the kind of a draft and when it was last changed", () => {
  expect(kindText("script")).toBe("脚本");
  const now = new Date("2026-11-12T10:00:00");
  expect(editedText("2026-11-12T08:00:00", now)).toBe("今天改过");
  expect(editedText("2026-11-09T08:00:00", now)).toBe("11 月 9 日改过");
});

it("keeps only search hits that are a sentence of some layer", () => {
  const hit = (layer: SearchHit["layer"]): SearchHit => ({ itemId: "c1", title: "比例", platform: { id: "p", name: "某站" }, layer, snippet: `${layer} 的那句`, match: [0, 1] });
  const out = quotableOf([hit("title"), hit("asr"), hit("note"), hit("ocr"), hit("author")]);
  expect(out.map((s) => s.layer)).toEqual(["asr", "ocr"]);
  expect(out[0]).toMatchObject({ itemId: "c1", text: "asr 的那句" });
});
