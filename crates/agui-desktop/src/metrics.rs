//! Metrics collection and performance monitoring
//!
//! Provides FPS tracking, latency metrics, memory monitoring, and
//! performance state for adaptive rendering.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Global metrics instance
static METRICS: once_cell::sync::Lazy<Arc<PerformanceMetrics>> =
    once_cell::sync::Lazy::new(|| Arc::new(PerformanceMetrics::new()));

/// Get the global metrics instance
pub fn get_metrics() -> Arc<PerformanceMetrics> {
    Arc::clone(&METRICS)
}

/// Performance state for adaptive rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceMode {
    /// Full quality rendering (60+ FPS)
    Normal,
    /// Reduced quality (30-60 FPS) - disable animations
    Reduced,
    /// Minimal rendering (<30 FPS) - simplified UI
    Minimal,
}

impl PerformanceMode {
    /// Whether animations should be enabled
    pub fn animations_enabled(&self) -> bool {
        matches!(self, PerformanceMode::Normal)
    }

    /// Whether syntax highlighting should be enabled
    pub fn syntax_highlighting_enabled(&self) -> bool {
        !matches!(self, PerformanceMode::Minimal)
    }

    /// Max visible stream items before virtualization kicks in harder
    pub fn max_visible_items(&self) -> usize {
        match self {
            PerformanceMode::Normal => 100,
            PerformanceMode::Reduced => 50,
            PerformanceMode::Minimal => 25,
        }
    }
}

/// Rolling window for calculating averages
#[derive(Debug)]
struct RollingWindow {
    values: VecDeque<f64>,
    capacity: usize,
    sum: f64,
}

impl RollingWindow {
    fn new(capacity: usize) -> Self {
        Self {
            values: VecDeque::with_capacity(capacity),
            capacity,
            sum: 0.0,
        }
    }

    fn push(&mut self, value: f64) {
        if self.values.len() >= self.capacity {
            if let Some(old) = self.values.pop_front() {
                self.sum -= old;
            }
        }
        self.values.push_back(value);
        self.sum += value;
    }

    fn average(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.sum / self.values.len() as f64
        }
    }

    fn min(&self) -> f64 {
        self.values.iter().copied().fold(f64::INFINITY, f64::min)
    }

    fn max(&self) -> f64 {
        self.values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

/// Performance metrics collector
#[derive(Debug)]
pub struct PerformanceMetrics {
    /// Frame times in milliseconds (rolling window)
    frame_times: RwLock<RollingWindow>,
    /// Event processing times in milliseconds
    event_times: RwLock<RollingWindow>,
    /// Total frames rendered
    frame_count: AtomicU64,
    /// Successful connection attempts
    connection_successes: AtomicU64,
    /// Failed connection attempts
    connection_failures: AtomicU64,
    /// Current stream item count
    stream_items: AtomicU64,
    /// Current artifact count
    artifact_count: AtomicU64,
    /// Memory usage in bytes (approximate)
    memory_bytes: AtomicU64,
    /// Last frame timestamp
    last_frame: RwLock<Option<Instant>>,
    /// Current performance mode
    performance_mode: RwLock<PerformanceMode>,
}

impl PerformanceMetrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            frame_times: RwLock::new(RollingWindow::new(60)), // 1 second at 60fps
            event_times: RwLock::new(RollingWindow::new(100)),
            frame_count: AtomicU64::new(0),
            connection_successes: AtomicU64::new(0),
            connection_failures: AtomicU64::new(0),
            stream_items: AtomicU64::new(0),
            artifact_count: AtomicU64::new(0),
            memory_bytes: AtomicU64::new(0),
            last_frame: RwLock::new(None),
            performance_mode: RwLock::new(PerformanceMode::Normal),
        }
    }

    /// Record a frame and calculate frame time
    pub fn record_frame(&self) {
        self.frame_count.fetch_add(1, Ordering::Relaxed);

        let now = Instant::now();
        let mut last_frame = self.last_frame.write().unwrap();

        if let Some(last) = *last_frame {
            let frame_time = now.duration_since(last).as_secs_f64() * 1000.0;
            self.frame_times.write().unwrap().push(frame_time);

            // Update performance mode based on FPS
            self.update_performance_mode();
        }

        *last_frame = Some(now);
    }

    /// Record event processing time
    pub fn record_event_time(&self, duration: Duration) {
        let millis = duration.as_secs_f64() * 1000.0;
        self.event_times.write().unwrap().push(millis);
    }

    /// Record a connection attempt
    pub fn record_connection(&self, success: bool) {
        if success {
            self.connection_successes.fetch_add(1, Ordering::Relaxed);
        } else {
            self.connection_failures.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Update stream item count
    pub fn set_stream_items(&self, count: u64) {
        self.stream_items.store(count, Ordering::Relaxed);
    }

    /// Update artifact count
    pub fn set_artifact_count(&self, count: u64) {
        self.artifact_count.store(count, Ordering::Relaxed);
    }

    /// Update memory usage
    pub fn set_memory_bytes(&self, bytes: u64) {
        self.memory_bytes.store(bytes, Ordering::Relaxed);
    }

    /// Get current FPS (frames per second)
    pub fn fps(&self) -> f64 {
        let avg_frame_time = self.frame_times.read().unwrap().average();
        if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time
        } else {
            0.0
        }
    }

    /// Get average frame time in milliseconds
    pub fn avg_frame_time_ms(&self) -> f64 {
        self.frame_times.read().unwrap().average()
    }

    /// Get average event processing time in milliseconds
    pub fn avg_event_time_ms(&self) -> f64 {
        self.event_times.read().unwrap().average()
    }

    /// Get total frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count.load(Ordering::Relaxed)
    }

    /// Get stream item count
    pub fn stream_items(&self) -> u64 {
        self.stream_items.load(Ordering::Relaxed)
    }

    /// Get current performance mode
    pub fn performance_mode(&self) -> PerformanceMode {
        *self.performance_mode.read().unwrap()
    }

    /// Get memory usage in MB
    pub fn memory_mb(&self) -> f64 {
        self.memory_bytes.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0)
    }

    /// Update performance mode based on current metrics
    fn update_performance_mode(&self) {
        let fps = self.fps();
        let frame_times = self.frame_times.read().unwrap();

        // Only update if we have enough samples
        if frame_times.len() < 30 {
            return;
        }

        let new_mode = if fps >= 55.0 {
            PerformanceMode::Normal
        } else if fps >= 25.0 {
            PerformanceMode::Reduced
        } else {
            PerformanceMode::Minimal
        };

        let mut mode = self.performance_mode.write().unwrap();
        if *mode != new_mode {
            tracing::info!(
                "Performance mode changed: {:?} -> {:?} (FPS: {:.1})",
                *mode,
                new_mode,
                fps
            );
            *mode = new_mode;
        }
    }

    /// Get a snapshot of current metrics for display
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            fps: self.fps(),
            avg_frame_time_ms: self.avg_frame_time_ms(),
            avg_event_time_ms: self.avg_event_time_ms(),
            frame_count: self.frame_count(),
            stream_items: self.stream_items(),
            artifact_count: self.artifact_count.load(Ordering::Relaxed),
            memory_mb: self.memory_mb(),
            performance_mode: self.performance_mode(),
            connection_successes: self.connection_successes.load(Ordering::Relaxed),
            connection_failures: self.connection_failures.load(Ordering::Relaxed),
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of metrics for display/logging
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub fps: f64,
    pub avg_frame_time_ms: f64,
    pub avg_event_time_ms: f64,
    pub frame_count: u64,
    pub stream_items: u64,
    pub artifact_count: u64,
    pub memory_mb: f64,
    pub performance_mode: PerformanceMode,
    pub connection_successes: u64,
    pub connection_failures: u64,
}

