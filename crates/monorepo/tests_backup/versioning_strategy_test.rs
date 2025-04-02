mod test_utils;

use std::collections::HashMap;
use sublime_monorepo_tools::{
    determine_bump_type_from_change, BumpReason, BumpType, Change, ChangeType, VersionBumpStrategy,
};

#[cfg(test)]
mod versioning_strategy_tests {
    use super::*;

    #[test]
    fn test_bump_type_determination() {
        // Create changes of different types
        let feature_change = Change::new("pkg-test", ChangeType::Feature, "Normal feature", false);
        let breaking_feature =
            Change::new("pkg-test", ChangeType::Feature, "Breaking feature", true);
        let fix_change = Change::new("pkg-test", ChangeType::Fix, "Bug fix", false);
        let breaking_fix = Change::new("pkg-test", ChangeType::Fix, "Breaking fix", true);
        let docs_change = Change::new("pkg-test", ChangeType::Documentation, "Update docs", false);

        // Test Independent strategy - all features enabled
        let independent_all = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        // Check bump types for independent strategy
        assert_eq!(
            determine_bump_type_from_change(&feature_change, &independent_all),
            BumpType::Minor
        );
        assert_eq!(
            determine_bump_type_from_change(&breaking_feature, &independent_all),
            BumpType::Major
        );
        assert_eq!(determine_bump_type_from_change(&fix_change, &independent_all), BumpType::Patch);
        assert_eq!(
            determine_bump_type_from_change(&breaking_fix, &independent_all),
            BumpType::Major
        );
        assert_eq!(
            determine_bump_type_from_change(&docs_change, &independent_all),
            BumpType::Patch
        );

        // Test Independent strategy - only breaking enabled
        let independent_breaking_only = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: false,
            patch_otherwise: false,
        };

        assert_eq!(
            determine_bump_type_from_change(&feature_change, &independent_breaking_only),
            BumpType::None
        );
        assert_eq!(
            determine_bump_type_from_change(&breaking_feature, &independent_breaking_only),
            BumpType::Major
        );

        // Test Independent strategy - no options enabled
        let independent_none = VersionBumpStrategy::Independent {
            major_if_breaking: false,
            minor_if_feature: false,
            patch_otherwise: false,
        };

        assert_eq!(
            determine_bump_type_from_change(&breaking_feature, &independent_none),
            BumpType::None
        );

        // Test Conventional Commits strategy
        let conventional = VersionBumpStrategy::ConventionalCommits { from_ref: None };

        assert_eq!(
            determine_bump_type_from_change(&feature_change, &conventional),
            BumpType::Minor
        );
        assert_eq!(
            determine_bump_type_from_change(&breaking_feature, &conventional),
            BumpType::Major
        );
        assert_eq!(determine_bump_type_from_change(&fix_change, &conventional), BumpType::Patch);

        // Synchronized strategy doesn't use the change for determining bump type
        let synchronized = VersionBumpStrategy::Synchronized { version: "2.0.0".to_string() };
        assert_eq!(determine_bump_type_from_change(&feature_change, &synchronized), BumpType::None);

