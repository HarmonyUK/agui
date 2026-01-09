//! Reasoning Accordion Component
//!
//! Renders Chain-of-Thought reasoning in a collapsible accordion.
//! Collapsed by default to reduce visual noise, with a summary showing.

use gpui::{div, prelude::*, px, AnyElement, Context, Window};

use super::colors;
use crate::stream::types::ReasoningBlock;

/// Callback type for toggle events
pub type OnToggle<V> = Box<dyn Fn(&mut V, &mut Window, &mut Context<V>, bool) + Send + Sync>;

/// Render a reasoning accordion
pub fn render_reasoning_accordion<V: 'static>(
    reasoning: &ReasoningBlock,
    on_toggle: Option<OnToggle<V>>,
    cx: &mut Context<V>,
) -> AnyElement {
    let content = reasoning.content.clone();
    let summary = reasoning
        .summary
        .clone()
        .unwrap_or_else(|| truncate_content(&content, 50));
    let expanded = reasoning.expanded;
    let duration_ms = reasoning.duration_ms;

    div()
        .w_full()
        .py_1()
        .child(
            div()
                .flex()
                .flex_col()
                .w_full()
                .bg(colors::reasoning_bg())
                .border_1()
                .border_color(colors::reasoning_border())
                .rounded_md()
                // Header (always visible)
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
                        .on_mouse_down(gpui::MouseButton::Left, {
                            let content_clone = content.clone();
                            move |_, _, _| {
                                // Toggle would be handled by parent
                            }
                        })
                        // Left side: icon and summary
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                // Expand/collapse icon
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors::reasoning_header())
                                        .child(if expanded { "▼" } else { "▶" }),
                                )
                                // "Thinking" label
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(colors::reasoning_header())
                                        .child("Thinking"),
                                )
                                // Summary (when collapsed)
                                .when(!expanded, |el| {
                                    el.child(
                                        div()
                                            .text_xs()
                                            .text_color(colors::reasoning_text())
                                            .child(format!("— {}", summary)),
                                    )
                                }),
                        )
                        // Right side: duration
                        .when_some(duration_ms, |el, ms| {
                            el.child(
                                div()
                                    .text_xs()
                                    .text_color(colors::text_muted())
                                    .child(format_duration(ms)),
                            )
                        }),
                )
                // Content (when expanded)
                .when(expanded, |el| {
                    el.child(
                        div()
                            .px_3()
                            .py_2()
                            .border_t_1()
                            .border_color(colors::reasoning_border())
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(colors::reasoning_text())
                                    .font_family("monospace")
                                    .max_h(px(300.0))
                                    .child(content),
                            ),
                    )
                }),
        )
        .into_any_element()
}

/// Truncate content for summary display
fn truncate_content(content: &str, max_len: usize) -> String {
    let first_line = content.lines().next().unwrap_or(content);
    if first_line.len() > max_len {
        format!("{}...", &first_line[..max_len])
    } else {
        first_line.to_string()
    }
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
    fn test_truncate_content() {
        assert_eq!(truncate_content("short", 50), "short");
        assert_eq!(
            truncate_content("This is a much longer string that should be truncated", 20),
            "This is a much longe..."
        );
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1500), "1.5s");
        assert_eq!(format_duration(65000), "1m 5s");
    }

    #[test]
    fn test_reasoning_block_creation() {
        let block = ReasoningBlock::new("Thinking about the problem...")
            .with_summary("Analyzing");
        assert_eq!(block.summary, Some("Analyzing".to_string()));
        assert!(!block.expanded);
    }
}
