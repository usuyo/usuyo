//! 凭据和密钥怎么交给插件（协议 §6、§2.11）：对着假插件（sh 脚本）和假钥匙串跑，不碰系统钥匙串。
#![cfg(all(unix, feature = "host"))]

use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use tempfile::TempDir;
use usuyo_plugin::protocol::{ErrorCode, StorageState};
use usuyo_plugin::{
    CallOptions, FetchInput, FetchRequest, HostConfig, ListRequest, LoginState, PluginError,
    PluginHost, Vault, discover,
};

/// 假钥匙串：内存里的表。
#[derive(Debug, Default)]
struct FakeVault(Mutex<BTreeMap<String, String>>);

impl Vault for FakeVault {
    fn read(&self, key: &str) -> Result<Option<String>, PluginError> {
        Ok(self.0.lock().unwrap().get(key).cloned())
    }
    fn write(&self, key: &str, value: &str) -> Result<(), PluginError> {
        self.0.lock().unwrap().insert(key.into(), value.into());
        Ok(())
    }
    fn remove(&self, key: &str) -> Result<(), PluginError> {
        self.0.lock().unwrap().remove(key);
        Ok(())
    }
}

/// 把收到的凭据和密钥抄进缓存目录，路径也记下来；stderr 里故意打出值。
const SCRIPT: &str = r#"
seen="$USUYO_CACHE_DIR/seen-$1.json"
c=null; [ -n "$USUYO_CREDENTIALS" ] && c=$(cat "$USUYO_CREDENTIALS")
k=null; [ -n "$USUYO_SECRETS" ] && k=$(cat "$USUYO_SECRETS")
w=null; [ -n "$USUYO_VISITOR" ] && w=$(cat "$USUYO_VISITOR")
printf '{"credentials": %s, "secrets": %s, "visitor": %s, "path": "%s", "txt": "%s", "vpath": "%s"}' \
  "$c" "$k" "$w" "$USUYO_CREDENTIALS" "$USUYO_COOKIES_TXT" "$USUYO_VISITOR" > "$seen.tmp"
mv "$seen.tmp" "$seen"
if [ -n "$USUYO_CREDENTIALS" ]; then echo "debug: $(cat "$USUYO_CREDENTIALS")" >&2; fi
if [ -n "$USUYO_VISITOR" ]; then echo "debug: $(cat "$USUYO_VISITOR")" >&2; fi
case "$1:$2" in
  fetch:*fail*)
    v=$(sed -n 's/.*"value":"\([^"]*\)".*/\1/p' "$USUYO_VISITOR" | head -1)
    printf '{"ok": false, "error": {"code": "login-required", "message": "visitor %s rejected", "hint": "换一份 %s"}}' "$v" "$v"
    exit 1 ;;
esac
case "$1" in
  canonicalize) echo '{"ok": true, "match": false}' ;;
  sources)
    v=$(sed -n 's/.*"value":"\([^"]*\)".*/\1/p' "$USUYO_CREDENTIALS" | head -1)
    printf '{"ok": true, "sources": "%s"}' "$v" ;;
  list)
    v=$(sed -n 's/.*"value":"\([^"]*\)".*/\1/p' "$USUYO_CREDENTIALS" | head -1)
    printf '{"ok": false, "error": {"code": "login-required", "message": "token %s rejected", "hint": "重新登录 %s"}}' "$v" "$v"
    exit 1 ;;
  fetch)
    cat <<JSON
{"ok": true, "kind": "article", "title": "t", "text": "正文", "images": [], "media": [],
 "platformTags": [], "completeness": "full", "missing": [],
 "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "session-api"},
 "credentialsUpdate": {"cookies": [
   {"name": "sid", "value": "rotated-value-1", "domain": ".example.com", "path": "/"},
   {"name": "stray", "value": "stray-value-1", "domain": ".other.test", "path": "/"}]}}
JSON
    ;;
esac
"#;

/// `fetch_login` 是 fetch 能力的 `login`；list 总要登录。
fn manifest(id: &str, site: &str, fetch_login: &str, extra: &str) -> String {
    let fetch_sites = if fetch_login == "none" {
        String::new()
    } else {
        format!("\n    sites: [{site}]")
    };
    format!(
        r#"protocol: 1
id: {id}
kind: fetch
name: Fake
version: 1.0.0
license: MIT
maintainers: [someone]
platforms: [example]
match: ["example.com/*"]
access: session-api
login: required
sites:
  - site: {site}
    name: 站点
    credential: cookies
    loginUrl: https://{site}/
    cookies: [sid]
    domains: [".{domain}"]
{extra}
capabilities:
  fetch:
    access: session-api
    login: {fetch_login}{fetch_sites}
    outputs: [text, meta]
  list:
    access: session-api
    login: required
    sites: [{site}]
    order: newest-first
permissions:
  fs: {{write: ["${{OUT}}"]}}
run:
  command: ["${{PLUGIN_DIR}}/plugin.sh"]
"#,
        domain = site.split_once('.').unwrap().1,
    )
}

