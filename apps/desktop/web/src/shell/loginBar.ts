// 登录窗口上面那一条（login-bar.html）。壳在页面加载前放好 `window.usuyoLogin`，
// 之后经 `window.usuyoSay` 往这里说一句；「登录好了」交回壳。这一条不碰 cookie。
import { invoke } from "@tauri-apps/api/core";
import "../styles.css";

interface BarSetup {
  mode: "auto" | "manual" | "visitor";
  name: string;
}

declare global {
  interface Window {
    usuyoLogin?: BarSetup;
    usuyoSay?: (text: string) => void;
  }
}

function barText(setup: BarSetup): string {
  switch (setup.mode) {
    case "visitor":
      return `这里不用登录。页面自己打开以后会生成一个访客身份，取到就关上；它不是你的${setup.name}账号。`;
    case "manual":
      return `在下面登录${setup.name}。这个窗口只给${setup.name}用，和别的站点、你日常的浏览器分开。登录好了点右边。`;
    default:
      return `在下面登录${setup.name}。这个窗口只给${setup.name}用，和别的站点、你日常的浏览器分开。登录好了会自己关上。`;
  }
}

const setup = window.usuyoLogin;
const say = document.getElementById("say");
const done = document.getElementById("done");
if (setup && say && done) {
  say.textContent = barText(setup);
  done.hidden = setup.mode !== "manual";
  done.addEventListener("click", () => {
    void invoke("login_done");
  });
  window.usuyoSay = (text) => {
    say.textContent = text;
  };
}
