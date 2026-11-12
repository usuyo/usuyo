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
if [ "$1" = health ]; then echo '{"ok": true, "ready": true, "protocol": 1, "version": "1.0.0", "checks": [{"name": "tool", "ok": false, "message": "没找到要的程序", "methods": ["later-tool"]}]}'; exit 0; fi
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
visitors:
  - site: www.example.com
    name: 例子
    url: https://www.example.com/
methods:
  - id: page
    name: 公开页
    access: public-web
    login: none
    account: none
    capabilities:
      fetch:
        outputs: [text, html, images, meta]
  - id: visitor
    name: 访客页
    access: public-web
    login: none
    account: visitor
    visitors: [www.example.com]
    capabilities:
      fetch:
        outputs: [text]
  - id: later-tool
    name: 包一个工具
    access: public-web
    login: none
    account: none
    capabilities:
      fetch:
        outputs: [text]
  - id: signed
    name: 签名接口
    access: private-api
    login: none
    account: none
    capabilities:
      fetch:
        outputs: [text]
  - id: phone
    name: 手机
    built: false
    access: vision-ui
    login: required
    account: own-phone
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
            clients: None,
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

fn handshake() -> axum::http::request::Builder {
    get(usuyo_api::handshake::PATH)
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
}

#[tokio::test]
async fn handshake_proves_without_the_token() {
    let f = fixture();
    let nonce = usuyo_api::handshake::new_nonce().unwrap();
    let body = || Body::from(json!({ "nonce": nonce }).to_string());

    // 不带令牌也回证明；对的令牌核得上，错的核不上
    let res = send(&f.app, handshake().body(body()).unwrap()).await;
    assert_eq!(res.status(), StatusCode::OK);
    let answer = json_of(res).await;
    assert_eq!(
        answer.as_object().unwrap().len(),
        1,
        "only the proof: {answer}"
    );
    let proof = answer["proof"].as_str().unwrap();
    assert!(!proof.contains(TOKEN));
    assert!(usuyo_api::handshake::verify(TOKEN, PORT, &nonce, proof));
    assert!(!usuyo_api::handshake::verify(
        "other-token",
        PORT,
        &nonce,
        proof
    ));

    // 名单里的来源可以握手；别的网页不行
    let req = handshake()
        .header(header::ORIGIN, "tauri://localhost")
        .body(body())
        .unwrap();
    assert_eq!(send(&f.app, req).await.status(), StatusCode::OK);
    for origin in ["http://evil.test", "null"] {
        let req = handshake()
            .header(header::ORIGIN, origin)
            .body(body())
            .unwrap();
        problem(send(&f.app, req).await, StatusCode::FORBIDDEN, "forbidden").await;
    }
    // Host 照样核对
    let req = Request::post(usuyo_api::handshake::PATH)
        .header(header::HOST, "evil.test")
        .header(header::CONTENT_TYPE, "application/json")
        .body(body())
        .unwrap();
    problem(send(&f.app, req).await, StatusCode::FORBIDDEN, "forbidden").await;

    // 请求体有上限；随机数要合格
    let big = json!({ "nonce": "0".repeat(4096) }).to_string();
    let req = handshake().body(Body::from(big)).unwrap();
    problem(
        send(&f.app, req).await,
        StatusCode::PAYLOAD_TOO_LARGE,
        "too-large",
    )
    .await;
    for bad in ["abc", &"Z".repeat(64)] {
        let req = handshake()
            .body(Body::from(json!({ "nonce": bad }).to_string()))
            .unwrap();
        problem(
            send(&f.app, req).await,
            StatusCode::BAD_REQUEST,
            "invalid-args",
        )
        .await;
    }
    // 只有 POST 不带令牌；GET 照样要令牌
    let req = get(usuyo_api::handshake::PATH).body(Body::empty()).unwrap();
    assert_eq!(send(&f.app, req).await.status(), StatusCode::UNAUTHORIZED);
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
    // 一条的 k 只管这一条：拿去读另一条的图不行
    let req = get("/items")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({"input": "https://example.test/b", "kind": "link"}).to_string(),
        ))
        .unwrap();
    let other = json_of(send(&f.app, req).await).await["id"]
        .as_str()
        .unwrap()
        .to_owned();
    f.engine.run_due().await.unwrap();
    let other_item = json_of(send(&f.app, authed(&format!("/items/{other}"))).await).await;
    let other_url = other_item["facets"][0]["images"][0]["url"]
        .as_str()
        .unwrap();
    let other_path = other_url.split('?').next().unwrap();
    assert_ne!(other_url.split("?k=").nth(1).unwrap(), k);
    problem(
        send(
            &f.app,
            get(&format!(
                "{}?k={k}",
                other_path
                    .strip_prefix(&format!("http://127.0.0.1:{PORT}"))
                    .unwrap()
            ))
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

#[tokio::test]
async fn purge_now_removes_a_trash_entry() {
    let f = fixture();
    let post = |path: &str| {
        get(path)
            .method("POST")
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .body(Body::empty())
            .unwrap()
    };
    let taken = f
        .engine
        .take("https://example.test/a", "link", Via::Api)
        .await
        .unwrap();
    let res = send(&f.app, post(&format!("/items/{}/trash", taken.id))).await;
    assert_eq!(res.status(), StatusCode::OK);
    let trash = f.engine.library_root().join(".usuyo/trash");
    let tid = std::fs::read_dir(&trash)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .file_name()
        .into_string()
        .unwrap();
    let listed = json_of(send(&f.app, authed("/trash")).await).await;
    assert_eq!(listed["trashDays"], 30);
    let row = &listed["entries"][0];
    assert_eq!(
        (&row["id"], &row["reason"], &row["itemId"]),
        (&json!(tid), &json!("delete"), &json!(taken.id))
    );
    assert!(row["trashedAt"].is_string() && row["expiresAt"].is_string());

    let path = format!("/trash/{tid}/purge");
    assert_eq!(
        send(&f.app, post(&path)).await.status(),
        StatusCode::NO_CONTENT
    );
    assert!(!trash.join(&tid).exists());
    let listed = json_of(send(&f.app, authed("/trash")).await).await;
    assert_eq!(listed["entries"], json!([]));
    problem(
        send(&f.app, authed(&format!("/items/{}", taken.id))).await,
        StatusCode::NOT_FOUND,
        "not-found",
    )
    .await;
    // 已经没有了：再清一次是 404，不是成功。
    problem(
        send(&f.app, post(&path)).await,
        StatusCode::NOT_FOUND,
        "not-found",
    )
    .await;
}

#[tokio::test]
async fn visitors_are_listed_stored_and_cleared_apart_from_logins() {
    let f = fixture();
    let v = json_of(send(&f.app, authed("/visitors")).await).await;
    assert_eq!(v[0]["site"], "www.example.com");
    assert_eq!(v[0]["state"], "missing");
    assert_eq!(v[0]["plugins"], json!(["fake/web"]));
    // 访客身份不是账号：不出现在 /sites 里
    assert_eq!(
        json_of(send(&f.app, authed("/sites")).await).await,
        json!([])
    );
    let plugins = json_of(send(&f.app, authed("/plugins")).await).await;
    assert_eq!(
        plugins["plugins"][0]["visitors"],
        json!(["www.example.com"])
    );

    let put = |body: String| {
        get("/visitors/www.example.com")
            .method("PUT")
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap()
    };
    // 形状不对时回的错误里没有值
    let bad = json!({"cookies": [{"name": "vid", "value": ["visitor-secret-1"], "domain": ".example.com"}]});
    let res = send(&f.app, put(bad.to_string())).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let text = json_of(res).await.to_string();
    assert!(!text.contains("visitor-secret-1"), "{text}");

    let good = json!({"cookies": [
        {"name": "vid", "value": "visitor-secret-1", "domain": ".example.com", "expires": 4102444800i64}]});
    let res = send(&f.app, put(good.to_string())).await;
    assert_eq!(res.status(), StatusCode::OK);
    let saved = json_of(res).await;
    assert_eq!(saved["state"], "present");
    assert_eq!(saved["expiresAt"], "2100-01-01T00:00:00.000Z");
    assert!(!saved.to_string().contains("visitor-secret-1"));
    let listed = json_of(send(&f.app, authed("/visitors")).await).await;
    assert!(!listed.to_string().contains("visitor-secret-1"));
    assert_eq!(
        json_of(send(&f.app, authed("/sites")).await).await,
        json!([])
    );

    let delete = get("/visitors/www.example.com")
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&f.app, delete).await.status(), StatusCode::NO_CONTENT);
    let v = json_of(send(&f.app, authed("/visitors")).await).await;
    assert_eq!(v[0]["state"], "missing");
}

