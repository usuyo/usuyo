// 类型上的检查，tsc 每次都跑：英文缺了键、变量和中文对不上、调用时少给了变量，都是编译错误。
// 这里只放类型，不进打包。
import type { t } from ".";
import type { Matches, Mirror } from "./messages";

type Equal<A, B> = [A] extends [B] ? ([B] extends [A] ? true : false) : false;
type Expect<T extends true> = T;

// 中文那一份用拉丁字母写，这个文件不在语言文件里。
type Zh = { saved: "yi baocun {count} xiang"; nav: { title: "shezhi"; link: "zai <link>shezhi</link> zhong genggai" } };

/** 和中文一样的键和变量：每一句原样留着。 */
export type SameShapePasses = Expect<Equal<Matches<Zh, { saved: { one: "{count} item saved"; other: "{count} items saved" }; nav: { title: "Settings"; link: "Change it in <link>Settings</link>" } }>["saved"], { one: "{count} item saved"; other: "{count} items saved" }>>;

/** 少了一个键：不合英文那一份的形状。 */
export type MissingKeyFails = Expect<Equal<{ saved: string; nav: { title: string } } extends Mirror<Zh> ? true : false, false>>;

/** 少了变量：那一句变成一条说明，和写的字对不上。 */
export type MissingVarFails = Expect<Equal<Matches<Zh, { saved: "Items saved"; nav: { title: "Settings"; link: "Change it in <link>Settings</link>" } }>["saved"], { placeholdersDiffer: { zhHans: "count"; en: never } }>>;

/** 少了标签：同上。 */
export type MissingTagFails = Expect<Equal<Matches<Zh, { saved: "{count}"; nav: { title: "Settings"; link: "Change it in Settings" } }>["nav"]["link"], { placeholdersDiffer: { zhHans: "link"; en: never } }>>;

/** 调用时要的变量：没有变量的一句不收参数，有的必须给全。 */
export type CallNeedsVars = Expect<Equal<Parameters<typeof t<"common.bytes">>, ["common.bytes", { count: number }]>>;
export type CallWithoutVars = Expect<Equal<Parameters<typeof t<"common.cancel">>, ["common.cancel"]>>;
