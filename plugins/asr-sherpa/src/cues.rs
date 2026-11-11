//! 从识别结果到 `asr.vtt`（feature 003 §5.3）：一个 VAD 段一条 cue，长于 `maxCueSec` 的按 token 时间点在标点处切开；
//! 去掉 `<|zh|>` 一类标签；开头一行 `NOTE` 写语言。都不碰模型，单独测。

/// 一条字幕，秒数相对整段音频。
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

/// 一个 VAD 段的识别结果：整句、逐 token 的字和起点（秒，相对这一段）。
pub struct Recognized<'a> {
    pub text: &'a str,
    pub tokens: &'a [String],
    /// 和 `tokens` 一样长时才用；sherpa-onnx 不给时为 `None`。
    pub timestamps: Option<&'a [f32]>,
}

/// 一个段切成 cue。段不长于 `max_cue` 时整段一条；更长时在窗口里最后一个标点后面切，
/// 窗口里没有标点就切在越过窗口的那个 token 前面。每条的结束时间不超过开始加 `max_cue`。
pub fn segment_cues(seg_start: f64, seg_dur: f64, r: &Recognized<'_>, max_cue: f64) -> Vec<Cue> {
    let times = r
        .timestamps
        .filter(|t| t.len() == r.tokens.len() && !t.is_empty());
    let Some(times) = times.filter(|_| seg_dur > max_cue) else {
        return vec![Cue {
            start: seg_start,
            end: seg_start + seg_dur,
            text: collapse_spaces(&strip_tags(r.text)),
        }];
    };
    let at = |i: usize| f64::from(times[i]);
    let mut cues = Vec::new();
    let push = |cues: &mut Vec<Cue>, from: usize, to: usize, start: f64, end: f64| {
        cues.push(Cue {
            start: seg_start + start,
            end: seg_start + end.min(start + max_cue),
            text: join_tokens(&r.tokens[from..to]),
        });
    };
    let mut first = 0usize;
    let mut cue_start = 0.0f64;
    let mut last_punct: Option<usize> = None;
    for i in 0..r.tokens.len() {
        if at(i) - cue_start > max_cue && i > first {
            let cut = match last_punct {
                Some(p) if p >= first => p + 1,
                _ => (i - 1).max(first + 1),
            };
            push(&mut cues, first, cut, cue_start, at(cut));
            first = cut;
            cue_start = at(cut);
            last_punct = None;
        }
        if is_punct(&r.tokens[i]) {
            last_punct = Some(i);
        }
    }
    push(&mut cues, first, r.tokens.len(), cue_start, seg_dur);
    cues
}

/// 去掉 `<|…|>` 标签。sherpa-onnx 2.0.1 的结果里已经没有它们（运行库自己去掉），这里防它以后变。
pub fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("<|") {
        out.push_str(&rest[..i]);
        rest = rest[i..].find("|>").map_or("", |j| &rest[i + j + 2..]);
    }
    out.push_str(rest);
    out
}

fn is_punct(token: &str) -> bool {
    token
        .trim_end()
        .chars()
        .last()
        .is_some_and(|c| "，。！？；、,.!?;".contains(c))
}

/// 把 token 拼回字：`▁` 是词前的空格。
fn join_tokens(tokens: &[String]) -> String {
    let s: String = tokens.iter().map(|t| t.replace('\u{2581}', " ")).collect();
    collapse_spaces(&strip_tags(&s))
}

fn collapse_spaces(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 按文字的种类猜语言。sherpa-onnx 的安全接口不给 SenseVoice 认出的语言（README 有说明），
/// 只能这样猜；分不出普通话和粤语。
pub fn guess_language(text: &str) -> &'static str {
    let (mut han, mut kana, mut hangul, mut latin) = (0usize, 0usize, 0usize, 0usize);
    for c in text.chars() {
        match c as u32 {
            0x3040..=0x30FF => kana += 1,
            0xAC00..=0xD7AF => hangul += 1,
            0x4E00..=0x9FFF | 0x3400..=0x4DBF => han += 1,
            _ if c.is_ascii_alphabetic() => latin += 1,
            _ => {}
        }
    }
    if kana > 0 && kana * 5 >= han {
        "ja"
    } else if hangul > han {
        "ko"
    } else if han > 0 {
        "zh"
    } else if latin > 0 {
        "en"
    } else {
        "und"
    }
}

/// feature 003 §6 的 `no-speech`：人声加起来不到 1 秒，或不到时长的 2%。
pub fn is_no_speech(speech_sec: f64, duration_sec: f64) -> bool {
    speech_sec < 1.0 || speech_sec < 0.02 * duration_sec
}

