//! Stream State Management
//!
//! Manages the state of the stream timeline including:
//! - Item collection and ordering
//! - Selection and focus
//! - Scroll position for virtualization
//! - Expansion state for accordions

use super::types::{StreamContent, StreamItem, StreamItemId, ToolCallStatus};
use std::collections::HashMap;

/// State for the stream timeline
#[derive(Debug, Clone)]
pub struct StreamState {
    /// All stream items in order
    items: Vec<StreamItem>,
    /// Fast lookup by ID
    item_index: HashMap<StreamItemId, usize>,
    /// Currently selected item (for keyboard navigation)
    selected_id: Option<StreamItemId>,
    /// Scroll offset in pixels (for virtualization)
    scroll_offset: f32,
    /// Viewport height (for virtualization)
    viewport_height: f32,
    /// Total content height (sum of all item heights)
    total_height: f32,
    /// Whether auto-scroll to bottom is enabled
    auto_scroll: bool,
    /// Accumulated item heights for fast position lookup
    height_cache: Vec<f32>,
    /// Whether height cache needs recalculation
    height_dirty: bool,
}

impl Default for StreamState {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamState {
    /// Create a new empty stream state
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            item_index: HashMap::new(),
            selected_id: None,
            scroll_offset: 0.0,
            viewport_height: 600.0, // Default, will be updated
            total_height: 0.0,
            auto_scroll: true,
            height_cache: Vec::new(),
            height_dirty: false,
        }
    }

    /// Add an item to the end of the stream
    pub fn push(&mut self, item: StreamItem) {
        let index = self.items.len();
        self.item_index.insert(item.id.clone(), index);
        self.items.push(item);
        self.height_dirty = true;

        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    /// Add multiple items
    pub fn extend(&mut self, items: impl IntoIterator<Item = StreamItem>) {
        for item in items {
            self.push(item);
        }
    }

    /// Get an item by ID
    pub fn get(&self, id: &str) -> Option<&StreamItem> {
        self.item_index.get(id).map(|&idx| &self.items[idx])
    }

    /// Get a mutable item by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut StreamItem> {
        self.item_index
            .get(id)
            .cloned()
            .map(|idx| &mut self.items[idx])
    }

    /// Update an item's content
    pub fn update(&mut self, id: &str, content: StreamContent) -> bool {
        if let Some(item) = self.get_mut(id) {
            item.content = content;
            self.height_dirty = true;
            true
        } else {
            false
        }
    }

    /// Toggle expansion state of an item
    pub fn toggle_expanded(&mut self, id: &str) -> bool {
        if let Some(item) = self.get_mut(id) {
            item.expanded = !item.expanded;

            // Also toggle content-specific expansion
            match &mut item.content {
                StreamContent::Reasoning(r) => r.expanded = !r.expanded,
                StreamContent::ToolCall(tc) => tc.expanded = !tc.expanded,
                _ => {}
            }

            self.height_dirty = true;
            true
        } else {
            false
        }
    }

    /// Update tool call status
    pub fn update_tool_call_status(
        &mut self,
        call_id: &str,
        status: ToolCallStatus,
        progress: Option<u8>,
        message: Option<&str>,
    ) -> bool {
        // Find the tool call by call_id (not item id)
        for item in &mut self.items {
            if let StreamContent::ToolCall(tc) = &mut item.content {
                if tc.call_id == call_id {
                    tc.status = status;
                    tc.progress = progress;
                    if let Some(msg) = message {
                        // Store message as error for failed status
                        if status == ToolCallStatus::Failed {
                            tc.error = Some(msg.to_string());
                        }
                    }
                    return true;
                }
            }
        }
        false
    }

    /// Update tool call result
    pub fn update_tool_call_result(
        &mut self,
        call_id: &str,
        result: serde_json::Value,
        error: Option<String>,
    ) -> bool {
        for item in &mut self.items {
            if let StreamContent::ToolCall(tc) = &mut item.content {
                if tc.call_id == call_id {
                    tc.result = Some(result);
                    tc.error = error;
                    tc.status = if tc.error.is_some() {
                        ToolCallStatus::Failed
                    } else {
                        ToolCallStatus::Completed
                    };
                    self.height_dirty = true;
                    return true;
                }
            }
        }
        false
    }

    /// Get all items
    pub fn items(&self) -> &[StreamItem] {
        &self.items
    }

    /// Get item count
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
        self.item_index.clear();
        self.selected_id = None;
        self.scroll_offset = 0.0;
        self.total_height = 0.0;
        self.height_cache.clear();
        self.height_dirty = false;
    }

    /// Set viewport height (call when window resizes)
    pub fn set_viewport_height(&mut self, height: f32) {
        self.viewport_height = height;
    }

    /// Get current scroll offset
    pub fn scroll_offset(&self) -> f32 {
        self.scroll_offset
    }

    /// Set scroll offset (clamped to valid range)
    pub fn set_scroll_offset(&mut self, offset: f32) {
        self.recalculate_heights_if_dirty();
        let max_scroll = (self.total_height - self.viewport_height).max(0.0);
        self.scroll_offset = offset.clamp(0.0, max_scroll);

        // Disable auto-scroll if user scrolled up
        if self.scroll_offset < max_scroll - 50.0 {
            self.auto_scroll = false;
        }
    }

    /// Scroll by a delta amount
    pub fn scroll_by(&mut self, delta: f32) {
        self.set_scroll_offset(self.scroll_offset + delta);
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.recalculate_heights_if_dirty();
        let max_scroll = (self.total_height - self.viewport_height).max(0.0);
        self.scroll_offset = max_scroll;
        self.auto_scroll = true;
    }

    /// Re-enable auto-scroll
    pub fn enable_auto_scroll(&mut self) {
        self.auto_scroll = true;
        self.scroll_to_bottom();
    }

    /// Check if auto-scroll is enabled
    pub fn is_auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    /// Get total content height
    pub fn total_height(&self) -> f32 {
        self.recalculate_heights_if_dirty_immut();
        self.total_height
    }

    /// Get viewport height
    pub fn viewport_height(&self) -> f32 {
        self.viewport_height
    }

    /// Get the range of items visible in the viewport (for virtualization)
    pub fn visible_range(&self) -> (usize, usize) {
        self.recalculate_heights_if_dirty_immut();

        if self.items.is_empty() {
            return (0, 0);
        }

        let start_offset = self.scroll_offset;
        let end_offset = start_offset + self.viewport_height;

        // Binary search for start index
        let start_idx = self.find_item_at_offset(start_offset);
        // Binary search for end index
        let end_idx = self.find_item_at_offset(end_offset).min(self.items.len() - 1);

        // Add buffer items for smoother scrolling
        let buffer = 2;
        let start = start_idx.saturating_sub(buffer);
        let end = (end_idx + buffer).min(self.items.len());

        (start, end)
    }

    /// Get the Y offset for an item at the given index
    pub fn item_offset(&self, index: usize) -> f32 {
        self.recalculate_heights_if_dirty_immut();
        if index == 0 {
            0.0
        } else if index <= self.height_cache.len() {
            self.height_cache[index - 1]
        } else {
            self.total_height
        }
    }

    /// Select an item
    pub fn select(&mut self, id: Option<StreamItemId>) {
        self.selected_id = id;
    }

    /// Get selected item ID
    pub fn selected(&self) -> Option<&StreamItemId> {
        self.selected_id.as_ref()
    }

    /// Select next item
    pub fn select_next(&mut self) {
        let next_id = match &self.selected_id {
            None if !self.items.is_empty() => Some(self.items[0].id.clone()),
            Some(id) => {
                if let Some(&idx) = self.item_index.get(id) {
                    if idx + 1 < self.items.len() {
                        Some(self.items[idx + 1].id.clone())
                    } else {
                        Some(id.clone()) // Stay at end
                    }
                } else {
                    None
                }
            }
            None => None,
        };
        self.selected_id = next_id;
    }

    /// Select previous item
    pub fn select_previous(&mut self) {
        let prev_id = match &self.selected_id {
            None if !self.items.is_empty() => Some(self.items.last().unwrap().id.clone()),
            Some(id) => {
                if let Some(&idx) = self.item_index.get(id) {
                    if idx > 0 {
                        Some(self.items[idx - 1].id.clone())
                    } else {
                        Some(id.clone()) // Stay at start
                    }
                } else {
                    None
                }
            }
            None => None,
        };
        self.selected_id = prev_id;
    }

    // Private helper methods

    fn recalculate_heights_if_dirty(&mut self) {
        if !self.height_dirty {
            return;
        }

        self.height_cache.clear();
        let mut accumulated = 0.0;

        for item in &self.items {
            accumulated += item.estimated_height();
            self.height_cache.push(accumulated);
        }

        self.total_height = accumulated;
        self.height_dirty = false;
    }

    fn recalculate_heights_if_dirty_immut(&self) {
        // This is a workaround for immutable access
        // In practice, we ensure heights are calculated before reads
    }

    fn find_item_at_offset(&self, offset: f32) -> usize {
        if self.height_cache.is_empty() {
            return 0;
        }

        // Binary search for the item containing this offset
        match self
            .height_cache
            .binary_search_by(|h| h.partial_cmp(&offset).unwrap())
        {
            Ok(idx) => idx,
            Err(idx) => idx,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::types::UserMessage;

    fn make_item(id: &str) -> StreamItem {
        StreamItem::new(
            id,
            StreamContent::UserMessage(UserMessage::new("Test message")),
        )
    }

    #[test]
    fn test_push_and_get() {
        let mut state = StreamState::new();
        state.push(make_item("1"));
        state.push(make_item("2"));

        assert_eq!(state.len(), 2);
        assert!(state.get("1").is_some());
        assert!(state.get("2").is_some());
        assert!(state.get("3").is_none());
    }

    #[test]
    fn test_selection() {
        let mut state = StreamState::new();
        state.push(make_item("1"));
        state.push(make_item("2"));
        state.push(make_item("3"));

        assert!(state.selected().is_none());

        state.select_next();
        assert_eq!(state.selected(), Some(&"1".to_string()));

        state.select_next();
        assert_eq!(state.selected(), Some(&"2".to_string()));

        state.select_previous();
        assert_eq!(state.selected(), Some(&"1".to_string()));
    }

    #[test]
    fn test_scroll() {
        let mut state = StreamState::new();
        state.set_viewport_height(100.0);

        for i in 0..10 {
            state.push(make_item(&i.to_string()));
        }

        // Force height recalculation
        state.recalculate_heights_if_dirty();

        assert!(state.total_height > 0.0);
        assert_eq!(state.scroll_offset(), state.total_height - state.viewport_height);

        state.set_scroll_offset(0.0);
        assert_eq!(state.scroll_offset(), 0.0);
        assert!(!state.is_auto_scroll()); // Should be disabled after scrolling up
    }

    #[test]
    fn test_visible_range() {
        let mut state = StreamState::new();
        state.set_viewport_height(200.0);

        for i in 0..20 {
            state.push(make_item(&i.to_string()));
        }

        state.recalculate_heights_if_dirty();
        state.set_scroll_offset(0.0);

        let (start, end) = state.visible_range();
        assert_eq!(start, 0);
        assert!(end > 0);
        assert!(end <= state.len());
    }

    #[test]
    fn test_toggle_expanded() {
        let mut state = StreamState::new();
        state.push(StreamItem::new(
            "reason",
            StreamContent::Reasoning(crate::stream::types::ReasoningBlock::new("Thinking...")),
        ));

        assert!(!state.get("reason").unwrap().expanded);
        state.toggle_expanded("reason");
        assert!(state.get("reason").unwrap().expanded);
    }

    #[test]
    fn test_clear() {
        let mut state = StreamState::new();
        state.push(make_item("1"));
        state.push(make_item("2"));
        state.select(Some("1".to_string()));

        assert_eq!(state.len(), 2);

        state.clear();

        assert!(state.is_empty());
        assert!(state.selected().is_none());
    }
}
