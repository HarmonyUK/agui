//! Hot reload support for development

use anyhow::Result;
use notify::{RecursiveMode, Result as NotifyResult, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

/// File change event for hot reload
#[derive(Debug, Clone)]
pub enum HotReloadEvent {
    FileChanged(PathBuf),
    AssetChanged(PathBuf),
}

/// Hot reload watcher for development
pub struct HotReloadWatcher {
    _watcher: notify::RecommendedWatcher,
    rx: mpsc::Receiver<HotReloadEvent>,
}

impl HotReloadWatcher {
    /// Create a new hot reload watcher
    pub fn new(watch_path: PathBuf) -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = notify::recommended_watcher(move |res: NotifyResult<notify::Event>| {
            if let Ok(event) = res {
                match event.kind {
                    notify::event::EventKind::Modify(_) => {
                        for path in event.paths {
                            // Ignore non-code files and cache directories
                            if should_watch(&path) {
                                let _ = tx.send(HotReloadEvent::FileChanged(path));
                            }
                        }
                    }
                    notify::event::EventKind::Create(_) => {
                        for path in event.paths {
                            if should_watch(&path) {
                                let _ = tx.send(HotReloadEvent::FileChanged(path));
                            }
                        }
                    }
                    _ => {}
                }
            }
        })?;

        watcher.watch(&watch_path, RecursiveMode::Recursive)?;

        tracing::info!("Hot reload watcher started for: {}", watch_path.display());

        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    /// Try to receive a hot reload event (non-blocking)
    pub fn try_recv(&self) -> Option<HotReloadEvent> {
        self.rx.try_recv().ok()
    }

    /// Receive with timeout
    pub fn recv_timeout(&self, duration: Duration) -> Option<HotReloadEvent> {
        self.rx.recv_timeout(duration).ok()
    }
}

/// Determine if a path should be watched
fn should_watch(path: &std::path::Path) -> bool {
    let path_str = path.to_string_lossy();

    // Ignore non-Rust source files
    if !path_str.ends_with(".rs") {
        return false;
    }

    // Ignore target directory
    if path_str.contains("/target/") || path_str.contains("\\target\\") {
        return false;
    }

    // Ignore hidden directories
    if path
        .components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
    {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_watch() {
        assert!(should_watch("src/main.rs".as_ref()));
        assert!(should_watch("src/app/mod.rs".as_ref()));
        assert!(!should_watch("target/debug/agui".as_ref()));
        assert!(!should_watch(".git/config".as_ref()));
        assert!(!should_watch("src/main.txt".as_ref()));
    }
}
