// 跟着收的几句话：跟着的是什么、收什么、多久一次、为什么停着、试列一次的结果。只说事实，不说还剩多少。
import type { FollowKind, FollowTake, FollowTrial, FollowView } from "../api/endpoints";
import { timeText } from "../plugins/words";

export const KIND_TEXT: Record<FollowKind, string> = {
  list: "你的收藏夹和列表",
  creator: "一个创作者",
  feed: "一个订阅源",
  keywords: "关键词",
};

/** 收什么，一句话。`topicName` 把主题 id 换成名字。 */
export function takeText(take: FollowTake, topicName: (id: string) => string): string {
  if (take.all) return "新出现的都收";
  const parts = [...take.topics.map((t) => `「${topicName(t)}」的关键词`), ...take.words.map((w) => `「${w}」`)];
  return parts.length > 0 ? `只收命中${parts.join("、")}的` : "只收命中的（还没选词）";
}

/** 多久一次、下次什么时候；停着的说为什么、到什么时候。 */
export function rhythmText(f: Pick<FollowView, "on" | "everyHours" | "nextAt" | "waiting">, now = new Date()): string {
  if (!f.on) return "你停下了，不再去看";
  const every = `每 ${f.everyHours} 小时看一次`;
  if (f.waiting) {
    const why = f.waiting.hint ?? f.waiting.code;
    return f.waiting.until ? `${why}，只停这一个，${timeText(f.waiting.until, now)} 自己再试` : `${why}，只停这一个，等你动手`;
  }
  return f.nextAt ? `${every}，下次 ${timeText(f.nextAt, now)}` : every;
}

/** 试列一次以后的那一句。 */
export function trialText(t: FollowTrial): string {
  const left = [
    t.left.stored > 0 ? `已经存着 ${t.left.stored} 条` : "",
    t.left.trashed > 0 ? `在回收站里 ${t.left.trashed} 条` : "",
    t.left.unmatched > 0 ? `没命中 ${t.left.unmatched} 条` : "",
  ].filter((x) => x !== "");
  const listed = `这一次列到 ${t.listed} 条${left.length > 0 ? `，其中${left.join("、")}` : ""}。`;
  return t.firstRun === "record"
    ? `${listed}第一次只记下现在有的，不收；以后新出现的，照下面这样收。`
    : `${listed}新加进去的直接收下，不逐条看；下面是这一次会收下的。`;
}

/** 能选的节奏：插件声明的最快的那一档起。 */
export function hourChoices(minHours: number): { id: string; text: string }[] {
  return [6, 12, 24, 72].filter((h) => h >= minHours).map((h) => ({ id: String(h), text: h === 72 ? "三天一次" : h === 24 ? "每天一次" : `每 ${h} 小时` }));
}
