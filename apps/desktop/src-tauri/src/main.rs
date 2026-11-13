// 发布版在 Windows 上不弹控制台窗口。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod daemon;
mod login;
mod site_cookies;
mod window;
mod words;

use std::sync::Arc;

use tauri::{Manager, RunEvent};

use crate::daemon::{Connection, Daemon};
use crate::login::Logins;
use crate::window::Chrome;
use crate::words::{Lang, Words};

/// 前端经 `src/shell/` 要内核的地址和令牌；内核还没起来时等它。
/// 前端连不上或令牌不对时再要一次，壳重新核对、必要时重新找（内核重启过就换了令牌）。
#[tauri::command]
async fn daemon_connection(daemon: tauri::State<'_, Arc<Daemon>>) -> Result<Connection, String> {
    let daemon = Arc::clone(&daemon);
    tauri::async_runtime::spawn_blocking(move || daemon.connection())
        .await
        .map_err(|e| e.to_string())?
}

/// 这个窗口开的时候去掉了系统标题栏没有（只在 macOS 上会去掉）。前端照它留出顶上那一条；
/// 设置里改的是下一次开窗口时的样子，所以不看设置。
#[tauri::command]
fn title_bar_hidden(chrome: tauri::State<'_, Chrome>) -> bool {
    chrome.hidden_title_bar
}

/// 细节进日志，给前端一句话（照设置的语言）。
fn plain(app: &tauri::AppHandle, what: Words, e: &anyhow::Error) -> String {
    eprintln!("usuyo: {}: {e:#}", what.pick(Lang::En));
    what.pick(lang(app)).to_owned()
}

fn lang(app: &tauri::AppHandle) -> Lang {
    Lang::of(app.state::<Arc<Daemon>>().machine_dir())
}

/// 开一个站点的登录窗口。前端只给站点名，登录页和要等的 cookie 由壳自己问内核。
#[tauri::command]
async fn open_login(
    app: tauri::AppHandle,
    logins: tauri::State<'_, Arc<Logins>>,
    site: String,
) -> Result<(), String> {
    let logins = Arc::clone(&logins);
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || logins.open_login(&app, &site))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| plain(&handle, words::CANT_OPEN_LOGIN, &e))
}

/// 开取访客身份的窗口（不登录）。前端只给站点名，打开哪一页、等哪些 cookie 由壳自己问内核。
/// `replace`（换一份）：先关掉开着的窗口、清掉它那份网页数据，免得页面还认得上一份。
#[tauri::command]
async fn open_visitor(
    app: tauri::AppHandle,
    logins: tauri::State<'_, Arc<Logins>>,
    site: String,
    replace: bool,
) -> Result<(), String> {
    let logins = Arc::clone(&logins);
    if replace {
        logins
            .clear(&app, &login::visitor_store(&site))
            .await
            .map_err(|e| plain(&app, words::CANT_CLEAR_OLD_VISITOR, &e))?;
    }
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || logins.open_visitor(&app, &site))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| plain(&handle, words::CANT_OPEN_VISITOR, &e))
}

/// 登录窗口上面那条里的「登录好了」。只认那条自己的页面。
#[tauri::command]
fn login_done(webview: tauri::Webview, logins: tauri::State<'_, Arc<Logins>>) {
    logins.pressed(webview.label());
}

/// 去掉访客身份时，先清掉它那个窗口的网页数据（再由前端去内核删），免得再取时页面还认得上一份。
#[tauri::command]
async fn clear_visitor(
    app: tauri::AppHandle,
    logins: tauri::State<'_, Arc<Logins>>,
    site: String,
) -> Result<(), String> {
    let logins = Arc::clone(&logins);
    logins
        .clear(&app, &login::visitor_store(&site))
        .await
        .map_err(|e| plain(&app, words::CANT_CLEAR_VISITOR, &e))
}

/// 退出登录以后清掉这个站点的网页数据。
#[tauri::command]
async fn clear_login(
    app: tauri::AppHandle,
    logins: tauri::State<'_, Arc<Logins>>,
    site: String,
) -> Result<(), String> {
    let logins = Arc::clone(&logins);
    logins
        .clear(&app, &site)
        .await
        .map_err(|e| plain(&app, words::CANT_CLEAR_LOGIN, &e))
}

