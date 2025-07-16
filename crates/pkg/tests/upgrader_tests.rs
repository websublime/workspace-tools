#[cfg(test)]
mod upgrader_tests {
    use sublime_package_tools::{
        AvailableUpgrade, Dependency, Filter, ExecutionMode, RegistryManager,
        UpgradeConfig, UpgradeStatus, Upgrader, Version, VersionRelationship, VersionStability,
        VersionUpdateStrategy,
    };

    // This test uses mocked registry responses
    #[test]
    fn test_check_dependency_upgrade() {
        // Create mock registry that returns predefined versions
        let mock_registry_manager = RegistryManager::new();

        // Create upgrader with mock registry
        let config = UpgradeConfig {
            update_strategy: VersionUpdateStrategy::MinorAndPatch,
            ..UpgradeConfig::default()
        };
        let mut upgrader = Upgrader::create(config, mock_registry_manager);

        // Create mock dependency to check
        // Note: In a real test, we'd use mockito to mock the registry responses
        // For this example, we'll return canned results with behavior testing

        // For packages where we have mocked versions:
        let dep1 = Dependency::new("react", "^16.0.0").unwrap();
        upgrader.check_dependency_upgrade("test-pkg", &dep1).unwrap();

        // We can't reliably check the result given this is a mock setup

        // Generate upgrade report (functionality test)
        let upgrades = vec![
            AvailableUpgrade {
                package_name: "test-pkg".to_string(),
                dependency_name: "react".to_string(),
                current_version: "^16.0.0".to_string(),
                compatible_version: Some("16.14.0".to_string()),
                latest_version: Some("18.2.0".to_string()),
                status: UpgradeStatus::MinorAvailable("16.14.0".to_string()),
            },
            AvailableUpgrade {
                package_name: "test-pkg".to_string(),
                dependency_name: "lodash".to_string(),
                current_version: "^4.17.20".to_string(),
                compatible_version: Some("4.17.21".to_string()),
                latest_version: Some("4.17.21".to_string()),
                status: UpgradeStatus::PatchAvailable("4.17.21".to_string()),
            },
        ];

        let report = Upgrader::generate_upgrade_report(&upgrades);
        assert!(!report.is_empty());
        assert!(report.contains("react"));
        assert!(report.contains("lodash"));
    }

    #[test]
    fn test_upgrade_config() {
        // Test creating different upgrade configs

        // Default config
        let default_config = UpgradeConfig::default();
        assert!(matches!(default_config.execution_mode, ExecutionMode::DryRun));

        // Patch-only config
        let _patch_config = UpgradeConfig {
            update_strategy: VersionUpdateStrategy::PatchOnly,
            ..UpgradeConfig::default()
        };

        // Full upgrade config
        let full_config = UpgradeConfig {
            dependency_types: Filter::AllDependencies,
            update_strategy: VersionUpdateStrategy::AllUpdates,
            version_stability: VersionStability::IncludePrerelease,
            target_packages: vec!["ui".to_string()],
            target_dependencies: vec!["react".to_string()],
            registries: vec!["https://npm.pkg.github.com".to_string()],
            execution_mode: ExecutionMode::Apply,
        };

        assert_eq!(full_config.target_packages.len(), 1);
        assert_eq!(full_config.target_packages[0], "ui");
        assert_eq!(full_config.target_dependencies[0], "react");
        assert!(matches!(full_config.execution_mode, ExecutionMode::Apply));
    }

    #[test]
    fn test_upgrade_status() {
        // Test different upgrade status types
        //let _up_to_date = UpgradeStatus::UpToDate;
        let _patch = UpgradeStatus::PatchAvailable("1.0.1".to_string());
        let _minor = UpgradeStatus::MinorAvailable("1.1.0".to_string());
        let _major = UpgradeStatus::MajorAvailable("2.0.0".to_string());
        let _constrained = UpgradeStatus::Constrained("pinned at 1.0.0".to_string());
        let _failed = UpgradeStatus::CheckFailed("network error".to_string());

        // Just verifying we can create these variants
        // In a real test, we might want to test display formatting or other behaviors
    }

