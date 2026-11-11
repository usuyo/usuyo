//! 接口的门（Host、Origin、令牌、媒体的 k）、Range、problem details、事件流。
//! 直接对着路由跑，不开端口、不碰网络；假插件是 sh 脚本。
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;
use usuyo_api::{ApiConfig, default_origins, router};
use usuyo_core::{Engine, EngineConfig, Via};

const PORT: u16 = 47615;
const TOKEN: &str = "test-token";

const PLUGIN: &str = r#"#!/bin/sh
if [ "$1" = canonicalize ]; then echo '{"ok": true, "match": false}'; exit 0; fi
out="$4"
mkdir -p "$out/images" && printf '0123456789' > "$out/images/001.png"
printf '<script>alert(1)</script>' > "$out/page.html"
cat <<JSON
{"ok": true, "kind": "article", "title": "<b>标题</b>", "text": "正文里有闷蒸。",
 "html": "page.html", "images": ["images/001.png"], "media": [], "platformTags": [],
 "completeness": "full", "missing": [],
 "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}
JSON
"#;

const MANIFEST: &str = r#"protocol: 1
id: fake/web
kind: fetch
name: 网页
version: 1.0.0
license: MIT
maintainers: [someone]
platforms: [web]
match: ["*"]
access: public-web
login: none
capabilities:
  fetch:
    access: public-web
    login: none
    outputs: [text, html, images, meta]
permissions:
  fs: {write: ["${OUT}"]}
run:
  command: ["${PLUGIN_DIR}/plugin.sh"]
"#;

struct Fixture {
    _tmp: TempDir,
    engine: Engine,
    app: Router,
}

fn fixture() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let machine = tmp.path().join("machine");
    let dir = machine.join("plugins/fake/web/1.0.0");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("usuyo-plugin.yaml"), MANIFEST).unwrap();
    std::fs::write(dir.join("plugin.sh"), PLUGIN).unwrap();
    std::fs::set_permissions(
        dir.join("plugin.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let lib = Engine::init_library(&tmp.path().join("lib"), &machine).unwrap();
    let engine = Engine::open(EngineConfig::new(lib, &machine)).unwrap();
    let app = router(
        engine.clone(),
        ApiConfig {
            port: PORT,
            token: TOKEN.into(),
            origins: default_origins(),
        },
    )
    .unwrap();
    Fixture {
        _tmp: tmp,
        engine,
        app,
    }
}

fn get(path: &str) -> axum::http::request::Builder {
    Request::get(path).header(header::HOST, format!("127.0.0.1:{PORT}"))
}

fn authed(path: &str) -> Request<Body> {
    get(path)
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .body(Body::empty())
        .unwrap()
}

async fn send(app: &Router, req: Request<Body>) -> Response<Body> {
    app.clone().oneshot(req).await.unwrap()
}

async fn json_of(res: Response<Body>) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn problem(res: Response<Body>, status: StatusCode, code: &str) {
    assert_eq!(res.status(), status);
    assert_eq!(
        res.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    let p = json_of(res).await;
    assert_eq!(p["code"], code);
    assert_eq!(p["type"], format!("urn:usuyo:problem:{code}"));
    assert_eq!(p["status"], status.as_u16());
    assert!(p["instance"].as_str().unwrap().starts_with("urn:uuid:"));
    assert!(p["title"].is_string() && p["detail"].is_string());
}

#[tokio::test]
async fn host_origin_and_token_rules() {
    let f = fixture();
    // 令牌对了才进
    assert_eq!(
        send(&f.app, authed("/health")).await.status(),
        StatusCode::OK
    );
    let anon = get("/health").body(Body::empty()).unwrap();
    problem(
        send(&f.app, anon).await,
        StatusCode::UNAUTHORIZED,
        "unauthorized",
    )
    .await;
    let wrong = get("/health")
        .header(header::AUTHORIZATION, "Bearer nope")
        .body(Body::empty())
        .unwrap();
    problem(
        send(&f.app, wrong).await,
        StatusCode::UNAUTHORIZED,
        "unauthorized",
    )
    .await;

    // DNS rebinding：Host 不是本机这个端口
    for host in ["evil.test", "127.0.0.1:1", "evil.test:47615"] {
        let req = Request::get("/health")
            .header(header::HOST, host)
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .body(Body::empty())
            .unwrap();
        problem(send(&f.app, req).await, StatusCode::FORBIDDEN, "forbidden").await;
    }

    // 名单里的来源不带令牌：Origin 谁都能伪造，不算数
    let req = get("/map")
        .header(header::ORIGIN, "tauri://localhost")
        .body(Body::empty())
        .unwrap();
    problem(
        send(&f.app, req).await,
        StatusCode::UNAUTHORIZED,
        "unauthorized",
    )
    .await;

    // 名单里的来源带令牌：能进，带上 CORS 头
    for origin in ["tauri://localhost", "http://tauri.localhost"] {
        let req = get("/map")
            .header(header::ORIGIN, origin)
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .body(Body::empty())
            .unwrap();
        let res = send(&f.app, req).await;
        assert_eq!(res.status(), StatusCode::OK, "{origin}");
        assert_eq!(res.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], origin);
        assert_eq!(json_of(res).await, json!({"regions": [], "points": []}));
    }

    // 别的网页，包括开发服务器的默认端口：带了令牌也不行
    for origin in [
        "http://evil.test",
        "null",
        "http://localhost:5173",
        "http://127.0.0.1:5173",
        "https://tauri.localhost",
    ] {
        let req = get("/map")
            .header(header::ORIGIN, origin)
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .body(Body::empty())
            .unwrap();
        problem(send(&f.app, req).await, StatusCode::FORBIDDEN, "forbidden").await;
    }

    // 预检不带令牌，名单里的来源才放行
    let pre = |origin: &str| {
        Request::options("/items")
            .header(header::HOST, format!("localhost:{PORT}"))
            .header(header::ORIGIN, origin)
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
            .header(
                header::ACCESS_CONTROL_REQUEST_HEADERS,
                "content-type, authorization",
            )
            .body(Body::empty())
            .unwrap()
    };
    let res = send(&f.app, pre("tauri://localhost")).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "tauri://localhost"
    );
    for origin in ["http://evil.test", "http://localhost:5173"] {
        assert_eq!(
            send(&f.app, pre(origin)).await.status(),
            StatusCode::FORBIDDEN
        );
    }
    // 不是预检的 OPTIONS 照样要令牌
    let bare = Request::options("/items")
        .header(header::HOST, format!("localhost:{PORT}"))
        .header(header::ORIGIN, "tauri://localhost")
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&f.app, bare).await.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn take_then_media_with_range() {
    let f = fixture();
    let req = get("/items")
        .method("POST")
        .header(header::ORIGIN, "tauri://localhost")
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({"input": "https://example.test/a", "kind": "link"}).to_string(),
        ))
        .unwrap();
    let res = send(&f.app, req).await;
    assert_eq!(res.status(), StatusCode::OK);
    let taken = json_of(res).await;
    assert_eq!(taken["seen"], false);
    assert_eq!(taken["platform"], json!({"id": "web", "name": "网页"}));
    let id = taken["id"].as_str().unwrap().to_owned();
    f.engine.run_due().await.unwrap();

    let item = json_of(send(&f.app, authed(&format!("/items/{id}"))).await).await;
    assert_eq!(item["stored"], "complete");
    // 图片尺寸照文件量（这张不是真的 PNG，量不出就是 0）
    assert_eq!(item["facets"][0]["images"][0]["width"], 0);
    // 平台来的标题原样当字给出，不渲染
    assert_eq!(item["title"], "<b>标题</b>");
    let url = item["facets"][0]["images"][0]["url"].as_str().unwrap();
    let path = url
        .strip_prefix(&format!("http://127.0.0.1:{PORT}"))
        .unwrap();
    assert!(path.contains("?k="));

    // <img> 那样的请求：没有 Origin、没有令牌，只有 k
    let req = get(path)
        .header(header::RANGE, "bytes=2-5")
        .body(Body::empty())
        .unwrap();
    let res = send(&f.app, req).await;
    assert_eq!(res.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(res.headers()[header::CONTENT_RANGE], "bytes 2-5/10");
    assert_eq!(res.headers()[header::CONTENT_TYPE], "image/png");
    assert_eq!(
        res.headers()[header::CONTENT_SECURITY_POLICY],
        "default-src 'none'; sandbox"
    );
    assert_eq!(res.headers()[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&bytes[..], b"2345");

    let whole = send(&f.app, get(path).body(Body::empty()).unwrap()).await;
    assert_eq!(whole.status(), StatusCode::OK);
    assert_eq!(whole.headers()[header::ACCEPT_RANGES], "bytes");

    // k 不对、或拿 k 去读别的接口
    let bad = format!("{}?k=nope", path.split('?').next().unwrap());
    problem(
        send(&f.app, get(&bad).body(Body::empty()).unwrap()).await,
        StatusCode::UNAUTHORIZED,
        "unauthorized",
    )
    .await;
    let k = path.split("?k=").nth(1).unwrap();
    problem(
        send(
            &f.app,
            get(&format!("/items/{id}?k={k}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await,
        StatusCode::UNAUTHORIZED,
        "unauthorized",
    )
    .await;
    // 带 k 的媒体地址，来源不在名单里也不行
    let req = get(path)
        .header(header::ORIGIN, "http://evil.test")
        .body(Body::empty())
        .unwrap();
    problem(send(&f.app, req).await, StatusCode::FORBIDDEN, "forbidden").await;
    // 网页原文不是媒体层，不经这里给出
    problem(
        send(&f.app, authed(&format!("/media/{id}/text"))).await,
        StatusCode::BAD_REQUEST,
        "invalid-args",
    )
    .await;
}

#[tokio::test]
async fn errors_are_problem_details() {
    let f = fixture();
    problem(
        send(&f.app, authed("/items/not-an-id")).await,
        StatusCode::BAD_REQUEST,
        "invalid-args",
    )
    .await;
    problem(
        send(&f.app, authed("/items/web-aaaaaaaaaaaaaaaaaaaaaaaaaa")).await,
        StatusCode::NOT_FOUND,
        "not-found",
    )
    .await;
    problem(
        send(&f.app, authed("/nope")).await,
        StatusCode::NOT_FOUND,
        "not-found",
    )
    .await;
    let post = |body: Body| {
        get("/items")
            .method("POST")
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .unwrap()
    };
    problem(
        send(&f.app, post(Body::from("{oops"))).await,
        StatusCode::BAD_REQUEST,
        "invalid-args",
    )
    .await;
    let huge = json!({"input": "x".repeat(100 << 10), "kind": "link"}).to_string();
    problem(
        send(&f.app, post(Body::from(huge))).await,
        StatusCode::PAYLOAD_TOO_LARGE,
        "too-large",
    )
    .await;
    let file = json!({"input": "/tmp/a.mp4", "kind": "file"}).to_string();
    problem(
        send(&f.app, post(Body::from(file))).await,
        StatusCode::UNPROCESSABLE_ENTITY,
        "unsupported-input",
    )
    .await;
    let delete = get("/items/web-aaaaaaaaaaaaaaaaaaaaaaaaaa")
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .body(Body::empty())
        .unwrap();
    problem(
        send(&f.app, delete).await,
        StatusCode::METHOD_NOT_ALLOWED,
        "invalid-args",
    )
    .await;
    problem(
        send(&f.app, authed("/search?q=")).await,
        StatusCode::BAD_REQUEST,
        "invalid-args",
    )
    .await;
}

#[tokio::test]
async fn events_stream_announces_takes() {
    let f = fixture();
    let res = send(&f.app, authed("/events")).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()[header::CONTENT_TYPE], "text/event-stream");
    let mut body = res.into_body();
    f.engine
        .take("https://example.test/a", "link", Via::Api)
        .await
        .unwrap();
    let frame = tokio::time::timeout(Duration::from_secs(5), body.frame())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let text = String::from_utf8(frame.into_data().unwrap().to_vec()).unwrap();
    assert!(text.starts_with("event: item-added\n"), "{text}");
    assert!(text.contains(r#""type":"item-added""#), "{text}");
}

#[tokio::test]
async fn create_topic_then_put_an_item_in_it() {
    let f = fixture();
    let post = |path: &str, body: Value| {
        get(path)
            .method("POST")
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    };
    let res = send(&f.app, post("/topics", json!({"name": "手冲咖啡"}))).await;
    assert_eq!(res.status(), StatusCode::OK);
    let topic = json_of(res).await;
    assert_eq!(topic["name"], "手冲咖啡");
    assert_eq!(topic["followed"], false);
    let again = json_of(send(&f.app, post("/topics", json!({"name": "手冲咖啡"}))).await).await;
    assert_eq!(again["id"], topic["id"]);
    problem(
        send(&f.app, post("/topics", json!({"name": " "}))).await,
        StatusCode::BAD_REQUEST,
        "invalid-args",
    )
    .await;

    let taken = f
        .engine
        .take("https://example.test/a", "link", Via::Api)
        .await
        .unwrap();
    let path = format!("/items/{}/topics", taken.id);
    let item = json_of(
        send(
            &f.app,
            post(&path, json!({"topicId": topic["id"], "member": true})),
        )
        .await,
    )
    .await;
    assert_eq!(item["topicIds"], json!([topic["id"]]));
    let health = json_of(send(&f.app, authed("/health")).await).await;
    assert!(health["library"].as_str().unwrap().ends_with("lib"));
}
