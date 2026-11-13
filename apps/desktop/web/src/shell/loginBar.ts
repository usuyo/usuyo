// 登录窗口上面那一条（login-bar.html）。壳在页面加载前放好 `window.usuyoLogin`，
// 之后经 `window.usuyoSay` 往这里说一句（壳按语言写好）；「完成登录」交回壳。这一条不碰 cookie。
// 语言：壳在 `lang` 里给它按设置定好的语言（这个网页窗口看不到主窗口记下的选择）；不记在这台电脑上。
import { invoke } from "@tauri-apps/api/core";
import { currentLocale, t } from "../i18n";
import { useLocaleStore } from "../i18n/locale";
import "../styles.css";

interface BarSetup {
  mode: "auto" | "manual" | "visitor";
  name: string;
  lang?: string;
}

declare global {
  interface Window {
    usuyoLogin?: BarSetup;
    usuyoSay?: (text: string) => void;
  }
}

function barText(setup: BarSetup): string {
  return t(setup.mode === "visitor" ? "shell.visitor" : setup.mode === "manual" ? "shell.manual" : "shell.auto", { site: setup.name });
}

const setup = window.usuyoLogin;
const say = document.getElementById("say");
const done = document.getElementById("done");
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
}