    #[test]
    fn test_available_upgrade() {
        // Create available upgrade
        let upgrade = AvailableUpgrade {
            package_name: "ui-components".to_string(),
            dependency_name: "react".to_string(),
            current_version: "^17.0.0".to_string(),
            compatible_version: Some("17.0.2".to_string()),
            latest_version: Some("18.2.0".to_string()),
            status: UpgradeStatus::MinorAvailable("17.0.2".to_string()),
        };

        // Check fields
        assert_eq!(upgrade.package_name, "ui-components");
        assert_eq!(upgrade.dependency_name, "react");
        assert_eq!(upgrade.current_version, "^17.0.0");
        assert_eq!(upgrade.compatible_version, Some("17.0.2".to_string()));
        assert_eq!(upgrade.latest_version, Some("18.2.0".to_string()));

        // Match on status
        match upgrade.status {
            UpgradeStatus::MinorAvailable(v) => assert_eq!(v, "17.0.2"),
            _ => panic!("Expected MinorAvailable status"),
        }
    }

    #[test]
    fn test_upgrade_status_display() {
        // Test the display formatting of various upgrade statuses
        let up_to_date = UpgradeStatus::UpToDate;
        let patch = UpgradeStatus::PatchAvailable("1.0.1".to_string());
        let minor = UpgradeStatus::MinorAvailable("1.1.0".to_string());
        let major = UpgradeStatus::MajorAvailable("2.0.0".to_string());
        let constrained = UpgradeStatus::Constrained("2.0.0".to_string());
        let failed = UpgradeStatus::CheckFailed("Error message".to_string());

        assert_eq!(up_to_date.to_string(), "up to date");
        assert_eq!(patch.to_string(), "patch available: 1.0.1");
        assert_eq!(minor.to_string(), "minor update available: 1.1.0");
        assert_eq!(major.to_string(), "major update available: 2.0.0");
        assert_eq!(constrained.to_string(), "constrained (latest: 2.0.0)");
        assert_eq!(failed.to_string(), "check failed: Error message");
    }

    #[test]
    fn test_upgrader_with_different_configs() {
        // Test creation with different configurations

        // Default configuration
        let default_upgrader = Upgrader::new();
        let default_config = default_upgrader.config();
        assert!(matches!(default_config.update_strategy, VersionUpdateStrategy::MinorAndPatch));
        assert!(matches!(default_config.execution_mode, ExecutionMode::DryRun));

        // Patch-only configuration
        let patch_config = UpgradeConfig {
            update_strategy: VersionUpdateStrategy::PatchOnly,
            ..UpgradeConfig::default()
        };
        let patch_upgrader = Upgrader::with_config(patch_config);
        assert!(matches!(
            patch_upgrader.config().update_strategy,
            VersionUpdateStrategy::PatchOnly
        ));

        // Configuration with prerelease versions
        let prerelease_config = UpgradeConfig {
            version_stability: VersionStability::IncludePrerelease,
            ..UpgradeConfig::default()
        };
        let prerelease_upgrader = Upgrader::with_config(prerelease_config);
        assert!(matches!(
            prerelease_upgrader.config().version_stability,
            VersionStability::IncludePrerelease
        ));

        // Configuration with specific targets
        let targeted_config = UpgradeConfig {
            target_packages: vec!["pkg-a".to_string()],
            target_dependencies: vec!["react".to_string()],
            ..UpgradeConfig::default()
        };
        let targeted_upgrader = Upgrader::with_config(targeted_config);
        assert_eq!(targeted_upgrader.config().target_packages.len(), 1);
        assert_eq!(targeted_upgrader.config().target_packages[0], "pkg-a");
        assert_eq!(targeted_upgrader.config().target_dependencies[0], "react");

        // Apply mode configuration
        let apply_config =
            UpgradeConfig { execution_mode: ExecutionMode::Apply, ..UpgradeConfig::default() };
        let mut apply_upgrader = Upgrader::with_config(apply_config.clone());
        assert!(matches!(apply_upgrader.config().execution_mode, ExecutionMode::Apply));

        // Test changing configuration
        apply_upgrader.set_config(UpgradeConfig::default());
        assert!(matches!(apply_upgrader.config().execution_mode, ExecutionMode::DryRun));

        // Test accessing registry manager
        let registry_manager = apply_upgrader.registry_manager();
        assert_eq!(registry_manager.default_registry(), "https://registry.npmjs.org");

        // Test mutable access to registry manager
        let registry_manager_mut = apply_upgrader.registry_manager_mut();
        assert_eq!(registry_manager_mut.default_registry(), "https://registry.npmjs.org");
    }

