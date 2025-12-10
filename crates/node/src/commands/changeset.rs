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
//! ```

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use napi_derive::napi;
use serde::Deserialize;

use crate::error::ErrorInfo;
use crate::types::changeset::{
    ChangesetAddApiResponse, ChangesetAddData, ChangesetAddParams, ChangesetDetailInfo,
    ChangesetUpdateApiResponse, ChangesetUpdateData, ChangesetUpdateParams, UpdateSummaryInfo,
};
use crate::validation::validators;

use sublime_cli_tools::cli::commands::{ChangesetCreateArgs, ChangesetUpdateArgs};
use sublime_cli_tools::commands::changeset::{execute_add, execute_update};
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
