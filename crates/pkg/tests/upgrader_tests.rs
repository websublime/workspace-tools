#[cfg(test)]
mod upgrader_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use sublime_package_tools::{
        AvailableUpgrade, Dependency, ExecutionMode, Package, RegistryManager, UpgradeConfig,
        UpgradeStatus, Upgrader, VersionStability, VersionUpdateStrategy,
    };

    #[test]
    fn test_upgrader_creation() {
        // Basic creation
        let _upgrader = Upgrader::new();

        // With config
        let config = UpgradeConfig {
            dependency_types: sublime_package_tools::DependencyFilter::ProductionOnly,
            update_strategy: VersionUpdateStrategy::MinorAndPatch,
            version_stability: VersionStability::StableOnly,
            target_packages: vec!["specific-package".to_string()],
            target_dependencies: vec!["react".to_string(), "lodash".to_string()],
            registries: vec!["https://registry.npmjs.org".to_string()],
            execution_mode: ExecutionMode::DryRun,
        };

        let _upgrader_with_config = Upgrader::with_config(config);

        // With registry manager
        let registry_manager = RegistryManager::new();
        let _upgrader_with_registry = Upgrader::with_registry_manager(registry_manager);

        // With both
        let config = UpgradeConfig::default();
        let registry_manager = RegistryManager::new();
        let _upgrader_combined = Upgrader::create(config, registry_manager);
    }

    #[test]
    fn test_upgrade_status_display() {
        // Test string representation of upgrade status
        assert_eq!(UpgradeStatus::UpToDate.to_string(), "up to date");
        assert_eq!(
            UpgradeStatus::PatchAvailable("1.0.1".to_string()).to_string(),
            "patch available: 1.0.1"
        );
        assert_eq!(
            UpgradeStatus::MinorAvailable("1.1.0".to_string()).to_string(),
            "minor update available: 1.1.0"
        );
        assert_eq!(
            UpgradeStatus::MajorAvailable("2.0.0".to_string()).to_string(),
            "major update available: 2.0.0"
        );
        assert_eq!(
            UpgradeStatus::Constrained("3.0.0".to_string()).to_string(),
            "constrained (latest: 3.0.0)"
        );
        assert_eq!(
            UpgradeStatus::CheckFailed("error".to_string()).to_string(),
            "check failed: error"
        );
    }

    #[test]
    fn test_available_upgrade_display() {
        // Test string representation of available upgrade
        let upgrade = AvailableUpgrade {
            package_name: "my-package".to_string(),
            dependency_name: "react".to_string(),
            current_version: "16.0.0".to_string(),
            compatible_version: Some("17.0.0".to_string()),
            latest_version: Some("18.0.0".to_string()),
            status: UpgradeStatus::MajorAvailable("17.0.0".to_string()),
        };

        assert_eq!(upgrade.to_string(), "react in my-package: 16.0.0 -> 17.0.0");

        // When no compatible version
        let upgrade_no_compatible = AvailableUpgrade {
            package_name: "my-package".to_string(),
            dependency_name: "react".to_string(),
            current_version: "16.0.0".to_string(),
            compatible_version: None,
            latest_version: Some("18.0.0".to_string()),
            status: UpgradeStatus::Constrained("18.0.0".to_string()),
        };

        assert_eq!(
            upgrade_no_compatible.to_string(),
            "react in my-package: 16.0.0 -> no compatible version"
        );
    }

    #[test]
    fn test_generate_upgrade_report() {
        // Create some upgrades
        let upgrades = vec![
            AvailableUpgrade {
                package_name: "pkg-a".to_string(),
                dependency_name: "dep1".to_string(),
                current_version: "1.0.0".to_string(),
                compatible_version: Some("1.1.0".to_string()),
                latest_version: Some("1.1.0".to_string()),
                status: UpgradeStatus::MinorAvailable("1.1.0".to_string()),
            },
            AvailableUpgrade {
                package_name: "pkg-a".to_string(),
                dependency_name: "dep2".to_string(),
                current_version: "2.0.0".to_string(),
                compatible_version: Some("2.0.1".to_string()),
                latest_version: Some("3.0.0".to_string()),
                status: UpgradeStatus::PatchAvailable("2.0.1".to_string()),
            },
            AvailableUpgrade {
                package_name: "pkg-b".to_string(),
                dependency_name: "dep3".to_string(),
                current_version: "1.0.0".to_string(),
                compatible_version: Some("2.0.0".to_string()),
                latest_version: Some("2.0.0".to_string()),
                status: UpgradeStatus::MajorAvailable("2.0.0".to_string()),
            },
        ];

        // Generate report
        let report = Upgrader::generate_upgrade_report(&upgrades);

        // Verify report structure
        assert!(report.contains("Available dependency upgrades:"));
        assert!(report.contains("Package: pkg-a"));
        assert!(report.contains("Package: pkg-b"));
        assert!(report.contains("dep1"));
        assert!(report.contains("dep2"));
        assert!(report.contains("dep3"));
        assert!(report.contains("1.0.0 -> 1.1.0"));
        assert!(report.contains("2.0.0 -> 2.0.1"));
        assert!(report.contains("1.0.0 -> 2.0.0"));
        assert!(report.contains("minor:"));
        assert!(report.contains("patch:"));
        assert!(report.contains("major:"));

        // Test empty report
        let empty_report = Upgrader::generate_upgrade_report(&[]);
        assert!(empty_report.contains("All dependencies are up to date."));
    }

    #[test]
    fn test_apply_upgrades() {
        // Create packages with dependencies
        let dep1 = Rc::new(RefCell::new(Dependency::new("dep1", "^1.0.0").unwrap()));
        let dep2 = Rc::new(RefCell::new(Dependency::new("dep2", "^2.0.0").unwrap()));

        let pkg = Rc::new(RefCell::new(
            Package::new("pkg-a", "1.0.0", Some(vec![Rc::clone(&dep1), Rc::clone(&dep2)])).unwrap(),
        ));

        // Create upgrades
        let upgrades = vec![
            AvailableUpgrade {
                package_name: "pkg-a".to_string(),
                dependency_name: "dep1".to_string(),
                current_version: "^1.0.0".to_string(),
                compatible_version: Some("^1.1.0".to_string()),
                latest_version: Some("^1.1.0".to_string()),
                status: UpgradeStatus::MinorAvailable("^1.1.0".to_string()),
            },
            AvailableUpgrade {
                package_name: "pkg-a".to_string(),
                dependency_name: "dep2".to_string(),
                current_version: "^2.0.0".to_string(),
                compatible_version: Some("^2.0.1".to_string()),
                latest_version: Some("^3.0.0".to_string()),
                status: UpgradeStatus::PatchAvailable("^2.0.1".to_string()),
            },
        ];

        // Create upgrader with Apply mode
        let config =
            UpgradeConfig { execution_mode: ExecutionMode::Apply, ..UpgradeConfig::default() };
        let upgrader = Upgrader::with_config(config);

        // Apply upgrades
        let result = upgrader.apply_upgrades(&[Rc::clone(&pkg)], &upgrades);
        assert!(result.is_ok());

        // Verify dependencies were updated
        let pkg_ref = pkg.borrow();
        assert_eq!(pkg_ref.dependencies()[0].borrow().version().to_string(), "^1.1.0");
        assert_eq!(pkg_ref.dependencies()[1].borrow().version().to_string(), "^2.0.1");
    }

    #[test]
    fn test_dry_run_mode() {
        // Create packages with dependencies
        let dep1 = Rc::new(RefCell::new(Dependency::new("dep1", "^1.0.0").unwrap()));
        let dep2 = Rc::new(RefCell::new(Dependency::new("dep2", "^2.0.0").unwrap()));

        let pkg = Rc::new(RefCell::new(
            Package::new("pkg-a", "1.0.0", Some(vec![Rc::clone(&dep1), Rc::clone(&dep2)])).unwrap(),
        ));

        // Create upgrades
        let upgrades = vec![AvailableUpgrade {
            package_name: "pkg-a".to_string(),
            dependency_name: "dep1".to_string(),
            current_version: "^1.0.0".to_string(),
            compatible_version: Some("^1.1.0".to_string()),
            latest_version: Some("^1.1.0".to_string()),
            status: UpgradeStatus::MinorAvailable("^1.1.0".to_string()),
        }];

        // Create upgrader with dry run mode
        let config =
            UpgradeConfig { execution_mode: ExecutionMode::DryRun, ..UpgradeConfig::default() };
        let upgrader = Upgrader::with_config(config);

        // Apply upgrades in dry run mode
        let result = upgrader.apply_upgrades(&[Rc::clone(&pkg)], &upgrades);
        assert!(result.is_ok());

        // Verify dependencies were NOT updated (dry run)
        let pkg_ref = pkg.borrow();
        assert_eq!(
            pkg_ref.dependencies()[0].borrow().version().to_string(),
            "^1.0.0" // Still the original version
        );
    }

    #[test]
    fn test_filter_by_target_packages() {
        // Create upgrader with target package filter
        let config = UpgradeConfig {
            target_packages: vec!["pkg-a".to_string()], // Only target pkg-a
            ..UpgradeConfig::default()
        };
        let upgrader = Upgrader::with_config(config);

        // We can't fully test check_all_upgrades without a mock registry,
        // but we can verify the package filtering logic by looking at the config
        assert_eq!(upgrader.config().target_packages.len(), 1);
        assert_eq!(upgrader.config().target_packages[0], "pkg-a");
    }

    #[test]
    fn test_filter_by_target_dependencies() {
        // Create upgrader with target dependency filter
        let config = UpgradeConfig {
            target_dependencies: vec!["react".to_string(), "lodash".to_string()],
            ..UpgradeConfig::default()
        };
        let upgrader = Upgrader::with_config(config);

        // Verify the dependency filtering config
        assert_eq!(upgrader.config().target_dependencies.len(), 2);
        assert!(upgrader.config().target_dependencies.contains(&"react".to_string()));
        assert!(upgrader.config().target_dependencies.contains(&"lodash".to_string()));
    }
}
