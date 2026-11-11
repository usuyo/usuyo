// 发布版在 Windows 上不弹控制台窗口。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod daemon;

use std::sync::Arc;

use tauri::{Manager, RunEvent};

use crate::daemon::{Connection, Daemon};

/// 前端经 `src/shell/` 要内核的地址和令牌；内核还没起来时等它。
/// 前端连不上或令牌不对时再要一次，壳重新核对、必要时重新找（内核重启过就换了令牌）。
#[tauri::command]
async fn daemon_connection(daemon: tauri::State<'_, Arc<Daemon>>) -> Result<Connection, String> {
    let daemon = Arc::clone(&daemon);
    tauri::async_runtime::spawn_blocking(move || daemon.connection())
        .await
        .map_err(|e| e.to_string())?
}

fn main() {
    let app = tauri::Builder::default()
        // 「打开原帖」经它交给系统浏览器；能开哪些网址由 capabilities/default.json 管（只有 http、https）。
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![daemon_connection])
        .setup(|app| {
            let plugins = app.path().resource_dir()?.join("plugins");
            let daemon = Arc::new(Daemon::new(plugins)?);
            app.manage(Arc::clone(&daemon));
            // 找已经在跑的内核，没有就起一个；窗口先开着，前端等 `daemon_connection`。
            std::thread::spawn(move || daemon.start());

            // TODO(login-window): 在这里按需开第二个 webview 窗口，
            // 指向某个平台的登录页，供获取插件取登录状态。尚未实现。
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
