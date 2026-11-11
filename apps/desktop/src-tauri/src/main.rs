// 发布版在 Windows 上不弹控制台窗口。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            // TODO(sidecar): 在这里把 `usuyo daemon` 作为 sidecar 启动，
            // 等它的本机端口可达后再放行前端；窗口关闭不停止它。尚未实现。

            // TODO(login-window): 在这里按需开第二个 webview 窗口，
            // 指向某个平台的登录页，供获取插件取登录状态。尚未实现。

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run the usuyo desktop shell");
}
