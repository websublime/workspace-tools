//! Event handler traits and implementations
//!
//! This module provides traits and utilities for handling events in the monorepo system.
//! Components can implement these traits to receive and process events from other components.

use super::MonorepoEvent;
use crate::error::Result;
use async_trait::async_trait;
use std::fmt;
use std::future::Future;

/// Synchronous event handler trait
pub trait EventHandler: Send + Sync {
    /// Handle an event synchronously
    fn handle_event(&self, event: MonorepoEvent) -> Result<()>;
    
    /// Get handler name for debugging
    fn handler_name(&self) -> &str;
    
    /// Check if this handler can process the given event
    fn can_handle(&self, event: &MonorepoEvent) -> bool {
        // Default implementation accepts all events
        let _ = event;
        true
    }
}

/// Asynchronous event handler trait
#[async_trait]
pub trait AsyncEventHandler: Send + Sync {
    /// Handle an event asynchronously
    async fn handle_event(&self, event: MonorepoEvent) -> Result<()>;
    
    /// Get handler name for debugging
    fn handler_name(&self) -> &str;
    
    /// Check if this handler can process the given event
    fn can_handle(&self, event: &MonorepoEvent) -> bool {
        // Default implementation accepts all events
        let _ = event;
        true
    }
    
    /// Called when handler is registered with event bus
    async fn on_register(&self) -> Result<()> {
        Ok(())
    }
    
    /// Called when handler is unregistered from event bus
    async fn on_unregister(&self) -> Result<()> {
        Ok(())
    }
}

/// A trait object wrapper for AsyncEventHandler to make it object-safe
pub struct AsyncEventHandlerWrapper {
    handler: Box<dyn AsyncEventHandlerTrait>,
}

impl fmt::Debug for AsyncEventHandlerWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncEventHandlerWrapper")
            .field("handler_name", &self.handler_name())
            .finish()
    }
}

impl AsyncEventHandlerWrapper {
    /// Create a new wrapper
    pub fn new<T: AsyncEventHandler + 'static>(handler: T) -> Self {
        Self {
            handler: Box::new(AsyncEventHandlerImpl(handler)),
        }
    }
}

#[async_trait]
impl AsyncEventHandler for AsyncEventHandlerWrapper {
    async fn handle_event(&self, event: MonorepoEvent) -> Result<()> {
        self.handler.handle_event_dyn(event).await
    }
    
    fn handler_name(&self) -> &str {
        self.handler.handler_name_dyn()
    }
    
    fn can_handle(&self, event: &MonorepoEvent) -> bool {
        self.handler.can_handle_dyn(event)
    }
    
    async fn on_register(&self) -> Result<()> {
        self.handler.on_register_dyn().await
    }
    
    async fn on_unregister(&self) -> Result<()> {
        self.handler.on_unregister_dyn().await
    }
}

/// Internal trait that is object-safe using boxed futures
#[async_trait]
trait AsyncEventHandlerTrait: Send + Sync {
    async fn handle_event_dyn(&self, event: MonorepoEvent) -> Result<()>;
    fn handler_name_dyn(&self) -> &str;
    fn can_handle_dyn(&self, event: &MonorepoEvent) -> bool;
    async fn on_register_dyn(&self) -> Result<()>;
    async fn on_unregister_dyn(&self) -> Result<()>;
}

/// Implementation wrapper
struct AsyncEventHandlerImpl<T: AsyncEventHandler>(T);

#[async_trait]
impl<T: AsyncEventHandler> AsyncEventHandlerTrait for AsyncEventHandlerImpl<T> {
    async fn handle_event_dyn(&self, event: MonorepoEvent) -> Result<()> {
        self.0.handle_event(event).await
    }
    
    fn handler_name_dyn(&self) -> &str {
        self.0.handler_name()
    }
    
    fn can_handle_dyn(&self, event: &MonorepoEvent) -> bool {
        self.0.can_handle(event)
    }
    
    async fn on_register_dyn(&self) -> Result<()> {
        self.0.on_register().await
    }
    
    async fn on_unregister_dyn(&self) -> Result<()> {
        self.0.on_unregister().await
    }
}

/// Function-based event handler for simple use cases
pub struct FunctionHandler<F> 
where
    F: Fn(MonorepoEvent) -> Result<()> + Send + Sync,
{
    /// Handler function
    handler_fn: F,
    
    /// Handler name
    name: String,
}

impl<F> FunctionHandler<F>
where
    F: Fn(MonorepoEvent) -> Result<()> + Send + Sync,
{
    /// Create a new function-based handler
    #[must_use]
    pub fn new(name: impl Into<String>, handler_fn: F) -> Self {
        Self {
            handler_fn,
            name: name.into(),
        }
    }
}

impl<F> EventHandler for FunctionHandler<F>
where
    F: Fn(MonorepoEvent) -> Result<()> + Send + Sync,
{
    fn handle_event(&self, event: MonorepoEvent) -> Result<()> {
        (self.handler_fn)(event)
    }
    
    fn handler_name(&self) -> &str {
        &self.name
    }
}