impl MetricsSnapshot {
    /// Format FPS for display
    pub fn fps_string(&self) -> String {
        format!("{:.0} FPS", self.fps)
    }

    /// Format memory for display
    pub fn memory_string(&self) -> String {
        format!("{:.1} MB", self.memory_mb)
    }

    /// Get performance mode label
    pub fn mode_label(&self) -> &'static str {
        match self.performance_mode {
            PerformanceMode::Normal => "Normal",
            PerformanceMode::Reduced => "Reduced",
            PerformanceMode::Minimal => "Minimal",
        }
    }
}

// Legacy API functions for backward compatibility
// These delegate to the global metrics instance

/// Initialize metrics collection
pub async fn init(_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Prometheus exporter if needed
    let _builder = metrics_exporter_prometheus::PrometheusBuilder::new().install_recorder()?;
    tracing::info!("Metrics initialized");
    Ok(())
}

/// Record frame render time (legacy API)
pub fn record_frame_time(millis: f64) {
    get_metrics().frame_times.write().unwrap().push(millis);
}

/// Increment frame counter (legacy API)
pub fn increment_frames() {
    get_metrics().record_frame();
}

/// Update current stream items count (legacy API)
pub fn set_stream_items(count: u32) {
    get_metrics().set_stream_items(count as u64);
}

/// Record event processing time (legacy API)
pub fn record_event_processing(millis: f64) {
    get_metrics()
        .event_times
        .write()
        .unwrap()
        .push(millis);
}

/// Record connection attempt (legacy API)
pub fn record_connection_attempt(success: bool) {
    get_metrics().record_connection(success);
}

/// Record memory usage (legacy API)
pub fn set_memory_usage(bytes: u64) {
    get_metrics().set_memory_bytes(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_window() {
        let mut window = RollingWindow::new(3);
        window.push(1.0);
        window.push(2.0);
        window.push(3.0);
        assert_eq!(window.average(), 2.0);

        // Adding a 4th value should evict the first
        window.push(4.0);
        assert_eq!(window.average(), 3.0); // (2+3+4)/3
    }

    #[test]
    fn test_performance_mode_thresholds() {
        let mode = PerformanceMode::Normal;
        assert!(mode.animations_enabled());
        assert!(mode.syntax_highlighting_enabled());

        let mode = PerformanceMode::Minimal;
        assert!(!mode.animations_enabled());
        assert!(!mode.syntax_highlighting_enabled());
    }

    #[test]
    fn test_metrics_snapshot() {
        let metrics = PerformanceMetrics::new();
        metrics.set_stream_items(100);
        metrics.set_artifact_count(5);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.stream_items, 100);
        assert_eq!(snapshot.artifact_count, 5);
    }

    #[test]
    fn test_fps_calculation() {
        let metrics = PerformanceMetrics::new();

        // Simulate 60fps (16.67ms per frame)
        for _ in 0..60 {
            metrics.frame_times.write().unwrap().push(16.67);
        }

        let fps = metrics.fps();
        assert!((fps - 60.0).abs() < 1.0);
    }
}
