#[cfg(test)]
#[allow(clippy::print_stdout)]
#[allow(clippy::uninlined_format_args)]
mod package_tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use ws_pkg::error::Result;
    use ws_pkg::types::package::PackageInfo;
    use ws_pkg::{Dependency, DependencyGraph, DependencyRegistry, Package};

    #[test]
    fn test_dependency() -> Result<()> {
        let dep_foo = Rc::new(RefCell::new(Dependency::new("@scope/foo", "1.0.0")?));

        assert_eq!(dep_foo.borrow().name(), "@scope/foo");
        assert_eq!(dep_foo.borrow().version_str(), "^1.0.0");

        let dep_bar = Rc::new(RefCell::new(Dependency::new("@scope/bar", "1.0.0")?));
        dep_bar.borrow().update_version("2.0.0")?;

        assert_eq!(dep_bar.borrow().name(), "@scope/bar");
        assert_eq!(dep_bar.borrow().version_str(), "^2.0.0");

        Ok(())
    }

    #[test]
    fn test_package() -> Result<()> {
        let mut registry = DependencyRegistry::new();
        let dep_foo = registry.get_or_create("@scope/foo", "1.0.0")?;
        let dep_bar = registry.get_or_create("@scope/bar", "1.1.0")?;

        let pkg_charlie = Rc::new(RefCell::new(Package::new(
            "@scope/charlie",
            "0.1.1",
            Some(vec![Rc::clone(&dep_bar), Rc::clone(&dep_foo)]),
        )?));

        assert_eq!(pkg_charlie.borrow().name(), "@scope/charlie");
        assert_eq!(pkg_charlie.borrow().version_str(), "0.1.1");
        assert_eq!(pkg_charlie.borrow().dependencies().len(), 2);
        assert_eq!(pkg_charlie.borrow().dependencies()[0].borrow().name(), "@scope/bar");
        assert_eq!(pkg_charlie.borrow().dependencies()[0].borrow().version_str(), "^1.1.0");
        assert_eq!(pkg_charlie.borrow().dependencies()[1].borrow().name(), "@scope/foo");
        assert_eq!(pkg_charlie.borrow().dependencies()[1].borrow().version_str(), "^1.0.0");

        pkg_charlie.borrow().update_version("0.2.0")?;
        pkg_charlie.borrow().update_dependency_version("@scope/foo", "2.0.0")?;

        assert_eq!(pkg_charlie.borrow().version_str(), "0.2.0");
        assert_eq!(pkg_charlie.borrow().dependencies()[1].borrow().version_str(), "^2.0.0");
        assert_eq!(dep_foo.borrow().version_str(), "^2.0.0");

        Ok(())
    }

    #[test]
    fn test_package_info() -> Result<()> {
        // Create a dependency
        let mut registry = DependencyRegistry::new();
        let dep_foo = registry.get_or_create("@scope/foo", "2.0.0")?;

        // Create a package with the dependency
        let pkg = Package::new("@scope/bar", "1.0.0", Some(vec![Rc::clone(&dep_foo)]))?;

        // Create package JSON
        let mut pkg_json = serde_json::Map::new();
        pkg_json.insert("name".to_string(), serde_json::Value::String("@scope/bar".to_string()));
        pkg_json.insert("version".to_string(), serde_json::Value::String("1.0.0".to_string()));

        let mut deps_map = serde_json::Map::new();
        deps_map.insert("@scope/foo".to_string(), serde_json::Value::String("2.0.0".to_string()));
        pkg_json.insert("dependencies".to_string(), serde_json::Value::Object(deps_map));

        // Create package info
        let pkg_info = PackageInfo::new(
            pkg.clone(),
            String::from("/path/to/package.json"),
            String::from("/path/to"),
            String::from("path/to"),
            serde_json::Value::Object(pkg_json),
        );

        // Update version through package info
        pkg_info.update_version("2.0.0")?;

        // Update dependency version through package info
        pkg_info.update_dependency_version("@scope/foo", "3.0.0")?;

        // Verify package version was updated
        assert_eq!(pkg_info.package.borrow().version_str(), "2.0.0");

        // Verify original dependency reference was updated
        assert_eq!(dep_foo.borrow().version_str(), "^3.0.0");

        // Verify package info JSON was updated
        assert_eq!(pkg_info.pkg_json.borrow()["version"].as_str().unwrap(), "2.0.0");
        assert_eq!(
            pkg_info.pkg_json.borrow()["dependencies"]["@scope/foo"].as_str().unwrap(),
            "3.0.0"
        );

        Ok(())
    }

    #[test]
    fn test_multiple_packages_share_dependency() -> Result<()> {
        // Create a shared dependency
        let mut registry = DependencyRegistry::new();
        let dep_foo = registry.get_or_create("@scope/foo", "2.0.0")?;

        // Create two packages with the same dependency
        let pkg1 = Rc::new(RefCell::new(Package::new(
            "@scope/bar",
            "1.0.0",
            Some(vec![Rc::clone(&dep_foo)]),
        )?));

        let pkg2 = Rc::new(RefCell::new(Package::new(
            "@scope/baz",
            "1.0.0",
            Some(vec![Rc::clone(&dep_foo)]),
        )?));

        // Update dependency through first package
        pkg1.borrow().update_dependency_version("@scope/foo", "3.0.0")?;

        // Verify all instances see the update
        assert_eq!(dep_foo.borrow().version_str(), "^3.0.0");
        assert_eq!(pkg1.borrow().dependencies()[0].borrow().version_str(), "^3.0.0");
        assert_eq!(pkg2.borrow().dependencies()[0].borrow().version_str(), "^3.0.0");

        Ok(())
    }

    #[test]
    fn test_dependency_graph() -> Result<()> {
        let mut registry = DependencyRegistry::new();

        // Create actual package nodes for the dependencies
        let pkg_foo = Package::new("@scope/foo", "1.0.0", None)?;
        let pkg_bar = Package::new("@scope/bar", "1.1.0", None)?;
        let pkg_baz = Package::new("@scope/baz", "1.2.0", None)?;

        // Then create dependencies that reference these packages
        let dep_foo = registry.get_or_create("@scope/foo", "1.0.0")?;
        let dep_bar = registry.get_or_create("@scope/bar", "1.1.0")?;
        let dep_baz = registry.get_or_create("@scope/baz", "1.2.0")?;

        let pkg_charlie = Package::new(
            "@scope/charlie",
            "0.1.1",
            Some(vec![Rc::clone(&dep_bar), Rc::clone(&dep_foo)]),
        )?;

        let pkg_delta = Package::new(
            "@scope/delta",
            "0.2.1",
            Some(vec![Rc::clone(&dep_baz), Rc::clone(&dep_foo)]),
        )?;

        let pkg_echo = Package::new(
            "@scope/echo",
            "0.3.1",
            Some(vec![Rc::clone(&dep_baz), Rc::clone(&dep_bar)]),
        )?;

        // Include all packages in the graph
        let pkgs = [pkg_foo, pkg_bar, pkg_baz, pkg_charlie, pkg_delta, pkg_echo];

        let dependency_graph = DependencyGraph::from(&pkgs[..]);

        // Now we can get the dependents of foo, which should be charlie and delta
        let dependents = dependency_graph.get_dependents(&"@scope/foo".to_string())?;

        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&"@scope/charlie".to_string()));
        assert!(dependents.contains(&"@scope/delta".to_string()));

        // Now we can get the dependents of bar, which should be charlie and echo
        let dependents = dependency_graph.get_dependents(&"@scope/bar".to_string())?;

        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&"@scope/charlie".to_string()));
        assert!(dependents.contains(&"@scope/echo".to_string()));

        Ok(())
    }

    #[test]
    fn test_circular_dependency_detection() -> Result<()> {
        let mut registry = DependencyRegistry::new();

        // Create packages with circular dependencies
        let dep_bar = registry.get_or_create("@scope/bar", "1.0.0")?;
        let dep_foo = registry.get_or_create("@scope/foo", "1.0.0")?;

        let pkg_foo = Package::new("@scope/foo", "1.0.0", Some(vec![Rc::clone(&dep_bar)]))?;

        let pkg_bar = Package::new("@scope/bar", "1.0.0", Some(vec![Rc::clone(&dep_foo)]))?;

        // Build graph with circular dependency
        let pkgs = [pkg_foo, pkg_bar];
        let dependency_graph = DependencyGraph::from(&pkgs[..]);

        // Detect circular dependency
        let result = dependency_graph.detect_circular_dependencies();
        assert!(result.is_err());

        if let Err(ws_pkg::error::PkgError::CircularDependency { path }) = result {
            assert_eq!(path.len(), 2);
            assert!(path.contains(&"@scope/foo".to_string()));
            assert!(path.contains(&"@scope/bar".to_string()));
        } else {
            panic!("Expected CircularDependency error");
        }

        Ok(())
    }
}
