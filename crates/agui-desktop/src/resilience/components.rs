//! UI components for resilience patterns.
//!
//! Provides connection status indicator, error overlay, and performance monitor.

use gpui::{div, prelude::*, px, rgb, Div};

use super::{AppError, ConnectionState, ErrorManager, ErrorSeverity};
use crate::metrics::{MetricsSnapshot, PerformanceMode};

/// Render connection status indicator
pub fn render_connection_status(state: ConnectionState) -> Div {
    let color = state.color();
    let label = state.label();

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        // Status dot
        .child(
            div()
                .w(px(8.0))
                .h(px(8.0))
                .rounded_full()
                .bg(rgb(color))
                // Pulsing animation for connecting states
                .when(
                    matches!(
                        state,
                        ConnectionState::Connecting | ConnectionState::Reconnecting { .. }
                    ),
                    |el| el.opacity(0.7),
                ),
        )
        // Status text
        .child(
            div()
                .text_xs()
                .text_color(rgb(color))
                .child(label),
        )
        // Retry info for reconnecting state
        .when(
            matches!(state, ConnectionState::Reconnecting { .. }),
            |el| {
                if let ConnectionState::Reconnecting {
                    attempt,
                    next_retry_secs,
                } = state
                {
                    el.child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x808080))
                            .child(format!("(attempt {}, {}s)", attempt, next_retry_secs)),
                    )
                } else {
                    el
                }
            },
        )
}

/// Render connection status for status bar (compact)
pub fn render_connection_status_compact(state: ConnectionState) -> Div {
    let color = state.color();

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .w(px(6.0))
                .h(px(6.0))
                .rounded_full()
                .bg(rgb(color)),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0xffffffcc))
                .child(state.label()),
        )
}

/// Render error overlay (shown when there are errors)
pub fn render_error_overlay(manager: &ErrorManager) -> Div {
    if !manager.has_errors() {
        return div();
    }

    div()
        .absolute()
        .top(px(40.0))
        .right(px(16.0))
        .flex()
        .flex_col()
        .gap_2()
        .max_w(px(400.0))
        .children(
            manager
                .errors()
                .iter()
                .enumerate()
                .map(|(idx, error)| render_error_toast(error, idx)),
        )
}

/// Render a single error toast
fn render_error_toast(error: &AppError, _index: usize) -> Div {
    let bg_color = match error.severity {
        ErrorSeverity::Info => 0x1e3a5f,
        ErrorSeverity::Warning => 0x5f4b1e,
        ErrorSeverity::Error => 0x5f1e1e,
        ErrorSeverity::Critical => 0x7f0000,
    };

    let border_color = error.severity.color();

    div()
        .flex()
        .flex_row()
        .items_start()
        .gap_2()
        .p_3()
        .bg(rgb(bg_color))
        .border_l_4()
        .border_color(rgb(border_color))
        .rounded_md()
        .shadow_lg()
        // Icon
        .child(
            div()
                .w(px(20.0))
                .h(px(20.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .bg(rgb(border_color))
                .text_xs()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .child(error.severity.icon()),
        )
        // Content
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xcccccc))
                        .child(error.message.clone()),
                )
                .when_some(error.retry_action.clone(), |el, action| {
                    el.child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x3794ff))
                            .cursor_pointer()
                            .hover(|el| el.underline())
                            .child(format!("Retry: {}", action)),
                    )
                }),
        )
        // Dismiss button (if dismissible)
        .when(error.dismissible, |el| {
            el.child(
                div()
                    .w(px(16.0))
                    .h(px(16.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .text_xs()
                    .text_color(rgb(0x808080))
                    .hover(|el| el.text_color(rgb(0xcccccc)))
                    .child("x"),
            )
        })
}

/// Render offline banner (shown when disconnected)
pub fn render_offline_banner(state: ConnectionState) -> Div {
    if !state.is_offline() {
        return div();
    }

    let (bg_color, message) = match state {
        ConnectionState::Connecting => (0x4a3c00, "Connecting to server..."),
        ConnectionState::Reconnecting { .. } => (0x4a3c00, "Connection lost. Reconnecting..."),
        ConnectionState::Disconnected => (0x3c3c3c, "Not connected. Working offline."),
        ConnectionState::Failed => (0x5f1e1e, "Connection failed. Please check your settings."),
        ConnectionState::Connected => return div(), // Shouldn't happen
    };

    div()
        .w_full()
        .h(px(28.0))
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap_2()
        .bg(rgb(bg_color))
        .border_b_1()
        .border_color(rgb(0x404040))
        .child(
            div()
                .text_sm()
                .text_color(rgb(0xdcdcaa))
                .child(message),
        )
        .when(
            matches!(state, ConnectionState::Reconnecting { .. }),
            |el| {
                if let ConnectionState::Reconnecting {
                    next_retry_secs, ..
                } = state
                {
                    el.child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x808080))
                            .child(format!("Retrying in {}s", next_retry_secs)),
                    )
                } else {
                    el
                }
            },
        )
}

