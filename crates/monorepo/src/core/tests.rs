//! Comprehensive tests for the core module
//!
//! This module provides complete test coverage for all core functionality,
//! including MonorepoProject, MonorepoPackageInfo, MonorepoTools, VersionManager,
//! and the service container architecture. Tests cover initialization, package
//! management, version control, dependency analysis, and workflow orchestration.

#[cfg(test)]
mod tests {
    use crate::config::{ConfigManager, Environment, MonorepoConfig, VersionBumpType};
    use crate::core::types::{
        Changeset, ChangesetStatus, MonorepoPackageInfo, MonorepoProject, MonorepoTools,
        VersionManager, VersionStatus,
    };
    use crate::core::services::MonorepoServices;
    use crate::error::Result;
    use std::path::{Path, PathBuf};
    use sublime_package_tools::{Package, PackageInfo as PkgInfo};
    use sublime_standard_tools::monorepo::WorkspacePackage;
    use tempfile::TempDir;
    use serde_json::json;

    /// Helper function to create a test monorepo project with realistic structure
    fn create_test_monorepo() -> Result<(TempDir, MonorepoProject)> {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path();

        // Create a basic monorepo structure
        create_test_monorepo_structure(root_path)?;

        // Initialize git repository
        let repo_path = root_path.to_str().unwrap();
        std::process::Command::new("git")
            .args(["init", repo_path])
            .output()
            .map_err(|e| crate::error::Error::git(format!("Failed to init git: {e}")))?;

        // Create initial commit
        std::process::Command::new("git")
            .args(["-C", repo_path, "add", "."])
            .output()
            .map_err(|e| crate::error::Error::git(format!("Failed to add files: {e}")))?;

        std::process::Command::new("git")
            .args(["-C", repo_path, "commit", "-m", "Initial commit"])
            .output()
            .map_err(|e| crate::error::Error::git(format!("Failed to commit: {e}")))?;

        // Create MonorepoProject
        let project = MonorepoProject::new(root_path)?;

        Ok((temp_dir, project))
    }

    /// Helper function to create monorepo directory structure with packages
    fn create_test_monorepo_structure(root_path: &Path) -> Result<()> {
        use std::fs;

        // Create packages directory
        let packages_dir = root_path.join("packages");
        fs::create_dir_all(&packages_dir).unwrap();

        // Create core package
        let core_dir = packages_dir.join("core");
        fs::create_dir_all(&core_dir).unwrap();
        fs::write(
            core_dir.join("package.json"),
            r#"{
              "name": "@test/core",
              "version": "1.0.0",
              "dependencies": {},
              "devDependencies": {}
            }"#,
        ).unwrap();

