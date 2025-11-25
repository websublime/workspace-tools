//! Example: Parsing configuration from content
//!
//! This example demonstrates how to parse package tools configuration from
//! TOML, YAML, or JSON content using `PackageToolsConfig::from_str()`.
//!
//! **Note**: The `sublime_pkg_tools` crate is agnostic to file discovery.
//! File discovery and reading should be handled by the calling application
//! (typically the CLI). This crate only provides configuration parsing.
//!
//! Run this example with:
//! ```bash
//! cargo run --example load_config
//! ```

use sublime_pkg_tools::config::{ConfigFormat, PackageToolsConfig};
use sublime_standard_tools::config::Configurable;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Package Tools Configuration Parsing Example ===\n");

    // Method 1: Using default configuration
    println!("Method 1: Using default configuration");
    println!("--------------------------------------");

    let config = PackageToolsConfig::default();
    config.validate()?;
    println!("Default configuration is valid ✓\n");
    print_config(&config);

    println!("\n");

    // Method 2: Parsing from TOML content
    println!("Method 2: Parsing from TOML content");
    println!("------------------------------------");

    let toml_content = r#"
[changeset]
path = ".custom-changesets"
history_path = ".custom-history"
available_environments = ["dev", "staging", "prod"]
default_environments = ["prod"]

[version]
strategy = "unified"
default_bump = "minor"

[changelog]
enabled = true
include_authors = true
"#;

    let config = PackageToolsConfig::from_str(toml_content, ConfigFormat::Toml)?;
    println!("Parsed TOML configuration successfully ✓\n");
    print_config(&config);

    println!("\n");

    // Method 3: Parsing from JSON content
    println!("Method 3: Parsing from JSON content");
    println!("------------------------------------");

    let json_content = r#"{
        "changeset": {
            "path": ".json-changesets"
        },
        "version": {
            "default_bump": "patch"
        }
    }"#;

    let config = PackageToolsConfig::from_str(json_content, ConfigFormat::Json)?;
    println!("Parsed JSON configuration successfully ✓\n");
    print_config(&config);

    println!("\n");

    // Method 4: Parsing from YAML content
    println!("Method 4: Parsing from YAML content");
    println!("------------------------------------");

    let yaml_content = r#"
changeset:
  path: ".yaml-changesets"
  available_environments:
    - development
    - production
  default_environments:
    - production

upgrade:
  auto_changeset: true
  changeset_bump: minor
"#;

    let config = PackageToolsConfig::from_str(yaml_content, ConfigFormat::Yaml)?;
    println!("Parsed YAML configuration successfully ✓\n");
    print_config(&config);

    println!("\n");

    // Method 5: Programmatic configuration
    println!("Method 5: Creating configuration programmatically");
    println!("-------------------------------------------------");

    let mut config = PackageToolsConfig::default();

    // Customize specific settings
    config.changeset.path = ".programmatic-changesets".to_string();
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
