//! Diff View Component
//!
//! Renders diff content in unified or side-by-side format.

use gpui::{div, prelude::*, px, rgb, Div, Pixels};

use super::colors;
use crate::stage::diff::{diff_stats, DiffStats};
use crate::stage::types::{DiffContent, DiffHunk, DiffLine, DiffLineType, ViewMode};

/// Configuration for diff view rendering
#[derive(Debug, Clone)]
pub struct DiffViewConfig {
    /// View mode (unified or side-by-side)
    pub view_mode: ViewMode,
    /// Show line numbers
    pub show_line_numbers: bool,
    /// Font size in pixels
    pub font_size: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Context lines to show around changes
    pub context_lines: usize,
    /// First visible line (for virtualization)
    pub first_visible_line: usize,
    /// Number of visible lines
    pub visible_lines: usize,
}

impl Default for DiffViewConfig {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Unified,
            show_line_numbers: true,
            font_size: 13.0,
            line_height: 1.4,
            context_lines: 3,
            first_visible_line: 0,
            visible_lines: 50,
        }
    }
}

impl DiffViewConfig {
    pub fn line_height_px(&self) -> Pixels {
        px(self.font_size * self.line_height)
    }
}

/// Render a diff view
pub fn render_diff_view(diff: &DiffContent, config: &DiffViewConfig) -> Div {
    match config.view_mode {
        ViewMode::Unified | ViewMode::InlineChanges => render_unified_diff(diff, config),
        ViewMode::SideBySide => render_side_by_side_diff(diff, config),
        ViewMode::Normal => {
            // For normal mode, just render as text (shouldn't reach here usually)
            render_unified_diff(diff, config)
        }
    }
}

/// Render unified diff view
fn render_unified_diff(diff: &DiffContent, config: &DiffViewConfig) -> Div {
    let line_height = config.line_height_px();
    let stats = diff_stats(&diff.hunks);

    div()
        .flex()
        .flex_col()
        .w_full()
        .h_full()
        .bg(rgb(colors::EDITOR_BG))
        // Stats header
        .child(render_diff_stats_header(&stats))
        // Hunks
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .overflow_hidden()
                .children(
                    diff.hunks.iter().map(|hunk| {
                        render_unified_hunk(hunk, config.show_line_numbers, line_height)
                    })
                )
        )
}

/// Render side-by-side diff view
fn render_side_by_side_diff(diff: &DiffContent, config: &DiffViewConfig) -> Div {
    let line_height = config.line_height_px();
    let stats = diff_stats(&diff.hunks);

    div()
        .flex()
        .flex_col()
        .w_full()
        .h_full()
        .bg(rgb(colors::EDITOR_BG))
        // Stats header
        .child(render_diff_stats_header(&stats))
        // Side-by-side panels
        .child(
            div()
                .flex()
                .flex_row()
                .flex_1()
                .overflow_hidden()
                // Left side (original) - use flex_1 for equal width
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .h_full()
                        .border_r_1()
                        .border_color(rgb(0x404040))
                        .overflow_hidden()
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .bg(rgb(0x252526))
                                .text_xs()
                                .text_color(rgb(0x808080))
                                .child("Original")
                        )
                        .children(
                            diff.hunks.iter().map(|hunk| {
                                render_side_hunk(hunk, true, config.show_line_numbers, line_height)
                            })
                        )
                )
                // Right side (modified) - use flex_1 for equal width
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .h_full()
                        .overflow_hidden()
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .bg(rgb(0x252526))
                                .text_xs()
                                .text_color(rgb(0x808080))
                                .child("Modified")
                        )
                        .children(
                            diff.hunks.iter().map(|hunk| {
                                render_side_hunk(hunk, false, config.show_line_numbers, line_height)
                            })
                        )
                )
        )
}

/// Render diff statistics header
fn render_diff_stats_header(stats: &DiffStats) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_4()
        .px_3()
        .py_2()
        .bg(rgb(0x252526))
        .border_b_1()
        .border_color(rgb(0x404040))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(0x4ec9b0))
                        .child(format!("+{}", stats.additions))
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(0xf14c4c))
                        .child(format!("-{}", stats.deletions))
                )
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x808080))
                .child(format!("{} hunks", stats.hunks))
        )
}

/// Render a single hunk in unified format
fn render_unified_hunk(hunk: &DiffHunk, show_line_numbers: bool, line_height: Pixels) -> Div {
    div()
        .flex()
        .flex_col()
        .w_full()
        .mb_2()
        // Hunk header
        .child(
            div()
                .w_full()
                .px_2()
                .py_1()
                .bg(rgb(colors::DIFF_HUNK_HEADER))
                .text_xs()
                .font_family("monospace")
                .text_color(rgb(0x808080))
                .child(format!(
                    "@@ -{},{} +{},{} @@",
                    hunk.old_start + 1,
                    hunk.old_count,
                    hunk.new_start + 1,
                    hunk.new_count
                ))
        )
        // Lines
        .children(
            hunk.lines.iter().map(|line| {
                render_unified_line(line, show_line_numbers, line_height)
            })
        )
}

