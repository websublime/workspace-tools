#[cfg(test)]
mod bump_tests {
    use serde_json::{from_str, to_string};
    use ws_pkg::bump::BumpOptions;
    use ws_pkg::types::version::Version;

    #[test]
    fn test_bump_options_serialization() {
        // Test creating and serializing bump options
        let options = BumpOptions {
            since: Some("v1.0.0".to_string()),
            release_as: Some(Version::Minor),
            fetch_all: Some(true),
            fetch_tags: Some(true),
            sync_deps: Some(true),
            push: Some(false),
        };

        // Serialize to JSON
        let json_str = to_string(&options).unwrap();

        // Verify serialization - check for "Minor" with uppercase M (not "minor")
        assert!(json_str.contains("\"since\""));
        assert!(json_str.contains("\"v1.0.0\""));
        assert!(json_str.contains("\"release_as\""));
        assert!(json_str.contains("\"Minor\"")); // Changed from "minor" to "Minor"
        assert!(json_str.contains("\"fetch_all\""));
        assert!(json_str.contains("true"));
        assert!(json_str.contains("\"push\""));
        assert!(json_str.contains("false"));
    }

    #[test]
    fn test_bump_options_deserialization() {
        // Test deserializing JSON to bump options
        // Use uppercase variants for Version enum
        let json_str = r#"{
            "since": "v1.0.0",
            "release_as": "Major",
            "fetch_all": true,
            "fetch_tags": false,
            "sync_deps": true,
            "push": false
        }"#;

        let options: BumpOptions = from_str(json_str).unwrap();

        // Verify deserialization
        assert_eq!(options.since, Some("v1.0.0".to_string()));
        assert!(matches!(options.release_as, Some(Version::Major)));
        assert_eq!(options.fetch_all, Some(true));
        assert_eq!(options.fetch_tags, Some(false));
        assert_eq!(options.sync_deps, Some(true));
        assert_eq!(options.push, Some(false));
    }

    #[test]
    fn test_bump_options_partial_deserialization() {
        // Test deserializing partial JSON to bump options
        // Use uppercase variant for Version enum
        let json_str = r#"{
            "since": "v1.0.0",
            "release_as": "Patch"
        }"#;

        let options: BumpOptions = from_str(json_str).unwrap();

        // Verify deserialization
        assert_eq!(options.since, Some("v1.0.0".to_string()));
        assert!(matches!(options.release_as, Some(Version::Patch)));
        assert_eq!(options.fetch_all, None);
        assert_eq!(options.fetch_tags, None);
        assert_eq!(options.sync_deps, None);
        assert_eq!(options.push, None);
    }

    #[test]
    fn test_bump_options_snapshot() {
        // Test creating options with snapshot release type
        let options = BumpOptions {
            since: None,
            release_as: Some(Version::Snapshot),
            fetch_all: None,
            fetch_tags: None,
            sync_deps: None,
            push: None,
        };

        // Serialize to JSON
        let json_str = to_string(&options).unwrap();

        // Verify serialization contains "Snapshot" (uppercase S)
        assert!(json_str.contains("\"Snapshot\""));

        // Deserialize back
        let deserialized: BumpOptions = from_str(&json_str).unwrap();

        // Verify snapshot type preserved
        assert!(matches!(deserialized.release_as, Some(Version::Snapshot)));
    }
}
