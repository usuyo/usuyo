import { cleanup, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { leaves, rich, setLanguageChoice, t } from ".";
import { formatBytes, formatDay, formatList } from "./format";
import type { Leaf } from "./messages";

const forms = (l: Leaf): string[] => (typeof l === "string" ? [l] : Object.values(l).filter((v): v is string => typeof v === "string"));
const names = (l: Leaf): string[] => [...new Set(forms(l).flatMap((s) => [...s.matchAll(/\{(\w+)\}|<(\w+)>/g)].map((m) => m[1] ?? m[2] ?? "")))].sort();

describe("language files", () => {
  it("have the same keys in both languages", () => {
    expect([...leaves("en").keys()].sort()).toEqual([...leaves("zh-Hans").keys()].sort());
  });

  it("have the same placeholders and tags in each sentence", () => {
    const en = leaves("en");
    const differ = [...leaves("zh-Hans")].filter(([k, zh]) => {
      const e = en.get(k);
      return e === undefined || names(zh).join() !== names(e).join();
    });
    expect(differ.map(([k]) => k)).toEqual([]);
  });

  it("write no CJK in English, except the language's own name and buttons quoted from Chinese-only script panels", () => {
    const cjk = /[\u3000-\u303f\u3400-\u4dbf\u4e00-\u9fff\uff00-\uffef]/;
    const quotedAsIs = (k: string) => k === "settings.general.zhHans" || k.startsWith("plugins.quoted.");
    expect([...leaves("en")].filter(([k, l]) => !quotedAsIs(k) && forms(l).some((s) => cjk.test(s))).map(([k]) => k)).toEqual([]);
  });
});

describe("sentences with numbers", () => {
  it("pick singular and plural in English and write numbers per language", () => {
    setLanguageChoice("en");
    expect(t("common.bytes", { count: 1 })).toBe("1 byte");
    expect(t("common.bytes", { count: 512 })).toBe("512 bytes");
    expect(formatBytes(163_000_000)).toBe("163 MB");
    expect(formatBytes(1_234_000_000)).toBe("1.2 GB");
    expect(formatList(["A", "B", "C"])).toBe("A, B, and C");
    expect(formatDay("2026-11-13T08:00:00.000Z", new Date("2026-11-14T00:00:00.000Z"))).toBe("Nov 13");
    setLanguageChoice("zh-Hans");
    expect(t("common.bytes", { count: 1 })).toBe("1 字节");
    expect(t("common.bytes", { count: 12345 })).toBe("12,345 字节");
    expect(formatList(["咖啡", "胶片", "城市"])).toBe("咖啡、胶片和城市");
    expect(formatList(["咖啡", "胶片", "城市"], "conjunction", "narrow")).toBe("咖啡、胶片、城市");
    expect(formatList(["7 个主题页", "1 篇文稿"])).toBe("7 个主题页和 1 篇文稿");
    expect(formatDay("2026-11-13T08:00:00.000Z", new Date("2026-11-14T00:00:00.000Z"))).toBe("11月13日");
    expect(formatDay("2025-11-13T08:00:00.000Z", new Date("2026-11-14T00:00:00.000Z"))).toBe("2025年11月13日");
  });

  it("put a space between Chinese and a Latin or numeric value filled in, and none in English", () => {
    setLanguageChoice("zh-Hans");
    expect(t("plugins.page.header", { count: 1, plugins: "X" })).toBe("由 X 插件获取。不需要账号就能获取的内容，不会使用账号。");
    expect(t("plugins.page.header", { count: 1, plugins: "B站" })).toBe("由B站插件获取。不需要账号就能获取的内容，不会使用账号。");
    setLanguageChoice("en");
    expect(t("plugins.page.header", { count: 1, plugins: "X" }).startsWith("Fetched by the X plugin.")).toBe(true);
  });

  it("fill the platform's word for the trash", () => {
    setLanguageChoice("en");
    expect(t("common.os.mac.trash")).toBe("Trash");
    setLanguageChoice("zh-Hans");
    expect(t("common.os.mac.trash")).toBe("废纸篓");
  });

  it("split a sentence with a tag into text and the element", () => {
    setLanguageChoice("en");
    render(<p>{rich("home.empty", { link: <a href="/plugins" /> })}</p>);
    const link = screen.getByRole("link");
    expect(link.getAttribute("href")).toBe("/plugins");
    expect(link.textContent).toBe(t("frame.nav.plugins"));
    cleanup();
  });
});
