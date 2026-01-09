//! Stage Module - Artifact Workspace (Zone C)
//!
//! Implements the Stage pane for displaying and editing artifacts:
//! - Code/text view with syntax highlighting
//! - Diff view for comparing versions
//! - Editable and read-only modes
//! - STATE_DELTA hydration
//! - Caching and chunking for large artifacts

pub mod cache;
pub mod components;
pub mod diff;
pub mod state;
pub mod syntax;
pub mod types;

pub use components::*;
pub use state::StageState;
pub use types::*;
