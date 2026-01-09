//! Input Component Renderers
//!
//! Renders text inputs, text areas, selects, toggles, sliders, and checkboxes.

use gpui::{div, prelude::*, px, AnyElement, Context, MouseButton};
use std::sync::Arc;

use super::colors;
use crate::renderer::form_state::{FormAction, FormValue};
use crate::renderer::schema::{CheckboxProps, SelectProps, SliderProps, TextAreaProps, TextInputProps, ToggleProps};
use crate::renderer::RenderContext;

/// Render a text input component
pub fn render_text_input<V: 'static + Render>(
    props: &TextInputProps,
    ctx: &RenderContext<'_, V>,
    _cx: &mut Context<V>,
) -> AnyElement {
    let full_id = ctx.full_id(&props.id);
    let has_error = props.error.is_some() || ctx.form_state.get_error(&full_id).is_some();

    let border_color = if has_error {
        colors::border_error()
    } else {
        colors::border_default()
    };

    // Get current value from form state or props
    let current_value = ctx
        .form_state
        .get_string(&full_id)
        .unwrap_or(&props.value);

    let mut container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .gap_1()
        .w_full();

    // Label
    if let Some(label) = &props.label {
        let mut label_element = div()
            .text_sm()
            .text_color(colors::text_secondary())
            .child(label.clone());

        if props.required {
            label_element = label_element.child(
                div()
                    .text_color(colors::error())
                    .ml_1()
                    .child("*"),
            );
        }

        container = container.child(label_element);
    }

    // Input field (styled div simulating input)
    let mut input = div()
        .flex()
        .items_center()
        .h(px(36.0))
        .px_3()
        .bg(colors::bg_input())
        .border_1()
        .border_color(border_color)
        .rounded(px(4.0))
        .text_sm();

    if current_value.is_empty() {
        if let Some(placeholder) = &props.placeholder {
            input = input
                .text_color(colors::text_muted())
                .child(placeholder.clone());
        }
    } else {
        // Mask password input
        let display_value = if matches!(props.input_type, crate::renderer::schema::InputType::Password) {
            "\u{2022}".repeat(current_value.len())
        } else {
            current_value.to_string()
        };
        input = input
            .text_color(colors::text_primary())
            .child(display_value);
    }

    if props.disabled {
        input = input.opacity(0.5).cursor_not_allowed();
    } else {
        input = input.cursor_text();
    }

    container = container.child(input);

    // Error message
    let error_msg = props.error.clone().or_else(|| ctx.form_state.get_error(&full_id).map(String::from));
    if let Some(error) = error_msg {
        container = container.child(
            div()
                .text_xs()
                .text_color(colors::error())
                .child(error),
        );
    }

    container.into_any_element()
}

/// Render a text area component
pub fn render_text_area<V: 'static + Render>(
    props: &TextAreaProps,
    ctx: &RenderContext<'_, V>,
    _cx: &mut Context<V>,
) -> AnyElement {
    let full_id = ctx.full_id(&props.id);
    let has_error = props.error.is_some() || ctx.form_state.get_error(&full_id).is_some();

    let border_color = if has_error {
        colors::border_error()
    } else {
        colors::border_default()
    };

    let current_value = ctx
        .form_state
        .get_string(&full_id)
        .unwrap_or(&props.value);

    let line_height = 20.0;
    let min_height = (props.rows as f32 * line_height) + 16.0; // padding

    let mut container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .gap_1()
        .w_full();

    // Label
    if let Some(label) = &props.label {
        let mut label_element = div()
            .text_sm()
            .text_color(colors::text_secondary())
            .child(label.clone());

        if props.required {
            label_element = label_element.child(
                div()
                    .text_color(colors::error())
                    .ml_1()
                    .child("*"),
            );
        }

        container = container.child(label_element);
    }

    // Text area
    let mut textarea = div()
        .flex()
        .flex_col()
        .min_h(px(min_height))
        .p_3()
        .bg(colors::bg_input())
        .border_1()
        .border_color(border_color)
        .rounded(px(4.0))
        .text_sm()
        .overflow_hidden();

    if current_value.is_empty() {
        if let Some(placeholder) = &props.placeholder {
            textarea = textarea
                .text_color(colors::text_muted())
                .child(placeholder.clone());
        }
    } else {
        textarea = textarea.text_color(colors::text_primary());
        // Render each line
        for line in current_value.lines() {
            textarea = textarea.child(div().child(line.to_string()));
        }
    }

    if props.disabled {
        textarea = textarea.opacity(0.5).cursor_not_allowed();
    } else {
        textarea = textarea.cursor_text();
    }

    container = container.child(textarea);

    // Error message
    if let Some(error) = &props.error {
        container = container.child(
            div()
                .text_xs()
                .text_color(colors::error())
                .child(error.clone()),
        );
    }

    container.into_any_element()
}

