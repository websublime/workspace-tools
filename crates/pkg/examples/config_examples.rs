//! Configuration system usage examples for sublime_pkg_tools.
//!
//! This file demonstrates various ways to use the configuration system,
//! including loading from files, environment variables, and programmatic
//! configuration.

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use sublime_pkg_tools::config::{
    ChangelogConfig, ChangesetConfig, ConventionalCommitType, ConventionalConfig, DependencyConfig,
    EnvMapping, PackageToolsConfig, PackageToolsConfigManager, RegistryConfig, ReleaseConfig,
    VersionConfig, ENV_PREFIX,
};
use sublime_pkg_tools::error::PackageResult;

/// Example 1: Basic configuration loading with defaults.
///
/// Demonstrates loading configuration with default values and basic usage.
async fn example_basic_config_loading() -> PackageResult<()> {
    println!("=== Example 1: Basic Configuration Loading ===");

    // Create a configuration manager
    let manager = PackageToolsConfigManager::new();

    // Load configuration (will use defaults if no config files exist)
    let config = manager.load_config().await?;

    println!("Loaded configuration:");
    println!("- Release strategy: {}", config.release.strategy);
    println!("- Changeset path: {:?}", config.changeset.path);
    println!("- Version commit hash length: {}", config.version.commit_hash_length);
    println!("- Available environments: {:?}", config.changeset.available_environments);

    // Validate the configuration
    manager.validate_config(&config)?;
    println!("✅ Configuration is valid!");

    Ok(())
}

/// Example 2: Configuration with environment variable overrides.
///
/// Shows how to use environment variables to override specific configuration values.
async fn example_env_overrides() -> PackageResult<()> {
    println!("\n=== Example 2: Environment Variable Overrides ===");

    // Set environment variables for configuration override
    env::set_var("SUBLIME_PACKAGE_TOOLS_RELEASE_STRATEGY", "unified");
    env::set_var("SUBLIME_PACKAGE_TOOLS_VERSION_COMMIT_HASH_LENGTH", "10");
    env::set_var("SUBLIME_PACKAGE_TOOLS_CHANGESET_AVAILABLE_ENVIRONMENTS", "dev,staging,prod");

    let manager = PackageToolsConfigManager::new();

    // Show what environment overrides are active
    let overrides = manager.get_env_overrides();
    println!("Active environment overrides:");
    for (key, value) in &overrides {
        if let Some(config_path) = EnvMapping::env_to_config_path(key) {
            println!("  {} -> {}: {}", key, config_path, value);
        }
    }

    // Load configuration with environment overrides
    let config = manager.load_config().await?;

    println!("\nConfiguration with overrides:");
    println!("- Release strategy: {}", config.release.strategy);
    println!("- Version commit hash length: {}", config.version.commit_hash_length);

    // Cleanup environment variables
    env::remove_var("SUBLIME_PACKAGE_TOOLS_RELEASE_STRATEGY");
    env::remove_var("SUBLIME_PACKAGE_TOOLS_VERSION_COMMIT_HASH_LENGTH");
    env::remove_var("SUBLIME_PACKAGE_TOOLS_CHANGESET_AVAILABLE_ENVIRONMENTS");

    Ok(())
}