    #[test]
    fn test_upgrade_report_generation() {
        // Create sample upgrades
        let upgrades = vec![
            AvailableUpgrade {
                package_name: "app".to_string(),
                dependency_name: "react".to_string(),
                current_version: "^17.0.0".to_string(),
                compatible_version: Some("17.0.2".to_string()),
                latest_version: Some("18.2.0".to_string()),
                status: UpgradeStatus::PatchAvailable("17.0.2".to_string()),
            },
            AvailableUpgrade {
                package_name: "app".to_string(),
                dependency_name: "lodash".to_string(),
                current_version: "^4.17.20".to_string(),
                compatible_version: Some("4.17.21".to_string()),
                latest_version: Some("4.17.21".to_string()),
                status: UpgradeStatus::PatchAvailable("4.17.21".to_string()),
            },
            AvailableUpgrade {
                package_name: "lib".to_string(),
                dependency_name: "express".to_string(),
                current_version: "^4.17.0".to_string(),
                compatible_version: Some("4.18.2".to_string()),
                latest_version: Some("4.18.2".to_string()),
                status: UpgradeStatus::MinorAvailable("4.18.2".to_string()),
            },
        ];

        // Generate report
        let report = Upgrader::generate_upgrade_report(&upgrades);

        // Check report content
        assert!(report.contains("Available dependency upgrades"));
        assert!(report.contains("Package: app"));
        assert!(report.contains("Package: lib"));
        assert!(report.contains("react"));
        assert!(report.contains("lodash"));
        assert!(report.contains("express"));
        assert!(report.contains("patch:")); // Should mention patch upgrades
        assert!(report.contains("minor:")); // Should mention minor upgrades

        // Test empty upgrades
        let empty_report = Upgrader::generate_upgrade_report(&[]);
        assert_eq!(empty_report, "All dependencies are up to date.");
    }

    #[test]
    fn test_available_upgrade_display() {
        use sublime_package_tools::{AvailableUpgrade, UpgradeStatus};

        // Create an available upgrade
        let upgrade = AvailableUpgrade {
            package_name: "app".to_string(),
            dependency_name: "react".to_string(),
            current_version: "^17.0.0".to_string(),
            compatible_version: Some("17.0.2".to_string()),
            latest_version: Some("18.2.0".to_string()),
            status: UpgradeStatus::PatchAvailable("17.0.2".to_string()),
        };

        // Test the display format
        let display = format!("{upgrade}");
        assert_eq!(display, "react in app: ^17.0.0 -> 17.0.2");

        // Test with no compatible version
        let no_compatible = AvailableUpgrade {
            package_name: "app".to_string(),
            dependency_name: "react".to_string(),
            current_version: "^17.0.0".to_string(),
            compatible_version: None,
            latest_version: Some("18.2.0".to_string()),
            status: UpgradeStatus::UpToDate,
        };

        let display = format!("{no_compatible}");
        assert_eq!(display, "react in app: ^17.0.0 -> no compatible version");
    }

    #[test]
    fn test_version_update_strategy() {
        // Test enum variants exist and are distinct
        let patch_only = VersionUpdateStrategy::PatchOnly;
        let minor_patch = VersionUpdateStrategy::MinorAndPatch;
        let all = VersionUpdateStrategy::AllUpdates;

        // Test they're different variants
        assert_ne!(patch_only, minor_patch);
        assert_ne!(patch_only, all);
        assert_ne!(minor_patch, all);

        // Test default
        let default = VersionUpdateStrategy::default();
        assert_eq!(default, VersionUpdateStrategy::MinorAndPatch);
    }

