---
id: "uag-rust-v1-1"
title: "Universal Agent GUI - Rust Desktop Specification v1.1"
type: "architecture"
scope: "internal"
created_at: "2026-01-08T23:43:00Z"
updated_at: "2026-01-08T23:43:00Z"
---

# Universal Agent GUI (UAG) - Rust Desktop Specification v1.1

**Status:** Draft / Request for Comment
**Date:** January 8, 2026
**Context:** Project AGUI
**Platform:** Rust Native (Desktop Only)

---

## 1. Executive Summary

The Universal Agent GUI (UAG) is a native desktop application built in Rust. It serves as the standardized "cockpit" for high-agency AI systems. It creates a strict separation between the **Conversation** (Intent) and the **Artifact** (Result), ensuring users can manage complex agentic workflows without losing context.

**Core Decisions:**
1.  **Native Desktop:** Optimized for high-performance, persistent desktop usage. No mobile constraints.
2.  **Strict Declarative UI:** The agent controls the UI via secure JSON schemas. No arbitrary code execution.
3.  **Backend-Driven State:** The GUI is a "Thin Client" view; the Agent/Orchestrator holds the source of truth.

---

## 2. Architecture: The "Thin Client" Pattern

To answer the **State Ownership** question: This specification mandates a **Backend-Driven Source of Truth**.

### 2.1 The Concept
*   **The Orchestrator (Backend):** A persistent process (local or remote) that manages the Agent loop, holds the `ConversationHistory`, `ArtifactState`, and `ToolOutputs`. It is the "Database" of the session.
*   **The GUI (Frontend/Rust):** A stateless renderer. When it starts, it handshakes with the Orchestrator to `SYNC` the current state.
    *   *Why?* If the GUI crashes or is closed, long-running agent jobs continue. When the GUI re-opens, it simply "rehydrates" the state. The GUI never "saves" the chat; it just asks the Orchestrator to append to it.

### 2.2 Data Flow
1.  **User Action:** User clicks "Approve Plan" in the Rust GUI.
2.  **Event Emission:** GUI sends `{"type": "USER_ACTION", "action": "APPROVE", "payload": {...}}` to the Orchestrator.
3.  **State Mutation:** Orchestrator receives event, updates its internal state machine (transitions from `PLANNING` to `EXECUTING`).
4.  **State Broadcast:** Orchestrator broadcasts a `STATE_DELTA` event.
5.  **Render:** GUI receives the delta and updates the view (e.g., changes the "Plan Card" color to Green).

---

## 3. The Universal Layout (Desktop Optimized)

We utilize a fixed 3-Column Layout, leveraging the screen real estate of desktop environments.

### Zone A: The Context Rail (Left - 250px-350px)
*The "Read-Only" State Viewer.*
- **Resource Tree:** A file-explorer style tree view of connected MCP Resources (DB tables, files).
- **Agent Roster:** List of active agents. Indicators for `Idle`, `Busy`, `Error`.
- **Session Info:** Token usage, total cost, duration.

### Zone B: The Stream (Center - Flexible Width)
*The Narrative Timeline.*
- **Rendering:** A virtualized list of "Event Blocks".
- **Block Types:**
    - `UserMessage`: Text bubbles.
    - `AgentReasoning`: Collapsible sections (default: collapsed) containing raw Chain-of-Thought text.
    - `ToolCall`: A structured widget displaying:
        - Header: Tool Name (e.g., `fs.read_file`)
        - Body: Key-Value pairs of arguments (formatted as a read-only property grid).
        - Footer: Status icon & Execution time.
    - `PlanCard`: A checklist widget. Items can be `pending`, `active`, or `done`.

### Zone C: The Stage (Right - Flexible/Collapsible)
*The Mutable Artifact Workspace.*
- **Purpose:** Displays the *current version* of the file, report, or data being generated.
- **Views:**
    - **Code/Text:** Syntax-highlighted editor (read-only or editable).
    - **Data Grid:** For structured data (CSV/JSON results).
    - **Diff View:** Side-by-side comparison when the Agent proposes changes.
- **Generative UI Container:**
    - If the Agent requests user input (e.g., "I need your API key"), it sends a **Declarative Schema** (see Section 4). The Stage renders this as a native Rust Form.

---

## 4. Strict Declarative UI Protocol

The Agent cannot send HTML or WASM. It must send a `UI_SCHEMA` JSON payload. The Rust client maps this schema to native widgets (e.g., using `iced` or `egui`).

### 4.1 The Schema Contract
The Agent sends a `render_request` event:
```json
{
  "type": "render_request",
  "id": "form_123",
  "components": [
    {
      "type": "header",
      "text": "Deployment Configuration"
    },
    {
      "type": "input",
      "label": "Environment Name",
      "key": "env_name",
      "required": true
    },
    {
      "type": "select",
      "label": "Region",
      "options": ["us-east-1", "eu-west-1"],
      "key": "region"
    }
  ]
}
```

### 4.2 Native Mapping
The Rust application parses `components`:
- `header` $\to$ `Text::new(...).size(24)`
- `input` $\to$ `TextInput::new(...)`
- `select` $\to$ `PickList::new(...)`

This guarantees that the Agent can *only* render UI elements that the native application has explicitly implemented and styled.

---

## 5. Technology Stack Recommendations (Rust)

To achieve a "Universal" desktop look with high performance:

1.  **GUI Framework:**
    - **Option A: `Iced`:** Pure Rust, Elm architecture. Excellent for the "State Delta" pattern because it strictly separates state from view. Very type-safe.
    - **Option B: `GPUI` (Zed's engine):** High performance, GPU accelerated. Best if we want "editor-grade" text rendering in Zone C.
    - **Option C: `Tauri` (v2):** Uses system webview. *User Note: You specified "Not a web project". Tauri technically renders HTML/JS, but the backend is Rust. If you want "Pixel-Perfect Native Widgets", avoid this. If you want "Easy CSS Styling", use this.*
    - **Recommendation:** **`Iced`** or **`Relm4` (GTK4)** for a true native feel without web tech.

2.  **Event Handling:**
    - Use `tokio` for the async runtime.
    - Use `Server-Sent Events (SSE)` or `WebSockets` to listen to the Orchestrator.

---

## 6. Next Steps

1.  **Select GUI Framework:** Confirm choice (e.g., Iced vs. GPUI).
2.  **Define JSON Schema:** Create the exact JSON spec for the "Declarative UI" components.
3.  **Mockup Zone B:** Design the specific "Event Blocks" for Tool Calls and Plans.