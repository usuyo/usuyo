//! `run` 的 `options`：读进来、补默认值、原样回显（feature 003 §5.2）。宿主重做时把回显的那份原样交回，
//! 所以回显的每个键这里都认。
//!
//! 清单 `settings` 只声明了 `model`、`language`、`itn`、`threads`：设置的键只能是小写加 `-`，类型没有小数，
//! `vadThreshold`、`maxCueSec` 只能经 `run` 的 `options` 交进来，默认值在这里。

use serde_json::{Map, Value};

/// 清单 `models[].value` 对应的设置值。
pub const MODEL_SETTING: &str = "sense-voice";
/// 切段用的 VAD，资产里那份。
pub const VAD_NAME: &str = "silero-vad-v5";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Auto,
    Zh,
    En,
    Ja,
    Ko,
    Yue,
}

impl Language {
    const ALL: [Language; 6] = [
        Language::Auto,
        Language::Zh,
        Language::En,
        Language::Ja,
        Language::Ko,
        Language::Yue,
    ];

    /// SenseVoice 认的写法，也是设置里的值。
    pub fn as_str(self) -> &'static str {
        match self {
            Language::Auto => "auto",
            Language::Zh => "zh",
            Language::En => "en",
            Language::Ja => "ja",
            Language::Ko => "ko",
            Language::Yue => "yue",
        }
    }

    /// 指定了语言时就是它；`auto` 没有。
    pub fn fixed(self) -> Option<&'static str> {
        (self != Language::Auto).then(|| self.as_str())
    }
}

/// 一次转写用的参数。
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    pub language: Language,
    /// SenseVoice 出标点和数字（逆文本规范化）。
    pub itn: bool,
    pub vad_threshold: f64,
    pub max_cue_sec: f64,
    pub threads: u32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            language: Language::Auto,
            itn: true,
            vad_threshold: 0.5,
            max_cue_sec: 15.0,
            threads: 2,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("option `{key}`: {why}")]
pub struct OptionError {
    key: String,
    why: String,
}

fn bad(key: &str, why: impl Into<String>) -> OptionError {
    OptionError {
        key: key.to_owned(),
        why: why.into(),
    }
}

impl Options {
    /// 读 `run` 的 `options`；没给的取默认，不认识的键和越界的值都报错。
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, OptionError> {
        let mut o = Options::default();
        for (key, value) in map {
            match key.as_str() {
                "model" => expect_str(key, value, MODEL_SETTING)?,
                "vad" => expect_str(key, value, VAD_NAME)?,
                "language" => {
                    let s = value.as_str().ok_or_else(|| bad(key, "must be a string"))?;
                    o.language = Language::ALL
                        .into_iter()
                        .find(|l| l.as_str() == s)
                        .ok_or_else(|| {
                            bad(key, format!("`{s}` is not auto, zh, en, ja, ko or yue"))
                        })?;
                }
                "itn" => o.itn = value.as_bool().ok_or_else(|| bad(key, "must be a bool"))?,
                "vadThreshold" => {
                    o.vad_threshold = number(key, value)?;
                    if !(o.vad_threshold > 0.0 && o.vad_threshold < 1.0) {
                        return Err(bad(key, "must be between 0 and 1"));
                    }
                }
                "maxCueSec" => {
                    o.max_cue_sec = number(key, value)?;
                    if !(1.0..=30.0).contains(&o.max_cue_sec) {
                        return Err(bad(key, "must be 1 to 30 seconds"));
                    }
                }
                "threads" => {
                    o.threads = value
                        .as_u64()
                        .filter(|n| (1..=16).contains(n))
                        .and_then(|n| u32::try_from(n).ok())
                        .ok_or_else(|| bad(key, "must be an integer from 1 to 16"))?;
                }
                _ => return Err(bad(key, "is not an option of this plugin")),
            }
        }
        Ok(o)
    }

    /// 实际用的参数，含默认值；结果里的 `options`。
    pub fn to_map(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("model".into(), MODEL_SETTING.into());
        m.insert("language".into(), self.language.as_str().into());
        m.insert("itn".into(), self.itn.into());
        m.insert("vad".into(), VAD_NAME.into());
        m.insert("vadThreshold".into(), self.vad_threshold.into());
        m.insert("maxCueSec".into(), self.max_cue_sec.into());
        m.insert("threads".into(), self.threads.into());
        m
    }
}

fn expect_str(key: &str, value: &Value, only: &str) -> Result<(), OptionError> {
    if value.as_str() == Some(only) {
        Ok(())
    } else {
        Err(bad(key, format!("only `{only}` is available")))
    }
}

fn number(key: &str, value: &Value) -> Result<f64, OptionError> {
    value
        .as_f64()
        .filter(|v| v.is_finite())
        .ok_or_else(|| bad(key, "must be a number"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn map(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn defaults_fill_in_and_echo_round_trips() {
        let o = Options::from_map(&map(json!({"vadThreshold": 0.4}))).unwrap();
        assert_eq!(o.vad_threshold, 0.4);
        assert_eq!(o.threads, 2);
        let echo = o.to_map();
        assert_eq!(
            Value::Object(echo.clone()),
            json!({"model": "sense-voice", "language": "auto", "itn": true, "vad": "silero-vad-v5",
                   "vadThreshold": 0.4, "maxCueSec": 15.0, "threads": 2})
        );
        // 重做时宿主把回显原样交回。
        assert_eq!(Options::from_map(&echo).unwrap(), o);
    }

    #[test]
    fn settings_from_the_manifest_are_accepted() {
        let o = Options::from_map(&map(
            json!({"model": "sense-voice", "language": "yue", "itn": false, "threads": 1, "maxCueSec": 10}),
        ))
        .unwrap();
        assert_eq!(o.language, Language::Yue);
        assert!(!o.itn);
        assert_eq!(o.max_cue_sec, 10.0);
    }

    #[test]
    fn bad_values_are_refused() {
        for bad in [
            json!({"language": "fr"}),
            json!({"itn": "yes"}),
            json!({"vadThreshold": 1.5}),
            json!({"vadThreshold": "0.5"}),
            json!({"maxCueSec": 0.5}),
            json!({"threads": 0}),
            json!({"threads": 2.5}),
            json!({"model": "fire-red"}),
            json!({"vad": "ten-vad"}),
            json!({"beam": 4}),
        ] {
            assert!(Options::from_map(&map(bad.clone())).is_err(), "{bad}");
        }
    }
}
