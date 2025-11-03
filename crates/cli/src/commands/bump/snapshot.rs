//! Bump snapshot structures and snapshot version generation.
//!
//! This module provides both data structures and command execution for version snapshots.
//!
//! # What
//!
//! Provides:
//! - **Data Structures**:
//!   - `BumpSnapshot` - Complete bump preview with all package updates
//!   - `PackageBumpInfo` - Information about a single package's version bump
//!   - `BumpSummary` - Summary statistics about the bump operation
//!   - JSON serialization support for automation and CI/CD
//!
//! - **Snapshot Generation**:
//!   - `execute_bump_snapshot` - Generate snapshot versions for testing
//!   - Includes git commit hash and branch information
//!   - Respects versioning strategy (Independent vs Unified)
//!   - Does NOT consume/archive changesets
//!
//! # How
//!
//! ## Data Structures
//!
//! These structures are populated by:
//! 1. Loading active changesets
//! 2. Using `VersionResolver` to calculate version bumps
//! 3. Determining which packages will bump based on versioning strategy
//! 4. Collecting all information into the snapshot structure
//!
//! The snapshot can then be:
//! - Displayed as a formatted table (human mode)
//! - Serialized to JSON (automation mode)
//!
//! ## Snapshot Generation
//!
//! The command flow:
//! 1. Opens Git repository to get current commit hash and branch
//! 2. Loads workspace configuration to determine versioning strategy
//! 3. Loads all pending changesets
//! 4. If no changesets exist, reports "nothing to snapshot"
//! 5. Determines which packages to snapshot based on strategy:
//!    - Independent: Only packages in changesets
//!    - Unified: All workspace packages
//! 6. For each package, generates snapshot version using `SnapshotGenerator`
//! 7. Outputs snapshot versions in table or JSON format
//! 8. Does NOT modify any files or consume changesets
//!
//! ### Snapshot Format
//!
//! Default format: `{version}-snapshot.{short_commit}`
//! Example: `1.2.3-snapshot.abc123f`
//!
//! Custom format can be specified via `--snapshot-format` flag.
//! Available variables:
//! - `{version}`: Base version (e.g., 1.2.3)
//! - `{branch}`: Git branch name (sanitized)
//! - `{commit}`: Full commit hash
//! - `{short_commit}`: Short commit hash (7 chars)
//! - `{timestamp}`: Unix timestamp
//!
//! # Why
//!
//! This module combines data structures with snapshot generation because:
//! - Both share the same data model (`BumpSnapshot`, `PackageBumpInfo`)
//! - Snapshot generation is a specialized variant of bump preview
//! - Keeps related functionality together
//! - Consistent with `preview.rs` and `execute.rs` patterns
//!
//! Snapshot versions are essential for:
//! - Testing unreleased changes in other projects
//! - Creating preview deployments
//! - Publishing canary/nightly builds
//! - Validating changes before official release
//! - CI/CD testing workflows
//!
//! # Examples
//!
//! ## Using Data Structures
//!
//! ```rust
//! use sublime_cli_tools::commands::bump::snapshot::{
//!     BumpSnapshot, PackageBumpInfo, BumpSummary
//! };
//! use sublime_pkg_tools::types::VersionBump;
//!
//! let package_info = PackageBumpInfo {
//!     name: "@org/core".to_string(),
//!     path: "packages/core".to_string(),
//!     current_version: "1.2.3".to_string(),
//!     next_version: "1.3.0".to_string(),
//!     bump_type: VersionBump::Minor,
//!     will_bump: true,
//!     reason: "direct change from changeset".to_string(),
//! };
//! ```
//!
//! ## Generating Snapshots
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::bump::execute_bump_snapshot;
//! use sublime_cli_tools::cli::commands::BumpArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let args = BumpArgs {
//!     dry_run: false,
//!     execute: false,
//!     snapshot: true,
//!     snapshot_format: None,  // Use default format
//!     prerelease: None,
//!     packages: None,
//!     git_tag: false,
//!     git_push: false,
//!     git_commit: false,
//!     no_changelog: false,
//!     no_archive: false,
//!     force: false,
//! };
//!
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_bump_snapshot(&args, &output, Path::new("."), None).await?;
//! # Ok(())
//! # }
//! ```

