//! Tool Call Card Component
//!
//! Renders tool calls with header, parameters, status, and results.
//! Shows a compact view by default with expandable params/results.

use gpui::{div, prelude::*, px, AnyElement, Context, Window};

use super::colors;
use crate::stream::types::{ToolCallBlock, ToolCallStatus};

/// Render a tool call card
pub fn render_tool_call_card<V: 'static>(
    tool_call: &ToolCallBlock,
    on_toggle: Option<Box<dyn Fn(&mut V, &mut Window, &mut Context<V>) + Send + Sync>>,
    cx: &mut Context<V>,
) -> AnyElement {
    let tool_name = tool_call.tool_name.clone();
    let call_id = tool_call.call_id.clone();
    let status = tool_call.status;
    let parameters = tool_call.parameters.clone();
    let result = tool_call.result.clone();
    let error = tool_call.error.clone();
    let expanded = tool_call.expanded;
    let progress = tool_call.progress;
    let duration_ms = tool_call.duration_ms;

    div()
        .w_full()
        .py_1()
        .child(
            div()
                .flex()
                .flex_col()
                .w_full()
                .bg(colors::tool_header_bg())
                .rounded_md()
                .border_1()
                .border_color(status_border_color(status))
                // Header
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .px_3()
                        .py_2()
                        .cursor_pointer()
                        .hover(|el| el.bg(gpui::rgb(0x2d2d30)))
                        // Left side: tool name and status
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                // Expand icon
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors::text_secondary())
                                        .child(if expanded { "▼" } else { "▶" }),
                                )
                                // Tool icon
                                .child(
                                    div()
                                        .text_sm()
                                        .child("🔧"),
                                )
                                // Tool name
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(colors::tool_name())
                                        .child(tool_name),
                                )
                                // Status badge
                                .child(render_status_badge(status)),
                        )
                        // Right side: duration and progress
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                // Progress indicator
                                .when(status == ToolCallStatus::Running, |el| {
                                    el.when_some(progress, |el, p| {
                                        el.child(render_progress_bar(p))
                                    })
                                })
                                // Duration
                                .when_some(duration_ms, |el, ms| {
                                    el.child(
                                        div()
                                            .text_xs()
                                            .text_color(colors::text_muted())
                                            .child(format_duration(ms)),
                                    )
                                }),
                        ),
                )
                // Progress bar (full width when running)
                .when(status == ToolCallStatus::Running && progress.is_some(), |el| {
                    el.child(render_full_progress_bar(progress.unwrap_or(0)))
                })
                // Parameters (when expanded)
                .when(expanded, |el| {
                    el.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .border_t_1()
                            .border_color(colors::divider())
                            // Parameters section
                            .child(render_json_section("Parameters", &parameters))
                            // Result section (if completed)
                            .when_some(result, |el, res| {
                                el.child(render_json_section("Result", &res))
                            })
                            // Error section (if failed)
                            .when_some(error, |el, err| {
                                el.child(render_error_section(&err))
                            }),
                    )
                }),
        )
        .into_any_element()
}

/// Get border color based on status
fn status_border_color(status: ToolCallStatus) -> gpui::Rgba {
    match status {
        ToolCallStatus::Pending => colors::status_pending(),
        ToolCallStatus::Running => colors::status_running(),
        ToolCallStatus::Completed => colors::status_completed(),
        ToolCallStatus::Failed => colors::status_failed(),
        ToolCallStatus::Cancelled => colors::status_cancelled(),
    }
}

/// Render status badge
fn render_status_badge(status: ToolCallStatus) -> gpui::Div {
    let (bg, text, label) = match status {
        ToolCallStatus::Pending => (gpui::rgb(0x404040), colors::text_secondary(), "Pending"),
        ToolCallStatus::Running => (gpui::rgb(0x0e639c), gpui::rgb(0xffffff), "Running"),
        ToolCallStatus::Completed => (gpui::rgb(0x2d5a27), gpui::rgb(0x4ec9b0), "Done"),
        ToolCallStatus::Failed => (gpui::rgb(0x5a2727), gpui::rgb(0xf14c4c), "Failed"),
        ToolCallStatus::Cancelled => (gpui::rgb(0x404040), colors::text_secondary(), "Cancelled"),
    };

    div()
        .px_2()
        .py_px()
        .rounded_sm()
        .bg(bg)
        .text_xs()
        .text_color(text)
        .child(label)
}

/// Render a small progress bar
fn render_progress_bar(progress: u8) -> gpui::Div {
    let width = (progress as f32 / 100.0) * 60.0;

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .w(px(60.0))
                .h(px(4.0))
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

/// Render full-width progress bar
fn render_full_progress_bar(progress: u8) -> gpui::Div {
    // Calculate width based on progress percentage (assume 400px container)
    let width = (progress as f32 / 100.0) * 400.0;

    div()
        .w_full()
        .h(px(3.0))
        .bg(colors::progress_bg())
        .child(
            div()
                .h_full()
                .w(px(width))
                .bg(colors::progress_fill()),
        )
}

/// Render a JSON section (parameters or result)
fn render_json_section(title: &str, value: &serde_json::Value) -> gpui::Div {
    let formatted = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());

    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(colors::text_secondary())
                .child(title.to_string()),
        )
        .child(
            div()
                .bg(colors::tool_param_bg())
                .rounded_sm()
                .px_2()
                .py_1()
                .max_h(px(200.0))
                .child(
                    div()
                        .text_xs()
                        .font_family("monospace")
                        .text_color(colors::text_primary())
                        .child(formatted),
                ),
        )
}

/// Render error section
fn render_error_section(error: &str) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(colors::status_failed())
                .child("Error"),
        )
        .child(
            div()
                .bg(gpui::rgb(0x3d1f1f))
                .border_1()
                .border_color(colors::status_failed())
                .rounded_sm()
                .px_2()
                .py_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors::status_failed())
                        .child(error.to_string()),
                ),
        )
}

/// Format duration in human-readable form
fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        let mins = ms / 60000;
        let secs = (ms % 60000) / 1000;
        format!("{}m {}s", mins, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_border_colors() {
        // Just verify the function doesn't panic
        let _ = status_border_color(ToolCallStatus::Pending);
        let _ = status_border_color(ToolCallStatus::Running);
        let _ = status_border_color(ToolCallStatus::Completed);
        let _ = status_border_color(ToolCallStatus::Failed);
        let _ = status_border_color(ToolCallStatus::Cancelled);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1500), "1.5s");
        assert_eq!(format_duration(65000), "1m 5s");
    }
}
