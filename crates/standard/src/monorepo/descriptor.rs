//! # Monorepo Descriptor Implementation
//!
//! ## What
//! This file implements the `MonorepoDescriptor` struct, providing methods to
//! analyze and query monorepo structures, packages, and dependencies.
//!
//! ## How
//! The implementation provides methods for accessing monorepo properties,
//! finding packages by name or path, and generating dependency graphs.
//!
//! ## Why
//! Monorepos have complex relationships between packages that need to be
//! navigated and analyzed. This implementation provides the tools to
//! efficiently work with these relationships.

use super::{MonorepoDescriptor, MonorepoKind, WorkspacePackage};
use crate::node::{PackageManager, RepoKind};
use crate::project::{ProjectInfo, ProjectKind, ProjectValidationStatus};

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

impl MonorepoDescriptor {
    /// Creates a new `MonorepoDescriptor` instance.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of monorepo (npm, yarn, pnpm, etc.)
    /// * `root` - The root directory of the monorepo
    /// * `packages` - A vector of packages found in the monorepo
    /// * `package_manager` - Optional package manager for this monorepo
    /// * `package_json` - Optional root package.json content
    /// * `validation_status` - Validation status of the monorepo
    ///
    /// # Returns
    ///
    /// A new `MonorepoDescriptor` instance with the provided properties.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind, WorkspacePackage};
    /// use sublime_standard_tools::project::ProjectValidationStatus;
    ///
    /// let root = PathBuf::from("/projects/my-monorepo");
    /// let packages = vec![
    ///     // Package definitions would go here
    /// ];
    ///
    /// let descriptor = MonorepoDescriptor::new(
    ///     MonorepoKind::YarnWorkspaces,
    ///     root,
    ///     packages,
    ///     None, // package_manager
    ///     None, // package_json
    ///     ProjectValidationStatus::NotValidated
    /// );
    /// ```
    #[must_use]
    pub fn new(
        kind: MonorepoKind,
        root: PathBuf,
        packages: Vec<WorkspacePackage>,
        package_manager: Option<PackageManager>,
        package_json: Option<package_json::PackageJson>,
        validation_status: ProjectValidationStatus,
    ) -> Self {
        // Build name-to-package map for quick lookups
        let mut name_to_package = HashMap::new();

        for (i, package) in packages.iter().enumerate() {
            name_to_package.insert(package.name.clone(), i);
        }

        Self {
            kind,
            root,
            packages,
            name_to_package,
            package_manager,
            package_json,
            validation_status,
        }
    }

    /// Creates a new `MonorepoDescriptor` instance with minimal information.
    ///
    /// This is a convenience constructor for cases where only basic information
    /// is available during initial detection.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of monorepo (npm, yarn, pnpm, etc.)
    /// * `root` - The root directory of the monorepo
    /// * `packages` - A vector of packages found in the monorepo
    ///
    /// # Returns
    ///
    /// A new `MonorepoDescriptor` instance with validation status set to `NotValidated`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind, WorkspacePackage};
    ///
    /// let root = PathBuf::from("/projects/my-monorepo");
    /// let packages = vec![
    ///     // Package definitions would go here
    /// ];
    ///
    /// let descriptor = MonorepoDescriptor::minimal(
    ///     MonorepoKind::YarnWorkspaces,
    ///     root,
    ///     packages
    /// );
    /// ```
    #[must_use]
    pub fn minimal(kind: MonorepoKind, root: PathBuf, packages: Vec<WorkspacePackage>) -> Self {
        Self::new(kind, root, packages, None, None, ProjectValidationStatus::NotValidated)
    }

    /// Returns the kind of monorepo.
    ///
    /// # Returns
    ///
    /// A reference to the `MonorepoKind` indicating what type of monorepo this is.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind};
    /// #
    /// # let descriptor = MonorepoDescriptor::new(
    /// #     MonorepoKind::YarnWorkspaces,
    /// #     PathBuf::from("/fake/path"),
    /// #     vec![]
    /// # );
    /// #
    /// let kind = descriptor.kind();
    /// assert!(matches!(kind, MonorepoKind::YarnWorkspaces));
    /// ```
    #[must_use]
    pub fn kind(&self) -> &MonorepoKind {
        &self.kind
    }