use crate::cli::commands::BumpArgs;
use crate::commands::bump::preview::{load_config, merge_changesets};
use crate::error::{CliError, Result};
use crate::output::styling::{StatusSymbol, print_item};
use crate::output::table::{ColumnAlignment, TableBuilder, TableTheme};
use crate::output::{JsonResponse, Output};
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use sublime_git_tools::Repo;
use sublime_pkg_tools::changeset::ChangesetManager;
use sublime_pkg_tools::types::{Changeset, PackageInfo, VersionBump};
use sublime_pkg_tools::version::{SnapshotContext, SnapshotGenerator, VersionResolver};
use sublime_standard_tools::filesystem::FileSystemManager;
use tracing::{debug, info, warn};

/// Complete snapshot of a version bump operation.
///
/// Contains all information about packages that will be bumped, changesets
/// being processed, and summary statistics. Used for both preview (dry-run)
/// and execute modes.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::bump::snapshot::BumpSnapshot;
///
/// let snapshot = BumpSnapshot {
///     strategy: "independent".to_string(),
///     packages: vec![],
///     changesets: vec![],
///     summary: Default::default(),
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BumpSnapshot {
    /// Versioning strategy being used (Independent or Unified).
    pub strategy: String,

    /// List of all workspace packages with their bump information.
    ///
    /// Includes both packages that will bump and those that won't.
    /// Check `will_bump` field to distinguish.
    pub packages: Vec<PackageBumpInfo>,

    /// List of changesets being processed in this bump.
    pub changesets: Vec<ChangesetInfo>,

    /// Summary statistics for the bump operation.
    pub summary: BumpSummary,
}

/// Information about a single package's version bump.
///
/// Contains current and next versions, bump type, and whether the package
/// will actually be bumped based on the versioning strategy and changesets.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::bump::snapshot::PackageBumpInfo;
/// use sublime_pkg_tools::types::VersionBump;
///
/// let info = PackageBumpInfo {
///     name: "@org/core".to_string(),
///     path: "packages/core".to_string(),
///     current_version: "1.2.3".to_string(),
///     next_version: "1.3.0".to_string(),
///     bump_type: VersionBump::Minor,
///     will_bump: true,
///     reason: "direct change from changeset".to_string(),
/// };
///
/// // Example: Package with dependency update but not bumping (Independent mode)
/// let info2 = PackageBumpInfo {
///     name: "@org/utils".to_string(),
///     path: "packages/utils".to_string(),
///     current_version: "2.0.0".to_string(),
///     next_version: "2.0.0".to_string(),
///     bump_type: VersionBump::None,
///     will_bump: false,
///     reason: "dependency updated (not bumping in independent mode)".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageBumpInfo {
    /// Package name (e.g., "@org/core").
    pub name: String,

    /// Relative path to package directory.
    pub path: String,

    /// Current version (e.g., "1.2.3").
    pub current_version: String,

    /// Next version after bump (e.g., "1.3.0").
    ///
    /// If `will_bump` is false, this is the same as `current_version`.
    pub next_version: String,

    /// Type of version bump.
    ///
    /// None if the package won't be bumped.
    pub bump_type: VersionBump,

    /// Whether this package will actually be bumped.
    ///
    /// - Independent strategy: true only if package is in a changeset
    /// - Unified strategy: true for all packages if any changeset exists
    pub will_bump: bool,

    /// Human-readable reason for bump or no-bump.
    ///
    /// Examples:
    /// - **Independent strategy**:
    ///   - "direct change from changeset" - Package is in a changeset and will bump
    ///   - "in changeset but version unchanged" - Package in changeset but no version change needed
    ///   - "dependency updated (not bumping in independent mode)" - Dependency changed but package not in changeset
    ///   - "not in any changeset" - Package not affected by any changeset
    /// - **Unified strategy**:
    ///   - "unified bump (package in changeset)" - Package is in a changeset causing unified bump
    ///   - "unified bump (all packages bumped together)" - Package bumped due to unified strategy
    pub reason: String,
}

