//! 量插件的引擎：一段 WAV 进，`asr.vtt` 出，往 stderr 打一行 JSON 的计时。和插件用同一套
//! VAD、识别器、cue 切分，只是不走 JSON-RPC。`bench/run.sh` 调它。
//!
//! bench <in.wav> <out.vtt> --models <${MODELS}> --vad <silero_vad_v5.onnx>
//!       [--threads 2] [--language auto] [--vad-threshold 0.5] [--max-cue 15]

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context as _, bail};
use serde_json::{Map, Value, json};
use usuyo_plugin_asr_sherpa::cues::{guess_language, is_no_speech, write_vtt};
use usuyo_plugin_asr_sherpa::engine::{
    ModelFiles, RecognizerKey, VadModel, load_recognizer, transcribe,
};
use usuyo_plugin_asr_sherpa::options::Options;
use usuyo_plugin_asr_sherpa::wav::Pcm16k;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let mut positional = Vec::new();
    let (mut models, mut vad) = (None, None);
    let mut options = Map::new();
    while let Some(a) = args.next() {
        let mut value = || args.next().with_context(|| format!("{a} needs a value"));
        match a.as_str() {
            "--models" => models = Some(PathBuf::from(value()?)),
            "--vad" => vad = Some(PathBuf::from(value()?)),
            "--threads" => {
                options.insert("threads".into(), json!(value()?.parse::<u64>()?));
            }
            "--language" => {
                options.insert("language".into(), Value::String(value()?));
            }
            "--vad-threshold" => {
                options.insert("vadThreshold".into(), json!(value()?.parse::<f64>()?));
            }
            "--max-cue" => {
                options.insert("maxCueSec".into(), json!(value()?.parse::<f64>()?));
            }
            _ => positional.push(a),
        }
    }
    let [input, output] = <[String; 2]>::try_from(positional).map_err(|_| {
        anyhow::anyhow!("usage: bench <in.wav> <out.vtt> --models <dir> --vad <file>")
    })?;
    let models = models.context("--models")?;
    let vad = VadModel::verify(&vad.context("--vad")?)?;
    let options = Options::from_map(&options)?;

    let t0 = Instant::now();
    let mut audio = Pcm16k::open(Path::new(&input))?;
    let key = RecognizerKey::new(ModelFiles::locate(&models)?, &options);
    let recognizer = load_recognizer(&key)?;
    let load_sec = t0.elapsed().as_secs_f64();

    let t_run = Instant::now();
    let t = transcribe(&recognizer, &vad, &mut audio, &options, |_| true)?;
    let run_sec = t_run.elapsed().as_secs_f64();

    let empty = is_no_speech(t.speech_sec, t.duration_sec);
    let lang = options
        .language
        .fixed()
        .unwrap_or_else(|| guess_language(&t.text));
    if empty {
        println!(
            "empty: no-speech speechSec={:.3} durationSec={:.3}",
            t.speech_sec, t.duration_sec
        );
    } else {
        std::fs::write(&output, write_vtt(lang, &t.cues))
            .with_context(|| format!("write {output}"))?;
    }
    if t.duration_sec == 0.0 {
        bail!("{input} has no samples");
    }
    let longest = t.cues.iter().map(|c| c.end - c.start).fold(0.0, f64::max);
    let total = t0.elapsed().as_secs_f64();
    eprintln!(
        "{}",
        json!({
            "durationSec": t.duration_sec,
            "speechSec": t.speech_sec,
            "segments": t.segments,
            "cues": if empty { 0 } else { t.cues.iter().filter(|c| !c.text.is_empty()).count() },
            "longestCueSec": longest,
            "lang": if empty { "" } else { lang },
            "threads": options.threads,
            "loadSec": load_sec,
            "runSec": run_sec,
            "totalSec": total,
            "rtfRun": run_sec / t.duration_sec,
            "rtfTotal": total / t.duration_sec,
        })
    );
    Ok(())
}
