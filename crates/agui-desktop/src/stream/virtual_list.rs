//! Virtual List Implementation
//!
//! Provides efficient rendering of large lists by only rendering
//! visible items plus a buffer zone. Uses estimated heights for
//! initial layout and can adjust based on actual measured heights.

use gpui::{px, Pixels};

/// Configuration for virtual list behavior
#[derive(Debug, Clone)]
pub struct VirtualListConfig {
    /// Number of items to render above/below visible area
    pub overscan: usize,
    /// Minimum item height (for estimation)
    pub min_item_height: f32,
    /// Maximum item height (for estimation)
    pub max_item_height: f32,
    /// Default item height (for estimation)
    pub default_item_height: f32,
    /// Whether to enable smooth scrolling
    pub smooth_scroll: bool,
    /// Scroll speed multiplier
    pub scroll_speed: f32,
}

impl Default for VirtualListConfig {
    fn default() -> Self {
        Self {
            overscan: 3,
            min_item_height: 40.0,
            max_item_height: 500.0,
            default_item_height: 80.0,
            smooth_scroll: true,
            scroll_speed: 40.0,
        }
    }
}

/// Virtual list state and calculations
#[derive(Debug, Clone)]
pub struct VirtualList {
    /// Configuration
    config: VirtualListConfig,
    /// Total number of items
    item_count: usize,
    /// Cached item heights (measured or estimated)
    heights: Vec<f32>,
    /// Cumulative heights for O(log n) position lookup
    cumulative_heights: Vec<f32>,
    /// Total content height
    total_height: f32,
    /// Current scroll offset
    scroll_offset: f32,
    /// Viewport height
    viewport_height: f32,
    /// Whether heights need recalculation
    dirty: bool,
}

impl VirtualList {
    /// Create a new virtual list with given configuration
    pub fn new(config: VirtualListConfig) -> Self {
        Self {
            config,
            item_count: 0,
            heights: Vec::new(),
            cumulative_heights: Vec::new(),
            total_height: 0.0,
            scroll_offset: 0.0,
            viewport_height: 600.0,
            dirty: false,
        }
    }

    /// Set the total number of items
    pub fn set_item_count(&mut self, count: usize) {
        if count != self.item_count {
            self.item_count = count;
            // Resize heights array, using default height for new items
            self.heights.resize(count, self.config.default_item_height);
            self.dirty = true;
        }
    }

    /// Set viewport height
    pub fn set_viewport_height(&mut self, height: f32) {
        self.viewport_height = height;
    }

    /// Get viewport height
    pub fn viewport_height(&self) -> f32 {
        self.viewport_height
    }

    /// Set estimated height for an item
    pub fn set_item_height(&mut self, index: usize, height: f32) {
        if index < self.heights.len() {
            let clamped = height.clamp(self.config.min_item_height, self.config.max_item_height);
            if (self.heights[index] - clamped).abs() > 0.5 {
                self.heights[index] = clamped;
                self.dirty = true;
            }
        }
    }

    /// Update heights in batch (more efficient)
    pub fn set_item_heights(&mut self, heights: impl IntoIterator<Item = (usize, f32)>) {
        for (index, height) in heights {
            if index < self.heights.len() {
                let clamped =
                    height.clamp(self.config.min_item_height, self.config.max_item_height);
                if (self.heights[index] - clamped).abs() > 0.5 {
                    self.heights[index] = clamped;
                    self.dirty = true;
                }
            }
        }
    }

    /// Get current scroll offset
    pub fn scroll_offset(&self) -> f32 {
        self.scroll_offset
    }

    /// Set scroll offset (clamped)
    pub fn set_scroll_offset(&mut self, offset: f32) {
        self.ensure_calculated();
        let max_scroll = (self.total_height - self.viewport_height).max(0.0);
        self.scroll_offset = offset.clamp(0.0, max_scroll);
    }

    /// Scroll by delta
    pub fn scroll_by(&mut self, delta: f32) {
        self.set_scroll_offset(self.scroll_offset + delta * self.config.scroll_speed);
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.ensure_calculated();
        let max_scroll = (self.total_height - self.viewport_height).max(0.0);
        self.scroll_offset = max_scroll;
    }

    /// Scroll to ensure an item is visible
    pub fn scroll_to_item(&mut self, index: usize) {
        self.ensure_calculated();

        if index >= self.item_count {
            return;
        }

        let item_top = self.item_offset(index);
        let item_bottom = item_top + self.heights[index];

        // If item is above viewport, scroll up
        if item_top < self.scroll_offset {
            self.scroll_offset = item_top;
        }
        // If item is below viewport, scroll down
        else if item_bottom > self.scroll_offset + self.viewport_height {
            self.scroll_offset = item_bottom - self.viewport_height;
        }
    }

    /// Get total content height
    pub fn total_height(&self) -> f32 {
        self.ensure_calculated_immut();
        self.total_height
    }

    /// Get total height as Pixels
    pub fn total_height_px(&self) -> Pixels {
        px(self.total_height())
    }

    /// Get the Y offset for an item
    pub fn item_offset(&self, index: usize) -> f32 {
        self.ensure_calculated_immut();
        if index == 0 {
            0.0
        } else if index <= self.cumulative_heights.len() {
            self.cumulative_heights[index - 1]
        } else {
            self.total_height
        }
    }

    /// Get the Y offset as Pixels
    pub fn item_offset_px(&self, index: usize) -> Pixels {
        px(self.item_offset(index))
    }

