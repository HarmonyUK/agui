//! Artifact View Component
//!
//! Main component that renders an artifact with appropriate view based on content type.

use gpui::{div, prelude::*, px, rgb, Div};

use super::colors;
use super::diff_view::{render_diff_view, DiffViewConfig};
use super::tabs::{render_empty_tabs, render_tab_bar_simple, TabItem};
use super::text_view::{render_text_view, TextViewConfig};
use crate::stage::state::StageState;
use crate::stage::syntax::SyntaxHighlighter;
use crate::stage::types::{Artifact, ArtifactContent, ContentType, ViewMode};

/// Render the complete Stage pane with tabs and artifact content
pub fn render_stage_pane(state: &mut StageState) -> Div {
    let has_artifacts = state.has_artifacts();

    div()
        .flex()
        .flex_col()
        .w_full()
        .h_full()
        .bg(rgb(colors::EDITOR_BG))
        .when(!has_artifacts, |el| el.child(render_empty_tabs()))
        .when(has_artifacts, |el| {
            // Build tab items
            let tabs: Vec<TabItem> = state
                .artifacts()
                .iter()
                .map(|a| TabItem::from_artifact(a, state.is_active(&a.id)))
                .collect();

            el.child(render_tab_bar_simple(&tabs))
                .child(render_active_artifact(state))
        })
}

/// Render the active artifact content
fn render_active_artifact(state: &mut StageState) -> Div {
    let view_mode = state.view_mode();
    let show_line_numbers = state.show_line_numbers();
    let font_size = state.font_size();
    let scroll_position = state.scroll_position();

    // Get artifact info before mutable borrow
    let artifact_info = state.active_artifact().map(|a| {
        (
            a.id.clone(),
            a.content_type,
            a.language.clone(),
            a.read_only,
            a.content.clone(),
            a.title.clone(),
        )
    });

    if let Some((_id, content_type, language, read_only, content, title)) = artifact_info {
        // Calculate visible lines based on scroll position
        let line_height = font_size * 1.4;
        let first_visible_line = (scroll_position / line_height) as usize;

        match content {
            ArtifactContent::Text(ref text_content) => {
                // Check if we should show diff view
                let should_show_diff = matches!(view_mode, ViewMode::InlineChanges | ViewMode::Unified | ViewMode::SideBySide)
                    && text_content.previous_content.is_some();

                if should_show_diff {
                    let original = text_content.previous_content.as_deref().unwrap_or("");
                    let modified = &text_content.content;

                    let diff_content = crate::stage::types::DiffContent::new(original, modified);
                    let config = DiffViewConfig {
                        view_mode,
                        show_line_numbers,
                        font_size,
                        first_visible_line,
                        ..Default::default()
                    };

                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .overflow_hidden()
                        .child(render_artifact_toolbar(&title, content_type, read_only, view_mode))
                        .child(render_diff_view(&diff_content, &config))
                } else {
                    // Normal text view
                    let highlighter = language.as_ref().map(|l| SyntaxHighlighter::new(l));
                    let config = TextViewConfig {
                        show_line_numbers,
                        font_size,
                        first_visible_line,
                        read_only,
                        ..Default::default()
                    };

                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .overflow_hidden()
                        .child(render_artifact_toolbar(&title, content_type, read_only, view_mode))
                        .child(render_text_view(
                            &text_content.content,
                            language.as_deref(),
                            &config,
                            highlighter.as_ref(),
                        ))
                }
            }
            ArtifactContent::Diff(ref diff_content) => {
                let config = DiffViewConfig {
                    view_mode,
                    show_line_numbers,
                    font_size,
                    first_visible_line,
                    ..Default::default()
                };

                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_hidden()
                    .child(render_artifact_toolbar(&title, content_type, read_only, view_mode))
                    .child(render_diff_view(diff_content, &config))
            }
        }
    } else {
        // No active artifact (shouldn't happen if has_artifacts is true)
        render_empty_tabs()
    }
}

