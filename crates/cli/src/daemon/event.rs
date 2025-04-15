//! Event system for the daemon service
//!
//! This module defines the event types, status enums, and handler
//! traits for working with repository events within the daemon.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::time::SystemTime;

/// Event type enum defining all possible events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// Repository added to monitoring
    RepositoryAdded {
        /// Name of the repository
        name: String,
        /// Path to the repository
        path: PathBuf,
    },

    /// Repository removed from monitoring
    RepositoryRemoved {
        /// Name of the repository
        name: String,
    },

    /// File changed in a repository
    FileChanged {
        /// Repository name
        repository: String,
        /// Path to the file
        path: PathBuf,
        /// Status of the change
        status: FileStatus,
    },

    /// Multiple files changed in a repository
    FilesChanged {
        /// Repository name
        repository: String,
        /// Number of changed files
        count: usize,
    },

    /// Git state changed (e.g., new commit, branch change)
    GitStateChanged {
        /// Repository name
        repository: String,
        /// New commit hash (if applicable)
        commit: Option<String>,
        /// New branch name (if applicable)
        branch: Option<String>,
        /// Reference type that changed (branch, tag, etc.)
        ref_type: Option<String>,
        /// Whether this is a remote reference
        is_remote: bool,
    },

    /// New commit pushed or pulled
    CommitEvent {
        /// Repository name
        repository: String,
        /// Commit hash
        hash: String,
        /// Commit message
        message: String,
        /// Author name
        author: String,
        /// Author email
        email: Option<String>,
        /// Whether this commit was pushed or pulled
        is_push: bool,
    },

    /// Branch created or deleted
    BranchEvent {
        /// Repository name
        repository: String,
        /// Branch name
        branch: String,
        /// Whether the branch was created (true) or deleted (false)
        created: bool,
        /// Whether this is a remote branch
        is_remote: bool,
    },

    /// Tag created or deleted
    TagEvent {
        /// Repository name
        repository: String,
        /// Tag name
        tag: String,
        /// Target commit hash
        target: Option<String>,
        /// Whether the tag was created (true) or deleted (false)
        created: bool,
    },

    /// Changes detected in a package
    PackageChanged {
        /// Repository name
        repository: String,
        /// Package name
        package: String,
        /// Number of changed files
        file_count: usize,
        /// Whether dependency file (package.json) was modified
        deps_changed: bool,
    },

    /// Package version changed
    PackageVersionChanged {
        /// Repository name
        repository: String,
        /// Package name
        package: String,
        /// Previous version
        previous_version: String,
        /// New version
        new_version: String,
    },

    /// Repository synchronization
    RepositorySync {
        /// Repository name
        repository: String,
        /// Operation type (pull, push, fetch)
        operation: String,
        /// Target (remote name, branch name)
        target: String,
        /// Whether the synchronization succeeded
        success: bool,
    },

    /// Daemon status change
    DaemonStatusChanged {
        /// Status message
        status: String,
    },

    /// Generic notification
    Notification {
        /// Notification message
        message: String,
        /// Notification level
        level: NotificationLevel,
        /// Associated repository (if any)
        repository: Option<String>,
    },

    /// Error occurred
    Error {
        /// Error message
        message: String,
        /// Associated repository (if any)
        repository: Option<String>,
        /// Error code or type (if available)
        error_type: Option<String>,
    },

    /// Task started or finished
    TaskEvent {
        /// Task name
        name: String,
        /// Whether task started (true) or finished (false)
        started: bool,
        /// Exit code (if finished)
        exit_code: Option<i32>,
        /// Duration in milliseconds (if finished)
        duration_ms: Option<u64>,
    },

    /// Custom event for extensibility
    Custom {
        /// Event name
        name: String,
        /// Event data as JSON
        data: serde_json::Value,
    },
}

/// File status enum representing possible file changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileStatus {
    /// File was added
    Added,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed
    Renamed(PathBuf),
    /// File has merge conflicts
    Conflict,
    /// File was moved
    Moved(PathBuf),
    /// File permissions changed
    PermissionChanged,
    /// File was staged
    Staged,
    /// File was unstaged
    Unstaged,
}

