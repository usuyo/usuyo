// 订阅的几句话：订阅的是什么、添加什么、多久一次、为什么暂停、试运行的结果。只说事实，不说还剩多少。
import type { FollowKind, FollowTake, FollowTrial, FollowView } from "../api/endpoints";
import { t } from "../i18n";
import { formatList } from "../i18n/format";
import { timeText } from "../plugins/words";

/** 订阅的那一种叫什么。 */
export function kindText(kind: FollowKind): string {
  return t(`follow.kind.${kind}`);
}

/** 几个词，各自加上引号再并列：「胶片」和「C41」。 */
export function quotedList(words: string[]): string {
  return formatList(words.map((text) => t("follow.quoted", { text })));
}

/** 添加什么，一句话。`topicName` 把主题 id 换成名字。 */
export function takeText(take: FollowTake, topicName: (id: string) => string): string {
  if (take.all) return t("follow.take.all");
  const parts = [...take.topics.map((id) => t("follow.take.topicWords", { topic: topicName(id) })), ...take.words.map((text) => t("follow.quoted", { text }))];
  return parts.length > 0 ? t("follow.take.matching", { list: formatList(parts) }) : t("follow.take.matchingNone");
}

/** 多久一次、下次什么时候；暂停的说为什么、到什么时候。 */
export function rhythmText(f: Pick<FollowView, "on" | "everyHours" | "nextAt" | "waiting">, now = new Date()): string {
  if (!f.on) return t("follow.rhythm.paused");
  if (f.waiting) {
    const why = f.waiting.hint ?? f.waiting.code;
    return f.waiting.until ? t("follow.rhythm.waitingUntil", { why, time: timeText(f.waiting.until, now) }) : t("follow.rhythm.waiting", { why });
  }
  return f.nextAt ? t("follow.rhythm.everyNext", { count: f.everyHours, time: timeText(f.nextAt, now) }) : t("follow.rhythm.every", { count: f.everyHours });
}

/** 试运行以后的那一句。 */
export function trialText(trial: FollowTrial): string {
  const parts = [
    ...(trial.left.stored > 0 ? [t("follow.trial.stored", { count: trial.left.stored })] : []),
    ...(trial.left.trashed > 0 ? [t("follow.trial.trashed", { count: trial.left.trashed })] : []),
    ...(trial.left.unmatched > 0 ? [t("follow.trial.unmatched", { count: trial.left.unmatched })] : []),
  ];
  const head = t("follow.trial.listed", { count: trial.listed });
  const listed = parts.length > 0 ? t("follow.trial.listedWith", { listed: head, parts: formatList(parts, "conjunction", "narrow") }) : t("follow.trial.listedPlain", { listed: head });
  return t("follow.trial.sentence", { listed, next: trial.firstRun === "record" ? t("follow.trial.record") : t("follow.trial.take") });
}

/** 能选的频率：插件声明的最快的那一档起。 */
export function hourChoices(minHours: number): { id: string; text: string }[] {
  return [6, 12, 24, 72]
    .filter((h) => h >= minHours)
    .map((h) => ({ id: String(h), text: h === 72 ? t("follow.hours.threeDays") : h === 24 ? t("follow.hours.daily") : t("follow.hours.every", { count: h }) }));
}
