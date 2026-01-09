---
id: doc-asc
title: AGUI Resilience Module Implementation
type: reference
scope: internal
created_at: '2026-01-09T04:42:07.509605Z'
updated_at: '2026-01-09T04:42:07.509605Z'
---

# AGUI Resilience Module Implementation

## Overview

The resilience module provides performance monitoring, connection management, error handling, and graceful degradation for AGUI. It implements the patterns defined in the Performance & Resilience Patterns document (doc-wej).

## Module Structure

```
src/
├── metrics.rs          # Performance metrics and FPS tracking
└── resilience/
    ├── mod.rs          # Module exports
    ├── state.rs        # State management (connection, batching, errors)
    └── components.rs   # UI components for status/error display
```

## API Reference

### Performance Metrics (`metrics.rs`)

**PerformanceMetrics** - Global metrics collector:
- `record_frame()` - Record frame and calculate timing
- `record_event_time(Duration)` - Record event processing time
- `fps()` - Get current FPS
- `performance_mode()` - Get adaptive rendering mode
- `snapshot()` - Get point-in-time metrics

**PerformanceMode** - Adaptive rendering states:
- `Normal` - Full quality (60+ FPS)
- `Reduced` - Degraded (30-60 FPS)
- `Minimal` - Simplified (<30 FPS)

### Connection State (`resilience/state.rs`)

**ConnectionState** - UI-friendly connection status:
- `Disconnected`, `Connecting`, `Connected`
- `Reconnecting { attempt, next_retry_secs }`
- `Failed`

**ReconnectStrategy** - Exponential backoff:
- `record_attempt()` - Record connection attempt
- `reset()` - Reset on successful connection
- `should_retry()` - Check if retry is allowed
- `next_delay()` - Get delay before next retry

### Update Batching

**UpdateBatcher<T>** - Reduces render frequency:
- `push(T)` - Add item to batch
- `should_flush()` - Check if batch should be processed
- `flush()` - Get and clear pending items

### Error Management

**ErrorManager** - Error collection and lifecycle:
- `add(AppError)` - Add new error
- `add_error(msg)` / `add_warning(msg)` - Convenience methods
- `dismiss(index)` - Dismiss by index
- `cleanup()` - Remove expired errors
- `errors()` - Get current errors

**ErrorSeverity**: `Info`, `Warning`, `Error`, `Critical`

### Session Compaction

**SessionCompactor** - Prevents unbounded growth:
- `needs_compaction(count)` - Check if compaction needed
- `items_to_remove(count)` - Calculate items to remove

## Configuration

### Metrics Defaults
- Rolling window: 60 samples (1 second at 60fps)
- FPS thresholds: 55+ Normal, 25+ Reduced, <25 Minimal

### Reconnection Defaults
- Base delay: 1 second
- Max delay: 60 seconds
- Unlimited retries

### Batching Defaults
- Max batch size: 10 items
- Min interval: 16ms (~60fps)

### Error Manager Defaults
- Max visible errors: 5
- Auto-dismiss: 10 seconds

### Session Compactor Defaults
- Max items: 2000
- Keep recent: 500

## Examples

### Metrics Collection

```rust
use agui_desktop::metrics::get_metrics;

// In render loop
let metrics = get_metrics();
metrics.record_frame();

// Get snapshot for display
let snapshot = metrics.snapshot();
println!("FPS: {}", snapshot.fps_string());

// Check performance mode
if !snapshot.performance_mode.animations_enabled() {
    // Skip animations
}
```

### Connection Management

```rust
use agui_desktop::resilience::{ConnectionState, ReconnectStrategy};

let mut strategy = ReconnectStrategy::new();

// On failure
strategy.record_attempt();
if strategy.should_retry() {
    let delay = strategy.next_delay();
    // Schedule retry
}

// On success
strategy.reset();
```

### Update Batching

```rust
use agui_desktop::resilience::UpdateBatcher;

let mut batcher: UpdateBatcher<Event> = UpdateBatcher::new(10, 16);

batcher.push(event);

if batcher.should_flush() {
    let batch = batcher.flush();
    // Process batch
}
```

### Error Display

```rust
use agui_desktop::resilience::{ErrorManager, AppError, ErrorSeverity};
use agui_desktop::resilience::components::render_error_overlay;

let mut errors = ErrorManager::new();
errors.add(AppError::new("Connection failed", ErrorSeverity::Error));

// In render
render_error_overlay(&errors);
```

## See Also

- doc-wej: AGUI: Performance & Resilience Patterns (requirements)
- doc-8hx: Stage Module Architecture (related module)
- Issue: agui-ver
