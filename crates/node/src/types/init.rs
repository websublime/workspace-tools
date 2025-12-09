//! Init command type definitions for Node.js bindings.
//!
//! # What
//!
//! This module defines all NAPI-compatible type structures for the init command,
//! including input parameters and response data types. These types enable JavaScript
//! and TypeScript consumers to initialize workspace configurations in a type-safe manner.
//!
//! # How
//!
//! Types are defined with the `#[napi(object)]` attribute to be automatically
//! exposed as JavaScript objects. The module provides:
//!
//! - **`InitParams`**: Input parameters for the init command
//! - **`InitData`**: Response data containing initialization result
//! - **`InitApiResponse`**: Concrete response wrapper for NAPI compatibility
//!
//! All types implement `Clone`, `Debug`, and `Serialize` for flexibility in
//! testing and serialization scenarios.
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
//! These types provide:
//!
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
//! import { init, InitParams, InitData } from '@websublime/workspace-tools';
//!
//! const params: InitParams = {
//!   root: '/path/to/workspace',
//!   strategy: 'independent',
//!   configFormat: 'toml',
//!   environments: ['dev', 'staging', 'prod'],
//!   defaultEnv: ['prod']
//! };
//!
//! const result = await init(params);
//!
//! if (result.success) {
//!   const data: InitData = result.data;
//!   console.log(`Created config: ${data.configFile}`);
//!   console.log(`Strategy: ${data.strategy}`);
//!   console.log(`Changesets at: ${data.changesetPath}`);
//! }
//! ```
//!
//! ## Rust Usage (Internal)
//!
//! ```rust,ignore
//! use sublime_node_tools::types::init::{InitParams, InitData, InitApiResponse};
//!
//! // Creating params for validation
//! let params = InitParams {
//!     root: "/path/to/workspace".to_string(),
//!     changeset_path: Some(".changesets".to_string()),
//!     environments: Some(vec!["dev".to_string(), "prod".to_string()]),
//!     default_env: Some(vec!["prod".to_string()]),
//!     strategy: Some("independent".to_string()),
//!     registry: None,
//!     config_format: Some("toml".to_string()),
//!     force: Some(false),
//! };
//!
//! // Constructing response data
//! let data = InitData {
//!     config_file: "repo.config.toml".to_string(),
//!     config_format: "toml".to_string(),
//!     strategy: "independent".to_string(),
//!     changeset_path: ".changesets".to_string(),
//!     environments: vec!["dev".to_string(), "prod".to_string()],
//!     default_environments: vec!["prod".to_string()],
//!     registry: "https://registry.npmjs.org".to_string(),
//! };
//!
//! let response = InitApiResponse::success(data);
//! ```

use napi_derive::napi;
use serde::Serialize;

use crate::error::ErrorInfo;

// ============================================================================
// Input Parameters
// ============================================================================