/// Information about a changeset being processed.
///
/// Summarizes a changeset's key details for display and JSON output.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::bump::snapshot::ChangesetInfo;
/// use sublime_pkg_tools::types::VersionBump;
///
/// let info = ChangesetInfo {
///     id: "feature-new-api".to_string(),
///     branch: "feature/new-api".to_string(),
///     bump_type: VersionBump::Minor,
///     packages: vec!["@org/core".to_string()],
///     commit_count: 5,
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangesetInfo {
    /// Changeset ID.
    pub id: String,

    /// Git branch name.
    pub branch: String,

    /// Bump type for this changeset.
    pub bump_type: VersionBump,

    /// List of packages affected by this changeset.
    pub packages: Vec<String>,

    /// Number of commits in this changeset.
    pub commit_count: usize,
}

/// Summary statistics for a bump operation.
///
/// Provides high-level counts and information about the bump.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::bump::snapshot::BumpSummary;
///
/// let summary = BumpSummary {
///     total_packages: 10,
///     packages_to_bump: 3,
///     packages_unchanged: 7,
///     total_changesets: 2,
///     has_circular_dependencies: false,
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BumpSummary {
    /// Total number of packages in workspace.
    pub total_packages: usize,

    /// Number of packages that will be bumped.
    pub packages_to_bump: usize,

    /// Number of packages that won't be bumped.
    pub packages_unchanged: usize,

    /// Total number of changesets being processed.
    pub total_changesets: usize,

    /// Whether circular dependencies were detected.
    pub has_circular_dependencies: bool,
}

impl BumpSummary {
    /// Creates a new BumpSummary from package and changeset counts.
    ///
    /// # Arguments
    ///
    /// * `total_packages` - Total number of packages in workspace
    /// * `packages_to_bump` - Number of packages that will be bumped
    /// * `total_changesets` - Number of changesets being processed
    /// * `has_circular_dependencies` - Whether circular dependencies exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::bump::snapshot::BumpSummary;
    ///
    /// let summary = BumpSummary::new(10, 3, 2, false);
    /// assert_eq!(summary.packages_unchanged, 7);
    /// ```
    #[must_use]
    pub const fn new(
        total_packages: usize,
        packages_to_bump: usize,
        total_changesets: usize,
        has_circular_dependencies: bool,
    ) -> Self {
        Self {
            total_packages,
            packages_to_bump,
            packages_unchanged: total_packages.saturating_sub(packages_to_bump),
            total_changesets,
            has_circular_dependencies,
        }
    }
}

// ============================================================================
// Snapshot Version Generation
// ============================================================================

