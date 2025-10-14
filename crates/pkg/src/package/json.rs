//! Package.json data structures and parsing logic.
//!
//! This module provides comprehensive data structures for representing the contents
//! of package.json files, following Node.js package.json specifications. It supports
//! all standard fields and provides type-safe access to package metadata.
//!
//! # What
//!
//! Defines Rust structures that map to package.json fields:
//! - Core package information (name, version, description)
//! - Dependencies and dependency types (runtime, dev, peer, optional)
//! - Scripts and repository information
//! - Author and contributor metadata
//! - Workspace and monorepo configuration
//!
//! # How
//!
//! Uses serde for JSON serialization/deserialization with custom handling
//! for flexible field types. Provides validation and normalization of
//! package.json data according to Node.js standards.
//!
//! # Why
//!
//! Type-safe representation of package.json ensures reliable access to
//! package metadata and prevents runtime errors from malformed data.
//! Supports all common package.json use cases including monorepos.

use crate::error::{PackageError, PackageResult};
use crate::version::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use sublime_standard_tools::filesystem::AsyncFileSystem;

/// Represents a complete package.json file structure.
///
/// This structure contains all standard package.json fields with proper typing
/// and validation. It supports both single packages and workspace configurations
/// for monorepos.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::package::PackageJson;
/// use std::str::FromStr;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let json_content = r#"
/// {
///   "name": "my-package",
///   "version": "1.0.0",
///   "description": "A sample package",
///   "main": "index.js"
/// }
/// "#;
///
/// let package: PackageJson = serde_json::from_str(json_content)?;
/// assert_eq!(package.name, "my-package");
/// assert_eq!(package.version.to_string(), "1.0.0");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageJson {
    /// Package name (required)
    pub name: String,

    /// Package version (required)
    pub version: Version,

    /// Package description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Main entry point
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main: Option<String>,

    /// Module entry point (ES modules)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,

    /// TypeScript type definitions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<String>,

    /// Alternative to 'types'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typings: Option<String>,

    /// Package keywords
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,

    /// Package homepage URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// Package license
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Package author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<PersonOrString>,

    /// Package contributors
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contributors: Vec<PersonOrString>,

    /// Repository information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<Repository>,

    /// Bug tracking information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bugs: Option<BugsInfo>,

    /// Scripts section
    #[serde(default, skip_serializing_if = "Scripts::is_empty")]
    pub scripts: Scripts,

    /// Runtime dependencies
    #[serde(default, skip_serializing_if = "Dependencies::is_empty")]
    pub dependencies: Dependencies,

    /// Development dependencies
    #[serde(rename = "devDependencies", default, skip_serializing_if = "Dependencies::is_empty")]
    pub dev_dependencies: Dependencies,

    /// Peer dependencies
    #[serde(rename = "peerDependencies", default, skip_serializing_if = "Dependencies::is_empty")]
    pub peer_dependencies: Dependencies,

    /// Optional dependencies
    #[serde(
        rename = "optionalDependencies",
        default,
        skip_serializing_if = "Dependencies::is_empty"
    )]
    pub optional_dependencies: Dependencies,

    /// Bundled dependencies
    #[serde(rename = "bundledDependencies", default, skip_serializing_if = "Vec::is_empty")]
    pub bundled_dependencies: Vec<String>,

    /// Alternative name for bundled dependencies
    #[serde(rename = "bundleDependencies", default, skip_serializing_if = "Vec::is_empty")]
    pub bundle_dependencies: Vec<String>,

    /// Package engines requirements
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub engines: HashMap<String, String>,

    /// Operating system requirements
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub os: Vec<String>,

    /// CPU architecture requirements
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cpu: Vec<String>,

    /// Package is private (not published)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,

    /// Package publishing configuration
    #[serde(rename = "publishConfig", skip_serializing_if = "Option::is_none")]
    pub publish_config: Option<PublishConfig>,

    /// Workspace configuration (for monorepos)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<WorkspaceConfig>,

    /// Files to include when publishing
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,

    /// Binary executables provided by package
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub bin: HashMap<String, String>,

    /// Man pages provided by package
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub man: Vec<String>,

    /// Directories configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directories: Option<DirectoriesConfig>,

    /// Configuration for package manager
    #[serde(rename = "packageManager", skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,

    /// Additional metadata not covered by standard fields
    #[serde(flatten)]
    pub metadata: PackageJsonMetadata,
}

