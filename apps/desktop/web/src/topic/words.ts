// 主题页上的几句话：一次读的是什么时候、谁读的；按关键词命中的写命中了哪几个词。
import type { AngleRun } from "../api/endpoints";
import { dayText } from "../item/format";

/** 「11 月 8 日读的」，带上模型时「Qwen3 8B · 本机，11 月 8 日读的」。 */
export function runText(run: Pick<AngleRun, "at" | "model">, withModel: boolean): string {
  return withModel ? `${run.model}，${dayText(run.at)}读的` : `${dayText(run.at)}读的`;
}

/** 「标题或内容里有『手冲』『水温』」。 */
export function hitText(words: string[]): string {
  return `标题或内容里有${words.map((w) => `「${w}」`).join("")}`;
}
