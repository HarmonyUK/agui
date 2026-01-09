//! Stage Types
//!
//! Data structures for artifacts displayed in the Stage pane.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for artifacts
pub type ArtifactId = String;

/// An artifact displayed in the Stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Unique identifier
    pub id: ArtifactId,
    /// Display title
    pub title: String,
    /// Content type (text, code, markdown, diff, etc.)
    pub content_type: ContentType,
    /// The artifact content
    pub content: ArtifactContent,
    /// Whether this artifact is read-only
    pub read_only: bool,
    /// Language hint for syntax highlighting
    pub language: Option<String>,
    /// Timestamp when opened
    pub opened_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when last modified
    pub modified_at: chrono::DateTime<chrono::Utc>,
    /// Whether there are unsaved changes
    pub dirty: bool,
    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Artifact {
    /// Create a new artifact from protocol ArtifactOpen event
    pub fn from_open(
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
        content_type: impl Into<String>,
        read_only: bool,
        language: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        let content_type_str = content_type.into();
        Self {
            id: id.into(),
            title: title.into(),
            content_type: ContentType::from_str(&content_type_str),
            content: ArtifactContent::Text(TextContent::new(content)),
            read_only,
            language,
            opened_at: now,
            modified_at: now,
            dirty: false,
            metadata: HashMap::new(),
        }
    }

    /// Update content from a full replace
    pub fn update_content(&mut self, new_content: impl Into<String>) {
        if let ArtifactContent::Text(ref mut text) = self.content {
            let new_text = new_content.into();
            // Store old content for diff if needed
            text.previous_content = Some(text.content.clone());
            text.content = new_text;
            self.modified_at = chrono::Utc::now();
            if !self.read_only {
                self.dirty = true;
            }
        }
    }

    /// Apply a diff/patch to the content
    pub fn apply_diff(&mut self, diff: &str) {
        // For now, treat diff as content replacement
        // TODO: Implement proper diff application
        self.update_content(diff);
    }

    /// Get content as string
    pub fn content_str(&self) -> &str {
        match &self.content {
            ArtifactContent::Text(t) => &t.content,
            ArtifactContent::Diff(d) => &d.unified,
        }
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.content_str().lines().count()
    }

    /// Check if content is large (for chunking)
    pub fn is_large(&self) -> bool {
        self.content_str().len() > 100_000 || self.line_count() > 3000
    }
}

/// Content type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    /// Plain text
    #[default]
    Text,
    /// Source code (use language hint for highlighting)
    Code,
    /// Markdown
    Markdown,
    /// Diff/patch
    Diff,
    /// JSON data
    Json,
    /// YAML data
    Yaml,
    /// TOML data
    Toml,
    /// XML/HTML
    Xml,
    /// Binary (show hex dump)
    Binary,
}

impl ContentType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "text" | "plain" | "text/plain" => Self::Text,
            "code" | "source" => Self::Code,
            "markdown" | "md" | "text/markdown" => Self::Markdown,
            "diff" | "patch" => Self::Diff,
            "json" | "application/json" => Self::Json,
            "yaml" | "yml" | "application/yaml" => Self::Yaml,
            "toml" | "application/toml" => Self::Toml,
            "xml" | "html" | "text/xml" | "text/html" => Self::Xml,
            "binary" | "application/octet-stream" => Self::Binary,
            _ => Self::Text,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Code => "Code",
            Self::Markdown => "Markdown",
            Self::Diff => "Diff",
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Toml => "TOML",
            Self::Xml => "XML",
            Self::Binary => "Binary",
        }
    }
}

/// Artifact content variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ArtifactContent {
    /// Plain text or code content
    Text(TextContent),
    /// Diff content (two versions)
    Diff(DiffContent),
}

/// Text/code content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    /// Current content
    pub content: String,
    /// Previous content (for showing inline diff)
    #[serde(default)]
    pub previous_content: Option<String>,
    /// Cursor position (line, column)
    #[serde(default)]
    pub cursor: Option<(usize, usize)>,
    /// Selection range (start_line, start_col, end_line, end_col)
    #[serde(default)]
    pub selection: Option<(usize, usize, usize, usize)>,
    /// Scroll position (line)
    #[serde(default)]
    pub scroll_line: usize,
}

