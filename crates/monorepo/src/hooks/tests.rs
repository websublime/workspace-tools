//! Comprehensive tests for the hooks module
//!
//! This module provides complete test coverage for all hook functionality,
//! including hook definitions, installation, validation, execution, and integration
//! with other monorepo systems.

#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use crate::changesets::{Changeset, ChangesetStatus};
    use crate::config::VersionBumpType;
    use crate::core::MonorepoProject;
    use crate::error::Result;
    use crate::hooks::*;
    use crate::tasks::TaskManager;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::TempDir;

    /// Helper function to create a test monorepo project
    fn create_test_project() -> Result<(TempDir, MonorepoProject)> {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path();

        // Initialize a git repository
        std::process::Command::new("git").args(["init"]).current_dir(root_path).output()?;

        // Create initial commit to have a valid repository
        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(root_path)
            .output()?;
        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(root_path)
            .output()?;

        // Create necessary directories
        let packages_dir = root_path.join("packages");
        std::fs::create_dir_all(&packages_dir)?;

        // Create a simple package
        let package_path = packages_dir.join("test-package");
        std::fs::create_dir_all(&package_path)?;

        // Create package.json
        let package_json = serde_json::json!({
            "name": "@test/package",
            "version": "1.0.0",
            "description": "Test package for hook tests"
        });
        std::fs::write(
            package_path.join("package.json"),
            serde_json::to_string_pretty(&package_json)?,
        )?;

        // Create root package.json to indicate package manager
        let root_package_json = serde_json::json!({
            "name": "test-monorepo",
            "version": "1.0.0",
            "private": true,
            "workspaces": ["packages/*"]
        });
        std::fs::write(
            root_path.join("package.json"),
            serde_json::to_string_pretty(&root_package_json)?,
        )?;

        // Create package-lock.json to indicate npm as package manager
        let package_lock = serde_json::json!({
            "name": "test-monorepo",
            "version": "1.0.0",
            "lockfileVersion": 3,
            "requires": true,
            "packages": {
                "": {
                    "name": "test-monorepo",
                    "version": "1.0.0",
                    "workspaces": ["packages/*"]
                }
            }
        });
        std::fs::write(
            root_path.join("package-lock.json"),
            serde_json::to_string_pretty(&package_lock)?,
        )?;

        // Create an initial commit with all the files
        std::process::Command::new("git").args(["add", "."]).current_dir(root_path).output()?;
        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(root_path)
            .output()?;

        // Create monorepo config and project
        let project = MonorepoProject::new(root_path)?;

        Ok((temp_dir, project))
    }

    /// Helper function to create a test hook definition
    fn create_test_hook_definition() -> HookDefinition {
        HookDefinition::new(
            HookScript::parallel_tasks(vec!["test".to_string(), "lint".to_string()]),
            "Test hook for validation",
        )
        .with_condition(HookCondition::files_changed(vec!["*.rs".to_string()]))
        .with_condition(HookCondition::on_branch("feature/*"))
        .with_env("NODE_ENV", "test")
        .with_timeout(Duration::from_secs(300))
    }

    /// Helper function to create a test execution context
    fn create_test_execution_context(
        operation: GitOperationType,
        affected_packages: Vec<String>,
    ) -> HookExecutionContext {
        HookExecutionContext {
            repository_root: PathBuf::from("/test/repo"),
            current_branch: "feature/test-branch".to_string(),
            previous_branch: None,
            current_commit: Some("abc123".to_string()),
            previous_commit: Some("def456".to_string()),
            changed_files: vec!["packages/test-package/src/index.ts".to_string()],
            affected_packages,
            environment: HashMap::new(),
            operation_type: operation,
            remote_info: None,
            commits: vec![CommitInfo {
                hash: "abc123".to_string(),
                message: "test: Add unit tests".to_string(),
                author_email: "test@example.com".to_string(),
                author_name: "Test Author".to_string(),
                changed_files: vec!["packages/test-package/src/index.ts".to_string()],
                timestamp: chrono::Utc::now(),
            }],
            is_merge: false,
            working_directory: PathBuf::from("/test/repo"),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_hook_type_operations() {
        // Test all hook types
        assert_eq!(HookType::PreCommit.git_hook_filename(), "pre-commit");
        assert_eq!(HookType::PrePush.git_hook_filename(), "pre-push");
        assert_eq!(HookType::PostCommit.git_hook_filename(), "post-commit");
        assert_eq!(HookType::PostMerge.git_hook_filename(), "post-merge");
        assert_eq!(HookType::PostCheckout.git_hook_filename(), "post-checkout");

        // Test Display implementation
        assert_eq!(HookType::PreCommit.to_string(), "pre-commit");
        assert_eq!(HookType::PrePush.to_string(), "pre-push");

        // Test all() method
        let all_types = HookType::all();
        assert_eq!(all_types.len(), 5);
        assert!(all_types.contains(&HookType::PreCommit));
        assert!(all_types.contains(&HookType::PrePush));
        assert!(all_types.contains(&HookType::PostCommit));
        assert!(all_types.contains(&HookType::PostMerge));
        assert!(all_types.contains(&HookType::PostCheckout));

        // Test equality and hashing
        let mut hook_map = HashMap::new();
        hook_map.insert(HookType::PreCommit, "pre-commit-value");
        assert_eq!(hook_map.get(&HookType::PreCommit), Some(&"pre-commit-value"));

        // Test blocking behavior
        assert!(HookType::PreCommit.is_blocking());
        assert!(HookType::PrePush.is_blocking());
        assert!(!HookType::PostCommit.is_blocking());
        assert!(!HookType::PostMerge.is_blocking());
        assert!(!HookType::PostCheckout.is_blocking());
    }

    #[test]
    fn test_hook_definition_creation() {
        let definition = create_test_hook_definition();

        assert!(definition.enabled);
        assert!(definition.fail_on_error);
        assert_eq!(definition.description, "Test hook for validation");
        assert_eq!(definition.timeout, Some(Duration::from_secs(300)));
        assert_eq!(definition.environment.get("NODE_ENV"), Some(&"test".to_string()));

        // Test script variants
        match &definition.script {
            HookScript::TaskExecution { tasks, parallel } => {
                assert_eq!(tasks.len(), 2);
                assert!(tasks.contains(&"test".to_string()));
                assert!(tasks.contains(&"lint".to_string()));
                assert!(parallel);
            }
            _ => panic!("Expected TaskExecution script"),
        }

        // Test conditions
        assert_eq!(definition.conditions.len(), 2);
        assert!(matches!(definition.conditions[0], HookCondition::FilesChanged { .. }));
        assert!(matches!(definition.conditions[1], HookCondition::OnBranch { .. }));
    }

    #[test]
    fn test_hook_script_variants() {
        // Test command script
        let command_script =
            HookScript::command("npm", vec!["test".to_string(), "--coverage".to_string()]);

        match command_script {
            HookScript::Command { cmd, args } => {
                assert_eq!(cmd, "npm");
                assert_eq!(args, vec!["test", "--coverage"]);
            }
            _ => panic!("Expected Command script"),
        }

        // Test script file
        let script_file = HookScript::script_file(PathBuf::from("./scripts/pre-commit.sh"));

        match script_file {
            HookScript::ScriptFile { path, args } => {
                assert_eq!(path, PathBuf::from("./scripts/pre-commit.sh"));
                assert!(args.is_empty());
            }
            _ => panic!("Expected ScriptFile script"),
        }

        // Test script file with args
        let script_with_args = HookScript::script_file_with_args(
            PathBuf::from("./scripts/pre-commit.sh"),
            vec!["--strict".to_string()],
        );

        match script_with_args {
            HookScript::ScriptFile { path, args } => {
                assert_eq!(path, PathBuf::from("./scripts/pre-commit.sh"));
                assert_eq!(args, vec!["--strict"]);
            }
            _ => panic!("Expected ScriptFile script"),
        }

        // Test sequence script
        let sequence_script = HookScript::sequence(vec![
            HookScript::tasks(vec!["lint".to_string()]),
            HookScript::tasks(vec!["test".to_string()]),
        ]);

        match sequence_script {
            HookScript::Sequence { scripts, stop_on_failure } => {
                assert_eq!(scripts.len(), 2);
                assert!(stop_on_failure);
            }
            _ => panic!("Expected Sequence script"),
        }
    }

    #[test]
    fn test_hook_conditions() {
        // Test all condition variants
        let conditions = vec![
            HookCondition::files_changed(vec!["*.rs".to_string()]),
            HookCondition::packages_changed(vec!["@test/package".to_string()]),
            HookCondition::dependencies_changed(),
            HookCondition::on_branch("main"),
            HookCondition::environment(crate::Environment::Development),
            HookCondition::changeset_exists(),
            HookCondition::env_var("CI", Some("true".to_string())),
            HookCondition::git_ref_exists("refs/heads/main"),
        ];

        // Verify all conditions are created correctly
        assert_eq!(conditions.len(), 8);
        assert!(matches!(conditions[0], HookCondition::FilesChanged { .. }));
        assert!(matches!(conditions[1], HookCondition::PackagesChanged { .. }));
        assert!(matches!(conditions[2], HookCondition::DependenciesChanged { .. }));
        assert!(matches!(conditions[3], HookCondition::OnBranch { .. }));
        assert!(matches!(conditions[4], HookCondition::Environment { .. }));
        assert!(matches!(conditions[5], HookCondition::ChangesetExists { .. }));
        assert!(matches!(conditions[6], HookCondition::EnvironmentVariable { .. }));
        assert!(matches!(conditions[7], HookCondition::GitRefExists { .. }));

        // Test specific condition properties
        if let HookCondition::FilesChanged { patterns, match_any } = &conditions[0] {
            assert_eq!(patterns.len(), 1);
            assert_eq!(patterns[0], "*.rs");
            assert!(match_any);
        }

        if let HookCondition::OnBranch { pattern } = &conditions[3] {
            assert_eq!(pattern, "main");
        }
    }

    #[test]
    fn test_hook_execution_context() {
        let context = create_test_execution_context(
            GitOperationType::Commit,
            vec!["@test/package".to_string()],
        );

        assert_eq!(context.operation_type, GitOperationType::Commit);
        assert_eq!(context.current_branch, "feature/test-branch");
        assert!(context.previous_branch.is_none());
        assert_eq!(context.affected_packages.len(), 1);
        assert_eq!(context.changed_files.len(), 1);
        assert_eq!(context.commits.len(), 1);
        assert!(context.remote_info.is_none());
        assert!(!context.is_merge);

        // Test commit info
        let commit_info = &context.commits[0];
        assert_eq!(commit_info.hash, "abc123");
        assert_eq!(commit_info.message, "test: Add unit tests");
        assert_eq!(commit_info.author_name, "Test Author");
        assert_eq!(commit_info.author_email, "test@example.com");
    }

    #[test]
    fn test_hook_installer_creation() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let installer = HookInstaller::new(&project)?;

        // Verify installer is created with correct hooks directory
        assert!(installer.hooks_dir.ends_with(".git/hooks"));
        assert!(!installer.hook_template.is_empty());

        Ok(())
    }

    #[test]
    fn test_hook_installation_and_removal() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let installer = HookInstaller::new(&project)?;

        // Test installing a hook
        let definition = create_test_hook_definition();
        installer.install_hook(&HookType::PreCommit, &definition)?;

        // Verify hook file exists
        let hook_path = installer.hooks_dir.join("pre-commit");
        assert!(hook_path.exists());

        // Verify hook is executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&hook_path)?;
            let permissions = metadata.permissions();
            assert!(permissions.mode() & 0o111 != 0);
        }

        // Test hook status
        let status = installer.get_installation_status()?;
        assert!(status.contains_key(&HookType::PreCommit));
        assert_eq!(status.get(&HookType::PreCommit), Some(&true));

        // Test uninstalling the hook
        let removed = installer.uninstall_hook(&HookType::PreCommit)?;
        assert!(removed);
        assert!(!hook_path.exists());

        // Test uninstalling non-existent hook
        let removed_again = installer.uninstall_hook(&HookType::PreCommit)?;
        assert!(!removed_again);

        Ok(())
    }

    #[test]
    fn test_uninstall_all_hooks() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let installer = HookInstaller::new(&project)?;

        // Install multiple hooks first
        let definition = create_test_hook_definition();
        installer.install_hook(&HookType::PreCommit, &definition)?;
        installer.install_hook(&HookType::PrePush, &definition)?;

        // Verify hooks are installed
        let status = installer.get_installation_status()?;
        assert_eq!(status.get(&HookType::PreCommit), Some(&true));
        assert_eq!(status.get(&HookType::PrePush), Some(&true));
        assert_eq!(status.get(&HookType::PostCommit), Some(&false));

        // Test uninstalling all hooks
        let uninstalled = installer.uninstall_all_hooks()?;
        assert_eq!(uninstalled.len(), 2);
        assert!(uninstalled.contains(&HookType::PreCommit));
        assert!(uninstalled.contains(&HookType::PrePush));

        Ok(())
    }

    #[test]
    fn test_hook_validator_creation() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let validator = HookValidator::new(&project);

        // Validator should be created successfully
        let _ = validator; // Use to avoid unused variable warning

        Ok(())
    }

    #[test]
    fn test_condition_evaluation() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let validator = HookValidator::new(&project);

        // Test files changed condition with matching files
        let context = create_test_execution_context(GitOperationType::Commit, vec![]);
        let files_condition = HookCondition::files_changed(vec!["*.ts".to_string()]);
        let files_met = validator.check_conditions(&[files_condition], &context)?;
        assert!(files_met);

        // Test packages changed condition
        let context_with_packages = create_test_execution_context(
            GitOperationType::Commit,
            vec!["@test/package".to_string()],
        );
        let packages_condition = HookCondition::packages_changed(vec!["@test/package".to_string()]);
        let packages_met =
            validator.check_conditions(&[packages_condition], &context_with_packages)?;
        assert!(packages_met);

        // Test branch condition
        let branch_condition = HookCondition::on_branch("feature/*");
        let branch_met = validator.check_conditions(&[branch_condition], &context)?;
        assert!(branch_met);

        // Test non-matching branch pattern
        let main_branch_condition = HookCondition::on_branch("main");
        let branch_not_met = validator.check_conditions(&[main_branch_condition], &context)?;
        assert!(!branch_not_met);

        Ok(())
    }

    #[test]
    fn test_environment_variable_condition() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let validator = HookValidator::new(&project);

        // Create context with environment variable
        let mut context = create_test_execution_context(GitOperationType::Commit, vec![]);
        context.environment.insert("CI".to_string(), "true".to_string());

        // Test matching environment variable
        let env_condition = HookCondition::env_var("CI", Some("true".to_string()));
        let env_met = validator.check_conditions(&[env_condition], &context)?;
        assert!(env_met);

        // Test non-matching value
        let env_wrong_value = HookCondition::env_var("CI", Some("false".to_string()));
        let env_not_met = validator.check_conditions(&[env_wrong_value], &context)?;
        assert!(!env_not_met);

        // Test just checking existence
        let env_exists = HookCondition::env_var("CI", None);
        let exists_met = validator.check_conditions(&[env_exists], &context)?;
        assert!(exists_met);

        Ok(())
    }

    #[test]
    fn test_changeset_validation() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let validator = HookValidator::new(&project);

        // Test validation with no affected packages
        let result = validator.validate_changeset_exists(&[])?;
        assert!(result.changeset_exists);
        assert!(result.validation_details.all_passed());

        // Test validation with affected packages (no changesets exist in test)
        let affected = vec!["@test/package".to_string()];
        let result = validator.validate_changeset_exists(&affected)?;
        assert!(!result.changeset_exists);
        assert!(!result.validation_details.all_passed());

        // Verify validation details
        let checks = &result.validation_details.checks;
        assert!(checks.contains_key("changeset_exists"));
        let check = &checks["changeset_exists"];
        assert!(!check.passed);
        assert!(check.message.contains("No changeset found"));

        Ok(())
    }

    #[test]
    fn test_hook_manager_creation() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;

        // Create a task manager
        let task_manager = TaskManager::new(&project)?;

        // Create hook manager
        let hook_manager = HookManager::new(&project, &task_manager)?;

        assert!(hook_manager.enabled);
        assert!(hook_manager.custom_hooks.is_empty());
        assert!(!hook_manager.default_hooks.is_empty());

        // Test from_project constructor
        let hook_manager2 = HookManager::from_project(&project, &task_manager)?;
        assert!(hook_manager2.enabled);

        Ok(())
    }

    #[test]
    fn test_hook_execution_results() {
        // Test PreCommitResult
        let pre_commit_result = PreCommitResult::new()
            .with_validation_passed(true)
            .with_affected_packages(vec!["@test/package".to_string()]);

        assert!(pre_commit_result.validation_passed);
        assert_eq!(pre_commit_result.affected_packages.len(), 1);
        assert!(pre_commit_result.changeset.is_none());
        assert!(pre_commit_result.required_actions.is_empty());

        // Test PrePushResult
        let pre_push_result = PrePushResult::new()
            .with_validation_passed(false)
            .with_commit_count(3)
            .with_affected_packages(vec!["@test/package".to_string()])
            .with_required_action("Fix tests");

        assert!(!pre_push_result.validation_passed);
        assert_eq!(pre_push_result.commit_count, 3);
        assert_eq!(pre_push_result.affected_packages.len(), 1);
        assert_eq!(pre_push_result.required_actions.len(), 1);

        // Test PostCommitResult
        let post_commit_result = PostCommitResult::new()
            .with_notification("Build started")
            .with_metadata("build_id", "123");

        assert_eq!(post_commit_result.notifications_sent.len(), 1);
        assert!(post_commit_result.metadata.contains_key("build_id"));
    }

    #[test]
    fn test_hook_execution_result() {
        // Test successful result
        let success_result = HookExecutionResult::new(HookType::PreCommit)
            .with_success()
            .with_stdout("All checks passed")
            .with_exit_code(0);

        assert_eq!(success_result.hook_type, HookType::PreCommit);
        assert_eq!(success_result.status, HookStatus::Success);
        assert!(success_result.is_success());
        assert!(!success_result.is_failure());
        assert_eq!(success_result.stdout, "All checks passed");
        assert_eq!(success_result.exit_code, Some(0));

        // Test failed result
        let error = HookError::new(HookErrorCode::ValidationFailed, "Missing changeset")
            .with_context("package", "@test/package")
            .with_cause("No changeset file found");

        let failed_result = HookExecutionResult::new(HookType::PrePush)
            .with_failure(error)
            .with_stderr("Validation failed")
            .with_exit_code(1);

        assert_eq!(failed_result.status, HookStatus::Failed);
        assert!(failed_result.is_failure());
        assert!(failed_result.error.is_some());
        assert_eq!(failed_result.stderr, "Validation failed");

        let hook_error = failed_result.error.unwrap();
        assert_eq!(hook_error.code, HookErrorCode::ValidationFailed);
        assert_eq!(hook_error.message, "Missing changeset");
        assert!(hook_error.context.contains_key("package"));
        assert_eq!(hook_error.cause, Some("No changeset file found".to_string()));
    }

    #[test]
    fn test_hook_error_codes() {
        // Test all error code variants
        let error_codes = [
            HookErrorCode::ExecutionFailed,
            HookErrorCode::ValidationFailed,
            HookErrorCode::ChangesetMissing,
            HookErrorCode::TaskFailed,
            HookErrorCode::InstallationFailed,
            HookErrorCode::ConfigurationError,
            HookErrorCode::SystemError,
        ];

        assert_eq!(error_codes.len(), 7);

        // Test error creation
        let error = HookError::new(HookErrorCode::ValidationFailed, "Test error")
            .with_context("test", "value");
        assert_eq!(error.code, HookErrorCode::ValidationFailed);
        assert_eq!(error.message, "Test error");
        assert!(error.context.contains_key("test"));
        assert!(error.cause.is_none());
    }

    #[test]
    fn test_validation_check() {
        // Test passed check
        let passed = ValidationCheck::passed("All tests passed");
        assert!(passed.passed);
        assert_eq!(passed.message, "All tests passed");
        assert!(passed.details.is_none());

        // Test failed check
        let failed = ValidationCheck::failed("Missing required files")
            .with_details("package.json not found");
        assert!(!failed.passed);
        assert_eq!(failed.message, "Missing required files");
        assert_eq!(failed.details, Some("package.json not found".to_string()));
    }

    #[test]
    fn test_hook_validation_result() {
        let mut result = HookValidationResult::new();
        assert!(result.all_passed());
        assert!(result.checks.is_empty());
        assert!(result.required_actions.is_empty());

        // Add checks
        result = result.with_check("files", ValidationCheck::passed("Files valid"));
        result = result.with_check("branch", ValidationCheck::failed("Wrong branch"));

        assert!(!result.all_passed()); // One check failed
        assert_eq!(result.checks.len(), 2);

        // Add required action
        result = result.with_required_action("Switch to main branch");
        assert_eq!(result.required_actions.len(), 1);

        // Test failed checks
        let failed = result.failed_checks();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].0, "branch");

        // Test builder pattern
        let chained_result = HookValidationResult::new()
            .with_check("test1", ValidationCheck::passed("OK"))
            .with_check("test2", ValidationCheck::passed("OK"))
            .with_required_action("Run tests");

        assert!(chained_result.all_passed());
        assert_eq!(chained_result.checks.len(), 2);
        assert_eq!(chained_result.required_actions.len(), 1);
    }

    #[test]
    fn test_sync_task_executor() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let task_manager = TaskManager::new(&project)?;
        let executor = SyncTaskExecutor::new(&task_manager)?;

        // Test executing non-existent task
        let result = executor.execute_task_sync("non-existent", &["@test/package".to_string()]);
        assert!(!result);

        // Test with empty packages
        let result = executor.execute_task_sync("test", &[]);
        assert!(result);

        Ok(())
    }

    #[test]
    fn test_dependency_type() {
        // Test all dependency type variants from the definitions enum
        let dep_types = [
            DependencyType::Production,
            DependencyType::Development,
            DependencyType::Peer,
            DependencyType::Optional,
            DependencyType::All,
        ];

        assert_eq!(dep_types.len(), 5);
        assert_eq!(dep_types[0], DependencyType::Production);
        assert_ne!(dep_types[0], dep_types[1]);
    }

    #[test]
    fn test_git_operation_type() {
        // Test all operation type variants
        let operations = [
            GitOperationType::Commit,
            GitOperationType::Push,
            GitOperationType::Merge,
            GitOperationType::Rebase,
            GitOperationType::Checkout,
            GitOperationType::Receive,
            GitOperationType::Update,
            GitOperationType::Unknown,
        ];

        assert_eq!(operations.len(), 8);
        assert_eq!(operations[0], GitOperationType::Commit);
        assert_ne!(operations[0], operations[1]);
    }

    #[test]
    fn test_remote_info() {
        let remote = RemoteInfo {
            name: "origin".to_string(),
            url: "https://github.com/test/repo.git".to_string(),
            target_branch: "main".to_string(),
            source_branch: "feature/test".to_string(),
        };

        assert_eq!(remote.name, "origin");
        assert_eq!(remote.url, "https://github.com/test/repo.git");
        assert_eq!(remote.target_branch, "main");
        assert_eq!(remote.source_branch, "feature/test");
    }

    #[test]
    fn test_hook_status() {
        // Test all status variants
        assert_eq!(HookStatus::Success, HookStatus::Success);
        assert_eq!(HookStatus::Failed, HookStatus::Failed);
        assert_eq!(HookStatus::Skipped, HookStatus::Skipped);
        assert_eq!(HookStatus::Pending, HookStatus::Pending);
        assert_eq!(HookStatus::Running, HookStatus::Running);
        assert_ne!(HookStatus::Success, HookStatus::Failed);
    }

    #[test]
    fn test_hook_definition_disabled() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let installer = HookInstaller::new(&project)?;

        // Create disabled hook definition
        let definition = create_test_hook_definition().with_enabled(false);

        // Try to install disabled hook
        installer.install_hook(&HookType::PreCommit, &definition)?;

        // Verify hook was not installed
        let hook_path = installer.hooks_dir.join("pre-commit");
        assert!(!hook_path.exists());

        Ok(())
    }

    #[test]
    fn test_hook_script_edge_cases() {
        // Test empty task list
        let empty_tasks = HookScript::tasks(vec![]);

        match empty_tasks {
            HookScript::TaskExecution { tasks, .. } => {
                assert!(tasks.is_empty());
            }
            _ => panic!("Expected TaskExecution"),
        }

        // Test command with no args
        let no_args_command = HookScript::command("ls", vec![]);

        match no_args_command {
            HookScript::Command { cmd, args } => {
                assert_eq!(cmd, "ls");
                assert!(args.is_empty());
            }
            _ => panic!("Expected Command"),
        }
    }

    #[test]
    fn test_changeset_validation_result_builder() {
        let changeset = Changeset {
            id: "test-changeset".to_string(),
            package: "@test/package".to_string(),
            version_bump: VersionBumpType::Minor,
            description: "Test changeset".to_string(),
            branch: "feature/test".to_string(),
            development_environments: vec![crate::Environment::Development],
            production_deployment: false,
            created_at: chrono::Utc::now(),
            author: "test@example.com".to_string(),
            status: ChangesetStatus::Pending,
        };

        let result = ChangesetValidationResult::new()
            .with_changeset_exists(true)
            .with_changeset(changeset)
            .with_validation_details(
                HookValidationResult::new().with_check("test", ValidationCheck::passed("OK")),
            );

        assert!(result.changeset_exists);
        assert!(result.changeset.is_some());
        assert!(result.validation_details.all_passed());
    }

    #[test]
    fn test_hook_execution_with_skipping() {
        let skipped_result = HookExecutionResult::new(HookType::PostCommit)
            .with_skipped("No post-commit actions configured")
            .with_metadata("reason", "no_config");

        assert_eq!(skipped_result.status, HookStatus::Skipped);
        assert!(skipped_result.is_skipped());
        assert!(skipped_result.metadata.contains_key("skip_reason"));
        assert!(skipped_result.metadata.contains_key("reason"));
    }

    #[test]
    fn test_complex_condition_combinations() {
        // Test complex condition combinations
        let files_condition =
            HookCondition::files_changed(vec!["*.rs".to_string(), "*.toml".to_string()]);
        let packages_condition =
            HookCondition::packages_changed(vec!["package1".to_string(), "package2".to_string()]);
        let branch_condition = HookCondition::on_branch("feature/*");
        let env_condition = HookCondition::environment(crate::Environment::Development);

        let definition = HookDefinition::new(
            HookScript::parallel_tasks(vec!["lint".to_string(), "test".to_string()]),
            "Complex hook definition",
        )
        .with_condition(files_condition)
        .with_condition(packages_condition)
        .with_condition(branch_condition)
        .with_condition(env_condition)
        .with_timeout(Duration::from_secs(600))
        .with_fail_on_error(false);

        assert_eq!(definition.conditions.len(), 4);
        assert_eq!(definition.timeout, Some(Duration::from_secs(600)));
        assert!(!definition.fail_on_error);
    }

    #[test]
    fn test_hook_default_timeout() {
        let default_timeout = HookDefinition::default_timeout();
        assert_eq!(default_timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_working_directory_resolution() {
        let definition =
            create_test_hook_definition().with_working_directory(PathBuf::from("/custom/path"));

        assert_eq!(definition.working_directory, Some(PathBuf::from("/custom/path")));
    }

    #[test]
    fn test_hook_result_duration_methods() {
        let result = HookExecutionResult::new(HookType::PreCommit).with_success();

        // Test duration methods - duration_ms returns u64 which is always >= 0
        let _duration = result.duration_ms(); // Just verify it compiles and runs
        assert!(result.error_message().is_none());

        let failed_result = HookExecutionResult::new(HookType::PrePush)
            .with_failure(HookError::new(HookErrorCode::TaskFailed, "Task failed"));

        assert!(failed_result.error_message().is_some());
        assert_eq!(failed_result.error_message().unwrap(), "Task failed");
    }

    #[test]
    fn test_context_with_remote_info() {
        let remote_info = RemoteInfo {
            name: "origin".to_string(),
            url: "https://github.com/test/repo.git".to_string(),
            target_branch: "main".to_string(),
            source_branch: "feature/test".to_string(),
        };

        let mut context = create_test_execution_context(GitOperationType::Push, vec![]);
        context.remote_info = Some(remote_info);

        assert!(context.remote_info.is_some());
        let remote = context.remote_info.unwrap();
        assert_eq!(remote.name, "origin");
        assert_eq!(remote.target_branch, "main");
    }
}
