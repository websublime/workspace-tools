//! Unified project representation for Node.js projects.
//!
//! This module provides a single, unified `Project` type that represents
//! all Node.js projects, whether simple or monorepo, eliminating the confusion
//! between `GenericProject` and `SimpleProject`.

#![warn(missing_docs)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use package_json::PackageJson;

use crate::monorepo::WorkspacePackage;
use crate::node::PackageManager;

use super::{ProjectInfo, ProjectKind, ProjectValidationStatus};

/// Organized representation of a project's dependencies.
///
/// This struct provides a structured way to access different types of
/// dependencies defined in a Node.js project's package.json file.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::project::Dependencies;
/// use std::collections::HashMap;
///
/// let mut deps = Dependencies::new();
/// deps.prod.insert("express".to_string(), "4.18.0".to_string());
/// deps.dev.insert("jest".to_string(), "29.0.0".to_string());
/// ```
#[derive(Debug, Clone, Default)]
pub struct Dependencies {
    /// Production dependencies (dependencies field in package.json)
    pub prod: HashMap<String, String>,
    /// Development dependencies (devDependencies field in package.json)
    pub dev: HashMap<String, String>,
    /// Peer dependencies (peerDependencies field in package.json)
    pub peer: HashMap<String, String>,
    /// Optional dependencies (optionalDependencies field in package.json)
    pub optional: HashMap<String, String>,
}

impl Dependencies {
    /// Creates a new empty Dependencies instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::project::Dependencies;
    ///
    /// let deps = Dependencies::new();
    /// assert!(deps.prod.is_empty());
    /// assert!(deps.dev.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total number of dependencies across all categories.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::project::Dependencies;
    /// use std::collections::HashMap;
    ///
    /// let mut deps = Dependencies::new();
    /// deps.prod.insert("express".to_string(), "4.18.0".to_string());
    /// deps.dev.insert("jest".to_string(), "29.0.0".to_string());
    /// assert_eq!(deps.total_count(), 2);
    /// ```
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.prod.len() + self.dev.len() + self.peer.len() + self.optional.len()
    }

    /// Checks if there are any dependencies defined.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::project::Dependencies;
    ///
    /// let deps = Dependencies::new();
    /// assert!(deps.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_count() == 0
    }

    /// Gets all dependencies as a single iterator.
    ///
    /// Returns an iterator over tuples of (dependency_name, version, dependency_type).
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::project::Dependencies;
    /// use std::collections::HashMap;
    ///
    /// let mut deps = Dependencies::new();
    /// deps.prod.insert("express".to_string(), "4.18.0".to_string());
    ///
    /// for (name, version, dep_type) in deps.all_dependencies() {
    ///     println!("{} @ {} ({})", name, version, dep_type);
    /// }
    /// ```
    pub fn all_dependencies(&self) -> impl Iterator<Item = (&str, &str, &'static str)> {
        self.prod
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str(), "prod"))
            .chain(self.dev.iter().map(|(k, v)| (k.as_str(), v.as_str(), "dev")))
            .chain(self.peer.iter().map(|(k, v)| (k.as_str(), v.as_str(), "peer")))
            .chain(self.optional.iter().map(|(k, v)| (k.as_str(), v.as_str(), "optional")))
    }
}

/// Unified representation of a Node.js project.
///
/// This struct combines the functionality of both `GenericProject` and `SimpleProject`
/// into a single, coherent type that can represent any Node.js project structure,
/// from simple single-package projects to complex monorepos.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::project::Project;
/// use sublime_standard_tools::project::ProjectKind;
/// use sublime_standard_tools::node::RepoKind;
/// use std::path::PathBuf;
///
/// // Create a simple project
/// let project = Project::new(
///     PathBuf::from("/path/to/project"),
///     ProjectKind::Repository(RepoKind::Simple),
/// );
///
/// assert!(!project.is_monorepo());
/// assert!(!project.has_internal_dependencies());
/// ```
#[derive(Debug)]
pub struct Project {
    /// Root directory containing package.json
    pub root: PathBuf,
    /// Project classification (simple, monorepo, etc.)
    pub kind: ProjectKind,
    /// Package manager information
    pub package_manager: Option<PackageManager>,
    /// Root package.json content
    pub package_json: Option<PackageJson>,
    /// External dependencies (from package.json)
    pub external_dependencies: Dependencies,
    /// Internal dependencies (monorepo packages)
    pub internal_dependencies: Vec<WorkspacePackage>,
    /// Validation status
    pub validation_status: ProjectValidationStatus,
}

impl Project {
    /// Creates a new Project instance with default configuration.
    ///
    /// # Arguments
    ///
    /// * `root` - Root directory of the project
    /// * `kind` - The type of project (simple or monorepo variant)
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::project::Project;
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use std::path::PathBuf;
    ///
    /// let project = Project::new(
    ///     PathBuf::from("/my/project"),
    ///     ProjectKind::Repository(RepoKind::Simple),
    /// );
    /// ```
    #[must_use]
    pub fn new(root: PathBuf, kind: ProjectKind) -> Self {
        Self {
            root,
            kind,
            package_manager: None,
            package_json: None,
            external_dependencies: Dependencies::default(),
            internal_dependencies: Vec::new(),
            validation_status: ProjectValidationStatus::NotValidated,
        }
    }

