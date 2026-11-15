fn main() {
    // 站点窗口往壳交东西的那一条（src/channel.rs）。做成应用自带的插件，命令才进权限表：
    // 权限 `usuyo-window:allow-submit` 只在开窗口时按插件清单的站点给那一块网页，别的窗口、别的命令不受影响。
    tauri_build::try_build(tauri_build::Attributes::new().plugin(
        "usuyo-window",
        tauri_build::InlinedPlugin::new().commands(&["submit"]),
    ))
    .expect("failed to run the tauri build script");
}
