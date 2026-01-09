//! AG-UI WebSocket Protocol
//!
//! Defines the JSON event contract, versioning, and error handling for
//! communication between the AGUI client and orchestrator.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Protocol version
pub const PROTOCOL_VERSION: &str = "0.1.0";

/// AG-UI event envelope that wraps all messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope {
    /// Protocol version
    pub version: String,
    /// Unique event ID (timestamp-based or UUID)
    pub id: String,
    /// Timestamp when event was created (ISO 8601)
    pub timestamp: String,
    /// The actual event payload
    pub event: Event,
}

impl EventEnvelope {
    /// Create a new event envelope
    pub fn new(event: Event) -> Self {
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let id = uuid::Uuid::new_v4().to_string();

        Self {
            version: PROTOCOL_VERSION.to_string(),
            id,
            timestamp: now,
            event,
        }
    }
}

/// All possible event types in the AG-UI protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Event {
    /// User sent a text message
    #[serde(rename = "TEXT_MESSAGE")]
    TextMessage(TextMessage),

    /// Tool call initiated
    #[serde(rename = "TOOL_CALL_REQUEST")]
    ToolCallRequest(ToolCallRequest),

    /// Tool call status update
    #[serde(rename = "TOOL_CALL_STATUS")]
    ToolCallStatus(ToolCallStatus),

    /// Tool call completed with result
    #[serde(rename = "TOOL_CALL_RESULT")]
    ToolCallResult(ToolCallResult),

    /// State delta update from orchestrator
    #[serde(rename = "STATE_DELTA")]
    StateDelta(StateDelta),

    /// Request to re-render UI components
    #[serde(rename = "RENDER_REQUEST")]
    RenderRequest(RenderRequest),

    /// Plan/reasoning card to display
    #[serde(rename = "PLAN_CARD")]
    PlanCard(PlanCard),

    /// Resource tree update (files, folders, etc.)
    #[serde(rename = "RESOURCE_TREE")]
    ResourceTree(ResourceTree),

    /// List of connected agents
    #[serde(rename = "AGENT_ROSTER")]
    AgentRoster(AgentRoster),

    /// Open an artifact (file, document, etc.)
    #[serde(rename = "ARTIFACT_OPEN")]
    ArtifactOpen(ArtifactOpen),

    /// Update an open artifact
    #[serde(rename = "ARTIFACT_UPDATE")]
    ArtifactUpdate(ArtifactUpdate),

    /// User action (button click, form submission, etc.)
    #[serde(rename = "USER_ACTION")]
    UserAction(UserAction),

    /// Protocol error from server
    #[serde(rename = "ERROR")]
    Error(ErrorEvent),

    /// Server connection status
    #[serde(rename = "CONNECTION_STATUS")]
    ConnectionStatus(ConnectionStatus),
}

/// Text message event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextMessage {
    /// Message sender (user ID, agent ID, system, etc.)
    pub sender: String,
    /// Message content
    pub content: String,
    /// Optional metadata (author name, avatar, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallRequest {
    /// Tool call ID
    pub id: String,
    /// Tool name/identifier
    pub tool_name: String,
    /// Tool parameters
    #[serde(default)]
    pub parameters: serde_json::Value,
    /// Agent requesting the tool call
    pub agent_id: String,
}

/// Tool call status update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallStatus {
    /// Tool call ID
    pub id: String,
    /// Current status
    pub status: ToolCallState,
    /// Progress percentage (0-100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<u8>,
    /// Status message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Tool call states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ToolCallState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallResult {
    /// Tool call ID
    pub id: String,
    /// Result value
    pub result: serde_json::Value,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// State delta update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateDelta {
    /// Path to the state property (e.g., "user.name", "session.timeout")
    pub path: String,
    /// Old value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<serde_json::Value>,
    /// New value
    pub new_value: serde_json::Value,
}

/// Render request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderRequest {
    /// Component ID to render
    pub component_id: String,
    /// Component schema/definition
    pub schema: serde_json::Value,
    /// Optional props for the component
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<HashMap<String, serde_json::Value>>,
}

/// Plan card
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanCard {
    /// Card ID
    pub id: String,
    /// Card title
    pub title: String,
    /// Card content (markdown)
    pub content: String,
    /// Card status
    pub status: CardStatus,
}

/// Card status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CardStatus {
    Active,
    Completed,
    Failed,
    Cancelled,
}

