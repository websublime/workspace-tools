//! File system watcher for the daemon service

use anyhow::{Context, Result};
use log::{debug, error, info};
use notify::{
    event::{ModifyKind, RenameMode},
    Event as NotifyEvent, EventKind, RecursiveMode, Watcher as NotifyWatcher,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

use crate::daemon::event::{Event, EventType, FileStatus};

/// Repository watcher
pub struct RepositoryWatcher {
    /// Path to the repository
    path: PathBuf,
    /// Name of the repository
    name: String,
    /// Patterns to include
    include_patterns: Vec<String>,
    /// Patterns to exclude
    exclude_patterns: Vec<String>,
    /// Internal watcher
    watcher: Option<Box<dyn notify::Watcher>>,
    /// Receiver for file system events
    event_rx: Option<mpsc::Receiver<Result<NotifyEvent, notify::Error>>>,
    /// Whether the watcher is running
    running: Arc<Mutex<bool>>,
    /// Thread handle
    thread_handle: Option<JoinHandle<()>>,
    /// Last event time
    last_event: Arc<Mutex<Option<SystemTime>>>,
    /// Event callback
    event_callback: Option<Arc<dyn Fn(Event) + Send + Sync>>, // Changed from Box to Arc
}

impl RepositoryWatcher {
    /// Create a new repository watcher
    pub fn new<P: AsRef<Path>, S: Into<String>>(path: P, name: S) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            name: name.into(),
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            watcher: None,
            event_rx: None,
            running: Arc::new(Mutex::new(false)),
            thread_handle: None,
            last_event: Arc::new(Mutex::new(None)),
            event_callback: None,
        }
    }

    /// Get the path of the repository
    pub fn path(&self) -> &PathBuf {
        // Since we're in the same module as the struct definition,
        // we should have access to private fields
        &self.path
    }

    /// Set include patterns
    pub fn with_include_patterns<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.include_patterns = patterns.into_iter().map(Into::into).collect();
        self
    }

    /// Set exclude patterns
    pub fn with_exclude_patterns<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.exclude_patterns = patterns.into_iter().map(Into::into).collect();
        self
    }

    /// Set event callback
    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(Event) + Send + Sync + 'static,
    {
        self.event_callback = Some(Arc::new(callback)); // Store in Arc instead of Box
        self
    }

    /// Start watching the repository
    pub fn start(&mut self) -> Result<()> {
        if self.is_running() {
            return Ok(());
        }

        // Create channel for file system events
        let (event_tx, event_rx) = mpsc::channel();

        // Create watcher
        let mut watcher = notify::recommended_watcher(event_tx)
            .context("Failed to create file system watcher")?;

        // Watch the repository
        watcher
            .watch(&self.path, RecursiveMode::Recursive)
            .with_context(|| format!("Failed to watch repository: {}", self.path.display()))?;

        // Store watcher and receiver
        self.watcher = Some(Box::new(watcher));
        self.event_rx = Some(event_rx);

        // Mark as running
        *self.running.lock().unwrap() = true;

        // Store references for thread
        let running = self.running.clone();
        let last_event = self.last_event.clone();
        let path = self.path.clone();
        let name = self.name.clone();
        let include_patterns = self.include_patterns.clone();
        let exclude_patterns = self.exclude_patterns.clone();
        let event_rx = self.event_rx.take().unwrap();
        let event_callback = self.event_callback.clone();

        // Spawn thread to handle events
        let thread_handle = thread::spawn(move || {
            info!("Started watching repository: {}", name);

            // Keep track of pending events
            let mut pending_events: HashMap<PathBuf, FileStatus> = HashMap::new();
            let mut last_flush = SystemTime::now();

            while *running.lock().unwrap() {
                // Check for events with a timeout
                match event_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(Ok(notify_event)) => {
                        // Process file system event
                        if let Some(file_event) = process_fs_event(
                            &notify_event,
                            &path,
                            &include_patterns,
                            &exclude_patterns,
                        ) {
                            // Update last event time
                            *last_event.lock().unwrap() = Some(SystemTime::now());

                            // Add to pending events
                            pending_events.insert(file_event.0, file_event.1);
                        }
                    }
                    Ok(Err(e)) => {
                        error!("File system watcher error: {}", e);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // No events received, check if we should flush pending events
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        error!("File system watcher channel disconnected");
                        break;
                    }
                }

                // Check if we should flush pending events
                let now = SystemTime::now();
                if now.duration_since(last_flush).unwrap_or_default() > Duration::from_millis(500)
                    && !pending_events.is_empty()
                {
                    flush_pending_events(&name, &pending_events, &event_callback);
                    pending_events.clear();
                    last_flush = now;
                }
            }

            info!("Stopped watching repository: {}", name);
        });

        // Store thread handle
        self.thread_handle = Some(thread_handle);

        Ok(())
    }

    /// Stop watching the repository
    pub fn stop(&mut self) -> Result<()> {
        if !self.is_running() {
            return Ok(());
        }

        // Mark as not running
        *self.running.lock().unwrap() = false;

        // Wait for thread to finish
        if let Some(thread_handle) = self.thread_handle.take() {
            if let Err(e) = thread_handle.join() {
                error!("Error joining watcher thread: {:?}", e);
            }
        }

        // Clear watcher
        self.watcher = None;

        Ok(())
    }

    /// Check if the watcher is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// Get the last event time
    pub fn last_event_time(&self) -> Option<SystemTime> {
        *self.last_event.lock().unwrap()
    }
}

