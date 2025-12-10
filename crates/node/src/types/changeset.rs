//! Changeset command type definitions for Node.js bindings.
//!
//! # What
//!
//! This module defines all NAPI-compatible type structures for changeset commands,
//! including input parameters and response data types. Changesets are the core
//! workflow mechanism for tracking changes before version bumps, enabling
//! controlled release processes in monorepo environments.
//!
//! # How
//!
//! Types are defined with the `#[napi(object)]` attribute to be automatically
//! exposed as JavaScript objects. The module provides:
//!
//! - **Input Parameters**: `ChangesetAddParams`, `ChangesetUpdateParams`,
//!   `ChangesetListParams`, `ChangesetShowParams`, `ChangesetRemoveParams`,
//!   `ChangesetHistoryParams`, `ChangesetCheckParams`
//! - **Response Data**: `ChangesetAddData`, `ChangesetUpdateData`,
//!   `ChangesetListData`, `ChangesetShowData`, `ChangesetRemoveData`,
//!   `ChangesetHistoryData`, `ChangesetCheckData`
//! - **Supporting Types**: `ChangesetDetailInfo`, `UpdateSummaryInfo`,
//!   `ArchivedChangesetInfo`, `ReleaseInfoData`, `ReleasedVersionEntry`
//! - **API Responses**: Type-safe response wrappers for each command
//!
//! All types implement `Clone`, `Debug`, and `Serialize` for flexibility in
//! testing and serialization scenarios.
//!
//! # Why
//!
//! Changesets are the foundation of the versioning workflow. They:
//! - Track which packages have changes pending release
//! - Record the bump type (major, minor, patch) for each change
//! - Support environment-specific releases (staging, production)
//! - Maintain a history of released changesets for auditing
//!
//! These types provide:
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
//! import {
//!   changesetAdd,
//!   changesetList,
//!   changesetShow,
//!   changesetUpdate,
//!   changesetRemove,
//!   changesetHistory,
//!   changesetCheck,
//!   ChangesetAddParams,
//!   ChangesetListData
//! } from '@websublime/workspace-tools';
//!
//! // Add a new changeset
//! const addResult = await changesetAdd({
//!   root: '.',
//!   packages: ['@scope/pkg1', '@scope/pkg2'],
//!   bump: 'minor',
//!   message: 'Add new feature',
//!   environments: ['staging', 'production']
//! });
//!
//! if (addResult.success) {
//!   console.log(`Created changeset: ${addResult.data.id}`);
//!   console.log(`Branch: ${addResult.data.branch}`);
//! }
//!
//! // List pending changesets
//! const listResult = await changesetList({ root: '.' });
//! if (listResult.success) {
//!   const data: ChangesetListData = listResult.data;
//!   console.log(`Found ${data.count} changesets`);
//!   for (const cs of data.changesets) {
//!     console.log(`  ${cs.branch}: ${cs.bump} (${cs.packages.join(', ')})`);
//!   }
//! }
//!
//! // Show a specific changeset
//! const showResult = await changesetShow({ root: '.', branch: 'feature/new-api' });
//! if (showResult.success) {
//!   console.log(`Changeset details: ${JSON.stringify(showResult.data.changeset)}`);
//! }
//!
//! // Check if current branch has a changeset
//! const checkResult = await changesetCheck({ root: '.' });
//! if (checkResult.success) {
//!   if (checkResult.data.hasChangeset) {
//!     console.log('Changeset exists for this branch');
//!   } else {
//!     console.log('No changeset found - please create one');
//!   }
//! }
//! ```
//!
//! ## Rust Usage (Internal)
//!
//! ```rust,ignore
//! use sublime_node_tools::types::changeset::{
//!     ChangesetAddParams, ChangesetAddData, ChangesetDetailInfo
//! };
//!
//! // Creating params for validation
//! let params = ChangesetAddParams::new(".")
//!     .with_packages(vec!["@scope/pkg1".to_string()])
//!     .with_bump("minor".to_string())
//!     .with_message("Add new feature".to_string());
//!
//! // Constructing response data
//! let data = ChangesetAddData {
//!     id: "feature-new-api".to_string(),
//!     branch: "feature/new-api".to_string(),
//!     packages: vec!["@scope/pkg1".to_string()],
//!     bump: "minor".to_string(),
//!     environments: vec!["staging".to_string()],
//!     created_at: "2024-01-15T10:30:00Z".to_string(),
//! };
//! ```

use napi_derive::napi;
use serde::Serialize;

use crate::error::ErrorInfo;

// ============================================================================
// Constants
// ============================================================================

/// Valid sort options for changeset list command.
///
/// These values are accepted by the `sort` parameter:
/// - `"date"`: Sort by creation date (default)
/// - `"bump"`: Sort by bump type (major > minor > patch)
/// - `"branch"`: Sort alphabetically by branch name
#[allow(dead_code)]
pub(crate) const VALID_SORT_OPTIONS: &[&str] = &["date", "bump", "branch"];

// ============================================================================
// Input Parameters
// ============================================================================