/// Example 3: Custom configuration creation and validation.
///
/// Demonstrates creating custom configuration programmatically and validating it.
async fn example_custom_config() -> PackageResult<()> {
    println!("\n=== Example 3: Custom Configuration ===");

    // Create a custom configuration
    let mut config = PackageToolsConfig {
        changeset: ChangesetConfig {
            path: PathBuf::from("custom-changesets"),
            history_path: PathBuf::from("custom-changesets/archive"),
            available_environments: vec![
                "dev".to_string(),
                "staging".to_string(),
                "prod".to_string(),
            ],
            default_environments: vec!["dev".to_string()],
            filename_format: "{branch}-{datetime}-custom.json".to_string(),
            max_pending_changesets: Some(50),
            auto_archive_applied: true,
        },
        version: VersionConfig {
            snapshot_format: "{version}-{commit}-SNAPSHOT".to_string(),
            commit_hash_length: 10,
            allow_snapshot_on_main: false,
            prerelease_format: Some("alpha.{number}".to_string()),
            build_metadata_format: Some("{timestamp}".to_string()),
        },
        release: ReleaseConfig {
            strategy: "unified".to_string(),
            tag_format: "v{version}".to_string(),
            env_tag_format: "v{version}-{environment}".to_string(),
            create_tags: true,
            push_tags: false, // Don't push in this example
            create_changelog: true,
            changelog_file: "RELEASES.md".to_string(),
            commit_message: "release: {package}@{version}".to_string(),
            dry_run_by_default: true,
            max_concurrent_releases: 3,
            release_timeout: 600,
        },
        registry: RegistryConfig {
            url: "https://npm.company.com".to_string(),
            timeout: 60,
            retry_attempts: 5,
            use_npmrc: false,
            registries: {
                let mut registries = HashMap::new();
                registries.insert(
                    "enterprise".to_string(),
                    sublime_pkg_tools::config::CustomRegistryConfig {
                        url: "https://npm.enterprise.com".to_string(),
                        auth_type: "token".to_string(),
                        auth_token: Some("${NPM_TOKEN}".to_string()),
                        auth_password: None,
                        timeout: Some(90),
                        default_access: Some("restricted".to_string()),
                    },
                );
                registries
            },
            default_access: "restricted".to_string(),
            skip_checks_in_dry_run: true,
        },
        dependency: DependencyConfig {
            propagate_updates: true,
            propagate_dev_dependencies: true,
            max_propagation_depth: 5,
            detect_circular: true,
            fail_on_circular: true,
            dependency_update_bump: "minor".to_string(),
            include_peer_dependencies: true,
            include_optional_dependencies: false,
        },
        conventional: ConventionalConfig {
            types: {
                let mut types = HashMap::new();

                types.insert(
                    "feat".to_string(),
                    ConventionalCommitType {
                        bump: "minor".to_string(),
                        changelog: true,
                        changelog_title: Some("✨ New Features".to_string()),
                        breaking: false,
                    },
                );

                types.insert(
                    "fix".to_string(),
                    ConventionalCommitType {
                        bump: "patch".to_string(),
                        changelog: true,
                        changelog_title: Some("🐛 Bug Fixes".to_string()),
                        breaking: false,
                    },
                );

                types.insert(
                    "perf".to_string(),
                    ConventionalCommitType {
                        bump: "patch".to_string(),
                        changelog: true,
                        changelog_title: Some("⚡ Performance Improvements".to_string()),
                        breaking: false,
                    },
                );

                types.insert(
                    "breaking".to_string(),
                    ConventionalCommitType {
                        bump: "major".to_string(),
                        changelog: true,
                        changelog_title: Some("💥 Breaking Changes".to_string()),
                        breaking: true,
                    },
                );

                types
            },
            parse_breaking_changes: true,
            require_conventional_commits: true,
            breaking_change_patterns: vec![
                "BREAKING CHANGE:".to_string(),
                "BREAKING-CHANGE:".to_string(),
                "!:".to_string(),
            ],
            default_bump_type: "patch".to_string(),
        },
        changelog: ChangelogConfig {
            include_commit_hash: false,
            include_authors: true,
            group_by_type: true,
            include_date: true,
            max_commits_per_release: Some(100),
            template_file: Some(PathBuf::from("templates/changelog.hbs")),
            custom_sections: {
                let mut sections = HashMap::new();
                sections.insert("migration".to_string(), "📋 Migration Guide".to_string());
                sections.insert("deprecation".to_string(), "⚠️ Deprecations".to_string());
                sections
            },
            link_commits: true,
            commit_url_format: Some("https://github.com/company/repo/commit/{hash}".to_string()),
        },
    };

    println!("Custom configuration created:");
    println!("- Changeset path: {:?}", config.changeset.path);
    println!("- Release strategy: {}", config.release.strategy);
    println!("- Registry URL: {}", config.registry.url);

    // Validate the custom configuration
    let manager = PackageToolsConfigManager::new();
    manager.validate_config(&config)?;
    println!("✅ Custom configuration is valid!");

    // Test configuration utility methods
    println!("\nConfiguration utility methods:");
    println!("- Bump type for 'feat': {}", config.get_bump_type("feat"));
    println!("- Should include 'fix' in changelog: {}", config.should_include_in_changelog("fix"));
    println!("- Is 'dev' available environment: {}", config.is_environment_available("dev"));

    // Test invalid configuration
    config.changeset.available_environments.clear();
    match manager.validate_config(&config) {
        Ok(()) => println!("❌ Should have failed validation"),
        Err(_) => println!("✅ Correctly detected invalid configuration"),
    }

    Ok(())
}

