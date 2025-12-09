//! Init command implementation for Node.js bindings.
//!
//! # What
//!
//! This module implements the `init` NAPI function that initializes a new workspace
//! configuration (repo.config) in the specified directory. It creates the configuration
//! file, sets up the changeset directory structure, and configures versioning settings.
//!
//! # How
//!
//! The implementation follows this flow:
//!
//! 1. **Parameter validation**: Validates the `root` path exists and is a directory,
//!    validates optional `strategy` and `configFormat` parameters against allowed values
//! 2. **Output capture**: Uses a `SharedBuffer` wrapper around `Arc<Mutex<Vec<u8>>>`
//!    to capture the CLI's JSON output without modifying the CLI crate
//! 3. **CLI execution**: Calls `execute_init` from `sublime_cli_tools` with JSON format
//!    and `non_interactive = true` (programmatic API must not prompt for input)
//! 4. **Response parsing**: Parses the JSON output using serde into intermediate types
//! 5. **Type conversion**: Converts CLI response types to NAPI-compatible types
//! 6. **Result wrapping**: Returns an `InitApiResponse` for consistent error handling
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
//! The init command is typically the first command run when setting up a new
//! workspace. It creates the configuration file (`repo.config`) that controls:
//!
//! - Versioning strategy (independent or unified)
//! - Changeset directory location
//! - Environment configurations
//! - NPM registry settings
//!
//! Key use cases:
//! - Initial workspace setup for changeset-based versioning
//! - Programmatic workspace initialization in CI/CD pipelines
//! - Automated project scaffolding and template creation
//! - IDE integration for project initialization
//!
//! # Examples
//!
//! ## TypeScript Usage
//!
//! ```typescript
//! import { init } from '@websublime/workspace-tools';
//!
//! // Initialize with default settings
//! const result = await init({ root: '.' });
//!
//! if (result.success) {
//!   console.log(`Created config: ${result.data.configFile}`);
//!   console.log(`Strategy: ${result.data.strategy}`);
//!   console.log(`Changesets at: ${result.data.changesetPath}`);
//! } else {
//!   console.error(`Error [${result.error.code}]: ${result.error.message}`);
//! }
//! ```
//!
//! ## With Custom Configuration
//!
//! ```typescript
//! const result = await init({
//!   root: '/path/to/project',
//!   strategy: 'independent',
//!   configFormat: 'toml',
//!   environments: ['dev', 'staging', 'prod'],
//!   defaultEnv: ['prod'],
//!   changesetPath: '.changesets'
//! });
//! ```
//!
//! ## Error Handling
//!
//! ```typescript
//! const result = await init({ root: '/nonexistent/path' });
//!
//! if (!result.success) {
//!   switch (result.error.code) {
//!     case 'ENOENT':
//!       console.error('Path not found:', result.error.message);
//!       break;
//!     case 'EVALIDATION':
//!       console.error('Invalid parameters:', result.error.message);
//!       break;
//!     case 'ECONFIG':
//!       console.error('Config already exists:', result.error.message);
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
use crate::types::init::{
    InitApiResponse, InitData, InitParams, VALID_CONFIG_FORMATS, VALID_STRATEGIES,
};
use crate::validation::validators;

use sublime_cli_tools::cli::commands::InitArgs;
use sublime_cli_tools::commands::init::execute_init;
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
    /// # Note
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

/// CLI JSON response wrapper for init command.
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

