//! State management for resilience patterns.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Connection state for UI display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected, not attempting
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Successfully connected
    Connected,
    /// Connection failed, will retry
    Reconnecting {
        /// Number of retry attempts so far
        attempt: u32,
        /// Time until next retry
        next_retry_secs: u32,
    },
    /// Permanently failed (e.g., invalid credentials)
    Failed,
}

impl ConnectionState {
    /// Get display label for the connection state
    pub fn label(&self) -> &'static str {
        match self {
            ConnectionState::Disconnected => "Disconnected",
            ConnectionState::Connecting => "Connecting...",
            ConnectionState::Connected => "Connected",
            ConnectionState::Reconnecting { .. } => "Reconnecting...",
            ConnectionState::Failed => "Connection Failed",
        }
    }

    /// Get color for status indicator (RGBA hex)
    pub fn color(&self) -> u32 {
        match self {
            ConnectionState::Disconnected => 0x808080,  // Gray
            ConnectionState::Connecting => 0xdcdcaa,    // Yellow
            ConnectionState::Connected => 0x4ec9b0,     // Green
            ConnectionState::Reconnecting { .. } => 0xce9178, // Orange
            ConnectionState::Failed => 0xf14c4c,        // Red
        }
    }

    /// Whether the app should show offline mode
    pub fn is_offline(&self) -> bool {
        !matches!(self, ConnectionState::Connected)
    }
}

/// Reconnection strategy with exponential backoff
#[derive(Debug, Clone)]
pub struct ReconnectStrategy {
    /// Base delay in milliseconds
    base_delay_ms: u64,
    /// Maximum delay in milliseconds
    max_delay_ms: u64,
    /// Current attempt number
    attempt: u32,
    /// Maximum number of attempts (0 = unlimited)
    max_attempts: u32,
    /// Last attempt timestamp
    last_attempt: Option<Instant>,
}

impl ReconnectStrategy {
    /// Create a new reconnection strategy
    pub fn new() -> Self {
        Self {
            base_delay_ms: 1000,      // 1 second
            max_delay_ms: 60000,      // 60 seconds
            attempt: 0,
            max_attempts: 0,          // Unlimited
            last_attempt: None,
        }
    }

    /// Record a connection attempt
    pub fn record_attempt(&mut self) {
        self.attempt += 1;
        self.last_attempt = Some(Instant::now());
    }

    /// Reset on successful connection
    pub fn reset(&mut self) {
        self.attempt = 0;
        self.last_attempt = None;
    }

    /// Get the delay before next retry
    pub fn next_delay(&self) -> Duration {
        let delay_ms = self.base_delay_ms * 2u64.pow(self.attempt.min(10));
        Duration::from_millis(delay_ms.min(self.max_delay_ms))
    }

    /// Check if we should retry
    pub fn should_retry(&self) -> bool {
        if self.max_attempts > 0 && self.attempt >= self.max_attempts {
            return false;
        }

        if let Some(last) = self.last_attempt {
            Instant::now().duration_since(last) >= self.next_delay()
        } else {
            true
        }
    }

    /// Get current attempt number
    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    /// Get seconds until next retry
    pub fn seconds_until_retry(&self) -> u32 {
        if let Some(last) = self.last_attempt {
            let elapsed = Instant::now().duration_since(last);
            let delay = self.next_delay();
            if elapsed < delay {
                (delay - elapsed).as_secs() as u32
            } else {
                0
            }
        } else {
            0
        }
    }
}

impl Default for ReconnectStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Update batcher for reducing render frequency
#[derive(Debug)]
pub struct UpdateBatcher<T> {
    /// Pending updates
    pending: VecDeque<T>,
    /// Maximum batch size
    max_batch_size: usize,
    /// Minimum time between flushes
    min_interval: Duration,
    /// Last flush timestamp
    last_flush: Option<Instant>,
}

impl<T> UpdateBatcher<T> {
    /// Create a new batcher
    pub fn new(max_batch_size: usize, min_interval_ms: u64) -> Self {
        Self {
            pending: VecDeque::with_capacity(max_batch_size),
            max_batch_size,
            min_interval: Duration::from_millis(min_interval_ms),
            last_flush: None,
        }
    }

    /// Add an update to the batch
    pub fn push(&mut self, update: T) {
        self.pending.push_back(update);
    }