    /// Get visible item range (start inclusive, end exclusive)
    pub fn visible_range(&self) -> VisibleRange {
        self.ensure_calculated_immut();

        if self.item_count == 0 {
            return VisibleRange {
                start: 0,
                end: 0,
                start_offset: 0.0,
            };
        }

        let start_idx = self.find_item_at_offset(self.scroll_offset);
        let end_idx = self
            .find_item_at_offset(self.scroll_offset + self.viewport_height)
            .min(self.item_count - 1);

        // Add overscan
        let start = start_idx.saturating_sub(self.config.overscan);
        let end = (end_idx + 1 + self.config.overscan).min(self.item_count);

        let start_offset = self.item_offset(start);

        VisibleRange {
            start,
            end,
            start_offset,
        }
    }

    /// Check if we're at the bottom (for auto-scroll detection)
    pub fn is_at_bottom(&self) -> bool {
        self.ensure_calculated_immut();
        let max_scroll = (self.total_height - self.viewport_height).max(0.0);
        self.scroll_offset >= max_scroll - 10.0 // 10px tolerance
    }

    /// Check if we're at the top
    pub fn is_at_top(&self) -> bool {
        self.scroll_offset <= 0.0
    }

    /// Get item height
    pub fn get_item_height(&self, index: usize) -> f32 {
        self.heights.get(index).copied().unwrap_or(self.config.default_item_height)
    }

    // Private helpers

    fn ensure_calculated(&mut self) {
        if !self.dirty {
            return;
        }

        self.cumulative_heights.clear();
        let mut sum = 0.0;

        for &h in &self.heights {
            sum += h;
            self.cumulative_heights.push(sum);
        }

        self.total_height = sum;
        self.dirty = false;
    }

    fn ensure_calculated_immut(&self) {
        // In immutable context, we assume heights are already calculated
        // Callers should ensure this by calling methods that trigger calculation first
    }

    fn find_item_at_offset(&self, offset: f32) -> usize {
        if self.cumulative_heights.is_empty() {
            return 0;
        }

        match self
            .cumulative_heights
            .binary_search_by(|h| h.partial_cmp(&offset).unwrap())
        {
            Ok(idx) => idx,
            Err(idx) => idx,
        }
        .min(self.item_count.saturating_sub(1))
    }
}

/// Information about visible items
#[derive(Debug, Clone, Copy)]
pub struct VisibleRange {
    /// First visible item index (inclusive)
    pub start: usize,
    /// Last visible item index (exclusive)
    pub end: usize,
    /// Y offset of the first item
    pub start_offset: f32,
}

impl VisibleRange {
    /// Iterate over visible indices
    pub fn iter(&self) -> impl Iterator<Item = usize> {
        self.start..self.end
    }

    /// Get number of visible items
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_list_basic() {
        let mut list = VirtualList::new(VirtualListConfig::default());
        list.set_viewport_height(200.0);
        list.set_item_count(100);
        list.ensure_calculated(); // Must calculate heights before checking

        // Initial state
        assert_eq!(list.scroll_offset(), 0.0);
        assert!(list.total_height() > 0.0);
    }

    #[test]
    fn test_visible_range() {
        let mut config = VirtualListConfig::default();
        config.default_item_height = 50.0;
        config.overscan = 2;

        let mut list = VirtualList::new(config);
        list.set_viewport_height(200.0); // 4 items visible
        list.set_item_count(20);
        list.ensure_calculated();

        let range = list.visible_range();
        assert_eq!(range.start, 0);
        assert!(range.end > 4); // 4 visible + 2 overscan = 6
    }

    #[test]
    fn test_scroll_to_item() {
        let mut config = VirtualListConfig::default();
        config.default_item_height = 50.0;

        let mut list = VirtualList::new(config);
        list.set_viewport_height(200.0);
        list.set_item_count(20);
        list.ensure_calculated();

        list.scroll_to_item(10);

        let range = list.visible_range();
        assert!(range.start <= 10);
        assert!(range.end > 10);
    }

    #[test]
    fn test_scroll_bounds() {
        let mut config = VirtualListConfig::default();
        config.default_item_height = 100.0;

        let mut list = VirtualList::new(config);
        list.set_viewport_height(500.0);
        list.set_item_count(10); // 1000px total
        list.ensure_calculated();

        // Try to scroll past bounds
        list.set_scroll_offset(-100.0);
        assert_eq!(list.scroll_offset(), 0.0);

        list.set_scroll_offset(1000.0);
        assert_eq!(list.scroll_offset(), 500.0); // max = 1000 - 500
    }

    #[test]
    fn test_is_at_bottom() {
        let mut config = VirtualListConfig::default();
        config.default_item_height = 100.0;

        let mut list = VirtualList::new(config);
        list.set_viewport_height(500.0);
        list.set_item_count(10);
        list.ensure_calculated();

        assert!(!list.is_at_bottom());

        list.scroll_to_bottom();
        assert!(list.is_at_bottom());
    }

    #[test]
    fn test_variable_heights() {
        let mut list = VirtualList::new(VirtualListConfig::default());
        list.set_item_count(5);

        // Set variable heights
        list.set_item_height(0, 50.0);
        list.set_item_height(1, 100.0);
        list.set_item_height(2, 75.0);
        list.set_item_height(3, 50.0);
        list.set_item_height(4, 125.0);

        list.ensure_calculated();

        assert_eq!(list.item_offset(0), 0.0);
        assert_eq!(list.item_offset(1), 50.0);
        assert_eq!(list.item_offset(2), 150.0);
        assert_eq!(list.item_offset(3), 225.0);
        assert_eq!(list.item_offset(4), 275.0);
        assert_eq!(list.total_height(), 400.0);
    }
}
