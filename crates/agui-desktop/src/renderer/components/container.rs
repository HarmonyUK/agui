//! Container Component Renderers
//!
//! Renders accordion, tabs, modal, drawer, and card components.

use gpui::{div, prelude::*, px, AnyElement, Context, MouseButton, Rgba};
use std::sync::Arc;

use super::colors;
use crate::renderer::form_state::FormAction;
use crate::renderer::schema::{AccordionProps, CardProps, DrawerPosition, DrawerProps, ModalProps, TabsProps};
use crate::renderer::{render_component, RenderContext};

/// Semi-transparent backdrop color
fn backdrop_color() -> Rgba {
    Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.5,
    }
}

/// Render an accordion component
pub fn render_accordion<V: 'static + Render>(
    props: &AccordionProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let mut container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .w_full()
        .border_1()
        .border_color(colors::border_default())
        .rounded(px(4.0))
        .overflow_hidden();

    for (idx, section) in props.sections.iter().enumerate() {
        let is_expanded = props.expanded.contains(&section.id);
        let section_ctx = ctx.with_prefix(&section.id);

        // Section header
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(44.0))
            .px_4()
            .bg(colors::bg_panel())
            .cursor_pointer()
            .hover(|style| style.bg(colors::bg_hover()))
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(colors::text_primary())
                    .child(section.title.clone()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(colors::text_secondary())
                    .child(if is_expanded { "\u{25B2}" } else { "\u{25BC}" }),
            );

        container = container.child(header);

        // Section content (only if expanded)
        if is_expanded {
            let content = div()
                .p_4()
                .bg(colors::bg_dark())
                .border_t_1()
                .border_color(colors::border_default())
                .child(render_component(&section.content, &section_ctx, cx));

            container = container.child(content);
        }

        // Add border between sections (except last)
        if idx < props.sections.len() - 1 && !is_expanded {
            container = container.child(
                div()
                    .h(px(1.0))
                    .w_full()
                    .bg(colors::border_default()),
            );
        }
    }

    container.into_any_element()
}

/// Render a tabs component
pub fn render_tabs<V: 'static + Render>(
    props: &TabsProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    // Determine active tab
    let active_tab_id = props
        .active_tab
        .as_ref()
        .or_else(|| props.tabs.first().map(|t| &t.id))
        .cloned();

    let mut container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .w_full();

    // Tab headers
    let mut tab_bar = div()
        .flex()
        .flex_row()
        .w_full()
        .border_b_1()
        .border_color(colors::border_default())
        .bg(colors::bg_panel());

    for tab in &props.tabs {
        let is_active = active_tab_id.as_ref() == Some(&tab.id);

        let mut tab_header = div()
            .flex()
            .items_center()
            .gap_2()
            .px_4()
            .py_2()
            .text_sm()
            .cursor_pointer();

        // Add icon if present
        if let Some(icon) = &tab.icon {
            tab_header = tab_header.child(
                div()
                    .text_color(colors::text_secondary())
                    .child(icon.clone()),
            );
        }

        tab_header = tab_header.child(
            div()
                .text_color(if is_active {
                    colors::text_primary()
                } else {
                    colors::text_secondary()
                })
                .font_weight(if is_active {
                    gpui::FontWeight::MEDIUM
                } else {
                    gpui::FontWeight::NORMAL
                })
                .child(tab.label.clone()),
        );

        // Active indicator
        if is_active {
            tab_header = tab_header
                .border_b_2()
                .border_color(colors::primary());
        }

        if tab.disabled {
            tab_header = tab_header.opacity(0.5).cursor_not_allowed();
        } else {
            tab_header = tab_header.hover(|style| style.bg(colors::bg_hover()));
        }

        tab_bar = tab_bar.child(tab_header);
    }

    container = container.child(tab_bar);

    // Tab content
    if let Some(active_id) = &active_tab_id {
        if let Some(active_tab) = props.tabs.iter().find(|t| &t.id == active_id) {
            let tab_ctx = ctx.with_prefix(&active_tab.id);
            container = container.child(
                div()
                    .p_4()
                    .child(render_component(&active_tab.content, &tab_ctx, cx)),
            );
        }
    }

    container.into_any_element()
}

/// Render a modal dialog component
pub fn render_modal<V: 'static + Render>(
    props: &ModalProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    if !props.open {
        return div().into_any_element();
    }

    let modal_ctx = ctx.with_prefix(&props.id);
    let action_callback = Arc::clone(&ctx.on_action);
    let close_action = props.close_action.clone();
    let component_id = ctx.full_id(&props.id);

    let modal_width = props.width.unwrap_or(480.0);

    // Backdrop
    let backdrop = div()
        .absolute()
        .inset_0()
        .bg(backdrop_color());

    // Modal container
    let mut modal = div()
        .absolute()
        .top(px(100.0))
        .left_1_2()
        .w(px(modal_width))
        .bg(colors::bg_panel())
        .border_1()
        .border_color(colors::border_default())
        .rounded(px(8.0))
        .shadow_lg()
        .overflow_hidden();

    // Header
    let mut header = div()
        .flex()
        .items_center()
        .justify_between()
        .h(px(48.0))
        .px_4()
        .border_b_1()
        .border_color(colors::border_default())
        .child(
            div()
                .text_base()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(colors::text_primary())
                .child(props.title.clone()),
        );

    // Close button
    if close_action.is_some() {
        let close_action_clone = close_action.clone();
        header = header.child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w(px(28.0))
                .h(px(28.0))
                .rounded(px(4.0))
                .cursor_pointer()
                .text_color(colors::text_secondary())
                .hover(|style| style.bg(colors::bg_hover()))
                .child("\u{2715}")
                .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
                    if let Some(action) = &close_action_clone {
                        let form_action = FormAction::button_click(component_id.clone(), action.clone());
                        action_callback(view, window, cx, form_action);
                    }
                })),
        );
    }

    modal = modal.child(header);

    // Content
    modal = modal.child(
        div()
            .p_4()
            .max_h(px(400.0))
            .overflow_hidden()
            .child(render_component(&props.content, &modal_ctx, cx)),
    );

    // Footer
    if let Some(footer) = &props.footer {
        modal = modal.child(
            div()
                .px_4()
                .py_3()
                .border_t_1()
                .border_color(colors::border_default())
                .bg(colors::bg_elevated())
                .child(render_component(footer, &modal_ctx, cx)),
        );
    }

    // Wrapper
    div()
        .id(gpui::SharedString::from(props.id.clone()))
        .absolute()
        .inset_0()
        .child(backdrop)
        .child(modal)
        .into_any_element()
}

