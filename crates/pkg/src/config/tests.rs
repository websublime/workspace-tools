#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod config_tests {
    use crate::config::{
        ChangelogConfig, ChangesetConfig, ConventionalConfig, DependencyConfig, PackageToolsConfig,
        RegistryConfig, ReleaseConfig, VersionConfig,
    };
    use std::path::PathBuf;
    use sublime_standard_tools::config::Configurable;

    #[test]
    fn test_default_config_creation() {
        let config = PackageToolsConfig::default();

        assert_eq!(config.changeset.path, PathBuf::from(".changesets"));
        assert_eq!(config.version.commit_hash_length, 7);
        assert_eq!(config.release.strategy, "independent");
        assert_eq!(config.dependency.dependency_update_bump, "patch");
    }

    #[test]
    fn test_config_validation() {
        let config = PackageToolsConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = PackageToolsConfig::default();
        invalid_config.changeset.available_environments.clear();
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_bump_type_lookup() {
        let config = PackageToolsConfig::default();

        assert_eq!(config.get_bump_type("feat"), "minor");
        assert_eq!(config.get_bump_type("fix"), "patch");
        assert_eq!(config.get_bump_type("unknown"), "patch"); // default
    }

    #[test]
    fn test_changelog_inclusion() {
        let config = PackageToolsConfig::default();

        assert!(config.should_include_in_changelog("feat"));
        assert!(config.should_include_in_changelog("fix"));
        assert!(!config.should_include_in_changelog("docs"));
        assert!(!config.should_include_in_changelog("unknown"));
    }

    #[test]
    fn test_environment_validation() {
        let config = PackageToolsConfig::default();

        assert!(config.is_environment_available("dev"));
        assert!(config.is_environment_available("prod"));
        assert!(!config.is_environment_available("unknown"));
    }

    #[test]
    fn test_version_config_validation() {
        let mut config = PackageToolsConfig::default();
        config.version.commit_hash_length = 0;
        assert!(config.validate().is_err());

        config.version.commit_hash_length = 50;
        assert!(config.validate().is_err());

        config.version.commit_hash_length = 7;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_release_strategy_validation() {
        let mut config = PackageToolsConfig::default();

        config.release.strategy = "invalid".to_string();
        assert!(config.validate().is_err());

        config.release.strategy = "independent".to_string();
        assert!(config.validate().is_ok());

        config.release.strategy = "unified".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_changeset_config_properties() {
        let changeset_config = ChangesetConfig::default();

        assert_eq!(changeset_config.path, PathBuf::from(".changesets"));
        assert_eq!(changeset_config.history_path, PathBuf::from(".changesets/history"));
        assert!(changeset_config.auto_archive_applied);
        assert_eq!(
            changeset_config.available_environments,
            vec!["dev", "test", "qa", "staging", "prod"]
        );
    }

    #[test]
    fn test_version_config_properties() {
        let version_config = VersionConfig::default();

        assert_eq!(version_config.commit_hash_length, 7);
    }

    #[test]
    fn test_release_config_properties() {
        let release_config = ReleaseConfig::default();

        assert_eq!(release_config.strategy, "independent");
    }

    #[test]
    fn test_dependency_config_properties() {
        let dependency_config = DependencyConfig::default();

        assert_eq!(dependency_config.dependency_update_bump, "patch");
    }

    #[test]
    fn test_conventional_config_properties() {
        let conventional_config = ConventionalConfig::default();

        assert!(conventional_config.types.contains_key("feat"));
        assert!(conventional_config.types.contains_key("fix"));
        assert_eq!(conventional_config.types["feat"].bump, "minor");
        assert_eq!(conventional_config.types["fix"].bump, "patch");
        assert!(conventional_config.types["feat"].changelog);
        assert!(conventional_config.types["fix"].changelog);
    }

    #[test]
    fn test_registry_config_properties() {
        let registry_config = RegistryConfig::default();

        assert_eq!(registry_config.url, "https://registry.npmjs.org");
    }

    #[test]
    fn test_changelog_config_properties() {
        let changelog_config = ChangelogConfig::default();

        assert!(changelog_config.include_commit_hash);
        assert!(changelog_config.include_authors);
        assert!(changelog_config.group_by_type);
        assert!(changelog_config.include_date);
    }

    #[test]
    fn test_config_serialization() {
        let config = PackageToolsConfig::default();

        // Test JSON serialization
        let json_result = serde_json::to_string(&config);
        assert!(json_result.is_ok());

        // Test JSON deserialization
        let json_str = json_result.unwrap();
        let deserialized: Result<PackageToolsConfig, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let deserialized_config = deserialized.unwrap();
        assert_eq!(
            deserialized_config.version.commit_hash_length,
            config.version.commit_hash_length
        );
        assert_eq!(deserialized_config.release.strategy, config.release.strategy);
    }

    #[test]
    fn test_config_custom_values() {
        let mut config = PackageToolsConfig::default();

        // Modify some values
        config.changeset.path = PathBuf::from("custom-changesets");
        config.version.commit_hash_length = 10;
        config.release.strategy = "unified".to_string();

        // Validate the custom configuration
        assert!(config.validate().is_ok());
        assert_eq!(config.changeset.path, PathBuf::from("custom-changesets"));
        assert_eq!(config.version.commit_hash_length, 10);
        assert_eq!(config.release.strategy, "unified");
    }
}