/// Execute the bump snapshot command.
///
/// Generates snapshot versions for testing unreleased changes. Respects
/// versioning strategy but does NOT consume changesets or modify files.
///
/// # Arguments
///
/// * `args` - Command arguments with optional snapshot format template
/// * `output` - Output handler for formatting and displaying results
/// * `root` - Workspace root directory
/// * `config_path` - Optional path to config file (from global `--config` option)
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if the operation fails.
///
/// # Errors
///
/// Returns an error if:
/// - The workspace is not initialized (no configuration found)
/// - Cannot load changesets or packages
/// - Git repository is not available or invalid
/// - Snapshot format template is invalid
/// - Version resolution fails
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::bump::execute_bump_snapshot;
/// use sublime_cli_tools::cli::commands::BumpArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = BumpArgs {
///     snapshot: true,
///     snapshot_format: Some("{version}-{branch}.{short_commit}".to_string()),
///     ..Default::default()
/// };
/// let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
/// execute_bump_snapshot(&args, &output, Path::new("."), None).await?;
/// # Ok(())
/// # }
/// ```
// Allow too_many_lines: This function orchestrates the complete snapshot generation workflow with many steps
// including git operations, configuration loading, changeset management, version resolution, snapshot generation,
// and output formatting. Breaking it into smaller functions would reduce readability and make the
// sequential workflow harder to follow. The length is justified by the complex multi-step process.
#[allow(clippy::too_many_lines)]
pub async fn execute_bump_snapshot(
    args: &BumpArgs,
    output: &Output,
    root: &Path,
    config_path: Option<&Path>,
) -> Result<()> {
    let workspace_root = root;
    info!("Executing snapshot version generation in workspace: {}", workspace_root.display());

    // Step 1: Open Git repository to get commit information
    let repo = Repo::open(workspace_root.to_str().ok_or_else(|| {
        CliError::execution("Workspace path contains invalid UTF-8".to_string())
    })?)
    .map_err(|e| {
        CliError::execution(format!(
            "Failed to open Git repository at {}: {}. Snapshot generation requires a Git repository.",
            workspace_root.display(),
            e
        ))
    })?;

    // Get current commit hash and branch
    let current_commit = repo.get_current_sha().map_err(|e| {
        CliError::execution(format!(
            "Failed to get current commit SHA: {e}. Repository may have no commits."
        ))
    })?;

    let current_branch = repo
        .get_current_branch()
        .map_err(|e| CliError::execution(format!("Failed to get current branch: {e}")))?;

    debug!("Current commit: {}", current_commit);
    debug!("Current branch: {}", current_branch);

    // Step 2: Load configuration
    let config = load_config(workspace_root, config_path).await?;
    info!("Configuration loaded successfully");
    debug!("Versioning strategy: {:?}", config.version.strategy);

    // Step 3: Determine snapshot format
    let snapshot_format = if let Some(ref custom_format) = args.snapshot_format {
        custom_format.clone()
    } else if !config.version.snapshot_format.is_empty() {
        config.version.snapshot_format.clone()
    } else {
        // Default format
        "{version}-snapshot.{short_commit}".to_string()
    };

    debug!("Snapshot format: {}", snapshot_format);

    // Step 4: Create snapshot generator
    let generator = SnapshotGenerator::new(&snapshot_format)
        .map_err(|e| CliError::execution(format!("Invalid snapshot format template: {e}")))?;

    // Step 5: Load all pending changesets
    let fs = FileSystemManager::new();
    let manager = ChangesetManager::new(workspace_root.to_path_buf(), fs.clone(), config.clone())
        .await
        .map_err(|e| CliError::execution(format!("Failed to create changeset manager: {e}")))?;

    let changesets = manager
        .list_pending()
        .await
        .map_err(|e| CliError::execution(format!("Failed to load changesets: {e}")))?;

    debug!("Loaded {} changeset(s)", changesets.len());

    // Step 6: Check if there are any changesets
    if changesets.is_empty() {
        if output.format().is_json() {
            let response: JsonResponse<BumpSnapshot> = JsonResponse::success(BumpSnapshot {
                strategy: config.version.strategy.to_string(),
                packages: vec![],
                changesets: vec![],
                summary: BumpSummary::default(),
            });
            output.json(&response)?;
        } else {
            output.info("No changesets found. Nothing to snapshot.")?;
        }
        return Ok(());
    }

    info!("Processing {} changeset(s) for snapshot generation", changesets.len());

    // Step 7: Create VersionResolver and discover packages
    let resolver = VersionResolver::new(workspace_root.to_path_buf(), config.clone())
        .await
        .map_err(|e| CliError::execution(format!("Failed to create version resolver: {e}")))?;

    let all_packages = resolver
        .discover_packages()
        .await
        .map_err(|e| CliError::execution(format!("Failed to discover packages: {e}")))?;

    debug!("Discovered {} workspace package(s)", all_packages.len());

    // Step 8: Generate snapshots based on strategy
    let snapshot =
        if config.version.strategy == sublime_pkg_tools::config::VersioningStrategy::Independent {
            build_independent_snapshots(
                &resolver,
                &generator,
                &changesets,
                &all_packages,
                workspace_root,
                &current_commit,
                &current_branch,
            )
            .await?
        } else {
            build_unified_snapshots(
                &resolver,
                &generator,
                &changesets,
                &all_packages,
                workspace_root,
                &current_commit,
                &current_branch,
            )
            .await?
        };

    debug!("Generated snapshot versions for {} packages", snapshot.packages.len());

    // Step 9: Output results
    if output.format().is_json() {
        let response: JsonResponse<BumpSnapshot> = JsonResponse::success(snapshot);
        output.json(&response)?;
    } else {
        output_snapshot_table(output, &snapshot)?;
    }

    Ok(())
}