/// Input parameters for the changeset add command.
///
/// This structure defines the parameters for creating a new changeset.
/// The root path is required; all other parameters are optional and will
/// use sensible defaults or auto-detection when not provided.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
/// - `bump`: The bump type (major, minor, patch)
/// - `environments`: List of environments for the changeset
/// - `branch`: Branch name (defaults to current Git branch)
/// - `message`: Optional description of the changes
/// - `packages`: List of packages to include (auto-detected if not provided)
/// - `force`: Overwrite existing changeset if one exists
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetAddParams {
///   root: string;
///   configPath?: string;
///   bump?: 'major' | 'minor' | 'patch';
///   environments?: string[];
///   branch?: string;
///   message?: string;
///   packages?: string[];
///   force?: boolean;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Minimal params - will auto-detect packages and use current branch
/// const minimal: ChangesetAddParams = { root: '.' };
///
/// // Full params with all options
/// const full: ChangesetAddParams = {
///   root: '/path/to/workspace',
///   configPath: '/path/to/repo.config.json',
///   bump: 'minor',
///   environments: ['staging', 'production'],
///   branch: 'feature/new-api',
///   message: 'Add new REST API endpoints',
///   packages: ['@scope/api', '@scope/client'],
///   force: true
/// };
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetAddParams {
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

    /// Bump type for version changes.
    ///
    /// Specifies how the version should be bumped for the affected packages.
    /// Valid values: `"major"`, `"minor"`, `"patch"`.
    ///
    /// If not provided, the command will prompt for selection in interactive
    /// mode or use the default from configuration.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bump: Option<String>,

    /// List of environments for the changeset.
    ///
    /// Environments allow targeting specific release channels (e.g., staging,
    /// production). If not provided, defaults from configuration are used.
    ///
    /// Example: `["staging", "production"]`
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environments: Option<Vec<String>>,

    /// Branch name for the changeset.
    ///
    /// If not provided, the current Git branch is used. The branch name
    /// is used to derive the changeset ID.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Optional message describing the changes.
    ///
    /// A human-readable description of what changes are included in this
    /// changeset. This message may be included in changelogs.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// List of packages to include in the changeset.
    ///
    /// Package names should match exactly as defined in each package's
    /// `package.json`. If not provided, packages are auto-detected from
    /// Git changes.
    ///
    /// Example: `["@scope/core", "@scope/utils"]`
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<String>>,

    /// Force overwrite of existing changeset.
    ///
    /// If `true`, any existing changeset for the branch will be replaced.
    /// If `false` (default), an error is returned if a changeset already exists.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

#[allow(dead_code)]
impl ChangesetAddParams {
    /// Creates a new `ChangesetAddParams` with the required root path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// A new `ChangesetAddParams` with all optional fields set to `None`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::changeset::ChangesetAddParams;
    ///
    /// let params = ChangesetAddParams::new(".");
    /// assert_eq!(params.root, ".");
    /// assert!(params.bump.is_none());
    /// ```
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            config_path: None,
            bump: None,
            environments: None,
            branch: None,
            message: None,
            packages: None,
            force: None,
        }
    }

    /// Sets the configuration path.
    ///
    /// # Arguments
    ///
    /// * `config_path` - The custom configuration file path
    ///
    /// # Returns
    ///
    /// Self with the config_path set.
    #[must_use]
    pub fn with_config_path(mut self, config_path: impl Into<String>) -> Self {
        self.config_path = Some(config_path.into());
        self
    }

    /// Sets the bump type.
    ///
    /// # Arguments
    ///
    /// * `bump` - The bump type (major, minor, patch)
    ///
    /// # Returns
    ///
    /// Self with the bump set.
    #[must_use]
    pub fn with_bump(mut self, bump: impl Into<String>) -> Self {
        self.bump = Some(bump.into());
        self
    }

    /// Sets the environments.
    ///
    /// # Arguments
    ///
    /// * `environments` - List of environment names
    ///
    /// # Returns
    ///
    /// Self with the environments set.
    #[must_use]
    pub fn with_environments(mut self, environments: Vec<String>) -> Self {
        self.environments = Some(environments);
        self
    }

    /// Sets the branch name.
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch name
    ///
    /// # Returns
    ///
    /// Self with the branch set.
    #[must_use]
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    /// Sets the message.
    ///
    /// # Arguments
    ///
    /// * `message` - The changeset message
    ///
    /// # Returns
    ///
    /// Self with the message set.
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Sets the packages.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package names
    ///
    /// # Returns
    ///
    /// Self with the packages set.
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.packages = Some(packages);
        self
    }

    /// Sets the force flag.
    ///
    /// # Arguments
    ///
    /// * `force` - Whether to force overwrite
    ///
    /// # Returns
    ///
    /// Self with the force flag set.
    #[must_use]
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = Some(force);
        self
    }
}

