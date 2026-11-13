//! `usuyo`：命令行和守护进程。
//!
//! 守护进程在跑时（`usuyo daemon`），`add`、`get`、`search`、`reindex` 经它的本机接口走，
//! 带 `<按机器的目录>/api-token` 里的令牌；不在跑就在本进程里打开资料库直接做。
//!
//! 插件从 `<按机器的目录>/plugins/<owner>/<repo>/<version>/` 找。开发时（debug 构建）
//! 先 `cargo build`，工作区里的 `plugins/web` 编出来的 `usuyo-plugin-web` 和本程序在同一个
//! 目录，会照安装的样子放进 `<按机器的目录>/dev-plugins/`；别的目录用 `--plugins` 或 `USUYO_PLUGINS` 加。
//! 按机器的目录默认是系统的（macOS `~/Library/Application Support/usuyo`），`USUYO_HOME` 可以换。
//!
//! 守护进程还看着收件文件夹 `<按机器的目录>/inbox/`：放进去的 `.txt`（一行一个链接，或整段分享文字）
//! 10 秒没变就收下里面的链接，文件移到 `.usuyo-done/<日期>/`，收不成的移到 `.usuyo-failed/`；别的文件不碰。
//! 登录站点的 cookie 和插件的密钥设置在系统钥匙串里（Linux 上是按机器目录里 0600 的 `secrets.json`）。

mod client;
mod machine;
mod redact;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::{Value, json};
use usuyo_core::{Engine, EngineConfig, Links, Via};

use crate::client::Client;

#[derive(Parser)]
#[command(
    name = "usuyo",
    version,
    about = "把各平台收下的内容存到自己电脑上，变成能搜的字"
)]
struct Cli {
    /// 资料库目录；不给就用 `usuyo init` 记下的那个。
    #[arg(long, global = true, env = "USUYO_LIBRARY")]
    library: Option<PathBuf>,
    /// 另外找插件的目录（可以给多个）。
    #[arg(
        long = "plugins",
        global = true,
        env = "USUYO_PLUGINS",
        value_delimiter = ','
    )]
    plugins: Vec<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 建一个新资料库，并记成当前的。
    Init { dir: PathBuf },
    /// 收下一条链接：马上回条目 id，再取内容。
    Add { url: String },
    /// 读一条（JSON）。
    Get { id: String },
    /// 搜正文、标题、作者、记一句；中文、拼音、拼音首字母都行。
    Search {
        #[arg(required = true)]
        query: Vec<String>,
    },
    /// 照资料库的文件重建索引。
    Reindex,
    /// 起本机接口（只绑 127.0.0.1）和后台取内容。收到 Ctrl-C 或 SIGTERM 时收尾退出。
    Daemon {
        #[arg(long, env = "USUYO_PORT", default_value_t = usuyo_api::DEFAULT_PORT)]
        port: u16,
        /// 还没有资料库时在这里建一个（桌面窗口第一次打开时用）；已经有了就不管它。
        #[arg(long, value_name = "DIR")]
        first_library: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let level = match cli.command {
        Command::Daemon { .. } => tracing_subscriber::filter::LevelFilter::INFO,
        _ => tracing_subscriber::filter::LevelFilter::WARN,
    };
    tracing_subscriber::fmt()
        .with_writer(redact::Redacted)
        .with_max_level(level)
        .init();
    let rt = tokio::runtime::Runtime::new().context("cannot start the async runtime")?;
    rt.block_on(run(cli))
}

async fn run(cli: Cli) -> Result<()> {
    let root = machine::root()?;
    if let Command::Init { dir } = &cli.command {
        let path = Engine::init_library(dir, &root)?;
        machine::remember_library(&root, &path)?;
        println!("建好了资料库：{}", path.display());
        return Ok(());
    }
    if !matches!(cli.command, Command::Daemon { .. })
        && let Some(client) = Client::find(&root)?
    {
        return through_daemon(&client, cli.command);
    }
    if let Command::Daemon {
        first_library: Some(dir),
        ..
    } = &cli.command
        && cli.library.is_none()
        && !machine::has_library(&root)?
    {
        let path = machine::first_library(dir, &root)?;
        tracing::info!(library = %path.display(), "created the first library");
    }
    let engine = open(&root, &cli)?;
    match cli.command {
        Command::Init { .. } => Ok(()),
        Command::Add { url } => {
            let taken = engine.take(&url, "link", Via::Cli).await?;
            println!("{}", taken.id);
            if taken.seen {
                println!("以前收下过，这次再取一次");
            }
            engine.run_due().await?;
            let item = engine.item(&taken.id, &Links::default())?;
            print_stored(&serde_json::to_value(&item)?);
            Ok(())
        }
        Command::Get { id } => {
            let item = engine.item(&id, &Links::default())?;
            println!("{}", serde_json::to_string_pretty(&item)?);
            Ok(())
        }
        Command::Search { query } => {
            let hits = engine.search(&query.join(" "), 50)?;
            print_hits(&serde_json::to_value(&hits)?);
            Ok(())
        }
        Command::Reindex => {
            let n = engine.reindex()?;
            println!("重建了索引：{} 条", n.items);
            for u in &n.unreadable {
                println!("读不了，跳过：{} （{}）", u.id, u.reason);
            }
            Ok(())
        }
        Command::Daemon { port, .. } => daemon(engine, &root, port).await,
    }
}

