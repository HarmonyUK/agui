//! Button Component Renderers
//!
//! Renders buttons and icon buttons with various styles.

use gpui::{div, prelude::*, px, AnyElement, Context, MouseButton, Rgba};
use std::sync::Arc;

use super::colors;
use crate::renderer::form_state::FormAction;
use crate::renderer::schema::{ButtonProps, ButtonVariant, IconButtonProps};
use crate::renderer::RenderContext;

/// Transparent color helper
fn transparent() -> Rgba {
    Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    }
}

/// Render a button component
pub fn render_button<V: 'static + Render>(
    props: &ButtonProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let (bg_color, bg_hover, text_color, border_color) = match props.variant {
        ButtonVariant::Primary => (
            colors::primary(),
            colors::primary_hover(),
            gpui::rgb(0xffffff),
            colors::primary(),
        ),
        ButtonVariant::Secondary => (
            colors::secondary(),
            colors::secondary_hover(),
            colors::text_primary(),
            colors::border_default(),
        ),
        ButtonVariant::Outline => (
            transparent(),
            colors::bg_hover(),
            colors::text_primary(),
            colors::border_default(),
        ),
        ButtonVariant::Ghost => (
            transparent(),
            colors::bg_hover(),
            colors::text_primary(),
            transparent(),
        ),
        ButtonVariant::Destructive => (
            colors::error(),
            gpui::rgb(0xff6b6b),
            gpui::rgb(0xffffff),
            colors::error(),
        ),
    };

    let disabled = props.disabled || props.loading;
    let action_callback = Arc::clone(&ctx.on_action);
    let action = props.action.clone();
    let component_id = ctx.full_id(&props.id);
    let payload = props.payload.clone();

    let mut button = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .items_center()
        .justify_center()
        .gap_2()
        .px_4()
        .py_2()
        .rounded(px(4.0))
        .text_sm()
        .font_weight(gpui::FontWeight::MEDIUM)
        .bg(bg_color)
        .border_1()
        .border_color(border_color)
        .text_color(text_color);

    // Add icon if present
    if let Some(icon) = &props.icon {
        button = button.child(
            div()
                .text_sm()
                .child(icon_to_char(icon)),
        );
    }

    // Add label or loading indicator
    if props.loading {
        button = button.child(
            div()
                .text_sm()
                .child("\u{21BB}"), // Clockwise arrow for loading
        );
    } else {
        button = button.child(props.label.clone());
    }

    if disabled {
        button = button
            .opacity(0.5)
            .cursor_not_allowed();
    } else {
        button = button
            .cursor_pointer()
            .hover(|style| style.bg(bg_hover))
            .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
                let form_action = FormAction::button_click(component_id.clone(), action.clone())
                    .with_payload(payload.clone().unwrap_or(serde_json::Value::Null));
                action_callback(view, window, cx, form_action);
            }));
    }

    button.into_any_element()
}

/// Render an icon button component
pub fn render_icon_button<V: 'static + Render>(
    props: &IconButtonProps,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let (bg_color, bg_hover, text_color) = match props.variant {
        ButtonVariant::Primary => (colors::primary(), colors::primary_hover(), gpui::rgb(0xffffff)),
        ButtonVariant::Secondary => (colors::secondary(), colors::secondary_hover(), colors::text_primary()),
        ButtonVariant::Outline => (transparent(), colors::bg_hover(), colors::text_primary()),
        ButtonVariant::Ghost => (transparent(), colors::bg_hover(), colors::text_primary()),
        ButtonVariant::Destructive => (colors::error(), gpui::rgb(0xff6b6b), gpui::rgb(0xffffff)),
    };

    let action_callback = Arc::clone(&ctx.on_action);
    let action = props.action.clone();
    let component_id = ctx.full_id(&props.id);
    let payload = props.payload.clone();

    let mut button = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .items_center()
        .justify_center()
        .w(px(32.0))
        .h(px(32.0))
        .rounded(px(4.0))
        .text_sm()
        .bg(bg_color)
        .text_color(text_color)
        .child(icon_to_char(&props.icon));

    if props.disabled {
        button = button
            .opacity(0.5)
            .cursor_not_allowed();
    } else {
        button = button
            .cursor_pointer()
            .hover(|style| style.bg(bg_hover))
            .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
                let form_action = FormAction::button_click(component_id.clone(), action.clone())
                    .with_payload(payload.clone().unwrap_or(serde_json::Value::Null));
                action_callback(view, window, cx, form_action);
            }));
    }

    // Add tooltip if provided (simplified - just title attribute effect)
    if let Some(_tooltip) = &props.tooltip {
        // In a full implementation, this would show a hover tooltip
        // For now, we just render the button
    }

    button.into_any_element()
}

/// Convert icon name to a unicode character (simplified icon set)
fn icon_to_char(icon: &str) -> &'static str {
    match icon {
        // Common actions
        "close" | "x" => "\u{2715}",
        "check" | "checkmark" => "\u{2713}",
        "plus" | "add" => "\u{002B}",
        "minus" | "remove" => "\u{2212}",
        "edit" | "pencil" => "\u{270E}",
        "delete" | "trash" => "\u{1F5D1}",
        "search" => "\u{1F50D}",
        "settings" | "gear" => "\u{2699}",
        "menu" | "hamburger" => "\u{2630}",

        // Navigation
        "arrow-left" | "back" => "\u{2190}",
        "arrow-right" | "forward" => "\u{2192}",
        "arrow-up" => "\u{2191}",
        "arrow-down" => "\u{2193}",
        "chevron-left" => "\u{2039}",
        "chevron-right" => "\u{203A}",
        "chevron-up" => "\u{2303}",
        "chevron-down" => "\u{2304}",

        // Files
        "file" => "\u{1F4C4}",
        "folder" => "\u{1F4C1}",
        "folder-open" => "\u{1F4C2}",
        "copy" => "\u{1F4CB}",
        "paste" => "\u{1F4CB}",
        "save" => "\u{1F4BE}",

        // Status
        "info" => "\u{2139}",
        "warning" => "\u{26A0}",
        "error" | "alert" => "\u{26A0}",
        "success" => "\u{2713}",

        // Media
        "play" => "\u{25B6}",
        "pause" => "\u{23F8}",
        "stop" => "\u{23F9}",
        "refresh" | "reload" => "\u{21BB}",

        // Communication
        "send" => "\u{27A4}",
        "mail" | "email" => "\u{2709}",
        "chat" | "message" => "\u{1F4AC}",

        // User
        "user" | "person" => "\u{1F464}",
        "users" | "people" => "\u{1F465}",

        // Misc
        "star" => "\u{2605}",
        "heart" => "\u{2665}",
        "lock" => "\u{1F512}",
        "unlock" => "\u{1F513}",
        "link" => "\u{1F517}",
        "external" => "\u{2197}",
        "download" => "\u{2B07}",
        "upload" => "\u{2B06}",

        // Default
        _ => "\u{25CF}", // Bullet point as fallback
    }
}
