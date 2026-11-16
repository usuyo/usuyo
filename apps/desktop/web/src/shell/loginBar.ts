// 登录窗口上面那一条（login-bar.html）。壳在页面加载前放好 `window.usuyoLogin`，
// 之后经 `window.usuyoSay` 往这里说一句（壳按语言写好）；「完成登录」交回壳。这一条不碰 cookie。
// 访客窗口里这一条把 `navigator.userAgent` 报给壳（`login_agent`），访客身份连同它交给 usuyo。
// 站点窗口（`mode: "reading"`）多一块面板：壳经 `window.usuyoPanel` 送来读到哪了和几个计数（只有数和固定的词），
// 按钮经 `reading_control` 交回壳。诊断信息是插件脚本给的回复结构，不含内容；「拷贝诊断信息」只拷到剪贴板。
// 语言：壳在 `lang` 里给它按设置定好的语言（这个网页窗口看不到主窗口记下的选择）；不记在这台电脑上。
import { invoke } from "@tauri-apps/api/core";
import { currentLocale, t } from "../i18n";
import { useLocaleStore } from "../i18n/locale";
import "../styles.css";

interface BarSetup {
  mode: "auto" | "manual" | "visitor" | "reading";
  name: string;
  lang?: string;
}

/** 壳送来的面板（src-tauri/src/reading.rs 的 `Session::view`）。 */
export interface PanelView {
  phase: "signing-in" | "reading" | "stopped" | "ended";
  reason: string | null;
  /** 已经交给 usuyo 的：落成几次导入、收下几条、几条不合插件的链接格式。 */
  handed: { imports: number; noted: number; dropped: number };
  /** 记下了、还没交的。 */
  unsent: number;
  /** 上一回没交成，壳过一会儿自己再交。 */
  failed: boolean;
  /** 读过了的列表；`reason` 不为空是跳过了。 */
  lists: { action: string; reason: string | null }[];
  auto: boolean;
  paused: boolean;
  requests: number;
  recognised: number;
  noted: { favorite: number; like: number; all: number };
  skipped: Record<string, number>;
  over: number;
  diagnosis: string;
}

declare global {
  interface Window {
    usuyoLogin?: BarSetup;
    usuyoSay?: (text: string) => void;
    usuyoPanel?: (view: PanelView) => void;
  }
}

function barText(setup: BarSetup): string {
  if (setup.mode === "reading") return t("shell.reading.signingIn", { site: setup.name });
  return t(setup.mode === "visitor" ? "shell.visitor" : setup.mode === "manual" ? "shell.manual" : "shell.auto", { site: setup.name });
}

const REASONS = ["end", "verify", "signin", "unknown", "errorCode", "quiet", "notab", "notYours"] as const;
const LIST_REASONS = ["notab", "quiet", "unknown"] as const;
type ListReason = (typeof LIST_REASONS)[number];
const isListReason = (w: string): w is ListReason => (LIST_REASONS as readonly string[]).includes(w);
const isList = (a: string): a is "favorite" | "like" => a === "favorite" || a === "like";
type Reason = (typeof REASONS)[number];
const isReason = (w: string): w is Reason => (REASONS as readonly string[]).includes(w);

/** 脚本给的停下的原因：认得的几个词照语言写，别的照原样。 */
export function reasonText(word: string | null): string {
  const w = word ?? "unknown";
  return isReason(w) ? t(`shell.reading.reason.${w}`) : t("shell.reading.reason.other", { word: w });
}

/** 面板第一行：读到哪了。 */
export function phaseText(v: PanelView, site: string): string {
  const ended = () => {
    if (v.failed) return t("shell.reading.failed");
    if (v.unsent > 0) return t("shell.reading.handing");
    const h = v.handed;
    const head = h.noted > 0 ? t("shell.reading.handed", { count: h.noted }) : t("shell.reading.handedNone");
    return h.dropped > 0 ? `${head} ${t("shell.reading.dropped", { count: h.dropped })}` : head;
  };
  const text: Record<PanelView["phase"], () => string> = {
    "signing-in": () => t("shell.reading.signingIn", { site }),
    reading: () => (v.paused ? t("shell.reading.paused") : v.auto ? t("shell.reading.auto") : t("shell.reading.manual")),
    stopped: () => t("shell.reading.stopped", { reason: reasonText(v.reason) }),
    ended,
  };
  return text[v.phase]();
}

/** 跳过了的列表：哪一个、为什么。 */
function skippedList(action: string, reason: string): string {
  const list = isList(action) ? t(`shell.reading.list.${action}`) : action;
  const why = isListReason(reason) ? t(`shell.reading.listReason.${reason}`) : t("shell.reading.reason.other", { word: reason });
  return t("shell.reading.listSkipped", { list, reason: why });
}

