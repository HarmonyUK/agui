//! Data Display Component Renderers
//!
//! Renders tables, trees, badges, progress bars, chips, toasts, diffs, and roster items.

use gpui::{div, prelude::*, px, AnyElement, Context, MouseButton, Rgba};
use std::sync::Arc;

use super::colors;
use crate::renderer::form_state::FormAction;
use crate::renderer::schema::{
    AgentStatusType, BadgeProps, BadgeVariant, ChipProps, ChipVariant, DiffProps, ProgressProps,
    ProgressVariant, RosterItemProps, SortDirection, TableProps, ToastProps, ToastVariant, TreeProps,
};
use crate::renderer::RenderContext;

/// Transparent color helper
fn transparent() -> Rgba {
    Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
}

/// Red highlight for diff deletions
fn diff_deletion_bg() -> Rgba {
    Rgba { r: 1.0, g: 0.3, b: 0.3, a: 0.2 }
}

/// Green highlight for diff additions
fn diff_addition_bg() -> Rgba {
    Rgba { r: 0.3, g: 1.0, b: 0.3, a: 0.2 }
}

/// Render a table component
pub fn render_table<V: 'static + Render>(
    props: &TableProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let action_callback = Arc::clone(&ctx.on_action);

    let mut container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .w_full()
        .border_1()
        .border_color(colors::border_default())
        .rounded(px(4.0))
        .overflow_hidden();

    // Header row
    let mut header_row = div()
        .flex()
        .flex_row()
        .bg(colors::bg_elevated())
        .border_b_1()
        .border_color(colors::border_default());

    for col in &props.columns {
        let mut header_cell = div()
            .flex()
            .items_center()
            .gap_1()
            .px_3()
            .py_2()
            .text_sm()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(colors::text_primary());

        if let Some(width) = col.width {
            header_cell = header_cell.w(px(width));
        } else {
            header_cell = header_cell.flex_1();
        }

        header_cell = header_cell.child(col.label.clone());

        // Sort indicator
        if props.sortable && col.sortable {
            let is_sorted = props.sort_column.as_ref() == Some(&col.id);
            if is_sorted {
                let sort_icon = match props.sort_direction {
                    SortDirection::Asc => "\u{25B2}",
                    SortDirection::Desc => "\u{25BC}",
                };
                header_cell = header_cell.child(
                    div()
                        .text_xs()
                        .text_color(colors::text_secondary())
                        .child(sort_icon),
                );
            }
            header_cell = header_cell.cursor_pointer();
        }

        header_row = header_row.child(header_cell);
    }

    container = container.child(header_row);

    // Data rows
    for row in &props.rows {
        let is_selected = props.selected.contains(&row.id);
        let row_id = row.id.clone();
        let component_id = ctx.full_id(&props.id);
        let action_callback_clone = Arc::clone(&action_callback);

        let row_bg = if is_selected {
            colors::bg_hover()
        } else {
            colors::bg_dark()
        };

        let mut data_row = div()
            .flex()
            .flex_row()
            .bg(row_bg)
            .border_b_1()
            .border_color(colors::border_default())
            .hover(|style| style.bg(colors::bg_hover()));

        for col in &props.columns {
            let cell_value = row.cells.get(&col.id);
            let display_value = cell_value
                .map(|v| format_cell_value(v))
                .unwrap_or_default();

            let mut cell = div()
                .flex()
                .items_center()
                .px_3()
                .py_2()
                .text_sm()
                .text_color(colors::text_primary());

            if let Some(width) = col.width {
                cell = cell.w(px(width));
            } else {
                cell = cell.flex_1();
            }

            cell = cell.child(display_value);
            data_row = data_row.child(cell);
        }

        if props.selectable {
            data_row = data_row
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
                    let form_action = FormAction::new("row_select", component_id.clone())
                        .with_payload(serde_json::json!({ "row_id": row_id.clone() }));
                    action_callback_clone(view, window, cx, form_action);
                }));
        }

        container = container.child(data_row);
    }

    container.into_any_element()
}

fn format_cell_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => if *b { "\u{2713}" } else { "\u{2717}" }.to_string(),
        serde_json::Value::Null => "—".to_string(),
        _ => value.to_string(),
    }
}

/// Render a tree view component
pub fn render_tree<V: 'static + Render>(
    props: &TreeProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let mut container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .w_full();

    for node in &props.nodes {
        container = container.child(render_tree_node(node, props, ctx, cx, 0));
    }

    container.into_any_element()
}

