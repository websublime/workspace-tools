//! # Test Fixtures Module
//!
//! This module provides utilities for creating and managing test fixtures.
//!
//! ## What
//!
//! Provides fixture builders for:
//! - Test monorepo structures
//! - Single package projects
//! - Package.json files
//! - Configuration files
//! - Changeset files
//!
//! ## How
//!
//! Uses builder patterns to construct complex test fixtures programmatically.
//! Fixtures can be written to the filesystem or used with mock filesystem.
//!
//! ## Why
//!
//! Fixture builders provide:
//! - Consistent test data across tests
//! - Easy setup of complex scenarios
//! - Reusable test structures
//! - Type-safe fixture construction

use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Builder for creating package.json fixtures
///
/// This builder allows creating package.json files with various configurations
/// for testing purposes.
///
/// # Examples
///
/// ```rust,ignore
/// let package_json = PackageJsonBuilder::new("my-package")
///     .version("1.0.0")
///     .description("Test package")
///     .add_dependency("react", "^18.0.0")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct PackageJsonBuilder {
    name: String,
    version: String,
    description: Option<String>,
    dependencies: HashMap<String, String>,
    dev_dependencies: HashMap<String, String>,
    peer_dependencies: HashMap<String, String>,
    scripts: HashMap<String, String>,
    workspaces: Option<Vec<String>>,
    private: Option<bool>,
}

impl PackageJsonBuilder {
    /// Creates a new package.json builder
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    ///
    /// # Returns
    ///
    /// A new `PackageJsonBuilder` instance
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "1.0.0".to_string(),
            description: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            workspaces: None,
            private: None,
        }
    }

    /// Sets the package version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Sets the package description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a dependency
    pub fn add_dependency(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.dependencies.insert(name.into(), version.into());
        self
    }

    /// Adds multiple dependencies
    pub fn add_dependencies(mut self, deps: HashMap<String, String>) -> Self {
        self.dependencies.extend(deps);
        self
    }

    /// Adds a dev dependency
    pub fn add_dev_dependency(
        mut self,
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        self.dev_dependencies.insert(name.into(), version.into());
        self
    }

    /// Adds a peer dependency
    pub fn add_peer_dependency(
        mut self,
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        self.peer_dependencies.insert(name.into(), version.into());
        self
    }

    /// Adds a script
    pub fn add_script(mut self, name: impl Into<String>, command: impl Into<String>) -> Self {
        self.scripts.insert(name.into(), command.into());
        self
    }

    /// Sets workspaces configuration
    pub fn workspaces(mut self, patterns: Vec<String>) -> Self {
        self.workspaces = Some(patterns);
        self
    }

    /// Sets the private flag
    pub fn private(mut self, private: bool) -> Self {
        self.private = Some(private);
        self
    }

    /// Builds the package.json as a JSON string
    ///
    /// # Returns
    ///
    /// A JSON string representation of the package.json
    pub fn build(self) -> String {
        let mut json = json!({
            "name": self.name,
            "version": self.version,
        });

        let obj = json.as_object_mut().unwrap();

        if let Some(desc) = self.description {
            obj.insert("description".to_string(), json!(desc));
        }

        if let Some(private) = self.private {
            obj.insert("private".to_string(), json!(private));
        }

        if !self.scripts.is_empty() {
            obj.insert("scripts".to_string(), json!(self.scripts));
        }

        if !self.dependencies.is_empty() {
            obj.insert("dependencies".to_string(), json!(self.dependencies));
        }

        if !self.dev_dependencies.is_empty() {
            obj.insert("devDependencies".to_string(), json!(self.dev_dependencies));
        }

        if !self.peer_dependencies.is_empty() {
            obj.insert("peerDependencies".to_string(), json!(self.peer_dependencies));
        }

        if let Some(workspaces) = self.workspaces {
            obj.insert("workspaces".to_string(), json!(workspaces));
        }

        serde_json::to_string_pretty(&json).unwrap()
    }
}

/// Builder for creating monorepo test fixtures
///
/// This builder helps create complete monorepo structures for testing.
///
/// # Examples
///
/// ```rust,ignore
/// let monorepo = MonorepoFixtureBuilder::new("my-monorepo")
///     .add_package("packages/pkg1", "1.0.0")
///     .add_package("packages/pkg2", "2.0.0")
///     .build();
/// ```
#[derive(Debug)]
pub struct MonorepoFixtureBuilder {
    root_name: String,
    packages: Vec<PackageFixture>,
    workspace_patterns: Vec<String>,
}

