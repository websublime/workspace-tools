//! Bump command type definitions for Node.js bindings.
//!
//! # What
//!
//! This module defines all NAPI-compatible type structures for bump commands,
//! including input parameters and response data types. Bump commands are the
//! culmination of the changeset workflow, translating pending changesets into
//! actual version updates for packages.
//!
//! # How
//!
//! Types are defined with the `#[napi(object)]` attribute to be automatically
//! exposed as JavaScript objects. The module provides:
//!
//! - **Input Parameters**: `BumpPreviewParams`, `BumpApplyParams`, `BumpSnapshotParams`
//! - **Response Data**: `BumpPreviewData`, `BumpApplyData`, `BumpSnapshotData`
//! - **Supporting Types**: `PackageVersionInfo`, `SnapshotVersionInfo`,
//!   `DependencyUpdateInfo`, `BumpSummaryInfo`
//! - **API Responses**: Type-safe response wrappers for each command
//!
//! All types implement `Clone`, `Debug`, and `Serialize` for flexibility in
//! testing and serialization scenarios.
//!
//! # Why
//!
//! Bump commands provide three distinct workflows:
//!
//! - **Preview**: Dry-run to see what versions would change (no modifications)
//! - **Apply**: Execute version bumps with optional Git integration
//! - **Snapshot**: Generate temporary pre-release versions for testing
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
//!   bumpPreview,
//!   bumpApply,
//!   bumpSnapshot,
//!   BumpPreviewParams,
//!   BumpApplyParams,
//!   BumpSnapshotParams
//! } from '@websublime/workspace-tools';
//!
//! // Preview version bumps (dry-run)
//! const previewParams: BumpPreviewParams = {
//!   root: '.',
//!   showDiff: true
//! };
//! const previewResult = await bumpPreview(previewParams);
//! if (previewResult.success) {
//!   for (const pkg of previewResult.data.packages) {
//!     console.log(`${pkg.name}: ${pkg.currentVersion} -> ${pkg.nextVersion}`);
//!   }
//! }
//!
//! // Apply version bumps with Git integration
//! const applyParams: BumpApplyParams = {
//!   root: '.',
//!   gitCommit: true,
//!   gitTag: true,
//!   gitPush: false
//! };
//! const applyResult = await bumpApply(applyParams);
//! if (applyResult.success) {
//!   console.log(`Updated ${applyResult.data.packagesUpdated} packages`);
//!   console.log(`Tags created: ${applyResult.data.tagsCreated.join(', ')}`);
//! }
//!
//! // Generate snapshot versions
//! const snapshotParams: BumpSnapshotParams = {
//!   root: '.',
//!   format: '{version}-snapshot.{short_commit}'
//! };
//! const snapshotResult = await bumpSnapshot(snapshotParams);
//! if (snapshotResult.success) {
//!   for (const pkg of snapshotResult.data.packages) {
//!     console.log(`${pkg.name}: ${pkg.snapshotVersion}`);
//!   }
//! }
//! ```
//!
//! ## Rust Usage (Internal)
//!
//! ```rust,ignore
//! use sublime_node_tools::types::bump::{
//!     BumpPreviewParams, BumpPreviewData, PackageVersionInfo
//! };
//!
//! // Creating params for validation
//! let params = BumpPreviewParams::new(".")
//!     .with_show_diff(true)
//!     .with_packages(vec!["@scope/pkg1".to_string()]);
//!
//! // Constructing response data
//! let version_info = PackageVersionInfo::new(
//!     "@scope/pkg1",
//!     "packages/pkg1",
//!     "1.0.0",
//!     "1.1.0",
//!     "minor"
//! );
//! ```

use napi_derive::napi;
use serde::Serialize;

use crate::error::ErrorInfo;

// ============================================================================
// Constants
// ============================================================================

/// Common prerelease tags used in semantic versioning.
///
/// These are the standard prerelease identifiers, but any valid tag
/// containing only ASCII alphanumerics and hyphens `[0-9A-Za-z-]` is accepted.
///
/// - `"alpha"`: Early development, unstable
/// - `"beta"`: Feature complete but may have bugs
/// - `"rc"`: Release candidate, near final release
#[allow(dead_code)]
pub(crate) const COMMON_PRERELEASE_TAGS: &[&str] = &["alpha", "beta", "rc"];

/// Valid dependency types for dependency updates.
///
/// - `"regular"`: Standard dependencies (dependencies)
/// - `"dev"`: Development dependencies (devDependencies)
/// - `"peer"`: Peer dependencies (peerDependencies)
/// - `"optional"`: Optional dependencies (optionalDependencies)
#[allow(dead_code)]
pub(crate) const VALID_DEPENDENCY_TYPES: &[&str] = &["regular", "dev", "peer", "optional"];

/// Default snapshot format template.
///
/// Variables available:
/// - `{version}`: Current package version
/// - `{branch}`: Current Git branch name (sanitized)
/// - `{short_commit}`: Short Git commit hash (7 characters)
/// - `{commit}`: Full Git commit hash
/// - `{timestamp}`: Unix timestamp
#[allow(dead_code)]
pub(crate) const DEFAULT_SNAPSHOT_FORMAT: &str = "{version}-snapshot.{short_commit}";

// ============================================================================
// Input Parameters
// ============================================================================

