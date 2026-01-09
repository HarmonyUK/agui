//! Stage State Management
//!
//! Manages the state of artifacts in the Stage pane including:
//! - Open artifacts (tabs)
//! - Active artifact selection
//! - View modes and settings
//! - STATE_DELTA hydration

use std::collections::HashMap;

use super::cache::ArtifactCache;
use super::syntax::SyntaxHighlighter;
use super::types::{Artifact, ArtifactContent, ArtifactId, DiffContent, ViewMode};
use crate::protocol::{ArtifactOpen, ArtifactUpdate, StateDelta};

/// State for the Stage pane
#[derive(Debug)]
pub struct StageState {
    /// Open artifacts (tab bar)
    artifacts: HashMap<ArtifactId, Artifact>,
    /// Order of artifacts (for tab ordering)
    artifact_order: Vec<ArtifactId>,
    /// Currently active artifact ID
    active_artifact: Option<ArtifactId>,
    /// Current view mode
    view_mode: ViewMode,
    /// Show line numbers
    show_line_numbers: bool,
    /// Word wrap enabled
    word_wrap: bool,
    /// Font size in pixels
    font_size: f32,
    /// Syntax highlighters cache
    highlighters: HashMap<String, SyntaxHighlighter>,
    /// Content cache
    cache: ArtifactCache,
    /// Scroll position per artifact
    scroll_positions: HashMap<ArtifactId, f32>,
}

impl Default for StageState {
    fn default() -> Self {
        Self::new()
    }
}

impl StageState {
    /// Create a new empty stage state
    pub fn new() -> Self {
        Self {
            artifacts: HashMap::new(),
            artifact_order: Vec::new(),
            active_artifact: None,
            view_mode: ViewMode::Normal,
            show_line_numbers: true,
            word_wrap: false,
            font_size: 13.0,
            highlighters: HashMap::new(),
            cache: ArtifactCache::new(),
            scroll_positions: HashMap::new(),
        }
    }

    // ==================== Artifact Management ====================

    /// Open a new artifact from protocol event
    pub fn open_artifact(&mut self, event: &ArtifactOpen) {
        let artifact = Artifact::from_open(
            &event.id,
            &event.title,
            &event.content,
            &event.content_type,
            event.read_only,
            event.language.clone(),
        );

        // Ensure highlighter is available
        if let Some(ref lang) = event.language {
            self.ensure_highlighter(lang);
        }

        // Insert or update
        if !self.artifacts.contains_key(&event.id) {
            self.artifact_order.push(event.id.clone());
        }
        self.artifacts.insert(event.id.clone(), artifact);

        // Make it active
        self.active_artifact = Some(event.id.clone());

        tracing::debug!("Opened artifact: {} ({})", event.title, event.id);
    }

    /// Update an artifact from protocol event
    pub fn update_artifact(&mut self, event: &ArtifactUpdate) -> bool {
        if let Some(artifact) = self.artifacts.get_mut(&event.id) {
            match event.change_type.as_str() {
                "full_replace" => {
                    artifact.update_content(&event.content);
                }
                "diff" | "patch" => {
                    artifact.apply_diff(&event.content);
                }
                _ => {
                    artifact.update_content(&event.content);
                }
            }

            // Invalidate cache for this artifact
            self.cache.invalidate_artifact(&event.id);

            tracing::debug!("Updated artifact: {}", event.id);
            true
        } else {
            tracing::warn!("Update for unknown artifact: {}", event.id);
            false
        }
    }

    /// Close an artifact
    pub fn close_artifact(&mut self, id: &str) -> bool {
        if self.artifacts.remove(id).is_some() {
            self.artifact_order.retain(|i| i != id);
            self.scroll_positions.remove(id);
            self.cache.invalidate_artifact(id);

            // Update active artifact if we closed the active one
            if self.active_artifact.as_deref() == Some(id) {
                self.active_artifact = self.artifact_order.last().cloned();
            }

            tracing::debug!("Closed artifact: {}", id);
            true
        } else {
            false
        }
    }

    /// Close all artifacts
    pub fn close_all(&mut self) {
        self.artifacts.clear();
        self.artifact_order.clear();
        self.active_artifact = None;
        self.scroll_positions.clear();
        self.cache.clear();
    }

