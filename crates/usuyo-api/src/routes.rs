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
use axum::routing::{delete, get, post, put};
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
use usuyo_core::view::{
    FileReaderView, ImportEntryView, ImportView, ItemView, MapView, PhoneDryRun, PhoneView,
    PluginView, PluginsView, SearchHit, SettingsPane, SettingsView, SiteView, SourceView,
    TakeResponse, Topic, TransformsView, TrashView, VisitorView,
};
use usuyo_core::{DeliveredLink, DriverAction, Engine, ImportChoice, Links, SettingsPatch, Via};
use usuyo_plugin::protocol::StorageState;

use crate::clients::{Client, ClientKind};
use crate::guard::{self, Caller, Gate};
use crate::handshake;
use crate::problem::Problem;
use crate::words;

/// 请求体上限：这里的请求都是几个短字段。
const BODY_LIMIT: usize = 64 << 10;
/// 慢请求（含读请求体）多久算超时；事件流不算。
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const SEARCH_LIMIT: usize = 50;
const SEARCH_MAX: usize = 200;

/// 握手的请求体上限：一个 64 位的随机数，加一个投递端 id。
const HANDSHAKE_BODY_LIMIT: usize = 256;
/// 登录窗口交来的 cookie 和 localStorage。
const LOGIN_BODY_LIMIT: usize = 256 << 10;
/// 一批链接（`/v1/batches`），最多 5000 条。
const BATCH_BODY_LIMIT: usize = 4 << 20;
/// 列来源要按节奏等上一次调用，再等插件跑完。
const SOURCES_TIMEOUT: Duration = Duration::from_secs(120);
/// 交一个文件：传完、再等插件读完第一页。
const UPLOAD_TIMEOUT: Duration = Duration::from_secs(30 * 60);

#[derive(Clone)]
struct AppState {
    engine: Engine,
    gate: Arc<Gate>,
    /// 资料库在哪；`/health` 不去拿资料库的锁。
    library: Arc<FsPath>,
}

impl AppState {
    /// 条目 `id` 的媒体地址：`k` 只对这一条有效。
    fn links(&self, id: &str) -> Links {
        Links {
            base: format!("http://127.0.0.1:{}", self.gate.port),
            key: Some(self.gate.media_key(id)),
        }
    }
}