fn open(root: &std::path::Path, cli: &Cli) -> Result<Engine> {
    let library = machine::library(root, cli.library.clone())?;
    let mut config = EngineConfig::new(&library, root);
    let mut roots = cli.plugins.clone();
    roots.extend(machine::stage_dev_web(root)?);
    roots.extend(config.plugin_roots);
    config.plugin_roots = roots;
    config.plugin_settings = machine::plugin_settings(root)?;
    config.transforms_auto = usuyo_core::transforms_auto(root)?;
    config.proxy = usuyo_core::proxy_setting(root)?;
    // 设置的语言是「跟随系统」时用系统的首选语言。
    config.system_language = usuyo_core::Lang::from_system(sys_locale::get_locale().as_deref());
    // 登录的 cookie 和密钥：macOS、Windows 进系统钥匙串，别的系统是按机器目录里 0600 的文件。
    config.vault = usuyo_core::system_vault(root);
    // 手机那一页只在 macOS 上管（iPhone 镜像那条路要它）。
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME").context("HOME is not set")?;
        config.phone = Some(usuyo_core::PhoneConfig::system(
            root,
            std::path::Path::new(&home),
        )?);
    }
    Engine::open(config).with_context(|| format!("cannot open library {}", library.display()))
}

async fn daemon(engine: Engine, root: &std::path::Path, port: u16) -> Result<()> {
    // 先拿到端口再写令牌和 daemon.json：端口被别的占着时不碰正在跑的那个的。
    // 绑上以后到开始应答之前，连进来的请求只排队，应答时用的已经是这次的令牌。
    let listener = usuyo_api::bind(port).await?;
    machine::make_private_root(root)?;
    let token_path = machine::token_path(root);
    let token = usuyo_api::create_token(&token_path)?;
    let pid = std::process::id();
    let info = machine::daemon_path(root);
    usuyo_core::write_private(
        &info,
        &serde_json::to_vec(&json!({"port": port, "pid": pid}))?,
    )
    .with_context(|| format!("cannot write {}", info.display()))?;
    tracing::info!(
        library = %engine.library_root().display(),
        plugins = ?engine.plugin_ids(),
        inbox = %engine.inbox_dir().display(),
        "listening on http://127.0.0.1:{port}"
    );
    let config = usuyo_api::ApiConfig {
        port,
        token: token.clone(),
        origins: usuyo_api::default_origins(),
        clients: Some(machine::clients_path(root)),
    };
    let result = usuyo_api::serve(listener, engine.clone(), config, stop_signal()).await;
    // 转换插件是常驻进程：接口停下以后一个个关掉，不留到守护进程之后。
    engine.shutdown().await;
    machine::forget_daemon(&info, pid);
    machine::forget_token(&token_path, &token);
    Ok(result?)
}

/// Ctrl-C，或（Unix 上）SIGTERM：桌面壳退出时发的是 SIGTERM。
async fn stop_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut term) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = term.recv() => {}
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "cannot listen for SIGTERM; only Ctrl-C stops the daemon");
                if tokio::signal::ctrl_c().await.is_err() {
                    std::future::pending::<()>().await;
                }
            }
        }
    }
    #[cfg(not(unix))]
    if tokio::signal::ctrl_c().await.is_err() {
        std::future::pending::<()>().await;
    }
    tracing::info!("stopping");
}

fn through_daemon(c: &Client, command: Command) -> Result<()> {
    match command {
        Command::Add { url } => {
            let taken = c.post("/items", Some(&json!({"input": url, "kind": "link"})))?;
            println!("{}", taken["id"].as_str().unwrap_or_default());
            println!("守护进程在后台取内容");
        }
        Command::Get { id } => {
            let item = c.get(&format!("/items/{id}"), &[])?;
            println!("{}", serde_json::to_string_pretty(&item)?);
        }
        Command::Search { query } => {
            let q = query.join(" ");
            print_hits(&c.get("/search", &[("q", &q)])?);
        }
        Command::Reindex => {
            let r = c.post("/reindex", None)?;
            println!("重建了索引：{} 条", r["items"]);
        }
        Command::Init { .. } | Command::Daemon { .. } => {}
    }
    Ok(())
}

fn print_stored(item: &Value) {
    let title = item["title"].as_str().unwrap_or_default();
    match (item["stored"].as_str(), item["missing"].as_str()) {
        (Some("link"), Some(why)) => println!("只有链接：{why}"),
        (Some("link"), None) => println!("只有链接，过会儿再取（守护进程在跑时会自己再试）"),
        (Some("partial"), Some(why)) => println!("存下了一部分：{title}（{why}）"),
        (_, Some(why)) => println!("存好了：{title}（{why}）"),
        _ => println!("存好了：{title}"),
    }
}

fn print_hits(hits: &Value) {
    let empty = Vec::new();
    let list = hits.as_array().unwrap_or(&empty);
    if list.is_empty() {
        println!("没有搜到");
    }
    for h in list {
        println!(
            "{}  {}\n    [{}] {}",
            h["itemId"].as_str().unwrap_or_default(),
            h["title"].as_str().unwrap_or_default(),
            h["layer"].as_str().unwrap_or_default(),
            h["snippet"].as_str().unwrap_or_default().replace('\n', " ")
        );
    }
}