/// Input parameters for the bump preview command.
///
/// This structure defines the parameters for previewing version bumps based on
/// pending changesets. The preview is a dry-run operation that shows what would
/// change without actually modifying any files.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
/// - `packages`: Optional filter to specific packages
/// - `show_diff`: Whether to show detailed version diffs
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BumpPreviewParams {
///   root: string;
///   configPath?: string;
///   packages?: string[];
///   showDiff?: boolean;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Minimal params - preview all packages
/// const minimal: BumpPreviewParams = { root: '.' };
///
/// // Preview specific packages with diff
/// const filtered: BumpPreviewParams = {
///   root: '/path/to/workspace',
///   packages: ['@scope/pkg1', '@scope/pkg2'],
///   showDiff: true
/// };
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BumpPreviewParams {
    /// Workspace root directory path.
    ///
    /// This is the absolute or relative path to the root of the workspace.
    /// For monorepos, this should point to the root where the package manager
    /// configuration is located.
    pub root: String,

    /// Optional custom configuration file path.
    ///
    /// If not provided, the command will search for configuration files
    /// in standard locations (`repo.config.json`, `repo.config.yaml`, etc.)
    /// within the workspace root.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,

    /// Filter to specific packages.
    ///
    /// When provided, only these packages will be included in the preview.
    /// Package names should include scope if applicable (e.g., `@scope/pkg`).
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<String>>,

    /// Whether to show detailed version diffs.
    ///
    /// When `true`, includes detailed information about what changes would
    /// be made to each package, including dependency updates.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_diff: Option<bool>,
}

#[allow(dead_code)]
impl BumpPreviewParams {
    /// Creates a new `BumpPreviewParams` with the required root path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// A new `BumpPreviewParams` instance with default optional values.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = BumpPreviewParams::new("/path/to/workspace");
    /// ```
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self { root: root.into(), config_path: None, packages: None, show_diff: None }
    }

    /// Sets the config path.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Self with the config path set.
    #[must_use]
    pub fn with_config_path(mut self, config_path: impl Into<String>) -> Self {
        self.config_path = Some(config_path.into());
        self
    }

    /// Sets the packages filter.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package names to filter
    ///
    /// # Returns
    ///
    /// Self with the packages filter set.
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.packages = Some(packages);
        self
    }

    /// Sets the show diff flag.
    ///
    /// # Arguments
    ///
    /// * `show_diff` - Whether to show detailed diffs
    ///
    /// # Returns
    ///
    /// Self with the show diff flag set.
    #[must_use]
    pub fn with_show_diff(mut self, show_diff: bool) -> Self {
        self.show_diff = Some(show_diff);
        self
    }
}