fn render_tree_node<V: 'static + Render>(
    node: &crate::renderer::schema::TreeNode,
    props: &TreeProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
    depth: usize,
) -> gpui::Div {
    let is_expanded = props.expanded.contains(&node.id);
    let is_selected = props.selected.as_ref() == Some(&node.id);
    let has_children = !node.children.is_empty();

    let indent = depth as f32 * 16.0;
    let action_callback = Arc::clone(&ctx.on_action);
    let node_id = node.id.clone();
    let component_id = ctx.full_id(&props.id);
    let on_select = props.on_select.clone();

    let bg_color = if is_selected {
        colors::bg_hover()
    } else {
        transparent()
    };

    // Node icon based on type
    let icon: String = node.icon.clone().unwrap_or_else(|| {
        if has_children {
            if is_expanded { "\u{1F4C2}" } else { "\u{1F4C1}" }
        } else {
            "\u{1F4C4}"
        }.to_string()
    });

    let mut node_row = div()
        .flex()
        .items_center()
        .gap_1()
        .h(px(28.0))
        .pl(px(indent))
        .pr_2()
        .bg(bg_color)
        .cursor_pointer()
        .hover(|style| style.bg(colors::bg_hover()));

    // Expand/collapse indicator
    if has_children {
        node_row = node_row.child(
            div()
                .w(px(16.0))
                .text_xs()
                .text_color(colors::text_secondary())
                .child(if is_expanded { "\u{25BC}" } else { "\u{25B6}" }),
        );
    } else {
        node_row = node_row.child(div().w(px(16.0)));
    }

    // Icon
    node_row = node_row.child(
        div()
            .text_sm()
            .mr_1()
            .child(icon),
    );

    // Label
    node_row = node_row.child(
        div()
            .text_sm()
            .text_color(colors::text_primary())
            .child(node.label.clone()),
    );

    // Click handler
    node_row = node_row.on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
        if let Some(action) = &on_select {
            let form_action = FormAction::new(action.clone(), component_id.clone())
                .with_payload(serde_json::json!({ "node_id": node_id.clone() }));
            action_callback(view, window, cx, form_action);
        }
    }));

    let mut container = div().flex().flex_col().child(node_row);

    // Render children if expanded
    if is_expanded && has_children {
        for child in &node.children {
            container = container.child(render_tree_node(child, props, ctx, cx, depth + 1));
        }
    }

    container
}

/// Render a badge component
pub fn render_badge<V: 'static + Render>(
    props: &BadgeProps,
    _ctx: &RenderContext<'_, V>,
    _cx: &mut Context<V>,
) -> AnyElement {
    let (bg_color, text_color) = match props.variant {
        BadgeVariant::Default => (colors::badge_default(), colors::text_primary()),
        BadgeVariant::Primary => (colors::primary(), gpui::rgb(0xffffff)),
        BadgeVariant::Secondary => (colors::secondary(), colors::text_primary()),
        BadgeVariant::Success => (colors::success(), gpui::rgb(0x1e1e1e)),
        BadgeVariant::Warning => (colors::warning(), gpui::rgb(0x1e1e1e)),
        BadgeVariant::Error => (colors::error(), gpui::rgb(0xffffff)),
        BadgeVariant::Info => (colors::info(), gpui::rgb(0xffffff)),
    };

    let mut badge = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .items_center()
        .gap_1()
        .px_2()
        .py_1()
        .bg(bg_color)
        .text_xs()
        .text_color(text_color)
        .rounded(px(10.0));

    // Dot indicator
    if props.dot {
        badge = badge.child(
            div()
                .w(px(6.0))
                .h(px(6.0))
                .bg(text_color)
                .rounded_full(),
        );
    }

    badge = badge.child(props.label.clone());

    badge.into_any_element()
}

/// Render a progress indicator
pub fn render_progress<V: 'static + Render>(
    props: &ProgressProps,
    _ctx: &RenderContext<'_, V>,
    _cx: &mut Context<V>,
) -> AnyElement {
    let percentage = props
        .value
        .map(|v| (v / props.max * 100.0).clamp(0.0, 100.0))
        .unwrap_or(0.0);

    let mut container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .gap_1()
        .w_full();

    // Label
    if let Some(label) = &props.label {
        let mut label_row = div()
            .flex()
            .items_center()
            .justify_between()
            .text_sm()
            .text_color(colors::text_secondary())
            .child(label.clone());

        if props.show_percentage {
            label_row = label_row.child(format!("{:.0}%", percentage));
        }

        container = container.child(label_row);
    }

    match props.variant {
        ProgressVariant::Linear | ProgressVariant::Indeterminate => {
            let mut track = div()
                .h(px(6.0))
                .w_full()
                .bg(colors::progress_bg())
                .rounded(px(3.0))
                .overflow_hidden();

            if props.value.is_some() {
                // Determinate progress
                track = track.child(
                    div()
                        .h_full()
                        .bg(colors::progress_fill())
                        .rounded(px(3.0))
                        .w(gpui::relative(percentage as f32 / 100.0)),
                );
            }

            container = container.child(track);
        }
        ProgressVariant::Circular => {
            // Simplified circular progress (using text representation)
            container = container.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(48.0))
                    .h(px(48.0))
                    .text_sm()
                    .text_color(colors::text_primary())
                    .child(format!("{:.0}%", percentage)),
            );
        }
    }

    container.into_any_element()
}

