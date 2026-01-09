//! Resilience patterns for AGUI
//!
//! Provides connection management, update batching, error handling,
//! and graceful degradation.

mod state;
pub mod components;

pub use state::*;
