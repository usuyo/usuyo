//! 路径和处理函数，一条对 `endpoints.ts` 的一个 `paths`。

use std::convert::Infallible;
use std::path::Path as FsPath;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{DefaultBodyLimit, Path, Query, Request, State};
use axum::http::{HeaderName, HeaderValue, Method, StatusCode, header};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router, middleware};
use futures_util::{Stream, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio_stream::wrappers::BroadcastStream;
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::ServeFile;
use tower_http::set_header::SetResponseHeaderLayer;
use usuyo_core::view::{ItemView, MapView, SearchHit, TakeResponse, Topic};
use usuyo_core::{Engine, Links, Via};

use crate::guard::{self, Gate};
use crate::problem::Problem;

/// 请求体上限：这里的请求都是几个短字段。
const BODY_LIMIT: usize = 64 << 10;
/// 慢请求（含读请求体）多久算超时；事件流不算。
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const SEARCH_LIMIT: usize = 50;
const SEARCH_MAX: usize = 200;

#[derive(Clone)]
struct AppState {
    engine: Engine,
    links: Arc<Links>,
}

pub(crate) fn build(engine: Engine, links: Links, gate: Arc<Gate>, origins: &[String]) -> Router {
    let state = AppState {
        engine,
        links: Arc::new(links),
    };
    let api = Router::new()
        .route("/health", get(health))
        .route("/map", get(map))
        .route("/topics", get(topics))
        .route("/topics/{id}/follow", post(follow))
        .route("/items", post(take))
        .route("/items/{id}", get(item))
        .route("/items/{id}/note", post(note))
        .route("/items/{id}/topics", post(topic_membership))
        .route("/items/{id}/trash", post(trash))
        .route("/items/{id}/restore", post(restore))
        .route("/items/{id}/restore-media", post(restore_media))
        .route("/items/{id}/text-only", post(text_only))
        .route("/items/{id}/refetch", post(refetch))
        .route("/search", get(search))
        .route("/reindex", post(reindex))
        .route("/media/{id}/{*layer}", get(media))
        .layer(middleware::from_fn(deadline));
    let origins: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| HeaderValue::from_str(o).ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::RANGE])
        .expose_headers([
            header::CONTENT_RANGE,
            header::ACCEPT_RANGES,
            header::RETRY_AFTER,
        ]);
    api.merge(Router::new().route("/events", get(events)))
        .fallback(|| async { Problem::new("not-found", "no such route") })
        .method_not_allowed_fallback(|| async {
            let mut p = Problem::new("invalid-args", "method not allowed on this route");
            p.status = 405;
            p
        })
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(cors)
        .layer(middleware::from_fn_with_state(gate, guard::check))
        .with_state(state)
}

type ApiResult<T> = Result<Json<T>, Problem>;

/// 慢请求（含读请求体）到点就回 `timeout`；事件流不经这里。
async fn deadline(req: Request, next: middleware::Next) -> Response {
    match tokio::time::timeout(REQUEST_TIMEOUT, next.run(req)).await {
        Ok(res) => res,
        Err(_) => Problem::new("timeout", "request took too long").into_response(),
    }
}

/// 同步的引擎调用放到阻塞线程上。
async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> usuyo_core::Result<T> + Send + 'static,
) -> Result<T, Problem> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| Problem::new("internal", e.to_string()))?
        .map_err(Problem::from)
}

fn body<T>(r: Result<Json<T>, JsonRejection>) -> Result<T, Problem> {
    match r {
        Ok(Json(v)) => Ok(v),
        Err(e) if e.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            Err(Problem::new("too-large", e.body_text()))
        }
        Err(e) => Err(Problem::new("invalid-args", e.body_text())),
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")}))
}

async fn map(State(s): State<AppState>) -> ApiResult<MapView> {
    Ok(Json(blocking(move || s.engine.map()).await?))
}

async fn topics(State(s): State<AppState>) -> ApiResult<Vec<Topic>> {
    Ok(Json(blocking(move || s.engine.topics()).await?))
}

#[derive(Deserialize)]
struct FollowRequest {
    followed: bool,
}

async fn follow(
    State(s): State<AppState>,
    Path(id): Path<String>,
    req: Result<Json<FollowRequest>, JsonRejection>,
) -> ApiResult<Topic> {
    let req = body(req)?;
    Ok(Json(
        blocking(move || s.engine.follow(&id, req.followed, Via::App)).await?,
    ))
}

#[derive(Deserialize)]
struct TakeRequest {
    input: String,
    kind: String,
}