/// Render a chip/tag component
pub fn render_chip<V: 'static + Render>(
    props: &ChipProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let (bg_color, text_color) = match props.variant {
        ChipVariant::Default => (colors::bg_input(), colors::text_primary()),
        ChipVariant::Primary => (colors::primary(), gpui::rgb(0xffffff)),
        ChipVariant::Secondary => (colors::secondary(), colors::text_primary()),
        ChipVariant::Success => (colors::success(), gpui::rgb(0x1e1e1e)),
        ChipVariant::Warning => (colors::warning(), gpui::rgb(0x1e1e1e)),
        ChipVariant::Error => (colors::error(), gpui::rgb(0xffffff)),
    };

    let mut chip = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .items_center()
        .gap_1()
        .px_2()
        .py_1()
        .bg(bg_color)
        .text_xs()
        .text_color(text_color)
        .rounded(px(12.0));

    // Icon
    if let Some(icon) = &props.icon {
        chip = chip.child(div().child(icon.clone()));
    }

    chip = chip.child(props.label.clone());

    // Dismiss button
    if props.dismissible {
        let action_callback = Arc::clone(&ctx.on_action);
        let dismiss_action = props.dismiss_action.clone();
        let component_id = ctx.full_id(&props.id);

        chip = chip.child(
            div()
                .ml_1()
                .cursor_pointer()
                .text_color(text_color)
                .opacity(0.7)
                .hover(|style| style.opacity(1.0))
                .child("\u{2715}")
                .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
                    let action = dismiss_action.as_ref().unwrap_or(&"dismiss".to_string()).clone();
                    let form_action = FormAction::button_click(component_id.clone(), action);
                    action_callback(view, window, cx, form_action);
                })),
        );
    }

    chip.into_any_element()
}

/// Render a toast notification
pub fn render_toast<V: 'static + Render>(
    props: &ToastProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let (bg_color, icon, text_color) = match props.variant {
        ToastVariant::Info => (colors::info(), "\u{2139}", gpui::rgb(0xffffff)),
        ToastVariant::Success => (colors::success(), "\u{2713}", gpui::rgb(0x1e1e1e)),
        ToastVariant::Warning => (colors::warning(), "\u{26A0}", gpui::rgb(0x1e1e1e)),
        ToastVariant::Error => (colors::error(), "\u{2715}", gpui::rgb(0xffffff)),
    };

    let mut toast = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .items_center()
        .gap_3()
        .px_4()
        .py_3()
        .bg(bg_color)
        .text_color(text_color)
        .rounded(px(6.0))
        .shadow_lg()
        .min_w(px(280.0))
        .max_w(px(400.0));

    // Icon
    toast = toast.child(div().text_lg().child(icon));

    // Message
    toast = toast.child(
        div()
            .flex_1()
            .text_sm()
            .child(props.message.clone()),
    );

    // Action button
    if let Some(action) = &props.action {
        let action_callback = Arc::clone(&ctx.on_action);
        let action_type = action.action.clone();
        let component_id = ctx.full_id(&props.id);

        toast = toast.child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .cursor_pointer()
                .child(action.label.clone())
                .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
                    let form_action = FormAction::button_click(component_id.clone(), action_type.clone());
                    action_callback(view, window, cx, form_action);
                })),
        );
    }

    toast.into_any_element()
}