/// `open_settings_pane` 能打开的，只有这三样：系统设置里屏幕录制、辅助功能那一栏，和 iPhone 镜像这个 App。
/// 守护进程的 `POST /phone/open-settings` 用同一张表（`crates/usuyo-core/src/phone/mod.rs`）。
fn settings_target(pane: &str) -> Option<&'static str> {
    match pane {
        "screen-recording" => {
            Some("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
        }
        "accessibility" => {
            Some("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        }
        "iphone-mirroring" => Some("/System/Applications/iPhone Mirroring.app"),
        _ => None,
    }
}

/// 打开系统设置的那一栏（或 iPhone 镜像），给授权的那一下由你自己点；壳不改任何设置。
#[tauri::command]
fn open_settings_pane(app: tauri::AppHandle, pane: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let target = settings_target(&pane)
        .ok_or_else(|| words::NO_SUCH_PANE.fill(lang(&app), &[("pane", &pane)]))?;
    let opened = if target.starts_with('/') {
        app.opener().open_path(target, None::<&str>)
    } else {
        app.opener().open_url(target, None::<&str>)
    };
    opened.map_err(|e| plain(&app, words::CANT_OPEN_SETTINGS, &anyhow::Error::new(e)))
}

fn main() {
    let app = tauri::Builder::default()
        // 「打开原帖」经它交给系统浏览器；能开哪些网址由 capabilities/default.json 管（只有 http、https）。
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            daemon_connection,
            open_login,
            open_visitor,
            login_done,
            clear_login,
            clear_visitor,
            open_settings_pane,
            title_bar_hidden
        ])
        .setup(|app| {
            let plugins = app.path().resource_dir()?.join("plugins");
            let daemon = Arc::new(Daemon::new(plugins)?);
            let logins = Arc::new(Logins::new(Arc::clone(&daemon)));
            app.manage(Arc::clone(&daemon));
            app.manage(Arc::clone(&logins));
            // 设置写坏了也要开得了窗口：照默认（不用系统标题栏）开，原因进日志。
            let system_title_bar = window::system_title_bar(daemon.machine_dir()).unwrap_or_else(|e| {
                eprintln!("usuyo: window.systemTitleBar not read, opening without the system title bar: {e:#}");
                false
            });
            app.manage(window::open_main(app, system_title_bar)?);
            // 调试版，对着本机自己起的测试页试登录窗口：`USUYO_OPEN_LOGIN=<站点>` 一启动就开那个站点的登录窗口，
            // `USUYO_CLEAR_LOGIN=<站点>` 一启动就清掉那个站点的网页数据（同「退出登录」的后一半）。
            #[cfg(debug_assertions)]
            {
                if let Ok(site) = std::env::var("USUYO_OPEN_LOGIN") {
                    let (handle, logins) = (app.handle().clone(), Arc::clone(&logins));
                    std::thread::spawn(move || {
                        if let Err(e) = logins.open_login(&handle, &site) {
                            eprintln!("usuyo: cannot open the login window: {e:#}");
                        }
                    });
                }
                if let Ok(site) = std::env::var("USUYO_CLEAR_LOGIN") {
                    let handle = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        match logins.clear(&handle, &site).await {
                            Ok(()) => eprintln!("usuyo: cleared the login data of {site}"),
                            Err(e) => eprintln!("usuyo: cannot clear the login data: {e:#}"),
                        }
                    });
                }
            }
            // 找已经在跑的内核，没有就起一个；窗口先开着，前端等 `daemon_connection`。
            std::thread::spawn(move || daemon.start());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build the usuyo desktop shell");
    app.run(|handle, event| {
        // 只停自己起的那个；本来就在跑的留着，命令行和 Agent 还在用。
        if let RunEvent::Exit = event {
            handle.state::<Arc<Daemon>>().stop();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::settings_target;

    #[test]
    fn only_the_three_panes_open() {
        assert!(
            settings_target("screen-recording")
                .unwrap()
                .ends_with("Privacy_ScreenCapture")
        );
        assert!(
            settings_target("accessibility")
                .unwrap()
                .ends_with("Privacy_Accessibility")
        );
        assert_eq!(
            settings_target("iphone-mirroring"),
            Some("/System/Applications/iPhone Mirroring.app")
        );
        assert_eq!(settings_target("general"), None);
        assert_eq!(
            settings_target("x-apple.systempreferences:com.apple.Users"),
            None
        );
    }
}