/// Input parameters for the changeset update command.
///
/// This structure defines the parameters for updating an existing changeset.
/// At least one update field (commit, packages, bump, or environments) should
/// be provided.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
/// - `id`: Changeset ID or branch name (defaults to current branch)
/// - `commit`: Commit hash to add to the changeset
/// - `packages`: Additional packages to add
/// - `bump`: New bump type to set
/// - `environments`: Additional environments to add
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetUpdateParams {
///   root: string;
///   configPath?: string;
///   id?: string;
///   commit?: string;
///   packages?: string[];
///   bump?: 'major' | 'minor' | 'patch';
///   environments?: string[];
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Add a commit to current branch's changeset
/// const addCommit: ChangesetUpdateParams = {
///   root: '.',
///   commit: 'abc123def456'
/// };
///
/// // Add packages to a specific changeset
/// const addPackages: ChangesetUpdateParams = {
///   root: '.',
///   id: 'feature/new-api',
///   packages: ['@scope/new-package']
/// };
///
/// // Upgrade bump type
/// const upgradeBump: ChangesetUpdateParams = {
///   root: '.',
///   bump: 'major'
/// };
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetUpdateParams {
    /// Workspace root directory path.
    pub root: String,

    /// Optional custom configuration file path.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,

    /// Changeset ID or branch name.
    ///
    /// If not provided, uses the current Git branch to identify the changeset.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Commit hash to add to the changeset.
    ///
    /// The full or abbreviated Git commit hash to associate with this changeset.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,

    /// Additional packages to add to the changeset.
    ///
    /// These packages will be added to the existing list of packages.
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<String>>,

    /// New bump type to set.
    ///
    /// Replaces the current bump type. Valid values: `"major"`, `"minor"`, `"patch"`.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bump: Option<String>,

    /// Additional environments to add.
    ///
    /// These environments will be added to the existing list.
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environments: Option<Vec<String>>,
}

#[allow(dead_code)]
impl ChangesetUpdateParams {
    /// Creates a new `ChangesetUpdateParams` with the required root path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// A new `ChangesetUpdateParams` with all optional fields set to `None`.
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            config_path: None,
            id: None,
            commit: None,
            packages: None,
            bump: None,
            environments: None,
        }
    }

    /// Sets the changeset ID or branch name.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the commit hash to add.
    #[must_use]
    pub fn with_commit(mut self, commit: impl Into<String>) -> Self {
        self.commit = Some(commit.into());
        self
    }

    /// Sets the packages to add.
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.packages = Some(packages);
        self
    }

    /// Sets the new bump type.
    #[must_use]
    pub fn with_bump(mut self, bump: impl Into<String>) -> Self {
        self.bump = Some(bump.into());
        self
    }

    /// Sets the environments to add.
    #[must_use]
    pub fn with_environments(mut self, environments: Vec<String>) -> Self {
        self.environments = Some(environments);
        self
    }
}

/// Input parameters for the changeset list command.
///
/// This structure defines the parameters for listing pending changesets.
/// All parameters are optional; when omitted, all pending changesets are
/// returned sorted by date.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
/// - `filter_package`: Filter changesets containing this package
/// - `filter_bump`: Filter by bump type
/// - `filter_env`: Filter by environment
/// - `sort`: Sort order (date, bump, branch)
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetListParams {
///   root: string;
///   configPath?: string;
///   filterPackage?: string;
///   filterBump?: 'major' | 'minor' | 'patch';
///   filterEnv?: string;
///   sort?: 'date' | 'bump' | 'branch';
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // List all changesets
/// const all: ChangesetListParams = { root: '.' };
///
/// // Filter by package and bump type
/// const filtered: ChangesetListParams = {
///   root: '.',
///   filterPackage: '@scope/core',
///   filterBump: 'major',
///   sort: 'date'
/// };
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetListParams {
    /// Workspace root directory path.
    pub root: String,

    /// Optional custom configuration file path.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,

    /// Filter by package name.
    ///
    /// Only return changesets that include the specified package.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_package: Option<String>,

    /// Filter by bump type.
    ///
    /// Only return changesets with the specified bump type.
    /// Valid values: `"major"`, `"minor"`, `"patch"`.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_bump: Option<String>,

    /// Filter by environment.
    ///
    /// Only return changesets that target the specified environment.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_env: Option<String>,

    /// Sort order for results.
    ///
    /// Valid values:
    /// - `"date"`: Sort by creation date (default, newest first)
    /// - `"bump"`: Sort by bump type (major > minor > patch)
    /// - `"branch"`: Sort alphabetically by branch name
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
}

#[allow(dead_code)]
impl ChangesetListParams {
    /// Creates a new `ChangesetListParams` with the required root path.
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            config_path: None,
            filter_package: None,
            filter_bump: None,
            filter_env: None,
            sort: None,
        }
    }

    /// Sets the package filter.
    #[must_use]
    pub fn with_filter_package(mut self, package: impl Into<String>) -> Self {
        self.filter_package = Some(package.into());
        self
    }

    /// Sets the bump type filter.
    #[must_use]
    pub fn with_filter_bump(mut self, bump: impl Into<String>) -> Self {
        self.filter_bump = Some(bump.into());
        self
    }

    /// Sets the environment filter.
    #[must_use]
    pub fn with_filter_env(mut self, env: impl Into<String>) -> Self {
        self.filter_env = Some(env.into());
        self
    }

    /// Sets the sort order.
    #[must_use]
    pub fn with_sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }
}

