//! Application configuration

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Log level for application logging
    pub log_level: String,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Metrics export port (if enabled)
    pub metrics_port: u16,
    /// Enable hot reload for development
    pub enable_hot_reload: bool,
    /// Project root directory for hot reload watching
    pub project_root: PathBuf,
    /// WebSocket URL for orchestrator connection
    pub orchestrator_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            enable_metrics: true,
            metrics_port: 9090,
            enable_hot_reload: cfg!(debug_assertions),
            project_root: std::env::current_dir().unwrap_or_default(),
            orchestrator_url: "ws://localhost:8765".to_string(),
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(log_level) = std::env::var("AGUI_LOG_LEVEL") {
            config.log_level = log_level;
        }

        if let Ok(enable_metrics) = std::env::var("AGUI_ENABLE_METRICS") {
            config.enable_metrics = enable_metrics.to_lowercase() == "true";
        }

        if let Ok(metrics_port) = std::env::var("AGUI_METRICS_PORT") {
            if let Ok(port) = metrics_port.parse() {
                config.metrics_port = port;
            }
        }

        if let Ok(enable_hot_reload) = std::env::var("AGUI_HOT_RELOAD") {
            config.enable_hot_reload = enable_hot_reload.to_lowercase() == "true";
        }

        if let Ok(project_root) = std::env::var("AGUI_PROJECT_ROOT") {
            config.project_root = PathBuf::from(project_root);
        }

        if let Ok(orchestrator_url) = std::env::var("AGUI_ORCHESTRATOR_URL") {
            config.orchestrator_url = orchestrator_url;
        }

        config
    }
}