/// Builds snapshot versions for Independent versioning strategy.
///
/// In Independent mode, only packages explicitly listed in changesets receive
/// snapshot versions. Each package gets its own snapshot version based on its
/// current version and the changeset bump type.
async fn build_independent_snapshots(
    resolver: &VersionResolver,
    generator: &SnapshotGenerator,
    changesets: &[Changeset],
    all_packages: &[PackageInfo],
    workspace_root: &Path,
    current_commit: &str,
    current_branch: &str,
) -> Result<BumpSnapshot> {
    debug!("Building snapshot versions for Independent versioning strategy");

    // Collect all packages that are in changesets
    let mut changeset_packages: HashSet<String> = HashSet::new();
    for changeset in changesets {
        changeset_packages.extend(changeset.packages.iter().cloned());
    }

    debug!("Packages in changesets: {:?}", changeset_packages);

    // Merge all changesets to get the combined bump requirements
    let merged_changeset = merge_changesets(changesets)?;

    // Resolve versions using the merged changeset
    let resolution = resolver
        .resolve_versions(&merged_changeset)
        .await
        .map_err(|e| CliError::execution(format!("Failed to resolve versions: {e}")))?;

    debug!("Resolved {} package updates", resolution.updates.len());

    // Get current timestamp
    // Allow cast_possible_wrap: i64 is required by SnapshotContext and can represent timestamps
    // until year 292 billion, which is practically infinite for our use case
    #[allow(clippy::cast_possible_wrap)]
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| CliError::execution(format!("Failed to get current timestamp: {e}")))?
        .as_secs() as i64;

    // Build snapshot versions for packages in changesets
    let mut packages_info = Vec::new();
    let mut packages_with_snapshots = 0;

    for update in &resolution.updates {
        let is_in_changeset = changeset_packages.contains(&update.name);

        if is_in_changeset {
            // Generate snapshot version
            let context = SnapshotContext {
                version: update.next_version.clone(),
                branch: sanitize_branch_name(current_branch),
                commit: current_commit.to_string(),
                timestamp,
            };

            let snapshot_version = generator.generate(&context).map_err(|e| {
                CliError::execution(format!(
                    "Failed to generate snapshot version for {}: {}",
                    update.name, e
                ))
            })?;

            let bump_type = calculate_bump_type(&update.current_version, &update.next_version);

            packages_info.push(PackageBumpInfo {
                name: update.name.clone(),
                path: update
                    .path
                    .strip_prefix(workspace_root)
                    .unwrap_or(&update.path)
                    .display()
                    .to_string(),
                current_version: update.current_version.to_string(),
                next_version: snapshot_version,
                bump_type,
                will_bump: true,
                reason: "snapshot from changeset".to_string(),
            });

            packages_with_snapshots += 1;
        }
    }

    // Build changeset info
    let changeset_infos: Vec<ChangesetInfo> = changesets
        .iter()
        .map(|cs| ChangesetInfo {
            id: cs.branch.clone(),
            branch: cs.branch.clone(),
            bump_type: cs.bump,
            packages: cs.packages.clone(),
            commit_count: cs.changes.len(),
        })
        .collect();

    let summary = BumpSummary::new(
        all_packages.len(),
        packages_with_snapshots,
        changesets.len(),
        !resolution.circular_dependencies.is_empty(),
    );

    if !resolution.circular_dependencies.is_empty() {
        warn!("Detected {} circular dependencies", resolution.circular_dependencies.len());
    }

    Ok(BumpSnapshot {
        strategy: "independent".to_string(),
        packages: packages_info,
        changesets: changeset_infos,
        summary,
    })
}

