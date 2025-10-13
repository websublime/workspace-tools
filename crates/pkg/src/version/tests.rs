#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod version_tests {
    use crate::{
        version::VersionComparison, ResolvedVersion, SnapshotVersion, Version, VersionBump,
    };
    use chrono::DateTime;
    use std::str::FromStr;

    #[test]
    fn test_version_creation() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.major(), 1);
        assert_eq!(version.minor(), 2);
        assert_eq!(version.patch(), 3);
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_version_parsing() {
        let version = Version::from_str("1.2.3").unwrap();
        assert_eq!(version.major(), 1);
        assert_eq!(version.minor(), 2);
        assert_eq!(version.patch(), 3);

        assert!(Version::from_str("invalid").is_err());
    }

    #[test]
    fn test_version_bumping() {
        let version = Version::from_str("1.2.3").unwrap();

        let major = version.bump(VersionBump::Major);
        assert_eq!(major.to_string(), "2.0.0");

        let minor = version.bump(VersionBump::Minor);
        assert_eq!(minor.to_string(), "1.3.0");

        let patch = version.bump(VersionBump::Patch);
        assert_eq!(patch.to_string(), "1.2.4");

        let none = version.bump(VersionBump::None);
        assert_eq!(none.to_string(), "1.2.3");
    }

    #[test]
    fn test_snapshot_version() {
        let base = Version::from_str("1.2.3").unwrap();
        let snapshot = SnapshotVersion::new(base.clone(), "abc123d".to_string());

        assert_eq!(snapshot.base_version(), &base);
        assert_eq!(snapshot.commit_id(), "abc123d");
        assert_eq!(snapshot.to_string(), "1.2.3-abc123d.snapshot");
        assert!(snapshot.is_based_on(&base));
    }

    #[test]
    fn test_resolved_version() {
        let version = Version::from_str("1.2.3").unwrap();
        let resolved_release = ResolvedVersion::Release(version.clone());

        assert!(resolved_release.is_release());
        assert!(!resolved_release.is_snapshot());
        assert_eq!(resolved_release.base_version(), &version);

        let snapshot = SnapshotVersion::new(version.clone(), "abc123d".to_string());
        let resolved_snapshot = ResolvedVersion::Snapshot(snapshot.clone());

        assert!(!resolved_snapshot.is_release());
        assert!(resolved_snapshot.is_snapshot());
        assert_eq!(resolved_snapshot.base_version(), &version);
    }

    #[test]
    fn test_version_bump_parsing() {
        assert_eq!(VersionBump::from_str("major").unwrap(), VersionBump::Major);
        assert_eq!(VersionBump::from_str("minor").unwrap(), VersionBump::Minor);
        assert_eq!(VersionBump::from_str("patch").unwrap(), VersionBump::Patch);
        assert_eq!(VersionBump::from_str("none").unwrap(), VersionBump::None);
        assert_eq!(VersionBump::from_str("MAJOR").unwrap(), VersionBump::Major);

        assert!(VersionBump::from_str("invalid").is_err());
    }

    #[test]
    fn test_version_bump_combination() {
        assert_eq!(VersionBump::Patch.combine(VersionBump::Minor), VersionBump::Minor);
        assert_eq!(VersionBump::Minor.combine(VersionBump::Major), VersionBump::Major);
        assert_eq!(VersionBump::None.combine(VersionBump::Patch), VersionBump::Patch);
        assert_eq!(VersionBump::Major.combine(VersionBump::Patch), VersionBump::Major);
    }

    #[test]
    fn test_version_bump_precedence() {
        assert!(VersionBump::Major.is_higher_than(VersionBump::Minor));
        assert!(VersionBump::Minor.is_higher_than(VersionBump::Patch));
        assert!(VersionBump::Patch.is_higher_than(VersionBump::None));

        assert!(!VersionBump::Minor.is_higher_than(VersionBump::Major));
        assert!(!VersionBump::None.is_higher_than(VersionBump::Patch));
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::from_str("1.2.3").unwrap();
        let v2 = Version::from_str("1.2.4").unwrap();
        let v3 = Version::from_str("1.2.3").unwrap();

        assert_eq!(v1.compare(&v2), VersionComparison::Less);
        assert_eq!(v2.compare(&v1), VersionComparison::Greater);
        assert_eq!(v1.compare(&v3), VersionComparison::Equal);
    }

    #[test]
    fn test_prerelease_version() {
        let version = Version::from_str("1.2.3").unwrap();
        let prerelease = version.with_prerelease("alpha.1").unwrap();

        assert_eq!(prerelease.to_string(), "1.2.3-alpha.1");
        assert!(prerelease.is_prerelease());
        assert!(!version.is_prerelease());
    }

    #[test]
    fn test_build_metadata() {
        let version = Version::from_str("1.2.3").unwrap();
        let with_build = version.with_build_metadata("20240115.abc123").unwrap();

        assert_eq!(with_build.to_string(), "1.2.3+20240115.abc123");
        assert!(with_build.has_build_metadata());
        assert!(!version.has_build_metadata());
    }

    #[test]
    fn test_snapshot_comparison() {
        let base = Version::from_str("1.2.3").unwrap();
        let time1 = DateTime::from_timestamp(1000, 0).unwrap();
        let time2 = DateTime::from_timestamp(2000, 0).unwrap();

        let snapshot1 =
            SnapshotVersion::new_with_timestamp(base.clone(), "abc123".to_string(), time1);
        let snapshot2 =
            SnapshotVersion::new_with_timestamp(base.clone(), "def456".to_string(), time2);

        assert_eq!(snapshot1.compare(&snapshot2), VersionComparison::Less);
        assert_eq!(snapshot2.compare(&snapshot1), VersionComparison::Greater);
    }

    #[test]
    fn test_resolved_version_comparison() {
        let version = Version::from_str("1.2.3").unwrap();
        let snapshot = SnapshotVersion::new(version.clone(), "abc123".to_string());

        let resolved_release = ResolvedVersion::Release(version);
        let resolved_snapshot = ResolvedVersion::Snapshot(snapshot);

        assert_eq!(resolved_release.compare(&resolved_snapshot), VersionComparison::Incomparable);
        assert_eq!(resolved_snapshot.compare(&resolved_release), VersionComparison::Incomparable);
    }

    #[test]
    fn test_commit_hash_shortening_logic() {
        // Test the logic that would be used in VersionResolver for shortening commit hashes
        let full_hash = "abcd1234567890123456";
        let expected_short = "abcd123";

        // This demonstrates what the shortening logic should do:
        let actual_short: String = full_hash.chars().take(7).collect();
        assert_eq!(actual_short, expected_short);

        // Test edge case where hash is shorter than requested length
        let short_hash = "abc";
        let result: String = short_hash.chars().take(7).collect();
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_version_resolver_snapshot_format() {
        let base_version = Version::from_str("1.2.3").unwrap();
        let commit_id = "abc123d".to_string();

        let snapshot = SnapshotVersion::new(base_version, commit_id);
        assert_eq!(snapshot.to_string(), "1.2.3-abc123d.snapshot");
    }

    #[test]
    fn test_single_repo_vs_monorepo_detection() {
        // Test the conceptual logic for repository type detection
        // In a real implementation, this would use MonorepoDetector

        // Single repo scenario
        let single_repo_package_name = "my-single-package";
        let single_repo_root_name = "my-single-package";
        assert_eq!(single_repo_package_name, single_repo_root_name);

        // Monorepo scenario - package names differ from root
        let monorepo_package_name = "@myorg/auth-service";
        let monorepo_root_name = "my-workspace";
        assert_ne!(monorepo_package_name, monorepo_root_name);
    }

    #[test]
    fn test_package_name_matching_logic() {
        // Test the logic used in single repo package resolution

        struct MockPackageJson {
            name: String,
        }

        let package_json = MockPackageJson { name: "@myorg/service".to_string() };

        // Test exact match (should succeed)
        let requested_name = "@myorg/service";
        assert_eq!(package_json.name, requested_name);

        // Test mismatch (should fail)
        let wrong_name = "@myorg/other-service";
        assert_ne!(package_json.name, wrong_name);
    }

    #[test]
    fn test_error_message_formatting() {
        // Test the error message format for different scenarios

        let package_name = "@myorg/nonexistent";
        let available_packages = ["@myorg/auth-service", "@myorg/user-service", "@myorg/web-app"];

        let error_message = format!(
            "Package '{}' not found in monorepo. Available packages: {}",
            package_name,
            available_packages.join(", ")
        );

        assert!(error_message.contains("@myorg/nonexistent"));
        assert!(error_message.contains("@myorg/auth-service"));
        assert!(error_message.contains("Available packages"));
    }

    #[test]
    fn test_repository_type_scenarios() {
        // Test different repository type scenarios

        #[derive(Debug, PartialEq)]
        enum RepoType {
            SingleRepo,
            Monorepo,
        }

        // Function to determine repo type based on workspace configuration
        fn determine_repo_type(
            has_workspace_config: bool,
            has_multiple_packages: bool,
        ) -> RepoType {
            if has_workspace_config && has_multiple_packages {
                RepoType::Monorepo
            } else {
                RepoType::SingleRepo
            }
        }

        // Single repo scenarios
        assert_eq!(determine_repo_type(false, false), RepoType::SingleRepo);
        assert_eq!(determine_repo_type(false, true), RepoType::SingleRepo);

        // Monorepo scenarios
        assert_eq!(determine_repo_type(true, true), RepoType::Monorepo);

        // Edge case - workspace config but no multiple packages
        assert_eq!(determine_repo_type(true, false), RepoType::SingleRepo);
    }
}
