//! Stream Timeline Component
//!
//! The main virtualized timeline view that renders stream items
//! efficiently with virtual scrolling for 1000+ items.

use gpui::{div, prelude::*, px, AnyElement, Context};

use super::{colors, stream_item::render_stream_item};
use crate::stream::{
    state::StreamState,
    types::StreamItem,
    virtual_list::{VirtualList, VirtualListConfig},
};

/// Stream timeline view configuration
#[derive(Debug, Clone)]
pub struct StreamTimelineConfig {
    /// Enable virtual scrolling
    pub virtualized: bool,
    /// Show scroll-to-bottom button
    pub show_scroll_button: bool,
    /// Auto-scroll on new messages
    pub auto_scroll: bool,
    /// Item padding
    pub item_padding: f32,
}

impl Default for StreamTimelineConfig {
    fn default() -> Self {
        Self {
            virtualized: true,
            show_scroll_button: true,
            auto_scroll: true,
            item_padding: 4.0,
        }
    }
}

/// Stream timeline component state
pub struct StreamTimeline {
    /// Stream state (items, selection, etc.)
    pub state: StreamState,
    /// Virtual list for efficient rendering
    pub virtual_list: VirtualList,
    /// Configuration
    pub config: StreamTimelineConfig,
    /// Currently hovered item
    pub hovered_item: Option<String>,
}

impl StreamTimeline {
    /// Create a new stream timeline
    pub fn new() -> Self {
        Self {
            state: StreamState::new(),
            virtual_list: VirtualList::new(VirtualListConfig::default()),
            config: StreamTimelineConfig::default(),
            hovered_item: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: StreamTimelineConfig) -> Self {
        Self {
            state: StreamState::new(),
            virtual_list: VirtualList::new(VirtualListConfig::default()),
            config,
            hovered_item: None,
        }
    }

    /// Add an item to the timeline
    pub fn push(&mut self, item: StreamItem) {
        // Update virtual list
        self.virtual_list.set_item_height(
            self.state.len(),
            item.estimated_height(),
        );

        // Add to state
        self.state.push(item);
        self.virtual_list.set_item_count(self.state.len());

        // Auto-scroll if enabled
        if self.config.auto_scroll && self.state.is_auto_scroll() {
            self.virtual_list.scroll_to_bottom();
        }
    }

    /// Set viewport height (call on resize)
    pub fn set_viewport_height(&mut self, height: f32) {
        self.state.set_viewport_height(height);
        self.virtual_list.set_viewport_height(height);
    }

    /// Handle scroll event
    pub fn scroll_by(&mut self, delta: f32) {
        self.virtual_list.scroll_by(delta);
        self.state.set_scroll_offset(self.virtual_list.scroll_offset());
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.virtual_list.scroll_to_bottom();
        self.state.enable_auto_scroll();
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.state.clear();
        self.virtual_list.set_item_count(0);
    }
}

impl Default for StreamTimeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the stream timeline
pub fn render_stream_timeline<V: 'static>(
    timeline: &StreamTimeline,
    cx: &mut Context<V>,
) -> AnyElement {
    let items = timeline.state.items();
    let selected_id = timeline.state.selected().cloned();
    let is_at_bottom = timeline.virtual_list.is_at_bottom();
    let show_scroll_button = timeline.config.show_scroll_button && !is_at_bottom;

    if items.is_empty() {
        // Empty state
        return render_empty_state(cx);
    }

    // Get visible range for virtualization
    let visible_range = timeline.virtual_list.visible_range();
    let scroll_offset = timeline.virtual_list.scroll_offset();
    let total_height = timeline.virtual_list.total_height();

    div()
        .flex()
        .flex_col()
        .w_full()
        .h_full()
        .relative()
        // Scrollable container
        .child(
            div()
                .flex()
                .flex_col()
                .w_full()
                .h_full()
                // Virtual scroll container
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .w_full()
                        .h(timeline.virtual_list.total_height_px())
                        .relative()
                        // Render only visible items
                        .children(
                            visible_range
                                .iter()
                                .filter_map(|idx| items.get(idx).map(|item| (idx, item)))
                                .map(|(idx, item)| {
                                    let offset = timeline.virtual_list.item_offset(idx);
                                    let selected = selected_id.as_ref() == Some(&item.id);

                                    div()
                                        .absolute()
                                        .top(px(offset))
                                        .left_0()
                                        .w_full()
                                        .child(render_stream_item::<V>(item, selected, None, cx))
                                })
                                .collect::<Vec<_>>(),
                        ),
                ),
        )
        // Scroll to bottom button
        .when(show_scroll_button, |el| {
            el.child(
                div()
                    .absolute()
                    .bottom(px(20.0))
                    .right(px(20.0))
                    .w(px(40.0))
                    .h(px(40.0))
                    .rounded_full()
                    .bg(colors::status_running())
                    .shadow_lg()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|el| el.bg(gpui::rgb(0x0098ff)))
                    .child(
                        div()
                            .text_lg()
                            .text_color(gpui::rgb(0xffffff))
                            .child("↓"),
                    ),
            )
        })
        .into_any_element()
}

/// Render empty state
fn render_empty_state<V: 'static>(cx: &mut Context<V>) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .h_full()
        .items_center()
        .justify_center()
        .gap_2()
        .child(
            div()
                .text_2xl()
                .text_color(colors::text_muted())
                .child("💬"),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors::text_secondary())
                .child("No messages yet"),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors::text_muted())
                .child("Start a conversation to see messages here"),
        )
        .into_any_element()
}

/// Render non-virtualized timeline (for small lists)
pub fn render_simple_timeline<V: 'static>(
    items: &[StreamItem],
    selected_id: Option<&str>,
    cx: &mut Context<V>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .h_full()
        .py_2()
        .children(
            items
                .iter()
                .map(|item| {
                    let selected = selected_id == Some(&item.id);
                    render_stream_item::<V>(item, selected, None, cx)
                })
                .collect::<Vec<_>>(),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::types::{StreamContent, UserMessage};

    fn make_item(id: &str) -> StreamItem {
        StreamItem::new(
            id,
            StreamContent::UserMessage(UserMessage::new("Test message")),
        )
    }

    #[test]
    fn test_timeline_creation() {
        let timeline = StreamTimeline::new();
        assert!(timeline.state.is_empty());
        assert!(timeline.config.virtualized);
    }

    #[test]
    fn test_timeline_push() {
        let mut timeline = StreamTimeline::new();
        timeline.push(make_item("1"));
        timeline.push(make_item("2"));

        assert_eq!(timeline.state.len(), 2);
    }

    #[test]
    fn test_timeline_clear() {
        let mut timeline = StreamTimeline::new();
        timeline.push(make_item("1"));
        timeline.push(make_item("2"));
        timeline.clear();

        assert!(timeline.state.is_empty());
    }

    #[test]
    fn test_viewport_height() {
        let mut timeline = StreamTimeline::new();
        timeline.set_viewport_height(500.0);

        assert_eq!(timeline.virtual_list.viewport_height(), 500.0);
    }
}
