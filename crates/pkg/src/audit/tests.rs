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
}
