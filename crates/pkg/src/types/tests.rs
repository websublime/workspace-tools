//! Tests for version types.
//!
//! **What**: Comprehensive test suite for Version, VersionBump, and VersioningStrategy types,
//! including unit tests and property-based tests.
//!
//! **How**: Uses standard Rust testing with `#[test]` attributes and proptest for property-based
//! testing to verify parsing, bumping, comparison, serialization, and error handling.
//!
//! **Why**: To ensure 100% test coverage and validate that version operations work correctly
//! across all scenarios, including edge cases and invalid inputs.

#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use super::{Version, VersionBump, VersioningStrategy};
use crate::error::VersionError;
use package_json::PackageJson;
use proptest::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use sublime_standard_tools::monorepo::WorkspacePackage;

// =============================================================================
// Version Tests
// =============================================================================

#[test]
fn test_version_parse_valid() {
    let test_cases = vec![
        ("0.0.0", 0, 0, 0),
        ("1.2.3", 1, 2, 3),
        ("10.20.30", 10, 20, 30),
        ("999.999.999", 999, 999, 999),
    ];

    for (input, major, minor, patch) in test_cases {
        let version = Version::parse(input).unwrap();
        assert_eq!(version.major(), major);
        assert_eq!(version.minor(), minor);
        assert_eq!(version.patch(), patch);
        assert_eq!(version.to_string(), input);
    }
}

#[test]
fn test_version_parse_with_prerelease() {
    let test_cases = vec![
        ("1.0.0-alpha", "alpha"),
        ("1.0.0-alpha.1", "alpha.1"),
        ("1.0.0-0.3.7", "0.3.7"),
        ("1.0.0-beta", "beta"),
        ("1.0.0-rc.1", "rc.1"),
    ];

    for (input, expected_pre) in test_cases {
        let version = Version::parse(input).unwrap();
        assert_eq!(version.prerelease(), expected_pre);
        assert!(version.is_prerelease());
    }
}

#[test]
fn test_version_parse_with_build() {
    let test_cases = vec![
        ("1.0.0+build", "build"),
        ("1.0.0+20231201", "20231201"),
        ("1.0.0+build.123", "build.123"),
    ];

    for (input, expected_build) in test_cases {
        let version = Version::parse(input).unwrap();
        assert_eq!(version.build(), expected_build);
    }
}

#[test]
fn test_version_parse_with_prerelease_and_build() {
    let version = Version::parse("1.0.0-beta.1+build.123").unwrap();
    assert_eq!(version.major(), 1);
    assert_eq!(version.minor(), 0);
    assert_eq!(version.patch(), 0);
    assert_eq!(version.prerelease(), "beta.1");
    assert_eq!(version.build(), "build.123");
    assert!(version.is_prerelease());
}

#[test]
fn test_version_parse_invalid() {
    let invalid_cases = vec![
        "",               // Empty string
        "1",              // Missing components
        "1.2",            // Missing patch
        "1.2.3.4",        // Too many components
        "v1.2.3",         // Leading 'v'
        "1.2.x",          // Non-numeric component
        "a.b.c",          // Alphabetic components
        "1.2.-3",         // Negative number
        "1.2.3-",         // Trailing dash
        "1.2.3+",         // Trailing plus
        "1.2.3-@invalid", // Invalid prerelease
    ];

    for invalid in invalid_cases {
        let result = Version::parse(invalid);
        assert!(result.is_err(), "Expected error for: {}", invalid);
        match result {
            Err(VersionError::ParseError { version, .. }) => {
                assert_eq!(version, invalid);
            }
            _ => panic!("Expected ParseError for: {}", invalid),
        }
    }
}

#[test]
fn test_version_new() {
    let version = Version::new(1, 2, 3);
    assert_eq!(version.major(), 1);
    assert_eq!(version.minor(), 2);
    assert_eq!(version.patch(), 3);
    assert_eq!(version.to_string(), "1.2.3");
    assert!(!version.is_prerelease());
}

#[test]
fn test_version_bump_major() {
    let version = Version::parse("1.2.3").unwrap();
    let bumped = version.bump(VersionBump::Major).unwrap();
    assert_eq!(bumped.to_string(), "2.0.0");
}

#[test]
fn test_version_bump_minor() {
    let version = Version::parse("1.2.3").unwrap();
    let bumped = version.bump(VersionBump::Minor).unwrap();
    assert_eq!(bumped.to_string(), "1.3.0");
}

#[test]
fn test_version_bump_patch() {
    let version = Version::parse("1.2.3").unwrap();
    let bumped = version.bump(VersionBump::Patch).unwrap();
    assert_eq!(bumped.to_string(), "1.2.4");
}

#[test]
fn test_version_bump_none() {
    let version = Version::parse("1.2.3").unwrap();
    let bumped = version.bump(VersionBump::None).unwrap();
    assert_eq!(bumped.to_string(), "1.2.3");
}