/// Builds snapshot versions for Unified versioning strategy.
///
/// In Unified mode, ALL workspace packages receive snapshot versions when
/// any changeset exists. All packages get the same snapshot version based
/// on the highest bump type from all changesets.
async fn build_unified_snapshots(
    resolver: &VersionResolver,
    generator: &SnapshotGenerator,
    changesets: &[Changeset],
    all_packages: &[PackageInfo],
    workspace_root: &Path,
    current_commit: &str,
    current_branch: &str,
) -> Result<BumpSnapshot> {
    debug!("Building snapshot versions for Unified versioning strategy");

    // Merge all changesets to get the combined bump requirements
    let merged_changeset = merge_changesets(changesets)?;

    // Resolve versions using the merged changeset
    let resolution = resolver
        .resolve_versions(&merged_changeset)
        .await
        .map_err(|e| CliError::execution(format!("Failed to resolve versions: {e}")))?;

    debug!("Resolved {} package updates", resolution.updates.len());

    // Get current timestamp
    // Allow cast_possible_wrap: i64 is required by SnapshotContext and can represent timestamps
    // until year 292 billion, which is practically infinite for our use case
    #[allow(clippy::cast_possible_wrap)]
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| CliError::execution(format!("Failed to get current timestamp: {e}")))?
        .as_secs() as i64;

    // In unified mode, ALL packages get snapshot versions
    let packages_with_snapshots = all_packages.len();

    // Build snapshot versions for all packages
    let mut packages_info = Vec::new();

    // Check which packages are directly in changesets for better reason messages
    let mut changeset_packages: HashSet<String> = HashSet::new();
    for changeset in changesets {
        changeset_packages.extend(changeset.packages.iter().cloned());
    }

    for update in &resolution.updates {
        // Generate snapshot version
        let context = SnapshotContext {
            version: update.next_version.clone(),
            branch: sanitize_branch_name(current_branch),
            commit: current_commit.to_string(),
            timestamp,
        };

        let snapshot_version = generator.generate(&context).map_err(|e| {
            CliError::execution(format!(
                "Failed to generate snapshot version for {}: {}",
                update.name, e
            ))
        })?;

        let bump_type = calculate_bump_type(&update.current_version, &update.next_version);
        let is_in_changeset = changeset_packages.contains(&update.name);

        // Provide clear reason for unified snapshot
        let reason = if is_in_changeset {
            "unified snapshot (package in changeset)".to_string()
        } else {
            "unified snapshot (all packages bumped together)".to_string()
        };

        packages_info.push(PackageBumpInfo {
            name: update.name.clone(),
            path: update
                .path
                .strip_prefix(workspace_root)
                .unwrap_or(&update.path)
                .display()
                .to_string(),
            current_version: update.current_version.to_string(),
            next_version: snapshot_version,
            bump_type,
            will_bump: true,
            reason,
        });
    }

    // Build changeset info
    let changeset_infos: Vec<ChangesetInfo> = changesets
        .iter()
        .map(|cs| ChangesetInfo {
            id: cs.branch.clone(),
            branch: cs.branch.clone(),
            bump_type: cs.bump,
            packages: cs.packages.clone(),
            commit_count: cs.changes.len(),
        })
        .collect();

    let summary = BumpSummary::new(
        all_packages.len(),
        packages_with_snapshots,
        changesets.len(),
        !resolution.circular_dependencies.is_empty(),
    );

    if !resolution.circular_dependencies.is_empty() {
        warn!("Detected {} circular dependencies", resolution.circular_dependencies.len());
    }

    Ok(BumpSnapshot {
        strategy: "unified".to_string(),
        packages: packages_info,
        changesets: changeset_infos,
        summary,
    })
}

