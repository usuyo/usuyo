#!/bin/bash
# Builds target/bench/long.wav (synthetic, ~11 min) and ref.txt from long.txt with seven zh_CN voices.
set -eu
cd "$(dirname "$0")"
FF=/opt/homebrew/bin/ffmpeg
voices=("Reed (Chinese (China mainland))" "Flo (Chinese (China mainland))" "Eddy (Chinese (China mainland))" "Sandy (Chinese (China mainland))" "Shelley (Chinese (China mainland))" "Grandpa (Chinese (China mainland))" "Rocko (Chinese (China mainland))")
w=../target/bench
mkdir -p $w/parts
: > ref.txt; : > $w/list.txt
$FF -hide_banner -loglevel error -y -f lavfi -i anullsrc=r=16000:cl=mono -t 0.8 -c:a pcm_s16le $w/parts/gap.wav
i=0
for pass in 0 1 2; do
  n=0
  while IFS= read -r line; do
    if [ $pass -eq 2 ] && [ $n -ge 5 ]; then break; fi
    v="${voices[$(( (i + pass*3) % 7 ))]}"
    say -v "$v" -o $w/parts/p$i.aiff "$line"
    $FF -hide_banner -loglevel error -y -i $w/parts/p$i.aiff -ac 1 -ar 16000 -c:a pcm_s16le $w/parts/p$i.wav
    printf "file 'parts/p%s.wav'\nfile 'parts/gap.wav'\n" $i >> $w/list.txt
    echo "$line" >> ref.txt
    i=$((i+1)); n=$((n+1))
  done < long.txt
done
$FF -hide_banner -loglevel error -y -f concat -safe 0 -i $w/list.txt -c:a pcm_s16le $w/long.wav