#[test]
fn test_version_bump_removes_prerelease() {
    let version = Version::parse("1.0.0-beta.1").unwrap();

    let major = version.bump(VersionBump::Major).unwrap();
    assert_eq!(major.to_string(), "2.0.0");
    assert!(!major.is_prerelease());

    let minor = version.bump(VersionBump::Minor).unwrap();
    assert_eq!(minor.to_string(), "1.1.0");
    assert!(!minor.is_prerelease());

    let patch = version.bump(VersionBump::Patch).unwrap();
    assert_eq!(patch.to_string(), "1.0.1");
    assert!(!patch.is_prerelease());
}

#[test]
fn test_version_bump_removes_build() {
    let version = Version::parse("1.0.0+build.123").unwrap();
    let bumped = version.bump(VersionBump::Patch).unwrap();
    assert_eq!(bumped.to_string(), "1.0.1");
    assert!(bumped.build().is_empty());
}

#[test]
fn test_version_bump_overflow_protection() {
    // Test major overflow
    let max_version = Version::new(u64::MAX, 0, 0);
    let result = max_version.bump(VersionBump::Major);
    assert!(result.is_err());
    match result {
        Err(VersionError::InvalidVersion { reason, .. }) => {
            assert!(reason.contains("overflow"));
        }
        _ => panic!("Expected InvalidVersion error"),
    }

    // Test minor overflow
    let max_minor = Version::new(1, u64::MAX, 0);
    let result = max_minor.bump(VersionBump::Minor);
    assert!(result.is_err());

    // Test patch overflow
    let max_patch = Version::new(1, 2, u64::MAX);
    let result = max_patch.bump(VersionBump::Patch);
    assert!(result.is_err());
}

#[test]
fn test_version_comparison() {
    let v1 = Version::parse("1.0.0").unwrap();
    let v2 = Version::parse("1.0.1").unwrap();
    let v3 = Version::parse("1.1.0").unwrap();
    let v4 = Version::parse("2.0.0").unwrap();
    let v5 = Version::parse("1.0.0").unwrap();

    // Less than
    assert!(v1 < v2);
    assert!(v2 < v3);
    assert!(v3 < v4);

    // Greater than
    assert!(v4 > v3);
    assert!(v3 > v2);
    assert!(v2 > v1);

    // Equal
    assert_eq!(v1, v5);
    assert!(!v1.lt(&v5));
    assert!(!v1.gt(&v5));
}

#[test]
fn test_version_comparison_with_prerelease() {
    let stable = Version::parse("1.0.0").unwrap();
    let alpha = Version::parse("1.0.0-alpha").unwrap();
    let beta = Version::parse("1.0.0-beta").unwrap();

    // Prerelease versions are less than stable
    assert!(alpha < stable);
    assert!(beta < stable);

    // Prerelease versions can be compared
    assert!(alpha < beta);
}

#[test]
fn test_version_snapshot_basic() {
    let version = Version::parse("1.2.3").unwrap();
    let snapshot = version.snapshot("abc123def456").unwrap();

    let snapshot_str = snapshot.to_string();
    assert!(snapshot_str.starts_with("1.2.3-snapshot-"));
    assert!(snapshot_str.contains("-abc123d")); // First 7 chars
    assert!(snapshot.is_prerelease());
}

#[test]
fn test_version_snapshot_short_hash() {
    let version = Version::parse("1.0.0").unwrap();
    let snapshot = version.snapshot("abc").unwrap();

    let snapshot_str = snapshot.to_string();
    assert!(snapshot_str.starts_with("1.0.0-snapshot-"));
    assert!(snapshot_str.contains("-abc"));
}

#[test]
fn test_version_snapshot_empty_hash_error() {
    let version = Version::parse("1.0.0").unwrap();
    let result = version.snapshot("");
    assert!(result.is_err());
    match result {
        Err(VersionError::SnapshotFailed { reason, .. }) => {
            assert!(reason.contains("empty"));
        }
        _ => panic!("Expected SnapshotFailed error"),
    }
}

#[test]
fn test_version_snapshot_format() {
    let version = Version::parse("2.5.7").unwrap();
    let snapshot = version.snapshot("1234567890").unwrap();

    // Verify it can be parsed as a valid version
    let snapshot_str = snapshot.to_string();
    let reparsed = Version::parse(&snapshot_str).unwrap();
    assert_eq!(reparsed.major(), 2);
    assert_eq!(reparsed.minor(), 5);
    assert_eq!(reparsed.patch(), 7);
    assert!(reparsed.is_prerelease());
}

#[test]
fn test_version_from_str() {
    let version: Version = "1.2.3".parse().unwrap();
    assert_eq!(version.to_string(), "1.2.3");

    let result: Result<Version, _> = "invalid".parse();
    assert!(result.is_err());
}

