// 发布版在 Windows 上不弹控制台窗口。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod daemon;
mod login;
mod site_cookies;

use std::sync::Arc;

use tauri::{Manager, RunEvent};

use crate::daemon::{Connection, Daemon};
use crate::login::Logins;

/// 前端经 `src/shell/` 要内核的地址和令牌；内核还没起来时等它。
/// 前端连不上或令牌不对时再要一次，壳重新核对、必要时重新找（内核重启过就换了令牌）。
#[tauri::command]
async fn daemon_connection(daemon: tauri::State<'_, Arc<Daemon>>) -> Result<Connection, String> {
    let daemon = Arc::clone(&daemon);
    tauri::async_runtime::spawn_blocking(move || daemon.connection())
        .await
        .map_err(|e| e.to_string())?
}

/// 细节进日志，给前端一句话。
fn plain(what: &str, e: &anyhow::Error) -> String {
    eprintln!("usuyo: {what}: {e:#}");
    format!("没能{what}")
}

/// 开一个站点的登录窗口。前端只给站点名，登录页和要等的 cookie 由壳自己问内核。
#[tauri::command]
async fn open_login(
    app: tauri::AppHandle,
    logins: tauri::State<'_, Arc<Logins>>,
    site: String,
) -> Result<(), String> {
    let logins = Arc::clone(&logins);
    tauri::async_runtime::spawn_blocking(move || logins.open_login(&app, &site))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| plain("打开登录窗口", &e))
}

/// 开取访客身份的窗口（不登录）。
#[tauri::command]
async fn open_visitor(
    app: tauri::AppHandle,
    logins: tauri::State<'_, Arc<Logins>>,
    plugin: String,
) -> Result<(), String> {
    let logins = Arc::clone(&logins);
    tauri::async_runtime::spawn_blocking(move || logins.open_visitor(&app, &plugin))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| plain("打开访客身份的窗口", &e))
}

/// 登录窗口上面那条里的「登录好了」。只认那条自己的页面。
#[tauri::command]
fn login_done(webview: tauri::Webview, logins: tauri::State<'_, Arc<Logins>>) {
    logins.pressed(webview.label());
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
        .map_err(|e| plain("清掉这个站点的网页数据", &e))
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
            clear_login
        ])
        .setup(|app| {
            let plugins = app.path().resource_dir()?.join("plugins");
            let daemon = Arc::new(Daemon::new(plugins)?);
            let logins = Arc::new(Logins::new(Arc::clone(&daemon)));
            app.manage(Arc::clone(&daemon));
            app.manage(Arc::clone(&logins));
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
