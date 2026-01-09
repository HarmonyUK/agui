//! Stream Timeline Types
//!
//! Defines the data structures for all timeline items.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for stream items
pub type StreamItemId = String;

/// A single item in the stream timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamItem {
    /// Unique identifier
    pub id: StreamItemId,
    /// Timestamp when this item was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The content/payload of this item
    pub content: StreamContent,
    /// Whether this item is expanded (for accordions, etc.)
    #[serde(default)]
    pub expanded: bool,
}

impl StreamItem {
    /// Create a new stream item
    pub fn new(id: impl Into<String>, content: StreamContent) -> Self {
        Self {
            id: id.into(),
            timestamp: chrono::Utc::now(),
            content,
            expanded: false,
        }
    }

    /// Get the estimated height of this item in pixels (for virtualization)
    pub fn estimated_height(&self) -> f32 {
        match &self.content {
            StreamContent::UserMessage(msg) => {
                // Base height + text lines estimate
                60.0 + (msg.content.len() as f32 / 60.0).ceil() * 20.0
            }
            StreamContent::AgentMessage(msg) => {
                // Agent messages tend to be longer
                80.0 + (msg.content.len() as f32 / 60.0).ceil() * 20.0
            }
            StreamContent::Reasoning(_) => {
                // Collapsed by default
                48.0
            }
            StreamContent::ToolCall(tc) => {
                // Header + status + collapsed params
                100.0 + if tc.expanded { 80.0 } else { 0.0 }
            }
            StreamContent::Plan(plan) => {
                // Header + items
                60.0 + (plan.items.len() as f32 * 32.0)
            }
            StreamContent::Approval(_) => {
                // Fixed height
                120.0
            }
            StreamContent::StatusUpdate(_) => {
                // Compact
                40.0
            }
            StreamContent::Divider => 24.0,
        }
    }
}

/// Content types for stream items
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamContent {
    /// User message (right-aligned bubble)
    UserMessage(UserMessage),
    /// Agent message (left-aligned bubble with markdown)
    AgentMessage(AgentMessage),
    /// Reasoning/Chain-of-Thought (collapsible accordion)
    Reasoning(ReasoningBlock),
    /// Tool call (card with header, params, status, result)
    ToolCall(ToolCallBlock),
    /// Plan/checklist (task list with completion state)
    Plan(PlanBlock),
    /// Approval gate (approve/reject buttons)
    Approval(ApprovalBlock),
    /// Status update (progress indicator)
    StatusUpdate(StatusBlock),
    /// Visual divider
    Divider,
}

/// User message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    /// Message text content
    pub content: String,
    /// Optional display name
    #[serde(default)]
    pub sender_name: Option<String>,
    /// Optional avatar URL
    #[serde(default)]
    pub avatar: Option<String>,
    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl UserMessage {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            sender_name: None,
            avatar: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_sender(mut self, name: impl Into<String>) -> Self {
        self.sender_name = Some(name.into());
        self
    }
}

/// Agent message content (supports markdown)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Markdown content
    pub content: String,
    /// Agent identifier
    pub agent_id: String,
    /// Optional agent display name
    #[serde(default)]
    pub agent_name: Option<String>,
    /// Optional agent avatar URL
    #[serde(default)]
    pub avatar: Option<String>,
    /// Whether this message is still streaming
    #[serde(default)]
    pub streaming: bool,
    /// Confidence level (0.0-1.0)
    #[serde(default)]
    pub confidence: Option<f32>,
}

impl AgentMessage {
    pub fn new(agent_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            agent_id: agent_id.into(),
            agent_name: None,
            avatar: None,
            streaming: false,
            confidence: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.agent_name = Some(name.into());
        self
    }

    pub fn streaming(mut self) -> Self {
        self.streaming = true;
        self
    }
}

/// Reasoning/Chain-of-Thought block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningBlock {
    /// The reasoning text (often internal monologue)
    pub content: String,
    /// Short summary/title for collapsed view
    #[serde(default)]
    pub summary: Option<String>,
    /// Whether this block is expanded
    #[serde(default)]
    pub expanded: bool,
    /// Duration in milliseconds (how long this thinking took)
    #[serde(default)]
    pub duration_ms: Option<u64>,
}

impl ReasoningBlock {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            summary: None,
            expanded: false,
            duration_ms: None,
        }
    }

    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }
}

/// Tool call block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallBlock {
    /// Tool call ID (from protocol)
    pub call_id: String,
    /// Tool name/identifier
    pub tool_name: String,
    /// Input parameters (JSON)
    pub parameters: serde_json::Value,
    /// Current status
    pub status: ToolCallStatus,
    /// Result value (if completed)
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    /// Error message (if failed)
    #[serde(default)]
    pub error: Option<String>,
    /// Duration in milliseconds
    #[serde(default)]
    pub duration_ms: Option<u64>,
    /// Progress percentage (0-100)
    #[serde(default)]
    pub progress: Option<u8>,
    /// Whether params/result are expanded
    #[serde(default)]
    pub expanded: bool,
}

/// Tool call status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ToolCallStatus {
    /// Waiting to execute
    #[default]
    Pending,
    /// Currently executing
    Running,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
    /// User cancelled
    Cancelled,
}

impl ToolCallStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ToolCallStatus::Completed | ToolCallStatus::Failed | ToolCallStatus::Cancelled
        )
    }

    pub fn label(&self) -> &'static str {
        match self {
            ToolCallStatus::Pending => "Pending",
            ToolCallStatus::Running => "Running",
            ToolCallStatus::Completed => "Completed",
            ToolCallStatus::Failed => "Failed",
            ToolCallStatus::Cancelled => "Cancelled",
        }
    }
}

