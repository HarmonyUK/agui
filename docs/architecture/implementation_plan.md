---
id: "agui-implementation-plan"
title: "AGUI Rust Client - Implementation Plan"
type: "architecture"
scope: "internal"
created_at: "2026-01-08T23:43:00Z"
updated_at: "2026-01-08T23:43:00Z"
---

# AGUI Rust Client - Implementation Plan

**Goal:** Build a native Rust desktop client for the Universal Agent GUI (UAG) specification.
**Framework Strategy:** `Iced` (for reliable, accessible, type-safe GUI) + `cosmic-text` (for rich text editing).

## Phase 1: The Skeleton (Foundations)
*Est. Duration: 1-2 Days*

*   **Task 1.1: Project Scaffolding**
    *   Initialize `agui-desktop` crate.
    *   Setup `iced` (latest version) with `tokio` feature.
    *   Define the main `Application` struct and `Message` enum.
*   **Task 1.2: The "Holy Grail" Layout**
    *   Implement the main `view` function to render a 3-column layout (`Row::new().push(ZoneA).push(ZoneB).push(ZoneC)`).
    *   Make columns resizable (SplitView pattern).
    *   **Deliverable:** A window that opens with 3 empty, colored rectangles that resize.

## Phase 2: The Protocol Layer (Data)
*Est. Duration: 2-3 Days*

*   **Task 2.1: AG-UI Types**
    *   Define Rust structs for all AG-UI events (`TextMessage`, `ToolCall`, `StateDelta`, etc.) using `serde`.
    *   Create the `ClientState` struct (the Source of Truth for the frontend).
*   **Task 2.2: Network Client**
    *   Implement an async "Subscription" in Iced to listen to SSE/WebSockets.
    *   Handle `ConnectionError`, `Reconnecting`, and `Connected` states.
    *   **Deliverable:** The app connects to a mock server and logs incoming events to stdout.

## Phase 3: Zone B - The Stream (Interaction)
*Est. Duration: 3-4 Days*

*   **Task 3.1: Event Blocks**
    *   Create Iced components for:
        *   `UserMessageBubble` (Right aligned, distinctive color).
        *   `AgentMessageBubble` (Left aligned, Markdown parsing).
*   **Task 3.2: Complex Widgets**
    *   Implement `ReasoningAccordion` (Collapsible view for Chain-of-Thought).
    *   Implement `ToolCallCard` (Header, Params grid, Status footer).
*   **Task 3.3: Virtualization**
    *   Ensure the list can handle 1000+ items without lag (using `scrollable` or a virtual list crate).

## Phase 4: Zone C - The Stage (Editor)
*Est. Duration: 4-5 Days*

*   **Task 4.1: Editor Integration**
    *   Integrate `cosmic-text` or a dedicated Iced editor widget.
    *   Implement syntax highlighting support (using `syntect`).
*   **Task 4.2: File Synchronization**
    *   Handle `ARTIFACT_OPEN` and `ARTIFACT_UPDATE` events to populate the editor.
    *   Implement "ReadOnly" vs "Editable" modes based on Agent state.

## Phase 5: Zone A - Context & Resources
*Est. Duration: 2 Days*

*   **Task 5.1: Resource Tree**
    *   Implement a TreeView widget for files/resources.
*   **Task 5.2: Agent Roster**
    *   Simple list of connected agents and their status.

## Phase 6: The Declarative Engine (Generative UI)
*Est. Duration: 5 Days*

*   **Task 6.1: Schema Definition**
    *   Finalize the JSON Schema for UI components.
*   **Task 6.2: Mapper Logic**
    *   Write the recursive function `render_component(schema) -> Element<Message>`.
    *   Implement basic widgets: `Input`, `Select`, `Button`, `Text`, `Card`.
*   **Task 6.3: Form State Handling**
    *   Create a system to track the state of generated forms and submit the result back to the Agent.

---

## Dependencies (Preliminary)
```toml
[dependencies]
iced = { version = "0.12", features = ["tokio", "svg", "image"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.11", features = ["json", "stream"] } # For SSE
eventsource-stream = "0.2"
cosmic-text = "0.11" # Or similar editor crate
syntect = "5.0"
```