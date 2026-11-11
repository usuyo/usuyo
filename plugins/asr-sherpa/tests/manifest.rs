//! 清单过仓库自己的清单类型，和代码里的名字、版本对得上。

use usuyo_plugin_asr_sherpa::engine::MODEL_NAME;
use usuyo_plugin_asr_sherpa::options::{Language, MODEL_SETTING, Options};
use usuyo_plugin_kit::protocol::{
    ConsumeKind, ModelFormat, PluginKind, PluginManifest, Produce, ProviderNeed,
};

const MANIFEST: &str = include_str!("../usuyo-plugin.yaml");

#[test]
fn manifest_parses_and_matches_the_code() {
    let parsed = PluginManifest::from_yaml(MANIFEST).expect("manifest is valid");
    assert!(
        parsed.unknown_fields.is_empty(),
        "unknown fields: {:?}",
        parsed.unknown_fields
    );
    let m = parsed.value;
    assert_eq!(m.header.id.as_str(), "builtin/asr-sherpa");
    // 宿主拿它和 `describe` 的版本比。
    assert_eq!(m.header.version.to_string(), env!("CARGO_PKG_VERSION"));
    assert_eq!(m.header.license, "MIT");
    assert_eq!(
        m.header.run.command,
        ["${PLUGIN_DIR}/usuyo-plugin-asr-sherpa"]
    );
    assert!(m.header.permissions.network.hosts.is_empty());

    let PluginKind::Transform(spec) = m.kind else {
        panic!("not a transform plugin");
    };
    assert_eq!(spec.consumes.len(), 1);
    assert_eq!(spec.consumes[0].kind, ConsumeKind::AudioPcm16k);
    assert_eq!(spec.produces, [Produce::Asr]);
    assert_eq!(spec.provider, ProviderNeed::None);

    let [model] = spec.models.as_slice() else {
        panic!("one model expected");
    };
    assert_eq!(model.name, MODEL_NAME);
    assert_eq!(
        model.sha256,
        "7d1efa2138a65b0b488df37f8b89e3d91a60676e416f515b952358d83dfd347e"
    );
    assert_eq!(model.size, 163_002_883);
    assert_eq!(model.format, ModelFormat::TarBz2);
    assert!(model.url.ends_with("int8-2024-07-17.tar.bz2"));
    assert_eq!(model.setting.as_deref(), Some("model"));
    assert_eq!(model.value.as_deref(), Some(MODEL_SETTING));
}

#[test]
fn setting_defaults_are_the_option_defaults() {
    let m = PluginManifest::from_yaml(MANIFEST).unwrap().value;
    let defaults: serde_json::Map<String, serde_json::Value> = m
        .header
        .settings
        .iter()
        .map(|s| {
            (
                s.key.clone(),
                s.default.clone().expect("every setting has a default"),
            )
        })
        .collect();
    let o = Options::from_map(&defaults).expect("setting defaults are valid options");
    assert_eq!(o, Options::default());
    assert_eq!(o.language, Language::Auto);
}
