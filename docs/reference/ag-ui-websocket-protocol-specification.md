---
id: doc-cu3
title: AG-UI WebSocket Protocol Specification
type: reference
scope: internal
created_at: '2026-01-09T01:23:31.774953Z'
updated_at: '2026-01-09T01:23:31.774953Z'
---

# AG-UI WebSocket Protocol Specification

## Overview

The AG-UI WebSocket protocol defines a JSON-based event contract for real-time communication between the AGUI client and orchestrator server. This specification covers event types, versioning, error handling, and protocol conventions.

**Current Protocol Version:** `0.1.0`

The protocol version is included in every event envelope to support future evolution while maintaining backward compatibility.

## Event Envelope Structure

All protocol messages follow this envelope structure:

```json
{
  "version": "0.1.0",
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-01-09T12:00:00.000Z",
  "event": { "type": "TEXT_MESSAGE", "..." }
}
```

Fields:
- **version**: Protocol version (semantic versioning)
- **id**: Unique event identifier (UUID v4)
- **timestamp**: RFC 3339 formatted timestamp (ISO 8601)
- **event**: The actual event payload (see Event Types section)

## Event Types

### Messaging Events

#### TEXT_MESSAGE
User or agent sends a text message. Includes optional metadata for sender info.

```json
{
  "type": "TEXT_MESSAGE",
  "sender": "user_123",
  "content": "What is the status?",
  "metadata": { "author_name": "John" }
}
```

### Tool Execution Events

#### TOOL_CALL_REQUEST
Agent requests execution of a tool/function with parameters.

```json
{
  "type": "TOOL_CALL_REQUEST",
  "id": "tc_abc123",
  "tool_name": "grep",
  "parameters": { "pattern": "error", "file": "*.log" },
  "agent_id": "agent_main"
}
```

#### TOOL_CALL_STATUS
Status update during execution (progress, current state).

```json
{
  "type": "TOOL_CALL_STATUS",
  "id": "tc_abc123",
  "status": "RUNNING",
  "progress": 75,
  "message": "Processing files..."
}
```

**Status Values:** `PENDING`, `RUNNING`, `COMPLETED`, `FAILED`, `CANCELLED`

#### TOOL_CALL_RESULT
Result of a completed tool call.

```json
{
  "type": "TOOL_CALL_RESULT",
  "id": "tc_abc123",
  "result": { "matches": ["error: connection", "error: timeout"] },
  "error": null
}
```

### State Management Events

#### STATE_DELTA
Incremental state change notification from server.

```json
{
  "type": "STATE_DELTA",
  "path": "user.preferences.theme",
  "old_value": "light",
  "new_value": "dark"
}
```

### UI Rendering Events

#### RENDER_REQUEST
Request client to render or update a UI component.

```json
{
  "type": "RENDER_REQUEST",
  "component_id": "form_settings",
  "schema": { "type": "form", "fields": [...] },
  "props": { "disabled": false }
}
```

#### PLAN_CARD
Display a plan or reasoning card (often chain-of-thought).

```json
{
  "type": "PLAN_CARD",
  "id": "plan_001",
  "title": "Analysis Plan",
  "content": "## Steps\n1. Analyze error\n2. Find root cause",
  "status": "ACTIVE"
}
```

**Status Values:** `ACTIVE`, `COMPLETED`, `FAILED`, `CANCELLED`

### Resource Management Events

#### RESOURCE_TREE
Update the file/resource tree view (hierarchical file listing).

```json
{
  "type": "RESOURCE_TREE",
  "root": {
    "id": "root_1",
    "name": "Project",
    "node_type": "folder",
    "children": [
      { "id": "f1", "name": "src", "node_type": "folder", "children": [] }
    ]
  }
}
```

#### AGENT_ROSTER
List of connected agents and their status.

```json
{
  "type": "AGENT_ROSTER",
  "agents": [
    {
      "id": "agent_main",
      "name": "Main Agent",
      "status": "BUSY",
      "metadata": { "version": "1.0" }
    }
  ]
}
```

**Status Values:** `ONLINE`, `BUSY`, `IDLE`, `OFFLINE`

### Artifact Events

#### ARTIFACT_OPEN
Open a file or document in the editor.

```json
{
  "type": "ARTIFACT_OPEN",
  "id": "art_123",
  "title": "main.rs",
  "content": "fn main() { println!(\"hello\"); }",
  "content_type": "code",
  "read_only": false,
  "language": "rust"
}
```

#### ARTIFACT_UPDATE
Update the content of an open artifact.

```json
{
  "type": "ARTIFACT_UPDATE",
  "id": "art_123",
  "content": "fn main() { println!(\"updated\"); }",
  "change_type": "full_replace"
}
```

**Change Types:** `full_replace` (entire content), `partial` (partial changes), `diff` (diff format)

### User Interaction Events

#### USER_ACTION
User action (button click, form submission, selection, etc.)

```json
{
  "type": "USER_ACTION",
  "action_type": "submit",
  "component_id": "form_settings",
  "payload": { "theme": "dark", "language": "en" }
}
```

### System Events

#### CONNECTION_STATUS
Server notifies client of connection state changes.

```json
{
  "type": "CONNECTION_STATUS",
  "status": "CONNECTED",
  "message": "Connected to orchestrator v0.1"
}
```

**Status Values:** `CONNECTING`, `CONNECTED`, `DISCONNECTED`, `RECONNECTING`, `FAILED`

#### ERROR
Protocol error from server.

```json
{
  "type": "ERROR",
  "code": "PARSE_ERROR",
  "message": "Invalid event format: missing 'type' field",
  "details": { "field": "event.type" }
}
```

## Error Handling

### Standard Error Codes

- `PARSE_ERROR` - JSON parsing failed
- `INVALID_EVENT` - Unknown or malformed event
- `INVALID_VERSION` - Incompatible protocol version
- `INVALID_EVENT_DATA` - Field data doesn't meet requirements
- `MISSING_FIELD` - Required field is missing
- `CONNECTION_ERROR` - Network/connection issue

All errors follow the standard format:

```json
{
  "type": "ERROR",
  "code": "ERROR_CODE",
  "message": "Human-readable description",
  "details": {}
}
```

## Mock Server (Development)

A mock WebSocket server is available for development and testing:

- **Default Port:** 3001
- **Endpoint:** `ws://127.0.0.1:3001/ws`
- **Auto-streaming:** Enabled by default (sends demo events every 2 seconds)
- **Implemented in:** `crates/agui-desktop/src/mock_server.rs`

Configuration:

```rust
MockServerConfig {
    port: 3001,
    auto_stream: true,
    stream_interval_ms: 2000,
}
```

## Implementation Notes

### Serialization

- Uses `serde`/`serde_json` for all serialization
- Event variant discrimination via `#[serde(tag = "type")]`
- Optional fields skipped during serialization: `#[serde(skip_serializing_if = "Option::is_none")]`
- Event type names use SCREAMING_SNAKE_CASE

### Type Safety

Defined in `crates/agui-desktop/src/protocol.rs` using Rust enums and structs for compile-time safety.

### Testing

Unit tests verify:
- Event serialization/deserialization round-trips
- Event envelope creation with unique IDs
- Protocol version handling
- Error event construction

Run tests: `cargo test protocol::`

## Version History

### v0.1.0 (Current)
- Initial protocol specification (2026-01-09)
- 12+ event types covering messaging, tools, UI, resources
- Error handling with standard codes
- Mock server support for development
- Comprehensive test coverage