#[test]
fn test_version_display() {
    let version = Version::parse("1.2.3-beta.1+build").unwrap();
    assert_eq!(format!("{}", version), "1.2.3-beta.1+build");
}

#[test]
fn test_version_serialization() {
    let version = Version::parse("1.2.3").unwrap();
    let json = serde_json::to_string(&version).unwrap();
    assert_eq!(json, r#""1.2.3""#);

    let deserialized: Version = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, version);
}

#[test]
fn test_version_serialization_with_prerelease() {
    let version = Version::parse("1.0.0-beta.1").unwrap();
    let json = serde_json::to_string(&version).unwrap();
    assert_eq!(json, r#""1.0.0-beta.1""#);

    let deserialized: Version = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, version);
}

#[test]
fn test_version_as_semver() {
    let version = Version::parse("1.2.3").unwrap();
    let semver = version.as_semver();
    assert_eq!(semver.major, 1);
    assert_eq!(semver.minor, 2);
    assert_eq!(semver.patch, 3);
}

// =============================================================================
// VersionBump Tests
// =============================================================================

#[test]
fn test_version_bump_parse_valid() {
    assert_eq!(VersionBump::parse("major").unwrap(), VersionBump::Major);
    assert_eq!(VersionBump::parse("minor").unwrap(), VersionBump::Minor);
    assert_eq!(VersionBump::parse("patch").unwrap(), VersionBump::Patch);
    assert_eq!(VersionBump::parse("none").unwrap(), VersionBump::None);
}

#[test]
fn test_version_bump_parse_case_insensitive() {
    assert_eq!(VersionBump::parse("MAJOR").unwrap(), VersionBump::Major);
    assert_eq!(VersionBump::parse("Minor").unwrap(), VersionBump::Minor);
    assert_eq!(VersionBump::parse("PaTcH").unwrap(), VersionBump::Patch);
    assert_eq!(VersionBump::parse("NoNe").unwrap(), VersionBump::None);
}

#[test]
fn test_version_bump_parse_invalid() {
    let invalid_cases = vec!["", "invalid", "maj", "min", "pat", "medium", "big"];

    for invalid in invalid_cases {
        let result = VersionBump::parse(invalid);
        assert!(result.is_err(), "Expected error for: {}", invalid);
        match result {
            Err(VersionError::InvalidBump { bump, reason }) => {
                assert_eq!(bump, invalid);
                assert!(reason.contains("expected"));
            }
            _ => panic!("Expected InvalidBump error for: {}", invalid),
        }
    }
}

#[test]
fn test_version_bump_as_str() {
    assert_eq!(VersionBump::Major.as_str(), "major");
    assert_eq!(VersionBump::Minor.as_str(), "minor");
    assert_eq!(VersionBump::Patch.as_str(), "patch");
    assert_eq!(VersionBump::None.as_str(), "none");
}

#[test]
fn test_version_bump_display() {
    assert_eq!(format!("{}", VersionBump::Major), "major");
    assert_eq!(format!("{}", VersionBump::Minor), "minor");
    assert_eq!(format!("{}", VersionBump::Patch), "patch");
    assert_eq!(format!("{}", VersionBump::None), "none");
}

#[test]
fn test_version_bump_from_str() {
    let bump: VersionBump = "major".parse().unwrap();
    assert_eq!(bump, VersionBump::Major);

    let result: Result<VersionBump, _> = "invalid".parse();
    assert!(result.is_err());
}

#[test]
fn test_version_bump_serialization() {
    let bump = VersionBump::Major;
    let json = serde_json::to_string(&bump).unwrap();
    assert_eq!(json, r#""major""#);

    let deserialized: VersionBump = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, bump);
}

