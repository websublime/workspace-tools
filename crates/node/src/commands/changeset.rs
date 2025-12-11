//! Changeset command implementations for Node.js bindings.
//!
//! # What
//!
//! This module implements the changeset NAPI functions that manage the changeset
//! workflow. Changesets track intended changes before they are applied as version
//! bumps, enabling better release management and changelog generation.
//!
//! # How
//!
//! The module provides the following functions:
//!
//! - `changeset_add`: Creates a new changeset for the current branch
//! - `changeset_update`: Updates an existing changeset with new packages, commits, bump type, or environments
//! - `changeset_list`: Lists all pending changesets with optional filtering and sorting
//!
//! Each function follows this pattern:
//! 1. Validates the input parameters
//! 2. Creates a `SharedBuffer` to capture CLI JSON output
//! 3. Converts NAPI params to CLI args with `non_interactive=true`
//! 4. Calls the appropriate `execute_*` function from `sublime_cli_tools`
//! 5. Parses the JSON output into intermediate types
//! 6. Converts to NAPI-compatible response types
//! 7. Returns an `ApiResponse<T>` with the result
//!
//! ## Non-Interactive Mode
//!
//! All NAPI functions force `non_interactive=true` because Node.js bindings
//! cannot support interactive terminal prompts. This means:
//! - Required parameters must be provided explicitly
//! - Optional parameters use defaults from configuration
//! - Auto-detection is used where possible (e.g., packages from git changes)
//!
//! # Why
//!
//! Changesets are the core of the version management workflow. They allow
//! developers to document their changes as they work, and then batch those
//! changes into coordinated version bumps and releases.
//!
//! # Examples
//!
//! ```typescript
//! import { changesetAdd, changesetUpdate } from '@websublime/workspace-tools';
//!
//! // Add a new changeset
//! const addResult = await changesetAdd({
//!   root: '.',
//!   packages: ['@scope/core', '@scope/utils'],
//!   bump: 'minor',
//!   message: 'Add new feature'
//! });
//!
//! if (addResult.success) {
//!   console.log(`Created changeset: ${addResult.data.id}`);
//!   console.log(`Branch: ${addResult.data.branch}`);
//!   console.log(`Packages: ${addResult.data.packages.join(', ')}`);
//! } else {
//!   console.error(`Error [${addResult.error.code}]: ${addResult.error.message}`);
//! }
//!
//! // Update an existing changeset
//! const updateResult = await changesetUpdate({
//!   root: '.',
//!   id: 'feature/new-api',
//!   packages: ['@scope/new-package'],
//!   bump: 'major'
//! });
//!
//! if (updateResult.success) {
//!   console.log(`Updated: ${updateResult.data.updated}`);
//!   console.log(`Packages added: ${updateResult.data.summary.packagesAdded}`);
//!   console.log(`Current packages: ${updateResult.data.changeset.packages.join(', ')}`);
//! } else {
//!   console.error(`Error [${updateResult.error.code}]: ${updateResult.error.message}`);
//! }
//!
//! // List all pending changesets
//! const listResult = await changesetList({
//!   root: '.',
//!   filterBump: 'minor',
//!   sort: 'date'
//! });
//!
//! if (listResult.success) {
//!   console.log(`Found ${listResult.data.count} changeset(s)`);
//!   for (const cs of listResult.data.changesets) {
//!     console.log(`- ${cs.branch}: ${cs.bump} (${cs.packages.length} packages)`);
//!   }
//! } else {
//!   console.error(`Error [${listResult.error.code}]: ${listResult.error.message}`);
//! }
//! ```

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use napi_derive::napi;
use serde::Deserialize;

use crate::error::ErrorInfo;
use crate::types::changeset::{
    ArchivedChangesetInfo, ChangesetAddApiResponse, ChangesetAddData, ChangesetAddParams,
    ChangesetDetailInfo, ChangesetHistoryApiResponse, ChangesetHistoryData, ChangesetHistoryParams,
    ChangesetListApiResponse, ChangesetListData, ChangesetListItemInfo, ChangesetListParams,
    ChangesetRemoveApiResponse, ChangesetRemoveData, ChangesetRemoveParams,
    ChangesetShowApiResponse, ChangesetShowData, ChangesetShowParams, ChangesetUpdateApiResponse,
    ChangesetUpdateData, ChangesetUpdateParams, ReleaseInfoData, ReleasedVersionEntry,
    UpdateSummaryInfo, VALID_SORT_OPTIONS,
};
use crate::validation::validators;

use sublime_cli_tools::cli::commands::{
    ChangesetCreateArgs, ChangesetDeleteArgs, ChangesetHistoryArgs, ChangesetListArgs,
    ChangesetShowArgs, ChangesetUpdateArgs,
};
use sublime_cli_tools::commands::changeset::{
    execute_add, execute_history, execute_list, execute_remove, execute_show, execute_update,
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
#[derive(Debug, Clone)]
pub(crate) struct SharedBuffer {
    /// The inner buffer wrapped in `Arc<Mutex>` for shared ownership.
    inner: Arc<Mutex<Vec<u8>>>,
}

impl SharedBuffer {
    /// Creates a new empty shared buffer.
    pub(crate) fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Extracts the bytes written to the buffer.
    ///
    /// This method clones the current buffer contents. The original buffer
    /// remains intact for potential further writes.
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
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.inner.lock() {
            Ok(mut guard) => guard.write(buf),
            Err(_) => Err(std::io::Error::other("Mutex poisoned")),
        }
    }

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

/// CLI JSON response wrapper for changeset add command.
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

/// CLI changeset add response data structure.
///
/// Mirrors the `ChangesetAddResponse` structure from the CLI's changeset add command.
#[derive(Debug, Deserialize)]
pub(crate) struct CliChangesetAddResponseData {
    /// Whether the changeset was created successfully.
    #[allow(dead_code)]
    pub(crate) success: bool,
    /// The created changeset details.
    pub(crate) changeset: CliChangesetInfo,
    /// Optional message for the changeset.
    #[allow(dead_code)]
    pub(crate) message: Option<String>,
}

/// CLI changeset information structure.
///
/// Mirrors the `ChangesetInfo` structure from the CLI's types module.
/// Field names use camelCase to match the JSON output format.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliChangesetInfo {
    /// Branch name (also serves as unique identifier).
    pub(crate) branch: String,
    /// Version bump type (major, minor, patch, none).
    pub(crate) bump: String,
    /// List of affected packages.
    pub(crate) packages: Vec<String>,
    /// Target environments.
    pub(crate) environments: Vec<String>,
    /// List of commit IDs.
    #[allow(dead_code)]
    pub(crate) commits: Vec<String>,
    /// Creation timestamp (RFC3339 format).
    pub(crate) created_at: String,
    /// Last update timestamp (RFC3339 format).
    #[allow(dead_code)]
    pub(crate) updated_at: String,
}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Converts CLI changeset info to NAPI-compatible `ChangesetAddData`.
///
/// This function performs a field-by-field conversion from the CLI's
/// internal types to the NAPI types exposed to JavaScript.
///
/// # Arguments
///
/// * `cli_info` - The parsed CLI changeset information
///
/// # Returns
///
/// A `ChangesetAddData` instance suitable for returning to JavaScript.
pub(crate) fn convert_to_napi_add_data(cli_info: CliChangesetInfo) -> ChangesetAddData {
    ChangesetAddData {
        // The id is derived from the branch name in the CLI
        id: cli_info.branch.clone(),
        branch: cli_info.branch,
        packages: cli_info.packages,
        bump: cli_info.bump,
        environments: cli_info.environments,
        created_at: cli_info.created_at,
    }
}

