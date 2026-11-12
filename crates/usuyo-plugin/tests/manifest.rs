//! 清单的合规和不合规样例（协议 §2、§10）。

use usuyo_plugin::PluginError;
use usuyo_plugin::protocol::{
    Access, Account, ConsumeKind, Delivers, FetchInputKind, Login, ModelFormat, OutputKind,
    PluginKind, PluginManifest, Produce,
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
fn the_core_fixture_plugin_parses() {
    let text = std::fs::read_to_string(format!(
        "{}/../usuyo-core/testdata/transform/fixture/usuyo-plugin.yaml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let m = parse(&text).unwrap();
    assert_eq!(m.fetch().unwrap().methods[0].id, "sample");
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
    // 顶层的层级由做了的方法算：没做的手机镜像不算
    assert_eq!(
        (spec.access(), spec.login()),
        (Access::BrowserSession, Login::Required)
    );
    let ids: Vec<&str> = spec.methods.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["share-page", "favorites", "userscript", "iphone"]);
    let share = spec.method("share-page").unwrap();
    assert!(share.built && share.callable() && share.on_by_default());
    assert_eq!(share.account, Account::None);
    let fetch = share.capabilities.fetch.as_ref().unwrap();
    assert!(fetch.outputs.contains(&OutputKind::Comments));
    assert_eq!(fetch.input, [FetchInputKind::Url].into());
    assert_eq!(fetch.uses.timeout.unwrap().get().as_secs(), 600);
    let favorites = spec.method("favorites").unwrap();
    assert_eq!(favorites.sites, ["www.douyin.com"]);
    assert!(favorites.capabilities.unfavorite.is_some());
    assert!(
        favorites
            .risk
            .as_ref()
            .is_some_and(|r| r.source.url.starts_with("https://"))
    );
    let script = spec.method("userscript").unwrap();
    assert_eq!(script.delivers, Some(Delivers::Links));
    assert!(script.built && !script.callable());
    let phone = spec.method("iphone").unwrap();
    assert!(!phone.built && !phone.callable());
    assert_eq!(m.header.permissions.exec, ["chrome"]);
    assert_eq!(m.header.permissions.limits.memory.unwrap().bytes(), 1 << 30);

    let m = parse(&testdata("manifest.import.yaml")).unwrap();
    let spec = m.fetch().unwrap();
    assert!(spec.match_patterns.is_empty());
    assert_eq!(
        spec.methods[0].capabilities.fetch.as_ref().unwrap().input,
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
        &replace(&w, "outputs: [text, html, images, meta]", "input: [url]"),
        "methods[0].capabilities.fetch.outputs",
    );
    rejects(
        &replace(&w, "outputs: [text, html", "outputs: [video, html"),
        "methods[0].capabilities.fetch.outputs[0]",
    );
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
            "      fetch:\n        outputs",
            "      fetch:\n        timeout: \"10\"\n        outputs",
        ),
        "methods[0].capabilities.fetch.timeout",
    );
    rejects(
        &replace(
            &w,
            "      fetch:\n        outputs",
            "      fetch:\n        order: unordered\n        outputs",
        ),
        "methods[0].capabilities.fetch.order",
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
            "    sites: [www.douyin.com]\n    interaction",
            "    sites: [www.x.com]\n    interaction",
        ),
        "methods[1].sites",
    );
    rejects(
        &replace(
            &d,
            "    account: none\n    capabilities",
            "    account: none\n    sites: [www.douyin.com]\n    capabilities",
        ),
        "methods[0].sites",
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
        "      fetch:\n        outputs",
        "      fetch:\n        timeout: 1000h\n        outputs",
    ))
    .unwrap();
    let t = m.fetch().unwrap().methods[0]
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
        "methods:\n  - id: page\n    name: 公开网页\n    access: public-web\n    login: none\n    account: none\n",
        "visitors:\n  - site: www.example.com\n    name: 例子\n    url: https://www.example.com/\n    cookies: [vid]\n    domains: [\".example.com\"]\nmethods:\n  - id: page\n    name: 公开网页\n    access: public-web\n    login: none\n    account: visitor\n    visitors: [www.example.com]\n",
    );
    let m = parse(&with).unwrap();
    let spec = m.fetch().unwrap();
    assert!(spec.sites.is_empty());
    assert_eq!(spec.visitors[0].cookies, ["vid"]);
    assert_eq!(spec.methods[0].visitors, ["www.example.com"]);

    rejects(
        &replace(
            &with,
            "visitors: [www.example.com]",
            "visitors: [www.other.test]",
        ),
        "methods[0].visitors",
    );
    rejects(
        &replace(&with, "    visitors: [www.example.com]\n", ""),
        "methods[0].visitors",
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
    // 访客身份不让 `login: none` 的方法用上账号站点
    rejects(
        &replace(
            &with,
            "    visitors: [www.example.com]\n",
            "    visitors: [www.example.com]\n    sites: [www.example.com]\n",
        ),
        "methods[0].sites",
    );
    rejects(
        &format!("{}visitors: []\n", testdata("manifest.transform.yaml")),
        "visitors",
    );
}

