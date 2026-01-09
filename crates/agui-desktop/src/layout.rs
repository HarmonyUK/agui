//! Layout management for the 3-column AGUI shell
//!
//! Manages the Context Rail (Zone A), Stream (Zone B), and Stage (Zone C)
//! with resizable widths, collapsible panels, and keyboard-driven pane switching.

use gpui::{px, Pixels};

/// The three panes in the UAG layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pane {
    /// Zone A - Context Rail (left panel)
    ContextRail,
    /// Zone B - Stream (center panel, default focus)
    #[default]
    Stream,
    /// Zone C - Stage (right panel)
    Stage,
}

impl Pane {
    /// Get the display name of the pane
    pub fn name(&self) -> &'static str {
        match self {
            Pane::ContextRail => "Context Rail",
            Pane::Stream => "Stream",
            Pane::Stage => "Stage",
        }
    }

    /// Get the keyboard shortcut hint (1-indexed)
    pub fn shortcut_hint(&self) -> &'static str {
        match self {
            Pane::ContextRail => "Ctrl+1",
            Pane::Stream => "Ctrl+2",
            Pane::Stage => "Ctrl+3",
        }
    }
}

/// Width constraints for panels
#[derive(Debug, Clone, Copy)]
pub struct PanelConstraints {
    /// Minimum width in pixels
    pub min_width: f32,
    /// Maximum width in pixels
    pub max_width: f32,
    /// Default width in pixels
    pub default_width: f32,
}

impl PanelConstraints {
    pub const fn new(min: f32, max: f32, default: f32) -> Self {
        Self {
            min_width: min,
            max_width: max,
            default_width: default,
        }
    }

    /// Clamp a width value to the constraints
    pub fn clamp(&self, width: f32) -> f32 {
        width.clamp(self.min_width, self.max_width)
    }

    /// Get the default width as Pixels
    pub fn default_pixels(&self) -> Pixels {
        px(self.default_width)
    }
}

/// Default constraints for the Context Rail
pub const CONTEXT_RAIL_CONSTRAINTS: PanelConstraints = PanelConstraints::new(200.0, 400.0, 280.0);

/// Default constraints for the Stage
pub const STAGE_CONSTRAINTS: PanelConstraints = PanelConstraints::new(300.0, 800.0, 400.0);