pub(crate) fn build(engine: Engine, gate: Arc<Gate>, origins: &[String]) -> Router {
    let state = AppState {
        library: engine.library_root().into(),
        engine,
        gate: Arc::clone(&gate),
    };
    let api = Router::new()
        .route(
            handshake::PATH,
            post(answer_handshake).layer(DefaultBodyLimit::max(HANDSHAKE_BODY_LIMIT)),
        )
        .route("/health", get(health))
        .route("/map", get(map))
        .route("/topics", get(topics).post(create_topic))
        .route("/topics/{id}/follow", post(follow))
        .route("/items", post(take))
        .route("/items/{id}", get(item))
        .route("/items/{id}/note", post(note))
        .route("/items/{id}/topics", post(topic_membership))
        .route("/items/{id}/trash", post(trash))
        .route("/items/{id}/restore", post(restore))
        .route("/items/{id}/restore-media", post(restore_media))
        .route("/trash", get(trash_list))
        .route("/trash/{id}/purge", post(purge_now))
        .route("/items/{id}/text-only", post(text_only))
        .route("/items/{id}/refetch", post(refetch))
        .route("/items/{id}/transform", post(item_transform))
        .route("/transforms", get(transforms))
        .route(
            "/transforms/{owner}/{repo}/models/{name}/download",
            post(model_download),
        )
        .route(
            "/transforms/{owner}/{repo}/models/{name}/remove",
            post(model_remove),
        )
        .route("/phone", get(phone))
        .route("/phone/ios/driver/install", post(phone_install))
        .route("/phone/ios/driver/update", post(phone_update))
        .route("/phone/ios/driver/remove", post(phone_remove))
        .route("/phone/open-settings", post(phone_open_settings))
        .route("/phone/ios/clipboard-test", post(phone_clipboard_test))
        .route("/phone/ios/dry-run", post(phone_dry_run))
        .route("/phone/android/devices", post(phone_android_devices))
        .route("/settings", get(settings).patch(patch_settings))
        .route("/trash/release-held", post(release_held))
        .route("/search", get(search))
        .route("/reindex", post(reindex))
        .route("/media/{id}/{*layer}", get(media))
        .route("/plugins", get(plugins))
        .route("/plugins/{owner}/{repo}/try-now", post(plugin_try_now))
        .route("/plugins/{owner}/{repo}/hold", post(plugin_hold))
        .route(
            "/plugins/{owner}/{repo}/methods/{method}",
            put(plugin_method_switch).delete(plugin_method_clear),
        )
        .route(
            "/plugins/{owner}/{repo}/methods/{method}/script",
            get(method_script),
        )
        .route(
            "/plugins/{owner}/{repo}/methods/{method}/script/reveal",
            post(method_script_reveal),
        )
        .route(
            "/plugins/{owner}/{repo}/secrets/{key}",
            put(plugin_secret_set).delete(plugin_secret_forget),
        )
        .route("/sites", get(sites))
        .route("/visitors", get(visitors))
        .route("/visitors/{site}", put(visitor_save).delete(visitor_forget))
        .route(
            "/sites/{site}/login",
            put(site_login)
                .delete(site_forget)
                .layer(DefaultBodyLimit::max(LOGIN_BODY_LIMIT)),
        )
        .route("/imports", get(imports).post(start_import))
        .route("/imports/{id}", get(import).delete(delete_import))
        .route("/imports/{id}/entries", get(import_entries))
        .route("/imports/{id}/confirm", post(confirm_import))
        .route("/imports/{id}/hold", post(hold_import))
        .route("/imports/{id}/resume", post(resume_import))
        .route("/deliver-tokens", get(clients).post(create_client))
        .route("/deliver-tokens/{id}", delete(revoke_client))
        .route("/inbox", get(inbox))
        .route("/file-readers", get(file_readers))
        .route("/v1/ping", get(ping))
        .route("/v1/deliver", post(deliver))
        .route(
            "/v1/batches",
            post(batch).layer(DefaultBodyLimit::max(BATCH_BODY_LIMIT)),
        )
        .layer(middleware::from_fn(deadline));
    let slow = Router::new()
        .route("/plugins/{owner}/{repo}/sources", get(plugin_sources))
        .layer(middleware::from_fn(slow_deadline));
    // 交文件：请求体自己数着写进副本，不经 64 KiB 的上限，时限另算
    let uploads = Router::new()
        .route("/file-imports", post(file_import))
        .route("/v1/file-imports", post(hand_over))
        .layer(DefaultBodyLimit::disable())
        .layer(middleware::from_fn(upload_deadline));
    let origins: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| HeaderValue::from_str(o).ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::RANGE])
        .expose_headers([
            header::CONTENT_RANGE,
            header::ACCEPT_RANGES,
            header::RETRY_AFTER,
        ]);
    api.merge(slow)
        .merge(uploads)
        .merge(Router::new().route("/events", get(events)))
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
        .layer(middleware::from_fn_with_state(
            state.engine.clone(),
            crate::problem::speak,
        ))
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

async fn upload_deadline(req: Request, next: middleware::Next) -> Response {
    match tokio::time::timeout(UPLOAD_TIMEOUT, next.run(req)).await {
        Ok(res) => res,
        Err(_) => Problem::new("timeout", "upload took too long")
            .with_hint(words::UPLOAD_TOO_SLOW.said())
            .into_response(),
    }
}

async fn slow_deadline(req: Request, next: middleware::Next) -> Response {
    match tokio::time::timeout(SOURCES_TIMEOUT, next.run(req)).await {
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

#[derive(Deserialize)]
struct HandshakeRequest {
    nonce: String,
    /// 投递端的 id（令牌 `.` 前面那段）：证明用它的整个令牌算。不给就用主令牌。
    client: Option<String>,
}

/// 证明自己是这次启动的内核，只回证明（见 [`crate::handshake`]）。
/// 不认识的投递端照样回一个证明（用随机的密钥），对面分不出那个 id 在不在。
async fn answer_handshake(
    State(s): State<AppState>,
    req: Result<Json<HandshakeRequest>, JsonRejection>,
) -> ApiResult<serde_json::Value> {
    let req = body(req)?;
    if !handshake::valid_nonce(&req.nonce) {
        return Err(Problem::new(
            "invalid-args",
            "nonce must be 64 lowercase hex characters",
        ));
    }
    let key = match &req.client {
        None => s.gate.token.clone(),
        Some(id) => match s.gate.clients.token_of(id) {
            Some(t) => t,
            None => handshake::random_hex().map_err(|e| Problem::new("internal", e.to_string()))?,
        },
    };
    let proof = handshake::proof(&key, s.gate.port, &req.nonce);
    Ok(Json(json!({ "proof": proof })))
}

async fn health(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "library": &*s.library,
    }))
}

