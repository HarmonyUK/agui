//! Status Update Component
//!
//! Renders progress and status messages in a compact format.

use gpui::{div, prelude::*, px, AnyElement, Context};

use super::colors;
use crate::stream::types::{StatusBlock, StatusType};

/// Render a status update
pub fn render_status_update<V: 'static>(
    status: &StatusBlock,
    cx: &mut Context<V>,
) -> AnyElement {
    let message = status.message.clone();
    let status_type = status.status_type;
    let progress = status.progress;
    let ephemeral = status.ephemeral;

    div()
        .w_full()
        .py_1()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap_2()
                .px_4()
                .py_1()
                .when(ephemeral, |el| {
                    el.opacity(0.8)
                })
                // Status icon
                .child(render_status_icon(status_type))
                // Message
                .child(
                    div()
                        .text_xs()
                        .text_color(status_text_color(status_type))
                        .child(message),
                )
                // Progress bar (if applicable)
                .when_some(progress, |el, p| {
                    el.child(render_mini_progress(p))
                }),
        )
        .into_any_element()
}

/// Render status icon
fn render_status_icon(status_type: StatusType) -> gpui::Div {
    let (icon, color) = match status_type {
        StatusType::Info => ("ℹ", colors::text_secondary()),
        StatusType::Success => ("✓", colors::status_completed()),
        StatusType::Warning => ("⚠", gpui::rgb(0xdcdcaa)),
        StatusType::Error => ("✗", colors::status_failed()),
        StatusType::Progress => ("◌", colors::status_running()),
    };

    div()
        .text_xs()
        .text_color(color)
        .child(icon)
}

/// Get text color for status type
fn status_text_color(status_type: StatusType) -> gpui::Rgba {
    match status_type {
        StatusType::Info => colors::text_secondary(),
        StatusType::Success => colors::status_completed(),
        StatusType::Warning => gpui::rgb(0xdcdcaa),
        StatusType::Error => colors::status_failed(),
        StatusType::Progress => colors::text_secondary(),
    }
}

/// Render mini progress bar
fn render_mini_progress(progress: u8) -> gpui::Div {
    let width = (progress as f32 / 100.0) * 40.0;

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .w(px(40.0))
                .h(px(3.0))
                .bg(colors::progress_bg())
                .rounded_sm()
                .child(
                    div()
                        .h_full()
                        .w(px(width))
                        .bg(colors::progress_fill())
                        .rounded_sm(),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors::text_muted())
                .child(format!("{}%", progress)),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_block_creation() {
        let info = StatusBlock::info("Processing...");
        assert_eq!(info.status_type, StatusType::Info);
        assert!(!info.ephemeral);

        let progress = StatusBlock::progress("Loading", 50);
        assert_eq!(progress.progress, Some(50));
        assert!(progress.ephemeral);
    }
}
