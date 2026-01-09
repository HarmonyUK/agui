---
id: doc-8hx
title: Stage Module Architecture
type: architecture
scope: internal
created_at: '2026-01-09T04:31:55.401250Z'
updated_at: '2026-01-09T04:31:55.401250Z'
---

# Stage Module Architecture

## Overview

The Stage module provides the artifact workspace (Zone C) in the AGUI 3-column layout. It renders code/text files with syntax highlighting, diff views for comparing versions, and supports both editable and read-only modes. The module integrates with the WebSocket protocol to hydrate state from `STATE_DELTA` events.

## Module Structure

```
stage/
├── mod.rs           # Module exports
├── types.rs         # Core data types (Artifact, DiffContent, ViewMode)
├── state.rs         # StageState - centralized state management
├── diff.rs          # LCS-based diff computation
├── syntax.rs        # Regex-based syntax highlighting (8 languages)
├── cache.rs         # LRU cache for highlighted lines
└── components/
    ├── mod.rs           # Component exports + color constants
    ├── artifact_view.rs # Main artifact rendering
    ├── text_view.rs     # Code/text view with line numbers
    ├── diff_view.rs     # Unified and side-by-side diff views
    └── tabs.rs          # Tab bar for multiple artifacts
```

## Components

### StageState (`state.rs`)

Central state manager for the Stage pane:
- Maintains list of open artifacts with tab order
- Tracks active artifact selection
- Manages view settings (view mode, font size, line numbers)
- Handles scroll position per artifact
- Caches syntax highlighters per language
- Hydrates from `STATE_DELTA`, `ARTIFACT_OPEN`, `ARTIFACT_UPDATE` events

### Types (`types.rs`)

Core data structures:
- **Artifact** - Represents an open file with id, title, content, language, dirty/read-only flags
- **ArtifactContent** - Enum for Text or Diff content
- **TextContent** - Text with optional previous content for diff generation
- **DiffContent** - Parsed diff with hunks and metadata
- **ViewMode** - Normal, Unified, SideBySide, InlineChanges
- **ContentType** - Code, Text, Markdown, JSON, Config, Unknown

### Diff Engine (`diff.rs`)

LCS (Longest Common Subsequence) based diff computation:
- `compute_unified_diff()` - Generates unified diff format
- `parse_hunks()` - Parses diff into structured hunks
- `diff_stats()` - Calculates additions/deletions/hunk counts
- Supports context lines around changes

### Syntax Highlighting (`syntax.rs`)

Regex-based highlighting for 8 languages:
- Rust, Python, JavaScript, Go, JSON, YAML, TOML, Bash
- Token types: Keyword, String, Number, Comment, Function, Type, Operator, etc.
- VS Code dark theme color palette
- Extensible `LanguageDefinition` structure

### Cache (`cache.rs`)

Performance optimization for large files:
- **LruCache<K,V>** - Generic LRU cache implementation
- **ArtifactCache** - Caches highlighted lines by (artifact_id, line_number, content_hash)
- **ChunkIterator** - Splits large content into chunks for streaming render

### UI Components

- **artifact_view.rs** - `render_stage_pane()` entry point, toolbar, preview
- **text_view.rs** - Line numbers gutter, syntax-highlighted content
- **diff_view.rs** - Unified and side-by-side diff rendering with stats header
- **tabs.rs** - Tab bar with dirty indicators, close buttons, active highlighting

## Design Decisions

### 1. LCS-based Diff Algorithm
Chose in-house LCS implementation over external crates for:
- Zero additional dependencies
- Control over output format
- Adequate performance for typical file sizes

### 2. Regex-based Syntax Highlighting
Selected regex over tree-sitter for initial implementation:
- Simpler integration
- No native dependencies
- Sufficient for basic highlighting
- Note: Consider tree-sitter upgrade for production accuracy

### 3. View Mode Cycling
Single `cycle_view_mode()` method cycles through modes in order:
- Normal → Unified → SideBySide → InlineChanges → Normal
- Simplifies UI (one button) while exposing all modes

### 4. STATE_DELTA Hydration
`apply_state_delta()` handles bulk state updates:
- Opens new artifacts from `artifacts_open`
- Updates existing artifacts from `artifacts_update`
- Preserves local state (scroll position, selection) during updates

## Data Flow

```
WebSocket Event → Protocol Parse → StageState.apply_*() → Component Render
     │                                    │
     │ ARTIFACT_OPEN                      │ open_artifact()
     │ ARTIFACT_UPDATE                    │ update_artifact()
     │ STATE_DELTA                        │ apply_state_delta()
```

## References

- Code: `crates/agui-desktop/src/stage/`
- Issue: agui-5lx
- Related: Stream module (`crates/agui-desktop/src/stream/`)
- Protocol: `crates/agui-desktop/src/protocol/`
