// 界面上的话从这里取：`t("settings.window.title")`、`t("trash.kept", { count: 3 })`；
// 一句里有链接、按键这类元素的用 `rich`。语言换了，RootLayout 重新挂上整页，所以这里直接读当前语言。
import { cloneElement, isValidElement, type ReactElement, type ReactNode } from "react";
import en from "./en";
import { currentLocale, type Locale } from "./locale";
import type { KeyOf, Leaf, LeafAt, Plural, TagsOf, Tree, VarsArg, VarsOf } from "./messages";
import zhHans from "./zh-Hans";

export type Messages = typeof zhHans;
export type MessageKey = KeyOf<Messages>;

const CATALOGS: Record<Locale, Tree> = { "zh-Hans": zhHans, en };

export { currentLocale, setLanguageChoice, useLocale, type LanguageChoice, type Locale } from "./locale";

type Os = "mac" | "windows" | "linux";

function os(): Os {
  const ua = typeof navigator === "undefined" ? "" : navigator.userAgent;
  return /Mac/.test(ua) ? "mac" : /Windows/.test(ua) ? "windows" : "linux";
}

/** 平台各有叫法的词，句子里写 `{trash}`、`{fileManager}`，这里按系统填。 */
function ambient(locale: Locale): Record<string, string> {
  const c = CATALOGS[locale];
  const word = (key: string) => {
    const v = lookup(c, `common.os.${os()}.${key}`);
    return typeof v === "string" ? v : key;
  };
  return { trash: word("trash"), fileManager: word("fileManager") };
}

function lookup(tree: Tree, key: string): Leaf | Tree | undefined {
  let node: Leaf | Tree | undefined = tree;
  for (const part of key.split(".")) {
    if (node === undefined || typeof node === "string" || isPlural(node)) return undefined;
    node = node[part];
  }
  return node;
}

const PLURAL_FORMS = new Set(["zero", "one", "two", "few", "many", "other"]);

function isPlural(v: Leaf | Tree): v is Plural {
  return typeof v === "object" && typeof v.other === "string" && Object.keys(v).every((k) => PLURAL_FORMS.has(k));
}

/** 取一句话的原文：单复数照 `count` 选。键对不上是写错了，测试会抓到；这里照样显示键，不让页面空掉。 */
function source(locale: Locale, key: string, count: unknown): string {
  const v = lookup(CATALOGS[locale], key);
  if (v === undefined) return key;
  if (typeof v === "string") return v;
  if (isPlural(v)) {
    const form = typeof count === "number" ? new Intl.PluralRules(locale).select(count) : "other";
    return v[form] ?? v.other;
  }
  return key;
}

function show(locale: Locale, value: unknown): string {
  return typeof value === "number" ? new Intl.NumberFormat(locale).format(value) : String(value);
}

const HAN = /[\u3400-\u4dbf\u4e00-\u9fff]/;
const LATIN = /[0-9A-Za-z]/;

/**
 * 中文句子里填进来的西文或数字挨着汉字时，照 §0.10 两边留一个空格（「由 X 插件获取」）。
 * 自己带汉字的值不加（「B站」、按系统格式写的「12月3日」）。
 */
function spaced(text: string, value: string, at: number, end: number): string {
  if (value === "" || HAN.test(value)) return value;
  const before = HAN.test(text[at - 1] ?? "") && LATIN.test(value[0] ?? "") ? " " : "";
  const after = HAN.test(text[end] ?? "") && LATIN.test(value.at(-1) ?? "") ? " " : "";
  return `${before}${value}${after}`;
}

function fill(locale: Locale, text: string, vars: Record<string, unknown>): string {
  return text.replace(/\{(\w+)\}/g, (whole: string, name: string, at: number) => {
    if (!(name in vars)) return whole;
    const value = show(locale, vars[name]);
    return locale === "zh-Hans" ? spaced(text, value, at, at + whole.length) : value;
  });
}

type Args<K extends MessageKey> = [Exclude<VarsOf<LeafAt<Messages, K>>, "trash" | "fileManager">] extends [never] ? [] : [vars: VarsArg<LeafAt<Messages, K>>];

/** 一句话，变量填好。数字按当前语言写（`12,345`）；日期、大小先用 `format.ts` 写成字再传进来。 */
export function t<K extends MessageKey>(key: K, ...args: Args<K>): string {
  const locale = currentLocale();
  const vars: Record<string, unknown> = { ...ambient(locale), ...(args[0] ?? {}) };
  return fill(locale, source(locale, key, vars.count), vars);
}

/** 一段换成的元素：没有自己的内容时，这一段的字放进去当内容。 */
type Tag = ReactElement<{ children?: ReactNode }>;
type RichArgs<K extends MessageKey> = VarsArg<LeafAt<Messages, K>> & { [T in TagsOf<LeafAt<Messages, K>>]: Tag };

/** 一句话里有界面元素：`rich("home.empty", { link: <Link to="/plugins" /> })`，`<link>` 里的字成了链接的字。变量只能是字，不拼元素。 */
export function rich<K extends MessageKey>(key: K, parts: RichArgs<K>): ReactNode {
  const locale = currentLocale();
  const given: Record<string, string | number | Tag> = parts;
  const vars: Record<string, unknown> = { ...ambient(locale), ...given };
  const text = fill(locale, source(locale, key, vars.count), vars);
  const out: ReactNode[] = [];
  let last = 0;
  for (const m of text.matchAll(/<(\w+)>(.*?)<\/\1>/g)) {
    const at = m.index;
    if (at > last) out.push(text.slice(last, at));
    const el = given[m[1] ?? ""];
    const inner = m[2] ?? "";
    out.push(isValidElement<{ children?: ReactNode }>(el) ? cloneElement(el, { key: at }, el.props.children ?? inner) : inner);
    last = at + m[0].length;
  }
  if (last < text.length) out.push(text.slice(last));
  return out;
}

/** 测试用：某种语言里所有的键，和每一句的原文。 */
export function leaves(locale: Locale): Map<string, Leaf> {
  const out = new Map<string, Leaf>();
  const walk = (node: Tree, prefix: string) => {
    for (const [k, v] of Object.entries(node)) {
      if (typeof v === "string" || isPlural(v)) out.set(`${prefix}${k}`, v);
      else walk(v, `${prefix}${k}.`);
    }
  };
  walk(CATALOGS[locale], "");
  return out;
}
