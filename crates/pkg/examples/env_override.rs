//! Example: Environment Variable Overrides
//!
//! This example demonstrates how to override configuration settings using environment variables.
//! Environment variables take precedence over values in the configuration file.
//!
//! Run this example with:
//! ```bash
//! # With environment variable overrides
//! PKG_TOOLS_VERSION_STRATEGY=unified \
//! PKG_TOOLS_CHANGESET_PATH=".custom-changesets" \
//! PKG_TOOLS_AUDIT_MIN_SEVERITY=info \
//! cargo run --example env_override
//!
//! # Without overrides (uses file or defaults)
//! cargo run --example env_override
//! ```

use std::env;
use sublime_pkg_tools::config::{PackageToolsConfig, VersioningStrategy, load_config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Environment Variable Override Example ===\n");

    // Display current environment variables
    println!("Current SUBLIME_PKG environment variables:");
    println!("------------------------------------------");
    let mut env_vars: Vec<(String, String)> =
        env::vars().filter(|(k, _)| k.starts_with("SUBLIME_PKG_")).collect();
    env_vars.sort_by(|a, b| a.0.cmp(&b.0));

    if env_vars.is_empty() {
        println!("(none set)\n");
    } else {
        for (key, value) in env_vars {
            println!("  {} = {}", key, value);
        }
        println!();
    }

    // Load configuration with environment variable support
    println!("Loading configuration with environment variable overrides...");
    let config = load_config().await?;

    println!("Configuration loaded successfully ✓\n");

    // Display resulting configuration
    print_detailed_config(&config);

    println!("\n");
    println!("=== Environment Variable Examples ===\n");
    println!("You can override any configuration setting using environment variables:");
    println!();

    println!("Changeset Configuration:");
    println!("  export SUBLIME_PKG_CHANGESET_PATH=\".custom-changesets\"");
    println!("  export SUBLIME_PKG_CHANGESET_HISTORY_PATH=\".custom-history\"");
    println!();

    println!("Version Configuration:");
    println!("  export SUBLIME_PKG_VERSION_STRATEGY=\"unified\"");
    println!("  export SUBLIME_PKG_VERSION_DEFAULT_BUMP=\"minor\"");
    println!("  export SUBLIME_PKG_VERSION_SNAPSHOT_FORMAT=\"{{version}}-snapshot.{{timestamp}}\"");
    println!();

    println!("Dependency Configuration:");
    println!("  export SUBLIME_PKG_DEPENDENCY_PROPAGATION_BUMP=\"minor\"");
    println!("  export SUBLIME_PKG_DEPENDENCY_PROPAGATE_DEPENDENCIES=\"true\"");
    println!("  export SUBLIME_PKG_DEPENDENCY_MAX_DEPTH=\"20\"");
    println!("  export SUBLIME_PKG_DEPENDENCY_FAIL_ON_CIRCULAR=\"false\"");
    println!();

    println!("Upgrade Configuration:");
    println!("  export SUBLIME_PKG_UPGRADE_AUTO_CHANGESET=\"true\"");
    println!(
        "  export SUBLIME_PKG_UPGRADE_REGISTRY_DEFAULT_REGISTRY=\"https://custom-registry.com\""
    );
    println!("  export SUBLIME_PKG_UPGRADE_REGISTRY_TIMEOUT_SECS=\"60\"");
    println!("  export SUBLIME_PKG_UPGRADE_BACKUP_ENABLED=\"true\"");
    println!();

    println!("Changelog Configuration:");
    println!("  export SUBLIME_PKG_CHANGELOG_ENABLED=\"true\"");
    println!("  export SUBLIME_PKG_CHANGELOG_FORMAT=\"conventional\"");
    println!("  export SUBLIME_PKG_CHANGELOG_INCLUDE_AUTHORS=\"true\"");
    println!("  export SUBLIME_PKG_CHANGELOG_REPOSITORY_URL=\"https://github.com/org/repo\"");
    println!();

    println!("Audit Configuration:");
    println!("  export SUBLIME_PKG_AUDIT_ENABLED=\"true\"");
    println!("  export SUBLIME_PKG_AUDIT_MIN_SEVERITY=\"info\"");
    println!();

    println!("Note: Environment variables take precedence over configuration file values.");
    println!("This is useful for:");
    println!("  - CI/CD pipelines");
    println!("  - Environment-specific overrides");
    println!("  - Testing different configurations");
    println!("  - Runtime customization");

    Ok(())
}