impl PackageJson {
    /// Reads and parses a package.json file from the filesystem.
    ///
    /// # Arguments
    ///
    /// * `filesystem` - The filesystem implementation to use for reading
    /// * `path` - Path to the package.json file
    ///
    /// # Returns
    ///
    /// A parsed PackageJson instance
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::package::PackageJson;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let package = PackageJson::read_from_path(&fs, Path::new("./package.json")).await?;
    /// println!("Package: {}", package.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_from_path<F>(filesystem: &F, path: &Path) -> PackageResult<Self>
    where
        F: AsyncFileSystem + Send + Sync,
    {
        let content = filesystem.read_file_string(path).await.map_err(|e| {
            PackageError::operation(
                "read_package_json",
                format!("Failed to read {}: {}", path.display(), e),
            )
        })?;

        Self::parse_from_str(&content).map_err(|e| {
            PackageError::operation(
                "parse_package_json",
                format!("Failed to parse {}: {}", path.display(), e),
            )
        })
    }

    /// Parses package.json content from a string.
    ///
    /// # Arguments
    ///
    /// * `content` - JSON content as a string
    ///
    /// # Returns
    ///
    /// A parsed PackageJson instance
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is malformed or missing required fields
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::PackageJson;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = r#"{"name": "test", "version": "1.0.0"}"#;
    /// let package = PackageJson::parse_from_str(content)?;
    /// assert_eq!(package.name, "test");
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_from_str(content: &str) -> PackageResult<Self> {
        serde_json::from_str(content).map_err(PackageError::Json)
    }