/// Render a select/dropdown component
pub fn render_select<V: 'static + Render>(
    props: &SelectProps,
    ctx: &RenderContext<'_, V>,
    _cx: &mut Context<V>,
) -> AnyElement {
    let full_id = ctx.full_id(&props.id);
    let has_error = props.error.is_some() || ctx.form_state.get_error(&full_id).is_some();

    let border_color = if has_error {
        colors::border_error()
    } else {
        colors::border_default()
    };

    // Get current selection from form state or props
    let current_value = ctx
        .form_state
        .get_string(&full_id)
        .map(|s| s.to_string())
        .or_else(|| props.value.clone());

    // Find the label for the current value
    let display_label = current_value
        .as_ref()
        .and_then(|v| props.options.iter().find(|o| &o.value == v))
        .map(|o| o.label.clone());

    let mut container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .gap_1()
        .w_full();

    // Label
    if let Some(label) = &props.label {
        let mut label_element = div()
            .text_sm()
            .text_color(colors::text_secondary())
            .child(label.clone());

        if props.required {
            label_element = label_element.child(
                div()
                    .text_color(colors::error())
                    .ml_1()
                    .child("*"),
            );
        }

        container = container.child(label_element);
    }

    // Select field (styled div)
    let mut select = div()
        .flex()
        .items_center()
        .justify_between()
        .h(px(36.0))
        .px_3()
        .bg(colors::bg_input())
        .border_1()
        .border_color(border_color)
        .rounded(px(4.0))
        .text_sm();

    // Display value or placeholder
    if let Some(label) = display_label {
        select = select.child(
            div()
                .text_color(colors::text_primary())
                .child(label),
        );
    } else if let Some(placeholder) = &props.placeholder {
        select = select.child(
            div()
                .text_color(colors::text_muted())
                .child(placeholder.clone()),
        );
    }

    // Dropdown arrow
    select = select.child(
        div()
            .text_color(colors::text_secondary())
            .child("\u{25BC}"), // Down arrow
    );

    if props.disabled {
        select = select.opacity(0.5).cursor_not_allowed();
    } else {
        select = select.cursor_pointer();
    }

    container = container.child(select);

    // Error message
    if let Some(error) = &props.error {
        container = container.child(
            div()
                .text_xs()
                .text_color(colors::error())
                .child(error.clone()),
        );
    }

    container.into_any_element()
}

/// Render a toggle/switch component
pub fn render_toggle<V: 'static + Render>(
    props: &ToggleProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let full_id = ctx.full_id(&props.id);
    let action_callback = Arc::clone(&ctx.on_action);
    let component_id = full_id.clone();

    // Get current state from form state or props
    let is_checked = ctx
        .form_state
        .get_bool(&full_id)
        .unwrap_or(props.checked);

    let track_color = if is_checked {
        colors::primary()
    } else {
        colors::bg_input()
    };

    let knob_position = if is_checked { px(18.0) } else { px(2.0) };

    let mut container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .items_center()
        .gap_2();

    // Toggle track
    let mut toggle = div()
        .relative()
        .w(px(40.0))
        .h(px(22.0))
        .bg(track_color)
        .rounded(px(11.0))
        .border_1()
        .border_color(colors::border_default())
        .child(
            // Knob
            div()
                .absolute()
                .top(px(2.0))
                .left(knob_position)
                .w(px(16.0))
                .h(px(16.0))
                .bg(gpui::rgb(0xffffff))
                .rounded_full(),
        );

    if props.disabled {
        toggle = toggle.opacity(0.5).cursor_not_allowed();
    } else {
        let new_checked = !is_checked;
        toggle = toggle
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
                let form_action = FormAction::value_change(component_id.clone(), FormValue::Bool(new_checked));
                action_callback(view, window, cx, form_action);
            }));
    }

    container = container.child(toggle);

    // Label
    if let Some(label) = &props.label {
        container = container.child(
            div()
                .text_sm()
                .text_color(colors::text_primary())
                .child(label.clone()),
        );
    }

    container.into_any_element()
}