/// Example 4: Configuration file formats and project-specific setup.
///
/// Shows how configuration would work with different file formats in a project.
async fn example_project_config_setup() -> PackageResult<()> {
    println!("\n=== Example 4: Project Configuration Setup ===");

    // Example TOML configuration content
    let toml_config = r#"
[package_tools.release]
strategy = "independent"
tag_format = "{package}@{version}"
dry_run_by_default = false

[package_tools.changeset]
path = ".changesets"
available_environments = ["dev", "test", "qa", "staging", "prod"]
default_environments = ["dev"]

[package_tools.version]
commit_hash_length = 7
allow_snapshot_on_main = false

[package_tools.conventional.types.feat]
bump = "minor"
changelog = true
changelog_title = "Features"

[package_tools.conventional.types.fix]
bump = "patch"
changelog = true
changelog_title = "Bug Fixes"
"#;

    println!("Example TOML configuration:");
    println!("{}", toml_config);

    // Example JSON configuration content
    let json_config = r#"{
  "package_tools": {
    "release": {
      "strategy": "unified",
      "create_changelog": true,
      "changelog_file": "CHANGELOG.md"
    },
    "registry": {
      "url": "https://registry.npmjs.org",
      "timeout": 30,
      "use_npmrc": true
    },
    "dependency": {
      "propagate_updates": true,
      "dependency_update_bump": "patch"
    }
  }
}"#;

    println!("\nExample JSON configuration:");
    println!("{}", json_config);

    // Show environment variable documentation
    println!("\nSupported environment variables:");
    let env_vars = EnvMapping::all_env_variables();
    for (i, var) in env_vars.iter().enumerate().take(10) {
        if let Some(config_path) = EnvMapping::env_to_config_path(var) {
            println!("  {}_{}={} -> {}", ENV_PREFIX, var, "<value>", config_path);
        }
        if i == 9 && env_vars.len() > 10 {
            println!("  ... and {} more", env_vars.len() - 10);
        }
    }

    Ok(())
}

/// Example 5: Configuration for different deployment environments.
///
/// Shows how to set up configuration for different deployment scenarios.
async fn example_deployment_configs() -> PackageResult<()> {
    println!("\n=== Example 5: Deployment Environment Configurations ===");

    // Development environment configuration
    let dev_config = PackageToolsConfig {
        release: ReleaseConfig {
            strategy: "independent".to_string(),
            dry_run_by_default: true,
            push_tags: false,
            max_concurrent_releases: 1,
            ..Default::default()
        },
        changeset: ChangesetConfig {
            default_environments: vec!["dev".to_string()],
            auto_archive_applied: false,
            ..Default::default()
        },
        registry: RegistryConfig { skip_checks_in_dry_run: true, ..Default::default() },
        ..Default::default()
    };

    println!("Development configuration:");
    println!("- Dry run by default: {}", dev_config.release.dry_run_by_default);
    println!("- Push tags: {}", dev_config.release.push_tags);
    println!("- Default environments: {:?}", dev_config.changeset.default_environments);

    // Production environment configuration
    let prod_config = PackageToolsConfig {
        release: ReleaseConfig {
            strategy: "unified".to_string(),
            dry_run_by_default: false,
            push_tags: true,
            max_concurrent_releases: 5,
            release_timeout: 900,
            ..Default::default()
        },
        changeset: ChangesetConfig {
            default_environments: vec!["prod".to_string()],
            auto_archive_applied: true,
            max_pending_changesets: Some(10),
            ..Default::default()
        },
        registry: RegistryConfig {
            retry_attempts: 5,
            timeout: 60,
            skip_checks_in_dry_run: false,
            ..Default::default()
        },
        conventional: ConventionalConfig {
            require_conventional_commits: true,
            ..Default::default()
        },
        ..Default::default()
    };

    println!("\nProduction configuration:");
    println!("- Dry run by default: {}", prod_config.release.dry_run_by_default);
    println!("- Push tags: {}", prod_config.release.push_tags);
    println!("- Release timeout: {} seconds", prod_config.release.release_timeout);
    println!(
        "- Require conventional commits: {}",
        prod_config.conventional.require_conventional_commits
    );

    // Validate both configurations
    let manager = PackageToolsConfigManager::new();
    manager.validate_config(&dev_config)?;
    manager.validate_config(&prod_config)?;
    println!("✅ Both configurations are valid!");

    Ok(())
}

