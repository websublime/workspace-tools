//! Tests for the audit manager module.
//!
//! **What**: Comprehensive test suite for the `AuditManager` initialization,
//! configuration handling, and accessor methods.
//!
//! **How**: Uses mock filesystems and test fixtures to verify that the audit
//! manager correctly initializes all subsystems and handles various configurations.
//!
//! **Why**: To ensure the audit manager foundation is robust and correctly
//! initializes all dependencies under various conditions.

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::bool_assert_comparison)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::audit::AuditManager;
    use crate::config::PackageToolsConfig;
    use std::path::PathBuf;
    use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
    use sublime_standard_tools::monorepo::MonorepoDetectorTrait;
    use tempfile::TempDir;

    /// Helper to create a temporary test workspace with a git repository.
    ///
    /// This creates:
    /// - A temporary directory
    /// - A git repository initialized in it
    /// - A package.json file
    ///
    /// Returns the temp directory (must be kept alive) and its path.
    async fn setup_test_workspace() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let workspace_path = temp_dir.path().to_path_buf();

        // Initialize git repository
        std::process::Command::new("git")
            .arg("init")
            .current_dir(&workspace_path)
            .output()
            .expect("Failed to initialize git repository");

        // Configure git user for commits
        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&workspace_path)
            .output()
            .expect("Failed to configure git user email");

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&workspace_path)
            .output()
            .expect("Failed to configure git user name");

        // Create a basic package.json
        let package_json = r#"{
  "name": "test-package",
  "version": "1.0.0",
  "description": "Test package",
  "dependencies": {}
}"#;

        let fs = FileSystemManager::new();
        fs.write_file(&workspace_path.join("package.json"), package_json.as_bytes())
            .await
            .expect("Failed to write package.json");

        // Create initial commit
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&workspace_path)
            .output()
            .expect("Failed to git add");

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&workspace_path)
            .output()
            .expect("Failed to git commit");

        (temp_dir, workspace_path)
    }

    #[tokio::test]
    async fn test_audit_manager_new_with_default_config() {
        let (_temp_dir, workspace_path) = setup_test_workspace().await;
        let config = PackageToolsConfig::default();

        let result = AuditManager::new(workspace_path.clone(), config).await;
        assert!(result.is_ok(), "AuditManager::new should succeed with default config");

        let manager = result.expect("Manager should be initialized");
        assert_eq!(manager.workspace_root(), &workspace_path, "Workspace root should match");
    }

    #[tokio::test]
    async fn test_audit_manager_new_with_custom_config() {
        let (_temp_dir, workspace_path) = setup_test_workspace().await;

        let mut config = PackageToolsConfig::default();
        config.audit.enabled = false;
        config.audit.min_severity = "critical".to_string();
        config.audit.sections.upgrades = false;

        let result = AuditManager::new(workspace_path.clone(), config.clone()).await;
        assert!(result.is_ok(), "AuditManager::new should succeed with custom config");

        let manager = result.expect("Manager should be initialized");
        assert_eq!(
            manager.config().audit.enabled,
            false,
            "Audit enabled setting should be preserved"
        );
        assert_eq!(
            manager.config().audit.min_severity,
            "critical",
            "Min severity should be preserved"
        );
        assert_eq!(
            manager.config().audit.sections.upgrades,
            false,
            "Upgrades section setting should be preserved"
        );
    }

    #[tokio::test]
    async fn test_audit_manager_accessors() {
        let (_temp_dir, workspace_path) = setup_test_workspace().await;
        let config = PackageToolsConfig::default();

        let manager = AuditManager::new(workspace_path.clone(), config.clone())
            .await
            .expect("Manager should be initialized");

        // Test workspace_root accessor
        assert_eq!(
            manager.workspace_root(),
            &workspace_path,
            "workspace_root() should return correct path"
        );

        // Test config accessor
        let retrieved_config = manager.config();
        assert_eq!(
            retrieved_config.audit.enabled, config.audit.enabled,
            "config() should return correct configuration"
        );

        // Test upgrade_manager accessor
        let _upgrade_mgr = manager.upgrade_manager();
        // Just verify it doesn't panic

        // Test changes_analyzer accessor
        let _analyzer = manager.changes_analyzer();
        // Just verify it doesn't panic

        // Test monorepo_detector accessor
        let _detector = manager.monorepo_detector();
        // Just verify it doesn't panic

        // Test filesystem accessor
        let _fs = manager.filesystem();
        // Just verify it doesn't panic
    }

    #[tokio::test]
    async fn test_audit_manager_with_nonexistent_workspace() {
        let workspace_path = PathBuf::from("/nonexistent/path/that/does/not/exist");
        let config = PackageToolsConfig::default();

        let result = AuditManager::new(workspace_path, config).await;
        assert!(result.is_err(), "AuditManager::new should fail with nonexistent workspace");

        if let Err(e) = result {
            let error_string = e.to_string();
            assert!(
                error_string.contains("Invalid workspace root")
                    || error_string.contains("does not exist"),
                "Error should indicate invalid workspace root, got: {}",
                error_string
            );
        }
    }

    #[tokio::test]
    async fn test_audit_manager_with_non_git_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let workspace_path = temp_dir.path().to_path_buf();

        // Create package.json but no git repository
        let package_json = r#"{
  "name": "test-package",
  "version": "1.0.0"
}"#;

        let fs = FileSystemManager::new();
        fs.write_file(&workspace_path.join("package.json"), package_json.as_bytes())
            .await
            .expect("Failed to write package.json");

        let config = PackageToolsConfig::default();

        let result = AuditManager::new(workspace_path, config).await;
        assert!(result.is_err(), "AuditManager::new should fail without git repository");

        if let Err(e) = result {
            let error_string = e.to_string();
            assert!(
                error_string.contains("Git") || error_string.contains("repository"),
                "Error should indicate git repository issue, got: {}",
                error_string
            );
        }
    }

    #[tokio::test]
    async fn test_audit_manager_with_all_sections_disabled() {
        let (_temp_dir, workspace_path) = setup_test_workspace().await;

        let mut config = PackageToolsConfig::default();
        config.audit.sections.upgrades = false;
        config.audit.sections.dependencies = false;
        config.audit.sections.breaking_changes = false;
        config.audit.sections.categorization = false;
        config.audit.sections.version_consistency = false;

        let result = AuditManager::new(workspace_path, config.clone()).await;
        assert!(result.is_ok(), "AuditManager::new should succeed even with all sections disabled");

        let manager = result.expect("Manager should be initialized");
        assert_eq!(
            manager.config().audit.sections.upgrades,
            false,
            "All sections should be disabled"
        );
    }

    #[tokio::test]
    async fn test_audit_manager_with_various_severity_levels() {
        let (_temp_dir, workspace_path) = setup_test_workspace().await;

        for severity in &["critical", "warning", "info"] {
            let mut config = PackageToolsConfig::default();
            config.audit.min_severity = severity.to_string();

            let result = AuditManager::new(workspace_path.clone(), config.clone()).await;
            assert!(
                result.is_ok(),
                "AuditManager::new should succeed with severity level '{}'",
                severity
            );

            let manager = result.expect("Manager should be initialized");
            assert_eq!(
                manager.config().audit.min_severity,
                *severity,
                "Severity level should be preserved"
            );
        }
    }

    #[tokio::test]
    async fn test_audit_manager_with_workspace_structure() {
        let (_temp_dir, workspace_path) = setup_test_workspace().await;

        // Create a workspace structure (npm workspaces)
        // Note: This test verifies that AuditManager initializes correctly
        // with a workspace structure, not that it detects it as a monorepo
        // (monorepo detection is tested in sublime_standard_tools)
        let package_json_with_workspaces = r#"{
  "name": "test-monorepo",
  "version": "1.0.0",
  "private": true,
  "workspaces": [
    "packages/*"
  ]
}"#;

        let fs = FileSystemManager::new();
        fs.write_file(
            &workspace_path.join("package.json"),
            package_json_with_workspaces.as_bytes(),
        )
        .await
        .expect("Failed to write package.json");

        // Create a package in the workspace
        let packages_dir = workspace_path.join("packages");
        fs.create_dir_all(&packages_dir).await.expect("Failed to create packages directory");

        let pkg_a_dir = packages_dir.join("pkg-a");
        fs.create_dir_all(&pkg_a_dir).await.expect("Failed to create pkg-a directory");

        let pkg_a_json = r#"{
  "name": "@test/pkg-a",
  "version": "1.0.0"
}"#;

        fs.write_file(&pkg_a_dir.join("package.json"), pkg_a_json.as_bytes())
            .await
            .expect("Failed to write pkg-a package.json");

        // Commit the changes to git
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&workspace_path)
            .output()
            .expect("Failed to git add");

        std::process::Command::new("git")
            .args(["commit", "-m", "Add workspace structure"])
            .current_dir(&workspace_path)
            .output()
            .expect("Failed to git commit");

        let config = PackageToolsConfig::default();

        // Verify that AuditManager initializes successfully with workspace structure
        let result = AuditManager::new(workspace_path.clone(), config).await;
        assert!(result.is_ok(), "AuditManager::new should succeed with workspace structure");

        let manager = result.expect("Manager should be initialized");

        // Verify monorepo detector is accessible (actual detection tested elsewhere)
        let _detector = manager.monorepo_detector();
        assert_eq!(manager.workspace_root(), &workspace_path, "Workspace root should be preserved");
    }

    #[tokio::test]
    async fn test_audit_manager_single_package_detection() {
        let (_temp_dir, workspace_path) = setup_test_workspace().await;
        let config = PackageToolsConfig::default();

        let manager = AuditManager::new(workspace_path.clone(), config)
            .await
            .expect("Manager should be initialized");

        // Verify it's detected as single package (no workspaces)
        let monorepo_kind = manager
            .monorepo_detector()
            .is_monorepo_root(&workspace_path)
            .await
            .expect("Should detect project structure");

        assert!(monorepo_kind.is_none(), "Should detect workspace as single package");
    }

    #[tokio::test]
    async fn test_audit_manager_preserves_upgrade_config() {
        let (_temp_dir, workspace_path) = setup_test_workspace().await;

        let mut config = PackageToolsConfig::default();
        config.upgrade.auto_changeset = false;
        config.upgrade.backup.enabled = false;

        let manager = AuditManager::new(workspace_path, config.clone())
            .await
            .expect("Manager should be initialized");

        assert_eq!(
            manager.config().upgrade.auto_changeset,
            config.upgrade.auto_changeset,
            "Upgrade config should be preserved"
        );
        assert_eq!(
            manager.config().upgrade.backup.enabled,
            config.upgrade.backup.enabled,
            "Backup config should be preserved"
        );
    }
}