/// Represents a package in a monorepo fixture
#[derive(Debug, Clone)]
pub struct PackageFixture {
    /// Relative path from monorepo root
    pub path: PathBuf,
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package dependencies
    pub dependencies: HashMap<String, String>,
    /// Additional files to create in the package
    pub files: HashMap<PathBuf, String>,
}

impl MonorepoFixtureBuilder {
    /// Creates a new monorepo fixture builder
    ///
    /// # Arguments
    ///
    /// * `root_name` - The name of the root package
    ///
    /// # Returns
    ///
    /// A new `MonorepoFixtureBuilder` instance
    pub fn new(root_name: impl Into<String>) -> Self {
        Self {
            root_name: root_name.into(),
            packages: Vec::new(),
            workspace_patterns: vec!["packages/*".to_string()],
        }
    }

    /// Sets custom workspace patterns
    pub fn workspace_patterns(mut self, patterns: Vec<String>) -> Self {
        self.workspace_patterns = patterns;
        self
    }

    /// Adds a package to the monorepo
    pub fn add_package(
        mut self,
        path: impl Into<PathBuf>,
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        self.packages.push(PackageFixture {
            path: path.into(),
            name: name.into(),
            version: version.into(),
            dependencies: HashMap::new(),
            files: HashMap::new(),
        });
        self
    }

    /// Adds a package with a builder
    pub fn add_package_with<F>(mut self, path: impl Into<PathBuf>, builder: F) -> Self
    where
        F: FnOnce(PackageFixtureBuilder) -> PackageFixture,
    {
        let package = builder(PackageFixtureBuilder::new(path.into()));
        self.packages.push(package);
        self
    }

    /// Builds the monorepo fixture
    ///
    /// # Returns
    ///
    /// A `MonorepoFixture` instance
    pub fn build(self) -> MonorepoFixture {
        MonorepoFixture {
            root_name: self.root_name,
            packages: self.packages,
            workspace_patterns: self.workspace_patterns,
        }
    }
}

/// Builder for creating individual package fixtures
#[derive(Debug)]
pub struct PackageFixtureBuilder {
    path: PathBuf,
    name: Option<String>,
    version: String,
    dependencies: HashMap<String, String>,
    files: HashMap<PathBuf, String>,
}

impl PackageFixtureBuilder {
    /// Creates a new package fixture builder
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            name: None,
            version: "1.0.0".to_string(),
            dependencies: HashMap::new(),
            files: HashMap::new(),
        }
    }

    /// Sets the package name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the package version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Adds a dependency
    pub fn add_dependency(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.dependencies.insert(name.into(), version.into());
        self
    }

    /// Adds a file to the package
    pub fn add_file(mut self, path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        self.files.insert(path.into(), content.into());
        self
    }

    /// Builds the package fixture
    pub fn build(self) -> PackageFixture {
        let name = self.name.unwrap_or_else(|| {
            self.path.file_name().and_then(|n| n.to_str()).unwrap_or("package").to_string()
        });

        PackageFixture {
            path: self.path,
            name,
            version: self.version,
            dependencies: self.dependencies,
            files: self.files,
        }
    }
}

/// Represents a complete monorepo fixture
#[derive(Debug)]
pub struct MonorepoFixture {
    /// Root package name
    pub root_name: String,
    /// All packages in the monorepo
    pub packages: Vec<PackageFixture>,
    /// Workspace patterns
    pub workspace_patterns: Vec<String>,
}

impl MonorepoFixture {
    /// Generates all files for the monorepo
    ///
    /// # Returns
    ///
    /// A map of file paths to their contents
    pub fn generate_files(&self) -> HashMap<PathBuf, String> {
        let mut files = HashMap::new();

        // Create root package.json
        let root_package = PackageJsonBuilder::new(&self.root_name)
            .version("0.0.0")
            .private(true)
            .workspaces(self.workspace_patterns.clone())
            .build();

        files.insert(PathBuf::from("package.json"), root_package);

        // Create package files
        for package in &self.packages {
            let package_dir = &package.path;

            // Create package.json
            let mut builder = PackageJsonBuilder::new(&package.name).version(&package.version);

            for (dep_name, dep_version) in &package.dependencies {
                builder = builder.add_dependency(dep_name, dep_version);
            }

            let package_json = builder.build();
            files.insert(package_dir.join("package.json"), package_json);

            // Create additional files
            for (file_path, content) in &package.files {
                files.insert(package_dir.join(file_path), content.clone());
            }

            // Create a basic index file if not provided
            if !package.files.contains_key(Path::new("index.js")) {
                files.insert(
                    package_dir.join("index.js"),
                    format!("// {}\nmodule.exports = {{}};\n", package.name),
                );
            }
        }

        files
    }