/// Parses the JSON response from the CLI and converts it to NAPI types.
///
/// # Arguments
///
/// * `json_bytes` - The raw JSON bytes captured from CLI output
///
/// # Returns
///
/// * `Ok(ChangesetAddData)` - Successfully parsed and converted changeset data
/// * `Err(ErrorInfo)` - Parsing failed or CLI returned an error
///
/// # Errors
///
/// Returns an error if:
/// - The JSON is malformed or cannot be parsed
/// - The CLI returned `success: false` with an error message
/// - The CLI returned `success: true` but `data` is missing
pub(crate) fn parse_changeset_add_response(
    json_bytes: &[u8],
) -> Result<ChangesetAddData, ErrorInfo> {
    // Convert bytes to string first for better error messages
    let json_str = std::str::from_utf8(json_bytes)
        .map_err(|e| ErrorInfo::execution(format!("Invalid UTF-8 in CLI response: {e}")))?;

    // Handle empty response
    if json_str.trim().is_empty() {
        return Err(ErrorInfo::execution("CLI returned empty response"));
    }

    // Parse the JSON response
    let response: CliJsonResponse<CliChangesetAddResponseData> = serde_json::from_str(json_str)
        .map_err(|e| {
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

    Ok(convert_to_napi_add_data(cli_data.changeset))
}

// ============================================================================
// Parameter Validation
// ============================================================================

/// Validates changeset add command parameters.
///
/// Ensures the root path is valid before executing the CLI command.
/// In non-interactive mode, certain parameters may be required.
///
/// # Arguments
///
/// * `params` - The changeset add parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path
/// * `Err(ErrorInfo)` - Validation failed
pub(crate) fn validate_params(params: &ChangesetAddParams) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    // Validate bump type if provided
    if let Some(ref bump) = params.bump {
        validators::bump_type_info(bump)?;
    }

    Ok(PathBuf::from(&params.root))
}

/// Converts NAPI parameters to CLI arguments.
///
/// This function transforms the NAPI-friendly `ChangesetAddParams` into the
/// CLI's `ChangesetCreateArgs` structure, always setting `non_interactive=true`
/// since NAPI bindings cannot support terminal prompts.
///
/// # Arguments
///
/// * `params` - The NAPI changeset add parameters
///
/// # Returns
///
/// A `ChangesetCreateArgs` instance ready for CLI execution.
pub(crate) fn convert_params_to_args(params: &ChangesetAddParams) -> ChangesetCreateArgs {
    ChangesetCreateArgs {
        bump: params.bump.clone(),
        env: params.environments.clone(),
        branch: params.branch.clone(),
        message: params.message.clone(),
        packages: params.packages.clone(),
        // Always non-interactive for NAPI - cannot support terminal prompts
        non_interactive: true,
        force: params.force.unwrap_or(false),
    }
}

// ============================================================================
// NAPI Function
// ============================================================================

/// Add a new changeset to the workspace.
///
/// Creates a new changeset for the current or specified branch. The changeset
/// records which packages are affected, the version bump type, and optionally
/// a message describing the changes.
///
/// This function always operates in non-interactive mode. If packages are not
/// specified, they will be auto-detected from git changes. If bump type is not
/// specified, an error will be returned (unlike the CLI which would prompt).
///
/// @param params - Changeset add parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `configPath`: Optional custom config file path
///   - `bump`: Version bump type (major, minor, patch)
///   - `environments`: Optional list of target environments
///   - `branch`: Optional branch name (defaults to current git branch)
///   - `message`: Optional message describing the changes
///   - `packages`: Optional list of packages (auto-detected if not provided)
///   - `force`: Optional flag to overwrite existing changeset
///
/// @returns `Promise<ChangesetAddApiResponse>` containing:
///   - On success: `{ success: true, data: ChangesetAddData }`
///   - On failure: `{ success: false, error: ErrorInfo }`
///
/// @example Basic usage with auto-detected packages
/// ```typescript
/// const result = await changesetAdd({
///   root: '/path/to/workspace',
///   bump: 'minor',
///   message: 'Add new API endpoints'
/// });
///
/// if (result.success) {
///   console.log(`Created changeset: ${result.data.id}`);
///   console.log(`Packages: ${result.data.packages.join(', ')}`);
/// }
/// ```
///
/// @example With explicit packages
/// ```typescript
/// const result = await changesetAdd({
///   root: '/path/to/workspace',
///   packages: ['@scope/core', '@scope/utils'],
///   bump: 'major',
///   message: 'Breaking API changes',
///   environments: ['staging', 'production']
/// });
/// ```
///
/// @example Force overwrite existing changeset
/// ```typescript
/// const result = await changesetAdd({
///   root: '/path/to/workspace',
///   packages: ['my-package'],
///   bump: 'patch',
///   force: true
/// });
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await changesetAdd({
///   root: '/nonexistent/path',
///   bump: 'minor'
/// });
///
/// if (!result.success) {
///   switch (result.error.code) {
///     case 'ENOENT':
///       console.error('Path not found');
///       break;
///     case 'EVALIDATION':
///       console.error('Invalid parameters:', result.error.message);
///       break;
///     case 'EGIT':
///       console.error('Git error:', result.error.message);
///       break;
///     default:
///       console.error(`Error: ${result.error.message}`);
///   }
/// }
/// ```
#[napi(js_name = "changesetAdd")]
pub async fn changeset_add(params: ChangesetAddParams) -> ChangesetAddApiResponse {
    // 1. Validate parameters (synchronous validation before spawning)
    let root_path = match validate_params(&params) {
        Ok(path) => path,
        Err(error) => return ChangesetAddApiResponse::failure(error),
    };

    // 2. Prepare config path
    let config_path: Option<PathBuf> = params.config_path.as_ref().map(PathBuf::from);

    // 3. Convert NAPI params to CLI args
    let args = convert_params_to_args(&params);

    // 4. Execute CLI command in a blocking task
    // The CLI's execute_add uses types that are not Send/Sync (RefCell, git2::Repository),
    // so we must run it on a blocking thread via spawn_blocking.
    let result = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        // This is necessary because execute_add is async but we're in a blocking context
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
            if let Err(cli_error) = execute_add(&args, &output, Some(root_path), config_path).await
            {
                return Err(ErrorInfo::from(cli_error));
            }

            // Extract and parse JSON
            let json_bytes = buffer.take_bytes();
            parse_changeset_add_response(&json_bytes)
        })
    })
    .await;

    // 5. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => ChangesetAddApiResponse::success(data),
        Ok(Err(error)) => ChangesetAddApiResponse::failure(error),
        Err(join_error) => ChangesetAddApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}

// ============================================================================
// Changeset Update - CLI Response Types
// ============================================================================

/// CLI JSON response data for changeset update command.
///
/// This type mirrors the `ChangesetUpdateResponse` structure from the CLI's
/// update command, used for deserializing the captured JSON output.
#[derive(Debug, Deserialize)]
pub(crate) struct CliChangesetUpdateResponseData {
    /// Whether the operation succeeded.
    #[allow(dead_code)]
    pub(crate) success: bool,
    /// Summary of what was updated.
    pub(crate) updated: CliUpdateSummary,
    /// The updated changeset details.
    pub(crate) changeset: CliUpdatedChangesetInfo,
}

/// CLI update summary structure.
///
/// Mirrors the `UpdateSummary` structure from the CLI's changeset update command.
/// Field names use snake_case to match the JSON output format.
#[derive(Debug, Deserialize)]
pub(crate) struct CliUpdateSummary {
    /// Number of packages added.
    pub(crate) packages_added: usize,
    /// Number of commits added.
    pub(crate) commits_added: usize,
    /// Whether bump type was changed.
    pub(crate) bump_updated: bool,
    /// Number of environments added.
    pub(crate) environments_added: usize,
}

/// CLI changeset information structure for update response.
///
/// Mirrors the `ChangesetInfo` structure from the CLI's update command.
/// Field names use camelCase to match the JSON output format.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliUpdatedChangesetInfo {
    /// Branch name (also serves as unique identifier).
    pub(crate) branch: String,
    /// Version bump type (major, minor, patch, none).
    pub(crate) bump: String,
    /// List of affected packages.
    pub(crate) packages: Vec<String>,
    /// Target environments.
    pub(crate) environments: Vec<String>,
    /// List of commit IDs.
    pub(crate) commits: Vec<String>,
    /// Creation timestamp (RFC3339 format).
    pub(crate) created_at: String,
    /// Last update timestamp (RFC3339 format).
    pub(crate) updated_at: String,
}

// ============================================================================
// Changeset Update - Conversion Functions
// ============================================================================

/// Converts CLI update summary to NAPI-compatible `UpdateSummaryInfo`.
///
/// This function performs a field-by-field conversion from the CLI's
/// internal types to the NAPI types exposed to JavaScript.
///
/// # Arguments
///
/// * `cli_summary` - The parsed CLI update summary
///
/// # Returns
///
/// An `UpdateSummaryInfo` instance suitable for returning to JavaScript.
pub(crate) fn convert_to_napi_update_summary(cli_summary: &CliUpdateSummary) -> UpdateSummaryInfo {
    // Safe truncation: these counts will never exceed u32::MAX in practice
    #[allow(clippy::cast_possible_truncation)]
    UpdateSummaryInfo {
        packages_added: cli_summary.packages_added as u32,
        commits_added: cli_summary.commits_added as u32,
        bump_updated: cli_summary.bump_updated,
        environments_added: cli_summary.environments_added as u32,
    }
}

