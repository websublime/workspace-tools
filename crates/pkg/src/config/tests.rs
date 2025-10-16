//! Tests for the configuration module.
//!
//! **What**: Comprehensive test suite for all configuration structures and validation logic.
//!
//! **How**: Tests are organized by configuration type, covering defaults, validation,
//! serialization, deserialization, and merging behavior.
//!
//! **Why**: To ensure configuration structures work correctly and maintain 100% test coverage.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use sublime_standard_tools::config::Configurable;

use crate::config::{
    load_config, load_config_from_file, validate_config, validate_path_format, validate_url_format,
    AuditConfig, AuditSectionsConfig, BackupConfig, BreakingChangesAuditConfig, ChangelogConfig,
    ChangelogFormat, ChangesetConfig, ConfigLoader, ConventionalConfig, DependencyAuditConfig,
    DependencyConfig, ExcludeConfig, GitConfig, MonorepoMode, PackageToolsConfig, RegistryConfig,
    TemplateConfig, UpgradeAuditConfig, UpgradeConfig, VersionConfig,
    VersionConsistencyAuditConfig, VersioningStrategy,
};

// =============================================================================
// PackageToolsConfig Tests
// =============================================================================

mod package_tools_config {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = PackageToolsConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = PackageToolsConfig::default();
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_config_deserialization() {
        // Test deserialization with a simple config
        let config = PackageToolsConfig::default();
        let serialized = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: PackageToolsConfig =
            serde_json::from_str(&serialized).expect("Should deserialize");

        // Verify key fields match
        assert_eq!(deserialized.changeset.path, config.changeset.path);
        assert_eq!(deserialized.version.strategy, config.version.strategy);
        assert_eq!(deserialized.dependency.propagation_bump, config.dependency.propagation_bump);
        assert_eq!(deserialized.upgrade.auto_changeset, config.upgrade.auto_changeset);
        assert_eq!(deserialized.changelog.enabled, config.changelog.enabled);
        assert_eq!(deserialized.git.include_breaking_warning, config.git.include_breaking_warning);
        assert_eq!(deserialized.audit.enabled, config.audit.enabled);
    }

    #[test]
    fn test_config_merge() {
        let mut base = PackageToolsConfig::default();
        let other = PackageToolsConfig::default();

        assert!(base.merge_with(other).is_ok());
    }

    #[test]
    fn test_nested_config_access() {
        let config = PackageToolsConfig::default();

        // Test that we can access nested configurations
        assert_eq!(config.changeset.path, ".changesets");
        assert_eq!(config.version.default_bump, "patch");
        assert!(config.dependency.propagate_dependencies);
        assert!(config.upgrade.auto_changeset);
        assert!(config.changelog.enabled);
        assert!(config.git.include_breaking_warning);
        assert!(config.audit.enabled);
    }
}

// =============================================================================
// ChangesetConfig Tests
// =============================================================================

mod changeset_config {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = ChangesetConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_values() {
        let config = ChangesetConfig::default();
        assert_eq!(config.path, ".changesets");
        assert_eq!(config.history_path, ".changesets/history");
        assert_eq!(config.available_environments, vec!["production"]);
        assert_eq!(config.default_environments, vec!["production"]);
    }

