#!/bin/sh
# Usage: bench/run.sh <wav> <model-dir> <label> [reps] [threads...]
# Appends one line per run to bench/results.tsv:
# label  wav  threads  mode  rep  maxrss_bytes  json
set -eu
S="$(cd "$(dirname "$0")/.." && pwd)"
wav="$1"; model="$2"; label="$3"; reps="${4:-3}"; shift 4 || shift $#
threads="${*:-0 2}"
for t in $threads; do
  for mode in normal background; do
    for rep in $(seq 1 "$reps"); do
      if [ "$mode" = background ]; then pre="/usr/sbin/taskpolicy -b"; else pre=""; fi
      err="$(mktemp)"
      $pre /usr/bin/time -l "$S/target/release/spike-asr" "$wav" "$S/out/$label.vtt" \
        --model "$model" --vad "$S/models/dl/silero_vad_v5.onnx" --threads "$t" >/dev/null 2>"$err"
      json="$(grep '^{' "$err")"
      rss="$(awk '/maximum resident set size/ {print $1}' "$err")"
      printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$label" "$(basename "$wav")" "$t" "$mode" "$rep" "$rss" "$json" >> "$S/bench/results.tsv"
      trash "$err"
    done
  done
done
