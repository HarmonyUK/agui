//! AGUI Desktop - Universal Agent GUI native desktop client
//!
//! A high-performance, declarative UI framework for agent orchestration.
//! Built on GPUI for GPU-accelerated rendering and hot-reload support.

pub mod app;
pub mod config;
pub mod hot_reload;
pub mod layout;
pub mod logging;
pub mod metrics;
pub mod mock_server;
pub mod protocol;
pub mod renderer;
pub mod resilience;
pub mod stage;
pub mod stream;

pub use app::AguiApp;
pub use config::AppConfig;
pub use layout::{LayoutState, Pane};
pub use mock_server::{MockServer, MockServerConfig};
pub use protocol::Event;
pub use renderer::{
    render_component, render_from_json, parse_component,
    FormAction, FormState, FormValue, RenderContext, Component,
};
pub use stage::{
    StageState, Artifact, ArtifactContent, ContentType, ViewMode,
    render_stage_pane, render_artifact_preview, render_artifact_status_bar,
};
pub use stream::{
    StreamState, StreamTimeline, StreamItem, StreamContent,
    UserMessage, AgentMessage, ReasoningBlock, ToolCallBlock,
    PlanBlock, ApprovalBlock, StatusBlock,
};
pub use resilience::{
    ConnectionState, ReconnectStrategy, UpdateBatcher,
    ErrorSeverity, AppError, ErrorManager, SessionCompactor,
};
pub use metrics::{
    PerformanceMode, PerformanceMetrics, MetricsSnapshot, get_metrics,
};

#[derive(Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Default for Version {
    fn default() -> Self {
        Self::new()
    }
}

impl Version {
    pub const fn new() -> Self {
        Self {
            major: 0,
            minor: 3,
            patch: 0,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub const VERSION: Version = Version {
    major: 0,
    minor: 3,
    patch: 0,
};
