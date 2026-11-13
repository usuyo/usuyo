// 语言文件的形状和类型检查。一句话是一个字符串，变量写 `{name}`，要换成界面元素的一段写 `<tag>…</tag>`；
// 英文要分单复数的写成 `{ one, other }`，按 `count` 选（Intl.PluralRules），中文一般只写 `other` 或直接写字符串。
// 英文的每一句和中文的同名，变量和标签一样：缺了键、多了或少了变量都过不了 tsc。

export interface Plural {
  zero?: string;
  one?: string;
  two?: string;
  few?: string;
  many?: string;
  other: string;
}

export type Leaf = string | Plural;

export interface Tree {
  readonly [key: string]: Leaf | Tree;
}

/** `{a}`、`{b}` → `"a" | "b"`。 */
type VarsIn<S extends string> = S extends `${string}{${infer P}}${infer Rest}` ? P | VarsIn<Rest> : never;

/** `<link>…</link>` → `"link"`。 */
type TagsIn<S extends string> = S extends `${string}<${infer T}>${infer Rest}` ? (T extends `/${string}` ? TagsIn<Rest> : T | TagsIn<Rest>) : never;

/** 是一句话：字符串，或只有单复数那几个键的对象（`{ article, other }` 这样的一组不算）。 */
export type IsLeaf<T> = T extends string ? true : T extends Plural ? ([Exclude<keyof T, keyof Plural>] extends [never] ? true : false) : false;

type Forms<L> = L extends string ? L : L extends Plural ? Extract<L[keyof L], string> : never;

/** 平台不同、界面自己填的词：`{trash}`（废纸篓、回收站）、`{fileManager}`（访达、文件资源管理器）。调用处不用给。 */
export type AmbientVar = "trash" | "fileManager";

export type VarsOf<L> = (L extends string ? never : "count") | VarsIn<Forms<L>>;
export type TagsOf<L> = TagsIn<Forms<L>>;

type Equal<A, B> = [A] extends [B] ? ([B] extends [A] ? true : false) : false;

/** 英文这一份的形状：和中文同样的键；一句话可以是字符串或单复数。 */
export type Mirror<Z> = { readonly [K in keyof Z]: IsLeaf<Z[K]> extends true ? Leaf : Mirror<Z[K]> };

/** 每一句的变量和标签都和中文一样；不一样的那一句变成一条说明，类型对不上。 */
export type Matches<Z, E> = {
  readonly [K in keyof Z]: K extends keyof E
    ? IsLeaf<Z[K]> extends true
      ? IsLeaf<E[K]> extends true
        ? Equal<VarsOf<Z[K]>, VarsOf<E[K]>> extends true
          ? Equal<TagsOf<Z[K]>, TagsOf<E[K]>> extends true
            ? E[K]
            : { placeholdersDiffer: { zhHans: TagsOf<Z[K]>; en: TagsOf<E[K]> } }
          : { placeholdersDiffer: { zhHans: VarsOf<Z[K]>; en: VarsOf<E[K]> } }
        : never
      : Matches<Z[K], E[K]>
    : never;
};

/** 写英文那一份：`export default mirror<typeof zh>()({ … })`。 */
export function mirror<Z>() {
  return <const E extends Mirror<Z>>(e: E & Matches<Z, E>): E => e;
}

/** `"settings.window.title"` 这样的键，只到一句话为止。 */
export type KeyOf<T, P extends string = ""> = {
  [K in keyof T & string]: IsLeaf<T[K]> extends true ? `${P}${K}` : KeyOf<T[K], `${P}${K}.`>;
}[keyof T & string];

export type LeafAt<T, K extends string> = K extends `${infer H}.${infer R}` ? (H extends keyof T ? LeafAt<T[H], R> : never) : K extends keyof T ? T[K] : never;

/** 一句话要的变量：`count` 是数，别的是字或数；平台的词由界面填。 */
export type VarsArg<L> = { [V in Exclude<VarsOf<L>, AmbientVar>]: V extends "count" ? number : string | number };