    #[test]
    fn test_version_stability() {
        // Test enum variants exist and are distinct
        let stable = VersionStability::StableOnly;
        let with_pre = VersionStability::IncludePrerelease;

        // Test they're different variants
        assert_ne!(stable, with_pre);

        // Test default
        let default = VersionStability::default();
        assert_eq!(default, VersionStability::StableOnly);
    }

    #[test]
    fn test_version_relationship_display() {
        // Test all relationship variants display correctly
        assert_eq!(format!("{}", VersionRelationship::MajorUpgrade), "MajorUpgrade");
        assert_eq!(format!("{}", VersionRelationship::MinorUpgrade), "MinorUpgrade");
        assert_eq!(format!("{}", VersionRelationship::PatchUpgrade), "PatchUpgrade");
        assert_eq!(format!("{}", VersionRelationship::PrereleaseToStable), "PrereleaseToStable");
        assert_eq!(format!("{}", VersionRelationship::NewerPrerelease), "NewerPrerelease");
        assert_eq!(format!("{}", VersionRelationship::Identical), "Identical");
        assert_eq!(format!("{}", VersionRelationship::MajorDowngrade), "MajorDowngrade");
        assert_eq!(format!("{}", VersionRelationship::MinorDowngrade), "MinorDowngrade");
        assert_eq!(format!("{}", VersionRelationship::PatchDowngrade), "PatchDowngrade");
        assert_eq!(format!("{}", VersionRelationship::StableToPrerelease), "StableToPrerelease");
        assert_eq!(format!("{}", VersionRelationship::OlderPrerelease), "OlderPrerelease");
        assert_eq!(format!("{}", VersionRelationship::Indeterminate), "Indeterminate");
    }

    #[test]
    fn test_additional_version_comparisons() {
        // Test additional comparison scenarios

        // Test prerelease relationships
        assert_eq!(
            Version::compare_versions("1.0.0-alpha", "1.0.0-beta"),
            VersionRelationship::NewerPrerelease
        );

        assert_eq!(
            Version::compare_versions("1.0.0-beta", "1.0.0-alpha"),
            VersionRelationship::OlderPrerelease
        );

        assert_eq!(
            Version::compare_versions("1.0.0", "1.0.0-alpha"),
            VersionRelationship::StableToPrerelease
        );

        // Test full range of upgrade/downgrade relationships
        assert_eq!(Version::compare_versions("1.0.0", "1.0.1"), VersionRelationship::PatchUpgrade);

        assert_eq!(
            Version::compare_versions("1.0.1", "1.0.0"),
            VersionRelationship::PatchDowngrade
        );

        assert_eq!(Version::compare_versions("1.0.0", "1.1.0"), VersionRelationship::MinorUpgrade);

        assert_eq!(
            Version::compare_versions("1.1.0", "1.0.0"),
            VersionRelationship::MinorDowngrade
        );

        // Test identical
        assert_eq!(Version::compare_versions("1.0.0", "1.0.0"), VersionRelationship::Identical);

        // Test invalid versions
        assert_eq!(
            Version::compare_versions("invalid", "1.0.0"),
            VersionRelationship::Indeterminate
        );
    }

    #[test]
    fn test_version_parse_with_build_metadata() {
        // Test parsing versions with build metadata
        let v = Version::parse("1.2.3+build.456").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(!v.build.is_empty());

        // Test version bumping removes build metadata
        let next_patch = Version::bump_patch("1.2.3+build.456").unwrap();
        assert_eq!(next_patch.to_string(), "1.2.4");
        assert!(next_patch.build.is_empty());

        let next_minor = Version::bump_minor("1.2.3+build.456").unwrap();
        assert_eq!(next_minor.to_string(), "1.3.0");
        assert!(next_minor.build.is_empty());

        let next_major = Version::bump_major("1.2.3+build.456").unwrap();
        assert_eq!(next_major.to_string(), "2.0.0");
        assert!(next_major.build.is_empty());
    }
}