/// Converts CLI changeset info to NAPI-compatible `ChangesetDetailInfo`.
///
/// This function performs a field-by-field conversion from the CLI's
/// internal types to the NAPI types exposed to JavaScript.
///
/// # Arguments
///
/// * `cli_info` - The parsed CLI changeset information
///
/// # Returns
///
/// A `ChangesetDetailInfo` instance suitable for returning to JavaScript.
pub(crate) fn convert_to_napi_changeset_detail(
    cli_info: &CliUpdatedChangesetInfo,
) -> ChangesetDetailInfo {
    ChangesetDetailInfo {
        id: cli_info.branch.clone(),
        branch: cli_info.branch.clone(),
        bump: cli_info.bump.clone(),
        packages: cli_info.packages.clone(),
        environments: cli_info.environments.clone(),
        commits: cli_info.commits.clone(),
        message: None, // CLI update response doesn't include message
        created_at: cli_info.created_at.clone(),
        updated_at: cli_info.updated_at.clone(),
    }
}

/// Converts CLI update response to NAPI-compatible `ChangesetUpdateData`.
///
/// # Arguments
///
/// * `cli_data` - The parsed CLI update response data
///
/// # Returns
///
/// A `ChangesetUpdateData` instance suitable for returning to JavaScript.
pub(crate) fn convert_to_napi_update_data(
    cli_data: &CliChangesetUpdateResponseData,
) -> ChangesetUpdateData {
    let summary = convert_to_napi_update_summary(&cli_data.updated);
    let changeset = convert_to_napi_changeset_detail(&cli_data.changeset);
    let updated = summary.has_changes();

    ChangesetUpdateData { updated, summary, changeset }
}

/// Parses the JSON response from the CLI update command and converts it to NAPI types.
///
/// # Arguments
///
/// * `json_bytes` - The raw JSON bytes captured from CLI output
///
/// # Returns
///
/// * `Ok(ChangesetUpdateData)` - Successfully parsed and converted update data
/// * `Err(ErrorInfo)` - Parsing failed or CLI returned an error
///
/// # Errors
///
/// Returns an error if:
/// - The JSON is malformed or cannot be parsed
/// - The CLI returned `success: false` with an error message
/// - The CLI returned `success: true` but `data` is missing
pub(crate) fn parse_changeset_update_response(
    json_bytes: &[u8],
) -> Result<ChangesetUpdateData, ErrorInfo> {
    // Convert bytes to string first for better error messages
    let json_str = std::str::from_utf8(json_bytes)
        .map_err(|e| ErrorInfo::execution(format!("Invalid UTF-8 in CLI response: {e}")))?;

    // Handle empty response
    if json_str.trim().is_empty() {
        return Err(ErrorInfo::execution("CLI returned empty response"));
    }

    // Parse the JSON response
    let response: CliJsonResponse<CliChangesetUpdateResponseData> = serde_json::from_str(json_str)
        .map_err(|e| {
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

    Ok(convert_to_napi_update_data(&cli_data))
}

// ============================================================================
// Changeset Update - Parameter Validation
// ============================================================================

/// Validates changeset update command parameters.
///
/// Ensures the root path is valid and that required parameters are provided
/// before executing the CLI command. For NAPI bindings, the `id` parameter
/// is required since auto-detection from git branch is not reliable in
/// programmatic contexts.
///
/// # Arguments
///
/// * `params` - The changeset update parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path
/// * `Err(ErrorInfo)` - Validation failed
///
/// # Errors
///
/// Returns an error if:
/// - The root path is empty, doesn't exist, or is not a directory
/// - The `id` parameter is not provided
/// - The bump type (if provided) is invalid
pub(crate) fn validate_update_params(params: &ChangesetUpdateParams) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    // Validate id is required for NAPI (cannot auto-detect git branch reliably)
    if params.id.is_none() {
        return Err(ErrorInfo::validation(
            "id is required for changesetUpdate. Provide the branch name or changeset ID.",
            Some("id"),
        ));
    }

    // Validate bump type if provided
    if let Some(ref bump) = params.bump {
        validators::bump_type_info(bump)?;
    }

    Ok(PathBuf::from(&params.root))
}

/// Converts NAPI parameters to CLI arguments.
///
/// This function transforms the NAPI-friendly `ChangesetUpdateParams` into the
/// CLI's `ChangesetUpdateArgs` structure.
///
/// # Arguments
///
/// * `params` - The NAPI changeset update parameters
///
/// # Returns
///
/// A `ChangesetUpdateArgs` instance ready for CLI execution.
pub(crate) fn convert_update_params_to_args(params: &ChangesetUpdateParams) -> ChangesetUpdateArgs {
    ChangesetUpdateArgs {
        id: params.id.clone(),
        commit: params.commit.clone(),
        packages: params.packages.clone(),
        bump: params.bump.clone(),
        env: params.environments.clone(),
    }
}

// ============================================================================
// Changeset Update - NAPI Function
// ============================================================================

/// Update an existing changeset in the workspace.
///
/// Modifies an existing changeset by adding packages, commits, environments,
/// or changing the bump type. The changeset is identified by the `id` parameter,
/// which corresponds to the branch name.
///
/// This function always operates in non-interactive mode. The `id` parameter
/// is required since auto-detection of the current git branch is not reliable
/// in programmatic contexts.
///
/// @param params - Changeset update parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `configPath`: Optional custom config file path
///   - `id`: Branch name or changeset ID (required)
///   - `commit`: Optional commit hash to add
///   - `packages`: Optional list of packages to add
///   - `bump`: Optional new bump type (major, minor, patch)
///   - `environments`: Optional list of environments to add
///
/// @returns `Promise<ChangesetUpdateApiResponse>` containing:
///   - On success: `{ success: true, data: ChangesetUpdateData }`
///   - On failure: `{ success: false, error: ErrorInfo }`
///
/// @example Add packages to an existing changeset
/// ```typescript
/// const result = await changesetUpdate({
///   root: '/path/to/workspace',
///   id: 'feature/new-api',
///   packages: ['@scope/new-package']
/// });
///
/// if (result.success) {
///   console.log(`Updated: ${result.data.updated}`);
///   console.log(`Packages added: ${result.data.summary.packagesAdded}`);
/// }
/// ```
///
/// @example Add a commit and change bump type
/// ```typescript
/// const result = await changesetUpdate({
///   root: '/path/to/workspace',
///   id: 'feature/breaking-change',
///   commit: 'abc123def456',
///   bump: 'major'
/// });
///
/// if (result.success) {
///   console.log(`Bump updated: ${result.data.summary.bumpUpdated}`);
///   console.log(`Current bump: ${result.data.changeset.bump}`);
/// }
/// ```
///
/// @example Add environments
/// ```typescript
/// const result = await changesetUpdate({
///   root: '/path/to/workspace',
///   id: 'feature/deploy',
///   environments: ['staging', 'production']
/// });
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await changesetUpdate({
///   root: '/path/to/workspace',
///   id: 'nonexistent-branch'
/// });
///
/// if (!result.success) {
///   switch (result.error.code) {
///     case 'ENOENT':
///       console.error('Path or changeset not found');
///       break;
///     case 'EVALIDATION':
///       console.error('Invalid parameters:', result.error.message);
///       break;
///     case 'EEXECUTION':
///       console.error('Update failed:', result.error.message);
///       break;
///     default:
///       console.error(`Error: ${result.error.message}`);
///   }
/// }
/// ```
#[napi(js_name = "changesetUpdate")]
pub async fn changeset_update(params: ChangesetUpdateParams) -> ChangesetUpdateApiResponse {
    // 1. Validate parameters (synchronous validation before spawning)
    let root_path = match validate_update_params(&params) {
        Ok(path) => path,
        Err(error) => return ChangesetUpdateApiResponse::failure(error),
    };

    // 2. Prepare config path
    let config_path: Option<PathBuf> = params.config_path.as_ref().map(PathBuf::from);

    // 3. Convert NAPI params to CLI args
    let args = convert_update_params_to_args(&params);

    // 4. Execute CLI command in a blocking task
    // The CLI's execute_update uses types that are not Send/Sync (RefCell, git2::Repository),
    // so we must run it on a blocking thread via spawn_blocking.
    let result = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        // This is necessary because execute_update is async but we're in a blocking context
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
            if let Err(cli_error) =
                execute_update(&args, &output, Some(root_path.as_path()), config_path.as_deref())
                    .await
            {
                return Err(ErrorInfo::from(cli_error));
            }

            // Extract and parse JSON
            let json_bytes = buffer.take_bytes();
            parse_changeset_update_response(&json_bytes)
        })
    })
    .await;

    // 5. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => ChangesetUpdateApiResponse::success(data),
        Ok(Err(error)) => ChangesetUpdateApiResponse::failure(error),
        Err(join_error) => ChangesetUpdateApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}

