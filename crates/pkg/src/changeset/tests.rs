#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod changeset_tests {
    use crate::{
        changeset::{
            ChangeEntry, ChangeReason, Changeset, ChangesetManager, ChangesetPackage,
            EnvironmentRelease, ReleaseInfo,
        },
        Version, VersionBump,
    };
    use std::{collections::HashMap, path::PathBuf};

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
    fn test_changeset_manager_creation() {
        let manager = ChangesetManager::new(
            PathBuf::from(".changesets"),
            PathBuf::from(".changesets/history"),
        );
        assert_eq!(manager.changeset_path, PathBuf::from(".changesets"));
        assert_eq!(manager.history_path, PathBuf::from(".changesets/history"));
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
    fn test_changeset_manager_paths() {
        let changeset_path = PathBuf::from("/custom/changesets");
        let history_path = PathBuf::from("/custom/history");

        let manager = ChangesetManager::new(changeset_path.clone(), history_path.clone());

        assert_eq!(manager.changeset_path, changeset_path);
        assert_eq!(manager.history_path, history_path);
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
