//! 清单过仓库自己的清单类型，和代码里的名字、版本对得上。

use usuyo_plugin_kit::protocol::{ConsumeKind, PluginKind, PluginManifest, Produce, ProviderNeed};

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
    assert_eq!(m.header.id.as_str(), "builtin/media");
    // 宿主拿它和 `describe` 的版本比。
    assert_eq!(m.header.version.to_string(), env!("CARGO_PKG_VERSION"));
    assert_eq!(m.header.license, "MIT");
    assert_eq!(m.header.run.command, ["${PLUGIN_DIR}/usuyo-plugin-media"]);
    assert!(m.header.permissions.network.hosts.is_empty());

    let PluginKind::Transform(spec) = m.kind else {
        panic!("not a transform plugin");
    };
    let consumed: Vec<ConsumeKind> = spec.consumes.iter().map(|c| c.kind).collect();
    assert_eq!(consumed, [ConsumeKind::Video, ConsumeKind::Audio]);
    assert_eq!(
        spec.produces,
        [
            Produce::Play,
            Produce::AudioPcm16k,
            Produce::Probe,
            Produce::Keyframes
        ]
    );
    assert_eq!(spec.provider, ProviderNeed::None);
    assert!(spec.models.is_empty());
}
