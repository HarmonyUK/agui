//! Text Component Renderers
//!
//! Renders text, markdown, headers, and code blocks.

use gpui::{div, prelude::*, px, AnyElement, Context};

use super::colors;
use crate::renderer::schema::{CodeProps, FontWeight, HeaderProps, MarkdownProps, TextProps, TextSize, TextAlign};
use crate::renderer::RenderContext;

/// Render a plain text component
pub fn render_text<V: 'static + Render>(
    props: &TextProps,
    _ctx: &RenderContext<'_, V>,
    _cx: &mut Context<V>,
) -> AnyElement {
    let font_size = match props.size {
        TextSize::Xs => px(10.0),
        TextSize::Sm => px(12.0),
        TextSize::Md => px(14.0),
        TextSize::Lg => px(16.0),
        TextSize::Xl => px(20.0),
        TextSize::Xxl => px(24.0),
    };

    let font_weight = match props.weight {
        FontWeight::Light => gpui::FontWeight::LIGHT,
        FontWeight::Normal => gpui::FontWeight::NORMAL,
        FontWeight::Medium => gpui::FontWeight::MEDIUM,
        FontWeight::Semibold => gpui::FontWeight::SEMIBOLD,
        FontWeight::Bold => gpui::FontWeight::BOLD,
    };

    let text_color = props
        .color
        .as_ref()
        .and_then(|c| parse_color(c))
        .unwrap_or_else(colors::text_primary);

    let mut element = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .text_size(font_size)
        .font_weight(font_weight)
        .text_color(text_color)
        .child(props.content.clone());

    // Apply text alignment
    element = match props.align {
        TextAlign::Left => element,
        TextAlign::Center => element.items_center().justify_center(),
        TextAlign::Right => element.items_end().justify_end(),
    };

    element.into_any_element()
}

/// Render a markdown component (simplified - renders as styled text blocks)
pub fn render_markdown<V: 'static + Render>(
    props: &MarkdownProps,
    _ctx: &RenderContext<'_, V>,
    _cx: &mut Context<V>,
) -> AnyElement {
    // Parse markdown into blocks (simplified implementation)
    let blocks = parse_markdown_simple(&props.content);

    let mut container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .gap_2();

    if let Some(max_height) = props.max_height {
        container = container.max_h(px(max_height)).overflow_y_scroll();
    }

    for block in blocks {
        container = container.child(block);
    }

    container.into_any_element()
}

/// Simple markdown parser that creates styled div elements
fn parse_markdown_simple(content: &str) -> Vec<gpui::Div> {
    let mut blocks = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            // Empty line - add spacer
            blocks.push(div().h(px(8.0)));
        } else if trimmed.starts_with("# ") {
            // H1
            blocks.push(
                div()
                    .text_size(px(24.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(colors::text_primary())
                    .mb_2()
                    .child(trimmed[2..].to_string()),
            );
        } else if trimmed.starts_with("## ") {
            // H2
            blocks.push(
                div()
                    .text_size(px(20.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(colors::text_primary())
                    .mb_2()
                    .child(trimmed[3..].to_string()),
            );
        } else if trimmed.starts_with("### ") {
            // H3
            blocks.push(
                div()
                    .text_size(px(16.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors::text_primary())
                    .mb_1()
                    .child(trimmed[4..].to_string()),
            );
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            // List item
            blocks.push(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .text_size(px(14.0))
                    .text_color(colors::text_primary())
                    .child(div().child("\u{2022}").text_color(colors::text_secondary()))
                    .child(div().child(trimmed[2..].to_string())),
            );
        } else if trimmed.starts_with("> ") {
            // Blockquote
            blocks.push(
                div()
                    .border_l_2()
                    .border_color(colors::border_default())
                    .pl_3()
                    .text_color(colors::text_secondary())
                    .text_size(px(14.0))
                    .child(trimmed[2..].to_string()),
            );
        } else if trimmed.starts_with("```") {
            // Code block start/end - skip for now (would need multi-line handling)
            continue;
        } else if trimmed.starts_with("`") && trimmed.ends_with("`") && trimmed.len() > 2 {
            // Inline code
            blocks.push(
                div()
                    .text_size(px(14.0))
                    .text_color(colors::text_primary())
                    .child(
                        div()
                            .bg(colors::bg_input())
                            .rounded(px(3.0))
                            .px_1()
                            .child(trimmed[1..trimmed.len() - 1].to_string()),
                    ),
            );
        } else {
            // Regular paragraph
            blocks.push(
                div()
                    .text_size(px(14.0))
                    .text_color(colors::text_primary())
                    .child(line.to_string()),
            );
        }
    }

    blocks
}

/// Render a header component
pub fn render_header<V: 'static + Render>(
    props: &HeaderProps,
    _ctx: &RenderContext<'_, V>,
    _cx: &mut Context<V>,
) -> AnyElement {
    let (font_size, margin_bottom) = match props.level {
        1 => (px(28.0), px(16.0)),
        2 => (px(24.0), px(12.0)),
        3 => (px(20.0), px(10.0)),
        4 => (px(18.0), px(8.0)),
        5 => (px(16.0), px(6.0)),
        _ => (px(14.0), px(4.0)),
    };

    div()
        .id(gpui::SharedString::from(props.id.clone()))
        .text_size(font_size)
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(colors::text_primary())
        .mb(margin_bottom)
        .child(props.content.clone())
        .into_any_element()
}

/// Render a code block component
pub fn render_code<V: 'static + Render>(
    props: &CodeProps,
    _ctx: &RenderContext<'_, V>,
    _cx: &mut Context<V>,
) -> AnyElement {
    let lines: Vec<_> = props.content.lines().collect();

    let mut code_container = div()
        .id(gpui::SharedString::from(props.id.clone()))
        .flex()
        .flex_col()
        .bg(colors::bg_elevated())
        .border_1()
        .border_color(colors::border_default())
        .rounded(px(4.0))
        .p_2()
        .overflow_x_scroll();

    // Add language label if provided
    if let Some(lang) = &props.language {
        code_container = code_container.child(
            div()
                .text_xs()
                .text_color(colors::text_muted())
                .mb_2()
                .child(lang.clone()),
        );
    }

    // Render lines
    for (idx, line) in lines.iter().enumerate() {
        let line_number = idx + 1;

        let line_element = if props.line_numbers {
            div()
                .flex()
                .flex_row()
                .gap_3()
                .child(
                    div()
                        .w(px(32.0))
                        .text_xs()
                        .text_color(colors::text_muted())
                        .text_right()
                        .child(format!("{}", line_number)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors::text_primary())
                        .child((*line).to_string()),
                )
        } else {
            div()
                .text_sm()
                .text_color(colors::text_primary())
                .child((*line).to_string())
        };

        code_container = code_container.child(line_element);
    }

    code_container.into_any_element()
}

/// Parse a color string (hex format)
fn parse_color(color: &str) -> Option<gpui::Rgba> {
    let hex = color.trim_start_matches('#');
    if hex.len() == 6 {
        // Parse hex string to u32
        let hex_val = u32::from_str_radix(hex, 16).ok()?;
        Some(gpui::rgb(hex_val))
    } else {
        None
    }
}
