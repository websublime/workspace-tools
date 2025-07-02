//! Event type definitions for the monorepo system
//!
//! This module defines all events that can be emitted and consumed by components
//! in the monorepo system. Events are categorized by their source and purpose.

use crate::changesets::Changeset;
use crate::tasks::TaskExecutionResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Priority levels for event processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EventPriority {
    /// Low priority - background operations
    Low = 0,
    /// Normal priority - regular operations
    Normal = 1,
    /// High priority - user-triggered operations
    High = 2,
    /// Critical priority - system integrity operations
    Critical = 3,
}

/// Context information for events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContext {
    /// Unique event identifier
    pub event_id: Uuid,
    
    /// Timestamp when event was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Priority of the event
    pub priority: EventPriority,
    
    /// Source component that emitted the event
    pub source: String,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl EventContext {
    /// Create a new event context
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            priority: EventPriority::Normal,
            source: source.into(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set event priority
    #[must_use]
    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// All possible events in the monorepo system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonorepoEvent {
    /// Configuration-related events
    Config(ConfigEvent),
    
    /// Task execution events
    Task(TaskEvent),
    
    /// Changeset management events
    Changeset(ChangesetEvent),
    
    /// Git hook events
    Hook(HookEvent),
    
    /// Package management events
    Package(PackageEvent),
    
    /// File system events
    FileSystem(FileSystemEvent),
    
    /// Workflow events
    Workflow(WorkflowEvent),
}

/// Configuration change events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigEvent {
    /// Configuration was updated
    Updated {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Configuration section that was modified
        section: String,
        /// Map of changed configuration values
        changes: HashMap<String, serde_json::Value>,
    },
    
    /// Configuration was reloaded from file
    Reloaded {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Path to the configuration file that was reloaded
        config_path: PathBuf,
    },
    
    /// Configuration validation failed
    ValidationFailed {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// List of validation error messages
        errors: Vec<String>,
    },
}

/// Task execution events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskEvent {
    /// Task execution started
    Started {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Name of the task being executed
        task_name: String,
        /// List of packages affected by this task
        packages: Vec<String>,
    },
    
    /// Task execution completed
    Completed {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Result of the task execution
        result: Box<TaskExecutionResult>,
    },
    
    /// Task execution failed
    Failed {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Name of the task that failed
        task_name: String,
        /// Error message describing the failure
        error: String,
    },
    
    /// Request to execute task for validation
    ValidationRequested {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Name of the task to validate
        task_name: String,
        /// List of packages to validate
        packages: Vec<String>,
    },
}

/// Changeset management events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangesetEvent {
    /// Changeset was created
    Created {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// The created changeset
        changeset: Changeset,
    },
    
    /// Changeset creation was requested
    CreationRequested {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// List of packages that need changesets
        packages: Vec<String>,
        /// Reason for requesting changeset creation
        reason: String,
    },
    
    /// Changeset validation completed
    Validated {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Unique identifier of the validated changeset
        changeset_id: String,
        /// Whether the changeset passed validation
        is_valid: bool,
        /// List of validation errors if any
        errors: Vec<String>,
    },
    
    /// Changesets were applied/consumed
    Applied {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// List of changeset IDs that were applied
        changesets: Vec<String>,
        /// List of packages affected by the applied changesets
        packages: Vec<String>,
    },
}

/// Git hook events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookEvent {
    /// Hook execution started
    Started {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Type of hook being executed (pre-commit, pre-push, etc.)
        hook_type: String,
        /// List of packages affected by the hook execution
        affected_packages: Vec<String>,
    },
    
    /// Hook execution completed
    Completed {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Type of hook that completed execution
        hook_type: String,
        /// Whether the hook execution was successful
        success: bool,
        /// Optional message from hook execution
        message: Option<String>,
    },
    
    /// Hook installation completed
    Installed {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// List of hook types that were installed
        hook_types: Vec<String>,
    },
    
    /// Hook validation failed
    ValidationFailed {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Type of hook that failed validation
        hook_type: String,
        /// List of required actions to fix validation
        required_actions: Vec<String>,
    },
}

