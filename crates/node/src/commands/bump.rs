//! Bump command implementations for Node.js bindings.
//!
//! # What
//!
//! This module implements the bump NAPI functions that handle version management
//! based on pending changesets. It supports previewing changes, applying versions,
//! and generating snapshot versions for pre-releases.
//!
//! # How
//!
//! The module provides the following functions:
//!
//! - `bumpPreview`: Shows what versions would be bumped without making changes (Story 5.2)
//! - `bumpApply`: Applies version bumps, updates dependencies, and optionally commits/tags (Story 5.3)
//! - `bumpSnapshot`: Generates snapshot versions for pre-release testing (Story 5.4)
//!
//! Each function:
//! 1. Validates the input parameters
//! 2. Calls the appropriate `execute_*` function from `sublime_cli_tools`
//! 3. Captures the JSON output using a `SharedBuffer`
//! 4. Parses the JSON response using serde
//! 5. Converts CLI types to NAPI-compatible types
//! 6. Returns a typed `ApiResponse` with the result
//!
//! ## SharedBuffer Pattern
//!
//! The `Output` struct from the CLI crate takes ownership of the writer and doesn't
//! expose an `into_inner()` method. To work around this limitation, we use a
//! `SharedBuffer` that wraps `Arc<Mutex<Vec<u8>>>`:
//!
//! ```text
//! SharedBuffer(Arc<Mutex<Vec<u8>>>)
//!        │
//!        ├── Clone → Output::new(..., SharedBuffer, ...)
//!        │                    │
//!        │                    ▼
//!        │              write() calls
//!        │                    │
//!        │                    ▼
//!        │              Arc<Mutex<Vec<u8>>> (shared)
//!        │
//!        └── After execution: extract bytes from Arc
//! ```
//!
//! # Why
//!
//! Version bumping is the culmination of the changeset workflow. These commands
//! provide fine-grained control over how versions are updated, including dry-run
//! capabilities and git integration for automated releases.
//!
//! Key use cases:
//! - Previewing version changes before applying them (CI/CD validation)
//! - Applying version bumps with optional Git integration
//! - Generating snapshot versions for pre-release testing
//! - Supporting prerelease workflows (alpha, beta, rc)
//!
//! # Examples
//!
//! ## TypeScript Usage
//!
//! ```typescript
//! import { bumpPreview, bumpApply, bumpSnapshot } from '@websublime/workspace-tools';
//!
//! // Preview version bumps (dry run)
//! const previewResult = await bumpPreview({
//!   root: '.',
//!   showDiff: true
//! });
//! if (previewResult.success) {
//!   for (const pkg of previewResult.data.packages) {
//!     console.log(`${pkg.name}: ${pkg.currentVersion} -> ${pkg.nextVersion} (${pkg.bump})`);
//!   }
//!   console.log(`\nTotal: ${previewResult.data.summary.totalPackages} packages`);
//! }
//!
//! // Apply version bumps with git commit and tags
//! const applyResult = await bumpApply({
//!   root: '.',
//!   gitCommit: true,
//!   gitTag: true,
//!   gitPush: false  // Don't push automatically
//! });
//! if (applyResult.success) {
//!   console.log(`Applied ${applyResult.data.packagesUpdated} version bumps`);
//!   console.log(`Git commit: ${applyResult.data.commitSha}`);
//!   console.log(`Git tags: ${applyResult.data.tagsCreated.join(', ')}`);
//! }
//!
//! // Generate snapshot versions for pre-release
//! const snapshotResult = await bumpSnapshot({
//!   root: '.',
//!   format: '{version}-snapshot.{timestamp}'
//! });
//! if (snapshotResult.success) {
//!   for (const pkg of snapshotResult.data.packages) {
//!     console.log(`${pkg.name}: ${pkg.snapshotVersion}`);
//!   }
//! }
//! ```
//!
//! ## Error Handling
//!
//! ```typescript
//! const result = await bumpPreview({ root: '/nonexistent/path' });
//!
//! if (!result.success) {
//!   switch (result.error.code) {
//!     case 'ENOENT':
//!       console.error('Path not found:', result.error.message);
//!       break;
//!     case 'EVALIDATION':
//!       console.error('Invalid parameters:', result.error.message);
//!       break;
//!     default:
//!       console.error('Unexpected error:', result.error.message);
//!   }
//! }
//! ```

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use napi_derive::napi;
use serde::Deserialize;

use crate::error::ErrorInfo;
use crate::types::bump::{
    BumpApplyApiResponse, BumpApplyData, BumpApplyParams, BumpPreviewApiResponse, BumpPreviewData,
    BumpPreviewParams, BumpSnapshotApiResponse, BumpSnapshotData, BumpSnapshotParams,
    BumpSummaryInfo, PackageVersionInfo, SnapshotVersionInfo,
};
use crate::validation::validators;

use sublime_cli_tools::cli::commands::BumpArgs;
use sublime_cli_tools::commands::bump::{
    execute_bump_apply, execute_bump_preview, execute_bump_snapshot,
};
use sublime_cli_tools::output::{Output, OutputFormat};

// ============================================================================
// SharedBuffer - Output Capture Mechanism
// ============================================================================