/// Notification level for generic notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel {
    /// Information
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Success
    Success,
    /// Debug information
    Debug,
}

/// Encapsulates an event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event type
    pub event_type: EventType,
    /// Timestamp
    pub timestamp: SystemTime,
    /// Source (component that generated the event)
    pub source: String,
    /// Event ID
    pub id: String,
    /// Associated topics/tags
    pub topics: Vec<String>,
    /// Priority (0-100, higher is more important)
    pub priority: u8,
}

/// Event handler trait for components that process events
pub trait EventHandler: Send + Sync {
    /// Handle an event
    fn handle_event(&self, event: &Event);

    /// Filter determining if this handler should process the event
    fn should_handle(&self, _event: &Event) -> bool {
        true // Default implementation handles all events
    }

    /// Get the handler name
    fn name(&self) -> &str;
}

impl Event {
    /// Create a new event
    pub fn new(event_type: EventType) -> Self {
        Self {
            event_type,
            timestamp: SystemTime::now(),
            source: "daemon".to_string(),
            id: format!("{:x}", rand::random::<u64>()),
            topics: Vec::new(),
            priority: 50, // Default medium priority
        }
    }

    /// Create a new event with a specific source
    pub fn new_with_source(event_type: EventType, source: &str) -> Self {
        Self {
            event_type,
            timestamp: SystemTime::now(),
            source: source.to_string(),
            id: format!("{:x}", rand::random::<u64>()),
            topics: Vec::new(),
            priority: 50,
        }
    }

    /// Create a new high-priority event
    pub fn new_high_priority(event_type: EventType) -> Self {
        Self {
            event_type,
            timestamp: SystemTime::now(),
            source: "daemon".to_string(),
            id: format!("{:x}", rand::random::<u64>()),
            topics: Vec::new(),
            priority: 80,
        }
    }

    /// Helper for creating an event with specified priority
    pub fn new_with_priority(event_type: EventType, priority: u8) -> Self {
        Self {
            event_type,
            timestamp: SystemTime::now(),
            source: "daemon".to_string(),
            id: format!("{:x}", rand::random::<u64>()),
            topics: Vec::new(),
            priority,
        }
    }

    /// Add topics to the event
    pub fn with_topics(mut self, topics: Vec<impl Into<String>>) -> Self {
        self.topics = topics.into_iter().map(Into::into).collect();
        self
    }

