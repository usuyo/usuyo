// 主题页上的几句话：一次读的是什么时候、谁读的；按关键词命中的写命中了哪几个词。
import type { AngleRun } from "../api/endpoints";
import { t } from "../i18n";
import { formatList } from "../i18n/format";
import { dayText } from "../item/format";

/** 「11月8日阅读」，带上模型时「Qwen3 8B · 本机，11月8日阅读」。 */
export function runText(run: Pick<AngleRun, "at" | "model">, withModel: boolean): string {
  const day = dayText(run.at);
  return withModel ? t("topic.angles.runWithModel", { model: run.model, day }) : t("topic.angles.run", { day });
}

/** 「标题或内容中包含「手冲」和「水温」」。 */
export function hitText(words: string[]): string {
  return t("topic.matching.hit", { words: formatList(words.map((word) => t("topic.matching.word", { word }))) });
}
