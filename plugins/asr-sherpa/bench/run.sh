#!/bin/sh
# Usage: bench/run.sh <wav> <models-dir> <vad> <label> [reps] [threads...]
# <models-dir> is laid out like the host's ${MODELS} (see README); <vad> is silero_vad_v5.onnx.
# Runs examples/bench.rs; appends one line per run to bench/results.tsv:
# label  wav  threads  mode  rep  maxrss_bytes  json
set -eu
S="$(cd "$(dirname "$0")/.." && pwd)"
wav="$1"; models="$2"; vad="$3"; label="$4"; reps="${5:-3}"; shift 5 || shift $#
threads="${*:-1 2}"
cargo build --release --quiet --manifest-path "$S/Cargo.toml" --example bench
out="$S/target/bench"; mkdir -p "$out"
for t in $threads; do
  for mode in normal background; do
    for rep in $(seq 1 "$reps"); do
      if [ "$mode" = background ]; then pre="/usr/sbin/taskpolicy -b"; else pre=""; fi
      err="$(mktemp)"
      $pre /usr/bin/time -l "$S/target/release/examples/bench" "$wav" "$out/$label.vtt" \
        --models "$models" --vad "$vad" --threads "$t" >/dev/null 2>"$err"
      json="$(grep '^{' "$err")"
      rss="$(awk '/maximum resident set size/ {print $1}' "$err")"
      printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$label" "$(basename "$wav")" "$t" "$mode" "$rep" "$rss" "$json" >> "$S/bench/results.tsv"
      trash "$err"
    done
  done
done
