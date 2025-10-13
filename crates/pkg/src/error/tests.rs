#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod error_tests {
    use crate::{
        error::{
            ChangelogError, ChangesetError, ConfigError, ConventionalCommitError, DependencyError,
            RegistryError, ReleaseError, VersionError,
        },
        PackageError, PackageResult,
    };
    use semver;
    use std::path::PathBuf;

    #[test]
    fn test_version_error_creation() {
        let error = VersionError::InvalidFormat {
            version: "not-a-version".to_string(),
            reason: "Invalid semver format".to_string(),
        };

        assert!(error.to_string().contains("not-a-version"));
        assert!(error.to_string().contains("Invalid semver format"));
    }

    #[test]
    fn test_changeset_error_creation() {
        let error = ChangesetError::NotFound { path: PathBuf::from("/path/to/changeset.json") };

        assert!(error.to_string().contains("/path/to/changeset.json"));
    }

    #[test]
    fn test_package_error_operation() {
        let error = PackageError::operation("test_operation", "test reason");

        match error {
            PackageError::Operation { operation, reason } => {
                assert_eq!(operation, "test_operation");
                assert_eq!(reason, "test reason");
            }
            _ => panic!("Expected Operation variant"),
        }
    }

    #[test]
    fn test_error_type_checks() {
        let version_error = PackageError::Version(VersionError::InvalidFormat {
            version: "test".to_string(),
            reason: "test".to_string(),
        });

        assert!(version_error.is_version_error());
        assert!(!version_error.is_changeset_error());
        assert!(!version_error.is_registry_error());
    }

    #[test]
    fn test_circular_dependency_error() {
        let error = DependencyError::CircularDependency {
            cycle: vec!["pkg-a".to_string(), "pkg-b".to_string(), "pkg-a".to_string()],
        };

        let error_string = error.to_string();
        assert!(error_string.contains("pkg-a"));
        assert!(error_string.contains("pkg-b"));
        assert!(error_string.contains("Circular dependency"));
    }

    #[test]
    fn test_registry_timeout_error() {
        let error = RegistryError::Timeout {
            registry: "https://registry.npmjs.org".to_string(),
            timeout_ms: 5000,
        };

        let error_string = error.to_string();
        assert!(error_string.contains("5000ms"));
        assert!(error_string.contains("timed out"));
    }

    #[test]
    fn test_version_error_variants() {
        // Test ParseFailed error
        let invalid_version = "1.0.0-invalid+";
        let semver_error = semver::Version::parse(invalid_version).unwrap_err();
        let parse_error = VersionError::ParseFailed {
            version: "1.0.0-invalid".to_string(),
            source: semver_error,
        };
        assert!(parse_error.to_string().contains("Failed to parse version"));
        assert!(parse_error.to_string().contains("1.0.0-invalid"));

        // Test SnapshotResolutionFailed error
        let snapshot_error = VersionError::SnapshotResolutionFailed {
            package: "test-package".to_string(),
            reason: "snapshot not found".to_string(),
        };
        assert!(snapshot_error.to_string().contains("Failed to resolve snapshot version"));
        assert!(snapshot_error.to_string().contains("test-package"));

        // Test Conflict error
        let conflict_error = VersionError::Conflict {
            package: "conflicted-package".to_string(),
            current: "1.0.0".to_string(),
            requested: "2.0.0".to_string(),
        };
        assert!(conflict_error.to_string().contains("Version conflict"));
        assert!(conflict_error.to_string().contains("conflicted-package"));
        assert!(conflict_error.to_string().contains("1.0.0"));
        assert!(conflict_error.to_string().contains("2.0.0"));
    }

    #[test]
    fn test_changeset_error_variants() {
        // Test InvalidFormat error
        let format_error = ChangesetError::InvalidFormat {
            path: PathBuf::from("/invalid/changeset.json"),
            reason: "missing required fields".to_string(),
        };
        assert!(format_error.to_string().contains("Invalid changeset format"));
        assert!(format_error.to_string().contains("/invalid/changeset.json"));

        // Test ValidationFailed error
        let validation_error = ChangesetError::ValidationFailed {
            changeset_id: "change-123".to_string(),
            errors: vec!["missing summary".to_string(), "invalid package".to_string()],
        };
        assert!(validation_error.to_string().contains("Changeset validation failed"));
        assert!(validation_error.to_string().contains("change-123"));

        // Test AlreadyExists error
        let exists_error =
            ChangesetError::AlreadyExists { changeset_id: "existing-change".to_string() };
        assert!(exists_error.to_string().contains("Changeset already exists"));
        assert!(exists_error.to_string().contains("existing-change"));
    }

    #[test]
    fn test_registry_error_variants() {
        // Test AuthenticationFailed error
        let auth_error = RegistryError::AuthenticationFailed {
            registry: "https://private.registry.com".to_string(),
            reason: "invalid token".to_string(),
        };
        assert!(auth_error.to_string().contains("Registry authentication failed"));
        assert!(auth_error.to_string().contains("https://private.registry.com"));

        // Test PackageNotFound error
        let not_found_error = RegistryError::PackageNotFound {
            package: "missing-package".to_string(),
            registry: "https://registry.npmjs.org".to_string(),
        };
        assert!(not_found_error.to_string().contains("not found"));
        assert!(not_found_error.to_string().contains("missing-package"));

        // Test PublishFailed error
        let publish_error = RegistryError::PublishFailed {
            package: "failed-package".to_string(),
            registry: "https://registry.npmjs.org".to_string(),
            reason: "version already exists".to_string(),
        };
        assert!(publish_error.to_string().contains("Failed to publish package"));
        assert!(publish_error.to_string().contains("failed-package"));
    }

    #[test]
    fn test_dependency_error_variants() {
        // Test ResolutionFailed error
        let resolution_error = DependencyError::ResolutionFailed {
            package: "unresolvable-package".to_string(),
            reason: "version constraints conflict".to_string(),
        };
        assert!(resolution_error.to_string().contains("Failed to resolve dependencies"));
        assert!(resolution_error.to_string().contains("unresolvable-package"));

        // Test MissingDependency error
        let missing_error = DependencyError::MissingDependency {
            package: "parent-package".to_string(),
            dependency: "missing-dep".to_string(),
        };
        assert!(missing_error.to_string().contains("Missing dependency"));
        assert!(missing_error.to_string().contains("parent-package"));
        assert!(missing_error.to_string().contains("missing-dep"));

        // Test InvalidSpecification error
        let spec_error = DependencyError::InvalidSpecification {
            package: "bad-spec-package".to_string(),
            spec: "invalid-spec".to_string(),
            reason: "malformed version range".to_string(),
        };
        assert!(spec_error.to_string().contains("Invalid dependency specification"));
        assert!(spec_error.to_string().contains("bad-spec-package"));
    }

    #[test]
    fn test_conventional_commit_error_variants() {
        // Test InvalidFormat error
        let format_error = ConventionalCommitError::InvalidFormat {
            commit: "bad commit message".to_string(),
            reason: "missing type and scope".to_string(),
        };
        assert!(format_error.to_string().contains("Invalid conventional commit format"));
        assert!(format_error.to_string().contains("bad commit message"));

        // Test UnknownType error
        let type_error = ConventionalCommitError::UnknownType {
            commit_type: "unknown".to_string(),
            commit: "unknown: some message".to_string(),
        };
        assert!(type_error.to_string().contains("Unknown commit type"));
        assert!(type_error.to_string().contains("unknown"));

        // Test ParseFailed error
        let parse_error = ConventionalCommitError::ParseFailed {
            commit: "malformed commit".to_string(),
            reason: "parsing error".to_string(),
        };
        assert!(parse_error.to_string().contains("Failed to parse commit"));
        assert!(parse_error.to_string().contains("malformed commit"));
    }

    #[test]
    fn test_release_error_variants() {
        // Test PlanningFailed error
        let planning_error =
            ReleaseError::PlanningFailed { reason: "missing changeset files".to_string() };
        assert!(planning_error.to_string().contains("Release planning failed"));
        assert!(planning_error.to_string().contains("missing changeset files"));

        // Test ExecutionFailed error
        let execution_error = ReleaseError::ExecutionFailed {
            environment: "production".to_string(),
            reason: "deployment failed".to_string(),
        };
        assert!(execution_error.to_string().contains("Release execution failed"));
        assert!(execution_error.to_string().contains("production"));

        // Test PackageReleaseFailed error
        let package_error = ReleaseError::PackageReleaseFailed {
            package: "failed-package".to_string(),
            environment: "staging".to_string(),
            reason: "build error".to_string(),
        };
        assert!(package_error.to_string().contains("Failed to release package"));
        assert!(package_error.to_string().contains("failed-package"));
        assert!(package_error.to_string().contains("staging"));
    }

    #[test]
    fn test_changelog_error_variants() {
        // Test GenerationFailed error
        let generation_error =
            ChangelogError::GenerationFailed { reason: "no release notes found".to_string() };
        assert!(generation_error.to_string().contains("Changelog generation failed"));
        assert!(generation_error.to_string().contains("no release notes found"));

        // Test TemplateNotFound error
        let template_error = ChangelogError::TemplateNotFound {
            template_path: PathBuf::from("/missing/template.hbs"),
        };
        assert!(template_error.to_string().contains("Changelog template not found"));
        assert!(template_error.to_string().contains("/missing/template.hbs"));

        // Test WriteFileFailed error
        let write_error = ChangelogError::WriteFileFailed {
            path: PathBuf::from("/protected/CHANGELOG.md"),
            reason: "permission denied".to_string(),
        };
        assert!(write_error.to_string().contains("Failed to write changelog to"));
        assert!(write_error.to_string().contains("/protected/CHANGELOG.md"));
    }

    #[test]
    fn test_config_error_variants() {
        // Test InvalidPackageConfig error
        let package_config_error = ConfigError::InvalidPackageConfig {
            field: "version_strategy".to_string(),
            reason: "unknown strategy type".to_string(),
        };
        assert!(package_config_error.to_string().contains("Invalid package tools configuration"));
        assert!(package_config_error.to_string().contains("version_strategy"));

        // Test InvalidEnvironmentConfig error
        let env_config_error = ConfigError::InvalidEnvironmentConfig {
            environment: "production".to_string(),
            reason: "missing registry URL".to_string(),
        };
        assert!(env_config_error.to_string().contains("Invalid environment configuration"));
        assert!(env_config_error.to_string().contains("production"));

        // Test InvalidRegistryConfig error
        let registry_config_error = ConfigError::InvalidRegistryConfig {
            registry: "private-registry".to_string(),
            reason: "invalid authentication method".to_string(),
        };
        assert!(registry_config_error.to_string().contains("Invalid registry configuration"));
        assert!(registry_config_error.to_string().contains("private-registry"));
    }

    #[test]
    fn test_package_error_type_check_methods() {
        // Test is_changeset_error
        let changeset_error = PackageError::Changeset(ChangesetError::NotFound {
            path: PathBuf::from("/test/changeset.json"),
        });
        assert!(changeset_error.is_changeset_error());
        assert!(!changeset_error.is_version_error());
        assert!(!changeset_error.is_registry_error());
        assert!(!changeset_error.is_dependency_error());
        assert!(!changeset_error.is_release_error());

        // Test is_registry_error
        let registry_error = PackageError::Registry(RegistryError::NetworkFailed {
            registry: "https://registry.npmjs.org".to_string(),
            reason: "connection timeout".to_string(),
        });
        assert!(registry_error.is_registry_error());
        assert!(!registry_error.is_version_error());
        assert!(!registry_error.is_changeset_error());
        assert!(!registry_error.is_dependency_error());
        assert!(!registry_error.is_release_error());

        // Test is_dependency_error
        let dependency_error =
            PackageError::Dependency(DependencyError::MaxDepthExceeded { max_depth: 100 });
        assert!(dependency_error.is_dependency_error());
        assert!(!dependency_error.is_version_error());
        assert!(!dependency_error.is_changeset_error());
        assert!(!dependency_error.is_registry_error());
        assert!(!dependency_error.is_release_error());

        // Test is_release_error
        let release_error = PackageError::Release(ReleaseError::StrategyNotSupported {
            strategy: "unsupported-strategy".to_string(),
        });
        assert!(release_error.is_release_error());
        assert!(!release_error.is_version_error());
        assert!(!release_error.is_changeset_error());
        assert!(!release_error.is_registry_error());
        assert!(!release_error.is_dependency_error());
    }

    #[test]
    fn test_package_result_type_alias() {
        // Test successful result
        let success: PackageResult<String> = Ok("success".to_string());
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), "success");

        // Test error result
        let error: PackageResult<String> = Err(PackageError::operation("test", "test error"));
        assert!(error.is_err());

        match error.unwrap_err() {
            PackageError::Operation { operation, reason } => {
                assert_eq!(operation, "test");
                assert_eq!(reason, "test error");
            }
            _ => panic!("Expected Operation error variant"),
        }
    }
}