/// Input parameters for the init command.
///
/// This structure defines the parameters that can be passed to the `init`
/// function from JavaScript/TypeScript. The root path is required, while
/// all other parameters are optional and will use CLI defaults if not specified.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `changeset_path`: Directory where changeset files will be stored
/// - `environments`: List of available environments
/// - `default_env`: List of default environments
/// - `strategy`: Versioning strategy ("independent" or "unified")
/// - `registry`: NPM registry URL
/// - `config_format`: Configuration file format ("json", "yaml", or "toml")
/// - `force`: Whether to overwrite existing configuration
///
/// # TypeScript Definition
///
/// ```text
/// interface InitParams {
///   root: string;                    // Workspace root directory path
///   changesetPath?: string;          // Changeset directory path (default: ".changesets")
///   environments?: string[];         // Available environments (e.g., ["dev", "staging", "prod"])
///   defaultEnv?: string[];           // Default environments (e.g., ["prod"])
///   strategy?: string;               // Versioning strategy: "independent" or "unified"
///   registry?: string;               // NPM registry URL (default: "https://registry.npmjs.org")
///   configFormat?: string;           // Config file format: "json", "yaml", or "toml"
///   force?: boolean;                 // Overwrite existing configuration
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Minimal params with just root
/// const params: InitParams = { root: '.' };
///
/// // Full configuration
/// const fullParams: InitParams = {
///   root: '/path/to/workspace',
///   changesetPath: '.changesets',
///   environments: ['dev', 'staging', 'prod'],
///   defaultEnv: ['prod'],
///   strategy: 'independent',
///   registry: 'https://registry.npmjs.org',
///   configFormat: 'toml',
///   force: false
/// };
/// ```
// Allow dead_code: This struct will be used by the init command in Story 3.4
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct InitParams {
    /// Workspace root directory path.
    ///
    /// This is the absolute or relative path to the root of the workspace.
    /// For monorepos, this should point to the root where the package manager
    /// configuration (e.g., `pnpm-workspace.yaml`) is located.
    pub root: String,

    /// Changeset directory path.
    ///
    /// Directory where changeset files will be stored. If not provided,
    /// defaults to `.changesets` in the workspace root.
    ///
    /// # Example
    ///
    /// ```typescript
    /// const params: InitParams = {
    ///   root: '.',
    ///   changesetPath: '.changesets'
    /// };
    /// ```
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changeset_path: Option<String>,

    /// List of available environments.
    ///
    /// Environments are used to configure different release channels
    /// or deployment targets. If not provided, no environments are configured.
    ///
    /// # Example
    ///
    /// ```typescript
    /// const params: InitParams = {
    ///   root: '.',
    ///   environments: ['dev', 'staging', 'prod']
    /// };
    /// ```
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environments: Option<Vec<String>>,

    /// List of default environments.
    ///
    /// Default environments are applied when no specific environment
    /// is specified during changeset creation or version bumping.
    ///
    /// # Example
    ///
    /// ```typescript
    /// const params: InitParams = {
    ///   root: '.',
    ///   environments: ['dev', 'staging', 'prod'],
    ///   defaultEnv: ['prod']
    /// };
    /// ```
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_env: Option<Vec<String>>,

    /// Versioning strategy.
    ///
    /// Determines how packages are versioned:
    /// - `"independent"`: Each package versions independently
    /// - `"unified"`: All packages share the same version
    ///
    /// If not provided, the CLI will determine an appropriate default
    /// based on the workspace structure.
    ///
    /// # Example
    ///
    /// ```typescript
    /// const params: InitParams = {
    ///   root: '.',
    ///   strategy: 'independent'
    /// };
    /// ```
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,

    /// NPM registry URL.
    ///
    /// The registry URL used for package publishing and version checks.
    /// If not provided, defaults to `https://registry.npmjs.org`.
    ///
    /// # Example
    ///
    /// ```typescript
    /// const params: InitParams = {
    ///   root: '.',
    ///   registry: 'https://npm.pkg.github.com'
    /// };
    /// ```
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,

    /// Configuration file format.
    ///
    /// The format to use for the generated configuration file:
    /// - `"json"`: Creates `repo.config.json`
    /// - `"yaml"`: Creates `repo.config.yaml`
    /// - `"toml"`: Creates `repo.config.toml`
    ///
    /// If not provided, the CLI will default to JSON format.
    ///
    /// # Example
    ///
    /// ```typescript
    /// const params: InitParams = {
    ///   root: '.',
    ///   configFormat: 'toml'
    /// };
    /// ```
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_format: Option<String>,

    /// Force overwrite existing configuration.
    ///
    /// If `true`, overwrites any existing configuration file.
    /// If `false` or not provided, the command will fail if a
    /// configuration file already exists.
    ///
    /// # Example
    ///
    /// ```typescript
    /// const params: InitParams = {
    ///   root: '.',
    ///   force: true  // Overwrite existing config
    /// };
    /// ```
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

// ============================================================================
// Response Data Types
// ============================================================================

/// Initialization result data.
///
/// Contains information about the created configuration file and the
/// settings that were applied during initialization.
///
/// # Fields
///
/// - `config_file`: Name of the created configuration file
/// - `config_format`: Format of the configuration file
/// - `strategy`: Versioning strategy applied
/// - `changeset_path`: Path to the changeset directory
/// - `environments`: List of configured environments
/// - `default_environments`: List of default environments
/// - `registry`: NPM registry URL
///
/// # TypeScript Definition
///
/// ```text
/// interface InitData {
///   configFile: string;              // Name of the created config file (e.g., "repo.config.toml")
///   configFormat: string;            // Format of the config file: "json", "yaml", or "toml"
///   strategy: string;                // Versioning strategy: "independent" or "unified"
///   changesetPath: string;           // Path to the changeset directory
///   environments: string[];          // Configured environments
///   defaultEnvironments: string[];   // Default environments
///   registry: string;                // NPM registry URL
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await init({ root: '.', strategy: 'independent' });
///
/// if (result.success) {
///   const data: InitData = result.data;
///   console.log(`Config created: ${data.configFile}`);
///   console.log(`Format: ${data.configFormat}`);
///   console.log(`Strategy: ${data.strategy}`);
///   console.log(`Changesets: ${data.changesetPath}`);
///   console.log(`Environments: ${data.environments.join(', ')}`);
///   console.log(`Defaults: ${data.defaultEnvironments.join(', ')}`);
///   console.log(`Registry: ${data.registry}`);
/// }
/// ```
// Allow dead_code: This struct will be used by the init command in Story 3.4
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct InitData {
    /// Name of the created configuration file.
    ///
    /// This is the filename without the full path, e.g., `"repo.config.toml"`.
    pub config_file: String,

    /// Format of the configuration file.
    ///
    /// Possible values:
    /// - `"json"`: JSON format
    /// - `"yaml"`: YAML format
    /// - `"toml"`: TOML format
    pub config_format: String,

    /// Versioning strategy applied.
    ///
    /// Possible values:
    /// - `"independent"`: Each package versions independently
    /// - `"unified"`: All packages share the same version
    pub strategy: String,

    /// Path to the changeset directory.
    ///
    /// Relative to the workspace root, e.g., `".changesets"`.
    pub changeset_path: String,

    /// List of configured environments.
    ///
    /// Empty array if no environments were configured.
    pub environments: Vec<String>,

    /// List of default environments.
    ///
    /// Empty array if no default environments were configured.
    pub default_environments: Vec<String>,

    /// NPM registry URL.
    ///
    /// The registry URL configured for package publishing,
    /// e.g., `"https://registry.npmjs.org"`.
    pub registry: String,
}

// ============================================================================
// Builder and Helper Implementations
// ============================================================================

#[allow(dead_code)]
impl InitParams {
    /// Creates a new `InitParams` with only the required root path.
    ///
    /// All optional fields are set to `None`, meaning CLI defaults will be used.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// A new `InitParams` instance with default optional values.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::InitParams;
    ///
    /// let params = InitParams::new("/path/to/workspace");
    /// assert_eq!(params.root, "/path/to/workspace");
    /// assert!(params.strategy.is_none());
    /// ```
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            changeset_path: None,
            environments: None,
            default_env: None,
            strategy: None,
            registry: None,
            config_format: None,
            force: None,
        }
    }

    /// Sets the changeset path.
    ///
    /// # Arguments
    ///
    /// * `path` - The changeset directory path
    ///
    /// # Returns
    ///
    /// The modified `InitParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::InitParams;
    ///
    /// let params = InitParams::new(".")
    ///     .with_changeset_path(".changesets");
    /// ```
    #[must_use]
    pub fn with_changeset_path(mut self, path: impl Into<String>) -> Self {
        self.changeset_path = Some(path.into());
        self
    }

    /// Sets the environments list.
    ///
    /// # Arguments
    ///
    /// * `environments` - List of environment names
    ///
    /// # Returns
    ///
    /// The modified `InitParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::InitParams;
    ///
    /// let params = InitParams::new(".")
    ///     .with_environments(vec!["dev", "staging", "prod"]);
    /// ```
    #[must_use]
    pub fn with_environments(mut self, environments: Vec<impl Into<String>>) -> Self {
        self.environments = Some(environments.into_iter().map(Into::into).collect());
        self
    }

    /// Sets the default environments list.
    ///
    /// # Arguments
    ///
    /// * `default_env` - List of default environment names
    ///
    /// # Returns
    ///
    /// The modified `InitParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::InitParams;
    ///
    /// let params = InitParams::new(".")
    ///     .with_default_env(vec!["prod"]);
    /// ```
    #[must_use]
    pub fn with_default_env(mut self, default_env: Vec<impl Into<String>>) -> Self {
        self.default_env = Some(default_env.into_iter().map(Into::into).collect());
        self
    }

    /// Sets the versioning strategy.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The versioning strategy ("independent" or "unified")
    ///
    /// # Returns
    ///
    /// The modified `InitParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::InitParams;
    ///
    /// let params = InitParams::new(".")
    ///     .with_strategy("independent");
    /// ```
    #[must_use]
    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.strategy = Some(strategy.into());
        self
    }

    /// Sets the NPM registry URL.
    ///
    /// # Arguments
    ///
    /// * `registry` - The NPM registry URL
    ///
    /// # Returns
    ///
    /// The modified `InitParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::InitParams;
    ///
    /// let params = InitParams::new(".")
    ///     .with_registry("https://npm.pkg.github.com");
    /// ```
    #[must_use]
    pub fn with_registry(mut self, registry: impl Into<String>) -> Self {
        self.registry = Some(registry.into());
        self
    }

    /// Sets the configuration file format.
    ///
    /// # Arguments
    ///
    /// * `format` - The configuration format ("json", "yaml", or "toml")
    ///
    /// # Returns
    ///
    /// The modified `InitParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::InitParams;
    ///
    /// let params = InitParams::new(".")
    ///     .with_config_format("toml");
    /// ```
    #[must_use]
    pub fn with_config_format(mut self, format: impl Into<String>) -> Self {
        self.config_format = Some(format.into());
        self
    }

    /// Sets the force overwrite flag.
    ///
    /// # Arguments
    ///
    /// * `force` - Whether to force overwrite existing configuration
    ///
    /// # Returns
    ///
    /// The modified `InitParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::InitParams;
    ///
    /// let params = InitParams::new(".")
    ///     .with_force(true);
    /// ```
    #[must_use]
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = Some(force);
        self
    }
}

