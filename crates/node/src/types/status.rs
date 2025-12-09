//! Status command type definitions for Node.js bindings.
//!
//! # What
//!
//! This module defines all NAPI-compatible type structures for the status command,
//! including input parameters and response data types. These types enable JavaScript
//! and TypeScript consumers to interact with workspace status information in a
//! type-safe manner.
//!
//! # How
//!
//! Types are defined with the `#[napi(object)]` attribute to be automatically
//! exposed as JavaScript objects. The module provides:
//!
//! - **`StatusParams`**: Input parameters for the status command
//! - **`StatusData`**: Response data containing comprehensive workspace status
//! - **`RepositoryInfo`**: Repository type and monorepo information
//! - **`PackageManagerInfo`**: Package manager and lock file details
//! - **`BranchInfo`**: Current Git branch information
//! - **`ChangesetInfo`**: Pending changeset details
//! - **`PackageInfo`**: Individual package metadata
//!
//! All types implement `Clone`, `Debug`, and `Serialize` for flexibility in
//! testing and serialization scenarios.
//!
//! # Why
//!
//! The status command is a fundamental operation that retrieves information
//! about the current workspace state. These types provide:
//!
//! - **Type safety**: Strong typing for JavaScript/TypeScript consumers
//! - **Documentation**: Self-documenting API through TypeScript definitions
//! - **Consistency**: Matches the CLI JSON output structure for compatibility
//! - **Validation**: Enables parameter validation before CLI execution
//!
//! # Examples
//!
//! ## TypeScript Usage
//!
//! ```typescript
//! import { status, StatusParams, StatusData } from '@websublime/workspace-tools';
//!
//! const params: StatusParams = {
//!   root: '/path/to/workspace',
//!   configPath: '/path/to/repo.config.json' // optional
//! };
//!
//! const result = await status(params);
//!
//! if (result.success) {
//!   const data: StatusData = result.data;
//!   console.log(`Repository kind: ${data.repository.kind}`);
//!   console.log(`Package manager: ${data.packageManager.name}`);
//!   console.log(`Lock file: ${data.packageManager.lockFile}`);
//!
//!   if (data.branch) {
//!     console.log(`Current branch: ${data.branch.name}`);
//!   }
//!
//!   console.log(`Pending changesets: ${data.changesets.length}`);
//!   console.log(`Packages:`);
//!   for (const pkg of data.packages) {
//!     console.log(`  - ${pkg.name}@${pkg.version} (${pkg.path})`);
//!   }
//! }
//! ```
//!
//! ## Rust Usage (Internal)
//!
//! ```rust,ignore
//! use sublime_node_tools::types::status::{StatusParams, StatusData, RepositoryInfo};
//!
//! // Creating params for validation
//! let params = StatusParams {
//!     root: "/path/to/workspace".to_string(),
//!     config_path: None,
//! };
//!
//! // Constructing response data
//! let data = StatusData {
//!     repository: RepositoryInfo {
//!         kind: "monorepo".to_string(),
//!         monorepo_type: Some("pnpm".to_string()),
//!     },
//!     package_manager: PackageManagerInfo {
//!         name: "pnpm".to_string(),
//!         lock_file: "pnpm-lock.yaml".to_string(),
//!     },
//!     branch: Some(BranchInfo { name: "main".to_string() }),
//!     changesets: vec![],
//!     packages: vec![],
//! };
//! ```

use napi_derive::napi;
use serde::Serialize;

// ============================================================================
// Input Parameters
// ============================================================================

/// Input parameters for the status command.
///
/// This structure defines the parameters that can be passed to the `status`
/// function from JavaScript/TypeScript. The root path is required, while
/// the config path is optional.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
///
/// # TypeScript Definition
///
/// ```typescript
/// interface StatusParams {
///   /** Workspace root directory path */
///   root: string;
///   /** Optional custom config file path */
///   configPath?: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Minimal params with just root
/// const params: StatusParams = { root: '.' };
///
/// // With custom config path
/// const paramsWithConfig: StatusParams = {
///   root: '/path/to/workspace',
///   configPath: '/path/to/custom/repo.config.json'
/// };
/// ```
// Allow dead_code until Story 3.2 implements the status command
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct StatusParams {
    /// Workspace root directory path.
    ///
    /// This is the absolute or relative path to the root of the workspace.
    /// For monorepos, this should point to the root where the package manager
    /// configuration (e.g., `pnpm-workspace.yaml`) is located.
    pub root: String,

    /// Optional custom configuration file path.
    ///
    /// If not provided, the command will search for configuration files
    /// in standard locations (`repo.config.json`, `repo.config.yaml`, etc.)
    /// within the workspace root.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
}