/// Render a single line in unified diff
fn render_unified_line(line: &DiffLine, show_line_numbers: bool, line_height: Pixels) -> Div {
    let (bg_color, prefix, text_color) = match line.line_type {
        DiffLineType::Addition => (colors::DIFF_ADD_BG, "+", 0x4ec9b0),
        DiffLineType::Deletion => (colors::DIFF_DEL_BG, "-", 0xf14c4c),
        DiffLineType::Context => (colors::DIFF_CONTEXT, " ", 0xcccccc),
        DiffLineType::Header => (colors::DIFF_HUNK_HEADER, "@", 0x808080),
    };

    let gutter_bg = match line.line_type {
        DiffLineType::Addition => colors::DIFF_ADD_GUTTER,
        DiffLineType::Deletion => colors::DIFF_DEL_GUTTER,
        _ => colors::GUTTER_BG,
    };

    div()
        .flex()
        .flex_row()
        .w_full()
        .h(line_height)
        .bg(rgb(bg_color))
        // Line numbers gutter
        .when(show_line_numbers, |el| {
            el.child(
                div()
                    .flex()
                    .flex_row()
                    .h_full()
                    .bg(rgb(gutter_bg))
                    // Old line number
                    .child(
                        div()
                            .w(px(40.0))
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_end()
                            .pr_1()
                            .text_xs()
                            .font_family("monospace")
                            .text_color(rgb(colors::LINE_NUMBER))
                            .child(
                                line.old_line
                                    .map(|n| format!("{}", n + 1))
                                    .unwrap_or_default()
                            )
                    )
                    // New line number
                    .child(
                        div()
                            .w(px(40.0))
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_end()
                            .pr_2()
                            .border_r_1()
                            .border_color(rgb(0x404040))
                            .text_xs()
                            .font_family("monospace")
                            .text_color(rgb(colors::LINE_NUMBER))
                            .child(
                                line.new_line
                                    .map(|n| format!("{}", n + 1))
                                    .unwrap_or_default()
                            )
                    )
            )
        })
        // Prefix (+, -, or space)
        .child(
            div()
                .w(px(16.0))
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .font_family("monospace")
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(text_color))
                .child(prefix)
        )
        // Content
        .child(
            div()
                .flex_1()
                .h_full()
                .flex()
                .items_center()
                .pl_1()
                .text_sm()
                .font_family("monospace")
                .text_color(rgb(text_color))
                .child(line.content.clone())
        )
}

/// Render a hunk for one side of side-by-side view
fn render_side_hunk(
    hunk: &DiffHunk,
    is_original: bool,
    show_line_numbers: bool,
    line_height: Pixels,
) -> Div {
    div()
        .flex()
        .flex_col()
        .w_full()
        .children(
            hunk.lines.iter().filter_map(|line| {
                // Filter lines based on which side we're rendering
                let should_show = match line.line_type {
                    DiffLineType::Addition => !is_original,
                    DiffLineType::Deletion => is_original,
                    DiffLineType::Context => true,
                    DiffLineType::Header => false,
                };

                if should_show {
                    Some(render_side_line(line, is_original, show_line_numbers, line_height))
                } else {
                    // Add empty placeholder to keep sides aligned
                    if (is_original && line.line_type == DiffLineType::Addition)
                        || (!is_original && line.line_type == DiffLineType::Deletion)
                    {
                        Some(render_empty_line(line_height))
                    } else {
                        None
                    }
                }
            })
        )
}

/// Render a line for side-by-side view
fn render_side_line(
    line: &DiffLine,
    is_original: bool,
    show_line_numbers: bool,
    line_height: Pixels,
) -> Div {
    let (bg_color, text_color) = match line.line_type {
        DiffLineType::Addition => (colors::DIFF_ADD_BG, 0x4ec9b0),
        DiffLineType::Deletion => (colors::DIFF_DEL_BG, 0xf14c4c),
        DiffLineType::Context => (colors::DIFF_CONTEXT, 0xcccccc),
        DiffLineType::Header => (colors::DIFF_HUNK_HEADER, 0x808080),
    };

    let line_num = if is_original {
        line.old_line
    } else {
        line.new_line
    };

    div()
        .flex()
        .flex_row()
        .w_full()
        .h(line_height)
        .bg(rgb(bg_color))
        // Line number
        .when(show_line_numbers, |el| {
            el.child(
                div()
                    .w(px(40.0))
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_end()
                    .pr_2()
                    .border_r_1()
                    .border_color(rgb(0x404040))
                    .text_xs()
                    .font_family("monospace")
                    .text_color(rgb(colors::LINE_NUMBER))
                    .child(line_num.map(|n| format!("{}", n + 1)).unwrap_or_default())
            )
        })
        // Content
        .child(
            div()
                .flex_1()
                .h_full()
                .flex()
                .items_center()
                .pl_2()
                .text_sm()
                .font_family("monospace")
                .text_color(rgb(text_color))
                .child(line.content.clone())
        )
}

/// Render an empty placeholder line (for alignment)
fn render_empty_line(line_height: Pixels) -> Div {
    div()
        .w_full()
        .h(line_height)
        .bg(rgb(0x252526))
}

/// Render inline changes (highlight changes within text)
pub fn render_inline_changes(
    original: &str,
    modified: &str,
    config: &DiffViewConfig,
) -> Div {
    let diff = crate::stage::types::DiffContent::new(original, modified);
    render_unified_diff(&diff, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = DiffViewConfig::default();
        assert_eq!(config.view_mode, ViewMode::Unified);
        assert!(config.show_line_numbers);
    }

    #[test]
    fn test_line_height() {
        let config = DiffViewConfig {
            font_size: 14.0,
            line_height: 1.5,
            ..Default::default()
        };
        assert_eq!(config.line_height_px(), px(21.0));
    }
}