/// A shared buffer wrapper for capturing CLI output.
///
/// The `Output` struct from `sublime_cli_tools` takes ownership of the writer
/// and doesn't expose an `into_inner()` method. This wrapper uses `Arc<Mutex<Vec<u8>>>`
/// to allow sharing the buffer between the caller and the Output, enabling
/// extraction of the written bytes after command execution.
///
/// # Thread Safety
///
/// The buffer is protected by a `Mutex` to ensure safe concurrent access,
/// although in typical usage the writes happen sequentially.
///
/// # Examples
///
/// ```rust,ignore
/// let buffer = SharedBuffer::new();
/// let output = Output::new(OutputFormat::Json, buffer.clone(), true);
///
/// // ... execute command that writes to output ...
///
/// let bytes = buffer.take_bytes();
/// ```
#[derive(Debug, Clone)]
pub(crate) struct SharedBuffer {
    /// The inner buffer wrapped in `Arc<Mutex>` for shared ownership.
    inner: Arc<Mutex<Vec<u8>>>,
}

impl SharedBuffer {
    /// Creates a new empty shared buffer.
    ///
    /// # Returns
    ///
    /// A new `SharedBuffer` instance ready for use with `Output::new()`.
    pub(crate) fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Extracts the bytes written to the buffer.
    ///
    /// This method clones the current buffer contents. The original buffer
    /// remains intact for potential further writes.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing all bytes written to the buffer.
    ///
    /// # Panics
    ///
    /// This method will return an empty Vec if the mutex is poisoned,
    /// rather than panicking, to maintain robustness.
    pub(crate) fn take_bytes(&self) -> Vec<u8> {
        match self.inner.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => {
                // If the mutex is poisoned, still try to get the data
                // This provides graceful degradation
                poisoned.into_inner().clone()
            }
        }
    }
}

impl Write for SharedBuffer {
    /// Writes data to the shared buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - The byte slice to write
    ///
    /// # Returns
    ///
    /// The number of bytes written, or an I/O error if the mutex is poisoned.
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.inner.lock() {
            Ok(mut guard) => guard.write(buf),
            Err(_) => Err(std::io::Error::other("Mutex poisoned")),
        }
    }

    /// Flushes the buffer.
    ///
    /// Since `Vec<u8>` is an in-memory buffer, this is a no-op.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// Required for `Output::new()` which expects `Write + Send`
// SAFETY: SharedBuffer uses Arc<Mutex<_>> which is Send + Sync
unsafe impl Send for SharedBuffer {}

// ============================================================================
// CLI Response Types (for parsing JSON output)
// ============================================================================

/// CLI JSON response wrapper for bump preview command.
///
/// This type mirrors the `JsonResponse<T>` structure from the CLI crate,
/// used for deserializing the captured JSON output.
#[derive(Debug, Deserialize)]
pub(crate) struct CliJsonResponse<T> {
    /// Whether the operation succeeded.
    pub(crate) success: bool,
    /// The response data (present when success is true).
    pub(crate) data: Option<T>,
    /// Error message (present when success is false).
    pub(crate) error: Option<String>,
}

/// CLI bump snapshot data structure.
///
/// Mirrors the `BumpSnapshot` structure from the CLI's bump command.
/// Field names use camelCase to match the JSON output format.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliBumpSnapshot {
    /// Versioning strategy being used (Independent or Unified).
    pub(crate) strategy: String,
    /// List of all workspace packages with their bump information.
    pub(crate) packages: Vec<CliPackageBumpInfo>,
    /// List of changesets being processed in this bump.
    pub(crate) changesets: Vec<CliChangesetInfo>,
    /// Summary statistics for the bump operation.
    #[allow(dead_code)]
    pub(crate) summary: CliBumpSummary,
}

/// CLI package bump information.
///
/// Mirrors the `PackageBumpInfo` structure from the CLI's bump command.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliPackageBumpInfo {
    /// Package name (e.g., "@org/core").
    pub(crate) name: String,
    /// Relative path to package directory.
    pub(crate) path: String,
    /// Current version (e.g., "1.2.3").
    pub(crate) current_version: String,
    /// Next version after bump (e.g., "1.3.0").
    pub(crate) next_version: String,
    /// Type of version bump (Major, Minor, Patch, None).
    pub(crate) bump_type: String,
    /// Whether this package will actually be bumped.
    pub(crate) will_bump: bool,
    /// Human-readable reason for bump or no-bump.
    #[allow(dead_code)]
    pub(crate) reason: String,
}

/// CLI changeset information.
///
/// Mirrors the `ChangesetInfo` structure from the CLI's bump command.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliChangesetInfo {
    /// Changeset ID.
    pub(crate) id: String,
    /// Git branch name.
    #[allow(dead_code)]
    pub(crate) branch: String,
    /// Bump type for this changeset.
    #[allow(dead_code)]
    pub(crate) bump_type: String,
    /// List of packages affected by this changeset.
    #[allow(dead_code)]
    pub(crate) packages: Vec<String>,
    /// Number of commits in this changeset.
    #[allow(dead_code)]
    pub(crate) commit_count: usize,
}

/// CLI bump summary information.
///
/// Mirrors the `BumpSummary` structure from the CLI's bump command.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliBumpSummary {
    /// Total number of packages in workspace.
    #[allow(dead_code)]
    pub(crate) total_packages: usize,
    /// Number of packages that will be bumped.
    #[allow(dead_code)]
    pub(crate) packages_to_bump: usize,
    /// Number of packages that won't be bumped.
    #[allow(dead_code)]
    pub(crate) packages_unchanged: usize,
    /// Total number of changesets being processed.
    #[allow(dead_code)]
    pub(crate) total_changesets: usize,
    /// Whether circular dependencies were detected.
    #[allow(dead_code)]
    pub(crate) has_circular_dependencies: bool,
}