/// Layout state for the 3-column shell
#[derive(Debug, Clone)]
pub struct LayoutState {
    /// Width of the Context Rail panel (in pixels)
    pub context_rail_width: f32,
    /// Width of the Stage panel (in pixels)
    pub stage_width: f32,
    /// Whether the Context Rail is collapsed
    pub context_rail_collapsed: bool,
    /// Whether the Stage is collapsed
    pub stage_collapsed: bool,
    /// Currently focused pane
    pub focused_pane: Pane,
    /// Constraints for Context Rail
    pub context_rail_constraints: PanelConstraints,
    /// Constraints for Stage
    pub stage_constraints: PanelConstraints,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutState {
    /// Create a new layout state with default values
    pub fn new() -> Self {
        Self {
            context_rail_width: CONTEXT_RAIL_CONSTRAINTS.default_width,
            stage_width: STAGE_CONSTRAINTS.default_width,
            context_rail_collapsed: false,
            stage_collapsed: false,
            focused_pane: Pane::Stream,
            context_rail_constraints: CONTEXT_RAIL_CONSTRAINTS,
            stage_constraints: STAGE_CONSTRAINTS,
        }
    }

    /// Focus a specific pane
    pub fn focus_pane(&mut self, pane: Pane) {
        // Expand the pane if it's collapsed
        match pane {
            Pane::ContextRail if self.context_rail_collapsed => {
                self.context_rail_collapsed = false;
            }
            Pane::Stage if self.stage_collapsed => {
                self.stage_collapsed = false;
            }
            _ => {}
        }
        self.focused_pane = pane;
        tracing::debug!("Focused pane: {:?}", pane);
    }

    /// Focus the next pane (cycles: ContextRail -> Stream -> Stage -> ContextRail)
    pub fn focus_next(&mut self) {
        let next = match self.focused_pane {
            Pane::ContextRail => Pane::Stream,
            Pane::Stream => Pane::Stage,
            Pane::Stage => Pane::ContextRail,
        };
        self.focus_pane(next);
    }

    /// Focus the previous pane (cycles: ContextRail <- Stream <- Stage <- ContextRail)
    pub fn focus_previous(&mut self) {
        let prev = match self.focused_pane {
            Pane::ContextRail => Pane::Stage,
            Pane::Stream => Pane::ContextRail,
            Pane::Stage => Pane::Stream,
        };
        self.focus_pane(prev);
    }

    /// Toggle collapse state for the Context Rail
    pub fn toggle_context_rail(&mut self) {
        self.context_rail_collapsed = !self.context_rail_collapsed;
        tracing::debug!(
            "Context Rail collapsed: {}",
            self.context_rail_collapsed
        );
        // If we collapsed the focused pane, move focus to Stream
        if self.context_rail_collapsed && self.focused_pane == Pane::ContextRail {
            self.focused_pane = Pane::Stream;
        }
    }

    /// Toggle collapse state for the Stage
    pub fn toggle_stage(&mut self) {
        self.stage_collapsed = !self.stage_collapsed;
        tracing::debug!("Stage collapsed: {}", self.stage_collapsed);
        // If we collapsed the focused pane, move focus to Stream
        if self.stage_collapsed && self.focused_pane == Pane::Stage {
            self.focused_pane = Pane::Stream;
        }
    }

    /// Set the Context Rail width (clamped to constraints)
    pub fn set_context_rail_width(&mut self, width: f32) {
        self.context_rail_width = self.context_rail_constraints.clamp(width);
    }

    /// Set the Stage width (clamped to constraints)
    pub fn set_stage_width(&mut self, width: f32) {
        self.stage_width = self.stage_constraints.clamp(width);
    }

    /// Get the Context Rail width as Pixels
    pub fn context_rail_pixels(&self) -> Pixels {
        px(self.context_rail_width)
    }

    /// Get the Stage width as Pixels
    pub fn stage_pixels(&self) -> Pixels {
        px(self.stage_width)
    }

    /// Get the effective width of the Context Rail (0 if collapsed)
    pub fn effective_context_rail_width(&self) -> f32 {
        if self.context_rail_collapsed {
            0.0
        } else {
            self.context_rail_width
        }
    }

    /// Get the effective width of the Stage (0 if collapsed)
    pub fn effective_stage_width(&self) -> f32 {
        if self.stage_collapsed {
            0.0
        } else {
            self.stage_width
        }
    }

    /// Check if a pane is the currently focused pane
    pub fn is_focused(&self, pane: Pane) -> bool {
        self.focused_pane == pane
    }

    /// Get a human-readable status string
    pub fn status_string(&self) -> String {
        let context_status = if self.context_rail_collapsed {
            "hidden"
        } else {
            "visible"
        };
        let stage_status = if self.stage_collapsed {
            "hidden"
        } else {
            "visible"
        };
        format!(
            "Focus: {} | Context: {} | Stage: {}",
            self.focused_pane.name(),
            context_status,
            stage_status
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_layout() {
        let layout = LayoutState::new();
        assert_eq!(layout.focused_pane, Pane::Stream);
        assert!(!layout.context_rail_collapsed);
        assert!(!layout.stage_collapsed);
    }

    #[test]
    fn test_focus_cycling() {
        let mut layout = LayoutState::new();
        assert_eq!(layout.focused_pane, Pane::Stream);

        layout.focus_next();
        assert_eq!(layout.focused_pane, Pane::Stage);

        layout.focus_next();
        assert_eq!(layout.focused_pane, Pane::ContextRail);

        layout.focus_next();
        assert_eq!(layout.focused_pane, Pane::Stream);
    }

    #[test]
    fn test_focus_previous() {
        let mut layout = LayoutState::new();
        assert_eq!(layout.focused_pane, Pane::Stream);

        layout.focus_previous();
        assert_eq!(layout.focused_pane, Pane::ContextRail);

        layout.focus_previous();
        assert_eq!(layout.focused_pane, Pane::Stage);
    }

    #[test]
    fn test_collapse_moves_focus() {
        let mut layout = LayoutState::new();
        layout.focus_pane(Pane::ContextRail);
        assert_eq!(layout.focused_pane, Pane::ContextRail);

        layout.toggle_context_rail();
        assert!(layout.context_rail_collapsed);
        assert_eq!(layout.focused_pane, Pane::Stream);
    }

    #[test]
    fn test_focus_expands_collapsed_pane() {
        let mut layout = LayoutState::new();
        layout.toggle_context_rail();
        assert!(layout.context_rail_collapsed);

        layout.focus_pane(Pane::ContextRail);
        assert!(!layout.context_rail_collapsed);
        assert_eq!(layout.focused_pane, Pane::ContextRail);
    }

    #[test]
    fn test_width_constraints() {
        let mut layout = LayoutState::new();

        // Test clamping to min
        layout.set_context_rail_width(50.0);
        assert_eq!(layout.context_rail_width, CONTEXT_RAIL_CONSTRAINTS.min_width);

        // Test clamping to max
        layout.set_context_rail_width(1000.0);
        assert_eq!(layout.context_rail_width, CONTEXT_RAIL_CONSTRAINTS.max_width);
    }

    #[test]
    fn test_effective_width() {
        let mut layout = LayoutState::new();
        assert_eq!(
            layout.effective_context_rail_width(),
            CONTEXT_RAIL_CONSTRAINTS.default_width
        );

        layout.toggle_context_rail();
        assert_eq!(layout.effective_context_rail_width(), 0.0);
    }
}