#[test]
fn methods_are_checked() {
    let d = douyin();
    // 旧形状说清楚换成了什么
    let old = "access: public-web\nlogin: none\ncapabilities:\n  fetch:\n    access: public-web\n    login: none\n    outputs: [text]\n";
    let message = rejects(
        &replace(&web(), "methods:\n", &format!("{old}methods:\n")),
        "access",
    );
    assert!(message.contains("methods"), "{message}");
    rejects(
        &replace(&web(), "methods:\n", "capabilities: {}\nmethods:\n"),
        "capabilities",
    );
    let message = rejects(
        &replace(
            &web(),
            "      fetch:\n        outputs",
            "      fetch:\n        login: none\n        outputs",
        ),
        "methods[0].capabilities.fetch.login",
    );
    assert!(message.contains("method"), "{message}");
    rejects(
        &replace(&d, "  - id: favorites", "  - id: share-page"),
        "methods[1].id",
    );
    rejects(
        &replace(&d, "  - id: favorites", "  - id: Favorites"),
        "methods[1].id",
    );
    // 做了、在插件里跑的要有能力；交链接的不能有
    rejects(
        &replace(&d, "    delivers: links\n", ""),
        "methods[2].capabilities",
    );
    rejects(
        &replace(
            &d,
            "    delivers: links\n",
            "    delivers: links\n    capabilities:\n      fetch: {outputs: [text]}\n",
        ),
        "methods[2].capabilities",
    );
    // 只有登录进 usuyo 的方法拿得到登录站点
    rejects(
        &replace(
            &d,
            "    account: own-browser\n",
            "    account: own-browser\n    sites: [www.douyin.com]\n",
        ),
        "methods[2].sites",
    );
    rejects(
        &replace(
            &d,
            "    account: in-usuyo\n    sites: [www.douyin.com]\n",
            "    account: in-usuyo\n",
        ),
        "methods[1].sites",
    );
    // 没有一种做了的不合格；没做的可以不写能力
    let none_built = replace(
        &web(),
        "    account: none\n",
        "    account: none\n    built: false\n",
    );
    rejects(&none_built, "methods");
    // 顶层的站点要有方法用它
    rejects(
        &replace(
            &d,
            "    account: in-usuyo\n    sites: [www.douyin.com]\n",
            "    account: own-app\n",
        ),
        "sites",
    );
    // 出处要是链接
    rejects(
        &replace(
            &d,
            "url: \"https://github.com/usuyo",
            "url: \"github.com/usuyo",
        ),
        "methods[1].risk.source.url",
    );
    // 来源从链接开始的写一句给链接框
    let link = replace(
        &d,
        "        order: newest-first\n",
        "        order: newest-first\n        link: 收藏夹的链接\n",
    );
    let m = parse(&link).unwrap();
    let list = m
        .fetch()
        .unwrap()
        .method("favorites")
        .unwrap()
        .capabilities
        .list
        .clone();
    assert_eq!(list.unwrap().link.as_deref(), Some("收藏夹的链接"));
    rejects(
        &replace(&link, "link: 收藏夹的链接", "link: \"  \""),
        "methods[1].capabilities.list.link",
    );
    rejects(
        &replace(
            &web(),
            "      fetch:\n        outputs",
            "      fetch:\n        link: x\n        outputs",
        ),
        "methods[0].capabilities.fetch.link",
    );
}