/// Async function-based event handler
pub struct AsyncFunctionHandler<F, Fut>
where
    F: Fn(MonorepoEvent) -> Fut + Send + Sync,
    Fut: Future<Output = Result<()>> + Send,
{
    /// Handler function
    handler_fn: F,
    
    /// Handler name
    name: String,
}

impl<F, Fut> AsyncFunctionHandler<F, Fut>
where
    F: Fn(MonorepoEvent) -> Fut + Send + Sync,
    Fut: Future<Output = Result<()>> + Send,
{
    /// Create a new async function-based handler
    #[must_use]
    pub fn new(name: impl Into<String>, handler_fn: F) -> Self {
        Self {
            handler_fn,
            name: name.into(),
        }
    }
}

#[async_trait]
impl<F, Fut> AsyncEventHandler for AsyncFunctionHandler<F, Fut>
where
    F: Fn(MonorepoEvent) -> Fut + Send + Sync,
    Fut: Future<Output = Result<()>> + Send,
{
    async fn handle_event(&self, event: MonorepoEvent) -> Result<()> {
        (self.handler_fn)(event).await
    }
    
    fn handler_name(&self) -> &str {
        &self.name
    }
}

/// Delegating handler that routes events to multiple sub-handlers
pub struct DelegatingHandler {
    /// Sub-handlers
    handlers: Vec<Box<dyn AsyncEventHandler>>,
    
    /// Handler name
    name: String,
}

impl DelegatingHandler {
    /// Create a new delegating handler
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            handlers: Vec::new(),
            name: name.into(),
        }
    }
    
    /// Add a sub-handler
    pub fn add_handler(&mut self, handler: Box<dyn AsyncEventHandler>) {
        self.handlers.push(handler);
    }
    
    /// Remove all handlers
    pub fn clear_handlers(&mut self) {
        self.handlers.clear();
    }
}

#[async_trait]
impl AsyncEventHandler for DelegatingHandler {
    async fn handle_event(&self, event: MonorepoEvent) -> Result<()> {
        for handler in &self.handlers {
            if handler.can_handle(&event) {
                if let Err(e) = handler.handle_event(event.clone()).await {
                    log::error!(
                        "Sub-handler '{}' failed to process event: {}", 
                        handler.handler_name(), 
                        e
                    );
                    // Continue processing other handlers
                }
            }
        }
        Ok(())
    }
    
    fn handler_name(&self) -> &str {
        &self.name
    }
    
    async fn on_register(&self) -> Result<()> {
        for handler in &self.handlers {
            handler.on_register().await?;
        }
        Ok(())
    }
    
    async fn on_unregister(&self) -> Result<()> {
        for handler in &self.handlers {
            handler.on_unregister().await?;
        }
        Ok(())
    }
}

/// Filtering handler that only processes events matching criteria
pub struct FilteringHandler<H>
where
    H: AsyncEventHandler,
{
    /// Underlying handler
    inner: H,
    
    /// Filter predicate
    filter: Box<dyn Fn(&MonorepoEvent) -> bool + Send + Sync>,
    
    /// Handler name
    name: String,
}

impl<H> FilteringHandler<H>
where
    H: AsyncEventHandler,
{
    /// Create a new filtering handler
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        inner: H,
        filter: Box<dyn Fn(&MonorepoEvent) -> bool + Send + Sync>,
    ) -> Self {
        Self {
            inner,
            filter,
            name: name.into(),
        }
    }
}

#[async_trait]
impl<H> AsyncEventHandler for FilteringHandler<H>
where
    H: AsyncEventHandler,
{
    async fn handle_event(&self, event: MonorepoEvent) -> Result<()> {
        if (self.filter)(&event) {
            self.inner.handle_event(event).await
        } else {
            Ok(())
        }
    }
    
    fn handler_name(&self) -> &str {
        &self.name
    }
    
    fn can_handle(&self, event: &MonorepoEvent) -> bool {
        (self.filter)(event) && self.inner.can_handle(event)
    }
    
    async fn on_register(&self) -> Result<()> {
        self.inner.on_register().await
    }
    
    async fn on_unregister(&self) -> Result<()> {
        self.inner.on_unregister().await
    }
}

/// Logging handler that logs all events
pub struct LoggingHandler {
    /// Log level
    level: log::Level,
    
    /// Handler name
    name: String,
}

impl LoggingHandler {
    /// Create a new logging handler
    #[must_use]
    pub fn new(name: impl Into<String>, level: log::Level) -> Self {
        Self {
            level,
            name: name.into(),
        }
    }
}

#[async_trait]
impl AsyncEventHandler for LoggingHandler {
    async fn handle_event(&self, event: MonorepoEvent) -> Result<()> {
        log::log!(
            self.level,
            "Event [{}]: {} from {} at {}",
            event.context().event_id,
            Self::get_event_description(&event),
            event.source(),
            event.context().timestamp.format("%H:%M:%S%.3f")
        );
        Ok(())
    }
    