/// Input parameters for the changeset show command.
///
/// This structure defines the parameters for showing details of a specific
/// changeset identified by branch name or changeset ID.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
/// - `branch`: Branch name or changeset ID (required)
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetShowParams {
///   root: string;
///   configPath?: string;
///   branch: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const params: ChangesetShowParams = {
///   root: '.',
///   branch: 'feature/new-api'
/// };
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetShowParams {
    /// Workspace root directory path.
    pub root: String,

    /// Optional custom configuration file path.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,

    /// Branch name or changeset ID.
    ///
    /// The identifier of the changeset to display. This can be either
    /// the full branch name or the derived changeset ID.
    pub branch: String,
}

#[allow(dead_code)]
impl ChangesetShowParams {
    /// Creates a new `ChangesetShowParams` with required parameters.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    /// * `branch` - The branch name or changeset ID
    #[must_use]
    pub fn new(root: impl Into<String>, branch: impl Into<String>) -> Self {
        Self { root: root.into(), config_path: None, branch: branch.into() }
    }

    /// Sets the configuration path.
    #[must_use]
    pub fn with_config_path(mut self, config_path: impl Into<String>) -> Self {
        self.config_path = Some(config_path.into());
        self
    }
}

/// Input parameters for the changeset remove command.
///
/// This structure defines the parameters for removing a changeset.
/// The branch is required to identify which changeset to remove.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
/// - `branch`: Branch name or changeset ID to remove (required)
/// - `force`: Skip confirmation (always true in API mode)
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetRemoveParams {
///   root: string;
///   configPath?: string;
///   branch: string;
///   force?: boolean;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const params: ChangesetRemoveParams = {
///   root: '.',
///   branch: 'feature/abandoned',
///   force: true
/// };
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetRemoveParams {
    /// Workspace root directory path.
    pub root: String,

    /// Optional custom configuration file path.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,

    /// Branch name or changeset ID to remove.
    pub branch: String,

    /// Skip confirmation prompt.
    ///
    /// In API mode, this is always treated as `true` since there is no
    /// interactive prompt. Included for consistency with CLI interface.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

#[allow(dead_code)]
impl ChangesetRemoveParams {
    /// Creates a new `ChangesetRemoveParams` with required parameters.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    /// * `branch` - The branch name or changeset ID to remove
    #[must_use]
    pub fn new(root: impl Into<String>, branch: impl Into<String>) -> Self {
        Self { root: root.into(), config_path: None, branch: branch.into(), force: None }
    }

    /// Sets the force flag.
    #[must_use]
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = Some(force);
        self
    }
}

/// Input parameters for the changeset history command.
///
/// This structure defines the parameters for querying archived changesets.
/// All filter parameters are optional; when omitted, all archived changesets
/// are returned.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
/// - `filter_package`: Filter by package name
/// - `filter_env`: Filter by environment
/// - `filter_bump`: Filter by bump type
/// - `since`: Start date filter (ISO 8601 format)
/// - `until`: End date filter (ISO 8601 format)
/// - `limit`: Maximum number of results
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetHistoryParams {
///   root: string;
///   configPath?: string;
///   filterPackage?: string;
///   filterEnv?: string;
///   filterBump?: 'major' | 'minor' | 'patch';
///   since?: string;
///   until?: string;
///   limit?: number;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Get all history
/// const all: ChangesetHistoryParams = { root: '.' };
///
/// // Get recent major releases for a package
/// const filtered: ChangesetHistoryParams = {
///   root: '.',
///   filterPackage: '@scope/core',
///   filterBump: 'major',
///   since: '2024-01-01',
///   limit: 10
/// };
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetHistoryParams {
    /// Workspace root directory path.
    pub root: String,

    /// Optional custom configuration file path.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,

    /// Filter by package name.
    ///
    /// Only return archived changesets that affected the specified package.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_package: Option<String>,

    /// Filter by environment.
    ///
    /// Only return archived changesets from the specified environment.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_env: Option<String>,

    /// Filter by bump type.
    ///
    /// Only return archived changesets with the specified bump type.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_bump: Option<String>,

    /// Start date filter (ISO 8601 format).
    ///
    /// Only return changesets created on or after this date.
    /// Example: `"2024-01-01"` or `"2024-01-01T00:00:00Z"`
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,

    /// End date filter (ISO 8601 format).
    ///
    /// Only return changesets created on or before this date.
    /// Example: `"2024-12-31"` or `"2024-12-31T23:59:59Z"`
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,

    /// Maximum number of results to return.
    ///
    /// Useful for pagination or limiting large result sets.
    #[napi(ts_type = "number | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[allow(dead_code)]
impl ChangesetHistoryParams {
    /// Creates a new `ChangesetHistoryParams` with the required root path.
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            config_path: None,
            filter_package: None,
            filter_env: None,
            filter_bump: None,
            since: None,
            until: None,
            limit: None,
        }
    }

    /// Sets the package filter.
    #[must_use]
    pub fn with_filter_package(mut self, package: impl Into<String>) -> Self {
        self.filter_package = Some(package.into());
        self
    }

    /// Sets the environment filter.
    #[must_use]
    pub fn with_filter_env(mut self, env: impl Into<String>) -> Self {
        self.filter_env = Some(env.into());
        self
    }

    /// Sets the bump type filter.
    #[must_use]
    pub fn with_filter_bump(mut self, bump: impl Into<String>) -> Self {
        self.filter_bump = Some(bump.into());
        self
    }

    /// Sets the start date filter.
    #[must_use]
    pub fn with_since(mut self, since: impl Into<String>) -> Self {
        self.since = Some(since.into());
        self
    }

    /// Sets the end date filter.
    #[must_use]
    pub fn with_until(mut self, until: impl Into<String>) -> Self {
        self.until = Some(until.into());
        self
    }

    /// Sets the result limit.
    #[must_use]
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Input parameters for the changeset check command.
///
/// This structure defines the parameters for checking if a changeset exists
/// for a specific branch. Useful for Git hooks to enforce changeset creation.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
/// - `branch`: Branch name to check (defaults to current Git branch)
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetCheckParams {
///   root: string;
///   configPath?: string;
///   branch?: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Check current branch
/// const current: ChangesetCheckParams = { root: '.' };
///
/// // Check specific branch
/// const specific: ChangesetCheckParams = {
///   root: '.',
///   branch: 'feature/new-api'
/// };
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetCheckParams {
    /// Workspace root directory path.
    pub root: String,

    /// Optional custom configuration file path.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,

    /// Branch name to check.
    ///
    /// If not provided, the current Git branch is used.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

#[allow(dead_code)]
impl ChangesetCheckParams {
    /// Creates a new `ChangesetCheckParams` with the required root path.
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self { root: root.into(), config_path: None, branch: None }
    }

    /// Sets the branch to check.
    #[must_use]
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }
}

// ============================================================================
// Supporting Types for Response Data
// ============================================================================

/// Detailed changeset information.
///
/// This structure contains the complete details of a changeset, including
/// all packages, commits, environments, and timestamps. Used in list, show,
/// and history responses.
///
/// # Fields
///
/// - `id`: Unique changeset identifier
/// - `branch`: Git branch name
/// - `bump`: Version bump type
/// - `packages`: List of affected packages
/// - `environments`: Target environments
/// - `commits`: Associated commit hashes
/// - `message`: Optional description
/// - `created_at`: Creation timestamp (ISO 8601)
/// - `updated_at`: Last update timestamp (ISO 8601)
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetDetailInfo {
///   id: string;
///   branch: string;
///   bump: string;
///   packages: string[];
///   environments: string[];
///   commits: string[];
///   message?: string;
///   createdAt: string;
///   updatedAt: string;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetDetailInfo {
    /// Unique changeset identifier.
    ///
    /// This ID is derived from the branch name and uniquely identifies
    /// the changeset within the workspace.
    pub id: String,

    /// Git branch name.
    ///
    /// The full branch name associated with this changeset.
    pub branch: String,

    /// Version bump type.
    ///
    /// One of: `"major"`, `"minor"`, `"patch"`, `"none"`.
    pub bump: String,

    /// List of affected packages.
    ///
    /// Package names exactly as defined in each package's `package.json`.
    pub packages: Vec<String>,

    /// Target environments.
    ///
    /// List of environments this changeset applies to.
    pub environments: Vec<String>,

    /// Associated commit hashes.
    ///
    /// Git commit hashes that are part of this changeset.
    pub commits: Vec<String>,

    /// Optional description message.
    ///
    /// Human-readable description of the changes.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Creation timestamp (ISO 8601 format).
    ///
    /// When the changeset was first created.
    /// Example: `"2024-01-15T10:30:00Z"`
    pub created_at: String,

    /// Last update timestamp (ISO 8601 format).
    ///
    /// When the changeset was last modified.
    /// Example: `"2024-01-15T14:45:00Z"`
    pub updated_at: String,
}

#[allow(dead_code)]
impl ChangesetDetailInfo {
    /// Creates a new `ChangesetDetailInfo` with required fields.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique changeset identifier
    /// * `branch` - Git branch name
    /// * `bump` - Version bump type
    /// * `created_at` - Creation timestamp
    /// * `updated_at` - Last update timestamp
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        branch: impl Into<String>,
        bump: impl Into<String>,
        created_at: impl Into<String>,
        updated_at: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            branch: branch.into(),
            bump: bump.into(),
            packages: Vec::new(),
            environments: Vec::new(),
            commits: Vec::new(),
            message: None,
            created_at: created_at.into(),
            updated_at: updated_at.into(),
        }
    }

    /// Sets the packages list.
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.packages = packages;
        self
    }

    /// Sets the environments list.
    #[must_use]
    pub fn with_environments(mut self, environments: Vec<String>) -> Self {
        self.environments = environments;
        self
    }

    /// Sets the commits list.
    #[must_use]
    pub fn with_commits(mut self, commits: Vec<String>) -> Self {
        self.commits = commits;
        self
    }

    /// Sets the message.
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

/// Summary of a changeset update operation.
///
/// Contains counts of what was added or modified during a changeset update.
/// This structure mirrors the CLI's `UpdateSummary` response.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface UpdateSummaryInfo {
///   packagesAdded: number;
///   commitsAdded: number;
///   bumpUpdated: boolean;
///   environmentsAdded: number;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Successful update response
/// if (result.success) {
///   const summary = result.data.summary;
///   console.log(`Packages added: ${summary.packagesAdded}`);
///   console.log(`Commits added: ${summary.commitsAdded}`);
///   console.log(`Bump updated: ${summary.bumpUpdated}`);
///   console.log(`Environments added: ${summary.environmentsAdded}`);
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct UpdateSummaryInfo {
    /// Number of packages added to the changeset.
    ///
    /// Counts only newly added packages; packages that already existed
    /// in the changeset are not included in this count.
    pub packages_added: u32,

    /// Number of commits added to the changeset.
    ///
    /// Typically 0 or 1, as only one commit can be added per update call.
    pub commits_added: u32,

    /// Whether the bump type was changed.
    ///
    /// `true` if the bump type was modified (e.g., from `patch` to `minor`),
    /// `false` if the bump type remained the same or was not specified.
    pub bump_updated: bool,

    /// Number of environments added to the changeset.
    ///
    /// Counts only newly added environments; environments that already
    /// existed in the changeset are not included in this count.
    pub environments_added: u32,
}

#[allow(dead_code)]
impl UpdateSummaryInfo {
    /// Creates a new `UpdateSummaryInfo` with specified values.
    ///
    /// # Arguments
    ///
    /// * `packages_added` - Number of packages added
    /// * `commits_added` - Number of commits added
    /// * `bump_updated` - Whether the bump type was changed
    /// * `environments_added` - Number of environments added
    ///
    /// # Returns
    ///
    /// A new `UpdateSummaryInfo` instance.
    #[must_use]
    pub fn new(
        packages_added: u32,
        commits_added: u32,
        bump_updated: bool,
        environments_added: u32,
    ) -> Self {
        Self { packages_added, commits_added, bump_updated, environments_added }
    }

    /// Creates an empty update summary (no changes).
    ///
    /// Used when an update operation results in no actual changes
    /// (e.g., all specified values already existed in the changeset).
    ///
    /// # Returns
    ///
    /// An `UpdateSummaryInfo` with all counts at zero and `bump_updated` as `false`.
    #[must_use]
    pub fn empty() -> Self {
        Self { packages_added: 0, commits_added: 0, bump_updated: false, environments_added: 0 }
    }

    /// Returns `true` if any changes were made.
    ///
    /// # Returns
    ///
    /// `true` if at least one package, commit, or environment was added,
    /// or if the bump type was updated.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        self.packages_added > 0
            || self.commits_added > 0
            || self.bump_updated
            || self.environments_added > 0
    }
}