    /// Get an artifact by ID
    pub fn get_artifact(&self, id: &str) -> Option<&Artifact> {
        self.artifacts.get(id)
    }

    /// Get mutable artifact by ID
    pub fn get_artifact_mut(&mut self, id: &str) -> Option<&mut Artifact> {
        self.artifacts.get_mut(id)
    }

    /// Get active artifact
    pub fn active_artifact(&self) -> Option<&Artifact> {
        self.active_artifact.as_ref().and_then(|id| self.artifacts.get(id))
    }

    /// Get mutable active artifact
    pub fn active_artifact_mut(&mut self) -> Option<&mut Artifact> {
        if let Some(id) = self.active_artifact.clone() {
            self.artifacts.get_mut(&id)
        } else {
            None
        }
    }

    /// Set active artifact
    pub fn set_active(&mut self, id: Option<ArtifactId>) {
        if let Some(ref new_id) = id {
            if self.artifacts.contains_key(new_id) {
                self.active_artifact = id;
            }
        } else {
            self.active_artifact = None;
        }
    }

    /// Select next artifact tab
    pub fn select_next_artifact(&mut self) {
        if self.artifact_order.is_empty() {
            return;
        }

        let current_idx = self.active_artifact.as_ref().and_then(|id| {
            self.artifact_order.iter().position(|i| i == id)
        });

        let next_idx = match current_idx {
            Some(idx) if idx + 1 < self.artifact_order.len() => idx + 1,
            _ => 0,
        };

        self.active_artifact = self.artifact_order.get(next_idx).cloned();
    }

    /// Select previous artifact tab
    pub fn select_previous_artifact(&mut self) {
        if self.artifact_order.is_empty() {
            return;
        }

        let current_idx = self.active_artifact.as_ref().and_then(|id| {
            self.artifact_order.iter().position(|i| i == id)
        });

        let prev_idx = match current_idx {
            Some(idx) if idx > 0 => idx - 1,
            _ => self.artifact_order.len().saturating_sub(1),
        };

        self.active_artifact = self.artifact_order.get(prev_idx).cloned();
    }

    /// Get list of open artifacts in order
    pub fn artifacts(&self) -> Vec<&Artifact> {
        self.artifact_order
            .iter()
            .filter_map(|id| self.artifacts.get(id))
            .collect()
    }

    /// Get artifact count
    pub fn artifact_count(&self) -> usize {
        self.artifacts.len()
    }

    /// Check if there are any open artifacts
    pub fn has_artifacts(&self) -> bool {
        !self.artifacts.is_empty()
    }

    /// Check if artifact is active
    pub fn is_active(&self, id: &str) -> bool {
        self.active_artifact.as_deref() == Some(id)
    }

    // ==================== STATE_DELTA Hydration ====================

    /// Apply a STATE_DELTA event
    pub fn apply_state_delta(&mut self, delta: &StateDelta) -> bool {
        // Parse the path to determine what to update
        // Expected paths:
        // - "artifact.<id>.content" - Update artifact content
        // - "artifact.<id>.title" - Update artifact title
        // - "artifact.<id>.read_only" - Update read-only status
        // - "stage.view_mode" - Update view mode
        // - "stage.show_line_numbers" - Update line numbers setting
        // - "stage.word_wrap" - Update word wrap setting
        // - "stage.font_size" - Update font size

        let parts: Vec<&str> = delta.path.split('.').collect();

        match parts.as_slice() {
            ["artifact", id, "content"] => {
                if let Some(content) = delta.new_value.as_str() {
                    if let Some(artifact) = self.artifacts.get_mut(*id) {
                        artifact.update_content(content);
                        self.cache.invalidate_artifact(id);
                        return true;
                    }
                }
            }
            ["artifact", id, "title"] => {
                if let Some(title) = delta.new_value.as_str() {
                    if let Some(artifact) = self.artifacts.get_mut(*id) {
                        artifact.title = title.to_string();
                        return true;
                    }
                }
            }
            ["artifact", id, "read_only"] => {
                if let Some(read_only) = delta.new_value.as_bool() {
                    if let Some(artifact) = self.artifacts.get_mut(*id) {
                        artifact.read_only = read_only;
                        return true;
                    }
                }
            }
            ["stage", "view_mode"] => {
                if let Some(mode_str) = delta.new_value.as_str() {
                    self.view_mode = match mode_str {
                        "normal" => ViewMode::Normal,
                        "side_by_side" => ViewMode::SideBySide,
                        "unified" => ViewMode::Unified,
                        "inline_changes" => ViewMode::InlineChanges,
                        _ => self.view_mode,
                    };
                    return true;
                }
            }
            ["stage", "show_line_numbers"] => {
                if let Some(show) = delta.new_value.as_bool() {
                    self.show_line_numbers = show;
                    return true;
                }
            }
            ["stage", "word_wrap"] => {
                if let Some(wrap) = delta.new_value.as_bool() {
                    self.word_wrap = wrap;
                    return true;
                }
            }
            ["stage", "font_size"] => {
                if let Some(size) = delta.new_value.as_f64() {
                    self.font_size = size as f32;
                    return true;
                }
            }
            _ => {
                tracing::debug!("Unknown state delta path: {}", delta.path);
            }
        }

        false
    }

