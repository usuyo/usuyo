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