    /// Writes the monorepo fixture to a directory
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory to write to
    ///
    /// # Errors
    ///
    /// Returns an error if any file operations fail
    pub fn write_to_dir(&self, root: &Path) -> std::io::Result<()> {
        let files = self.generate_files();

        for (path, content) in files {
            let full_path = root.join(&path);

            // Create parent directories
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Write file
            std::fs::write(full_path, content)?;
        }

        Ok(())
    }
}

/// Creates a simple single-package fixture
///
/// # Arguments
///
/// * `name` - The package name
/// * `version` - The package version
///
/// # Returns
///
/// A map of file paths to their contents
pub fn create_single_package_fixture(
    name: impl Into<String>,
    version: impl Into<String>,
) -> HashMap<PathBuf, String> {
    let mut files = HashMap::new();

    let package_json = PackageJsonBuilder::new(name).version(version).build();

    files.insert(PathBuf::from("package.json"), package_json);
    files.insert(PathBuf::from("index.js"), "module.exports = {};\n".to_string());

    files
}

/// Creates a basic monorepo fixture with two packages
///
/// # Returns
///
/// A `MonorepoFixture` instance with default packages
pub fn create_basic_monorepo_fixture() -> MonorepoFixture {
    MonorepoFixtureBuilder::new("test-monorepo")
        .add_package("packages/pkg-a", "pkg-a", "1.0.0")
        .add_package("packages/pkg-b", "pkg-b", "2.0.0")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_json_builder_basic() {
        let json = PackageJsonBuilder::new("test-package").version("1.2.3").build();

        assert!(json.contains(r#""name": "test-package""#));
        assert!(json.contains(r#""version": "1.2.3""#));
    }

    #[test]
    fn test_package_json_builder_with_dependencies() {
        let json = PackageJsonBuilder::new("test-package")
            .add_dependency("react", "^18.0.0")
            .add_dependency("vue", "^3.0.0")
            .build();

        assert!(json.contains(r#""dependencies""#));
        assert!(json.contains(r#""react": "^18.0.0""#));
        assert!(json.contains(r#""vue": "^3.0.0""#));
    }

    #[test]
    fn test_package_json_builder_with_workspaces() {
        let json = PackageJsonBuilder::new("monorepo")
            .workspaces(vec!["packages/*".to_string()])
            .private(true)
            .build();

        assert!(json.contains(r#""private": true"#));
        assert!(json.contains(r#""workspaces""#));
    }

    #[test]
    fn test_monorepo_fixture_generation() {
        let fixture = MonorepoFixtureBuilder::new("test-repo")
            .add_package("packages/pkg1", "pkg1", "1.0.0")
            .add_package("packages/pkg2", "pkg2", "2.0.0")
            .build();

        let files = fixture.generate_files();

        // Should have root package.json + 2 packages with package.json and index.js
        assert!(files.len() >= 5);
        assert!(files.contains_key(&PathBuf::from("package.json")));
        assert!(files.contains_key(&PathBuf::from("packages/pkg1/package.json")));
        assert!(files.contains_key(&PathBuf::from("packages/pkg2/package.json")));
    }

    #[test]
    fn test_create_single_package_fixture() {
        let files = create_single_package_fixture("my-pkg", "1.0.0");

        assert_eq!(files.len(), 2);
        assert!(files.contains_key(&PathBuf::from("package.json")));
        assert!(files.contains_key(&PathBuf::from("index.js")));
    }

    #[test]
    fn test_create_basic_monorepo_fixture() {
        let fixture = create_basic_monorepo_fixture();

        assert_eq!(fixture.packages.len(), 2);
        assert_eq!(fixture.root_name, "test-monorepo");
    }
}