        // Manual strategy doesn't use the change for determining bump type
        let manual = VersionBumpStrategy::Manual(HashMap::new());
        assert_eq!(determine_bump_type_from_change(&feature_change, &manual), BumpType::None);
    }

    #[test]
    fn test_version_bump_strategy_serialization() {
        // Test Synchronized strategy
        let synchronized = VersionBumpStrategy::Synchronized { version: "2.0.0".to_string() };
        let json = serde_json::to_string(&synchronized).expect("Failed to serialize Synchronized");
        assert!(json.contains("\"type\":\"synchronized\""));
        assert!(json.contains("\"version\":\"2.0.0\""));

        // Test Independent strategy
        let independent = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };
        let json = serde_json::to_string(&independent).expect("Failed to serialize Independent");
        assert!(json.contains("\"type\":\"independent\""));
        assert!(json.contains("\"majorIfBreaking\":true"));

        // Test ConventionalCommits strategy
        let conventional =
            VersionBumpStrategy::ConventionalCommits { from_ref: Some("v1.0.0".to_string()) };
        let json =
            serde_json::to_string(&conventional).expect("Failed to serialize ConventionalCommits");
        assert!(json.contains("\"type\":\"conventionalcommits\""));
        assert!(json.contains("\"fromRef\":\"v1.0.0\""));

        // Test Manual strategy
        let mut versions = HashMap::new();
        versions.insert("pkg-a".to_string(), "1.1.0".to_string());
        let manual = VersionBumpStrategy::Manual(versions);
        let json = serde_json::to_string(&manual).expect("Failed to serialize Manual");
        assert!(json.contains("\"type\":\"manual\""));
        assert!(json.contains("\"pkg-a\":\"1.1.0\""));
    }

    #[test]
    fn test_version_bump_strategy_deserialization() {
        // Test Synchronized deserialization
        let json = r#"{"type":"synchronized","version":"2.0.0"}"#;
        let deserialized: VersionBumpStrategy =
            serde_json::from_str(json).expect("Failed to deserialize");

        match deserialized {
            VersionBumpStrategy::Synchronized { version } => {
                assert_eq!(version, "2.0.0");
            }
            _ => panic!("Expected Synchronized strategy"),
        }

        // Test Independent deserialization
        let json = r#"{"type":"independent","majorIfBreaking":true,"minorIfFeature":false,"patchOtherwise":true}"#;
        let deserialized: VersionBumpStrategy =
            serde_json::from_str(json).expect("Failed to deserialize");

        match deserialized {
            VersionBumpStrategy::Independent {
                major_if_breaking,
                minor_if_feature,
                patch_otherwise,
            } => {
                assert!(major_if_breaking);
                assert!(!minor_if_feature);
                assert!(patch_otherwise);
            }
            _ => panic!("Expected Independent strategy"),
        }

        // Test ConventionalCommits deserialization
        let json = r#"{"type":"conventionalcommits","fromRef":null}"#;
        let deserialized: VersionBumpStrategy =
            serde_json::from_str(json).expect("Failed to deserialize");

        match deserialized {
            VersionBumpStrategy::ConventionalCommits { from_ref } => {
                assert_eq!(from_ref, None);
            }
            _ => panic!("Expected ConventionalCommits strategy"),
        }
    }

    #[test]
    fn test_bump_type_display() {
        assert_eq!(BumpType::Major.to_string(), "major");
        assert_eq!(BumpType::Minor.to_string(), "minor");
        assert_eq!(BumpType::Patch.to_string(), "patch");
        assert_eq!(BumpType::Snapshot.to_string(), "snapshot");
        assert_eq!(BumpType::None.to_string(), "none");
    }

    #[test]
    fn test_bump_reason() {
        // Create various bump reasons
        let breaking = BumpReason::Breaking("API change".to_string());
        let feature = BumpReason::Feature("New feature".to_string());
        let _fix = BumpReason::Fix("Bug fix".to_string());
        let _other = BumpReason::Other("Other change".to_string());
        let _dep_update = BumpReason::DependencyUpdate("Dependency updated".to_string());
        //let _manual = BumpReason::Manual;

        // Verify they can be serialized and deserialized
        let breaking_json = serde_json::to_string(&breaking).expect("Failed to serialize Breaking");
        let feature_json = serde_json::to_string(&feature).expect("Failed to serialize Feature");

        let deser_breaking: BumpReason =
            serde_json::from_str(&breaking_json).expect("Failed to deserialize");
        let deser_feature: BumpReason =
            serde_json::from_str(&feature_json).expect("Failed to deserialize");

        // The actual contents depend on the implementation of the Serialize/Deserialize traits
        // but we can at least ensure it roundtrips correctly
        match deser_breaking {
            BumpReason::Breaking(msg) => assert_eq!(msg, "API change"),
            _ => panic!("Expected Breaking reason"),
        }

        match deser_feature {
            BumpReason::Feature(msg) => assert_eq!(msg, "New feature"),
            _ => panic!("Expected Feature reason"),
        }
    }
}
