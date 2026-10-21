GO ?= go
BIN := bin
GOLANGCI_LINT_VERSION := v2.15.0
GOLANGCI_LINT := $(BIN)/golangci-lint-$(GOLANGCI_LINT_VERSION)
VERSION ?= $(shell git describe --tags --always --dirty 2>/dev/null || echo dev)

.PHONY: init dev check lint test fmt build

## init: 安装依赖（Go 模块、lint 工具）。pnpm 与 sidecar 检查随 M1/M3 加入。
init: $(GOLANGCI_LINT)
	$(GO) mod download

# 工具装进仓库内 bin/，文件名带版本，换版本会自动重装。
$(GOLANGCI_LINT):
	GOBIN=$(CURDIR)/$(BIN) $(GO) install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@$(GOLANGCI_LINT_VERSION)
	mv $(BIN)/golangci-lint $(GOLANGCI_LINT)

## dev: 启动内核守护进程。桌面壳开发模式随 M3 加入。
dev:
	$(GO) run ./cmd/xiecun daemon

## check: lint + test
check: lint test

lint: $(GOLANGCI_LINT)
	$(GOLANGCI_LINT) run ./...

test:
	$(GO) test ./...

## fmt: 按 .golangci.yml 的 formatters 格式化
fmt: $(GOLANGCI_LINT)
	$(GOLANGCI_LINT) fmt ./...

## build: 产出 bin/xiecun（无 cgo，见 ADR-001）
build:
	CGO_ENABLED=0 $(GO) build -trimpath -ldflags "-s -w -X main.version=$(VERSION)" -o $(BIN)/xiecun ./cmd/xiecun