/// Calculates the bump type by comparing two versions.
///
/// Determines whether the version change is a major, minor, or patch bump.
fn calculate_bump_type(
    current: &sublime_pkg_tools::types::Version,
    next: &sublime_pkg_tools::types::Version,
) -> VersionBump {
    if next.major() > current.major() {
        VersionBump::Major
    } else if next.minor() > current.minor() {
        VersionBump::Minor
    } else if next.patch() > current.patch() {
        VersionBump::Patch
    } else {
        VersionBump::None
    }
}

/// Sanitizes a branch name for use in snapshot versions.
///
/// Replaces characters that are invalid in semver identifiers.
/// Valid characters: ASCII alphanumeric, dash, dot
///
/// # Examples
///
/// ```rust
/// # // Internal function, tested via unit tests
/// # fn sanitize_branch_name(branch: &str) -> String {
/// #     branch.chars()
/// #         .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '.' { c } else { '-' })
/// #         .collect()
/// # }
/// assert_eq!(sanitize_branch_name("feature/new-api"), "feature-new-api");
/// assert_eq!(sanitize_branch_name("feat/#123-fix"), "feat--123-fix");
/// ```
pub(crate) fn sanitize_branch_name(branch: &str) -> String {
    branch
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '.' { c } else { '-' })
        .collect()
}

/// Outputs snapshot results as a formatted table.
///
/// Displays changesets being processed, packages with their snapshot versions,
/// and a summary section with statistics.
fn output_snapshot_table(output: &Output, snapshot: &BumpSnapshot) -> Result<()> {
    // Display strategy
    StatusSymbol::Info.print_line(&format!("Strategy: {}", snapshot.strategy));
    output.blank_line()?;

    // Display important note
    StatusSymbol::Info.print_line("Mode: Snapshot (changesets will NOT be consumed)");
    output.blank_line()?;

    // Display changesets section
    if !snapshot.changesets.is_empty() {
        StatusSymbol::Info
            .print_line(&format!("Active Changesets: {} changeset(s)", snapshot.changesets.len()));

        let mut changeset_table = TableBuilder::new()
            .theme(TableTheme::Minimal)
            .columns(&["Branch", "Bump", "Packages", "Commits"])
            .alignment(3, ColumnAlignment::Right)
            .build();

        for cs in &snapshot.changesets {
            changeset_table.add_row(&[
                &cs.branch,
                &cs.bump_type.to_string().to_lowercase(),
                &cs.packages.len().to_string(),
                &cs.commit_count.to_string(),
            ]);
        }

        output.table(&mut changeset_table)?;
        output.blank_line()?;
    }

    // Display packages section
    StatusSymbol::Info.print_line(&format!(
        "Snapshot Versions: {} package(s)",
        snapshot.summary.packages_to_bump
    ));

    let mut package_table = TableBuilder::new()
        .theme(TableTheme::Minimal)
        .columns(&["Package", "Current", "Snapshot Version", "Bump"])
        .alignment(1, ColumnAlignment::Center)
        .alignment(3, ColumnAlignment::Center)
        .build();

    for pkg in &snapshot.packages {
        let bump_display = pkg.bump_type.to_string().to_lowercase();

        package_table.add_row(&[&pkg.name, &pkg.current_version, &pkg.next_version, &bump_display]);
    }

    output.table(&mut package_table)?;
    output.blank_line()?;

    // Display summary
    StatusSymbol::Info.print_line("Summary:");
    print_item("  Total packages", &snapshot.summary.total_packages.to_string(), false);
    print_item("  With snapshots", &snapshot.summary.packages_to_bump.to_string(), false);
    print_item("  Unchanged", &snapshot.summary.packages_unchanged.to_string(), true);

    if snapshot.summary.has_circular_dependencies {
        output.blank_line()?;
        output.warning("Circular dependencies detected. Review dependency graph.")?;
    }

    output.blank_line()?;
    StatusSymbol::Success.print_line("Snapshot versions generated successfully!");
    output.info("Note: No files were modified. Changesets remain active for future release.")?;

    Ok(())
}