/// Render a slider/range component
pub fn render_slider<V: 'static + Render>(
    props: &SliderProps,
    ctx: &RenderContext<'_, V>,
    _cx: &mut Context<V>,
) -> AnyElement {
    let full_id = ctx.full_id(&props.id);

    // Get current value from form state or props
    let current_value = ctx
        .form_state
        .get_number(&full_id)
        .unwrap_or(props.value);

    // Calculate percentage
    let range = props.max - props.min;
    let percentage = if range > 0.0 {
        ((current_value - props.min) / range * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let mut container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .gap_1()
        .w_full();

    // Label with optional value display
    if props.label.is_some() || props.show_value {
        let mut label_row = div()
            .flex()
            .items_center()
            .justify_between()
            .text_sm()
            .text_color(colors::text_secondary());

        if let Some(label) = &props.label {
            label_row = label_row.child(div().child(label.clone()));
        }

        if props.show_value {
            label_row = label_row.child(
                div()
                    .text_color(colors::text_primary())
                    .child(format!("{:.1}", current_value)),
            );
        }

        container = container.child(label_row);
    }

    // Slider track
    let track = div()
        .relative()
        .h(px(6.0))
        .w_full()
        .bg(colors::progress_bg())
        .rounded(px(3.0))
        // Filled portion
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .h_full()
                .bg(colors::primary())
                .rounded(px(3.0))
                .w(gpui::relative(percentage as f32 / 100.0)),
        )
        // Thumb
        .child(
            div()
                .absolute()
                .top(px(-5.0))
                .left(gpui::relative(percentage as f32 / 100.0))
                .w(px(16.0))
                .h(px(16.0))
                .bg(colors::primary())
                .border_2()
                .border_color(gpui::rgb(0xffffff))
                .rounded_full()
                .ml(px(-8.0)), // Center the thumb
        );

    let mut slider_container = div()
        .py_2()
        .child(track);

    if props.disabled {
        slider_container = slider_container.opacity(0.5).cursor_not_allowed();
    } else {
        slider_container = slider_container.cursor_pointer();
    }

    container = container.child(slider_container);

    container.into_any_element()
}

/// Render a checkbox component
pub fn render_checkbox<V: 'static + Render>(
    props: &CheckboxProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let full_id = ctx.full_id(&props.id);
    let action_callback = Arc::clone(&ctx.on_action);
    let component_id = full_id.clone();

    // Get current state from form state or props
    let is_checked = ctx
        .form_state
        .get_bool(&full_id)
        .unwrap_or(props.checked);

    let checkbox_content = if props.indeterminate {
        "\u{2212}" // Minus sign for indeterminate
    } else if is_checked {
        "\u{2713}" // Checkmark
    } else {
        ""
    };

    let bg_color = if is_checked || props.indeterminate {
        colors::primary()
    } else {
        colors::bg_input()
    };

    let mut container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .items_center()
        .gap_2();

    // Checkbox box
    let mut checkbox = div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(18.0))
        .h(px(18.0))
        .bg(bg_color)
        .border_1()
        .border_color(colors::border_default())
        .rounded(px(3.0))
        .text_xs()
        .text_color(gpui::rgb(0xffffff))
        .child(checkbox_content);

    if props.disabled {
        checkbox = checkbox.opacity(0.5).cursor_not_allowed();
    } else {
        let new_checked = !is_checked;
        checkbox = checkbox
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
                let form_action = FormAction::value_change(component_id.clone(), FormValue::Bool(new_checked));
                action_callback(view, window, cx, form_action);
            }));
    }

    container = container.child(checkbox);

    // Label
    container = container.child(
        div()
            .text_sm()
            .text_color(if props.disabled {
                colors::text_muted()
            } else {
                colors::text_primary()
            })
            .child(props.label.clone()),
    );

    container.into_any_element()
}
