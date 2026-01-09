//! Integration tests for AGUI Desktop

use agui_desktop::{app::AguiApp, config::AppConfig, VERSION};

#[test]
fn test_app_creation() {
    let app = AguiApp::new();
    assert!(!app.is_connected);
    assert!(app.is_healthy());
}

#[test]
fn test_config_default() {
    let config = AppConfig::default();
    assert_eq!(config.log_level, "info");
    assert!(config.enable_metrics);
    assert_eq!(config.metrics_port, 9090);
}

#[test]
fn test_version_display() {
    let version = VERSION.to_string();
    assert_eq!(version, "0.1.0");
}

#[test]
fn test_app_connection_status() {
    let app = AguiApp::new();
    assert_eq!(app.connection_status(), "Disconnected");
}
