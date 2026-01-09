//! Stream Timeline Module
//!
//! Implements the virtualized timeline for Zone B (Stream) with:
//! - User/agent message bubbles (with markdown support)
//! - Reasoning accordions (collapsible Chain-of-Thought)
//! - Tool-call cards (header, params, status, result)
//! - Plan checklists (task lists with completion state)
//! - Approval gates (approve/reject actions)
//! - Progress/status indicators
//!
//! Uses virtual scrolling for performant rendering of 1000+ items.

pub mod components;
pub mod state;
pub mod types;
pub mod virtual_list;

pub use components::*;
pub use state::StreamState;
pub use types::*;
pub use virtual_list::VirtualList;