/// Render a drawer component
pub fn render_drawer<V: 'static + Render>(
    props: &DrawerProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    if !props.open {
        return div().into_any_element();
    }

    let drawer_ctx = ctx.with_prefix(&props.id);
    let action_callback = Arc::clone(&ctx.on_action);
    let close_action = props.close_action.clone();
    let component_id = ctx.full_id(&props.id);

    let drawer_size = props.width.unwrap_or(320.0);

    // Backdrop
    let backdrop = div()
        .absolute()
        .inset_0()
        .bg(backdrop_color());

    // Drawer panel position
    let mut drawer = div()
        .absolute()
        .bg(colors::bg_panel())
        .border_color(colors::border_default())
        .flex()
        .flex_col();

    drawer = match props.position {
        DrawerPosition::Right => drawer
            .top_0()
            .right_0()
            .bottom_0()
            .w(px(drawer_size))
            .border_l_1(),
        DrawerPosition::Left => drawer
            .top_0()
            .left_0()
            .bottom_0()
            .w(px(drawer_size))
            .border_r_1(),
        DrawerPosition::Top => drawer
            .top_0()
            .left_0()
            .right_0()
            .h(px(drawer_size))
            .border_b_1(),
        DrawerPosition::Bottom => drawer
            .bottom_0()
            .left_0()
            .right_0()
            .h(px(drawer_size))
            .border_t_1(),
    };

    // Header (if title provided)
    if let Some(title) = &props.title {
        let mut header = div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(48.0))
            .px_4()
            .border_b_1()
            .border_color(colors::border_default())
            .child(
                div()
                    .text_base()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors::text_primary())
                    .child(title.clone()),
            );

        if close_action.is_some() {
            let close_action_clone = close_action.clone();
            header = header.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(28.0))
                    .h(px(28.0))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .text_color(colors::text_secondary())
                    .hover(|style| style.bg(colors::bg_hover()))
                    .child("\u{2715}")
                    .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
                        if let Some(action) = &close_action_clone {
                            let form_action = FormAction::button_click(component_id.clone(), action.clone());
                            action_callback(view, window, cx, form_action);
                        }
                    })),
            );
        }

        drawer = drawer.child(header);
    }

    // Content
    drawer = drawer.child(
        div()
            .flex_1()
            .p_4()
            .overflow_hidden()
            .child(render_component(&props.content, &drawer_ctx, cx)),
    );

    div()
        .id(gpui::SharedString::from(props.id.clone()))
        .absolute()
        .inset_0()
        .child(backdrop)
        .child(drawer)
        .into_any_element()
}

/// Render a card component
pub fn render_card<V: 'static + Render>(
    props: &CardProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let card_ctx = ctx.with_prefix(&props.id);
    let padding = props.padding.unwrap_or(16.0);

    let mut card = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .bg(colors::bg_panel())
        .border_1()
        .border_color(colors::border_default())
        .rounded(px(8.0))
        .overflow_hidden();

    // Custom header
    if let Some(header) = &props.header {
        card = card.child(
            div()
                .p_4()
                .border_b_1()
                .border_color(colors::border_default())
                .child(render_component(header, &card_ctx, cx)),
        );
    }
    // Or title/subtitle header
    else if props.title.is_some() || props.subtitle.is_some() {
        let mut header = div()
            .flex()
            .flex_col()
            .gap_1()
            .p_4()
            .border_b_1()
            .border_color(colors::border_default());

        if let Some(title) = &props.title {
            header = header.child(
                div()
                    .text_base()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors::text_primary())
                    .child(title.clone()),
            );
        }

        if let Some(subtitle) = &props.subtitle {
            header = header.child(
                div()
                    .text_sm()
                    .text_color(colors::text_secondary())
                    .child(subtitle.clone()),
            );
        }

        card = card.child(header);
    }

    // Content
    card = card.child(
        div()
            .p(px(padding))
            .child(render_component(&props.content, &card_ctx, cx)),
    );

    // Footer
    if let Some(footer) = &props.footer {
        card = card.child(
            div()
                .p_4()
                .border_t_1()
                .border_color(colors::border_default())
                .bg(colors::bg_elevated())
                .child(render_component(footer, &card_ctx, cx)),
        );
    }

    card.into_any_element()
}
