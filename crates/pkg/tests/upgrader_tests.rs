#[cfg(test)]
mod upgrader_tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use ws_pkg::registry::{LocalRegistry, RegistryManager};
    use ws_pkg::types::dependency::Dependency;
    use ws_pkg::types::package::Package;
    use ws_pkg::types::version::{VersionStability, VersionUpdateStrategy};
    use ws_pkg::upgrader::{
        AvailableUpgrade, DependencyUpgrader, ExecutionMode, UpgradeConfig, UpgradeStatus,
    };

    // Helper functions
    fn create_test_dependency(name: &str, version: &str) -> Rc<RefCell<Dependency>> {
        Rc::new(RefCell::new(Dependency::new(name, version).unwrap()))
    }

    fn create_test_package(
        name: &str,
        version: &str,
        dependencies: Option<Vec<Rc<RefCell<Dependency>>>>,
    ) -> Package {
        Package::new(name, version, dependencies).unwrap()
    }

    // Create a test registry manager with local registry
    fn setup_test_registry() -> (LocalRegistry, RegistryManager) {
        let local_registry = LocalRegistry::new();

        // Add test packages to the registry
        local_registry.add_package("dep1", vec!["1.0.0", "1.0.1", "1.1.0", "2.0.0"]).unwrap();
        local_registry.add_package("dep2", vec!["2.0.0", "2.0.1"]).unwrap();
        local_registry.add_package("dep3", vec!["1.0.0-beta", "1.0.0"]).unwrap();

        // Create registry manager
        let mut manager = RegistryManager::new();

        // Add the local registry instance
        manager
            .add_registry_instance(
                "https://test-registry",
                std::sync::Arc::new(local_registry.clone()),
            )
            .unwrap();

        manager.set_default_registry("https://test-registry").unwrap();

        (local_registry, manager)
    }

    #[test]
    fn test_upgrade_config() {
        // Test default config
        let config = UpgradeConfig::default();
        assert!(matches!(config.update_strategy, VersionUpdateStrategy::MinorAndPatch));
        assert!(matches!(config.version_stability, VersionStability::StableOnly));
        assert!(matches!(config.execution_mode, ExecutionMode::DryRun));
        assert!(config.target_packages.is_empty());
        assert!(config.target_dependencies.is_empty());

        // Test from update strategy
        let update_strategy = VersionUpdateStrategy::AllUpdates;
        let config = UpgradeConfig::from(&update_strategy);
        assert!(matches!(config.update_strategy, VersionUpdateStrategy::AllUpdates));

        // Test with registries
        let registries = vec!["https://custom-registry.com".to_string()];
        let config = UpgradeConfig::with_registries(registries.clone());
        assert_eq!(config.registries, registries);
    }

    #[test]
    fn test_dependency_upgrader_creation() {
        // Test default creation
        let upgrader = DependencyUpgrader::new();
        assert!(matches!(upgrader.config().update_strategy, VersionUpdateStrategy::MinorAndPatch));

        // Test with config
        let config = UpgradeConfig {
            update_strategy: VersionUpdateStrategy::AllUpdates,
            ..Default::default()
        };
        let upgrader = DependencyUpgrader::with_config(config);
        assert!(matches!(upgrader.config().update_strategy, VersionUpdateStrategy::AllUpdates));
    }

    #[test]
    fn test_check_dependency_upgrade() {
        let (_, manager) = setup_test_registry();
        let mut upgrader = DependencyUpgrader::with_registry_manager(manager);

        // Create a test dependency
        let dep = Dependency::new("dep1", "^1.0.0").unwrap();

        // Check for upgrades
        let upgrade = upgrader.check_dependency_upgrade("test-pkg", &dep).unwrap();

        // Verify upgrade info
        assert_eq!(upgrade.package_name, "test-pkg");
        assert_eq!(upgrade.dependency_name, "dep1");
        assert_eq!(upgrade.current_version, "^1.0.0");
        assert_eq!(upgrade.compatible_version, Some("1.1.0".to_string()));

        // Major version should not be included by default
        assert_ne!(upgrade.latest_version, Some("2.0.0".to_string()));

        match upgrade.status {
            UpgradeStatus::MinorAvailable(_) => {} // Expected
            _ => panic!("Expected MinorAvailable status"),
        }
    }

    #[test]
    fn test_check_package_upgrades() {
        let (_, manager) = setup_test_registry();
        let mut upgrader = DependencyUpgrader::with_registry_manager(manager);

        // Create package with multiple dependencies
        let deps = vec![
            create_test_dependency("dep1", "^1.0.0"),
            create_test_dependency("dep2", "^2.0.0"),
            create_test_dependency("dep3", "^1.0.0-beta"),
        ];
        let pkg = create_test_package("test-pkg", "1.0.0", Some(deps));

        // Check upgrades
        let upgrades = upgrader.check_package_upgrades(&pkg).unwrap();

        // Should find upgrades for dep1 and dep3 but not dep2 which is already at latest minor
        let dep_upgrades: HashMap<_, _> =
            upgrades.iter().map(|u| (u.dependency_name.clone(), u.clone())).collect();

        assert!(dep_upgrades.contains_key("dep1"));
        assert!(matches!(dep_upgrades["dep1"].status, UpgradeStatus::MinorAvailable(_)));

        assert!(dep_upgrades.contains_key("dep3"));
        assert!(matches!(dep_upgrades["dep3"].status, UpgradeStatus::PatchAvailable(_)));

        // dep2 should not have updates (unless it found a patch update)
        if let Some(dep2) = dep_upgrades.get("dep2") {
            assert!(matches!(dep2.status, UpgradeStatus::PatchAvailable(_)));
        }
    }

    #[test]
    fn test_major_version_upgrades() {
        let (_, manager) = setup_test_registry();
        let mut upgrader = DependencyUpgrader::with_registry_manager(manager);

        // Configure to include major version upgrades
        upgrader.set_config(UpgradeConfig {
            update_strategy: VersionUpdateStrategy::AllUpdates,
            ..Default::default()
        });

        // Create package with dependency that has a major update available
        let deps = vec![create_test_dependency("dep1", "^1.0.0")];
        let pkg = create_test_package("test-pkg", "1.0.0", Some(deps));

        // Check upgrades
        let upgrades = upgrader.check_package_upgrades(&pkg).unwrap();

        // Should find major upgrade for dep1
        assert_eq!(upgrades.len(), 1);
        assert_eq!(upgrades[0].dependency_name, "dep1");
        assert_eq!(upgrades[0].compatible_version, Some("2.0.0".to_string()));
        assert!(matches!(upgrades[0].status, UpgradeStatus::MajorAvailable(_)));
    }

    #[test]
    fn test_prerelease_handling() {
        let (_, manager) = setup_test_registry();
        let mut upgrader = DependencyUpgrader::with_registry_manager(manager);

        // Configure to include prereleases
        upgrader.set_config(UpgradeConfig {
            version_stability: VersionStability::IncludePrerelease,
            ..Default::default()
        });

        // Create package with dependency on stable version
        let deps = vec![create_test_dependency("dep3", "^1.0.0-beta")];
        let pkg = create_test_package("test-pkg", "1.0.0", Some(deps));

        // Check upgrades
        let upgrades = upgrader.check_package_upgrades(&pkg).unwrap();

        // Should suggest stable version
        assert_eq!(upgrades.len(), 1);
        assert_eq!(upgrades[0].dependency_name, "dep3");
        assert_eq!(upgrades[0].compatible_version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_dependency_filtering() {
        let (_, manager) = setup_test_registry();
        let mut upgrader = DependencyUpgrader::with_registry_manager(manager);

        // Configure to target specific dependencies
        upgrader.set_config(UpgradeConfig {
            target_dependencies: vec!["dep1".to_string()],
            ..Default::default()
        });

        // Create package with multiple dependencies
        let deps = vec![
            create_test_dependency("dep1", "^1.0.0"),
            create_test_dependency("dep2", "^2.0.0"),
        ];
        let pkg = create_test_package("test-pkg", "1.0.0", Some(deps));

        // Check upgrades
        let upgrades = upgrader.check_package_upgrades(&pkg).unwrap();

        // Should only find upgrades for dep1
        assert_eq!(upgrades.len(), 1);
        assert_eq!(upgrades[0].dependency_name, "dep1");
    }

    #[test]
    fn test_package_filtering() {
        let (_, manager) = setup_test_registry();
        let mut upgrader = DependencyUpgrader::with_registry_manager(manager);

        // Configure to target specific packages
        upgrader.set_config(UpgradeConfig {
            target_packages: vec!["included-pkg".to_string()],
            ..Default::default()
        });

        // Create two packages
        let deps1 = vec![create_test_dependency("dep1", "^1.0.0")];
        let pkg1 = create_test_package("included-pkg", "1.0.0", Some(deps1));

        let deps2 = vec![create_test_dependency("dep1", "^1.0.0")];
        let pkg2 = create_test_package("excluded-pkg", "1.0.0", Some(deps2));

        // Check upgrades for both packages
        let upgrades1 = upgrader.check_package_upgrades(&pkg1).unwrap();
        let upgrades2 = upgrader.check_package_upgrades(&pkg2).unwrap();

        // Should only find upgrades for included package
        assert_eq!(upgrades1.len(), 1);
        assert!(upgrades2.is_empty());
    }

    #[test]
    fn test_apply_upgrades() {
        let (_, manager) = setup_test_registry();
        let mut upgrader = DependencyUpgrader::with_registry_manager(manager);

        // Configure to apply upgrades
        upgrader.set_config(UpgradeConfig {
            execution_mode: ExecutionMode::Apply,
            ..Default::default()
        });

        // Create package with dependency
        let deps = vec![create_test_dependency("dep1", "^1.0.0")];
        let pkg = Rc::new(RefCell::new(create_test_package("test-pkg", "1.0.0", Some(deps))));

        // Create an upgrade to apply
        let upgrades = vec![AvailableUpgrade {
            package_name: "test-pkg".to_string(),
            dependency_name: "dep1".to_string(),
            current_version: "^1.0.0".to_string(),
            compatible_version: Some("1.1.0".to_string()),
            latest_version: Some("2.0.0".to_string()),
            status: UpgradeStatus::MinorAvailable("1.1.0".to_string()),
        }];

        // Apply the upgrades
        let applied = upgrader.apply_upgrades(&[Rc::clone(&pkg)], &upgrades).unwrap();

        // Verify the upgrade was applied
        assert_eq!(applied.len(), 1);

        // Check the package dependency was updated
        let pkg_borrow = pkg.borrow();
        let updated_dep = pkg_borrow.dependencies()[0].borrow();
        assert_eq!(updated_dep.version_str(), "^1.1.0");
    }

    #[test]
    fn test_upgrade_report_generation() {
        let (_, manager) = setup_test_registry();
        let upgrader = DependencyUpgrader::with_registry_manager(manager);

        // Create some upgrades
        let upgrades = vec![
            AvailableUpgrade {
                package_name: "pkg1".to_string(),
                dependency_name: "dep1".to_string(),
                current_version: "^1.0.0".to_string(),
                compatible_version: Some("1.1.0".to_string()),
                latest_version: Some("2.0.0".to_string()),
                status: UpgradeStatus::MinorAvailable("1.1.0".to_string()),
            },
            AvailableUpgrade {
                package_name: "pkg1".to_string(),
                dependency_name: "dep2".to_string(),
                current_version: "^2.0.0".to_string(),
                compatible_version: Some("3.0.0".to_string()),
                latest_version: Some("3.0.0".to_string()),
                status: UpgradeStatus::MajorAvailable("3.0.0".to_string()),
            },
        ];

        // Generate report
        let report = upgrader.generate_upgrade_report(&upgrades);

        // Verify the report contains key information
        assert!(report.contains("Available dependency upgrades"));
        assert!(report.contains("pkg1"));
        assert!(report.contains("dep1"));
        assert!(report.contains("dep2"));
        assert!(report.contains("1.0.0"));
        assert!(report.contains("1.1.0"));
        assert!(report.contains("minor"));
        assert!(report.contains("major"));
    }

    #[test]
    fn test_upgrade_status_display() {
        // Test display format for upgrade statuses
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
            UpgradeStatus::Constrained("2.0.0".to_string()).to_string(),
            "constrained (latest: 2.0.0)"
        );
        assert_eq!(
            UpgradeStatus::CheckFailed("Error".to_string()).to_string(),
            "check failed: Error"
        );
    }

    #[test]
    fn test_available_upgrade_display() {
        // Test display format for available upgrades
        let upgrade = AvailableUpgrade {
            package_name: "pkg1".to_string(),
            dependency_name: "dep1".to_string(),
            current_version: "^1.0.0".to_string(),
            compatible_version: Some("1.1.0".to_string()),
            latest_version: Some("2.0.0".to_string()),
            status: UpgradeStatus::MinorAvailable("1.1.0".to_string()),
        };

        let display = format!("{upgrade}");
        assert_eq!(display, "dep1 in pkg1: ^1.0.0 -> 1.1.0");

        // Test with no compatible version
        let upgrade = AvailableUpgrade {
            package_name: "pkg1".to_string(),
            dependency_name: "dep1".to_string(),
            current_version: "^1.0.0".to_string(),
            compatible_version: None,
            latest_version: Some("2.0.0".to_string()),
            status: UpgradeStatus::Constrained("2.0.0".to_string()),
        };

        let display = format!("{upgrade}");
        assert_eq!(display, "dep1 in pkg1: ^1.0.0 -> no compatible version");
    }
}
