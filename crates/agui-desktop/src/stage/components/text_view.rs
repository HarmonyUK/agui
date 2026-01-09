//! Text View Component
//!
//! Renders text/code content with line numbers and syntax highlighting.

use gpui::{div, prelude::*, px, rgb, Div, Pixels};

use super::colors;
use crate::stage::syntax::{SyntaxHighlighter, Token};

/// Configuration for text view rendering
#[derive(Debug, Clone)]
pub struct TextViewConfig {
    /// Show line numbers
    pub show_line_numbers: bool,
    /// Font size in pixels
    pub font_size: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Tab width in spaces
    pub tab_width: usize,
    /// Word wrap enabled
    pub word_wrap: bool,
    /// Current cursor line (0-indexed, None for no cursor)
    pub cursor_line: Option<usize>,
    /// Selection range (start_line, start_col, end_line, end_col)
    pub selection: Option<(usize, usize, usize, usize)>,
    /// First visible line (for virtualization)
    pub first_visible_line: usize,
    /// Number of visible lines
    pub visible_lines: usize,
    /// Read-only mode
    pub read_only: bool,
}

impl Default for TextViewConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            font_size: 13.0,
            line_height: 1.4,
            tab_width: 4,
            word_wrap: false,
            cursor_line: None,
            selection: None,
            first_visible_line: 0,
            visible_lines: 50,
            read_only: false,
        }
    }
}

impl TextViewConfig {
    /// Calculate line height in pixels
    pub fn line_height_px(&self) -> Pixels {
        px(self.font_size * self.line_height)
    }

    /// Calculate gutter width based on line count
    pub fn gutter_width(&self, total_lines: usize) -> Pixels {
        if !self.show_line_numbers {
            return px(0.0);
        }
        // Calculate width needed for line numbers
        let digits = (total_lines as f32).log10().ceil() as usize;
        let digit_width = self.font_size * 0.6; // Approximate monospace digit width
        px(digit_width * (digits.max(2) as f32) + 24.0) // Add padding
    }
}

/// Render a text view with syntax highlighting
pub fn render_text_view(
    content: &str,
    _language: Option<&str>,
    config: &TextViewConfig,
    highlighter: Option<&SyntaxHighlighter>,
) -> Div {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let gutter_width = config.gutter_width(total_lines);
    let line_height = config.line_height_px();

    // Determine visible range
    let start_line = config.first_visible_line;
    let end_line = (start_line + config.visible_lines).min(total_lines);
    let visible_lines = &lines[start_line..end_line];

    // Pre-highlight all visible lines if highlighter is available
    let highlighted: Vec<Vec<Token>> = if let Some(hl) = highlighter {
        visible_lines.iter().map(|line| hl.highlight_line(line)).collect()
    } else {
        Vec::new()
    };

    // Build the view
    div()
        .flex()
        .flex_row()
        .w_full()
        .h_full()
        .bg(rgb(colors::EDITOR_BG))
        .overflow_hidden()
        // Gutter (line numbers)
        .when(config.show_line_numbers, |el| {
            el.child(
                div()
                    .flex()
                    .flex_col()
                    .w(gutter_width)
                    .h_full()
                    .bg(rgb(colors::GUTTER_BG))
                    .border_r_1()
                    .border_color(rgb(0x303030))
                    .children(
                        (start_line..end_line).map(|line_num| {
                            let is_current = config.cursor_line == Some(line_num);
                            div()
                                .h(line_height)
                                .flex()
                                .items_center()
                                .justify_end()
                                .pr_2()
                                .text_xs()
                                .font_family("monospace")
                                .text_color(rgb(if is_current {
                                    colors::LINE_NUMBER_ACTIVE
                                } else {
                                    colors::LINE_NUMBER
                                }))
                                .when(is_current, |el| el.bg(rgb(colors::CURRENT_LINE_BG)))
                                .child(format!("{}", line_num + 1))
                        })
                    )
            )
        })
        // Content area
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .h_full()
                .overflow_hidden()
                .pl_2()
                .children(
                    visible_lines.iter().enumerate().map(|(idx, line)| {
                        let line_num = start_line + idx;
                        let is_current = config.cursor_line == Some(line_num);
                        let tokens = highlighted.get(idx);

                        render_line(
                            line,
                            line_num,
                            is_current,
                            tokens,
                            line_height,
                            config.font_size,
                            config.tab_width,
                        )
                    })
                )
        )
}