/// Helper function to print detailed configuration
fn print_detailed_config(config: &PackageToolsConfig) {
    println!("Current Configuration:");
    println!("=====================");

    println!("\n[Changeset]");
    println!("  path: {}", config.changeset.path);
    println!("  history_path: {}", config.changeset.history_path);
    println!("  available_environments: {:?}", config.changeset.available_environments);
    println!("  default_environments: {:?}", config.changeset.default_environments);

    println!("\n[Version]");
    println!("  strategy: {:?}", config.version.strategy);
    match config.version.strategy {
        VersioningStrategy::Independent => {
            println!("    → Each package versioned independently");
        }
        VersioningStrategy::Unified => {
            println!("    → All packages share the same version");
        }
    }
    println!("  default_bump: {}", config.version.default_bump);
    println!("  snapshot_format: {}", config.version.snapshot_format);

    println!("\n[Dependency]");
    println!("  propagation_bump: {}", config.dependency.propagation_bump);
    println!("  propagate_dependencies: {}", config.dependency.propagate_dependencies);
    println!("  propagate_dev_dependencies: {}", config.dependency.propagate_dev_dependencies);
    println!("  propagate_peer_dependencies: {}", config.dependency.propagate_peer_dependencies);
    println!("  max_depth: {}", config.dependency.max_depth);
    println!("  fail_on_circular: {}", config.dependency.fail_on_circular);
    println!("  skip_workspace_protocol: {}", config.dependency.skip_workspace_protocol);
    println!("  skip_file_protocol: {}", config.dependency.skip_file_protocol);
    println!("  skip_link_protocol: {}", config.dependency.skip_link_protocol);
    println!("  skip_portal_protocol: {}", config.dependency.skip_portal_protocol);

    println!("\n[Upgrade]");
    println!("  auto_changeset: {}", config.upgrade.auto_changeset);
    println!("  changeset_bump: {}", config.upgrade.changeset_bump);
    println!("\n  [Registry]");
    println!("    default_registry: {}", config.upgrade.registry.default_registry);
    println!("    timeout_secs: {}", config.upgrade.registry.timeout_secs);
    println!("    retry_attempts: {}", config.upgrade.registry.retry_attempts);
    println!("    retry_delay_ms: {}", config.upgrade.registry.retry_delay_ms);
    println!("    read_npmrc: {}", config.upgrade.registry.read_npmrc);
    if !config.upgrade.registry.scoped_registries.is_empty() {
        println!("    scoped_registries:");
        for (scope, url) in &config.upgrade.registry.scoped_registries {
            println!("      {} → {}", scope, url);
        }
    }
    println!("\n  [Backup]");
    println!("    enabled: {}", config.upgrade.backup.enabled);
    println!("    backup_dir: {}", config.upgrade.backup.backup_dir);
    println!("    keep_after_success: {}", config.upgrade.backup.keep_after_success);
    println!("    max_backups: {}", config.upgrade.backup.max_backups);

    println!("\n[Changelog]");
    println!("  enabled: {}", config.changelog.enabled);
    println!("  format: {:?}", config.changelog.format);
    println!("  filename: {}", config.changelog.filename);
    println!("  include_commit_links: {}", config.changelog.include_commit_links);
    println!("  include_issue_links: {}", config.changelog.include_issue_links);
    println!("  include_authors: {}", config.changelog.include_authors);
    if let Some(repo_url) = &config.changelog.repository_url {
        println!("  repository_url: {}", repo_url);
    }
    println!("  monorepo_mode: {:?}", config.changelog.monorepo_mode);
    println!("  version_tag_format: {}", config.changelog.version_tag_format);
    println!("  root_tag_format: {}", config.changelog.root_tag_format);

    println!("\n[Git]");
    println!("  include_breaking_warning: {}", config.git.include_breaking_warning);

    println!("\n[Audit]");
    println!("  enabled: {}", config.audit.enabled);
    println!("  min_severity: {:?}", config.audit.min_severity);
    println!("\n  [Sections]");
    println!("    upgrades: {}", config.audit.sections.upgrades);
    println!("    dependencies: {}", config.audit.sections.dependencies);
    println!("    breaking_changes: {}", config.audit.sections.breaking_changes);
    println!("    categorization: {}", config.audit.sections.categorization);
    println!("    version_consistency: {}", config.audit.sections.version_consistency);
    println!("\n  [Upgrades]");
    println!("    include_patch: {}", config.audit.upgrades.include_patch);
    println!("    include_minor: {}", config.audit.upgrades.include_minor);
    println!("    include_major: {}", config.audit.upgrades.include_major);
    println!("    deprecated_as_critical: {}", config.audit.upgrades.deprecated_as_critical);
    println!("\n  [Dependencies]");
    println!("    check_circular: {}", config.audit.dependencies.check_circular);
    println!("    check_missing: {}", config.audit.dependencies.check_missing);
    println!("    check_unused: {}", config.audit.dependencies.check_unused);
    println!("    check_version_conflicts: {}", config.audit.dependencies.check_version_conflicts);
    println!("\n  [Breaking Changes]");
    println!(
        "    check_conventional_commits: {}",
        config.audit.breaking_changes.check_conventional_commits
    );
    println!("    check_changelog: {}", config.audit.breaking_changes.check_changelog);
    println!("\n  [Version Consistency]");
    println!(
        "    fail_on_inconsistency: {}",
        config.audit.version_consistency.fail_on_inconsistency
    );
    println!(
        "    warn_on_inconsistency: {}",
        config.audit.version_consistency.warn_on_inconsistency
    );
}
