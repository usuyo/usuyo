// 导出页的几句话：会写出什么、某一次做了什么。数字只说这一次。
import type { ExportRun, ExportView } from "../api/endpoints";

export function willText(will: ExportView["will"]): string {
  const parts = [
    will.items > 0 ? `${will.items} 篇条目` : "",
    will.topics > 0 ? `${will.topics} 篇主题页` : "",
    will.drafts > 0 ? `${will.drafts} 篇成稿` : "",
  ].filter((x) => x !== "");
  const head = parts.length > 0 ? `会写出 ${parts.join("、")}。` : "现在没有要写出的。";
  return will.privateLeft > 0 ? `${head}私密的 ${will.privateLeft} 条不导出。` : head;
}

/** 某一次导出的一句：写出几篇；读回、两边都改过、没碰的有就说。 */
export function runText(run: ExportRun): string {
  if (run.state === "failed") return run.hint ? `没导出：${run.hint}` : "没导出";
  return [
    `写出 ${run.written} 篇`,
    run.readBack.length > 0 ? `读回 ${run.readBack.length} 处` : "",
    run.both.length > 0 ? `两边都改过 ${run.both.length} 处` : "",
    run.left.length > 0 ? `没读也没写 ${run.left.length} 篇` : "",
  ]
    .filter((x) => x !== "")
    .join("，");
}