/// Input parameters for the bump apply command.
///
/// This structure defines the parameters for applying version bumps to packages.
/// Unlike preview, this command actually modifies files and can optionally
/// integrate with Git for committing and tagging releases.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
/// - `packages`: Optional filter to specific packages
/// - `git_commit`: Whether to create a Git commit with version changes
/// - `git_tag`: Whether to create Git tags for releases
/// - `git_push`: Whether to push Git tags to remote
/// - `prerelease`: Prerelease tag for pre-release versions (alpha, beta, rc, or custom)
/// - `no_changelog`: Whether to skip changelog generation
/// - `no_archive`: Whether to keep changesets active after bump
/// - `always_archive`: Whether to force archiving even for prerelease versions
/// - `force`: Whether to skip confirmation prompts
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BumpApplyParams {
///   root: string;
///   configPath?: string;
///   packages?: string[];
///   gitCommit?: boolean;
///   gitTag?: boolean;
///   gitPush?: boolean;
///   prerelease?: string;
///   noChangelog?: boolean;
///   noArchive?: boolean;
///   alwaysArchive?: boolean;
///   force?: boolean;
/// }
/// ```
///
/// # Prerelease Support
///
/// The `prerelease` parameter creates semver-compliant pre-release versions:
/// - `"alpha"` → `1.2.3 → 1.3.0-alpha.0`
/// - `"beta"` → `1.2.3 → 1.3.0-beta.0`
/// - `"rc"` → `1.2.3 → 1.3.0-rc.0`
/// - Any custom tag → `1.2.3 → 1.3.0-{tag}.0`
///
/// # Examples
///
/// ```typescript
/// // Minimal apply - just bump versions
/// const minimal: BumpApplyParams = { root: '.' };
///
/// // Full release with Git integration
/// const release: BumpApplyParams = {
///   root: '.',
///   gitCommit: true,
///   gitTag: true,
///   gitPush: true,
///   force: true
/// };
///
/// // Beta prerelease
/// const beta: BumpApplyParams = {
///   root: '.',
///   prerelease: 'beta',
///   gitCommit: true,
///   gitTag: true
/// };
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BumpApplyParams {
    /// Workspace root directory path.
    ///
    /// This is the absolute or relative path to the root of the workspace.
    pub root: String,

    /// Optional custom configuration file path.
    ///
    /// If not provided, the command will search for configuration files
    /// in standard locations within the workspace root.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,

    /// Filter to specific packages.
    ///
    /// When provided, only these packages will be bumped.
    /// Package names should include scope if applicable.
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<String>>,

    /// Whether to create a Git commit with version changes.
    ///
    /// When `true`, creates a commit containing all modified files
    /// (package.json, CHANGELOG.md, etc.) with a conventional commit message.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<bool>,

    /// Whether to create Git tags for releases.
    ///
    /// When `true`, creates tags in the format `{package}@{version}` for
    /// each bumped package. Requires `git_commit` to be effective.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_tag: Option<bool>,

    /// Whether to push Git tags to remote.
    ///
    /// When `true`, pushes the created tags to the remote repository.
    /// Requires `git_tag` to be `true` to have any effect.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_push: Option<bool>,

    /// Prerelease tag for pre-release versions.
    ///
    /// Creates semver-compliant pre-release versions. Common values:
    /// - `"alpha"`: Early development (`1.3.0-alpha.0`)
    /// - `"beta"`: Feature complete (`1.3.0-beta.0`)
    /// - `"rc"`: Release candidate (`1.3.0-rc.0`)
    ///
    /// Custom tags are also supported. Must contain only ASCII
    /// alphanumerics and hyphens `[0-9A-Za-z-]`.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerelease: Option<String>,

    /// Whether to skip changelog generation.
    ///
    /// When `true`, CHANGELOG.md files will not be updated during the bump.
    /// Useful for quick internal releases or when changelogs are managed
    /// separately.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_changelog: Option<bool>,

    /// Whether to keep changesets active after bump.
    ///
    /// When `true`, changesets are not archived after version bump.
    /// Useful for partial releases or when you want to accumulate
    /// more changes before archiving.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_archive: Option<bool>,

    /// Whether to force archiving even for prerelease versions.
    ///
    /// By default, prerelease versions don't archive changesets (since
    /// the final release will archive them). Set this to `true` to
    /// archive changesets even for prerelease versions.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_archive: Option<bool>,

    /// Whether to skip confirmation prompts.
    ///
    /// When `true`, applies changes without asking for confirmation.
    /// Recommended for CI/CD environments. The NAPI API defaults to
    /// `true` since it's typically used programmatically.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

#[allow(dead_code)]
impl BumpApplyParams {
    /// Creates a new `BumpApplyParams` with the required root path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// A new `BumpApplyParams` instance with default optional values.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = BumpApplyParams::new("/path/to/workspace");
    /// ```
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            config_path: None,
            packages: None,
            git_commit: None,
            git_tag: None,
            git_push: None,
            prerelease: None,
            no_changelog: None,
            no_archive: None,
            always_archive: None,
            force: None,
        }
    }

    /// Sets the config path.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Self with the config path set.
    #[must_use]
    pub fn with_config_path(mut self, config_path: impl Into<String>) -> Self {
        self.config_path = Some(config_path.into());
        self
    }

    /// Sets the packages filter.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package names to filter
    ///
    /// # Returns
    ///
    /// Self with the packages filter set.
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.packages = Some(packages);
        self
    }

    /// Sets the git commit flag.
    ///
    /// # Arguments
    ///
    /// * `git_commit` - Whether to create a Git commit
    ///
    /// # Returns
    ///
    /// Self with the git commit flag set.
    #[must_use]
    pub fn with_git_commit(mut self, git_commit: bool) -> Self {
        self.git_commit = Some(git_commit);
        self
    }

    /// Sets the git tag flag.
    ///
    /// # Arguments
    ///
    /// * `git_tag` - Whether to create Git tags
    ///
    /// # Returns
    ///
    /// Self with the git tag flag set.
    #[must_use]
    pub fn with_git_tag(mut self, git_tag: bool) -> Self {
        self.git_tag = Some(git_tag);
        self
    }

    /// Sets the git push flag.
    ///
    /// # Arguments
    ///
    /// * `git_push` - Whether to push Git tags
    ///
    /// # Returns
    ///
    /// Self with the git push flag set.
    #[must_use]
    pub fn with_git_push(mut self, git_push: bool) -> Self {
        self.git_push = Some(git_push);
        self
    }

    /// Sets the prerelease tag.
    ///
    /// # Arguments
    ///
    /// * `prerelease` - The prerelease tag (alpha, beta, rc, or custom)
    ///
    /// # Returns
    ///
    /// Self with the prerelease tag set.
    #[must_use]
    pub fn with_prerelease(mut self, prerelease: impl Into<String>) -> Self {
        self.prerelease = Some(prerelease.into());
        self
    }

    /// Sets the no changelog flag.
    ///
    /// # Arguments
    ///
    /// * `no_changelog` - Whether to skip changelog generation
    ///
    /// # Returns
    ///
    /// Self with the no changelog flag set.
    #[must_use]
    pub fn with_no_changelog(mut self, no_changelog: bool) -> Self {
        self.no_changelog = Some(no_changelog);
        self
    }

    /// Sets the no archive flag.
    ///
    /// # Arguments
    ///
    /// * `no_archive` - Whether to keep changesets active
    ///
    /// # Returns
    ///
    /// Self with the no archive flag set.
    #[must_use]
    pub fn with_no_archive(mut self, no_archive: bool) -> Self {
        self.no_archive = Some(no_archive);
        self
    }

    /// Sets the always archive flag.
    ///
    /// # Arguments
    ///
    /// * `always_archive` - Whether to force archiving for prereleases
    ///
    /// # Returns
    ///
    /// Self with the always archive flag set.
    #[must_use]
    pub fn with_always_archive(mut self, always_archive: bool) -> Self {
        self.always_archive = Some(always_archive);
        self
    }

    /// Sets the force flag.
    ///
    /// # Arguments
    ///
    /// * `force` - Whether to skip confirmation prompts
    ///
    /// # Returns
    ///
    /// Self with the force flag set.
    #[must_use]
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = Some(force);
        self
    }

    /// Convenience method to set all Git options at once.
    ///
    /// # Arguments
    ///
    /// * `commit` - Whether to create a Git commit
    /// * `tag` - Whether to create Git tags
    /// * `push` - Whether to push tags to remote
    ///
    /// # Returns
    ///
    /// Self with all Git options set.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = BumpApplyParams::new(".")
    ///     .with_git_options(true, true, false);
    /// ```
    #[must_use]
    pub fn with_git_options(mut self, commit: bool, tag: bool, push: bool) -> Self {
        self.git_commit = Some(commit);
        self.git_tag = Some(tag);
        self.git_push = Some(push);
        self
    }
}

