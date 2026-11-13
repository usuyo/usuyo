// 日期、时间、数字、大小、并列的几项，按当前语言写，用系统的 Intl。日期照各语言自己的写法（中文「11月13日」，不加空格）。
import { t } from ".";
import { currentLocale } from "./locale";

/** 「11月13日」「Nov 13」；不是今年的带上年份。 */
export function formatDay(iso: string, now = new Date()): string {
  const d = new Date(iso);
  return new Intl.DateTimeFormat(currentLocale(), {
    month: "short",
    day: "numeric",
    ...(d.getFullYear() === now.getFullYear() ? {} : { year: "numeric" }),
  }).format(d);
}

/** 小时：中文补成两位（`09:05`，和系统的时钟一样），英文不补（`9:05 AM`）。 */
function hour(): "2-digit" | "numeric" {
  return currentLocale() === "zh-Hans" ? "2-digit" : "numeric";
}

/** 时分（`14:05`、`2:05 PM`），要秒时带秒。 */
export function formatClockTime(iso: string, seconds = false): string {
  return new Intl.DateTimeFormat(currentLocale(), { hour: hour(), minute: "2-digit", ...(seconds ? { second: "2-digit" } : {}) }).format(new Date(iso));
}

/** 日期和时分：「11月13日 14:05」「Nov 13, 2:05 PM」。 */
export function formatDayTime(iso: string, now = new Date()): string {
  const d = new Date(iso);
  return new Intl.DateTimeFormat(currentLocale(), {
    month: "short",
    day: "numeric",
    hour: hour(),
    minute: "2-digit",
    ...(d.getFullYear() === now.getFullYear() ? {} : { year: "numeric" }),
  }).format(d);
}

export function formatNumber(n: number): string {
  return new Intl.NumberFormat(currentLocale()).format(n);
}

const UNITS = ["KB", "MB", "GB", "TB"];

/** 文件大小，十进制，和访达一样：`163 MB`、`1.2 GB`；不到 1 KB 写字节。 */
export function formatBytes(bytes: number): string {
  if (bytes < 1000) return t("common.bytes", { count: bytes });
  let n = bytes / 1000;
  let unit = 0;
  while (n >= 1000 && unit < UNITS.length - 1) {
    n /= 1000;
    unit++;
  }
  const digits = n < 10 ? 1 : 0;
  return `${new Intl.NumberFormat(currentLocale(), { maximumFractionDigits: digits }).format(n)} ${UNITS[unit] ?? ""}`;
}

/**
 * 并列的几项：「A、B和C」「A, B, and C」；几个短句并列用 `narrow`：「A、B、C」「A, B, C」。
 * 中文里「和」「或」挨着数字或西文时照 §0.10 留一个空格（「7 个主题页和 1 篇文稿」）。
 */
export function formatList(items: readonly string[], type: "conjunction" | "disjunction" = "conjunction", style: "long" | "narrow" = "long"): string {
  const locale = currentLocale();
  const parts = new Intl.ListFormat(locale, { type, style }).formatToParts(items);
  if (locale !== "zh-Hans") return parts.map((p) => p.value).join("");
  const latin = /[0-9A-Za-z]/;
  return parts
    .map((p, i) => {
      if (p.type !== "literal" || p.value.trim() === "" || /^[\u3001\uFF0C]$/.test(p.value)) return p.value;
      const before = parts[i - 1]?.value.at(-1) ?? "";
      const after = parts[i + 1]?.value.at(0) ?? "";
      return `${latin.test(before) ? " " : ""}${p.value}${latin.test(after) ? " " : ""}`;
    })
    .join("");
}
