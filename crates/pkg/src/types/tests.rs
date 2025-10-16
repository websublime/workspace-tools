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
use proptest::prelude::*;

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