/// Input parameters for the bump snapshot command.
///
/// This structure defines the parameters for generating snapshot versions.
/// Snapshots are temporary, non-persisted versions used for testing and
/// CI/CD preview deployments. Unlike regular bumps, snapshots don't archive
/// changesets or create changelogs.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
/// - `packages`: Optional filter to specific packages
/// - `format`: Snapshot version format template
///
/// # Format Template Variables
///
/// The `format` parameter supports these variables:
/// - `{version}`: Current package version (e.g., `1.2.3`)
/// - `{branch}`: Current Git branch name (sanitized, e.g., `feature-x`)
/// - `{short_commit}`: Short Git commit hash (7 chars, e.g., `abc123f`)
/// - `{commit}`: Full Git commit hash
/// - `{timestamp}`: Unix timestamp
///
/// Default format: `{version}-snapshot.{short_commit}`
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BumpSnapshotParams {
///   root: string;
///   configPath?: string;
///   packages?: string[];
///   format?: string;
/// }
/// ```
///
/// # Snapshot vs Prerelease
///
/// | Aspect | Snapshot | Prerelease |
/// |--------|----------|------------|
/// | SemVer Compliant | No | Yes |
/// | Persisted | No | Yes |
/// | Changesets Archived | No | Optional |
/// | Use Case | Testing/CI | Staging/Beta |
/// | Example | `1.2.3-snapshot.abc123f` | `1.3.0-beta.0` |
///
/// # Examples
///
/// ```typescript
/// // Default format
/// const basic: BumpSnapshotParams = { root: '.' };
///
/// // Custom format with branch
/// const withBranch: BumpSnapshotParams = {
///   root: '.',
///   format: '{version}-{branch}.{short_commit}'
/// };
///
/// // Timestamp-based
/// const timestamped: BumpSnapshotParams = {
///   root: '.',
///   format: '{version}-dev.{timestamp}'
/// };
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BumpSnapshotParams {
    /// Workspace root directory path.
    ///
    /// This is the absolute or relative path to the root of the workspace.
    pub root: String,

    /// Optional custom configuration file path.
    ///
    /// If not provided, the command will search for configuration files
    /// in standard locations within the workspace root.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,

    /// Filter to specific packages.
    ///
    /// When provided, only these packages will get snapshot versions.
    /// Package names should include scope if applicable.
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<String>>,

    /// Snapshot version format template.
    ///
    /// Supports the following variables:
    /// - `{version}`: Current package version
    /// - `{branch}`: Current Git branch (sanitized)
    /// - `{short_commit}`: Short Git commit hash (7 chars)
    /// - `{commit}`: Full Git commit hash
    /// - `{timestamp}`: Unix timestamp
    ///
    /// Default: `{version}-snapshot.{short_commit}`
    ///
    /// Example: `{version}-{branch}.{short_commit}` →
    /// `1.2.3-feature-x.abc123f`
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

#[allow(dead_code)]
impl BumpSnapshotParams {
    /// Creates a new `BumpSnapshotParams` with the required root path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// A new `BumpSnapshotParams` instance with default optional values.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = BumpSnapshotParams::new("/path/to/workspace");
    /// ```
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self { root: root.into(), config_path: None, packages: None, format: None }
    }

    /// Sets the config path.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Self with the config path set.
    #[must_use]
    pub fn with_config_path(mut self, config_path: impl Into<String>) -> Self {
        self.config_path = Some(config_path.into());
        self
    }

    /// Sets the packages filter.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package names to filter
    ///
    /// # Returns
    ///
    /// Self with the packages filter set.
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.packages = Some(packages);
        self
    }

    /// Sets the snapshot format.
    ///
    /// # Arguments
    ///
    /// * `format` - The format template for snapshot versions
    ///
    /// # Returns
    ///
    /// Self with the format set.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = BumpSnapshotParams::new(".")
    ///     .with_format("{version}-{branch}.{short_commit}");
    /// ```
    #[must_use]
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }
}

// ============================================================================
// Response Data Types - Supporting Types
// ============================================================================

/// Dependency update information for a package version bump.
///
/// This structure captures information about how a dependency version
/// was updated as part of the version bump process. Dependencies are
/// updated when the package they depend on is bumped.
///
/// # Fields
///
/// - `name`: The dependency package name
/// - `dependency_type`: The type of dependency (regular, dev, peer, optional)
/// - `old_version`: The previous version specification
/// - `new_version`: The new version specification
///
/// # TypeScript Definition
///
/// ```typescript
/// interface DependencyUpdateInfo {
///   name: string;
///   dependencyType: 'regular' | 'dev' | 'peer' | 'optional';
///   oldVersion: string;
///   newVersion: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const update: DependencyUpdateInfo = {
///   name: '@scope/core',
///   dependencyType: 'regular',
///   oldVersion: '^1.0.0',
///   newVersion: '^1.1.0'
/// };
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct DependencyUpdateInfo {
    /// The dependency package name.
    ///
    /// This is the name of the package that was updated as a dependency.
    /// May include scope (e.g., `@scope/package`).
    pub name: String,

    /// The type of dependency.
    ///
    /// One of: `regular`, `dev`, `peer`, `optional`
    pub dependency_type: String,

    /// The previous version specification.
    ///
    /// This is the version range or exact version that was previously
    /// specified in package.json (e.g., `^1.0.0`, `~1.0.0`, `1.0.0`).
    pub old_version: String,

    /// The new version specification.
    ///
    /// This is the updated version range or exact version after the bump.
    pub new_version: String,
}

#[allow(dead_code)]
impl DependencyUpdateInfo {
    /// Creates a new `DependencyUpdateInfo`.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency package name
    /// * `dependency_type` - The type of dependency
    /// * `old_version` - The previous version specification
    /// * `new_version` - The new version specification
    ///
    /// # Returns
    ///
    /// A new `DependencyUpdateInfo` instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let update = DependencyUpdateInfo::new(
    ///     "@scope/core",
    ///     "regular",
    ///     "^1.0.0",
    ///     "^1.1.0"
    /// );
    /// ```
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        dependency_type: impl Into<String>,
        old_version: impl Into<String>,
        new_version: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            dependency_type: dependency_type.into(),
            old_version: old_version.into(),
            new_version: new_version.into(),
        }
    }

    /// Creates a regular (runtime) dependency update.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency package name
    /// * `old_version` - The previous version specification
    /// * `new_version` - The new version specification
    ///
    /// # Returns
    ///
    /// A new `DependencyUpdateInfo` with `dependency_type` set to `"regular"`.
    #[must_use]
    pub fn regular(
        name: impl Into<String>,
        old_version: impl Into<String>,
        new_version: impl Into<String>,
    ) -> Self {
        Self::new(name, "regular", old_version, new_version)
    }

    /// Creates a dev dependency update.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency package name
    /// * `old_version` - The previous version specification
    /// * `new_version` - The new version specification
    ///
    /// # Returns
    ///
    /// A new `DependencyUpdateInfo` with `dependency_type` set to `"dev"`.
    #[must_use]
    pub fn dev(
        name: impl Into<String>,
        old_version: impl Into<String>,
        new_version: impl Into<String>,
    ) -> Self {
        Self::new(name, "dev", old_version, new_version)
    }

    /// Creates a peer dependency update.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency package name
    /// * `old_version` - The previous version specification
    /// * `new_version` - The new version specification
    ///
    /// # Returns
    ///
    /// A new `DependencyUpdateInfo` with `dependency_type` set to `"peer"`.
    #[must_use]
    pub fn peer(
        name: impl Into<String>,
        old_version: impl Into<String>,
        new_version: impl Into<String>,
    ) -> Self {
        Self::new(name, "peer", old_version, new_version)
    }

    /// Creates an optional dependency update.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency package name
    /// * `old_version` - The previous version specification
    /// * `new_version` - The new version specification
    ///
    /// # Returns
    ///
    /// A new `DependencyUpdateInfo` with `dependency_type` set to `"optional"`.
    #[must_use]
    pub fn optional(
        name: impl Into<String>,
        old_version: impl Into<String>,
        new_version: impl Into<String>,
    ) -> Self {
        Self::new(name, "optional", old_version, new_version)
    }
}