/// 不登录、只用访客身份的插件：`fetch` 列了访客身份，`list` 没列。访客身份和 fake/a 的账号是同一个主机。
const VISITOR_MANIFEST: &str = r#"protocol: 1
id: fake/visitor
kind: fetch
name: Fake visitor
version: 1.0.0
license: MIT
maintainers: [someone]
platforms: [example]
match: ["example.com/*"]
access: public-web
login: none
visitors:
  - site: www.example.com
    name: 站点
    url: https://www.example.com/
    cookies: [vid]
    domains: [".example.com"]
capabilities:
  fetch:
    access: public-web
    login: none
    visitors: [www.example.com]
    outputs: [text, meta]
  list:
    access: public-web
    login: none
    order: newest-first
permissions:
  fs: {write: ["${OUT}"]}
run:
  command: ["${PLUGIN_DIR}/plugin.sh"]
"#;

struct Fixture {
    tmp: TempDir,
    host: PluginHost,
    vault: Arc<FakeVault>,
}

fn install(dir: &Path, id: &str, manifest: &str) {
    let d = dir.join(id).join("1.0.0");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("usuyo-plugin.yaml"), manifest).unwrap();
    let path = d.join("plugin.sh");
    std::fs::write(
        &path,
        format!("#!/bin/sh\nout=\"\"\nif [ \"$1\" = fetch ]; then out=\"$4\"; fi\n{SCRIPT}\n"),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

fn fixture() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let plugins = tmp.path().join("plugins");
    let secret = "settings:\n  - {key: token, label: 令牌, type: string, secret: true}\n  - {key: endpoint, label: 地址, type: string}";
    install(
        &plugins,
        "fake/a",
        &manifest("fake/a", "www.example.com", "optional", secret),
    );
    install(
        &plugins,
        "fake/b",
        &manifest("fake/b", "www.other.test", "optional", ""),
    );
    install(
        &plugins,
        "fake/none",
        &manifest("fake/none", "www.example.com", "none", ""),
    );
    install(&plugins, "fake/visitor", VISITOR_MANIFEST);
    let found = discover(&plugins).unwrap();
    assert!(found.rejected.is_empty(), "{:?}", found.rejected);
    let vault = Arc::new(FakeVault::default());
    let host = PluginHost::new(
        HostConfig {
            work_dir: tmp.path().join("lib/.usuyo/tmp"),
            cache_dir: tmp.path().join("cache"),
            locale: "zh-CN".into(),
            limits: Default::default(),
        },
        found.fetch,
    )
    .with_vault(vault.clone(), &tmp.path().join("machine"));
    Fixture { tmp, host, vault }
}

impl Fixture {
    fn plugin(&self, id: &str) -> &usuyo_plugin::FetchPlugin {
        self.host.plugin(id).unwrap()
    }

    fn seen(&self, id: &str, cmd: &str) -> Value {
        let p = self
            .tmp
            .path()
            .join("cache")
            .join(id)
            .join(format!("seen-{cmd}.json"));
        serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap()
    }

    fn login(&self) {
        let state: StorageState = serde_json::from_value(json!({
            "cookies": [
                {"name": "sid", "value": "secret-sid-value", "domain": ".example.com", "path": "/", "expires": -1},
                {"name": "api", "value": "secret-api-value", "domain": "api.example.com", "path": "/"},
                {"name": "other", "value": "foreign-value-9", "domain": ".other.test", "path": "/"},
                {"name": "old", "value": "expired-value-1", "domain": ".example.com", "path": "/", "expires": 1000}
            ],
            "origins": [{"origin": "https://www.example.com", "localStorage": [{"name": "k", "value": "local-value-1"}]}]
        }))
        .unwrap();
        let site = self.host.save_site("www.example.com", &state).unwrap();
        assert_eq!(site.login, LoginState::Present);
        let other: StorageState = serde_json::from_value(json!({"cookies": [
            {"name": "sid", "value": "b-site-value-7", "domain": ".other.test", "path": "/"}]}))
        .unwrap();
        self.host.save_site("www.other.test", &other).unwrap();
    }
}

