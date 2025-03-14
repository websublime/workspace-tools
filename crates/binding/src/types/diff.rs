use crate::errors::handle_pkg_result;
use crate::types::package::Package;
use napi::bindgen_prelude::*;
use napi::Result as NapiResult;
use napi_derive::napi;
use ws_pkg::types::diff::{
    ChangeType as WsChangeType, DependencyChange as WsDependencyChange,
    PackageDiff as WsPackageDiff,
};

/// JavaScript binding for ws_pkg::types::diff::ChangeType enum
#[napi]
pub enum ChangeType {
    /// Package was added
    Added,
    /// Package was removed
    Removed,
    /// Package version was updated
    Updated,
    /// No change detected
    Unchanged,
}

impl From<WsChangeType> for ChangeType {
    fn from(change_type: WsChangeType) -> Self {
        match change_type {
            WsChangeType::Added => ChangeType::Added,
            WsChangeType::Removed => ChangeType::Removed,
            WsChangeType::Updated => ChangeType::Updated,
            WsChangeType::Unchanged => ChangeType::Unchanged,
        }
    }
}

impl From<ChangeType> for WsChangeType {
    fn from(change_type: ChangeType) -> Self {
        match change_type {
            ChangeType::Added => WsChangeType::Added,
            ChangeType::Removed => WsChangeType::Removed,
            ChangeType::Updated => WsChangeType::Updated,
            ChangeType::Unchanged => WsChangeType::Unchanged,
        }
    }
}

/// JavaScript binding for ws_pkg::types::diff::DependencyChange
/// Represents a change in a dependency
#[napi(object)]
pub struct DependencyChange {
    /// Name of the dependency
    pub name: String,
    /// Previous version (null if newly added)
    pub previous_version: Option<String>,
    /// Current version (null if removed)
    pub current_version: Option<String>,
    /// Type of change
    pub change_type: ChangeType,
    /// Whether this is a breaking change based on semver
    pub breaking: bool,
}

impl From<WsDependencyChange> for DependencyChange {
    fn from(change: WsDependencyChange) -> Self {
        Self {
            name: change.name,
            previous_version: change.previous_version,
            current_version: change.current_version,
            change_type: change.change_type.into(),
            breaking: change.breaking,
        }
    }
}

/// JavaScript binding for ws_pkg::types::diff::PackageDiff
/// The complete diff between two package versions
#[napi]
pub struct PackageDiff {
    inner: WsPackageDiff,
}

#[napi]
#[allow(clippy::inherent_to_string)]
impl PackageDiff {
    /// Create a new package diff between two packages
    ///
    /// @param {Package} previous - The previous package version
    /// @param {Package} current - The current package version
    /// @returns {PackageDiff} A diff of the changes between packages
    #[napi(ts_return_type = "PackageDiff")]
    pub fn between(previous: &Package, current: &Package) -> NapiResult<PackageDiff> {
        let diff = handle_pkg_result(WsPackageDiff::between(&previous.inner, &current.inner))?;
        Ok(PackageDiff { inner: diff })
    }

    /// Get the package name
    ///
    /// @returns {string} The package name
    #[napi(getter)]
    pub fn package_name(&self) -> String {
        self.inner.package_name.clone()
    }

    /// Get the previous version
    ///
    /// @returns {string} The previous version
    #[napi(getter)]
    pub fn previous_version(&self) -> String {
        self.inner.previous_version.clone()
    }

    /// Get the current version
    ///
    /// @returns {string} The current version
    #[napi(getter)]
    pub fn current_version(&self) -> String {
        self.inner.current_version.clone()
    }

    /// Get dependency changes
    ///
    /// @returns {DependencyChange[]} The dependency changes
    #[napi(getter)]
    pub fn dependency_changes(&self) -> Vec<DependencyChange> {
        self.inner.dependency_changes.iter().cloned().map(DependencyChange::from).collect()
    }

    /// Get whether this diff is a breaking change
    ///
    /// @returns {boolean} True if this is a breaking change (major version bump)
    #[napi(getter)]
    pub fn breaking_change(&self) -> bool {
        self.inner.breaking_change
    }

    /// Count the number of breaking changes in dependencies
    ///
    /// @returns {number} The count of breaking changes
    #[napi]
    pub fn count_breaking_changes(&self) -> u32 {
        self.inner.count_breaking_changes() as u32
    }

    /// Count the changes by type
    ///
    /// @returns {Object} Map of change types to counts
    #[napi(ts_return_type = "Record<string, number>")]
    pub fn count_changes_by_type(&self, env: Env) -> NapiResult<Object> {
        let counts = self.inner.count_changes_by_type();
        let mut result = env.create_object()?;

        // Convert the counts map to a JavaScript object
        if counts.contains_key(&WsChangeType::Added) {
            result.set_named_property("added", counts[&WsChangeType::Added] as i32)?;
        }
        if counts.contains_key(&WsChangeType::Removed) {
            result.set_named_property("removed", counts[&WsChangeType::Removed] as i32)?;
        }
        if counts.contains_key(&WsChangeType::Updated) {
            result.set_named_property("updated", counts[&WsChangeType::Updated] as i32)?;
        }
        if counts.contains_key(&WsChangeType::Unchanged) {
            result.set_named_property("unchanged", counts[&WsChangeType::Unchanged] as i32)?;
        }

        Ok(result)
    }

