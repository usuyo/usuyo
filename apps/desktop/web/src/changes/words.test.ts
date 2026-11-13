import { expect, it } from "vitest";
import type { ChangeBatch, ChangeOp } from "../api/endpoints";
import { setLanguageChoice } from "../i18n";
import { actorText, byWho, dayHeading, opText, stateText } from "./words";

const op = (o: Partial<ChangeOp> & Pick<ChangeOp, "op">): ChangeOp => ({ id: "o", target: { kind: "item", id: "c7", title: "这周的豆子" }, ...o });

it("says what each change did in plain words", () => {
  expect(opText(op({ op: "add-item" }))).toBe("添加了「这周的豆子」");
  expect(opText(op({ op: "add-to-topic", after: "咖啡" }))).toBe("将「这周的豆子」添加到「咖啡」");
  expect(opText(op({ op: "edit-page", target: { kind: "topic", id: "coffee", title: "咖啡" } }))).toBe("编辑了「咖啡」中的一段");
  expect(opText(op({ op: "set-note" }))).toBe("编辑了「这周的豆子」的备注");
});

it("names who did it and where a batch stands", () => {
  expect(actorText({ kind: "agent", name: "Codex", device: "m" })).toBe("Codex");
  expect(actorText({ kind: "readback", target: "Obsidian", device: "m" })).toBe("从 Obsidian 读回");
  expect(stateText({ state: "proposed" })).toBe("建议的更改：接受后生效");
  expect(stateText({ state: "declined" })).toBe("已全部拒绝，资料库未更改");
});

it("heads each day as today, yesterday or a date", () => {
  const now = new Date("2026-11-12T10:00:00");
  expect(dayHeading("2026-11-12T01:00:00", now)).toBe("今天");
  expect(dayHeading("2026-11-11T23:00:00", now)).toBe("昨天");
  expect(dayHeading("2026-11-09T12:00:00", now)).toBe("11月9日");
});

it("says the same in English", () => {
  setLanguageChoice("en");
  expect(opText(op({ op: "add-to-topic", after: "Coffee" }))).toBe("Added “这周的豆子” to “Coffee”");
  expect(stateText({ state: "proposed" })).toBe("Suggested changes: take effect when accepted");
  expect(dayHeading("2026-11-09T12:00:00", new Date("2026-11-12T10:00:00"))).toBe("Nov 9");
});

it("filters by who did it", () => {
  const b = (by: ChangeBatch["by"]): ChangeBatch => ({ id: "b", at: "", by, state: "applied", via: "", ops: [] });
  expect(byWho(b({ kind: "sync", device: "d" }), "other")).toBe(true);
  expect(byWho(b({ kind: "you", device: "d" }), "agent")).toBe(false);
});
