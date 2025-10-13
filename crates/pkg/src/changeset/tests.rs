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