/// Package-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageEvent {
    /// Package was updated
    Updated {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Name of the package that was updated
        package_name: String,
        /// Previous version of the package
        old_version: String,
        /// New version of the package
        new_version: String,
    },
    
    /// Package dependencies changed
    DependenciesChanged {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Name of the package with dependency changes
        package_name: String,
        /// List of newly added dependencies
        added: Vec<String>,
        /// List of removed dependencies
        removed: Vec<String>,
        /// List of updated dependencies
        updated: Vec<String>,
    },
    
    /// Package was published
    Published {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Name of the package that was published
        package_name: String,
        /// Version that was published
        version: String,
        /// Registry where the package was published
        registry: String,
    },
    
    /// Package discovery completed
    DiscoveryCompleted {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// List of packages that were discovered
        packages: Vec<String>,
        /// Workspace patterns used for discovery
        patterns: Vec<String>,
    },
}

/// File system events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSystemEvent {
    /// Files were changed
    FilesChanged {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// List of file paths that were changed
        changed_files: Vec<PathBuf>,
        /// List of packages affected by the file changes
        affected_packages: Vec<String>,
    },
    
    /// Workspace structure changed
    WorkspaceChanged {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// List of packages that were added to the workspace
        added_packages: Vec<String>,
        /// List of packages that were removed from the workspace
        removed_packages: Vec<String>,
    },
    
    /// Configuration file changed
    ConfigFileChanged {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Path to the configuration file that changed
        config_path: PathBuf,
    },
}

/// Workflow execution events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    /// Workflow started
    Started {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Type of workflow being executed
        workflow_type: String,
        /// List of packages targeted by the workflow
        target_packages: Vec<String>,
    },
    
    /// Workflow completed
    Completed {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Type of workflow that completed
        workflow_type: String,
        /// Whether the workflow completed successfully
        success: bool,
        /// Results and outputs from the workflow execution
        results: HashMap<String, serde_json::Value>,
    },
    
    /// Workflow stage completed
    StageCompleted {
        /// Event execution context with metadata and timing
        context: EventContext,
        /// Type of workflow containing the completed stage
        workflow_type: String,
        /// Name of the stage that completed
        stage: String,
        /// Whether the stage completed successfully
        success: bool,
    },
}

impl MonorepoEvent {
    /// Get the event context
    #[must_use]
    pub fn context(&self) -> &EventContext {
        match self {
            Self::Config(event) => match event {
                ConfigEvent::Updated { context, .. } |
                ConfigEvent::Reloaded { context, .. } |
                ConfigEvent::ValidationFailed { context, .. } => context,
            },
            Self::Task(event) => match event {
                TaskEvent::Started { context, .. } |
                TaskEvent::Completed { context, .. } |
                TaskEvent::Failed { context, .. } |
                TaskEvent::ValidationRequested { context, .. } => context,
            },
            Self::Changeset(event) => match event {
                ChangesetEvent::Created { context, .. } |
                ChangesetEvent::CreationRequested { context, .. } |
                ChangesetEvent::Validated { context, .. } |
                ChangesetEvent::Applied { context, .. } => context,
            },
            Self::Hook(event) => match event {
                HookEvent::Started { context, .. } |
                HookEvent::Completed { context, .. } |
                HookEvent::Installed { context, .. } |
                HookEvent::ValidationFailed { context, .. } => context,
            },
            Self::Package(event) => match event {
                PackageEvent::Updated { context, .. } |
                PackageEvent::DependenciesChanged { context, .. } |
                PackageEvent::Published { context, .. } |
                PackageEvent::DiscoveryCompleted { context, .. } => context,
            },
            Self::FileSystem(event) => match event {
                FileSystemEvent::FilesChanged { context, .. } |
                FileSystemEvent::WorkspaceChanged { context, .. } |
                FileSystemEvent::ConfigFileChanged { context, .. } => context,
            },
            Self::Workflow(event) => match event {
                WorkflowEvent::Started { context, .. } |
                WorkflowEvent::Completed { context, .. } |
                WorkflowEvent::StageCompleted { context, .. } => context,
            },
        }
    }
    
    /// Get the event priority
    #[must_use]
    pub fn priority(&self) -> EventPriority {
        self.context().priority
    }
    
    /// Get the event source component
    #[must_use]
    pub fn source(&self) -> &str {
        &self.context().source
    }
}

impl Default for EventPriority {
    fn default() -> Self {
        Self::Normal
    }
}