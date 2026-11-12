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

# 查文档：给的每个文件的锚点和引用，再查全仓库的相对链接（调研 05 里有 10 条已知的假阳性）。
# 例：just docs docs/BUILD.md docs/SPEC.md
docs +files:
    #!/usr/bin/env bash
    set -euo pipefail
    for f in {{files}}; do python3 -I tools/check_md.py "$f"; done
    python3 -I tools/resolve_links.py .

# 开发时、只在自己的机器上：把旁边 `../usuyo-fetch/` 下的平台获取插件和本仓库 `plugins/` 下的三个转换插件
# 编成 release，照安装的样子放进按机器目录的 `plugins/<owner>/<repo>/<version>/`（认 `USUYO_HOME`，
# 没设就是系统的位置）。平台插件不进本仓库、不进安装包（CLAUDE.md 原则 1）；装好以后重启守护进程才会找到。
# 转换插件的资产：media 的 ffmpeg、ffprobe 是指向 Homebrew 那份的链接（没装 Homebrew 的 ffmpeg 就跳过 media），
# asr-sherpa 带 VAD 文件，ocr-vision 只在 macOS 上编。第一次编要联网取依赖，之后离线也能编。
dev-plugins fetch="../usuyo-fetch":
    #!/usr/bin/env bash
    set -euo pipefail
    case "$(uname -s)" in
      Darwin) default="$HOME/Library/Application Support/usuyo" ;;
      *) default="${XDG_STATE_HOME:-$HOME/.local/state}/usuyo" ;;
    esac
    root="${USUYO_HOME:-$default}/plugins"
    field() { sed -n "s/^$1:[[:space:]]*//p" "$2" | head -1; }
    # 照安装的样子放一个转换插件：清单和可执行文件；回目标目录
    place() {
      local src=$1 bin=$2
      local manifest="$src/usuyo-plugin.yaml"
      local dest="$root/$(field id "$manifest")/$(field version "$manifest")"
      mkdir -p "$dest"
      cp "$manifest" "$dest/usuyo-plugin.yaml"
      cp "$bin" "$dest/$(basename "$bin")"
      echo "$dest"
    }
    ffmpeg="" ffprobe=""
    for dir in /opt/homebrew/bin /usr/local/bin; do
      if [ -x "$dir/ffmpeg" ] && [ -x "$dir/ffprobe" ]; then ffmpeg="$dir/ffmpeg" ffprobe="$dir/ffprobe"; break; fi
    done
    if [ -n "$ffmpeg" ]; then
      cargo build --release --locked --manifest-path plugins/media/Cargo.toml
      dest=$(place plugins/media plugins/media/target/release/usuyo-plugin-media)
      ln -sfn "$ffmpeg" "$dest/ffmpeg"
      ln -sfn "$ffprobe" "$dest/ffprobe"
      echo "装好了 builtin/media → $dest（ffmpeg 链到 $ffmpeg）"
    else
      echo "跳过 builtin/media：没找到 Homebrew 的 ffmpeg 和 ffprobe（brew install ffmpeg）；合起来能播、口播、关键帧都要它"
    fi
    cargo build --release --locked --manifest-path plugins/asr-sherpa/Cargo.toml
    dest=$(place plugins/asr-sherpa plugins/asr-sherpa/target/release/usuyo-plugin-asr-sherpa)
    cp plugins/asr-sherpa/silero_vad_v5.onnx "$dest/silero_vad_v5.onnx"
    echo "装好了 builtin/asr-sherpa → $dest"
    if [ "$(uname -s)" = Darwin ]; then
      swift build -c release --package-path plugins/ocr-vision
      dest=$(place plugins/ocr-vision plugins/ocr-vision/.build/release/usuyo-plugin-ocr-vision)
      echo "装好了 builtin/ocr-vision → $dest"
    else
      echo "跳过 builtin/ocr-vision：只在 macOS 上有"
    fi
    for name in xiaohongshu wechat-channels bilibili x; do
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

# 开发和测试用的获取插件 fixture/local（feature 003 §12）：`https://fixture.test/<名字>` 取到的是
# crates/usuyo-core/testdata/transform/ 里合成的样例（dash、note、silent、quiet）。认 `USUYO_HOME`；
# 只在开发机上装，不进安装包。
dev-fixture-plugin:
    #!/usr/bin/env bash
    set -euo pipefail
    case "$(uname -s)" in
      Darwin) default="$HOME/Library/Application Support/usuyo" ;;
      *) default="${XDG_STATE_HOME:-$HOME/.local/state}/usuyo" ;;
    esac
    src=crates/usuyo-core/testdata/transform
    version=$(sed -n 's/^version:[[:space:]]*//p' "$src/fixture/usuyo-plugin.yaml" | head -1)
    dest="${USUYO_HOME:-$default}/plugins/fixture/local/$version"
    mkdir -p "$dest"
    cp "$src/fixture/usuyo-plugin.yaml" "$src/fixture/plugin.sh" "$dest/"
    chmod 755 "$dest/plugin.sh"
    fx="$dest/fixtures"
    if [ -d "$fx" ]; then trash "$fx"; fi
    mkdir -p "$fx/dash/media" "$fx/note/images" "$fx/silent/media" "$fx/quiet/media"
    cp "$src/video-only.mp4" "$fx/dash/media/video.mp4"
    cp "$src/speech.m4a" "$fx/dash/media/audio.m4a"
    cp "$src/subtitle.zh.vtt" "$fx/dash/media/subtitle.zh.vtt"
    cp "$src/card.png" "$fx/note/images/001.png"
    cp "$src/card-copy.png" "$fx/note/images/002.png"
    cp "$src/silent.mp4" "$fx/silent/media/video.mp4"
    cp "$src/quiet.mp4" "$fx/quiet/media/video.mp4"
    for name in dash note silent quiet; do cp "$src/fixture/$name.json" "$fx/$name/fetch.json"; done
    echo "装好了 fixture/local $version → $dest"
