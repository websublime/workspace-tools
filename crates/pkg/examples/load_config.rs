//! Example: Loading configuration from a file
//!
//! This example demonstrates how to load package tools configuration from a TOML file
//! using the ConfigManager from sublime_standard_tools.
//!
//! Run this example with:
//! ```bash
//! cargo run --example load_config
//! ```

use sublime_pkg_tools::config::{
    ConfigLoader, PackageToolsConfig, load_config, load_config_from_file,
};
use sublime_standard_tools::config::Configurable;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Package Tools Configuration Loading Example ===\n");

    // Method 1: Using the convenience function
    println!("Method 1: Using load_config convenience function");
    println!("--------------------------------------------------");

    match load_config().await {
        Ok(config) => {
            println!("Loaded configuration from file successfully!\n");
            print_config(&config);
        }
        Err(e) => {
            println!("Could not load config file: {}", e);
            println!("Using default configuration instead...\n");
            match ConfigLoader::load_defaults().await {
                Ok(config) => print_config(&config),
                Err(e) => println!("Error loading defaults: {}", e),
            }
        }
    }

    println!("\n");

    // Method 2: Loading defaults only
    println!("Method 2: Loading defaults only");
    println!("--------------------------------");

    let config = ConfigLoader::load_defaults().await?;

    // Validate the configuration
    config.validate()?;
    println!("Configuration is valid ✓\n");

    print_config(&config);

    println!("\n");

    // Method 3: Load from specific file
    println!("Method 3: Loading from specific file");
    println!("-------------------------------------");

    // Try to load from examples directory
    match load_config_from_file("examples/basic-config.toml").await {
        Ok(config) => {
            println!("Loaded configuration from examples/basic-config.toml\n");
            print_config(&config);
        }
        Err(e) => {
            println!("Could not load examples/basic-config.toml: {}", e);
        }
    }

    println!("\n");

    // Method 4: Programmatic configuration
    println!("Method 4: Creating configuration programmatically");
    println!("-------------------------------------------------");

    let mut config = PackageToolsConfig::default();

    // Customize specific settings
    config.changeset.path = ".custom-changesets".to_string();
    config.version.default_bump = "minor".to_string();
    config.changelog.include_authors = true;

    // Validate before use
    config.validate()?;

    println!("Created custom configuration programmatically:");
    print_config(&config);

    Ok(())
}

/// Helper function to print configuration details
fn print_config(config: &PackageToolsConfig) {
    println!("Changeset Configuration:");
    println!("  Path: {}", config.changeset.path);
    println!("  History Path: {}", config.changeset.history_path);
    println!("  Available Environments: {:?}", config.changeset.available_environments);
    println!("  Default Environments: {:?}", config.changeset.default_environments);

    println!("\nVersion Configuration:");
    println!("  Strategy: {:?}", config.version.strategy);
    println!("  Default Bump: {}", config.version.default_bump);
    println!("  Snapshot Format: {}", config.version.snapshot_format);

    println!("\nDependency Configuration:");
    println!("  Propagation Bump: {}", config.dependency.propagation_bump);
    println!("  Propagate Dependencies: {}", config.dependency.propagate_dependencies);
    println!("  Propagate Dev Dependencies: {}", config.dependency.propagate_dev_dependencies);
    println!("  Propagate Peer Dependencies: {}", config.dependency.propagate_peer_dependencies);
    println!("  Max Depth: {}", config.dependency.max_depth);
    println!("  Fail on Circular: {}", config.dependency.fail_on_circular);

    println!("\nUpgrade Configuration:");
    println!("  Auto Changeset: {}", config.upgrade.auto_changeset);
    println!("  Changeset Bump: {}", config.upgrade.changeset_bump);
    println!("  Default Registry: {}", config.upgrade.registry.default_registry);
    println!("  Timeout: {}s", config.upgrade.registry.timeout_secs);
    println!("  Retry Attempts: {}", config.upgrade.registry.retry_attempts);

    println!("\nChangelog Configuration:");
    println!("  Enabled: {}", config.changelog.enabled);
    println!("  Format: {:?}", config.changelog.format);
    println!("  Filename: {}", config.changelog.filename);
    println!("  Include Commit Links: {}", config.changelog.include_commit_links);
    println!("  Include Issue Links: {}", config.changelog.include_issue_links);
    println!("  Include Authors: {}", config.changelog.include_authors);
    println!("  Monorepo Mode: {:?}", config.changelog.monorepo_mode);

    println!("\nGit Configuration:");
    println!("  Include Breaking Warning: {}", config.git.include_breaking_warning);

    println!("\nAudit Configuration:");
    println!("  Enabled: {}", config.audit.enabled);
    println!("  Min Severity: {:?}", config.audit.min_severity);
    println!("  Sections Enabled:");
    println!("    Upgrades: {}", config.audit.sections.upgrades);
    println!("    Dependencies: {}", config.audit.sections.dependencies);
    println!("    Breaking Changes: {}", config.audit.sections.breaking_changes);
    println!("    Categorization: {}", config.audit.sections.categorization);
    println!("    Version Consistency: {}", config.audit.sections.version_consistency);
}
