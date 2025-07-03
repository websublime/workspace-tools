//! Plugin system type definitions
//!
//! Core traits and types for the plugin system including the main MonorepoPlugin trait
//! and supporting structures for plugin metadata, context, and results.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main trait that all monorepo plugins must implement
///
/// This trait defines the standard interface for all plugins, providing
/// a consistent way to interact with different types of plugins.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::plugins::{MonorepoPlugin, PluginInfo, PluginContext, PluginResult};
/// use sublime_monorepo_tools::error::Result;
/// use serde_json::Value;
///
/// struct MyCustomPlugin;
///
/// impl MonorepoPlugin for MyCustomPlugin {
///     fn info(&self) -> PluginInfo {
///         PluginInfo {
///             name: "my-custom-plugin".to_string(),
///             version: "1.0.0".to_string(),
///             description: "A custom plugin example".to_string(),
///             author: "My Team".to_string(),
///             capabilities: Default::default(),
///         }
///     }
///
///     fn initialize(&mut self, context: &PluginContext) -> Result<()> {
///         // Plugin initialization logic
///         Ok(())
///     }
///
///     fn execute_command(&self, command: &str, args: &[String], context: &PluginContext) -> Result<PluginResult> {
///         match command {
///             "analyze" => {
///                 // Use context.packages, context.repository, etc. for real analysis
///                 Ok(PluginResult::success("Analysis completed"))
///             },
///             _ => Err(Error::plugin("Unknown command")),
///         }
///     }
/// }
/// ```
pub trait MonorepoPlugin: Send + Sync {
    /// Get plugin information and metadata
    ///
    /// Returns static information about the plugin including name, version,
    /// description, and capabilities.
    fn info(&self) -> PluginInfo;

    /// Initialize the plugin with the given context
    ///
    /// Called once when the plugin is loaded. Use this to perform any
    /// setup required by the plugin.
    ///
    /// # Arguments
    ///
    /// * `context` - Plugin execution context with access to monorepo data
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails
    fn initialize(&mut self, context: &PluginContext) -> Result<()>;

    /// Execute a plugin command with the given arguments and context
    ///
    /// This is the main entry point for plugin functionality. Commands
    /// are plugin-specific and defined by each plugin implementation.
    /// Context provides access to monorepo services for real functionality.
    ///
    /// # Arguments
    ///
    /// * `command` - Command name to execute
    /// * `args` - Command arguments
    /// * `context` - Plugin execution context with access to monorepo services
    ///
    /// # Returns
    ///
    /// Plugin execution result containing data or error information
    fn execute_command(
        &self,
        command: &str,
        args: &[String],
        context: &PluginContext,
    ) -> Result<PluginResult>;

    /// Get the lifecycle state of the plugin
    ///
    /// Default implementation returns Active. Override to provide
    /// custom lifecycle management.
    fn lifecycle_state(&self) -> PluginLifecycle {
        PluginLifecycle::Active
    }

    /// Clean up plugin resources
    ///
    /// Called when the plugin is being unloaded. Use this to clean up
    /// any resources or perform final operations.
    ///
    /// Default implementation does nothing.
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Plugin metadata and information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin name (unique identifier)
    pub name: String,
    /// Plugin version (semantic version)
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Plugin author or team
    pub author: String,
    /// Plugin capabilities and features
    pub capabilities: PluginCapabilities,
}

/// Plugin execution context providing access to monorepo data
///
/// Contains references to the monorepo project and configuration
/// that plugins can use during execution.
///
/// Uses direct borrowing from MonorepoProject components instead of Arc.
pub struct PluginContext<'a> {
    /// Direct reference to configuration
    pub(crate) config_ref: &'a crate::config::MonorepoConfig,
    /// Direct reference to packages
    pub(crate) packages: &'a [crate::core::MonorepoPackageInfo],
    /// Direct reference to repository
    pub(crate) repository: &'a sublime_git_tools::Repo,
    /// Direct reference to file system manager
    pub(crate) file_system: &'a sublime_standard_tools::filesystem::FileSystemManager,
    /// Direct reference to root path
    pub(crate) root_path: &'a std::path::Path,
    /// Plugin-specific configuration
    pub(crate) config: HashMap<String, serde_json::Value>,
    /// Working directory for plugin operations
    pub(crate) working_directory: std::path::PathBuf,
}

impl<'a> PluginContext<'a> {
    /// Create a new plugin context with direct borrowing from project
    ///
    /// Uses borrowing instead of Arc to eliminate Arc proliferation
    /// and work with Rust ownership principles.
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to monorepo project
    /// * `config` - Plugin configuration
    /// * `working_directory` - Working directory for plugin operations
    pub fn new(
        project: &'a crate::core::MonorepoProject,
        config: HashMap<String, serde_json::Value>,
        working_directory: std::path::PathBuf,
    ) -> Self {
        Self {
            config_ref: &project.config,
            packages: &project.packages,
            repository: &project.repository,
            file_system: &project.file_system,
            root_path: &project.root_path,
            config,
            working_directory,
        }
    }

    /// Get a configuration value for the plugin
    ///
    /// # Arguments
    ///
    /// * `key` - Configuration key
    ///
    /// # Returns
    ///
    /// Configuration value if found
    pub fn get_config(&self, key: &str) -> Option<&serde_json::Value> {
        self.config.get(key)
    }

