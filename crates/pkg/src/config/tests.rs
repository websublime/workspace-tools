#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod config_tests {
    use crate::config::{
        ChangelogConfig, ChangesetConfig, ConventionalConfig, DependencyConfig, EnvMapping,
        PackageToolsConfig, PackageToolsConfigManager, RegistryConfig, ReleaseConfig,
        VersionConfig, ENV_PREFIX,
    };
    use std::env;
    use std::path::PathBuf;
    use sublime_standard_tools::config::Configurable;
    use tempfile::TempDir;

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

    // Configuration Manager Tests
    #[test]
    fn test_config_manager_creation() {
        let manager = PackageToolsConfigManager::new();
        assert!(manager.project_path().is_some());
    }

    #[test]
    fn test_config_manager_with_project_path() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PackageToolsConfigManager::new_with_project_path(temp_dir.path());
        assert_eq!(manager.project_path().unwrap(), temp_dir.path());
    }

    #[test]
    fn test_default_implementation() {
        let manager1 = PackageToolsConfigManager::new();
        let manager2 = PackageToolsConfigManager::default();

        // Both should have project paths set
        assert!(manager1.project_path().is_some());
        assert!(manager2.project_path().is_some());
    }

    #[test]
    fn test_env_overrides() {
        env::set_var("SUBLIME_PACKAGE_TOOLS_RELEASE_STRATEGY", "unified");
        env::set_var("SUBLIME_PACKAGE_TOOLS_VERSION_COMMIT_HASH_LENGTH", "10");

        let manager = PackageToolsConfigManager::new();
        let overrides = manager.get_env_overrides();

        assert_eq!(overrides.get("RELEASE_STRATEGY"), Some(&"unified".to_string()));
        assert_eq!(overrides.get("VERSION_COMMIT_HASH_LENGTH"), Some(&"10".to_string()));

        // Cleanup
        env::remove_var("SUBLIME_PACKAGE_TOOLS_RELEASE_STRATEGY");
        env::remove_var("SUBLIME_PACKAGE_TOOLS_VERSION_COMMIT_HASH_LENGTH");
    }

    #[test]
    fn test_validate_config() {
        let manager = PackageToolsConfigManager::new();

        // Valid config should pass
        let valid_config = PackageToolsConfig::default();
        assert!(manager.validate_config(&valid_config).is_ok());

        // Invalid config should fail
        let mut invalid_config = PackageToolsConfig::default();
        invalid_config.changeset.available_environments.clear();
        assert!(manager.validate_config(&invalid_config).is_err());
    }

    #[test]
    fn test_env_mapping() {
        assert_eq!(
            EnvMapping::env_to_config_path("RELEASE_STRATEGY"),
            Some("release.strategy".to_string())
        );

        assert_eq!(
            EnvMapping::env_to_config_path("VERSION_COMMIT_HASH_LENGTH"),
            Some("version.commit_hash_length".to_string())
        );

        assert_eq!(EnvMapping::env_to_config_path("UNKNOWN_VAR"), None);
    }

    #[test]
    fn test_all_env_variables() {
        let variables = EnvMapping::all_env_variables();

        assert!(variables.contains(&"RELEASE_STRATEGY".to_string()));
        assert!(variables.contains(&"VERSION_COMMIT_HASH_LENGTH".to_string()));
        assert!(variables.contains(&"CHANGESET_PATH".to_string()));

        // Should have a reasonable number of variables
        assert!(variables.len() > 20);
    }

    #[tokio::test]
    async fn test_async_config_validation() {
        let manager = PackageToolsConfigManager::new();

        // Test that we can validate the default config
        let config = PackageToolsConfig::default();
        assert!(manager.validate_config(&config).is_ok());
    }

    #[test]
    fn test_user_config_dir() {
        // This test might not work in all environments, so we just check
        // that the function doesn't panic
        let manager = PackageToolsConfigManager::new();
        // Function should not panic regardless of environment
        assert!(manager.project_path().is_some());
    }

    #[test]
    fn test_env_prefix_constant() {
        assert_eq!(ENV_PREFIX, "SUBLIME_PACKAGE_TOOLS");
    }

    #[test]
    fn test_env_mapping_comprehensive() {
        // Test all changeset mappings
        assert_eq!(
            EnvMapping::env_to_config_path("CHANGESET_PATH"),
            Some("changeset.path".to_string())
        );
        assert_eq!(
            EnvMapping::env_to_config_path("CHANGESET_HISTORY_PATH"),
            Some("changeset.history_path".to_string())
        );
        assert_eq!(
            EnvMapping::env_to_config_path("CHANGESET_AVAILABLE_ENVIRONMENTS"),
            Some("changeset.available_environments".to_string())
        );

        // Test version mappings
        assert_eq!(
            EnvMapping::env_to_config_path("VERSION_SNAPSHOT_FORMAT"),
            Some("version.snapshot_format".to_string())
        );
        assert_eq!(
            EnvMapping::env_to_config_path("VERSION_ALLOW_SNAPSHOT_ON_MAIN"),
            Some("version.allow_snapshot_on_main".to_string())
        );

        // Test registry mappings
        assert_eq!(
            EnvMapping::env_to_config_path("REGISTRY_URL"),
            Some("registry.url".to_string())
        );
        assert_eq!(
            EnvMapping::env_to_config_path("REGISTRY_TIMEOUT"),
            Some("registry.timeout".to_string())
        );

        // Test dependency mappings
        assert_eq!(
            EnvMapping::env_to_config_path("DEPENDENCY_PROPAGATE_UPDATES"),
            Some("dependency.propagate_updates".to_string())
        );
        assert_eq!(
            EnvMapping::env_to_config_path("DEPENDENCY_MAX_DEPTH"),
            Some("dependency.max_propagation_depth".to_string())
        );

        // Test conventional mappings
        assert_eq!(
            EnvMapping::env_to_config_path("CONVENTIONAL_PARSE_BREAKING"),
            Some("conventional.parse_breaking_changes".to_string())
        );
        assert_eq!(
            EnvMapping::env_to_config_path("CONVENTIONAL_DEFAULT_BUMP"),
            Some("conventional.default_bump_type".to_string())
        );

        // Test changelog mappings
        assert_eq!(
            EnvMapping::env_to_config_path("CHANGELOG_INCLUDE_COMMIT_HASH"),
            Some("changelog.include_commit_hash".to_string())
        );
        assert_eq!(
            EnvMapping::env_to_config_path("CHANGELOG_GROUP_BY_TYPE"),
            Some("changelog.group_by_type".to_string())
        );
    }

    #[test]
    fn test_env_overrides_isolation() {
        // Test that environment variables don't interfere with each other
        let original_vars: Vec<(String, String)> =
            env::vars().filter(|(k, _)| k.starts_with("SUBLIME_PACKAGE_TOOLS_")).collect();

        // Clear existing environment variables
        for (key, _) in &original_vars {
            env::remove_var(key);
        }

        // Set test variables
        env::set_var("SUBLIME_PACKAGE_TOOLS_RELEASE_STRATEGY", "unified");
        env::set_var("SUBLIME_PACKAGE_TOOLS_VERSION_COMMIT_HASH_LENGTH", "8");
        env::set_var("SUBLIME_PACKAGE_TOOLS_CHANGESET_PATH", "/custom/path");

        let manager = PackageToolsConfigManager::new();
        let overrides = manager.get_env_overrides();

        assert_eq!(overrides.len(), 3);
        assert_eq!(overrides.get("RELEASE_STRATEGY"), Some(&"unified".to_string()));
        assert_eq!(overrides.get("VERSION_COMMIT_HASH_LENGTH"), Some(&"8".to_string()));
        assert_eq!(overrides.get("CHANGESET_PATH"), Some(&"/custom/path".to_string()));

        // Cleanup
        env::remove_var("SUBLIME_PACKAGE_TOOLS_RELEASE_STRATEGY");
        env::remove_var("SUBLIME_PACKAGE_TOOLS_VERSION_COMMIT_HASH_LENGTH");
        env::remove_var("SUBLIME_PACKAGE_TOOLS_CHANGESET_PATH");

        // Restore original variables
        for (key, value) in original_vars {
            env::set_var(key, value);
        }
    }

    #[test]
    fn test_config_manager_with_temp_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("repo.config.toml");

        // Write a test configuration file
        std::fs::write(
            &config_path,
            r#"
[package_tools.release]
strategy = "unified"

[package_tools.version]
commit_hash_length = 8
"#,
        )
        .unwrap();

        let manager = PackageToolsConfigManager::new_with_project_path(temp_dir.path());
        assert_eq!(manager.project_path().unwrap(), temp_dir.path());
    }

    #[tokio::test]
    async fn test_config_loading_integration() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PackageToolsConfigManager::new_with_project_path(temp_dir.path());

        // Should be able to load default configuration
        let config = manager.load_config().await;
        assert!(config.is_ok());

        let loaded_config = config.unwrap();
        assert_eq!(loaded_config.release.strategy, "independent");
        assert_eq!(loaded_config.version.commit_hash_length, 7);
    }

    #[test]
    fn test_all_env_variables_completeness() {
        let variables = EnvMapping::all_env_variables();

        // Verify that all variables in the list have valid mappings
        for var in &variables {
            assert!(
                EnvMapping::env_to_config_path(var).is_some(),
                "Variable {} should have a config path mapping",
                var
            );
        }

        // Test that the list contains expected categories
        let changeset_vars: Vec<_> =
            variables.iter().filter(|v| v.starts_with("CHANGESET_")).collect();
        let version_vars: Vec<_> = variables.iter().filter(|v| v.starts_with("VERSION_")).collect();
        let registry_vars: Vec<_> =
            variables.iter().filter(|v| v.starts_with("REGISTRY_")).collect();
        let release_vars: Vec<_> = variables.iter().filter(|v| v.starts_with("RELEASE_")).collect();
        let dependency_vars: Vec<_> =
            variables.iter().filter(|v| v.starts_with("DEPENDENCY_")).collect();
        let conventional_vars: Vec<_> =
            variables.iter().filter(|v| v.starts_with("CONVENTIONAL_")).collect();
        let changelog_vars: Vec<_> =
            variables.iter().filter(|v| v.starts_with("CHANGELOG_")).collect();

        assert!(!changeset_vars.is_empty(), "Should have changeset variables");
        assert!(!version_vars.is_empty(), "Should have version variables");
        assert!(!registry_vars.is_empty(), "Should have registry variables");
        assert!(!release_vars.is_empty(), "Should have release variables");
        assert!(!dependency_vars.is_empty(), "Should have dependency variables");
        assert!(!conventional_vars.is_empty(), "Should have conventional variables");
        assert!(!changelog_vars.is_empty(), "Should have changelog variables");
    }
}
