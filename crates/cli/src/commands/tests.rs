//! Tests for command implementations.
//!
//! This module contains comprehensive tests for all command implementations.
//! Each command module has its own test submodule.
//!
//! # What
//!
//! Provides unit and integration tests for:
//! - Init command: workspace initialization and configuration generation
//! - Config commands: configuration validation and display
//! - Changeset commands: changeset workflow operations
//! - Version commands: version bumping and management
//! - Upgrade commands: dependency upgrade detection and application
//! - Audit commands: workspace auditing and health checks
//!
//! # How
//!
//! Tests use:
//! - `tempfile` for temporary test directories
//! - Mock filesystem implementations where needed
//! - Real filesystem for integration tests
//! - Comprehensive assertion coverage
//!
//! # Why
//!
//! 100% test coverage ensures reliability and helps catch regressions.
//! Tests document expected behavior and serve as examples.

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod init_tests {
    use crate::cli::commands::InitArgs;
    use crate::commands::init::execute_init;
    use crate::output::OutputFormat;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper to create a test workspace with package.json
    fn create_test_workspace() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let package_json = serde_json::json!({
            "name": "test-project",
            "version": "1.0.0"
        });
        fs::write(
            temp_dir.path().join("package.json"),
            serde_json::to_string_pretty(&package_json).expect("Failed to serialize"),
        )
        .expect("Failed to write package.json");
        temp_dir
    }

    /// Helper to create a test monorepo workspace
    fn create_test_monorepo() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let package_json = serde_json::json!({
            "name": "test-monorepo",
            "version": "1.0.0",
            "workspaces": ["packages/*"]
        });
        fs::write(
            temp_dir.path().join("package.json"),
            serde_json::to_string_pretty(&package_json).expect("Failed to serialize"),
        )
        .expect("Failed to write package.json");

        // Create a workspace package
        let packages_dir = temp_dir.path().join("packages");
        fs::create_dir_all(&packages_dir).expect("Failed to create packages dir");

        let pkg1_dir = packages_dir.join("pkg1");
        fs::create_dir_all(&pkg1_dir).expect("Failed to create pkg1 dir");
        let pkg1_json = serde_json::json!({
            "name": "@test/pkg1",
            "version": "1.0.0"
        });
        fs::write(
            pkg1_dir.join("package.json"),
            serde_json::to_string_pretty(&pkg1_json).expect("Failed to serialize"),
        )
        .expect("Failed to write pkg1 package.json");

        temp_dir
    }

    #[tokio::test]
    async fn test_init_fails_without_package_json() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: None,
            default_env: None,
            strategy: None,
            registry: "https://registry.npmjs.org".to_string(),
            config_format: None,
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("package.json"));
    }

    #[tokio::test]
    async fn test_init_non_interactive_single_package() {
        let temp_dir = create_test_workspace();

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "staging".to_string(), "prod".to_string()]),
            default_env: Some(vec!["prod".to_string()]),
            strategy: Some("independent".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("toml".to_string()),
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_ok(), "Init failed: {result:?}");

        // Verify config file was created
        let config_path = temp_dir.path().join("repo.config.toml");
        assert!(config_path.exists(), "Config file not created");

        // Verify .changesets directory was created
        let changesets_dir = temp_dir.path().join(".changesets");
        assert!(changesets_dir.exists(), ".changesets directory not created");

        // Verify .changesets/history directory was created
        let history_dir = changesets_dir.join("history");
        assert!(history_dir.exists(), ".changesets/history directory not created");

        // Verify .gitkeep exists in history
        let gitkeep = history_dir.join(".gitkeep");
        assert!(gitkeep.exists(), ".gitkeep not created in history");

        // Verify .wnt-backups directory was created
        let backups_dir = temp_dir.path().join(".wnt-backups");
        assert!(backups_dir.exists(), ".wnt-backups directory not created");

        // Verify .gitignore was updated
        let gitignore_path = temp_dir.path().join(".gitignore");
        assert!(gitignore_path.exists(), ".gitignore not created");
        let gitignore_content =
            fs::read_to_string(gitignore_path).expect("Failed to read .gitignore");
        assert!(gitignore_content.contains(".wnt-backups/"), ".wnt-backups not in .gitignore");

        // Verify example changeset was created
        let example_path = changesets_dir.join("README-example.yaml");
        assert!(example_path.exists(), "Example changeset not created");
    }

    #[tokio::test]
    async fn test_init_with_yaml_format() {
        let temp_dir = create_test_workspace();

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "prod".to_string()]),
            default_env: Some(vec!["prod".to_string()]),
            strategy: Some("unified".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("yaml".to_string()),
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_ok(), "Init failed: {result:?}");

        // Verify YAML config file was created
        let config_path = temp_dir.path().join("repo.config.yaml");
        assert!(config_path.exists(), "YAML config file not created");

        // Verify it's valid YAML
        let content = fs::read_to_string(&config_path).expect("Failed to read config");
        let _parsed: serde_yaml::Value = serde_yaml::from_str(&content).expect("Invalid YAML");
    }

    #[tokio::test]
    async fn test_init_with_json_format() {
        let temp_dir = create_test_workspace();

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "prod".to_string()]),
            default_env: Some(vec!["prod".to_string()]),
            strategy: Some("independent".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("json".to_string()),
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_ok(), "Init failed: {result:?}");

        // Verify JSON config file was created
        let config_path = temp_dir.path().join("repo.config.json");
        assert!(config_path.exists(), "JSON config file not created");

        // Verify it's valid JSON
        let content = fs::read_to_string(&config_path).expect("Failed to read config");
        let _parsed: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");
    }

    #[tokio::test]
    async fn test_init_fails_with_existing_config() {
        let temp_dir = create_test_workspace();

        // Create existing config
        fs::write(temp_dir.path().join("repo.config.toml"), "# existing config")
            .expect("Failed to write existing config");

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "prod".to_string()]),
            default_env: Some(vec!["prod".to_string()]),
            strategy: Some("independent".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("toml".to_string()),
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn test_init_with_force_overwrites_existing() {
        let temp_dir = create_test_workspace();

        // Create existing config
        fs::write(temp_dir.path().join("repo.config.toml"), "# existing config")
            .expect("Failed to write existing config");

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "prod".to_string()]),
            default_env: Some(vec!["prod".to_string()]),
            strategy: Some("independent".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("toml".to_string()),
            force: true,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_ok(), "Init with force failed: {result:?}");

        // Verify config was overwritten (should have actual content, not "# existing config")
        let content = fs::read_to_string(temp_dir.path().join("repo.config.toml"))
            .expect("Failed to read config");
        assert!(!content.contains("# existing config"));
        assert!(content.contains("changeset") || content.contains("version"));
    }

    #[tokio::test]
    async fn test_init_validates_invalid_strategy() {
        let temp_dir = create_test_workspace();

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "prod".to_string()]),
            default_env: Some(vec!["prod".to_string()]),
            strategy: Some("invalid-strategy".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("toml".to_string()),
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid strategy"));
    }

    #[tokio::test]
    async fn test_init_validates_invalid_format() {
        let temp_dir = create_test_workspace();

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "prod".to_string()]),
            default_env: Some(vec!["prod".to_string()]),
            strategy: Some("independent".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("xml".to_string()),
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid config format"));
    }

    #[tokio::test]
    async fn test_init_validates_default_env_in_environments() {
        let temp_dir = create_test_workspace();

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "prod".to_string()]),
            default_env: Some(vec!["staging".to_string()]), // not in environments
            strategy: Some("independent".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("toml".to_string()),
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not in available environments"));
    }

    #[tokio::test]
    async fn test_init_validates_registry_url() {
        let temp_dir = create_test_workspace();

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "prod".to_string()]),
            default_env: Some(vec!["prod".to_string()]),
            strategy: Some("independent".to_string()),
            registry: "invalid-url".to_string(), // missing protocol
            config_format: Some("toml".to_string()),
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("http"));
    }

    #[tokio::test]
    async fn test_init_uses_defaults_when_not_provided() {
        let temp_dir = create_test_workspace();

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"), // default value
            environments: None, // should default to dev,staging,production
            default_env: None,  // should default to production
            strategy: None,     // should default based on workspace type
            registry: "https://registry.npmjs.org".to_string(), // default value
            config_format: None, // should default to toml
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_ok(), "Init with defaults failed: {result:?}");

        // Verify default config file format (toml)
        let config_path = temp_dir.path().join("repo.config.toml");
        assert!(config_path.exists(), "Default TOML config not created");

        // Verify default changeset path
        let changesets_dir = temp_dir.path().join(".changesets");
        assert!(changesets_dir.exists(), "Default .changesets directory not created");
    }

    #[tokio::test]
    async fn test_init_preserves_existing_gitignore() {
        let temp_dir = create_test_workspace();

        // Create existing .gitignore with content
        let existing_content = "node_modules/\ndist/\n";
        fs::write(temp_dir.path().join(".gitignore"), existing_content)
            .expect("Failed to write .gitignore");

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "prod".to_string()]),
            default_env: Some(vec!["prod".to_string()]),
            strategy: Some("independent".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("toml".to_string()),
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_ok(), "Init failed: {result:?}");

        // Verify existing content is preserved
        let gitignore_content = fs::read_to_string(temp_dir.path().join(".gitignore"))
            .expect("Failed to read .gitignore");
        assert!(gitignore_content.contains("node_modules/"), "Existing content lost");
        assert!(gitignore_content.contains("dist/"), "Existing content lost");
        assert!(gitignore_content.contains(".wnt-backups/"), "New content not added");
    }

    #[tokio::test]
    async fn test_init_monorepo_detection() {
        let temp_dir = create_test_monorepo();

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "prod".to_string()]),
            default_env: Some(vec!["prod".to_string()]),
            strategy: Some("unified".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("toml".to_string()),
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_ok(), "Init on monorepo failed: {result:?}");

        // Verify config was created
        let config_path = temp_dir.path().join("repo.config.toml");
        assert!(config_path.exists(), "Config not created for monorepo");
    }

    #[tokio::test]
    async fn test_init_json_output_format() {
        let temp_dir = create_test_workspace();

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "prod".to_string()]),
            default_env: Some(vec!["prod".to_string()]),
            strategy: Some("independent".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("toml".to_string()),
            force: false,
            non_interactive: true,
        };

        // This would normally print to stdout, we're just testing it doesn't panic
        let result = execute_init(&args, temp_dir.path(), OutputFormat::Json).await;

        assert!(result.is_ok(), "Init with JSON output failed: {result:?}");
    }

    #[tokio::test]
    async fn test_init_custom_changeset_path() {
        let temp_dir = create_test_workspace();

        let args = InitArgs {
            changeset_path: PathBuf::from("custom/changesets"),
            environments: Some(vec!["dev".to_string(), "prod".to_string()]),
            default_env: Some(vec!["prod".to_string()]),
            strategy: Some("independent".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("toml".to_string()),
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_ok(), "Init with custom path failed: {result:?}");

        // Verify custom changeset directory was created
        let changesets_dir = temp_dir.path().join("custom/changesets");
        assert!(changesets_dir.exists(), "Custom changeset directory not created");

        // Verify history subdirectory
        let history_dir = changesets_dir.join("history");
        assert!(history_dir.exists(), "History subdirectory not created in custom path");
    }

    #[tokio::test]
    async fn test_init_multiple_default_environments() {
        let temp_dir = create_test_workspace();

        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "staging".to_string(), "prod".to_string()]),
            default_env: Some(vec!["staging".to_string(), "prod".to_string()]),
            strategy: Some("independent".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("toml".to_string()),
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_ok(), "Init with multiple defaults failed: {result:?}");
    }

    #[tokio::test]
    async fn test_init_doesnt_duplicate_gitignore_entries() {
        let temp_dir = create_test_workspace();

        // Initialize once
        let args = InitArgs {
            changeset_path: PathBuf::from(".changesets"),
            environments: Some(vec!["dev".to_string(), "prod".to_string()]),
            default_env: Some(vec!["prod".to_string()]),
            strategy: Some("independent".to_string()),
            registry: "https://registry.npmjs.org".to_string(),
            config_format: Some("toml".to_string()),
            force: false,
            non_interactive: true,
        };

        let result = execute_init(&args, temp_dir.path(), OutputFormat::Quiet).await;
        assert!(result.is_ok(), "First init failed: {result:?}");

        let gitignore_path = temp_dir.path().join(".gitignore");
        let gitignore_content =
            fs::read_to_string(&gitignore_path).expect("Failed to read .gitignore");
        let backup_count = gitignore_content.matches(".wnt-backups/").count();

        // Initialize again with force
        let args_force = InitArgs { force: true, ..args };

        let result = execute_init(&args_force, temp_dir.path(), OutputFormat::Quiet).await;
        assert!(result.is_ok(), "Second init failed: {result:?}");

        let gitignore_content_after = fs::read_to_string(&gitignore_path)
            .expect("Failed to read .gitignore after second init");
        let backup_count_after = gitignore_content_after.matches(".wnt-backups/").count();

        // Should still only have one entry
        assert_eq!(backup_count, backup_count_after, "Gitignore entries were duplicated");
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod config_tests {
    use crate::cli::commands::ConfigShowArgs;
    use crate::commands::config::execute_show;
    use crate::output::OutputFormat;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a test workspace with package.json
    fn create_test_workspace() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let package_json = serde_json::json!({
            "name": "test-project",
            "version": "1.0.0"
        });
        fs::write(
            temp_dir.path().join("package.json"),
            serde_json::to_string_pretty(&package_json).expect("Failed to serialize"),
        )
        .expect("Failed to write package.json");
        temp_dir
    }

    /// Helper to create a config file
    fn create_config_file(temp_dir: &TempDir, format: &str) {
        let config = sublime_pkg_tools::config::PackageToolsConfig::default();
        let config_content = match format {
            "toml" => toml::to_string_pretty(&config).expect("Failed to serialize TOML"),
            "json" => serde_json::to_string_pretty(&config).expect("Failed to serialize JSON"),
            "yaml" => serde_yaml::to_string(&config).expect("Failed to serialize YAML"),
            _ => panic!("Unsupported format: {format}"),
        };
        let config_filename = format!("repo.config.{format}");
        fs::write(temp_dir.path().join(config_filename), config_content)
            .expect("Failed to write config file");
    }

    #[tokio::test]
    async fn test_config_show_with_existing_toml_config() {
        let temp_dir = create_test_workspace();
        create_config_file(&temp_dir, "toml");

        let args = ConfigShowArgs {};
        let result = execute_show(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_ok(), "Config show failed: {result:?}");
    }

    #[tokio::test]
    async fn test_config_show_with_existing_json_config() {
        let temp_dir = create_test_workspace();
        create_config_file(&temp_dir, "json");

        let args = ConfigShowArgs {};
        let result = execute_show(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_ok(), "Config show with JSON config failed: {result:?}");
    }

    #[tokio::test]
    async fn test_config_show_with_existing_yaml_config() {
        let temp_dir = create_test_workspace();
        create_config_file(&temp_dir, "yaml");

        let args = ConfigShowArgs {};
        let result = execute_show(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_ok(), "Config show with YAML config failed: {result:?}");
    }

    #[tokio::test]
    async fn test_config_show_without_config_uses_defaults() {
        let temp_dir = create_test_workspace();
        // Don't create config file

        let args = ConfigShowArgs {};
        let result = execute_show(&args, temp_dir.path(), OutputFormat::Quiet).await;

        // Should succeed with default config
        assert!(result.is_ok(), "Config show without config should use defaults: {result:?}");
    }

    #[tokio::test]
    async fn test_config_show_human_format() {
        let temp_dir = create_test_workspace();
        create_config_file(&temp_dir, "toml");

        let args = ConfigShowArgs {};
        let result = execute_show(&args, temp_dir.path(), OutputFormat::Human).await;

        assert!(result.is_ok(), "Config show in human format failed: {result:?}");
    }

    #[tokio::test]
    async fn test_config_show_json_format() {
        let temp_dir = create_test_workspace();
        create_config_file(&temp_dir, "toml");

        let args = ConfigShowArgs {};
        let result = execute_show(&args, temp_dir.path(), OutputFormat::Json).await;

        assert!(result.is_ok(), "Config show in JSON format failed: {result:?}");
    }

    #[tokio::test]
    async fn test_config_show_json_compact_format() {
        let temp_dir = create_test_workspace();
        create_config_file(&temp_dir, "toml");

        let args = ConfigShowArgs {};
        let result = execute_show(&args, temp_dir.path(), OutputFormat::JsonCompact).await;

        assert!(result.is_ok(), "Config show in JSON compact format failed: {result:?}");
    }

    #[tokio::test]
    async fn test_config_show_quiet_format() {
        let temp_dir = create_test_workspace();
        create_config_file(&temp_dir, "toml");

        let args = ConfigShowArgs {};
        let result = execute_show(&args, temp_dir.path(), OutputFormat::Quiet).await;

        assert!(result.is_ok(), "Config show in quiet format failed: {result:?}");
    }
}
