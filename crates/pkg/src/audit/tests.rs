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
#[allow(clippy::unwrap_used)]
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

    // ==================== Upgrade Audit Tests ====================

    #[tokio::test]
    async fn test_audit_upgrades_section_disabled() {
        let (_temp_dir, workspace_path) = setup_test_workspace().await;

        let mut config = PackageToolsConfig::default();
        config.audit.sections.upgrades = false;

        let manager =
            AuditManager::new(workspace_path, config).await.expect("Manager should be initialized");

        let result = manager.audit_upgrades().await;
        assert!(result.is_err(), "audit_upgrades should fail when section is disabled");

        if let Err(e) = result {
            let error_string = e.to_string();
            assert!(
                error_string.contains("disabled") || error_string.contains("upgrades"),
                "Error should indicate section is disabled, got: {}",
                error_string
            );
        }
    }

    #[tokio::test]
    async fn test_audit_upgrades_with_no_dependencies() {
        let (_temp_dir, workspace_path) = setup_test_workspace().await;
        let config = PackageToolsConfig::default();

        let manager =
            AuditManager::new(workspace_path, config).await.expect("Manager should be initialized");

        let result = manager.audit_upgrades().await;
        assert!(result.is_ok(), "audit_upgrades should succeed even with no dependencies");

        let section = result.expect("Should have upgrade section");
        assert_eq!(section.total_upgrades, 0, "Should have no upgrades");
        assert_eq!(section.major_upgrades, 0, "Should have no major upgrades");
        assert_eq!(section.minor_upgrades, 0, "Should have no minor upgrades");
        assert_eq!(section.patch_upgrades, 0, "Should have no patch upgrades");
        assert!(section.deprecated_packages.is_empty(), "Should have no deprecated packages");
        assert!(section.issues.is_empty(), "Should have no issues");
        assert!(!section.has_upgrades(), "has_upgrades should return false");
        assert!(!section.has_deprecated_packages(), "has_deprecated_packages should return false");
    }

    #[tokio::test]
    async fn test_upgrade_audit_section_empty() {
        use crate::audit::UpgradeAuditSection;

        let section = UpgradeAuditSection::empty();
        assert_eq!(section.total_upgrades, 0);
        assert_eq!(section.major_upgrades, 0);
        assert_eq!(section.minor_upgrades, 0);
        assert_eq!(section.patch_upgrades, 0);
        assert!(section.deprecated_packages.is_empty());
        assert!(section.upgrades_by_package.is_empty());
        assert!(section.issues.is_empty());
        assert!(!section.has_upgrades());
        assert!(!section.has_deprecated_packages());
        assert_eq!(section.critical_issue_count(), 0);
        assert_eq!(section.warning_issue_count(), 0);
        assert_eq!(section.info_issue_count(), 0);
    }

    #[tokio::test]
    async fn test_upgrade_audit_section_accessors() {
        use crate::audit::UpgradeAuditSection;

        let section = UpgradeAuditSection::empty();

        // Test upgrades_for_package with non-existent package
        let upgrades = section.upgrades_for_package("nonexistent");
        assert!(upgrades.is_empty());
    }

    #[tokio::test]
    async fn test_deprecated_package_structure() {
        use crate::audit::DeprecatedPackage;

        let deprecated = DeprecatedPackage {
            name: "old-lib".to_string(),
            current_version: "1.0.0".to_string(),
            deprecation_message: "This package is deprecated, use new-lib instead".to_string(),
            alternative: Some("new-lib".to_string()),
        };

        assert_eq!(deprecated.name, "old-lib");
        assert_eq!(deprecated.current_version, "1.0.0");
        assert!(deprecated.deprecation_message.contains("deprecated"));
        assert_eq!(deprecated.alternative, Some("new-lib".to_string()));
    }

    #[tokio::test]
    async fn test_audit_issue_creation() {
        use crate::audit::{AuditIssue, IssueCategory, IssueSeverity};

        let issue = AuditIssue::new(
            IssueSeverity::Critical,
            IssueCategory::Upgrades,
            "Test issue".to_string(),
            "Test description".to_string(),
        );

        assert_eq!(issue.severity, IssueSeverity::Critical);
        assert_eq!(issue.category, IssueCategory::Upgrades);
        assert_eq!(issue.title, "Test issue");
        assert_eq!(issue.description, "Test description");
        assert!(issue.affected_packages.is_empty());
        assert!(issue.suggestion.is_none());
        assert!(issue.metadata.is_empty());
        assert!(issue.is_critical());
        assert!(!issue.is_warning());
        assert!(!issue.is_info());
    }

    #[tokio::test]
    async fn test_audit_issue_mutations() {
        use crate::audit::{AuditIssue, IssueCategory, IssueSeverity};

        let mut issue = AuditIssue::new(
            IssueSeverity::Warning,
            IssueCategory::Upgrades,
            "Test".to_string(),
            "Desc".to_string(),
        );

        issue.add_affected_package("pkg-a".to_string());
        issue.add_affected_package("pkg-b".to_string());
        assert_eq!(issue.affected_packages.len(), 2);

        issue.set_suggestion("Do something".to_string());
        assert_eq!(issue.suggestion, Some("Do something".to_string()));

        issue.add_metadata("key1".to_string(), "value1".to_string());
        issue.add_metadata("key2".to_string(), "value2".to_string());
        assert_eq!(issue.metadata.len(), 2);
        assert_eq!(issue.metadata.get("key1"), Some(&"value1".to_string()));

        assert!(!issue.is_critical());
        assert!(issue.is_warning());
        assert!(!issue.is_info());
    }

    #[tokio::test]
    async fn test_issue_severity_ordering() {
        use crate::audit::IssueSeverity;

        assert!(IssueSeverity::Critical > IssueSeverity::Warning);
        assert!(IssueSeverity::Warning > IssueSeverity::Info);
        assert!(IssueSeverity::Critical > IssueSeverity::Info);

        assert_eq!(IssueSeverity::Critical.as_str(), "critical");
        assert_eq!(IssueSeverity::Warning.as_str(), "warning");
        assert_eq!(IssueSeverity::Info.as_str(), "info");
    }

    #[tokio::test]
    async fn test_issue_category_display() {
        use crate::audit::IssueCategory;

        assert_eq!(IssueCategory::Upgrades.as_str(), "upgrades");
        assert_eq!(IssueCategory::Dependencies.as_str(), "dependencies");
        assert_eq!(IssueCategory::BreakingChanges.as_str(), "breaking_changes");
        assert_eq!(IssueCategory::VersionConsistency.as_str(), "version_consistency");
        assert_eq!(IssueCategory::Security.as_str(), "security");
        assert_eq!(IssueCategory::Other.as_str(), "other");
    }

    #[tokio::test]
    async fn test_audit_upgrades_with_enabled_config() {
        let (_temp_dir, workspace_path) = setup_test_workspace().await;

        let mut config = PackageToolsConfig::default();
        config.audit.sections.upgrades = true;
        config.audit.upgrades.include_patch = true;
        config.audit.upgrades.include_minor = true;
        config.audit.upgrades.include_major = true;

        let manager =
            AuditManager::new(workspace_path, config).await.expect("Manager should be initialized");

        let result = manager.audit_upgrades().await;
        assert!(result.is_ok(), "audit_upgrades should succeed with enabled config");
    }

    #[tokio::test]
    async fn test_extract_alternative_from_deprecation_message() {
        use crate::audit::sections::upgrades::extract_alternative;

        // Test "use X instead" pattern
        let msg1 = "This package is deprecated, use new-package instead";
        assert_eq!(extract_alternative(msg1), Some("new-package".to_string()));

        // Test "migrate to X" pattern
        let msg2 = "Please migrate to modern-lib for continued support";
        assert_eq!(extract_alternative(msg2), Some("modern-lib".to_string()));

        // Test "replaced by X" pattern
        let msg3 = "This has been replaced by better-package.";
        assert_eq!(extract_alternative(msg3), Some("better-package".to_string()));

        // Test with no alternative
        let msg4 = "This package is no longer maintained";
        assert_eq!(extract_alternative(msg4), None);

        // Test case insensitivity
        let msg5 = "USE replacement-lib instead";
        assert_eq!(extract_alternative(msg5), Some("replacement-lib".to_string()));
    }

    #[tokio::test]
    async fn test_upgrade_audit_section_serialization() {
        use crate::audit::UpgradeAuditSection;

        let section = UpgradeAuditSection::empty();

        // Test that it can be serialized to JSON
        let json_result = serde_json::to_string(&section);
        assert!(json_result.is_ok(), "Should serialize to JSON");

        let json = json_result.expect("JSON serialization succeeded");
        assert!(json.contains("total_upgrades"));
        assert!(json.contains("major_upgrades"));
        assert!(json.contains("deprecated_packages"));
        assert!(json.contains("issues"));

        // Test deserialization
        let deserialized: Result<UpgradeAuditSection, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "Should deserialize from JSON");
    }

    #[tokio::test]
    async fn test_deprecated_package_serialization() {
        use crate::audit::DeprecatedPackage;

        let deprecated = DeprecatedPackage {
            name: "old-lib".to_string(),
            current_version: "1.0.0".to_string(),
            deprecation_message: "Deprecated".to_string(),
            alternative: Some("new-lib".to_string()),
        };

        let json_result = serde_json::to_string(&deprecated);
        assert!(json_result.is_ok(), "Should serialize to JSON");

        let json = json_result.expect("JSON serialization succeeded");
        let deserialized: Result<DeprecatedPackage, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "Should deserialize from JSON");

        let dep = deserialized.expect("Deserialization succeeded");
        assert_eq!(dep.name, "old-lib");
        assert_eq!(dep.alternative, Some("new-lib".to_string()));
    }

    #[tokio::test]
    async fn test_audit_issue_serialization() {
        use crate::audit::{AuditIssue, IssueCategory, IssueSeverity};
        use std::collections::HashMap;

        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());

        let issue = AuditIssue {
            severity: IssueSeverity::Warning,
            category: IssueCategory::Upgrades,
            title: "Test".to_string(),
            description: "Desc".to_string(),
            affected_packages: vec!["pkg-a".to_string()],
            suggestion: Some("Fix it".to_string()),
            metadata,
        };

        let json_result = serde_json::to_string(&issue);
        assert!(json_result.is_ok(), "Should serialize to JSON");

        let json = json_result.expect("JSON serialization succeeded");
        let deserialized: Result<AuditIssue, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "Should deserialize from JSON");

        let deserialized_issue = deserialized.expect("Deserialization succeeded");
        assert_eq!(deserialized_issue.title, "Test");
        assert_eq!(deserialized_issue.severity, IssueSeverity::Warning);
        assert_eq!(deserialized_issue.metadata.get("key"), Some(&"value".to_string()));
    }

    // Dependency Audit Section Tests (Story 10.3)

    #[tokio::test]
    async fn test_audit_dependencies_section_empty() {
        use crate::audit::sections::dependencies::DependencyAuditSection;

        let section = DependencyAuditSection::empty();

        assert_eq!(section.circular_dependencies.len(), 0);
        assert_eq!(section.version_conflicts.len(), 0);
        assert_eq!(section.issues.len(), 0);
        assert!(!section.has_circular_dependencies());
        assert!(!section.has_version_conflicts());
        assert_eq!(section.critical_issue_count(), 0);
        assert_eq!(section.warning_issue_count(), 0);
        assert_eq!(section.info_issue_count(), 0);
    }

    #[tokio::test]
    async fn test_audit_dependencies_no_circular_deps() {
        use crate::audit::sections::dependencies::audit_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use package_json::PackageJson;
        use std::path::PathBuf;

        // Create a simple linear dependency chain: A -> B -> C
        let package_a_json = PackageJson {
            name: "pkg-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some([("pkg-b".to_string(), "^1.0.0".to_string())].into_iter().collect()),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let package_b_json = PackageJson {
            name: "pkg-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some([("pkg-c".to_string(), "^1.0.0".to_string())].into_iter().collect()),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let package_c_json = PackageJson {
            name: "pkg-c".to_string(),
            version: "1.0.0".to_string(),
            dependencies: None,
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a_json, None, PathBuf::from("packages/a")),
            PackageInfo::new(package_b_json, None, PathBuf::from("packages/b")),
            PackageInfo::new(package_c_json, None, PathBuf::from("packages/c")),
        ];

        let config = PackageToolsConfig::default();
        let workspace_root = PathBuf::from(".");

        let result = audit_dependencies(&workspace_root, &packages, &config).await;
        assert!(result.is_ok());

        let section = result.expect("Should succeed");
        assert_eq!(section.circular_dependencies.len(), 0);
        assert!(!section.has_circular_dependencies());
        assert_eq!(section.critical_issue_count(), 0);
    }

    #[tokio::test]
    async fn test_audit_dependencies_with_circular_deps() {
        use crate::audit::sections::dependencies::audit_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use package_json::PackageJson;
        use std::path::PathBuf;

        // Create a circular dependency: A -> B -> A
        let package_a_json = PackageJson {
            name: "pkg-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some([("pkg-b".to_string(), "^1.0.0".to_string())].into_iter().collect()),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let package_b_json = PackageJson {
            name: "pkg-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some([("pkg-a".to_string(), "^1.0.0".to_string())].into_iter().collect()),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a_json, None, PathBuf::from("packages/a")),
            PackageInfo::new(package_b_json, None, PathBuf::from("packages/b")),
        ];

        let config = PackageToolsConfig::default();
        let workspace_root = PathBuf::from(".");

        let result = audit_dependencies(&workspace_root, &packages, &config).await;
        assert!(result.is_ok());

        let section = result.expect("Should succeed");
        assert!(section.has_circular_dependencies());
        assert_eq!(section.circular_dependencies.len(), 1);

        let circular_dep = &section.circular_dependencies[0];
        assert!(circular_dep.involves("pkg-a"));
        assert!(circular_dep.involves("pkg-b"));

        // Should generate critical issue
        assert_eq!(section.critical_issue_count(), 1);
        let critical_issue =
            section.issues.iter().find(|i| i.is_critical()).expect("Should have critical issue");
        assert_eq!(critical_issue.title, "Circular dependency detected");
        assert!(critical_issue.affected_packages.contains(&"pkg-a".to_string()));
        assert!(critical_issue.affected_packages.contains(&"pkg-b".to_string()));
    }

    #[tokio::test]
    async fn test_audit_dependencies_with_complex_circular_deps() {
        use crate::audit::sections::dependencies::audit_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use package_json::PackageJson;
        use std::path::PathBuf;

        // Create a circular dependency: A -> B -> C -> A
        let package_a_json = PackageJson {
            name: "pkg-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some([("pkg-b".to_string(), "^1.0.0".to_string())].into_iter().collect()),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let package_b_json = PackageJson {
            name: "pkg-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some([("pkg-c".to_string(), "^1.0.0".to_string())].into_iter().collect()),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let package_c_json = PackageJson {
            name: "pkg-c".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some([("pkg-a".to_string(), "^1.0.0".to_string())].into_iter().collect()),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a_json, None, PathBuf::from("packages/a")),
            PackageInfo::new(package_b_json, None, PathBuf::from("packages/b")),
            PackageInfo::new(package_c_json, None, PathBuf::from("packages/c")),
        ];

        let config = PackageToolsConfig::default();
        let workspace_root = PathBuf::from(".");

        let result = audit_dependencies(&workspace_root, &packages, &config).await;
        assert!(result.is_ok());

        let section = result.expect("Should succeed");
        assert!(section.has_circular_dependencies());
        assert_eq!(section.circular_dependencies.len(), 1);

        let circular_dep = &section.circular_dependencies[0];
        assert_eq!(circular_dep.len(), 3);
        assert!(circular_dep.involves("pkg-a"));
        assert!(circular_dep.involves("pkg-b"));
        assert!(circular_dep.involves("pkg-c"));
    }

    #[tokio::test]
    async fn test_audit_dependencies_circular_deps_disabled() {
        use crate::audit::sections::dependencies::audit_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use package_json::PackageJson;
        use std::path::PathBuf;

        // Create a circular dependency: A -> B -> A
        let package_a_json = PackageJson {
            name: "pkg-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some([("pkg-b".to_string(), "^1.0.0".to_string())].into_iter().collect()),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let package_b_json = PackageJson {
            name: "pkg-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some([("pkg-a".to_string(), "^1.0.0".to_string())].into_iter().collect()),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a_json, None, PathBuf::from("packages/a")),
            PackageInfo::new(package_b_json, None, PathBuf::from("packages/b")),
        ];

        let mut config = PackageToolsConfig::default();
        config.audit.dependencies.check_circular = false;
        let workspace_root = PathBuf::from(".");

        let result = audit_dependencies(&workspace_root, &packages, &config).await;
        assert!(result.is_ok());

        let section = result.expect("Should succeed");
        // Should not detect circular dependencies when disabled
        assert_eq!(section.circular_dependencies.len(), 0);
        assert_eq!(section.critical_issue_count(), 0);
    }

    #[tokio::test]
    async fn test_audit_dependencies_no_version_conflicts() {
        use crate::audit::sections::dependencies::audit_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use package_json::PackageJson;
        use std::path::PathBuf;

        // Create packages with same external dependency version
        let package_a_json = PackageJson {
            name: "pkg-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(
                [("lodash".to_string(), "^4.17.21".to_string())].into_iter().collect(),
            ),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let package_b_json = PackageJson {
            name: "pkg-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(
                [("lodash".to_string(), "^4.17.21".to_string())].into_iter().collect(),
            ),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a_json, None, PathBuf::from("packages/a")),
            PackageInfo::new(package_b_json, None, PathBuf::from("packages/b")),
        ];

        let config = PackageToolsConfig::default();
        let workspace_root = PathBuf::from(".");

        let result = audit_dependencies(&workspace_root, &packages, &config).await;
        assert!(result.is_ok());

        let section = result.expect("Should succeed");
        assert_eq!(section.version_conflicts.len(), 0);
        assert!(!section.has_version_conflicts());
        assert_eq!(section.warning_issue_count(), 0);
    }

    #[tokio::test]
    async fn test_audit_dependencies_with_version_conflicts() {
        use crate::audit::sections::dependencies::audit_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use package_json::PackageJson;
        use std::path::PathBuf;

        // Create packages with different external dependency versions
        let package_a_json = PackageJson {
            name: "pkg-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(
                [("lodash".to_string(), "^4.17.21".to_string())].into_iter().collect(),
            ),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let package_b_json = PackageJson {
            name: "pkg-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(
                [("lodash".to_string(), "^3.10.1".to_string())].into_iter().collect(),
            ),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a_json, None, PathBuf::from("packages/a")),
            PackageInfo::new(package_b_json, None, PathBuf::from("packages/b")),
        ];

        let config = PackageToolsConfig::default();
        let workspace_root = PathBuf::from(".");

        let result = audit_dependencies(&workspace_root, &packages, &config).await;
        assert!(result.is_ok());

        let section = result.expect("Should succeed");
        assert!(section.has_version_conflicts());
        assert_eq!(section.version_conflicts.len(), 1);

        let conflict = &section.version_conflicts[0];
        assert_eq!(conflict.dependency_name, "lodash");
        assert_eq!(conflict.version_count(), 2);
        assert!(conflict.describe().contains("lodash"));

        // Should generate warning issue
        assert_eq!(section.warning_issue_count(), 1);
        let warning_issue =
            section.issues.iter().find(|i| i.is_warning()).expect("Should have warning issue");
        assert!(warning_issue.title.contains("lodash"));
        assert!(warning_issue.affected_packages.contains(&"pkg-a".to_string()));
        assert!(warning_issue.affected_packages.contains(&"pkg-b".to_string()));
    }

    #[tokio::test]
    async fn test_audit_dependencies_version_conflicts_disabled() {
        use crate::audit::sections::dependencies::audit_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use package_json::PackageJson;
        use std::path::PathBuf;

        // Create packages with different external dependency versions
        let package_a_json = PackageJson {
            name: "pkg-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(
                [("lodash".to_string(), "^4.17.21".to_string())].into_iter().collect(),
            ),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let package_b_json = PackageJson {
            name: "pkg-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(
                [("lodash".to_string(), "^3.10.1".to_string())].into_iter().collect(),
            ),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a_json, None, PathBuf::from("packages/a")),
            PackageInfo::new(package_b_json, None, PathBuf::from("packages/b")),
        ];

        let mut config = PackageToolsConfig::default();
        config.audit.dependencies.check_version_conflicts = false;
        let workspace_root = PathBuf::from(".");

        let result = audit_dependencies(&workspace_root, &packages, &config).await;
        assert!(result.is_ok());

        let section = result.expect("Should succeed");
        // Should not detect version conflicts when disabled
        assert_eq!(section.version_conflicts.len(), 0);
        assert_eq!(section.warning_issue_count(), 0);
    }

    #[tokio::test]
    async fn test_audit_dependencies_ignores_workspace_protocol() {
        use crate::audit::sections::dependencies::audit_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use package_json::PackageJson;
        use std::path::PathBuf;

        // Create packages with workspace protocol dependencies
        let package_a_json = PackageJson {
            name: "pkg-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(
                [
                    ("pkg-b".to_string(), "workspace:*".to_string()),
                    ("lodash".to_string(), "^4.17.21".to_string()),
                ]
                .into_iter()
                .collect(),
            ),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let package_b_json = PackageJson {
            name: "pkg-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(
                [("lodash".to_string(), "^4.17.21".to_string())].into_iter().collect(),
            ),
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a_json, None, PathBuf::from("packages/a")),
            PackageInfo::new(package_b_json, None, PathBuf::from("packages/b")),
        ];

        let config = PackageToolsConfig::default();
        let workspace_root = PathBuf::from(".");

        let result = audit_dependencies(&workspace_root, &packages, &config).await;
        assert!(result.is_ok());

        let section = result.expect("Should succeed");
        // No conflicts because lodash versions are the same
        assert_eq!(section.version_conflicts.len(), 0);
    }

    #[tokio::test]
    async fn test_audit_dependencies_ignores_dev_dependencies() {
        use crate::audit::sections::dependencies::audit_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use package_json::PackageJson;
        use std::path::PathBuf;

        // Create packages with different dev dependency versions
        let package_a_json = PackageJson {
            name: "pkg-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: None,
            dev_dependencies: Some(
                [("jest".to_string(), "^27.0.0".to_string())].into_iter().collect(),
            ),
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let package_b_json = PackageJson {
            name: "pkg-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: None,
            dev_dependencies: Some(
                [("jest".to_string(), "^26.0.0".to_string())].into_iter().collect(),
            ),
            peer_dependencies: None,
            optional_dependencies: None,
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a_json, None, PathBuf::from("packages/a")),
            PackageInfo::new(package_b_json, None, PathBuf::from("packages/b")),
        ];

        let config = PackageToolsConfig::default();
        let workspace_root = PathBuf::from(".");

        let result = audit_dependencies(&workspace_root, &packages, &config).await;
        assert!(result.is_ok());

        let section = result.expect("Should succeed");
        // No conflicts because dev dependencies are ignored
        assert_eq!(section.version_conflicts.len(), 0);
    }

    #[tokio::test]
    async fn test_audit_dependencies_section_disabled() {
        use crate::audit::sections::dependencies::audit_dependencies;
        use crate::config::PackageToolsConfig;
        use std::path::PathBuf;

        let packages = vec![];
        let mut config = PackageToolsConfig::default();
        config.audit.sections.dependencies = false;
        let workspace_root = PathBuf::from(".");

        let result = audit_dependencies(&workspace_root, &packages, &config).await;
        assert!(result.is_ok());

        let section = result.expect("Should succeed");
        assert_eq!(section.circular_dependencies.len(), 0);
        assert_eq!(section.version_conflicts.len(), 0);
        assert_eq!(section.issues.len(), 0);
    }

    #[tokio::test]
    async fn test_version_conflict_accessors() {
        use crate::audit::sections::dependencies::{VersionConflict, VersionUsage};

        let conflict = VersionConflict {
            dependency_name: "react".to_string(),
            versions: vec![
                VersionUsage {
                    package_name: "pkg-a".to_string(),
                    version_spec: "^17.0.0".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-b".to_string(),
                    version_spec: "^16.0.0".to_string(),
                },
            ],
        };

        assert_eq!(conflict.version_count(), 2);
        let description = conflict.describe();
        assert!(description.contains("react"));
        assert!(description.contains("pkg-a"));
        assert!(description.contains("^17.0.0"));
    }

    #[tokio::test]
    async fn test_dependency_audit_section_accessors() {
        use crate::audit::sections::dependencies::{
            DependencyAuditSection, VersionConflict, VersionUsage,
        };
        use crate::types::CircularDependency;

        let mut section = DependencyAuditSection::empty();
        section.circular_dependencies.push(CircularDependency::new(vec![
            "pkg-a".to_string(),
            "pkg-b".to_string(),
            "pkg-a".to_string(),
        ]));

        section.version_conflicts.push(VersionConflict {
            dependency_name: "lodash".to_string(),
            versions: vec![
                VersionUsage {
                    package_name: "pkg-a".to_string(),
                    version_spec: "^4.0.0".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-c".to_string(),
                    version_spec: "^3.0.0".to_string(),
                },
            ],
        });

        assert_eq!(section.circular_dependencies.len(), 1);
        assert_eq!(section.version_conflicts.len(), 1);
        assert!(section.has_circular_dependencies());
        assert!(section.has_version_conflicts());

        let pkg_a_cycles = section.circular_dependencies_for_package("pkg-a");
        assert_eq!(pkg_a_cycles.len(), 1);

        let pkg_c_cycles = section.circular_dependencies_for_package("pkg-c");
        assert_eq!(pkg_c_cycles.len(), 0);

        let lodash_conflict = section.version_conflicts_for_dependency("lodash");
        assert!(lodash_conflict.is_some());

        let react_conflict = section.version_conflicts_for_dependency("react");
        assert!(react_conflict.is_none());
    }

    #[tokio::test]
    async fn test_dependency_audit_section_serialization() {
        use crate::audit::sections::dependencies::{
            DependencyAuditSection, VersionConflict, VersionUsage,
        };

        let mut section = DependencyAuditSection::empty();
        section.version_conflicts.push(VersionConflict {
            dependency_name: "lodash".to_string(),
            versions: vec![VersionUsage {
                package_name: "pkg-a".to_string(),
                version_spec: "^4.0.0".to_string(),
            }],
        });

        let json_result = serde_json::to_string(&section);
        assert!(json_result.is_ok(), "Should serialize to JSON");

        let json = json_result.expect("JSON serialization succeeded");
        let deserialized: Result<DependencyAuditSection, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "Should deserialize from JSON");

        let deserialized_section = deserialized.expect("Deserialization succeeded");
        assert_eq!(deserialized_section.version_conflicts.len(), 1);
        assert_eq!(deserialized_section.version_conflicts[0].dependency_name, "lodash");
    }

    // ============================================================================
    // Dependency Categorization Tests (Story 10.4)
    // ============================================================================

    #[tokio::test]
    async fn test_categorize_dependencies_empty_workspace() {
        use crate::audit::sections::categorize_dependencies;
        use crate::config::PackageToolsConfig;

        let packages = vec![];
        let config = PackageToolsConfig::default();

        let result = categorize_dependencies(&packages, &config).await;
        assert!(result.is_ok(), "Should handle empty workspace");

        let categorization = result.expect("Categorization succeeded");
        assert_eq!(categorization.stats.total_packages, 0);
        assert_eq!(categorization.stats.internal_packages, 0);
        assert_eq!(categorization.stats.external_packages, 0);
        assert_eq!(categorization.stats.workspace_links, 0);
        assert_eq!(categorization.stats.local_links, 0);
    }

    #[tokio::test]
    async fn test_categorize_dependencies_single_package_no_deps() {
        use crate::audit::sections::categorize_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;

        let package_json = package_json::PackageJson {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        };

        let packages =
            vec![PackageInfo::new(package_json, None, PathBuf::from("/workspace/packages/test"))];

        let config = PackageToolsConfig::default();
        let result = categorize_dependencies(&packages, &config).await;
        assert!(result.is_ok(), "Should handle single package with no dependencies");

        let categorization = result.expect("Categorization succeeded");
        assert_eq!(categorization.stats.total_packages, 1);
        assert_eq!(categorization.stats.internal_packages, 0);
        assert_eq!(categorization.stats.external_packages, 0);
    }

    #[tokio::test]
    async fn test_categorize_dependencies_with_external_packages() {
        use crate::audit::sections::categorize_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use std::collections::HashMap;

        let mut dependencies = HashMap::new();
        dependencies.insert("lodash".to_string(), "^4.17.21".to_string());
        dependencies.insert("express".to_string(), "^4.18.0".to_string());

        let package_json = package_json::PackageJson {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(dependencies),
            ..Default::default()
        };

        let packages =
            vec![PackageInfo::new(package_json, None, PathBuf::from("/workspace/packages/test"))];

        let config = PackageToolsConfig::default();
        let result = categorize_dependencies(&packages, &config).await;
        assert!(result.is_ok(), "Should categorize external packages");

        let categorization = result.expect("Categorization succeeded");
        assert_eq!(categorization.stats.total_packages, 1);
        assert_eq!(categorization.stats.external_packages, 2);
        assert_eq!(categorization.external_packages.len(), 2);

        let lodash = categorization.external_packages.iter().find(|p| p.name == "lodash");
        assert!(lodash.is_some(), "Should find lodash");
        assert_eq!(lodash.unwrap().version_spec, "^4.17.21");
        assert_eq!(lodash.unwrap().used_by, vec!["test-package"]);
    }

    #[tokio::test]
    async fn test_categorize_dependencies_with_internal_packages() {
        use crate::audit::sections::categorize_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use std::collections::HashMap;

        // Create two packages where one depends on the other
        let package_a_json = package_json::PackageJson {
            name: "package-a".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        };

        let mut dependencies = HashMap::new();
        dependencies.insert("package-a".to_string(), "^1.0.0".to_string());

        let package_b_json = package_json::PackageJson {
            name: "package-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(dependencies),
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a_json, None, PathBuf::from("/workspace/packages/a")),
            PackageInfo::new(package_b_json, None, PathBuf::from("/workspace/packages/b")),
        ];

        let config = PackageToolsConfig::default();
        let result = categorize_dependencies(&packages, &config).await;
        assert!(result.is_ok(), "Should categorize internal packages");

        let categorization = result.expect("Categorization succeeded");
        assert_eq!(categorization.stats.total_packages, 2);
        assert_eq!(categorization.stats.internal_packages, 1);
        assert_eq!(categorization.internal_packages.len(), 1);

        let internal_pkg = &categorization.internal_packages[0];
        assert_eq!(internal_pkg.name, "package-a");
        assert_eq!(internal_pkg.used_by, vec!["package-b"]);
        assert_eq!(internal_pkg.version, Some("1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_categorize_dependencies_with_workspace_protocol() {
        use crate::audit::sections::categorize_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use std::collections::HashMap;

        let package_a_json = package_json::PackageJson {
            name: "package-a".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        };

        let mut dependencies = HashMap::new();
        dependencies.insert("package-a".to_string(), "workspace:*".to_string());

        let package_b_json = package_json::PackageJson {
            name: "package-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(dependencies),
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a_json, None, PathBuf::from("/workspace/packages/a")),
            PackageInfo::new(package_b_json, None, PathBuf::from("/workspace/packages/b")),
        ];

        let config = PackageToolsConfig::default();
        let result = categorize_dependencies(&packages, &config).await;
        assert!(result.is_ok(), "Should categorize workspace links");

        let categorization = result.expect("Categorization succeeded");
        assert_eq!(categorization.stats.workspace_links, 1);
        assert_eq!(categorization.workspace_links.len(), 1);

        let link = &categorization.workspace_links[0];
        assert_eq!(link.package_name, "package-b");
        assert_eq!(link.dependency_name, "package-a");
        assert_eq!(link.version_spec, "workspace:*");
    }

    #[tokio::test]
    async fn test_categorize_dependencies_with_local_protocols() {
        use crate::audit::sections::categorize_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use std::collections::HashMap;

        let mut dependencies = HashMap::new();
        dependencies.insert("utils-lib".to_string(), "file:../utils".to_string());
        dependencies.insert("core-lib".to_string(), "link:./core".to_string());
        dependencies.insert("shared-lib".to_string(), "portal:../shared".to_string());

        let package_json = package_json::PackageJson {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(dependencies),
            ..Default::default()
        };

        let packages =
            vec![PackageInfo::new(package_json, None, PathBuf::from("/workspace/packages/test"))];

        let config = PackageToolsConfig::default();
        let result = categorize_dependencies(&packages, &config).await;
        assert!(result.is_ok(), "Should categorize local links");

        let categorization = result.expect("Categorization succeeded");
        assert_eq!(categorization.stats.local_links, 3);
        assert_eq!(categorization.local_links.len(), 3);

        // Check file: protocol
        let file_link =
            categorization.local_links.iter().find(|l| l.dependency_name == "utils-lib");
        assert!(file_link.is_some(), "Should find file: link");
        assert_eq!(file_link.unwrap().path, "../utils");

        // Check link: protocol
        let link_link = categorization.local_links.iter().find(|l| l.dependency_name == "core-lib");
        assert!(link_link.is_some(), "Should find link: link");
        assert_eq!(link_link.unwrap().path, "./core");

        // Check portal: protocol
        let portal_link =
            categorization.local_links.iter().find(|l| l.dependency_name == "shared-lib");
        assert!(portal_link.is_some(), "Should find portal: link");
        assert_eq!(portal_link.unwrap().path, "../shared");
    }

    #[tokio::test]
    async fn test_categorize_dependencies_mixed_dependency_types() {
        use crate::audit::sections::categorize_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use std::collections::HashMap;

        // Package A: library package
        let package_a_json = package_json::PackageJson {
            name: "package-a".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        };

        // Package B: depends on A internally, lodash externally, and uses workspace protocol
        let mut dependencies_b = HashMap::new();
        dependencies_b.insert("package-a".to_string(), "workspace:^".to_string());
        dependencies_b.insert("lodash".to_string(), "^4.17.21".to_string());
        dependencies_b.insert("local-utils".to_string(), "file:../utils".to_string());

        let package_b_json = package_json::PackageJson {
            name: "package-b".to_string(),
            version: "2.0.0".to_string(),
            dependencies: Some(dependencies_b),
            ..Default::default()
        };

        // Package C: also depends on A
        let mut dependencies_c = HashMap::new();
        dependencies_c.insert("package-a".to_string(), "^1.0.0".to_string());
        dependencies_c.insert("lodash".to_string(), "^4.17.0".to_string());

        let package_c_json = package_json::PackageJson {
            name: "package-c".to_string(),
            version: "1.5.0".to_string(),
            dependencies: Some(dependencies_c),
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a_json, None, PathBuf::from("/workspace/packages/a")),
            PackageInfo::new(package_b_json, None, PathBuf::from("/workspace/packages/b")),
            PackageInfo::new(package_c_json, None, PathBuf::from("/workspace/packages/c")),
        ];

        let config = PackageToolsConfig::default();
        let result = categorize_dependencies(&packages, &config).await;
        assert!(result.is_ok(), "Should categorize mixed dependencies");

        let categorization = result.expect("Categorization succeeded");
        assert_eq!(categorization.stats.total_packages, 3);
        assert_eq!(categorization.stats.internal_packages, 1); // package-a
        assert_eq!(categorization.stats.external_packages, 1); // lodash (unique)
        assert_eq!(categorization.stats.workspace_links, 1); // package-b -> package-a
        assert_eq!(categorization.stats.local_links, 1); // local-utils

        // Verify internal package
        let internal_pkg = &categorization.internal_packages[0];
        assert_eq!(internal_pkg.name, "package-a");
        assert_eq!(internal_pkg.used_by.len(), 1); // only package-c (workspace link doesn't count)
        assert!(internal_pkg.used_by.contains(&"package-c".to_string()));

        // Verify external package (lodash used by both B and C)
        let lodash = categorization.external_packages.iter().find(|p| p.name == "lodash");
        assert!(lodash.is_some(), "Should find lodash");
        assert_eq!(lodash.unwrap().used_by.len(), 2);
    }

    #[tokio::test]
    async fn test_categorization_percentages() {
        use crate::audit::sections::categorize_dependencies;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use std::collections::HashMap;

        // Create a workspace with 2 internal and 3 external packages
        let mut dependencies = HashMap::new();
        dependencies.insert("internal-1".to_string(), "^1.0.0".to_string());
        dependencies.insert("internal-2".to_string(), "^2.0.0".to_string());
        dependencies.insert("lodash".to_string(), "^4.17.21".to_string());
        dependencies.insert("express".to_string(), "^4.18.0".to_string());
        dependencies.insert("react".to_string(), "^18.0.0".to_string());

        let package_json = package_json::PackageJson {
            name: "main-package".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(dependencies),
            ..Default::default()
        };

        let internal_1_json = package_json::PackageJson {
            name: "internal-1".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        };

        let internal_2_json = package_json::PackageJson {
            name: "internal-2".to_string(),
            version: "2.0.0".to_string(),
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_json, None, PathBuf::from("/workspace/main")),
            PackageInfo::new(
                internal_1_json,
                None,
                PathBuf::from("/workspace/packages/internal-1"),
            ),
            PackageInfo::new(
                internal_2_json,
                None,
                PathBuf::from("/workspace/packages/internal-2"),
            ),
        ];

        let config = PackageToolsConfig::default();
        let result = categorize_dependencies(&packages, &config).await;
        assert!(result.is_ok(), "Should calculate percentages");

        let categorization = result.expect("Categorization succeeded");
        assert_eq!(categorization.stats.internal_packages, 2);
        assert_eq!(categorization.stats.external_packages, 3);

        // 2 internal out of 5 total = 40%
        let internal_pct = categorization.internal_percentage();
        assert!((internal_pct - 40.0).abs() < 0.1, "Internal percentage should be ~40%");

        // 3 external out of 5 total = 60%
        let external_pct = categorization.external_percentage();
        assert!((external_pct - 60.0).abs() < 0.1, "External percentage should be ~60%");
    }

    #[tokio::test]
    async fn test_generate_categorization_issues() {
        use crate::audit::sections::{categorize_dependencies, generate_categorization_issues};
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use std::collections::HashMap;

        // Create a highly-used internal package
        let mut dependencies = HashMap::new();
        dependencies.insert("core-lib".to_string(), "workspace:*".to_string());

        let mut packages = vec![PackageInfo::new(
            package_json::PackageJson {
                name: "core-lib".to_string(),
                version: "1.0.0".to_string(),
                ..Default::default()
            },
            None,
            PathBuf::from("/workspace/packages/core"),
        )];

        // Create 6 packages that depend on core-lib
        for i in 1..=6 {
            let mut deps = HashMap::new();
            deps.insert("core-lib".to_string(), "^1.0.0".to_string());

            packages.push(PackageInfo::new(
                package_json::PackageJson {
                    name: format!("package-{}", i),
                    version: "1.0.0".to_string(),
                    dependencies: Some(deps),
                    ..Default::default()
                },
                None,
                PathBuf::from(format!("/workspace/packages/{}", i)),
            ));
        }

        let config = PackageToolsConfig::default();
        let categorization = categorize_dependencies(&packages, &config).await.unwrap();
        let issues = generate_categorization_issues(&categorization);

        assert!(!issues.is_empty(), "Should generate issues");

        // Should have issue for highly-used package
        let highly_used_issue = issues.iter().find(|i| i.title.contains("Highly-used"));
        assert!(highly_used_issue.is_some(), "Should have highly-used package issue");

        // Should have summary issue
        let summary_issue = issues.iter().find(|i| i.title.contains("summary"));
        assert!(summary_issue.is_some(), "Should have summary issue");
    }

    #[tokio::test]
    async fn test_local_link_type_parsing() {
        use crate::audit::sections::LocalLinkType;

        assert_eq!(LocalLinkType::from_version_spec("file:../utils"), Some(LocalLinkType::File));
        assert_eq!(LocalLinkType::from_version_spec("link:./core"), Some(LocalLinkType::Link));
        assert_eq!(
            LocalLinkType::from_version_spec("portal:../shared"),
            Some(LocalLinkType::Portal)
        );
        assert_eq!(LocalLinkType::from_version_spec("^1.0.0"), None);
        assert_eq!(LocalLinkType::from_version_spec("workspace:*"), None);
    }

    #[tokio::test]
    async fn test_local_link_type_display() {
        use crate::audit::sections::LocalLinkType;

        assert_eq!(LocalLinkType::File.as_str(), "file");
        assert_eq!(LocalLinkType::Link.as_str(), "link");
        assert_eq!(LocalLinkType::Portal.as_str(), "portal");

        assert_eq!(LocalLinkType::File.protocol_prefix(), "file:");
        assert_eq!(LocalLinkType::Link.protocol_prefix(), "link:");
        assert_eq!(LocalLinkType::Portal.protocol_prefix(), "portal:");

        assert_eq!(format!("{}", LocalLinkType::File), "file");
        assert_eq!(format!("{}", LocalLinkType::Link), "link");
        assert_eq!(format!("{}", LocalLinkType::Portal), "portal");
    }

    #[tokio::test]
    async fn test_local_link_type_edge_cases() {
        use crate::audit::sections::LocalLinkType;

        // Test case sensitivity
        assert_eq!(LocalLinkType::from_version_spec("FILE:../utils"), None);
        assert_eq!(LocalLinkType::from_version_spec("Link:./core"), None);
        assert_eq!(LocalLinkType::from_version_spec("PORTAL:../shared"), None);

        // Test partial matches
        assert_eq!(LocalLinkType::from_version_spec("file"), None);
        assert_eq!(LocalLinkType::from_version_spec("link"), None);
        assert_eq!(LocalLinkType::from_version_spec("portal"), None);

        // Test empty and whitespace
        assert_eq!(LocalLinkType::from_version_spec(""), None);
        assert_eq!(LocalLinkType::from_version_spec(" file:../utils"), None);

        // Test mixed protocols
        assert_eq!(
            LocalLinkType::from_version_spec("file:link:../utils"),
            Some(LocalLinkType::File)
        );
    }

    #[tokio::test]
    async fn test_categorization_serialization() {
        use crate::audit::sections::{
            CategorizationStats, DependencyCategorization, ExternalPackage, InternalPackage,
            LocalLink, LocalLinkType, WorkspaceLink,
        };

        let categorization = DependencyCategorization {
            internal_packages: vec![InternalPackage {
                name: "core".to_string(),
                path: PathBuf::from("packages/core"),
                version: Some("1.0.0".to_string()),
                used_by: vec!["app".to_string()],
            }],
            external_packages: vec![ExternalPackage {
                name: "lodash".to_string(),
                version_spec: "^4.17.21".to_string(),
                used_by: vec!["app".to_string()],
                is_deprecated: false,
            }],
            workspace_links: vec![WorkspaceLink {
                package_name: "app".to_string(),
                dependency_name: "utils".to_string(),
                version_spec: "workspace:*".to_string(),
            }],
            local_links: vec![LocalLink {
                package_name: "app".to_string(),
                dependency_name: "local-lib".to_string(),
                link_type: LocalLinkType::File,
                path: "../local".to_string(),
            }],
            stats: CategorizationStats {
                total_packages: 2,
                internal_packages: 1,
                external_packages: 1,
                workspace_links: 1,
                local_links: 1,
            },
        };

        let json_result = serde_json::to_string(&categorization);
        assert!(json_result.is_ok(), "Should serialize to JSON");

        let json = json_result.expect("JSON serialization succeeded");
        let deserialized: Result<DependencyCategorization, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "Should deserialize from JSON");

        let deserialized_cat = deserialized.expect("Deserialization succeeded");
        assert_eq!(deserialized_cat.stats.total_packages, 2);
        assert_eq!(deserialized_cat.internal_packages.len(), 1);
        assert_eq!(deserialized_cat.external_packages.len(), 1);
        assert_eq!(deserialized_cat.workspace_links.len(), 1);
        assert_eq!(deserialized_cat.local_links.len(), 1);
    }

    #[tokio::test]
    async fn test_audit_manager_categorize_dependencies() {
        let (_temp_dir, workspace_path) = setup_test_workspace().await;

        let config = PackageToolsConfig::default();
        let manager =
            AuditManager::new(workspace_path, config).await.expect("Manager should initialize");

        let result = manager.categorize_dependencies().await;
        assert!(result.is_ok(), "Should categorize dependencies for workspace");

        let categorization = result.expect("Categorization succeeded");
        assert_eq!(categorization.stats.total_packages, 1); // The test package
    }

    // ===== Breaking Changes Audit Tests =====

    #[test]
    fn test_breaking_changes_section_empty() {
        use crate::audit::BreakingChangesAuditSection;

        let section = BreakingChangesAuditSection::empty();
        assert_eq!(section.total_breaking_changes, 0);
        assert!(section.packages_with_breaking.is_empty());
        assert!(section.issues.is_empty());
        assert!(!section.has_breaking_changes());
        assert_eq!(section.affected_package_count(), 0);
        assert_eq!(section.critical_issue_count(), 0);
        assert_eq!(section.warning_issue_count(), 0);
    }

    #[test]
    fn test_breaking_changes_section_accessors() {
        use crate::audit::{
            BreakingChange, BreakingChangeSource, BreakingChangesAuditSection,
            PackageBreakingChanges,
        };
        use crate::types::Version;

        let mut section = BreakingChangesAuditSection::empty();
        section.total_breaking_changes = 3;
        section.packages_with_breaking.push(PackageBreakingChanges {
            package_name: "@myorg/core".to_string(),
            current_version: Some(Version::parse("1.2.3").unwrap()),
            next_version: Some(Version::parse("2.0.0").unwrap()),
            breaking_changes: vec![
                BreakingChange {
                    description: "Removed deprecated API".to_string(),
                    commit_hash: Some("abc123".to_string()),
                    source: BreakingChangeSource::ConventionalCommit,
                },
                BreakingChange {
                    description: "Changed function signature".to_string(),
                    commit_hash: Some("def456".to_string()),
                    source: BreakingChangeSource::ConventionalCommit,
                },
            ],
        });

        assert!(section.has_breaking_changes());
        assert_eq!(section.affected_package_count(), 1);
        assert!(section.breaking_changes_for_package("@myorg/core").is_some());
        assert!(section.breaking_changes_for_package("@myorg/other").is_none());

        let core_breaking = section.breaking_changes_for_package("@myorg/core").unwrap();
        assert_eq!(core_breaking.breaking_change_count(), 2);
        assert!(core_breaking.is_major_bump());
    }

    #[test]
    fn test_package_breaking_changes_methods() {
        use crate::audit::{BreakingChange, BreakingChangeSource, PackageBreakingChanges};
        use crate::types::Version;

        let major_bump = PackageBreakingChanges {
            package_name: "@myorg/core".to_string(),
            current_version: Some(Version::parse("1.2.3").unwrap()),
            next_version: Some(Version::parse("2.0.0").unwrap()),
            breaking_changes: vec![],
        };
        assert!(major_bump.is_major_bump());

        let minor_bump = PackageBreakingChanges {
            package_name: "@myorg/core".to_string(),
            current_version: Some(Version::parse("1.2.3").unwrap()),
            next_version: Some(Version::parse("1.3.0").unwrap()),
            breaking_changes: vec![],
        };
        assert!(!minor_bump.is_major_bump());

        let no_version = PackageBreakingChanges {
            package_name: "@myorg/core".to_string(),
            current_version: None,
            next_version: None,
            breaking_changes: vec![BreakingChange {
                description: "Breaking change".to_string(),
                commit_hash: None,
                source: BreakingChangeSource::Changeset,
            }],
        };
        assert!(!no_version.is_major_bump());
        assert_eq!(no_version.breaking_change_count(), 1);
    }

    #[test]
    fn test_breaking_change_methods() {
        use crate::audit::{BreakingChange, BreakingChangeSource};

        let commit_change = BreakingChange {
            description: "API change".to_string(),
            commit_hash: Some("abc123".to_string()),
            source: BreakingChangeSource::ConventionalCommit,
        };
        assert!(commit_change.has_commit());
        assert!(commit_change.is_from_conventional_commit());
        assert!(!commit_change.is_from_changeset());
        assert!(!commit_change.is_from_changelog());

        let changeset_change = BreakingChange {
            description: "Major refactor".to_string(),
            commit_hash: None,
            source: BreakingChangeSource::Changeset,
        };
        assert!(!changeset_change.has_commit());
        assert!(!changeset_change.is_from_conventional_commit());
        assert!(changeset_change.is_from_changeset());
        assert!(!changeset_change.is_from_changelog());

        let changelog_change = BreakingChange {
            description: "Breaking API change".to_string(),
            commit_hash: None,
            source: BreakingChangeSource::Changelog,
        };
        assert!(!changelog_change.has_commit());
        assert!(!changelog_change.is_from_conventional_commit());
        assert!(!changelog_change.is_from_changeset());
        assert!(changelog_change.is_from_changelog());
    }

    #[test]
    fn test_breaking_change_source_serialization() {
        use crate::audit::BreakingChangeSource;

        let sources = vec![
            BreakingChangeSource::ConventionalCommit,
            BreakingChangeSource::Changelog,
            BreakingChangeSource::Changeset,
        ];

        for source in sources {
            let json_result = serde_json::to_string(&source);
            assert!(json_result.is_ok(), "Should serialize BreakingChangeSource");

            let json = json_result.unwrap();
            let deserialized: Result<BreakingChangeSource, _> = serde_json::from_str(&json);
            assert!(deserialized.is_ok(), "Should deserialize BreakingChangeSource");
            assert_eq!(deserialized.unwrap(), source);
        }
    }

    #[test]
    fn test_breaking_changes_section_serialization() {
        use crate::audit::{
            BreakingChange, BreakingChangeSource, BreakingChangesAuditSection,
            PackageBreakingChanges,
        };
        use crate::types::Version;

        let mut section = BreakingChangesAuditSection::empty();
        section.total_breaking_changes = 2;
        section.packages_with_breaking.push(PackageBreakingChanges {
            package_name: "@myorg/core".to_string(),
            current_version: Some(Version::parse("1.0.0").unwrap()),
            next_version: Some(Version::parse("2.0.0").unwrap()),
            breaking_changes: vec![
                BreakingChange {
                    description: "Removed old API".to_string(),
                    commit_hash: Some("abc123".to_string()),
                    source: BreakingChangeSource::ConventionalCommit,
                },
                BreakingChange {
                    description: "Major version bump".to_string(),
                    commit_hash: None,
                    source: BreakingChangeSource::Changeset,
                },
            ],
        });

        let json_result = serde_json::to_string(&section);
        assert!(json_result.is_ok(), "Should serialize BreakingChangesAuditSection");

        let json = json_result.unwrap();
        let deserialized: Result<BreakingChangesAuditSection, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "Should deserialize BreakingChangesAuditSection");

        let deserialized_section = deserialized.unwrap();
        assert_eq!(deserialized_section.total_breaking_changes, 2);
        assert_eq!(deserialized_section.packages_with_breaking.len(), 1);
        assert_eq!(deserialized_section.packages_with_breaking[0].breaking_changes.len(), 2);
    }

    #[test]
    fn test_breaking_changes_config_default() {
        use crate::config::BreakingChangesAuditConfig;

        let config = BreakingChangesAuditConfig::default();
        assert!(config.check_conventional_commits);
        assert!(config.check_changelog);
    }

    #[test]
    fn test_breaking_changes_section_with_multiple_packages() {
        use crate::audit::{
            BreakingChange, BreakingChangeSource, BreakingChangesAuditSection,
            PackageBreakingChanges,
        };
        use crate::types::Version;

        let mut section = BreakingChangesAuditSection::empty();
        section.total_breaking_changes = 4;

        section.packages_with_breaking.push(PackageBreakingChanges {
            package_name: "@myorg/core".to_string(),
            current_version: Some(Version::parse("1.0.0").unwrap()),
            next_version: Some(Version::parse("2.0.0").unwrap()),
            breaking_changes: vec![
                BreakingChange {
                    description: "Breaking 1".to_string(),
                    commit_hash: Some("abc".to_string()),
                    source: BreakingChangeSource::ConventionalCommit,
                },
                BreakingChange {
                    description: "Breaking 2".to_string(),
                    commit_hash: Some("def".to_string()),
                    source: BreakingChangeSource::ConventionalCommit,
                },
            ],
        });

        section.packages_with_breaking.push(PackageBreakingChanges {
            package_name: "@myorg/utils".to_string(),
            current_version: Some(Version::parse("0.5.0").unwrap()),
            next_version: Some(Version::parse("1.0.0").unwrap()),
            breaking_changes: vec![
                BreakingChange {
                    description: "Breaking 3".to_string(),
                    commit_hash: None,
                    source: BreakingChangeSource::Changeset,
                },
                BreakingChange {
                    description: "Breaking 4".to_string(),
                    commit_hash: None,
                    source: BreakingChangeSource::Changelog,
                },
            ],
        });

        assert!(section.has_breaking_changes());
        assert_eq!(section.affected_package_count(), 2);
        assert_eq!(section.total_breaking_changes, 4);
        assert!(section.breaking_changes_for_package("@myorg/core").is_some());
        assert!(section.breaking_changes_for_package("@myorg/utils").is_some());
        assert!(section.breaking_changes_for_package("@myorg/other").is_none());

        let core = section.breaking_changes_for_package("@myorg/core").unwrap();
        assert_eq!(core.breaking_change_count(), 2);
        assert!(core.is_major_bump());

        let utils = section.breaking_changes_for_package("@myorg/utils").unwrap();
        assert_eq!(utils.breaking_change_count(), 2);
        assert!(utils.is_major_bump());
    }

    // Version Consistency Tests (Story 10.6)

    #[tokio::test]
    async fn test_version_consistency_section_empty() {
        use crate::audit::VersionConsistencyAuditSection;

        let section = VersionConsistencyAuditSection::empty();
        assert_eq!(section.inconsistencies.len(), 0);
        assert_eq!(section.issues.len(), 0);
        assert!(!section.has_inconsistencies());
        assert_eq!(section.critical_issue_count(), 0);
        assert_eq!(section.warning_issue_count(), 0);
        assert_eq!(section.info_issue_count(), 0);
    }

    #[tokio::test]
    async fn test_audit_version_consistency_no_internal_deps() {
        use crate::audit::sections::audit_version_consistency;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use std::collections::HashSet;

        let package_json = package_json::PackageJson {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        };

        let packages = vec![PackageInfo::new(package_json, None, PathBuf::from("/workspace/test"))];

        let internal_names: HashSet<String> =
            packages.iter().map(|p| p.name().to_string()).collect();

        let config = PackageToolsConfig::default();
        let result = audit_version_consistency(&packages, &internal_names, &config).await;
        assert!(result.is_ok(), "Should handle packages with no internal dependencies");

        let section = result.expect("Audit succeeded");
        assert_eq!(section.inconsistencies.len(), 0);
        assert_eq!(section.issues.len(), 0);
    }

    #[tokio::test]
    async fn test_audit_version_consistency_consistent_versions() {
        use crate::audit::sections::audit_version_consistency;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use std::collections::HashMap;
        use std::collections::HashSet;

        let mut deps_a = HashMap::new();
        deps_a.insert("@myorg/core".to_string(), "workspace:*".to_string());

        let mut deps_b = HashMap::new();
        deps_b.insert("@myorg/core".to_string(), "workspace:*".to_string());

        let package_a = package_json::PackageJson {
            name: "@myorg/app-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(deps_a),
            ..Default::default()
        };

        let package_b = package_json::PackageJson {
            name: "@myorg/app-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(deps_b),
            ..Default::default()
        };

        let package_core = package_json::PackageJson {
            name: "@myorg/core".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a, None, PathBuf::from("/workspace/app-a")),
            PackageInfo::new(package_b, None, PathBuf::from("/workspace/app-b")),
            PackageInfo::new(package_core, None, PathBuf::from("/workspace/core")),
        ];

        let internal_names: HashSet<String> =
            packages.iter().map(|p| p.name().to_string()).collect();

        let config = PackageToolsConfig::default();
        let result = audit_version_consistency(&packages, &internal_names, &config).await;
        assert!(result.is_ok(), "Should handle consistent versions");

        let section = result.expect("Audit succeeded");
        assert_eq!(section.inconsistencies.len(), 0);
        assert_eq!(section.issues.len(), 0);
    }

    #[tokio::test]
    async fn test_audit_version_consistency_with_inconsistencies() {
        use crate::audit::sections::audit_version_consistency;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use std::collections::HashMap;
        use std::collections::HashSet;

        let mut deps_a = HashMap::new();
        deps_a.insert("@myorg/core".to_string(), "^1.0.0".to_string());

        let mut deps_b = HashMap::new();
        deps_b.insert("@myorg/core".to_string(), "^1.1.0".to_string());

        let package_a = package_json::PackageJson {
            name: "@myorg/app-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(deps_a),
            ..Default::default()
        };

        let package_b = package_json::PackageJson {
            name: "@myorg/app-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(deps_b),
            ..Default::default()
        };

        let package_core = package_json::PackageJson {
            name: "@myorg/core".to_string(),
            version: "1.1.0".to_string(),
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a, None, PathBuf::from("/workspace/app-a")),
            PackageInfo::new(package_b, None, PathBuf::from("/workspace/app-b")),
            PackageInfo::new(package_core, None, PathBuf::from("/workspace/core")),
        ];

        let internal_names: HashSet<String> =
            packages.iter().map(|p| p.name().to_string()).collect();

        let config = PackageToolsConfig::default();
        let result = audit_version_consistency(&packages, &internal_names, &config).await;
        assert!(result.is_ok(), "Should detect inconsistencies");

        let section = result.expect("Audit succeeded");
        assert_eq!(section.inconsistencies.len(), 1);
        assert!(section.has_inconsistencies());

        let inconsistency = &section.inconsistencies[0];
        assert_eq!(inconsistency.package_name, "@myorg/core");
        assert_eq!(inconsistency.version_count(), 2);
        assert_eq!(inconsistency.versions_used.len(), 2);

        // Should generate warning issues by default
        assert_eq!(section.warning_issue_count(), 1);
        assert_eq!(section.critical_issue_count(), 0);
    }

    #[tokio::test]
    async fn test_audit_version_consistency_recommends_workspace_protocol() {
        use crate::audit::sections::audit_version_consistency;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use std::collections::HashMap;
        use std::collections::HashSet;

        let mut deps_a = HashMap::new();
        deps_a.insert("@myorg/core".to_string(), "workspace:*".to_string());

        let mut deps_b = HashMap::new();
        deps_b.insert("@myorg/core".to_string(), "^1.0.0".to_string());

        let package_a = package_json::PackageJson {
            name: "@myorg/app-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(deps_a),
            ..Default::default()
        };

        let package_b = package_json::PackageJson {
            name: "@myorg/app-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(deps_b),
            ..Default::default()
        };

        let package_core = package_json::PackageJson {
            name: "@myorg/core".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a, None, PathBuf::from("/workspace/app-a")),
            PackageInfo::new(package_b, None, PathBuf::from("/workspace/app-b")),
            PackageInfo::new(package_core, None, PathBuf::from("/workspace/core")),
        ];

        let internal_names: HashSet<String> =
            packages.iter().map(|p| p.name().to_string()).collect();

        let config = PackageToolsConfig::default();
        let result = audit_version_consistency(&packages, &internal_names, &config).await;
        assert!(result.is_ok());

        let section = result.expect("Audit succeeded");
        assert_eq!(section.inconsistencies.len(), 1);

        let inconsistency = &section.inconsistencies[0];
        assert_eq!(inconsistency.recommended_version, "workspace:*");
    }

    #[tokio::test]
    async fn test_audit_version_consistency_with_fail_on_inconsistency() {
        use crate::audit::sections::audit_version_consistency;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use std::collections::HashMap;
        use std::collections::HashSet;

        let mut deps_a = HashMap::new();
        deps_a.insert("@myorg/core".to_string(), "^1.0.0".to_string());

        let mut deps_b = HashMap::new();
        deps_b.insert("@myorg/core".to_string(), "^1.1.0".to_string());

        let package_a = package_json::PackageJson {
            name: "@myorg/app-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(deps_a),
            ..Default::default()
        };

        let package_b = package_json::PackageJson {
            name: "@myorg/app-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(deps_b),
            ..Default::default()
        };

        let package_core = package_json::PackageJson {
            name: "@myorg/core".to_string(),
            version: "1.1.0".to_string(),
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a, None, PathBuf::from("/workspace/app-a")),
            PackageInfo::new(package_b, None, PathBuf::from("/workspace/app-b")),
            PackageInfo::new(package_core, None, PathBuf::from("/workspace/core")),
        ];

        let internal_names: HashSet<String> =
            packages.iter().map(|p| p.name().to_string()).collect();

        let mut config = PackageToolsConfig::default();
        config.audit.version_consistency.fail_on_inconsistency = true;

        let result = audit_version_consistency(&packages, &internal_names, &config).await;
        assert!(result.is_ok());

        let section = result.expect("Audit succeeded");
        assert_eq!(section.inconsistencies.len(), 1);

        // Should generate critical issues when fail_on_inconsistency is true
        assert_eq!(section.critical_issue_count(), 1);
        assert_eq!(section.warning_issue_count(), 0);
    }

    #[tokio::test]
    async fn test_audit_version_consistency_section_disabled() {
        use crate::audit::sections::audit_version_consistency;
        use crate::config::PackageToolsConfig;
        use std::collections::HashSet;

        let packages = vec![];
        let internal_names = HashSet::new();

        let mut config = PackageToolsConfig::default();
        config.audit.sections.version_consistency = false;

        let result = audit_version_consistency(&packages, &internal_names, &config).await;
        assert!(result.is_err(), "Should return error when section is disabled");

        if let Err(e) = result {
            assert!(matches!(e, crate::error::AuditError::SectionDisabled { .. }));
        }
    }

    #[tokio::test]
    async fn test_version_inconsistency_methods() {
        use crate::audit::{VersionInconsistency, VersionUsage};

        #[allow(clippy::too_many_lines)]
        let inconsistency = VersionInconsistency {
            package_name: "@myorg/core".to_string(),
            versions_used: vec![
                VersionUsage {
                    package_name: "app-a".to_string(),
                    version_spec: "^1.0.0".to_string(),
                },
                VersionUsage {
                    package_name: "app-b".to_string(),
                    version_spec: "^1.0.0".to_string(),
                },
                VersionUsage {
                    package_name: "app-c".to_string(),
                    version_spec: "^1.1.0".to_string(),
                },
            ],
            recommended_version: "^1.1.0".to_string(),
        };

        assert_eq!(inconsistency.version_count(), 3);

        let unique = inconsistency.unique_versions();
        assert_eq!(unique.len(), 2);
        assert!(unique.contains(&"^1.0.0".to_string()));
        assert!(unique.contains(&"^1.1.0".to_string()));

        let description = inconsistency.describe();
        assert!(description.contains("@myorg/core"));
        assert!(description.contains("3 different versions"));
    }

    #[tokio::test]
    async fn test_version_consistency_section_accessors() {
        use crate::audit::{AuditIssue, IssueCategory, IssueSeverity};
        use crate::audit::{VersionConsistencyAuditSection, VersionInconsistency, VersionUsage};

        #[allow(clippy::too_many_lines)]
        let inconsistency1 = VersionInconsistency {
            package_name: "@myorg/core".to_string(),
            versions_used: vec![
                VersionUsage {
                    package_name: "app-a".to_string(),
                    version_spec: "^1.0.0".to_string(),
                },
                VersionUsage {
                    package_name: "app-b".to_string(),
                    version_spec: "^1.1.0".to_string(),
                },
            ],
            recommended_version: "^1.1.0".to_string(),
        };

        let inconsistency2 = VersionInconsistency {
            package_name: "@myorg/utils".to_string(),
            versions_used: vec![
                VersionUsage {
                    package_name: "app-a".to_string(),
                    version_spec: "workspace:*".to_string(),
                },
                VersionUsage {
                    package_name: "app-c".to_string(),
                    version_spec: "^2.0.0".to_string(),
                },
            ],
            recommended_version: "workspace:*".to_string(),
        };

        let section = VersionConsistencyAuditSection {
            inconsistencies: vec![inconsistency1, inconsistency2],
            issues: vec![
                AuditIssue::new(
                    IssueSeverity::Warning,
                    IssueCategory::VersionConsistency,
                    "Test issue 1".to_string(),
                    "Description 1".to_string(),
                ),
                AuditIssue::new(
                    IssueSeverity::Critical,
                    IssueCategory::VersionConsistency,
                    "Test issue 2".to_string(),
                    "Description 2".to_string(),
                ),
            ],
        };

        assert_eq!(section.inconsistencies.len(), 2);
        assert!(section.has_inconsistencies());
        assert_eq!(section.warning_issue_count(), 1);
        assert_eq!(section.critical_issue_count(), 1);
        assert_eq!(section.info_issue_count(), 0);

        assert!(section.inconsistency_for_package("@myorg/core").is_some());
        assert!(section.inconsistency_for_package("@myorg/utils").is_some());
        assert!(section.inconsistency_for_package("@myorg/other").is_none());

        let core = section.inconsistency_for_package("@myorg/core").unwrap();
        assert_eq!(core.version_count(), 2);
        assert_eq!(core.recommended_version, "^1.1.0");
    }

    #[tokio::test]
    async fn test_audit_version_consistency_multiple_inconsistencies() {
        use crate::audit::sections::audit_version_consistency;
        use crate::config::PackageToolsConfig;
        use crate::types::PackageInfo;
        use std::collections::HashMap;
        use std::collections::HashSet;

        let mut deps_a = HashMap::new();
        deps_a.insert("@myorg/core".to_string(), "^1.0.0".to_string());
        deps_a.insert("@myorg/utils".to_string(), "workspace:*".to_string());

        let mut deps_b = HashMap::new();
        deps_b.insert("@myorg/core".to_string(), "^1.1.0".to_string());
        deps_b.insert("@myorg/utils".to_string(), "^2.0.0".to_string());

        let package_a = package_json::PackageJson {
            name: "@myorg/app-a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(deps_a),
            ..Default::default()
        };

        let package_b = package_json::PackageJson {
            name: "@myorg/app-b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(deps_b),
            ..Default::default()
        };

        let package_core = package_json::PackageJson {
            name: "@myorg/core".to_string(),
            version: "1.1.0".to_string(),
            ..Default::default()
        };

        let package_utils = package_json::PackageJson {
            name: "@myorg/utils".to_string(),
            version: "2.0.0".to_string(),
            ..Default::default()
        };

        let packages = vec![
            PackageInfo::new(package_a, None, PathBuf::from("/workspace/app-a")),
            PackageInfo::new(package_b, None, PathBuf::from("/workspace/app-b")),
            PackageInfo::new(package_core, None, PathBuf::from("/workspace/core")),
            PackageInfo::new(package_utils, None, PathBuf::from("/workspace/utils")),
        ];

        let internal_names: HashSet<String> =
            packages.iter().map(|p| p.name().to_string()).collect();

        let config = PackageToolsConfig::default();
        let result = audit_version_consistency(&packages, &internal_names, &config).await;
        assert!(result.is_ok());

        let section = result.expect("Audit succeeded");
        assert_eq!(section.inconsistencies.len(), 2);

        // Verify both packages have inconsistencies
        assert!(section.inconsistency_for_package("@myorg/core").is_some());
        assert!(section.inconsistency_for_package("@myorg/utils").is_some());
    }

    #[tokio::test]
    async fn test_version_consistency_serialization() {
        use crate::audit::{VersionConsistencyAuditSection, VersionInconsistency, VersionUsage};

        let inconsistency = VersionInconsistency {
            package_name: "@myorg/core".to_string(),
            versions_used: vec![
                VersionUsage {
                    package_name: "app-a".to_string(),
                    version_spec: "^1.0.0".to_string(),
                },
                VersionUsage {
                    package_name: "app-b".to_string(),
                    version_spec: "^1.1.0".to_string(),
                },
            ],
            recommended_version: "^1.1.0".to_string(),
        };

        let section =
            VersionConsistencyAuditSection { inconsistencies: vec![inconsistency], issues: vec![] };

        let json = serde_json::to_string(&section).expect("Should serialize to JSON");
        assert!(json.contains("@myorg/core"));
        assert!(json.contains("^1.0.0"));
        assert!(json.contains("^1.1.0"));

        let deserialized: VersionConsistencyAuditSection =
            serde_json::from_str(&json).expect("Should deserialize from JSON");
        assert_eq!(deserialized.inconsistencies.len(), 1);
        assert_eq!(deserialized.inconsistencies[0].package_name, "@myorg/core");
    }

    #[tokio::test]
    async fn test_audit_manager_version_consistency() {
        use crate::audit::AuditManager;
        use crate::config::PackageToolsConfig;
        use std::collections::HashMap;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let workspace_root = temp_dir.path().to_path_buf();

        // Initialize git repository
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&workspace_root)
            .output()
            .expect("Failed to initialize git repo");

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&workspace_root)
            .output()
            .expect("Failed to configure git");

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&workspace_root)
            .output()
            .expect("Failed to configure git");

        // Create a simple package with inconsistent dependencies
        let mut deps = HashMap::new();
        deps.insert("lodash".to_string(), "^4.17.20".to_string());

        let package_json = package_json::PackageJson {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            dependencies: Some(deps),
            ..Default::default()
        };

        let package_json_content =
            serde_json::to_string_pretty(&package_json).expect("Failed to serialize package.json");

        let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
        fs.write_file_string(&workspace_root.join("package.json"), package_json_content.as_str())
            .await
            .expect("Failed to write package.json");

        // Create initial commit
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&workspace_root)
            .output()
            .expect("Failed to add files");

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&workspace_root)
            .output()
            .expect("Failed to commit");

        let config = PackageToolsConfig::default();
        let manager = AuditManager::new(workspace_root.clone(), config)
            .await
            .expect("Failed to create audit manager");

        let result = manager.audit_version_consistency().await;
        assert!(result.is_ok(), "Should audit version consistency");
    }
}