/// Entry in the released versions map.
///
/// Represents a package name and its released version. This structure is used
/// instead of a HashMap for NAPI compatibility.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ReleasedVersionEntry {
///   packageName: string;
///   version: string;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ReleasedVersionEntry {
    /// Package name.
    pub package_name: String,

    /// Released version.
    pub version: String,
}

#[allow(dead_code)]
impl ReleasedVersionEntry {
    /// Creates a new `ReleasedVersionEntry`.
    #[must_use]
    pub fn new(package_name: impl Into<String>, version: impl Into<String>) -> Self {
        Self { package_name: package_name.into(), version: version.into() }
    }
}

/// Release information for an archived changeset.
///
/// Contains details about when and how a changeset was released.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ReleaseInfoData {
///   releasedAt: string;
///   releasedBy: string;
///   releaseCommit: string;
///   releasedVersions: ReleasedVersionEntry[];
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ReleaseInfoData {
    /// Release timestamp (ISO 8601 format).
    ///
    /// When the changeset was released.
    pub released_at: String,

    /// User who performed the release.
    ///
    /// Git user or system identifier.
    pub released_by: String,

    /// Git commit hash of the release.
    ///
    /// The commit that applied the version bump.
    pub release_commit: String,

    /// Versions that were released for each package.
    ///
    /// List of package name to version mappings.
    pub released_versions: Vec<ReleasedVersionEntry>,
}

