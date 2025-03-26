use napi::bindgen_prelude::*;
use sublime_package_tools::Dependency as SublimeDependency;

use super::error::{version_format_napi_error, JsVersionError};

/// Represents a package dependency with a name and version.
///
/// This class provides functionality to create, inspect and manipulate
/// package dependencies in Sublime Text packages.
#[napi]
pub struct Dependency {
    pub(crate) inner: SublimeDependency,
}

#[napi]
#[allow(clippy::inherent_to_string)]
impl Dependency {
    /// Creates a new Dependency instance with the specified name and version.
    ///
    /// ```js
    /// const { Dependency } = require('sublime-package-tools');
    ///
    /// // Create a new dependency
    /// const dep = new Dependency('package-name', '>=1.0.0');
    ///
    /// // This will throw if the version format is invalid
    /// try {
    ///   const invalidDep = new Dependency('package-name', 'not-a-version');
    /// } catch (err) {
    ///   console.error('Invalid version format:', err.message);
    /// }
    /// ```
    ///
    /// @param {string} name - The name of the dependency
    /// @param {string} version - The version requirement (semver format)
    /// @throws {JsVersionError} - If the version format is invalid
    #[napi(constructor)]
    pub fn new(name: String, version: String) -> Result<Self, JsVersionError> {
        let inner = SublimeDependency::new(&name, &version).map_err(version_format_napi_error)?;
        Ok(Self { inner })
    }

    /// Gets the name of the dependency.
    ///
    /// ```js
    /// const dep = new Dependency('package-name', '>=1.0.0');
    /// console.log(dep.name); // Outputs: package-name
    /// ```
    ///
    /// @returns {string} The name of the dependency
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Gets the version requirement of the dependency.
    ///
    /// ```js
    /// const dep = new Dependency('package-name', '>=1.0.0');
    /// console.log(dep.version); // Outputs: >=1.0.0
    /// ```
    ///
    /// @returns {string} The version requirement
    #[napi(getter)]
    pub fn version(&self) -> String {
        self.inner.version().to_string()
    }

    /// Returns a string representation of the dependency.
    ///
    /// ```js
    /// const dep = new Dependency('package-name', '>=1.0.0');
    /// console.log(dep.toString()); // Outputs: package-name@>=1.0.0
    /// ```
    ///
    /// @returns {string} String representation in the format "name@version"
    #[napi]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Updates the version requirement of the dependency.
    ///
    /// ```js
    /// const dep = new Dependency('package-name', '>=1.0.0');
    ///
    /// // Update to a new version requirement
    /// try {
    ///   dep.update_version('>=2.0.0');
    ///   console.log(dep.version); // Outputs: >=2.0.0
    /// } catch (err) {
    ///   console.error('Invalid version format:', err.message);
    /// }
    /// ```
    ///
    /// @param {string} new_version - The new version requirement (semver format)
    /// @throws {JsVersionError} - If the new version format is invalid
    #[napi]
    pub fn update_version(&self, new_version: String) -> Result<(), JsVersionError> {
        self.inner.update_version(&new_version).map_err(version_format_napi_error)
    }

    /// Checks if a given version string satisfies this dependency's version requirement.
    ///
    /// ```js
    /// const dep = new Dependency('package-name', '>=1.0.0');
    ///
    /// try {
    ///   console.log(dep.matches('1.0.0'));  // Outputs: true
    ///   console.log(dep.matches('0.9.0'));  // Outputs: false
    ///   console.log(dep.matches('2.0.0'));  // Outputs: true
    /// } catch (err) {
    ///   console.error('Invalid version format:', err.message);
    /// }
    /// ```
    ///
    /// @param {string} other - The version to check against this dependency's requirement
    /// @returns {boolean} True if the given version satisfies this dependency's requirement
    /// @throws {JsVersionError} - If the version format is invalid
    #[napi]
    pub fn matches(&self, other: String) -> Result<bool, JsVersionError> {
        self.inner.matches(&other).map_err(version_format_napi_error)
    }
}
