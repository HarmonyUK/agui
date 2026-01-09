---
id: doc-qw2
title: 'AGUI: Desktop Scaffold Implementation'
type: guide
scope: internal
created_at: '2026-01-09T01:16:50.009092Z'
updated_at: '2026-01-09T01:16:50.009092Z'
---

# AGUI: Desktop Scaffold Implementation

## Overview

This guide documents the AGUI Desktop project scaffold setup (agui-3q9), which establishes the foundational Rust application structure, development tooling, and testing infrastructure for the Universal Agent GUI desktop client.

The scaffold uses **Iced 0.12** as the primary GUI framework (native Rust, cross-platform) and provides:
- Structured workspace with agui-desktop crate
- Complete dev tooling (fmt, clippy, tests)
- Logging/metrics infrastructure
- Hot-reload support for development
- Comprehensive test coverage
- Release-optimized binary

## Project Structure

```
agui/
├── Cargo.toml (workspace)
├── Makefile (14 development commands)
├── rustfmt.toml (code formatting)
├── .clippy.toml (linting rules)
├── .editorconfig (editor consistency)
└── crates/agui-desktop/
    ├── src/
    │   ├── lib.rs - Library exports & version
    │   ├── main.rs - Iced GUI entry point (3-column layout)
    │   ├── app.rs - Application state machine
    │   ├── config.rs - Environment configuration
    │   ├── logging.rs - Structured logging setup
    │   ├── metrics.rs - Prometheus metrics
    │   └── hot_reload.rs - File watcher for dev
    ├── tests/
    │   └── integration_tests.rs - Integration test suite
    ├── Cargo.toml
    └── README.md
```

## Core Components

### Application State (app.rs)
- `AguiApp` struct: Main state machine
- `Message` enum: State transitions (Connected, Disconnected, Error, Frame)
- Frame counting and health checks
- Unit tests for state management

### Configuration (config.rs)
- Environment variable support (AGUI_LOG_LEVEL, AGUI_HOT_RELOAD, etc.)
- Default sensible values
- Per-environment (dev vs. release) overrides

### Logging (logging.rs)
- Tracing-based structured logging
- Support for debug/info/warn/error levels
- Startup/shutdown logging helpers
- Human-readable and machine-parseable output

### Metrics (metrics.rs)
- Prometheus exporter integration (ready for future implementation)
- Hooks for frame timing, event processing, connections
- Memory usage tracking
- Extensible metric recording API

### Hot Reload (hot_reload.rs)
- File watcher using `notify` crate
- Smart filtering (ignores target/, .git/, etc.)
- Development-only feature (disabled in release)
- Enables rapid development iteration

### GUI (main.rs)
- Iced Sandbox application
- 3-column layout matching UAG specification:
  - **Zone A (Context Rail)**: 300px sidebar
  - **Zone B (Stream)**: Flexible center area
  - **Zone C (Stage)**: 400px right workspace
- Status bar with connection state and frame counter

## Development Workflow

### Building
```bash
# Debug build (fast)
make build

# Release build (optimized)
make release

# Dev build with logging + hot reload
make dev
```

### Testing
```bash
# Unit tests only
make test

# All tests with verbose output
make test-all

# Run specific test
cargo test app::tests::test_new_app
```

### Code Quality
```bash
# Format code
make fmt

# Check formatting
make fmt-check

# Run clippy linter
make clippy

# Auto-fix clippy warnings
make clippy-fix

# Run all quality checks
make check
```

### Running
```bash
# Run with defaults
make run

# Run with custom settings
AGUI_LOG_LEVEL=debug AGUI_HOT_RELOAD=true make run
```

## Configuration

Environment variables control behavior:

| Variable | Default | Description |
|----------|---------|-------------|
| AGUI_LOG_LEVEL | info | debug, info, warn, error |
| AGUI_ENABLE_METRICS | true | Enable Prometheus metrics |
| AGUI_METRICS_PORT | 9090 | Metrics exporter port |
| AGUI_HOT_RELOAD | true (debug) | Enable file watching |
| AGUI_PROJECT_ROOT | . | Root for hot reload watching |
| AGUI_ORCHESTRATOR_URL | ws://localhost:8765 | Orchestrator WebSocket |

## Test Coverage

**Unit Tests (5):**
- app.rs: App creation, message handling, error states, frame counting
- hot_reload.rs: File filtering logic

**Integration Tests (4):**
- App instantiation
- Configuration defaults
- Version display
- Connection status

**Result:** 9/9 tests passing ✅

## Dependencies

Key workspace dependencies:
- **iced** (0.12) - GUI framework
- **tokio** (1.35) - Async runtime
- **tracing** (0.1) - Structured logging
- **metrics** (0.21) - Metrics collection
- **notify** (6.1) - File watching
- **anyhow/thiserror** - Error handling

## Build Artifacts

- **Debug binary:** `target/debug/agui`
- **Release binary:** `target/release/agui` (14M optimized)
- **Documentation:** `cargo doc --no-deps --open`

## Next Phases

This scaffold unblocks:
1. **agui-263** - 3-column layout skeleton
2. **agui-n59** - WebSocket protocol/schema
3. **agui-xwe** - Stream virtualization
4. **agui-5lx** - Stage workspace features

## Quality Metrics

✅ Code formatting: Pass (rustfmt)
✅ Linting: Pass (clippy, no warnings)
✅ Unit tests: 5/5 passing
✅ Integration tests: 4/4 passing
✅ Debug build: Success
✅ Release build: Success (5m 23s)
✅ Documentation: Complete

## Related

- docs/architecture/uag_rust_v1.1.md - Rust desktop specification
- docs/architecture/implementation_plan.md - Implementation phases
- docs/design/agui-gpui-goals.md - AGUI goals and acceptance criteria
- crates/agui-desktop/README.md - Crate-specific documentation
