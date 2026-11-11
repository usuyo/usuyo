//! WP0 spike: 16 kHz mono WAV -> WebVTT with silero VAD v5 + SenseVoice-Small int8.
//!
//! Usage: spike-asr <in.wav> <out.vtt> --model <dir> --vad <silero_vad_v5.onnx>
//!        [--threads N] [--lang auto] [--vad-threshold 0.5] [--max-cue 15]
//! `--threads 0` leaves the crate default in place.
//! Prints one JSON line of timings to stderr. With no speech, writes nothing and
//! prints `empty: no-speech …` to stdout. `SPIKE_DEBUG=1` dumps each raw result.

use std::fmt::Write as _;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use sherpa_onnx::{
    OfflineRecognizer, OfflineRecognizerConfig, OfflineRecognizerResult,
    OfflineSenseVoiceModelConfig, SileroVadModelConfig, VadModelConfig, VoiceActivityDetector,
    Wave,
};

const SAMPLE_RATE: i32 = 16_000;
const VAD_WINDOW: usize = 512;

struct Args {
    input: String,
    output: String,
    model_dir: String,
    vad: String,
    threads: i32,
    lang: String,
    vad_threshold: f32,
    max_cue: f32,
}

fn parse_args() -> Result<Args> {
    let mut it = std::env::args().skip(1);
    let mut pos = Vec::new();
    let (mut model_dir, mut vad, mut lang) = (None, None, "auto".to_string());
    let (mut threads, mut vad_threshold, mut max_cue) = (0, 0.5, 15.0);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--model" => model_dir = it.next(),
            "--vad" => vad = it.next(),
            "--lang" => lang = it.next().context("--lang")?,
            "--threads" => threads = it.next().context("--threads")?.parse()?,
            "--vad-threshold" => vad_threshold = it.next().context("--vad-threshold")?.parse()?,
            "--max-cue" => max_cue = it.next().context("--max-cue")?.parse()?,
            _ => pos.push(a),
        }
    }
    let [input, output] =
        <[String; 2]>::try_from(pos).map_err(|_| anyhow::anyhow!("need <in.wav> <out.vtt>"))?;
    Ok(Args {
        input,
        output,
        model_dir: model_dir.context("--model")?,
        vad: vad.context("--vad")?,
        threads,
        lang,
        vad_threshold,
        max_cue,
    })
}

struct Cue {
    start: f32,
    end: f32,
    text: String,
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let t0 = Instant::now();

    let wave = Wave::read(&args.input).with_context(|| format!("read {}", args.input))?;
    if wave.sample_rate() != SAMPLE_RATE {
        bail!("expected {SAMPLE_RATE} Hz, got {}", wave.sample_rate());
    }
    let samples = wave.samples();
    let duration = samples.len() as f32 / SAMPLE_RATE as f32;

    let t_load = Instant::now();
    let vad_config = VadModelConfig {
        silero_vad: SileroVadModelConfig {
            model: Some(args.vad.clone()),
            threshold: args.vad_threshold,
            min_silence_duration: 0.5,
            min_speech_duration: 0.25,
            window_size: VAD_WINDOW as i32,
            max_speech_duration: 30.0,
        },
        sample_rate: SAMPLE_RATE,
        num_threads: 1,
        ..Default::default()
    };
    let vad = VoiceActivityDetector::create(&vad_config, 60.0).context("create VAD")?;

    let mut config = OfflineRecognizerConfig::default();
    config.model_config.sense_voice = OfflineSenseVoiceModelConfig {
        model: Some(format!("{}/model.int8.onnx", args.model_dir)),
        language: Some(args.lang.clone()),
        use_itn: true,
    };
    config.model_config.tokens = Some(format!("{}/tokens.txt", args.model_dir));
    if args.threads > 0 {
        config.model_config.num_threads = args.threads;
    }
    let recognizer = OfflineRecognizer::create(&config).context("create recognizer")?;
    let load_sec = t_load.elapsed().as_secs_f32();

    // Segments are decoded as the VAD emits them, so memory does not grow with duration.
    let t_run = Instant::now();
    let mut speech_sec = 0.0f32;
    let mut segments = 0usize;
    let mut cues = Vec::new();
    let mut text_all = String::new();
    let mut decode = |vad: &VoiceActivityDetector| -> Result<()> {
        while let Some(seg) = vad.front() {
            let start = seg.start() as f32 / SAMPLE_RATE as f32;
            let seg_samples = seg.samples();
            let seg_dur = seg_samples.len() as f32 / SAMPLE_RATE as f32;
            speech_sec += seg_dur;
            segments += 1;
            let stream = recognizer.create_stream();
            stream.accept_waveform(SAMPLE_RATE, seg_samples);
            recognizer.decode(&stream);
            let result = stream.get_result().context("recognizer result")?;
            if std::env::var_os("SPIKE_DEBUG").is_some() {
                eprintln!(
                    "text={:?}\ntokens={:?}\ntimestamps={:?}",
                    result.text, result.tokens, result.timestamps
                );
            }
            text_all.push_str(&result.text);
            cues.extend(segment_cues(start, seg_dur, &result, args.max_cue));
            vad.pop();
        }
        Ok(())
    };
    for chunk in samples.chunks(VAD_WINDOW) {
        vad.accept_waveform(chunk);
        decode(&vad)?;
    }
    vad.flush();
    decode(&vad)?;
    let run_sec = t_run.elapsed().as_secs_f32();