fn with_body(method: &str, path: &str, body: Value) -> Request<Body> {
    get(path)
        .method(method)
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn transforms_settings_and_the_transform_route() {
    let f = fixture();
    // 没装转换插件：列表是空的
    let t = json_of(send(&f.app, authed("/transforms")).await).await;
    assert_eq!(t, json!({"plugins": [], "auto": true}));
    problem(
        send(
            &f.app,
            with_body(
                "POST",
                "/transforms/builtin/asr-sherpa/models/x/download",
                json!({}),
            ),
        )
        .await,
        StatusCode::NOT_FOUND,
        "not-found",
    )
    .await;

    let taken = f
        .engine
        .take("https://example.test/a", "link", Via::Api)
        .await
        .unwrap();
    f.engine.run_due().await.unwrap();
    let path = format!("/items/{}/transform", taken.id);
    let res = send(&f.app, with_body("POST", &path, json!({"jobs": "pending"}))).await;
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    assert_eq!(json_of(res).await, json!({"queued": true}));
    for bad in [
        json!({}),
        json!({"jobs": "all"}),
        json!({"jobs": "pending", "job": "asr"}),
        json!({"job": "nope"}),
        json!({"job": "asr"}),
    ] {
        problem(
            send(&f.app, with_body("POST", &path, bad)).await,
            StatusCode::BAD_REQUEST,
            "invalid-args",
        )
        .await;
    }
    let item = json_of(send(&f.app, authed(&format!("/items/{}", taken.id))).await).await;
    assert!(item["pending"].is_array());

    // 设置：读、改、坏的回 400
    let s = json_of(send(&f.app, authed("/settings")).await).await;
    assert_eq!(s["trashDays"], 30);
    assert_eq!(s["network"]["mode"], "system");
    assert_eq!(s["transforms"]["auto"], true);
    assert!(s["logs"].as_str().unwrap().ends_with("logs/daemon.log"));
    assert!(s.get("trashHeld").is_none());
    let res = send(
        &f.app,
        with_body(
            "PATCH",
            "/settings",
            json!({"trashDays": 90, "network": {"mode": "direct"}}),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let s = json_of(res).await;
    assert_eq!(s["trashDays"], 90);
    assert_eq!(s["network"]["mode"], "direct");
    problem(
        send(
            &f.app,
            with_body("PATCH", "/settings", json!({"color": "red"})),
        )
        .await,
        StatusCode::BAD_REQUEST,
        "invalid-args",
    )
    .await;

    // 没拦下什么：照时间移除也没有可移除的
    let res = send(&f.app, with_body("POST", "/trash/release-held", json!({}))).await;
    assert_eq!(json_of(res).await, json!({"removed": 0}));

    // 窗口能发 PATCH 的预检
    let pre = get("/settings")
        .method("OPTIONS")
        .header(header::ORIGIN, "tauri://localhost")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "PATCH")
        .body(Body::empty())
        .unwrap();
    let res = send(&f.app, pre).await;
    assert_eq!(res.status(), StatusCode::OK);
    let allowed = res
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_METHODS)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    assert!(allowed.contains("PATCH"), "{allowed}");
}

#[tokio::test]
async fn plugins_list_every_method_with_what_blocks_it_and_switches_one() {
    let f = fixture();
    let plugins = json_of(send(&f.app, authed("/plugins")).await).await;
    let p = &plugins["plugins"][0];
    // 顶层的层级由做了的方法算；没做的手机不算
    assert_eq!(p["access"], "private-api");
    let methods = p["methods"].as_array().unwrap();
    let by = |id: &str| methods.iter().find(|m| m["id"] == id).unwrap().clone();
    assert_eq!(methods.len(), 5);
    let page = by("page");
    assert_eq!(
        (page["usable"].clone(), page["enabled"].clone()),
        (json!(true), json!(true))
    );
    assert_eq!(page["account"], json!({"kind": "none"}));
    assert_eq!(
        page["gets"]["fetch"],
        json!(["text", "html", "images", "meta"])
    );
    let visitor = by("visitor");
    assert_eq!(
        visitor["account"],
        json!({"kind": "visitor", "sites": ["www.example.com"], "where": "例子"})
    );
    assert_eq!(visitor["usable"], false);
    assert_eq!(visitor["blockers"][0]["code"], "visitor-missing");
    // 插件的本机检查只挡它点名的方法
    let tool = by("later-tool");
    assert_eq!(
        tool["blockers"],
        json!([{"code": "check-failed", "hint": "没找到要的程序"}])
    );
    // private-api 默认关着；没做的开不了
    let signed = by("signed");
    assert_eq!(
        (
            signed["enabled"].clone(),
            signed["blockers"][0]["code"].clone()
        ),
        (json!(false), json!("off"))
    );
    let phone = by("phone");
    assert_eq!(
        (phone["built"].clone(), phone["blockers"][0]["code"].clone()),
        (json!(false), json!("not-built"))
    );

    let switch = |method: &str, body: &str| {
        get(&format!("/plugins/fake/web/methods/{method}"))
            .method("PUT")
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    };
    let res = send(&f.app, switch("signed", r#"{"enabled": true}"#)).await;
    assert_eq!(res.status(), StatusCode::OK);
    let p = json_of(res).await;
    let signed = p["methods"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == "signed")
        .unwrap()
        .clone();
    assert_eq!(
        (signed["enabled"].clone(), signed["usable"].clone()),
        (json!(true), json!(true))
    );
    let res = send(&f.app, switch("page", r#"{"enabled": false}"#)).await;
    assert_eq!(res.status(), StatusCode::OK);
    problem(
        send(&f.app, switch("phone", r#"{"enabled": true}"#)).await,
        StatusCode::UNPROCESSABLE_ENTITY,
        "unsupported-input",
    )
    .await;
    problem(
        send(&f.app, switch("nope", r#"{"enabled": true}"#)).await,
        StatusCode::NOT_FOUND,
        "not-found",
    )
    .await;
    problem(
        send(&f.app, switch("page", r#"{"on": true}"#)).await,
        StatusCode::BAD_REQUEST,
        "invalid-args",
    )
    .await;

    // 开关写进按机器的 settings.json
    let settings: serde_json::Value = serde_json::from_slice(
        &std::fs::read(f._tmp.path().join("machine/settings.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        settings["plugins"]["fake/web"]["methods"],
        json!({"signed": {"enabled": true}, "page": {"enabled": false}})
    );
    // 来源要说是哪种方法；不能列的方法和坏链接回 4xx
    problem(
        send(&f.app, authed("/plugins/fake/web/sources")).await,
        StatusCode::BAD_REQUEST,
        "invalid-args",
    )
    .await;
    problem(
        send(&f.app, authed("/plugins/fake/web/sources?method=page")).await,
        StatusCode::UNPROCESSABLE_ENTITY,
        "unsupported-input",
    )
    .await;
    problem(
        send(
            &f.app,
            authed("/plugins/fake/web/sources?method=page&link=ftp%3A%2F%2Fa.test%2F"),
        )
        .await,
        StatusCode::BAD_REQUEST,
        "invalid-args",
    )
    .await;
}