// ============================================================================
// Response Data Types
// ============================================================================

/// Repository type information.
///
/// Contains information about the repository structure, indicating whether
/// it's a simple single-package repository or a monorepo with multiple packages.
///
/// # Fields
///
/// - `kind`: The repository type ("simple", "monorepo", or "unknown")
/// - `monorepo_type`: The monorepo type if applicable
///
/// # TypeScript Definition
///
/// ```typescript
/// interface RepositoryInfo {
///   /** Repository kind: "simple", "monorepo", or "unknown" */
///   kind: string;
///   /** Monorepo type if applicable (npm, yarn, pnpm, bun, deno, custom) */
///   monorepoType?: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Simple repository (single package)
/// const simple: RepositoryInfo = { kind: 'simple' };
///
/// // Monorepo with pnpm workspaces
/// const monorepo: RepositoryInfo = {
///   kind: 'monorepo',
///   monorepoType: 'pnpm'
/// };
/// ```
// Allow dead_code until Story 3.2 implements the status command
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct RepositoryInfo {
    /// Repository kind identifier.
    ///
    /// Possible values:
    /// - `"simple"`: Single package repository
    /// - `"monorepo"`: Multi-package workspace
    /// - `"unknown"`: Unable to determine repository type
    pub kind: String,

    /// Monorepo type if the repository is a monorepo.
    ///
    /// This field is only present when `kind` is `"monorepo"`.
    /// Possible values include:
    /// - `"npm"`: npm workspaces
    /// - `"yarn"`: Yarn workspaces
    /// - `"pnpm"`: pnpm workspaces
    /// - `"bun"`: Bun workspaces
    /// - `"deno"`: Deno workspaces
    /// - `"custom"`: Custom workspace configuration
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monorepo_type: Option<String>,
}

/// Package manager information.
///
/// Contains details about the detected package manager being used
/// in the workspace, including the lock file name.
///
/// # Fields
///
/// - `name`: The package manager name
/// - `lock_file`: The name of the lock file used
///
/// # TypeScript Definition
///
/// ```typescript
/// interface PackageManagerInfo {
///   /** Package manager name (npm, yarn, pnpm, bun, jsr, unknown) */
///   name: string;
///   /** Lock file name */
///   lockFile: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // pnpm package manager
/// const pnpm: PackageManagerInfo = {
///   name: 'pnpm',
///   lockFile: 'pnpm-lock.yaml'
/// };
///
/// // npm package manager
/// const npm: PackageManagerInfo = {
///   name: 'npm',
///   lockFile: 'package-lock.json'
/// };
/// ```
// Allow dead_code until Story 3.2 implements the status command
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct PackageManagerInfo {
    /// Package manager name.
    ///
    /// Possible values:
    /// - `"npm"`: Node Package Manager
    /// - `"yarn"`: Yarn package manager
    /// - `"pnpm"`: pnpm package manager
    /// - `"bun"`: Bun package manager
    /// - `"jsr"`: JSR package manager
    /// - `"unknown"`: Unable to detect package manager
    pub name: String,

    /// Lock file name used by the package manager.
    ///
    /// Examples:
    /// - `"package-lock.json"` for npm
    /// - `"yarn.lock"` for Yarn
    /// - `"pnpm-lock.yaml"` for pnpm
    /// - `"bun.lockb"` for Bun
    pub lock_file: String,
}

