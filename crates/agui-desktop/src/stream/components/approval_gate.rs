//! Approval Gate Component
//!
//! Renders approval requests with action buttons (Approve/Reject/etc).
//! Blocks the agent workflow until user takes action.

use gpui::{div, prelude::*, px, AnyElement, Context, Window};

use super::colors;
use crate::stream::types::{ApprovalAction, ApprovalActionVariant, ApprovalBlock, ApprovalResolution};

/// Callback type for approval actions
pub type OnApprovalAction<V> = Box<dyn Fn(&mut V, &mut Window, &mut Context<V>, String) + Send + Sync>;

/// Render an approval gate
pub fn render_approval_gate<V: 'static>(
    approval: &ApprovalBlock,
    on_action: Option<OnApprovalAction<V>>,
    cx: &mut Context<V>,
) -> AnyElement {
    let title = approval.title.clone();
    let description = approval.description.clone();
    let content = approval.content.clone();
    let content_type = approval.content_type.clone();
    let actions = approval.actions.clone();
    let resolution = approval.resolution.clone();
    let blocking = approval.blocking;

    let is_resolved = resolution.is_some();

    div()
        .w_full()
        .py_2()
        .child(
            div()
                .flex()
                .flex_col()
                .w_full()
                .bg(colors::approval_bg())
                .rounded_md()
                .border_2()
                .border_color(if is_resolved {
                    colors::divider()
                } else {
                    colors::approval_border()
                })
                .when(!is_resolved && blocking, |el| {
                    el.shadow_lg()
                })
                // Header
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .px_4()
                        .py_3()
                        .border_b_1()
                        .border_color(colors::divider())
                        // Title
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_lg()
                                        .child(if is_resolved { "✓" } else { "⚠" }),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(colors::text_primary())
                                        .child(title),
                                ),
                        )
                        // Blocking indicator
                        .when(blocking && !is_resolved, |el| {
                            el.child(
                                div()
                                    .px_2()
                                    .py_px()
                                    .bg(gpui::rgb(0x5a2727))
                                    .rounded_sm()
                                    .text_xs()
                                    .text_color(colors::status_failed())
                                    .child("Blocking"),
                            )
                        }),
                )
                // Description
                .when_some(description, |el, desc| {
                    el.child(
                        div()
                            .px_4()
                            .py_2()
                            .text_sm()
                            .text_color(colors::text_secondary())
                            .child(desc),
                    )
                })
                // Content preview (code, etc.)
                .when_some(content, |el, c| {
                    el.child(
                        div()
                            .mx_4()
                            .mb_2()
                            .bg(gpui::rgb(0x1e1e1e))
                            .rounded_sm()
                            .max_h(px(200.0))
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .text_xs()
                                    .font_family("monospace")
                                    .text_color(colors::text_primary())
                                    .child(c),
                            ),
                    )
                })
                // Resolution info (if resolved)
                .when_some(resolution, |el, res| {
                    el.child(render_resolution_info(&res))
                })
                // Action buttons (if not resolved)
                .when(!is_resolved, |el| {
                    el.child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_end()
                            .gap_2()
                            .px_4()
                            .py_3()
                            .border_t_1()
                            .border_color(colors::divider())
                            .children(
                                actions
                                    .iter()
                                    .map(|action| render_action_button(action))
                                    .collect::<Vec<_>>(),
                            ),
                    )
                }),
        )
        .into_any_element()
}

/// Render an action button
fn render_action_button(action: &ApprovalAction) -> gpui::Div {
    let (bg, hover_bg, text) = match action.variant {
        ApprovalActionVariant::Primary => (
            colors::approve_button(),
            gpui::rgb(0x5ed9c0),
            gpui::rgb(0x1e1e1e),
        ),
        ApprovalActionVariant::Secondary => (
            colors::divider(),
            gpui::rgb(0x505050),
            colors::text_primary(),
        ),
        ApprovalActionVariant::Destructive => (
            colors::reject_button(),
            gpui::rgb(0xf56c6c),
            gpui::rgb(0xffffff),
        ),
    };

    div()
        .px_4()
        .py_2()
        .rounded_md()
        .bg(bg)
        .text_sm()
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(text)
        .cursor_pointer()
        .hover(|el| el.bg(hover_bg))
        .child(action.label.clone())
}

/// Render resolution info
fn render_resolution_info(resolution: &ApprovalResolution) -> gpui::Div {
    let action_id = resolution.action_id.clone();
    let timestamp = resolution.timestamp.format("%H:%M:%S").to_string();
    let comment = resolution.comment.clone();

    div()
        .px_4()
        .py_2()
        .bg(gpui::rgb(0x1e3a1e))
        .border_t_1()
        .border_color(colors::status_completed())
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors::status_completed())
                                .child("✓"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors::status_completed())
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(format!("Resolved: {}", action_id)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors::text_muted())
                                .child(format!("at {}", timestamp)),
                        ),
                )
                .when_some(comment, |el, c| {
                    el.child(
                        div()
                            .text_xs()
                            .text_color(colors::text_secondary())
                            .child(format!("\"{}\"", c)),
                    )
                }),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_action_creation() {
        let action = ApprovalAction {
            id: "approve".to_string(),
            label: "Approve".to_string(),
            variant: ApprovalActionVariant::Primary,
            payload: None,
        };
        assert_eq!(action.id, "approve");
        assert_eq!(action.variant, ApprovalActionVariant::Primary);
    }
}