/// Render performance indicator (for status bar)
pub fn render_performance_indicator(snapshot: &MetricsSnapshot) -> Div {
    let (color, label) = match snapshot.performance_mode {
        PerformanceMode::Normal => (0x4ec9b0, "Normal"),
        PerformanceMode::Reduced => (0xdcdcaa, "Reduced"),
        PerformanceMode::Minimal => (0xf14c4c, "Minimal"),
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0xffffffcc))
                .child(snapshot.fps_string()),
        )
        .when(!matches!(snapshot.performance_mode, PerformanceMode::Normal), |el| {
            // Use a darker background based on mode
            let bg = match snapshot.performance_mode {
                PerformanceMode::Normal => 0x1e3a2e,   // Darker green
                PerformanceMode::Reduced => 0x3a3a1e, // Darker yellow
                PerformanceMode::Minimal => 0x3a1e1e, // Darker red
            };
            el.child(
                div()
                    .px_1()
                    .rounded_sm()
                    .bg(rgb(bg))
                    .text_xs()
                    .text_color(rgb(color))
                    .child(label),
            )
        })
}

/// Render detailed performance monitor panel
pub fn render_performance_panel(snapshot: &MetricsSnapshot) -> Div {
    div()
        .flex()
        .flex_col()
        .w(px(200.0))
        .p_3()
        .bg(rgb(0x252526))
        .rounded_md()
        .border_1()
        .border_color(rgb(0x3c3c3c))
        .gap_2()
        // Header
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(0xcccccc))
                .child("Performance"),
        )
        // FPS
        .child(render_metric_row("FPS", &snapshot.fps_string()))
        // Frame time
        .child(render_metric_row(
            "Frame Time",
            &format!("{:.1}ms", snapshot.avg_frame_time_ms),
        ))
        // Event time
        .child(render_metric_row(
            "Event Time",
            &format!("{:.1}ms", snapshot.avg_event_time_ms),
        ))
        // Memory
        .child(render_metric_row("Memory", &snapshot.memory_string()))
        // Items
        .child(render_metric_row(
            "Stream Items",
            &snapshot.stream_items.to_string(),
        ))
        .child(render_metric_row(
            "Artifacts",
            &snapshot.artifact_count.to_string(),
        ))
        // Mode
        .child(
            div()
                .flex()
                .flex_row()
                .justify_between()
                .pt_2()
                .border_t_1()
                .border_color(rgb(0x3c3c3c))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x808080))
                        .child("Mode"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(match snapshot.performance_mode {
                            PerformanceMode::Normal => 0x4ec9b0,
                            PerformanceMode::Reduced => 0xdcdcaa,
                            PerformanceMode::Minimal => 0xf14c4c,
                        }))
                        .child(snapshot.mode_label()),
                ),
        )
}

/// Helper to render a metric row
fn render_metric_row(label: &str, value: &str) -> Div {
    div()
        .flex()
        .flex_row()
        .justify_between()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x808080))
                .child(label.to_string()),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0xcccccc))
                .child(value.to_string()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_status_colors() {
        let connected = ConnectionState::Connected;
        assert_eq!(connected.color(), 0x4ec9b0);

        let disconnected = ConnectionState::Disconnected;
        assert_eq!(disconnected.color(), 0x808080);
    }

    #[test]
    fn test_error_severity_colors() {
        assert_eq!(ErrorSeverity::Error.color(), 0xf14c4c);
        assert_eq!(ErrorSeverity::Warning.color(), 0xdcdcaa);
    }
}
