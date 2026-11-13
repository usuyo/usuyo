// 导出页的几句话：会写出什么、某一次做了什么。数字只说这一次。
import type { ExportRun, ExportView } from "../api/endpoints";
import { t } from "../i18n";
import { formatList } from "../i18n/format";

export function willText(will: ExportView["will"]): string {
  const parts = [
    ...(will.items > 0 ? [t("export.will.items", { count: will.items })] : []),
    ...(will.topics > 0 ? [t("export.will.topics", { count: will.topics })] : []),
    ...(will.drafts > 0 ? [t("export.will.drafts", { count: will.drafts })] : []),
  ];
  const head = parts.length > 0 ? t("export.will.line", { list: formatList(parts) }) : t("export.will.nothing");
  return will.privateLeft > 0 ? t("export.will.withPrivate", { will: head, privateLeft: t("export.will.privateLeft", { count: will.privateLeft }) }) : head;
}

/** 某一次导出的一句：写出几篇；读回、两边都改过、跳过的有就说，几项之间用语言自己的分句符号。 */
export function runText(run: ExportRun): string {
  if (run.state === "failed") return run.hint ? t("export.run.failed", { hint: run.hint }) : t("export.run.failedPlain");
  return [
    t("export.run.written", { count: run.written }),
    ...(run.readBack.length > 0 ? [t("export.run.readBack", { count: run.readBack.length })] : []),
    ...(run.both.length > 0 ? [t("export.run.both", { count: run.both.length })] : []),
    ...(run.left.length > 0 ? [t("export.run.left", { count: run.left.length })] : []),
  ].join(t("common.clauseSeparator"));
}