// ============================================================================
// Changeset List - CLI Response Types
// ============================================================================

/// CLI JSON response data for changeset list command.
///
/// This type mirrors the `ChangesetListResponse` structure from the CLI's
/// list command, used for deserializing the captured JSON output.
#[derive(Debug, Deserialize)]
pub(crate) struct CliChangesetListResponseData {
    /// Whether the operation succeeded.
    #[allow(dead_code)]
    pub(crate) success: bool,
    /// List of changesets.
    pub(crate) changesets: Vec<CliChangesetListItem>,
    /// Total count of changesets.
    #[allow(dead_code)]
    pub(crate) total: usize,
}

/// CLI changeset list item structure.
///
/// Mirrors the `ChangesetListItem` structure from the CLI's list command.
/// Field names use snake_case to match the JSON output format.
#[derive(Debug, Deserialize)]
pub(crate) struct CliChangesetListItem {
    /// Branch name (also serves as unique identifier).
    pub(crate) branch: String,
    /// Version bump type (major, minor, patch, none).
    pub(crate) bump: String,
    /// List of affected packages.
    pub(crate) packages: Vec<String>,
    /// Target environments.
    pub(crate) environments: Vec<String>,
    /// Number of commits in the changeset.
    #[allow(dead_code)]
    pub(crate) commit_count: usize,
    /// Creation timestamp (RFC3339 format).
    pub(crate) created_at: String,
    /// Last update timestamp (RFC3339 format).
    pub(crate) updated_at: String,
}

// ============================================================================
// Changeset List - Conversion Functions
// ============================================================================

/// Converts a CLI changeset list item to NAPI-compatible `ChangesetListItemInfo`.
///
/// This function converts the CLI's list item format to the NAPI type,
/// preserving the `commit_count` field. For full commit details,
/// use the `changesetShow` command.
///
/// # Arguments
///
/// * `cli_item` - The parsed CLI changeset list item
///
/// # Returns
///
/// A `ChangesetListItemInfo` instance suitable for returning to JavaScript.
pub(crate) fn convert_list_item_to_napi(cli_item: CliChangesetListItem) -> ChangesetListItemInfo {
    // Safe truncation: commit counts will never exceed u32::MAX in practice
    #[allow(clippy::cast_possible_truncation)]
    let commit_count = cli_item.commit_count as u32;

    ChangesetListItemInfo {
        // The id is derived from the branch name
        id: cli_item.branch.clone(),
        branch: cli_item.branch,
        bump: cli_item.bump,
        packages: cli_item.packages,
        environments: cli_item.environments,
        commit_count,
        created_at: cli_item.created_at,
        updated_at: cli_item.updated_at,
    }
}

/// Converts CLI list response to NAPI-compatible `ChangesetListData`.
///
/// # Arguments
///
/// * `cli_data` - The parsed CLI list response data
///
/// # Returns
///
/// A `ChangesetListData` instance suitable for returning to JavaScript.
pub(crate) fn convert_to_napi_list_data(
    cli_data: CliChangesetListResponseData,
) -> ChangesetListData {
    let changesets: Vec<ChangesetListItemInfo> =
        cli_data.changesets.into_iter().map(convert_list_item_to_napi).collect();

    ChangesetListData::new(changesets)
}

/// Parses the JSON response from the CLI list command and converts it to NAPI types.
///
/// # Arguments
///
/// * `json_bytes` - The raw JSON bytes captured from CLI output
///
/// # Returns
///
/// * `Ok(ChangesetListData)` - Successfully parsed and converted list data
/// * `Err(ErrorInfo)` - Parsing failed or CLI returned an error
///
/// # Errors
///
/// Returns an error if:
/// - The JSON is malformed or cannot be parsed
/// - The CLI returned `success: false` with an error message
/// - The CLI returned `success: true` but `data` is missing
pub(crate) fn parse_changeset_list_response(
    json_bytes: &[u8],
) -> Result<ChangesetListData, ErrorInfo> {
    // Convert bytes to string first for better error messages
    let json_str = std::str::from_utf8(json_bytes)
        .map_err(|e| ErrorInfo::execution(format!("Invalid UTF-8 in CLI response: {e}")))?;

    // Handle empty response
    if json_str.trim().is_empty() {
        return Err(ErrorInfo::execution("CLI returned empty response"));
    }

    // Parse the JSON response
    let response: CliJsonResponse<CliChangesetListResponseData> = serde_json::from_str(json_str)
        .map_err(|e| {
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

    Ok(convert_to_napi_list_data(cli_data))
}

// ============================================================================
// Changeset List - Parameter Validation
// ============================================================================

/// Validates changeset list command parameters.
///
/// Ensures the root path is valid and that optional filter/sort parameters
/// have valid values before executing the CLI command.
///
/// # Arguments
///
/// * `params` - The changeset list parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path
/// * `Err(ErrorInfo)` - Validation failed
///
/// # Errors
///
/// Returns an error if:
/// - The root path is empty, doesn't exist, or is not a directory
/// - The `filter_bump` parameter (if provided) is not a valid bump type
/// - The `sort` parameter (if provided) is not a valid sort option
pub(crate) fn validate_list_params(params: &ChangesetListParams) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    // Validate bump type filter if provided
    if let Some(ref bump) = params.filter_bump {
        validators::bump_type_info(bump)?;
    }

    // Validate sort option if provided
    if let Some(ref sort) = params.sort
        && !VALID_SORT_OPTIONS.contains(&sort.as_str())
    {
        return Err(ErrorInfo::validation(
            format!(
                "invalid sort option '{}'. Valid options are: {}",
                sort,
                VALID_SORT_OPTIONS.join(", ")
            ),
            Some("sort"),
        ));
    }

    Ok(PathBuf::from(&params.root))
}

/// Converts NAPI parameters to CLI arguments.
///
/// This function transforms the NAPI-friendly `ChangesetListParams` into the
/// CLI's `ChangesetListArgs` structure.
///
/// # Arguments
///
/// * `params` - The NAPI changeset list parameters
///
/// # Returns
///
/// A `ChangesetListArgs` instance ready for CLI execution.
pub(crate) fn convert_list_params_to_args(params: &ChangesetListParams) -> ChangesetListArgs {
    ChangesetListArgs {
        filter_package: params.filter_package.clone(),
        filter_bump: params.filter_bump.clone(),
        filter_env: params.filter_env.clone(),
        // Default to "date" if not provided (matches CLI default)
        sort: params.sort.clone().unwrap_or_else(|| "date".to_string()),
    }
}

// ============================================================================
// Changeset List - NAPI Function
// ============================================================================

/// List all pending changesets in the workspace.
///
/// Retrieves all pending (not yet released) changesets with optional filtering
/// by package, bump type, or environment. Results can be sorted by date, branch
/// name, or bump type.
///
/// @param params - Changeset list parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `configPath`: Optional custom config file path
///   - `filterPackage`: Optional filter by package name
///   - `filterBump`: Optional filter by bump type (major, minor, patch)
///   - `filterEnv`: Optional filter by environment
///   - `sort`: Sort order (date, branch, bump). Defaults to "date"
///
/// @returns `Promise<ChangesetListApiResponse>` containing:
///   - On success: `{ success: true, data: ChangesetListData }`
///   - On failure: `{ success: false, error: ErrorInfo }`
///
/// @example List all changesets
/// ```typescript
/// const result = await changesetList({
///   root: '/path/to/workspace'
/// });
///
/// if (result.success) {
///   console.log(`Found ${result.data.count} changeset(s)`);
///   for (const cs of result.data.changesets) {
///     console.log(`- ${cs.branch}: ${cs.bump}`);
///   }
/// }
/// ```
///
/// @example Filter by bump type
/// ```typescript
/// const result = await changesetList({
///   root: '/path/to/workspace',
///   filterBump: 'major'
/// });
///
/// if (result.success) {
///   console.log('Major version changesets:');
///   result.data.changesets.forEach(cs => {
///     console.log(`  ${cs.branch}: ${cs.packages.join(', ')}`);
///   });
/// }
/// ```
///
/// @example Filter by package
/// ```typescript
/// const result = await changesetList({
///   root: '/path/to/workspace',
///   filterPackage: '@scope/core'
/// });
///
/// if (result.success) {
///   console.log(`Changesets affecting @scope/core: ${result.data.count}`);
/// }
/// ```
///
/// @example Sort by branch name
/// ```typescript
/// const result = await changesetList({
///   root: '/path/to/workspace',
///   sort: 'branch'
/// });
/// ```
///
/// @example Filter by environment
/// ```typescript
/// const result = await changesetList({
///   root: '/path/to/workspace',
///   filterEnv: 'production'
/// });
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await changesetList({
///   root: '/nonexistent/path'
/// });
///
/// if (!result.success) {
///   switch (result.error.code) {
///     case 'ENOENT':
///       console.error('Path not found');
///       break;
///     case 'EVALIDATION':
///       console.error('Invalid parameters:', result.error.message);
///       break;
///     case 'ECONFIG':
///       console.error('Workspace not initialized');
///       break;
///     default:
///       console.error(`Error: ${result.error.message}`);
///   }
/// }
/// ```
#[napi(js_name = "changesetList")]
pub async fn changeset_list(params: ChangesetListParams) -> ChangesetListApiResponse {
    // 1. Validate parameters (synchronous validation before spawning)
    let root_path = match validate_list_params(&params) {
        Ok(path) => path,
        Err(error) => return ChangesetListApiResponse::failure(error),
    };

    // 2. Prepare config path
    let config_path: Option<PathBuf> = params.config_path.as_ref().map(PathBuf::from);

    // 3. Convert NAPI params to CLI args
    let args = convert_list_params_to_args(&params);

    // 4. Execute CLI command in a blocking task
    // The CLI's execute_list uses types that are not Send/Sync (RefCell, git2::Repository),
    // so we must run it on a blocking thread via spawn_blocking.
    let result = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        // This is necessary because execute_list is async but we're in a blocking context
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
            if let Err(cli_error) =
                execute_list(&args, &output, Some(root_path.as_path()), config_path.as_deref())
                    .await
            {
                return Err(ErrorInfo::from(cli_error));
            }

            // Extract and parse JSON
            let json_bytes = buffer.take_bytes();
            parse_changeset_list_response(&json_bytes)
        })
    })
    .await;

    // 5. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => ChangesetListApiResponse::success(data),
        Ok(Err(error)) => ChangesetListApiResponse::failure(error),
        Err(join_error) => ChangesetListApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}

