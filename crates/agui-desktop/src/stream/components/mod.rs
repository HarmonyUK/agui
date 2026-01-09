//! Stream Timeline Components
//!
//! Provides rendering functions for all stream timeline items:
//! - User/Agent message bubbles
//! - Reasoning accordions
//! - Tool call cards
//! - Plan checklists
//! - Approval gates
//! - Status updates

pub mod agent_bubble;
pub mod approval_gate;
pub mod plan_checklist;
pub mod reasoning_accordion;
pub mod status_update;
pub mod stream_item;
pub mod stream_timeline;
pub mod tool_call_card;
pub mod user_bubble;

pub use agent_bubble::*;
pub use approval_gate::*;
pub use plan_checklist::*;
pub use reasoning_accordion::*;
pub use status_update::*;
pub use stream_item::*;
pub use stream_timeline::*;
pub use tool_call_card::*;
pub use user_bubble::*;

/// Common color palette for stream components
pub mod colors {
    use gpui::Rgba;

    /// Convert hex color to Rgba
    pub fn hex(value: u32) -> Rgba {
        gpui::rgb(value)
    }

    // Message bubbles
    pub fn user_bubble_bg() -> Rgba {
        hex(0x0e639c) // Blue
    }
    pub fn user_bubble_text() -> Rgba {
        hex(0xffffff)
    }
    pub fn agent_bubble_bg() -> Rgba {
        hex(0x2d2d30) // Dark gray
    }
    pub fn agent_bubble_text() -> Rgba {
        hex(0xcccccc)
    }

    // Reasoning
    pub fn reasoning_bg() -> Rgba {
        hex(0x1e1e1e)
    }
    pub fn reasoning_border() -> Rgba {
        hex(0x404040)
    }
    pub fn reasoning_text() -> Rgba {
        hex(0x808080) // Muted
    }
    pub fn reasoning_header() -> Rgba {
        hex(0x606060)
    }

    // Tool calls
    pub fn tool_header_bg() -> Rgba {
        hex(0x252526)
    }
    pub fn tool_param_bg() -> Rgba {
        hex(0x1e1e1e)
    }
    pub fn tool_name() -> Rgba {
        hex(0xdcdcaa) // Yellow (function-like)
    }

    // Status colors
    pub fn status_pending() -> Rgba {
        hex(0x808080)
    }
    pub fn status_running() -> Rgba {
        hex(0x007acc) // Blue
    }
    pub fn status_completed() -> Rgba {
        hex(0x4ec9b0) // Green
    }
    pub fn status_failed() -> Rgba {
        hex(0xf14c4c) // Red
    }
    pub fn status_cancelled() -> Rgba {
        hex(0x808080)
    }

    // Plan checklist
    pub fn plan_bg() -> Rgba {
        hex(0x252526)
    }
    pub fn plan_item_pending() -> Rgba {
        hex(0x606060)
    }
    pub fn plan_item_active() -> Rgba {
        hex(0x007acc)
    }
    pub fn plan_item_completed() -> Rgba {
        hex(0x4ec9b0)
    }
    pub fn plan_item_skipped() -> Rgba {
        hex(0x606060)
    }

    // Approval gate
    pub fn approval_bg() -> Rgba {
        hex(0x2d2d30)
    }
    pub fn approval_border() -> Rgba {
        hex(0x007acc)
    }
    pub fn approve_button() -> Rgba {
        hex(0x4ec9b0)
    }
    pub fn reject_button() -> Rgba {
        hex(0xf14c4c)
    }

    // Progress
    pub fn progress_bg() -> Rgba {
        hex(0x3c3c3c)
    }
    pub fn progress_fill() -> Rgba {
        hex(0x0e639c)
    }

    // General
    pub fn divider() -> Rgba {
        hex(0x3c3c3c)
    }
    pub fn text_primary() -> Rgba {
        hex(0xcccccc)
    }
    pub fn text_secondary() -> Rgba {
        hex(0x808080)
    }
    pub fn text_muted() -> Rgba {
        hex(0x606060)
    }
}
