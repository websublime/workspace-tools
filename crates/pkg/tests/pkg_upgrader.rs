mod upgrader_tests {
    use semver::Version;
    use std::{cell::RefCell, collections::HashMap, rc::Rc};
    use ws_pkg::{
        graph::DependencyFilter, AvailableUpgrade, DependencyRegistry, DependencyUpgrader,
        ExecutionMode, Package, PackageRegistry, PkgError, Result, UpgradeConfig, UpgradeStatus,
        VersionStability, VersionUpdateStrategy,
    };

    #[derive(Default, Clone)]
    pub struct MockRegistry {
        versions: HashMap<String, Vec<String>>,
    }

    impl MockRegistry {
        /// Create a new mock registry
        pub fn new() -> Self {
            Self { versions: HashMap::new() }
        }

        /// Add a package with versions to the mock registry
        #[allow(clippy::needless_pass_by_value)]
        pub fn add_package(&mut self, name: &str, versions: Vec<&str>) {
            self.versions.insert(
                name.to_string(),
                versions.iter().map(std::string::ToString::to_string).collect(),
            );
        }
    }

    impl PackageRegistry for MockRegistry {
        fn get_latest_version(&self, package_name: &str) -> Result<Option<String>> {
            Ok(self.versions.get(package_name).and_then(|versions| {
                if versions.is_empty() {
                    return None;
                }

                // Find the highest version according to semver
                let mut latest = &versions[0];
                for version in versions.iter().skip(1) {
                    if let (Ok(latest_ver), Ok(current_ver)) =
                        (Version::parse(latest), Version::parse(version))
                    {
                        if current_ver > latest_ver {
                            latest = version;
                        }
                    }
                }
                Some(latest.clone())
            }))
        }

        fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>> {
            Ok(self.versions.get(package_name).cloned().unwrap_or_default())
        }

        fn get_package_info(&self, package_name: &str, version: &str) -> Result<serde_json::Value> {
            if let Some(versions) = self.versions.get(package_name) {
                if versions.contains(&version.to_string()) {
                    return Ok(serde_json::json!({
                        "name": package_name,
                        "version": version,
                        "dependencies": {}
                    }));
                }
            }

            Err(PkgError::Other {
                message: format!("Package {package_name}@{version} not found in mock registry"),
            })
        }
    }

    #[test]
    fn test_mock_registry() -> Result<()> {
        let mut registry = MockRegistry::new();

        // Add some mock packages
        registry.add_package("@scope/foo", vec!["1.0.0", "1.1.0", "2.0.0"]);

        // Test getting latest version
        let latest = registry.get_latest_version("@scope/foo")?;
        assert_eq!(latest, Some("2.0.0".to_string()));

        // Test getting all versions
        let versions = registry.get_all_versions("@scope/foo")?;
        assert_eq!(versions, vec!["1.0.0", "1.1.0", "2.0.0"]);

        // Test getting package info
        let info = registry.get_package_info("@scope/foo", "1.0.0")?;
        assert_eq!(info["name"], "@scope/foo");
        assert_eq!(info["version"], "1.0.0");

        Ok(())
    }

    #[test]
    fn test_basic_upgrade_check() -> Result<()> {
        // Setup mock registry with versions
        let mut registry = MockRegistry::new();
        registry.add_package("@scope/foo", vec!["1.0.0", "1.1.0", "2.0.0"]);
        registry.add_package("@scope/bar", vec!["1.0.0", "1.0.1"]);

        // Create upgrader with default config
        let mut upgrader = DependencyUpgrader::new(registry);

        // Create dependency registry
        let mut dep_registry = DependencyRegistry::new();

        // Create dependencies
        let dep_foo = dep_registry.get_or_create("@scope/foo", "1.0.0")?;
        let dep_bar = dep_registry.get_or_create("@scope/bar", "1.0.0")?;

        // Create a package with these dependencies
        let pkg = Package::new(
            "@scope/test",
            "1.0.0",
            Some(vec![Rc::clone(&dep_foo), Rc::clone(&dep_bar)]),
        )?;

        // Check for upgrades
        let upgrades = upgrader.check_package_upgrades(&pkg)?;

        // We should have found upgrades for both dependencies
        assert_eq!(upgrades.len(), 2);

        // Check the foo dependency upgrade
        let foo_upgrade = upgrades.iter().find(|&u| u.dependency_name == "@scope/foo").unwrap();
        assert_eq!(foo_upgrade.current_version, "^1.0.0");
        assert_eq!(foo_upgrade.compatible_version, Some("1.1.0".to_string()));
        assert_eq!(foo_upgrade.latest_version, Some("2.0.0".to_string()));

        // Check the bar dependency upgrade
        let bar_upgrade = upgrades.iter().find(|&u| u.dependency_name == "@scope/bar").unwrap();
        assert_eq!(bar_upgrade.current_version, "^1.0.0");
        assert_eq!(bar_upgrade.compatible_version, Some("1.0.1".to_string()));

        Ok(())
    }

    #[test]
    fn test_version_requirement_constraints() -> Result<()> {
        // Setup mock registry with versions
        let mut registry = MockRegistry::new();
        registry.add_package("@scope/foo", vec!["1.0.0", "1.1.0", "2.0.0"]);

        // Create upgrader
        let mut upgrader = DependencyUpgrader::new(registry);

        // Create dependency registry
        let mut dep_registry = DependencyRegistry::new();

        // Create dependency with pinned version (exact match)
        let dep_foo_pinned = dep_registry.get_or_create("@scope/foo", "1.0.0")?;
        dep_foo_pinned.borrow_mut().update_version("1.0.0")?;

        // Create a package with this dependency
        let pkg = Package::new("@scope/test", "1.0.0", Some(vec![Rc::clone(&dep_foo_pinned)]))?;

        // Check for upgrades
        let upgrades = upgrader.check_package_upgrades(&pkg)?;

        // Since we've pinned the version, there should be an upgrade with a constrained status
        assert_eq!(upgrades.len(), 1);
        let upgrade = &upgrades[0];

        match &upgrade.status {
            UpgradeStatus::Constrained(v) => {
                assert_eq!(v, "2.0.0");
            }
            status => panic!("Expected Constrained status but got {status:?}"),
        }

        Ok(())
    }

    #[test]
    fn test_upgrade_types() -> Result<()> {
        // Setup mock registry with versions
        let mut registry = MockRegistry::new();
        registry.add_package("@scope/patch", vec!["1.0.0", "1.0.1"]);
        registry.add_package("@scope/minor", vec!["1.0.0", "1.1.0"]);
        registry.add_package("@scope/major", vec!["1.0.0", "2.0.0"]);

        // Create upgrader that includes major updates
        let config = UpgradeConfig::from(&VersionUpdateStrategy::AllUpdates);
        let mut upgrader = DependencyUpgrader::with_config(registry, config);

        // Create dependency registry
        let mut dep_registry = DependencyRegistry::new();

        // Create dependencies
        let dep_patch = dep_registry.get_or_create("@scope/patch", "1.0.0")?;
        let dep_minor = dep_registry.get_or_create("@scope/minor", "1.0.0")?;
        let dep_major = dep_registry.get_or_create("@scope/major", "1.0.0")?;

        // Create a package with these dependencies
        let pkg = Package::new(
            "@scope/test",
            "1.0.0",
            Some(vec![Rc::clone(&dep_patch), Rc::clone(&dep_minor), Rc::clone(&dep_major)]),
        )?;

        // Check for upgrades
        let upgrades = upgrader.check_package_upgrades(&pkg)?;

        // We should have found upgrades for all dependencies
        assert_eq!(upgrades.len(), 3);

        // Check the patch upgrade
        let patch_upgrade = upgrades.iter().find(|&u| u.dependency_name == "@scope/patch").unwrap();
        match &patch_upgrade.status {
            UpgradeStatus::PatchAvailable(v) => {
                assert_eq!(v, "1.0.1");
            }
            status => panic!("Expected PatchAvailable but got {status:?}"),
        }

        // Check the minor upgrade
        let minor_upgrade = upgrades.iter().find(|&u| u.dependency_name == "@scope/minor").unwrap();
        match &minor_upgrade.status {
            UpgradeStatus::MinorAvailable(v) => {
                assert_eq!(v, "1.1.0");
            }
            status => panic!("Expected MinorAvailable but got {status:?}"),
        }

        // Check the major upgrade
        let major_upgrade = upgrades.iter().find(|&u| u.dependency_name == "@scope/major").unwrap();
        match &major_upgrade.status {
            UpgradeStatus::MajorAvailable(v) => {
                assert_eq!(v, "2.0.0");
            }
            status => panic!("Expected MajorAvailable but got {status:?}"),
        }

        Ok(())
    }

    #[test]
    fn test_upgrade_filtering() -> Result<()> {
        // Setup mock registry with versions
        let mut registry = MockRegistry::new();
        registry.add_package("@scope/foo", vec!["1.0.0", "1.1.0", "2.0.0"]);
        registry.add_package("@scope/bar", vec!["1.0.0", "1.0.1"]);

        // Create a completely explicit config
        let config = UpgradeConfig {
            dependency_types: DependencyFilter::WithDevelopment,
            update_strategy: VersionUpdateStrategy::MinorAndPatch,
            version_stability: VersionStability::StableOnly,
            target_packages: Vec::new(),
            target_dependencies: vec!["@scope/foo".to_string()], // Explicitly set target
            registries: vec!["https://registry.npmjs.org".to_string()],
            execution_mode: ExecutionMode::DryRun,
        };

        let mut upgrader = DependencyUpgrader::with_config(registry.clone(), config);

        // Create dependency registry
        let mut dep_registry = DependencyRegistry::new();

        // Create dependencies
        let dep_foo = dep_registry.get_or_create("@scope/foo", "1.0.0")?;
        let dep_bar = dep_registry.get_or_create("@scope/bar", "1.0.0")?;

        // Create a package with these dependencies
        let pkg = Package::new(
            "@scope/test",
            "1.0.0",
            Some(vec![Rc::clone(&dep_foo), Rc::clone(&dep_bar)]),
        )?;

        // Check for upgrades
        let upgrades = upgrader.check_package_upgrades(&pkg)?;

        // We should have found only one upgrade (for foo)
        assert_eq!(upgrades.len(), 1);
        assert_eq!(upgrades[0].dependency_name, "@scope/foo");
        assert_eq!(upgrades[0].compatible_version, Some("1.1.0".to_string()));

        // Create a completely explicit config to avoid any From conversions
        let config = UpgradeConfig {
            dependency_types: DependencyFilter::WithDevelopment,
            update_strategy: VersionUpdateStrategy::MinorAndPatch,
            version_stability: VersionStability::StableOnly,
            target_packages: vec!["@scope/other".to_string()], // Should NOT match our package
            target_dependencies: Vec::new(),
            registries: vec!["https://registry.npmjs.org".to_string()],
            execution_mode: ExecutionMode::DryRun,
        };

        // Create a new registry to avoid caching issues
        let mut new_registry = MockRegistry::new();
        new_registry.add_package("@scope/foo", vec!["1.0.0", "1.1.0", "2.0.0"]);
        new_registry.add_package("@scope/bar", vec!["1.0.0", "1.0.1"]);
        let mut upgrader = DependencyUpgrader::with_config(new_registry, config);

        // Check for upgrades, should find none since package doesn't match filter
        let upgrades = upgrader.check_package_upgrades(&pkg)?;
        assert!(upgrades.is_empty());

        Ok(())
    }

    #[test]
    fn test_apply_upgrades() -> Result<()> {
        // Setup mock registry
        let mut registry = MockRegistry::new();
        registry.add_package("@scope/foo", vec!["1.0.0", "1.1.0"]);

        // Create upgrader with Apply mode
        let config = UpgradeConfig { execution_mode: ExecutionMode::Apply, ..Default::default() };
        let upgrader = DependencyUpgrader::with_config(registry, config);

        // Create dependency registry
        let mut dep_registry = DependencyRegistry::new();

        // Create dependency
        let dep_foo = dep_registry.get_or_create("@scope/foo", "1.0.0")?;

        // Create a package with this dependency
        let pkg = Rc::new(RefCell::new(Package::new(
            "@scope/test",
            "1.0.0",
            Some(vec![Rc::clone(&dep_foo)]),
        )?));

        // Create upgrades to apply
        let upgrades = vec![AvailableUpgrade {
            package_name: "@scope/test".to_string(),
            dependency_name: "@scope/foo".to_string(),
            current_version: "^1.0.0".to_string(),
            compatible_version: Some("1.1.0".to_string()),
            latest_version: Some("1.1.0".to_string()),
            status: UpgradeStatus::MinorAvailable("1.1.0".to_string()),
        }];

        // Apply the upgrades
        let applied = upgrader.apply_upgrades(&[Rc::clone(&pkg)], &upgrades)?;

        // Check that the upgrades were applied
        assert_eq!(applied.len(), 1);
        assert_eq!(dep_foo.borrow().version_str(), "^1.1.0"); // Dependency was updated

        // Test with dry_run = true
        let config = UpgradeConfig::default();
        let upgrader = DependencyUpgrader::with_config(MockRegistry::new(), config);

        // Reset dependency version
        dep_foo.borrow_mut().update_version("1.0.0")?;

        // Try to apply upgrades
        let applied = upgrader.apply_upgrades(&[Rc::clone(&pkg)], &upgrades)?;

        // Check that no changes were made since dry_run is true
        assert_eq!(applied.len(), 1);
        assert_eq!(dep_foo.borrow().version_str(), "^1.0.0"); // Version didn't change

        Ok(())
    }

    #[test]
    fn test_generate_upgrade_report() {
        // Setup mock registry
        let registry = MockRegistry::new();

        // Create upgrader
        let upgrader = DependencyUpgrader::new(registry);

        // Create some upgrade objects
        let upgrades = vec![
            AvailableUpgrade {
                package_name: "@scope/pkg1".to_string(),
                dependency_name: "@scope/foo".to_string(),
                current_version: "^1.0.0".to_string(),
                compatible_version: Some("1.1.0".to_string()),
                latest_version: Some("2.0.0".to_string()),
                status: UpgradeStatus::MinorAvailable("1.1.0".to_string()),
            },
            AvailableUpgrade {
                package_name: "@scope/pkg1".to_string(),
                dependency_name: "@scope/bar".to_string(),
                current_version: "^1.0.0".to_string(),
                compatible_version: Some("1.0.1".to_string()),
                latest_version: Some("1.0.1".to_string()),
                status: UpgradeStatus::PatchAvailable("1.0.1".to_string()),
            },
            AvailableUpgrade {
                package_name: "@scope/pkg2".to_string(),
                dependency_name: "@scope/baz".to_string(),
                current_version: "^1.0.0".to_string(),
                compatible_version: Some("2.0.0".to_string()),
                latest_version: Some("2.0.0".to_string()),
                status: UpgradeStatus::MajorAvailable("2.0.0".to_string()),
            },
        ];

        // Generate report
        let report = upgrader.generate_upgrade_report(&upgrades);

        // Check that report contains all expected packages and dependencies
        assert!(report.contains("@scope/pkg1"));
        assert!(report.contains("@scope/pkg2"));
        assert!(report.contains("@scope/foo"));
        assert!(report.contains("@scope/bar"));
        assert!(report.contains("@scope/baz"));

        // Check upgrade types in report
        assert!(report.contains("minor: 1.1.0"));
        assert!(report.contains("patch: 1.0.1"));
        assert!(report.contains("major: 2.0.0"));

        // Test empty report
        let empty_report = upgrader.generate_upgrade_report(&[]);
        assert_eq!(empty_report, "All dependencies are up to date.");
    }

    #[test]
    fn test_prereleases() -> Result<()> {
        // Setup mock registry with prerelease versions
        let mut registry = MockRegistry::new();
        registry.add_package("@scope/foo", vec!["1.0.0", "1.1.0-alpha.1", "1.1.0-beta.1"]);

        // Create upgrader that includes prereleases
        let config = UpgradeConfig {
            version_stability: VersionStability::IncludePrerelease,
            ..Default::default()
        };
        let mut upgrader = DependencyUpgrader::with_config(registry, config);

        // Create dependency registry
        let mut dep_registry = DependencyRegistry::new();

        // Create dependency
        let dep_foo = dep_registry.get_or_create("@scope/foo", "1.0.0")?;

        // Create a package with this dependency
        let pkg = Package::new("@scope/test", "1.0.0", Some(vec![Rc::clone(&dep_foo)]))?;

        // Check for upgrades
        let upgrades = upgrader.check_package_upgrades(&pkg)?;

        // We should have found an upgrade to the beta version
        assert_eq!(upgrades.len(), 1);
        if !upgrades.is_empty() {
            let upgrade = &upgrades[0];
            assert_eq!(upgrade.compatible_version, Some("1.1.0-beta.1".to_string()));
        }

        // Now try without including prereleases
        let config = UpgradeConfig::default();
        let mut upgrader = DependencyUpgrader::with_config(MockRegistry::new(), config.clone());
        upgrader.set_config(config.clone());

        // Add prereleases to the registry
        let mut registry = MockRegistry::new();
        registry.add_package("@scope/foo", vec!["1.0.0", "1.1.0-alpha.1", "1.1.0-beta.1"]);
        let mut upgrader = DependencyUpgrader::with_config(registry, config);

        // Check for upgrades
        let upgrades = upgrader.check_package_upgrades(&pkg)?;

        // We should have found no upgrades since prereleases are excluded
        assert!(upgrades.is_empty());

        Ok(())
    }

    #[test]
    fn test_check_all_upgrades() -> Result<()> {
        // Setup mock registry
        let mut registry = MockRegistry::new();
        registry.add_package("@scope/foo", vec!["1.0.0", "1.1.0"]);
        registry.add_package("@scope/bar", vec!["1.0.0", "1.0.1"]);

        // Create upgrader
        let mut upgrader = DependencyUpgrader::new(registry);

        // Create dependency registry
        let mut dep_registry = DependencyRegistry::new();

        // Create dependencies
        let dep_foo = dep_registry.get_or_create("@scope/foo", "1.0.0")?;
        let dep_bar = dep_registry.get_or_create("@scope/bar", "1.0.0")?;

        // Create packages with these dependencies
        let pkg1 = Package::new("@scope/pkg1", "1.0.0", Some(vec![Rc::clone(&dep_foo)]))?;

        let pkg2 = Package::new("@scope/pkg2", "1.0.0", Some(vec![Rc::clone(&dep_bar)]))?;

        // Check all upgrades
        let upgrades = upgrader.check_all_upgrades(&[pkg1, pkg2])?;

        // We should have found upgrades for both packages
        assert_eq!(upgrades.len(), 2);

        // One upgrade should be for foo in pkg1
        let foo_upgrade = upgrades.iter().find(|&u| u.dependency_name == "@scope/foo").unwrap();
        assert_eq!(foo_upgrade.package_name, "@scope/pkg1");

        // One upgrade should be for bar in pkg2
        let bar_upgrade = upgrades.iter().find(|&u| u.dependency_name == "@scope/bar").unwrap();
        assert_eq!(bar_upgrade.package_name, "@scope/pkg2");

        Ok(())
    }
}