    /// Set the event priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Get the event timestamp as a formatted string
    pub fn formatted_time(&self) -> String {
        let datetime: DateTime<Utc> = self.timestamp.into();
        datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    /// Get a short description of the event
    pub fn short_description(&self) -> String {
        match &self.event_type {
            EventType::RepositoryAdded { name, .. } => format!("Repository added: {}", name),

            EventType::RepositoryRemoved { name } => format!("Repository removed: {}", name),

            EventType::FileChanged { repository, path, status } => {
                format!("File {} in {}: {:?}", path.display(), repository, status)
            }

            EventType::FilesChanged { repository, count } => {
                format!("{} files changed in {}", count, repository)
            }

            EventType::GitStateChanged { repository, branch, commit, .. } => {
                if let Some(branch) = branch {
                    format!("Branch changed to {} in {}", branch, repository)
                } else if let Some(commit) = commit {
                    format!("New commit {} in {}", &commit[..8], repository)
                } else {
                    format!("Git state changed in {}", repository)
                }
            }

            EventType::CommitEvent { repository, hash, message, author, .. } => format!(
                "Commit {} by {} in {}: {}",
                &hash[..8],
                author,
                repository,
                message.lines().next().unwrap_or("").chars().take(30).collect::<String>()
            ),

            EventType::BranchEvent { repository, branch, created, .. } => {
                if *created {
                    format!("Branch {} created in {}", branch, repository)
                } else {
                    format!("Branch {} deleted in {}", branch, repository)
                }
            }

            EventType::TagEvent { repository, tag, created, .. } => {
                if *created {
                    format!("Tag {} created in {}", tag, repository)
                } else {
                    format!("Tag {} deleted in {}", tag, repository)
                }
            }

            EventType::PackageChanged { repository, package, file_count, deps_changed } => {
                if *deps_changed {
                    format!(
                        "Package {} changed in {} ({} files, dependencies updated)",
                        package, repository, file_count
                    )
                } else {
                    format!("Package {} changed in {} ({} files)", package, repository, file_count)
                }
            }

            EventType::PackageVersionChanged {
                repository,
                package,
                previous_version,
                new_version,
            } => format!(
                "Package {} version changed: {} → {} in {}",
                package, previous_version, new_version, repository
            ),

            EventType::RepositorySync { repository, operation, target, success } => {
                if *success {
                    format!("{} {} successful for {}", operation, target, repository)
                } else {
                    format!("{} {} failed for {}", operation, target, repository)
                }
            }

            EventType::DaemonStatusChanged { status } => format!("Daemon status: {}", status),

            EventType::Notification { message, level, repository } => {
                if let Some(repo) = repository {
                    format!("{:?} notification for {}: {}", level, repo, message)
                } else {
                    format!("{:?} notification: {}", level, message)
                }
            }

            EventType::Error { message, repository, .. } => {
                if let Some(repo) = repository {
                    format!("Error in {}: {}", repo, message)
                } else {
                    format!("Error: {}", message)
                }
            }

            EventType::TaskEvent { name, started, exit_code, duration_ms } => {
                if *started {
                    format!("Task started: {}", name)
                } else if let Some(code) = exit_code {
                    if *code == 0 {
                        format!(
                            "Task completed successfully: {} ({}ms)",
                            name,
                            duration_ms.unwrap_or(0)
                        )
                    } else {
                        format!("Task failed: {} (exit code {})", name, code)
                    }
                } else {
                    format!("Task finished: {}", name)
                }
            }

            EventType::Custom { name, .. } => format!("Custom event: {}", name),
        }
    }

    /// Check if event is an error
    pub fn is_error(&self) -> bool {
        matches!(
            self.event_type,
            EventType::Error { .. }
                | EventType::Notification { level: NotificationLevel::Error, .. }
        )
    }

    /// Check if event is high priority
    pub fn is_high_priority(&self) -> bool {
        self.priority >= 75
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.formatted_time(), self.short_description())
    }
}

impl fmt::Display for FileStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileStatus::Added => write!(f, "Added"),
            FileStatus::Modified => write!(f, "Modified"),
            FileStatus::Deleted => write!(f, "Deleted"),
            FileStatus::Renamed(path) => write!(f, "Renamed to {}", path.display()),
            FileStatus::Conflict => write!(f, "Conflict"),
            FileStatus::Moved(path) => write!(f, "Moved to {}", path.display()),
            FileStatus::PermissionChanged => write!(f, "Permissions changed"),
            FileStatus::Staged => write!(f, "Staged"),
            FileStatus::Unstaged => write!(f, "Unstaged"),
        }
    }
}

impl fmt::Display for NotificationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationLevel::Info => write!(f, "Info"),
            NotificationLevel::Warning => write!(f, "Warning"),
            NotificationLevel::Error => write!(f, "Error"),
            NotificationLevel::Success => write!(f, "Success"),
            NotificationLevel::Debug => write!(f, "Debug"),
        }
    }
}

/// Convenience function to create an error event
pub fn error_event(message: &str, repository: Option<&str>) -> Event {
    Event::new_high_priority(EventType::Error {
        message: message.to_string(),
        repository: repository.map(ToString::to_string),
        error_type: None,
    })
}

/// Convenience function to create an info notification event
pub fn info_event(message: &str, repository: Option<&str>) -> Event {
    Event::new(EventType::Notification {
        message: message.to_string(),
        level: NotificationLevel::Info,
        repository: repository.map(ToString::to_string),
    })
}

/// Convenience function to create a warning notification event
pub fn warning_event(message: &str, repository: Option<&str>) -> Event {
    Event::new_with_priority(
        EventType::Notification {
            message: message.to_string(),
            level: NotificationLevel::Warning,
            repository: repository.map(ToString::to_string),
        },
        70,
    )
}

/// Convenience function to create a success notification event
pub fn success_event(message: &str, repository: Option<&str>) -> Event {
    Event::new(EventType::Notification {
        message: message.to_string(),
        level: NotificationLevel::Success,
        repository: repository.map(ToString::to_string),
    })
}
