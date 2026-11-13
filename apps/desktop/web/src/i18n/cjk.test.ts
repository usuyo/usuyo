/// <reference types="node" />
// 界面上的话都从语言文件里取：语言文件和测试以外的源码里，字符串、模板和 JSX 文字里不出现中日韩文字。注释不算。
import { readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

const SRC = join(import.meta.dirname, "..");
const CJK = /[　-〿㐀-䶿一-鿿＀-￯]/;

/** 语言文件、测试和样例库（样例内容是平台上的原文，不是界面上的话）不查。 */
function skipped(rel: string): boolean {
  return rel.startsWith("i18n/zh-Hans/") || rel.startsWith("i18n/en/") || rel.startsWith("mock/") || /\.test\.tsx?$/.test(rel) || rel === "test-setup.ts";
}

function files(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    const path = join(dir, e.name);
    if (e.isDirectory()) return files(path);
    return /\.tsx?$/.test(e.name) ? [path] : [];
  });
}

/** 去掉注释，留下字符串、模板、正则和代码；JSX 文字留在代码里。只认得这个仓库写得出的写法。 */
export function withoutComments(source: string): string {
  let out = "";
  let i = 0;
  // 上一个有意义的字符：决定 `/` 是除号还是正则的开头。
  let last = "";
  while (i < source.length) {
    const c = source[i] ?? "";
    const next = source[i + 1] ?? "";
    if (c === "/" && next === "/") {
      while (i < source.length && source[i] !== "\n") i++;
      continue;
    }
    if (c === "/" && next === "*") {
      const end = source.indexOf("*/", i + 2);
      i = end === -1 ? source.length : end + 2;
      continue;
    }
    if (c === '"' || c === "'" || c === "`") {
      let j = i + 1;
      while (j < source.length && source[j] !== c) j += source[j] === "\\" ? 2 : 1;
      out += source.slice(i, j + 1);
      i = j + 1;
      last = c;
      continue;
    }
    if (c === "/" && (last === "" || "(,=:[!&|?{};+-*%<>~^".includes(last))) {
      let j = i + 1;
      let inClass = false;
      while (j < source.length && source[j] !== "\n" && (source[j] !== "/" || inClass)) {
        if (source[j] === "\\") j++;
        else if (source[j] === "[") inClass = true;
        else if (source[j] === "]") inClass = false;
        j++;
      }
      out += source.slice(i, j + 1);
      i = j + 1;
      last = "/";
      continue;
    }
    out += c;
    if (!/\s/.test(c)) last = c;
    i++;
  }
  return out;
}

describe("interface strings", () => {
  it("come from the language files: no CJK outside them, the tests and the sample library", () => {
    const found = files(SRC).flatMap((path) => {
      const rel = relative(SRC, path);
      if (skipped(rel)) return [];
      return withoutComments(readFileSync(path, "utf8"))
        .split("\n")
        .filter((line) => CJK.test(line))
        .map((line) => `${rel}: ${line.trim()}`);
    });
    expect(found).toEqual([]);
  });

  it("the scanner keeps strings and drops comments", () => {
    expect(withoutComments('const a = "字"; // 注释\n/* 注释 */ const b = 1 / 2; const r = /x\\/y/;')).toBe('const a = "字"; \n const b = 1 / 2; const r = /x\\/y/;');
  });
});