// ============================================================================
// CLI Response Types for bump apply (execute result)
// ============================================================================

/// CLI JSON response wrapper for bump apply (execute) command.
///
/// This type mirrors the `JsonResponse<ExecuteResult>` structure from the CLI crate,
/// used for deserializing the captured JSON output from `execute_bump_apply`.
///
/// Note: The CLI's `ExecuteResult` uses snake_case for field names (no `rename_all`
/// attribute), so we must use snake_case here to match the JSON output.
#[derive(Debug, Deserialize)]
pub(crate) struct CliApplyJsonResponse {
    /// Whether the operation succeeded.
    pub(crate) success: bool,
    /// The response data (present when success is true).
    pub(crate) data: Option<CliExecuteResult>,
    /// Error message (present when success is false).
    pub(crate) error: Option<String>,
}

/// CLI execute result data structure.
///
/// Mirrors the `ExecuteResult` structure from the CLI's bump execute command.
/// Note: The CLI uses snake_case for JSON serialization (no `rename_all` attribute).
#[derive(Debug, Deserialize)]
pub(crate) struct CliExecuteResult {
    /// Versioning strategy used (Independent or Unified).
    pub(crate) strategy: String,
    /// Number of packages that were updated.
    pub(crate) packages_updated: usize,
    /// Number of changesets that were archived.
    pub(crate) changesets_archived: usize,
    /// List of files that were modified (as PathBuf serialized strings).
    pub(crate) files_modified: Vec<String>,
    /// List of Git tags that were created.
    pub(crate) tags_created: Vec<String>,
    /// Git commit SHA (if commit was created).
    pub(crate) commit_sha: Option<String>,
    /// Full snapshot of the bump operation (not used in NAPI response, but required for parsing).
    #[allow(dead_code)]
    pub(crate) snapshot: CliBumpSnapshot,
}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Converts a bump type string to the NAPI-compatible format.
///
/// The CLI uses enum variants like "Major", "Minor", "Patch", "None",
/// while the NAPI interface uses lowercase strings.
///
/// # Arguments
///
/// * `bump_type` - The bump type string from CLI (e.g., "Major", "Minor")
///
/// # Returns
///
/// A lowercase bump type string (e.g., "major", "minor", "patch", "none").
fn normalize_bump_type(bump_type: &str) -> String {
    bump_type.to_lowercase()
}

/// Converts CLI bump snapshot data to NAPI-compatible `BumpPreviewData`.
///
/// This function performs a conversion from the CLI's internal types to the
/// NAPI types exposed to JavaScript, filtering to only include packages
/// that will actually be bumped.
///
/// # Arguments
///
/// * `cli_data` - The parsed CLI response data
///
/// # Returns
///
/// A `BumpPreviewData` instance suitable for returning to JavaScript.
pub(crate) fn convert_to_napi_preview(cli_data: CliBumpSnapshot) -> BumpPreviewData {
    // Filter to only packages that will be bumped and convert to NAPI types
    let packages: Vec<PackageVersionInfo> = cli_data
        .packages
        .into_iter()
        .filter(|p| p.will_bump)
        .map(|p| PackageVersionInfo {
            name: p.name,
            path: p.path,
            current_version: p.current_version,
            next_version: p.next_version,
            bump: normalize_bump_type(&p.bump_type),
            dependency_updates: Vec::new(), // Dependency updates not available in preview
        })
        .collect();

    // Calculate summary from filtered packages
    let summary = calculate_summary_from_packages(&packages);

    // Extract just the changeset IDs
    let changesets: Vec<String> = cli_data.changesets.into_iter().map(|c| c.id).collect();

    BumpPreviewData { strategy: cli_data.strategy.to_lowercase(), packages, summary, changesets }
}

/// Calculates bump summary statistics from a list of packages.
///
/// # Arguments
///
/// * `packages` - The list of package version info
///
/// # Returns
///
/// A `BumpSummaryInfo` with counts of each bump type.
#[allow(clippy::cast_possible_truncation)]
// Justification: It's practically impossible to have more than 4 billion packages in a
// workspace. The u32 limit of ~4.29 billion packages is sufficient for any real-world
// monorepo. This truncation would only occur in an unrealistic edge case.
fn calculate_summary_from_packages(packages: &[PackageVersionInfo]) -> BumpSummaryInfo {
    let total_packages = packages.len() as u32;
    let major_bumps = packages.iter().filter(|p| p.bump == "major").count() as u32;
    let minor_bumps = packages.iter().filter(|p| p.bump == "minor").count() as u32;
    let patch_bumps = packages.iter().filter(|p| p.bump == "patch").count() as u32;

    BumpSummaryInfo { total_packages, major_bumps, minor_bumps, patch_bumps }
}

