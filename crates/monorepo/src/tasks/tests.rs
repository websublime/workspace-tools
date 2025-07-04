//! Comprehensive tests for the tasks module
//!
//! This module provides complete test coverage for all task functionality,
//! including task definitions, execution, conditions, and real-world scenarios.

#[allow(clippy::panic)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::unnecessary_wraps)]
#[cfg(test)]
mod tests {
    use crate::error::Result;
    use crate::tasks::types::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    /// Helper function to create a test task definition
    fn create_test_task() -> TaskDefinition {
        TaskDefinition {
            name: "test-task".to_string(),
            description: "A test task for validation".to_string(),
            commands: vec![TaskCommand {
                command: TaskCommandCore {
                    program: "echo".to_string(),
                    args: vec!["hello".to_string()],
                    current_dir: None,
                    env: HashMap::new(),
                    timeout: None,
                },
                shell: false,
                expected_exit_codes: vec![0],
            }],
            package_scripts: vec![PackageScript {
                package_name: None,
                script_name: "test".to_string(),
                working_directory: None,
                extra_args: vec![],
                package_manager: None,
            }],
            dependencies: vec!["lint".to_string()],
            conditions: vec![TaskCondition::PackagesChanged { packages: vec!["core".to_string()] }],
            priority: TaskPriority::Normal,
            scope: TaskScope::AffectedPackages,
            continue_on_error: false,
            timeout: Some(TaskTimeout::Fixed(Duration::from_secs(300))),
            environment: TaskEnvironment::default(),
        }
    }

    #[test]
    fn test_task_definition_creation() {
        let task = TaskDefinition::new("test-task", "A test task");

        assert_eq!(task.name, "test-task");
        assert_eq!(task.description, "A test task");
        assert!(task.commands.is_empty());
        assert!(task.package_scripts.is_empty());
        assert!(task.dependencies.is_empty());
        assert!(task.conditions.is_empty());
        assert_eq!(task.priority, TaskPriority::Normal);
        assert_eq!(task.scope, TaskScope::AffectedPackages);
        assert!(!task.continue_on_error);
        assert!(task.timeout.is_none());
    }

