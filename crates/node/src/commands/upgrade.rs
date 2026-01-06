//! Upgrade command implementations for Node.js bindings.
//!
//! # What
//!
//! This module implements the upgrade NAPI functions that handle dependency
//! upgrades from npm registries. It provides controlled upgrade workflows with
//! backup and restore capabilities for safety.
//!
//! # How
//!
//! The module provides the following functions:
//!
//! - `upgrade_check`: Checks for available dependency upgrades
//! - `upgrade_apply`: Applies selected upgrades to package.json files (TODO: Story 8.3)
//! - `backup_list`: Lists available backups (TODO: Story 8.4)
//! - `backup_restore`: Restores package.json files from a backup (TODO: Story 8.4)
//! - `backup_clean`: Cleans old backups (TODO: Story 8.4)
//!
//! Each function:
//! 1. Validates the input parameters
//! 2. Calls the appropriate `execute_*` function from `sublime_cli_tools`
//! 3. Captures the JSON output
//! 4. Returns a type-safe API response with the result
//!
//! ## Implementation Pattern
//!
//! The upgrade commands follow the standard NAPI command pattern:
//!
//! ```text
//! JavaScript Call
//!        │
//!        ▼
//! Parameter Validation (validators::root, etc.)
//!        │
//!        ▼
//! spawn_blocking (for sync file operations)
//!        │
//!        ▼
//! SharedBuffer Output Capture
//!        │
//!        ▼
//! CLI execute_* Function
//!        │
//!        ▼
//! JSON Response Parsing
//!        │
//!        ▼
//! NAPI Type Conversion
//!        │
//!        ▼
//! ApiResponse<Data>
//! ```
//!
//! # Why
//!
//! Dependency management is critical for maintaining healthy projects. These
//! commands provide safe, controlled upgrade workflows that integrate with
//! the changeset system for tracking upgrade-related changes.
//!
//! # Examples
//!
//! ```typescript
//! import {
//!   upgradeCheck,
//!   // upgradeApply,     // TODO: Story 8.3
//!   // backupList,       // TODO: Story 8.4
//!   // backupRestore,    // TODO: Story 8.4
//!   // backupClean       // TODO: Story 8.4
//! } from '@websublime/workspace-tools';
//!
//! // Check for available upgrades
//! const checkResult = await upgradeCheck({
//!   root: '.',
//!   includeMajor: false,
//!   includeMinor: true,
//!   includePatch: true
//! });
//! if (checkResult.success) {
//!   console.log(`Found ${checkResult.data.summary.totalUpgrades} upgrades:`);
//!   console.log(`  Major: ${checkResult.data.summary.majorUpgrades}`);
//!   console.log(`  Minor: ${checkResult.data.summary.minorUpgrades}`);
//!   console.log(`  Patch: ${checkResult.data.summary.patchUpgrades}`);
//!
//!   for (const pkg of checkResult.data.packages) {
//!     for (const dep of pkg.dependencies) {
//!       console.log(`${pkg.packageName}: ${dep.name} ${dep.currentVersion} -> ${dep.latestVersion}`);
//!     }
//!   }
//! }
//! ```

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use napi_derive::napi;
use serde::Deserialize;

use crate::error::ErrorInfo;
use crate::types::upgrade::{
    DependencyUpgradeInfo, PackageUpgradeInfo, UpgradeCheckApiResponse, UpgradeCheckData,
    UpgradeCheckParams, UpgradeSummaryInfo,
};
use crate::validation::validators;

use sublime_cli_tools::cli::commands::UpgradeCheckArgs;
use sublime_cli_tools::commands::upgrade::execute_upgrade_check;
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

/// CLI JSON response wrapper for upgrade check command.
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

/// CLI upgrade check response data structure.
///
/// Mirrors the `UpgradeCheckResponse` structure from the CLI's upgrade command.
#[derive(Debug, Deserialize)]
pub(crate) struct CliUpgradeCheckData {
    /// List of packages with available upgrades.
    pub(crate) packages: Vec<CliPackageUpgradeInfo>,
    /// Summary statistics.
    pub(crate) summary: CliUpgradeSummary,
}

