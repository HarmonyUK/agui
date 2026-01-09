//! Plan Checklist Component
//!
//! Renders a plan/task list with checkboxes showing completion state.
//! Supports nested items and progress tracking.

use gpui::{div, prelude::*, px, AnyElement, Context, Window};

use super::colors;
use crate::stream::types::{PlanBlock, PlanItem, PlanItemStatus, PlanStatus};

/// Callback type for item click
pub type OnItemClick<V> = Box<dyn Fn(&mut V, &mut Window, &mut Context<V>, String) + Send + Sync>;

/// Render a plan checklist
pub fn render_plan_checklist<V: 'static>(
    plan: &PlanBlock,
    on_item_click: Option<OnItemClick<V>>,
    cx: &mut Context<V>,
) -> AnyElement {
    let title = plan.title.clone();
    let items = plan.items.clone();
    let status = plan.status;
    let completion = plan.completion_percentage();

    div()
        .w_full()
        .py_2()
        .child(
            div()
                .flex()
                .flex_col()
                .w_full()
                .bg(colors::plan_bg())
                .rounded_md()
                .border_1()
                .border_color(plan_border_color(status))
                // Header
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .px_3()
                        .py_2()
                        .border_b_1()
                        .border_color(colors::divider())
                        // Left: icon and title
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .child("📋"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(colors::text_primary())
                                        .child(title),
                                )
                                .child(render_plan_status_badge(status)),
                        )
                        // Right: progress
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(render_completion_ring(completion))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors::text_secondary())
                                        .child(format!("{}%", completion)),
                                ),
                        ),
                )
                // Items
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .py_1()
                        .children(
                            items
                                .iter()
                                .map(|item| render_plan_item(item, 0))
                                .collect::<Vec<_>>(),
                        ),
                ),
        )
        .into_any_element()
}

/// Render a single plan item (recursive for nested items)
fn render_plan_item(item: &PlanItem, depth: usize) -> gpui::Div {
    let description = item.description.clone();
    let status = item.status;
    let children = item.children.clone();
    let indent = (depth as f32) * 20.0;

    div()
        .flex()
        .flex_col()
        .w_full()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px_3()
                .py_1()
                .pl(px(12.0 + indent))
                .hover(|el| el.bg(gpui::rgb(0x2d2d30)))
                .cursor_pointer()
                // Checkbox/status indicator
                .child(render_item_checkbox(status))
                // Description
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(item_text_color(status))
                        .when(status == PlanItemStatus::Completed, |el| {
                            el.line_through()
                        })
                        .child(description),
                ),
        )
        // Nested children
        .when(!children.is_empty(), |el| {
            el.children(
                children
                    .iter()
                    .map(|child| render_plan_item(child, depth + 1))
                    .collect::<Vec<_>>(),
            )
        })
}

/// Render checkbox based on status
fn render_item_checkbox(status: PlanItemStatus) -> gpui::Div {
    let (icon, color) = match status {
        PlanItemStatus::Pending => ("○", colors::plan_item_pending()),
        PlanItemStatus::InProgress => ("◐", colors::plan_item_active()),
        PlanItemStatus::Completed => ("✓", colors::plan_item_completed()),
        PlanItemStatus::Skipped => ("⊘", colors::plan_item_skipped()),
        PlanItemStatus::Failed => ("✗", colors::status_failed()),
    };

    div()
        .w(px(16.0))
        .h(px(16.0))
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .text_color(color)
        .child(icon)
}

/// Get text color based on item status
fn item_text_color(status: PlanItemStatus) -> gpui::Rgba {
    match status {
        PlanItemStatus::Pending => colors::text_primary(),
        PlanItemStatus::InProgress => colors::plan_item_active(),
        PlanItemStatus::Completed => colors::text_secondary(),
        PlanItemStatus::Skipped => colors::text_muted(),
        PlanItemStatus::Failed => colors::status_failed(),
    }
}

/// Get border color based on plan status
fn plan_border_color(status: PlanStatus) -> gpui::Rgba {
    match status {
        PlanStatus::Draft => colors::divider(),
        PlanStatus::PendingApproval => gpui::rgb(0xdcdcaa), // Yellow
        PlanStatus::Active => colors::status_running(),
        PlanStatus::Completed => colors::status_completed(),
        PlanStatus::Cancelled => colors::text_muted(),
    }
}

/// Render plan status badge
fn render_plan_status_badge(status: PlanStatus) -> gpui::Div {
    let (bg, text, label) = match status {
        PlanStatus::Draft => (gpui::rgb(0x404040), colors::text_secondary(), "Draft"),
        PlanStatus::PendingApproval => (gpui::rgb(0x5a5a27), gpui::rgb(0xdcdcaa), "Pending"),
        PlanStatus::Active => (gpui::rgb(0x0e639c), gpui::rgb(0xffffff), "Active"),
        PlanStatus::Completed => (gpui::rgb(0x2d5a27), colors::status_completed(), "Done"),
        PlanStatus::Cancelled => (gpui::rgb(0x404040), colors::text_muted(), "Cancelled"),
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

/// Render completion ring/circle
fn render_completion_ring(percentage: u8) -> gpui::Div {
    // Simple representation - a colored circle
    let color = if percentage == 100 {
        colors::status_completed()
    } else if percentage > 0 {
        colors::status_running()
    } else {
        colors::text_muted()
    };

    div()
        .w(px(16.0))
        .h(px(16.0))
        .rounded_full()
        .border_2()
        .border_color(color)
        .when(percentage == 100, |el| el.bg(color))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_block_completion() {
        let plan = PlanBlock::new("Test Plan").with_items(vec![
            PlanItem::new("1", "First"),
            PlanItem {
                id: "2".to_string(),
                description: "Second".to_string(),
                status: PlanItemStatus::Completed,
                children: vec![],
            },
        ]);

        assert_eq!(plan.completion_percentage(), 50);
    }

    #[test]
    fn test_plan_item_creation() {
        let item = PlanItem::new("test-1", "Test item");
        assert_eq!(item.id, "test-1");
        assert_eq!(item.description, "Test item");
        assert_eq!(item.status, PlanItemStatus::Pending);
        assert!(item.children.is_empty());
    }
}