/// 写 WebVTT：头、`NOTE language: <lang>`、每条非空的 cue。cue 的字照 WebVTT 转义 `&`、`<`、`>`。
pub fn write_vtt(lang: &str, cues: &[Cue]) -> String {
    let mut vtt = format!("WEBVTT\n\nNOTE language: {lang}\n");
    for c in cues.iter().filter(|c| !c.text.is_empty()) {
        vtt.push_str(&format!(
            "\n{} --> {}\n{}\n",
            timestamp(c.start),
            timestamp(c.end),
            escape(&c.text)
        ));
    }
    vtt
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn timestamp(sec: f64) -> String {
    let ms = (sec.max(0.0) * 1000.0).round() as u64;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1000 % 60,
        ms % 1000
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(s: &[&str]) -> Vec<String> {
        s.iter().map(|t| t.to_string()).collect()
    }

    #[test]
    fn short_segment_is_one_cue() {
        let toks = tokens(&["大", "家", "好", "。"]);
        let r = Recognized {
            text: "<|zh|><|NEUTRAL|><|Speech|><|withitn|>大家好。",
            tokens: &toks,
            timestamps: Some(&[0.0, 0.2, 0.4, 0.6]),
        };
        let cues = segment_cues(3.0, 1.5, &r, 15.0);
        assert_eq!(
            cues,
            vec![Cue {
                start: 3.0,
                end: 4.5,
                text: "大家好。".into()
            }]
        );
    }

    #[test]
    fn long_segment_splits_after_punctuation() {
        // 每 1 秒一个 token，第 6 个后面有逗号，第 14 个后面有句号；最长 10 秒。
        let mut toks = Vec::new();
        let mut times = Vec::new();
        for i in 0..24 {
            let t = match i {
                5 => "五，".to_string(),
                13 => "完。".to_string(),
                _ => format!("字{i}"),
            };
            toks.push(t);
            times.push(i as f32);
        }
        let r = Recognized {
            text: "",
            tokens: &toks,
            timestamps: Some(&times),
        };
        let cues = segment_cues(100.0, 24.5, &r, 10.0);
        assert_eq!(cues.len(), 3, "{cues:?}");
        assert!(cues[0].text.ends_with("五，"));
        assert_eq!((cues[0].start, cues[0].end), (100.0, 106.0));
        assert!(cues[1].text.starts_with("字6") && cues[1].text.ends_with("完。"));
        assert_eq!((cues[1].start, cues[1].end), (106.0, 114.0));
        assert_eq!((cues[2].start, cues[2].end), (114.0, 124.0));
        for c in &cues {
            assert!(c.end - c.start <= 10.0 + 1e-9, "{c:?}");
        }
    }

    #[test]
    fn long_segment_without_punctuation_splits_at_max_cue() {
        let toks: Vec<String> = (0..30).map(|i| format!("\u{2581}w{i}")).collect();
        let times: Vec<f32> = (0..30).map(|i| i as f32).collect();
        let r = Recognized {
            text: "",
            tokens: &toks,
            timestamps: Some(&times),
        };
        let cues = segment_cues(0.0, 30.0, &r, 15.0);
        assert!(cues.len() >= 2, "{cues:?}");
        for c in &cues {
            assert!(c.end - c.start <= 15.0 + 1e-9, "{c:?}");
            assert!(!c.text.starts_with(' ') && !c.text.contains("  "));
        }
        assert!(cues[0].text.starts_with("w0 w1"));
        let joined: Vec<&str> = cues.iter().flat_map(|c| c.text.split(' ')).collect();
        assert_eq!(joined.len(), 30, "no token lost or doubled");
    }

    #[test]
    fn timestamps_that_do_not_line_up_keep_the_segment_whole() {
        let toks = tokens(&["a", "b"]);
        let r = Recognized {
            text: "a b",
            tokens: &toks,
            timestamps: Some(&[0.0]),
        };
        assert_eq!(segment_cues(0.0, 20.0, &r, 15.0).len(), 1);
    }

    #[test]
    fn tags_are_stripped() {
        assert_eq!(strip_tags("<|zh|><|NEUTRAL|>你好<|x|>呀"), "你好呀");
        assert_eq!(strip_tags("a <| unterminated"), "a ");
        assert_eq!(strip_tags("no tags"), "no tags");
    }

    #[test]
    fn language_from_script() {
        assert_eq!(guess_language("今天讲手冲咖啡"), "zh");
        assert_eq!(guess_language("we talk about coffee"), "en");
        assert_eq!(guess_language("コーヒーの話"), "ja");
        assert_eq!(guess_language("커피 이야기"), "ko");
        assert_eq!(guess_language("我们开了一个 meeting"), "zh");
        assert_eq!(guess_language("123 。"), "und");
    }

    #[test]
    fn no_speech_rule() {
        assert!(is_no_speech(0.9, 5.0));
        assert!(!is_no_speech(1.0, 5.0));
        assert!(is_no_speech(10.0, 600.0));
        assert!(!is_no_speech(12.0, 600.0));
    }

    #[test]
    fn vtt_is_written_and_escaped() {
        let cues = vec![
            Cue {
                start: 0.25,
                end: 3_725.5,
                text: "A <b> & C --> D".into(),
            },
            Cue {
                start: 4.0,
                end: 5.0,
                text: String::new(),
            },
        ];
        assert_eq!(
            write_vtt("zh", &cues),
            "WEBVTT\n\nNOTE language: zh\n\n00:00:00.250 --> 01:02:05.500\nA &lt;b&gt; &amp; C --&gt; D\n"
        );
    }
}