    // ==================== View Settings ====================

    /// Get current view mode
    pub fn view_mode(&self) -> ViewMode {
        self.view_mode
    }

    /// Set view mode
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    /// Cycle to next view mode
    pub fn cycle_view_mode(&mut self) {
        self.view_mode = self.view_mode.cycle_next();
    }

    /// Toggle line numbers
    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
    }

    /// Get line numbers setting
    pub fn show_line_numbers(&self) -> bool {
        self.show_line_numbers
    }

    /// Toggle word wrap
    pub fn toggle_word_wrap(&mut self) {
        self.word_wrap = !self.word_wrap;
    }

    /// Get word wrap setting
    pub fn word_wrap(&self) -> bool {
        self.word_wrap
    }

    /// Get font size
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Set font size
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size.clamp(8.0, 32.0);
    }

    // ==================== Syntax Highlighting ====================

    /// Get or create a syntax highlighter for a language
    pub fn get_highlighter(&mut self, language: &str) -> &SyntaxHighlighter {
        self.ensure_highlighter(language);
        self.highlighters.get(language).unwrap()
    }

    /// Ensure highlighter exists for language
    fn ensure_highlighter(&mut self, language: &str) {
        if !self.highlighters.contains_key(language) {
            self.highlighters
                .insert(language.to_string(), SyntaxHighlighter::new(language));
        }
    }

    // ==================== Scrolling ====================

    /// Get scroll position for active artifact
    pub fn scroll_position(&self) -> f32 {
        self.active_artifact
            .as_ref()
            .and_then(|id| self.scroll_positions.get(id))
            .copied()
            .unwrap_or(0.0)
    }

    /// Set scroll position for active artifact
    pub fn set_scroll_position(&mut self, position: f32) {
        if let Some(id) = self.active_artifact.clone() {
            self.scroll_positions.insert(id, position.max(0.0));
        }
    }

    /// Scroll by delta for active artifact
    pub fn scroll_by(&mut self, delta: f32) {
        let current = self.scroll_position();
        self.set_scroll_position(current + delta);
    }

    // ==================== Diff Support ====================

    /// Create a diff view between two versions
    pub fn create_diff(&mut self, artifact_id: &str, old_content: &str, new_content: &str) {
        if let Some(artifact) = self.artifacts.get_mut(artifact_id) {
            artifact.content = ArtifactContent::Diff(DiffContent::new(old_content, new_content));
            self.view_mode = ViewMode::Unified;
            self.cache.invalidate_artifact(artifact_id);
        }
    }

    /// Show inline diff for active artifact (if it has previous content)
    pub fn show_inline_diff(&mut self) {
        if let Some(artifact) = self.active_artifact_mut() {
            if let ArtifactContent::Text(ref text) = artifact.content {
                if text.previous_content.is_some() {
                    self.view_mode = ViewMode::InlineChanges;
                }
            }
        }
    }

    // ==================== Editing ====================

    /// Update content of active artifact (for editing)
    pub fn update_active_content(&mut self, new_content: &str) {
        if let Some(artifact) = self.active_artifact_mut() {
            if !artifact.read_only {
                artifact.update_content(new_content);
                if let Some(id) = self.active_artifact.clone() {
                    self.cache.invalidate_artifact(&id);
                }
            }
        }
    }

    /// Mark active artifact as saved
    pub fn mark_saved(&mut self) {
        if let Some(artifact) = self.active_artifact_mut() {
            artifact.dirty = false;
        }
    }

    /// Check if any artifacts have unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.artifacts.values().any(|a| a.dirty)
    }

    /// Get list of artifacts with unsaved changes
    pub fn unsaved_artifacts(&self) -> Vec<&Artifact> {
        self.artifacts.values().filter(|a| a.dirty).collect()
    }

    // ==================== Cache Access ====================

    /// Get mutable cache reference
    pub fn cache_mut(&mut self) -> &mut ArtifactCache {
        &mut self.cache
    }

    /// Get cache reference
    pub fn cache(&self) -> &ArtifactCache {
        &self.cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_artifact_open(id: &str, title: &str, content: &str) -> ArtifactOpen {
        ArtifactOpen {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            content_type: "code".to_string(),
            read_only: false,
            language: Some("rust".to_string()),
        }
    }

    #[test]
    fn test_open_artifact() {
        let mut state = StageState::new();
        let event = make_artifact_open("test-1", "test.rs", "fn main() {}");

        state.open_artifact(&event);

        assert_eq!(state.artifact_count(), 1);
        assert!(state.is_active("test-1"));
        assert_eq!(state.active_artifact().unwrap().title, "test.rs");
    }

    #[test]
    fn test_close_artifact() {
        let mut state = StageState::new();
        state.open_artifact(&make_artifact_open("1", "a.rs", "a"));
        state.open_artifact(&make_artifact_open("2", "b.rs", "b"));

        assert_eq!(state.artifact_count(), 2);
        assert!(state.is_active("2")); // Last opened is active

        state.close_artifact("2");

        assert_eq!(state.artifact_count(), 1);
        assert!(state.is_active("1")); // Falls back to remaining artifact
    }

    #[test]
    fn test_artifact_navigation() {
        let mut state = StageState::new();
        state.open_artifact(&make_artifact_open("1", "a.rs", "a"));
        state.open_artifact(&make_artifact_open("2", "b.rs", "b"));
        state.open_artifact(&make_artifact_open("3", "c.rs", "c"));

        assert!(state.is_active("3"));

        state.select_previous_artifact();
        assert!(state.is_active("2"));

        state.select_previous_artifact();
        assert!(state.is_active("1"));

        state.select_next_artifact();
        assert!(state.is_active("2"));
    }

    #[test]
    fn test_update_artifact() {
        let mut state = StageState::new();
        state.open_artifact(&make_artifact_open("1", "test.rs", "original"));

        let update = ArtifactUpdate {
            id: "1".to_string(),
            content: "modified".to_string(),
            change_type: "full_replace".to_string(),
        };

        assert!(state.update_artifact(&update));
        assert_eq!(state.active_artifact().unwrap().content_str(), "modified");
    }

    #[test]
    fn test_state_delta() {
        let mut state = StageState::new();
        state.open_artifact(&make_artifact_open("1", "test.rs", "original"));

        let delta = StateDelta {
            path: "artifact.1.content".to_string(),
            old_value: Some(serde_json::Value::String("original".to_string())),
            new_value: serde_json::Value::String("updated".to_string()),
        };

        assert!(state.apply_state_delta(&delta));
        assert_eq!(state.get_artifact("1").unwrap().content_str(), "updated");
    }

    #[test]
    fn test_view_mode_cycling() {
        let mut state = StageState::new();
        assert_eq!(state.view_mode(), ViewMode::Normal);

        state.cycle_view_mode();
        assert_eq!(state.view_mode(), ViewMode::InlineChanges);

        state.cycle_view_mode();
        assert_eq!(state.view_mode(), ViewMode::Unified);
    }

    #[test]
    fn test_dirty_tracking() {
        let mut state = StageState::new();
        state.open_artifact(&make_artifact_open("1", "test.rs", "original"));

        assert!(!state.has_unsaved_changes());

        state.update_active_content("modified");
        assert!(state.has_unsaved_changes());

        state.mark_saved();
        assert!(!state.has_unsaved_changes());
    }
}