#[allow(dead_code)]
impl InitData {
    /// Creates a new `InitData` instance.
    ///
    /// # Arguments
    ///
    /// * `config_file` - Name of the created configuration file
    /// * `config_format` - Format of the configuration file
    /// * `strategy` - Versioning strategy applied
    /// * `changeset_path` - Path to the changeset directory
    /// * `environments` - List of configured environments
    /// * `default_environments` - List of default environments
    /// * `registry` - NPM registry URL
    ///
    /// # Returns
    ///
    /// A new `InitData` instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::InitData;
    ///
    /// let data = InitData::new(
    ///     "repo.config.toml",
    ///     "toml",
    ///     "independent",
    ///     ".changesets",
    ///     vec!["dev".to_string(), "prod".to_string()],
    ///     vec!["prod".to_string()],
    ///     "https://registry.npmjs.org",
    /// );
    /// ```
    #[must_use]
    pub fn new(
        config_file: impl Into<String>,
        config_format: impl Into<String>,
        strategy: impl Into<String>,
        changeset_path: impl Into<String>,
        environments: Vec<String>,
        default_environments: Vec<String>,
        registry: impl Into<String>,
    ) -> Self {
        Self {
            config_file: config_file.into(),
            config_format: config_format.into(),
            strategy: strategy.into(),
            changeset_path: changeset_path.into(),
            environments,
            default_environments,
            registry: registry.into(),
        }
    }

