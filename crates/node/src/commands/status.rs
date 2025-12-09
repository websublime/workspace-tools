//! Status command implementation for Node.js bindings.
//!
//! # What
//!
//! This module implements the `status` NAPI function that retrieves comprehensive
//! workspace information, including repository type, package manager, Git branch,
//! pending changesets, and all workspace packages.
//!
//! # How
//!
//! The implementation follows this flow:
//!
//! 1. **Parameter validation**: Validates the `root` path exists and is a directory
//! 2. **Output capture**: Uses a `SharedBuffer` wrapper around `Arc<Mutex<Vec<u8>>>`
//!    to capture the CLI's JSON output without modifying the CLI crate
//! 3. **CLI execution**: Calls `execute_status` from `sublime_cli_tools` with JSON format
//! 4. **Response parsing**: Parses the JSON output using serde into intermediate types
//! 5. **Type conversion**: Converts CLI response types to NAPI-compatible types
//! 6. **Result wrapping**: Returns an `ApiResponse<StatusData>` for consistent error handling
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
//! The status command is a fundamental operation for workspace management,
//! providing a quick overview of the workspace state. It's often the first
//! command users run to understand their workspace configuration.
//!
//! Key use cases:
//! - Verifying workspace setup before running operations
//! - CI/CD pipeline workspace validation
//! - IDE integration for workspace information display
//! - Scripting and automation scenarios
//!
//! # Examples
//!
//! ## TypeScript Usage
//!
//! ```typescript
//! import { status } from '@websublime/workspace-tools';
//!
//! const result = await status({ root: '.' });
//!
//! if (result.success) {
//!   console.log(`Repository: ${result.data.repository.kind}`);
//!   console.log(`Package Manager: ${result.data.packageManager.name}`);
//!
//!   if (result.data.branch) {
//!     console.log(`Branch: ${result.data.branch.name}`);
//!   }
//!
//!   console.log(`Packages: ${result.data.packages.length}`);
//!   for (const pkg of result.data.packages) {
//!     console.log(`  - ${pkg.name}@${pkg.version} (${pkg.path})`);
//!   }
//! } else {
//!   console.error(`Error [${result.error.code}]: ${result.error.message}`);
//! }
//! ```
//!
//! ## Error Handling
//!
//! ```typescript
//! const result = await status({ root: '/nonexistent/path' });
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
use crate::types::status::{
    BranchInfo, ChangesetInfo, PackageInfo, PackageManagerInfo, RepositoryInfo, StatusApiResponse,
    StatusData, StatusParams,
};
use crate::validation::validators;

use sublime_cli_tools::cli::commands::StatusArgs;
use sublime_cli_tools::commands::status::execute_status;
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

/// CLI JSON response wrapper for status command.
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

/// CLI status response data structure.
///
/// Mirrors the `StatusJsonResponse` structure from the CLI's status command.
/// Field names use camelCase to match the JSON output format.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliStatusData {
    /// Repository information.
    pub(crate) repository: CliRepositoryInfo,
    /// Package manager information.
    pub(crate) package_manager: CliPackageManagerInfo,
    /// Current Git branch (optional).
    pub(crate) branch: Option<CliBranchInfo>,
    /// List of pending changesets.
    pub(crate) changesets: Vec<CliChangesetInfo>,
    /// List of workspace packages.
    pub(crate) packages: Vec<CliPackageInfo>,
}

/// CLI repository information.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliRepositoryInfo {
    /// Repository kind: "simple", "monorepo", or "unknown".
    pub(crate) kind: String,
    /// Monorepo type if applicable.
    pub(crate) monorepo_type: Option<String>,
}

/// CLI package manager information.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliPackageManagerInfo {
    /// Package manager name.
    pub(crate) name: String,
    /// Lock file name.
    pub(crate) lock_file: String,
}

/// CLI branch information.
#[derive(Debug, Deserialize)]
pub(crate) struct CliBranchInfo {
    /// Branch name.
    pub(crate) name: String,
}

/// CLI changeset information.
#[derive(Debug, Deserialize)]
pub(crate) struct CliChangesetInfo {
    /// Changeset ID.
    pub(crate) id: String,
}

