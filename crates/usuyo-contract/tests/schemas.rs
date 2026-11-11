//! 用 testdata 的固定样例校验 schema 和类型。

use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use usuyo_contract::{
    CardMeta, Changeset, DraftMeta, JournalEntry, Keyframes, Manifest, NotesMeta, Ocr, RunRecord,
    SCHEMAS, TopicMeta,
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
    check_valid::<RunRecord>("run", run_extras);
    check_valid::<Keyframes>("keyframes", |k| {
        k.extra.len() + k.frames.iter().map(|f| f.extra.len()).sum::<usize>()
    });
    check_valid::<Ocr>("ocr", |o| {
        o.extra.len()
            + o.images
                .iter()
                .map(|i| i.extra.len() + i.lines.iter().map(|l| l.extra.len()).sum::<usize>())
                .sum::<usize>()
    });
}

fn run_extras(r: &RunRecord) -> usize {
    r.extra.len()
        + r.tool.as_ref().map_or(0, |t| t.extra.len())
        + r.model.as_ref().map_or(0, |m| m.extra.len())
        + r.inputs
            .iter()
            .map(|i| {
                i.extra.len()
                    + i.via
                        .iter()
                        .map(|v| v.extra.len() + v.tool.as_ref().map_or(0, |t| t.extra.len()))
                        .sum::<usize>()
            })
            .sum::<usize>()
        + r.outputs.iter().map(|o| o.extra.len()).sum::<usize>()
        + r.marks.iter().map(|m| m.extra.len()).sum::<usize>()
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
fn run_unknown_fields_survive_round_trip() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
    let mut value = read(&dir.join("run.asr.json"));
    value["x-other"] = serde_json::json!({"keep": [1, 2]});
    value["inputs"][0]["via"][0]["futureField"] = Value::Bool(true);
    value["outputs"][0]["x-note"] = Value::String("留着".into());
    value["job"] = Value::String("summary".into());
    value["marks"] = serde_json::json!([{"code": "future-mark", "level": 2}]);
    let typed: RunRecord = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&typed).unwrap(), value);

    let mut frames = read(&dir.join("ocr.frames.json"));
    frames["images"][1]["lines"][0]["angle"] = serde_json::json!(3.5);
    let typed: Ocr = serde_json::from_value(frames.clone()).unwrap();
    assert_eq!(serde_json::to_value(&typed).unwrap(), frames);
}

#[test]
fn record_paths_must_stay_inside() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
    let run: RunRecord = serde_json::from_value(read(&dir.join("run.frames.json"))).unwrap();
    assert_eq!(run.bad_path(), None);
    for bad in ["/etc/passwd", "../manifest.json", "raw/../../x", ""] {
        let mut r = run.clone();
        r.inputs[1].path = bad.into();
        assert_eq!(r.bad_path(), Some(bad));
        let mut r = run.clone();
        r.outputs[0].path = bad.into();
        assert_eq!(r.bad_path(), Some(bad));
    }
    let mut kf: Keyframes = serde_json::from_value(read(&dir.join("keyframes.full.json"))).unwrap();
    assert_eq!(kf.bad_path(), None);
    kf.frames[0].path = "../../raw/x.jpg".into();
    assert_eq!(kf.bad_path(), Some("../../raw/x.jpg"));
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
