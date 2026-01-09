---
id: doc-r9t
title: AGUI gpui goals
type: context
scope: internal
created_at: '2026-01-09T00:29:57.881458Z'
updated_at: '2026-01-09T00:29:57.881458Z'
---

# AGUI gpui goals

## Overview

Shared goals and acceptance targets for the gpui-based AGUI desktop client, to align downstream beads and verify intent.

## Key Information

- Single-agent UI (one active agent at a time) using UAG three-column layout: Context Rail (Zone A), Stream (Zone B), Stage (Zone C).
- Transport: WebSocket to orchestrator; JSON events for text, tool calls, plan cards, state deltas, artifacts, render requests, user actions, connection lifecycle.
- Declarative UI only; extended schema covers forms, tables, diff/code views, accordions/tabs, toasts/modals, status badges, layout primitives.
- Artifact-first: Stage shows current artifact (editable/read-only) with diff/highlight; Stream focuses on narrative/tooling; Context Rail shows resources/roster/stats.
- Framework: gpui (no webview fallback). Packaging via cargo-dist: AppImage/DEB/RPM (Linux first), DMG (macOS), MSI (Windows).
- Perf target: smooth on ~10-core Intel CPU, iGPU, 16 GB RAM, 512 GB SSD; handle 1–2k+ stream items without noticeable lag.
- Resilience: reconnect/backoff, reduced-detail fallback, batching/diffing updates, memoized renders, asset caching, error/offline UI.

## Related Documents

- docs/design/implementation_plan.md
- docs/design/uag_foundation_v1.md
- docs/design/uag_rust_v1.1.md

## Delivery milestones (beads)

1) agui-xwd (plan) – anchors intent.  
2) agui-3q9 (scaffold/tooling) – gpui crate, dev tooling.  
3) agui-263 (layout shell) – 3-column frame + status bar + keyboard switching.  
4) agui-n59 (protocol/schema + mock) – JSON contract + mock orchestrator.  
5) agui-xwe (stream + virtualization) – timeline blocks, perf benchmarks.  
6) agui-5lx (stage workspace) – code/text/diff views, artifact hydration.  
7) agui-2gh (declarative renderer) – extended schema mapping + forms.  
8) agui-ver (perf/resilience) – batching/caching/compaction/metrics/error UX.  
9) agui-6kd (packaging) – cargo-dist multi-platform artifacts + smoke tests.
