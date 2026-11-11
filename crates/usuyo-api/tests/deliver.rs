//! 投递接口和投递令牌、握手认投递端、`/v1/` 的 Origin 规则、登录站点的路由。
//! 直接对着路由跑，不开端口、不碰网络；假插件是 sh 脚本；钥匙串是按机器目录里的文件（引擎的默认）。
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;
use usuyo_api::{ApiConfig, default_origins, handshake, router};
use usuyo_core::{Engine, EngineConfig};

const PORT: u16 = 47999;
const TOKEN: &str = "owner-token";

const PLUGIN: &str = r#"#!/bin/sh
if [ "$1" = canonicalize ]; then echo '{"ok": true, "match": false}'; exit 0; fi
echo '{"ok": false, "error": {"code": "network", "message": "offline in tests"}}'; exit 1
"#;

const MANIFEST: &str = r#"protocol: 1
id: fake/site
kind: fetch
name: 例子
version: 1.0.0
license: MIT
maintainers: [someone]
platforms: [web]
match: ["*"]
access: session-api
login: optional
sites:
  - {site: www.example.test, name: 例子, credential: cookies, loginUrl: "https://www.example.test/login", cookies: [sid], domains: [".example.test"]}
capabilities:
  fetch:
    access: session-api
    login: optional
    sites: [www.example.test]
    outputs: [text, meta]
permissions:
  fs: {write: ["${OUT}"]}
run:
  command: ["${PLUGIN_DIR}/plugin.sh"]
"#;

struct Fixture {
    tmp: TempDir,
    app: Router,
}

fn fixture() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let machine = tmp.path().join("machine");
    let dir = machine.join("plugins/fake/site/1.0.0");
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
        engine,
        ApiConfig {
            port: PORT,
            token: TOKEN.into(),
            origins: default_origins(),
            clients: Some(machine.join("deliver-tokens.json")),
        },
    )
    .unwrap();
    Fixture { tmp, app }
}

fn req(method: &str, path: &str, token: Option<&str>, body: Option<Value>) -> Request<Body> {
    let mut b = Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, format!("127.0.0.1:{PORT}"));
    if let Some(t) = token {
        b = b.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    match body {
        Some(v) => b
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => b.body(Body::empty()).unwrap(),
    }
}

async fn send(app: &Router, r: Request<Body>) -> Response<Body> {
    app.clone().oneshot(r).await.unwrap()
}

async fn json_of(res: Response<Body>) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

