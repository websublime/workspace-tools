//! Tests for error types in package tools.
//!
//! This module contains all tests for error types, organized by error category.

use crate::error::{
    AuditError, AuditResult, ChangelogError, ChangelogResult, ChangesError, ChangesResult,
    ChangesetError, ChangesetResult, ConfigError, ConfigResult, Error, Result, UpgradeError,
    UpgradeResult, VersionError, VersionResult,
};
use std::path::PathBuf;

// =============================================================================
// Config Error Tests
// =============================================================================

mod config {
    use super::*;

    #[test]
    fn test_config_error_not_found() {
        let error = ConfigError::NotFound { path: PathBuf::from("/path/to/config.toml") };

        assert_eq!(error.as_ref(), "configuration file not found");
        assert!(error.to_string().contains("not found"));
        assert!(error.to_string().contains("/path/to/config.toml"));
    }

    #[test]
    fn test_config_error_parse_error() {
        let error = ConfigError::ParseError {
            path: PathBuf::from("config.toml"),
            reason: "invalid TOML syntax at line 5".to_string(),
        };

        assert_eq!(error.as_ref(), "configuration parse error");
        assert!(error.to_string().contains("parse"));
        assert!(error.to_string().contains("config.toml"));
        assert!(error.to_string().contains("line 5"));
    }

    #[test]
    fn test_config_error_validation_failed() {
        let errors =
            vec!["Invalid version format".to_string(), "Missing required field: name".to_string()];

        let error = ConfigError::ValidationFailed { errors: errors.clone() };

        assert_eq!(error.as_ref(), "configuration validation failed");
        assert_eq!(error.count(), 2);

        let formatted = error.errors();
        assert!(formatted.contains("Invalid version format"));
        assert!(formatted.contains("Missing required field: name"));
    }

    #[test]
    fn test_config_error_unsupported_format() {
        let error = ConfigError::UnsupportedFormat { format: "XML".to_string() };

        assert_eq!(error.as_ref(), "unsupported configuration format");
        assert!(error.to_string().contains("XML"));
    }

    #[test]
    fn test_config_error_env_var_error() {
        let error = ConfigError::EnvVarError {
            var_name: "PKG_TOOLS_VERSION".to_string(),
            reason: "not a valid semver string".to_string(),
        };

        assert_eq!(error.as_ref(), "environment variable error");
        assert!(error.to_string().contains("PKG_TOOLS_VERSION"));
        assert!(error.to_string().contains("not a valid semver"));
    }

    #[test]
    fn test_config_error_merge_conflict() {
        let error = ConfigError::MergeConflict {
            field: "version.strategy".to_string(),
            reason: "cannot merge 'independent' and 'unified'".to_string(),
        };

        assert_eq!(error.as_ref(), "configuration merge conflict");
        assert!(error.to_string().contains("version.strategy"));
    }

    #[test]
    fn test_config_error_invalid_path() {
        let error = ConfigError::InvalidPath {
            path: PathBuf::from(""),
            reason: "path cannot be empty".to_string(),
        };

        assert_eq!(error.as_ref(), "invalid configuration path");
        assert!(error.to_string().contains("path cannot be empty"));
    }

    #[test]
    fn test_config_error_missing_field() {
        let error = ConfigError::MissingField { field: "package_tools.changeset.path".to_string() };

        assert_eq!(error.as_ref(), "missing required configuration field");
        assert!(error.to_string().contains("package_tools.changeset.path"));
    }

    #[test]
    fn test_config_error_invalid_field_type() {
        let error = ConfigError::InvalidFieldType {
            field: "timeout".to_string(),
            expected: "integer".to_string(),
            actual: "string".to_string(),
        };

        assert_eq!(error.as_ref(), "invalid configuration field type");
        assert!(error.to_string().contains("timeout"));
        assert!(error.to_string().contains("expected integer"));
        assert!(error.to_string().contains("got string"));
    }

    #[test]
    fn test_config_error_permission_denied() {
        let error =
            ConfigError::PermissionDenied { path: PathBuf::from("/etc/pkg-tools/config.toml") };

        assert_eq!(error.as_ref(), "configuration permission denied");
        assert!(error.to_string().contains("/etc/pkg-tools/config.toml"));
    }

    #[test]
    fn test_config_error_circular_dependency() {
        let error = ConfigError::CircularDependency {
            cycle: "config.toml -> base.toml -> config.toml".to_string(),
        };

        assert_eq!(error.as_ref(), "circular configuration dependency");
        assert!(error.to_string().contains("config.toml -> base.toml"));
    }

    #[test]
    fn test_config_error_count_non_validation() {
        let error = ConfigError::NotFound { path: PathBuf::from("test.toml") };

        assert_eq!(error.count(), 1);
    }