async fn take(
    State(s): State<AppState>,
    req: Result<Json<TakeRequest>, JsonRejection>,
) -> ApiResult<TakeResponse> {
    let req = body(req)?;
    Ok(Json(s.engine.take(&req.input, &req.kind, Via::App).await?))
}

async fn item(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ItemView> {
    Ok(Json(blocking(move || s.engine.item(&id, &s.links)).await?))
}

#[derive(Deserialize)]
struct NoteRequest {
    text: String,
}

async fn note(
    State(s): State<AppState>,
    Path(id): Path<String>,
    req: Result<Json<NoteRequest>, JsonRejection>,
) -> ApiResult<ItemView> {
    let req = body(req)?;
    Ok(Json(
        blocking(move || s.engine.set_note(&id, &req.text, Via::App, &s.links)).await?,
    ))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TopicMembershipRequest {
    topic_id: String,
    member: bool,
}

async fn topic_membership(
    State(s): State<AppState>,
    Path(id): Path<String>,
    req: Result<Json<TopicMembershipRequest>, JsonRejection>,
) -> ApiResult<ItemView> {
    let req = body(req)?;
    Ok(Json(
        blocking(move || {
            s.engine
                .set_topic(&id, &req.topic_id, req.member, Via::App, &s.links)
        })
        .await?,
    ))
}

async fn trash(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ItemView> {
    Ok(Json(
        blocking(move || s.engine.trash(&id, Via::App, &s.links)).await?,
    ))
}

async fn restore(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ItemView> {
    Ok(Json(
        blocking(move || s.engine.restore(&id, Via::App, &s.links)).await?,
    ))
}

async fn restore_media(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ItemView> {
    Ok(Json(
        blocking(move || s.engine.restore_media(&id, Via::App, &s.links)).await?,
    ))
}

async fn text_only(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ItemView> {
    Ok(Json(
        blocking(move || s.engine.text_only(&id, Via::App, &s.links)).await?,
    ))
}

async fn refetch(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ItemView> {
    Ok(Json(
        blocking(move || s.engine.refetch(&id, &s.links)).await?,
    ))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<usize>,
}

async fn search(
    State(s): State<AppState>,
    q: Result<Query<SearchQuery>, QueryRejection>,
) -> ApiResult<Vec<SearchHit>> {
    let Query(q) = q.map_err(|e| Problem::new("invalid-args", e.body_text()))?;
    let limit = q.limit.unwrap_or(SEARCH_LIMIT).clamp(1, SEARCH_MAX);
    Ok(Json(blocking(move || s.engine.search(&q.q, limit)).await?))
}

async fn reindex(State(s): State<AppState>) -> ApiResult<serde_json::Value> {
    let n = blocking(move || s.engine.reindex()).await?;
    Ok(Json(json!({"items": n})))
}

/// 媒体：支持 Range（视频拖进度）。类型只给白名单里的，别的不让浏览器当网页或脚本打开。
async fn media(
    State(s): State<AppState>,
    Path((id, layer)): Path<(String, String)>,
    req: Request,
) -> Result<Response, Problem> {
    let path = blocking(move || s.engine.media(&id, &layer)).await?;
    let ctype = media_type(&path);
    let res = match ServeFile::new(&path).oneshot(req).await {
        Ok(r) => r,
        Err(never) => match never {},
    };
    let mut res = res.map(axum::body::Body::new);
    let h = res.headers_mut();
    if h.contains_key(header::CONTENT_TYPE) {
        h.insert(header::CONTENT_TYPE, HeaderValue::from_static(ctype));
    }
    h.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; sandbox"),
    );
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=3600"),
    );
    h.insert(
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("cross-origin"),
    );
    Ok(res)
}

fn media_type(path: &FsPath) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "m4a" => "audio/mp4",
        "mp3" => "audio/mpeg",
        "ogg" | "opus" => "audio/ogg",
        // svg、html 和不认识的：只当字节下载
        _ => "application/octet-stream",
    }
}

/// 事件流：`event: <type>`，`data:` 是事件的 JSON。守护进程要停时结束。
async fn events(
    State(s): State<AppState>,
    stop: Option<Extension<CancellationToken>>,
) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
    let stop = stop.map(|Extension(t)| t).unwrap_or_default();
    let stream = BroadcastStream::new(s.engine.subscribe())
        .filter_map(|r| async move {
            // 落后太多丢掉的事件不补：前端重新拉一次就是最新的。
            let e = r.ok()?;
            let name = serde_json::to_value(&e).ok()?["type"].as_str()?.to_owned();
            SseEvent::default().event(name).json_data(&e).ok().map(Ok)
        })
        .take_until(stop.cancelled_owned());
    Sse::new(stream).keep_alive(KeepAlive::default())
}
