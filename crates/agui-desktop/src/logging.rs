//! Structured logging setup

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init(log_level: &str) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(log_level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true),
        )
        .init();

    Ok(())
}

/// Helper to log application startup
pub fn log_startup(version: &str, config_summary: &str) {
    tracing::info!("=== AGUI Desktop {} ===", version);
    tracing::info!("Configuration: {}", config_summary);
    tracing::debug!("Startup logging initialized");
}

/// Helper to log application shutdown
pub fn log_shutdown(reason: &str) {
    tracing::info!("Shutting down: {}", reason);
}