/// CLI package information.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliPackageInfo {
    /// Package name.
    pub(crate) name: String,
    /// Package version.
    pub(crate) version: String,
    /// Package path.
    pub(crate) path: String,
}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Converts CLI status data to NAPI-compatible `StatusData`.
///
/// This function performs a straightforward field-by-field conversion from
/// the CLI's internal types to the NAPI types exposed to JavaScript.
///
/// # Arguments
///
/// * `cli_data` - The parsed CLI response data
///
/// # Returns
///
/// A `StatusData` instance suitable for returning to JavaScript.
pub(crate) fn convert_to_napi_status(cli_data: CliStatusData) -> StatusData {
    StatusData {
        repository: RepositoryInfo {
            kind: cli_data.repository.kind,
            monorepo_type: cli_data.repository.monorepo_type,
        },
        package_manager: PackageManagerInfo {
            name: cli_data.package_manager.name,
            lock_file: cli_data.package_manager.lock_file,
        },
        branch: cli_data.branch.map(|b| BranchInfo { name: b.name }),
        changesets: cli_data
            .changesets
            .into_iter()
            .map(|c| ChangesetInfo { id: c.id })
            .collect(),
        packages: cli_data
            .packages
            .into_iter()
            .map(|p| PackageInfo { name: p.name, version: p.version, path: p.path })
            .collect(),
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
/// * `Ok(StatusData)` - Successfully parsed and converted status data
/// * `Err(ErrorInfo)` - Parsing failed or CLI returned an error
///
/// # Errors
///
/// Returns an error if:
/// - The JSON is malformed or cannot be parsed
/// - The CLI returned `success: false` with an error message
/// - The CLI returned `success: true` but `data` is missing
pub(crate) fn parse_status_response(json_bytes: &[u8]) -> Result<StatusData, ErrorInfo> {
    // Convert bytes to string first for better error messages
    let json_str = std::str::from_utf8(json_bytes).map_err(|e| {
        ErrorInfo::execution(format!("Invalid UTF-8 in CLI response: {e}"))
    })?;

    // Handle empty response
    if json_str.trim().is_empty() {
        return Err(ErrorInfo::execution("CLI returned empty response"));
    }

    // Parse the JSON response
    let response: CliJsonResponse<CliStatusData> =
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
    let cli_data = response.data.ok_or_else(|| {
        ErrorInfo::execution("CLI returned success but no data")
    })?;

    Ok(convert_to_napi_status(cli_data))
}

// ============================================================================
// Parameter Validation
// ============================================================================

/// Validates status command parameters.
///
/// Ensures the root path is valid before executing the CLI command.
///
/// # Arguments
///
/// * `params` - The status parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path
/// * `Err(ErrorInfo)` - Validation failed
pub(crate) fn validate_params(params: &StatusParams) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    Ok(PathBuf::from(&params.root))
}

// ============================================================================
// NAPI Function
// ============================================================================

/// Get workspace status information.
///
/// Returns comprehensive information about the workspace including repository type,
/// package manager, Git branch, pending changesets, and all workspace packages.
///
/// This function is the main entry point for Node.js applications to retrieve
/// workspace status. It handles all the complexity of CLI invocation and response
/// parsing internally.
///
/// @param params - Status parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `configPath`: Optional custom config file path
///
/// @returns `Promise<ApiResponse<StatusData>>` containing:
///   - On success: `{ success: true, data: StatusData }`
///   - On failure: `{ success: false, error: ErrorInfo }`
///
/// @example Basic usage
/// ```typescript
/// const result = await status({ root: '/path/to/project' });
/// if (result.success) {
///   console.log(`Found ${result.data.packages.length} packages`);
///   console.log(`Package manager: ${result.data.packageManager.name}`);
/// } else {
///   console.error(`Error: ${result.error.code} - ${result.error.message}`);
/// }
/// ```
///
/// @example With custom config path
/// ```typescript
/// const result = await status({
///   root: '/path/to/project',
///   configPath: '/path/to/custom/repo.config.json'
/// });
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await status({ root: '/nonexistent' });
/// if (!result.success) {
///   if (result.error.code === 'ENOENT') {
///     console.error('Path not found');
///   } else if (result.error.code === 'EVALIDATION') {
///     console.error('Invalid parameters');
///   }
/// }
/// ```
#[napi]
pub async fn status(params: StatusParams) -> StatusApiResponse {
    // 1. Validate parameters (synchronous validation before spawning)
    let root_path = match validate_params(&params) {
        Ok(path) => path,
        Err(error) => return StatusApiResponse::failure(error),
    };

    // 2. Prepare config path
    let config_path: Option<PathBuf> = params.config_path.as_ref().map(PathBuf::from);

    // 3. Execute CLI command in a blocking task
    // The CLI's execute_status uses types that are not Send/Sync (RefCell, git2::Repository),
    // so we must run it on a blocking thread via spawn_blocking.
    let result = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        // This is necessary because execute_status is async but we're in a blocking context
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                return Err(ErrorInfo::execution(format!(
                    "Failed to create runtime: {e}"
                )));
            }
        };

        rt.block_on(async {
            // Create shared buffer for output capture
            let buffer = SharedBuffer::new();

            // Create Output with JSON format
            let output = Output::new(OutputFormat::Json, buffer.clone(), true);

            // Create status args
            let args = StatusArgs {};

            // Execute the CLI command
            let config_path_ref: Option<&Path> = config_path.as_deref();
            if let Err(cli_error) =
                execute_status(&args, &output, &root_path, config_path_ref).await
            {
                return Err(ErrorInfo::from(cli_error));
            }

            // Extract and parse JSON
            let json_bytes = buffer.take_bytes();
            parse_status_response(&json_bytes)
        })
    })
    .await;

    // 4. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => StatusApiResponse::success(data),
        Ok(Err(error)) => StatusApiResponse::failure(error),
        Err(join_error) => StatusApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}
