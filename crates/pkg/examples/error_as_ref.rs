//! Example demonstrating AsRef<str> implementation for all error types in sublime_pkg_tools.
//!
//! This example shows how all error types implement AsRef<str> which provides
//! a consistent way to get string identifiers for error variants.

use std::path::PathBuf;
use sublime_pkg_tools::error::{
    ChangelogError, ChangesetError, CommitTypeParseError, ConfigError, ConventionalCommitError,
    DependencyError, PackageError, RegistryError, ReleaseError, VersionError,
};

fn main() {
    println!("=== AsRef<str> Implementation Examples ===\n");

    // PackageError examples
    println!("PackageError variants:");
    let version_err = PackageError::Version(VersionError::InvalidFormat {
        version: "not-a-version".to_string(),
        reason: "Invalid format".to_string(),
    });
    println!("  Version error: {}", version_err.as_ref());

    let io_err =
        PackageError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"));
    println!("  I/O error: {}", io_err.as_ref());

    let operation_err = PackageError::Operation {
        operation: "test_operation".to_string(),
        reason: "test failure".to_string(),
    };
    println!("  Operation error: {}\n", operation_err.as_ref());

    // VersionError examples
    println!("VersionError variants:");
    let parse_err = VersionError::ParseFailed {
        version: "1.2.invalid".to_string(),
        source: semver::Version::parse("1.2.invalid").unwrap_err(),
    };
    println!("  Parse failed: {}", parse_err.as_ref());

    let conflict_err = VersionError::Conflict {
        package: "my-package".to_string(),
        current: "1.0.0".to_string(),
        requested: "2.0.0".to_string(),
    };
    println!("  Version conflict: {}", conflict_err.as_ref());

    let snapshot_err = VersionError::SnapshotResolutionFailed {
        package: "test-pkg".to_string(),
        reason: "No git repository".to_string(),
    };
    println!("  Snapshot resolution: {}\n", snapshot_err.as_ref());

    // ChangesetError examples
    println!("ChangesetError variants:");
    let not_found_err =
        ChangesetError::NotFound { path: PathBuf::from(".changesets/missing.json") };
    println!("  Not found: {}", not_found_err.as_ref());

    let validation_err = ChangesetError::ValidationFailed {
        changeset_id: "feat-auth-123".to_string(),
        errors: vec!["Missing summary".to_string()],
    };
    println!("  Validation failed: {}", validation_err.as_ref());

    let creation_err = ChangesetError::CreationFailed {
        branch: "feat/new-feature".to_string(),
        reason: "No commits found".to_string(),
    };
    println!("  Creation failed: {}\n", creation_err.as_ref());

    // RegistryError examples
    println!("RegistryError variants:");
    let auth_err = RegistryError::AuthenticationFailed {
        registry: "https://registry.npmjs.org".to_string(),
        reason: "Invalid token".to_string(),
    };
    println!("  Authentication failed: {}", auth_err.as_ref());

    let publish_err = RegistryError::PublishFailed {
        package: "@myorg/my-package".to_string(),
        registry: "https://registry.npmjs.org".to_string(),
        reason: "Package already exists".to_string(),
    };
    println!("  Publish failed: {}", publish_err.as_ref());

    let network_err = RegistryError::NetworkFailed {
        registry: "https://registry.npmjs.org".to_string(),
        reason: "Connection timeout".to_string(),
    };
    println!("  Network failed: {}\n", network_err.as_ref());

    // DependencyError examples
    println!("DependencyError variants:");
    let circular_err = DependencyError::CircularDependency {
        cycle: vec!["pkg-a".to_string(), "pkg-b".to_string(), "pkg-a".to_string()],
    };
    println!("  Circular dependency: {}", circular_err.as_ref());

    let resolution_err = DependencyError::ResolutionFailed {
        package: "my-package".to_string(),
        reason: "Missing dependency".to_string(),
    };
    println!("  Resolution failed: {}", resolution_err.as_ref());

    let missing_dep_err = DependencyError::MissingDependency {
        package: "consumer-pkg".to_string(),
        dependency: "missing-dep".to_string(),
    };
    println!("  Missing dependency: {}\n", missing_dep_err.as_ref());

    // ReleaseError examples
    println!("ReleaseError variants:");
    let planning_err = ReleaseError::PlanningFailed { reason: "No changesets found".to_string() };
    println!("  Planning failed: {}", planning_err.as_ref());

    let execution_err = ReleaseError::ExecutionFailed {
        environment: "production".to_string(),
        reason: "Deploy script failed".to_string(),
    };
    println!("  Execution failed: {}", execution_err.as_ref());

    let tag_err = ReleaseError::TagCreationFailed {
        tag: "v1.0.0".to_string(),
        reason: "Tag already exists".to_string(),
    };
    println!("  Tag creation failed: {}\n", tag_err.as_ref());

    // ChangelogError examples
    println!("ChangelogError variants:");
    let generation_err =
        ChangelogError::GenerationFailed { reason: "No releases found".to_string() };
    println!("  Generation failed: {}", generation_err.as_ref());

    let template_err =
        ChangelogError::TemplateNotFound { template_path: PathBuf::from("changelog-template.hbs") };
    println!("  Template not found: {}", template_err.as_ref());

    let write_err = ChangelogError::WriteFileFailed {
        path: PathBuf::from("CHANGELOG.md"),
        reason: "Permission denied".to_string(),
    };
    println!("  Write file failed: {}\n", write_err.as_ref());

    // ConfigError examples
    println!("ConfigError variants:");
    let package_config_err = ConfigError::InvalidPackageConfig {
        field: "strategy".to_string(),
        reason: "Unknown strategy type".to_string(),
    };
    println!("  Invalid package config: {}", package_config_err.as_ref());

    let env_config_err = ConfigError::InvalidEnvironmentConfig {
        environment: "production".to_string(),
        reason: "Missing registry URL".to_string(),
    };
    println!("  Invalid environment config: {}", env_config_err.as_ref());

    let registry_config_err = ConfigError::InvalidRegistryConfig {
        registry: "npm-registry".to_string(),
        reason: "Invalid authentication type".to_string(),
    };
    println!("  Invalid registry config: {}\n", registry_config_err.as_ref());

    // ConventionalCommitError examples
    println!("ConventionalCommitError variants:");
    let format_err = ConventionalCommitError::InvalidFormat {
        commit: "bad commit message".to_string(),
        reason: "Missing type prefix".to_string(),
    };
    println!("  Invalid format: {}", format_err.as_ref());

    let unknown_type_err = ConventionalCommitError::UnknownType {
        commit_type: "unknown".to_string(),
        commit: "unknown: some change".to_string(),
    };
    println!("  Unknown type: {}", unknown_type_err.as_ref());

    let parse_commit_err = ConventionalCommitError::ParseFailed {
        commit: "malformed commit".to_string(),
        reason: "Invalid syntax".to_string(),
    };
    println!("  Parse failed: {}\n", parse_commit_err.as_ref());

    // CommitTypeParseError examples
    println!("CommitTypeParseError variants:");
    let empty_err = CommitTypeParseError::Empty;
    println!("  Empty: {}", empty_err.as_ref());

    let invalid_format_err = CommitTypeParseError::InvalidFormat("bad-format".to_string());
    println!("  Invalid format: {}", invalid_format_err.as_ref());

    println!("\n=== Summary ===");
    println!("All error types in sublime_pkg_tools implement AsRef<str>");
    println!("This provides a consistent way to identify error variants");
    println!("without needing to match on the full error structure.");
    println!("This is useful for logging, metrics, and error categorization.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_errors_implement_as_ref_str() {
        // Test that all error types implement AsRef<str>
        let version_err =
            VersionError::InvalidFormat { version: "test".to_string(), reason: "test".to_string() };
        assert_eq!(version_err.as_ref(), "VersionError::InvalidFormat");

        let changeset_err = ChangesetError::NotFound { path: PathBuf::from("test.json") };
        assert_eq!(changeset_err.as_ref(), "ChangesetError::NotFound");

        let registry_err = RegistryError::InvalidConfig { reason: "test".to_string() };
        assert_eq!(registry_err.as_ref(), "RegistryError::InvalidConfig");

        let dependency_err =
            DependencyError::CircularDependency { cycle: vec!["a".to_string(), "b".to_string()] };
        assert_eq!(dependency_err.as_ref(), "DependencyError::CircularDependency");

        let release_err = ReleaseError::PlanningFailed { reason: "test".to_string() };
        assert_eq!(release_err.as_ref(), "ReleaseError::PlanningFailed");

        let changelog_err = ChangelogError::GenerationFailed { reason: "test".to_string() };
        assert_eq!(changelog_err.as_ref(), "ChangelogError::GenerationFailed");

        let config_err = ConfigError::InvalidPackageConfig {
            field: "test".to_string(),
            reason: "test".to_string(),
        };
        assert_eq!(config_err.as_ref(), "ConfigError::InvalidPackageConfig");

        let conventional_err = ConventionalCommitError::InvalidFormat {
            commit: "test".to_string(),
            reason: "test".to_string(),
        };
        assert_eq!(conventional_err.as_ref(), "ConventionalCommitError::InvalidFormat");

        let commit_type_err = CommitTypeParseError::Empty;
        assert_eq!(commit_type_err.as_ref(), "CommitTypeParseError::Empty");

        let package_err = PackageError::Version(version_err);
        assert_eq!(package_err.as_ref(), "PackageError::Version");
    }

    #[test]
    fn test_as_ref_str_consistency() {
        // Test that AsRef<str> returns consistent identifiers
        let err1 = VersionError::InvalidFormat {
            version: "1.0.0".to_string(),
            reason: "test1".to_string(),
        };
        let err2 = VersionError::InvalidFormat {
            version: "2.0.0".to_string(),
            reason: "test2".to_string(),
        };

        // Same variant should return same string identifier
        assert_eq!(err1.as_ref(), err2.as_ref());
        assert_eq!(err1.as_ref(), "VersionError::InvalidFormat");

        let err3 = VersionError::Conflict {
            package: "pkg1".to_string(),
            current: "1.0.0".to_string(),
            requested: "2.0.0".to_string(),
        };

        // Different variant should return different string identifier
        assert_ne!(err1.as_ref(), err3.as_ref());
        assert_eq!(err3.as_ref(), "VersionError::Conflict");
    }
}