/// Version information for a package being bumped.
///
/// This structure captures the full version transition for a package,
/// including the bump type and any dependency updates that resulted
/// from this package being bumped.
///
/// # Fields
///
/// - `name`: Package name (may include scope)
/// - `path`: Package path relative to workspace root
/// - `current_version`: Current version before bump
/// - `next_version`: Next version after bump
/// - `bump`: Bump type applied (major, minor, patch, none)
/// - `dependency_updates`: List of dependency updates for this package
///
/// # TypeScript Definition
///
/// ```typescript
/// interface PackageVersionInfo {
///   name: string;
///   path: string;
///   currentVersion: string;
///   nextVersion: string;
///   bump: 'major' | 'minor' | 'patch' | 'none';
///   dependencyUpdates: DependencyUpdateInfo[];
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const pkg: PackageVersionInfo = {
///   name: '@scope/core',
///   path: 'packages/core',
///   currentVersion: '1.0.0',
///   nextVersion: '1.1.0',
///   bump: 'minor',
///   dependencyUpdates: []
/// };
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct PackageVersionInfo {
    /// Package name.
    ///
    /// The full package name, including scope if applicable
    /// (e.g., `@scope/package` or `package`).
    pub name: String,

    /// Package path relative to workspace root.
    ///
    /// The file system path to the package directory, relative to
    /// the workspace root (e.g., `packages/core`).
    pub path: String,

    /// Current version before bump.
    ///
    /// The version string currently in package.json before any
    /// changes are applied (e.g., `1.0.0`).
    pub current_version: String,

    /// Next version after bump.
    ///
    /// The version string that will be (or was) written to
    /// package.json after the bump (e.g., `1.1.0`).
    pub next_version: String,

    /// Bump type applied.
    ///
    /// One of: `major`, `minor`, `patch`, `none`
    pub bump: String,

    /// List of dependency updates for this package.
    ///
    /// When this package depends on other packages that were bumped,
    /// those dependency version specifications are also updated.
    pub dependency_updates: Vec<DependencyUpdateInfo>,
}

#[allow(dead_code)]
impl PackageVersionInfo {
    /// Creates a new `PackageVersionInfo`.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name (may include scope)
    /// * `path` - Package path relative to workspace root
    /// * `current_version` - Current version before bump
    /// * `next_version` - Next version after bump
    /// * `bump` - Bump type applied
    ///
    /// # Returns
    ///
    /// A new `PackageVersionInfo` with empty dependency updates.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let info = PackageVersionInfo::new(
    ///     "@scope/core",
    ///     "packages/core",
    ///     "1.0.0",
    ///     "1.1.0",
    ///     "minor"
    /// );
    /// ```
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        path: impl Into<String>,
        current_version: impl Into<String>,
        next_version: impl Into<String>,
        bump: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            current_version: current_version.into(),
            next_version: next_version.into(),
            bump: bump.into(),
            dependency_updates: Vec::new(),
        }
    }

    /// Sets the dependency updates for this package.
    ///
    /// # Arguments
    ///
    /// * `updates` - List of dependency updates
    ///
    /// # Returns
    ///
    /// Self with the dependency updates set.
    #[must_use]
    pub fn with_dependency_updates(mut self, updates: Vec<DependencyUpdateInfo>) -> Self {
        self.dependency_updates = updates;
        self
    }

    /// Adds a single dependency update to this package.
    ///
    /// # Arguments
    ///
    /// * `update` - The dependency update to add
    ///
    /// # Returns
    ///
    /// Self with the dependency update added.
    #[must_use]
    pub fn add_dependency_update(mut self, update: DependencyUpdateInfo) -> Self {
        self.dependency_updates.push(update);
        self
    }

    /// Returns true if this package has a major bump.
    #[must_use]
    pub fn is_major(&self) -> bool {
        self.bump == "major"
    }

    /// Returns true if this package has a minor bump.
    #[must_use]
    pub fn is_minor(&self) -> bool {
        self.bump == "minor"
    }

    /// Returns true if this package has a patch bump.
    #[must_use]
    pub fn is_patch(&self) -> bool {
        self.bump == "patch"
    }

    /// Returns true if this package has no bump.
    #[must_use]
    pub fn is_none(&self) -> bool {
        self.bump == "none"
    }
}

/// Snapshot version information for a package.
///
/// This structure captures the snapshot version generated for a package,
/// including both the original version and the generated snapshot version.
///
/// # Fields
///
/// - `name`: Package name (may include scope)
/// - `path`: Package path relative to workspace root
/// - `original_version`: Original version from package.json
/// - `snapshot_version`: Generated snapshot version
///
/// # TypeScript Definition
///
/// ```typescript
/// interface SnapshotVersionInfo {
///   name: string;
///   path: string;
///   originalVersion: string;
///   snapshotVersion: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const snapshot: SnapshotVersionInfo = {
///   name: '@scope/core',
///   path: 'packages/core',
///   originalVersion: '1.0.0',
///   snapshotVersion: '1.0.0-snapshot.abc123f'
/// };
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct SnapshotVersionInfo {
    /// Package name.
    ///
    /// The full package name, including scope if applicable.
    pub name: String,

    /// Package path relative to workspace root.
    ///
    /// The file system path to the package directory.
    pub path: String,

    /// Original version from package.json.
    ///
    /// The version before snapshot generation (e.g., `1.0.0`).
    pub original_version: String,

    /// Generated snapshot version.
    ///
    /// The snapshot version generated using the format template
    /// (e.g., `1.0.0-snapshot.abc123f`).
    pub snapshot_version: String,
}