    fn handler_name(&self) -> &str {
        &self.name
    }
}

impl LoggingHandler {
    /// Get human-readable event description
    fn get_event_description(event: &MonorepoEvent) -> String {
        match event {
            MonorepoEvent::Config(config_event) => match config_event {
                super::types::ConfigEvent::Updated { section, .. } => format!("Config updated: {section}"),
                super::types::ConfigEvent::Reloaded { .. } => "Config reloaded".to_string(),
                super::types::ConfigEvent::ValidationFailed { .. } => "Config validation failed".to_string(),
            },
            MonorepoEvent::Task(task_event) => match task_event {
                super::types::TaskEvent::Started { task_name, .. } => format!("Task started: {task_name}"),
                super::types::TaskEvent::Completed { result, .. } => format!("Task completed: {}", result.task_name),
                super::types::TaskEvent::Failed { task_name, .. } => format!("Task failed: {task_name}"),
                super::types::TaskEvent::ValidationRequested { task_name, .. } => format!("Task validation requested: {task_name}"),
            },
            MonorepoEvent::Changeset(changeset_event) => match changeset_event {
                super::types::ChangesetEvent::Created { changeset, .. } => format!("Changeset created: {}", changeset.id),
                super::types::ChangesetEvent::CreationRequested { .. } => "Changeset creation requested".to_string(),
                super::types::ChangesetEvent::Validated { changeset_id, .. } => format!("Changeset validated: {changeset_id}"),
                super::types::ChangesetEvent::Applied { changesets, .. } => format!("Changesets applied: {}", changesets.len()),
            },
            MonorepoEvent::Hook(hook_event) => match hook_event {
                super::types::HookEvent::Started { hook_type, .. } => format!("Hook started: {hook_type}"),
                super::types::HookEvent::Completed { hook_type, success, .. } => format!("Hook completed: {hook_type} ({})", if *success { "success" } else { "failed" }),
                super::types::HookEvent::Installed { hook_types, .. } => format!("Hooks installed: {}", hook_types.len()),
                super::types::HookEvent::ValidationFailed { hook_type, .. } => format!("Hook validation failed: {hook_type}"),
            },
            MonorepoEvent::Package(package_event) => match package_event {
                super::types::PackageEvent::Updated { package_name, new_version, .. } => format!("Package updated: {package_name}@{new_version}"),
                super::types::PackageEvent::DependenciesChanged { package_name, .. } => format!("Dependencies changed: {package_name}"),
                super::types::PackageEvent::Published { package_name, version, .. } => format!("Package published: {package_name}@{version}"),
                super::types::PackageEvent::DiscoveryCompleted { packages, .. } => format!("Package discovery completed: {} packages", packages.len()),
            },
            MonorepoEvent::FileSystem(fs_event) => match fs_event {
                super::types::FileSystemEvent::FilesChanged { changed_files, .. } => format!("Files changed: {} files", changed_files.len()),
                super::types::FileSystemEvent::WorkspaceChanged { added_packages, removed_packages, .. } => format!("Workspace changed: +{} -{} packages", added_packages.len(), removed_packages.len()),
                super::types::FileSystemEvent::ConfigFileChanged { .. } => "Config file changed".to_string(),
            },
            MonorepoEvent::Workflow(workflow_event) => match workflow_event {
                super::types::WorkflowEvent::Started { workflow_type, .. } => format!("Workflow started: {workflow_type}"),
                super::types::WorkflowEvent::Completed { workflow_type, success, .. } => format!("Workflow completed: {workflow_type} ({})", if *success { "success" } else { "failed" }),
                super::types::WorkflowEvent::StageCompleted { workflow_type, stage, .. } => format!("Workflow stage completed: {workflow_type}::{stage}"),
            },
        }
    }
}

/// Statistics collecting handler
pub struct StatsHandler {
    /// Handler name
    name: String,
}

impl StatsHandler {
    /// Create a new statistics handler
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
}

#[async_trait]
impl AsyncEventHandler for StatsHandler {
    async fn handle_event(&self, event: MonorepoEvent) -> Result<()> {
        // In a real implementation, this would update metrics/statistics
        // For now, we just log the event processing
        log::debug!(
            "Statistics: Processed {} event from {} with priority {:?}",
            match &event {
                MonorepoEvent::Config(_) => "Config",
                MonorepoEvent::Task(_) => "Task",
                MonorepoEvent::Changeset(_) => "Changeset",
                MonorepoEvent::Hook(_) => "Hook",
                MonorepoEvent::Package(_) => "Package",
                MonorepoEvent::FileSystem(_) => "FileSystem",
                MonorepoEvent::Workflow(_) => "Workflow",
            },
            event.source(),
            event.priority()
        );
        Ok(())
    }
    
    fn handler_name(&self) -> &str {
        &self.name
    }
}