/// Render a diff view
pub fn render_diff<V: 'static + Render>(
    props: &DiffProps,
    _ctx: &RenderContext<'_, V>,
    _cx: &mut Context<V>,
) -> AnyElement {
    // Simple line-by-line diff visualization
    let old_lines: Vec<_> = props.old_content.lines().collect();
    let new_lines: Vec<_> = props.new_content.lines().collect();

    let mut container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .w_full()
        .border_1()
        .border_color(colors::border_default())
        .rounded(px(4.0))
        .overflow_hidden();

    // Language label
    if let Some(lang) = &props.language {
        container = container.child(
            div()
                .px_3()
                .py_1()
                .bg(colors::bg_elevated())
                .border_b_1()
                .border_color(colors::border_default())
                .text_xs()
                .text_color(colors::text_muted())
                .child(lang.clone()),
        );
    }

    let mut diff_content = div()
        .flex()
        .flex_col()
        .p_2()
        .overflow_hidden()
        .text_sm();

    // Compute simple diff
    let max_lines = old_lines.len().max(new_lines.len());
    for i in 0..max_lines {
        let old_line = old_lines.get(i);
        let new_line = new_lines.get(i);

        match (old_line, new_line) {
            (Some(old), Some(new)) if old == new => {
                // Unchanged
                diff_content = diff_content.child(
                    div()
                        .flex()
                        .flex_row()
                        .text_color(colors::text_primary())
                        .child(div().w(px(20.0)).text_color(colors::text_muted()).child(" "))
                        .child((*old).to_string()),
                );
            }
            (Some(old), Some(new)) => {
                // Changed
                diff_content = diff_content
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .bg(diff_deletion_bg())
                            .text_color(colors::error())
                            .child(div().w(px(20.0)).child("-"))
                            .child((*old).to_string()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .bg(diff_addition_bg())
                            .text_color(colors::success())
                            .child(div().w(px(20.0)).child("+"))
                            .child((*new).to_string()),
                    );
            }
            (Some(old), None) => {
                // Deleted
                diff_content = diff_content.child(
                    div()
                        .flex()
                        .flex_row()
                        .bg(diff_deletion_bg())
                        .text_color(colors::error())
                        .child(div().w(px(20.0)).child("-"))
                        .child((*old).to_string()),
                );
            }
            (None, Some(new)) => {
                // Added
                diff_content = diff_content.child(
                    div()
                        .flex()
                        .flex_row()
                        .bg(diff_addition_bg())
                        .text_color(colors::success())
                        .child(div().w(px(20.0)).child("+"))
                        .child((*new).to_string()),
                );
            }
            (None, None) => {}
        }
    }

    container = container.child(diff_content);

    container.into_any_element()
}

/// Render an agent roster item
pub fn render_roster_item<V: 'static + Render>(
    props: &RosterItemProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let status_color = match props.status {
        AgentStatusType::Online => colors::success(),
        AgentStatusType::Busy => colors::warning(),
        AgentStatusType::Idle => colors::text_secondary(),
        AgentStatusType::Offline => colors::text_muted(),
    };

    let status_text = match props.status {
        AgentStatusType::Online => "Online",
        AgentStatusType::Busy => "Busy",
        AgentStatusType::Idle => "Idle",
        AgentStatusType::Offline => "Offline",
    };

    let action_callback = Arc::clone(&ctx.on_action);
    let on_click = props.on_click.clone();
    let component_id = ctx.full_id(&props.id);

    let mut item = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .items_center()
        .gap_3()
        .px_3()
        .py_2()
        .rounded(px(4.0))
        .hover(|style| style.bg(colors::bg_hover()));

    // Avatar or placeholder
    item = item.child(
        div()
            .flex()
            .items_center()
            .justify_center()
            .w(px(36.0))
            .h(px(36.0))
            .bg(colors::bg_input())
            .rounded_full()
            .text_sm()
            .text_color(colors::text_primary())
            .child(
                props
                    .avatar
                    .clone()
                    .unwrap_or_else(|| props.name.chars().next().unwrap_or('?').to_string()),
            ),
    );

    // Name and subtitle
    let mut info = div().flex().flex_col().flex_1();

    info = info.child(
        div()
            .text_sm()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(colors::text_primary())
            .child(props.name.clone()),
    );

    if let Some(subtitle) = &props.subtitle {
        info = info.child(
            div()
                .text_xs()
                .text_color(colors::text_secondary())
                .child(subtitle.clone()),
        );
    }

    item = item.child(info);

    // Status indicator
    item = item.child(
        div()
            .flex()
            .items_center()
            .gap_1()
            .child(
                div()
                    .w(px(8.0))
                    .h(px(8.0))
                    .bg(status_color)
                    .rounded_full(),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors::text_secondary())
                    .child(status_text),
            ),
    );

    if on_click.is_some() {
        item = item
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
                if let Some(action) = &on_click {
                    let form_action = FormAction::button_click(component_id.clone(), action.clone());
                    action_callback(view, window, cx, form_action);
                }
            }));
    }

    item.into_any_element()
}