    /// Returns the root directory of the monorepo.
    ///
    /// # Returns
    ///
    /// A reference to the Path of the monorepo's root directory.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind};
    /// #
    /// # let descriptor = MonorepoDescriptor::new(
    /// #     MonorepoKind::YarnWorkspaces,
    /// #     PathBuf::from("/projects/my-monorepo"),
    /// #     vec![]
    /// # );
    /// #
    /// let root = descriptor.root();
    /// assert_eq!(root, PathBuf::from("/projects/my-monorepo"));
    /// ```
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns all packages in the monorepo.
    ///
    /// # Returns
    ///
    /// A slice containing all `WorkspacePackage` instances in the monorepo.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind};
    /// #
    /// # let descriptor = MonorepoDescriptor::new(
    /// #     MonorepoKind::YarnWorkspaces,
    /// #     PathBuf::from("/fake/path"),
    /// #     vec![]
    /// # );
    /// #
    /// let packages = descriptor.packages();
    /// println!("Found {} packages", packages.len());
    /// ```
    #[must_use]
    pub fn packages(&self) -> &[WorkspacePackage] {
        &self.packages
    }

    /// Gets a package by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the package to find
    ///
    /// # Returns
    ///
    /// * `Some(&WorkspacePackage)` - If a package with the given name exists
    /// * `None` - If no package with the given name exists
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind, WorkspacePackage};
    /// #
    /// # let ui_pkg = WorkspacePackage {
    /// #     name: "ui".to_string(),
    /// #     version: "1.0.0".to_string(),
    /// #     location: PathBuf::from("packages/ui"),
    /// #     absolute_path: PathBuf::from("/projects/monorepo/packages/ui"),
    /// #     workspace_dependencies: vec![],
    /// #     workspace_dev_dependencies: vec![],
    /// # };
    /// #
    /// # let descriptor = MonorepoDescriptor::new(
    /// #     MonorepoKind::YarnWorkspaces,
    /// #     PathBuf::from("/projects/monorepo"),
    /// #     vec![ui_pkg]
    /// # );
    /// #
    /// if let Some(package) = descriptor.get_package("ui") {
    ///     println!("Found UI package at {}", package.location.display());
    /// }
    /// ```
    #[must_use]
    pub fn get_package(&self, name: &str) -> Option<&WorkspacePackage> {
        self.name_to_package.get(name).map(|&i| &self.packages[i])
    }

    /// Generates a dependency graph for the monorepo.
    ///
    /// This method creates a mapping from package names to the packages
    /// that depend on them (their dependents).
    ///
    /// # Returns
    ///
    /// A `HashMap` mapping package names to vectors of their dependent packages.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind, WorkspacePackage};
    /// #
    /// # let pkg1 = WorkspacePackage {
    /// #     name: "shared".to_string(),
    /// #     version: "1.0.0".to_string(),
    /// #     location: PathBuf::from("packages/shared"),
    /// #     absolute_path: PathBuf::from("/fake/path/packages/shared"),
    /// #     workspace_dependencies: vec![],
    /// #     workspace_dev_dependencies: vec![],
    /// # };
    /// #
    /// # let pkg2 = WorkspacePackage {
    /// #     name: "app".to_string(),
    /// #     version: "1.0.0".to_string(),
    /// #     location: PathBuf::from("packages/app"),
    /// #     absolute_path: PathBuf::from("/fake/path/packages/app"),
    /// #     workspace_dependencies: vec!["shared".to_string()],
    /// #     workspace_dev_dependencies: vec![],
    /// # };
    /// #
    /// # let descriptor = MonorepoDescriptor::new(
    /// #     MonorepoKind::YarnWorkspaces,
    /// #     PathBuf::from("/fake/path"),
    /// #     vec![pkg1, pkg2]
    /// # );
    /// #
    /// let graph = descriptor.get_dependency_graph();
    ///
    /// // Find all packages that depend on "shared"
    /// if let Some(dependents) = graph.get("shared") {
    ///     println!("{} packages depend on shared", dependents.len());
    ///     for pkg in dependents {
    ///         println!("  - {}", pkg.name);
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn get_dependency_graph(&self) -> HashMap<&str, Vec<&WorkspacePackage>> {
        let mut dependency_graph: HashMap<&str, Vec<&WorkspacePackage>> = HashMap::new();

        // Initialize the graph with empty vectors for each package
        for package in &self.packages {
            dependency_graph.insert(&package.name, Vec::new());
        }

        // Populate the graph by iterating through all packages
        for package in &self.packages {
            // Add this package as a dependent to each of its dependencies
            for dep_name in &package.workspace_dependencies {
                if let Some(dependents) = dependency_graph.get_mut(dep_name.as_str()) {
                    dependents.push(package);
                }
            }

            // Also do the same for dev dependencies
            for dep_name in &package.workspace_dev_dependencies {
                if let Some(dependents) = dependency_graph.get_mut(dep_name.as_str()) {
                    dependents.push(package);
                }
            }
        }

        dependency_graph
    }

    /// Finds all workspace dependencies of a given package.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The name of the package to find dependencies for
    ///
    /// # Returns
    ///
    /// A vector of references to the `WorkspacePackage` objects that the
    /// specified package depends on.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind, WorkspacePackage};
    /// #
    /// # let pkg1 = WorkspacePackage {
    /// #     name: "shared".to_string(),
    /// #     version: "1.0.0".to_string(),
    /// #     location: PathBuf::from("packages/shared"),
    /// #     absolute_path: PathBuf::from("/fake/path/packages/shared"),
    /// #     workspace_dependencies: vec![],
    /// #     workspace_dev_dependencies: vec![],
    /// # };
    /// #
    /// # let pkg2 = WorkspacePackage {
    /// #     name: "app".to_string(),
    /// #     version: "1.0.0".to_string(),
    /// #     location: PathBuf::from("packages/app"),
    /// #     absolute_path: PathBuf::from("/fake/path/packages/app"),
    /// #     workspace_dependencies: vec!["shared".to_string()],
    /// #     workspace_dev_dependencies: vec![],
    /// # };
    /// #
    /// # let descriptor = MonorepoDescriptor::new(
    /// #     MonorepoKind::YarnWorkspaces,
    /// #     PathBuf::from("/fake/path"),
    /// #     vec![pkg1, pkg2]
    /// # );
    /// #
    /// // Find all dependencies of the "app" package
    /// let deps = descriptor.find_dependencies_by_name("app");
    /// for dep in deps {
    ///     println!("app depends on: {}", dep.name);
    /// }
    /// ```
    #[must_use]
    pub fn find_dependencies_by_name(&self, package_name: &str) -> Vec<&WorkspacePackage> {
        // First, find the package
        if let Some(package) = self.get_package(package_name) {
            // Collect all dependencies (both regular and dev)
            let all_deps: Vec<&String> = package
                .workspace_dependencies
                .iter()
                .chain(package.workspace_dev_dependencies.iter())
                .collect();

            // Return the corresponding package references
            all_deps.into_iter().filter_map(|dep_name| self.get_package(dep_name)).collect()
        } else {
            Vec::new()
        }
    }

    /// Finds the package that contains a specific path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to locate within the monorepo
    ///
    /// # Returns
    ///
    /// * `Some(&WorkspacePackage)` - If the path is within a package
    /// * `None` - If the path is not within any package
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::{Path, PathBuf};
    /// # use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind, WorkspacePackage};
    /// #
    /// # let pkg = WorkspacePackage {
    /// #     name: "ui".to_string(),
    /// #     version: "1.0.0".to_string(),
    /// #     location: PathBuf::from("packages/ui"),
    /// #     absolute_path: PathBuf::from("/projects/monorepo/packages/ui"),
    /// #     workspace_dependencies: vec![],
    /// #     workspace_dev_dependencies: vec![],
    /// # };
    /// #
    /// # let descriptor = MonorepoDescriptor::new(
    /// #     MonorepoKind::YarnWorkspaces,
    /// #     PathBuf::from("/projects/monorepo"),
    /// #     vec![pkg]
    /// # );
    /// #
    /// // Find which package contains a file
    /// let file_path = Path::new("/projects/monorepo/packages/ui/src/Button.js");
    /// if let Some(package) = descriptor.find_package_for_path(file_path) {
    ///     println!("File is in package: {}", package.name);
    /// }
    /// ```
    #[must_use]
    pub fn find_package_for_path(&self, path: &Path) -> Option<&WorkspacePackage> {
        // Normalize and make path absolute for comparison
        let abs_path = if path.is_absolute() { path.to_path_buf() } else { self.root.join(path) };

        self.packages.iter().find(|pkg| abs_path.starts_with(&pkg.absolute_path))
    }
}

