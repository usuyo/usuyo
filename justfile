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

# 内核守护进程
dev:
    cargo run -p usuyo-cli -- daemon

build:
    cargo build --release -p usuyo-cli