/// Parses the JSON response from the CLI and converts it to NAPI types.
///
/// # Arguments
///
/// * `json_bytes` - The raw JSON bytes captured from CLI output
///
/// # Returns
///
/// * `Ok(BumpPreviewData)` - Successfully parsed and converted preview data
/// * `Err(ErrorInfo)` - Parsing failed or CLI returned an error
///
/// # Errors
///
/// Returns an error if:
/// - The JSON is malformed or cannot be parsed
/// - The CLI returned `success: false` with an error message
/// - The CLI returned `success: true` but `data` is missing
pub(crate) fn parse_preview_response(json_bytes: &[u8]) -> Result<BumpPreviewData, ErrorInfo> {
    // Convert bytes to string first for better error messages
    let json_str = std::str::from_utf8(json_bytes)
        .map_err(|e| ErrorInfo::execution(format!("Invalid UTF-8 in CLI response: {e}")))?;

    // Handle empty response
    if json_str.trim().is_empty() {
        return Err(ErrorInfo::execution("CLI returned empty response"));
    }

    // Parse the JSON response
    let response: CliJsonResponse<CliBumpSnapshot> =
        serde_json::from_str(json_str).map_err(|e| {
            ErrorInfo::execution(format!(
                "Failed to parse CLI JSON response: {e} (length={})",
                json_str.len()
            ))
        })?;

    // Check for CLI-level errors
    if !response.success {
        let error_message = response.error.unwrap_or_else(|| "Unknown CLI error".to_string());
        return Err(ErrorInfo::execution(error_message));
    }

    // Extract and convert data
    let cli_data =
        response.data.ok_or_else(|| ErrorInfo::execution("CLI returned success but no data"))?;

    Ok(convert_to_napi_preview(cli_data))
}

// ============================================================================
// Conversion Functions for bump apply
// ============================================================================

/// Converts CLI execute result to NAPI-compatible `BumpApplyData`.
///
/// This function performs a conversion from the CLI's internal types to the
/// NAPI types exposed to JavaScript.
///
/// # Arguments
///
/// * `cli_data` - The parsed CLI execute result
///
/// # Returns
///
/// A `BumpApplyData` instance suitable for returning to JavaScript.
#[allow(clippy::cast_possible_truncation)]
// Justification: It's practically impossible to have more than 4 billion packages
// or changesets in a workspace. The u32 limit is sufficient for any real-world scenario.
pub(crate) fn convert_to_napi_apply(cli_data: CliExecuteResult) -> BumpApplyData {
    BumpApplyData {
        strategy: cli_data.strategy.to_lowercase(),
        packages_updated: cli_data.packages_updated as u32,
        changesets_archived: cli_data.changesets_archived as u32,
        files_modified: cli_data.files_modified,
        tags_created: cli_data.tags_created,
        commit_sha: cli_data.commit_sha,
    }
}

/// Converts CLI bump snapshot data to NAPI-compatible `BumpSnapshotData`.
///
/// This function performs a conversion from the CLI's internal types to the
/// NAPI types exposed to JavaScript. For snapshot mode, the `next_version`
/// field contains the generated snapshot version.
///
/// # Arguments
///
/// * `cli_data` - The parsed CLI response data
/// * `format` - The snapshot format template that was used
///
/// # Returns
///
/// A `BumpSnapshotData` instance suitable for returning to JavaScript.
///
/// # Conversion Details
///
/// - `SnapshotVersionInfo.original_version` = `PackageBumpInfo.current_version`
/// - `SnapshotVersionInfo.snapshot_version` = `PackageBumpInfo.next_version`
/// - Only packages where `will_bump` is true are included
pub(crate) fn convert_to_napi_snapshot(
    cli_data: CliBumpSnapshot,
    format: String,
) -> BumpSnapshotData {
    // Convert packages that will be bumped to snapshot version info
    let packages: Vec<SnapshotVersionInfo> = cli_data
        .packages
        .into_iter()
        .filter(|p| p.will_bump)
        .map(|p| SnapshotVersionInfo {
            name: p.name,
            path: p.path,
            original_version: p.current_version,
            snapshot_version: p.next_version,
        })
        .collect();

    BumpSnapshotData { strategy: cli_data.strategy.to_lowercase(), packages, format }
}

/// Parses the JSON response from the CLI apply command and converts it to NAPI types.
///
/// # Arguments
///
/// * `json_bytes` - The raw JSON bytes captured from CLI output
///
/// # Returns
///
/// * `Ok(BumpApplyData)` - Successfully parsed and converted apply data
/// * `Err(ErrorInfo)` - Parsing failed or CLI returned an error
///
/// # Errors
///
/// Returns an error if:
/// - The JSON is malformed or cannot be parsed
/// - The CLI returned `success: false` with an error message
/// - The CLI returned `success: true` but `data` is missing
pub(crate) fn parse_apply_response(json_bytes: &[u8]) -> Result<BumpApplyData, ErrorInfo> {
    // Convert bytes to string first for better error messages
    let json_str = std::str::from_utf8(json_bytes)
        .map_err(|e| ErrorInfo::execution(format!("Invalid UTF-8 in CLI response: {e}")))?;

    // Handle empty response
    if json_str.trim().is_empty() {
        return Err(ErrorInfo::execution("CLI returned empty response"));
    }

    // Parse the JSON response
    let response: CliApplyJsonResponse = serde_json::from_str(json_str).map_err(|e| {
        ErrorInfo::execution(format!(
            "Failed to parse CLI JSON response: {e} (length={})",
            json_str.len()
        ))
    })?;

    // Check for CLI-level errors
    if !response.success {
        let error_message = response.error.unwrap_or_else(|| "Unknown CLI error".to_string());
        return Err(ErrorInfo::execution(error_message));
    }

    // Extract and convert data
    let cli_data =
        response.data.ok_or_else(|| ErrorInfo::execution("CLI returned success but no data"))?;

    Ok(convert_to_napi_apply(cli_data))
}

