//! Form Component Renderer
//!
//! Renders form containers with submit handling.

use gpui::{div, prelude::*, px, AnyElement, Context, MouseButton};
use std::sync::Arc;

use super::colors;
use crate::renderer::form_state::FormAction;
use crate::renderer::schema::FormProps;
use crate::renderer::{render_component, RenderContext};

/// Render a form component
pub fn render_form<V: 'static + Render>(
    props: &FormProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let form_ctx = ctx.with_prefix(&props.id);
    let action_callback = Arc::clone(&ctx.on_action);
    let submit_action = props.submit_action.clone();
    let cancel_action = props.cancel_action.clone();
    let form_id = ctx.full_id(&props.id);

    let mut form = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .gap_4()
        .w_full();

    // Render child components
    for (idx, child) in props.children.iter().enumerate() {
        let child_ctx = form_ctx.with_prefix(&format!("{}", idx));
        form = form.child(render_component(child, &child_ctx, cx));
    }

    // Form actions (submit/cancel buttons)
    let submit_label = props.submit_label.clone().unwrap_or_else(|| "Submit".to_string());
    let cancel_label = props.cancel_label.clone().unwrap_or_else(|| "Cancel".to_string());

    let mut action_row = div()
        .flex()
        .flex_row()
        .gap_2()
        .justify_end()
        .mt_4()
        .pt_4()
        .border_t_1()
        .border_color(colors::border_default());

    // Cancel button (if cancel_action provided)
    if let Some(cancel) = cancel_action {
        let action_callback_clone = Arc::clone(&action_callback);
        let form_id_clone = form_id.clone();

        action_row = action_row.child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .px_4()
                .py_2()
                .rounded(px(4.0))
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .bg(colors::secondary())
                .border_1()
                .border_color(colors::border_default())
                .text_color(colors::text_primary())
                .cursor_pointer()
                .hover(|style| style.bg(colors::secondary_hover()))
                .child(cancel_label)
                .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
                    let form_action = FormAction::button_click(form_id_clone.clone(), cancel.clone());
                    action_callback_clone(view, window, cx, form_action);
                })),
        );
    }

    // Submit button
    let action_callback_clone = Arc::clone(&action_callback);
    let form_id_clone = form_id.clone();
    let form_state = ctx.form_state.clone();

    action_row = action_row.child(
        div()
            .flex()
            .items_center()
            .justify_center()
            .px_4()
            .py_2()
            .rounded(px(4.0))
            .text_sm()
            .font_weight(gpui::FontWeight::MEDIUM)
            .bg(colors::primary())
            .border_1()
            .border_color(colors::primary())
            .text_color(gpui::rgb(0xffffff))
            .cursor_pointer()
            .hover(|style| style.bg(colors::primary_hover()))
            .child(submit_label)
            .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
                // Collect form values
                let form_values = form_state.get_form_values(&form_id_clone);
                let form_action = FormAction::form_submit(
                    form_id_clone.clone(),
                    submit_action.clone(),
                    form_values,
                );
                action_callback_clone(view, window, cx, form_action);
            })),
    );

    form = form.child(action_row);

    form.into_any_element()
}
