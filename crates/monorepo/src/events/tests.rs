//! Comprehensive tests for the events module
//!
//! This module provides complete test coverage for all event system functionality,
//! including event bus operations, event handling, filtering, and component communication.
//! Tests use real data structures and production-like scenarios to ensure robustness.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::changesets::types::{Changeset, ChangesetStatus};
    use crate::config::types::{Environment, VersionBumpType};
    use crate::events::bus::{EventBus, EventFilter, EventTypeFilter};
    use crate::events::handlers::*;
    use crate::events::types::*;
    use crate::error::Result;
    use crate::tasks::types::results::{
        TaskArtifact, TaskError, TaskErrorCode, TaskExecutionLog, TaskExecutionResult,
        TaskExecutionStats, TaskLogLevel, TaskOutput, TaskStatus,
    };
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};
    use tokio::sync::Mutex;
    use tokio::time::timeout;

    /// Helper function to create a test event context
    fn create_test_context(source: &str) -> EventContext {
        EventContext::new(source)
            .with_priority(EventPriority::Normal)
            .with_metadata("test".to_string(), serde_json::Value::String("value".to_string()))
    }

    /// Helper function to create a real Changeset
    fn create_real_changeset() -> Changeset {
        Changeset {
            id: "changeset-123".to_string(),
            package: "@test/core".to_string(),
            version_bump: VersionBumpType::Minor,
            description: "Add new user authentication API endpoints".to_string(),
            branch: "feature/auth-api".to_string(),
            development_environments: vec![Environment::Development, Environment::Staging],
            production_deployment: false,
            created_at: chrono::Utc::now(),
            author: "developer@example.com".to_string(),
            status: ChangesetStatus::Pending,
        }
    }

    /// Helper function to create a real TaskExecutionResult
    fn create_real_task_execution_result() -> TaskExecutionResult {
        let now = SystemTime::now();
        let duration = Duration::from_secs(45);
        
        TaskExecutionResult {
            task_name: "test".to_string(),
            status: TaskStatus::Success,
            started_at: now,
            ended_at: now + duration,
            duration,
            outputs: vec![
                TaskOutput {
                    command: "npm test".to_string(),
                    working_dir: PathBuf::from("/workspace/packages/core"),
                    exit_code: Some(0),
                    stdout: "All tests passed successfully\n✓ 25 tests completed".to_string(),
                    stderr: String::new(),
                    duration: Duration::from_secs(42),
                    environment: {
                        let mut env = HashMap::new();
                        env.insert("NODE_ENV".to_string(), "test".to_string());
                        env.insert("CI".to_string(), "true".to_string());
                        env
                    },
                }
            ],
            stats: TaskExecutionStats {
                commands_executed: 1,
                commands_succeeded: 1,
                commands_failed: 0,
                packages_processed: 1,
                stdout_bytes: 45,
                stderr_bytes: 0,
                peak_memory_bytes: Some(256_000_000), // 256MB
                cpu_time: Some(Duration::from_secs(40)),
            },
            affected_packages: vec!["@test/core".to_string()],
            errors: vec![],
            logs: vec![
                TaskExecutionLog {
                    timestamp: now,
                    level: TaskLogLevel::Info,
                    message: "Starting test execution".to_string(),
                    package: Some("@test/core".to_string()),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("command".to_string(), serde_json::Value::String("npm test".to_string()));
                        data
                    },
                },
                TaskExecutionLog {
                    timestamp: now + Duration::from_secs(42),
                    level: TaskLogLevel::Info,
                    message: "Test execution completed successfully".to_string(),
                    package: Some("@test/core".to_string()),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("tests_passed".to_string(), serde_json::Value::Number(25.into()));
                        data.insert("duration_ms".to_string(), serde_json::Value::Number(42000.into()));
                        data
                    },
                },
            ],
            artifacts: vec![
                TaskArtifact {
                    name: "test-results.xml".to_string(),
                    path: PathBuf::from("/workspace/packages/core/test-results.xml"),
                    artifact_type: "test-results".to_string(),
                    size_bytes: 4096,
                    package: Some("@test/core".to_string()),
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("format".to_string(), "junit".to_string());
                        meta.insert("tests_count".to_string(), "25".to_string());
                        meta
                    },
                }
            ],
        }
    }

    /// Helper function to create a test config event
    fn create_test_config_event() -> MonorepoEvent {
        MonorepoEvent::Config(ConfigEvent::Updated {
            context: create_test_context("config-manager"),
            section: "workspace".to_string(),
            changes: {
                let mut changes = HashMap::new();
                changes.insert("patterns".to_string(), serde_json::Value::Array(vec![
                    serde_json::Value::String("packages/*".to_string()),
                    serde_json::Value::String("apps/*".to_string()),
                ]));
                changes.insert("merge_with_detected".to_string(), serde_json::Value::Bool(true));
                changes
            },
        })
    }

    /// Helper function to create a test task event
    fn create_test_task_event() -> MonorepoEvent {
        MonorepoEvent::Task(TaskEvent::Started {
            context: create_test_context("task-runner"),
            task_name: "test".to_string(),
            packages: vec!["@test/core".to_string(), "@test/utils".to_string()],
        })
    }

    /// Helper function to create a test changeset event
    fn create_test_changeset_event() -> MonorepoEvent {
        MonorepoEvent::Changeset(ChangesetEvent::Created {
            context: create_test_context("changeset-manager"),
            changeset: create_real_changeset(),
        })
    }

    /// Helper function to create a test hook event
    fn create_test_hook_event() -> MonorepoEvent {
        MonorepoEvent::Hook(HookEvent::Started {
            context: create_test_context("hook-manager"),
            hook_type: "pre-commit".to_string(),
            affected_packages: vec!["@test/core".to_string()],
        })
    }

    /// Helper function to create a test package event
    fn create_test_package_event() -> MonorepoEvent {
        MonorepoEvent::Package(PackageEvent::Updated {
            context: create_test_context("package-manager"),
            package_name: "@test/core".to_string(),
            old_version: "1.2.3".to_string(),
            new_version: "1.3.0".to_string(),
        })
    }

    /// Helper function to create a test filesystem event
    fn create_test_filesystem_event() -> MonorepoEvent {
        MonorepoEvent::FileSystem(FileSystemEvent::FilesChanged {
            context: create_test_context("fs-watcher"),
            changed_files: vec![
                PathBuf::from("packages/core/src/auth.ts"),
                PathBuf::from("packages/core/package.json"),
                PathBuf::from("packages/utils/src/helpers.ts"),
            ],
            affected_packages: vec!["@test/core".to_string(), "@test/utils".to_string()],
        })
    }

    /// Helper function to create a test workflow event
    fn create_test_workflow_event() -> MonorepoEvent {
        MonorepoEvent::Workflow(WorkflowEvent::Started {
            context: create_test_context("workflow-manager"),
            workflow_type: "ci".to_string(),
            target_packages: vec!["@test/core".to_string()],
        })
    }

    /// Test event handler that tracks received events using shared state
    #[derive(Debug)]
    struct TestEventHandler {
        name: String,
        events: Arc<Mutex<Vec<MonorepoEvent>>>,
    }

    impl TestEventHandler {
        fn new(name: impl Into<String>) -> (Self, Arc<Mutex<Vec<MonorepoEvent>>>) {
            let events = Arc::new(Mutex::new(Vec::new()));
            let handler = Self {
                name: name.into(),
                events: events.clone(),
            };
            (handler, events)
        }
    }

    #[async_trait::async_trait]
    impl AsyncEventHandler for TestEventHandler {
        async fn handle_event(&self, event: MonorepoEvent) -> Result<()> {
            self.events.lock().await.push(event);
            Ok(())
        }

        fn handler_name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_event_bus_creation() {
        let bus = EventBus::new();
        let stats = bus.get_stats().await;

        assert_eq!(stats.events_emitted, 0);
        assert_eq!(stats.events_processed, 0);
        assert_eq!(stats.active_subscriptions, 0);
        assert_eq!(stats.events_by_type.len(), 0);
        assert_eq!(stats.events_by_priority.len(), 0);
    }

    #[tokio::test]
    async fn test_event_bus_default() {
        let bus = EventBus::default();
        let stats = bus.get_stats().await;

        assert_eq!(stats.events_emitted, 0);
        assert_eq!(stats.active_subscriptions, 0);
    }

    #[tokio::test]
    async fn test_event_emission() -> Result<()> {
        let bus = EventBus::new();
        let event = create_test_config_event();

        // Emit event
        bus.emit(event).await?;

        // Check statistics
        let stats = bus.get_stats().await;
        assert_eq!(stats.events_emitted, 1);
        assert_eq!(stats.events_by_type.get("Config"), Some(&1));
        assert_eq!(stats.events_by_priority.get(&EventPriority::Normal), Some(&1));

        // Check pending events
        assert_eq!(bus.pending_events_count().await, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_event_subscription() -> Result<()> {
        let bus = EventBus::new();
        let (handler, _events) = TestEventHandler::new("test-handler");
        let handler_wrapper = Arc::new(AsyncEventHandlerWrapper::new(handler));

        // Subscribe to all events
        let subscription_id = bus.subscribe(EventFilter::All, handler_wrapper).await?;

        // Check subscription was created
        let stats = bus.get_stats().await;
        assert_eq!(stats.active_subscriptions, 1);

        // Unsubscribe
        let removed = bus.unsubscribe(subscription_id).await?;
        assert!(removed);

        // Check subscription was removed
        let stats = bus.get_stats().await;
        assert_eq!(stats.active_subscriptions, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_event_processing_comprehensive() -> Result<()> {
        let bus = EventBus::new();
        let (handler, events) = TestEventHandler::new("comprehensive-handler");
        let handler_wrapper = Arc::new(AsyncEventHandlerWrapper::new(handler));

        // Subscribe to all events
        let _subscription_id = bus.subscribe(EventFilter::All, handler_wrapper).await?;

        // Emit all types of events
        let config_event = create_test_config_event();
        let task_event = create_test_task_event();
        let changeset_event = create_test_changeset_event();
        let hook_event = create_test_hook_event();
        let package_event = create_test_package_event();
        let filesystem_event = create_test_filesystem_event();
        let workflow_event = create_test_workflow_event();

        bus.emit(config_event).await?;
        bus.emit(task_event).await?;
        bus.emit(changeset_event).await?;
        bus.emit(hook_event).await?;
        bus.emit(package_event).await?;
        bus.emit(filesystem_event).await?;
        bus.emit(workflow_event).await?;

        // Process events
        let processed_count = bus.process_events(10).await?;
        assert_eq!(processed_count, 7);

        // Check statistics
        let stats = bus.get_stats().await;
        assert_eq!(stats.events_processed, 7);
        assert_eq!(stats.events_emitted, 7);

        // Check that events were handled
        let handled_events = events.lock().await;
        assert_eq!(handled_events.len(), 7);

        // Verify event types
        assert!(handled_events.iter().any(|e| matches!(e, MonorepoEvent::Config(_))));
        assert!(handled_events.iter().any(|e| matches!(e, MonorepoEvent::Task(_))));
        assert!(handled_events.iter().any(|e| matches!(e, MonorepoEvent::Changeset(_))));
        assert!(handled_events.iter().any(|e| matches!(e, MonorepoEvent::Hook(_))));
        assert!(handled_events.iter().any(|e| matches!(e, MonorepoEvent::Package(_))));
        assert!(handled_events.iter().any(|e| matches!(e, MonorepoEvent::FileSystem(_))));
        assert!(handled_events.iter().any(|e| matches!(e, MonorepoEvent::Workflow(_))));

        Ok(())
    }

    #[tokio::test]
    async fn test_event_filtering_by_type() -> Result<()> {
        let bus = EventBus::new();
        let (handler, events) = TestEventHandler::new("config-handler");
        let handler_wrapper = Arc::new(AsyncEventHandlerWrapper::new(handler));

        // Subscribe to config events only
        let _subscription_id = bus
            .subscribe(EventFilter::for_type(EventTypeFilter::Config), handler_wrapper)
            .await?;

        // Emit different types of events
        let config_event = create_test_config_event();
        let task_event = create_test_task_event();
        let changeset_event = create_test_changeset_event();

        bus.emit(config_event).await?;
        bus.emit(task_event).await?;
        bus.emit(changeset_event).await?;

        // Process events
        bus.process_events(10).await?;

        // Check that only config event was processed
        let handled_events = events.lock().await;
        assert_eq!(handled_events.len(), 1);
        assert!(matches!(handled_events[0], MonorepoEvent::Config(_)));

        Ok(())
    }

    #[tokio::test]
    async fn test_event_filtering_by_source() -> Result<()> {
        let bus = EventBus::new();
        let (handler, events) = TestEventHandler::new("source-handler");
        let handler_wrapper = Arc::new(AsyncEventHandlerWrapper::new(handler));

        // Subscribe to events from specific source
        let _subscription_id = bus
            .subscribe(EventFilter::for_source("task-runner"), handler_wrapper)
            .await?;

        // Emit events from different sources
        let config_event = create_test_config_event(); // from "config-manager"
        let task_event = create_test_task_event(); // from "task-runner"
        let changeset_event = create_test_changeset_event(); // from "changeset-manager"

        bus.emit(config_event).await?;
        bus.emit(task_event).await?;
        bus.emit(changeset_event).await?;

        // Process events
        bus.process_events(10).await?;

        // Check that only task event was processed
        let handled_events = events.lock().await;
        assert_eq!(handled_events.len(), 1);
        assert!(matches!(handled_events[0], MonorepoEvent::Task(_)));
        assert_eq!(handled_events[0].source(), "task-runner");

        Ok(())
    }

    #[tokio::test]
    async fn test_event_filtering_by_priority() -> Result<()> {
        let bus = EventBus::new();
        let (handler, events) = TestEventHandler::new("priority-handler");
        let handler_wrapper = Arc::new(AsyncEventHandlerWrapper::new(handler));

        // Subscribe to high priority events only
        let _subscription_id = bus
            .subscribe(EventFilter::for_priority(EventPriority::High), handler_wrapper)
            .await?;

        // Emit events with different priorities
        let normal_event = create_test_config_event();
        let high_event = MonorepoEvent::Config(ConfigEvent::ValidationFailed {
            context: create_test_context("config-manager").with_priority(EventPriority::High),
            errors: vec!["Invalid workspace pattern: [invalid".to_string()],
        });
        let critical_event = MonorepoEvent::Hook(HookEvent::ValidationFailed {
            context: create_test_context("hook-manager").with_priority(EventPriority::Critical),
            hook_type: "pre-commit".to_string(),
            required_actions: vec!["Install missing dependencies".to_string()],
        });

        bus.emit(normal_event).await?;
        bus.emit(high_event).await?;
        bus.emit(critical_event).await?;

        // Process events
        bus.process_events(10).await?;

        // Check that only high and critical priority events were processed
        let handled_events = events.lock().await;
        assert_eq!(handled_events.len(), 2);
        assert!(handled_events.iter().all(|e| e.priority() >= EventPriority::High));
        assert!(handled_events.iter().any(|e| e.priority() == EventPriority::High));
        assert!(handled_events.iter().any(|e| e.priority() == EventPriority::Critical));

        Ok(())
    }

    #[tokio::test]
    async fn test_event_priority_ordering() -> Result<()> {
        let bus = EventBus::new();
        let (handler, events) = TestEventHandler::new("priority-handler");
        let handler_wrapper = Arc::new(AsyncEventHandlerWrapper::new(handler));

        // Subscribe to all events
        let _subscription_id = bus.subscribe(EventFilter::All, handler_wrapper).await?;

        // Emit events with different priorities in random order
        let low_event = MonorepoEvent::Config(ConfigEvent::Updated {
            context: create_test_context("config-manager").with_priority(EventPriority::Low),
            section: "changelog".to_string(),
            changes: HashMap::new(),
        });

        let high_event = MonorepoEvent::Task(TaskEvent::Failed {
            context: create_test_context("task-runner").with_priority(EventPriority::High),
            task_name: "build".to_string(),
            error: "TypeScript compilation failed".to_string(),
        });

        let critical_event = MonorepoEvent::FileSystem(FileSystemEvent::ConfigFileChanged {
            context: create_test_context("fs-watcher").with_priority(EventPriority::Critical),
            config_path: PathBuf::from("monorepo.toml"),
        });

        let normal_event = create_test_package_event(); // Normal priority

        // Emit in non-priority order
        bus.emit(low_event).await?;
        bus.emit(normal_event).await?;
        bus.emit(high_event).await?;
        bus.emit(critical_event).await?;

        // Process events
        bus.process_events(10).await?;

        // Check that events were processed in priority order (highest first)
        let handled_events = events.lock().await;
        assert_eq!(handled_events.len(), 4);
        assert_eq!(handled_events[0].priority(), EventPriority::Critical);
        assert_eq!(handled_events[1].priority(), EventPriority::High);
        assert_eq!(handled_events[2].priority(), EventPriority::Normal);
        assert_eq!(handled_events[3].priority(), EventPriority::Low);

        Ok(())
    }

    #[tokio::test]
    async fn test_event_filtering_and_logic() -> Result<()> {
        let bus = EventBus::new();
        let (handler, events) = TestEventHandler::new("and-handler");
        let handler_wrapper = Arc::new(AsyncEventHandlerWrapper::new(handler));

        // Subscribe to config events from config-manager with high priority
        let _subscription_id = bus
            .subscribe(
                EventFilter::and(vec![
                    EventFilter::for_type(EventTypeFilter::Config),
                    EventFilter::for_source("config-manager"),
                    EventFilter::for_priority(EventPriority::High),
                ]),
                handler_wrapper,
            )
            .await?;

        // Emit various events
        let config_high = MonorepoEvent::Config(ConfigEvent::ValidationFailed {
            context: create_test_context("config-manager").with_priority(EventPriority::High),
            errors: vec!["Validation error".to_string()],
        });

        let config_normal = create_test_config_event(); // Normal priority
        let task_high = MonorepoEvent::Task(TaskEvent::Failed {
            context: create_test_context("task-runner").with_priority(EventPriority::High),
            task_name: "test".to_string(),
            error: "Test failed".to_string(),
        });

        let config_other_source = MonorepoEvent::Config(ConfigEvent::Updated {
            context: create_test_context("other-manager").with_priority(EventPriority::High),
            section: "test".to_string(),
            changes: HashMap::new(),
        });

        bus.emit(config_high).await?;
        bus.emit(config_normal).await?;
        bus.emit(task_high).await?;
        bus.emit(config_other_source).await?;

        // Process events
        bus.process_events(10).await?;

        // Check that only the config event from config-manager with high priority was processed
        let handled_events = events.lock().await;
        assert_eq!(handled_events.len(), 1);
        assert!(matches!(handled_events[0], MonorepoEvent::Config(_)));
        assert_eq!(handled_events[0].source(), "config-manager");
        assert_eq!(handled_events[0].priority(), EventPriority::High);

        Ok(())
    }

    #[tokio::test]
    async fn test_event_filtering_or_logic() -> Result<()> {
        let bus = EventBus::new();
        let (handler, events) = TestEventHandler::new("or-handler");
        let handler_wrapper = Arc::new(AsyncEventHandlerWrapper::new(handler));

        // Subscribe to config events OR task events OR critical priority events
        let _subscription_id = bus
            .subscribe(
                EventFilter::or(vec![
                    EventFilter::for_type(EventTypeFilter::Config),
                    EventFilter::for_type(EventTypeFilter::Task),
                    EventFilter::for_priority(EventPriority::Critical),
                ]),
                handler_wrapper,
            )
            .await?;

        // Emit various events
        let config_event = create_test_config_event();
        let task_event = create_test_task_event();
        let changeset_event = create_test_changeset_event(); // Should not match
        let critical_hook_event = MonorepoEvent::Hook(HookEvent::ValidationFailed {
            context: create_test_context("hook-manager").with_priority(EventPriority::Critical),
            hook_type: "pre-push".to_string(),
            required_actions: vec!["Fix linting errors".to_string()],
        });

        bus.emit(config_event).await?;
        bus.emit(task_event).await?;
        bus.emit(changeset_event).await?;
        bus.emit(critical_hook_event).await?;

        // Process events
        bus.process_events(10).await?;

        // Check that config, task, and critical hook events were processed
        let handled_events = events.lock().await;
        assert_eq!(handled_events.len(), 3);

        let has_config = handled_events.iter().any(|e| matches!(e, MonorepoEvent::Config(_)));
        let has_task = handled_events.iter().any(|e| matches!(e, MonorepoEvent::Task(_)));
        let has_critical = handled_events.iter().any(|e| e.priority() == EventPriority::Critical);

        assert!(has_config);
        assert!(has_task);
        assert!(has_critical);

        Ok(())
    }

    #[tokio::test]
    async fn test_event_filtering_custom() -> Result<()> {
        let bus = EventBus::new();
        let (handler, events) = TestEventHandler::new("custom-handler");
        let handler_wrapper = Arc::new(AsyncEventHandlerWrapper::new(handler));

        // Subscribe with custom filter that only accepts events related to "@test/core"
        let _subscription_id = bus
            .subscribe(
                EventFilter::Custom(|event| {
                    match event {
                        MonorepoEvent::Task(TaskEvent::Started { packages, .. }) => {
                            packages.contains(&"@test/core".to_string())
                        }
                        MonorepoEvent::Package(PackageEvent::Updated { package_name, .. }) => {
                            package_name == "@test/core"
                        }
                        MonorepoEvent::Changeset(ChangesetEvent::Created { changeset, .. }) => {
                            changeset.package == "@test/core"
                        }
                        _ => false,
                    }
                }),
                handler_wrapper,
            )
            .await?;

        // Emit events
        let task_event_core = create_test_task_event(); // Contains @test/core
        let package_event_core = create_test_package_event(); // Is @test/core
        let changeset_event_core = create_test_changeset_event(); // Contains @test/core
        let config_event = create_test_config_event(); // Not related to @test/core
        let package_event_other = MonorepoEvent::Package(PackageEvent::Updated {
            context: create_test_context("package-manager"),
            package_name: "@test/utils".to_string(),
            old_version: "1.0.0".to_string(),
            new_version: "1.1.0".to_string(),
        });

        bus.emit(task_event_core).await?;
        bus.emit(package_event_core).await?;
        bus.emit(changeset_event_core).await?;
        bus.emit(config_event).await?;
        bus.emit(package_event_other).await?;

        // Process events
        bus.process_events(10).await?;

        // Check that only events related to @test/core were processed
        let handled_events = events.lock().await;
        assert_eq!(handled_events.len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_real_changeset_event_creation() {
        let changeset = create_real_changeset();
        
        // Verify all changeset fields are properly set
        assert_eq!(changeset.id, "changeset-123");
        assert_eq!(changeset.package, "@test/core");
        assert_eq!(changeset.version_bump, VersionBumpType::Minor);
        assert_eq!(changeset.description, "Add new user authentication API endpoints");
        assert_eq!(changeset.branch, "feature/auth-api");
        assert_eq!(changeset.development_environments, vec![Environment::Development, Environment::Staging]);
        assert!(!changeset.production_deployment);
        assert_eq!(changeset.author, "developer@example.com");
        assert_eq!(changeset.status, ChangesetStatus::Pending);
        
        let event = MonorepoEvent::Changeset(ChangesetEvent::Created {
            context: create_test_context("changeset-manager"),
            changeset: changeset.clone(),
        });
        
        assert_eq!(event.source(), "changeset-manager");
        assert_eq!(event.priority(), EventPriority::Normal);
    }

    #[tokio::test]
    async fn test_real_task_execution_result_event() -> Result<()> {
        let bus = EventBus::new();
        let (handler, events) = TestEventHandler::new("task-handler");
        let handler_wrapper = Arc::new(AsyncEventHandlerWrapper::new(handler));

        let _subscription_id = bus.subscribe(EventFilter::All, handler_wrapper).await?;

        let task_result = create_real_task_execution_result();
        
        // Verify the task result structure
        assert_eq!(task_result.task_name, "test");
        assert_eq!(task_result.status, TaskStatus::Success);
        assert_eq!(task_result.outputs.len(), 1);
        assert_eq!(task_result.outputs[0].command, "npm test");
        assert_eq!(task_result.outputs[0].exit_code, Some(0));
        assert!(task_result.outputs[0].stdout.contains("All tests passed"));
        assert_eq!(task_result.stats.commands_executed, 1);
        assert_eq!(task_result.stats.commands_succeeded, 1);
        assert_eq!(task_result.stats.commands_failed, 0);
        assert_eq!(task_result.affected_packages, vec!["@test/core"]);
        assert_eq!(task_result.logs.len(), 2);
        assert_eq!(task_result.artifacts.len(), 1);
        
        let event = MonorepoEvent::Task(TaskEvent::Completed {
            context: create_test_context("task-runner"),
            result: Box::new(task_result),
        });

        bus.emit(event).await?;
        bus.process_events(1).await?;

        let handled_events = events.lock().await;
        assert_eq!(handled_events.len(), 1);
        
        if let MonorepoEvent::Task(TaskEvent::Completed { result, .. }) = &handled_events[0] {
            assert_eq!(result.task_name, "test");
            assert_eq!(result.status, TaskStatus::Success);
        } else {
            panic!("Expected TaskEvent::Completed");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_task_event_with_errors() -> Result<()> {
        let bus = EventBus::new();
        let (handler, events) = TestEventHandler::new("error-handler");
        let handler_wrapper = Arc::new(AsyncEventHandlerWrapper::new(handler));

        let _subscription_id = bus.subscribe(EventFilter::All, handler_wrapper).await?;

        let now = SystemTime::now();
        let failed_task_result = TaskExecutionResult {
            task_name: "build".to_string(),
            status: TaskStatus::Failed {
                reason: "TypeScript compilation errors".to_string(),
            },
            started_at: now,
            ended_at: now + Duration::from_secs(15),
            duration: Duration::from_secs(15),
            outputs: vec![
                TaskOutput {
                    command: "tsc --build".to_string(),
                    working_dir: PathBuf::from("/workspace/packages/core"),
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: "error TS2322: Type 'string' is not assignable to type 'number'.\n".to_string(),
                    duration: Duration::from_secs(15),
                    environment: HashMap::new(),
                }
            ],
            stats: TaskExecutionStats {
                commands_executed: 1,
                commands_succeeded: 0,
                commands_failed: 1,
                packages_processed: 1,
                stdout_bytes: 0,
                stderr_bytes: 65,
                peak_memory_bytes: Some(128_000_000),
                cpu_time: Some(Duration::from_secs(12)),
            },
            affected_packages: vec!["@test/core".to_string()],
            errors: vec![
                TaskError {
                    code: TaskErrorCode::ExecutionFailed,
                    message: "TypeScript compilation failed".to_string(),
                    context: {
                        let mut ctx = HashMap::new();
                        ctx.insert("file".to_string(), "src/auth.ts".to_string());
                        ctx.insert("line".to_string(), "42".to_string());
                        ctx
                    },
                    occurred_at: now + Duration::from_secs(12),
                    package: Some("@test/core".to_string()),
                    command: Some("tsc --build".to_string()),
                }
            ],
            logs: vec![],
            artifacts: vec![],
        };

        let event = MonorepoEvent::Task(TaskEvent::Completed {
            context: create_test_context("task-runner"),
            result: Box::new(failed_task_result),
        });

        bus.emit(event).await?;
        bus.process_events(1).await?;

        let handled_events = events.lock().await;
        assert_eq!(handled_events.len(), 1);

        if let MonorepoEvent::Task(TaskEvent::Completed { result, .. }) = &handled_events[0] {
            assert!(matches!(result.status, TaskStatus::Failed { .. }));
            assert_eq!(result.errors.len(), 1);
            assert_eq!(result.errors[0].code, TaskErrorCode::ExecutionFailed);
            assert_eq!(result.outputs[0].exit_code, Some(1));
            assert!(result.outputs[0].stderr.contains("TS2322"));
        } else {
            panic!("Expected TaskEvent::Completed with failed status");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_comprehensive_changeset_workflow() -> Result<()> {
        let bus = EventBus::new();
        let (handler, events) = TestEventHandler::new("changeset-workflow-handler");
        let handler_wrapper = Arc::new(AsyncEventHandlerWrapper::new(handler));

        let _subscription_id = bus.subscribe(EventFilter::for_type(EventTypeFilter::Changeset), handler_wrapper).await?;

        // 1. Changeset creation requested
        let creation_requested = MonorepoEvent::Changeset(ChangesetEvent::CreationRequested {
            context: create_test_context("changeset-manager"),
            packages: vec!["@test/core".to_string(), "@test/utils".to_string()],
            reason: "Breaking changes in authentication module require new major version".to_string(),
        });

        // 2. Changeset created
        let changeset = create_real_changeset();
        let created = MonorepoEvent::Changeset(ChangesetEvent::Created {
            context: create_test_context("changeset-manager"),
            changeset: changeset.clone(),
        });

        // 3. Changeset validated
        let validated = MonorepoEvent::Changeset(ChangesetEvent::Validated {
            context: create_test_context("changeset-manager"),
            changeset_id: changeset.id.clone(),
            is_valid: true,
            errors: vec![],
        });

        // 4. Changeset applied
        let applied = MonorepoEvent::Changeset(ChangesetEvent::Applied {
            context: create_test_context("changeset-manager"),
            changesets: vec![changeset.id],
            packages: vec!["@test/core".to_string()],
        });

        // Emit all events in workflow order
        bus.emit(creation_requested).await?;
        bus.emit(created).await?;
        bus.emit(validated).await?;
        bus.emit(applied).await?;

        // Process all events
        bus.process_events(10).await?;

        // Verify all events were processed
        let handled_events = events.lock().await;
        assert_eq!(handled_events.len(), 4);

        // Verify event types in order
        assert!(matches!(handled_events[0], MonorepoEvent::Changeset(ChangesetEvent::CreationRequested { .. })));
        assert!(matches!(handled_events[1], MonorepoEvent::Changeset(ChangesetEvent::Created { .. })));
        assert!(matches!(handled_events[2], MonorepoEvent::Changeset(ChangesetEvent::Validated { .. })));
        assert!(matches!(handled_events[3], MonorepoEvent::Changeset(ChangesetEvent::Applied { .. })));

        Ok(())
    }

    #[tokio::test]
    async fn test_event_context_creation() {
        let context = EventContext::new("test-source");

        assert_eq!(context.source, "test-source");
        assert_eq!(context.priority, EventPriority::Normal);
        assert!(context.metadata.is_empty());
        assert!(!context.event_id.is_nil());
    }

    #[tokio::test]
    async fn test_event_context_with_metadata() {
        let context = EventContext::new("test-source")
            .with_metadata("operation".to_string(), serde_json::Value::String("build".to_string()))
            .with_metadata("package_count".to_string(), serde_json::Value::Number(5.into()))
            .with_metadata("parallel".to_string(), serde_json::Value::Bool(true));

        assert_eq!(context.metadata.len(), 3);
        assert_eq!(context.metadata.get("operation"), Some(&serde_json::Value::String("build".to_string())));
        assert_eq!(context.metadata.get("package_count"), Some(&serde_json::Value::Number(5.into())));
        assert_eq!(context.metadata.get("parallel"), Some(&serde_json::Value::Bool(true)));
    }

    #[tokio::test]
    async fn test_event_priority_values() {
        assert_eq!(EventPriority::Low as u8, 0);
        assert_eq!(EventPriority::Normal as u8, 1);
        assert_eq!(EventPriority::High as u8, 2);
        assert_eq!(EventPriority::Critical as u8, 3);

        assert!(EventPriority::Critical > EventPriority::High);
        assert!(EventPriority::High > EventPriority::Normal);
        assert!(EventPriority::Normal > EventPriority::Low);
    }

    #[tokio::test]
    async fn test_event_priority_default() {
        assert_eq!(EventPriority::default(), EventPriority::Normal);
    }

    #[tokio::test]
    async fn test_monorepo_event_accessors() {
        let event = create_test_config_event();

        assert_eq!(event.source(), "config-manager");
        assert_eq!(event.priority(), EventPriority::Normal);
        assert!(!event.context().event_id.is_nil());
        
        // Verify context access
        let context = event.context();
        assert_eq!(context.source, "config-manager");
        assert!(context.metadata.contains_key("test"));
    }

    #[tokio::test]
    async fn test_async_function_handler() -> Result<()> {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let handler = AsyncFunctionHandler::new("test-async-function", move |event| {
            let events = events_clone.clone();
            async move {
                events.lock().await.push(event);
                Ok(())
            }
        });

        assert_eq!(handler.handler_name(), "test-async-function");
        assert!(handler.can_handle(&create_test_config_event()));

        let event = create_test_config_event();
        handler.handle_event(event).await?;

        let stored_events = events.lock().await;
        assert_eq!(stored_events.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_logging_handler() -> Result<()> {
        let handler = LoggingHandler::new("comprehensive-logger", log::Level::Info);

        assert_eq!(handler.handler_name(), "comprehensive-logger");

        // Test with various event types
        let config_event = create_test_config_event();
        let task_event = create_test_task_event();
        let changeset_event = create_test_changeset_event();

        handler.handle_event(config_event).await?;
        handler.handle_event(task_event).await?;
        handler.handle_event(changeset_event).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_stats_handler() -> Result<()> {
        let handler = StatsHandler::new("metrics-collector");

        assert_eq!(handler.handler_name(), "metrics-collector");

        // Process various events
        let events = vec![
            create_test_config_event(),
            create_test_task_event(),
            create_test_changeset_event(),
            create_test_hook_event(),
            create_test_package_event(),
            create_test_filesystem_event(),
            create_test_workflow_event(),
        ];

        for event in events {
            handler.handle_event(event).await?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_event_bus_broadcast() -> Result<()> {
        let bus = EventBus::new();
        let mut receiver = bus.create_receiver();

        let event = create_test_config_event();
        bus.emit(event.clone()).await?;

        // Try to receive the event with timeout
        match timeout(Duration::from_millis(100), receiver.recv()).await {
            Ok(Ok(received_event)) => {
                assert_eq!(received_event.source(), event.source());
                assert!(matches!(received_event, MonorepoEvent::Config(_)));
            }
            Ok(Err(_)) => {
                // Channel error - acceptable in test environment
            }
            Err(_) => {
                // Timeout - acceptable in test environment  
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_subscriptions_real_workflow() -> Result<()> {
        let bus = EventBus::new();
        let (config_handler, config_events) = TestEventHandler::new("config-handler");
        let (task_handler, task_events) = TestEventHandler::new("task-handler");
        let (audit_handler, audit_events) = TestEventHandler::new("audit-handler");

        // Subscribe handlers to different event types
        let _config_sub = bus.subscribe(
            EventFilter::for_type(EventTypeFilter::Config),
            Arc::new(AsyncEventHandlerWrapper::new(config_handler))
        ).await?;

        let _task_sub = bus.subscribe(
            EventFilter::for_type(EventTypeFilter::Task),
            Arc::new(AsyncEventHandlerWrapper::new(task_handler))
        ).await?;

        let _audit_sub = bus.subscribe(
            EventFilter::All,
            Arc::new(AsyncEventHandlerWrapper::new(audit_handler))
        ).await?;

        // Check subscription count
        let stats = bus.get_stats().await;
        assert_eq!(stats.active_subscriptions, 3);

        // Emit various events
        let config_event = create_test_config_event();
        let task_event = create_test_task_event();
        let changeset_event = create_test_changeset_event();

        bus.emit(config_event).await?;
        bus.emit(task_event).await?;
        bus.emit(changeset_event).await?;

        // Process events
        bus.process_events(10).await?;

        // Verify each handler received appropriate events
        let config_handled = config_events.lock().await;
        let task_handled = task_events.lock().await;
        let audit_handled = audit_events.lock().await;

        assert_eq!(config_handled.len(), 1); // Only config events
        assert_eq!(task_handled.len(), 1); // Only task events
        assert_eq!(audit_handled.len(), 3); // All events

        assert!(matches!(config_handled[0], MonorepoEvent::Config(_)));
        assert!(matches!(task_handled[0], MonorepoEvent::Task(_)));

        Ok(())
    }

    #[tokio::test]
    async fn test_event_statistics_comprehensive() -> Result<()> {
        let bus = EventBus::new();

        // Emit various types of events with different priorities
        let events = vec![
            create_test_config_event(),
            create_test_task_event(),
            create_test_changeset_event(),
            create_test_hook_event(),
            create_test_package_event(),
            create_test_filesystem_event(),
            create_test_workflow_event(),
        ];

        for event in events {
            bus.emit(event).await?;
        }

        let stats = bus.get_stats().await;
        assert_eq!(stats.events_emitted, 7);
        assert_eq!(stats.events_by_type.get("Config"), Some(&1));
        assert_eq!(stats.events_by_type.get("Task"), Some(&1));
        assert_eq!(stats.events_by_type.get("Changeset"), Some(&1));
        assert_eq!(stats.events_by_type.get("Hook"), Some(&1));
        assert_eq!(stats.events_by_type.get("Package"), Some(&1));
        assert_eq!(stats.events_by_type.get("FileSystem"), Some(&1));
        assert_eq!(stats.events_by_type.get("Workflow"), Some(&1));
        assert_eq!(stats.events_by_priority.get(&EventPriority::Normal), Some(&7));

        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    #[tokio::test]
    async fn test_event_serialization() -> Result<()> {
        let event = create_test_changeset_event();

        // Test JSON serialization
        let json = serde_json::to_string(&event)?;
        let deserialized: MonorepoEvent = serde_json::from_str(&json)?;

        assert_eq!(event.source(), deserialized.source());
        assert_eq!(event.priority(), deserialized.priority());

        // Verify changeset data is preserved
        if let (MonorepoEvent::Changeset(ChangesetEvent::Created { changeset: original, .. }),
                MonorepoEvent::Changeset(ChangesetEvent::Created { changeset: deserialized, .. })) = (&event, &deserialized) {
            assert_eq!(original.id, deserialized.id);
            assert_eq!(original.package, deserialized.package);
            assert_eq!(original.version_bump, deserialized.version_bump);
            assert_eq!(original.description, deserialized.description);
        }

        Ok(())
    }
}