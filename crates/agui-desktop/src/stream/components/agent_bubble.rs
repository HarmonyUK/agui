//! Agent Message Bubble Component
//!
//! Renders agent messages as left-aligned bubbles with markdown support
//! and optional confidence indicator.

use gpui::{div, prelude::*, px, AnyElement, Context, Window};

use super::colors;
use crate::stream::types::AgentMessage;

/// Render callback type
pub type OnAgentBubbleClick<V> = Option<Box<dyn Fn(&mut V, &mut Window, &mut Context<V>) + Send + Sync>>;

/// Render an agent message bubble
///
/// Note: Full markdown rendering would require a markdown parser.
/// This implementation provides basic text rendering with styling.
/// A real implementation would integrate pulldown-cmark or similar.
pub fn render_agent_bubble<V: 'static>(
    message: &AgentMessage,
    selected: bool,
    on_click: OnAgentBubbleClick<V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let content = message.content.clone();
    let agent_name = message.agent_name.clone().unwrap_or_else(|| message.agent_id.clone());
    let is_streaming = message.streaming;
    let confidence = message.confidence;

    div()
        .w_full()
        .flex()
        .flex_row()
        .justify_start() // Left-align the bubble
        .py_2()
        .child(
            div()
                .flex()
                .flex_col()
                .max_w(px(600.0))
                .gap_1()
                // Agent header with name and status
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors::text_secondary())
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(agent_name),
                        )
                        // Streaming indicator
                        .when(is_streaming, |el| {
                            el.child(
                                div()
                                    .text_xs()
                                    .text_color(colors::status_running())
                                    .child("typing..."),
                            )
                        })
                        // Confidence indicator
                        .when_some(confidence, |el, conf| {
                            el.child(render_confidence_indicator(conf))
                        }),
                )
                // Message bubble
                .child(
                    div()
                        .bg(colors::agent_bubble_bg())
                        .rounded_lg()
                        .rounded_bl_none() // Speech bubble tail
                        .px_4()
                        .py_3()
                        .text_color(colors::agent_bubble_text())
                        .text_sm()
                        .when(selected, |el| {
                            el.shadow_md()
                                .border_2()
                                .border_color(colors::status_running())
                        })
                        // Render markdown content (simplified for now)
                        .child(render_markdown_content(&content)),
                ),
        )
        .when(on_click.is_some(), |el| {
            el.cursor_pointer()
        })
        .into_any_element()
}

/// Render markdown content (simplified implementation)
///
/// A full implementation would use a markdown parser like pulldown-cmark
/// and render each block appropriately. This is a basic version that
/// handles plain text and code blocks.
fn render_markdown_content(content: &str) -> gpui::Div {
    let mut container = div().flex().flex_col().gap_2();

    // Simple parsing: split by code blocks
    let parts: Vec<&str> = content.split("```").collect();

    for (i, part) in parts.iter().enumerate() {
        let is_code_block = i % 2 == 1;

        if part.trim().is_empty() {
            continue;
        }

        if is_code_block {
            // Code block
            let (lang, code) = if let Some((first_line, rest)) = part.split_once('\n') {
                let lang = first_line.trim();
                (
                    if lang.is_empty() { None } else { Some(lang) },
                    rest.trim(),
                )
            } else {
                (None, part.trim())
            };

            container = container.child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    // Language label
                    .when_some(lang, |el, l| {
                        el.child(
                            div()
                                .bg(gpui::rgb(0x1e1e1e))
                                .px_2()
                                .py_1()
                                .text_xs()
                                .text_color(colors::text_muted())
                                .rounded_t_md()
                                .child(l.to_string()),
                        )
                    })
                    // Code content
                    .child(
                        div()
                            .bg(gpui::rgb(0x1e1e1e))
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .when(lang.is_some(), |el| el.rounded_tl_none())
                            .text_xs()
                            .font_family("monospace")
                            .text_color(gpui::rgb(0xd4d4d4))
                            .child(code.to_string()),
                    ),
            );
        } else {
            // Regular text - split by paragraphs
            for paragraph in part.split("\n\n") {
                let trimmed = paragraph.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Check for headers
                if trimmed.starts_with("# ") {
                    container = container.child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(colors::text_primary())
                            .child(trimmed[2..].to_string()),
                    );
                } else if trimmed.starts_with("## ") {
                    container = container.child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(colors::text_primary())
                            .child(trimmed[3..].to_string()),
                    );
                } else if trimmed.starts_with("### ") {
                    container = container.child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(colors::text_primary())
                            .child(trimmed[4..].to_string()),
                    );
                } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                    // List items
                    for line in trimmed.lines() {
                        let line = line.trim();
                        if line.starts_with("- ") || line.starts_with("* ") {
                            container = container.child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_2()
                                    .child(div().text_color(colors::text_secondary()).child("•"))
                                    .child(
                                        div()
                                            .text_color(colors::text_primary())
                                            .child(line[2..].to_string()),
                                    ),
                            );
                        }
                    }
                } else {
                    // Regular paragraph
                    container = container.child(
                        div()
                            .text_color(colors::text_primary())
                            .child(trimmed.to_string()),
                    );
                }
            }
        }
    }

    container
}

/// Render confidence indicator (colored ring/dot)
fn render_confidence_indicator(confidence: f32) -> gpui::Div {
    let color = if confidence >= 0.8 {
        colors::status_completed() // High confidence - green
    } else if confidence >= 0.5 {
        gpui::rgb(0xdcdcaa) // Medium confidence - yellow
    } else {
        colors::status_failed() // Low confidence - red
    };

    let percentage = (confidence * 100.0) as u32;

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .size(px(8.0))
                .rounded_full()
                .bg(color),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors::text_muted())
                .child(format!("{}%", percentage)),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_message_creation() {
        let msg = AgentMessage::new("claude", "Hello, I can help!");
        assert_eq!(msg.agent_id, "claude");
        assert_eq!(msg.content, "Hello, I can help!");
        assert!(!msg.streaming);
    }

    #[test]
    fn test_agent_message_streaming() {
        let msg = AgentMessage::new("claude", "Thinking...")
            .with_name("Claude")
            .streaming();
        assert!(msg.streaming);
        assert_eq!(msg.agent_name, Some("Claude".to_string()));
    }
}