/** 面板第三行：略过了哪些、超过上限没记的。 */
export function detailText(v: PanelView): string {
  const kinds = Object.entries(v.skipped).filter(([, n]) => n > 0);
  const parts: string[] = [];
  if (kinds.length > 0) {
    const total = kinds.reduce((a, [, n]) => a + n, 0);
    const named = kinds
      .map(([kind, count]) => (kind === "noId" || kind === "noToken" ? t(`shell.reading.skipKind.${kind}`, { count }) : t("shell.reading.skipKind.other", { kind, count })))
      .join(", ");
    parts.push(t("shell.reading.skipped", { count: total, kinds: named }));
  }
  if (v.over > 0) parts.push(t("shell.reading.over", { count: v.over }));
  // 还在读、停着时已经交了的；读完以后第一行就说了
  if (v.phase !== "ended" && v.handed.noted > 0) parts.push(t("shell.reading.handedSoFar", { count: v.handed.noted }));
  if (v.phase !== "ended" && v.failed) parts.push(t("shell.reading.failed"));
  for (const l of v.lists) if (l.reason) parts.push(skippedList(l.action, l.reason));
  return parts.join(" ");
}

const control = (c: "auto" | "manual" | "pause" | "resume" | "finish") => {
  void invoke("reading_control", { control: c });
};

const byId = (id: string) => document.getElementById(id);

const setup = window.usuyoLogin;
const say = byId("say");
const done = byId("done");
if (setup?.lang === "zh-Hans" || setup?.lang === "en") useLocaleStore.setState({ choice: setup.lang, locale: setup.lang });
if (setup && say && done) {
  document.documentElement.lang = currentLocale();
  document.title = t("shell.title");
  say.textContent = barText(setup);
  done.textContent = t("shell.done");
  done.hidden = setup.mode !== "manual";
  done.addEventListener("click", () => {
    void invoke("login_done");
  });
  window.usuyoSay = (text) => {
    say.textContent = text;
  };
  // 访客身份随这个窗口的 UA 交给 usuyo（这一条和下面的网页是同一个引擎），插件请求时用同一个
  if (setup.mode === "visitor") void invoke("login_agent", { userAgent: navigator.userAgent });
  if (setup.mode === "reading") reading(setup.name, say);
}

function reading(site: string, line: HTMLElement) {
  const panel = byId("panel");
  const counts = byId("counts");
  const detail = byId("detail");
  const auto = byId("auto");
  const autoLabel = byId("auto-label");
  const pause = byId("pause");
  const finish = byId("finish");
  const copy = byId("copy");
  const copyNote = byId("copy-note");
  if (!panel || !counts || !detail || !autoLabel || !copyNote) return;
  if (!(auto instanceof HTMLInputElement) || !(pause instanceof HTMLButtonElement) || !(finish instanceof HTMLButtonElement) || !(copy instanceof HTMLButtonElement)) return;
  panel.hidden = false;
  autoLabel.textContent = t("shell.reading.autoScroll");
  finish.textContent = t("shell.reading.finish");
  copy.textContent = t("shell.reading.copy");
  copyNote.textContent = t("shell.reading.copyNote");
  auto.checked = true;
  let view: PanelView | null = null;
  let paused = false;
  pause.textContent = t("shell.reading.pause");

  auto.addEventListener("change", () => control(auto.checked ? "auto" : "manual"));
  pause.addEventListener("click", () => control(paused ? "resume" : "pause"));
  finish.addEventListener("click", () => control("finish"));
  copy.addEventListener("click", () => {
    const text = view?.diagnosis ?? "";
    navigator.clipboard.writeText(text).then(
      () => {
        copyNote.textContent = t("shell.reading.copied");
      },
      () => {
        copyNote.textContent = t("shell.reading.copyFailed");
      },
    );
  });

  window.usuyoPanel = (v) => {
    view = v;
    // 停下了也用「继续」：继续时壳从停下的地方接着读
    paused = v.paused || v.phase === "stopped";
    line.textContent = phaseText(v, site);
    counts.textContent = t("shell.reading.counts", {
      requests: v.requests,
      recognised: v.recognised,
      noted: v.noted.all,
      favorite: v.noted.favorite,
      like: v.noted.like,
    });
    detail.textContent = detailText(v);
    auto.checked = v.auto;
    pause.textContent = paused ? t("shell.reading.resume") : t("shell.reading.pause");
    const over = v.phase === "ended";
    pause.disabled = over || v.phase === "signing-in";
    finish.disabled = over;
    auto.disabled = over;
    copy.disabled = v.diagnosis === "";
  };
}
