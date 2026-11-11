//! 用 testdata 的固定样例校验 schema 和类型。

use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use usuyo_contract::{
    CardMeta, Changeset, DraftMeta, JournalEntry, Manifest, NotesMeta, SCHEMAS, TopicMeta,
};

fn read(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn validator(name: &str) -> jsonschema::Validator {
    let (_, text) = SCHEMAS.iter().find(|(n, _)| *n == name).unwrap();
    let schema: Value = serde_json::from_str(text).unwrap();
    assert_eq!(
        schema["$id"],
        Value::String(format!("urn:usuyo:schema:{name}:1"))
    );
    jsonschema::options()
        .should_validate_formats(true)
        .build(&schema)
        .unwrap()
}

/// 样例过 schema，读成类型后没有不认识的字段，再写回和原文按值相等。
fn check_valid<T: DeserializeOwned + Serialize>(name: &str, extra_of: fn(&T) -> usize) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
    let v = validator(name);
    let mut seen = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let file = path.file_name().unwrap().to_str().unwrap().to_owned();
        if !file.starts_with(&format!("{name}.")) || !file.ends_with(".json") {
            continue;
        }
        seen += 1;
        let value = read(&path);
        let errors: Vec<String> = v.iter_errors(&value).map(|e| e.to_string()).collect();
        assert!(errors.is_empty(), "{file}: {errors:?}");
        let typed: T = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(
            extra_of(&typed),
            0,
            "{file} has fields the type does not know"
        );
        assert_eq!(
            serde_json::to_value(&typed).unwrap(),
            value,
            "{file} round trip"
        );
    }
    assert!(seen > 0, "no samples for {name}");
}

#[test]
fn samples_are_valid() {
    check_valid::<Manifest>("manifest", |m| m.extra.len() + m.tags.extra.len());
    check_valid::<JournalEntry>("journal-entry", |e| e.extra.len());
    check_valid::<Changeset>("changeset", |c| c.extra.len());
    check_valid::<TopicMeta>("topic", |t| t.extra.len());
    check_valid::<CardMeta>("card", |t| t.extra.len());
    check_valid::<DraftMeta>("draft", |t| t.extra.len());
    check_valid::<NotesMeta>("notes", |t| t.extra.len());
}

#[test]
fn invalid_samples_fail() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/invalid");
    let mut seen = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let file = path.file_name().unwrap().to_str().unwrap().to_owned();
        let name = file.split('.').next().unwrap();
        assert!(
            !validator(name).is_valid(&read(&path)),
            "{file} should not validate"
        );
        seen += 1;
    }
    assert!(seen >= 7);
}

#[test]
fn unknown_fields_survive_round_trip() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
    let mut value = read(&dir.join("manifest.full.json"));
    value["x-other"] = serde_json::json!({"keep": [1, 2]});
    value["snapshots"][0]["futureField"] = Value::Bool(true);
    value["origins"][0]["x-note"] = Value::String("留着".into());
    let typed: Manifest = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&typed).unwrap(), value);
}

#[test]
fn ops_parse_from_changeset() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
    let c: Changeset = serde_json::from_value(read(&dir.join("changeset.full.json"))).unwrap();
    for op in &c.ops {
        let typed = op.typed().unwrap();
        assert_eq!(typed.name(), op.op);
        let (name, args) = typed.to_parts().unwrap();
        assert_eq!(name, op.op);
        assert_eq!(args, op.args);
    }
    assert!(matches!(
        usuyo_contract::Op::from_parts("no-such-op", &Value::Null),
        Err(usuyo_contract::ContractError::UnknownOp(_))
    ));
}
