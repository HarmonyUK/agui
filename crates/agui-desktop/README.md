# AGUI Desktop - Universal Agent GUI

A high-performance, native Rust desktop application framework for the Universal Agent GUI (UAG) specification. Built with Iced for a reliable, type-safe UI and optimized for seamless agent orchestration workflows.

## Quick Start

### Build

```bash
# Debug build
make build

# Release build (optimized)
make release
```

### Run

```bash
# Run with default settings
make run

# Run with development settings (debug logging + hot reload)
make dev

# Run with custom orchestrator URL
AGUI_ORCHESTRATOR_URL=ws://your-server:8765 make run
```

### Test

```bash
# Run all unit tests
make test

# Run all tests with verbose output
make test-all

# Run specific test
cargo test app::tests::test_new_app
```

## Development

### Code Quality

```bash
# Format code with rustfmt
make fmt

# Check formatting without making changes
make fmt-check

# Run clippy linter
make clippy

# Run all quality checks
make check
```

### Documentation

```bash
# Build and open API documentation
make docs
```

## Configuration

Configure AGUI via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `AGUI_LOG_LEVEL` | `info` | Logging level (debug, info, warn, error) |
| `AGUI_ENABLE_METRICS` | `true` | Enable Prometheus metrics collection |
| `AGUI_METRICS_PORT` | `9090` | Port for metrics exporter |
| `AGUI_HOT_RELOAD` | `true` (debug), `false` (release) | Enable file watching for development |
| `AGUI_PROJECT_ROOT` | Current directory | Root path to watch for hot reload |
| `AGUI_ORCHESTRATOR_URL` | `ws://localhost:8765` | WebSocket URL for orchestrator connection |

Example:

```bash
AGUI_LOG_LEVEL=debug AGUI_HOT_RELOAD=true cargo run
```

## Project Structure

```
crates/agui-desktop/
├── src/
│   ├── main.rs              # Application entry point and UI setup
│   ├── lib.rs               # Library exports and version info
│   ├── app.rs               # Core application state and message handling
│   ├── config.rs            # Configuration management
│   ├── logging.rs           # Structured logging setup
│   ├── metrics.rs           # Metrics collection and export
│   └── hot_reload.rs        # File watching for development
├── tests/
│   └── integration_tests.rs # Integration tests
├── Cargo.toml               # Crate dependencies and metadata
└── README.md               # This file
```

## Architecture

### Three-Column Layout

AGUI implements the Universal Agent GUI specification with a flexible three-column layout:

- **Zone A (Context Rail)**: Left sidebar for resources, agent roster, and session info
- **Zone B (Stream)**: Center timeline for agent messages, tool calls, and reasoning
- **Zone C (Stage)**: Right workspace for artifact editing and rendering

### Modules

#### `app` - Application State
- `AguiApp`: Core application state machine
- `Message`: Enum for handling state transitions
- Tests: Unit tests for state management

#### `config` - Configuration
- `AppConfig`: Application configuration from environment
- Default values and environment variable support

#### `logging` - Structured Logging
- Tracing-based logging with JSON and human-readable output
- Startup/shutdown logging helpers

#### `metrics` - Performance Metrics
- Prometheus exporter integration
- Frame timing, event processing, connection metrics
- Extensible metric recording functions

#### `hot_reload` - Development Support
- File watching using `notify` crate
- Smart filtering of watched files (ignores target/, .git/, etc.)
- `HotReloadEvent` for watcher events

## Dev Tooling

### Rust Formatting (rustfmt)
- Configuration in `rustfmt.toml`
- Max line width: 100 characters
- Consistent with Rust idioms

### Linting (Clippy)
- Configuration in `.clippy.toml`
- Enforces warnings as errors in CI
- Type complexity, cognitive complexity, and documentation checks

### Editor Config
- `.editorconfig` file for cross-editor consistency
- 4-space indentation for Rust, 2-space for TOML/YAML

### Testing
- Unit tests in module files (src/app.rs, src/hot_reload.rs)
- Integration tests in tests/ directory
- Fast test execution with optimized debug profiles

## Makefile Commands

| Command | Description |
|---------|-------------|
| `make help` | Show all available commands |
| `make build` | Build debug binary |
| `make release` | Build optimized release binary |
| `make dev` | Build with debug logging and hot reload |
| `make run` | Run the application |
| `make test` | Run unit tests |
| `make test-all` | Run all tests verbosely |
| `make fmt` | Format code with rustfmt |
| `make fmt-check` | Check formatting without changes |
| `make clippy` | Run clippy linter |
| `make clippy-fix` | Auto-fix clippy warnings |
| `make check` | Run fmt-check and clippy |
| `make clean` | Remove build artifacts |
| `make docs` | Build and open documentation |
| `make watch` | Watch and rebuild on changes |

## Dependencies

Key dependencies include:

- **iced** (0.12): Cross-platform GUI framework
- **tokio** (1.35): Async runtime
- **tracing** (0.1): Structured logging
- **metrics** (0.21): Performance metrics
- **notify** (6.1): File watching
- **anyhow**: Error handling

## Performance Considerations

- Dev profile: Optimized for compile speed (`opt-level = 1`)
- Release profile: Optimized for runtime performance (`opt-level = 3`, LTO enabled)
- Hot reload: Disabled in release builds by default
- Metrics: Can be disabled via `AGUI_ENABLE_METRICS=false`

## Future Enhancements

### Phase 2: Core UI Components
- Zone B (Stream) implementation with event rendering
- Zone C (Stage) with code/text editor integration
- Declarative UI schema mapping

### Phase 3: Orchestrator Integration
- WebSocket client for orchestrator connection
- Event deserialization and state synchronization
- Connection lifecycle management

### Phase 4: Advanced Features
- Virtualized list rendering for large message streams
- Diff viewer for artifact changes
- Advanced search and filtering
- Plugin system for custom renderers

## Contributing

When contributing to agui-desktop:

1. Ensure all tests pass: `make test-all`
2. Check code quality: `make check`
3. Format code: `make fmt`
4. Add tests for new functionality
5. Update documentation as needed

## License

MIT - See root LICENSE file

## Related Documentation

- [UAG Rust Specification](../../docs/architecture/uag_rust_v1.1.md)
- [AGUI Implementation Plan](../../docs/architecture/implementation_plan.md)
- [AGUI gpui Goals](../../docs/design/agui-gpui-goals.md)