#[test]
fn private_api_and_client_hook_methods_are_off_by_default() {
    let m = parse(&replace(
        &web(),
        "access: public-web\n    login",
        "access: private-api\n    login",
    ))
    .unwrap();
    assert!(!m.fetch().unwrap().methods[0].on_by_default());
    let m = parse(&replace(
        &web(),
        "access: public-web\n    login",
        "access: client-hook\n    login",
    ))
    .unwrap();
    assert!(!m.fetch().unwrap().methods[0].on_by_default());
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

#[test]
fn per_platform_tier_sets_the_default_switch_and_rank_on_that_platform() {
    let yaml = replace(&web(), "platforms: [web]", "platforms: [web, youtube]");
    let yaml = replace(
        &yaml,
        "    account: none\n",
        "    account: none\n    perPlatform:\n      youtube: {access: private-api}\n",
    );
    let m = parse(&yaml).unwrap();
    let spec = m.fetch().unwrap();
    let method = &spec.methods[0];
    let declared = &spec.platforms;
    assert!(method.on_by_default());
    assert!(method.on_by_default_on("web", declared));
    assert!(
        !method.on_by_default_on("youtube", declared),
        "这一层抬高了的平台默认关着"
    );
    assert_eq!(method.access_on("youtube", declared), Access::PrivateApi);
    assert_eq!(method.rank_on("youtube", declared).0, Access::PrivateApi);
    assert_eq!(method.rank_on("web", declared).0, Access::PublicWeb);
    // 不在 `platforms` 里的平台（认不出、落成别的平台的链接）照最高的那一层
    assert_eq!(method.access_on("other", declared), Access::PrivateApi);
}

#[test]
fn unbuilt_methods_declare_what_they_would_get() {
    // 没做的写 `list` 可以不写 `order`：宿主不调它，只拿来说会取得什么
    let m = parse(&replace(
        &douyin(),
        "    account: own-phone\n    interaction: foreground\n",
        "    account: own-phone\n    interaction: foreground\n    drives: iphone\n    capabilities:\n      list:\n        sources:\n          - {source: \"douyin:likes\", title: 喜欢, action: like}\n",
    ))
    .unwrap();
    let spec = m.fetch().unwrap();
    let iphone = spec.method("iphone").unwrap();
    assert!(!iphone.callable());
    assert_eq!(iphone.drives, Some(usuyo_plugin::protocol::Drives::Iphone));
    let list = iphone.capabilities.list.as_ref().unwrap();
    assert_eq!(list.sources[0].title, "喜欢");
    // 做了的照旧要写
    rejects(
        &replace(
            &web(),
            "      fetch:\n        outputs: [text, html, images, meta]",
            "      list:\n        sources: []",
        ),
        "methods[0].capabilities.list.order",
    );
}

#[test]
fn drives_goes_with_where_the_account_is() {
    let base = replace(
        &douyin(),
        "    account: own-phone\n    interaction: foreground\n",
        "    account: own-phone\n    interaction: foreground\n    drives: desktop-app\n",
    );
    rejects(&base, "methods[3].drives");
    let ok = replace(
        &douyin(),
        "    account: own-phone\n    interaction: foreground\n",
        "    account: own-app\n    interaction: foreground\n    drives: desktop-app\n",
    );
    parse(&ok).unwrap();
}

#[test]
fn file_readers_say_what_they_read() {
    let text = testdata("manifest.import.yaml");
    let m = parse(&text).unwrap();
    let list = m.fetch().unwrap().methods[0]
        .capabilities
        .list
        .clone()
        .unwrap();
    assert!(list.reads.unwrap().contains("Takeout"));
    let without = text
        .lines()
        .filter(|l| !l.trim_start().starts_with("reads:"))
        .collect::<Vec<_>>()
        .join("\n");
    rejects(&without, "methods[0].capabilities.list.reads");
}
