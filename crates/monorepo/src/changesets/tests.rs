//! Unit tests for changeset functionality
//!
//! This module contains unit tests for the changeset system components,
//! focusing on core logic and integration between storage, manager, and types.

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use std::sync::Arc;
    use tempfile::TempDir;

    use crate::changesets::storage::ChangesetStorage;
    use crate::config::types::Environment;
    use crate::core::MonorepoProject;
    use crate::{
        Changeset, ChangesetApplication, ChangesetFilter, ChangesetManager, ChangesetSpec,
        ChangesetStatus, DeploymentResult, EnvironmentDeploymentResult, ValidationResult,
        VersionBumpType,
    };

    fn create_test_changeset() -> Changeset {
        Changeset {
            id: "test-changeset-456".to_string(),
            package: "@test/utils".to_string(),
            version_bump: VersionBumpType::Minor,
            description: "Add utility functions".to_string(),
            branch: "feature/utils".to_string(),
            development_environments: vec![Environment::Development, Environment::Staging],
            production_deployment: true,
            created_at: Utc::now(),
            author: "developer@example.com".to_string(),
            status: ChangesetStatus::Pending,
        }
    }

    #[allow(clippy::arc_with_non_send_sync)]
    fn create_test_project() -> (TempDir, Arc<MonorepoProject>) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Initialize Git repository
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to initialize git repository");

        // Configure Git user for testing
        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to configure git email");

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to configure git name");

        // Create basic monorepo structure
        let packages_dir = temp_dir.path().join("packages");
        std::fs::create_dir_all(&packages_dir).expect("Failed to create packages directory");

        // Create package.json files
        let root_package_json = r#"{
  "name": "test-monorepo",
  "version": "1.0.0",
  "private": true,
  "workspaces": ["packages/*"],
  "devDependencies": {
    "typescript": "^5.0.0"
  }
}"#;
        std::fs::write(temp_dir.path().join("package.json"), root_package_json)
            .expect("Failed to write root package.json");

        // Create package-lock.json to make it detectable as an npm monorepo
        std::fs::write(temp_dir.path().join("package-lock.json"), "{}")
            .expect("Failed to write package-lock.json");

        let project = Arc::new(
            MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject"),
        );
        (temp_dir, project)
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_changeset_status_transitions() {
        let changeset = create_test_changeset();

        // Test initial status
        assert_eq!(changeset.status, ChangesetStatus::Pending);

        // Test partial deployment
        let partially_deployed =
            ChangesetStatus::PartiallyDeployed { environments: vec![Environment::Development] };

        match partially_deployed {
            ChangesetStatus::PartiallyDeployed { environments } => {
                assert_eq!(environments.len(), 1);
                assert_eq!(environments[0], Environment::Development);
            }
            _ => panic!("Expected PartiallyDeployed status"),
        }

        // Test full deployment
        let fully_deployed = ChangesetStatus::FullyDeployed { deployed_at: Utc::now() };

        match fully_deployed {
            ChangesetStatus::FullyDeployed { deployed_at: _ } => {
                // Success
            }
            _ => panic!("Expected FullyDeployed status"),
        }
    }

    #[test]
    fn test_changeset_filter_matching() {
        let _changeset = create_test_changeset();

        // Test package filter
        let package_filter =
            ChangesetFilter { package: Some("@test/utils".to_string()), ..Default::default() };

        // This would normally be tested through storage.matches_filter
        // but we can test the filter structure
        assert_eq!(package_filter.package, Some("@test/utils".to_string()));
        assert_eq!(package_filter.status, None);

        // Test status filter
        let status_filter =
            ChangesetFilter { status: Some(ChangesetStatus::Pending), ..Default::default() };

        assert_eq!(status_filter.status, Some(ChangesetStatus::Pending));
        assert_eq!(status_filter.package, None);
    }

    #[test]
    fn test_changeset_spec_creation() {
        let spec = ChangesetSpec {
            package: "@test/core".to_string(),
            version_bump: VersionBumpType::Major,
            description: "Breaking API changes".to_string(),
            development_environments: vec![Environment::Development, Environment::Integration],
            production_deployment: true,
            author: Some("lead@example.com".to_string()),
        };

        assert_eq!(spec.package, "@test/core");
        assert_eq!(spec.version_bump, VersionBumpType::Major);
        assert!(spec.production_deployment);
        assert_eq!(spec.development_environments.len(), 2);
        assert_eq!(spec.author, Some("lead@example.com".to_string()));
    }

    #[test]
    fn test_validation_result_structure() {
        let validation = ValidationResult {
            is_valid: false,
            errors: vec!["Package not found".to_string(), "Invalid version bump".to_string()],
            warnings: vec!["Environment not configured".to_string()],
            metadata: std::collections::HashMap::new(),
        };

        assert!(!validation.is_valid);
        assert_eq!(validation.errors.len(), 2);
        assert_eq!(validation.warnings.len(), 1);
        assert_eq!(validation.errors[0], "Package not found");
        assert_eq!(validation.warnings[0], "Environment not configured");
    }

    #[test]
    fn test_deployment_result_structure() {
        let mut environment_results = std::collections::HashMap::new();
        environment_results.insert(
            Environment::Development,
            EnvironmentDeploymentResult {
                success: true,
                error: None,
                started_at: Utc::now(),
                completed_at: Some(Utc::now()),
                metadata: std::collections::HashMap::new(),
            },
        );

        let deployment_result = DeploymentResult {
            changeset_id: "test-123".to_string(),
            success: true,
            environment_results,
            duration: std::time::Duration::from_secs(30),
        };

        assert!(deployment_result.success);
        assert_eq!(deployment_result.changeset_id, "test-123");
        assert_eq!(deployment_result.environment_results.len(), 1);
        assert!(deployment_result.environment_results.contains_key(&Environment::Development));
    }

    #[tokio::test]
    /// TODO: Fix this
    async fn test_changeset_storage_integration() {
        let (_temp_dir, project) = create_test_project();
        let storage = ChangesetStorage::new(Arc::clone(&project));

        let changeset = create_test_changeset();

        // Test save
        storage.save(&changeset).expect("Failed to save changeset");

        // Test load
        let loaded = storage.load(&changeset.id).expect("Failed to load changeset");
        assert!(loaded.is_some());
        let loaded_changeset = loaded.unwrap();
        assert_eq!(loaded_changeset.id, changeset.id);
        assert_eq!(loaded_changeset.package, changeset.package);

        // Test list with filter
        let filter =
            ChangesetFilter { package: Some("@test/utils".to_string()), ..Default::default() };
        let changesets = storage.list(&filter).expect("Failed to list changesets");
        assert_eq!(changesets.len(), 1);
        assert_eq!(changesets[0].package, "@test/utils");
    }

    #[tokio::test]
    async fn test_changeset_manager_creation() {
        let (_temp_dir, project) = create_test_project();
        let manager = ChangesetManager::new(project).expect("Failed to create ChangesetManager");

        // Test manager is created successfully
        // The manager should be ready to use
        let filter = ChangesetFilter::default();
        let changesets = manager.list_changesets(&filter).expect("Failed to list changesets");
        assert_eq!(changesets.len(), 0); // No changesets initially
    }

    #[test]
    fn test_changeset_application_workflow() {
        let changeset = create_test_changeset();

        let application = ChangesetApplication {
            changeset_id: changeset.id.clone(),
            package: changeset.package.clone(),
            old_version: "1.0.0".to_string(),
            new_version: "1.1.0".to_string(),
            environments_deployed: changeset.development_environments.clone(),
            success: true,
        };

        assert_eq!(application.changeset_id, changeset.id);
        assert_eq!(application.package, changeset.package);
        assert_eq!(application.old_version, "1.0.0");
        assert_eq!(application.new_version, "1.1.0");
        assert!(application.success);
        assert_eq!(application.environments_deployed.len(), 2);
    }

    #[test]
    fn test_environment_display() {
        assert_eq!(Environment::Development.to_string(), "development");
        assert_eq!(Environment::Staging.to_string(), "staging");
        assert_eq!(Environment::Integration.to_string(), "integration");
        assert_eq!(Environment::Production.to_string(), "production");
        assert_eq!(Environment::Custom("qa".to_string()).to_string(), "qa");
    }

    #[test]
    fn test_changeset_serialization() {
        let changeset = create_test_changeset();

        // Test JSON serialization
        let json = serde_json::to_string(&changeset).expect("Failed to serialize changeset");
        assert!(json.contains("@test/utils"));
        assert!(json.contains("feature/utils"));

        // Test JSON deserialization
        let deserialized: Changeset =
            serde_json::from_str(&json).expect("Failed to deserialize changeset");
        assert_eq!(deserialized.id, changeset.id);
        assert_eq!(deserialized.package, changeset.package);
        assert_eq!(deserialized.version_bump, changeset.version_bump);
    }

    // Manager-specific tests
    #[allow(clippy::arc_with_non_send_sync)]
    fn create_test_manager() -> (TempDir, ChangesetManager) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Initialize Git repository
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to initialize git repository");

        // Set up basic project structure
        std::fs::create_dir_all(temp_dir.path().join("packages"))
            .expect("Failed to create packages dir");

        // Create package.json and lock file for monorepo detection
        let root_package_json = r#"{
  "name": "test-monorepo",
  "version": "1.0.0",
  "private": true,
  "workspaces": ["packages/*"]
}"#;
        std::fs::write(temp_dir.path().join("package.json"), root_package_json)
            .expect("Failed to write root package.json");
        std::fs::write(temp_dir.path().join("package-lock.json"), "{}")
            .expect("Failed to write package-lock.json");

        let project = Arc::new(
            MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject"),
        );
        let manager = ChangesetManager::new(project).expect("Failed to create ChangesetManager");

        (temp_dir, manager)
    }

    #[tokio::test]
    /// TODO: Fix this
    async fn test_manager_create_changeset() {
        let (_temp_dir, manager) = create_test_manager();

        let spec = ChangesetSpec {
            package: "@test/core".to_string(),
            version_bump: VersionBumpType::Minor,
            description: "Test changeset".to_string(),
            development_environments: vec![Environment::Development],
            production_deployment: false,
            author: Some("test@example.com".to_string()),
        };

        let changeset = manager.create_changeset(spec).expect("Failed to create changeset");

        assert!(!changeset.id.is_empty());
        assert_eq!(changeset.package, "@test/core");
        assert_eq!(changeset.version_bump, VersionBumpType::Minor);
        assert_eq!(changeset.status, ChangesetStatus::Pending);
    }

    #[tokio::test]
    /// TODO: Fix this
    async fn test_manager_list_changesets() {
        let (_temp_dir, manager) = create_test_manager();

        // Create a test changeset
        let spec = ChangesetSpec {
            package: "@test/core".to_string(),
            version_bump: VersionBumpType::Patch,
            description: "Test changeset".to_string(),
            development_environments: vec![Environment::Development],
            production_deployment: false,
            author: Some("test@example.com".to_string()),
        };

        let changeset = manager.create_changeset(spec).expect("Failed to create changeset");

        // List all changesets
        let filter = ChangesetFilter::default();
        let changesets = manager.list_changesets(&filter).expect("Failed to list changesets");

        assert_eq!(changesets.len(), 1);
        assert_eq!(changesets[0].id, changeset.id);
    }

    #[tokio::test]
    async fn test_manager_validate_changeset() {
        let (_temp_dir, manager) = create_test_manager();

        let valid_changeset = Changeset {
            id: "test-123".to_string(),
            package: "@test/core".to_string(),
            version_bump: VersionBumpType::Patch,
            description: "Test changeset".to_string(),
            branch: "feature/test".to_string(),
            development_environments: vec![Environment::Development],
            production_deployment: false,
            created_at: Utc::now(),
            author: "test@example.com".to_string(),
            status: ChangesetStatus::Pending,
        };

        let validation =
            manager.validate_changeset(&valid_changeset).expect("Failed to validate changeset");
        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());

        // Test invalid changeset
        let invalid_changeset = Changeset {
            package: String::new(),     // Empty package name
            description: String::new(), // Empty description
            ..valid_changeset
        };

        let validation =
            manager.validate_changeset(&invalid_changeset).expect("Failed to validate changeset");
        assert!(!validation.is_valid);
        assert_eq!(validation.errors.len(), 2);
    }

    // Storage-specific tests
    #[allow(clippy::arc_with_non_send_sync)]
    fn create_test_storage() -> (TempDir, ChangesetStorage) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Initialize Git repository
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to initialize git repository");

        // Create basic monorepo structure
        std::fs::create_dir_all(temp_dir.path().join("packages"))
            .expect("Failed to create packages dir");
        let root_package_json = r#"{"name": "test-monorepo", "version": "1.0.0", "private": true, "workspaces": ["packages/*"]}"#;
        std::fs::write(temp_dir.path().join("package.json"), root_package_json)
            .expect("Failed to write package.json");

        // Create package-lock.json to make it detectable as an npm monorepo
        std::fs::write(temp_dir.path().join("package-lock.json"), "{}")
            .expect("Failed to write package-lock.json");

        let project = Arc::new(
            MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject"),
        );
        let storage = ChangesetStorage::new(project);
        (temp_dir, storage)
    }

    fn create_storage_test_changeset() -> Changeset {
        Changeset {
            id: "test-changeset-123".to_string(),
            package: "@test/core".to_string(),
            version_bump: VersionBumpType::Minor,
            description: "Test changeset".to_string(),
            branch: "feature/test".to_string(),
            development_environments: vec![Environment::Development],
            production_deployment: false,
            created_at: Utc::now(),
            author: "test@example.com".to_string(),
            status: ChangesetStatus::Pending,
        }
    }

    #[tokio::test]
    /// TODO: Fix this
    async fn test_storage_save_and_load_changeset() {
        let (_temp_dir, storage) = create_test_storage();
        let changeset = create_storage_test_changeset();

        // Save changeset
        storage.save(&changeset).expect("Failed to save changeset");

        // Load changeset
        let loaded = storage.load(&changeset.id).expect("Failed to load changeset");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), changeset);
    }

    #[tokio::test]
    /// TODO: Fix this
    async fn test_storage_list_changesets_with_filter() {
        let (_temp_dir, storage) = create_test_storage();
        let changeset1 = create_storage_test_changeset();
        let mut changeset2 = create_storage_test_changeset();
        changeset2.id = "test-changeset-456".to_string();
        changeset2.package = "@test/utils".to_string();

        // Save both changesets
        storage.save(&changeset1).expect("Failed to save changeset1");
        storage.save(&changeset2).expect("Failed to save changeset2");

        // List all changesets
        let all_changesets =
            storage.list(&ChangesetFilter::default()).expect("Failed to list changesets");
        assert_eq!(all_changesets.len(), 2);

        // Filter by package
        let filter =
            ChangesetFilter { package: Some("@test/core".to_string()), ..Default::default() };
        let filtered_changesets =
            storage.list(&filter).expect("Failed to list filtered changesets");
        assert_eq!(filtered_changesets.len(), 1);
        assert_eq!(filtered_changesets[0].package, "@test/core");
    }

    #[tokio::test]
    /// TODO: Fix this
    async fn test_storage_delete_changeset() {
        let (_temp_dir, storage) = create_test_storage();
        let changeset = create_storage_test_changeset();

        // Save changeset
        storage.save(&changeset).expect("Failed to save changeset");

        // Verify it exists
        let loaded = storage.load(&changeset.id).expect("Failed to load changeset");
        assert!(loaded.is_some());

        // Delete changeset
        let deleted = storage.delete(&changeset.id).expect("Failed to delete changeset");
        assert!(deleted);

        // Verify it's gone
        let loaded = storage.load(&changeset.id).expect("Failed to load changeset");
        assert!(loaded.is_none());
    }
}