/// Resource tree update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceTree {
    /// Root node
    pub root: ResourceNode,
}

/// Resource tree node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceNode {
    /// Node ID
    pub id: String,
    /// Node display name
    pub name: String,
    /// Node type (file, folder, etc.)
    pub node_type: String,
    /// Child nodes
    #[serde(default)]
    pub children: Vec<ResourceNode>,
}

/// Agent roster
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentRoster {
    /// List of connected agents
    pub agents: Vec<AgentInfo>,
}

/// Agent information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentInfo {
    /// Agent ID
    pub id: String,
    /// Agent name
    pub name: String,
    /// Agent status
    pub status: AgentStatus,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Agent status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Online,
    Busy,
    Idle,
    Offline,
}

/// Open artifact event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactOpen {
    /// Artifact ID
    pub id: String,
    /// Artifact title
    pub title: String,
    /// Artifact content
    pub content: String,
    /// Content type (text, code, markdown, etc.)
    pub content_type: String,
    /// Whether artifact is read-only
    #[serde(default)]
    pub read_only: bool,
    /// Language hint for syntax highlighting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// Update artifact event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactUpdate {
    /// Artifact ID
    pub id: String,
    /// Updated content
    pub content: String,
    /// Change type (full_replace, partial, diff, etc.)
    #[serde(default = "default_change_type")]
    pub change_type: String,
}

fn default_change_type() -> String {
    "full_replace".to_string()
}

/// User action event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserAction {
    /// Action type (click, submit, type, etc.)
    pub action_type: String,
    /// Component ID that triggered the action
    pub component_id: String,
    /// Action payload
    #[serde(default)]
    pub payload: serde_json::Value,
}

/// Connection status event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectionStatus {
    /// Connection status
    pub status: ConnectionState,
    /// Message describing the status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Connection states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
    Reconnecting,
    Failed,
}

/// Error event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorEvent {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Detailed error information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Protocol parsing and validation errors
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid event type: {0}")]
    InvalidEventType(String),

    #[error("Invalid protocol version: {0}")]
    InvalidVersion(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid event data: {0}")]
    InvalidEventData(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_envelope_creation() {
        let event = Event::TextMessage(TextMessage {
            sender: "user1".to_string(),
            content: "Hello".to_string(),
            metadata: None,
        });

        let envelope = EventEnvelope::new(event.clone());

        assert_eq!(envelope.version, PROTOCOL_VERSION);
        assert!(!envelope.id.is_empty());
        assert!(!envelope.timestamp.is_empty());
        assert_eq!(envelope.event, event);
    }

    #[test]
    fn test_text_message_serialization() {
        let msg = TextMessage {
            sender: "user1".to_string(),
            content: "Hello World".to_string(),
            metadata: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: TextMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_tool_call_serialization() {
        let tool_call = ToolCallRequest {
            id: "tc_001".to_string(),
            tool_name: "grep".to_string(),
            parameters: serde_json::json!({"pattern": "test"}),
            agent_id: "agent_1".to_string(),
        };

        let json = serde_json::to_string(&tool_call).unwrap();
        let deserialized: ToolCallRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(tool_call, deserialized);
    }

    #[test]
    fn test_event_enum_serialization() {
        let text_msg = Event::TextMessage(TextMessage {
            sender: "user".to_string(),
            content: "test".to_string(),
            metadata: None,
        });

        let json = serde_json::to_value(&text_msg).unwrap();
        assert_eq!(json["type"], "TEXT_MESSAGE");

        let tool_call = Event::ToolCallRequest(ToolCallRequest {
            id: "tc_001".to_string(),
            tool_name: "test".to_string(),
            parameters: serde_json::json!({}),
            agent_id: "agent".to_string(),
        });

        let json = serde_json::to_value(&tool_call).unwrap();
        assert_eq!(json["type"], "TOOL_CALL_REQUEST");
    }

    #[test]
    fn test_state_delta_serialization() {
        let delta = StateDelta {
            path: "user.name".to_string(),
            old_value: Some(serde_json::Value::String("John".to_string())),
            new_value: serde_json::Value::String("Jane".to_string()),
        };

        let json = serde_json::to_string(&delta).unwrap();
        let deserialized: StateDelta = serde_json::from_str(&json).unwrap();

        assert_eq!(delta, deserialized);
    }
}