    /// Get a typed configuration value
    ///
    /// # Arguments
    ///
    /// * `key` - Configuration key
    ///
    /// # Returns
    ///
    /// Deserialized configuration value if found and valid
    pub fn get_typed_config<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.config.get(key) {
            Some(value) => {
                let typed_value = serde_json::from_value(value.clone())
                    .map_err(|e| Error::plugin(format!("Invalid config for key {key}: {e}")))?;
                Ok(Some(typed_value))
            }
            None => Ok(None),
        }
    }

    /// Get reference to configuration
    pub fn config(&self) -> &crate::config::MonorepoConfig {
        self.config_ref
    }

    /// Get reference to packages
    pub fn packages(&self) -> &[crate::core::MonorepoPackageInfo] {
        self.packages
    }

    /// Get reference to repository
    pub fn repository(&self) -> &sublime_git_tools::Repo {
        self.repository
    }

    /// Get reference to file system manager
    pub fn file_system(&self) -> &sublime_standard_tools::filesystem::FileSystemManager {
        self.file_system
    }

    /// Get reference to root path
    pub fn root_path(&self) -> &std::path::Path {
        self.root_path
    }

    /// Get reference to working directory
    pub fn working_directory(&self) -> &std::path::Path {
        &self.working_directory
    }
}

/// Result of plugin command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    /// Whether the command succeeded
    pub success: bool,
    /// Result data (command-specific)
    pub data: serde_json::Value,
    /// Error message if command failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PluginResult {
    /// Create a successful result
    ///
    /// # Arguments
    ///
    /// * `data` - Result data
    pub fn success(data: impl Serialize) -> Self {
        Self {
            success: true,
            data: serde_json::to_value(data).unwrap_or(serde_json::Value::Null),
            error: None,
            execution_time_ms: 0,
            metadata: HashMap::new(),
        }
    }

    /// Create a successful result with execution time
    ///
    /// # Arguments
    ///
    /// * `data` - Result data
    /// * `execution_time_ms` - Execution time in milliseconds
    pub fn success_with_time(data: impl Serialize, execution_time_ms: u64) -> Self {
        Self {
            success: true,
            data: serde_json::to_value(data).unwrap_or(serde_json::Value::Null),
            error: None,
            execution_time_ms,
            metadata: HashMap::new(),
        }
    }

    /// Create an error result
    ///
    /// # Arguments
    ///
    /// * `error` - Error message
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: serde_json::Value::Null,
            error: Some(error.into()),
            execution_time_ms: 0,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the result
    ///
    /// # Arguments
    ///
    /// * `key` - Metadata key
    /// * `value` - Metadata value
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), json_value);
        }
        self
    }
}

/// Plugin error type for plugin-specific errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginError {
    /// Error message
    pub message: String,
    /// Error code (plugin-specific)
    pub code: Option<String>,
    /// Additional error context
    pub context: HashMap<String, serde_json::Value>,
}

impl PluginError {
    /// Create a new plugin error
    ///
    /// # Arguments
    ///
    /// * `message` - Error message
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), code: None, context: HashMap::new() }
    }

    /// Create a plugin error with code
    ///
    /// # Arguments
    ///
    /// * `message` - Error message
    /// * `code` - Error code
    pub fn with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self { message: message.into(), code: Some(code.into()), context: HashMap::new() }
    }
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Plugin error: {message}", message = self.message)?;
        if let Some(code) = &self.code {
            write!(f, " (code: {code})")?;
        }
        Ok(())
    }
}

impl std::error::Error for PluginError {}

/// Plugin command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Command arguments
    pub arguments: Vec<PluginArgument>,
    /// Whether command supports async execution
    pub async_support: bool,
}

/// Plugin command argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginArgument {
    /// Argument name
    pub name: String,
    /// Argument description
    pub description: String,
    /// Whether argument is required
    pub required: bool,
    /// Argument type
    pub arg_type: PluginArgumentType,
    /// Default value
    pub default_value: Option<String>,
}

/// Plugin argument types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginArgumentType {
    /// String argument
    String,
    /// Integer argument
    Integer,
    /// Boolean argument
    Boolean,
    /// Array argument
    Array,
    /// Object argument
    Object,
}

/// Plugin capabilities and features
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PluginCapabilities {
    /// Supported commands
    pub commands: Vec<PluginCommand>,
    /// Whether plugin supports async execution
    pub async_support: bool,
    /// Whether plugin supports parallel execution
    pub parallel_support: bool,
    /// Plugin categories (e.g., "analyzer", "generator", "validator")
    pub categories: Vec<String>,
    /// Supported file types or patterns
    pub file_patterns: Vec<String>,
}

/// Plugin lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginLifecycle {
    /// Plugin is being loaded
    Loading,
    /// Plugin is active and ready
    Active,
    /// Plugin is suspended/paused
    Suspended,
    /// Plugin is being unloaded
    Unloading,
    /// Plugin has errored
    Errored,
}


impl std::fmt::Display for PluginLifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginLifecycle::Loading => write!(f, "Loading"),
            PluginLifecycle::Active => write!(f, "Active"),
            PluginLifecycle::Suspended => write!(f, "Suspended"),
            PluginLifecycle::Unloading => write!(f, "Unloading"),
            PluginLifecycle::Errored => write!(f, "Errored"),
        }
    }
}