#[test]
fn test_version_bump_all_variants_serialization() {
    let variants = vec![
        (VersionBump::Major, r#""major""#),
        (VersionBump::Minor, r#""minor""#),
        (VersionBump::Patch, r#""patch""#),
        (VersionBump::None, r#""none""#),
    ];

    for (bump, expected_json) in variants {
        let json = serde_json::to_string(&bump).unwrap();
        assert_eq!(json, expected_json);

        let deserialized: VersionBump = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, bump);
    }
}

#[test]
fn test_version_bump_clone_copy() {
    let bump = VersionBump::Major;
    let cloned = bump;
    assert_eq!(bump, cloned);
}

// =============================================================================
// VersioningStrategy Tests
// =============================================================================

#[test]
fn test_versioning_strategy_parse_valid() {
    assert_eq!(VersioningStrategy::parse("independent").unwrap(), VersioningStrategy::Independent);
    assert_eq!(VersioningStrategy::parse("unified").unwrap(), VersioningStrategy::Unified);
}

#[test]
fn test_versioning_strategy_parse_case_insensitive() {
    assert_eq!(VersioningStrategy::parse("INDEPENDENT").unwrap(), VersioningStrategy::Independent);
    assert_eq!(VersioningStrategy::parse("Unified").unwrap(), VersioningStrategy::Unified);
    assert_eq!(VersioningStrategy::parse("InDePeNdEnT").unwrap(), VersioningStrategy::Independent);
}

#[test]
fn test_versioning_strategy_parse_invalid() {
    let invalid_cases = vec!["", "invalid", "mixed", "separate", "together"];

    for invalid in invalid_cases {
        let result = VersioningStrategy::parse(invalid);
        assert!(result.is_err(), "Expected error for: {}", invalid);
        match result {
            Err(VersionError::InvalidStrategy { strategy }) => {
                assert_eq!(strategy, invalid);
            }
            _ => panic!("Expected InvalidStrategy error for: {}", invalid),
        }
    }
}

#[test]
fn test_versioning_strategy_as_str() {
    assert_eq!(VersioningStrategy::Independent.as_str(), "independent");
    assert_eq!(VersioningStrategy::Unified.as_str(), "unified");
}

#[test]
fn test_versioning_strategy_display() {
    assert_eq!(format!("{}", VersioningStrategy::Independent), "independent");
    assert_eq!(format!("{}", VersioningStrategy::Unified), "unified");
}

#[test]
fn test_versioning_strategy_from_str() {
    let strategy: VersioningStrategy = "independent".parse().unwrap();
    assert_eq!(strategy, VersioningStrategy::Independent);

    let result: Result<VersioningStrategy, _> = "invalid".parse();
    assert!(result.is_err());
}

#[test]
fn test_versioning_strategy_is_independent() {
    assert!(VersioningStrategy::Independent.is_independent());
    assert!(!VersioningStrategy::Unified.is_independent());
}

#[test]
fn test_versioning_strategy_is_unified() {
    assert!(VersioningStrategy::Unified.is_unified());
    assert!(!VersioningStrategy::Independent.is_unified());
}

#[test]
fn test_versioning_strategy_default() {
    let default_strategy = VersioningStrategy::default();
    assert_eq!(default_strategy, VersioningStrategy::Independent);
}

#[test]
fn test_versioning_strategy_serialization() {
    let strategy = VersioningStrategy::Independent;
    let json = serde_json::to_string(&strategy).unwrap();
    assert_eq!(json, r#""independent""#);

    let deserialized: VersioningStrategy = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, strategy);
}

#[test]
fn test_versioning_strategy_all_variants_serialization() {
    let variants = vec![
        (VersioningStrategy::Independent, r#""independent""#),
        (VersioningStrategy::Unified, r#""unified""#),
    ];

    for (strategy, expected_json) in variants {
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, expected_json);

        let deserialized: VersioningStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, strategy);
    }
}

// =============================================================================
// Property-Based Tests
// =============================================================================

proptest! {
    /// Property: Any valid semver string can be parsed and roundtrips correctly
    #[test]
    fn prop_version_parse_roundtrip(major in 0u64..1000, minor in 0u64..1000, patch in 0u64..1000) {
        let version_str = format!("{}.{}.{}", major, minor, patch);
        let version = Version::parse(&version_str).unwrap();
        assert_eq!(version.to_string(), version_str);
        assert_eq!(version.major(), major);
        assert_eq!(version.minor(), minor);
        assert_eq!(version.patch(), patch);
    }

    /// Property: Bumping major always results in X+1.0.0
    #[test]
    fn prop_bump_major(major in 0u64..100, minor in 0u64..100, patch in 0u64..100) {
        let version = Version::new(major, minor, patch);
        let bumped = version.bump(VersionBump::Major).unwrap();
        assert_eq!(bumped.major(), major + 1);
        assert_eq!(bumped.minor(), 0);
        assert_eq!(bumped.patch(), 0);
    }

    /// Property: Bumping minor always results in X.Y+1.0
    #[test]
    fn prop_bump_minor(major in 0u64..100, minor in 0u64..100, patch in 0u64..100) {
        let version = Version::new(major, minor, patch);
        let bumped = version.bump(VersionBump::Minor).unwrap();
        assert_eq!(bumped.major(), major);
        assert_eq!(bumped.minor(), minor + 1);
        assert_eq!(bumped.patch(), 0);
    }

    /// Property: Bumping patch always results in X.Y.Z+1
    #[test]
    fn prop_bump_patch(major in 0u64..100, minor in 0u64..100, patch in 0u64..100) {
        let version = Version::new(major, minor, patch);
        let bumped = version.bump(VersionBump::Patch).unwrap();
        assert_eq!(bumped.major(), major);
        assert_eq!(bumped.minor(), minor);
        assert_eq!(bumped.patch(), patch + 1);
    }

    /// Property: Bumping none never changes the version
    #[test]
    fn prop_bump_none_no_change(major in 0u64..1000, minor in 0u64..1000, patch in 0u64..1000) {
        let version = Version::new(major, minor, patch);
        let bumped = version.bump(VersionBump::None).unwrap();
        assert_eq!(bumped, version);
    }

    /// Property: Version comparison is transitive
    #[test]
    fn prop_version_comparison_transitive(
        v1 in 0u64..10,
        v2 in 10u64..20,
        v3 in 20u64..30
    ) {
        let a = Version::new(v1, 0, 0);
        let b = Version::new(v2, 0, 0);
        let c = Version::new(v3, 0, 0);
        assert!(a < b);
        assert!(b < c);
        assert!(a < c); // Transitivity
    }

    /// Property: Version equality is reflexive
    #[test]
    fn prop_version_equality_reflexive(major in 0u64..100, minor in 0u64..100, patch in 0u64..100) {
        let version = Version::new(major, minor, patch);
        assert_eq!(version, version.clone());
    }

    /// Property: Snapshot versions are always prerelease
    #[test]
    fn prop_snapshot_is_prerelease(
        major in 0u64..100,
        minor in 0u64..100,
        patch in 0u64..100,
        hash in "[a-f0-9]{40}"
    ) {
        let version = Version::new(major, minor, patch);
        let snapshot = version.snapshot(&hash).unwrap();
        assert!(snapshot.is_prerelease());
        assert_eq!(snapshot.major(), major);
        assert_eq!(snapshot.minor(), minor);
        assert_eq!(snapshot.patch(), patch);
    }

    /// Property: Serialization roundtrip preserves version
    #[test]
    fn prop_serialization_roundtrip(major in 0u64..100, minor in 0u64..100, patch in 0u64..100) {
        let version = Version::new(major, minor, patch);
        let json = serde_json::to_string(&version).unwrap();
        let deserialized: Version = serde_json::from_str(&json).unwrap();
        assert_eq!(version, deserialized);
    }
}

// ============================================================================
// PackageInfo Tests
// ============================================================================

mod package_info_tests {
    use super::*;
    use crate::types::{DependencyType, PackageInfo};

    /// Helper function to create a minimal PackageJson for testing
    fn create_test_package_json(name: &str, version: &str) -> PackageJson {
        PackageJson {
            name: name.to_string(),
            version: version.to_string(),
            ..PackageJson::default()
        }
    }

    /// Helper function to create a PackageJson with dependencies
    fn create_package_json_with_deps(
        name: &str,
        version: &str,
        deps: Vec<(&str, &str)>,
        dev_deps: Vec<(&str, &str)>,
    ) -> PackageJson {
        let mut pkg = create_test_package_json(name, version);

        if !deps.is_empty() {
            let mut dependencies = HashMap::new();
            for (dep_name, dep_version) in deps {
                dependencies.insert(dep_name.to_string(), dep_version.to_string());
            }
            pkg.dependencies = Some(dependencies);
        }

        if !dev_deps.is_empty() {
            let mut dev_dependencies = HashMap::new();
            for (dep_name, dep_version) in dev_deps {
                dev_dependencies.insert(dep_name.to_string(), dep_version.to_string());
            }
            pkg.dev_dependencies = Some(dev_dependencies);
        }

        pkg
    }

    /// Helper function to create a WorkspacePackage for testing
    fn create_test_workspace_package(
        name: &str,
        version: &str,
        workspace_deps: Vec<String>,
    ) -> WorkspacePackage {
        WorkspacePackage {
            name: name.to_string(),
            version: version.to_string(),
            location: PathBuf::from(format!("packages/{}", name)),
            absolute_path: PathBuf::from(format!("/workspace/packages/{}", name)),
            workspace_dependencies: workspace_deps,
            workspace_dev_dependencies: vec![],
        }
    }

    #[test]
    fn test_package_info_new() {
        let pkg_json = create_test_package_json("test-package", "1.0.0");
        let path = PathBuf::from("/path/to/package");

        let info = PackageInfo::new(pkg_json, None, path.clone());

        assert_eq!(info.name(), "test-package");
        assert_eq!(info.version().to_string(), "1.0.0");
        assert_eq!(info.path(), &path);
        assert!(info.workspace().is_none());
    }

    #[test]
    fn test_package_info_with_workspace() {
        let pkg_json = create_test_package_json("test-package", "1.0.0");
        let workspace = create_test_workspace_package("test-package", "1.0.0", vec![]);
        let path = PathBuf::from("/path/to/package");

        let info = PackageInfo::new(pkg_json, Some(workspace), path);

        assert_eq!(info.name(), "test-package");
        assert!(info.workspace().is_some());
    }

    #[test]
    fn test_package_info_name() {
        let pkg_json = create_test_package_json("my-awesome-package", "2.3.4");
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        assert_eq!(info.name(), "my-awesome-package");
    }

    #[test]
    fn test_package_info_name_empty() {
        let pkg_json = PackageJson::default();
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        assert_eq!(info.name(), "");
    }

    #[test]
    fn test_package_info_version() {
        let pkg_json = create_test_package_json("test", "3.2.1");
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        assert_eq!(info.version().to_string(), "3.2.1");
    }

    #[test]
    fn test_package_info_version_invalid_defaults_to_0_0_0() {
        let pkg_json = PackageJson {
            name: "test".to_string(),
            version: "not-a-valid-version".to_string(),
            ..PackageJson::default()
        };
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        assert_eq!(info.version().to_string(), "0.0.0");
    }

    #[test]
    fn test_package_info_version_empty_defaults_to_0_0_0() {
        let pkg_json = PackageJson {
            name: "test".to_string(),
            version: "".to_string(),
            ..PackageJson::default()
        };
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        assert_eq!(info.version().to_string(), "0.0.0");
    }

    #[test]
    fn test_package_info_all_dependencies_empty() {
        let pkg_json = create_test_package_json("test", "1.0.0");
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let deps = info.all_dependencies();
        assert!(deps.is_empty());
    }

    #[test]
    fn test_package_info_all_dependencies_with_regular_deps() {
        let pkg_json = create_package_json_with_deps(
            "test",
            "1.0.0",
            vec![("lodash", "^4.17.21"), ("react", "^18.0.0")],
            vec![],
        );
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let deps = info.all_dependencies();
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|(name, version, dep_type)| name == "lodash"
            && version == "^4.17.21"
            && *dep_type == DependencyType::Regular));
        assert!(deps.iter().any(|(name, version, dep_type)| name == "react"
            && version == "^18.0.0"
            && *dep_type == DependencyType::Regular));
    }

    #[test]
    fn test_package_info_all_dependencies_with_dev_deps() {
        let pkg_json = create_package_json_with_deps(
            "test",
            "1.0.0",
            vec![],
            vec![("jest", "^29.0.0"), ("eslint", "^8.0.0")],
        );
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let deps = info.all_dependencies();
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|(name, version, dep_type)| name == "jest"
            && version == "^29.0.0"
            && *dep_type == DependencyType::Dev));
        assert!(deps.iter().any(|(name, version, dep_type)| name == "eslint"
            && version == "^8.0.0"
            && *dep_type == DependencyType::Dev));
    }

    #[test]
    fn test_package_info_all_dependencies_mixed() {
        let pkg_json = create_package_json_with_deps(
            "test",
            "1.0.0",
            vec![("lodash", "^4.17.21")],
            vec![("jest", "^29.0.0")],
        );
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let deps = info.all_dependencies();
        assert_eq!(deps.len(), 2);
        assert!(deps
            .iter()
            .any(|(name, _, dep_type)| name == "lodash" && *dep_type == DependencyType::Regular));
        assert!(deps
            .iter()
            .any(|(name, _, dep_type)| name == "jest" && *dep_type == DependencyType::Dev));
    }

    #[test]
    fn test_package_info_all_dependencies_filters_workspace_protocol() {
        let pkg_json = create_package_json_with_deps(
            "test",
            "1.0.0",
            vec![("lodash", "^4.17.21"), ("@myorg/core", "workspace:*"), ("react", "^18.0.0")],
            vec![],
        );
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let deps = info.all_dependencies();
        assert_eq!(deps.len(), 2);
        assert!(!deps.iter().any(|(name, _, _)| name == "@myorg/core"));
    }

    #[test]
    fn test_package_info_all_dependencies_filters_file_protocol() {
        let pkg_json = create_package_json_with_deps(
            "test",
            "1.0.0",
            vec![("local-lib", "file:../local-lib"), ("lodash", "^4.17.21")],
            vec![],
        );
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let deps = info.all_dependencies();
        assert_eq!(deps.len(), 1);
        assert!(!deps.iter().any(|(name, _, _)| name == "local-lib"));
    }

    #[test]
    fn test_package_info_all_dependencies_filters_link_protocol() {
        let pkg_json = create_package_json_with_deps(
            "test",
            "1.0.0",
            vec![("linked-lib", "link:../linked"), ("lodash", "^4.17.21")],
            vec![],
        );
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let deps = info.all_dependencies();
        assert_eq!(deps.len(), 1);
        assert!(!deps.iter().any(|(name, _, _)| name == "linked-lib"));
    }

    #[test]
    fn test_package_info_all_dependencies_filters_portal_protocol() {
        let pkg_json = create_package_json_with_deps(
            "test",
            "1.0.0",
            vec![("portal-lib", "portal:../portal"), ("lodash", "^4.17.21")],
            vec![],
        );
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let deps = info.all_dependencies();
        assert_eq!(deps.len(), 1);
        assert!(!deps.iter().any(|(name, _, _)| name == "portal-lib"));
    }

    #[test]
    fn test_package_info_dependencies() {
        let pkg_json = create_package_json_with_deps(
            "test",
            "1.0.0",
            vec![("lodash", "^4.17.21"), ("react", "^18.0.0")],
            vec![("jest", "^29.0.0")],
        );
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let deps = info.dependencies();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps.get("lodash"), Some(&"^4.17.21".to_string()));
        assert_eq!(deps.get("react"), Some(&"^18.0.0".to_string()));
        assert!(!deps.contains_key("jest"));
    }

    #[test]
    fn test_package_info_dev_dependencies() {
        let pkg_json = create_package_json_with_deps(
            "test",
            "1.0.0",
            vec![("lodash", "^4.17.21")],
            vec![("jest", "^29.0.0"), ("eslint", "^8.0.0")],
        );
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let dev_deps = info.dev_dependencies();
        assert_eq!(dev_deps.len(), 2);
        assert_eq!(dev_deps.get("jest"), Some(&"^29.0.0".to_string()));
        assert_eq!(dev_deps.get("eslint"), Some(&"^8.0.0".to_string()));
        assert!(!dev_deps.contains_key("lodash"));
    }

    #[test]
    fn test_package_info_peer_dependencies() {
        let mut pkg_json = create_test_package_json("test", "1.0.0");
        let mut peer_deps = HashMap::new();
        peer_deps.insert("react".to_string(), ">=16.0.0".to_string());
        pkg_json.peer_dependencies = Some(peer_deps);

        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let peer_deps = info.peer_dependencies();
        assert_eq!(peer_deps.len(), 1);
        assert_eq!(peer_deps.get("react"), Some(&">=16.0.0".to_string()));
    }

    #[test]
    fn test_package_info_optional_dependencies() {
        let mut pkg_json = create_test_package_json("test", "1.0.0");
        let mut optional_deps = HashMap::new();
        optional_deps.insert("fsevents".to_string(), "^2.3.0".to_string());
        pkg_json.optional_dependencies = Some(optional_deps);

        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let optional_deps = info.optional_dependencies();
        assert_eq!(optional_deps.len(), 1);
        assert_eq!(optional_deps.get("fsevents"), Some(&"^2.3.0".to_string()));
    }

    #[test]
    fn test_package_info_is_internal_no_workspace() {
        let pkg_json = create_test_package_json("test", "1.0.0");
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        assert!(!info.is_internal());
    }

    #[test]
    fn test_package_info_is_internal_with_empty_workspace_deps() {
        let pkg_json = create_test_package_json("test", "1.0.0");
        let workspace = create_test_workspace_package("test", "1.0.0", vec![]);
        let info = PackageInfo::new(pkg_json, Some(workspace), PathBuf::from("."));

        assert!(!info.is_internal());
    }

    #[test]
    fn test_package_info_is_internal_with_workspace_deps() {
        let pkg_json = create_test_package_json("test", "1.0.0");
        let workspace = create_test_workspace_package("test", "1.0.0", vec!["core".to_string()]);
        let info = PackageInfo::new(pkg_json, Some(workspace), PathBuf::from("."));

        assert!(info.is_internal());
    }

    #[test]
    fn test_package_info_internal_dependencies_no_workspace() {
        let pkg_json = create_test_package_json("test", "1.0.0");
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let internal = info.internal_dependencies();
        assert!(internal.is_empty());
    }

    #[test]
    fn test_package_info_internal_dependencies_with_workspace() {
        let pkg_json = create_test_package_json("test", "1.0.0");
        let workspace = create_test_workspace_package(
            "test",
            "1.0.0",
            vec!["core".to_string(), "utils".to_string()],
        );
        let info = PackageInfo::new(pkg_json, Some(workspace), PathBuf::from("."));

        let internal = info.internal_dependencies();
        assert_eq!(internal.len(), 2);
        assert!(internal.contains(&"core".to_string()));
        assert!(internal.contains(&"utils".to_string()));
    }

    #[test]
    fn test_package_info_external_dependencies() {
        let pkg_json = create_package_json_with_deps(
            "test",
            "1.0.0",
            vec![("lodash", "^4.17.21"), ("react", "^18.0.0"), ("core", "^1.0.0")],
            vec![],
        );
        let workspace = create_test_workspace_package("test", "1.0.0", vec!["core".to_string()]);
        let info = PackageInfo::new(pkg_json, Some(workspace), PathBuf::from("."));

        let external = info.external_dependencies();
        assert_eq!(external.len(), 2);
        assert!(external.contains_key("lodash"));
        assert!(external.contains_key("react"));
        assert!(!external.contains_key("core")); // Internal dependency
    }

    #[test]
    fn test_package_info_is_skipped_version_spec_workspace() {
        assert!(PackageInfo::is_skipped_version_spec("workspace:*"));
        assert!(PackageInfo::is_skipped_version_spec("workspace:^1.0.0"));
        assert!(PackageInfo::is_skipped_version_spec("workspace:~"));
    }

    #[test]
    fn test_package_info_is_skipped_version_spec_file() {
        assert!(PackageInfo::is_skipped_version_spec("file:../local"));
        assert!(PackageInfo::is_skipped_version_spec("file:./relative"));
        assert!(PackageInfo::is_skipped_version_spec("file:/absolute/path"));
    }

    #[test]
    fn test_package_info_is_skipped_version_spec_link() {
        assert!(PackageInfo::is_skipped_version_spec("link:../linked"));
        assert!(PackageInfo::is_skipped_version_spec("link:./local"));
    }

    #[test]
    fn test_package_info_is_skipped_version_spec_portal() {
        assert!(PackageInfo::is_skipped_version_spec("portal:../portal"));
        assert!(PackageInfo::is_skipped_version_spec("portal:./local"));
    }

    #[test]
    fn test_package_info_is_skipped_version_spec_normal_versions() {
        assert!(!PackageInfo::is_skipped_version_spec("^1.2.3"));
        assert!(!PackageInfo::is_skipped_version_spec("~2.0.0"));
        assert!(!PackageInfo::is_skipped_version_spec(">=3.0.0"));
        assert!(!PackageInfo::is_skipped_version_spec("1.0.0"));
        assert!(!PackageInfo::is_skipped_version_spec("*"));
        assert!(!PackageInfo::is_skipped_version_spec("latest"));
    }

    #[test]
    fn test_package_info_package_json_accessor() {
        let pkg_json = create_test_package_json("test", "1.0.0");
        let info = PackageInfo::new(pkg_json, None, PathBuf::from("."));

        let retrieved = info.package_json();
        assert_eq!(retrieved.name, "test");
        assert_eq!(retrieved.version, "1.0.0");
    }
}

