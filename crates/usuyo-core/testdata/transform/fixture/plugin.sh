#!/bin/sh
# fixture/local：canonicalize 认 fixture.test/<名字>；fetch 把 fixtures/<名字>/ 拷进输出目录，
# 打印它旁边的 fetch.json。参数照协议 §3：`canonicalize <url>`、`fetch <url> --out <dir> --method sample`、`health`。
set -eu
here=$(dirname "$0")
if [ "$1" = health ]; then
  echo '{"ok": true, "ready": true, "protocol": 1, "version": "0.1.0"}'
  exit 0
fi
name=${2##*/}
name=${name%%\?*}
case "$name" in
  dash|note|silent|quiet) ;;
  *)
    if [ "$1" = canonicalize ]; then echo '{"ok": true, "match": false}'; exit 0; fi
    echo '{"ok": false, "error": {"code": "not-found", "message": "no such fixture"}}'
    exit 1
    ;;
esac
case "$1" in
  canonicalize)
    printf '{"ok": true, "match": true, "identity": {"platform": "fixture", "contentId": "%s", "url": "https://fixture.test/%s"}}\n' "$name" "$name"
    ;;
  fetch)
    out=$4
    src="$here/fixtures/$name"
    for f in $(cd "$src" && find . -type f ! -name fetch.json); do
      mkdir -p "$out/$(dirname "$f")"
      cp "$src/$f" "$out/$f"
    done
    cat "$src/fetch.json"
    ;;
  *)
    echo '{"ok": false, "error": {"code": "unsupported-input", "message": "only canonicalize and fetch"}}'
    exit 1
    ;;
esac