#[allow(dead_code)]
impl ReleaseInfoData {
    /// Creates a new `ReleaseInfoData`.
    #[must_use]
    pub fn new(
        released_at: impl Into<String>,
        released_by: impl Into<String>,
        release_commit: impl Into<String>,
        released_versions: Vec<ReleasedVersionEntry>,
    ) -> Self {
        Self {
            released_at: released_at.into(),
            released_by: released_by.into(),
            release_commit: release_commit.into(),
            released_versions,
        }
    }
}

/// Archived changeset information.
///
/// Combines the original changeset details with release information.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ArchivedChangesetInfo {
///   changeset: ChangesetDetailInfo;
///   releaseInfo: ReleaseInfoData;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ArchivedChangesetInfo {
    /// The original changeset details.
    pub changeset: ChangesetDetailInfo,

    /// Release information.
    pub release_info: ReleaseInfoData,
}

#[allow(dead_code)]
impl ArchivedChangesetInfo {
    /// Creates a new `ArchivedChangesetInfo`.
    #[must_use]
    pub fn new(changeset: ChangesetDetailInfo, release_info: ReleaseInfoData) -> Self {
        Self { changeset, release_info }
    }
}

// ============================================================================
// Response Data Types
// ============================================================================

/// Response data for the changeset add command.
///
/// Contains information about the newly created changeset.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetAddData {
///   id: string;
///   branch: string;
///   packages: string[];
///   bump: string;
///   environments: string[];
///   createdAt: string;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetAddData {
    /// Unique changeset identifier.
    pub id: String,

    /// Git branch name.
    pub branch: String,

    /// List of affected packages.
    pub packages: Vec<String>,

    /// Version bump type.
    pub bump: String,

    /// Target environments.
    pub environments: Vec<String>,

    /// Creation timestamp (ISO 8601 format).
    pub created_at: String,
}

#[allow(dead_code)]
impl ChangesetAddData {
    /// Creates a new `ChangesetAddData`.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        branch: impl Into<String>,
        bump: impl Into<String>,
        created_at: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            branch: branch.into(),
            packages: Vec::new(),
            bump: bump.into(),
            environments: Vec::new(),
            created_at: created_at.into(),
        }
    }

    /// Sets the packages list.
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.packages = packages;
        self
    }

    /// Sets the environments list.
    #[must_use]
    pub fn with_environments(mut self, environments: Vec<String>) -> Self {
        self.environments = environments;
        self
    }
}

