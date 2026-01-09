---
id: "uag-foundation-v1"
title: "Universal Agent GUI - Foundation Specification v1.0"
type: "architecture"
scope: "internal"
created_at: "2026-01-08T23:43:00Z"
updated_at: "2026-01-08T23:43:00Z"
---

# Universal Agent GUI (UAG) - Foundation Specification v1.0

**Status:** Draft / Request for Comment
**Date:** January 8, 2026
**Author:** Lead GUI Designer
**Context:** Project AGUI

---

## 1. Executive Summary: "Claude Code for the GUI"

The goal of the Universal Agent GUI (UAG) is to standardize the visual interface for high-agency AI systems. Just as *Claude Code* standardized the CLI workflow for developer agents (Explore $\to$ Plan $\to$ Code $\to$ Commit), UAG aims to standardize the graphical workflow for general-purpose agents.

**Core Philosophy:**
1.  **Separation of Intent & Artifact:** The discussion about the work is not the work itself. The UI must physically separate the *Chat/Reasoning* stream from the *Artifact/State* view.
2.  **Transparency as UX:** Trust is built by visibility. Tool calls, reasoning chains, and confidence scores are not debug info; they are primary UI elements.
3.  **Universal Container, Specialized Content:** The shell is standardized; the content is generative.

---

## 2. The Universal Shell Anatomy

To be "universal," the layout must accommodate various agent types (coding, research, creative) without redesign. We propose a **Three-Column "Holy Grail" Layout**:

### Zone A: The Context Rail (Left - Collapsible)
*Persisting context and resources.*
- **Session Context:** Attached files, active memories, and user preferences.
- **Resource Monitor:** (MCP Layer) Live view of connected databases, APIs, or file systems the agent can access.
- **Agent Team:** If multi-agent, a roster of active agents and their current status (Idle, Thinking, Executing).

### Zone B: The Stream (Center - The Narrative)
*The history of intent, reasoning, and communication.*
- **Interaction Mode:** A linear timeline of user prompts and agent responses.
- **Reasoning Accordions:** "Thought bubbles" that are collapsed by default but expand to show the Chain-of-Thought (CoT).
- **Tool Events:** Discrete cards showing `[Tool Request] -> [Input Params] -> [Status Spinner] -> [Output Summary]`.
- **Approval Gates:** Explicit UI blocks demanding user action (Approve/Reject/Modify) before execution proceeds.

### Zone C: The Stage (Right - The Artifact)
*The mutable shared state.*
- **Dynamic Viewport:** This area renders the *result* of the work.
    - *For Coders:* A syntax-highlighted diff view or preview.
    - *For Writers:* A rich-text document editor.
    - *For Analysts:* Interactive charts or data tables.
- **Generative UI Host:** If the agent needs to ask a complex question (e.g., a form to gather missing params), the form renders here, keeping the Stream clean.
- **Tabs/History:** Users can scrub back through versions of the artifact.

---

## 3. Interaction Protocols & UX Patterns

### 3.1 The "Explore-Plan-Act" Loop
Drawing from *Claude Code*, the UI must make the agent's lifecycle states explicit. The global status indicator (top bar) transitions through these phases:
1.  **Explore/Listen:** Agent is gathering context. UI highlights the "Context Rail".
2.  **Plan:** Agent proposes a course of action.
    *   *UI Pattern:* **The Plan Card**. A structured list of intended steps rendered in the Stream. The user must click "Confirm Plan" or "Edit Plan" to proceed.
3.  **Act/Execute:** Agent fires tool calls.
    *   *UI Pattern:* **The Action HUD**. A non-blocking overlay on the Stage showing real-time tool execution (e.g., "Scanning file...", "Querying API...").
4.  **Verify:** Agent checks its work.

### 3.2 Artifact-First Design
In chat-based AIs, code/text snippets often get lost in the scrollback. UAG promotes **Artifacts** to first-class citizens.
- Any substantial output (code, report, image) is automatically extracted from the chat stream and pinned to **The Stage**.
- **Bi-directional Sync:** If the user edits the Artifact on the Stage manually, a `STATE_DELTA` event is sent to the agent, ensuring the agent knows the user interfered.

### 3.3 Progressive Disclosure of Complexity
- **Level 1 (User):** Simple chat + Final Artifact.
- **Level 2 (Reviewer):** Plan Cards + Tool Input/Outputs summaries.
- **Level 3 (Debug/Audit):** Full JSON payloads of tool calls, raw CoT logs, latency metrics.
*Implementation:* A global "Detail Level" slider or toggle affects the density of the Stream.

---

## 4. Technical Foundations (The Protocol Stack)

The UAG Spec mandates adherence to the following standards to ensure interoperability:

1.  **Transport Layer:** **AG-UI Protocol**.
    - Must support `TEXT_MESSAGE_CONTENT`, `TOOL_CALL_START`, `TOOL_CALL_FINISH`, and `STATE_DELTA`.
    - **Streaming is Mandatory:** The UI must parse partial JSON chunks to animate tool arguments as they are generated.

2.  **Capability Layer:** **MCP (Model Context Protocol)**.
    - The "Context Rail" is essentially an MCP Client visualizer. It lists available Prompts, Resources, and Tools as discoverable UI elements.

3.  **Delegation Layer:** **A2A**.
    - When a task is delegated, the UI spawns a **Sub-Agent Card** in the Stream. This card acts as a mini-terminal for the sub-agent, containing its own nested state, preventing the main stream from becoming cluttered.

---

## 5. Critical UI Components (The Kit)

To build this, we need a standard component library (The "UAG Design System"):

1.  **The Thought Bubble:** A distinct visual container for internal monologue. Styling: Low contrast, monospace font, collapsible.
2.  **The Tool Chip:** A compact pill showing `Tool Name`, `Duration`, and `Status (Success/Fail)`. Hovering reveals inputs/outputs.
3.  **The Diff/Merge View:** A robust component for showing "Before" and "After" states of text or structured data.
4.  **The Confidence Meter:** A subtle indicator (e.g., a glow color or ring) around the agent's avatar indicating certainty level.
5.  **The "Stop/Intervene" Button:** A massive, always-visible panic button that doesn't just kill the connection but sends a high-priority `INTERRUPT` signal to the agent runtime to gracefully halt.

---

## 6. Open Questions & Next Steps

To move from Spec to Wireframe, we must decide:
1.  **Mobile Strategy:** Does "Universal" include mobile? The Three-Column layout collapses poorly. *Proposal: On mobile, The Stage becomes a drawer/modal.*
2.  **Generative Limits:** Do we allow agents to emit raw HTML/JS (Open-Ended UI) or strictly JSON-schema UI (Declarative)? *Recommendation: Strict Declarative (JSON-to-Component) for security and consistency.*
3.  **Persistence:** Does the UI own the history, or does the Agent? *Recommendation: The UI is a view; the Agent (or a neutral Orchestrator) owns the session state.*

---
**Prepared for:** User (The Architect)
**Next Phase:** Wireframing & Protocol Binding Definition