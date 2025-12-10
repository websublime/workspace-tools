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
//! import { changesetAdd } from '@websublime/workspace-tools';
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
//! ```

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use napi_derive::napi;
use serde::Deserialize;

use crate::error::ErrorInfo;
use crate::types::changeset::{ChangesetAddApiResponse, ChangesetAddData, ChangesetAddParams};
use crate::validation::validators;

use sublime_cli_tools::cli::commands::ChangesetCreateArgs;
use sublime_cli_tools::commands::changeset::execute_add;
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
pub(crate) fn parse_changeset_add_response(json_bytes: &[u8]) -> Result<ChangesetAddData, ErrorInfo> {
    // Convert bytes to string first for better error messages
    let json_str = std::str::from_utf8(json_bytes)
        .map_err(|e| ErrorInfo::execution(format!("Invalid UTF-8 in CLI response: {e}")))?;

    // Handle empty response
    if json_str.trim().is_empty() {
        return Err(ErrorInfo::execution("CLI returned empty response"));
    }

    // Parse the JSON response
    let response: CliJsonResponse<CliChangesetAddResponseData> =
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
            if let Err(cli_error) =
                execute_add(&args, &output, Some(root_path), config_path).await
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
