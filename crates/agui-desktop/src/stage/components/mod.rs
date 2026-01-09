//! Stage Components
//!
//! UI components for rendering artifacts in the Stage pane.

pub mod artifact_view;
pub mod diff_view;
pub mod tabs;
pub mod text_view;

pub use artifact_view::*;
pub use diff_view::*;
pub use tabs::*;
pub use text_view::*;

/// Common colors for Stage components (VS Code dark theme)
pub mod colors {
    /// Background color for editor
    pub const EDITOR_BG: u32 = 0x1e1e1e;
    /// Background for line numbers gutter
    pub const GUTTER_BG: u32 = 0x1e1e1e;
    /// Line number text color
    pub const LINE_NUMBER: u32 = 0x858585;
    /// Current line number (highlighted)
    pub const LINE_NUMBER_ACTIVE: u32 = 0xc6c6c6;
    /// Current line background
    pub const CURRENT_LINE_BG: u32 = 0x282828;
    /// Selection background
    pub const SELECTION_BG: u32 = 0x264f78;
    /// Cursor color
    pub const CURSOR: u32 = 0xaeafad;
    /// Tab bar background
    pub const TAB_BAR_BG: u32 = 0x252526;
    /// Active tab background
    pub const TAB_ACTIVE_BG: u32 = 0x1e1e1e;
    /// Inactive tab background
    pub const TAB_INACTIVE_BG: u32 = 0x2d2d2d;
    /// Tab border
    pub const TAB_BORDER: u32 = 0x1e1e1e;
    /// Tab text (inactive)
    pub const TAB_TEXT: u32 = 0x808080;
    /// Tab text (active)
    pub const TAB_TEXT_ACTIVE: u32 = 0xffffff;
    /// Dirty indicator (dot)
    pub const DIRTY_DOT: u32 = 0xffffff;
    /// Close button
    pub const CLOSE_BUTTON: u32 = 0x808080;
    /// Close button hover
    pub const CLOSE_HOVER: u32 = 0xffffff;
    /// Diff addition background
    pub const DIFF_ADD_BG: u32 = 0x2d4a2d;
    /// Diff addition line number background
    pub const DIFF_ADD_GUTTER: u32 = 0x3d6b3d;
    /// Diff deletion background
    pub const DIFF_DEL_BG: u32 = 0x4a2d2d;
    /// Diff deletion line number background
    pub const DIFF_DEL_GUTTER: u32 = 0x6b3d3d;
    /// Diff context (unchanged) - slightly dimmed
    pub const DIFF_CONTEXT: u32 = 0x1e1e1e;
    /// Hunk header background
    pub const DIFF_HUNK_HEADER: u32 = 0x2d2d5a;
    /// Scrollbar track
    pub const SCROLLBAR_TRACK: u32 = 0x1e1e1e;
    /// Scrollbar thumb
    pub const SCROLLBAR_THUMB: u32 = 0x424242;
    /// Scrollbar thumb hover
    pub const SCROLLBAR_HOVER: u32 = 0x4e4e4e;
}