/// Example 6: Dynamic configuration based on project detection.
///
/// Shows how to adapt configuration based on detected project characteristics.
async fn example_dynamic_config() -> PackageResult<()> {
    println!("\n=== Example 6: Dynamic Configuration ===");

    let manager = PackageToolsConfigManager::new();
    let mut config = manager.load_config().await?;

    // Simulate project detection results
    let is_monorepo = true;
    let has_conventional_commits = true;
    let is_public_package = false;

    println!("Detected project characteristics:");
    println!("- Is monorepo: {}", is_monorepo);
    println!("- Uses conventional commits: {}", has_conventional_commits);
    println!("- Is public package: {}", is_public_package);

    // Adapt configuration based on detection
    if is_monorepo {
        config.release.strategy = "independent".to_string();
        config.dependency.propagate_updates = true;
        config.dependency.max_propagation_depth = 10;
        println!("✅ Configured for monorepo with independent versioning");
    }

    if has_conventional_commits {
        config.conventional.require_conventional_commits = true;
        config.release.create_changelog = true;
        println!("✅ Enabled strict conventional commit parsing");
    }

    if !is_public_package {
        config.registry.default_access = "restricted".to_string();
        config.registry.skip_checks_in_dry_run = true;
        println!("✅ Configured for private package registry");
    }

    // Validate the dynamically configured setup
    manager.validate_config(&config)?;
    println!("✅ Dynamic configuration is valid!");

    Ok(())
}

/// Main function that runs all examples.
#[tokio::main]
async fn main() -> PackageResult<()> {
    println!("🚀 sublime_pkg_tools Configuration Examples\n");

    // Run all examples
    example_basic_config_loading().await?;
    example_env_overrides().await?;
    example_custom_config().await?;
    example_project_config_setup().await?;
    example_deployment_configs().await?;
    example_dynamic_config().await?;

    println!("\n✅ All configuration examples completed successfully!");
    println!("\n📚 Key Takeaways:");
    println!(
        "  • Configuration can be loaded from files (TOML/JSON/YAML) or environment variables"
    );
    println!("  • Environment variables use the {} prefix", ENV_PREFIX);
    println!("  • All configuration is validated before use");
    println!("  • Configuration can be adapted dynamically based on project characteristics");
    println!("  • Both development and production scenarios are supported");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_config_loading() {
        assert!(example_basic_config_loading().await.is_ok());
    }

    #[tokio::test]
    async fn test_env_overrides() {
        assert!(example_env_overrides().await.is_ok());
    }

    #[tokio::test]
    async fn test_custom_config() {
        assert!(example_custom_config().await.is_ok());
    }

    #[tokio::test]
    async fn test_project_config_setup() {
        assert!(example_project_config_setup().await.is_ok());
    }

    #[tokio::test]
    async fn test_deployment_configs() {
        assert!(example_deployment_configs().await.is_ok());
    }

    #[tokio::test]
    async fn test_dynamic_config() {
        assert!(example_dynamic_config().await.is_ok());
    }
}