    /// Creates a new `InitData` with default values.
    ///
    /// Useful for testing or creating baseline configurations.
    ///
    /// # Arguments
    ///
    /// * `config_file` - Name of the configuration file
    /// * `strategy` - Versioning strategy
    ///
    /// # Returns
    ///
    /// A new `InitData` instance with default values for optional fields.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::InitData;
    ///
    /// let data = InitData::with_defaults("repo.config.json", "independent");
    /// assert_eq!(data.config_format, "json");
    /// assert_eq!(data.changeset_path, ".changesets");
    /// ```
    #[must_use]
    pub fn with_defaults(config_file: impl Into<String>, strategy: impl Into<String>) -> Self {
        use std::path::Path;

        let config_file = config_file.into();
        let path = Path::new(&config_file);
        let config_format =
            if path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("toml")) {
                "toml"
            } else if path.extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml")
            }) {
                "yaml"
            } else {
                "json"
            };

        Self {
            config_file,
            config_format: config_format.to_string(),
            strategy: strategy.into(),
            changeset_path: ".changesets".to_string(),
            environments: Vec::new(),
            default_environments: Vec::new(),
            registry: "https://registry.npmjs.org".to_string(),
        }
    }
}

// ============================================================================
// API Response Type (Concrete, non-generic for NAPI compatibility)
// ============================================================================