    /// Serializes the package.json to a formatted JSON string.
    ///
    /// # Returns
    ///
    /// A pretty-formatted JSON string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::PackageJson;
    /// use sublime_pkg_tools::version::Version;
    /// use std::str::FromStr;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package = PackageJson {
    ///     name: "test-package".to_string(),
    ///     version: Version::from_str("1.0.0")?,
    ///     ..Default::default()
    /// };
    ///
    /// let json = package.to_pretty_json()?;
    /// assert!(json.contains("test-package"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_pretty_json(&self) -> PackageResult<String> {
        serde_json::to_string_pretty(self).map_err(PackageError::Json)
    }

    /// Gets all dependencies regardless of type.
    ///
    /// # Returns
    ///
    /// A vector of (name, version, type) tuples for all dependencies
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{PackageJson, DependencyType};
    /// use std::collections::HashMap;
    ///
    /// # fn example() {
    /// let mut package = PackageJson::default();
    /// package.dependencies.insert("lodash".to_string(), "^4.17.21".to_string());
    /// package.dev_dependencies.insert("jest".to_string(), "^29.0.0".to_string());
    ///
    /// let all_deps = package.get_all_dependencies();
    /// assert_eq!(all_deps.len(), 2);
    /// # }
    /// ```
    pub fn get_all_dependencies(&self) -> Vec<(String, String, DependencyType)> {
        let mut deps = Vec::new();

        for (name, version) in &self.dependencies.0 {
            deps.push((name.clone(), version.clone(), DependencyType::Runtime));
        }

        for (name, version) in &self.dev_dependencies.0 {
            deps.push((name.clone(), version.clone(), DependencyType::Development));
        }

        for (name, version) in &self.peer_dependencies.0 {
            deps.push((name.clone(), version.clone(), DependencyType::Peer));
        }

        for (name, version) in &self.optional_dependencies.0 {
            deps.push((name.clone(), version.clone(), DependencyType::Optional));
        }

        deps
    }

    /// Gets a dependency version regardless of type.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name to find
    ///
    /// # Returns
    ///
    /// The version and type if found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{PackageJson, DependencyType};
    ///
    /// # fn example() {
    /// let mut package = PackageJson::default();
    /// package.dependencies.insert("lodash".to_string(), "^4.17.21".to_string());
    ///
    /// let dep = package.get_dependency("lodash");
    /// assert!(dep.is_some());
    /// let (version, dep_type) = dep.unwrap();
    /// assert_eq!(version, "^4.17.21");
    /// assert_eq!(dep_type, DependencyType::Runtime);
    /// # }
    /// ```
    pub fn get_dependency(&self, name: &str) -> Option<(String, DependencyType)> {
        if let Some(version) = self.dependencies.0.get(name) {
            return Some((version.clone(), DependencyType::Runtime));
        }

        if let Some(version) = self.dev_dependencies.0.get(name) {
            return Some((version.clone(), DependencyType::Development));
        }

        if let Some(version) = self.peer_dependencies.0.get(name) {
            return Some((version.clone(), DependencyType::Peer));
        }

        if let Some(version) = self.optional_dependencies.0.get(name) {
            return Some((version.clone(), DependencyType::Optional));
        }

        None
    }

    /// Checks if the package has any workspace configuration.
    ///
    /// # Returns
    ///
    /// True if this package.json defines workspaces
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{PackageJson, WorkspaceConfig};
    ///
    /// # fn example() {
    /// let mut package = PackageJson::default();
    /// assert!(!package.is_workspace_root());
    ///
    /// package.workspaces = Some(WorkspaceConfig::Packages(vec!["packages/*".to_string()]));
    /// assert!(package.is_workspace_root());
    /// # }
    /// ```
    pub fn is_workspace_root(&self) -> bool {
        self.workspaces.is_some()
    }

    /// Gets the workspace patterns if this is a workspace root.
    ///
    /// # Returns
    ///
    /// A vector of workspace patterns if configured
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{PackageJson, WorkspaceConfig};
    ///
    /// # fn example() {
    /// let mut package = PackageJson::default();
    /// package.workspaces = Some(WorkspaceConfig::Packages(vec!["packages/*".to_string()]));
    ///
    /// let patterns = package.get_workspace_patterns();
    /// assert_eq!(patterns, vec!["packages/*"]);
    /// # }
    /// ```
    pub fn get_workspace_patterns(&self) -> Vec<String> {
        match &self.workspaces {
            Some(WorkspaceConfig::Packages(patterns)) => patterns.clone(),
            Some(WorkspaceConfig::Detailed { packages, .. }) => packages.clone(),
            None => Vec::new(),
        }
    }
}

impl Default for PackageJson {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: Version::new(0, 0, 0),
            description: None,
            main: None,
            module: None,
            types: None,
            typings: None,
            keywords: Vec::new(),
            homepage: None,
            license: None,
            author: None,
            contributors: Vec::new(),
            repository: None,
            bugs: None,
            scripts: Scripts::default(),
            dependencies: Dependencies::default(),
            dev_dependencies: Dependencies::default(),
            peer_dependencies: Dependencies::default(),
            optional_dependencies: Dependencies::default(),
            bundled_dependencies: Vec::new(),
            bundle_dependencies: Vec::new(),
            engines: HashMap::new(),
            os: Vec::new(),
            cpu: Vec::new(),
            private: None,
            publish_config: None,
            workspaces: None,
            files: Vec::new(),
            bin: HashMap::new(),
            man: Vec::new(),
            directories: None,
            package_manager: None,
            metadata: PackageJsonMetadata::default(),
        }
    }
}

/// Represents a person as either a string or structured object.
///
/// Node.js package.json allows author and contributors to be either
/// a simple string or an object with name, email, and url fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum PersonOrString {
    /// Simple string representation
    String(String),
    /// Structured person object
    Person(Person),
}

