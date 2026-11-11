#!/bin/sh
# Regenerates the synthetic transform samples in this directory (macOS only:
# needs `say`, Swift with AppKit, and an ffmpeg with libx264 and aac).
# CI uses the committed files; run this only to change them.
set -eu

cd "$(dirname "$0")"
FFMPEG="${FFMPEG:-/opt/homebrew/bin/ffmpeg}"
ff() { "$FFMPEG" -hide_banner -loglevel error -y "$@"; }

tmp="$(mktemp -d)"
trap 'trash "$tmp" 2>/dev/null || true' EXIT

# Spoken Chinese, about 7.4 s; AAC 64k mono in M4A.
say -v "Reed (Chinese (China mainland))" -o "$tmp/speech.aiff" \
    "大家好，今天讲手冲咖啡的三段式注水。水温九十二度，粉水比一比十五。"
ff -i "$tmp/speech.aiff" -ac 1 -ar 16000 -c:a aac -b:a 64k -map_metadata -1 speech.m4a

# Picture only, 7.5 s; pairs with speech.m4a under the 1 s / 2 % rule.
ff -f lavfi -i testsrc2=size=320x180:rate=25 -t 7.5 -an \
    -c:v libx264 -preset veryslow -crf 35 -pix_fmt yuv420p -map_metadata -1 video-only.mp4

# Text card and a byte-identical copy.
swiftc -O card.swift -o "$tmp/card"
"$tmp/card" card.png
cp card.png card-copy.png

# The card as a still video with no audio track, 5 s.
ff -loop 1 -framerate 5 -i card.png -t 5 -an \
    -vf scale=540:720,format=yuv420p -c:v libx264 -tune stillimage -crf 30 -map_metadata -1 silent.mp4

# Picture plus a silent audio track, 5 s.
ff -f lavfi -i testsrc2=size=320x180:rate=25 -f lavfi -i anullsrc=r=16000:cl=mono -t 5 \
    -c:v libx264 -preset veryslow -crf 35 -pix_fmt yuv420p -c:a aac -b:a 32k -map_metadata -1 quiet.mp4