// ============================================================================
// DependencyType Tests
// ============================================================================

mod dependency_type_tests {
    use crate::types::DependencyType;

    #[test]
    fn test_dependency_type_as_str() {
        assert_eq!(DependencyType::Regular.as_str(), "dependencies");
        assert_eq!(DependencyType::Dev.as_str(), "devDependencies");
        assert_eq!(DependencyType::Peer.as_str(), "peerDependencies");
        assert_eq!(DependencyType::Optional.as_str(), "optionalDependencies");
    }

    #[test]
    fn test_dependency_type_is_production() {
        assert!(DependencyType::Regular.is_production());
        assert!(!DependencyType::Dev.is_production());
        assert!(!DependencyType::Peer.is_production());
        assert!(!DependencyType::Optional.is_production());
    }

    #[test]
    fn test_dependency_type_is_development() {
        assert!(!DependencyType::Regular.is_development());
        assert!(DependencyType::Dev.is_development());
        assert!(!DependencyType::Peer.is_development());
        assert!(!DependencyType::Optional.is_development());
    }

    #[test]
    fn test_dependency_type_is_peer() {
        assert!(!DependencyType::Regular.is_peer());
        assert!(!DependencyType::Dev.is_peer());
        assert!(DependencyType::Peer.is_peer());
        assert!(!DependencyType::Optional.is_peer());
    }