    #[test]
    fn test_task_definition_builder() {
        let task = TaskDefinition::new("build", "Build task")
            .with_priority(TaskPriority::High)
            .with_dependency("lint")
            .with_continue_on_error(true);

        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.dependencies, vec!["lint"]);
        assert!(task.continue_on_error);
    }

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);
        assert!(TaskPriority::Custom(150) > TaskPriority::High);
        assert!(TaskPriority::Custom(25) < TaskPriority::Normal);
    }

    #[test]
    fn test_task_priority_from_config() {
        let priority = TaskPriority::from_config_value(0);
        assert_eq!(priority, TaskPriority::Low);

        let priority = TaskPriority::from_config_value(50);
        assert_eq!(priority, TaskPriority::Normal);

        let priority = TaskPriority::from_config_value(100);
        assert_eq!(priority, TaskPriority::High);

        let priority = TaskPriority::from_config_value(200);
        assert_eq!(priority, TaskPriority::Critical);

        let priority = TaskPriority::from_config_value(75);
        assert_eq!(priority, TaskPriority::Custom(75));
    }

    #[test]
    fn test_task_command_creation() {
        let command = TaskCommand {
            command: TaskCommandCore {
                program: "npm".to_string(),
                args: vec!["test".to_string()],
                current_dir: Some(PathBuf::from("packages/core")),
                env: {
                    let mut env = HashMap::new();
                    env.insert("NODE_ENV".to_string(), "test".to_string());
                    env
                },
                timeout: Some(Duration::from_secs(30)),
            },
            shell: true,
            expected_exit_codes: vec![0, 1],
        };

        assert_eq!(command.command.program, "npm");
        assert_eq!(command.command.args, vec!["test"]);
        assert_eq!(command.command.current_dir, Some(PathBuf::from("packages/core")));
        assert_eq!(command.command.env.get("NODE_ENV"), Some(&"test".to_string()));
        assert!(command.shell);
        assert_eq!(command.expected_exit_codes, vec![0, 1]);
    }

    #[test]
    fn test_package_script_creation() {
        let script = PackageScript {
            package_name: Some("core".to_string()),
            script_name: "build".to_string(),
            working_directory: Some(PathBuf::from("packages/core")),
            extra_args: vec!["--watch".to_string()],
            package_manager: Some("pnpm".to_string()),
        };

        assert_eq!(script.package_name, Some("core".to_string()));
        assert_eq!(script.script_name, "build");
        assert_eq!(script.working_directory, Some(PathBuf::from("packages/core")));
        assert_eq!(script.extra_args, vec!["--watch"]);
        assert_eq!(script.package_manager, Some("pnpm".to_string()));
    }

    #[test]
    fn test_task_timeout_types() {
        let fixed = TaskTimeout::Fixed(Duration::from_secs(300));
        let per_package = TaskTimeout::PerPackage(Duration::from_secs(60));
        let dynamic = TaskTimeout::Dynamic {
            base: Duration::from_secs(120),
            per_package: Duration::from_secs(30),
        };

        match fixed {
            TaskTimeout::Fixed(duration) => assert_eq!(duration, Duration::from_secs(300)),
            _ => panic!("Expected Fixed timeout"),
        }

        match per_package {
            TaskTimeout::PerPackage(duration) => assert_eq!(duration, Duration::from_secs(60)),
            _ => panic!("Expected PerPackage timeout"),
        }

        match dynamic {
            TaskTimeout::Dynamic { base, per_package } => {
                assert_eq!(base, Duration::from_secs(120));
                assert_eq!(per_package, Duration::from_secs(30));
            }
            _ => panic!("Expected Dynamic timeout"),
        }
    }

    #[test]
    fn test_task_timeout_calculation() {
        // Test fixed timeout
        let timeout = TaskTimeout::Fixed(Duration::from_secs(300));
        assert_eq!(timeout.calculate_timeout(5), Duration::from_secs(300));

        // Test per-package timeout
        let timeout = TaskTimeout::PerPackage(Duration::from_secs(60));
        assert_eq!(timeout.calculate_timeout(3), Duration::from_secs(180));

        // Test dynamic timeout
        let timeout = TaskTimeout::Dynamic {
            base: Duration::from_secs(120),
            per_package: Duration::from_secs(30),
        };
        assert_eq!(timeout.calculate_timeout(4), Duration::from_secs(240));
    }

    #[test]
    fn test_task_environment() {
        let mut env = TaskEnvironment::default();
        env.variables.insert("NODE_ENV".to_string(), "production".to_string());
        env.variables.insert("DEBUG".to_string(), "true".to_string());
        env.inherit.push("PATH".to_string());
        env.inherit.push("HOME".to_string());
        env.unset.push("TEMP_VAR".to_string());

        assert_eq!(env.variables.get("NODE_ENV"), Some(&"production".to_string()));
        assert_eq!(env.variables.get("DEBUG"), Some(&"true".to_string()));
        assert!(env.inherit.contains(&"PATH".to_string()));
        assert!(env.inherit.contains(&"HOME".to_string()));
        assert!(env.unset.contains(&"TEMP_VAR".to_string()));
    }

    #[test]
    fn test_task_environment_merging() {
        let mut env1 = TaskEnvironment::default();
        env1.variables.insert("NODE_ENV".to_string(), "development".to_string());
        env1.inherit.push("PATH".to_string());

        let mut env2 = TaskEnvironment::default();
        env2.variables.insert("DEBUG".to_string(), "true".to_string());
        env2.variables.insert("NODE_ENV".to_string(), "production".to_string());
        env2.unset.push("TEMP".to_string());

        let merged = env1.merge(&env2);

        // env2 should override env1
        assert_eq!(merged.variables.get("NODE_ENV"), Some(&"production".to_string()));
        assert_eq!(merged.variables.get("DEBUG"), Some(&"true".to_string()));
        assert!(merged.inherit.contains(&"PATH".to_string()));
        assert!(merged.unset.contains(&"TEMP".to_string()));
    }

    #[test]
    fn test_task_status_variants() {
        let pending = TaskStatus::Pending;
        let running = TaskStatus::Running;
        let success = TaskStatus::Success;
        let failed = TaskStatus::Failed { reason: "Command failed".to_string() };
        let skipped = TaskStatus::Skipped { reason: "Condition not met".to_string() };
        let cancelled = TaskStatus::Cancelled;
        let timed_out = TaskStatus::TimedOut { after: Duration::from_secs(300) };

        assert_eq!(pending, TaskStatus::Pending);
        assert_eq!(running, TaskStatus::Running);
        assert_eq!(success, TaskStatus::Success);
        assert_eq!(cancelled, TaskStatus::Cancelled);

        match failed {
            TaskStatus::Failed { reason } => assert_eq!(reason, "Command failed"),
            _ => panic!("Expected Failed status"),
        }

        match skipped {
            TaskStatus::Skipped { reason } => assert_eq!(reason, "Condition not met"),
            _ => panic!("Expected Skipped status"),
        }

        match timed_out {
            TaskStatus::TimedOut { after } => assert_eq!(after, Duration::from_secs(300)),
            _ => panic!("Expected TimedOut status"),
        }
    }

    #[test]
    fn test_task_output_creation() {
        let output = TaskOutput {
            command: "echo hello".to_string(),
            working_dir: PathBuf::from("/tmp"),
            exit_code: Some(0),
            stdout: "hello\n".to_string(),
            stderr: String::new(),
            duration: Duration::from_millis(100),
            environment: {
                let mut env = HashMap::new();
                env.insert("PATH".to_string(), "/usr/bin".to_string());
                env
            },
        };

        assert_eq!(output.command, "echo hello");
        assert_eq!(output.working_dir, PathBuf::from("/tmp"));
        assert_eq!(output.exit_code, Some(0));
        assert_eq!(output.stdout, "hello\n");
        assert!(output.stderr.is_empty());
        assert_eq!(output.duration, Duration::from_millis(100));
        assert_eq!(output.environment.get("PATH"), Some(&"/usr/bin".to_string()));
    }

    #[test]
    fn test_task_error_codes() {
        let error = TaskError {
            code: TaskErrorCode::CommandNotFound,
            message: "Command 'nonexistent' not found".to_string(),
            context: {
                let mut context = HashMap::new();
                context.insert("command".to_string(), "nonexistent".to_string());
                context
            },
            occurred_at: SystemTime::now(),
            package: Some("core".to_string()),
            command: Some("nonexistent --version".to_string()),
        };

        assert_eq!(error.code, TaskErrorCode::CommandNotFound);
        assert_eq!(error.message, "Command 'nonexistent' not found");
        assert_eq!(error.context.get("command"), Some(&"nonexistent".to_string()));
        assert_eq!(error.package, Some("core".to_string()));
        assert_eq!(error.command, Some("nonexistent --version".to_string()));
    }

    #[test]
    fn test_task_execution_stats() {
        let stats = TaskExecutionStats {
            commands_executed: 5,
            commands_succeeded: 4,
            commands_failed: 1,
            packages_processed: 3,
            stdout_bytes: 1024,
            stderr_bytes: 256,
            peak_memory_bytes: Some(1048576),
            cpu_time: Some(Duration::from_secs(2)),
        };

        assert_eq!(stats.commands_executed, 5);
        assert_eq!(stats.commands_succeeded, 4);
        assert_eq!(stats.commands_failed, 1);
        assert_eq!(stats.packages_processed, 3);
        assert_eq!(stats.stdout_bytes, 1024);
        assert_eq!(stats.stderr_bytes, 256);
        assert_eq!(stats.peak_memory_bytes, Some(1048576));
        assert_eq!(stats.cpu_time, Some(Duration::from_secs(2)));
    }

    #[test]
    fn test_task_log_level_ordering() {
        assert!(TaskLogLevel::Error > TaskLogLevel::Warning);
        assert!(TaskLogLevel::Warning > TaskLogLevel::Info);
        assert!(TaskLogLevel::Info > TaskLogLevel::Debug);
    }

    #[test]
    fn test_task_artifact_creation() {
        let artifact = TaskArtifact {
            name: "bundle.js".to_string(),
            path: PathBuf::from("dist/bundle.js"),
            artifact_type: "bundle".to_string(),
            size_bytes: 1024000,
            package: Some("core".to_string()),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("minified".to_string(), "true".to_string());
                metadata
            },
        };

        assert_eq!(artifact.name, "bundle.js");
        assert_eq!(artifact.path, PathBuf::from("dist/bundle.js"));
        assert_eq!(artifact.artifact_type, "bundle");
        assert_eq!(artifact.size_bytes, 1024000);
        assert_eq!(artifact.package, Some("core".to_string()));
        assert_eq!(artifact.metadata.get("minified"), Some(&"true".to_string()));
    }

    #[test]
    fn test_task_scope_variants() {
        let global = TaskScope::Global;
        let package = TaskScope::Package("core".to_string());
        let affected = TaskScope::AffectedPackages;
        let all = TaskScope::AllPackages;

        assert_eq!(global, TaskScope::Global);
        assert_eq!(affected, TaskScope::AffectedPackages);
        assert_eq!(all, TaskScope::AllPackages);

        match package {
            TaskScope::Package(name) => assert_eq!(name, "core"),
            _ => panic!("Expected Package scope"),
        }
    }

    #[test]
    fn test_task_scope_default() {
        let default_scope = TaskScope::default();
        assert_eq!(default_scope, TaskScope::AffectedPackages);
    }

    #[test]
    fn test_task_registry_creation() {
        let registry = TaskRegistry::new();

        assert!(registry.tasks.is_empty());
        assert!(registry.scope_index.is_empty());
        assert!(registry.priority_index.is_empty());
    }

    #[test]
    fn test_task_registry_operations() -> Result<()> {
        let mut registry = TaskRegistry::new();
        let task = create_test_task();

        // Test adding task
        registry.add_task(task.clone())?;
        assert_eq!(registry.tasks.len(), 1);
        assert!(registry.tasks.contains_key("test-task"));

        // Test getting task
        let retrieved = registry.get_task("test-task");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test-task");

        // Test updating task
        let mut updated_task = task.clone();
        updated_task.description = "Updated description".to_string();
        registry.update_task(updated_task)?;

        let retrieved = registry.get_task("test-task").unwrap();
        assert_eq!(retrieved.description, "Updated description");

        // Test removing task
        let removed = registry.remove_task("test-task")?;
        assert!(removed);
        assert!(registry.tasks.is_empty());

        Ok(())
    }

    #[test]
    fn test_task_registry_filtering() -> Result<()> {
        let mut registry = TaskRegistry::new();

        // Add tasks with different scopes and priorities
        let task1 = TaskDefinition::new("task1", "First task").with_priority(TaskPriority::High);
        let task2 = TaskDefinition::new("task2", "Second task").with_priority(TaskPriority::Low);
        let task3 = TaskDefinition::new("task3", "Third task").with_priority(TaskPriority::High);

        registry.add_task(task1)?;
        registry.add_task(task2)?;
        registry.add_task(task3)?;

        // Test filtering by scope
        let affected_tasks = registry.get_tasks_by_scope(&TaskScope::AffectedPackages);
        assert_eq!(affected_tasks.len(), 3);

        // Test filtering by priority
        let high_priority_tasks = registry.get_tasks_by_priority(TaskPriority::High);
        assert_eq!(high_priority_tasks.len(), 2);

        let low_priority_tasks = registry.get_tasks_by_priority(TaskPriority::Low);
        assert_eq!(low_priority_tasks.len(), 1);

        Ok(())
    }

    #[test]
    fn test_task_registry_dependency_resolution() -> Result<()> {
        let mut registry = TaskRegistry::new();

        // Create tasks with dependencies
        let task1 = TaskDefinition::new("build", "Build task");
        let task2 = TaskDefinition::new("test", "Test task").with_dependency("build");
        let task3 = TaskDefinition::new("deploy", "Deploy task").with_dependency("test");

        registry.add_task(task1)?;
        registry.add_task(task2)?;
        registry.add_task(task3)?;

        // Test dependency resolution
        let dependencies = registry.get_dependencies("deploy");
        assert_eq!(dependencies.len(), 1);
        assert!(dependencies.contains(&"test".to_string()));

        let all_dependencies = registry.get_all_dependencies("deploy");
        assert_eq!(all_dependencies.len(), 2);
        assert!(all_dependencies.contains(&"test".to_string()));
        assert!(all_dependencies.contains(&"build".to_string()));

        Ok(())
    }

    #[test]
    fn test_task_validation() -> Result<()> {
        let task = create_test_task();

        // Test valid task
        assert!(task.validate().is_ok());

        // Test invalid task (empty name)
        let mut invalid_task = task.clone();
        invalid_task.name = String::new();
        assert!(invalid_task.validate().is_err());

        // Test invalid task (empty description)
        let mut invalid_task = task.clone();
        invalid_task.description = String::new();
        assert!(invalid_task.validate().is_err());

        // Test invalid task (no commands or scripts)
        let mut invalid_task = task.clone();
        invalid_task.commands.clear();
        invalid_task.package_scripts.clear();
        assert!(invalid_task.validate().is_err());

        Ok(())
    }

    #[test]
    fn test_task_serialization() -> Result<()> {
        let task = create_test_task();

        // Test JSON serialization
        let json = serde_json::to_string_pretty(&task)?;
        let deserialized: TaskDefinition = serde_json::from_str(&json)?;

        assert_eq!(task.name, deserialized.name);
        assert_eq!(task.description, deserialized.description);
        assert_eq!(task.commands.len(), deserialized.commands.len());
        assert_eq!(task.package_scripts.len(), deserialized.package_scripts.len());
        assert_eq!(task.dependencies, deserialized.dependencies);
        assert_eq!(task.priority, deserialized.priority);
        assert_eq!(task.scope, deserialized.scope);

        Ok(())
    }

    #[test]
    fn test_task_execution_result_serialization() -> Result<()> {
        let result = TaskExecutionResult {
            task_name: "test-task".to_string(),
            status: TaskStatus::Success,
            started_at: SystemTime::now(),
            ended_at: SystemTime::now(),
            duration: Duration::from_secs(10),
            outputs: vec![],
            stats: TaskExecutionStats::default(),
            affected_packages: vec!["core".to_string()],
            errors: vec![],
            logs: vec![],
            artifacts: vec![],
        };

        // Test JSON serialization
        let json = serde_json::to_string_pretty(&result)?;
        let deserialized: TaskExecutionResult = serde_json::from_str(&json)?;

        assert_eq!(result.task_name, deserialized.task_name);
        assert_eq!(result.status, deserialized.status);
        assert_eq!(result.duration, deserialized.duration);
        assert_eq!(result.affected_packages, deserialized.affected_packages);

        Ok(())
    }

    #[test]
    fn test_execution_context_creation() {
        let context = ExecutionContext::new()
            .with_affected_packages(vec!["core".to_string(), "utils".to_string()])
            .with_branch("feature/test");

        assert_eq!(context.affected_packages.len(), 2);
        assert!(context.affected_packages.contains(&"core".to_string()));
        assert!(context.affected_packages.contains(&"utils".to_string()));
        assert_eq!(context.current_branch, Some("feature/test".to_string()));
    }

    #[test]
    fn test_execution_context_with_changed_files() {
        let files = vec![
            sublime_git_tools::GitChangedFile {
                path: "src/index.ts".to_string(),
                status: sublime_git_tools::GitFileStatus::Modified,
                staged: false,
                workdir: true,
            },
            sublime_git_tools::GitChangedFile {
                path: "package.json".to_string(),
                status: sublime_git_tools::GitFileStatus::Modified,
                staged: true,
                workdir: false,
            },
        ];

        let context = ExecutionContext::new().with_changed_files(files);

        assert_eq!(context.changed_files.len(), 2);
        assert_eq!(context.changed_files[0].path, "src/index.ts");
        assert_eq!(context.changed_files[1].path, "package.json");
    }

    #[test]
    fn test_task_manager_creation() -> Result<()> {
        // Test basic manager creation without full project setup
        let registry = TaskRegistry::new();
        assert!(registry.tasks.is_empty());
        assert!(registry.scope_index.is_empty());
        assert!(registry.priority_index.is_empty());

        // Test execution context creation
        let context = ExecutionContext::new();
        assert!(context.affected_packages.is_empty());
        assert!(context.changed_files.is_empty());
        assert!(context.current_branch.is_none());

        Ok(())
    }
}