/// Process a file system event
fn process_fs_event(
    event: &NotifyEvent,
    base_path: &Path,
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> Option<(PathBuf, FileStatus)> {
    // Get the path from the event
    let path = match event.paths.first() {
        Some(path) => path,
        None => return None,
    };

    // Convert to relative path
    let rel_path = match path.strip_prefix(base_path) {
        Ok(rel_path) => rel_path,
        Err(_) => return None,
    };

    // Skip if path matches exclude patterns
    for pattern in exclude_patterns {
        if glob_match::glob_match(pattern, rel_path.to_string_lossy().as_ref()) {
            return None;
        }
    }

    // Skip if include patterns are specified and path doesn't match any
    if !include_patterns.is_empty() {
        let mut matched = false;
        for pattern in include_patterns {
            if glob_match::glob_match(pattern, rel_path.to_string_lossy().as_ref()) {
                matched = true;
                break;
            }
        }
        if !matched {
            return None;
        }
    }

    // Determine file status based on event kind
    let status = match event.kind {
        EventKind::Create(_) => FileStatus::Added,
        EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
            // This is the destination of a rename
            FileStatus::Added
        }
        EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
            // This is the source of a rename
            if event.paths.len() > 1 {
                // Some implementations provide both paths in a single event
                FileStatus::Renamed(event.paths[1].clone())
            } else {
                // Otherwise treat as deleted
                FileStatus::Deleted
            }
        }
        EventKind::Modify(ModifyKind::Name(_)) => {
            // Other name modifications
            FileStatus::Modified
        }
        EventKind::Modify(_) => FileStatus::Modified,
        EventKind::Remove(_) => FileStatus::Deleted,
        _ => return None, // Skip other events
    };

    Some((path.to_path_buf(), status))
}

/// Flush pending events
fn flush_pending_events(
    repo_name: &str,
    pending_events: &HashMap<PathBuf, FileStatus>,
    event_callback: &Option<Arc<dyn Fn(Event) + Send + Sync>>,
) {
    if pending_events.is_empty() {
        return;
    }

    if pending_events.len() == 1 {
        // Single file change
        let (path, status) = pending_events.iter().next().unwrap();
        let event = Event::new(EventType::FileChanged {
            repository: repo_name.to_string(),
            path: path.clone(),
            status: status.clone(),
        });

        debug!("File changed: {} in {}", path.display(), repo_name);

        // Call event callback if set
        if let Some(callback) = event_callback {
            callback(event);
        }
    } else {
        // Multiple file changes
        let event = Event::new(EventType::FilesChanged {
            repository: repo_name.to_string(),
            count: pending_events.len(),
        });

        debug!("{} files changed in {}", pending_events.len(), repo_name);

        // Call event callback if set
        if let Some(callback) = event_callback {
            callback(event);
        }
    }
}

impl Drop for RepositoryWatcher {
    fn drop(&mut self) {
        if self.is_running() {
            if let Err(e) = self.stop() {
                error!("Error stopping watcher: {}", e);
            }
        }
    }
}