    /// Check if batch should be flushed
    pub fn should_flush(&self) -> bool {
        if self.pending.is_empty() {
            return false;
        }

        // Flush if batch is full
        if self.pending.len() >= self.max_batch_size {
            return true;
        }

        // Flush if enough time has passed
        if let Some(last) = self.last_flush {
            Instant::now().duration_since(last) >= self.min_interval
        } else {
            true
        }
    }

    /// Flush and return all pending updates
    pub fn flush(&mut self) -> Vec<T> {
        self.last_flush = Some(Instant::now());
        self.pending.drain(..).collect()
    }

    /// Get number of pending updates
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl<T> Default for UpdateBatcher<T> {
    fn default() -> Self {
        Self::new(10, 16) // 10 items or ~60fps
    }
}

/// Error severity for display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Informational message
    Info,
    /// Warning - operation may have issues
    Warning,
    /// Error - operation failed but app continues
    Error,
    /// Critical - app may not function correctly
    Critical,
}

impl ErrorSeverity {
    /// Get color for error display
    pub fn color(&self) -> u32 {
        match self {
            ErrorSeverity::Info => 0x3794ff,     // Blue
            ErrorSeverity::Warning => 0xdcdcaa,  // Yellow
            ErrorSeverity::Error => 0xf14c4c,    // Red
            ErrorSeverity::Critical => 0xff0000, // Bright red
        }
    }

    /// Get icon for error display
    pub fn icon(&self) -> &'static str {
        match self {
            ErrorSeverity::Info => "i",
            ErrorSeverity::Warning => "!",
            ErrorSeverity::Error => "x",
            ErrorSeverity::Critical => "X",
        }
    }
}

/// Application error for display
#[derive(Debug, Clone)]
pub struct AppError {
    /// Error message
    pub message: String,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Timestamp when error occurred
    pub timestamp: Instant,
    /// Whether error is dismissible
    pub dismissible: bool,
    /// Action to retry (if applicable)
    pub retry_action: Option<String>,
}

impl AppError {
    /// Create a new error
    pub fn new(message: impl Into<String>, severity: ErrorSeverity) -> Self {
        Self {
            message: message.into(),
            severity,
            timestamp: Instant::now(),
            dismissible: true,
            retry_action: None,
        }
    }

    /// Create with retry action
    pub fn with_retry(mut self, action: impl Into<String>) -> Self {
        self.retry_action = Some(action.into());
        self
    }

    /// Make non-dismissible
    pub fn non_dismissible(mut self) -> Self {
        self.dismissible = false;
        self
    }

    /// Check if error has expired (for auto-dismiss)
    pub fn is_expired(&self, max_age: Duration) -> bool {
        self.dismissible && Instant::now().duration_since(self.timestamp) > max_age
    }
}

/// Error manager for tracking and displaying errors
#[derive(Debug, Default)]
pub struct ErrorManager {
    /// Current errors
    errors: Vec<AppError>,
    /// Maximum errors to display
    max_errors: usize,
    /// Auto-dismiss duration
    auto_dismiss: Duration,
}

impl ErrorManager {
    /// Create a new error manager
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            max_errors: 5,
            auto_dismiss: Duration::from_secs(10),
        }
    }

    /// Add an error
    pub fn add(&mut self, error: AppError) {
        // Remove expired errors first
        self.cleanup();

        // Add new error
        self.errors.push(error);

        // Trim to max
        while self.errors.len() > self.max_errors {
            // Remove oldest dismissible error
            if let Some(idx) = self.errors.iter().position(|e| e.dismissible) {
                self.errors.remove(idx);
            } else {
                break;
            }
        }
    }

    /// Add a simple error message
    pub fn add_error(&mut self, message: impl Into<String>) {
        self.add(AppError::new(message, ErrorSeverity::Error));
    }

    /// Add a warning
    pub fn add_warning(&mut self, message: impl Into<String>) {
        self.add(AppError::new(message, ErrorSeverity::Warning));
    }

    /// Dismiss an error by index
    pub fn dismiss(&mut self, index: usize) {
        if index < self.errors.len() && self.errors[index].dismissible {
            self.errors.remove(index);
        }
    }

    /// Clear all dismissible errors
    pub fn clear_dismissible(&mut self) {
        self.errors.retain(|e| !e.dismissible);
    }

    /// Remove expired errors
    pub fn cleanup(&mut self) {
        self.errors.retain(|e| !e.is_expired(self.auto_dismiss));
    }

    /// Get current errors
    pub fn errors(&self) -> &[AppError] {
        &self.errors
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get highest severity error
    pub fn highest_severity(&self) -> Option<ErrorSeverity> {
        self.errors
            .iter()
            .map(|e| e.severity)
            .max_by_key(|s| match s {
                ErrorSeverity::Info => 0,
                ErrorSeverity::Warning => 1,
                ErrorSeverity::Error => 2,
                ErrorSeverity::Critical => 3,
            })
    }
}

