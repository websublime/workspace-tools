//! Tests for the hooks management system
//!
//! This module contains unit tests for hook types, definitions, and basic functionality.

#[cfg(test)]
mod tests {
    use crate::hooks::{
        GitOperationType, HookCondition, HookDefinition, HookExecutionContext, HookScript, HookType,
    };
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_hook_execution_context() {
        let temp_dir = TempDir::new().unwrap();

        let context = HookExecutionContext::new(temp_dir.path().to_path_buf(), "main")
            .with_operation_type(GitOperationType::Commit)
            .with_changed_files(vec!["src/lib.rs".to_string(), "Cargo.toml".to_string()])
            .with_affected_packages(vec!["package-a".to_string()])
            .with_env("NODE_ENV".to_string(), "development".to_string());

        assert_eq!(context.current_branch, "main");
        assert!(context.is_commit());
        assert!(!context.is_push());
        assert_eq!(context.changed_files.len(), 2);
        assert_eq!(context.affected_packages.len(), 1);
        assert_eq!(context.get_env("NODE_ENV"), Some(&"development".to_string()));
    }

    #[tokio::test]
    async fn test_hook_type_utilities() {
        assert_eq!(HookType::PreCommit.git_hook_filename(), "pre-commit");
        assert_eq!(HookType::PrePush.git_hook_filename(), "pre-push");
        assert_eq!(HookType::PostCommit.git_hook_filename(), "post-commit");

        assert!(HookType::PreCommit.is_blocking());
        assert!(HookType::PrePush.is_blocking());
        assert!(!HookType::PostCommit.is_blocking());

        let all_hooks = HookType::all();
        assert_eq!(all_hooks.len(), 5);
        assert!(all_hooks.contains(&HookType::PreCommit));
        assert!(all_hooks.contains(&HookType::PrePush));
    }

    #[tokio::test]
    async fn test_hook_definition_builder() {
        let definition = HookDefinition::new(
            HookScript::command("npm".to_string(), vec!["test".to_string()]),
            "Run tests",
        )
        .with_condition(HookCondition::packages_changed(vec!["frontend".to_string()]))
        .with_fail_on_error(false)
        .with_timeout(std::time::Duration::from_secs(300))
        .with_env("CI".to_string(), "true".to_string())
        .with_enabled(true);

        assert_eq!(definition.description, "Run tests");
        assert!(!definition.fail_on_error);
        assert_eq!(definition.timeout, Some(std::time::Duration::from_secs(300)));
        assert_eq!(definition.environment.get("CI"), Some(&"true".to_string()));
        assert!(definition.enabled);
        assert_eq!(definition.conditions.len(), 1);
    }

    #[tokio::test]
    #[allow(clippy::panic)]
    async fn test_hook_script_types() {
        // Test task execution script
        let task_script = HookScript::tasks(vec!["lint".to_string(), "test".to_string()]);
        if let HookScript::TaskExecution { tasks, parallel } = task_script {
            assert_eq!(tasks.len(), 2);
            assert!(!parallel);
        } else {
            panic!("Expected TaskExecution script");
        }

        // Test parallel task execution
        let parallel_script = HookScript::parallel_tasks(vec!["build".to_string()]);
        if let HookScript::TaskExecution { tasks: _, parallel } = parallel_script {
            assert!(parallel);
        } else {
            panic!("Expected TaskExecution script");
        }

        // Test command script
        let command_script = HookScript::command("echo", vec!["hello".to_string()]);
        if let HookScript::Command { cmd, args } = command_script {
            assert_eq!(cmd, "echo");
            assert_eq!(args.len(), 1);
        } else {
            panic!("Expected Command script");
        }
    }

    #[tokio::test]
    #[allow(clippy::panic)]
    async fn test_hook_conditions() {
        // Test files changed condition
        let files_condition = HookCondition::files_changed(vec!["*.rs".to_string()]);
        if let HookCondition::FilesChanged { patterns, match_any } = files_condition {
            assert_eq!(patterns.len(), 1);
            assert!(match_any);
        } else {
            panic!("Expected FilesChanged condition");
        }

        // Test branch condition
        let branch_condition = HookCondition::on_branch("main");
        if let HookCondition::OnBranch { pattern } = branch_condition {
            assert_eq!(pattern, "main");
        } else {
            panic!("Expected OnBranch condition");
        }

        // Test environment variable condition
        let env_condition = HookCondition::env_var("NODE_ENV", Some("production".to_string()));
        if let HookCondition::EnvironmentVariable { name, value } = env_condition {
            assert_eq!(name, "NODE_ENV");
            assert_eq!(value, Some("production".to_string()));
        } else {
            panic!("Expected EnvironmentVariable condition");
        }
    }
}