    /// Checks if this is a monorepo project.
    ///
    /// # Returns
    ///
    /// `true` if the project is any type of monorepo, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::project::Project;
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    /// use std::path::PathBuf;
    ///
    /// let simple = Project::new(
    ///     PathBuf::from("/simple"),
    ///     ProjectKind::Repository(RepoKind::Simple),
    /// );
    /// assert!(!simple.is_monorepo());
    ///
    /// let monorepo = Project::new(
    ///     PathBuf::from("/monorepo"),
    ///     ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces)),
    /// );
    /// assert!(monorepo.is_monorepo());
    /// ```
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        self.kind.is_monorepo()
    }

    /// Checks if the project has internal dependencies (workspace packages).
    ///
    /// This is typically true for monorepos that have been fully analyzed.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::project::Project;
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use std::path::PathBuf;
    ///
    /// let project = Project::new(
    ///     PathBuf::from("/my/project"),
    ///     ProjectKind::Repository(RepoKind::Simple),
    /// );
    /// assert!(!project.has_internal_dependencies());
    /// ```
    #[must_use]
    pub fn has_internal_dependencies(&self) -> bool {
        !self.internal_dependencies.is_empty()
    }

    /// Gets all dependencies (both external and internal).
    ///
    /// Returns an iterator that includes both external npm dependencies
    /// and internal workspace dependencies.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::project::Project;
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use std::path::PathBuf;
    ///
    /// let project = Project::new(
    ///     PathBuf::from("/my/project"),
    ///     ProjectKind::Repository(RepoKind::Simple),
    /// );
    ///
    /// for (name, version, dep_type) in project.get_all_dependencies() {
    ///     println!("{} @ {} ({})", name, version, dep_type);
    /// }
    /// ```
    pub fn get_all_dependencies(&self) -> impl Iterator<Item = (&str, &str, &'static str)> {
        self.external_dependencies.all_dependencies()
    }

    /// Gets the workspace packages for monorepo projects.
    ///
    /// # Returns
    ///
    /// A slice of workspace packages. Empty for non-monorepo projects.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::project::Project;
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use std::path::PathBuf;
    ///
    /// let project = Project::new(
    ///     PathBuf::from("/my/monorepo"),
    ///     ProjectKind::Repository(RepoKind::Simple),
    /// );
    ///
    /// let packages = project.get_workspace_packages();
    /// assert!(packages.is_empty()); // No packages until detected
    /// ```
    #[must_use]
    pub fn get_workspace_packages(&self) -> &[WorkspacePackage] {
        &self.internal_dependencies
    }

    /// Returns the root directory of the project.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::project::Project;
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use std::path::{Path, PathBuf};
    ///
    /// let project = Project::new(
    ///     PathBuf::from("/my/project"),
    ///     ProjectKind::Repository(RepoKind::Simple),
    /// );
    ///
    /// assert_eq!(project.root(), Path::new("/my/project"));
    /// ```
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the project type name.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::project::Project;
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    /// use std::path::PathBuf;
    ///
    /// let simple = Project::new(
    ///     PathBuf::from("/simple"),
    ///     ProjectKind::Repository(RepoKind::Simple),
    /// );
    /// assert_eq!(simple.project_type(), "simple");
    ///
    /// let yarn_mono = Project::new(
    ///     PathBuf::from("/mono"),
    ///     ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces)),
    /// );
    /// assert_eq!(yarn_mono.project_type(), "yarn monorepo");
    /// ```
    #[must_use]
    pub fn project_type(&self) -> String {
        self.kind.name()
    }

    /// Sets the validation status for this project.
    ///
    /// # Arguments
    ///
    /// * `status` - The validation status to set
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::project::Project;
    /// use sublime_standard_tools::project::{ProjectKind, ProjectValidationStatus};
    /// use sublime_standard_tools::node::RepoKind;
    /// use std::path::PathBuf;
    ///
    /// let mut project = Project::new(
    ///     PathBuf::from("/my/project"),
    ///     ProjectKind::Repository(RepoKind::Simple),
    /// );
    ///
    /// project.set_validation_status(ProjectValidationStatus::Valid);
    /// ```
    pub fn set_validation_status(&mut self, status: ProjectValidationStatus) {
        self.validation_status = status;
    }
}

impl ProjectInfo for Project {
    fn root(&self) -> &Path {
        &self.root
    }

    fn package_manager(&self) -> Option<&PackageManager> {
        self.package_manager.as_ref()
    }

    fn package_json(&self) -> Option<&PackageJson> {
        self.package_json.as_ref()
    }

    fn validation_status(&self) -> &ProjectValidationStatus {
        &self.validation_status
    }

    fn kind(&self) -> ProjectKind {
        self.kind.clone()
    }
}
