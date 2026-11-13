// 样例库里替内核和插件说的话：照内核一样两种语言并排写，按当前语言给。平台上的原文（标题、正文、评论）不翻译。
import { currentLocale } from "../i18n";

export function said(zhHans: string, en: string): string {
  return currentLocale() === "en" ? en : zhHans;
}
