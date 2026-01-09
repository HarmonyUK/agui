//! UI Component Renderers
//!
//! Each submodule provides rendering functions for a category of UI components.

pub mod button;
pub mod container;
pub mod data;
pub mod form;
pub mod input;
pub mod layout;
pub mod text;

/// Common color palette for components
pub mod colors {
    use gpui::Rgba;

    /// Convert hex color to Rgba
    pub fn hex_to_rgba(hex: u32) -> Rgba {
        gpui::rgb(hex)
    }

    // Background colors
    pub fn bg_dark() -> Rgba {
        hex_to_rgba(0x1e1e1e)
    }
    pub fn bg_panel() -> Rgba {
        hex_to_rgba(0x252526)
    }
    pub fn bg_elevated() -> Rgba {
        hex_to_rgba(0x2d2d30)
    }
    pub fn bg_input() -> Rgba {
        hex_to_rgba(0x3c3c3c)
    }
    pub fn bg_hover() -> Rgba {
        hex_to_rgba(0x404040)
    }

    // Border colors
    pub fn border_default() -> Rgba {
        hex_to_rgba(0x3c3c3c)
    }
    pub fn border_focused() -> Rgba {
        hex_to_rgba(0x007acc)
    }
    pub fn border_error() -> Rgba {
        hex_to_rgba(0xf14c4c)
    }

    // Text colors
    pub fn text_primary() -> Rgba {
        hex_to_rgba(0xcccccc)
    }
    pub fn text_secondary() -> Rgba {
        hex_to_rgba(0x808080)
    }
    pub fn text_muted() -> Rgba {
        hex_to_rgba(0x606060)
    }
    pub fn text_link() -> Rgba {
        hex_to_rgba(0x3794ff)
    }

    // Accent colors
    pub fn primary() -> Rgba {
        hex_to_rgba(0x007acc)
    }
    pub fn primary_hover() -> Rgba {
        hex_to_rgba(0x0098ff)
    }
    pub fn secondary() -> Rgba {
        hex_to_rgba(0x3c3c3c)
    }
    pub fn secondary_hover() -> Rgba {
        hex_to_rgba(0x505050)
    }

    // Status colors
    pub fn success() -> Rgba {
        hex_to_rgba(0x4ec9b0)
    }
    pub fn warning() -> Rgba {
        hex_to_rgba(0xdcdcaa)
    }
    pub fn error() -> Rgba {
        hex_to_rgba(0xf14c4c)
    }
    pub fn info() -> Rgba {
        hex_to_rgba(0x3794ff)
    }

    // Badge variants
    pub fn badge_default() -> Rgba {
        hex_to_rgba(0x4d4d4d)
    }

    // Progress bar
    pub fn progress_bg() -> Rgba {
        hex_to_rgba(0x3c3c3c)
    }
    pub fn progress_fill() -> Rgba {
        hex_to_rgba(0x0e639c)
    }
}