impl Fixture {
    fn visit(&self) {
        let state: StorageState = serde_json::from_value(json!({
            "cookies": [
                {"name": "vid", "value": "visitor-vid-value", "domain": ".example.com", "path": "/", "expires": 4102444800i64},
                {"name": "late", "value": "visitor-late-value", "domain": "www.example.com", "path": "/"},
                {"name": "other", "value": "foreign-value-9", "domain": ".other.test", "path": "/"}
            ],
            "origins": [{"origin": "https://www.example.com", "localStorage": [{"name": "k", "value": "visitor-local-1"}]}]
        }))
        .unwrap();
        let v = self.host.save_visitor("www.example.com", &state).unwrap();
        assert_eq!(v.state, LoginState::Present);
        assert_eq!(v.expires, Some(4102444800.0));
    }
}

fn fetch_request() -> FetchRequest {
    FetchRequest {
        input: FetchInput::Url(url::Url::parse("https://example.com/p/1").unwrap()),
        want: None,
        input_file: None,
    }
}

fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(files_under(&p));
            }
            out.push(p);
        }
    }
    out
}

#[tokio::test]
async fn only_the_declared_site_reaches_the_plugin_and_the_file_is_gone_after() {
    let f = fixture();
    f.login();
    let snap = f
        .host
        .fetch(f.plugin("fake/a"), fetch_request(), &CallOptions::default())
        .await
        .unwrap();
    let seen = f.seen("fake/a", "fetch");
    let names: Vec<&str> = seen["credentials"]["storageState"]["cookies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    // `.example.com` 写在 domains 里，所以 api.example.com 的也算；别的站点和过期的不给
    assert_eq!(names, ["sid", "api"]);
    assert_eq!(
        seen["credentials"]["sites"],
        json!([{"site": "www.example.com", "source": "login-window"}])
    );
    assert_eq!(
        seen["credentials"]["storageState"]["origins"][0]["localStorage"][0]["value"],
        "local-value-1"
    );

    // 快照还在（输出目录还没移走），凭据文件已经不在了，也不在资料库的暂存目录里
    let path = PathBuf::from(seen["path"].as_str().unwrap());
    assert!(
        path.starts_with(f.tmp.path().join("machine/run")),
        "{path:?}"
    );
    assert!(!path.exists() && !Path::new(seen["txt"].as_str().unwrap()).exists());
    assert!(snap.out_dir().exists());
    for file in files_under(&f.tmp.path().join("lib")) {
        if file.is_file() {
            let text = std::fs::read_to_string(&file).unwrap_or_default();
            assert!(!text.contains("secret-sid-value"), "{file:?}");
        }
    }
    drop(snap);
    assert_eq!(
        std::fs::read_dir(f.tmp.path().join("machine/run"))
            .unwrap()
            .count(),
        0
    );
}

#[tokio::test]
async fn login_none_and_canonicalize_get_nothing() {
    let f = fixture();
    f.login();
    f.host
        .fetch(
            f.plugin("fake/none"),
            fetch_request(),
            &CallOptions::default(),
        )
        .await
        .unwrap();
    assert_eq!(f.seen("fake/none", "fetch")["path"], "");
    f.host
        .canonicalize(
            f.plugin("fake/a"),
            &url::Url::parse("https://example.com/p/1").unwrap(),
            &CallOptions::default(),
        )
        .await
        .unwrap();
    assert_eq!(f.seen("fake/a", "canonicalize")["path"], "");
    // fake/b 只拿到它自己声明的站点
    f.host
        .fetch(f.plugin("fake/b"), fetch_request(), &CallOptions::default())
        .await
        .unwrap();
    let seen = f.seen("fake/b", "fetch");
    let text = seen["credentials"].to_string();
    assert!(
        text.contains("b-site-value-7") && !text.contains("secret-sid-value"),
        "{text}"
    );
}

#[tokio::test]
async fn values_are_scrubbed_from_errors_and_stderr() {
    let f = fixture();
    f.login();
    let e = f
        .host
        .list(
            f.plugin("fake/a"),
            ListRequest {
                source: "example:favorites".into(),
                cursor: None,
                since: None,
                input_file: None,
            },
            &CallOptions::default(),
        )
        .await
        .unwrap_err();
    let PluginError::Call {
        failure, stderr, ..
    } = &e
    else {
        panic!("{e:?}")
    };
    assert_eq!(failure.code, ErrorCode::LoginRequired);
    let all = format!(
        "{e} {e:?} {} {:?} {}",
        failure.message,
        failure.hint,
        stderr.join("\n")
    );
    assert!(!all.contains("secret-sid-value"), "{all}");
    assert!(all.contains("[redacted]"), "{all}");
}

#[tokio::test]
async fn values_are_scrubbed_from_host_made_errors() {
    // 信封形状不对时，宿主拼的错误会引用插件的原文；里面夹着交出去的值也要洗掉。
    let f = fixture();
    f.login();
    let e = f
        .host
        .sources(f.plugin("fake/a"), &CallOptions::default())
        .await
        .unwrap_err();
    let PluginError::Call { failure, .. } = &e else {
        panic!("{e:?}")
    };
    assert_eq!(failure.code, ErrorCode::InvalidOutput);
    let all = format!("{e} {e:?} {}", failure.message);
    // 假插件把收到的第一个 cookie 值写进了形状不对的信封
    assert!(!all.contains("local-value-1"), "{all}");
    assert!(!all.contains("secret-sid-value"), "{all}");
    // 宿主拼的错误只说哪一栏，不引用值
    assert_eq!(
        failure.message,
        "envelope at `sources` has the wrong type or shape"
    );
}

#[tokio::test]
async fn secret_settings_travel_as_usuyo_secrets() {
    let f = fixture();
    let a = f.plugin("fake/a").clone();
    let err = f.host.set_secret(&a, "endpoint", "x").unwrap_err();
    assert!(
        matches!(err, PluginError::Credentials { .. }),
        "不是密钥的设置不收"
    );
    f.host.set_secret(&a, "token", "tok-123456").unwrap();
    assert_eq!(
        f.host
            .secrets_set(a.id())
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>(),
        ["token"]
    );
    f.host
        .fetch(&a, fetch_request(), &CallOptions::default())
        .await
        .unwrap();
    assert_eq!(
        f.seen("fake/a", "fetch")["secrets"],
        json!({"token": "tok-123456"})
    );
    // 索引里只有名字，值只在钥匙串里
    let index = std::fs::read_to_string(f.tmp.path().join("machine/logins.json")).unwrap();
    assert!(!index.contains("tok-123456") && !index.contains("secret-sid-value"));
    f.host.set_secret(&a, "token", "").unwrap();
    assert!(
        f.vault
            .0
            .lock()
            .unwrap()
            .get("secret:fake/a:token")
            .is_none()
    );
}

#[tokio::test]
async fn credentials_update_is_written_back_for_the_declared_site_only() {
    let f = fixture();
    f.login();
    let snap = f
        .host
        .fetch(f.plugin("fake/a"), fetch_request(), &CallOptions::default())
        .await
        .unwrap();
    assert!(snap.output.credentials_update.is_none(), "不随快照往下走");
    let stored = f.vault.0.lock().unwrap()["site:www.example.com"].clone();
    assert!(stored.contains("rotated-value-1") && !stored.contains("secret-sid-value"));
    assert!(!stored.contains("stray-value-1"));
    let other = f.vault.0.lock().unwrap()["site:www.other.test"].clone();
    assert!(!other.contains("stray-value-1"), "别的站点不被这个插件改");
}

#[tokio::test]
async fn saving_and_forgetting_a_site() {
    let f = fixture();
    let foreign: StorageState = serde_json::from_value(json!({"cookies": [
        {"name": "x", "value": "v-v-v-v", "domain": ".nowhere.test"}]}))
    .unwrap();
    assert!(f.host.save_site("www.example.com", &foreign).is_err());
    assert!(f.host.save_site("www.undeclared.test", &foreign).is_err());
    f.login();
    let sites = f.host.sites().unwrap();
    let a = sites
        .iter()
        .find(|s| s.site.site == "www.example.com")
        .unwrap();
    assert_eq!(a.plugins.len(), 2, "两个插件声明同一个站点，只登录一次");
    f.host.forget_site("www.example.com").unwrap();
    let a = f
        .host
        .sites()
        .unwrap()
        .into_iter()
        .find(|s| s.site.site == "www.example.com")
        .unwrap();
    assert_eq!(a.login, LoginState::Missing);
    assert!(
        !f.vault
            .0
            .lock()
            .unwrap()
            .contains_key("site:www.example.com")
    );
}

#[tokio::test]
async fn the_visitor_reaches_only_calls_that_list_it() {
    let f = fixture();
    f.login();
    f.visit();
    // 列了访客身份的 fetch：拿到 USUYO_VISITOR，没有账号
    f.host
        .fetch(
            f.plugin("fake/visitor"),
            fetch_request(),
            &CallOptions::default(),
        )
        .await
        .unwrap();
    let seen = f.seen("fake/visitor", "fetch");
    assert_eq!(seen["credentials"], Value::Null, "访客身份的调用不带账号");
    assert_eq!(
        seen["visitor"]["sites"],
        json!([{"site": "www.example.com", "source": "visitor-window"}])
    );
    let names: Vec<&str> = seen["visitor"]["storageState"]["cookies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["vid", "late"]);
    assert_eq!(seen["visitor"]["storageState"]["origins"], json!([]));
    let path = PathBuf::from(seen["vpath"].as_str().unwrap());
    assert!(
        path.starts_with(f.tmp.path().join("machine/run")),
        "{path:?}"
    );
    assert!(!path.exists(), "调用结束就删");

    // 没列访客身份的 list：什么都不交
    f.host
        .list(
            f.plugin("fake/visitor"),
            ListRequest {
                source: "example:favorites".into(),
                cursor: None,
                since: None,
                input_file: None,
            },
            &CallOptions::default(),
        )
        .await
        .unwrap_err();
    let seen = f.seen("fake/visitor", "list");
    assert_eq!(seen["visitor"], Value::Null);
    assert_eq!(seen["vpath"], "");

    // 同一主机的账号调用拿不到访客身份
    f.host
        .fetch(f.plugin("fake/a"), fetch_request(), &CallOptions::default())
        .await
        .unwrap();
    let seen = f.seen("fake/a", "fetch");
    assert_eq!(seen["visitor"], Value::Null);
    assert!(
        !seen["credentials"]
            .to_string()
            .contains("visitor-vid-value")
    );
}

#[tokio::test]
async fn a_visitor_and_an_account_for_the_same_host_stay_apart() {
    let f = fixture();
    f.visit();
    // 访客身份不是登录：/sites 里这个主机还是没登录
    let site = f
        .host
        .sites()
        .unwrap()
        .into_iter()
        .find(|s| s.site.site == "www.example.com")
        .unwrap();
    assert_eq!(site.login, LoginState::Missing);
    // 只有访客身份的调用交回 credentialsUpdate，也不建账号那份
    f.host
        .fetch(
            f.plugin("fake/visitor"),
            fetch_request(),
            &CallOptions::default(),
        )
        .await
        .unwrap();
    {
        let vault = f.vault.0.lock().unwrap();
        assert!(!vault.contains_key("site:www.example.com"));
        let stored = &vault["visitor:www.example.com"];
        assert!(stored.contains("visitor-vid-value") && !stored.contains("rotated-value-1"));
        assert!(
            !stored.contains("visitor-local-1"),
            "访客身份不存 localStorage"
        );
    }
    // 登录以后两份各在各的；去掉访客身份不动账号
    f.login();
    f.host.forget_visitor("www.example.com").unwrap();
    let vault = f.vault.0.lock().unwrap();
    assert!(vault.contains_key("site:www.example.com"));
    assert!(!vault.contains_key("visitor:www.example.com"));
    drop(vault);
    let v = &f.host.visitors().unwrap()[0];
    assert_eq!((v.state, v.expires), (LoginState::Missing, None));
    assert_eq!(v.plugins.len(), 1);
    // 索引里只有名字
    let index = std::fs::read_to_string(f.tmp.path().join("machine/logins.json")).unwrap();
    assert!(!index.contains("visitor-vid-value"));
}

#[tokio::test]
async fn visitor_values_are_scrubbed_and_foreign_cookies_refused() {
    let f = fixture();
    let foreign: StorageState = serde_json::from_value(json!({"cookies": [
        {"name": "x", "value": "v-v-v-v", "domain": ".nowhere.test"}]}))
    .unwrap();
    assert!(f.host.save_visitor("www.example.com", &foreign).is_err());
    assert!(
        f.host
            .save_visitor("www.undeclared.test", &foreign)
            .is_err()
    );
    f.visit();
    let e = f
        .host
        .fetch(
            f.plugin("fake/visitor"),
            FetchRequest {
                input: FetchInput::Url(url::Url::parse("https://example.com/fail").unwrap()),
                want: None,
                input_file: None,
            },
            &CallOptions::default(),
        )
        .await
        .unwrap_err();
    let PluginError::Call {
        failure, stderr, ..
    } = &e
    else {
        panic!("{e:?}")
    };
    let all = format!(
        "{e} {e:?} {} {:?} {}",
        failure.message,
        failure.hint,
        stderr.join("\n")
    );
    assert!(!all.contains("visitor-vid-value"), "{all}");
    assert!(!all.contains("visitor-late-value"), "{all}");
    assert!(all.contains("[redacted]"), "{all}");
}
