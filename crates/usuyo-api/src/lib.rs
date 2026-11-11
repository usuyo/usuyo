//! 本机 HTTP 接口（axum），照前端的 `apps/desktop/web/src/api/endpoints.ts`。
//! 不开 `server` 时只剩 [`handshake`]：桌面壳和命令行用它确认对面是内核。
//!
//! 只绑 127.0.0.1；门的规则见 [`guard`]。事件用 SSE（`GET /events`），浏览器的 `EventSource` 直接能接。
//! 从资料库读出来的平台内容都是不可信的字：只经 JSON 往外给；媒体按白名单定类型，
//! 别的一律 `application/octet-stream`，再加 `nosniff` 和沙箱 CSP，不会被当网页打开。

#[cfg(feature = "server")]
mod guard;
pub mod handshake;
#[cfg(feature = "server")]
mod problem;
#[cfg(feature = "server")]
mod routes;

#[cfg(feature = "server")]
use std::future::Future;
#[cfg(feature = "server")]
use std::net::{Ipv4Addr, SocketAddr};
#[cfg(feature = "server")]
use std::path::Path;
#[cfg(feature = "server")]
use std::sync::Arc;

#[cfg(feature = "server")]
use axum::Router;
#[cfg(feature = "server")]
use usuyo_core::Engine;

#[cfg(feature = "server")]
pub use problem::Problem;

/// 默认端口，和前端的 `DEFAULT_API_BASE` 一致。
pub const DEFAULT_PORT: u16 = 47615;

#[cfg(feature = "server")]
/// 允许的浏览器来源：只有打包好的桌面壳的 webview（macOS、Linux 是 `tauri://localhost`，
/// Windows 是 `http://tauri.localhost`）。开发时前端经 Vite 的代理走，代理不带 `Origin`、带令牌。
pub fn default_origins() -> Vec<String> {
    ["tauri://localhost", "http://tauri.localhost"]
        .map(String::from)
        .to_vec()
}

#[cfg(feature = "server")]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error at {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot bind {addr}: {source}")]
    Bind {
        addr: SocketAddr,
        #[source]
        source: std::io::Error,
    },
    #[error("system random source failed: {0}")]
    Random(getrandom::Error),
}

#[cfg(feature = "server")]
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub port: u16,
    /// 命令行和别的程序带的令牌。
    pub token: String,
    pub origins: Vec<String>,
}

#[cfg(feature = "server")]
/// 整个接口。`port` 用来核对 `Host`，也拼进媒体地址和握手的消息。
pub fn router(engine: Engine, config: ApiConfig) -> Result<Router, Error> {
    // 媒体的 `k` 由它按条目算，和令牌无关；每次启动换一个。
    let media_secret = handshake::random_hex().map_err(Error::Random)?;
    let gate = Arc::new(guard::Gate {
        port: config.port,
        token: config.token,
        media_secret,
        origins: config.origins.clone(),
    });
    Ok(routes::build(engine, gate, &config.origins))
}

#[cfg(feature = "server")]
/// 绑 127.0.0.1:`port`。和 [`serve`] 分开，好让调用方确认端口拿到了再登记自己。
pub async fn bind(port: u16) -> Result<tokio::net::TcpListener, Error> {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|source| Error::Bind { addr, source })
}

#[cfg(feature = "server")]
/// 在绑好的端口上跑后台任务和接口，`shutdown` 完成时停。
pub async fn serve(
    listener: tokio::net::TcpListener,
    engine: Engine,
    config: ApiConfig,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), Error> {
    let stop = tokio_util::sync::CancellationToken::new();
    let app = router(engine.clone(), config)?.layer(axum::Extension(stop.clone()));
    let worker = {
        let engine = engine.clone();
        tokio::spawn(async move { engine.run_worker().await })
    };
    let stopping = stop.clone();
    let result = axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown.await;
            // 事件流不会自己结束，先让它们收尾。
            stopping.cancel();
        })
        .await
        .map_err(|source| Error::Io {
            path: "<listener>".into(),
            source,
        });
    worker.abort();
    result
}

#[cfg(feature = "server")]
/// 生成这次启动的令牌，写进只有自己能读（0600）的文件；以前的那个作废。
/// 先写到旁边的临时文件再换名，读的一方不会读到半个令牌。
pub fn create_token(path: &Path) -> Result<String, Error> {
    let io = |source| Error::Io {
        path: path.to_owned(),
        source,
    };
    let token = handshake::random_hex().map_err(Error::Random)?;
    let tmp = path.with_extension("tmp");
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
    let mut f = opts.open(&tmp).map_err(io)?;
    // 临时文件早就在时 `mode` 不管用，再设一次。
    #[cfg(unix)]
    f.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))
        .map_err(io)?;
    use std::io::Write;
    f.write_all(token.as_bytes()).map_err(io)?;
    f.sync_all().map_err(io)?;
    std::fs::rename(&tmp, path).map_err(io)?;
    Ok(token)
}