/// CLI package upgrade information.
#[derive(Debug, Deserialize)]
pub(crate) struct CliPackageUpgradeInfo {
    /// Package name.
    pub(crate) name: String,
    /// Package path.
    pub(crate) path: String,
    /// List of dependency upgrades.
    pub(crate) upgrades: Vec<CliDependencyUpgradeInfo>,
}

/// CLI dependency upgrade information.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliDependencyUpgradeInfo {
    /// Dependency package name.
    pub(crate) package: String,
    /// Current version.
    pub(crate) current_version: String,
    /// Latest available version.
    pub(crate) latest_version: String,
    /// Upgrade type (major, minor, patch).
    #[serde(rename = "type")]
    pub(crate) upgrade_type: String,
    /// Whether this is a breaking change.
    #[allow(dead_code)]
    pub(crate) breaking: bool,
}

/// CLI upgrade summary.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliUpgradeSummary {
    /// Total packages analyzed.
    pub(crate) total_packages: u32,
    /// Packages with upgrades.
    #[allow(dead_code)]
    pub(crate) packages_with_upgrades: u32,
    /// Total upgrades available.
    pub(crate) total_upgrades: u32,
    /// Major upgrades count.
    #[serde(rename = "major")]
    pub(crate) major_upgrades: u32,
    /// Minor upgrades count.
    #[serde(rename = "minor")]
    pub(crate) minor_upgrades: u32,
    /// Patch upgrades count.
    #[serde(rename = "patch")]
    pub(crate) patch_upgrades: u32,
}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Converts CLI upgrade check data to NAPI-compatible `UpgradeCheckData`.
///
/// This function performs a field-by-field conversion from the CLI's internal
/// types to the NAPI types exposed to JavaScript.
///
/// # Arguments
///
/// * `cli_data` - The parsed CLI response data
///
/// # Returns
///
/// An `UpgradeCheckData` instance suitable for returning to JavaScript.
pub(crate) fn convert_to_napi_upgrade_check(cli_data: CliUpgradeCheckData) -> UpgradeCheckData {
    let packages: Vec<PackageUpgradeInfo> = cli_data
        .packages
        .into_iter()
        .map(|pkg| PackageUpgradeInfo {
            package_name: pkg.name,
            package_path: pkg.path,
            dependencies: pkg
                .upgrades
                .into_iter()
                .map(|dep| DependencyUpgradeInfo {
                    name: dep.package,
                    current_version: dep.current_version,
                    latest_version: dep.latest_version,
                    upgrade_type: dep.upgrade_type,
                    // Determine dependency type from the upgrade type (CLI doesn't provide this)
                    // Default to "regular" since the CLI doesn't distinguish in the check response
                    dependency_type: "regular".to_string(),
                })
                .collect(),
        })
        .collect();

    let summary = UpgradeSummaryInfo {
        packages_analyzed: cli_data.summary.total_packages,
        total_upgrades: cli_data.summary.total_upgrades,
        major_upgrades: cli_data.summary.major_upgrades,
        minor_upgrades: cli_data.summary.minor_upgrades,
        patch_upgrades: cli_data.summary.patch_upgrades,
    };

    UpgradeCheckData { packages, summary }
}

/// Parses the JSON response from the CLI and converts it to NAPI types.
///
/// # Arguments
///
/// * `json_bytes` - The raw JSON bytes captured from CLI output
///
/// # Returns
///
/// * `Ok(UpgradeCheckData)` - Successfully parsed and converted upgrade check data
/// * `Err(ErrorInfo)` - Parsing failed or CLI returned an error
///
/// # Errors
///
/// Returns an error if:
/// - The JSON is malformed or cannot be parsed
/// - The CLI returned `success: false` with an error message
/// - The CLI returned `success: true` but `data` is missing
pub(crate) fn parse_upgrade_check_response(
    json_bytes: &[u8],
) -> Result<UpgradeCheckData, ErrorInfo> {
    // Convert bytes to string first for better error messages
    let json_str = std::str::from_utf8(json_bytes)
        .map_err(|e| ErrorInfo::execution(format!("Invalid UTF-8 in CLI response: {e}")))?;

    // Handle empty response
    if json_str.trim().is_empty() {
        return Err(ErrorInfo::execution("CLI returned empty response"));
    }

    // Parse the JSON response
    let response: CliJsonResponse<CliUpgradeCheckData> =
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

    Ok(convert_to_napi_upgrade_check(cli_data))
}