/// Git branch information.
///
/// Contains the name of the current Git branch, if available.
/// This information is useful for determining the context of
/// pending changesets and version bumps.
///
/// # Fields
///
/// - `name`: The branch name
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BranchInfo {
///   /** Branch name */
///   name: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const branch: BranchInfo = { name: 'main' };
/// const feature: BranchInfo = { name: 'feature/add-new-api' };
/// ```
// Allow dead_code until Story 3.2 implements the status command
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BranchInfo {
    /// Git branch name.
    ///
    /// This is the name of the currently checked-out branch.
    /// It does not include the `refs/heads/` prefix.
    pub name: String,
}

/// Changeset information.
///
/// Represents a pending changeset that has been created but not yet
/// consumed by a version bump operation. Each changeset is identified
/// by a unique ID derived from the branch name.
///
/// # Fields
///
/// - `id`: The unique changeset identifier
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetInfo {
///   /** Changeset ID (derived from branch name) */
///   id: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const changeset: ChangesetInfo = { id: 'feature-add-login' };
/// const fix: ChangesetInfo = { id: 'fix-memory-leak' };
/// ```
// Allow dead_code until Story 3.2 implements the status command
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetInfo {
    /// Changeset unique identifier.
    ///
    /// This ID is typically derived from the Git branch name that
    /// created the changeset. It uniquely identifies the changeset
    /// within the workspace.
    pub id: String,
}

/// Package information.
///
/// Contains metadata about a single package within the workspace,
/// including its name, version, and relative path.
///
/// # Fields
///
/// - `name`: The package name (may include scope)
/// - `version`: The current package version
/// - `path`: The package path relative to workspace root
///
/// # TypeScript Definition
///
/// ```typescript
/// interface PackageInfo {
///   /** Package name (may include scope like @org/package) */
///   name: string;
///   /** Package version (semver) */
///   version: string;
///   /** Package path relative to workspace root */
///   path: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Scoped package
/// const pkg: PackageInfo = {
///   name: '@websublime/core',
///   version: '1.2.3',
///   path: 'packages/core'
/// };
///
/// // Unscoped package
/// const utils: PackageInfo = {
///   name: 'utils',
///   version: '0.1.0',
///   path: 'packages/utils'
/// };
/// ```
// Allow dead_code until Story 3.2 implements the status command
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct PackageInfo {
    /// Package name.
    ///
    /// This is the name as defined in the package's `package.json`.
    /// It may include a scope prefix (e.g., `@organization/package-name`).
    pub name: String,

    /// Package version.
    ///
    /// The current version of the package as defined in its `package.json`.
    /// Follows semantic versioning format (e.g., `1.2.3`, `0.1.0-beta.1`).
    pub version: String,

    /// Package path relative to workspace root.
    ///
    /// This is the directory path where the package is located,
    /// relative to the workspace root directory.
    /// For simple repositories, this is typically `"."`.
    pub path: String,
}

/// Status command response data.
///
/// This is the main response structure returned by the status command,
/// containing comprehensive information about the workspace state.
///
/// # Fields
///
/// - `repository`: Repository type information
/// - `package_manager`: Package manager details
/// - `branch`: Current Git branch (if available)
/// - `changesets`: List of pending changesets
/// - `packages`: List of workspace packages
///
/// # TypeScript Definition
///
/// ```typescript
/// interface StatusData {
///   /** Repository information */
///   repository: RepositoryInfo;
///   /** Package manager information */
///   packageManager: PackageManagerInfo;
///   /** Current branch (if available) */
///   branch?: BranchInfo;
///   /** Pending changesets */
///   changesets: ChangesetInfo[];
///   /** Workspace packages */
///   packages: PackageInfo[];
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const status: StatusData = {
///   repository: { kind: 'monorepo', monorepoType: 'pnpm' },
///   packageManager: { name: 'pnpm', lockFile: 'pnpm-lock.yaml' },
///   branch: { name: 'main' },
///   changesets: [{ id: 'feature-login' }],
///   packages: [
///     { name: '@org/core', version: '1.0.0', path: 'packages/core' },
///     { name: '@org/utils', version: '0.5.0', path: 'packages/utils' }
///   ]
/// };
/// ```
// Allow dead_code until Story 3.2 implements the status command
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct StatusData {
    /// Repository type information.
    ///
    /// Contains details about whether this is a simple repository
    /// or a monorepo, and if monorepo, which type.
    pub repository: RepositoryInfo,

    /// Package manager information.
    ///
    /// Details about the detected package manager and its lock file.
    pub package_manager: PackageManagerInfo,

    /// Current Git branch information.
    ///
    /// This field is `None` if:
    /// - Git is not available
    /// - The directory is not a Git repository
    /// - Git is in a detached HEAD state
    #[napi(ts_type = "BranchInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<BranchInfo>,

    /// List of pending changesets.
    ///
    /// These are changesets that have been created but not yet
    /// consumed by a version bump operation.
    pub changesets: Vec<ChangesetInfo>,

    /// List of workspace packages.
    ///
    /// For simple repositories, this contains a single package.
    /// For monorepos, this contains all packages in the workspace.
    pub packages: Vec<PackageInfo>,
}

// ============================================================================
// Constructors and Helper Methods
// ============================================================================

#[allow(dead_code)]
impl StatusParams {
    /// Creates a new `StatusParams` instance with the given root path.
    ///
    /// This is a convenience constructor for creating params with only
    /// the required `root` field, leaving `config_path` as `None`.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// A new `StatusParams` instance with the specified root and no config path.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::status::StatusParams;
    ///
    /// let params = StatusParams::new("/path/to/workspace");
    /// assert_eq!(params.root, "/path/to/workspace");
    /// assert!(params.config_path.is_none());
    /// ```
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self { root: root.into(), config_path: None }
    }

    /// Creates a new `StatusParams` instance with root and config path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    /// * `config_path` - The custom configuration file path
    ///
    /// # Returns
    ///
    /// A new `StatusParams` instance with both root and config path set.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::status::StatusParams;
    ///
    /// let params = StatusParams::with_config(
    ///     "/path/to/workspace",
    ///     "/path/to/config.json"
    /// );
    /// assert_eq!(params.root, "/path/to/workspace");
    /// assert_eq!(params.config_path, Some("/path/to/config.json".to_string()));
    /// ```
    #[must_use]
    pub fn with_config(root: impl Into<String>, config_path: impl Into<String>) -> Self {
        Self { root: root.into(), config_path: Some(config_path.into()) }
    }
}

#[allow(dead_code)]
impl RepositoryInfo {
    /// Creates a new `RepositoryInfo` for a simple (single package) repository.
    ///
    /// # Returns
    ///
    /// A new `RepositoryInfo` with `kind` set to `"simple"` and no monorepo type.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::status::RepositoryInfo;
    ///
    /// let info = RepositoryInfo::simple();
    /// assert_eq!(info.kind, "simple");
    /// assert!(info.monorepo_type.is_none());
    /// ```
    #[must_use]
    pub fn simple() -> Self {
        Self { kind: "simple".to_string(), monorepo_type: None }
    }

    /// Creates a new `RepositoryInfo` for a monorepo.
    ///
    /// # Arguments
    ///
    /// * `monorepo_type` - The type of monorepo (e.g., "pnpm", "npm", "yarn")
    ///
    /// # Returns
    ///
    /// A new `RepositoryInfo` with `kind` set to `"monorepo"` and the specified type.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::status::RepositoryInfo;
    ///
    /// let info = RepositoryInfo::monorepo("pnpm");
    /// assert_eq!(info.kind, "monorepo");
    /// assert_eq!(info.monorepo_type, Some("pnpm".to_string()));
    /// ```
    #[must_use]
    pub fn monorepo(monorepo_type: impl Into<String>) -> Self {
        Self { kind: "monorepo".to_string(), monorepo_type: Some(monorepo_type.into()) }
    }

    /// Creates a new `RepositoryInfo` for an unknown repository type.
    ///
    /// # Returns
    ///
    /// A new `RepositoryInfo` with `kind` set to `"unknown"` and no monorepo type.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::status::RepositoryInfo;
    ///
    /// let info = RepositoryInfo::unknown();
    /// assert_eq!(info.kind, "unknown");
    /// assert!(info.monorepo_type.is_none());
    /// ```
    #[must_use]
    pub fn unknown() -> Self {
        Self { kind: "unknown".to_string(), monorepo_type: None }
    }

    /// Checks if this repository is a simple (single package) repository.
    ///
    /// # Returns
    ///
    /// `true` if the repository kind is `"simple"`, `false` otherwise.
    #[must_use]
    pub fn is_simple(&self) -> bool {
        self.kind == "simple"
    }

    /// Checks if this repository is a monorepo.
    ///
    /// # Returns
    ///
    /// `true` if the repository kind is `"monorepo"`, `false` otherwise.
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        self.kind == "monorepo"
    }
}

#[allow(dead_code)]
impl PackageManagerInfo {
    /// Creates a new `PackageManagerInfo` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The package manager name
    /// * `lock_file` - The lock file name
    ///
    /// # Returns
    ///
    /// A new `PackageManagerInfo` instance with the specified values.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::status::PackageManagerInfo;
    ///
    /// let info = PackageManagerInfo::new("pnpm", "pnpm-lock.yaml");
    /// assert_eq!(info.name, "pnpm");
    /// assert_eq!(info.lock_file, "pnpm-lock.yaml");
    /// ```
    #[must_use]
    pub fn new(name: impl Into<String>, lock_file: impl Into<String>) -> Self {
        Self { name: name.into(), lock_file: lock_file.into() }
    }

    /// Creates a `PackageManagerInfo` for npm.
    #[must_use]
    pub fn npm() -> Self {
        Self::new("npm", "package-lock.json")
    }

    /// Creates a `PackageManagerInfo` for yarn.
    #[must_use]
    pub fn yarn() -> Self {
        Self::new("yarn", "yarn.lock")
    }

    /// Creates a `PackageManagerInfo` for pnpm.
    #[must_use]
    pub fn pnpm() -> Self {
        Self::new("pnpm", "pnpm-lock.yaml")
    }

    /// Creates a `PackageManagerInfo` for bun.
    #[must_use]
    pub fn bun() -> Self {
        Self::new("bun", "bun.lockb")
    }

    /// Creates a `PackageManagerInfo` for an unknown package manager.
    #[must_use]
    pub fn unknown() -> Self {
        Self::new("unknown", "")
    }
}

#[allow(dead_code)]
impl BranchInfo {
    /// Creates a new `BranchInfo` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The branch name
    ///
    /// # Returns
    ///
    /// A new `BranchInfo` instance with the specified name.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::status::BranchInfo;
    ///
    /// let branch = BranchInfo::new("main");
    /// assert_eq!(branch.name, "main");
    /// ```
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[allow(dead_code)]
impl ChangesetInfo {
    /// Creates a new `ChangesetInfo` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - The changeset identifier
    ///
    /// # Returns
    ///
    /// A new `ChangesetInfo` instance with the specified ID.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::status::ChangesetInfo;
    ///
    /// let changeset = ChangesetInfo::new("feature-login");
    /// assert_eq!(changeset.id, "feature-login");
    /// ```
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[allow(dead_code)]
impl PackageInfo {
    /// Creates a new `PackageInfo` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The package name (may include scope)
    /// * `version` - The package version
    /// * `path` - The package path relative to workspace root
    ///
    /// # Returns
    ///
    /// A new `PackageInfo` instance with the specified values.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::status::PackageInfo;
    ///
    /// let pkg = PackageInfo::new("@org/core", "1.2.3", "packages/core");
    /// assert_eq!(pkg.name, "@org/core");
    /// assert_eq!(pkg.version, "1.2.3");
    /// assert_eq!(pkg.path, "packages/core");
    /// ```
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self { name: name.into(), version: version.into(), path: path.into() }
    }
}

#[allow(dead_code)]
impl StatusData {
    /// Creates a new `StatusData` builder for constructing status responses.
    ///
    /// This method provides a convenient way to create `StatusData` with
    /// sensible defaults and a fluent builder pattern.
    ///
    /// # Arguments
    ///
    /// * `repository` - Repository type information
    /// * `package_manager` - Package manager information
    ///
    /// # Returns
    ///
    /// A new `StatusData` instance with empty changesets and packages.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::status::{StatusData, RepositoryInfo, PackageManagerInfo};
    ///
    /// let data = StatusData::new(
    ///     RepositoryInfo::simple(),
    ///     PackageManagerInfo::pnpm(),
    /// );
    /// assert!(data.changesets.is_empty());
    /// assert!(data.packages.is_empty());
    /// ```
    #[must_use]
    pub fn new(repository: RepositoryInfo, package_manager: PackageManagerInfo) -> Self {
        Self {
            repository,
            package_manager,
            branch: None,
            changesets: Vec::new(),
            packages: Vec::new(),
        }
    }

    /// Sets the branch information.
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch information to set
    ///
    /// # Returns
    ///
    /// The modified `StatusData` instance for method chaining.
    #[must_use]
    pub fn with_branch(mut self, branch: BranchInfo) -> Self {
        self.branch = Some(branch);
        self
    }

    /// Sets the changesets list.
    ///
    /// # Arguments
    ///
    /// * `changesets` - The list of changesets
    ///
    /// # Returns
    ///
    /// The modified `StatusData` instance for method chaining.
    #[must_use]
    pub fn with_changesets(mut self, changesets: Vec<ChangesetInfo>) -> Self {
        self.changesets = changesets;
        self
    }

    /// Sets the packages list.
    ///
    /// # Arguments
    ///
    /// * `packages` - The list of packages
    ///
    /// # Returns
    ///
    /// The modified `StatusData` instance for method chaining.
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<PackageInfo>) -> Self {
        self.packages = packages;
        self
    }
}
