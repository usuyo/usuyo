#!/bin/sh
# 把内核 `usuyo` 和预装的网页插件编好，放到打包要的位置：
# binaries/usuyo-<目标三元组>（Tauri 的 externalBin）和 core-plugins/builtin/web/<版本>/（资源）。
# 用法：sh stage-core.sh [release|debug]；tauri.conf.json 的 beforeBuildCommand、beforeDevCommand 调它。
set -eu
profile="${1:-release}"
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
case "$profile" in
  release) cargo build --release --manifest-path "$repo/Cargo.toml" -p usuyo-cli -p usuyo-plugin-web ;;
  debug) cargo build --manifest-path "$repo/Cargo.toml" -p usuyo-cli -p usuyo-plugin-web ;;
  *) echo "usage: stage-core.sh [release|debug]" >&2; exit 2 ;;
esac
out="${CARGO_TARGET_DIR:-$repo/target}/$profile"
triple="$(rustc -vV | sed -n 's/^host: //p')"
mkdir -p "$here/binaries"
cp "$out/usuyo" "$here/binaries/usuyo-$triple"
version="$(sed -n 's/^version: *//p' "$repo/plugins/web/usuyo-plugin.yaml")"
dir="$here/core-plugins/builtin/web/$version"
mkdir -p "$dir"
cp "$repo/plugins/web/usuyo-plugin.yaml" "$dir/usuyo-plugin.yaml"
cp "$out/usuyo-plugin-web" "$dir/usuyo-plugin-web"
chmod 755 "$here/binaries/usuyo-$triple" "$dir/usuyo-plugin-web"