/// Response data for the changeset update command.
///
/// Contains the result of the update operation, including a summary of what
/// was changed and the current state of the changeset after the update.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetUpdateData {
///   updated: boolean;
///   summary: UpdateSummaryInfo;
///   changeset: ChangesetDetailInfo;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await changesetUpdate({
///   root: '.',
///   id: 'feature/new-api',
///   packages: ['@scope/new-package'],
///   bump: 'minor'
/// });
///
/// if (result.success) {
///   console.log(`Updated: ${result.data.updated}`);
///   console.log(`Packages added: ${result.data.summary.packagesAdded}`);
///   console.log(`Current packages: ${result.data.changeset.packages.join(', ')}`);
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetUpdateData {
    /// Whether the update was performed.
    ///
    /// `true` if at least one change was applied to the changeset,
    /// `false` if all specified values already existed.
    pub updated: bool,

    /// Summary of what was updated.
    ///
    /// Contains counts of packages, commits, and environments added,
    /// as well as whether the bump type was changed.
    pub summary: UpdateSummaryInfo,

    /// The updated changeset details.
    ///
    /// Contains the complete state of the changeset after the update,
    /// including all packages, commits, environments, and timestamps.
    pub changeset: ChangesetDetailInfo,
}

#[allow(dead_code)]
impl ChangesetUpdateData {
    /// Creates a new `ChangesetUpdateData` with all fields.
    ///
    /// # Arguments
    ///
    /// * `updated` - Whether any changes were applied
    /// * `summary` - Summary of what was updated
    /// * `changeset` - The updated changeset details
    ///
    /// # Returns
    ///
    /// A new `ChangesetUpdateData` instance.
    #[must_use]
    pub fn new(updated: bool, summary: UpdateSummaryInfo, changeset: ChangesetDetailInfo) -> Self {
        Self { updated, summary, changeset }
    }

    /// Creates a successful update response.
    ///
    /// # Arguments
    ///
    /// * `summary` - Summary of what was updated
    /// * `changeset` - The updated changeset details
    ///
    /// # Returns
    ///
    /// A `ChangesetUpdateData` with `updated` set to `true`.
    #[must_use]
    pub fn success(summary: UpdateSummaryInfo, changeset: ChangesetDetailInfo) -> Self {
        Self { updated: true, summary, changeset }
    }
}

/// Response data for the changeset list command.
///
/// Contains the list of pending changesets.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetListData {
///   changesets: ChangesetDetailInfo[];
///   count: number;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetListData {
    /// List of pending changesets.
    pub changesets: Vec<ChangesetDetailInfo>,

    /// Total count of changesets.
    pub count: u32,
}

#[allow(dead_code)]
impl ChangesetListData {
    /// Creates a new `ChangesetListData`.
    #[must_use]
    pub fn new(changesets: Vec<ChangesetDetailInfo>) -> Self {
        // Saturating conversion is safe here - we won't have more than u32::MAX changesets
        #[allow(clippy::cast_possible_truncation)]
        let count = changesets.len() as u32;
        Self { changesets, count }
    }

    /// Creates an empty list response.
    #[must_use]
    pub fn empty() -> Self {
        Self { changesets: Vec::new(), count: 0 }
    }
}

/// Response data for the changeset show command.
///
/// Contains the details of a specific changeset.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetShowData {
///   changeset: ChangesetDetailInfo;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetShowData {
    /// The changeset details.
    pub changeset: ChangesetDetailInfo,
}

#[allow(dead_code)]
impl ChangesetShowData {
    /// Creates a new `ChangesetShowData`.
    #[must_use]
    pub fn new(changeset: ChangesetDetailInfo) -> Self {
        Self { changeset }
    }
}

/// Response data for the changeset remove command.
///
/// Contains the result of the remove operation.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetRemoveData {
///   removed: boolean;
///   branch: string;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetRemoveData {
    /// Whether the changeset was removed.
    pub removed: bool,

    /// Branch name of the removed changeset.
    pub branch: String,
}

#[allow(dead_code)]
impl ChangesetRemoveData {
    /// Creates a new `ChangesetRemoveData`.
    #[must_use]
    pub fn new(removed: bool, branch: impl Into<String>) -> Self {
        Self { removed, branch: branch.into() }
    }

    /// Creates a successful removal response.
    #[must_use]
    pub fn success(branch: impl Into<String>) -> Self {
        Self { removed: true, branch: branch.into() }
    }
}

/// Response data for the changeset history command.
///
/// Contains archived changesets matching the query.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetHistoryData {
///   archived: ArchivedChangesetInfo[];
///   count: number;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetHistoryData {
    /// List of archived changesets.
    pub archived: Vec<ArchivedChangesetInfo>,

    /// Total count of results.
    pub count: u32,
}

#[allow(dead_code)]
impl ChangesetHistoryData {
    /// Creates a new `ChangesetHistoryData`.
    #[must_use]
    pub fn new(archived: Vec<ArchivedChangesetInfo>) -> Self {
        // Saturating conversion is safe here - we won't have more than u32::MAX archived changesets
        #[allow(clippy::cast_possible_truncation)]
        let count = archived.len() as u32;
        Self { archived, count }
    }

    /// Creates an empty history response.
    #[must_use]
    pub fn empty() -> Self {
        Self { archived: Vec::new(), count: 0 }
    }
}

/// Response data for the changeset check command.
///
/// Contains the result of the changeset existence check.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetCheckData {
///   hasChangeset: boolean;
///   branch?: string;
///   packages?: string[];
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetCheckData {
    /// Whether a changeset exists for the branch.
    pub has_changeset: bool,

    /// Branch name that was checked.
    ///
    /// Present when a changeset exists.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Packages in the existing changeset.
    ///
    /// Present when a changeset exists.
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<String>>,
}

