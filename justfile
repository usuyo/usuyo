default: check

# 装前端依赖；Rust 依赖由 cargo 自己取
init:
    cd apps/desktop/web && pnpm install

fmt:
    cargo fmt --all

lint:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

check: lint test

# 内核守护进程；前端另开一个终端 `cd apps/desktop/web && pnpm dev`，经 Vite 的代理找它（代理在服务端带令牌）
dev:
    cargo run -p usuyo-cli -- daemon

# 桌面应用（macOS 出 .app，不签名）：apps/desktop/src-tauri/target/release/bundle/macos/usuyo.app
app:
    cd apps/desktop/src-tauri && cargo tauri build --bundles app

build:
    cargo build --release -p usuyo-cli

# 开发时、只在自己的机器上：把旁边 `../usuyo-fetch/` 下三个平台获取插件编成 release，照安装的样子
# 放进按机器目录的 `plugins/<owner>/<repo>/<version>/`（认 `USUYO_HOME`，没设就是系统的位置）。
# 平台插件不进本仓库、不进安装包（CLAUDE.md 原则 1）；装好以后重启守护进程才会找到。
dev-plugins fetch="../usuyo-fetch":
    #!/usr/bin/env bash
    set -euo pipefail
    case "$(uname -s)" in
      Darwin) default="$HOME/Library/Application Support/usuyo" ;;
      *) default="${XDG_STATE_HOME:-$HOME/.local/state}/usuyo" ;;
    esac
    root="${USUYO_HOME:-$default}/plugins"
    for name in xiaohongshu wechat-channels bilibili; do
      src="{{fetch}}/$name"
      manifest="$src/usuyo-plugin.yaml"
      id=$(sed -n 's/^id:[[:space:]]*//p' "$manifest" | head -1)
      version=$(sed -n 's/^version:[[:space:]]*//p' "$manifest" | head -1)
      bin="usuyo-fetch-$name"
      cargo build --release --locked --offline --manifest-path "$src/Cargo.toml" --bin "$bin"
      dest="$root/$id/$version"
      mkdir -p "$dest"
      cp "$manifest" "$dest/usuyo-plugin.yaml"
      cp "$src/target/release/$bin" "$dest/$bin"
      echo "装好了 $id $version → $dest"
    done