/// Parses the JSON response from the CLI snapshot command and converts it to NAPI types.
///
/// The snapshot command uses the same `BumpSnapshot` response structure as preview,
/// but the `next_version` field contains the generated snapshot version instead
/// of the bumped version.
///
/// # Arguments
///
/// * `json_bytes` - The raw JSON bytes captured from CLI output
/// * `format` - The snapshot format template that was used
///
/// # Returns
///
/// * `Ok(BumpSnapshotData)` - Successfully parsed and converted snapshot data
/// * `Err(ErrorInfo)` - Parsing failed or CLI returned an error
///
/// # Errors
///
/// Returns an error if:
/// - The JSON is malformed or cannot be parsed
/// - The CLI returned `success: false` with an error message
/// - The CLI returned `success: true` but `data` is missing
pub(crate) fn parse_snapshot_response(
    json_bytes: &[u8],
    format: String,
) -> Result<BumpSnapshotData, ErrorInfo> {
    // Convert bytes to string first for better error messages
    let json_str = std::str::from_utf8(json_bytes)
        .map_err(|e| ErrorInfo::execution(format!("Invalid UTF-8 in CLI response: {e}")))?;

    // Handle empty response
    if json_str.trim().is_empty() {
        return Err(ErrorInfo::execution("CLI returned empty response"));
    }

    // Parse the JSON response (same structure as preview)
    let response: CliJsonResponse<CliBumpSnapshot> =
        serde_json::from_str(json_str).map_err(|e| {
            ErrorInfo::execution(format!(
                "Failed to parse CLI JSON response: {e} (length={})",
                json_str.len()
            ))
        })?;

    // Check for CLI-level errors
    if !response.success {
        let error_message = response.error.unwrap_or_else(|| "Unknown CLI error".to_string());
        return Err(ErrorInfo::execution(error_message));
    }

    // Extract and convert data
    let cli_data =
        response.data.ok_or_else(|| ErrorInfo::execution("CLI returned success but no data"))?;

    Ok(convert_to_napi_snapshot(cli_data, format))
}

// ============================================================================
// Parameter Validation
// ============================================================================

/// Validates bump preview command parameters.
///
/// Ensures the root path is valid before executing the CLI command.
///
/// # Arguments
///
/// * `params` - The preview parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path
/// * `Err(ErrorInfo)` - Validation failed
pub(crate) fn validate_preview_params(params: &BumpPreviewParams) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    Ok(PathBuf::from(&params.root))
}

/// Validates bump apply command parameters.
///
/// Ensures the root path is valid and validates the prerelease tag if provided.
///
/// # Arguments
///
/// * `params` - The apply parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path
/// * `Err(ErrorInfo)` - Validation failed
pub(crate) fn validate_apply_params(params: &BumpApplyParams) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    // Validate prerelease tag if provided
    if let Some(ref tag) = params.prerelease {
        validators::prerelease_tag(tag)?;
    }

    Ok(PathBuf::from(&params.root))
}

/// Validates bump snapshot command parameters.
///
/// Ensures the root path is valid and validates the snapshot format if provided.
/// The snapshot format must contain at least one valid template variable.
///
/// # Arguments
///
/// * `params` - The snapshot parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path
/// * `Err(ErrorInfo)` - Validation failed
///
/// # Errors
///
/// Returns an error if:
/// - The root path does not exist or is not a directory
/// - The snapshot format is provided but contains no valid template variables
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_node_tools::commands::bump::validate_snapshot_params;
/// use sublime_node_tools::types::bump::BumpSnapshotParams;
///
/// let params = BumpSnapshotParams::new("/path/to/workspace")
///     .with_format("{version}-snapshot.{short_commit}");
///
/// let result = validate_snapshot_params(&params);
/// assert!(result.is_ok());
/// ```
pub(crate) fn validate_snapshot_params(params: &BumpSnapshotParams) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    // Validate snapshot format if provided
    if let Some(ref format) = params.format {
        validators::snapshot_format(format)?;
    }

    Ok(PathBuf::from(&params.root))
}

/// Converts `BumpPreviewParams` to CLI `BumpArgs`.
///
/// This function sets the appropriate flags for preview mode (dry_run = true)
/// and maps the NAPI parameters to CLI arguments.
///
/// # Arguments
///
/// * `params` - The NAPI preview parameters
///
/// # Returns
///
/// A `BumpArgs` struct configured for preview mode.
pub(crate) fn convert_params_to_args(params: &BumpPreviewParams) -> BumpArgs {
    BumpArgs {
        // Preview mode: dry_run = true, no execution
        dry_run: true,
        execute: false,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,

        // Package filter from params
        packages: params.packages.clone(),

        // No git operations in preview mode
        git_tag: false,
        git_push: false,
        git_commit: false,

        // No changelog/archive changes in preview
        no_changelog: true,
        no_archive: true,
        always_archive: false,

        // Skip confirmations (API is non-interactive)
        force: true,

        // Show diff from params
        show_diff: params.show_diff.unwrap_or(false),
    }
}