/// Render a single line with optional syntax highlighting
fn render_line(
    line: &str,
    _line_num: usize,
    is_current: bool,
    tokens: Option<&Vec<Token>>,
    line_height: Pixels,
    font_size: f32,
    tab_width: usize,
) -> Div {
    let line_content = if line.is_empty() {
        // Empty line - add a space to maintain height
        " ".to_string()
    } else {
        // Replace tabs with spaces
        line.replace('\t', &" ".repeat(tab_width))
    };

    let base = div()
        .h(line_height)
        .flex()
        .items_center()
        .w_full()
        .when(is_current, |el| el.bg(rgb(colors::CURRENT_LINE_BG)));

    if let Some(tokens) = tokens {
        if tokens.is_empty() {
            // No tokens - render plain
            base.child(
                div()
                    .text_sm()
                    .font_family("monospace")
                    .text_color(rgb(0xcccccc))
                    .child(line_content)
            )
        } else {
            // Render with syntax highlighting
            base.child(render_highlighted_line(&line_content, tokens, font_size))
        }
    } else {
        // No highlighter - render plain
        base.child(
            div()
                .text_sm()
                .font_family("monospace")
                .text_color(rgb(0xcccccc))
                .child(line_content)
        )
    }
}

/// Render a line with syntax highlighting tokens
fn render_highlighted_line(line: &str, tokens: &[Token], _font_size: f32) -> Div {
    let chars: Vec<char> = line.chars().collect();
    let mut segments: Vec<(String, u32)> = Vec::new();
    let mut last_end = 0;

    for token in tokens {
        // Add any text before this token as plain text
        if token.start > last_end {
            let text: String = chars[last_end..token.start].iter().collect();
            if !text.is_empty() {
                segments.push((text, 0xcccccc));
            }
        }

        // Add the token
        let end = token.end.min(chars.len());
        let text: String = chars[token.start..end].iter().collect();
        if !text.is_empty() {
            segments.push((text, token.token_type.color()));
        }

        last_end = end;
    }

    // Add any remaining text
    if last_end < chars.len() {
        let text: String = chars[last_end..].iter().collect();
        if !text.is_empty() {
            segments.push((text, 0xcccccc));
        }
    }

    // If no segments, render the whole line
    if segments.is_empty() {
        return div()
            .text_sm()
            .font_family("monospace")
            .text_color(rgb(0xcccccc))
            .child(line.to_string());
    }

    div()
        .flex()
        .flex_row()
        .font_family("monospace")
        .text_sm()
        .children(
            segments.into_iter().map(|(text, color)| {
                div().text_color(rgb(color)).child(text)
            })
        )
}

/// Render a simple code block (for use in other components)
pub fn render_code_block(code: &str, language: Option<&str>) -> Div {
    let highlighter = language.map(SyntaxHighlighter::new);
    let lines: Vec<&str> = code.lines().collect();

    let highlighted: Vec<Vec<Token>> = if let Some(ref hl) = highlighter {
        lines.iter().map(|line| hl.highlight_line(line)).collect()
    } else {
        Vec::new()
    };

    div()
        .flex()
        .flex_col()
        .w_full()
        .bg(rgb(0x1e1e1e))
        .rounded_md()
        .p_2()
        .overflow_hidden()
        .children(
            lines.iter().enumerate().map(|(idx, line)| {
                let tokens = highlighted.get(idx);
                let line_content = if line.is_empty() { " " } else { *line };

                if let Some(tokens) = tokens {
                    if !tokens.is_empty() {
                        return render_highlighted_line(line_content, tokens, 13.0);
                    }
                }

                div()
                    .text_sm()
                    .font_family("monospace")
                    .text_color(rgb(0xcccccc))
                    .child(line_content.to_string())
            })
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = TextViewConfig::default();
        assert!(config.show_line_numbers);
        assert!(!config.word_wrap);
        assert!(config.read_only == false);
    }

    #[test]
    fn test_gutter_width() {
        let config = TextViewConfig::default();

        // Small file
        let width = config.gutter_width(10);
        assert!(width > px(0.0));

        // Large file should have wider gutter
        let width_large = config.gutter_width(10000);
        assert!(width_large > width);
    }

    #[test]
    fn test_line_height() {
        let config = TextViewConfig {
            font_size: 14.0,
            line_height: 1.5,
            ..Default::default()
        };

        let height = config.line_height_px();
        assert_eq!(height, px(21.0));
    }
}