// ============================================================================
// Changeset Show - CLI Response Types
// ============================================================================

/// CLI JSON response data structure for changeset show command.
///
/// This structure mirrors the `ChangesetShowResponse` from the CLI's show command,
/// used for deserializing the captured JSON output.
#[derive(Debug, Deserialize)]
pub(crate) struct CliChangesetShowResponseData {
    /// Whether the operation succeeded.
    #[allow(dead_code)]
    pub(crate) success: bool,

    /// The changeset details.
    pub(crate) changeset: CliChangesetShowItem,
}

/// Detailed changeset information from CLI show output.
///
/// Contains all fields returned by the changeset show command.
#[derive(Debug, Deserialize)]
pub(crate) struct CliChangesetShowItem {
    /// Branch name (also serves as unique identifier).
    pub(crate) branch: String,

    /// Version bump type (lowercase string).
    pub(crate) bump: String,

    /// List of affected packages.
    pub(crate) packages: Vec<String>,

    /// Target environments.
    pub(crate) environments: Vec<String>,

    /// List of commit IDs.
    pub(crate) commits: Vec<String>,

    /// Creation timestamp (RFC3339 format).
    pub(crate) created_at: String,

    /// Last update timestamp (RFC3339 format).
    pub(crate) updated_at: String,
}

// ============================================================================
// Changeset Show - Conversion Functions
// ============================================================================

/// Converts CLI changeset show item to NAPI changeset detail info.
///
/// Transforms the internal CLI response structure into the NAPI-compatible
/// `ChangesetDetailInfo` type for JavaScript consumption.
///
/// # Arguments
///
/// * `item` - The CLI changeset show item to convert
///
/// # Returns
///
/// A `ChangesetDetailInfo` with all fields populated from the CLI item.
pub(crate) fn convert_show_item_to_napi(item: CliChangesetShowItem) -> ChangesetDetailInfo {
    ChangesetDetailInfo::new(
        item.branch.clone(), // id is derived from branch
        item.branch,
        item.bump,
        item.created_at,
        item.updated_at,
    )
    .with_packages(item.packages)
    .with_environments(item.environments)
    .with_commits(item.commits)
}

/// Converts CLI response data to NAPI show data.
///
/// Transforms the parsed CLI JSON response into the `ChangesetShowData`
/// structure expected by JavaScript consumers.
///
/// # Arguments
///
/// * `data` - The CLI response data
///
/// # Returns
///
/// A `ChangesetShowData` containing the changeset details.
pub(crate) fn convert_to_napi_show_data(data: CliChangesetShowResponseData) -> ChangesetShowData {
    ChangesetShowData::new(convert_show_item_to_napi(data.changeset))
}

/// Parses the JSON output from the CLI changeset show command.
///
/// This function deserializes the raw bytes from the CLI's JSON output
/// into the appropriate NAPI response type.
///
/// # Arguments
///
/// * `json_bytes` - Raw bytes from the captured CLI output
///
/// # Returns
///
/// * `Ok(ChangesetShowData)` - Successfully parsed changeset details
/// * `Err(ErrorInfo)` - Parsing failed or CLI returned an error
///
/// # Errors
///
/// Returns an error if:
/// - The output is empty
/// - The output is not valid UTF-8
/// - The output is not valid JSON
/// - The CLI returned an error response
pub(crate) fn parse_changeset_show_response(
    json_bytes: &[u8],
) -> Result<ChangesetShowData, ErrorInfo> {
    // Handle empty output
    if json_bytes.is_empty() || json_bytes.iter().all(u8::is_ascii_whitespace) {
        return Err(ErrorInfo::execution("Empty response from CLI"));
    }

    // Convert bytes to string
    let json_str = std::str::from_utf8(json_bytes)
        .map_err(|e| ErrorInfo::execution(format!("Invalid UTF-8 in CLI output: {e}")))?;

    // Parse the outer JSON response structure
    let response: CliJsonResponse<CliChangesetShowResponseData> = serde_json::from_str(json_str)
        .map_err(|e| ErrorInfo::execution(format!("Failed to parse CLI JSON: {e}")))?;

    // Check if the CLI returned an error
    if !response.success {
        let error_msg = response.error.unwrap_or_else(|| "Unknown CLI error".to_string());
        return Err(ErrorInfo::execution(error_msg));
    }

    // Extract the data
    let data =
        response.data.ok_or_else(|| ErrorInfo::execution("CLI returned success but no data"))?;

    Ok(convert_to_napi_show_data(data))
}

// ============================================================================
// Changeset Show - Validation
// ============================================================================

/// Validates changeset show command parameters.
///
/// Ensures the root path is valid and the branch parameter is provided.
///
/// # Arguments
///
/// * `params` - The changeset show parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path
/// * `Err(ErrorInfo)` - Validation failed
///
/// # Errors
///
/// Returns an error if:
/// - The root path is empty, doesn't exist, or is not a directory
/// - The branch parameter is empty
pub(crate) fn validate_show_params(params: &ChangesetShowParams) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    // Validate branch is not empty
    if params.branch.trim().is_empty() {
        return Err(ErrorInfo::validation(
            "branch parameter is required and cannot be empty",
            Some("branch"),
        ));
    }

    Ok(PathBuf::from(&params.root))
}

/// Converts NAPI parameters to CLI arguments.
///
/// This function transforms the NAPI-friendly `ChangesetShowParams` into the
/// CLI's `ChangesetShowArgs` structure.
///
/// # Arguments
///
/// * `params` - The NAPI changeset show parameters
///
/// # Returns
///
/// A `ChangesetShowArgs` instance ready for CLI execution.
pub(crate) fn convert_show_params_to_args(params: &ChangesetShowParams) -> ChangesetShowArgs {
    ChangesetShowArgs { branch: params.branch.clone() }
}

// ============================================================================
// Changeset Show - NAPI Function
// ============================================================================

