//! User Message Bubble Component
//!
//! Renders user messages as right-aligned bubbles with distinctive styling.

use gpui::{div, prelude::*, px, AnyElement, Context, Window};

use super::colors;
use crate::stream::types::UserMessage;

/// Render callback type
pub type OnUserBubbleClick<V> = Option<Box<dyn Fn(&mut V, &mut Window, &mut Context<V>) + Send + Sync>>;

/// Render a user message bubble
pub fn render_user_bubble<V: 'static>(
    message: &UserMessage,
    selected: bool,
    on_click: OnUserBubbleClick<V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let content = message.content.clone();
    let sender_name = message.sender_name.clone();

    // Calculate approximate line count for height estimation
    let _line_count = (content.len() / 60).max(1);

    div()
        .w_full()
        .flex()
        .flex_row()
        .justify_end() // Right-align the bubble
        .py_2()
        .child(
            div()
                .flex()
                .flex_col()
                .max_w(px(500.0))
                .gap_1()
                // Sender name (if provided)
                .when_some(sender_name, |el, name| {
                    el.child(
                        div()
                            .text_xs()
                            .text_color(colors::text_secondary())
                            .text_right()
                            .child(name),
                    )
                })
                // Message bubble
                .child(
                    div()
                        .bg(colors::user_bubble_bg())
                        .rounded_lg()
                        .rounded_br_none() // Speech bubble tail
                        .px_4()
                        .py_2()
                        .text_color(colors::user_bubble_text())
                        .text_sm()
                        .when(selected, |el| {
                            el.shadow_md()
                                .border_2()
                                .border_color(gpui::rgb(0xffffff))
                        })
                        .child(content),
                ),
        )
        .when_some(on_click, |el, callback| {
            el.cursor_pointer()
                .on_mouse_down(gpui::MouseButton::Left, move |_, _, _| {
                    // Click handling would go here
                })
        })
        .into_any_element()
}

/// User bubble component for use in the stream timeline
pub struct UserBubbleView {
    pub message: UserMessage,
    pub selected: bool,
}

impl UserBubbleView {
    pub fn new(message: UserMessage) -> Self {
        Self {
            message,
            selected: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message_creation() {
        let msg = UserMessage::new("Hello, world!");
        assert_eq!(msg.content, "Hello, world!");
        assert!(msg.sender_name.is_none());
    }

    #[test]
    fn test_user_message_with_sender() {
        let msg = UserMessage::new("Hello!").with_sender("Alice");
        assert_eq!(msg.sender_name, Some("Alice".to_string()));
    }
}
