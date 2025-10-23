//! Tests for common traits module.

use super::*;
use std::collections::HashMap;

// Test implementation of Named
struct TestNamed {
    name: String,
}

impl Named for TestNamed {
    fn name(&self) -> &str {
        &self.name
    }
}

// Test implementation of Versionable
struct TestVersionable {
    version: Version,
}

impl Versionable for TestVersionable {
    fn version(&self) -> &Version {
        &self.version
    }
}

// Test implementation of Identifiable
struct TestIdentifiable {
    name: String,
    version: Version,
}

impl Named for TestIdentifiable {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Versionable for TestIdentifiable {
    fn version(&self) -> &Version {
        &self.version
    }
}

impl Identifiable for TestIdentifiable {}

// Test implementation of HasDependencies
struct TestHasDependencies {
    deps: HashMap<PackageName, String>,
    dev_deps: HashMap<PackageName, String>,
    peer_deps: HashMap<PackageName, String>,
}

impl HasDependencies for TestHasDependencies {
    fn dependencies(&self) -> &HashMap<PackageName, String> {
        &self.deps
    }

    fn dev_dependencies(&self) -> &HashMap<PackageName, String> {
        &self.dev_deps
    }

    fn peer_dependencies(&self) -> &HashMap<PackageName, String> {
        &self.peer_deps
    }
}

#[test]
fn test_named_trait() {
    let item = TestNamed { name: "@myorg/core".to_string() };
    assert_eq!(item.name(), "@myorg/core");
}

#[test]
fn test_versionable_trait() {
    let version = Version::parse("1.2.3").map_err(|e| format!("Failed to parse version: {:?}", e));
    assert!(version.is_ok(), "Version parsing should succeed");

    if let Ok(v) = version {
        let item = TestVersionable { version: v };
        assert_eq!(item.version().to_string(), "1.2.3");
    }
}

#[test]
fn test_identifiable_trait() {
    let version = Version::parse("1.2.3").map_err(|e| format!("Failed to parse version: {:?}", e));
    assert!(version.is_ok(), "Version parsing should succeed");

    if let Ok(v) = version {
        let item = TestIdentifiable { name: "@myorg/core".to_string(), version: v };

        assert_eq!(item.name(), "@myorg/core");
        assert_eq!(item.version().to_string(), "1.2.3");
        assert_eq!(item.identifier(), "@myorg/core@1.2.3");
    }
}

#[test]
fn test_identifiable_default_identifier() {
    let version = Version::parse("2.0.0").map_err(|e| format!("Failed to parse version: {:?}", e));
    assert!(version.is_ok(), "Version parsing should succeed");

    if let Ok(v) = version {
        let item = TestIdentifiable { name: "my-package".to_string(), version: v };

        let id = item.identifier();
        assert!(id.contains("my-package"), "Identifier should contain package name");
        assert!(id.contains("2.0.0"), "Identifier should contain version");
        assert_eq!(id, "my-package@2.0.0");
    }
}

#[test]
fn test_has_dependencies_trait() {
    let mut deps = HashMap::new();
    deps.insert("react".to_string(), "^18.0.0".to_string());

    let mut dev_deps = HashMap::new();
    dev_deps.insert("typescript".to_string(), "^5.0.0".to_string());

    let mut peer_deps = HashMap::new();
    peer_deps.insert("react-dom".to_string(), "^18.0.0".to_string());

    let item = TestHasDependencies { deps, dev_deps, peer_deps };

    assert_eq!(item.dependencies().len(), 1);
    assert_eq!(item.dev_dependencies().len(), 1);
    assert_eq!(item.peer_dependencies().len(), 1);
}

#[test]
fn test_all_dependencies_merges_correctly() {
    let mut deps = HashMap::new();
    deps.insert("react".to_string(), "^18.0.0".to_string());

    let mut dev_deps = HashMap::new();
    dev_deps.insert("typescript".to_string(), "^5.0.0".to_string());

    let mut peer_deps = HashMap::new();
    peer_deps.insert("react-dom".to_string(), "^18.0.0".to_string());

    let item = TestHasDependencies { deps, dev_deps, peer_deps };

    let all = item.all_dependencies();
    assert_eq!(all.len(), 3);
    assert!(all.contains_key("react"));
    assert!(all.contains_key("typescript"));
    assert!(all.contains_key("react-dom"));
}

#[test]
fn test_all_dependencies_empty() {
    let deps = HashMap::new();
    let dev_deps = HashMap::new();
    let peer_deps = HashMap::new();

    let item = TestHasDependencies { deps, dev_deps, peer_deps };

    let all = item.all_dependencies();
    assert_eq!(all.len(), 0);
}

#[test]
fn test_has_dependencies_with_overlapping_names() {
    let mut deps = HashMap::new();
    deps.insert("lodash".to_string(), "^4.17.21".to_string());

    let mut dev_deps = HashMap::new();
    dev_deps.insert("lodash".to_string(), "^4.17.20".to_string());

    let peer_deps = HashMap::new();

    let item = TestHasDependencies { deps, dev_deps, peer_deps };

    let all = item.all_dependencies();
    // Should have 1 entry (later inserts overwrite)
    assert_eq!(all.len(), 1);
    assert!(all.contains_key("lodash"));
}

#[test]
fn test_named_with_empty_string() {
    let item = TestNamed { name: String::new() };
    assert_eq!(item.name(), "");
}

#[test]
fn test_identifiable_with_prerelease_version() {
    let version =
        Version::parse("1.0.0-alpha.1").map_err(|e| format!("Failed to parse version: {:?}", e));
    assert!(version.is_ok(), "Prerelease version parsing should succeed");

    if let Ok(v) = version {
        let item = TestIdentifiable { name: "test-pkg".to_string(), version: v };
        let id = item.identifier();
        assert!(id.contains("1.0.0-alpha.1"), "Identifier should contain prerelease version");
    }
}
