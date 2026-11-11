//! 本机 HTTP 接口（axum），照前端的 `apps/desktop/web/src/api/endpoints.ts`。
//!
//! 只绑 127.0.0.1；门的规则见 [`guard`]。事件用 SSE（`GET /events`），浏览器的 `EventSource` 直接能接。
//! 从资料库读出来的平台内容都是不可信的字：只经 JSON 往外给；媒体按白名单定类型，
//! 别的一律 `application/octet-stream`，再加 `nosniff` 和沙箱 CSP，不会被当网页打开。

mod guard;
mod problem;
mod routes;

use std::future::Future;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::Arc;

use axum::Router;
use usuyo_core::{Engine, Links};

pub use problem::Problem;

/// 默认端口，和前端的 `DEFAULT_API_BASE` 一致。
pub const DEFAULT_PORT: u16 = 47615;

/// 允许的浏览器来源：Tauri 2 的 webview（macOS、Linux 是 `tauri://localhost`，Windows 是
/// `http(s)://tauri.localhost`）和 Vite 开发服务器（`apps/desktop/web/vite.config.ts` 固定 5173）。
pub fn default_origins() -> Vec<String> {
    [
        "tauri://localhost",
        "http://tauri.localhost",
        "https://tauri.localhost",
        "http://localhost:5173",
        "http://127.0.0.1:5173",
    ]
    .map(String::from)
    .to_vec()
}

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

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub port: u16,
    /// 命令行和别的程序带的令牌。
    pub token: String,
    pub origins: Vec<String>,
}

/// 整个接口。`port` 用来核对 `Host`，也拼进媒体地址。
pub fn router(engine: Engine, config: ApiConfig) -> Result<Router, Error> {
    let media_key = guard::random_secret().map_err(Error::Random)?;
    let links = Links {
        base: format!("http://127.0.0.1:{}", config.port),
        key: Some(media_key.clone()),
    };
    let gate = Arc::new(guard::Gate {
        port: config.port,
        token: config.token,
        media_key,
        origins: config.origins.clone(),
    });
    Ok(routes::build(engine, links, gate, &config.origins))
}

/// 绑 127.0.0.1:`port`，跑后台任务和接口，`shutdown` 完成时停。
pub async fn serve(
    engine: Engine,
    config: ApiConfig,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), Error> {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, config.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|source| Error::Bind { addr, source })?;
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

/// 读本机接口令牌，没有就生成一个；文件只有自己能读（0600）。
pub fn load_or_create_token(path: &Path) -> Result<String, Error> {
    let io = |source| Error::Io {
        path: path.to_owned(),
        source,
    };
    match std::fs::read_to_string(path) {
        Ok(t) if !t.trim().is_empty() => return Ok(t.trim().to_owned()),
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(io(e)),
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io)?;
    }
    let token = guard::random_secret().map_err(Error::Random)?;
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    use std::io::Write;
    let mut f = opts.open(path).map_err(io)?;
    f.write_all(token.as_bytes()).map_err(io)?;
    Ok(token)
}