/// Show details of a specific changeset.
///
/// Retrieves detailed information about a specific changeset identified by
/// its branch name or changeset ID. Returns all metadata including packages,
/// environments, commits, and timestamps.
///
/// @param params - Changeset show parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `configPath`: Optional custom config file path
///   - `branch`: Branch name or changeset ID (required)
///
/// @returns `Promise<ChangesetShowApiResponse>` - Response containing:
///   - `success`: Whether the operation succeeded
///   - `data`: Changeset details if successful
///   - `error`: Error information if failed
///
/// ## Success Response
///
/// When successful, `data` contains:
/// - `changeset.id`: Unique changeset identifier
/// - `changeset.branch`: Git branch name
/// - `changeset.bump`: Version bump type
/// - `changeset.packages`: List of affected packages
/// - `changeset.environments`: Target environments
/// - `changeset.commits`: Associated commit hashes
/// - `changeset.createdAt`: Creation timestamp (ISO 8601)
/// - `changeset.updatedAt`: Last update timestamp (ISO 8601)
///
/// ## Error Codes
///
/// - `EVALIDATION`: Invalid parameters (empty root or branch)
/// - `ENOENT`: Path or changeset not found
/// - `ECONFIG`: Workspace not initialized
/// - `EEXECUTION`: CLI command failed
///
/// @example Basic usage
/// ```typescript
/// const result = await changesetShow({
///   root: '/path/to/workspace',
///   branch: 'feature/new-api'
/// });
///
/// if (result.success) {
///   const { changeset } = result.data;
///   console.log(`Changeset: ${changeset.branch}`);
///   console.log(`Bump: ${changeset.bump}`);
///   console.log(`Packages: ${changeset.packages.join(', ')}`);
///   console.log(`Created: ${changeset.createdAt}`);
/// }
/// ```
///
/// @example With custom config
/// ```typescript
/// const result = await changesetShow({
///   root: '/path/to/workspace',
///   configPath: '/path/to/custom.config.json',
///   branch: 'feature/auth-system'
/// });
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await changesetShow({
///   root: '/path/to/workspace',
///   branch: 'nonexistent-branch'
/// });
///
/// if (!result.success) {
///   switch (result.error.code) {
///     case 'ENOENT':
///       console.error('Changeset not found');
///       break;
///     case 'EVALIDATION':
///       console.error('Invalid parameters:', result.error.message);
///       break;
///     case 'ECONFIG':
///       console.error('Workspace not initialized');
///       break;
///     default:
///       console.error(`Error: ${result.error.message}`);
///   }
/// }
/// ```
#[napi(js_name = "changesetShow")]
pub async fn changeset_show(params: ChangesetShowParams) -> ChangesetShowApiResponse {
    // 1. Validate parameters (synchronous validation before spawning)
    let root_path = match validate_show_params(&params) {
        Ok(path) => path,
        Err(error) => return ChangesetShowApiResponse::failure(error),
    };

    // 2. Prepare config path
    let config_path: Option<PathBuf> = params.config_path.as_ref().map(PathBuf::from);

    // 3. Convert NAPI params to CLI args
    let args = convert_show_params_to_args(&params);

    // 4. Execute CLI command in a blocking task
    // The CLI's execute_show uses types that are not Send/Sync (RefCell, git2::Repository),
    // so we must run it on a blocking thread via spawn_blocking.
    let result = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        // This is necessary because execute_show is async but we're in a blocking context
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
            if let Err(cli_error) =
                execute_show(&args, &output, Some(root_path.as_path()), config_path.as_deref())
                    .await
            {
                return Err(ErrorInfo::from(cli_error));
            }

            // Extract and parse JSON
            let json_bytes = buffer.take_bytes();
            parse_changeset_show_response(&json_bytes)
        })
    })
    .await;

    // 5. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => ChangesetShowApiResponse::success(data),
        Ok(Err(error)) => ChangesetShowApiResponse::failure(error),
        Err(join_error) => ChangesetShowApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}

// ============================================================================
// Changeset Remove - CLI Response Types
// ============================================================================

/// CLI changeset remove response data structure.
///
/// Mirrors the `ChangesetRemoveResponse` structure from the CLI's changeset remove command.
/// Used for deserializing the JSON output captured from the CLI.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct CliChangesetRemoveResponseData {
    /// Whether the operation succeeded.
    #[serde(default)]
    pub(crate) success: bool,
    /// The branch name that was removed.
    pub(crate) branch: String,
    /// Whether the changeset was archived before removal.
    #[serde(default)]
    pub(crate) archived: bool,
    /// Details of the removed changeset.
    pub(crate) changeset: CliRemovedChangesetInfo,
}

/// CLI removed changeset info structure.
///
/// Contains details about the changeset that was removed.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct CliRemovedChangesetInfo {
    /// Branch name (also serves as unique identifier).
    pub(crate) branch: String,
    /// Version bump type.
    pub(crate) bump: String,
    /// List of affected packages.
    #[serde(default)]
    pub(crate) packages: Vec<String>,
    /// Target environments.
    #[serde(default)]
    pub(crate) environments: Vec<String>,
    /// Number of commits.
    #[serde(default)]
    pub(crate) commit_count: usize,
}

// ============================================================================
// Changeset Remove - Conversion Functions
// ============================================================================

/// Converts CLI remove response data to NAPI `ChangesetRemoveData`.
///
/// This function transforms the deserialized CLI response into the
/// NAPI-compatible `ChangesetRemoveData` structure.
///
/// # Arguments
///
/// * `cli_data` - The CLI response data from the changeset remove command
///
/// # Returns
///
/// A `ChangesetRemoveData` instance with the converted data.
pub(crate) fn convert_to_napi_remove_data(
    cli_data: &CliChangesetRemoveResponseData,
) -> ChangesetRemoveData {
    ChangesetRemoveData::new(true, &cli_data.branch)
}

/// Parses the JSON bytes from CLI output into `ChangesetRemoveData`.
///
/// This function handles:
/// - Empty or whitespace-only output
/// - Invalid UTF-8 encoding
/// - Invalid JSON format
/// - CLI error responses
/// - Successful responses with data
///
/// # Arguments
///
/// * `json_bytes` - Raw bytes captured from CLI output
///
/// # Returns
///
/// * `Ok(ChangesetRemoveData)` - Successfully parsed response
/// * `Err(ErrorInfo)` - Error details if parsing failed
///
/// # Errors
///
/// Returns an error if:
/// - The output is empty
/// - The output is not valid UTF-8
/// - The output is not valid JSON
/// - The CLI returned an error response
/// - The success response has no data
pub(crate) fn parse_changeset_remove_response(
    json_bytes: &[u8],
) -> Result<ChangesetRemoveData, ErrorInfo> {
    // Check for empty output
    let json_str = match String::from_utf8(json_bytes.to_vec()) {
        Ok(s) if s.trim().is_empty() => {
            return Err(ErrorInfo::execution("CLI returned empty output"));
        }
        Ok(s) => s,
        Err(e) => {
            return Err(ErrorInfo::execution(format!("Invalid UTF-8 in CLI output: {e}")));
        }
    };

    // Parse JSON
    let response: CliJsonResponse<CliChangesetRemoveResponseData> = serde_json::from_str(&json_str)
        .map_err(|e| ErrorInfo::execution(format!("Failed to parse CLI JSON response: {e}")))?;

    // Check for CLI error
    if !response.success {
        let error_message = response.error.unwrap_or_else(|| "Unknown CLI error".to_string());
        return Err(ErrorInfo::execution(error_message));
    }

    // Extract data
    match response.data {
        Some(data) => Ok(convert_to_napi_remove_data(&data)),
        None => Err(ErrorInfo::execution("CLI returned success but no data")),
    }
}

// ============================================================================
// Changeset Remove - Validation
// ============================================================================

/// Validates the parameters for the changeset remove command.
///
/// Performs the following validations:
/// - Root path exists and is a directory
/// - Branch parameter is not empty
///
/// # Arguments
///
/// * `params` - The changeset remove parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path as a `PathBuf`
/// * `Err(ErrorInfo)` - Validation error with details
///
/// # Errors
///
/// Returns an error if:
/// - The root path is empty, doesn't exist, or is not a directory
/// - The branch parameter is empty
pub(crate) fn validate_remove_params(params: &ChangesetRemoveParams) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    // Validate branch is not empty
    if params.branch.trim().is_empty() {
        return Err(ErrorInfo::validation(
            "branch parameter is required and cannot be empty",
            Some("branch"),
        ));
    }

    Ok(PathBuf::from(&params.root))
}

/// Converts NAPI parameters to CLI arguments.
///
/// This function transforms the NAPI-friendly `ChangesetRemoveParams` into the
/// CLI's `ChangesetDeleteArgs` structure.
///
/// **Note**: The `force` flag is always set to `true` in the API layer because
/// there is no interactive confirmation prompt available in programmatic usage.
/// The caller is expected to handle confirmation in their own application if needed.
///
/// # Arguments
///
/// * `params` - The NAPI changeset remove parameters
///
/// # Returns
///
/// A `ChangesetDeleteArgs` instance ready for CLI execution.
pub(crate) fn convert_remove_params_to_args(params: &ChangesetRemoveParams) -> ChangesetDeleteArgs {
    ChangesetDeleteArgs {
        branch: params.branch.clone(),
        // Always force in API mode - no interactive prompts available
        force: true,
    }
}

// ============================================================================
// Changeset Remove - NAPI Function
// ============================================================================