impl TextContent {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            previous_content: None,
            cursor: None,
            selection: None,
            scroll_line: 0,
        }
    }

    /// Get a range of lines (for chunked rendering)
    pub fn lines_range(&self, start: usize, end: usize) -> Vec<&str> {
        self.content
            .lines()
            .skip(start)
            .take(end - start)
            .collect()
    }
}

/// Diff content (for diff view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffContent {
    /// Original/left side content
    pub original: String,
    /// Modified/right side content
    pub modified: String,
    /// Unified diff representation
    pub unified: String,
    /// Parsed diff hunks
    #[serde(default)]
    pub hunks: Vec<DiffHunk>,
}

impl DiffContent {
    pub fn new(original: impl Into<String>, modified: impl Into<String>) -> Self {
        let original = original.into();
        let modified = modified.into();
        let unified = crate::stage::diff::compute_unified_diff(&original, &modified);
        let hunks = crate::stage::diff::parse_hunks(&unified);
        Self {
            original,
            modified,
            unified,
            hunks,
        }
    }
}

/// A diff hunk (section of changes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    /// Starting line in original
    pub old_start: usize,
    /// Line count in original
    pub old_count: usize,
    /// Starting line in modified
    pub new_start: usize,
    /// Line count in modified
    pub new_count: usize,
    /// Lines in this hunk
    pub lines: Vec<DiffLine>,
}

/// A single line in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    /// Line content (without +/- prefix)
    pub content: String,
    /// Line type
    pub line_type: DiffLineType,
    /// Line number in original (if applicable)
    pub old_line: Option<usize>,
    /// Line number in modified (if applicable)
    pub new_line: Option<usize>,
}

/// Type of diff line
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffLineType {
    /// Unchanged context line
    Context,
    /// Added line (only in modified)
    Addition,
    /// Removed line (only in original)
    Deletion,
    /// Hunk header
    Header,
}

/// Display mode for the artifact viewer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    /// Normal view (text/code with highlighting)
    #[default]
    Normal,
    /// Side-by-side diff view
    SideBySide,
    /// Unified diff view
    Unified,
    /// Inline changes (highlight changes in normal view)
    InlineChanges,
}

impl ViewMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::SideBySide => "Side by Side",
            Self::Unified => "Unified Diff",
            Self::InlineChanges => "Inline Changes",
        }
    }

    pub fn cycle_next(&self) -> Self {
        match self {
            Self::Normal => Self::InlineChanges,
            Self::InlineChanges => Self::Unified,
            Self::Unified => Self::SideBySide,
            Self::SideBySide => Self::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_creation() {
        let artifact = Artifact::from_open(
            "test-1",
            "test.rs",
            "fn main() {}",
            "code",
            false,
            Some("rust".to_string()),
        );

        assert_eq!(artifact.id, "test-1");
        assert_eq!(artifact.title, "test.rs");
        assert_eq!(artifact.content_type, ContentType::Code);
        assert!(!artifact.read_only);
        assert_eq!(artifact.language, Some("rust".to_string()));
    }

    #[test]
    fn test_content_type_parsing() {
        assert_eq!(ContentType::from_str("code"), ContentType::Code);
        assert_eq!(ContentType::from_str("application/json"), ContentType::Json);
        assert_eq!(ContentType::from_str("markdown"), ContentType::Markdown);
        assert_eq!(ContentType::from_str("unknown"), ContentType::Text);
    }

    #[test]
    fn test_artifact_update() {
        let mut artifact = Artifact::from_open("test", "test.txt", "hello", "text", false, None);

        artifact.update_content("world");
        assert_eq!(artifact.content_str(), "world");
        assert!(artifact.dirty);

        if let ArtifactContent::Text(ref t) = artifact.content {
            assert_eq!(t.previous_content, Some("hello".to_string()));
        }
    }

    #[test]
    fn test_line_count() {
        let artifact = Artifact::from_open("test", "test.txt", "line1\nline2\nline3", "text", false, None);
        assert_eq!(artifact.line_count(), 3);
    }

    #[test]
    fn test_view_mode_cycle() {
        let mode = ViewMode::Normal;
        assert_eq!(mode.cycle_next(), ViewMode::InlineChanges);
        assert_eq!(mode.cycle_next().cycle_next(), ViewMode::Unified);
    }
}