/// Converts `BumpApplyParams` to CLI `BumpArgs`.
///
/// This function sets the appropriate flags for execute mode (execute = true)
/// and maps all NAPI parameters to CLI arguments including git operations,
/// prerelease support, and changelog/archive control.
///
/// # Arguments
///
/// * `params` - The NAPI apply parameters
///
/// # Returns
///
/// A `BumpArgs` struct configured for execute mode.
pub(crate) fn convert_apply_params_to_args(params: &BumpApplyParams) -> BumpArgs {
    BumpArgs {
        // Execute mode: execute = true, no dry run
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,

        // Prerelease support
        prerelease: params.prerelease.clone(),

        // Package filter from params
        packages: params.packages.clone(),

        // Git operations from params
        git_tag: params.git_tag.unwrap_or(false),
        git_push: params.git_push.unwrap_or(false),
        git_commit: params.git_commit.unwrap_or(false),

        // Changelog and archive control
        no_changelog: params.no_changelog.unwrap_or(false),
        no_archive: params.no_archive.unwrap_or(false),
        always_archive: params.always_archive.unwrap_or(false),

        // Skip confirmations (API is non-interactive by default)
        force: params.force.unwrap_or(true),

        // No diff display in execute mode
        show_diff: false,
    }
}

/// Default snapshot format template.
///
/// This is used when no custom format is provided by the user or configuration.
const DEFAULT_SNAPSHOT_FORMAT: &str = "{version}-snapshot.{short_commit}";

/// Converts `BumpSnapshotParams` to CLI `BumpArgs`.
///
/// This function sets the appropriate flags for snapshot mode (snapshot = true)
/// and maps the NAPI parameters to CLI arguments. Snapshot mode generates
/// temporary pre-release versions for testing without consuming changesets.
///
/// # Arguments
///
/// * `params` - The NAPI snapshot parameters
///
/// # Returns
///
/// A `BumpArgs` struct configured for snapshot mode.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_node_tools::commands::bump::convert_snapshot_params_to_args;
/// use sublime_node_tools::types::bump::BumpSnapshotParams;
///
/// let params = BumpSnapshotParams::new(".")
///     .with_format("{version}-{branch}.{short_commit}");
///
/// let args = convert_snapshot_params_to_args(&params);
/// assert!(args.snapshot);
/// assert_eq!(args.snapshot_format, Some("{version}-{branch}.{short_commit}".to_string()));
/// ```
pub(crate) fn convert_snapshot_params_to_args(params: &BumpSnapshotParams) -> BumpArgs {
    BumpArgs {
        // Snapshot mode: snapshot = true, no dry run, no execute
        dry_run: false,
        execute: false,
        snapshot: true,
        snapshot_format: params.format.clone(),

        // No prerelease in snapshot mode (they are mutually exclusive)
        prerelease: None,

        // Package filter from params
        packages: params.packages.clone(),

        // No git operations in snapshot mode
        git_tag: false,
        git_push: false,
        git_commit: false,

        // No changelog generation in snapshot mode
        no_changelog: true,

        // Changesets are NOT archived in snapshot mode
        no_archive: true,
        always_archive: false,

        // Skip confirmations (API is non-interactive)
        force: true,

        // No diff display in snapshot mode
        show_diff: false,
    }
}

// ============================================================================
// NAPI Functions
// ============================================================================

/// Preview version bumps without applying changes.
///
/// Returns comprehensive information about what versions would change based on
/// pending changesets. This is a dry-run operation that does not modify any files.
///
/// This function is the main entry point for Node.js applications to preview
/// version bumps. It handles all the complexity of CLI invocation and response
/// parsing internally.
///
/// @param params - Preview parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `configPath`: Optional custom config file path
///   - `packages`: Optional filter to specific packages
///   - `showDiff`: Whether to show detailed version diffs
///
/// @returns `Promise<ApiResponse<BumpPreviewData>>` containing:
///   - On success: `{ success: true, data: BumpPreviewData }`
///   - On failure: `{ success: false, error: ErrorInfo }`
///
/// @example Basic usage
/// ```typescript
/// const result = await bumpPreview({ root: '/path/to/project' });
/// if (result.success) {
///   console.log(`Strategy: ${result.data.strategy}`);
///   console.log(`Packages to bump: ${result.data.packages.length}`);
///   for (const pkg of result.data.packages) {
///     console.log(`  ${pkg.name}: ${pkg.currentVersion} -> ${pkg.nextVersion}`);
///   }
/// } else {
///   console.error(`Error: ${result.error.code} - ${result.error.message}`);
/// }
/// ```
///
/// @example With package filter and diff
/// ```typescript
/// const result = await bumpPreview({
///   root: '/path/to/project',
///   packages: ['@scope/core', '@scope/utils'],
///   showDiff: true
/// });
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await bumpPreview({ root: '/nonexistent' });
/// if (!result.success) {
///   if (result.error.code === 'ENOENT') {
///     console.error('Path not found');
///   } else if (result.error.code === 'EVALIDATION') {
///     console.error('Invalid parameters');
///   }
/// }
/// ```
#[napi(js_name = "bumpPreview")]
pub async fn bump_preview(params: BumpPreviewParams) -> BumpPreviewApiResponse {
    // 1. Validate parameters (synchronous validation before spawning)
    let root_path = match validate_preview_params(&params) {
        Ok(path) => path,
        Err(error) => return BumpPreviewApiResponse::failure(error),
    };

    // 2. Prepare config path
    let config_path: Option<PathBuf> = params.config_path.as_ref().map(PathBuf::from);

    // 3. Convert params to CLI args
    let args = convert_params_to_args(&params);

    // 4. Execute CLI command in a blocking task
    // The CLI's execute_bump_preview uses types that are not Send/Sync (RefCell, git2::Repository),
    // so we must run it on a blocking thread via spawn_blocking.
    let result = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        // This is necessary because execute_bump_preview is async but we're in a blocking context
        let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(e) => {
                return Err(ErrorInfo::execution(format!("Failed to create runtime: {e}")));
            }
        };

        rt.block_on(async {
            // Create shared buffer for output capture
            let buffer = SharedBuffer::new();

            // Create Output with JSON format
            let output = Output::new(OutputFormat::Json, buffer.clone(), true);

            // Execute the CLI command
            let config_path_ref: Option<&Path> = config_path.as_deref();
            if let Err(cli_error) =
                execute_bump_preview(&args, &output, &root_path, config_path_ref).await
            {
                return Err(ErrorInfo::from(cli_error));
            }

            // Extract and parse JSON
            let json_bytes = buffer.take_bytes();
            parse_preview_response(&json_bytes)
        })
    })
    .await;

    // 5. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => BumpPreviewApiResponse::success(data),
        Ok(Err(error)) => BumpPreviewApiResponse::failure(error),
        Err(join_error) => BumpPreviewApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}

