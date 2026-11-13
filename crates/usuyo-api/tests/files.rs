//! 从文件导入的路由和浏览器脚本的内容投递（协议 §3.9、§5.1）。直接对着路由跑，不开端口、不碰网络。
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;
use usuyo_api::{ApiConfig, default_origins, router};
use usuyo_core::{Engine, EngineConfig};

const PORT: u16 = 47998;
const TOKEN: &str = "owner-token";
const READER: &str = "fake%2Freader%23export";

/// `list` 读 `USUYO_INPUT`：文件里有 `broken` 就说读不了，不然交一条只有 `ref` 的。
const PLUGIN: &str = r#"#!/bin/sh
case "$1" in
  health) echo '{"ok": true, "ready": true, "protocol": 1, "version": "1.0.0"}' ;;
  canonicalize) echo '{"ok": true, "match": false}' ;;
  list)
    if grep -q broken "$USUYO_INPUT"; then
      echo '{"ok": false, "error": {"code": "unsupported-input", "message": "not ours", "hint": "这不是例子的导出文件"}}'; exit 1
    fi
    echo '{"ok": true, "items": [{"ref": "page/1", "title": "登录后才看得到的一页"}], "nextCursor": null, "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "browser-session"}}'
    ;;
  *) echo '{"ok": false, "error": {"code": "internal", "message": "not in this test"}}'; exit 1 ;;
esac
"#;

const MANIFEST: &str = r#"protocol: 1
id: fake/reader
kind: fetch
name: 例子
version: 1.0.0
license: MIT
maintainers: [someone]
platforms: [example]
methods:
  - id: export
    name: 脚本交的内容
    access: browser-session
    login: none
    account: own-browser
    capabilities:
      list:
        order: unordered
        files: ["*.json"]
        reads: 例子的浏览器脚本交来的页面内容
      fetch:
        input: [ref]
        outputs: [text]
permissions:
  fs: {read: ["${INPUT}"], write: ["${OUT}"]}
run:
  command: ["${PLUGIN_DIR}/plugin.sh"]
"#;

struct Fixture {
    tmp: TempDir,
    app: Router,
}

impl Fixture {
    fn copies(&self) -> PathBuf {
        let libs = self.tmp.path().join("machine/libraries");
        let lib = std::fs::read_dir(&libs).unwrap().next().unwrap().unwrap();
        lib.path().join("imports")
    }

    fn copy_count(&self) -> usize {
        std::fs::read_dir(self.copies())
            .map(|d| d.count())
            .unwrap_or(0)
    }
}

fn fixture() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let machine = tmp.path().join("machine");
    let dir = machine.join("plugins/fake/reader/1.0.0");
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

fn req(method: &str, path: &str, token: Option<&str>, body: Body) -> Request<Body> {
    let mut b = Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, format!("127.0.0.1:{PORT}"));
    if let Some(t) = token {
        b = b.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    b.header(header::CONTENT_TYPE, "application/octet-stream")
        .body(body)
        .unwrap()
}

async fn send(app: &Router, r: Request<Body>) -> Response<Body> {
    app.clone().oneshot(r).await.unwrap()
}

async fn json_of(res: Response<Body>) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

async fn new_token(app: &Router, kind: &str) -> String {
    let r = Request::builder()
        .method("POST")
        .uri("/deliver-tokens")
        .header(header::HOST, format!("127.0.0.1:{PORT}"))
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({"name": "例子脚本", "kind": kind}).to_string(),
        ))
        .unwrap();
    let v = json_of(send(app, r).await).await;
    v["token"].as_str().unwrap().into()
}

