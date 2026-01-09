//! Layout Component Renderers
//!
//! Renders row, column, stack, spacer, and divider components.

use gpui::{div, prelude::*, px, AnyElement, Context};

use super::colors;
use crate::renderer::schema::{AlignItems, ColumnProps, DividerProps, JustifyContent, RowProps, SpacerProps, StackProps};
use crate::renderer::{render_component, RenderContext};

/// Render a row (horizontal) layout component
pub fn render_row<V: 'static + Render>(
    props: &RowProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let row_ctx = ctx.with_prefix(&props.id);

    let mut row = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_row();

    // Gap
    if let Some(gap) = props.gap {
        row = row.gap(px(gap));
    }

    // Alignment
    row = match props.align {
        AlignItems::Start => row.items_start(),
        AlignItems::Center => row.items_center(),
        AlignItems::End => row.items_end(),
        AlignItems::Stretch => row,
    };

    // Justification
    row = match props.justify {
        JustifyContent::Start => row.justify_start(),
        JustifyContent::Center => row.justify_center(),
        JustifyContent::End => row.justify_end(),
        JustifyContent::SpaceBetween => row.justify_between(),
        JustifyContent::SpaceAround => row.justify_around(),
        JustifyContent::SpaceEvenly => row.justify_around() // gpui doesn't have justify_evenly,
    };

    // Wrap
    if props.wrap {
        row = row.flex_wrap();
    }

    // Render children
    for (idx, child) in props.children.iter().enumerate() {
        let child_ctx = row_ctx.with_prefix(&format!("{}", idx));
        row = row.child(render_component(child, &child_ctx, cx));
    }

    row.into_any_element()
}

/// Render a column (vertical) layout component
pub fn render_column<V: 'static + Render>(
    props: &ColumnProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let col_ctx = ctx.with_prefix(&props.id);

    let mut col = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col();

    // Gap
    if let Some(gap) = props.gap {
        col = col.gap(px(gap));
    }

    // Alignment
    col = match props.align {
        AlignItems::Start => col.items_start(),
        AlignItems::Center => col.items_center(),
        AlignItems::End => col.items_end(),
        AlignItems::Stretch => col,
    };

    // Justification
    col = match props.justify {
        JustifyContent::Start => col.justify_start(),
        JustifyContent::Center => col.justify_center(),
        JustifyContent::End => col.justify_end(),
        JustifyContent::SpaceBetween => col.justify_between(),
        JustifyContent::SpaceAround => col.justify_around(),
        JustifyContent::SpaceEvenly => col.justify_around() // gpui doesn't have justify_evenly,
    };

    // Render children
    for (idx, child) in props.children.iter().enumerate() {
        let child_ctx = col_ctx.with_prefix(&format!("{}", idx));
        col = col.child(render_component(child, &child_ctx, cx));
    }

    col.into_any_element()
}

/// Render a stack (layered) layout component
pub fn render_stack<V: 'static + Render>(
    props: &StackProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let stack_ctx = ctx.with_prefix(&props.id);

    let mut stack = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .relative();

    // Alignment for positioned children
    stack = match props.align {
        AlignItems::Start => stack.items_start(),
        AlignItems::Center => stack.items_center(),
        AlignItems::End => stack.items_end(),
        AlignItems::Stretch => stack,
    };

    // Render children - each absolutely positioned on top of each other
    for (idx, child) in props.children.iter().enumerate() {
        let child_ctx = stack_ctx.with_prefix(&format!("{}", idx));
        let child_element = render_component(child, &child_ctx, cx);

        // First child is the base, others are overlaid
        if idx == 0 {
            stack = stack.child(child_element);
        } else {
            stack = stack.child(
                div()
                    .absolute()
                    .inset_0()
                    .child(child_element),
            );
        }
    }

    stack.into_any_element()
}

/// Render a spacer component
pub fn render_spacer<V: 'static + Render>(
    props: &SpacerProps,
    _ctx: &RenderContext<'_, V>,
    _cx: &mut Context<V>,
) -> AnyElement {
    let mut spacer = div().id(gpui::SharedString::from(props.id.clone()));

    if props.flex {
        spacer = spacer.flex_1();
    } else {
        if let Some(width) = props.width {
            spacer = spacer.w(px(width));
        }
        if let Some(height) = props.height {
            spacer = spacer.h(px(height));
        }
    }

    spacer.into_any_element()
}

/// Render a divider component
pub fn render_divider<V: 'static + Render>(
    props: &DividerProps,
    _ctx: &RenderContext<'_, V>,
    _cx: &mut Context<V>,
) -> AnyElement {
    let margin = props.margin.unwrap_or(8.0);

    let divider = if props.vertical {
        div()
            .id(gpui::SharedString::from(props.id.clone()))
            .w(px(1.0))
            .h_full()
            .bg(colors::border_default())
            .mx(px(margin))
    } else {
        div()
            .id(gpui::SharedString::from(props.id.clone()))
            .h(px(1.0))
            .w_full()
            .bg(colors::border_default())
            .my(px(margin))
    };

    divider.into_any_element()
}
