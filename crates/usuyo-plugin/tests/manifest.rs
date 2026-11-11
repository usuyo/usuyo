//! 清单的合规和不合规样例（协议 §2、§10）。

use usuyo_plugin::PluginError;
use usuyo_plugin::protocol::{
    Access, ConsumeKind, FetchInputKind, Login, ModelFormat, OutputKind, PluginKind,
    PluginManifest, Produce,
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

#[test]
fn env_cannot_reach_host_variables_or_agents() {
    let w = web();
    for name in [
        "HOME",
        "Path",
        "TMPDIR",
        "XDG_CONFIG_HOME",
        "USUYO_CALL_ID",
        "SSH_AUTH_SOCK",
        "GITHUB_AUTH",
    ] {
        rejects(
            &replace(&w, "  fs:\n", &format!("  env: [{name}]\n  fs:\n")),
            "permissions.env[0]",
        );
    }
    parse(&replace(&w, "  fs:\n", "  env: [LANG, TZ]\n  fs:\n")).unwrap();
}

#[test]
fn capability_timeout_has_a_ceiling() {
    let m = parse(&replace(
        &web(),
        "    login: none\n    outputs",
        "    login: none\n    timeout: 1000h\n    outputs",
    ))
    .unwrap();
    let t = m
        .fetch()
        .unwrap()
        .capabilities
        .fetch
        .as_ref()
        .unwrap()
        .uses
        .timeout;
    assert!(
        t.is_some_and(|t| t.get() <= std::time::Duration::from_secs(2 * 3600)),
        "{t:?}"
    );
}

#[test]
fn visitors_are_declared_apart_from_sites_and_work_without_login() {
    let with = replace(
        &web(),
        "capabilities:\n  fetch:\n    access: public-web\n    login: none\n",
        "visitors:\n  - site: www.example.com\n    name: 例子\n    url: https://www.example.com/\n    cookies: [vid]\n    domains: [\".example.com\"]\ncapabilities:\n  fetch:\n    access: public-web\n    login: none\n    visitors: [www.example.com]\n",
    );
    let m = parse(&with).unwrap();
    let spec = m.fetch().unwrap();
    assert!(spec.sites.is_empty());
    assert_eq!(spec.visitors[0].cookies, ["vid"]);
    let fetch = spec.capabilities.fetch.as_ref().unwrap();
    assert_eq!(fetch.uses.visitors, ["www.example.com"]);

    rejects(
        &replace(
            &with,
            "visitors: [www.example.com]",
            "visitors: [www.other.test]",
        ),
        "capabilities.fetch.visitors",
    );
    rejects(
        &replace(&with, "url: https://", "url: http://"),
        "visitors[0].url",
    );
    rejects(
        &replace(
            &with,
            "url: https://www.example.com/",
            "url: www.example.com",
        ),
        "visitors[0].url",
    );
    rejects(
        &replace(&with, "\".example.com\"", "\".evil.test\""),
        "visitors[0].domains[0]",
    );
    rejects(
        &replace(
            &with,
            "  - site: www.example.com",
            "  - site: WWW.example.com",
        ),
        "visitors[0].site",
    );
    // 访客身份不让 `login: none` 的能力用上账号站点
    rejects(
        &replace(
            &with,
            "    visitors: [www.example.com]\n",
            "    visitors: [www.example.com]\n    sites: [www.example.com]\n",
        ),
        "capabilities.fetch.sites",
    );
    rejects(
        &format!("{}visitors: []\n", testdata("manifest.transform.yaml")),
        "visitors",
    );
}

fn asr() -> String {
    testdata("manifest.transform.yaml")
}

#[test]
fn transform_manifest_names_products_not_paths() {
    let m = parse(&asr()).unwrap();
    let PluginKind::Transform(spec) = m.kind else {
        panic!("not a transform plugin");
    };
    assert_eq!(spec.produces, [Produce::Asr]);
    assert_eq!(spec.consumes[0].kind, ConsumeKind::AudioPcm16k);
    let model = &spec.models[0];
    assert_eq!(model.format, ModelFormat::TarBz2);
    assert_eq!(model.mirrors.len(), 1);
    assert!(model.license.starts_with("FunASR"));

    let media = replace(
        &replace(
            &asr(),
            "consumes: [audio:pcm16k]",
            "consumes: [media:video, media:audio]",
        ),
        "produces: [asr]",
        "produces: [play, audio:pcm16k, probe, keyframes]",
    );
    let PluginKind::Transform(spec) = parse(&media).unwrap().kind else {
        panic!("not a transform plugin");
    };
    assert_eq!(spec.produces.len(), 4);
    for format in ["file", "tar.gz", "zip"] {
        parse(&replace(
            &asr(),
            "format: tar.bz2",
            &format!("format: {format}"),
        ))
        .unwrap();
    }

    // 旧的路径式写法不再认
    for old in [
        "derived/asr.vtt",
        "derived/ocr/*.json",
        "derived/keyframes/",
        "derived/audio/",
    ] {
        rejects(
            &replace(&asr(), "produces: [asr]", &format!("produces: [\"{old}\"]")),
            "produces[0]",
        );
    }
    rejects(
        &replace(&asr(), "produces: [asr]", "produces: [asr, asr]"),
        "produces",
    );
}

#[test]
fn transform_models_are_checked() {
    let a = asr();
    for format in ["tar.xz", "7z", "dir"] {
        rejects(
            &replace(&a, "format: tar.bz2", &format!("format: {format}")),
            "models[0].format",
        );
    }
    rejects(&replace(&a, "    format: tar.bz2\n", ""), "models[0]");
    rejects(
        &replace(
            &a,
            "    license: FunASR Model Open Source License（可商用，署名并写明模型名）\n",
            "",
        ),
        "models[0]",
    );
    rejects(&replace(&a, "    size: 165783878\n", ""), "models[0]");
    rejects(&replace(&a, "size: 165783878", "size: 0"), "models[0].size");
    rejects(
        &replace(
            &a,
            "license: FunASR Model Open Source License（可商用，署名并写明模型名）",
            "license: \"  \"",
        ),
        "models[0].license",
    );
    for name in ["../escape", ".hidden", "Upper", "a/b", ""] {
        rejects(
            &replace(
                &a,
                "name: sense-voice-int8-2025-09-09",
                &format!("name: \"{name}\""),
            ),
            "models[0].name",
        );
    }
    rejects(
        &replace(
            &a,
            "mirrors: [https://example.com/mirror/",
            "mirrors: [file:///etc/",
        ),
        "models[0].mirrors[0]",
    );
    let twice = replace(
        &a,
        "mirrors: [https://example.com/mirror/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2025-09-09.tar.bz2]",
        "mirrors: [https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2025-09-09.tar.bz2]",
    );
    rejects(&twice, "models[0].mirrors[0]");
    let model = &a[a.find("  - name:").unwrap()..a.find("permissions:").unwrap()];
    rejects(
        &replace(&a, "permissions:", &format!("{model}permissions:")),
        "models[1].name",
    );
}