    #[test]
    fn test_config_result_ok() {
        let result: ConfigResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_result_err() {
        let result: ConfigResult<String> =
            Err(ConfigError::InvalidConfig { message: "test error".to_string() });
        assert!(result.is_err());
    }
}

// =============================================================================
// Version Error Tests
// =============================================================================

mod version {
    use super::*;

    #[test]
    fn test_version_error_invalid_version() {
        let error = VersionError::InvalidVersion {
            version: "not-a-version".to_string(),
            reason: "does not match semver format".to_string(),
        };

        assert_eq!(error.as_ref(), "invalid version");
        assert!(error.to_string().contains("not-a-version"));
        assert!(error.to_string().contains("does not match semver format"));
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_version_error_parse_error() {
        let error = VersionError::ParseError {
            version: "1.x.0".to_string(),
            reason: "invalid character 'x'".to_string(),
        };

        assert_eq!(error.as_ref(), "version parse error");
        assert!(error.to_string().contains("1.x.0"));
        assert!(error.to_string().contains("invalid character"));
    }

    #[test]
    fn test_version_error_circular_dependency() {
        let cycle = vec![
            "pkg-a".to_string(),
            "pkg-b".to_string(),
            "pkg-c".to_string(),
            "pkg-a".to_string(),
        ];
        let error = VersionError::CircularDependency { cycle: cycle.clone() };

        assert_eq!(error.as_ref(), "circular dependency");
        assert_eq!(error.cycle(), Some(&cycle));
        assert_eq!(error.cycle_display(), Some("pkg-a -> pkg-b -> pkg-c -> pkg-a".to_string()));
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_version_error_package_not_found() {
        let error = VersionError::PackageNotFound {
            name: "my-package".to_string(),
            workspace_root: PathBuf::from("/workspace"),
        };

        assert_eq!(error.as_ref(), "package not found");
        assert!(error.to_string().contains("my-package"));
        assert!(error.to_string().contains("/workspace"));
    }

    #[test]
    fn test_version_error_resolution_failed() {
        let error = VersionError::ResolutionFailed {
            package: "pkg-a".to_string(),
            reason: "conflicting constraints".to_string(),
        };

        assert_eq!(error.as_ref(), "version resolution failed");
        assert!(error.to_string().contains("pkg-a"));
        assert!(error.to_string().contains("conflicting constraints"));
    }

    #[test]
    fn test_version_error_propagation_failed() {
        let error = VersionError::PropagationFailed {
            from: "pkg-a".to_string(),
            to: "pkg-b".to_string(),
            reason: "version constraint violation".to_string(),
        };

        assert_eq!(error.as_ref(), "version propagation failed");
        assert!(error.to_string().contains("pkg-a"));
        assert!(error.to_string().contains("pkg-b"));
    }

    #[test]
    fn test_version_error_invalid_strategy() {
        let error = VersionError::InvalidStrategy { strategy: "random".to_string() };

        assert_eq!(error.as_ref(), "invalid versioning strategy");
        assert!(error.to_string().contains("random"));
        assert!(error.to_string().contains("independent"));
        assert!(error.to_string().contains("unified"));
    }

    #[test]
    fn test_version_error_filesystem() {
        let error = VersionError::FileSystemError {
            path: PathBuf::from("package.json"),
            reason: "permission denied".to_string(),
        };

        assert_eq!(error.as_ref(), "filesystem error");
        assert!(error.to_string().contains("package.json"));
        assert!(error.is_recoverable());
    }

    #[test]
    fn test_version_error_max_depth_exceeded() {
        let error =
            VersionError::MaxDepthExceeded { package: "pkg-root".to_string(), max_depth: 10 };

        assert_eq!(error.as_ref(), "max propagation depth exceeded");
        assert!(error.to_string().contains("pkg-root"));
        assert!(error.to_string().contains("10"));
    }

    #[test]
    fn test_version_error_no_packages_to_update() {
        let error = VersionError::NoPackagesToUpdate;

        assert_eq!(error.as_ref(), "no packages to update");
        assert_eq!(error.cycle(), None);
    }

    #[test]
    fn test_version_error_version_conflict() {
        let error = VersionError::VersionConflict {
            dependency: "lodash".to_string(),
            conflict: "pkg-a requires ^4.0.0, pkg-b requires ^3.0.0".to_string(),
        };

        assert_eq!(error.as_ref(), "version conflict");
        assert!(error.to_string().contains("lodash"));
        assert!(error.to_string().contains("pkg-a requires"));
    }

    #[test]
    fn test_version_result_ok() {
        let result: VersionResult<String> = Ok("1.0.0".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_version_result_err() {
        let result: VersionResult<String> = Err(VersionError::NoPackagesToUpdate);
        assert!(result.is_err());
    }
}

// =============================================================================
// Changeset Error Tests
// =============================================================================

mod changeset {
    use super::*;

    #[test]
    fn test_changeset_error_not_found() {
        let error = ChangesetError::NotFound { branch: "feature/test".to_string() };

        assert_eq!(error.as_ref(), "changeset not found");
        assert!(error.to_string().contains("feature/test"));
        assert!(!error.is_transient());
    }

    #[test]
    fn test_changeset_error_invalid_branch() {
        let error = ChangesetError::InvalidBranch {
            branch: "".to_string(),
            reason: "branch name cannot be empty".to_string(),
        };

        assert_eq!(error.as_ref(), "invalid branch name");
        assert!(error.to_string().contains("branch name cannot be empty"));
    }

    #[test]
    fn test_changeset_error_validation_failed() {
        let errors = vec![
            "Missing required field: bump".to_string(),
            "Empty packages list".to_string(),
            "Invalid environment: 'prod'".to_string(),
        ];

        let error = ChangesetError::ValidationFailed { errors: errors.clone() };

        assert_eq!(error.as_ref(), "changeset validation failed");
        assert_eq!(error.count(), 3);

        let formatted = error.errors();
        assert!(formatted.contains("Missing required field: bump"));
        assert!(formatted.contains("Empty packages list"));
        assert!(formatted.contains("Invalid environment"));
    }

    #[test]
    fn test_changeset_error_storage_error() {
        let error = ChangesetError::StorageError {
            path: PathBuf::from(".changesets/main.json"),
            reason: "file is corrupted".to_string(),
        };

        assert_eq!(error.as_ref(), "changeset storage error");
        assert!(error.to_string().contains(".changesets/main.json"));
        assert!(error.to_string().contains("corrupted"));
        assert!(error.is_transient());
    }

    #[test]
    fn test_changeset_error_already_exists() {
        let error = ChangesetError::AlreadyExists {
            branch: "main".to_string(),
            path: PathBuf::from(".changesets/main.json"),
        };

        assert_eq!(error.as_ref(), "changeset already exists");
        assert!(error.to_string().contains("main"));
    }

    #[test]
    fn test_changeset_error_git_error() {
        let error = ChangesetError::GitError {
            operation: "fetch commits".to_string(),
            reason: "repository not found".to_string(),
        };

        assert_eq!(error.as_ref(), "git error");
        assert!(error.to_string().contains("fetch commits"));
        assert!(error.is_transient());
    }

    #[test]
    fn test_changeset_error_archive_error() {
        let error = ChangesetError::ArchiveError {
            branch: "release/v1.0".to_string(),
            reason: "history directory does not exist".to_string(),
        };

        assert_eq!(error.as_ref(), "changeset archive error");
        assert!(error.to_string().contains("release/v1.0"));
    }

    #[test]
    fn test_changeset_error_invalid_environment() {
        let error = ChangesetError::InvalidEnvironment {
            environment: "prod".to_string(),
            available: vec![
                "development".to_string(),
                "staging".to_string(),
                "production".to_string(),
            ],
        };

        assert_eq!(error.as_ref(), "invalid environment");
        assert!(error.to_string().contains("prod"));
    }

    #[test]
    fn test_changeset_error_empty_changeset() {
        let error = ChangesetError::EmptyChangeset { branch: "main".to_string() };

        assert_eq!(error.as_ref(), "empty changeset");
        assert!(error.to_string().contains("no packages"));
    }

    #[test]
    fn test_changeset_error_concurrent_modification() {
        let error = ChangesetError::ConcurrentModification {
            branch: "main".to_string(),
            expected: "2024-01-01T12:00:00Z".to_string(),
            actual: "2024-01-01T12:05:00Z".to_string(),
        };

        assert_eq!(error.as_ref(), "concurrent modification");
        assert!(error.to_string().contains("main"));
        assert!(error.is_transient());
    }

    #[test]
    fn test_changeset_error_lock_failed() {
        let error = ChangesetError::LockFailed {
            branch: "feature/test".to_string(),
            reason: "already locked by another process".to_string(),
        };

        assert_eq!(error.as_ref(), "lock failed");
        assert!(error.is_transient());
    }

    #[test]
    fn test_changeset_error_permission_denied() {
        let error = ChangesetError::PermissionDenied {
            path: PathBuf::from("/root/.changesets"),
            operation: "create directory".to_string(),
        };

        assert_eq!(error.as_ref(), "permission denied");
        assert!(error.to_string().contains("/root/.changesets"));
    }

    #[test]
    fn test_changeset_result_ok() {
        let result: ChangesetResult<String> = Ok("changeset-id".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_changeset_result_err() {
        let result: ChangesetResult<String> =
            Err(ChangesetError::NotFound { branch: "main".to_string() });
        assert!(result.is_err());
    }
}

// =============================================================================
// Changes Error Tests
// =============================================================================

mod changes {
    use super::*;

    #[test]
    fn test_changes_error_git_error() {
        let error = ChangesError::GitError {
            operation: "diff".to_string(),
            reason: "invalid object".to_string(),
        };

        assert_eq!(error.as_ref(), "git error");
        assert!(error.to_string().contains("diff"));
        assert!(error.to_string().contains("invalid object"));
        assert!(error.is_transient());
        assert!(error.is_git_related());
    }

    #[test]
    fn test_changes_error_invalid_commit_ref() {
        let error = ChangesError::InvalidCommitRef {
            reference: "HEAD~999".to_string(),
            reason: "reference does not exist".to_string(),
        };

        assert_eq!(error.as_ref(), "invalid commit reference");
        assert!(error.to_string().contains("HEAD~999"));
        assert!(error.is_git_related());
    }

    #[test]
    fn test_changes_error_invalid_commit_range() {
        let error = ChangesError::InvalidCommitRange {
            from: "abc123".to_string(),
            to: "def456".to_string(),
            reason: "from is after to".to_string(),
        };

        assert_eq!(error.as_ref(), "invalid commit range");
        assert!(error.to_string().contains("abc123"));
        assert!(error.to_string().contains("def456"));
        assert!(error.is_git_related());
    }

    #[test]
    fn test_changes_error_package_not_found() {
        let error = ChangesError::PackageNotFound {
            file: PathBuf::from("src/utils/helper.js"),
            workspace_root: PathBuf::from("/workspace"),
        };

        assert_eq!(error.as_ref(), "package not found");
        assert!(error.to_string().contains("src/utils/helper.js"));
        assert!(!error.is_transient());
    }

    #[test]
    fn test_changes_error_no_packages_found() {
        let error =
            ChangesError::NoPackagesFound { workspace_root: PathBuf::from("/empty-workspace") };

        assert_eq!(error.as_ref(), "no packages found");
        assert!(error.to_string().contains("empty-workspace"));
    }

    #[test]
    fn test_changes_error_filesystem_error() {
        let error = ChangesError::FileSystemError {
            path: PathBuf::from("package.json"),
            reason: "permission denied".to_string(),
        };

        assert_eq!(error.as_ref(), "filesystem error");
        assert!(error.to_string().contains("package.json"));
        assert!(error.is_transient());
        assert!(!error.is_git_related());
    }

    #[test]
    fn test_changes_error_package_json_parse_error() {
        let error = ChangesError::PackageJsonParseError {
            path: PathBuf::from("packages/core/package.json"),
            reason: "unexpected end of JSON".to_string(),
        };

        assert_eq!(error.as_ref(), "package.json parse error");
        assert!(error.to_string().contains("packages/core/package.json"));
    }

    #[test]
    fn test_changes_error_uncommitted_changes() {
        let error = ChangesError::UncommittedChanges {
            reason: "5 files modified, 2 files added".to_string(),
        };

        assert_eq!(error.as_ref(), "uncommitted changes");
        assert!(error.is_git_related());
    }

    #[test]
    fn test_changes_error_no_changes_detected() {
        let error =
            ChangesError::NoChangesDetected { scope: "commit range HEAD~10..HEAD".to_string() };

        assert_eq!(error.as_ref(), "no changes detected");
        assert!(error.to_string().contains("HEAD~10..HEAD"));
    }

    #[test]
    fn test_changes_error_file_outside_workspace() {
        let error = ChangesError::FileOutsideWorkspace {
            path: PathBuf::from("/external/file.js"),
            workspace_root: PathBuf::from("/workspace"),
        };

        assert_eq!(error.as_ref(), "file outside workspace");
        assert!(error.to_string().contains("/external/file.js"));
    }

    #[test]
    fn test_changes_error_repository_not_found() {
        let error = ChangesError::RepositoryNotFound { path: PathBuf::from("/not-a-repo") };

        assert_eq!(error.as_ref(), "repository not found");
        assert!(error.is_git_related());
    }

    #[test]
    fn test_changes_error_merge_conflict() {
        let error = ChangesError::MergeConflict { file: PathBuf::from("src/main.rs") };

        assert_eq!(error.as_ref(), "merge conflict");
        assert!(error.is_git_related());
    }

    #[test]
    fn test_changes_error_timeout() {
        let error = ChangesError::Timeout { duration_secs: 30 };

        assert_eq!(error.as_ref(), "analysis timeout");
        assert!(error.to_string().contains("30"));
        assert!(error.is_transient());
    }

    #[test]
    fn test_changes_error_monorepo_detection_failed() {
        let error = ChangesError::MonorepoDetectionFailed {
            reason: "conflicting workspace configurations".to_string(),
        };

        assert_eq!(error.as_ref(), "monorepo detection failed");
        assert!(error.to_string().contains("conflicting"));
    }

    #[test]
    fn test_changes_result_ok() {
        let result: ChangesResult<Vec<String>> = Ok(vec!["file1.js".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_changes_result_err() {
        let result: ChangesResult<Vec<String>> =
            Err(ChangesError::NoChangesDetected { scope: "working directory".to_string() });
        assert!(result.is_err());
    }
}

// =============================================================================
// Changelog Error Tests
// =============================================================================

mod changelog {
    use super::*;

    #[test]
    fn test_changelog_error_not_found() {
        let error = ChangelogError::NotFound { path: PathBuf::from("CHANGELOG.md") };

        assert_eq!(error.as_ref(), "changelog not found");
        assert!(error.to_string().contains("CHANGELOG.md"));
        assert!(!error.is_transient());
        assert!(!error.is_git_related());
    }

    #[test]
    fn test_changelog_error_parse_error() {
        let error =
            ChangelogError::ParseError { line: 42, reason: "unexpected header format".to_string() };

        assert_eq!(error.as_ref(), "changelog parse error");
        assert!(error.to_string().contains("42"));
        assert!(error.to_string().contains("unexpected header"));
        assert!(!error.is_transient());
        assert!(error.is_parse_related());
    }

    #[test]
    fn test_changelog_error_invalid_format() {
        let error = ChangelogError::InvalidFormat {
            expected: "Keep a Changelog".to_string(),
            actual: "unknown".to_string(),
        };

        assert_eq!(error.as_ref(), "invalid changelog format");
        assert!(error.to_string().contains("Keep a Changelog"));
        assert!(error.is_parse_related());
    }

    #[test]
    fn test_changelog_error_generation_failed() {
        let error = ChangelogError::GenerationFailed {
            version: "1.0.0".to_string(),
            reason: "no commits found".to_string(),
        };

        assert_eq!(error.as_ref(), "changelog generation failed");
        assert!(error.to_string().contains("1.0.0"));
        assert!(error.to_string().contains("no commits"));
    }

    #[test]
    fn test_changelog_error_conventional_commit_parse() {
        let error = ChangelogError::ConventionalCommitParseError {
            commit: "abc123".to_string(),
            reason: "missing type field".to_string(),
        };

        assert_eq!(error.as_ref(), "conventional commit parse error");
        assert!(error.to_string().contains("abc123"));
        assert!(error.is_parse_related());
    }

    #[test]
    fn test_changelog_error_git_error() {
        let error = ChangelogError::GitError {
            operation: "fetch tags".to_string(),
            reason: "remote not found".to_string(),
        };

        assert_eq!(error.as_ref(), "git error");
        assert!(error.to_string().contains("fetch tags"));
        assert!(error.is_transient());
        assert!(error.is_git_related());
    }

    #[test]
    fn test_changelog_error_version_not_found() {
        let error =
            ChangelogError::VersionNotFound { reason: "no version tags in repository".to_string() };

        assert_eq!(error.as_ref(), "version not found");
        assert!(error.is_git_related());
    }

    #[test]
    fn test_changelog_error_filesystem() {
        let error = ChangelogError::FileSystemError {
            path: PathBuf::from("CHANGELOG.md"),
            reason: "permission denied".to_string(),
        };

        assert_eq!(error.as_ref(), "filesystem error");
        assert!(error.is_transient());
        assert!(!error.is_git_related());
    }

    #[test]
    fn test_changelog_error_template_error() {
        let error =
            ChangelogError::TemplateError { reason: "undefined variable: version".to_string() };

        assert_eq!(error.as_ref(), "template error");
        assert!(error.to_string().contains("undefined variable"));
    }

    #[test]
    fn test_changelog_error_empty_changelog() {
        let error = ChangelogError::EmptyChangelog { version: "2.0.0".to_string() };

        assert_eq!(error.as_ref(), "empty changelog");
        assert!(error.to_string().contains("2.0.0"));
        assert!(error.to_string().contains("no commits"));
    }

    #[test]
    fn test_changelog_error_repository_url_missing() {
        let error = ChangelogError::RepositoryUrlMissing { link_type: "commit".to_string() };

        assert_eq!(error.as_ref(), "repository url missing");
        assert!(error.to_string().contains("commit"));
    }

    #[test]
    fn test_changelog_error_update_failed() {
        let error = ChangelogError::UpdateFailed {
            path: PathBuf::from("CHANGELOG.md"),
            reason: "file is read-only".to_string(),
        };

        assert_eq!(error.as_ref(), "update failed");
        assert!(error.is_transient());
    }

    #[test]
    fn test_changelog_error_changelog_exists() {
        let error = ChangelogError::ChangelogExists {
            version: "1.0.0".to_string(),
            path: PathBuf::from("CHANGELOG.md"),
        };

        assert_eq!(error.as_ref(), "changelog exists");
        assert!(error.to_string().contains("1.0.0"));
    }

    #[test]
    fn test_changelog_result_ok() {
        let result: ChangelogResult<String> = Ok("changelog content".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_changelog_result_err() {
        let result: ChangelogResult<String> =
            Err(ChangelogError::EmptyChangelog { version: "1.0.0".to_string() });
        assert!(result.is_err());
    }
}

// =============================================================================
// Upgrade Error Tests
// =============================================================================

mod upgrade {
    use super::*;

    #[test]
    fn test_upgrade_error_registry_error() {
        let error = UpgradeError::RegistryError {
            package: "lodash".to_string(),
            reason: "connection timeout".to_string(),
        };

        assert_eq!(error.as_ref(), "registry error");
        assert!(error.to_string().contains("lodash"));
        assert!(error.to_string().contains("connection timeout"));
        assert!(error.is_transient());
        assert!(error.is_registry_related());
    }

    #[test]
    fn test_upgrade_error_package_not_found() {
        let error = UpgradeError::PackageNotFound {
            package: "nonexistent-package".to_string(),
            registry: "https://registry.npmjs.org".to_string(),
        };

        assert_eq!(error.as_ref(), "package not found");
        assert!(error.to_string().contains("nonexistent-package"));
        assert!(error.is_registry_related());
    }

    #[test]
    fn test_upgrade_error_authentication_failed() {
        let error = UpgradeError::AuthenticationFailed {
            registry: "https://private.registry.com".to_string(),
            reason: "invalid token".to_string(),
        };

        assert_eq!(error.as_ref(), "authentication failed");
        assert!(error.is_registry_related());
    }

    #[test]
    fn test_upgrade_error_backup_failed() {
        let error = UpgradeError::BackupFailed {
            path: PathBuf::from("/backups/20240101"),
            reason: "disk full".to_string(),
        };

        assert_eq!(error.as_ref(), "backup failed");
        assert!(error.to_string().contains("/backups/20240101"));
        assert!(!error.is_transient());
        assert!(error.is_backup_related());
    }

    #[test]
    fn test_upgrade_error_no_backup() {
        let error = UpgradeError::NoBackup { path: PathBuf::from("/backups/latest") };

        assert_eq!(error.as_ref(), "no backup");
        assert!(error.is_backup_related());
    }

    #[test]
    fn test_upgrade_error_rollback_failed() {
        let error = UpgradeError::RollbackFailed { reason: "corrupted backup".to_string() };

        assert_eq!(error.as_ref(), "rollback failed");
        assert!(error.is_backup_related());
    }

    #[test]
    fn test_upgrade_error_no_upgrades_available() {
        let error = UpgradeError::NoUpgradesAvailable;

        assert_eq!(error.as_ref(), "no upgrades available");
        assert!(!error.is_transient());
        assert!(!error.is_registry_related());
    }

    #[test]
    fn test_upgrade_error_deprecated_package() {
        let error = UpgradeError::DeprecatedPackage {
            package: "request".to_string(),
            message: "Deprecated, use axios instead".to_string(),
            alternative: Some("axios".to_string()),
        };

        assert_eq!(error.as_ref(), "deprecated package");
        assert!(error.to_string().contains("request"));
        assert_eq!(error.alternative(), Some(&"axios".to_string()));
    }

    #[test]
    fn test_upgrade_error_deprecated_package_no_alternative() {
        let error = UpgradeError::DeprecatedPackage {
            package: "old-lib".to_string(),
            message: "No longer maintained".to_string(),
            alternative: None,
        };

        assert_eq!(error.alternative(), None);
    }

    #[test]
    fn test_upgrade_error_network_error() {
        let error = UpgradeError::NetworkError { reason: "DNS lookup failed".to_string() };

        assert_eq!(error.as_ref(), "network error");
        assert!(error.is_transient());
        assert!(error.is_registry_related());
    }

    #[test]
    fn test_upgrade_error_concurrent_modification() {
        let error = UpgradeError::ConcurrentModification { path: PathBuf::from("package.json") };

        assert_eq!(error.as_ref(), "concurrent modification");
        assert!(error.is_transient());
    }

    #[test]
    fn test_upgrade_error_rate_limit() {
        let error = UpgradeError::RateLimitExceeded {
            registry: "https://registry.npmjs.org".to_string(),
            reason: "retry after 60 seconds".to_string(),
        };

        assert_eq!(error.as_ref(), "rate limit exceeded");
        assert!(error.is_registry_related());
    }

    #[test]
    fn test_upgrade_error_filesystem() {
        let error = UpgradeError::FileSystemError {
            path: PathBuf::from("package.json"),
            reason: "permission denied".to_string(),
        };

        assert_eq!(error.as_ref(), "filesystem error");
        assert!(error.is_transient());
        assert!(!error.is_backup_related());
    }

    #[test]
    fn test_upgrade_error_max_backups_exceeded() {
        let error =
            UpgradeError::MaxBackupsExceeded { path: PathBuf::from("/backups"), max_backups: 10 };

        assert_eq!(error.as_ref(), "max backups exceeded");
        assert!(error.to_string().contains("10"));
        assert!(error.is_backup_related());
    }

    #[test]
    fn test_upgrade_result_ok() {
        let result: UpgradeResult<usize> = Ok(3);
        assert!(result.is_ok());
    }

    #[test]
    fn test_upgrade_result_err() {
        let result: UpgradeResult<usize> = Err(UpgradeError::NoUpgradesAvailable);
        assert!(result.is_err());
    }
}

// =============================================================================
// Audit Error Tests
// =============================================================================

mod audit {
    use super::*;

    #[test]
    fn test_audit_error_section_disabled() {
        let error = AuditError::SectionDisabled { section: "upgrades".to_string() };

        assert_eq!(error.as_ref(), "audit section disabled");
        assert!(error.to_string().contains("upgrades"));
        assert!(!error.is_transient());
        assert!(!error.is_fatal());
    }

    #[test]
    fn test_audit_error_analysis_failed() {
        let error = AuditError::AnalysisFailed {
            section: "dependencies".to_string(),
            reason: "graph construction failed".to_string(),
        };

        assert_eq!(error.as_ref(), "audit analysis failed");
        assert!(error.to_string().contains("dependencies"));
        assert!(error.to_string().contains("graph construction"));
    }

    #[test]
    fn test_audit_error_report_generation_failed() {
        let error = AuditError::ReportGenerationFailed { reason: "missing template".to_string() };

        assert_eq!(error.as_ref(), "report generation failed");
        assert!(!error.is_fatal());
    }

    #[test]
    fn test_audit_error_invalid_config() {
        let error = AuditError::InvalidConfig { reason: "missing min_severity".to_string() };

        assert_eq!(error.as_ref(), "invalid configuration");
        assert!(!error.is_transient());
        assert!(error.is_fatal());
    }

    #[test]
    fn test_audit_error_circular_dependency_detection() {
        let error = AuditError::CircularDependencyDetectionFailed {
            reason: "maximum depth exceeded".to_string(),
        };

        assert_eq!(error.as_ref(), "circular dependency detection failed");
        assert!(error.is_dependency_related());
    }

    #[test]
    fn test_audit_error_filesystem() {
        let error = AuditError::FileSystemError {
            path: PathBuf::from("package.json"),
            reason: "permission denied".to_string(),
        };

        assert_eq!(error.as_ref(), "filesystem error");
        assert!(error.is_transient());
        assert!(!error.is_dependency_related());
    }

    #[test]
    fn test_audit_error_git_error() {
        let error = AuditError::GitError {
            operation: "fetch commits".to_string(),
            reason: "network error".to_string(),
        };

        assert_eq!(error.as_ref(), "git error");
        assert!(error.is_transient());
    }

    #[test]
    fn test_audit_error_threshold_exceeded() {
        let error = AuditError::ThresholdExceeded {
            threshold_type: "critical".to_string(),
            limit: 5,
            actual: 10,
        };

        assert_eq!(error.as_ref(), "threshold exceeded");
        assert!(error.to_string().contains("critical"));
        assert!(error.to_string().contains("5"));
        assert!(error.to_string().contains("10"));
    }

    #[test]
    fn test_audit_error_invalid_severity() {
        let error = AuditError::InvalidSeverity { severity: "high".to_string() };

        assert_eq!(error.as_ref(), "invalid severity");
        assert!(error.to_string().contains("high"));
        assert!(error.to_string().contains("critical"));
    }

    #[test]
    fn test_audit_error_timeout() {
        let error = AuditError::Timeout { duration_secs: 120 };

        assert_eq!(error.as_ref(), "timeout");
        assert!(error.to_string().contains("120"));
        assert!(error.is_transient());
    }

    #[test]
    fn test_audit_error_workspace_analysis_failed() {
        let error = AuditError::WorkspaceAnalysisFailed { reason: "no packages found".to_string() };

        assert_eq!(error.as_ref(), "workspace analysis failed");
        assert!(error.is_fatal());
    }

    #[test]
    fn test_audit_error_dependency_graph_failed() {
        let error = AuditError::DependencyGraphFailed { reason: "circular reference".to_string() };

        assert_eq!(error.as_ref(), "dependency graph failed");
        assert!(error.is_dependency_related());
    }

    #[test]
    fn test_audit_error_version_conflict_detection() {
        let error = AuditError::VersionConflictDetectionFailed {
            reason: "incompatible versions".to_string(),
        };

        assert!(error.is_dependency_related());
    }

    #[test]
    fn test_audit_error_registry_error() {
        let error = AuditError::RegistryError { reason: "connection timeout".to_string() };

        assert_eq!(error.as_ref(), "registry error");
        assert!(error.is_transient());
    }

    #[test]
    fn test_audit_result_ok() {
        let result: AuditResult<String> = Ok("audit complete".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_audit_result_err() {
        let result: AuditResult<String> = Err(AuditError::NoIssuesFound);
        assert!(result.is_err());
    }
}

// =============================================================================
// Main Error Type Tests
// =============================================================================

mod main_error {
    use super::*;

    #[test]
    fn test_error_config_variant() {
        let error = Error::Config(ConfigError::NotFound { path: PathBuf::from("test.toml") });

        assert_eq!(error.as_ref(), "configuration file not found");
        assert!(error.to_string().contains("Configuration error"));
        assert!(!error.is_transient());
    }

    #[test]
    fn test_error_version_variant() {
        let error = Error::Version(VersionError::InvalidVersion {
            version: "invalid".to_string(),
            reason: "not semver".to_string(),
        });

        assert_eq!(error.as_ref(), "invalid version");
        assert!(error.to_string().contains("Version error"));
    }

    #[test]
    fn test_error_changeset_variant() {
        let error = Error::Changeset(ChangesetError::NotFound { branch: "main".to_string() });

        assert_eq!(error.as_ref(), "changeset not found");
        assert!(error.to_string().contains("Changeset error"));
    }

    #[test]
    fn test_error_changes_variant() {
        let error = Error::Changes(ChangesError::NoPackagesFound {
            workspace_root: PathBuf::from("/workspace"),
        });

        assert_eq!(error.as_ref(), "no packages found");
        assert!(error.to_string().contains("Changes analysis error"));
    }

    #[test]
    fn test_error_changelog_variant() {
        let error =
            Error::Changelog(ChangelogError::NotFound { path: PathBuf::from("CHANGELOG.md") });

        assert_eq!(error.as_ref(), "changelog not found");
        assert!(error.to_string().contains("Changelog error"));
    }

    #[test]
    fn test_error_upgrade_variant() {
        let error = Error::Upgrade(UpgradeError::NoUpgradesAvailable);

        assert_eq!(error.as_ref(), "no upgrades available");
        assert!(error.to_string().contains("Upgrade error"));
    }

    #[test]
    fn test_error_audit_variant() {
        let error = Error::Audit(AuditError::NoIssuesFound);

        assert_eq!(error.as_ref(), "no issues found");
        assert!(error.to_string().contains("Audit error"));
    }

    #[test]
    fn test_error_filesystem_variant() {
        let error = Error::FileSystem("file not found".to_string());

        assert_eq!(error.as_ref(), "filesystem error");
        assert!(error.is_transient());
    }

    #[test]
    fn test_error_git_variant() {
        let error = Error::Git("commit not found".to_string());

        assert_eq!(error.as_ref(), "git error");
        assert!(error.is_transient());
    }

    #[test]
    fn test_error_filesystem_helper() {
        let error = Error::filesystem_error(PathBuf::from("test.json"), "permission denied");

        assert!(error.to_string().contains("test.json"));
        assert!(error.to_string().contains("permission denied"));
    }

    #[test]
    fn test_error_git_helper() {
        let error = Error::git_error("fetch", "network error");

        assert!(error.to_string().contains("fetch"));
        assert!(error.to_string().contains("network error"));
    }

    #[test]
    fn test_result_ok() {
        let result: Result<String> = Ok("success".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_result_err() {
        let result: Result<String> =
            Err(Error::Config(ConfigError::InvalidConfig { message: "test".to_string() }));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_transient_check() {
        let transient = Error::FileSystem("lock error".to_string());
        assert!(transient.is_transient());

        let not_transient =
            Error::Config(ConfigError::InvalidConfig { message: "bad config".to_string() });
        assert!(!not_transient.is_transient());
    }

    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error: Error = io_error.into();

        assert!(matches!(error, Error::IO(_)));
        assert!(error.is_transient());
    }

    #[test]
    fn test_from_json_error() {
        let json_error =
            serde_json::from_str::<serde_json::Value>("{invalid json}").map_err(Error::from).err();

        assert!(json_error.is_some());
        assert!(matches!(json_error, Some(Error::Json(_))), "Expected Json error variant");
    }
}