/// Apply version bumps to packages.
///
/// Applies version changes based on pending changesets. This is the main
/// release command that modifies package.json files, generates changelogs,
/// archives changesets, and optionally creates Git commits and tags.
///
/// This function is the main entry point for Node.js applications to apply
/// version bumps. It handles all the complexity of CLI invocation and response
/// parsing internally.
///
/// @param params - Apply parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `configPath`: Optional custom config file path
///   - `packages`: Optional filter to specific packages
///   - `gitCommit`: Whether to create a Git commit with version changes
///   - `gitTag`: Whether to create Git tags for releases
///   - `gitPush`: Whether to push Git tags to remote
///   - `prerelease`: Prerelease tag (alpha, beta, rc, or custom)
///   - `noChangelog`: Whether to skip changelog generation
///   - `noArchive`: Whether to keep changesets active after bump
///   - `alwaysArchive`: Whether to force archiving for prereleases
///   - `force`: Whether to skip confirmation prompts (default: true)
///
/// @returns `Promise<ApiResponse<BumpApplyData>>` containing:
///   - On success: `{ success: true, data: BumpApplyData }`
///   - On failure: `{ success: false, error: ErrorInfo }`
///
/// @example Basic usage - apply bumps without Git operations
/// ```typescript
/// const result = await bumpApply({ root: '/path/to/project' });
/// if (result.success) {
///   console.log(`Updated ${result.data.packagesUpdated} packages`);
///   console.log(`Archived ${result.data.changesetsArchived} changesets`);
///   console.log(`Modified files: ${result.data.filesModified.join(', ')}`);
/// } else {
///   console.error(`Error: ${result.error.code} - ${result.error.message}`);
/// }
/// ```
///
/// @example With Git operations
/// ```typescript
/// const result = await bumpApply({
///   root: '/path/to/project',
///   gitCommit: true,
///   gitTag: true,
///   gitPush: true
/// });
/// if (result.success) {
///   console.log(`Commit SHA: ${result.data.commitSha}`);
///   console.log(`Tags created: ${result.data.tagsCreated.join(', ')}`);
/// }
/// ```
///
/// @example Prerelease version (beta)
/// ```typescript
/// const result = await bumpApply({
///   root: '/path/to/project',
///   prerelease: 'beta',
///   gitCommit: true,
///   gitTag: true
/// });
/// // Creates versions like 1.3.0-beta.0
/// ```
///
/// @example Skip changelog and archive
/// ```typescript
/// const result = await bumpApply({
///   root: '/path/to/project',
///   noChangelog: true,
///   noArchive: true
/// });
/// // Updates versions but keeps changesets and skips changelog
/// ```
///
/// @example Force archive for prerelease
/// ```typescript
/// const result = await bumpApply({
///   root: '/path/to/project',
///   prerelease: 'rc',
///   alwaysArchive: true,  // Archive changesets even for prerelease
///   gitCommit: true,
///   gitTag: true
/// });
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await bumpApply({ root: '/nonexistent' });
/// if (!result.success) {
///   if (result.error.code === 'ENOENT') {
///     console.error('Path not found');
///   } else if (result.error.code === 'EVALIDATION') {
///     console.error('Invalid parameters:', result.error.message);
///   } else if (result.error.code === 'EGIT') {
///     console.error('Git operation failed:', result.error.message);
///   }
/// }
/// ```
#[napi(js_name = "bumpApply")]
pub async fn bump_apply(params: BumpApplyParams) -> BumpApplyApiResponse {
    // 1. Validate parameters (synchronous validation before spawning)
    let root_path = match validate_apply_params(&params) {
        Ok(path) => path,
        Err(error) => return BumpApplyApiResponse::failure(error),
    };

    // 2. Prepare config path
    let config_path: Option<PathBuf> = params.config_path.as_ref().map(PathBuf::from);

    // 3. Convert params to CLI args
    let args = convert_apply_params_to_args(&params);

    // 4. Execute CLI command in a blocking task
    // The CLI's execute_bump_apply uses types that are not Send/Sync (RefCell, git2::Repository),
    // so we must run it on a blocking thread via spawn_blocking.
    let result = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        // This is necessary because execute_bump_apply is async but we're in a blocking context
        let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(e) => {
                return Err(ErrorInfo::execution(format!("Failed to create runtime: {e}")));
            }
        };

        rt.block_on(async {
            // Create shared buffer for output capture
            let buffer = SharedBuffer::new();

            // Create Output with JSON format
            let output = Output::new(OutputFormat::Json, buffer.clone(), true);

            // Execute the CLI command
            let config_path_ref: Option<&Path> = config_path.as_deref();
            if let Err(cli_error) =
                execute_bump_apply(&args, &output, &root_path, config_path_ref).await
            {
                return Err(ErrorInfo::from(cli_error));
            }

            // Extract and parse JSON
            let json_bytes = buffer.take_bytes();
            parse_apply_response(&json_bytes)
        })
    })
    .await;

    // 5. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => BumpApplyApiResponse::success(data),
        Ok(Err(error)) => BumpApplyApiResponse::failure(error),
        Err(join_error) => BumpApplyApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}