async fn new_token(app: &Router, kind: &str) -> (String, String) {
    let res = send(
        app,
        req(
            "POST",
            "/deliver-tokens",
            Some(TOKEN),
            Some(json!({"name": "xhs 脚本", "kind": kind})),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let v = json_of(res).await;
    (
        v["id"].as_str().unwrap().into(),
        v["token"].as_str().unwrap().into(),
    )
}

#[tokio::test]
async fn a_deliver_token_hands_in_links_and_reads_nothing() {
    let f = fixture();
    let (id, token) = new_token(&f.app, "extension").await;
    assert!(token.starts_with(&format!("{id}.")));

    let ping = send(&f.app, req("GET", "/v1/ping", Some(&token), None)).await;
    assert_eq!(ping.status(), StatusCode::OK);
    let v = json_of(ping).await;
    assert_eq!(
        (v["name"].as_str(), v["client"].as_str()),
        (Some("usuyo"), Some("xhs 脚本"))
    );

    for path in [
        "/map",
        "/plugins",
        "/sites",
        "/imports",
        "/deliver-tokens",
        "/search?q=x",
        "/health",
    ] {
        let res = send(&f.app, req("GET", path, Some(&token), None)).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "{path}");
    }
    let res = send(
        &f.app,
        req(
            "POST",
            "/items",
            Some(&token),
            Some(json!({"input": "https://a.test/", "kind": "link"})),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let res = send(
        &f.app,
        req(
            "POST",
            "/v1/deliver",
            Some(&token),
            Some(json!({"text": "看这个 https://a.test/x 复制打开"})),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let v = json_of(res).await;
    assert_eq!(v["state"], "accepted");
    assert_eq!(v["duplicate"], false);
    let item = json_of(
        send(
            &f.app,
            req(
                "GET",
                &format!("/items/{}", v["id"].as_str().unwrap()),
                Some(TOKEN),
                None,
            ),
        )
        .await,
    )
    .await;
    assert_eq!(item["url"], "https://a.test/x");
    let manifest_path = f
        .tmp
        .path()
        .join("lib/items/web")
        .join(item["id"].as_str().unwrap())
        .join("manifest.json");
    let m: Value = serde_json::from_slice(&std::fs::read(manifest_path).unwrap()).unwrap();
    assert_eq!(m["origins"][0]["entry"], "extension");

    let res = send(
        &f.app,
        req(
            "POST",
            "/v1/batches",
            Some(&token),
            Some(json!({"name": "小红书收藏夹", "action": "favorite", "links": [
                {"url": "https://a.test/1", "title": "一", "actedAt": "2026-10-30T21:00:00+08:00"},
                {"url": "https://a.test/2"}
            ]})),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let batch = json_of(res).await["id"].as_str().unwrap().to_owned();
    let imp = json_of(
        send(
            &f.app,
            req("GET", &format!("/imports/{batch}"), Some(TOKEN), None),
        )
        .await,
    )
    .await;
    assert_eq!(
        (imp["kind"].as_str(), imp["client"].as_str()),
        (Some("delivered"), Some("xhs 脚本"))
    );
    assert_eq!(imp["state"], "listing");

    // 确认要说清两组各收不收；例外按 `seq` 给
    let confirm = format!("/imports/{batch}/confirm");
    let res = send(&f.app, req("POST", &confirm, Some(TOKEN), Some(json!({})))).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(json_of(res).await.to_string().contains("new"));
    let choice = json!({"new": true, "trashed": false, "except": [1]});
    let res = send(&f.app, req("POST", &confirm, Some(TOKEN), Some(choice))).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(json_of(res).await.to_string().contains("still listing"));

    // 坏的一批整个不收
    let res = send(
        &f.app,
        req(
            "POST",
            "/v1/batches",
            Some(&token),
            Some(json!({"links": [{"url": "ftp://a.test/"}]})),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let res = send(
        &f.app,
        req(
            "POST",
            "/v1/batches",
            Some(&token),
            Some(json!({"links": [{"url": "https://a.test/", "actedAt": "昨天"}]})),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn unknown_and_revoked_tokens_are_refused() {
    let f = fixture();
    let (id, token) = new_token(&f.app, "mobile").await;
    for bad in [
        "nope",
        "0123456789abcdef.wrong",
        TOKEN.trim_end_matches('n'),
    ] {
        let res = send(&f.app, req("GET", "/v1/ping", Some(bad), None)).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "{bad}");
    }
    let list = json_of(send(&f.app, req("GET", "/deliver-tokens", Some(TOKEN), None)).await).await;
    assert_eq!(list[0]["id"], id);
    assert!(
        list[0].get("token").is_none()
            && !list.to_string().contains(token.split_once('.').unwrap().1)
    );
    let res = send(
        &f.app,
        req(
            "DELETE",
            &format!("/deliver-tokens/{id}"),
            Some(TOKEN),
            None,
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let res = send(&f.app, req("GET", "/v1/ping", Some(&token), None)).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "撤销马上生效");
    // 令牌文件只给自己读
    let mode = std::fs::metadata(f.tmp.path().join("machine/deliver-tokens.json"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[tokio::test]
async fn the_handshake_proves_the_listener_to_a_deliver_client() {
    let f = fixture();
    let (id, token) = new_token(&f.app, "extension").await;
    let nonce = handshake::new_nonce().unwrap();
    let res = send(
        &f.app,
        req(
            "POST",
            "/handshake",
            None,
            Some(json!({"nonce": nonce, "client": id})),
        ),
    )
    .await;
    let proof = json_of(res).await["proof"].as_str().unwrap().to_owned();
    assert!(
        handshake::verify(&token, PORT, &nonce, &proof),
        "投递端用自己的令牌核对"
    );
    assert!(!handshake::verify(TOKEN, PORT, &nonce, &proof));
    let res = send(
        &f.app,
        req(
            "POST",
            "/handshake",
            None,
            Some(json!({"nonce": nonce, "client": "ffffffffffffffff"})),
        ),
    )
    .await;
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "不认识的 id 也照样答，分不出在不在"
    );
}

#[tokio::test]
async fn v1_takes_extension_origins_only() {
    let f = fixture();
    let (_, token) = new_token(&f.app, "extension").await;
    let with_origin = |origin: &str| {
        let mut r = req("GET", "/v1/ping", Some(&token), None);
        r.headers_mut()
            .insert(header::ORIGIN, origin.parse().unwrap());
        r
    };
    for ok in ["chrome-extension://abcdef", "moz-extension://1234-5678"] {
        let res = send(&f.app, with_origin(ok)).await;
        assert_eq!(res.status(), StatusCode::OK, "{ok}");
        assert!(
            res.headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_none(),
            "不回 CORS 头"
        );
    }
    for bad in [
        "http://evil.test",
        "null",
        "tauri://localhost",
        "https://www.xiaohongshu.com",
    ] {
        assert_eq!(
            send(&f.app, with_origin(bad)).await.status(),
            StatusCode::FORBIDDEN,
            "{bad}"
        );
    }
}

#[tokio::test]
async fn login_routes_store_cookies_and_never_answer_with_values() {
    let f = fixture();
    let sites = json_of(send(&f.app, req("GET", "/sites", Some(TOKEN), None)).await).await;
    assert_eq!(sites[0]["site"], "www.example.test");
    assert_eq!(sites[0]["loginUrl"], "https://www.example.test/login");
    assert_eq!(sites[0]["login"], "missing");
    let body = json!({"cookies": [
        {"name": "sid", "value": "s3cret-cookie", "domain": ".example.test", "path": "/", "httpOnly": true, "secure": true, "sameSite": "Lax", "expires": 4102444800.0},
        {"name": "tracker", "value": "other-site", "domain": ".elsewhere.test"}
    ], "origins": []});
    let res = send(
        &f.app,
        req(
            "PUT",
            "/sites/www.example.test/login",
            Some(TOKEN),
            Some(body),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let v = json_of(res).await;
    assert_eq!(v["login"], "present");
    assert!(!v.to_string().contains("s3cret-cookie"));
    let all = json_of(send(&f.app, req("GET", "/sites", Some(TOKEN), None)).await)
        .await
        .to_string()
        + &json_of(send(&f.app, req("GET", "/plugins", Some(TOKEN), None)).await)
            .await
            .to_string();
    assert!(!all.contains("s3cret-cookie"));
    // 别的站点的 cookie 没存
    let stored = std::fs::read_to_string(f.tmp.path().join("machine/secrets.json")).unwrap();
    assert!(stored.contains("s3cret-cookie") && !stored.contains("other-site"));
    let index = std::fs::read_to_string(f.tmp.path().join("machine/logins.json")).unwrap();
    assert!(!index.contains("s3cret-cookie"));

    let res = send(
        &f.app,
        req(
            "PUT",
            "/sites/www.undeclared.test/login",
            Some(TOKEN),
            Some(json!({"cookies": []})),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let res = send(
        &f.app,
        req("DELETE", "/sites/www.example.test/login", Some(TOKEN), None),
    )
    .await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let sites = json_of(send(&f.app, req("GET", "/sites", Some(TOKEN), None)).await).await;
    assert_eq!(sites[0]["login"], "missing");
}

#[tokio::test]
async fn big_batches_and_login_bodies_fit_their_own_limits() {
    let f = fixture();
    let (_, token) = new_token(&f.app, "extension").await;
    let links: Vec<Value> = (0..5000)
        .map(|i| json!({"url": format!("https://a.test/note/{i:05}?xsec_token=abcdefghijklmnop"), "title": "一条收藏的笔记标题"}))
        .collect();
    let body = json!({"name": "很多", "links": links});
    assert!(body.to_string().len() > 64 << 10);
    let start = std::time::Instant::now();
    let res = send(&f.app, req("POST", "/v1/batches", Some(&token), Some(body))).await;
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    assert!(
        start.elapsed() < std::time::Duration::from_secs(10),
        "{:?}",
        start.elapsed()
    );
    let too_many: Vec<Value> = (0..5001)
        .map(|i| json!({"url": format!("https://a.test/{i}")}))
        .collect();
    let res = send(
        &f.app,
        req(
            "POST",
            "/v1/batches",
            Some(&token),
            Some(json!({"links": too_many})),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let cookies: Vec<Value> = (0..400)
        .map(|i| json!({"name": format!("c{i}"), "value": "v".repeat(200), "domain": ".example.test"}))
        .collect();
    let mut cookies = cookies;
    cookies.push(json!({"name": "sid", "value": "x-session-1", "domain": ".example.test"}));
    let body = json!({"cookies": cookies});
    assert!(body.to_string().len() > 64 << 10);
    let res = send(
        &f.app,
        req(
            "PUT",
            "/sites/www.example.test/login",
            Some(TOKEN),
            Some(body),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    // 别的路由还是 64 KiB
    let res = send(
        &f.app,
        req(
            "POST",
            "/v1/deliver",
            Some(&token),
            Some(json!({"text": "x".repeat(70 << 10)})),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