/// API response for the init command.
///
/// This is a concrete (non-generic) response type specifically for the init
/// command. It uses `#[napi(object)]` to enable automatic conversion to
/// JavaScript objects.
///
/// napi-rs cannot use generic types with `#[napi(object)]`, so each command
/// that returns structured data needs its own concrete response type.
///
/// # Fields
///
/// - `success`: Whether the operation succeeded
/// - `data`: The init data (present when success is true)
/// - `error`: Error information (present when success is false)
///
/// # TypeScript Definition
///
/// ```text
/// interface InitApiResponse {
///   success: boolean;       // Whether the operation succeeded
///   data?: InitData;        // Init data (present on success)
///   error?: ErrorInfo;      // Error information (present on failure)
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await init({ root: '.', strategy: 'independent' });
///
/// if (result.success) {
///   // result.data is InitData
///   console.log(`Created: ${result.data.configFile}`);
///   console.log(`Strategy: ${result.data.strategy}`);
/// } else {
///   // result.error is ErrorInfo
///   console.error(`[${result.error.code}] ${result.error.message}`);
/// }
/// ```
// Allow dead_code: This struct will be used by the init command in Story 3.4
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct InitApiResponse {
    /// Whether the operation succeeded.
    ///
    /// - `true`: Operation completed successfully, `data` field will be present
    /// - `false`: Operation failed, `error` field will be present
    pub success: bool,

    /// The init data (only present when `success` is `true`).
    ///
    /// Contains information about the created configuration file and
    /// the settings that were applied.
    #[napi(ts_type = "InitData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<InitData>,

    /// Error information (only present when `success` is `false`).
    ///
    /// Contains structured error information with a Node.js-style error code,
    /// message, optional context, and error kind.
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl InitApiResponse {
    /// Creates a successful init response with data.
    ///
    /// # Arguments
    ///
    /// * `data` - The init data to include
    ///
    /// # Returns
    ///
    /// A new `InitApiResponse` with `success = true` and the provided data.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::{InitApiResponse, InitData};
    ///
    /// let data = InitData::with_defaults("repo.config.toml", "independent");
    /// let response = InitApiResponse::success(data);
    /// assert!(response.success);
    /// assert!(response.data.is_some());
    /// ```
    #[must_use]
    pub fn success(data: InitData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failed init response with error information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information to include
    ///
    /// # Returns
    ///
    /// A new `InitApiResponse` with `success = false` and the provided error.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::InitApiResponse;
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::validation("Invalid strategy", Some("strategy"));
    /// let response = InitApiResponse::failure(error);
    /// assert!(!response.success);
    /// assert!(response.error.is_some());
    /// ```
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns whether this response represents a success.
    ///
    /// # Returns
    ///
    /// `true` if the operation succeeded, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::{InitApiResponse, InitData};
    ///
    /// let data = InitData::with_defaults("repo.config.json", "unified");
    /// let response = InitApiResponse::success(data);
    /// assert!(response.is_success());
    /// ```
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns whether this response represents a failure.
    ///
    /// # Returns
    ///
    /// `true` if the operation failed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::init::InitApiResponse;
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::configuration("Config already exists");
    /// let response = InitApiResponse::failure(error);
    /// assert!(response.is_failure());
    /// ```
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

// ============================================================================
// Validation Constants
// ============================================================================

/// Valid versioning strategies.
///
/// These values are accepted by the `strategy` parameter:
/// - `"independent"`: Each package versions independently
/// - `"unified"`: All packages share the same version
// Allow dead_code: This constant will be used for validation in Story 3.4
#[allow(dead_code)]
pub const VALID_STRATEGIES: &[&str] = &["independent", "unified"];

/// Valid configuration file formats.
///
/// These values are accepted by the `configFormat` parameter:
/// - `"json"`: JSON format (creates `repo.config.json`)
/// - `"yaml"`: YAML format (creates `repo.config.yaml`)
/// - `"toml"`: TOML format (creates `repo.config.toml`)
// Allow dead_code: This constant will be used for validation in Story 3.4
#[allow(dead_code)]
pub const VALID_CONFIG_FORMATS: &[&str] = &["json", "yaml", "toml"];