/// Generate snapshot versions for testing.
///
/// Creates temporary, non-persisted versions for branch builds and preview
/// deployments. Snapshot versions are NOT SemVer compliant and are intended
/// for testing purposes only.
///
/// **Key characteristics:**
/// - Does NOT consume or archive changesets
/// - Does NOT create Git commits or tags
/// - Does NOT generate changelogs
/// - Uses format templates with variables like `{version}`, `{branch}`, `{short_commit}`
///
/// This function is the main entry point for Node.js applications to generate
/// snapshot versions. It handles all the complexity of CLI invocation and response
/// parsing internally.
///
/// @param params - Snapshot parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `configPath`: Optional custom config file path
///   - `packages`: Optional filter to specific packages
///   - `format`: Snapshot format template (default: `{version}-snapshot.{short_commit}`)
///
/// @returns `Promise<ApiResponse<BumpSnapshotData>>` containing:
///   - On success: `{ success: true, data: BumpSnapshotData }`
///   - On failure: `{ success: false, error: ErrorInfo }`
///
/// @example Basic usage with default format
/// ```typescript
/// const result = await bumpSnapshot({ root: '/path/to/project' });
/// if (result.success) {
///   console.log(`Format used: ${result.data.format}`);
///   for (const pkg of result.data.packages) {
///     console.log(`${pkg.name}: ${pkg.originalVersion} -> ${pkg.snapshotVersion}`);
///   }
/// } else {
///   console.error(`Error: ${result.error.code} - ${result.error.message}`);
/// }
/// ```
///
/// @example Custom format with branch and commit
/// ```typescript
/// const result = await bumpSnapshot({
///   root: '/path/to/project',
///   format: '{version}-{branch}.{short_commit}'
/// });
/// // Generates versions like: 1.2.3-feature-x.abc123f
/// ```
///
/// @example Timestamp-based format
/// ```typescript
/// const result = await bumpSnapshot({
///   root: '/path/to/project',
///   format: '{version}-dev.{timestamp}'
/// });
/// // Generates versions like: 1.2.3-dev.1699876543
/// ```
///
/// @example Filter to specific packages
/// ```typescript
/// const result = await bumpSnapshot({
///   root: '/path/to/project',
///   packages: ['@scope/core', '@scope/utils'],
///   format: '{version}-snapshot.{short_commit}'
/// });
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await bumpSnapshot({
///   root: '/path/to/project',
///   format: 'invalid-no-variables'
/// });
/// if (!result.success) {
///   if (result.error.code === 'EVALIDATION') {
///     console.error('Invalid format:', result.error.message);
///   }
/// }
/// ```
#[napi(js_name = "bumpSnapshot")]
pub async fn bump_snapshot(params: BumpSnapshotParams) -> BumpSnapshotApiResponse {
    // 1. Validate parameters (synchronous validation before spawning)
    let root_path = match validate_snapshot_params(&params) {
        Ok(path) => path,
        Err(error) => return BumpSnapshotApiResponse::failure(error),
    };

    // 2. Prepare config path
    let config_path: Option<PathBuf> = params.config_path.as_ref().map(PathBuf::from);

    // 3. Determine the format to use (user-provided or default)
    let format_used = params.format.clone().unwrap_or_else(|| DEFAULT_SNAPSHOT_FORMAT.to_string());

    // 4. Convert params to CLI args
    let args = convert_snapshot_params_to_args(&params);

    // Clone format for use inside the blocking task
    let format_for_parse = format_used.clone();

    // 5. Execute CLI command in a blocking task
    // The CLI's execute_bump_snapshot uses types that are not Send/Sync (RefCell, git2::Repository),
    // so we must run it on a blocking thread via spawn_blocking.
    let result = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        // This is necessary because execute_bump_snapshot is async but we're in a blocking context
        let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(e) => {
                return Err(ErrorInfo::execution(format!("Failed to create runtime: {e}")));
            }
        };

        rt.block_on(async {
            // Create shared buffer for output capture
            let buffer = SharedBuffer::new();

            // Create Output with JSON format
            let output = Output::new(OutputFormat::Json, buffer.clone(), true);

            // Execute the CLI command
            let config_path_ref: Option<&Path> = config_path.as_deref();
            if let Err(cli_error) =
                execute_bump_snapshot(&args, &output, &root_path, config_path_ref).await
            {
                return Err(ErrorInfo::from(cli_error));
            }

            // Extract and parse JSON
            let json_bytes = buffer.take_bytes();
            parse_snapshot_response(&json_bytes, format_for_parse)
        })
    })
    .await;

    // 6. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => BumpSnapshotApiResponse::success(data),
        Ok(Err(error)) => BumpSnapshotApiResponse::failure(error),
        Err(join_error) => BumpSnapshotApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}
