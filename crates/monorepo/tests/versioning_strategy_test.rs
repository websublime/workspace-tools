mod fixtures;

#[cfg(test)]
mod versioning_strategy_tests {
    use std::collections::HashMap;
    use sublime_monorepo_tools::{
        BumpReason, BumpType, ChangelogOptions, PackageVersionChange, VersionBumpStrategy,
    };

    #[test]
    fn test_bump_type_display() {
        assert_eq!(BumpType::Major.to_string(), "major");
        assert_eq!(BumpType::Minor.to_string(), "minor");
        assert_eq!(BumpType::Patch.to_string(), "patch");
        assert_eq!(BumpType::Snapshot.to_string(), "snapshot");
        assert_eq!(BumpType::None.to_string(), "none");
    }

    #[test]
    fn test_version_bump_strategy_creation() {
        // Test Independent strategy
        let independent = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        // Verify Independent settings
        if let VersionBumpStrategy::Independent {
            major_if_breaking,
            minor_if_feature,
            patch_otherwise,
        } = &independent
        {
            assert!(major_if_breaking);
            assert!(minor_if_feature);
            assert!(patch_otherwise);
        } else {
            panic!("Expected Independent strategy");
        }

        // Test Synchronized strategy
        let synchronized = VersionBumpStrategy::Synchronized { version: "2.0.0".to_string() };

        // Verify Synchronized settings
        if let VersionBumpStrategy::Synchronized { version } = &synchronized {
            assert_eq!(version, "2.0.0");
        } else {
            panic!("Expected Synchronized strategy");
        }

        // Test ConventionalCommits strategy
        let conventional =
            VersionBumpStrategy::ConventionalCommits { from_ref: Some("v1.0.0".to_string()) };

        // Verify ConventionalCommits settings
        if let VersionBumpStrategy::ConventionalCommits { from_ref } = &conventional {
            assert_eq!(from_ref, &Some("v1.0.0".to_string()));
        } else {
            panic!("Expected ConventionalCommits strategy");
        }

        // Test Manual strategy
        let mut versions = HashMap::new();
        versions.insert("@scope/package-foo".to_string(), "1.1.0".to_string());
        versions.insert("@scope/package-bar".to_string(), "2.0.0".to_string());
        let manual = VersionBumpStrategy::Manual(versions.clone());

        // Verify Manual settings
        if let VersionBumpStrategy::Manual(map) = &manual {
            assert_eq!(map.len(), 2);
            assert_eq!(map.get("@scope/package-foo"), Some(&"1.1.0".to_string()));
            assert_eq!(map.get("@scope/package-bar"), Some(&"2.0.0".to_string()));
        } else {
            panic!("Expected Manual strategy");
        }

        // Test default strategy
        let default_strategy = VersionBumpStrategy::default();
        if let VersionBumpStrategy::Independent {
            major_if_breaking,
            minor_if_feature,
            patch_otherwise,
        } = default_strategy
        {
            assert!(major_if_breaking);
            assert!(minor_if_feature);
            assert!(patch_otherwise);
        } else {
            panic!("Default strategy should be Independent");
        }
    }

    #[test]
    fn test_package_version_change() {
        let change = PackageVersionChange {
            package_name: "@scope/package-foo".to_string(),
            previous_version: "1.0.0".to_string(),
            new_version: "1.1.0".to_string(),
            bump_type: BumpType::Minor,
            is_dependency_update: false,
            is_cycle_update: false,
            cycle_group: None,
        };

        assert_eq!(change.package_name, "@scope/package-foo");
        assert_eq!(change.previous_version, "1.0.0");
        assert_eq!(change.new_version, "1.1.0");
        assert_eq!(change.bump_type.to_string(), "minor");
        assert!(!change.is_dependency_update);
        assert!(!change.is_cycle_update);
        assert!(change.cycle_group.is_none());

        // Test with cycle information
        let cycle_change = PackageVersionChange {
            package_name: "@scope/package-foo".to_string(),
            previous_version: "1.0.0".to_string(),
            new_version: "2.0.0".to_string(),
            bump_type: BumpType::Major,
            is_dependency_update: false,
            is_cycle_update: true,
            cycle_group: Some(vec![
                "@scope/package-foo".to_string(),
                "@scope/package-bar".to_string(),
                "@scope/package-baz".to_string(),
            ]),
        };

        assert!(cycle_change.is_cycle_update);
        assert_eq!(cycle_change.cycle_group.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_bump_reason() {
        // Test various BumpReason variants
        let reasons = [
            BumpReason::Breaking("API change".to_string()),
            BumpReason::Feature("New component".to_string()),
            BumpReason::Fix("Bug fix".to_string()),
            BumpReason::Other("General update".to_string()),
            BumpReason::DependencyUpdate("Updated dependency".to_string()),
            BumpReason::Manual,
        ];

        // Just ensure we can create and store these reason types
        assert_eq!(reasons.len(), 6);
    }

    #[test]
    fn test_changelog_options() {
        // Test default changelog options
        let default_options = ChangelogOptions::default();
        assert!(default_options.update_existing);
        assert_eq!(default_options.filename, "CHANGELOG.md");
        assert!(default_options.include_version_details);
        assert!(default_options.include_release_date);
        assert_eq!(default_options.header_template, "# Changelog\n\n");
        assert_eq!(default_options.change_template, "- {type}: {description} {breaking}\n");

        // Test custom changelog options
        let custom_options = ChangelogOptions {
            update_existing: false,
            filename: "CHANGES.md".to_string(),
            include_version_details: false,
            include_release_date: false,
            header_template: "# Release History\n\n".to_string(),
            change_template: "* {type}: {description} {breaking}\n".to_string(),
        };

        assert!(!custom_options.update_existing);
        assert_eq!(custom_options.filename, "CHANGES.md");
        assert!(!custom_options.include_version_details);
        assert!(!custom_options.include_release_date);
        assert_eq!(custom_options.header_template, "# Release History\n\n");
        assert_eq!(custom_options.change_template, "* {type}: {description} {breaking}\n");

        // Test new() method
        let new_options = ChangelogOptions::new();
        assert_eq!(new_options.filename, default_options.filename);
    }
}
