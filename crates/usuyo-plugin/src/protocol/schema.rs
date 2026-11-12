//! 从类型生成 JSON Schema（draft 2020-12），给别的语言的插件作者和 plugin-kit 用。
//! 转换插件的 JSON-RPC 消息各一份：`describe`、`health`、`run` 的结果，`run`、`progress`、`cancel` 的参数，`error`。
//! 提交在 `crates/usuyo-plugin/schema/`，测试核对和类型一致。

use schemars::{JsonSchema, Schema, schema_for};
use serde_json::{Value, json};

use super::envelope::{CallFailure, PluginEvent};
use super::fetch::{Canonicalized, FetchHealth, FetchOutput, ListPage, SourcesOutput};
use super::manifest::WireManifest;
use super::transform::{Cancel, Describe, Health, Progress, RpcError, RunParams, RunResult};

/// 成功的信封：子命令的字段加 `ok: true`。
fn success<T: JsonSchema>(title: &str) -> Schema {
    let mut schema = schema_for!(T);
    let obj = schema.ensure_object();
    obj.insert("title".into(), json!(title));
    if let Some(Value::Object(props)) = obj.get_mut("properties") {
        props.insert("ok".into(), json!({"const": true}));
    }
    match obj.get_mut("required") {
        Some(Value::Array(req)) => req.insert(0, json!("ok")),
        _ => {
            obj.insert("required".into(), json!(["ok"]));
        }
    }
    schema
}

/// 失败的信封：`{"ok": false, "error": {...}}`。
fn failure() -> Schema {
    let mut schema = schema_for!(CallFailure);
    let obj = schema.ensure_object();
    let defs = obj.remove("$defs");
    let mut error = obj.clone();
    error.remove("$schema");
    let mut out = serde_json::Map::new();
    out.insert(
        "$schema".into(),
        json!("https://json-schema.org/draft/2020-12/schema"),
    );
    out.insert("title".into(), json!("ErrorOutput"));
    out.insert("type".into(), json!("object"));
    out.insert(
        "properties".into(),
        json!({"ok": {"const": false}, "error": Value::Object(error)}),
    );
    out.insert("required".into(), json!(["ok", "error"]));
    if let Some(defs) = defs {
        out.insert("$defs".into(), defs);
    }
    Schema::from(out)
}

/// 每份 schema 的文件名和内容。
pub fn schemas() -> Vec<(&'static str, Schema)> {
    vec![
        ("manifest.schema.json", schema_for!(WireManifest)),
        (
            "canonicalize-output.schema.json",
            success::<Canonicalized>("CanonicalizeOutput"),
        ),
        (
            "fetch-output.schema.json",
            success::<FetchOutput>("FetchOutput"),
        ),
        (
            "sources-output.schema.json",
            success::<SourcesOutput>("SourcesOutput"),
        ),
        ("list-output.schema.json", success::<ListPage>("ListOutput")),
        (
            "health-output.schema.json",
            success::<FetchHealth>("HealthOutput"),
        ),
        ("error-output.schema.json", failure()),
        ("event.schema.json", schema_for!(PluginEvent)),
        (
            "transform-describe-result.schema.json",
            schema_for!(Describe),
        ),
        ("transform-health-result.schema.json", schema_for!(Health)),
        ("transform-run-params.schema.json", schema_for!(RunParams)),
        ("transform-run-result.schema.json", schema_for!(RunResult)),
        (
            "transform-progress-params.schema.json",
            schema_for!(Progress),
        ),
        ("transform-cancel-params.schema.json", schema_for!(Cancel)),
        ("transform-error.schema.json", schema_for!(RpcError)),
    ]
}
