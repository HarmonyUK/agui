//! Main application structure and state management

#[derive(Clone)]
pub enum Message {
    /// Frame update event
    Frame,
    /// Connection to orchestrator established
    Connected,
    /// Connection lost
    Disconnected(String),
    /// Error occurred
    Error(String),
}

/// Main AGUI application state
pub struct AguiApp {
    /// Is the app connected to the orchestrator?
    pub is_connected: bool,
    /// Current error message, if any
    pub error: Option<String>,
    /// Frame counter for performance monitoring
    pub frame_count: u64,
}

impl AguiApp {
    /// Create a new AGUI application instance
    pub fn new() -> Self {
        tracing::debug!("Creating new AguiApp instance");
        Self {
            is_connected: false,
            error: None,
            frame_count: 0,
        }
    }

    /// Handle incoming message
    pub fn handle_message(&mut self, message: Message) {
        match message {
            Message::Frame => {
                self.frame_count += 1;
                crate::metrics::increment_frames();

                if self.frame_count.is_multiple_of(60) {
                    tracing::trace!("Frame count: {}", self.frame_count);
                }
            }
            Message::Connected => {
                tracing::info!("Connected to orchestrator");
                self.is_connected = true;
                self.error = None;
                crate::metrics::record_connection_attempt(true);
            }
            Message::Disconnected(reason) => {
                tracing::warn!("Disconnected from orchestrator: {}", reason);
                self.is_connected = false;
                self.error = Some(reason);
                crate::metrics::record_connection_attempt(false);
            }
            Message::Error(msg) => {
                tracing::error!("Application error: {}", msg);
                self.error = Some(msg);
            }
        }
    }

    /// Check if app is in a healthy state
    pub fn is_healthy(&self) -> bool {
        self.error.is_none()
    }

    /// Get current connection status
    pub fn connection_status(&self) -> &'static str {
        if self.is_connected {
            "Connected"
        } else {
            "Disconnected"
        }
    }
}

impl Default for AguiApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_app() {
        let app = AguiApp::new();
        assert!(!app.is_connected);
        assert!(app.error.is_none());
        assert_eq!(app.frame_count, 0);
        assert!(app.is_healthy());
    }

    #[test]
    fn test_handle_connected() {
        let mut app = AguiApp::new();
        app.handle_message(Message::Connected);
        assert!(app.is_connected);
        assert!(app.is_healthy());
    }

    #[test]
    fn test_handle_error() {
        let mut app = AguiApp::new();
        let error_msg = "Connection failed";
        app.handle_message(Message::Error(error_msg.to_string()));
        assert!(!app.is_healthy());
        assert_eq!(app.error.as_deref(), Some(error_msg));
    }

    #[test]
    fn test_frame_counting() {
        let mut app = AguiApp::new();
        for _ in 0..100 {
            app.handle_message(Message::Frame);
        }
        assert_eq!(app.frame_count, 100);
    }
}