    #[test]
    fn test_empty_path_validation() {
        let config = ChangesetConfig { path: String::new(), ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_history_path_validation() {
        let config = ChangesetConfig { history_path: String::new(), ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_available_environments_validation() {
        let config = ChangesetConfig { available_environments: vec![], ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_default_environment() {
        let config = ChangesetConfig {
            available_environments: vec!["production".to_string()],
            default_environments: vec!["staging".to_string()],
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_multiple_environments() {
        let config = ChangesetConfig {
            available_environments: vec![
                "development".to_string(),
                "staging".to_string(),
                "production".to_string(),
            ],
            default_environments: vec!["staging".to_string(), "production".to_string()],
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_serialization() {
        let config = ChangesetConfig::default();
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "path": ".changesets",
            "history_path": ".changesets/history",
            "available_environments": ["production"],
            "default_environments": ["production"]
        }"#;

        let result: Result<ChangesetConfig, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ChangesetConfig::default());
    }

    #[test]
    fn test_merge() {
        let mut base = ChangesetConfig::default();
        let override_config = ChangesetConfig {
            path: ".custom-changesets".to_string(),
            history_path: ".custom-history".to_string(),
            available_environments: vec!["dev".to_string(), "prod".to_string()],
            default_environments: vec!["prod".to_string()],
        };

        assert!(base.merge_with(override_config.clone()).is_ok());
        assert_eq!(base.path, ".custom-changesets");
        assert_eq!(base.history_path, ".custom-history");
        assert_eq!(base.available_environments, override_config.available_environments);
        assert_eq!(base.default_environments, override_config.default_environments);
    }
}

// =============================================================================
// VersionConfig Tests
// =============================================================================

mod version_config {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = VersionConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_values() {
        let config = VersionConfig::default();
        assert_eq!(config.strategy, VersioningStrategy::Independent);
        assert_eq!(config.default_bump, "patch");
        assert_eq!(config.snapshot_format, "{version}-{branch}.{timestamp}");
    }

    #[test]
    fn test_versioning_strategy_serialization() {
        let independent = VersioningStrategy::Independent;
        let serialized = serde_json::to_string(&independent).unwrap();
        assert_eq!(serialized, r#""independent""#);

        let unified = VersioningStrategy::Unified;
        let serialized = serde_json::to_string(&unified).unwrap();
        assert_eq!(serialized, r#""unified""#);
    }

    #[test]
    fn test_versioning_strategy_deserialization() {
        let json = r#""independent""#;
        let result: Result<VersioningStrategy, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), VersioningStrategy::Independent);

        let json = r#""unified""#;
        let result: Result<VersioningStrategy, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), VersioningStrategy::Unified);
    }

    #[test]
    fn test_invalid_default_bump() {
        let config = VersionConfig { default_bump: "invalid".to_string(), ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_bump_types() {
        for bump in &["major", "minor", "patch", "none"] {
            let config = VersionConfig { default_bump: bump.to_string(), ..Default::default() };
            assert!(config.validate().is_ok(), "Bump type '{}' should be valid", bump);
        }
    }

    #[test]
    fn test_empty_snapshot_format() {
        let config = VersionConfig { snapshot_format: String::new(), ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_serialization() {
        let config = VersionConfig::default();
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "strategy": "independent",
            "default_bump": "patch",
            "snapshot_format": "{version}-{branch}.{timestamp}"
        }"#;

        let result: Result<VersionConfig, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), VersionConfig::default());
    }

    #[test]
    fn test_merge() {
        let mut base = VersionConfig::default();
        let override_config = VersionConfig {
            strategy: VersioningStrategy::Unified,
            default_bump: "minor".to_string(),
            snapshot_format: "{version}-snapshot".to_string(),
        };

        assert!(base.merge_with(override_config.clone()).is_ok());
        assert_eq!(base.strategy, VersioningStrategy::Unified);
        assert_eq!(base.default_bump, "minor");
        assert_eq!(base.snapshot_format, "{version}-snapshot");
    }

    #[test]
    fn test_custom_snapshot_format() {
        let config = VersionConfig {
            snapshot_format: "{version}-{short_hash}".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}

// =============================================================================
// DependencyConfig Tests
// =============================================================================

mod dependency_config {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = DependencyConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_values() {
        let config = DependencyConfig::default();
        assert_eq!(config.propagation_bump, "patch");
        assert!(config.propagate_dependencies);
        assert!(!config.propagate_dev_dependencies);
        assert!(config.propagate_peer_dependencies);
        assert_eq!(config.max_depth, 10);
        assert!(config.fail_on_circular);
        assert!(config.skip_workspace_protocol);
        assert!(config.skip_file_protocol);
        assert!(config.skip_link_protocol);
        assert!(config.skip_portal_protocol);
    }

    #[test]
    fn test_invalid_propagation_bump() {
        let config =
            DependencyConfig { propagation_bump: "invalid".to_string(), ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_bump_types() {
        for bump in &["major", "minor", "patch", "none"] {
            let config =
                DependencyConfig { propagation_bump: bump.to_string(), ..Default::default() };
            assert!(config.validate().is_ok(), "Bump type '{}' should be valid", bump);
        }
    }

    #[test]
    fn test_no_propagation_enabled() {
        let config = DependencyConfig {
            propagate_dependencies: false,
            propagate_dev_dependencies: false,
            propagate_peer_dependencies: false,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_only_dev_dependencies_enabled() {
        let config = DependencyConfig {
            propagate_dependencies: false,
            propagate_dev_dependencies: true,
            propagate_peer_dependencies: false,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_serialization() {
        let config = DependencyConfig::default();
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "propagation_bump": "patch",
            "propagate_dependencies": true,
            "propagate_dev_dependencies": false,
            "propagate_peer_dependencies": true,
            "max_depth": 10,
            "fail_on_circular": true,
            "skip_workspace_protocol": true,
            "skip_file_protocol": true,
            "skip_link_protocol": true,
            "skip_portal_protocol": true
        }"#;

        let result: Result<DependencyConfig, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DependencyConfig::default());
    }

    #[test]
    fn test_merge() {
        let mut base = DependencyConfig::default();
        let override_config = DependencyConfig {
            propagation_bump: "minor".to_string(),
            propagate_dependencies: false,
            propagate_dev_dependencies: true,
            propagate_peer_dependencies: false,
            max_depth: 5,
            fail_on_circular: false,
            skip_workspace_protocol: false,
            skip_file_protocol: false,
            skip_link_protocol: false,
            skip_portal_protocol: false,
        };

        assert!(base.merge_with(override_config.clone()).is_ok());
        assert_eq!(base.propagation_bump, "minor");
        assert!(!base.propagate_dependencies);
        assert!(base.propagate_dev_dependencies);
        assert!(!base.propagate_peer_dependencies);
        assert_eq!(base.max_depth, 5);
        assert!(!base.fail_on_circular);
        assert!(!base.skip_workspace_protocol);
        assert!(!base.skip_file_protocol);
        assert!(!base.skip_link_protocol);
        assert!(!base.skip_portal_protocol);
    }

    #[test]
    fn test_max_depth_zero() {
        let config = DependencyConfig { max_depth: 0, ..Default::default() };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_all_protocols_disabled() {
        let config = DependencyConfig {
            skip_workspace_protocol: false,
            skip_file_protocol: false,
            skip_link_protocol: false,
            skip_portal_protocol: false,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}

// =============================================================================
// GitConfig Tests
// =============================================================================

mod git_config {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = GitConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_values() {
        let config = GitConfig::default();
        assert_eq!(config.merge_commit_template, "chore(release): release version {version}");
        assert_eq!(
            config.monorepo_merge_commit_template,
            "chore(release): release packages\n\n{packages}"
        );
        assert!(config.include_breaking_warning);
        assert_eq!(config.breaking_warning_template, "⚠️ BREAKING CHANGES\n\n{changes}");
    }

    #[test]
    fn test_empty_merge_commit_template() {
        let config = GitConfig { merge_commit_template: String::new(), ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_monorepo_merge_commit_template() {
        let config =
            GitConfig { monorepo_merge_commit_template: String::new(), ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_breaking_warning_template_when_enabled() {
        let config = GitConfig {
            include_breaking_warning: true,
            breaking_warning_template: String::new(),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_breaking_warning_template_when_disabled() {
        let config = GitConfig {
            include_breaking_warning: false,
            breaking_warning_template: String::new(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_custom_templates() {
        let config = GitConfig {
            merge_commit_template: "release: v{version}".to_string(),
            monorepo_merge_commit_template: "release: {count} packages".to_string(),
            include_breaking_warning: true,
            breaking_warning_template: "BREAKING: {changes}".to_string(),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_serialization() {
        let config = GitConfig::default();
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "merge_commit_template": "chore(release): release version {version}",
            "monorepo_merge_commit_template": "chore(release): release packages\n\n{packages}",
            "include_breaking_warning": true,
            "breaking_warning_template": "⚠️ BREAKING CHANGES\n\n{changes}"
        }"#;

        let result: Result<GitConfig, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), GitConfig::default());
    }

    #[test]
    fn test_merge() {
        let mut base = GitConfig::default();
        let override_config = GitConfig {
            merge_commit_template: "custom: {version}".to_string(),
            monorepo_merge_commit_template: "custom: {packages}".to_string(),
            include_breaking_warning: false,
            breaking_warning_template: "BREAKING: {changes}".to_string(),
        };

        assert!(base.merge_with(override_config.clone()).is_ok());
        assert_eq!(base.merge_commit_template, "custom: {version}");
        assert_eq!(base.monorepo_merge_commit_template, "custom: {packages}");
        assert!(!base.include_breaking_warning);
        assert_eq!(base.breaking_warning_template, "BREAKING: {changes}");
    }

    #[test]
    fn test_template_placeholders() {
        let config = GitConfig::default();
        assert!(config.merge_commit_template.contains("{version}"));
        assert!(config.monorepo_merge_commit_template.contains("{packages}"));
        assert!(config.breaking_warning_template.contains("{changes}"));
    }

    #[test]
    fn test_breaking_warning_disabled() {
        let config = GitConfig { include_breaking_warning: false, ..Default::default() };
        assert!(config.validate().is_ok());
        assert!(!config.include_breaking_warning);
    }
}

// =============================================================================
// ChangelogConfig Tests
// =============================================================================

mod changelog_config {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = ChangelogConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_values() {
        let config = ChangelogConfig::default();
        assert!(config.enabled);
        assert_eq!(config.format, ChangelogFormat::KeepAChangelog);
        assert_eq!(config.filename, "CHANGELOG.md");
        assert!(config.include_commit_links);
        assert!(config.include_issue_links);
        assert!(!config.include_authors);
    }

    #[test]
    fn test_empty_filename_validation() {
        let config = ChangelogConfig { filename: String::new(), ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_conventional_config_default() {
        let config = ConventionalConfig::default();
        assert!(config.enabled);
        assert!(config.types.contains_key("feat"));
        assert!(config.types.contains_key("fix"));
        assert_eq!(config.breaking_section, "Breaking Changes");
    }

    #[test]
    fn test_format_serialization() {
        let format = ChangelogFormat::KeepAChangelog;
        let serialized = serde_json::to_string(&format).unwrap();
        assert_eq!(serialized, r#""keep-a-changelog""#);
    }

    #[test]
    fn test_monorepo_mode_serialization() {
        let mode = MonorepoMode::PerPackage;
        let serialized = serde_json::to_string(&mode).unwrap();
        assert_eq!(serialized, r#""per-package""#);
    }
}

// =============================================================================
// UpgradeConfig Tests
// =============================================================================

mod upgrade_config {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_default_upgrade_config_is_valid() {
        let config = UpgradeConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_registry_config_is_valid() {
        let config = RegistryConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_backup_config_is_valid() {
        let config = BackupConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_values() {
        let config = UpgradeConfig::default();
        assert!(config.auto_changeset);
        assert_eq!(config.changeset_bump, "patch");
        assert_eq!(config.registry.default_registry, "https://registry.npmjs.org");
        assert!(config.backup.enabled);
    }

    #[test]
    fn test_invalid_changeset_bump() {
        let config = UpgradeConfig { changeset_bump: "invalid".to_string(), ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_registry_url() {
        let config = UpgradeConfig {
            registry: RegistryConfig { default_registry: String::new(), ..Default::default() },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_zero_timeout() {
        let config = UpgradeConfig {
            registry: RegistryConfig { timeout_secs: 0, ..Default::default() },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_backup_dir() {
        let config = UpgradeConfig {
            backup: BackupConfig { backup_dir: String::new(), ..Default::default() },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_scoped_registries() {
        let mut scoped = HashMap::new();
        scoped.insert("myorg".to_string(), "https://npm.myorg.com".to_string());

        let config = UpgradeConfig {
            registry: RegistryConfig { scoped_registries: scoped.clone(), ..Default::default() },
            ..Default::default()
        };

        assert!(config.validate().is_ok());
        assert_eq!(
            config.registry.scoped_registries.get("myorg").unwrap(),
            "https://npm.myorg.com"
        );
    }

    #[test]
    fn test_serialization() {
        let config = UpgradeConfig::default();
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "registry": {
                "default_registry": "https://registry.npmjs.org",
                "scoped_registries": {},
                "auth_tokens": {},
                "timeout_secs": 30,
                "retry_attempts": 3,
                "retry_delay_ms": 1000,
                "read_npmrc": true
            },
            "auto_changeset": true,
            "changeset_bump": "patch",
            "backup": {
                "enabled": true,
                "backup_dir": ".pkg-backups",
                "keep_after_success": false,
                "max_backups": 5
            }
        }"#;

        let result: Result<UpgradeConfig, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_merge() {
        let mut base = UpgradeConfig::default();
        let override_config = UpgradeConfig {
            registry: RegistryConfig {
                default_registry: "https://custom.registry.com".to_string(),
                ..Default::default()
            },
            auto_changeset: false,
            changeset_bump: "minor".to_string(),
            backup: BackupConfig { max_backups: 10, ..Default::default() },
        };

        assert!(base.merge_with(override_config).is_ok());
        assert_eq!(base.registry.default_registry, "https://custom.registry.com");
        assert!(!base.auto_changeset);
        assert_eq!(base.changeset_bump, "minor");
        assert_eq!(base.backup.max_backups, 10);
    }
}

// =============================================================================
// AuditConfig Tests
// =============================================================================

mod audit_config {
    use super::*;

    #[test]
    fn test_default_audit_config_is_valid() {
        let config = AuditConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_values() {
        let config = AuditConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_severity, "warning");
        assert!(config.sections.upgrades);
        assert!(config.sections.dependencies);
        assert!(config.upgrades.include_major);
        assert!(config.dependencies.check_circular);
        assert!(config.breaking_changes.check_conventional_commits);
        assert!(config.version_consistency.warn_on_inconsistency);
    }

    #[test]
    fn test_invalid_severity() {
        let config = AuditConfig { min_severity: "invalid".to_string(), ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_severities() {
        for severity in &["critical", "warning", "info"] {
            let config = AuditConfig { min_severity: severity.to_string(), ..Default::default() };
            assert!(config.validate().is_ok(), "Severity '{}' should be valid", severity);
        }
    }

    #[test]
    fn test_serialization() {
        let config = AuditConfig::default();
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "enabled": true,
            "min_severity": "warning",
            "sections": {
                "upgrades": true,
                "dependencies": true,
                "breaking_changes": true,
                "categorization": true,
                "version_consistency": true
            },
            "upgrades": {
                "include_patch": true,
                "include_minor": true,
                "include_major": true,
                "deprecated_as_critical": true
            },
            "dependencies": {
                "check_circular": true,
                "check_missing": false,
                "check_unused": false,
                "check_version_conflicts": true
            },
            "breaking_changes": {
                "check_conventional_commits": true,
                "check_changelog": true
            },
            "version_consistency": {
                "fail_on_inconsistency": false,
                "warn_on_inconsistency": true
            }
        }"#;

        let result: Result<AuditConfig, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AuditConfig::default());
    }

    #[test]
    fn test_merge() {
        let mut base = AuditConfig::default();
        let override_config = AuditConfig {
            enabled: false,
            min_severity: "critical".to_string(),
            sections: AuditSectionsConfig {
                upgrades: false,
                dependencies: false,
                breaking_changes: false,
                categorization: false,
                version_consistency: false,
            },
            upgrades: UpgradeAuditConfig {
                include_patch: false,
                include_minor: false,
                include_major: false,
                deprecated_as_critical: false,
            },
            dependencies: DependencyAuditConfig {
                check_circular: false,
                check_missing: true,
                check_unused: true,
                check_version_conflicts: false,
            },
            breaking_changes: BreakingChangesAuditConfig {
                check_conventional_commits: false,
                check_changelog: false,
            },
            version_consistency: VersionConsistencyAuditConfig {
                fail_on_inconsistency: true,
                warn_on_inconsistency: false,
            },
        };

        assert!(base.merge_with(override_config.clone()).is_ok());
        assert!(!base.enabled);
        assert_eq!(base.min_severity, "critical");
        assert!(!base.sections.upgrades);
        assert!(!base.upgrades.include_major);
        assert!(base.dependencies.check_missing);
        assert!(!base.breaking_changes.check_conventional_commits);
        assert!(base.version_consistency.fail_on_inconsistency);
    }

    #[test]
    fn test_all_sections_disabled() {
        let config = AuditConfig {
            sections: AuditSectionsConfig {
                upgrades: false,
                dependencies: false,
                breaking_changes: false,
                categorization: false,
                version_consistency: false,
            },
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_all_checks_disabled() {
        let config = AuditConfig {
            dependencies: DependencyAuditConfig {
                check_circular: false,
                check_missing: false,
                check_unused: false,
                check_version_conflicts: false,
            },
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}

// =============================================================================
// Configuration Loading Tests
// =============================================================================

#[cfg(test)]
mod loader_tests {
    use std::fs;
    use tempfile::TempDir;

    use crate::config::{load_config, ConfigLoader};

    #[tokio::test]
    async fn test_load_defaults() {
        let result = ConfigLoader::load_defaults().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_from_nonexistent_files() {
        // Should succeed with just defaults
        let result = ConfigLoader::load_from_files(vec!["nonexistent.toml"]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_from_toml_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test-config.toml");

        let toml_content = r#"
[changeset]
path = ".custom-changesets"
history_path = ".custom-history"
available_environments = ["dev", "prod"]
default_environments = ["prod"]

[version]
strategy = "unified"
default_bump = "minor"
"#;

        fs::write(&config_path, toml_content).unwrap();

        let result = ConfigLoader::load_from_file(&config_path).await;
        assert!(result.is_ok());

        if let Ok(config) = result {
            assert_eq!(config.changeset.path, ".custom-changesets");
            assert_eq!(config.changeset.history_path, ".custom-history");
            assert_eq!(config.version.default_bump, "minor");
        }
    }
}

// =============================================================================
// Enhanced Validation Tests
// =============================================================================

#[cfg(test)]
mod validation_tests {
    use crate::config::{
        validate_config, validate_path_format, validate_url_format, PackageToolsConfig,
    };

    #[test]
    fn test_validate_default_config() {
        let config = PackageToolsConfig::default();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_changeset_parent_directory() {
        let mut config = PackageToolsConfig::default();
        config.changeset.path = "../changesets".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_same_changeset_paths() {
        let mut config = PackageToolsConfig::default();
        config.changeset.path = ".changesets".to_string();
        config.changeset.history_path = ".changesets".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_duplicate_environments() {
        let mut config = PackageToolsConfig::default();
        config.changeset.available_environments = vec!["prod".to_string(), "prod".to_string()];
        config.changeset.default_environments = vec!["prod".to_string()];

        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_environment_with_whitespace() {
        let mut config = PackageToolsConfig::default();
        config.changeset.available_environments = vec!["prod test".to_string()];
        config.changeset.default_environments = vec!["prod test".to_string()];

        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_registry_url() {
        let mut config = PackageToolsConfig::default();
        config.upgrade.registry.default_registry = "not-a-url".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_high_max_depth() {
        let mut config = PackageToolsConfig::default();
        config.dependency.max_depth = 150;

        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_no_propagation() {
        let mut config = PackageToolsConfig::default();
        config.dependency.propagate_dependencies = false;
        config.dependency.propagate_dev_dependencies = false;
        config.dependency.propagate_peer_dependencies = false;

        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_format_empty() {
        let result = validate_path_format("", "test_field");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_format_valid() {
        let result = validate_path_format(".changesets", "test_field");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_url_format_valid_https() {
        let result = validate_url_format("https://registry.npmjs.org", "test_field");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_url_format_valid_http() {
        let result = validate_url_format("http://localhost:4873", "test_field");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_url_format_invalid() {
        let result = validate_url_format("not-a-url", "test_field");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_url_format_empty() {
        let result = validate_url_format("", "test_field");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_snapshot_format_missing_placeholder() {
        let mut config = PackageToolsConfig::default();
        config.version.snapshot_format = "invalid-format".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_zero_timeout() {
        let mut config = PackageToolsConfig::default();
        config.upgrade.registry.timeout_secs = 0;

        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_high_timeout() {
        let mut config = PackageToolsConfig::default();
        config.upgrade.registry.timeout_secs = 500;

        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_too_many_retries() {
        let mut config = PackageToolsConfig::default();
        config.upgrade.registry.retry_attempts = 20;

        let result = validate_config(&config);
        assert!(result.is_err());
    }
}
