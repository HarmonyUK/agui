//! Tab Bar Component
//!
//! Renders the tab bar for managing multiple open artifacts.

use gpui::{div, prelude::*, px, rgb, Div};

use super::colors;
use crate::stage::types::Artifact;

/// Tab item for rendering
#[derive(Debug, Clone)]
pub struct TabItem {
    /// Artifact ID
    pub id: String,
    /// Display title
    pub title: String,
    /// Whether this tab is active
    pub active: bool,
    /// Whether this tab has unsaved changes
    pub dirty: bool,
    /// Content type label
    pub content_type: String,
    /// Read-only indicator
    pub read_only: bool,
}

impl TabItem {
    /// Create from an artifact
    pub fn from_artifact(artifact: &Artifact, is_active: bool) -> Self {
        Self {
            id: artifact.id.clone(),
            title: artifact.title.clone(),
            active: is_active,
            dirty: artifact.dirty,
            content_type: artifact.content_type.label().to_string(),
            read_only: artifact.read_only,
        }
    }
}

/// Callback type for tab actions
pub type TabCallback = Box<dyn Fn(&str)>;

/// Render the tab bar
pub fn render_tab_bar<V: 'static + gpui::Render>(
    tabs: &[TabItem],
    _cx: &mut gpui::Context<V>,
    on_select: impl Fn(&str) + 'static + Clone,
    on_close: impl Fn(&str) + 'static + Clone,
) -> Div {
    div()
        .flex()
        .flex_row()
        .w_full()
        .h(px(35.0))
        .bg(rgb(colors::TAB_BAR_BG))
        .border_b_1()
        .border_color(rgb(colors::TAB_BORDER))
        .overflow_hidden()
        .children(
            tabs.iter().map(|tab| {
                let tab_id = tab.id.clone();
                let tab_id_close = tab.id.clone();
                let on_select = on_select.clone();
                let on_close = on_close.clone();

                render_tab(tab)
                    .on_mouse_down(gpui::MouseButton::Left, move |_, _, _| {
                        on_select(&tab_id);
                    })
                    .child(
                        // Close button
                        div()
                            .ml_2()
                            .w(px(16.0))
                            .h(px(16.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|el| el.bg(rgb(0x404040)))
                            .text_xs()
                            .text_color(rgb(colors::CLOSE_BUTTON))
                            .child("×")
                            .on_mouse_down(gpui::MouseButton::Left, move |_, _, _| {
                                on_close(&tab_id_close);
                            })
                    )
            })
        )
}

/// Render a single tab (without close button - that's added by render_tab_bar)
fn render_tab(tab: &TabItem) -> Div {
    let bg_color = if tab.active {
        colors::TAB_ACTIVE_BG
    } else {
        colors::TAB_INACTIVE_BG
    };

    let text_color = if tab.active {
        colors::TAB_TEXT_ACTIVE
    } else {
        colors::TAB_TEXT
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .h_full()
        .px_3()
        .bg(rgb(bg_color))
        .border_r_1()
        .border_color(rgb(colors::TAB_BORDER))
        .cursor_pointer()
        .when(!tab.active, |el| el.hover(|el| el.bg(rgb(0x2a2a2a))))
        // Active tab top border indicator
        .when(tab.active, |el| {
            el.border_t_2().border_color(rgb(0x007acc))
        })
        // Dirty indicator
        .when(tab.dirty, |el| {
            el.child(
                div()
                    .w(px(8.0))
                    .h(px(8.0))
                    .rounded_full()
                    .bg(rgb(colors::DIRTY_DOT))
                    .mr_2()
            )
        })
        // Read-only indicator
        .when(tab.read_only, |el| {
            el.child(
                div()
                    .text_xs()
                    .text_color(rgb(0x808080))
                    .mr_1()
                    .child("🔒")
            )
        })
        // Title
        .child(
            div()
                .text_sm()
                .text_color(rgb(text_color))
                .max_w(px(150.0))
                .truncate()
                .child(tab.title.clone())
        )
}

/// Render a simplified tab bar (for when callbacks aren't needed)
pub fn render_tab_bar_simple(tabs: &[TabItem]) -> Div {
    div()
        .flex()
        .flex_row()
        .w_full()
        .h(px(35.0))
        .bg(rgb(colors::TAB_BAR_BG))
        .border_b_1()
        .border_color(rgb(colors::TAB_BORDER))
        .overflow_hidden()
        .children(tabs.iter().map(|tab| render_tab_simple(tab)))
}

/// Render a single tab (simple version without interactivity)
fn render_tab_simple(tab: &TabItem) -> Div {
    let bg_color = if tab.active {
        colors::TAB_ACTIVE_BG
    } else {
        colors::TAB_INACTIVE_BG
    };

    let text_color = if tab.active {
        colors::TAB_TEXT_ACTIVE
    } else {
        colors::TAB_TEXT
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .h_full()
        .px_3()
        .bg(rgb(bg_color))
        .border_r_1()
        .border_color(rgb(colors::TAB_BORDER))
        .when(tab.active, |el| {
            el.border_t_2().border_color(rgb(0x007acc))
        })
        .when(tab.dirty, |el| {
            el.child(
                div()
                    .w(px(8.0))
                    .h(px(8.0))
                    .rounded_full()
                    .bg(rgb(colors::DIRTY_DOT))
                    .mr_2()
            )
        })
        .when(tab.read_only, |el| {
            el.child(
                div()
                    .text_xs()
                    .text_color(rgb(0x808080))
                    .mr_1()
                    .child("🔒")
            )
        })
        .child(
            div()
                .text_sm()
                .text_color(rgb(text_color))
                .max_w(px(150.0))
                .truncate()
                .child(tab.title.clone())
        )
        // Close button (non-interactive in simple version)
        .child(
            div()
                .ml_2()
                .w(px(16.0))
                .h(px(16.0))
                .flex()
                .items_center()
                .justify_center()
                .text_xs()
                .text_color(rgb(colors::CLOSE_BUTTON))
                .child("×")
        )
}

/// Render empty state when no tabs are open
pub fn render_empty_tabs() -> Div {
    div()
        .flex()
        .flex_col()
        .w_full()
        .h_full()
        .items_center()
        .justify_center()
        .gap_3()
        .child(
            div()
                .text_3xl()
                .text_color(rgb(0x404040))
                .child("📄")
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x808080))
                .child("No artifacts open")
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x606060))
                .child("Open files will appear here")
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stage::types::Artifact;

    #[test]
    fn test_tab_from_artifact() {
        let artifact = Artifact::from_open(
            "test-1",
            "test.rs",
            "fn main() {}",
            "code",
            false,
            Some("rust".to_string()),
        );

        let tab = TabItem::from_artifact(&artifact, true);

        assert_eq!(tab.id, "test-1");
        assert_eq!(tab.title, "test.rs");
        assert!(tab.active);
        assert!(!tab.dirty);
        assert!(!tab.read_only);
    }

    #[test]
    fn test_tab_dirty_indicator() {
        let mut artifact = Artifact::from_open("test", "test.txt", "hello", "text", false, None);
        artifact.dirty = true;

        let tab = TabItem::from_artifact(&artifact, false);
        assert!(tab.dirty);
    }

    #[test]
    fn test_tab_read_only() {
        let artifact = Artifact::from_open("test", "test.txt", "hello", "text", true, None);

        let tab = TabItem::from_artifact(&artifact, false);
        assert!(tab.read_only);
    }
}
