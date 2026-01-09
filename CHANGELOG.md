# Changelog

All notable changes to AGUI Desktop will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-01-09

### Added
- **Performance & Resilience Module** (`resilience/`)
  - `PerformanceMetrics` with FPS tracking and rolling window averages
  - `PerformanceMode` adaptive rendering (Normal/Reduced/Minimal)
  - `ConnectionState` with reconnection strategy and exponential backoff
  - `UpdateBatcher` for reducing render frequency
  - `ErrorManager` with severity levels and auto-dismiss
  - `SessionCompactor` for long-running session memory management
  - UI components: connection status, offline banner, error toasts, performance panel

- **Stage Module** (`stage/`)
  - Artifact workspace (Zone C) with tabbed interface
  - Code/text view with line numbers
  - Syntax highlighting for 8 languages (Rust, Python, JS, Go, JSON, YAML, TOML, Bash)
  - Unified and side-by-side diff views
  - LCS-based diff algorithm
  - LRU cache for highlighted lines
  - STATE_DELTA hydration from WebSocket events
  - Read-only and editable modes

### Changed
- Upgraded metrics module from stubs to full implementation

### Dependencies
- Added `once_cell = "1.19"` for lazy static initialization

## [0.2.0] - 2026-01-09

### Added
- **Stream Module** (`stream/`)
  - Virtual scrolling timeline for conversation history
  - User and agent message bubbles
  - Reasoning accordion with expand/collapse
  - Tool call cards with status indicators
  - Plan checklist with completion tracking
  - Approval gates with action buttons
  - Status update blocks

- **Declarative Renderer** (`renderer/`)
  - JSON-to-GPUI component rendering
  - Form state management
  - Dynamic UI from protocol events

## [0.1.0] - 2026-01-09

### Added
- Initial project structure
- **Core Infrastructure**
  - GPUI-based desktop application scaffold
  - 3-column layout (Context Rail, Stream, Stage)
  - WebSocket protocol handling
  - Hot reload support
  - Mock server for development
  - Logging and basic metrics stubs

- **Configuration**
  - `AppConfig` with window size, theme, connection settings
  - `.clippy.toml` and `rustfmt.toml` for code style

[Unreleased]: https://github.com/user/agui/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/user/agui/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/user/agui/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/user/agui/releases/tag/v0.1.0
