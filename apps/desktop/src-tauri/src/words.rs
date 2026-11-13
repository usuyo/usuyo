//! 壳自己给界面的话：每一句的简体中文和英文并排写（SPEC §0.10）。语言照按机器的 `settings.json` 的 `language`，
//! 和内核读同一项（`crates/usuyo-core/src/settings.rs`）；`system`、没写、读不了都跟随系统。

use std::path::Path;

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    ZhHans,
    En,
}

impl Lang {
    /// 这台机器现在的语言。每次用到时现读，设置里换了马上跟着换。
    pub fn of(machine_root: &Path) -> Lang {
        let chosen = std::fs::read(machine_root.join("settings.json"))
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            .and_then(|v| v.get("language").and_then(Value::as_str).map(str::to_owned));
        match chosen.as_deref() {
            Some("zh-Hans") => Lang::ZhHans,
            Some("en") => Lang::En,
            _ => Lang::system(),
        }
    }

    fn system() -> Lang {
        match sys_locale::get_locale() {
            Some(t) if t.to_ascii_lowercase().starts_with("zh") => Lang::ZhHans,
            _ => Lang::En,
        }
    }

    pub fn tag(self) -> &'static str {
        match self {
            Lang::ZhHans => "zh-Hans",
            Lang::En => "en",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Words {
    zh: &'static str,
    en: &'static str,
}

const fn w(zh: &'static str, en: &'static str) -> Words {
    Words { zh, en }
}

impl Words {
    pub fn pick(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::ZhHans => self.zh,
            Lang::En => self.en,
        }
    }

    /// 填好 `{name}` 这样的变量。
    pub fn fill(&self, lang: Lang, vars: &[(&str, &dyn std::fmt::Display)]) -> String {
        let mut out = self.pick(lang).to_owned();
        for (name, value) in vars {
            out = out.replace(&format!("{{{name}}}"), &value.to_string());
        }
        out
    }
}

// daemon.rs：连不上 usuyo 服务
pub const DAEMON_LATE: Words = w(
    "usuyo 服务没有按时启动。重新打开 usuyo 后再试。",
    "The usuyo service didn't start in time. Reopen usuyo and try again.",
);
pub const DAEMON_UNREACHABLE: Words = w(
    "无法连接 usuyo 服务。",
    "Can't connect to the usuyo service.",
);
pub const DAEMON_OTHER_PORT: Words = w(
    "另一个 usuyo 服务正在端口 {port} 上运行，窗口只能连接端口 {expected}。",
    "Another usuyo service is running on port {port}. This window can only connect to port {expected}.",
);
pub const DAEMON_EXITED: Words = w(
    "usuyo 服务未能启动。详细信息在 {log}。",
    "The usuyo service couldn't start. Details are in {log}.",
);
pub const PORT_TAKEN: Words = w(
    "端口 {port} 已被其他程序占用，usuyo 服务未能启动。",
    "Port {port} is in use by another program, so the usuyo service couldn't start.",
);

// main.rs：前端调用壳失败时的一句
pub const CANT_OPEN_LOGIN: Words = w("无法打开登录窗口。", "Couldn't open the sign-in window.");
pub const CANT_CLEAR_OLD_VISITOR: Words = w(
    "无法清除上一份访客身份的网页数据。",
    "Couldn't clear the website data of the previous visitor identity.",
);
pub const CANT_OPEN_VISITOR: Words = w(
    "无法打开获取访客身份的窗口。",
    "Couldn't open the window for getting a visitor identity.",
);
pub const CANT_CLEAR_VISITOR: Words = w(
    "无法清除访客身份的网页数据。",
    "Couldn't clear the visitor identity's website data.",
);
pub const CANT_CLEAR_LOGIN: Words = w(
    "无法清除这个站点的网页数据。",
    "Couldn't clear this site's website data.",
);
pub const NO_SUCH_PANE: Words = w(
    "没有这个设置面板：{pane}",
    "There's no settings pane named {pane}.",
);
pub const CANT_OPEN_SETTINGS: Words = w("无法打开系统设置。", "Couldn't open System Settings.");

// login.rs：登录窗口的标题和上面那条里的话
pub const LOGIN_TITLE: Words = w("登录{site} · usuyo", "Sign In to {site} · usuyo");
pub const VISITOR_TITLE: Words = w(
    "获取{site}访客身份 · usuyo",
    "Get {site} Visitor Identity · usuyo",
);
pub const NOT_SIGNED_IN_YET: Words = w(
    "尚未检测到这个站点的登录。登录后再点按一次。",
    "No sign-in to this site was detected. Sign in, then click again.",
);
pub const SENDING_LOGIN: Words = w(
    "已检测到登录，正在发送到 usuyo…",
    "Signed in. Sending to usuyo…",
);
pub const SEND_FAILED: Words = w(
    "无法发送到 usuyo 服务。稍后会自动重试。",
    "Couldn't send to the usuyo service. It will try again shortly.",
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_follows_settings_json() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        std::fs::write(&file, r#"{"language": "en"}"#).unwrap();
        assert_eq!(Lang::of(dir.path()), Lang::En);
        std::fs::write(&file, r#"{"language": "zh-Hans"}"#).unwrap();
        assert_eq!(Lang::of(dir.path()), Lang::ZhHans);
        assert_eq!(
            PORT_TAKEN.fill(Lang::En, &[("port", &47615)]),
            "Port 47615 is in use by another program, so the usuyo service couldn't start."
        );
    }
}