/// Plan/checklist block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanBlock {
    /// Plan title
    pub title: String,
    /// Plan items/steps
    pub items: Vec<PlanItem>,
    /// Overall plan status
    #[serde(default)]
    pub status: PlanStatus,
    /// Whether this plan is editable
    #[serde(default)]
    pub editable: bool,
}

impl PlanBlock {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
            status: PlanStatus::Draft,
            editable: true,
        }
    }

    pub fn with_items(mut self, items: Vec<PlanItem>) -> Self {
        self.items = items;
        self
    }

    /// Calculate completion percentage
    pub fn completion_percentage(&self) -> u8 {
        if self.items.is_empty() {
            return 0;
        }
        let completed = self
            .items
            .iter()
            .filter(|i| i.status == PlanItemStatus::Completed)
            .count();
        ((completed * 100) / self.items.len()) as u8
    }
}

/// Plan item/step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    /// Item ID
    pub id: String,
    /// Item description
    pub description: String,
    /// Item status
    #[serde(default)]
    pub status: PlanItemStatus,
    /// Sub-items (nested)
    #[serde(default)]
    pub children: Vec<PlanItem>,
}

impl PlanItem {
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            status: PlanItemStatus::Pending,
            children: Vec::new(),
        }
    }
}

/// Plan item status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlanItemStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Skipped,
    Failed,
}

/// Plan overall status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlanStatus {
    /// Being edited/drafted
    #[default]
    Draft,
    /// Awaiting user confirmation
    PendingApproval,
    /// Confirmed and executing
    Active,
    /// All items completed
    Completed,
    /// Plan was cancelled
    Cancelled,
}

/// Approval gate block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalBlock {
    /// Title/question for approval
    pub title: String,
    /// Detailed description
    #[serde(default)]
    pub description: Option<String>,
    /// What's being approved (code, action, etc.)
    #[serde(default)]
    pub content: Option<String>,
    /// Content type for syntax highlighting
    #[serde(default)]
    pub content_type: Option<String>,
    /// Available actions
    pub actions: Vec<ApprovalAction>,
    /// Current resolution status
    #[serde(default)]
    pub resolution: Option<ApprovalResolution>,
    /// Whether this is a blocking gate
    #[serde(default = "default_true")]
    pub blocking: bool,
}

fn default_true() -> bool {
    true
}

/// Available approval action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalAction {
    /// Action ID
    pub id: String,
    /// Display label
    pub label: String,
    /// Button variant (primary, secondary, destructive)
    #[serde(default)]
    pub variant: ApprovalActionVariant,
    /// Optional payload to send with action
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// Approval action variant
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalActionVariant {
    #[default]
    Primary,
    Secondary,
    Destructive,
}

/// Resolution of an approval gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResolution {
    /// Which action was taken
    pub action_id: String,
    /// When it was resolved
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Optional comment from user
    #[serde(default)]
    pub comment: Option<String>,
}

/// Status update block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBlock {
    /// Status message
    pub message: String,
    /// Status type
    #[serde(default)]
    pub status_type: StatusType,
    /// Progress percentage (0-100, if applicable)
    #[serde(default)]
    pub progress: Option<u8>,
    /// Whether this is a transient/ephemeral status
    #[serde(default)]
    pub ephemeral: bool,
}

impl StatusBlock {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status_type: StatusType::Info,
            progress: None,
            ephemeral: false,
        }
    }

    pub fn progress(message: impl Into<String>, progress: u8) -> Self {
        Self {
            message: message.into(),
            status_type: StatusType::Progress,
            progress: Some(progress.min(100)),
            ephemeral: true,
        }
    }
}

/// Status type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum StatusType {
    #[default]
    Info,
    Success,
    Warning,
    Error,
    Progress,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message() {
        let msg = UserMessage::new("Hello, world!").with_sender("Alice");
        assert_eq!(msg.content, "Hello, world!");
        assert_eq!(msg.sender_name, Some("Alice".to_string()));
    }

    #[test]
    fn test_agent_message() {
        let msg = AgentMessage::new("claude", "I can help with that.")
            .with_name("Claude")
            .streaming();
        assert!(msg.streaming);
        assert_eq!(msg.agent_name, Some("Claude".to_string()));
    }

    #[test]
    fn test_plan_completion() {
        let plan = PlanBlock::new("Test Plan").with_items(vec![
            PlanItem::new("1", "First step"),
            PlanItem {
                id: "2".to_string(),
                description: "Second step".to_string(),
                status: PlanItemStatus::Completed,
                children: vec![],
            },
            PlanItem {
                id: "3".to_string(),
                description: "Third step".to_string(),
                status: PlanItemStatus::Completed,
                children: vec![],
            },
        ]);

        assert_eq!(plan.completion_percentage(), 66); // 2/3 = 66%
    }

    #[test]
    fn test_tool_call_status() {
        assert!(!ToolCallStatus::Running.is_terminal());
        assert!(ToolCallStatus::Completed.is_terminal());
        assert!(ToolCallStatus::Failed.is_terminal());
    }

    #[test]
    fn test_estimated_height() {
        let item = StreamItem::new(
            "test",
            StreamContent::UserMessage(UserMessage::new("Short message")),
        );
        let height = item.estimated_height();
        assert!(height > 0.0);
    }
}