/// Structured person information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Person {
    /// Person's name
    pub name: String,
    /// Person's email address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Person's URL/homepage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Repository information for the package.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Repository {
    /// Simple string URL
    String(String),
    /// Detailed repository object
    Detailed {
        /// Repository type (usually "git")
        #[serde(rename = "type")]
        repo_type: String,
        /// Repository URL
        url: String,
        /// Directory within repository (for monorepos)
        #[serde(skip_serializing_if = "Option::is_none")]
        directory: Option<String>,
    },
}

/// Bug tracking information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum BugsInfo {
    /// Simple string URL
    String(String),
    /// Detailed bugs object
    Detailed {
        /// Bug tracker URL
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// Email for bug reports
        #[serde(skip_serializing_if = "Option::is_none")]
        email: Option<String>,
    },
}

/// Package scripts section.
///
/// Wrapper around HashMap to provide convenience methods and
/// proper serialization behavior.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Scripts(pub HashMap<String, String>);

impl Scripts {
    /// Creates a new empty scripts section.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Checks if the scripts section is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Inserts a script.
    pub fn insert(&mut self, name: String, command: String) -> Option<String> {
        self.0.insert(name, command)
    }

    /// Gets a script by name.
    pub fn get(&self, name: &str) -> Option<&String> {
        self.0.get(name)
    }

    /// Removes a script by name.
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.0.remove(name)
    }
}

/// Package dependencies section.
///
/// Wrapper around HashMap to provide convenience methods and
/// proper serialization behavior.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Dependencies(pub HashMap<String, String>);

impl Dependencies {
    /// Creates a new empty dependencies section.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Checks if the dependencies section is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Inserts a dependency.
    pub fn insert(&mut self, name: String, version: String) -> Option<String> {
        self.0.insert(name, version)
    }

    /// Gets a dependency version by name.
    pub fn get(&self, name: &str) -> Option<&String> {
        self.0.get(name)
    }

    /// Removes a dependency by name.
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.0.remove(name)
    }

    /// Gets all dependency names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }

    /// Gets all dependency entries.
    pub fn entries(&self) -> impl Iterator<Item = (&String, &String)> {
        self.0.iter()
    }
}

/// Type of dependency relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Runtime dependency
    Runtime,
    /// Development dependency
    Development,
    /// Peer dependency
    Peer,
    /// Optional dependency
    Optional,
}

/// Workspace configuration for monorepos.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum WorkspaceConfig {
    /// Simple array of workspace patterns
    Packages(Vec<String>),
    /// Detailed workspace configuration
    Detailed {
        /// Workspace package patterns
        packages: Vec<String>,
        /// Packages to exclude from workspace
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        nohoist: Vec<String>,
    },
}

/// Publishing configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublishConfig {
    /// Registry URL for publishing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    /// Access level (public/restricted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<String>,
    /// Tag to publish as
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// Directories configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectoriesConfig {
    /// Library directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lib: Option<String>,
    /// Binary directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin: Option<String>,
    /// Man pages directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub man: Option<String>,
    /// Documentation directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    /// Example directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    /// Test directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<String>,
}

/// Additional metadata not covered by standard fields.
///
/// This captures any additional fields in the package.json that
/// are not part of the standard Node.js specification.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PackageJsonMetadata {
    /// Additional fields not covered by standard package.json
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

impl PackageJsonMetadata {
    /// Creates new empty metadata.
    pub fn new() -> Self {
        Self { additional: HashMap::new() }
    }

    /// Checks if metadata is empty.
    pub fn is_empty(&self) -> bool {
        self.additional.is_empty()
    }

    /// Gets a metadata field by key.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.additional.get(key)
    }

    /// Inserts a metadata field.
    pub fn insert(&mut self, key: String, value: serde_json::Value) -> Option<serde_json::Value> {
        self.additional.insert(key, value)
    }

    /// Removes a metadata field.
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.additional.remove(key)
    }
}