/// Session compaction for long-running sessions
#[derive(Debug)]
pub struct SessionCompactor {
    /// Maximum stream items before compaction
    max_stream_items: usize,
    /// Items to keep after compaction
    keep_recent: usize,
    /// Last compaction timestamp
    last_compaction: Option<Instant>,
}

impl SessionCompactor {
    /// Create a new session compactor
    pub fn new(max_items: usize, keep_recent: usize) -> Self {
        Self {
            max_stream_items: max_items,
            keep_recent,
            last_compaction: None,
        }
    }

    /// Check if compaction is needed
    pub fn needs_compaction(&self, current_items: usize) -> bool {
        current_items > self.max_stream_items
    }

    /// Calculate how many items to remove
    pub fn items_to_remove(&self, current_items: usize) -> usize {
        if current_items > self.max_stream_items {
            current_items - self.keep_recent
        } else {
            0
        }
    }

    /// Record a compaction
    pub fn record_compaction(&mut self) {
        self.last_compaction = Some(Instant::now());
    }
}

impl Default for SessionCompactor {
    fn default() -> Self {
        Self::new(2000, 500) // Keep last 500 of 2000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_labels() {
        assert_eq!(ConnectionState::Connected.label(), "Connected");
        assert_eq!(ConnectionState::Disconnected.label(), "Disconnected");
        assert!(!ConnectionState::Connected.is_offline());
        assert!(ConnectionState::Disconnected.is_offline());
    }

    #[test]
    fn test_reconnect_strategy() {
        let mut strategy = ReconnectStrategy::new();
        assert!(strategy.should_retry());
        assert_eq!(strategy.attempt(), 0);

        strategy.record_attempt();
        assert_eq!(strategy.attempt(), 1);

        strategy.reset();
        assert_eq!(strategy.attempt(), 0);
    }

    #[test]
    fn test_exponential_backoff() {
        let mut strategy = ReconnectStrategy::new();

        // First retry: 1s
        assert_eq!(strategy.next_delay(), Duration::from_millis(1000));

        strategy.record_attempt();
        // Second retry: 2s
        assert_eq!(strategy.next_delay(), Duration::from_millis(2000));

        strategy.record_attempt();
        // Third retry: 4s
        assert_eq!(strategy.next_delay(), Duration::from_millis(4000));
    }

    #[test]
    fn test_update_batcher() {
        let mut batcher: UpdateBatcher<i32> = UpdateBatcher::new(3, 1000);

        batcher.push(1);
        batcher.push(2);
        assert_eq!(batcher.pending_count(), 2);
        // First flush is always allowed (no last_flush timestamp yet)
        assert!(batcher.should_flush());

        // Flush the first batch
        let batch1 = batcher.flush();
        assert_eq!(batch1, vec![1, 2]);
        assert_eq!(batcher.pending_count(), 0);

        // Now add items again - should not flush immediately (interval not passed)
        batcher.push(3);
        batcher.push(4);
        assert_eq!(batcher.pending_count(), 2);
        assert!(!batcher.should_flush()); // Interval hasn't passed yet

        // But if batch is full, should flush
        batcher.push(5);
        assert!(batcher.should_flush()); // Full

        let batch2 = batcher.flush();
        assert_eq!(batch2, vec![3, 4, 5]);
        assert_eq!(batcher.pending_count(), 0);
    }

    #[test]
    fn test_error_manager() {
        let mut manager = ErrorManager::new();
        assert!(!manager.has_errors());

        manager.add_error("Test error");
        assert!(manager.has_errors());
        assert_eq!(manager.errors().len(), 1);

        manager.dismiss(0);
        assert!(!manager.has_errors());
    }

    #[test]
    fn test_session_compactor() {
        let compactor = SessionCompactor::new(100, 50);

        assert!(!compactor.needs_compaction(50));
        assert!(!compactor.needs_compaction(100));
        assert!(compactor.needs_compaction(150));

        assert_eq!(compactor.items_to_remove(150), 100); // 150 - 50
    }
}