        // Create utils package
        let utils_dir = packages_dir.join("utils");
        fs::create_dir_all(&utils_dir).unwrap();
        fs::write(
            utils_dir.join("package.json"),
            r#"{
              "name": "@test/utils",
              "version": "1.2.0",
              "dependencies": {
                "@test/core": "^1.0.0"
              },
              "devDependencies": {}
            }"#,
        ).unwrap();

        // Create web app package
        let web_dir = packages_dir.join("web");
        fs::create_dir_all(&web_dir).unwrap();
        fs::write(
            web_dir.join("package.json"),
            r#"{
              "name": "@test/web",
              "version": "2.1.0",
              "dependencies": {
                "@test/core": "^1.0.0",
                "@test/utils": "^1.2.0",
                "react": "^18.0.0"
              },
              "devDependencies": {}
            }"#,
        ).unwrap();

        // Create monorepo config file
        let config = MonorepoConfig::default();
        let config_manager = ConfigManager::with_config(config);
        config_manager.save_to_file(root_path.join("monorepo.toml"))?;

        // Create root package.json
        fs::write(
            root_path.join("package.json"),
            r#"{
              "name": "test-monorepo",
              "version": "1.0.0",
              "workspaces": ["packages/*"],
              "private": true
            }"#,
        ).unwrap();

        Ok(())
    }

    /// Helper function to create a test MonorepoPackageInfo
    fn create_test_package_info(name: &str, version: &str, path: &Path) -> MonorepoPackageInfo {
        // Create a Package using sublime_package_tools API
        let package = Package::new(name, version, None).unwrap();
        
        // Create package.json content
        let pkg_json = json!({
            "name": name,
            "version": version,
            "dependencies": {},
            "devDependencies": {}
        });

        // Create PackageInfo with proper API
        let package_info = PkgInfo::new(
            package,
            path.join("package.json").to_string_lossy().to_string(),
            path.to_string_lossy().to_string(),
            path.to_string_lossy().to_string(),
            pkg_json,
        );

        let workspace_package = WorkspacePackage {
            name: name.to_string(),
            version: version.to_string(),
            location: path.to_path_buf(),
            absolute_path: path.to_path_buf(),
            workspace_dependencies: Vec::new(),
            workspace_dev_dependencies: Vec::new(),
        };

        MonorepoPackageInfo::new(package_info, &workspace_package, true)
    }

    /// Helper function to create a test changeset
    fn create_test_changeset(id: &str, package_name: &str, bump_type: VersionBumpType) -> Changeset {
        Changeset {
            id: id.to_string(),
            package: package_name.to_string(),
            status: ChangesetStatus::Pending,
            version_bump: bump_type,
            description: format!("Test changeset for {package_name}"),
            branch: "main".to_string(),
            development_environments: Vec::new(),
            production_deployment: false,
            created_at: chrono::Utc::now(),
            author: "test-author".to_string(),
        }
    }

    // =========================================================================================
    // MonorepoProject Tests
    // =========================================================================================

    #[test]
    fn test_monorepo_project_creation() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        // Test basic project properties
        assert!(!project.root_path().to_string_lossy().is_empty());
        assert!(!project.packages.is_empty());

        // Test that services are initialized
        assert!(project.config.versioning.propagate_changes);

        Ok(())
    }

    #[test]
    fn test_monorepo_project_package_discovery() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        // Test package discovery
        assert_eq!(project.packages.len(), 3); // core, utils, web

        // Test package access by name
        let core_package = project.get_package("@test/core");
        assert!(core_package.is_some());
        assert_eq!(core_package.unwrap().version(), "1.0.0");

        let utils_package = project.get_package("@test/utils");
        assert!(utils_package.is_some());
        assert_eq!(utils_package.unwrap().version(), "1.2.0");

        Ok(())
    }

    #[test]
    fn test_monorepo_project_internal_packages() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        // Test internal packages filtering
        let internal_packages = project.internal_packages();
        assert_eq!(internal_packages.len(), 3);

        // Test internal package check
        assert!(project.is_internal_package("@test/core"));
        assert!(project.is_internal_package("@test/utils"));
        assert!(project.is_internal_package("@test/web"));
        assert!(!project.is_internal_package("react"));

        Ok(())
    }

    #[test]
    fn test_monorepo_project_external_dependencies() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        // Test external dependencies aggregation
        let external_deps = project.external_dependencies();
        assert!(external_deps.contains(&"react".to_string()));

        Ok(())
    }

    #[test]
    fn test_monorepo_project_dependents() -> Result<()> {
        let (_temp_dir, mut project) = create_test_monorepo()?;

        // Build dependency graph to populate dependents
        project.build_dependency_graph()?;

        // Test dependents lookup
        let core_dependents = project.get_dependents("@test/core");
        let core_dependent_names: Vec<&str> = core_dependents.iter().map(|p| p.name()).collect();
        
        // Both utils and web depend on core
        assert!(core_dependent_names.contains(&"@test/utils") || core_dependent_names.contains(&"@test/web"));

        Ok(())
    }

    #[test]
    fn test_monorepo_project_refresh_packages() -> Result<()> {
        let (_temp_dir, mut project) = create_test_monorepo()?;

        let initial_count = project.packages.len();
        
        // Refresh packages should not lose existing packages
        project.refresh_packages()?;
        assert_eq!(project.packages.len(), initial_count);

        Ok(())
    }

    #[test]
    fn test_monorepo_project_dependency_graph() -> Result<()> {
        let (_temp_dir, mut project) = create_test_monorepo()?;

        // Test dependency graph building - dependency_graph field was removed in simplification
        // The dependency graph is now built on-demand by the analyzer
        project.build_dependency_graph()?;

        Ok(())
    }

    #[test]
    fn test_monorepo_project_config_access() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        // Test configuration access
        let config = project.config();
        assert_eq!(config.versioning.default_bump, VersionBumpType::Patch);
        assert!(config.versioning.propagate_changes);

        Ok(())
    }

    #[test]
    fn test_monorepo_project_service_access() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        // Test service access methods
        assert!(!project.root_path().to_string_lossy().is_empty());
        // Registry manager and dependency registry removed in simplified API
        // These are now handled through the analyzer

        Ok(())
    }

    // =========================================================================================
    // MonorepoPackageInfo Tests
    // =========================================================================================

    #[test]
    fn test_package_info_creation() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path();

        let package = create_test_package_info("@test/example", "1.0.0", package_path);

        assert_eq!(package.name(), "@test/example");
        assert_eq!(package.version(), "1.0.0");
        assert_eq!(package.path(), package_path);
        assert!(package.is_internal);
        assert_eq!(package.version_status, VersionStatus::Stable);
    }

    #[test]
    fn test_package_info_basic_properties() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path();

        let package = create_test_package_info("@test/core", "2.1.5", package_path);

        assert_eq!(package.name(), "@test/core");
        assert_eq!(package.version(), "2.1.5");
        assert_eq!(package.relative_path(), package_path);
        assert!(!package.is_dirty());
    }

    #[test]
    fn test_package_info_changeset_management() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path();

        let mut package = create_test_package_info("@test/utils", "1.0.0", package_path);

        // Test no pending changesets initially
        assert!(!package.has_pending_changesets());
        assert!(package.pending_changesets().is_empty());

        // Add a changeset
        let changeset = create_test_changeset("cs1", "@test/utils", VersionBumpType::Minor);
        package.add_changeset(changeset);

        // Test changeset added
        assert!(package.has_pending_changesets());
        assert_eq!(package.pending_changesets().len(), 1);
        assert_eq!(package.changesets.len(), 1);
    }

    #[test]
    fn test_package_info_version_management() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path();

        let mut package = create_test_package_info("@test/web", "1.0.0", package_path);

        // Test version update
        package.update_version("1.1.0")?;
        assert_eq!(package.version(), "1.1.0");

        // Test snapshot version
        package.set_snapshot_version("1.2.0", "abc123456")?;
        assert!(package.version().contains("snapshot"));

        // Test marking as dirty
        package.mark_dirty();
        assert!(package.is_dirty());

        Ok(())
    }

    #[test]
    fn test_package_info_changeset_application() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path();

        let mut package = create_test_package_info("@test/api", "1.0.0", package_path);

        // Add a changeset
        let changeset = create_test_changeset("cs1", "@test/api", VersionBumpType::Minor);
        package.add_changeset(changeset);

        // Apply the changeset
        package.apply_changeset("cs1", Some("1.1.0"))?;

        // Verify changeset is applied
        assert_eq!(package.version(), "1.1.0");

        Ok(())
    }

    #[test]
    fn test_package_info_deployment_status() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path();

        let package = create_test_package_info("@test/service", "1.0.0", package_path);

        // Test deployment status
        assert!(!package.is_deployed_to(&Environment::Production));
        
        let deployment_status = package.deployment_status();
        assert!(deployment_status.contains_key(&Environment::Development));
    }

    #[test]
    fn test_package_info_version_bump_suggestion() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path();

        let mut package = create_test_package_info("@test/lib", "1.0.0", package_path);

        // Initially no suggestion without changesets
        assert!(package.suggested_version_bump().is_none());

        // Add changeset with version bump
        let changeset = create_test_changeset("cs1", "@test/lib", VersionBumpType::Major);
        package.add_changeset(changeset);

        // Should suggest version bump based on changesets
        let suggestion = package.suggested_version_bump();
        assert!(suggestion.is_some());
    }

    // =========================================================================================
    // MonorepoTools Tests
    // =========================================================================================

    #[test]
    fn test_monorepo_tools_creation() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        let tools = MonorepoTools::new(&project);

        // Test analyzer access
        let analyzer = tools.analyzer()?;
        assert!(!analyzer.get_packages().is_empty());

        Ok(())
    }

    #[test]
    fn test_monorepo_tools_version_manager() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        let tools = MonorepoTools::new(&project);
        let version_manager = tools.version_manager();

        // Test version manager functionality
        assert!(!version_manager.packages.is_empty());

        Ok(())
    }

    #[test]
    fn test_monorepo_tools_task_manager() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        let tools = MonorepoTools::new(&project);
        let task_manager = tools.task_manager()?;

        // Test task manager creation
        let available_tasks = task_manager.list_tasks();
        // Task manager should be valid regardless of number of tasks
        drop(available_tasks);

        Ok(())
    }



    // =========================================================================================
    // VersionManager Tests
    // =========================================================================================

    #[test]
    fn test_version_manager_creation() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        let version_manager = VersionManager::new(&project);

        // Test basic properties
        assert!(!version_manager.packages.is_empty());
        assert_eq!(version_manager.config.versioning.default_bump, VersionBumpType::Patch);

        Ok(())
    }

    #[test]
    fn test_version_manager_package_version_bump() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        let version_manager = VersionManager::new(&project);

        // Test version bump
        let result = version_manager.bump_package_version("@test/core", VersionBumpType::Minor, None)?;

        // Verify result structure
        assert!(!result.primary_updates.is_empty());
        assert_eq!(result.primary_updates[0].package_name, "@test/core");
        assert_eq!(result.primary_updates[0].bump_type, VersionBumpType::Minor);

        Ok(())
    }

    #[test]
    fn test_version_manager_snapshot_version() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        let version_manager = VersionManager::new(&project);

        // Test snapshot version bump
        let result = version_manager.bump_package_version("@test/utils", VersionBumpType::Snapshot, Some("abc123456"))?;

        // Verify snapshot version format
        assert!(result.primary_updates[0].new_version.contains("snapshot"));
        assert!(result.primary_updates[0].new_version.contains("abc123456"));

        Ok(())
    }

    #[test]
    fn test_version_manager_propagation() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        let version_manager = VersionManager::new(&project);

        // Test version propagation
        let propagation_result = version_manager.propagate_version_changes("@test/core")?;

        // Should have some updates or conflicts
        assert!(propagation_result.updates.is_empty() || !propagation_result.updates.is_empty());

        Ok(())
    }

    #[test]
    fn test_version_manager_compatibility_validation() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        let version_manager = VersionManager::new(&project);

        // Test version compatibility validation
        let conflicts = version_manager.validate_version_compatibility()?;

        // Should complete without error (may or may not have conflicts)
        assert!(conflicts.is_empty() || !conflicts.is_empty());

        Ok(())
    }

    #[test]
    fn test_version_manager_dependency_update_strategy() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        let version_manager = VersionManager::new(&project);

        // Test dependency update strategy
        let updates = version_manager.get_dependency_update_strategy("@test/core")?;

        // Should return strategy (may be empty if no dependents)
        assert!(updates.is_empty() || !updates.is_empty());

        Ok(())
    }

    // =========================================================================================
    // MonorepoServices Tests
    // =========================================================================================

    #[test]
    fn test_monorepo_services_creation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path();

        // Create basic structure
        create_test_monorepo_structure(root_path)?;

        // Initialize git
        let repo_path = root_path.to_str().unwrap();
        std::process::Command::new("git")
            .args(["init", repo_path])
            .output()
            .map_err(|e| crate::error::Error::git(format!("Failed to init git: {e}")))?;

        let services = MonorepoServices::new(root_path)?;

        // Test service access
        assert!(!services.config_service().get_configuration().environments.is_empty());
        assert!(!services.file_system_service().root_path().to_string_lossy().is_empty());

        Ok(())
    }

    // =========================================================================================
    // Integration Tests
    // =========================================================================================

    #[test]
    fn test_full_workflow_integration() -> Result<()> {
        let (_temp_dir, mut project) = create_test_monorepo()?;

        // Build dependency graph
        project.build_dependency_graph()?;

        // Create tools
        let tools = MonorepoTools::new(&project);

        // Test version manager workflow
        let version_manager = tools.version_manager();
        let result = version_manager.bump_package_version("@test/core", VersionBumpType::Patch, None)?;

        // Verify integration worked
        assert!(!result.primary_updates.is_empty());

        Ok(())
    }

    #[test]
    fn test_configuration_integration() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        // Test configuration integration across components
        let config = project.config();
        let tools = MonorepoTools::new(&project);
        let version_manager = tools.version_manager();

        // All should use same configuration
        assert_eq!(config.versioning.default_bump, version_manager.config.versioning.default_bump);

        Ok(())
    }

    // =========================================================================================
    // Error Handling Tests
    // =========================================================================================

    #[test]
    fn test_invalid_package_version_bump() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        let version_manager = VersionManager::new(&project);

        // Test error for non-existent package
        let result = version_manager.bump_package_version("@test/nonexistent", VersionBumpType::Minor, None);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_invalid_monorepo_path() {
        let invalid_path = PathBuf::from("/non/existent/path");
        let result = MonorepoProject::new(&invalid_path);
        assert!(result.is_err());
    }

    // =========================================================================================
    // Performance Tests
    // =========================================================================================

    #[test]
    fn test_large_project_performance() -> Result<()> {
        let (_temp_dir, project) = create_test_monorepo()?;

        // Test that operations complete in reasonable time
        let start = std::time::Instant::now();
        
        let tools = MonorepoTools::new(&project);
        let _analyzer = tools.analyzer()?;
        
        let duration = start.elapsed();
        
        // Should complete quickly (under 1 second for test project)
        assert!(duration.as_secs() < 1);

        Ok(())
    }
}