    let longest = cues.iter().map(|c| c.end - c.start).fold(0.0, f32::max);
    let report = |cues: usize, lang: &str| {
        let total = t0.elapsed().as_secs_f32();
        eprintln!(
            "{{\"durationSec\":{duration:.3},\"speechSec\":{speech_sec:.3},\"segments\":{segments},\"cues\":{cues},\"longestCueSec\":{longest:.2},\"lang\":\"{lang}\",\"threads\":{},\"loadSec\":{load_sec:.3},\"runSec\":{run_sec:.3},\"totalSec\":{total:.3},\"rtfRun\":{:.4},\"rtfTotal\":{:.4}}}",
            args.threads,
            run_sec / duration,
            total / duration
        );
    };

    // §6 no-speech: under 1 s of speech in total, or under 2 % of the duration.
    if speech_sec < 1.0 || speech_sec < 0.02 * duration {
        report(0, "");
        println!("empty: no-speech speechSec={speech_sec:.3} durationSec={duration:.3}");
        return Ok(());
    }

    let lang = script_lang(&text_all);
    let mut vtt = format!("WEBVTT\n\nNOTE language: {lang}\n");
    let mut written = 0;
    for c in cues.iter().filter(|c| !c.text.is_empty()) {
        write!(vtt, "\n{} --> {}\n{}\n", ts(c.start), ts(c.end), c.text)?;
        written += 1;
    }
    std::fs::write(&args.output, vtt).with_context(|| format!("write {}", args.output))?;
    report(written, lang);
    Ok(())
}

/// The safe `OfflineRecognizerResult` drops the `lang` field that the C API's
/// JSON carries, so the spike guesses from the script of the text. It cannot
/// tell Mandarin from Cantonese.
fn script_lang(text: &str) -> &'static str {
    let (mut han, mut kana, mut hangul, mut latin) = (0, 0, 0, 0);
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

/// Removes `<|…|>` tags. With sherpa-onnx 2.0.1 they never reach `text` or
/// `tokens` (the runtime strips them), so this only guards against a change there.
fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("<|") {
        out.push_str(&rest[..i]);
        rest = rest[i..].find("|>").map_or("", |j| &rest[i + j + 2..]);
    }
    out.push_str(rest);
    out
}

fn is_punct(tok: &str) -> bool {
    tok.trim_end().chars().last().is_some_and(|c| "，。！？；、,.!?;".contains(c))
}

fn join_tokens(tokens: &[String]) -> String {
    let s: String = tokens.iter().map(|t| t.replace('\u{2581}', " ")).collect();
    strip_tags(&s).split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One cue per VAD segment. A segment longer than `max_cue` is cut after the last
/// punctuation token inside the window, or before the token that crosses it when
/// there is none. Cue ends are clamped so no cue exceeds `max_cue`.
fn segment_cues(seg_start: f32, seg_dur: f32, r: &OfflineRecognizerResult, max_cue: f32) -> Vec<Cue> {
    let times = r.timestamps.as_deref().filter(|t| t.len() == r.tokens.len() && !t.is_empty());
    let Some(times) = times.filter(|_| seg_dur > max_cue) else {
        return vec![Cue {
            start: seg_start,
            end: seg_start + seg_dur,
            text: strip_tags(&r.text).trim().to_string(),
        }];
    };

    let mut cues = Vec::new();
    let mut first = 0usize;
    let mut cue_start = 0.0f32;
    let mut last_punct: Option<usize> = None;
    let push = |cues: &mut Vec<Cue>, from: usize, to: usize, start: f32, end: f32| {
        cues.push(Cue {
            start: seg_start + start,
            end: seg_start + end.min(start + max_cue),
            text: join_tokens(&r.tokens[from..to]),
        });
    };
    for i in 0..r.tokens.len() {
        if times[i] - cue_start > max_cue && i > first {
            let cut = match last_punct {
                Some(p) if p >= first => p + 1,
                _ => (i - 1).max(first + 1),
            };
            push(&mut cues, first, cut, cue_start, times[cut]);
            first = cut;
            cue_start = times[cut];
            last_punct = None;
        }
        if is_punct(&r.tokens[i]) {
            last_punct = Some(i);
        }
    }
    push(&mut cues, first, r.tokens.len(), cue_start, seg_dur);
    cues
}

fn ts(sec: f32) -> String {
    let ms = (sec.max(0.0) * 1000.0).round() as u64;
    format!("{:02}:{:02}:{:02}.{:03}", ms / 3_600_000, ms / 60_000 % 60, ms / 1000 % 60, ms % 1000)
}