#[allow(dead_code)]
impl ChangesetCheckData {
    /// Creates a response indicating a changeset exists.
    #[must_use]
    pub fn exists(branch: impl Into<String>, packages: Vec<String>) -> Self {
        Self { has_changeset: true, branch: Some(branch.into()), packages: Some(packages) }
    }

    /// Creates a response indicating no changeset exists.
    #[must_use]
    pub fn not_found() -> Self {
        Self { has_changeset: false, branch: None, packages: None }
    }
}

// ============================================================================
// API Response Wrappers
// ============================================================================

/// API response for the changeset add command.
///
/// Wraps `ChangesetAddData` with success/error handling.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetAddApiResponse {
///   success: boolean;
///   data?: ChangesetAddData;
///   error?: ErrorInfo;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetAddApiResponse {
    /// Whether the operation succeeded.
    pub success: bool,

    /// The add data (only present when `success` is `true`).
    #[napi(ts_type = "ChangesetAddData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ChangesetAddData>,

    /// Error information (only present when `success` is `false`).
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl ChangesetAddApiResponse {
    /// Creates a successful response with data.
    #[must_use]
    pub fn success(data: ChangesetAddData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failed response with error information.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns whether this response represents a success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns whether this response represents a failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// API response for the changeset update command.
///
/// Wraps `ChangesetUpdateData` with success/error handling.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetUpdateApiResponse {
///   success: boolean;
///   data?: ChangesetUpdateData;
///   error?: ErrorInfo;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetUpdateApiResponse {
    /// Whether the operation succeeded.
    pub success: bool,

    /// The update data (only present when `success` is `true`).
    #[napi(ts_type = "ChangesetUpdateData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ChangesetUpdateData>,

    /// Error information (only present when `success` is `false`).
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl ChangesetUpdateApiResponse {
    /// Creates a successful response with data.
    #[must_use]
    pub fn success(data: ChangesetUpdateData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failed response with error information.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns whether this response represents a success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns whether this response represents a failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// API response for the changeset list command.
///
/// Wraps `ChangesetListData` with success/error handling.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetListApiResponse {
///   success: boolean;
///   data?: ChangesetListData;
///   error?: ErrorInfo;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetListApiResponse {
    /// Whether the operation succeeded.
    pub success: bool,

    /// The list data (only present when `success` is `true`).
    #[napi(ts_type = "ChangesetListData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ChangesetListData>,

    /// Error information (only present when `success` is `false`).
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl ChangesetListApiResponse {
    /// Creates a successful response with data.
    #[must_use]
    pub fn success(data: ChangesetListData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failed response with error information.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns whether this response represents a success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns whether this response represents a failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// API response for the changeset show command.
///
/// Wraps `ChangesetShowData` with success/error handling.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetShowApiResponse {
///   success: boolean;
///   data?: ChangesetShowData;
///   error?: ErrorInfo;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetShowApiResponse {
    /// Whether the operation succeeded.
    pub success: bool,

    /// The show data (only present when `success` is `true`).
    #[napi(ts_type = "ChangesetShowData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ChangesetShowData>,

    /// Error information (only present when `success` is `false`).
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl ChangesetShowApiResponse {
    /// Creates a successful response with data.
    #[must_use]
    pub fn success(data: ChangesetShowData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failed response with error information.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns whether this response represents a success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns whether this response represents a failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// API response for the changeset remove command.
///
/// Wraps `ChangesetRemoveData` with success/error handling.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetRemoveApiResponse {
///   success: boolean;
///   data?: ChangesetRemoveData;
///   error?: ErrorInfo;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetRemoveApiResponse {
    /// Whether the operation succeeded.
    pub success: bool,

    /// The remove data (only present when `success` is `true`).
    #[napi(ts_type = "ChangesetRemoveData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ChangesetRemoveData>,

    /// Error information (only present when `success` is `false`).
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl ChangesetRemoveApiResponse {
    /// Creates a successful response with data.
    #[must_use]
    pub fn success(data: ChangesetRemoveData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failed response with error information.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns whether this response represents a success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns whether this response represents a failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// API response for the changeset history command.
///
/// Wraps `ChangesetHistoryData` with success/error handling.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetHistoryApiResponse {
///   success: boolean;
///   data?: ChangesetHistoryData;
///   error?: ErrorInfo;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetHistoryApiResponse {
    /// Whether the operation succeeded.
    pub success: bool,

    /// The history data (only present when `success` is `true`).
    #[napi(ts_type = "ChangesetHistoryData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ChangesetHistoryData>,

    /// Error information (only present when `success` is `false`).
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl ChangesetHistoryApiResponse {
    /// Creates a successful response with data.
    #[must_use]
    pub fn success(data: ChangesetHistoryData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failed response with error information.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns whether this response represents a success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns whether this response represents a failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// API response for the changeset check command.
///
/// Wraps `ChangesetCheckData` with success/error handling.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetCheckApiResponse {
///   success: boolean;
///   data?: ChangesetCheckData;
///   error?: ErrorInfo;
/// }
/// ```
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetCheckApiResponse {
    /// Whether the operation succeeded.
    pub success: bool,

    /// The check data (only present when `success` is `true`).
    #[napi(ts_type = "ChangesetCheckData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ChangesetCheckData>,

    /// Error information (only present when `success` is `false`).
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl ChangesetCheckApiResponse {
    /// Creates a successful response with data.
    #[must_use]
    pub fn success(data: ChangesetCheckData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failed response with error information.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns whether this response represents a success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns whether this response represents a failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}