    #[test]
    fn test_dependency_type_is_optional() {
        assert!(!DependencyType::Regular.is_optional());
        assert!(!DependencyType::Dev.is_optional());
        assert!(!DependencyType::Peer.is_optional());
        assert!(DependencyType::Optional.is_optional());
    }

    #[test]
    fn test_dependency_type_display() {
        assert_eq!(format!("{}", DependencyType::Regular), "dependencies");
        assert_eq!(format!("{}", DependencyType::Dev), "devDependencies");
        assert_eq!(format!("{}", DependencyType::Peer), "peerDependencies");
        assert_eq!(format!("{}", DependencyType::Optional), "optionalDependencies");
    }

    #[test]
    fn test_dependency_type_clone_copy() {
        let dep_type = DependencyType::Regular;
        let cloned = dep_type;
        assert_eq!(dep_type, cloned);
    }

    #[test]
    fn test_dependency_type_equality() {
        assert_eq!(DependencyType::Regular, DependencyType::Regular);
        assert_ne!(DependencyType::Regular, DependencyType::Dev);
        assert_ne!(DependencyType::Dev, DependencyType::Peer);
        assert_ne!(DependencyType::Peer, DependencyType::Optional);
    }

    #[test]
    fn test_dependency_type_serialization() {
        let dep_type = DependencyType::Regular;
        let serialized = serde_json::to_string(&dep_type).unwrap();
        let deserialized: DependencyType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(dep_type, deserialized);
    }

    #[test]
    fn test_dependency_type_all_variants_serialization() {
        let types = vec![
            DependencyType::Regular,
            DependencyType::Dev,
            DependencyType::Peer,
            DependencyType::Optional,
        ];

        for dep_type in types {
            let serialized = serde_json::to_string(&dep_type).unwrap();
            let deserialized: DependencyType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(dep_type, deserialized);
        }
    }
}