/// CLI init response data structure.
///
/// Mirrors the `InitResult` structure from the CLI's init command.
/// Field names use camelCase to match the JSON output format.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliInitData {
    /// Name of the created configuration file.
    pub(crate) config_file: String,
    /// Format of the configuration file (json, yaml, toml).
    pub(crate) config_format: String,
    /// Versioning strategy applied (independent or unified).
    pub(crate) strategy: String,
    /// Path to the changeset directory.
    pub(crate) changeset_path: String,
    /// List of configured environments.
    pub(crate) environments: Vec<String>,
    /// List of default environments.
    pub(crate) default_environments: Vec<String>,
    /// NPM registry URL.
    pub(crate) registry: String,
}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Converts CLI init data to NAPI-compatible `InitData`.
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
/// An `InitData` instance suitable for returning to JavaScript.
pub(crate) fn convert_to_napi_init(cli_data: CliInitData) -> InitData {
    InitData {
        config_file: cli_data.config_file,
        config_format: cli_data.config_format,
        strategy: cli_data.strategy,
        changeset_path: cli_data.changeset_path,
        environments: cli_data.environments,
        default_environments: cli_data.default_environments,
        registry: cli_data.registry,
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
/// * `Ok(InitData)` - Successfully parsed and converted init data
/// * `Err(ErrorInfo)` - Parsing failed or CLI returned an error
///
/// # Errors
///
/// Returns an error if:
/// - The JSON is malformed or cannot be parsed
/// - The CLI returned `success: false` with an error message
/// - The CLI returned `success: true` but `data` is missing
pub(crate) fn parse_init_response(json_bytes: &[u8]) -> Result<InitData, ErrorInfo> {
    // Convert bytes to string first for better error messages
    let json_str = std::str::from_utf8(json_bytes)
        .map_err(|e| ErrorInfo::execution(format!("Invalid UTF-8 in CLI response: {e}")))?;

    // Handle empty response
    if json_str.trim().is_empty() {
        return Err(ErrorInfo::execution("CLI returned empty response"));
    }

    // Parse the JSON response
    let response: CliJsonResponse<CliInitData> = serde_json::from_str(json_str).map_err(|e| {
        ErrorInfo::execution(format!(
            "Failed to parse CLI JSON response: {e} (length={})",
            json_str.len()
        ))
    })?;

    // Check for CLI-level errors
    if !response.success {
        let error_message = response.error.unwrap_or_else(|| "Unknown CLI error".to_string());
        // Check if it's a configuration error (e.g., config already exists)
        if error_message.contains("already exists") {
            return Err(ErrorInfo::configuration(error_message));
        }
        return Err(ErrorInfo::execution(error_message));
    }

    // Extract and convert data
    let cli_data =
        response.data.ok_or_else(|| ErrorInfo::execution("CLI returned success but no data"))?;

    Ok(convert_to_napi_init(cli_data))
}

// ============================================================================
// Parameter Validation
// ============================================================================

/// Validates init command parameters.
///
/// Ensures the root path is valid and optional parameters have valid values
/// before executing the CLI command.
///
/// # Arguments
///
/// * `params` - The init parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path
/// * `Err(ErrorInfo)` - Validation failed
///
/// # Validation Rules
///
/// 1. Root path must exist and be a directory
/// 2. Strategy (if provided) must be "independent" or "unified"
/// 3. Config format (if provided) must be "json", "yaml", or "toml"
pub(crate) fn validate_params(params: &InitParams) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    // Validate strategy if provided
    if let Some(ref strategy) = params.strategy {
        let strategy_lower = strategy.to_lowercase();
        if !VALID_STRATEGIES.contains(&strategy_lower.as_str()) {
            return Err(ErrorInfo::validation(
                format!(
                    "Invalid strategy '{}'. Must be one of: {}",
                    strategy,
                    VALID_STRATEGIES.join(", ")
                ),
                Some("strategy"),
            ));
        }
    }

    // Validate config format if provided
    if let Some(ref format) = params.config_format {
        let format_lower = format.to_lowercase();
        if !VALID_CONFIG_FORMATS.contains(&format_lower.as_str()) {
            return Err(ErrorInfo::validation(
                format!(
                    "Invalid config format '{}'. Must be one of: {}",
                    format,
                    VALID_CONFIG_FORMATS.join(", ")
                ),
                Some("configFormat"),
            ));
        }
    }

    Ok(PathBuf::from(&params.root))
}

/// Converts `InitParams` to `InitArgs` for CLI execution.
///
/// This function maps NAPI parameters to CLI arguments, ensuring that
/// `non_interactive` is always `true` for programmatic usage.
///
/// # Arguments
///
/// * `params` - The NAPI init parameters
///
/// # Returns
///
/// An `InitArgs` instance ready for CLI execution.
pub(crate) fn convert_params_to_args(params: &InitParams) -> InitArgs {
    InitArgs {
        changeset_path: params
            .changeset_path
            .as_ref()
            .map_or_else(|| PathBuf::from(".changesets"), PathBuf::from),
        environments: params.environments.clone(),
        default_env: params.default_env.clone(),
        strategy: params.strategy.clone(),
        registry: params.registry.clone().unwrap_or_else(|| "https://registry.npmjs.org".into()),
        config_format: params.config_format.clone(),
        force: params.force.unwrap_or(false),
        // Always non-interactive for programmatic API
        non_interactive: true,
    }
}

// ============================================================================
// NAPI Function
// ============================================================================

/// Initialize a workspace with changeset-based version management.
///
/// Creates a configuration file (repo.config), sets up the changeset directory
/// structure, and configures versioning settings for the workspace.
///
/// This function is the main entry point for Node.js applications to initialize
/// workspaces. It handles all the complexity of CLI invocation and response
/// parsing internally.
///
/// **Note**: This function always runs in non-interactive mode. All required
/// configuration must be provided via parameters or will use sensible defaults.
///
/// @param params - Init parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `changesetPath`: Directory for changeset files (default: ".changesets")
///   - `environments`: Available environments (default: `["dev", "staging", "production"]`)
///   - `defaultEnv`: Default environments (default: `["production"]`)
///   - `strategy`: Versioning strategy - "independent" or "unified"
///   - `registry`: NPM registry URL (default: `https://registry.npmjs.org`)
///   - `configFormat`: Config file format - "json", "yaml", or "toml" (default: "toml")
///   - `force`: Overwrite existing configuration (default: false)
///
/// @returns `Promise<ApiResponse<InitData>>` containing:
///   - On success: `{ success: true, data: InitData }`
///   - On failure: `{ success: false, error: ErrorInfo }`
///
/// @example Basic initialization
/// ```typescript
/// const result = await init({ root: '.' });
/// if (result.success) {
///   console.log(`Created: ${result.data.configFile}`);
///   console.log(`Strategy: ${result.data.strategy}`);
/// } else {
///   console.error(`Error: ${result.error.code} - ${result.error.message}`);
/// }
/// ```
///
/// @example With custom configuration
/// ```typescript
/// const result = await init({
///   root: '/path/to/project',
///   strategy: 'independent',
///   configFormat: 'toml',
///   environments: ['dev', 'staging', 'prod'],
///   defaultEnv: ['prod']
/// });
/// ```
///
/// @example Force overwrite existing config
/// ```typescript
/// const result = await init({
///   root: '.',
///   force: true,
///   strategy: 'unified'
/// });
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await init({ root: '/nonexistent' });
/// if (!result.success) {
///   if (result.error.code === 'ENOENT') {
///     console.error('Path not found');
///   } else if (result.error.code === 'ECONFIG') {
///     console.error('Config already exists, use force: true to overwrite');
///   }
/// }
/// ```
#[napi]
pub async fn init(params: InitParams) -> InitApiResponse {
    // 1. Validate parameters (synchronous validation before spawning)
    let root_path = match validate_params(&params) {
        Ok(path) => path,
        Err(error) => return InitApiResponse::failure(error),
    };

    // 2. Convert NAPI params to CLI args
    let args = convert_params_to_args(&params);

    // 3. Execute CLI command in a blocking task
    // The CLI's execute_init uses types that are not Send/Sync (RefCell, git2::Repository),
    // so we must run it on a blocking thread via spawn_blocking.
    let result = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        // This is necessary because execute_init is async but we're in a blocking context
        let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(e) => {
                return Err(ErrorInfo::execution(format!("Failed to create runtime: {e}")));
            }
        };

        rt.block_on(async {
            // Create shared buffer for output capture
            let buffer = SharedBuffer::new();

            // Create Output with JSON format and no color
            let output = Output::new(OutputFormat::Json, buffer.clone(), true);

            // Execute the CLI command
            // config_path is None as init creates the config file
            let config_path: Option<&Path> = None;
            if let Err(cli_error) = execute_init(&args, &output, &root_path, config_path).await {
                return Err(ErrorInfo::from(cli_error));
            }

            // Extract and parse JSON
            let json_bytes = buffer.take_bytes();
            parse_init_response(&json_bytes)
        })
    })
    .await;

    // 4. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => InitApiResponse::success(data),
        Ok(Err(error)) => InitApiResponse::failure(error),
        Err(join_error) => InitApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}