impl ProjectInfo for MonorepoDescriptor {
    fn root(&self) -> &Path {
        &self.root
    }

    fn package_manager(&self) -> Option<&PackageManager> {
        self.package_manager.as_ref()
    }

    fn package_json(&self) -> Option<&package_json::PackageJson> {
        self.package_json.as_ref()
    }

    fn validation_status(&self) -> &ProjectValidationStatus {
        &self.validation_status
    }

    fn kind(&self) -> ProjectKind {
        ProjectKind::Repository(RepoKind::Monorepo(self.kind.clone()))
    }
}

impl MonorepoDescriptor {
    /// Sets the package manager for this monorepo.
    ///
    /// # Arguments
    ///
    /// * `package_manager` - The package manager to set
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind};
    /// # use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
    /// # use sublime_standard_tools::project::ProjectValidationStatus;
    /// #
    /// # let mut descriptor = MonorepoDescriptor::minimal(
    /// #     MonorepoKind::YarnWorkspaces,
    /// #     PathBuf::from("/projects/my-monorepo"),
    /// #     vec![]
    /// # );
    /// #
    /// let manager = PackageManager::new(PackageManagerKind::Yarn, "/projects/my-monorepo");
    /// descriptor.set_package_manager(Some(manager));
    /// ```
    pub fn set_package_manager(&mut self, package_manager: Option<PackageManager>) {
        self.package_manager = package_manager;
    }

    /// Sets the root package.json content for this monorepo.
    ///
    /// # Arguments
    ///
    /// * `package_json` - The root package.json content to set
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind};
    /// # use package_json::PackageJson;
    /// #
    /// # let mut descriptor = MonorepoDescriptor::minimal(
    /// #     MonorepoKind::YarnWorkspaces,
    /// #     PathBuf::from("/projects/my-monorepo"),
    /// #     vec![]
    /// # );
    /// #
    /// // Assuming package_json is loaded from file
    /// // descriptor.set_package_json(Some(package_json));
    /// ```
    pub fn set_package_json(&mut self, package_json: Option<package_json::PackageJson>) {
        self.package_json = package_json;
    }

    /// Sets the validation status for this monorepo.
    ///
    /// # Arguments
    ///
    /// * `validation_status` - The validation status to set
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind};
    /// # use sublime_standard_tools::project::ProjectValidationStatus;
    /// #
    /// # let mut descriptor = MonorepoDescriptor::minimal(
    /// #     MonorepoKind::YarnWorkspaces,
    /// #     PathBuf::from("/projects/my-monorepo"),
    /// #     vec![]
    /// # );
    /// #
    /// descriptor.set_validation_status(ProjectValidationStatus::Valid);
    /// ```
    pub fn set_validation_status(&mut self, validation_status: ProjectValidationStatus) {
        self.validation_status = validation_status;
    }

    /// Gets a mutable reference to the validation status.
    ///
    /// This is useful for validators that need to update the status in place.
    ///
    /// # Returns
    ///
    /// A mutable reference to the monorepo's validation status.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind};
    /// # use sublime_standard_tools::project::ProjectValidationStatus;
    /// #
    /// # let mut descriptor = MonorepoDescriptor::minimal(
    /// #     MonorepoKind::YarnWorkspaces,
    /// #     PathBuf::from("/projects/my-monorepo"),
    /// #     vec![]
    /// # );
    /// #
    /// *descriptor.validation_status_mut() = ProjectValidationStatus::Valid;
    /// ```
    pub fn validation_status_mut(&mut self) -> &mut ProjectValidationStatus {
        &mut self.validation_status
    }
}

impl Clone for MonorepoDescriptor {
    fn clone(&self) -> Self {
        let json: Option<package_json::PackageJson> = {
            let root_path = self.root.clone();
            let pkg_json_path = root_path.join("package.json");
            let mut pkg_json =
                package_json::PackageJsonManager::with_file_path(pkg_json_path.as_path());

            if pkg_json.read_ref().is_ok() {
                let json_ref = pkg_json.as_ref();
                match serde_json::to_value(json_ref).and_then(serde_json::from_value) {
                    Ok(value) => Some(value),
                    Err(e) => {
                        log::warn!(
                            "Failed to serialize/deserialize package.json at {}: {}",
                            pkg_json_path.display(),
                            e
                        );
                        None
                    }
                }
            } else {
                None
            }
        };

        Self {
            kind: self.kind.clone(),
            root: self.root.clone(),
            packages: self.packages.clone(),
            name_to_package: self.name_to_package.clone(),
            package_manager: self.package_manager.clone(),
            package_json: json,
            validation_status: self.validation_status.clone(),
        }
    }
}