/// Remove a changeset from the workspace.
///
/// Deletes a changeset identified by its branch name. The changeset is archived
/// before deletion for recovery purposes. In API mode, the operation always
/// proceeds without confirmation (equivalent to `--force` flag in CLI).
///
/// @param params - Changeset remove parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `configPath`: Optional custom config file path
///   - `branch`: Branch name or changeset ID to remove (required)
///   - `force`: Ignored in API mode (always treated as true)
///
/// @returns `Promise<ChangesetRemoveApiResponse>` - Response containing:
///   - `success`: Whether the operation succeeded
///   - `data`: Removal confirmation if successful
///   - `error`: Error information if failed
///
/// ## Success Response
///
/// When successful, `data` contains:
/// - `removed`: Boolean indicating the changeset was removed (always true on success)
/// - `branch`: The branch name that was removed
///
/// ## Behavior Notes
///
/// - The changeset is archived before deletion for potential recovery
/// - The archive includes a marker indicating manual deletion (not a release)
/// - In API mode, no confirmation prompt is shown (force mode is implicit)
///
/// ## Error Codes
///
/// - `EVALIDATION`: Invalid parameters (empty root or branch)
/// - `ENOENT`: Path or changeset not found
/// - `ECONFIG`: Workspace not initialized
/// - `EEXECUTION`: CLI command failed
///
/// @example Basic usage
/// ```typescript
/// const result = await changesetRemove({
///   root: '/path/to/workspace',
///   branch: 'feature/abandoned-work'
/// });
///
/// if (result.success) {
///   console.log(`Removed changeset: ${result.data.branch}`);
/// } else {
///   console.error(`Error: ${result.error.message}`);
/// }
/// ```
///
/// @example With custom config
/// ```typescript
/// const result = await changesetRemove({
///   root: '/path/to/workspace',
///   configPath: '/path/to/custom.config.json',
///   branch: 'feature/obsolete'
/// });
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await changesetRemove({
///   root: '/path/to/workspace',
///   branch: 'nonexistent-branch'
/// });
///
/// if (!result.success) {
///   switch (result.error.code) {
///     case 'ENOENT':
///       console.error('Changeset not found');
///       break;
///     case 'EVALIDATION':
///       console.error('Invalid parameters:', result.error.message);
///       break;
///     case 'ECONFIG':
///       console.error('Workspace not initialized');
///       break;
///     default:
///       console.error(`Error: ${result.error.message}`);
///   }
/// }
/// ```
///
/// @example Cleanup workflow
/// ```typescript
/// // List all changesets, then remove stale ones
/// const listResult = await changesetList({ root: '.' });
///
/// if (listResult.success) {
///   for (const changeset of listResult.data.changesets) {
///     // Check if changeset is older than 30 days
///     const createdAt = new Date(changeset.createdAt);
///     const thirtyDaysAgo = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);
///
///     if (createdAt < thirtyDaysAgo) {
///       const removeResult = await changesetRemove({
///         root: '.',
///         branch: changeset.branch
///       });
///
///       if (removeResult.success) {
///         console.log(`Removed stale changeset: ${changeset.branch}`);
///       }
///     }
///   }
/// }
/// ```
#[napi(js_name = "changesetRemove")]
pub async fn changeset_remove(params: ChangesetRemoveParams) -> ChangesetRemoveApiResponse {
    // 1. Validate parameters (synchronous validation before spawning)
    let root_path = match validate_remove_params(&params) {
        Ok(path) => path,
        Err(error) => return ChangesetRemoveApiResponse::failure(error),
    };

    // 2. Prepare config path
    let config_path: Option<PathBuf> = params.config_path.as_ref().map(PathBuf::from);

    // 3. Convert NAPI params to CLI args
    let args = convert_remove_params_to_args(&params);

    // 4. Execute CLI command in a blocking task
    // The CLI's execute_remove uses types that are not Send/Sync (RefCell, git2::Repository),
    // so we must run it on a blocking thread via spawn_blocking.
    let result = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        // This is necessary because execute_remove is async but we're in a blocking context
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
            if let Err(cli_error) =
                execute_remove(&args, &output, Some(root_path.as_path()), config_path.as_deref())
                    .await
            {
                return Err(ErrorInfo::from(cli_error));
            }

            // Extract and parse JSON
            let json_bytes = buffer.take_bytes();
            parse_changeset_remove_response(&json_bytes)
        })
    })
    .await;

    // 5. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => ChangesetRemoveApiResponse::success(data),
        Ok(Err(error)) => ChangesetRemoveApiResponse::failure(error),
        Err(join_error) => ChangesetRemoveApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}

// ============================================================================
// Changeset History - CLI Response Types
// ============================================================================

/// CLI JSON response data structure for changeset history command.
///
/// This structure mirrors the `ChangesetHistoryResponse` from the CLI's history command,
/// used for deserializing the captured JSON output.
#[derive(Debug, Deserialize)]
pub(crate) struct CliChangesetHistoryResponseData {
    /// Whether the operation succeeded.
    #[allow(dead_code)]
    pub(crate) success: bool,

    /// List of archived changesets.
    pub(crate) changesets: Vec<CliArchivedChangesetInfo>,

    /// Total count of results.
    #[allow(dead_code)]
    pub(crate) total: usize,
}

/// Archived changeset information from CLI history output.
///
/// Contains both changeset details and release information.
#[derive(Debug, Deserialize)]
pub(crate) struct CliArchivedChangesetInfo {
    /// Branch name (also serves as unique identifier).
    pub(crate) branch: String,

    /// Version bump type (major, minor, patch, none).
    pub(crate) bump: String,

    /// List of affected packages.
    pub(crate) packages: Vec<String>,

    /// Target environments.
    pub(crate) environments: Vec<String>,

    /// List of commit IDs.
    pub(crate) commits: Vec<String>,

    /// Changeset creation timestamp (RFC3339 format).
    pub(crate) created_at: String,

    /// Changeset last update timestamp (RFC3339 format).
    pub(crate) updated_at: String,

    /// Package versions map (package name -> version).
    pub(crate) versions: std::collections::HashMap<String, String>,

    /// Git commit hash of the release.
    pub(crate) git_commit: String,

    /// Release timestamp (RFC3339 format).
    pub(crate) applied_at: String,

    /// User/system that performed the release.
    pub(crate) applied_by: String,
}

// ============================================================================
// Changeset History - Conversion Functions
// ============================================================================

/// Converts a CLI archived changeset info to NAPI `ArchivedChangesetInfo`.
///
/// Maps the flat CLI structure to the nested NAPI structure with separate
/// changeset details and release information sections.
///
/// # Arguments
///
/// * `cli` - The CLI archived changeset info to convert
///
/// # Returns
///
/// A NAPI-compatible `ArchivedChangesetInfo` instance.
pub(crate) fn convert_archived_changeset_to_napi(
    cli: CliArchivedChangesetInfo,
) -> ArchivedChangesetInfo {
    // Convert versions HashMap to Vec<ReleasedVersionEntry>
    let released_versions: Vec<ReleasedVersionEntry> = cli
        .versions
        .into_iter()
        .map(|(package_name, version)| ReleasedVersionEntry::new(package_name, version))
        .collect();

    // Build the changeset detail info
    let changeset = ChangesetDetailInfo::new(
        cli.branch.clone(),
        cli.branch.clone(),
        cli.bump,
        cli.created_at,
        cli.updated_at,
    )
    .with_packages(cli.packages)
    .with_environments(cli.environments)
    .with_commits(cli.commits);

    // Build the release info
    let release_info =
        ReleaseInfoData::new(cli.applied_at, cli.applied_by, cli.git_commit, released_versions);

    ArchivedChangesetInfo::new(changeset, release_info)
}

/// Converts CLI history response to NAPI `ChangesetHistoryData`.
///
/// # Arguments
///
/// * `cli_data` - The CLI response data to convert
///
/// # Returns
///
/// A NAPI-compatible `ChangesetHistoryData` instance.
pub(crate) fn convert_to_napi_history_data(
    cli_data: CliChangesetHistoryResponseData,
) -> ChangesetHistoryData {
    let archived: Vec<ArchivedChangesetInfo> =
        cli_data.changesets.into_iter().map(convert_archived_changeset_to_napi).collect();

    ChangesetHistoryData::new(archived)
}

// ============================================================================
// Changeset History - Response Parsing
// ============================================================================