#[allow(dead_code)]
impl SnapshotVersionInfo {
    /// Creates a new `SnapshotVersionInfo`.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name (may include scope)
    /// * `path` - Package path relative to workspace root
    /// * `original_version` - Original version from package.json
    /// * `snapshot_version` - Generated snapshot version
    ///
    /// # Returns
    ///
    /// A new `SnapshotVersionInfo` instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let info = SnapshotVersionInfo::new(
    ///     "@scope/core",
    ///     "packages/core",
    ///     "1.0.0",
    ///     "1.0.0-snapshot.abc123f"
    /// );
    /// ```
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        path: impl Into<String>,
        original_version: impl Into<String>,
        snapshot_version: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            original_version: original_version.into(),
            snapshot_version: snapshot_version.into(),
        }
    }
}

/// Summary information for a bump operation.
///
/// This structure provides aggregated statistics about the version
/// bumps that were previewed or applied.
///
/// # Fields
///
/// - `total_packages`: Total number of packages affected
/// - `major_bumps`: Number of major version bumps
/// - `minor_bumps`: Number of minor version bumps
/// - `patch_bumps`: Number of patch version bumps
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BumpSummaryInfo {
///   totalPackages: number;
///   majorBumps: number;
///   minorBumps: number;
///   patchBumps: number;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const summary: BumpSummaryInfo = {
///   totalPackages: 5,
///   majorBumps: 1,
///   minorBumps: 3,
///   patchBumps: 1
/// };
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BumpSummaryInfo {
    /// Total number of packages affected by the bump.
    pub total_packages: u32,

    /// Number of major version bumps.
    pub major_bumps: u32,

    /// Number of minor version bumps.
    pub minor_bumps: u32,

    /// Number of patch version bumps.
    pub patch_bumps: u32,
}

#[allow(dead_code)]
impl BumpSummaryInfo {
    /// Creates a new `BumpSummaryInfo`.
    ///
    /// # Arguments
    ///
    /// * `total_packages` - Total number of packages affected
    /// * `major_bumps` - Number of major version bumps
    /// * `minor_bumps` - Number of minor version bumps
    /// * `patch_bumps` - Number of patch version bumps
    ///
    /// # Returns
    ///
    /// A new `BumpSummaryInfo` instance.
    #[must_use]
    pub fn new(total_packages: u32, major_bumps: u32, minor_bumps: u32, patch_bumps: u32) -> Self {
        Self { total_packages, major_bumps, minor_bumps, patch_bumps }
    }

    /// Creates an empty summary (no bumps).
    ///
    /// # Returns
    ///
    /// A new `BumpSummaryInfo` with all counts set to zero.
    #[must_use]
    pub fn empty() -> Self {
        Self { total_packages: 0, major_bumps: 0, minor_bumps: 0, patch_bumps: 0 }
    }

    /// Creates a summary from a list of package version info.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package version information
    ///
    /// # Returns
    ///
    /// A new `BumpSummaryInfo` computed from the packages.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    // Justification: It's practically impossible to have more than 4 billion packages in a
    // workspace. The u32 limit of ~4.29 billion packages is sufficient for any real-world
    // monorepo. This truncation would only occur in an unrealistic edge case.
    pub fn from_packages(packages: &[PackageVersionInfo]) -> Self {
        let total_packages = packages.len() as u32;
        let major_bumps = packages.iter().filter(|p| p.is_major()).count() as u32;
        let minor_bumps = packages.iter().filter(|p| p.is_minor()).count() as u32;
        let patch_bumps = packages.iter().filter(|p| p.is_patch()).count() as u32;

        Self { total_packages, major_bumps, minor_bumps, patch_bumps }
    }

    /// Returns true if there are any breaking changes (major bumps).
    #[must_use]
    pub fn has_breaking_changes(&self) -> bool {
        self.major_bumps > 0
    }
}

// ============================================================================
// Response Data Types - Main Data Structures
// ============================================================================

/// Response data for the bump preview command.
///
/// This structure contains the complete preview of version bumps that
/// would be applied, including all package versions, dependency updates,
/// and a summary of the changes.
///
/// # Fields
///
/// - `strategy`: Version strategy used (independent or unified)
/// - `packages`: List of packages that will be bumped
/// - `summary`: Summary statistics of the bump
/// - `changesets`: IDs of changesets that will be consumed
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BumpPreviewData {
///   strategy: 'independent' | 'unified';
///   packages: PackageVersionInfo[];
///   summary: BumpSummaryInfo;
///   changesets: string[];
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const preview: BumpPreviewData = {
///   strategy: 'independent',
///   packages: [
///     {
///       name: '@scope/core',
///       path: 'packages/core',
///       currentVersion: '1.0.0',
///       nextVersion: '1.1.0',
///       bump: 'minor',
///       dependencyUpdates: []
///     }
///   ],
///   summary: {
///     totalPackages: 1,
///     majorBumps: 0,
///     minorBumps: 1,
///     patchBumps: 0
///   },
///   changesets: ['feature-new-api']
/// };
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BumpPreviewData {
    /// Version strategy used.
    ///
    /// Either `"independent"` (each package has its own version) or
    /// `"unified"` (all packages share the same version).
    pub strategy: String,

    /// List of packages that will be bumped.
    ///
    /// Contains detailed version transition information for each
    /// package, including dependency updates.
    pub packages: Vec<PackageVersionInfo>,

    /// Summary statistics of the bump.
    ///
    /// Aggregated counts of total packages and bump types.
    pub summary: BumpSummaryInfo,

    /// IDs of changesets that will be consumed.
    ///
    /// These changesets will be archived after the bump is applied.
    pub changesets: Vec<String>,
}

#[allow(dead_code)]
impl BumpPreviewData {
    /// Creates a new `BumpPreviewData`.
    ///
    /// # Arguments
    ///
    /// * `strategy` - Version strategy (independent or unified)
    /// * `packages` - List of package version information
    /// * `changesets` - List of changeset IDs to be consumed
    ///
    /// # Returns
    ///
    /// A new `BumpPreviewData` with summary computed from packages.
    #[must_use]
    pub fn new(
        strategy: impl Into<String>,
        packages: Vec<PackageVersionInfo>,
        changesets: Vec<String>,
    ) -> Self {
        let summary = BumpSummaryInfo::from_packages(&packages);
        Self { strategy: strategy.into(), packages, summary, changesets }
    }

    /// Creates an empty preview (no bumps).
    ///
    /// # Arguments
    ///
    /// * `strategy` - Version strategy (independent or unified)
    ///
    /// # Returns
    ///
    /// A new empty `BumpPreviewData`.
    #[must_use]
    pub fn empty(strategy: impl Into<String>) -> Self {
        Self {
            strategy: strategy.into(),
            packages: Vec::new(),
            summary: BumpSummaryInfo::empty(),
            changesets: Vec::new(),
        }
    }

    /// Returns true if there are packages to bump.
    #[must_use]
    pub fn has_packages(&self) -> bool {
        !self.packages.is_empty()
    }

    /// Returns true if there are breaking changes.
    #[must_use]
    pub fn has_breaking_changes(&self) -> bool {
        self.summary.has_breaking_changes()
    }
}

/// Response data for the bump apply command.
///
/// This structure contains the results of applying version bumps,
/// including the number of packages updated, changesets archived,
/// files modified, and Git integration results.
///
/// # Fields
///
/// - `strategy`: Version strategy used (independent or unified)
/// - `packages_updated`: Number of packages that were bumped
/// - `changesets_archived`: Number of changesets that were archived
/// - `files_modified`: List of files that were modified
/// - `tags_created`: List of Git tags that were created
/// - `commit_sha`: Git commit SHA (if gitCommit was true)
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BumpApplyData {
///   strategy: 'independent' | 'unified';
///   packagesUpdated: number;
///   changesetsArchived: number;
///   filesModified: string[];
///   tagsCreated: string[];
///   commitSha?: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const apply: BumpApplyData = {
///   strategy: 'independent',
///   packagesUpdated: 3,
///   changesetsArchived: 2,
///   filesModified: [
///     'packages/core/package.json',
///     'packages/core/CHANGELOG.md',
///     'packages/utils/package.json'
///   ],
///   tagsCreated: ['@scope/core@1.1.0', '@scope/utils@2.0.0'],
///   commitSha: 'abc123def456789'
/// };
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BumpApplyData {
    /// Version strategy used.
    ///
    /// Either `"independent"` or `"unified"`.
    pub strategy: String,

    /// Number of packages that were bumped.
    pub packages_updated: u32,

    /// Number of changesets that were archived.
    pub changesets_archived: u32,

    /// List of files that were modified.
    ///
    /// Paths are relative to the workspace root.
    pub files_modified: Vec<String>,

    /// List of Git tags that were created.
    ///
    /// Format: `{package}@{version}` (e.g., `@scope/core@1.1.0`)
    pub tags_created: Vec<String>,

    /// Git commit SHA (if gitCommit was true).
    ///
    /// The full 40-character SHA of the commit containing
    /// all version bump changes.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
}

#[allow(dead_code)]
impl BumpApplyData {
    /// Creates a new `BumpApplyData`.
    ///
    /// # Arguments
    ///
    /// * `strategy` - Version strategy (independent or unified)
    /// * `packages_updated` - Number of packages updated
    /// * `changesets_archived` - Number of changesets archived
    ///
    /// # Returns
    ///
    /// A new `BumpApplyData` with empty lists for files and tags.
    #[must_use]
    pub fn new(
        strategy: impl Into<String>,
        packages_updated: u32,
        changesets_archived: u32,
    ) -> Self {
        Self {
            strategy: strategy.into(),
            packages_updated,
            changesets_archived,
            files_modified: Vec::new(),
            tags_created: Vec::new(),
            commit_sha: None,
        }
    }

    /// Sets the files modified.
    ///
    /// # Arguments
    ///
    /// * `files` - List of modified file paths
    ///
    /// # Returns
    ///
    /// Self with the files modified set.
    #[must_use]
    pub fn with_files_modified(mut self, files: Vec<String>) -> Self {
        self.files_modified = files;
        self
    }

    /// Sets the tags created.
    ///
    /// # Arguments
    ///
    /// * `tags` - List of Git tags created
    ///
    /// # Returns
    ///
    /// Self with the tags created set.
    #[must_use]
    pub fn with_tags_created(mut self, tags: Vec<String>) -> Self {
        self.tags_created = tags;
        self
    }

    /// Sets the commit SHA.
    ///
    /// # Arguments
    ///
    /// * `sha` - The Git commit SHA
    ///
    /// # Returns
    ///
    /// Self with the commit SHA set.
    #[must_use]
    pub fn with_commit_sha(mut self, sha: impl Into<String>) -> Self {
        self.commit_sha = Some(sha.into());
        self
    }

    /// Returns true if a Git commit was created.
    #[must_use]
    pub fn has_commit(&self) -> bool {
        self.commit_sha.is_some()
    }

    /// Returns true if Git tags were created.
    #[must_use]
    pub fn has_tags(&self) -> bool {
        !self.tags_created.is_empty()
    }
}

/// Response data for the bump snapshot command.
///
/// This structure contains the results of generating snapshot versions,
/// including the list of packages with their snapshot versions and the
/// format template that was used.
///
/// # Fields
///
/// - `strategy`: Version strategy used (independent or unified)
/// - `packages`: List of packages with snapshot versions
/// - `format`: The format template that was used
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BumpSnapshotData {
///   strategy: 'independent' | 'unified';
///   packages: SnapshotVersionInfo[];
///   format: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const snapshot: BumpSnapshotData = {
///   strategy: 'independent',
///   packages: [
///     {
///       name: '@scope/core',
///       path: 'packages/core',
///       originalVersion: '1.0.0',
///       snapshotVersion: '1.0.0-snapshot.abc123f'
///     }
///   ],
///   format: '{version}-snapshot.{short_commit}'
/// };
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BumpSnapshotData {
    /// Version strategy used.
    ///
    /// Either `"independent"` or `"unified"`.
    pub strategy: String,

    /// List of packages with snapshot versions.
    ///
    /// Contains the original and generated snapshot version for each package.
    pub packages: Vec<SnapshotVersionInfo>,

    /// The format template that was used.
    ///
    /// This is either the user-provided format or the default format
    /// `{version}-snapshot.{short_commit}`.
    pub format: String,
}

#[allow(dead_code)]
impl BumpSnapshotData {
    /// Creates a new `BumpSnapshotData`.
    ///
    /// # Arguments
    ///
    /// * `strategy` - Version strategy (independent or unified)
    /// * `packages` - List of snapshot version information
    /// * `format` - The format template used
    ///
    /// # Returns
    ///
    /// A new `BumpSnapshotData` instance.
    #[must_use]
    pub fn new(
        strategy: impl Into<String>,
        packages: Vec<SnapshotVersionInfo>,
        format: impl Into<String>,
    ) -> Self {
        Self { strategy: strategy.into(), packages, format: format.into() }
    }

    /// Creates an empty snapshot data.
    ///
    /// # Arguments
    ///
    /// * `strategy` - Version strategy (independent or unified)
    /// * `format` - The format template used
    ///
    /// # Returns
    ///
    /// A new empty `BumpSnapshotData`.
    #[must_use]
    pub fn empty(strategy: impl Into<String>, format: impl Into<String>) -> Self {
        Self { strategy: strategy.into(), packages: Vec::new(), format: format.into() }
    }

    /// Returns the number of packages with snapshot versions.
    #[must_use]
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }
}

// ============================================================================
// API Response Types
// ============================================================================

/// API response for the bump preview command.
///
/// This structure wraps `BumpPreviewData` in the standard `ApiResponse`
/// format, providing a consistent interface for success and error cases.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BumpPreviewApiResponse {
///   success: boolean;
///   data?: BumpPreviewData;
///   error?: ErrorInfo;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await bumpPreview({ root: '.' });
///
/// if (result.success) {
///   console.log(`Will bump ${result.data.packages.length} packages`);
/// } else {
///   console.error(`Error: ${result.error.message}`);
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BumpPreviewApiResponse {
    /// Whether the operation was successful.
    pub success: bool,

    /// The preview data if successful.
    ///
    /// Contains the complete preview of version bumps including all
    /// packages, dependency updates, and summary.
    #[napi(ts_type = "BumpPreviewData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<BumpPreviewData>,

    /// Error information if the operation failed.
    ///
    /// Contains the error code, message, and context when the
    /// operation fails.
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl BumpPreviewApiResponse {
    /// Creates a successful response with preview data.
    ///
    /// # Arguments
    ///
    /// * `data` - The bump preview data
    ///
    /// # Returns
    ///
    /// A new successful `BumpPreviewApiResponse`.
    #[must_use]
    pub fn success(data: BumpPreviewData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failure response with error information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information
    ///
    /// # Returns
    ///
    /// A new failed `BumpPreviewApiResponse`.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns true if the response indicates success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns true if the response indicates failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// API response for the bump apply command.
///
/// This structure wraps `BumpApplyData` in the standard `ApiResponse`
/// format, providing a consistent interface for success and error cases.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BumpApplyApiResponse {
///   success: boolean;
///   data?: BumpApplyData;
///   error?: ErrorInfo;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await bumpApply({
///   root: '.',
///   gitCommit: true,
///   gitTag: true
/// });
///
/// if (result.success) {
///   console.log(`Updated ${result.data.packagesUpdated} packages`);
///   if (result.data.commitSha) {
///     console.log(`Commit: ${result.data.commitSha}`);
///   }
/// } else {
///   console.error(`Error: ${result.error.message}`);
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BumpApplyApiResponse {
    /// Whether the operation was successful.
    pub success: bool,

    /// The apply result data if successful.
    ///
    /// Contains information about what was updated, including
    /// packages, files, and Git integration results.
    #[napi(ts_type = "BumpApplyData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<BumpApplyData>,

    /// Error information if the operation failed.
    ///
    /// Contains the error code, message, and context when the
    /// operation fails.
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl BumpApplyApiResponse {
    /// Creates a successful response with apply data.
    ///
    /// # Arguments
    ///
    /// * `data` - The bump apply data
    ///
    /// # Returns
    ///
    /// A new successful `BumpApplyApiResponse`.
    #[must_use]
    pub fn success(data: BumpApplyData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failure response with error information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information
    ///
    /// # Returns
    ///
    /// A new failed `BumpApplyApiResponse`.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns true if the response indicates success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns true if the response indicates failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// API response for the bump snapshot command.
///
/// This structure wraps `BumpSnapshotData` in the standard `ApiResponse`
/// format, providing a consistent interface for success and error cases.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BumpSnapshotApiResponse {
///   success: boolean;
///   data?: BumpSnapshotData;
///   error?: ErrorInfo;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await bumpSnapshot({
///   root: '.',
///   format: '{version}-{branch}.{short_commit}'
/// });
///
/// if (result.success) {
///   for (const pkg of result.data.packages) {
///     console.log(`${pkg.name}: ${pkg.snapshotVersion}`);
///   }
/// } else {
///   console.error(`Error: ${result.error.message}`);
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BumpSnapshotApiResponse {
    /// Whether the operation was successful.
    pub success: bool,

    /// The snapshot result data if successful.
    ///
    /// Contains the list of packages with their generated
    /// snapshot versions.
    #[napi(ts_type = "BumpSnapshotData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<BumpSnapshotData>,

    /// Error information if the operation failed.
    ///
    /// Contains the error code, message, and context when the
    /// operation fails.
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl BumpSnapshotApiResponse {
    /// Creates a successful response with snapshot data.
    ///
    /// # Arguments
    ///
    /// * `data` - The bump snapshot data
    ///
    /// # Returns
    ///
    /// A new successful `BumpSnapshotApiResponse`.
    #[must_use]
    pub fn success(data: BumpSnapshotData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failure response with error information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information
    ///
    /// # Returns
    ///
    /// A new failed `BumpSnapshotApiResponse`.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns true if the response indicates success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns true if the response indicates failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}