/// Render the toolbar above the artifact content
fn render_artifact_toolbar(
    title: &str,
    content_type: ContentType,
    read_only: bool,
    view_mode: ViewMode,
) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .h(px(28.0))
        .px_3()
        .bg(rgb(0x252526))
        .border_b_1()
        .border_color(rgb(0x404040))
        // Left side - title and info
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xcccccc))
                        .child(title.to_string())
                )
                .child(
                    div()
                        .px_2()
                        .py_px()
                        .rounded_sm()
                        .bg(rgb(0x3c3c3c))
                        .text_xs()
                        .text_color(rgb(0x808080))
                        .child(content_type.label())
                )
                .when(read_only, |el| {
                    el.child(
                        div()
                            .px_2()
                            .py_px()
                            .rounded_sm()
                            .bg(rgb(0x4a3c00))
                            .text_xs()
                            .text_color(rgb(0xdcdcaa))
                            .child("Read-only")
                    )
                })
        )
        // Right side - view mode indicator
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x808080))
                        .child(view_mode.label())
                )
        )
}

/// Render a mini preview of an artifact (for context rail or thumbnails)
pub fn render_artifact_preview(artifact: &Artifact, max_lines: usize) -> Div {
    let content = artifact.content_str();
    let lines: Vec<&str> = content.lines().take(max_lines).collect();
    let total_lines = artifact.line_count();
    let truncated = total_lines > max_lines;

    div()
        .flex()
        .flex_col()
        .w_full()
        .bg(rgb(0x1e1e1e))
        .rounded_md()
        .border_1()
        .border_color(rgb(0x3c3c3c))
        .overflow_hidden()
        // Header
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_2()
                .py_1()
                .bg(rgb(0x252526))
                .border_b_1()
                .border_color(rgb(0x3c3c3c))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(rgb(0xcccccc))
                                .child(artifact.title.clone())
                        )
                        .when(artifact.dirty, |el| {
                            el.child(
                                div()
                                    .w(px(6.0))
                                    .h(px(6.0))
                                    .rounded_full()
                                    .bg(rgb(0xffffff))
                            )
                        })
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x606060))
                        .child(format!("{} lines", total_lines))
                )
        )
        // Content preview
        .child(
            div()
                .flex()
                .flex_col()
                .p_2()
                .max_h(px(120.0))
                .overflow_hidden()
                .children(lines.iter().map(|line| {
                    let display_line = if line.is_empty() { " " } else { *line };
                    div()
                        .text_xs()
                        .font_family("monospace")
                        .text_color(rgb(0x808080))
                        .truncate()
                        .child(display_line.to_string())
                }))
                .when(truncated, |el| {
                    el.child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x606060))
                            .child("...")
                    )
                })
        )
}

/// Render artifact status bar (shown at bottom of stage)
pub fn render_artifact_status_bar(state: &StageState) -> Div {
    let artifact_count = state.artifact_count();
    let active_info = state.active_artifact().map(|a| {
        (
            a.title.clone(),
            a.line_count(),
            a.content_type.label().to_string(),
            a.language.clone(),
            a.read_only,
        )
    });

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .h(px(22.0))
        .px_3()
        .bg(rgb(0x007acc))
        .text_xs()
        .text_color(rgb(0xffffff))
        // Left side
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .when_some(active_info, |el, (title, lines, content_type, lang, read_only)| {
                    el.child(
                        div()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(title)
                    )
                    .child(
                        div()
                            .text_color(rgb(0xffffffcc))
                            .child(format!("{} lines", lines))
                    )
                    .child(
                        div()
                            .text_color(rgb(0xffffffcc))
                            .child(content_type)
                    )
                    .when_some(lang, |el, l| {
                        el.child(
                            div()
                                .text_color(rgb(0xffffffaa))
                                .child(l)
                        )
                    })
                    .when(read_only, |el| {
                        el.child(
                            div()
                                .text_color(rgb(0xffffffaa))
                                .child("🔒")
                        )
                    })
                })
        )
        // Right side
        .child(
            div()
                .text_color(rgb(0xffffffaa))
                .child(format!("{} open", artifact_count))
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stage::types::Artifact;

    #[test]
    fn test_empty_stage() {
        let mut state = StageState::new();
        assert!(!state.has_artifacts());
    }

    #[test]
    fn test_artifact_preview() {
        let artifact = Artifact::from_open(
            "test",
            "test.rs",
            "fn main() {\n    println!(\"Hello\");\n}",
            "code",
            false,
            Some("rust".to_string()),
        );

        // Just verify it doesn't panic
        let _ = render_artifact_preview(&artifact, 5);
    }
}