/// Parses the JSON output from the CLI history command.
///
/// Handles both success and error responses from the CLI, converting them
/// to the appropriate NAPI types.
///
/// # Arguments
///
/// * `json_bytes` - Raw bytes from the CLI output buffer
///
/// # Returns
///
/// A `ChangesetHistoryData` on success, or an `ErrorInfo` on failure.
///
/// # Errors
///
/// Returns an error if:
/// - The output buffer is empty
/// - The bytes cannot be converted to UTF-8
/// - The JSON cannot be parsed
/// - The CLI returned an error response
pub(crate) fn parse_changeset_history_response(
    json_bytes: &[u8],
) -> Result<ChangesetHistoryData, ErrorInfo> {
    // Check for empty output
    if json_bytes.is_empty() {
        return Err(ErrorInfo::execution("CLI returned empty output"));
    }

    // Convert bytes to string
    let json_str = std::str::from_utf8(json_bytes)
        .map_err(|e| ErrorInfo::execution(format!("Invalid UTF-8 in CLI output: {e}")))?;

    // Trim whitespace
    let json_str = json_str.trim();
    if json_str.is_empty() {
        return Err(ErrorInfo::execution("CLI returned empty output"));
    }

    // Parse the JSON response
    let cli_response: CliJsonResponse<CliChangesetHistoryResponseData> =
        serde_json::from_str(json_str)
            .map_err(|e| ErrorInfo::execution(format!("Failed to parse CLI JSON output: {e}")))?;

    // Check for CLI-level error
    if !cli_response.success {
        let error_msg = cli_response.error.unwrap_or_else(|| String::from("Unknown CLI error"));
        return Err(ErrorInfo::execution(error_msg));
    }

    // Extract data or return error
    let cli_data = cli_response
        .data
        .ok_or_else(|| ErrorInfo::execution("CLI returned success but no data"))?;

    // Convert to NAPI types
    Ok(convert_to_napi_history_data(cli_data))
}

// ============================================================================
// Changeset History - Validation
// ============================================================================

/// Validates the history command parameters.
///
/// Checks that:
/// - The root path is not empty
/// - The root path exists and is a directory
/// - If filter_bump is provided, it's a valid bump type
///
/// # Arguments
///
/// * `params` - The history parameters to validate
///
/// # Returns
///
/// The validated root path as a `PathBuf` on success, or an `ErrorInfo` on failure.
pub(crate) fn validate_history_params(
    params: &ChangesetHistoryParams,
) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    // Validate bump type if provided
    if let Some(ref bump) = params.filter_bump {
        validators::bump_type_info(bump)?;
    }

    Ok(PathBuf::from(&params.root))
}

// ============================================================================
// Changeset History - Parameter Conversion
// ============================================================================

/// Converts NAPI history params to CLI args.
///
/// Maps the JavaScript-friendly parameter names to the CLI argument structure.
///
/// # Arguments
///
/// * `params` - The NAPI parameters to convert
///
/// # Returns
///
/// A `ChangesetHistoryArgs` instance ready for CLI execution.
pub(crate) fn convert_history_params_to_args(
    params: &ChangesetHistoryParams,
) -> ChangesetHistoryArgs {
    ChangesetHistoryArgs {
        filter_package: params.filter_package.clone(),
        filter_env: params.filter_env.clone(),
        filter_bump: params.filter_bump.clone(),
        since: params.since.clone(),
        until: params.until.clone(),
        limit: params.limit.map(|l| l as usize),
    }
}

// ============================================================================
// Changeset History - NAPI Function
// ============================================================================

/// Queries the changeset history with optional filtering.
///
/// This function queries archived changesets from the workspace history,
/// supporting various filter options for package, environment, bump type,
/// date range, and result limit.
///
/// # Parameters
///
/// - `root`: Workspace root directory path (required)
/// - `configPath`: Optional path to custom configuration file
/// - `filterPackage`: Filter by package name
/// - `filterEnv`: Filter by environment
/// - `filterBump`: Filter by bump type (major, minor, patch)
/// - `since`: Start date filter (ISO 8601 format)
/// - `until`: End date filter (ISO 8601 format)
/// - `limit`: Maximum number of results
///
/// # Returns
///
/// An `ApiResponse` containing `ChangesetHistoryData` with the list of archived
/// changesets matching the query, or an error if the operation fails.
///
/// ## Success Response
///
/// ```typescript
/// {
///   success: true,
///   data: {
///     archived: [
///       {
///         changeset: {
///           id: "feature/add-api",
///           branch: "feature/add-api",
///           bump: "minor",
///           packages: ["@scope/core"],
///           environments: ["production"],
///           commits: ["abc123"],
///           createdAt: "2024-01-15T10:30:00Z",
///           updatedAt: "2024-01-15T14:45:00Z"
///         },
///         releaseInfo: {
///           releasedAt: "2024-01-16T10:00:00Z",
///           releasedBy: "CI",
///           releaseCommit: "def456",
///           releasedVersions: [
///             { packageName: "@scope/core", version: "2.0.0" }
///           ]
///         }
///       }
///     ],
///     count: 1
///   }
/// }
/// ```
///
/// ## Error Codes
///
/// - `EVALIDATION`: Invalid parameters (empty root or invalid bump type)
/// - `ENOENT`: Path not found
/// - `ECONFIG`: Workspace not initialized
/// - `EEXECUTION`: CLI command failed
///
/// @example Basic usage - get all history
/// ```typescript
/// const result = await changesetHistory({
///   root: '/path/to/workspace'
/// });
///
/// if (result.success) {
///   console.log(`Found ${result.data.count} archived changesets`);
///   for (const item of result.data.archived) {
///     console.log(`- ${item.changeset.branch}: ${item.changeset.bump}`);
///     console.log(`  Released: ${item.releaseInfo.releasedAt}`);
///   }
/// }
/// ```
///
/// @example Filter by package
/// ```typescript
/// const result = await changesetHistory({
///   root: '/path/to/workspace',
///   filterPackage: '@scope/core'
/// });
///
/// if (result.success) {
///   console.log(`Releases for @scope/core: ${result.data.count}`);
/// }
/// ```
///
/// @example Filter by date range
/// ```typescript
/// const result = await changesetHistory({
///   root: '/path/to/workspace',
///   since: '2024-01-01',
///   until: '2024-12-31',
///   limit: 10
/// });
/// ```
///
/// @example Filter by bump type
/// ```typescript
/// const result = await changesetHistory({
///   root: '/path/to/workspace',
///   filterBump: 'major'
/// });
///
/// if (result.success) {
///   console.log('Major releases:');
///   result.data.archived.forEach(item => {
///     const versions = item.releaseInfo.releasedVersions
///       .map(v => `${v.packageName}@${v.version}`)
///       .join(', ');
///     console.log(`  ${item.changeset.branch}: ${versions}`);
///   });
/// }
/// ```
///
/// @example Multiple filters
/// ```typescript
/// const result = await changesetHistory({
///   root: '/path/to/workspace',
///   filterPackage: '@scope/core',
///   filterEnv: 'production',
///   filterBump: 'minor',
///   since: '2024-06-01',
///   limit: 5
/// });
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await changesetHistory({
///   root: '/nonexistent/path'
/// });
///
/// if (!result.success) {
///   switch (result.error.code) {
///     case 'ENOENT':
///       console.error('Path not found');
///       break;
///     case 'EVALIDATION':
///       console.error('Invalid parameters:', result.error.message);
///       break;
///     case 'ECONFIG':
///       console.error('Workspace not initialized');
///       break;
///     default:
///       console.error(`Error: ${result.error.message}`);
///   }
/// }
/// ```
#[napi(js_name = "changesetHistory")]
pub async fn changeset_history(params: ChangesetHistoryParams) -> ChangesetHistoryApiResponse {
    // 1. Validate parameters (synchronous validation before spawning)
    let root_path = match validate_history_params(&params) {
        Ok(path) => path,
        Err(error) => return ChangesetHistoryApiResponse::failure(error),
    };

    // 2. Prepare config path
    let config_path: Option<PathBuf> = params.config_path.as_ref().map(PathBuf::from);

    // 3. Convert NAPI params to CLI args
    let args = convert_history_params_to_args(&params);

    // 4. Execute CLI command in a blocking task
    // The CLI's execute_history uses types that are not Send/Sync (RefCell, git2::Repository),
    // so we must run it on a blocking thread via spawn_blocking.
    let result = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        // This is necessary because execute_history is async but we're in a blocking context
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
            if let Err(cli_error) =
                execute_history(&args, &output, Some(root_path.as_path()), config_path.as_deref())
                    .await
            {
                return Err(ErrorInfo::from(cli_error));
            }

            // Extract and parse JSON
            let json_bytes = buffer.take_bytes();
            parse_changeset_history_response(&json_bytes)
        })
    })
    .await;

    // 5. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => ChangesetHistoryApiResponse::success(data),
        Ok(Err(error)) => ChangesetHistoryApiResponse::failure(error),
        Err(join_error) => ChangesetHistoryApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}