    /// Get a human-readable string representation of the diff
    ///
    /// @returns {string} A formatted string representation of the diff
    #[napi]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }
}

#[cfg(test)]
mod diff_binding_tests {
    use super::*;
    use crate::types::dependency::Dependency;

    #[test]
    fn test_change_type_conversion() {
        // Test WsChangeType -> ChangeType
        assert!(matches!(ChangeType::from(WsChangeType::Added), ChangeType::Added));
        assert!(matches!(ChangeType::from(WsChangeType::Removed), ChangeType::Removed));
        assert!(matches!(ChangeType::from(WsChangeType::Updated), ChangeType::Updated));
        assert!(matches!(ChangeType::from(WsChangeType::Unchanged), ChangeType::Unchanged));

        // Test ChangeType -> WsChangeType
        assert!(matches!(WsChangeType::from(ChangeType::Added), WsChangeType::Added));
        assert!(matches!(WsChangeType::from(ChangeType::Removed), WsChangeType::Removed));
        assert!(matches!(WsChangeType::from(ChangeType::Updated), WsChangeType::Updated));
        assert!(matches!(WsChangeType::from(ChangeType::Unchanged), WsChangeType::Unchanged));
    }

    #[test]
    fn test_dependency_change_conversion() {
        // Create a WsDependencyChange
        let ws_change = WsDependencyChange::new(
            "test-dep",
            Some("1.0.0"),
            Some("2.0.0"),
            WsChangeType::Updated,
        );

        // Convert to our DependencyChange
        let change = DependencyChange::from(ws_change);

        // Verify the conversion
        assert_eq!(change.name, "test-dep");
        assert_eq!(change.previous_version, Some("1.0.0".to_string()));
        assert_eq!(change.current_version, Some("2.0.0".to_string()));
        assert!(matches!(change.change_type, ChangeType::Updated));
        assert!(change.breaking); // Should be true since it's a major version bump
    }

    #[test]
    fn test_package_diff_basics() {
        // Create two versions of a package with dependencies
        let mut pkg1 = Package::new("test-pkg".to_string(), "1.0.0".to_string());
        let dep1 = Dependency::new("dep1".to_string(), "^1.0.0".to_string());
        pkg1.add_dependency(&dep1);

        let mut pkg2 = Package::new("test-pkg".to_string(), "2.0.0".to_string());
        let dep2 = Dependency::new("dep1".to_string(), "^2.0.0".to_string());
        pkg2.add_dependency(&dep2);

        // Create a diff between them
        let diff = PackageDiff::between(&pkg1, &pkg2).unwrap();

        // Verify basic properties
        assert_eq!(diff.package_name(), "test-pkg");
        assert_eq!(diff.previous_version(), "1.0.0");
        assert_eq!(diff.current_version(), "2.0.0");
        assert!(diff.breaking_change()); // Major version bump is breaking

        // Verify dependency changes
        let changes = diff.dependency_changes();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].name, "dep1");
        assert_eq!(changes[0].previous_version, Some("^1.0.0".to_string()));
        assert_eq!(changes[0].current_version, Some("^2.0.0".to_string()));
        assert!(matches!(changes[0].change_type, ChangeType::Updated));
        assert!(changes[0].breaking); // Major version bump is breaking
    }

    #[test]
    fn test_count_breaking_changes() {
        // Create two versions of a package with multiple dependencies
        let mut pkg1 = Package::new("test-pkg".to_string(), "1.0.0".to_string());
        let dep1 = Dependency::new("dep1".to_string(), "^1.0.0".to_string());
        let dep2 = Dependency::new("dep2".to_string(), "^1.0.0".to_string());
        pkg1.add_dependency(&dep1);
        pkg1.add_dependency(&dep2);

        let mut pkg2 = Package::new("test-pkg".to_string(), "1.1.0".to_string());
        let dep1b = Dependency::new("dep1".to_string(), "^2.0.0".to_string()); // Breaking
        let dep2b = Dependency::new("dep2".to_string(), "^1.1.0".to_string()); // Non-breaking
        pkg2.add_dependency(&dep1b);
        pkg2.add_dependency(&dep2b);

        // Create a diff
        let diff = PackageDiff::between(&pkg1, &pkg2).unwrap();

        // Verify breaking changes count
        assert_eq!(diff.count_breaking_changes(), 1);
    }

    #[test]
    fn test_to_string() {
        // Create two packages with dependencies
        let mut pkg1 = Package::new("test-pkg".to_string(), "1.0.0".to_string());
        let dep1 = Dependency::new("dep1".to_string(), "^1.0.0".to_string());
        pkg1.add_dependency(&dep1);

        let mut pkg2 = Package::new("test-pkg".to_string(), "2.0.0".to_string());
        let dep2 = Dependency::new("dep1".to_string(), "^2.0.0".to_string());
        pkg2.add_dependency(&dep2);

        // Create a diff
        let diff = PackageDiff::between(&pkg1, &pkg2).unwrap();

        // Get string representation
        let diff_str = diff.to_string();

        // Verify it contains key information
        assert!(diff_str.contains("test-pkg"));
        assert!(diff_str.contains("1.0.0"));
        assert!(diff_str.contains("2.0.0"));
        assert!(diff_str.contains("Breaking change"));
        assert!(diff_str.contains("dep1"));
    }
}