async fn map(State(s): State<AppState>) -> ApiResult<MapView> {
    Ok(Json(blocking(move || s.engine.map()).await?))
}

async fn topics(State(s): State<AppState>) -> ApiResult<Vec<Topic>> {
    Ok(Json(blocking(move || s.engine.topics()).await?))
}

#[derive(Deserialize)]
struct CreateTopicRequest {
    name: String,
}

async fn create_topic(
    State(s): State<AppState>,
    req: Result<Json<CreateTopicRequest>, JsonRejection>,
) -> ApiResult<Topic> {
    let req = body(req)?;
    Ok(Json(
        blocking(move || s.engine.create_topic(&req.name, Via::App)).await?,
    ))
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
    let links = s.links(&id);
    Ok(Json(blocking(move || s.engine.item(&id, &links)).await?))
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
    let links = s.links(&id);
    Ok(Json(
        blocking(move || s.engine.set_note(&id, &req.text, Via::App, &links)).await?,
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
    let links = s.links(&id);
    Ok(Json(
        blocking(move || {
            s.engine
                .set_topic(&id, &req.topic_id, req.member, Via::App, &links)
        })
        .await?,
    ))
}

async fn trash(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ItemView> {
    let links = s.links(&id);
    Ok(Json(
        blocking(move || s.engine.trash(&id, Via::App, &links)).await?,
    ))
}

async fn restore(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ItemView> {
    let links = s.links(&id);
    Ok(Json(
        blocking(move || s.engine.restore(&id, Via::App, &links)).await?,
    ))
}

async fn restore_media(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ItemView> {
    let links = s.links(&id);
    Ok(Json(
        blocking(move || s.engine.restore_media(&id, Via::App, &links)).await?,
    ))
}

async fn trash_list(State(s): State<AppState>) -> ApiResult<TrashView> {
    Ok(Json(blocking(move || s.engine.trash_list()).await?))
}

/// 回收站里「立即清除」一项，不等 `trashDays`。
async fn purge_now(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, Problem> {
    blocking(move || s.engine.purge_now(&id, Via::App)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn text_only(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ItemView> {
    let links = s.links(&id);
    Ok(Json(
        blocking(move || s.engine.text_only(&id, Via::App, &links)).await?,
    ))
}

async fn refetch(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ItemView> {
    let links = s.links(&id);
    Ok(Json(blocking(move || s.engine.refetch(&id, &links)).await?))
}

/// `{jobs: "pending"}`，或 `{job, plugin?, options?}`。
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TransformRequest {
    jobs: Option<String>,
    job: Option<String>,
    plugin: Option<String>,
    options: Option<serde_json::Map<String, serde_json::Value>>,
}

/// 打开条目页时把还没转的挪到最前；或者再转一次、换一个插件转。回 202，转换在后台跑。
async fn item_transform(
    State(s): State<AppState>,
    Path(id): Path<String>,
    req: Result<Json<TransformRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<serde_json::Value>), Problem> {
    let req = body(req)?;
    match req {
        TransformRequest {
            jobs: Some(jobs),
            job: None,
            plugin: None,
            options: None,
        } if jobs == "pending" => {
            blocking(move || s.engine.transform_now(&id)).await?;
        }
        TransformRequest {
            jobs: None,
            job: Some(job),
            plugin,
            options,
        } => {
            blocking(move || {
                s.engine
                    .redo_transform(&id, &job, plugin.as_deref(), options.unwrap_or_default())
            })
            .await?;
        }
        _ => {
            return Err(Problem::new(
                "invalid-args",
                "give {\"jobs\": \"pending\"} or {job, plugin?, options?}",
            ));
        }
    }
    Ok((StatusCode::ACCEPTED, Json(json!({"queued": true}))))
}

async fn transforms(State(s): State<AppState>) -> Json<TransformsView> {
    Json(s.engine.transforms_view().await)
}

/// 下一个模型：已经在了回 200，开始下（或已经在下）回 202，下完发 `transforms-updated`。
async fn model_download(
    State(s): State<AppState>,
    Path((owner, repo, name)): Path<(String, String, String)>,
) -> Result<(StatusCode, Json<serde_json::Value>), Problem> {
    if s.engine.download_model(&plugin_id(&owner, &repo), &name)? {
        Ok((StatusCode::OK, Json(json!({"state": "present"}))))
    } else {
        Ok((StatusCode::ACCEPTED, Json(json!({"state": "downloading"}))))
    }
}

async fn model_remove(
    State(s): State<AppState>,
    Path((owner, repo, name)): Path<(String, String, String)>,
) -> Result<StatusCode, Problem> {
    blocking(move || s.engine.remove_model(&plugin_id(&owner, &repo), &name)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn phone(State(s): State<AppState>) -> ApiResult<PhoneView> {
    Ok(Json(s.engine.phone_view().await?))
}

async fn phone_install(State(s): State<AppState>) -> ApiResult<PhoneView> {
    Ok(Json(s.engine.phone_driver(DriverAction::Install).await?))
}

async fn phone_update(State(s): State<AppState>) -> ApiResult<PhoneView> {
    Ok(Json(s.engine.phone_driver(DriverAction::Update).await?))
}

async fn phone_remove(State(s): State<AppState>) -> ApiResult<PhoneView> {
    Ok(Json(s.engine.phone_driver(DriverAction::Remove).await?))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PaneRequest {
    pane: SettingsPane,
}

/// 打开系统设置的那一栏（或 iPhone 镜像这个 App），204。
async fn phone_open_settings(
    State(s): State<AppState>,
    req: Result<Json<PaneRequest>, JsonRejection>,
) -> Result<StatusCode, Problem> {
    let PaneRequest { pane } = body(req)?;
    s.engine.open_phone_settings(pane).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn phone_clipboard_test(State(s): State<AppState>) -> ApiResult<PhoneView> {
    Ok(Json(s.engine.phone_clipboard_test().await?))
}

async fn phone_dry_run(State(s): State<AppState>) -> ApiResult<PhoneDryRun> {
    Ok(Json(s.engine.phone_dry_run().await?))
}

/// 跑一次 `adb devices -l`（会起 adb 的服务，之后留在后台），回 `PhoneView`。
async fn phone_android_devices(State(s): State<AppState>) -> ApiResult<PhoneView> {
    Ok(Json(s.engine.phone_android_devices().await?))
}

async fn settings(State(s): State<AppState>) -> ApiResult<SettingsView> {
    Ok(Json(blocking(move || s.engine.settings()).await?))
}

async fn patch_settings(
    State(s): State<AppState>,
    req: Result<Json<SettingsPatch>, JsonRejection>,
) -> ApiResult<SettingsView> {
    let patch = body(req)?;
    Ok(Json(
        blocking(move || s.engine.patch_settings(patch, Via::App)).await?,
    ))
}

/// 「照时间移除」：时钟守卫拦下的回收站项照时间移除。
async fn release_held(State(s): State<AppState>) -> ApiResult<serde_json::Value> {
    let removed = blocking(move || s.engine.release_held_trash()).await?;
    Ok(Json(json!({ "removed": removed })))
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
    Ok(Json(json!({"items": n.items, "unreadable": n.unreadable})))
}

// —— 获取插件和登录站点 ——

/// 插件 id 是 `owner/repo`，路径里拆成两段。
fn plugin_id(owner: &str, repo: &str) -> String {
    format!("{owner}/{repo}")
}

async fn plugins(State(s): State<AppState>) -> ApiResult<PluginsView> {
    Ok(Json(s.engine.plugins().await?))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourcesQuery {
    method: String,
    /// 来源从这条链接开始（协议 §2.3.1）。
    #[serde(default)]
    link: Option<String>,
}

async fn plugin_sources(
    State(s): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    q: Result<Query<SourcesQuery>, QueryRejection>,
) -> ApiResult<Vec<SourceView>> {
    let Query(q) = q.map_err(|e| {
        Problem::new("invalid-args", e.body_text()).with_hint(words::NAME_THE_METHOD.said())
    })?;
    let link = q.link.as_deref().map(parse_link).transpose()?;
    Ok(Json(
        s.engine
            .plugin_sources(&plugin_id(&owner, &repo), &q.method, link.as_ref())
            .await?,
    ))
}

/// 你给的一条链接（或带链接的分享文字，取第一条，[`usuyo_core::link_in`]）：带主机的 http(s)，
/// 不带用户名密码，至多 4096 字节。原因里不写链接本身。
fn parse_link(text: &str) -> Result<url::Url, Problem> {
    let hint = if usuyo_core::link_count(text) > 1 {
        words::BAD_LINK_FIRST
    } else {
        words::BAD_LINK
    };
    let bad =
        |why: &str| Problem::new("invalid-args", format!("link: {why}")).with_hint(hint.said());
    let u = usuyo_core::link_in(text).ok_or_else(|| bad("no http(s) link with a host"))?;
    if u.as_str().len() > 4096 {
        return Err(bad("too long"));
    }
    if !u.username().is_empty() || u.password().is_some() {
        return Err(bad("links with a user name or password are not taken"));
    }
    Ok(u)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MethodSwitchRequest {
    enabled: bool,
}

async fn plugin_method_switch(
    State(s): State<AppState>,
    Path((owner, repo, method)): Path<(String, String, String)>,
    req: Result<Json<MethodSwitchRequest>, JsonRejection>,
) -> ApiResult<PluginView> {
    let req = body(req)?;
    Ok(Json(
        blocking(move || {
            s.engine
                .set_method(&plugin_id(&owner, &repo), &method, req.enabled)
        })
        .await?,
    ))
}

/// 去掉拨过的开关，回到各平台各自那一层的默认，回整个插件。
async fn plugin_method_clear(
    State(s): State<AppState>,
    Path((owner, repo, method)): Path<(String, String, String)>,
) -> ApiResult<PluginView> {
    Ok(Json(
        blocking(move || s.engine.clear_method(&plugin_id(&owner, &repo), &method)).await?,
    ))
}

/// 一种交链接的方法的脚本全文：只认装着的清单写了的那个文件，路径不从请求里来。
async fn method_script(
    State(s): State<AppState>,
    Path((owner, repo, method)): Path<(String, String, String)>,
) -> ApiResult<serde_json::Value> {
    let text = blocking(move || {
        s.engine
            .method_script_text(&plugin_id(&owner, &repo), &method)
    })
    .await?;
    Ok(Json(json!({ "text": text })))
}

/// 在访达（或系统的文件管理器）里显示那个脚本文件，204。
async fn method_script_reveal(
    State(s): State<AppState>,
    Path((owner, repo, method)): Path<(String, String, String)>,
) -> Result<StatusCode, Problem> {
    s.engine
        .reveal_method_script(&plugin_id(&owner, &repo), &method)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn plugin_try_now(
    State(s): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
) -> ApiResult<PluginView> {
    Ok(Json(
        blocking(move || s.engine.try_plugin_now(&plugin_id(&owner, &repo))).await?,
    ))
}

async fn plugin_hold(
    State(s): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
) -> ApiResult<PluginView> {
    Ok(Json(
        blocking(move || s.engine.hold_plugin(&plugin_id(&owner, &repo))).await?,
    ))
}

#[derive(Deserialize)]
struct SecretRequest {
    value: String,
}

async fn plugin_secret_set(
    State(s): State<AppState>,
    Path((owner, repo, key)): Path<(String, String, String)>,
    req: Result<Json<SecretRequest>, JsonRejection>,
) -> ApiResult<PluginView> {
    let req = body(req)?;
    if req.value.trim().is_empty() {
        return Err(Problem::new(
            "invalid-args",
            "value is empty; DELETE the setting to remove it",
        ));
    }
    Ok(Json(
        blocking(move || {
            s.engine
                .set_plugin_secret(&plugin_id(&owner, &repo), &key, &req.value)
        })
        .await?,
    ))
}

async fn plugin_secret_forget(
    State(s): State<AppState>,
    Path((owner, repo, key)): Path<(String, String, String)>,
) -> ApiResult<PluginView> {
    Ok(Json(
        blocking(move || {
            s.engine
                .set_plugin_secret(&plugin_id(&owner, &repo), &key, "")
        })
        .await?,
    ))
}

async fn sites(State(s): State<AppState>) -> ApiResult<Vec<SiteView>> {
    Ok(Json(blocking(move || s.engine.sites()).await?))
}

/// 交来的 cookie。解析不了时只说形状不对：解析器的原话会引用值。
fn cookies_body(r: Result<Json<StorageState>, JsonRejection>) -> Result<StorageState, Problem> {
    match r {
        Ok(Json(v)) => Ok(v),
        Err(e) if e.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            Err(Problem::new("too-large", e.body_text()))
        }
        Err(_) => Err(Problem::new(
            "invalid-args",
            "the body is not a storageState with cookies",
        )),
    }
}

/// 壳的登录窗口读出 cookie 以后交到这里：只留属于这个站点的，存进钥匙串。回的里面没有值。
async fn site_login(
    State(s): State<AppState>,
    Path(site): Path<String>,
    req: Result<Json<StorageState>, JsonRejection>,
) -> ApiResult<SiteView> {
    let state = cookies_body(req)?;
    Ok(Json(
        blocking(move || s.engine.save_site(&site, &state)).await?,
    ))
}

async fn site_forget(
    State(s): State<AppState>,
    Path(site): Path<String>,
) -> Result<StatusCode, Problem> {
    blocking(move || s.engine.forget_site(&site)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn visitors(State(s): State<AppState>) -> ApiResult<Vec<VisitorView>> {
    Ok(Json(blocking(move || s.engine.visitors()).await?))
}

/// 壳的访客窗口读出 cookie 以后交到这里：只留属于这个访客身份的，和同一主机的账号分开存。回的里面没有值。
async fn visitor_save(
    State(s): State<AppState>,
    Path(site): Path<String>,
    req: Result<Json<StorageState>, JsonRejection>,
) -> ApiResult<VisitorView> {
    let state = cookies_body(req)?;
    Ok(Json(
        blocking(move || s.engine.save_visitor(&site, &state)).await?,
    ))
}

async fn visitor_forget(
    State(s): State<AppState>,
    Path(site): Path<String>,
) -> Result<StatusCode, Problem> {
    blocking(move || s.engine.forget_visitor(&site)).await?;
    Ok(StatusCode::NO_CONTENT)
}

// —— 导入 ——

async fn imports(State(s): State<AppState>) -> ApiResult<Vec<ImportView>> {
    Ok(Json(blocking(move || s.engine.imports()).await?))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartImportRequest {
    plugin_id: String,
    method: String,
    source: String,
    #[serde(default)]
    title: String,
}

async fn start_import(
    State(s): State<AppState>,
    req: Result<Json<StartImportRequest>, JsonRejection>,
) -> ApiResult<ImportView> {
    let req = body(req).map_err(|p| p.with_hint(words::BAD_IMPORT_REQUEST.said()))?;
    Ok(Json(
        blocking(move || {
            s.engine
                .start_import(&req.plugin_id, &req.method, &req.source, &req.title)
        })
        .await?,
    ))
}

async fn import(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ImportView> {
    Ok(Json(blocking(move || s.engine.import(&id)).await?))
}

async fn delete_import(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, Problem> {
    blocking(move || s.engine.delete_import(&id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct EntriesQuery {
    standing: Option<String>,
    #[serde(default)]
    offset: u32,
    limit: Option<u32>,
}

async fn import_entries(
    State(s): State<AppState>,
    Path(id): Path<String>,
    q: Result<Query<EntriesQuery>, QueryRejection>,
) -> ApiResult<Vec<ImportEntryView>> {
    let Query(q) = q.map_err(|e| {
        Problem::new("invalid-args", e.body_text()).with_hint(words::BAD_PREVIEW_QUERY.said())
    })?;
    Ok(Json(
        blocking(move || {
            s.engine
                .import_entries(&id, q.standing.as_deref(), q.offset, q.limit.unwrap_or(100))
        })
        .await?,
    ))
}

/// `{new, trashed, except}`：两组各一个整组的选择，加上和自己那组选得不一样的条目。
async fn confirm_import(
    State(s): State<AppState>,
    Path(id): Path<String>,
    req: Result<Json<ImportChoice>, JsonRejection>,
) -> ApiResult<ImportView> {
    let choice = body(req).map_err(|p| p.with_hint(words::BAD_CONFIRM_REQUEST.said()))?;
    Ok(Json(
        blocking(move || s.engine.confirm_import(&id, &choice)).await?,
    ))
}

async fn hold_import(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ImportView> {
    Ok(Json(blocking(move || s.engine.hold_import(&id)).await?))
}

async fn resume_import(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<ImportView> {
    Ok(Json(blocking(move || s.engine.resume_import(&id)).await?))
}

// —— 投递令牌、收件文件夹 ——

async fn clients(State(s): State<AppState>) -> Json<Vec<Client>> {
    Json(s.gate.clients.list())
}

#[derive(Deserialize)]
struct CreateClientRequest {
    name: String,
    kind: ClientKind,
}

/// 发一个投递令牌；`token` 只在这里回一次。
async fn create_client(
    State(s): State<AppState>,
    req: Result<Json<CreateClientRequest>, JsonRejection>,
) -> ApiResult<serde_json::Value> {
    let req = body(req)?;
    let name = req.name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(Problem::new(
            "invalid-args",
            "name must be 1 to 80 characters",
        ));
    }
    let (client, token) = s
        .gate
        .clients
        .create(name, req.kind)
        .map_err(|e| Problem::new("internal", e.to_string()))?;
    let mut v =
        serde_json::to_value(client).map_err(|e| Problem::new("internal", e.to_string()))?;
    v["token"] = json!(token);
    Ok(Json(v))
}

async fn revoke_client(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, Problem> {
    let gone = s
        .gate
        .clients
        .revoke(&id)
        .map_err(|e| Problem::new("internal", e.to_string()))?;
    if gone {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(Problem::new("not-found", "no such deliver token"))
    }
}

async fn inbox(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({ "path": s.engine.inbox_dir() }))
}

// —— 从文件导入（协议 §3.9）——

async fn file_readers(State(s): State<AppState>) -> Json<Vec<FileReaderView>> {
    Json(s.engine.file_readers())
}

#[derive(Deserialize)]
struct FileImportQuery {
    /// `<插件 id>#<方法 id>`（`FileReader.id`）。
    reader: String,
    /// 交来的文件名，内核洗过才用。
    name: String,
}

/// 窗口交一个文件：`?reader=&name=`，请求体是文件本身。
async fn file_import(
    State(s): State<AppState>,
    q: Result<Query<FileImportQuery>, QueryRejection>,
    body: axum::body::Body,
) -> ApiResult<ImportView> {
    let Query(q) = file_query(q)?;
    Ok(Json(
        receive_file(&s.engine, q, body, usuyo_core::FILE_IMPORT_MAX, None).await?,
    ))
}

/// 浏览器脚本交一个装着已取到内容的文件（协议 §5.1）：只认 `extension` 类投递令牌（和主令牌），
/// 落成一次 `file` 类导入，等你在导入页确认。只回导入的 id：投递令牌读不了导入。
async fn hand_over(
    State(s): State<AppState>,
    Extension(caller): Extension<Caller>,
    q: Result<Query<FileImportQuery>, QueryRejection>,
    body: axum::body::Body,
) -> Result<(StatusCode, Json<serde_json::Value>), Problem> {
    let client = match &caller {
        Caller::Deliver(c) if c.kind == ClientKind::Extension => Some(c.name.clone()),
        Caller::Deliver(_) => {
            return Err(
                Problem::new("forbidden", "only extension tokens hand over content")
                    .with_hint(words::ONLY_EXTENSIONS_HAND_OVER.said()),
            );
        }
        _ => None,
    };
    let Query(q) = file_query(q)?;
    let view = receive_file(
        &s.engine,
        q,
        body,
        usuyo_core::HANDOVER_MAX,
        client.as_deref(),
    )
    .await?;
    Ok((StatusCode::ACCEPTED, Json(json!({ "id": view.id }))))
}

fn file_query(
    q: Result<Query<FileImportQuery>, QueryRejection>,
) -> Result<Query<FileImportQuery>, Problem> {
    q.map_err(|e| {
        Problem::new("invalid-args", e.body_text()).with_hint(words::BAD_FILE_QUERY.said())
    })
}

/// 把请求体一块块写进副本，数着大小；超了、断了都回错，副本随 `FileUpload` 去掉。写完交给插件读第一页。
async fn receive_file(
    engine: &Engine,
    q: FileImportQuery,
    body: axum::body::Body,
    max: u64,
    client: Option<&str>,
) -> Result<ImportView, Problem> {
    use tokio::io::AsyncWriteExt;
    let e = engine.clone();
    let upload = blocking(move || e.prepare_file_import(&q.reader, &q.name)).await?;
    let internal = |e: std::io::Error| Problem::new("internal", format!("upload copy: {e}"));
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(upload.path())
        .await
        .map_err(internal)?;
    let mut stream = body.into_data_stream();
    let mut total: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            Problem::new("invalid-args", format!("reading the body: {e}"))
                .with_hint(words::FILE_INCOMPLETE.said())
        })?;
        total = total.saturating_add(chunk.len() as u64);
        if total > max {
            return Err(
                Problem::new("too-large", format!("the file is larger than {max} bytes"))
                    .with_hint(words::FILE_TOO_LARGE.with(&[("max", &(max >> 20))])),
            );
        }
        file.write_all(&chunk).await.map_err(internal)?;
    }
    if total == 0 {
        return Err(
            Problem::new("invalid-args", "the file is empty").with_hint(words::FILE_EMPTY.said())
        );
    }
    file.sync_all().await.map_err(internal)?;
    drop(file);
    Ok(engine.finish_file_import(upload, client).await?)
}

// —— 投递接口（协议 §5.1）：投递令牌只能进这几条 ——

/// 谁交的：投递端按它的入口类型记，主令牌按 `api` 记。
fn deliverer(caller: &Caller) -> (Via, Option<String>) {
    match caller {
        Caller::Deliver(c) => (c.kind.via(), Some(c.name.clone())),
        _ => (Via::Api, None),
    }
}

async fn ping(Extension(caller): Extension<Caller>) -> Json<serde_json::Value> {
    let (_, client) = deliverer(&caller);
    Json(json!({
        "name": "usuyo",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": 1,
        "client": client,
    }))
}

#[derive(Deserialize)]
struct DeliverRequest {
    url: Option<String>,
    text: Option<String>,
}

/// 收下一条链接（或带链接的分享文字）。只收链接：已经取到的内容还不收。
async fn deliver(
    State(s): State<AppState>,
    Extension(caller): Extension<Caller>,
    req: Result<Json<DeliverRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<serde_json::Value>), Problem> {
    let req = body(req)?;
    let input = req
        .url
        .or(req.text)
        .ok_or_else(|| Problem::new("invalid-args", "give `url` or `text`"))?;
    let (via, _) = deliverer(&caller);
    let taken = s.engine.take(&input, "link", via).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"id": taken.id, "state": "accepted", "duplicate": taken.seen})),
    ))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchLink {
    url: String,
    title: Option<String>,
    acted_at: Option<String>,
}

#[derive(Deserialize)]
struct BatchRequest {
    #[serde(default)]
    name: String,
    action: Option<String>,
    links: Vec<BatchLink>,
}

const ACTIONS: [&str; 5] = ["favorite", "like", "share", "view", "unknown"];

/// 交一批链接：进导入的预览，你确认了才按节奏去收。
async fn batch(
    State(s): State<AppState>,
    Extension(caller): Extension<Caller>,
    req: Result<Json<BatchRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<serde_json::Value>), Problem> {
    let req = body(req)?;
    if let Some(a) = &req.action
        && !ACTIONS.contains(&a.as_str())
    {
        return Err(Problem::new(
            "invalid-args",
            format!("unknown action `{a}`"),
        ));
    }
    if req.links.is_empty() || req.links.len() > usuyo_core::BATCH_MAX {
        return Err(Problem::new(
            "invalid-args",
            format!("a batch has 1 to {} links", usuyo_core::BATCH_MAX),
        ));
    }
    let mut links = Vec::with_capacity(req.links.len());
    for (i, l) in req.links.into_iter().enumerate() {
        let bad = |why: &str| {
            Problem::new("invalid-args", format!("links[{i}]: {why}"))
                .with_hint(words::BAD_LINK_AT.with(&[("n", &(i + 1))]))
        };
        let url = usuyo_core::link_in(&l.url).ok_or_else(|| bad("not an http(s) link"))?;
        if !url.username().is_empty() || url.password().is_some() {
            return Err(bad("links with a user name or password are not taken"));
        }
        if let Some(t) = &l.acted_at
            && t.parse::<jiff::Timestamp>().is_err()
        {
            return Err(bad("`actedAt` is not RFC 3339"));
        }
        links.push(DeliveredLink {
            url,
            title: l.title.map(|t| t.chars().take(500).collect()),
            acted_at: l.acted_at,
        });
    }
    let (_, client) = deliverer(&caller);
    let view = blocking(move || {
        s.engine
            .deliver_batch(&req.name, req.action.as_deref(), links, client.as_deref())
    })
    .await?;
    Ok((StatusCode::ACCEPTED, Json(json!({ "id": view.id }))))
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
    // 窗口（`tauri://localhost`）和开发页面（`localhost:5173`）对 127.0.0.1 都是跨站，
    // 收紧成 same-site 图就出不来；能看到哪一条由按条目的 `k` 管。
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