// ============================================================================
// Parameter Validation
// ============================================================================

/// Validates upgrade check command parameters.
///
/// Ensures the root path is valid and all filter options are consistent.
///
/// # Arguments
///
/// * `params` - The upgrade check parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path
/// * `Err(ErrorInfo)` - Validation failed
pub(crate) fn validate_upgrade_check_params(
    params: &UpgradeCheckParams,
) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    // Validate that at least one upgrade type is enabled
    // Default behavior: if no explicit include_* flags are set, include all types
    let include_major = params.include_major.unwrap_or(true);
    let include_minor = params.include_minor.unwrap_or(true);
    let include_patch = params.include_patch.unwrap_or(true);

    if !include_major && !include_minor && !include_patch {
        return Err(ErrorInfo::validation(
            "At least one upgrade type (major, minor, or patch) must be enabled",
            Some("include_major, include_minor, include_patch"),
        ));
    }

    Ok(PathBuf::from(&params.root))
}

/// Converts NAPI params to CLI args.
///
/// The CLI uses `no_*` flags (negative logic) while the NAPI API uses
/// `include_*` flags (positive logic). This function performs the conversion.
///
/// # Arguments
///
/// * `params` - The NAPI parameters
///
/// # Returns
///
/// CLI-compatible `UpgradeCheckArgs`.
pub(crate) fn convert_params_to_args(params: &UpgradeCheckParams) -> UpgradeCheckArgs {
    // Convert positive include_* flags to negative no_* flags
    // Default behavior: include all types (so no_* is false by default)
    let no_major = !params.include_major.unwrap_or(true);
    let no_minor = !params.include_minor.unwrap_or(true);
    let no_patch = !params.include_patch.unwrap_or(true);

    // Dev dependencies: default is to include them
    let no_dev = !params.include_dev_dependencies.unwrap_or(true);

    // Peer dependencies: default is to exclude them
    let peer = params.include_peer_dependencies.unwrap_or(false);

    UpgradeCheckArgs {
        no_major,
        no_minor,
        no_patch,
        no_dev,
        peer,
        packages: params.packages.clone(),
        registry: None, // Registry override not exposed in NAPI API
    }
}

// ============================================================================
// NAPI Function - upgradeCheck
// ============================================================================

