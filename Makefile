.PHONY: help build release test test-all fmt fmt-check clippy clippy-fix clean check dev run docs

help:
	@echo "AGUI Desktop Development Commands"
	@echo ""
	@echo "Building:"
	@echo "  make build         - Build debug binary"
	@echo "  make release       - Build optimized release binary"
	@echo "  make dev           - Build with dev dependencies and run"
	@echo "  make run           - Run the application"
	@echo ""
	@echo "Testing & Validation:"
	@echo "  make test          - Run unit tests"
	@echo "  make test-all      - Run all tests with verbose output"
	@echo "  make check         - Run clippy and format checks"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt           - Format code with rustfmt"
	@echo "  make fmt-check     - Check if code is formatted"
	@echo "  make clippy        - Run clippy linter"
	@echo "  make clippy-fix    - Fix clippy warnings"
	@echo ""
	@echo "Utilities:"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make docs          - Build documentation"

build:
	cargo build

release:
	cargo build --release

test:
	cargo test --lib

test-all:
	cargo test --all -- --nocapture

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

clippy-fix:
	cargo clippy --fix --allow-dirty --allow-staged

check: fmt-check clippy
	@echo "✓ Code quality checks passed"

clean:
	cargo clean

dev:
	AGUI_LOG_LEVEL=debug AGUI_HOT_RELOAD=true cargo build

run:
	cargo run --bin agui

docs:
	cargo doc --no-deps --open

# Development command that enables hot reload
watch:
	cargo watch -x build -x clippy -x test
