//! 清单的合规和不合规样例（协议 §2、§10）。

use usuyo_plugin::PluginError;
use usuyo_plugin::protocol::{
    Access, FetchInputKind, Login, OutputKind, PluginKind, PluginManifest,
};

fn testdata(name: &str) -> String {
    let path = format!("{}/testdata/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn parse(yaml: &str) -> Result<PluginManifest, PluginError> {
    PluginManifest::from_yaml(yaml).map(|p| p.value)
}

/// 期望在 `field` 上报错；返回错误信息给调用方再查。
fn rejects(yaml: &str, field: &str) -> String {
    match parse(yaml) {
        Err(PluginError::Manifest {
            field: got,
            message,
        }) => {
            assert_eq!(got, field, "wrong field; message: {message}");
            message
        }
        Err(other) => panic!("expected a manifest error at `{field}`, got {other}"),
        Ok(_) => panic!("expected a manifest error at `{field}`, but it parsed"),
    }
}

fn web() -> String {
    testdata("manifest.minimal.yaml")
}

fn douyin() -> String {
    testdata("manifest.full.yaml")
}

fn replace(base: &str, from: &str, to: &str) -> String {
    assert!(base.contains(from), "fixture lacks `{from}`");
    base.replacen(from, to, 1)
}

#[test]
fn web_plugin_manifest_is_the_minimal_sample() {
    let shipped = std::fs::read_to_string(format!(
        "{}/../../plugins/web/usuyo-plugin.yaml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    assert_eq!(shipped, web());
}

#[test]
fn valid_samples_parse() {
    let m = parse(&web()).unwrap();
    let spec = m.fetch().unwrap();
    assert!(spec.is_web());
    assert!(m.header.id.is_builtin());
    assert!(spec.matches(&url::Url::parse("https://example.com/a").unwrap()));

    let m = parse(&douyin()).unwrap();
    let spec = m.fetch().unwrap();
    assert_eq!(
        (spec.access, spec.login),
        (Access::BrowserSession, Login::Required)
    );
    let fetch = spec.capabilities.fetch.as_ref().unwrap();
    assert!(fetch.outputs.contains(&OutputKind::Comments));
    assert_eq!(fetch.input, [FetchInputKind::Url].into());
    assert_eq!(fetch.uses.timeout.unwrap().get().as_secs(), 600);
    assert!(spec.capabilities.unfavorite.is_some());
    assert_eq!(m.header.permissions.exec, ["chrome"]);
    assert_eq!(m.header.permissions.limits.memory.unwrap().bytes(), 1 << 30);

    let m = parse(&testdata("manifest.import.yaml")).unwrap();
    let spec = m.fetch().unwrap();
    assert!(spec.match_patterns.is_empty());
    assert_eq!(
        spec.capabilities.fetch.as_ref().unwrap().input,
        [FetchInputKind::Ref].into()
    );

    assert!(matches!(
        parse(&testdata("manifest.transform.yaml")).unwrap().kind,
        PluginKind::Transform(_)
    ));
    assert!(matches!(
        parse(&testdata("manifest.export.yaml")).unwrap().kind,
        PluginKind::Export(_)
    ));
}

#[test]
fn unknown_fields_are_reported_not_fatal() {
    let yaml = format!("{}color: blue\n", web());
    let parsed = PluginManifest::from_yaml(&yaml).unwrap();
    assert_eq!(parsed.unknown_fields, ["color"]);
}

#[test]
fn invalid_samples_name_the_field() {
    let w = web();
    let d = douyin();
    rejects(&replace(&w, "protocol: 1", "protocol: 2"), "protocol");
    rejects(&replace(&w, "id: builtin/web", "id: Builtin/Web"), "id");
    rejects(
        &replace(&w, "version: 0.0.0", "version: \"1.0\""),
        "version",
    );
    rejects(
        &replace(&w, "maintainers: [usuyo]", "maintainers: []"),
        "maintainers",
    );
    rejects(&format!("{w}consumes: [text]\n"), "consumes");
    rejects(
        &replace(&w, "    outputs: [text, html, images, meta]\n", ""),
        "capabilities.fetch.outputs",
    );
    rejects(
        &replace(&w, "outputs: [text, html", "outputs: [video, html"),
        "capabilities.fetch.outputs[0]",
    );
    let message = rejects(
        &replace(
            &w,
            "access: public-web\nlogin",
            "access: session-api\nlogin",
        ),
        "access",
    );
    assert!(message.contains("public-web"), "{message}");
    rejects(&replace(&w, "match: [\"*\"]\n", ""), "match");
    rejects(
        &replace(&w, "match: [\"*\"]", "match: [\"https://*\"]"),
        "match[0]",
    );
    rejects(
        &replace(&w, "    write: [\"${OUT}\"]", "    write: [\"/tmp\"]"),
        "permissions.fs.write[0]",
    );
    rejects(
        &replace(&w, "mode: allowlist", "mode: none"),
        "permissions.network.hosts",
    );
    rejects(
        &replace(&w, "  fs:\n", "  env: [GITHUB_TOKEN]\n  fs:\n"),
        "permissions.env[0]",
    );
    rejects(
        &replace(
            &w,
            "command: [\"${PLUGIN_DIR}/usuyo-plugin-web\"]",
            "command: [\"node\", \"main.js\"]",
        ),
        "run.command[0]",
    );
    rejects(
        &replace(
            &w,
            "    login: none\n    outputs",
            "    login: none\n    timeout: \"10\"\n    outputs",
        ),
        "capabilities.fetch.timeout",
    );
    rejects(
        &replace(
            &w,
            "    login: none\n    outputs",
            "    login: none\n    order: unordered\n    outputs",
        ),
        "capabilities.fetch.order",
    );
    rejects(
        &replace(&d, "\".douyin.com\"", "\".evil.com\""),
        "sites[0].domains[0]",
    );
    rejects(
        &replace(&d, "domains: [\".douyin.com\"]", "domains: [\"com\"]"),
        "sites[0].domains[0]",
    );
    rejects(
        &replace(
            &d,
            "    sites: [www.douyin.com]\n    order",
            "    sites: [www.x.com]\n    order",
        ),
        "capabilities.list.sites",
    );
    rejects(
        &replace(
            &d,
            "    login: none\n    timeout",
            "    login: none\n    sites: [www.douyin.com]\n    timeout",
        ),
        "capabilities.fetch.sites",
    );
    rejects(
        &replace(&d, "    loginUrl: https://www.douyin.com/\n", ""),
        "sites[0].loginUrl",
    );
    rejects(
        &replace(&d, "  douyin: {batch", "  bilibili: {batch"),
        "pacing.bilibili",
    );
    rejects(
        &replace(&d, "default: 1080p", "default: 4k"),
        "settings[0].default",
    );
    rejects(
        &replace(&d, "sha256: 4b22", "sha256: XX22"),
        "install.assets.darwin-arm64.sha256",
    );
    rejects(
        &replace(&d, "login: required\nsites", "login: optional\nsites"),
        "login",
    );
    rejects(
        &replace(&testdata("manifest.export.yaml"), ", \"${TARGET}\"", ""),
        "permissions.fs.read",
    );
    let message = rejects("protocol: 1\nid: [", "line 3");
    assert!(message.contains("YAML"), "{message}");
}