/// Check for available dependency upgrades.
///
/// Detects which dependencies in the workspace have newer versions available
/// from the npm registry. This is a read-only operation that does not modify
/// any files.
///
/// The function checks all packages in the workspace and returns a comprehensive
/// list of available upgrades along with summary statistics.
///
/// @param params - Upgrade check parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `configPath`: Optional custom configuration file path
///   - `includeMajor`: Whether to include major version upgrades (default: true)
///   - `includeMinor`: Whether to include minor version upgrades (default: true)
///   - `includePatch`: Whether to include patch version upgrades (default: true)
///   - `includeDevDependencies`: Whether to check devDependencies (default: true)
///   - `includePeerDependencies`: Whether to check peerDependencies (default: false)
///   - `packages`: Optional list of packages to check (checks all if not specified)
///
/// @returns `Promise<UpgradeCheckApiResponse>` containing:
///   - On success: `{ success: true, data: UpgradeCheckData }`
///   - On failure: `{ success: false, error: ErrorInfo }`
///
/// @example Basic usage
/// ```typescript
/// const result = await upgradeCheck({ root: '.' });
/// if (result.success) {
///   console.log(`Found ${result.data.summary.totalUpgrades} upgrades`);
///   for (const pkg of result.data.packages) {
///     console.log(`${pkg.packageName}:`);
///     for (const dep of pkg.dependencies) {
///       console.log(`  ${dep.name}: ${dep.currentVersion} -> ${dep.latestVersion}`);
///     }
///   }
/// }
/// ```
///
/// @example Safe upgrades only (no major version changes)
/// ```typescript
/// const result = await upgradeCheck({
///   root: '/path/to/workspace',
///   includeMajor: false,
///   includeMinor: true,
///   includePatch: true
/// });
/// if (result.success) {
///   console.log(`Safe upgrades: ${result.data.summary.minorUpgrades + result.data.summary.patchUpgrades}`);
/// }
/// ```
///
/// @example Check specific packages
/// ```typescript
/// const result = await upgradeCheck({
///   root: '.',
///   packages: ['@scope/pkg1', '@scope/pkg2']
/// });
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await upgradeCheck({ root: '/invalid/path' });
/// if (!result.success) {
///   switch (result.error.code) {
///     case 'ENOENT':
///       console.error('Path not found');
///       break;
///     case 'EVALIDATION':
///       console.error('Invalid parameters:', result.error.message);
///       break;
///     case 'ENETWORK':
///       console.error('Network error - registry unreachable');
///       break;
///     default:
///       console.error(`Error: ${result.error.message}`);
///   }
/// }
/// ```
#[napi]
pub async fn upgrade_check(params: UpgradeCheckParams) -> UpgradeCheckApiResponse {
    // 1. Validate parameters (synchronous validation before spawning)
    let root_path = match validate_upgrade_check_params(&params) {
        Ok(path) => path,
        Err(error) => return UpgradeCheckApiResponse::failure(error),
    };

    // 2. Convert params to CLI args
    let args = convert_params_to_args(&params);

    // 3. Execute CLI command in a blocking task
    // The CLI's execute_upgrade_check uses types that may not be Send/Sync,
    // so we run it on a blocking thread via spawn_blocking.
    let result = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        // This is necessary because execute_upgrade_check is async but we're in a blocking context
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
            if let Err(cli_error) = execute_upgrade_check(&args, &output, &root_path).await {
                return Err(ErrorInfo::from(cli_error));
            }

            // Extract and parse JSON
            let json_bytes = buffer.take_bytes();
            parse_upgrade_check_response(&json_bytes)
        })
    })
    .await;

    // 4. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => UpgradeCheckApiResponse::success(data),
        Ok(Err(error)) => UpgradeCheckApiResponse::failure(error),
        Err(join_error) => UpgradeCheckApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}

// ============================================================================
// TODO: Story 8.3 - upgradeApply
// ============================================================================

// Implementation outline for upgradeApply:
//
// #[napi]
// pub async fn upgrade_apply(params: UpgradeApplyParams) -> UpgradeApplyApiResponse {
//     // 1. Validate parameters
//     if let Err(e) = validate_upgrade_apply_params(&params) {
//         return UpgradeApplyApiResponse::failure(e);
//     }
//
//     // 2. Convert params to CLI args
//     // 3. Optionally create backup if params.create_backup is true
//     // 4. Create Output with JSON format for capturing
//     // 5. Call execute_upgrade_apply from sublime_cli_tools
//     // 6. Parse response with applied, skipped, failed counts
//     // 7. Return UpgradeApplyApiResponse::success(data) or ::failure(error)
// }

// ============================================================================
// TODO: Story 8.4 - Backup Commands
// ============================================================================

// Implementation outline for backupList:
//
// #[napi]
// pub async fn backup_list(params: BackupListParams) -> BackupListApiResponse {
//     // 1. Validate parameters
//     if let Err(e) = validate_backup_list_params(&params) {
//         return BackupListApiResponse::failure(e);
//     }
//
//     // 2. Create BackupManager instance
//     // 3. Call list_backups
//     // 4. Return list of backup metadata
// }
//
// Implementation outline for backupRestore:
//
// #[napi]
// pub async fn backup_restore(params: BackupRestoreParams) -> BackupRestoreApiResponse {
//     // 1. Validate parameters (including backup_id not empty)
//     if let Err(e) = validate_backup_restore_params(&params) {
//         return BackupRestoreApiResponse::failure(e);
//     }
//
//     // 2. Create BackupManager instance
//     // 3. Call restore_backup with the backup_id
//     // 4. Return restore results
// }
//
// Implementation outline for backupClean:
//
// #[napi]
// pub async fn backup_clean(params: BackupCleanParams) -> BackupCleanApiResponse {
//     // 1. Validate parameters
//     if let Err(e) = validate_backup_clean_params(&params) {
//         return BackupCleanApiResponse::failure(e);
//     }
//
//     // 2. Create BackupManager instance
//     // 3. Call clean_backups with keep_count
//     // 4. Return cleanup results
// }