#[tokio::test]
async fn the_window_lists_readers_and_imports_a_file() {
    let f = fixture();
    let res = send(
        &f.app,
        req("GET", "/file-readers", Some(TOKEN), Body::empty()),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let readers = json_of(res).await;
    assert_eq!(readers[0]["id"], "fake/reader#export");
    assert_eq!(readers[0]["reads"], "例子的浏览器脚本交来的页面内容");

    let res = send(
        &f.app,
        req(
            "POST",
            &format!("/file-imports?reader={READER}&name=%E5%AF%BC%E5%87%BA.json"),
            Some(TOKEN),
            Body::from("x".repeat(100 << 10)),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK, "比 64 KiB 的上限大也收");
    let v = json_of(res).await;
    assert_eq!(v["kind"], "file");
    assert_eq!(v["state"], "ready");
    assert_eq!(v["file"], json!({"name": "导出.json", "reader": "例子"}));

    let res = send(
        &f.app,
        req(
            "GET",
            &format!("/imports/{}/entries", v["id"].as_str().unwrap()),
            Some(TOKEN),
            Body::empty(),
        ),
    )
    .await;
    let entries = json_of(res).await;
    assert_eq!(entries[0]["title"], "登录后才看得到的一页");
    assert!(entries[0].get("url").is_none(), "{entries}");
}

#[tokio::test]
async fn an_unreadable_or_empty_file_answers_with_a_hint_and_leaves_nothing() {
    let f = fixture();
    let res = send(
        &f.app,
        req(
            "POST",
            &format!("/file-imports?reader={READER}&name=a.json"),
            Some(TOKEN),
            Body::from("broken"),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let p = json_of(res).await;
    assert_eq!(p["code"], "invalid-args");
    assert_eq!(p["hint"], "这不是例子的导出文件");
    assert!(
        p["detail"].as_str().unwrap().is_ascii(),
        "detail 留英文给日志：{p}"
    );
    assert_eq!(f.copy_count(), 0);

    let res = send(
        &f.app,
        req(
            "POST",
            &format!("/file-imports?reader={READER}&name=a.json"),
            Some(TOKEN),
            Body::empty(),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_of(res).await["hint"], "文件是空的。");
    assert_eq!(f.copy_count(), 0);

    let res = send(
        &f.app,
        req(
            "POST",
            "/file-imports?name=a.json",
            Some(TOKEN),
            Body::from("x"),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(json_of(res).await["hint"].is_string());
}

#[tokio::test]
async fn a_script_hands_over_content_and_reads_nothing_back() {
    let f = fixture();
    let token = new_token(&f.app, "extension").await;
    let res = send(
        &f.app,
        req(
            "POST",
            &format!("/v1/file-imports?reader={READER}&name=page.json"),
            Some(&token),
            Body::from(r#"{"pages": []}"#),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let v = json_of(res).await;
    let id = v["id"].as_str().unwrap().to_owned();
    assert_eq!(v.as_object().unwrap().len(), 1, "只回 id：{v}");

    // 交的令牌读不了它
    let res = send(
        &f.app,
        req(
            "GET",
            &format!("/imports/{id}"),
            Some(&token),
            Body::empty(),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    // 窗口里照样看得到，等你确认；记着是谁交的
    let res = send(
        &f.app,
        req("GET", &format!("/imports/{id}"), Some(TOKEN), Body::empty()),
    )
    .await;
    let view = json_of(res).await;
    assert_eq!(view["kind"], "file");
    assert_eq!(view["client"], "例子脚本");
    assert_eq!(view["state"], "ready");

    // 只能交链接的令牌交不了内容
    let mobile = new_token(&f.app, "mobile").await;
    let res = send(
        &f.app,
        req(
            "POST",
            &format!("/v1/file-imports?reader={READER}&name=page.json"),
            Some(&mobile),
            Body::from("{}"),
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    // 网页的来源进不来
    let mut r = req(
        "POST",
        &format!("/v1/file-imports?reader={READER}&name=page.json"),
        Some(&token),
        Body::from("{}"),
    );
    r.headers_mut()
        .insert(header::ORIGIN, "https://evil.test".parse().unwrap());
    assert_eq!(send(&f.app, r).await.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn an_oversized_hand_over_is_refused_and_leaves_nothing() {
    let f = fixture();
    let token = new_token(&f.app, "extension").await;
    // 一块 1 MiB 反复交，比上限多一块；不在内存里摆一整份
    let chunk = axum::body::Bytes::from(vec![b'x'; 1 << 20]);
    let n = (usuyo_core::HANDOVER_MAX >> 20) + 1;
    let big = Body::from_stream(futures_util::stream::iter(
        (0..n).map(move |_| Ok::<_, std::io::Error>(chunk.clone())),
    ));
    let res = send(
        &f.app,
        req(
            "POST",
            &format!("/v1/file-imports?reader={READER}&name=big.json"),
            Some(&token),
            big,
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert!(json_of(res).await["hint"].is_string());
    assert_eq!(f.copy_count(), 0, "超了的副本去掉");
}

#[tokio::test]
async fn share_text_is_taken_for_its_first_link() {
    let f = fixture();
    let batch = |url: &str| {
        Request::builder()
            .method("POST")
            .uri("/v1/batches")
            .header(header::HOST, format!("127.0.0.1:{PORT}"))
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"links": [{"url": url}]}).to_string()))
            .unwrap()
    };
    let res = send(
        &f.app,
        batch("某某发布了一篇小红书笔记，快来看吧！ http://xhslink.com/a/AbCdEf，复制本条信息，打开【小红书】App查看精彩内容！"),
    )
    .await;
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let res = send(&f.app, batch("没有链接")).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_of(res).await["hint"], "第 1 个链接无法使用。");
    let res = send(&f.app, batch("先 https://a:b@a.test/1 再 https://b.test/2")).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST, "只看第一条");
}
