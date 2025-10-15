#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod changeset_tests {
    use crate::{
        changeset::{
            ChangeEntry, ChangeReason, Changeset, ChangesetPackage, EnvironmentRelease, ReleaseInfo,
        },
        Version, VersionBump,
    };
    use std::collections::HashMap;

    #[test]
    fn test_changeset_creation() {
        let changeset = Changeset::default();
        assert!(changeset.branch.is_empty());
        assert!(changeset.packages.is_empty());
        assert!(changeset.release_info.is_none());
    }

    #[test]
    fn test_changeset_new() {
        let changeset = Changeset::new("feat/user-auth".to_string(), "dev@example.com".to_string());

        assert_eq!(changeset.branch, "feat/user-auth");
        assert_eq!(changeset.author, "dev@example.com");
        assert_eq!(changeset.releases, vec!["dev"]);
        assert!(changeset.packages.is_empty());
        assert!(changeset.is_pending());
        assert!(!changeset.is_applied());
    }

    #[test]
    fn test_changeset_generate_id() {
        let changeset = Changeset::new("feat/user-auth".to_string(), "dev@example.com".to_string());

        let id = changeset.generate_id();
        assert!(id.starts_with("feat-user-auth-"));
        assert!(id.ends_with("Z"));
        assert!(!id.contains('/'));
    }

    #[test]
    fn test_changeset_generate_filename() {
        let changeset =
            Changeset::new("bugfix/memory-leak".to_string(), "dev@example.com".to_string());

        let filename = changeset.generate_filename();
        assert!(filename.starts_with("bugfix-memory-leak-"));
        assert!(filename.ends_with(".json"));
    }

    #[test]
    fn test_changeset_sanitize_branch_name() {
        let changeset =
            Changeset::new("feat/user:auth*test?".to_string(), "dev@example.com".to_string());

        let id = changeset.generate_id();
        assert!(id.starts_with("feat-user-auth-test-"));
        assert!(!id.contains('/'));
        assert!(!id.contains(':'));
        assert!(!id.contains('*'));
        assert!(!id.contains('?'));
    }

    #[test]
    fn test_changeset_add_package() {
        let mut changeset = Changeset::new("feat/auth".to_string(), "dev@example.com".to_string());

        let package = ChangesetPackage {
            name: "auth-service".to_string(),
            bump: VersionBump::Minor,
            current_version: Version::new(1, 0, 0).into(),
            next_version: Version::new(1, 1, 0).into(),
            reason: ChangeReason::DirectChanges { commits: vec!["abc123".to_string()] },
            dependency: None,
            changes: vec![],
        };

        changeset.add_package(package);
        assert_eq!(changeset.packages.len(), 1);
        assert_eq!(changeset.packages[0].name, "auth-service");
    }

    #[test]
    fn test_changeset_target_environments() {
        let mut changeset = Changeset::new("feat/auth".to_string(), "dev@example.com".to_string());

        changeset.add_target_environment("staging".to_string());
        changeset.add_target_environment("prod".to_string());

        assert_eq!(changeset.releases.len(), 3);
        assert!(changeset.releases.contains(&"dev".to_string()));
        assert!(changeset.releases.contains(&"staging".to_string()));
        assert!(changeset.releases.contains(&"prod".to_string()));

        // Adding duplicate should not increase count
        changeset.add_target_environment("staging".to_string());
        assert_eq!(changeset.releases.len(), 3);

        // Remove environment
        assert!(changeset.remove_target_environment("staging"));
        assert_eq!(changeset.releases.len(), 2);
        assert!(!changeset.releases.contains(&"staging".to_string()));

        // Remove non-existent environment
        assert!(!changeset.remove_target_environment("non-existent"));
    }

    #[test]
    fn test_changeset_find_package() {
        let mut changeset = Changeset::new("feat/auth".to_string(), "dev@example.com".to_string());

        let package = ChangesetPackage {
            name: "auth-service".to_string(),
            bump: VersionBump::Minor,
            current_version: Version::new(1, 0, 0).into(),
            next_version: Version::new(1, 1, 0).into(),
            reason: ChangeReason::DirectChanges { commits: vec!["abc123".to_string()] },
            dependency: None,
            changes: vec![],
        };

        changeset.add_package(package);

        let found = changeset.find_package("auth-service");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "auth-service");

        let not_found = changeset.find_package("non-existent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_changeset_validation_empty() {
        let changeset = Changeset::default();
        let result = changeset.validate(None);
        assert!(result.is_err());

        if let Err(crate::error::ChangesetError::ValidationFailed { errors, .. }) = result {
            assert!(errors.iter().any(|e| e.contains("Branch name cannot be empty")));
            assert!(errors.iter().any(|e| e.contains("Author cannot be empty")));
            assert!(errors.iter().any(|e| e.contains("At least one package change is required")));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }

    #[test]
    fn test_changeset_validation_valid() {
        let mut changeset = Changeset::new("feat/auth".to_string(), "dev@example.com".to_string());

        let package = ChangesetPackage {
            name: "auth-service".to_string(),
            bump: VersionBump::Minor,
            current_version: Version::new(1, 0, 0).into(),
            next_version: Version::new(1, 1, 0).into(),
            reason: ChangeReason::DirectChanges { commits: vec!["abc123".to_string()] },
            dependency: None,
            changes: vec![ChangeEntry {
                change_type: "feat".to_string(),
                description: "Add authentication".to_string(),
                breaking: false,
                commit: Some("abc123".to_string()),
            }],
        };

        changeset.add_package(package);

        let result = changeset.validate(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_changeset_validation_environments() {
        let mut changeset = Changeset::new("feat/auth".to_string(), "dev@example.com".to_string());

        changeset.add_target_environment("invalid-env".to_string());

        let package = ChangesetPackage {
            name: "auth-service".to_string(),
            bump: VersionBump::Minor,
            current_version: Version::new(1, 0, 0).into(),
            next_version: Version::new(1, 1, 0).into(),
            reason: ChangeReason::DirectChanges { commits: vec!["abc123".to_string()] },
            dependency: None,
            changes: vec![],
        };

        changeset.add_package(package);

        let available_envs = vec!["dev".to_string(), "staging".to_string(), "prod".to_string()];
        let result = changeset.validate(Some(&available_envs));
        assert!(result.is_err());

        if let Err(crate::error::ChangesetError::ValidationFailed { errors, .. }) = result {
            assert!(errors
                .iter()
                .any(|e| e.contains("invalid-env") && e.contains("not in available environments")));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }

    #[test]
    fn test_changeset_get_bump_summary() {
        let mut changeset = Changeset::new("feat/multi".to_string(), "dev@example.com".to_string());

        let package1 = ChangesetPackage {
            name: "pkg1".to_string(),
            bump: VersionBump::Minor,
            current_version: Version::new(1, 0, 0).into(),
            next_version: Version::new(1, 1, 0).into(),
            reason: ChangeReason::DirectChanges { commits: vec!["abc".to_string()] },
            dependency: None,
            changes: vec![],
        };

        let package2 = ChangesetPackage {
            name: "pkg2".to_string(),
            bump: VersionBump::Patch,
            current_version: Version::new(2, 0, 0).into(),
            next_version: Version::new(2, 0, 1).into(),
            reason: ChangeReason::DirectChanges { commits: vec!["def".to_string()] },
            dependency: None,
            changes: vec![],
        };

        let package3 = ChangesetPackage {
            name: "pkg3".to_string(),
            bump: VersionBump::Minor,
            current_version: Version::new(3, 0, 0).into(),
            next_version: Version::new(3, 1, 0).into(),
            reason: ChangeReason::DirectChanges { commits: vec!["ghi".to_string()] },
            dependency: None,
            changes: vec![],
        };

        changeset.add_package(package1);
        changeset.add_package(package2);
        changeset.add_package(package3);

        let summary = changeset.get_bump_summary();
        assert_eq!(summary.get(&VersionBump::Minor), Some(&2));
        assert_eq!(summary.get(&VersionBump::Patch), Some(&1));
        assert_eq!(summary.get(&VersionBump::Major), None);
    }

    #[test]
    fn test_change_reason_equality() {
        let reason1 = ChangeReason::DirectChanges { commits: vec!["abc".to_string()] };
        let reason2 = ChangeReason::DirectChanges { commits: vec!["abc".to_string()] };
        let reason3 = ChangeReason::DirectChanges { commits: vec!["def".to_string()] };

        assert_eq!(reason1, reason2);
        assert_ne!(reason1, reason3);

        let dep_reason1 = ChangeReason::DependencyUpdate {
            dependency: "pkg".to_string(),
            old_version: "1.0.0".to_string(),
            new_version: "1.1.0".to_string(),
        };
        let dep_reason2 = ChangeReason::DependencyUpdate {
            dependency: "pkg".to_string(),
            old_version: "1.0.0".to_string(),
            new_version: "1.1.0".to_string(),
        };

        assert_eq!(dep_reason1, dep_reason2);
        assert_ne!(reason1, dep_reason1);
    }

    #[test]
    fn test_changeset_with_packages() {
        let mut changeset = Changeset::default();
        let package = ChangesetPackage {
            name: "package-a".to_string(),
            bump: VersionBump::Minor,
            current_version: Version::new(1, 0, 0).into(),
            next_version: Version::new(1, 1, 0).into(),
            reason: ChangeReason::DirectChanges { commits: vec!["abc123".to_string()] },
            dependency: None,
            changes: vec![ChangeEntry {
                change_type: "feat".to_string(),
                description: "Added new feature".to_string(),
                breaking: false,
                commit: None,
            }],
        };

        changeset.packages.push(package);

        assert_eq!(changeset.packages.len(), 1);
        assert_eq!(changeset.packages[0].name, "package-a");
        assert_eq!(changeset.packages[0].changes[0].description, "Added new feature");
    }

    #[test]
    fn test_changeset_serialization() {
        let mut changeset = Changeset::default();
        changeset.branch = "feature/test".to_string();
        changeset.author = "Test Author".to_string();

        // Test that we can serialize the changeset
        let json_result = serde_json::to_string(&changeset);
        assert!(json_result.is_ok());

        // Test that we can deserialize it back
        let json_str = json_result.unwrap();
        let deserialized: Result<Changeset, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let deserialized_changeset = deserialized.unwrap();
        assert_eq!(deserialized_changeset.branch, "feature/test");
        assert_eq!(deserialized_changeset.author, "Test Author");
    }

    #[test]
    fn test_changeset_package_creation() {
        let package = ChangesetPackage {
            name: "test-package".to_string(),
            bump: VersionBump::Patch,
            current_version: Version::new(2, 1, 0).into(),
            next_version: Version::new(2, 1, 1).into(),
            reason: ChangeReason::DirectChanges { commits: vec!["def456".to_string()] },
            dependency: None,
            changes: vec![ChangeEntry {
                change_type: "fix".to_string(),
                description: "Fixed critical bug".to_string(),
                breaking: false,
                commit: None,
            }],
        };

        assert_eq!(package.name, "test-package");
        assert_eq!(package.bump, VersionBump::Patch);
        assert_eq!(package.changes[0].description, "Fixed critical bug");
    }

    #[test]
    fn test_release_info_creation() {
        let mut environments = HashMap::new();
        environments.insert(
            "production".to_string(),
            EnvironmentRelease { released_at: chrono::Utc::now(), tag: "v1.2.3".to_string() },
        );

        let release_info = ReleaseInfo {
            applied_at: chrono::Utc::now(),
            applied_by: "deploy-bot".to_string(),
            git_commit: "abc123def456".to_string(),
            environments_released: environments,
        };

        assert_eq!(release_info.applied_by, "deploy-bot");
        assert_eq!(release_info.git_commit, "abc123def456");
        assert!(release_info.environments_released.contains_key("production"));
    }

    #[test]
    fn test_changeset_with_release_info() {
        let mut changeset = Changeset::default();
        let mut environments = HashMap::new();
        environments.insert(
            "staging".to_string(),
            EnvironmentRelease {
                released_at: chrono::Utc::now(),
                tag: "v1.2.3-staging".to_string(),
            },
        );

        let release_info = ReleaseInfo {
            applied_at: chrono::Utc::now(),
            applied_by: "staging-bot".to_string(),
            git_commit: "def456abc789".to_string(),
            environments_released: environments,
        };

        changeset.release_info = Some(release_info);

        assert!(changeset.release_info.is_some());
        let stored_release_info = changeset.release_info.as_ref().unwrap();
        assert_eq!(stored_release_info.applied_by, "staging-bot");
        assert_eq!(stored_release_info.git_commit, "def456abc789");
        assert!(stored_release_info.environments_released.contains_key("staging"));
    }

    #[test]
    fn test_empty_changeset_properties() {
        let changeset = Changeset::default();

        assert!(changeset.branch.is_empty());
        assert!(changeset.author.is_empty());
        assert!(changeset.packages.is_empty());
        assert!(changeset.release_info.is_none());
        assert_eq!(changeset.releases, vec!["dev".to_string()]);
    }

    #[test]
    fn test_change_entry_creation() {
        let change_entry = ChangeEntry {
            change_type: "feat".to_string(),
            description: "Add new API endpoint".to_string(),
            breaking: false,
            commit: None,
        };

        assert_eq!(change_entry.change_type, "feat");
        assert_eq!(change_entry.description, "Add new API endpoint");
        assert!(!change_entry.breaking);
    }

    #[test]
    fn test_change_entry_breaking() {
        let breaking_change = ChangeEntry {
            change_type: "feat".to_string(),
            description: "Remove deprecated API".to_string(),
            breaking: true,
            commit: None,
        };

        assert!(breaking_change.breaking);
    }

    #[test]
    fn test_changeset_package_with_dependency() {
        let package = ChangesetPackage {
            name: "dependent-package".to_string(),
            bump: VersionBump::Patch,
            current_version: Version::new(1, 0, 0).into(),
            next_version: Version::new(1, 0, 1).into(),
            reason: ChangeReason::DependencyUpdate {
                dependency: "updated-dep".to_string(),
                old_version: "1.0.0".to_string(),
                new_version: "1.1.0".to_string(),
            },
            dependency: Some("updated-dep".to_string()),
            changes: vec![],
        };

        assert_eq!(package.dependency, Some("updated-dep".to_string()));
        assert!(matches!(package.reason, ChangeReason::DependencyUpdate { .. }));
    }

    #[test]
    fn test_environment_release() {
        let env_release =
            EnvironmentRelease { released_at: chrono::Utc::now(), tag: "v2.0.0".to_string() };

        assert_eq!(env_release.tag, "v2.0.0");
    }
}

// Storage and Manager Tests Module
#[allow(clippy::unwrap_used)]
mod storage_and_manager_tests {
    use crate::changeset::{
        ChangeEntry, ChangeReason, Changeset, ChangesetManager, ChangesetPackage, ChangesetStorage,
        ChangesetSummary, EnvironmentRelease, FileBasedChangesetStorage, ReleaseInfo,
    };
    use crate::config::ChangesetConfig;
    use crate::version::{Version, VersionBump};
    use std::collections::HashMap;
    use tempfile::TempDir;

    // Helper functions for test setup
    fn create_test_storage(
    ) -> (FileBasedChangesetStorage<sublime_standard_tools::filesystem::FileSystemManager>, TempDir)
    {
        let temp_dir = TempDir::new().unwrap();
        let config = ChangesetConfig::default();
        let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
        let storage = FileBasedChangesetStorage::new(fs, temp_dir.path().to_path_buf(), config);
        (storage, temp_dir)
    }

    fn create_test_manager() -> (
        ChangesetManager<
            FileBasedChangesetStorage<sublime_standard_tools::filesystem::FileSystemManager>,
        >,
        TempDir,
    ) {
        let temp_dir = TempDir::new().unwrap();
        let config = ChangesetConfig::default();
        let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
        let storage = FileBasedChangesetStorage::new(fs, temp_dir.path().to_path_buf(), config);
        let manager = ChangesetManager::new(storage);
        (manager, temp_dir)
    }

    fn create_test_changeset(branch: &str) -> Changeset {
        let mut changeset = Changeset::new(branch.to_string(), "test@example.com".to_string());
        changeset.add_target_environment("dev".to_string());

        let package = ChangesetPackage {
            name: "@test/package".to_string(),
            bump: VersionBump::Minor,
            current_version: Version::new(1, 0, 0).into(),
            next_version: Version::new(1, 1, 0).into(),
            reason: ChangeReason::DirectChanges { commits: vec!["abc123".to_string()] },
            dependency: None,
            changes: vec![ChangeEntry {
                change_type: "feat".to_string(),
                description: "Add feature".to_string(),
                breaking: false,
                commit: Some("abc123".to_string()),
            }],
        };

        changeset.add_package(package);
        changeset
    }

    fn create_release_info() -> ReleaseInfo {
        let mut environments = HashMap::new();
        environments.insert(
            "dev".to_string(),
            EnvironmentRelease { released_at: chrono::Utc::now(), tag: "v1.1.0-dev".to_string() },
        );

        ReleaseInfo {
            applied_at: chrono::Utc::now(),
            applied_by: "test".to_string(),
            git_commit: "abc1234".to_string(),
            environments_released: environments,
        }
    }

    // Storage Tests
    #[tokio::test]
    async fn test_storage_save_and_load() {
        let (storage, _temp) = create_test_storage();
        let changeset = create_test_changeset("feat/test");

        let id = storage.save(&changeset).await.unwrap();
        let loaded = storage.load(&id).await.unwrap();

        assert_eq!(loaded.branch, "feat/test");
        assert_eq!(loaded.author, "test@example.com");
        assert_eq!(loaded.packages.len(), 1);
    }

    #[tokio::test]
    async fn test_storage_save_duplicate_fails() {
        let (storage, _temp) = create_test_storage();
        let changeset = create_test_changeset("feat/test");

        storage.save(&changeset).await.unwrap();
        let result = storage.save(&changeset).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::error::ChangesetError::AlreadyExists { .. }));
    }

    #[tokio::test]
    async fn test_storage_exists() {
        let (storage, _temp) = create_test_storage();
        let changeset = create_test_changeset("feat/test");

        let id = storage.save(&changeset).await.unwrap();

        assert!(storage.exists(&id).await.unwrap());
        assert!(!storage.exists("nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_storage_delete() {
        let (storage, _temp) = create_test_storage();
        let changeset = create_test_changeset("feat/test");

        let id = storage.save(&changeset).await.unwrap();
        storage.delete(&id).await.unwrap();

        assert!(!storage.exists(&id).await.unwrap());
    }

    #[tokio::test]
    async fn test_storage_delete_nonexistent_fails() {
        let (storage, _temp) = create_test_storage();

        let result = storage.delete("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_storage_list_pending() {
        let (storage, _temp) = create_test_storage();

        let cs1 = create_test_changeset("feat/one");
        let cs2 = create_test_changeset("feat/two");

        storage.save(&cs1).await.unwrap();
        storage.save(&cs2).await.unwrap();

        let pending = storage.list_pending().await.unwrap();
        assert_eq!(pending.len(), 2);
    }

    #[tokio::test]
    async fn test_storage_list_pending_empty() {
        let (storage, _temp) = create_test_storage();

        let pending = storage.list_pending().await.unwrap();
        assert_eq!(pending.len(), 0);
    }

    #[tokio::test]
    async fn test_storage_list_history_empty() {
        let (storage, _temp) = create_test_storage();

        let history = storage.list_history().await.unwrap();
        assert_eq!(history.len(), 0);
    }

    #[tokio::test]
    async fn test_storage_archive() {
        let (storage, _temp) = create_test_storage();
        let mut changeset = create_test_changeset("feat/test");
        let release_info = create_release_info();
        changeset.apply_release_info(release_info);

        let id = storage.save(&changeset).await.unwrap();
        storage.archive(&id).await.unwrap();

        // Should be in history
        let history = storage.list_history().await.unwrap();
        assert_eq!(history.len(), 1);

        // Should not be in pending
        let pending = storage.list_pending().await.unwrap();
        assert_eq!(pending.len(), 0);
    }

    #[tokio::test]
    async fn test_storage_archive_without_release_info_fails() {
        let (storage, _temp) = create_test_storage();
        let changeset = create_test_changeset("feat/test");

        let id = storage.save(&changeset).await.unwrap();
        let result = storage.archive(&id).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_storage_load_from_history() {
        let (storage, _temp) = create_test_storage();
        let mut changeset = create_test_changeset("feat/test");
        let release_info = create_release_info();
        changeset.apply_release_info(release_info);

        let id = storage.save(&changeset).await.unwrap();
        storage.archive(&id).await.unwrap();

        let loaded = storage.load_from_history(&id).await.unwrap();
        assert!(loaded.is_applied());
    }

    #[tokio::test]
    async fn test_storage_query_by_branch() {
        let (storage, _temp) = create_test_storage();

        let cs1 = create_test_changeset("feat/auth");
        let cs2 = create_test_changeset("feat/other");

        storage.save(&cs1).await.unwrap();
        storage.save(&cs2).await.unwrap();

        let results = storage.query_by_branch("feat/auth").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_storage_query_by_branch_no_matches() {
        let (storage, _temp) = create_test_storage();

        let results = storage.query_by_branch("feat/nonexistent").await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_storage_query_history_by_date() {
        let (storage, _temp) = create_test_storage();
        let mut changeset = create_test_changeset("feat/test");
        let release_info = create_release_info();
        changeset.apply_release_info(release_info);

        let id = storage.save(&changeset).await.unwrap();
        storage.archive(&id).await.unwrap();

        let start = chrono::Utc::now() - chrono::Duration::days(1);
        let end = chrono::Utc::now() + chrono::Duration::days(1);

        let results = storage.query_history_by_date(start, end).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_storage_query_history_by_package() {
        let (storage, _temp) = create_test_storage();
        let mut changeset = create_test_changeset("feat/test");
        let release_info = create_release_info();
        changeset.apply_release_info(release_info);

        let id = storage.save(&changeset).await.unwrap();
        storage.archive(&id).await.unwrap();

        let results = storage.query_history_by_package("@test/package").await.unwrap();
        assert_eq!(results.len(), 1);

        let results = storage.query_history_by_package("@test/other").await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_storage_get_latest_for_branch() {
        let (storage, _temp) = create_test_storage();

        let cs = create_test_changeset("feat/test");
        let id = storage.save(&cs).await.unwrap();

        let latest = storage.get_latest_for_branch("feat/test").await.unwrap();
        assert_eq!(latest, Some(id));
    }

    #[tokio::test]
    async fn test_storage_get_latest_for_branch_none() {
        let (storage, _temp) = create_test_storage();

        let latest = storage.get_latest_for_branch("feat/nonexistent").await.unwrap();
        assert_eq!(latest, None);
    }

    // Manager Tests
    #[tokio::test]
    async fn test_manager_create_and_load() {
        let (manager, _temp) = create_test_manager();
        let changeset = create_test_changeset("feat/test");

        let id = manager.create(&changeset).await.unwrap();
        let loaded = manager.load(&id).await.unwrap();

        assert_eq!(loaded.branch, "feat/test");
        assert!(!loaded.is_applied());
    }

    #[tokio::test]
    async fn test_manager_create_empty_packages_fails() {
        let (manager, _temp) = create_test_manager();
        let changeset = Changeset::new("feat/test".to_string(), "test@example.com".to_string());

        let result = manager.create(&changeset).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_manager_create_already_applied_fails() {
        let (manager, _temp) = create_test_manager();
        let mut changeset = create_test_changeset("feat/test");
        let release_info = create_release_info();
        changeset.apply_release_info(release_info);

        let result = manager.create(&changeset).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_manager_exists() {
        let (manager, _temp) = create_test_manager();
        let changeset = create_test_changeset("feat/test");

        let id = manager.create(&changeset).await.unwrap();

        assert!(manager.exists(&id).await.unwrap());
        assert!(!manager.exists("nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_manager_delete() {
        let (manager, _temp) = create_test_manager();
        let changeset = create_test_changeset("feat/test");

        let id = manager.create(&changeset).await.unwrap();
        manager.delete(&id).await.unwrap();

        assert!(!manager.exists(&id).await.unwrap());
    }

    #[tokio::test]
    async fn test_manager_list_pending() {
        let (manager, _temp) = create_test_manager();

        let cs1 = create_test_changeset("feat/one");
        let cs2 = create_test_changeset("feat/two");

        manager.create(&cs1).await.unwrap();
        manager.create(&cs2).await.unwrap();

        let pending = manager.list_pending().await.unwrap();
        assert_eq!(pending.len(), 2);
    }

    #[tokio::test]
    async fn test_manager_list_history() {
        let (manager, _temp) = create_test_manager();
        let changeset = create_test_changeset("feat/test");

        let id = manager.create(&changeset).await.unwrap();
        let release_info = create_release_info();
        manager.apply(&id, release_info).await.unwrap();

        let history = manager.list_history().await.unwrap();
        assert_eq!(history.len(), 1);
    }

    #[tokio::test]
    async fn test_manager_apply_and_archive() {
        let (manager, _temp) = create_test_manager();
        let changeset = create_test_changeset("feat/test");

        let id = manager.create(&changeset).await.unwrap();
        let release_info = create_release_info();
        manager.apply(&id, release_info).await.unwrap();

        // Should be in history now
        let history = manager.list_history().await.unwrap();
        assert_eq!(history.len(), 1);

        // Should not be in pending
        let pending = manager.list_pending().await.unwrap();
        assert_eq!(pending.len(), 0);
    }

    #[tokio::test]
    async fn test_manager_apply_already_applied_fails() {
        let (manager, _temp) = create_test_manager();
        let changeset = create_test_changeset("feat/test");

        let id = manager.create(&changeset).await.unwrap();
        let release_info = create_release_info();
        manager.apply(&id, release_info.clone()).await.unwrap();

        let result = manager.apply(&id, release_info).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_manager_load_from_history() {
        let (manager, _temp) = create_test_manager();
        let changeset = create_test_changeset("feat/test");

        let id = manager.create(&changeset).await.unwrap();
        let release_info = create_release_info();
        manager.apply(&id, release_info).await.unwrap();

        let loaded = manager.load_from_history(&id).await.unwrap();
        assert!(loaded.is_applied());
    }

    #[tokio::test]
    async fn test_manager_query_by_branch() {
        let (manager, _temp) = create_test_manager();

        let cs1 = create_test_changeset("feat/auth");
        let cs2 = create_test_changeset("feat/other");

        manager.create(&cs1).await.unwrap();
        manager.create(&cs2).await.unwrap();

        let results = manager.query_by_branch("feat/auth").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_manager_query_history_by_date() {
        let (manager, _temp) = create_test_manager();
        let changeset = create_test_changeset("feat/test");

        let id = manager.create(&changeset).await.unwrap();
        let release_info = create_release_info();
        manager.apply(&id, release_info).await.unwrap();

        let start = chrono::Utc::now() - chrono::Duration::days(1);
        let end = chrono::Utc::now() + chrono::Duration::days(1);

        let results = manager.query_history_by_date(start, end).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_manager_query_history_by_package() {
        let (manager, _temp) = create_test_manager();
        let changeset = create_test_changeset("feat/test");

        let id = manager.create(&changeset).await.unwrap();
        let release_info = create_release_info();
        manager.apply(&id, release_info).await.unwrap();

        let results = manager.query_history_by_package("@test/package").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_manager_get_latest_for_branch() {
        let (manager, _temp) = create_test_manager();

        let cs = create_test_changeset("feat/test");
        let id = manager.create(&cs).await.unwrap();

        let latest = manager.get_latest_for_branch("feat/test").await.unwrap();
        assert_eq!(latest, Some(id));

        let none = manager.get_latest_for_branch("feat/nonexistent").await.unwrap();
        assert_eq!(none, None);
    }

    #[tokio::test]
    async fn test_manager_get_summary() {
        let (manager, _temp) = create_test_manager();

        let cs1 = create_test_changeset("feat/one");
        let cs2 = create_test_changeset("feat/two");

        manager.create(&cs1).await.unwrap();
        manager.create(&cs2).await.unwrap();

        let summary = manager.get_summary().await.unwrap();
        assert_eq!(summary.pending_count, 2);
        assert_eq!(summary.history_count, 0);
        assert!(summary.has_pending());
        assert!(!summary.has_history());
        assert_eq!(summary.total_count(), 2);
    }

    #[tokio::test]
    async fn test_manager_get_summary_with_history() {
        let (manager, _temp) = create_test_manager();

        let cs1 = create_test_changeset("feat/one");
        let cs2 = create_test_changeset("feat/two");

        let id1 = manager.create(&cs1).await.unwrap();
        manager.create(&cs2).await.unwrap();

        // Apply one changeset
        let release_info = create_release_info();
        manager.apply(&id1, release_info).await.unwrap();

        let summary = manager.get_summary().await.unwrap();
        assert_eq!(summary.pending_count, 1);
        assert_eq!(summary.history_count, 1);
        assert!(summary.has_pending());
        assert!(summary.has_history());
        assert_eq!(summary.total_count(), 2);
    }

    #[test]
    fn test_changeset_summary_methods() {
        let summary = ChangesetSummary {
            pending_count: 5,
            history_count: 10,
            pending_ids: vec!["id1".to_string()],
            history_ids: vec!["id2".to_string()],
        };

        assert!(summary.has_pending());
        assert!(summary.has_history());
        assert_eq!(summary.total_count(), 15);
    }

    #[test]
    fn test_changeset_summary_empty() {
        let summary = ChangesetSummary {
            pending_count: 0,
            history_count: 0,
            pending_ids: vec![],
            history_ids: vec![],
        };

        assert!(!summary.has_pending());
        assert!(!summary.has_history());
        assert_eq!(summary.total_count(), 0);
    }
}

// Tests for PackageChangeDetector
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod detector_tests {
    use crate::changeset::PackageChangeDetector;
    use std::path::{Path, PathBuf};
    use sublime_standard_tools::filesystem::FileSystemManager;
    use tempfile::TempDir;

    fn create_test_detector(root: &Path) -> PackageChangeDetector<FileSystemManager> {
        PackageChangeDetector::new(root.to_path_buf(), FileSystemManager::new())
    }

    async fn create_test_package_json(
        dir: &Path,
        name: &str,
        version: &str,
    ) -> Result<(), std::io::Error> {
        let content = format!(
            r#"{{
  "name": "{}",
  "version": "{}",
  "description": "Test package"
}}"#,
            name, version
        );
        tokio::fs::write(dir.join("package.json"), content).await
    }

    #[tokio::test]
    async fn test_detector_creation() {
        let temp_dir = TempDir::new().unwrap();
        let detector = create_test_detector(temp_dir.path());

        assert_eq!(detector.workspace_root, temp_dir.path());
    }

    #[tokio::test]
    async fn test_detect_empty_files() {
        let temp_dir = TempDir::new().unwrap();
        let detector = create_test_detector(temp_dir.path());

        let result = detector.detect_affected_packages(&[]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_is_monorepo_single_package() {
        let temp_dir = TempDir::new().unwrap();
        create_test_package_json(temp_dir.path(), "@test/single", "1.0.0").await.unwrap();

        let detector = create_test_detector(temp_dir.path());
        let is_mono = detector.is_monorepo().await.unwrap();

        // Single package.json at root = not a monorepo
        assert!(!is_mono);
    }

    #[tokio::test]
    async fn test_detect_single_package() {
        let temp_dir = TempDir::new().unwrap();
        create_test_package_json(temp_dir.path(), "@test/single", "1.0.0").await.unwrap();

        let src_dir = temp_dir.path().join("src");
        tokio::fs::create_dir_all(&src_dir).await.unwrap();
        tokio::fs::write(src_dir.join("index.ts"), "console.log('test');").await.unwrap();

        let detector = create_test_detector(temp_dir.path());
        let changed_files = vec![temp_dir.path().join("src/index.ts")];

        let result = detector.detect_affected_packages(&changed_files).await.unwrap();

        assert_eq!(result.len(), 1);
        assert!(result.contains_key("@test/single"));
        assert_eq!(result["@test/single"].len(), 1);
    }

    #[tokio::test]
    async fn test_list_all_packages_single() {
        let temp_dir = TempDir::new().unwrap();
        create_test_package_json(temp_dir.path(), "@test/single", "1.0.0").await.unwrap();

        let detector = create_test_detector(temp_dir.path());
        let packages = detector.list_all_packages().await.unwrap();

        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].metadata.name, "@test/single");
        assert_eq!(packages[0].metadata.version.to_string(), "1.0.0");
    }

    #[tokio::test]
    async fn test_get_package_for_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let detector = create_test_detector(temp_dir.path());

        let result =
            detector.get_package_for_file(&PathBuf::from("nonexistent/file.ts")).await.unwrap();
        assert!(result.is_none());
    }
}

// Tests for ChangesetBuilder
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod builder_tests {
    use crate::changeset::{ChangeReason, ChangesetBuilder};
    use crate::config::PackageToolsConfig;
    use crate::conventional::ConventionalCommitService;
    use crate::version::VersionBump;
    use sublime_git_tools::Repo;
    use tempfile::TempDir;

    struct TestRepoWithService {
        _temp_dir: TempDir,
        repo: Repo,
        commit_service: ConventionalCommitService,
    }

    async fn create_test_git_repo() -> Result<TestRepoWithService, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().to_str().ok_or("Invalid path")?;
        let repo = Repo::create(repo_path)?;

        // Create initial package.json
        let package_json = r#"{
  "name": "@test/package",
  "version": "1.0.0",
  "description": "Test package"
}"#;
        tokio::fs::write(temp_dir.path().join("package.json"), package_json).await?;

        // Create a file to commit
        tokio::fs::write(temp_dir.path().join("README.md"), "# Test").await?;

        // Initial commit
        repo.add_all()?;
        repo.commit("chore: initial commit")?;

        // Create commit service
        let config = PackageToolsConfig::default();
        let commit_service = ConventionalCommitService::new(repo, config)?;

        let repo_path = temp_dir.path().to_str().ok_or("Invalid path")?;
        let repo = Repo::open(repo_path)?;

        Ok(TestRepoWithService { _temp_dir: temp_dir, repo, commit_service })
    }

    #[tokio::test]
    async fn test_builder_creation() {
        let test_setup = create_test_git_repo().await.unwrap();
        let config = PackageToolsConfig::default();

        let builder = ChangesetBuilder::new(
            &test_setup.repo,
            &test_setup.commit_service,
            test_setup.repo.get_repo_path().to_path_buf(),
            config,
        );
        assert!(builder.is_ok());
    }

    #[tokio::test]
    async fn test_builder_no_changes_detected() {
        let test_setup = create_test_git_repo().await.unwrap();
        let config = PackageToolsConfig::default();

        let builder = ChangesetBuilder::new(
            &test_setup.repo,
            &test_setup.commit_service,
            test_setup.repo.get_repo_path().to_path_buf(),
            config,
        )
        .unwrap();

        // No commits since the initial commit
        let result = builder
            .from_commits_since(None, "test@example.com".to_string(), vec!["prod".to_string()])
            .await;

        // Should return Ok with empty or no package changes when no relevant changes
        // (initial commit exists but may not result in version bumps)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_builder_from_commits_with_changes() {
        let mut test_setup = create_test_git_repo().await.unwrap();

        // Add a new file
        let repo_path = test_setup.repo.get_repo_path().to_path_buf();
        let src_dir = repo_path.join("src");
        tokio::fs::create_dir_all(&src_dir).await.unwrap();
        tokio::fs::write(src_dir.join("index.ts"), "export const test = 1;").await.unwrap();
        test_setup.repo.add_all().unwrap();
        test_setup.repo.commit("feat: add new feature").unwrap();

        // Need to recreate service after new commits
        let config = PackageToolsConfig::default();
        test_setup.commit_service =
            ConventionalCommitService::new(test_setup.repo, config.clone()).unwrap();
        test_setup.repo = Repo::open(repo_path.to_str().unwrap()).unwrap();

        let builder = ChangesetBuilder::new(
            &test_setup.repo,
            &test_setup.commit_service,
            repo_path.clone(),
            config,
        )
        .unwrap();

        let changeset = builder
            .from_commits_since(None, "test@example.com".to_string(), vec!["dev".to_string()])
            .await
            .unwrap();

        // Verify changeset structure
        assert_eq!(changeset.author, "test@example.com");
        assert_eq!(changeset.releases, vec!["dev".to_string()]);
        assert!(!changeset.packages.is_empty());

        // Should have detected the package
        let package = &changeset.packages[0];
        assert_eq!(package.name, "@test/package");
        // Version comparison needs to account for ResolvedVersion
        assert_eq!(package.bump, VersionBump::Minor); // feat = minor
    }

    #[tokio::test]
    async fn test_builder_breaking_change() {
        let mut test_setup = create_test_git_repo().await.unwrap();

        // Add a breaking change
        let repo_path = test_setup.repo.get_repo_path().to_path_buf();
        let src_dir = repo_path.join("src");
        tokio::fs::create_dir_all(&src_dir).await.unwrap();
        tokio::fs::write(src_dir.join("api.ts"), "export const breaking = true;").await.unwrap();
        test_setup.repo.add_all().unwrap();
        test_setup.repo.commit("feat!: breaking change").unwrap();

        let config = PackageToolsConfig::default();
        test_setup.commit_service =
            ConventionalCommitService::new(test_setup.repo, config.clone()).unwrap();
        test_setup.repo = Repo::open(repo_path.to_str().unwrap()).unwrap();

        let builder =
            ChangesetBuilder::new(&test_setup.repo, &test_setup.commit_service, repo_path, config)
                .unwrap();

        let changeset = builder
            .from_commits_since(None, "test@example.com".to_string(), vec!["prod".to_string()])
            .await
            .unwrap();

        // Breaking change should result in major bump
        let package = &changeset.packages[0];
        assert_eq!(package.bump, VersionBump::Major);
        assert!(package.changes.iter().any(|c| c.breaking));
    }

    #[tokio::test]
    async fn test_builder_fix_commit() {
        let mut test_setup = create_test_git_repo().await.unwrap();

        // Add a fix
        let repo_path = test_setup.repo.get_repo_path().to_path_buf();
        let src_dir = repo_path.join("src");
        tokio::fs::create_dir_all(&src_dir).await.unwrap();
        tokio::fs::write(src_dir.join("bug.ts"), "export const fixed = true;").await.unwrap();
        test_setup.repo.add_all().unwrap();
        test_setup.repo.commit("fix: resolve critical bug").unwrap();

        let config = PackageToolsConfig::default();
        test_setup.commit_service =
            ConventionalCommitService::new(test_setup.repo, config.clone()).unwrap();
        test_setup.repo = Repo::open(repo_path.to_str().unwrap()).unwrap();

        let builder =
            ChangesetBuilder::new(&test_setup.repo, &test_setup.commit_service, repo_path, config)
                .unwrap();

        let changeset = builder
            .from_commits_since(None, "test@example.com".to_string(), vec!["dev".to_string()])
            .await
            .unwrap();

        // Fix should result in patch bump
        let package = &changeset.packages[0];
        assert_eq!(package.bump, VersionBump::Patch);
        assert!(!package.changes.is_empty());
    }

    #[tokio::test]
    async fn test_builder_multiple_commits() {
        let mut test_setup = create_test_git_repo().await.unwrap();

        // Add multiple commits
        let repo_path = test_setup.repo.get_repo_path().to_path_buf();
        let src_dir = repo_path.join("src");
        tokio::fs::create_dir_all(&src_dir).await.unwrap();
        tokio::fs::write(src_dir.join("feature1.ts"), "export const f1 = 1;").await.unwrap();
        test_setup.repo.add_all().unwrap();
        test_setup.repo.commit("feat: add feature 1").unwrap();

        tokio::fs::write(src_dir.join("feature2.ts"), "export const f2 = 2;").await.unwrap();
        test_setup.repo.add_all().unwrap();
        test_setup.repo.commit("feat: add feature 2").unwrap();

        tokio::fs::write(src_dir.join("fix.ts"), "export const fix = true;").await.unwrap();
        test_setup.repo.add_all().unwrap();
        test_setup.repo.commit("fix: resolve issue").unwrap();

        let config = PackageToolsConfig::default();
        test_setup.commit_service =
            ConventionalCommitService::new(test_setup.repo, config.clone()).unwrap();
        test_setup.repo = Repo::open(repo_path.to_str().unwrap()).unwrap();

        let builder =
            ChangesetBuilder::new(&test_setup.repo, &test_setup.commit_service, repo_path, config)
                .unwrap();

        let changeset = builder
            .from_commits_since(None, "test@example.com".to_string(), vec!["dev".to_string()])
            .await
            .unwrap();

        // Should have detected multiple changes
        let package = &changeset.packages[0];

        // Debug output
        eprintln!("Package changes count: {}", package.changes.len());
        for change in &package.changes {
            eprintln!("  - {:?}: {}", change.change_type, change.description);
        }
        eprintln!("Calculated bump: {:?}", package.bump);

        assert_eq!(package.bump, VersionBump::Minor); // feat > fix
                                                      // Includes initial commit + feature 1 + feature 2 + fix = 4 commits
                                                      // But may include more depending on how commits are counted
        assert!(package.changes.len() >= 3);

        // Verify the new commits are included
        assert!(package.changes.iter().any(|c| c.description.contains("feature 1")));
        assert!(package.changes.iter().any(|c| c.description.contains("feature 2")));
        assert!(package.changes.iter().any(|c| c.description.contains("resolve issue")));
    }

    #[tokio::test]
    async fn test_builder_with_multiple_environments() {
        let mut test_setup = create_test_git_repo().await.unwrap();

        // Add a change
        let repo_path = test_setup.repo.get_repo_path().to_path_buf();
        let src_dir = repo_path.join("src");
        tokio::fs::create_dir_all(&src_dir).await.unwrap();
        tokio::fs::write(src_dir.join("new.ts"), "export const new = true;").await.unwrap();
        test_setup.repo.add_all().unwrap();
        test_setup.repo.commit("feat: add new module").unwrap();

        let config = PackageToolsConfig::default();
        test_setup.commit_service =
            ConventionalCommitService::new(test_setup.repo, config.clone()).unwrap();
        test_setup.repo = Repo::open(repo_path.to_str().unwrap()).unwrap();

        let builder =
            ChangesetBuilder::new(&test_setup.repo, &test_setup.commit_service, repo_path, config)
                .unwrap();

        let changeset = builder
            .from_commits_since(
                None,
                "ci-bot@example.com".to_string(),
                vec!["dev".to_string(), "qa".to_string(), "staging".to_string()],
            )
            .await
            .unwrap();

        // Verify multiple environments
        assert_eq!(changeset.releases.len(), 3);
        assert!(changeset.releases.contains(&"dev".to_string()));
        assert!(changeset.releases.contains(&"qa".to_string()));
        assert!(changeset.releases.contains(&"staging".to_string()));
    }

    #[tokio::test]
    async fn test_builder_change_reason_tracking() {
        let mut test_setup = create_test_git_repo().await.unwrap();

        // Add commits
        let repo_path = test_setup.repo.get_repo_path().to_path_buf();
        let src_dir = repo_path.join("src");
        tokio::fs::create_dir_all(&src_dir).await.unwrap();
        tokio::fs::write(src_dir.join("code.ts"), "export const code = 1;").await.unwrap();
        test_setup.repo.add_all().unwrap();
        test_setup.repo.commit("feat: add code").unwrap();

        let config = PackageToolsConfig::default();
        test_setup.commit_service =
            ConventionalCommitService::new(test_setup.repo, config.clone()).unwrap();
        test_setup.repo = Repo::open(repo_path.to_str().unwrap()).unwrap();

        let builder =
            ChangesetBuilder::new(&test_setup.repo, &test_setup.commit_service, repo_path, config)
                .unwrap();

        let changeset = builder
            .from_commits_since(None, "dev@example.com".to_string(), vec!["dev".to_string()])
            .await
            .unwrap();

        let package = &changeset.packages[0];

        // Verify reason is DirectChanges
        match &package.reason {
            ChangeReason::DirectChanges { commits } => {
                // Commits contains commit hashes, not messages
                // Should have at least the commits we added
                assert!(commits.len() >= 1);
                // Verify we have commit hashes (non-empty strings)
                assert!(commits.iter().all(|c| !c.is_empty()));
            }
            _ => panic!("Expected DirectChanges reason"),
        }
    }
}
