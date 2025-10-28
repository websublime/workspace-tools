//! Standard configuration for the sublime-standard-tools crate.
//!
//! This module defines the standard configuration structure used throughout
//! the crate, providing comprehensive configuration options for all components.
//!
//! ## Environment Variable Overrides
//!
//! Many default values can be overridden using environment variables:
//!
//! ### Package Manager Configuration
//! - `SUBLIME_PACKAGE_MANAGER_ORDER`: Comma-separated list of package managers (npm,yarn,pnpm,bun,jsr)
//! - `SUBLIME_PACKAGE_MANAGER`: Preferred package manager name
//!
//! ### Monorepo Configuration
//! - `SUBLIME_WORKSPACE_PATTERNS`: Comma-separated workspace patterns (e.g., "packages/*,apps/*")
//! - `SUBLIME_PACKAGE_DIRECTORIES`: Comma-separated package directory names
//! - `SUBLIME_EXCLUDE_PATTERNS`: Comma-separated exclude patterns for monorepo detection
//! - `SUBLIME_MAX_SEARCH_DEPTH`: Maximum search depth (1-20)
//!
//! ### Command Configuration
//! - `SUBLIME_COMMAND_TIMEOUT`: Command execution timeout in seconds (1-3600)
//! - `SUBLIME_MAX_CONCURRENT`: Maximum concurrent commands (1-100)
//! - `SUBLIME_BUFFER_SIZE`: Command output buffer size in bytes (256-65536)
//! - `SUBLIME_COLLECTION_WINDOW_MS`: Queue collection window in milliseconds (1-1000)
//! - `SUBLIME_COLLECTION_SLEEP_US`: Queue collection sleep in microseconds (10-10000)
//! - `SUBLIME_IDLE_SLEEP_MS`: Queue idle sleep in milliseconds (1-1000)
//!
//! ### Filesystem Configuration
//! - `SUBLIME_IGNORE_PATTERNS`: Comma-separated filesystem ignore patterns
//! - `SUBLIME_ASYNC_BUFFER_SIZE`: Async I/O buffer size in bytes (1024-1048576)
//! - `SUBLIME_MAX_CONCURRENT_IO`: Maximum concurrent I/O operations (1-1000)
//! - `SUBLIME_IO_TIMEOUT`: I/O operation timeout in seconds (1-300)
//!
//! All environment variables are validated with reasonable bounds and will fall back
//! to hardcoded defaults if invalid values are provided.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::error::ConfigResult;
use crate::node::PackageManagerKind;

use super::traits::Configurable;

/// The standard configuration for sublime-standard-tools.
///
/// This configuration covers all aspects of the crate's behavior, from
/// package manager detection to filesystem operations.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::config::StandardConfig;
///
/// let config = StandardConfig::default();
/// assert_eq!(config.version, "1.0");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardConfig {
    /// Configuration version for migration support
    #[serde(default = "default_version")]
    pub version: String,

    /// Package manager configuration
    #[serde(default)]
    pub package_managers: PackageManagerConfig,

    /// Monorepo detection configuration
    #[serde(default)]
    pub monorepo: MonorepoConfig,

    /// Command execution configuration
    #[serde(default)]
    pub commands: CommandConfig,

    /// Filesystem configuration
    #[serde(default)]
    pub filesystem: FilesystemConfig,

    /// Validation configuration
    #[serde(default)]
    pub validation: ValidationConfig,
}

impl Default for StandardConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            package_managers: PackageManagerConfig::default(),
            monorepo: MonorepoConfig::default(),
            commands: CommandConfig::default(),
            filesystem: FilesystemConfig::default(),
            validation: ValidationConfig::default(),
        }
    }
}

impl Configurable for StandardConfig {
    fn validate(&self) -> ConfigResult<()> {
        // Validate package manager configuration
        if self.package_managers.detection_order.is_empty() {
            return Err("Package manager detection order cannot be empty".into());
        }

        // Validate command timeouts
        if self.commands.default_timeout.as_secs() == 0 {
            return Err("Default command timeout must be greater than 0".into());
        }

        // Validate filesystem configuration
        if self.filesystem.async_io.buffer_size == 0 {
            return Err("Async I/O buffer size must be greater than 0".into());
        }

        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        // Version is always taken from other if different
        if self.version != other.version {
            self.version = other.version;
        }

        // Merge sub-configurations
        self.package_managers.merge_with(other.package_managers)?;
        self.monorepo.merge_with(other.monorepo)?;
        self.commands.merge_with(other.commands)?;
        self.filesystem.merge_with(other.filesystem)?;
        self.validation.merge_with(other.validation)?;

        Ok(())
    }
}

/// Package manager detection and behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerConfig {
    /// Detection order for package managers
    #[serde(default = "default_detection_order")]
    pub detection_order: Vec<PackageManagerKind>,

    /// Custom lock file names for each package manager
    #[serde(default)]
    pub custom_lock_files: HashMap<PackageManagerKind, String>,

    /// Whether to detect from environment variables
    #[serde(default = "default_true")]
    pub detect_from_env: bool,

    /// Environment variable name for preferred package manager
    #[serde(default = "default_package_manager_env")]
    pub env_var_name: String,

    /// Custom binary paths for package managers
    #[serde(default)]
    pub binary_paths: HashMap<PackageManagerKind, PathBuf>,

    /// Fallback package manager if none detected
    #[serde(default)]
    pub fallback: Option<PackageManagerKind>,
}

impl Default for PackageManagerConfig {
    fn default() -> Self {
        Self {
            detection_order: default_detection_order(),
            custom_lock_files: HashMap::new(),
            detect_from_env: true,
            env_var_name: default_package_manager_env(),
            binary_paths: HashMap::new(),
            fallback: None,
        }
    }
}

impl PackageManagerConfig {
    #[allow(clippy::unnecessary_wraps)]
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        if !other.detection_order.is_empty() {
            self.detection_order = other.detection_order;
        }
        self.custom_lock_files.extend(other.custom_lock_files);
        self.detect_from_env = other.detect_from_env;
        if !other.env_var_name.is_empty() {
            self.env_var_name = other.env_var_name;
        }
        self.binary_paths.extend(other.binary_paths);
        if other.fallback.is_some() {
            self.fallback = other.fallback;
        }
        Ok(())
    }
}

/// Monorepo detection and workspace configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonorepoConfig {
    /// Custom workspace directory patterns
    #[serde(default = "default_workspace_patterns")]
    pub workspace_patterns: Vec<String>,

    /// Additional directories to check for packages
    #[serde(default = "default_package_directories")]
    pub package_directories: Vec<String>,

    /// Patterns to exclude from package detection
    #[serde(default = "default_exclude_patterns")]
    pub exclude_patterns: Vec<String>,

    /// Maximum depth for recursive package search
    #[serde(default = "default_max_depth")]
    pub max_search_depth: usize,

    /// Whether to follow symlinks during search
    #[serde(default)]
    pub follow_symlinks: bool,

    /// Custom patterns for workspace detection in package.json
    #[serde(default)]
    pub custom_workspace_fields: Vec<String>,
}

impl Default for MonorepoConfig {
    fn default() -> Self {
        Self {
            workspace_patterns: default_workspace_patterns(),
            package_directories: default_package_directories(),
            exclude_patterns: default_exclude_patterns(),
            max_search_depth: default_max_depth(),
            follow_symlinks: false,
            custom_workspace_fields: default_custom_workspace_fields(),
        }
    }
}

impl MonorepoConfig {
    #[allow(clippy::unnecessary_wraps)]
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        if !other.workspace_patterns.is_empty() {
            self.workspace_patterns = other.workspace_patterns;
        }
        if !other.package_directories.is_empty() {
            self.package_directories = other.package_directories;
        }
        if !other.exclude_patterns.is_empty() {
            self.exclude_patterns = other.exclude_patterns;
        }
        if other.max_search_depth > 0 {
            self.max_search_depth = other.max_search_depth;
        }
        self.follow_symlinks = other.follow_symlinks;
        if !other.custom_workspace_fields.is_empty() {
            self.custom_workspace_fields = other.custom_workspace_fields;
        }
        Ok(())
    }
}

/// Command execution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    /// Default timeout for command execution
    #[serde(default = "default_command_timeout", with = "humantime_serde")]
    pub default_timeout: Duration,

    /// Timeout overrides for specific commands
    #[serde(default)]
    pub timeout_overrides: HashMap<String, Duration>,

    /// Buffer size for command output streaming
    #[serde(default = "default_buffer_size")]
    pub stream_buffer_size: usize,

    /// Read timeout for streaming output
    #[serde(default = "default_stream_timeout", with = "humantime_serde")]
    pub stream_read_timeout: Duration,

    /// Maximum concurrent commands in queue
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_commands: usize,

    /// Environment variables to set for all commands
    #[serde(default)]
    pub env_vars: HashMap<String, String>,

    /// Whether to inherit parent process environment
    #[serde(default = "default_true")]
    pub inherit_env: bool,

    /// Queue collection window duration in milliseconds
    #[serde(default = "default_collection_window_ms")]
    pub queue_collection_window_ms: u64,

    /// Queue collection sleep duration in microseconds
    #[serde(default = "default_collection_sleep_us")]
    pub queue_collection_sleep_us: u64,

    /// Queue idle sleep duration in milliseconds
    #[serde(default = "default_idle_sleep_ms")]
    pub queue_idle_sleep_ms: u64,
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            default_timeout: default_command_timeout(),
            timeout_overrides: HashMap::new(),
            stream_buffer_size: default_buffer_size(),
            stream_read_timeout: default_stream_timeout(),
            max_concurrent_commands: default_max_concurrent(),
            env_vars: HashMap::new(),
            inherit_env: true,
            queue_collection_window_ms: default_collection_window_ms(),
            queue_collection_sleep_us: default_collection_sleep_us(),
            queue_idle_sleep_ms: default_idle_sleep_ms(),
        }
    }
}

impl CommandConfig {
    #[allow(clippy::unnecessary_wraps)]
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        if other.default_timeout.as_secs() > 0 {
            self.default_timeout = other.default_timeout;
        }
        self.timeout_overrides.extend(other.timeout_overrides);
        if other.stream_buffer_size > 0 {
            self.stream_buffer_size = other.stream_buffer_size;
        }
        if other.stream_read_timeout.as_secs() > 0 {
            self.stream_read_timeout = other.stream_read_timeout;
        }
        if other.max_concurrent_commands > 0 {
            self.max_concurrent_commands = other.max_concurrent_commands;
        }
        self.env_vars.extend(other.env_vars);
        self.inherit_env = other.inherit_env;
        if other.queue_collection_window_ms > 0 {
            self.queue_collection_window_ms = other.queue_collection_window_ms;
        }
        if other.queue_collection_sleep_us > 0 {
            self.queue_collection_sleep_us = other.queue_collection_sleep_us;
        }
        if other.queue_idle_sleep_ms > 0 {
            self.queue_idle_sleep_ms = other.queue_idle_sleep_ms;
        }
        Ok(())
    }
}

/// Filesystem operation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    /// Path conventions overrides
    #[serde(default)]
    pub path_conventions: HashMap<String, PathBuf>,

    /// Async I/O configuration
    #[serde(default)]
    pub async_io: AsyncIoConfig,

    /// File operation retry configuration
    #[serde(default)]
    pub retry: RetryConfig,

    /// Patterns to ignore during directory traversal
    #[serde(default = "default_ignore_patterns")]
    pub ignore_patterns: Vec<String>,
}

impl Default for FilesystemConfig {
    fn default() -> Self {
        Self {
            path_conventions: HashMap::new(),
            async_io: AsyncIoConfig::default(),
            retry: RetryConfig::default(),
            ignore_patterns: default_ignore_patterns(),
        }
    }
}

impl FilesystemConfig {
    #[allow(clippy::unnecessary_wraps)]
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.path_conventions.extend(other.path_conventions);
        self.async_io.merge_with(other.async_io)?;
        self.retry.merge_with(other.retry)?;
        if !other.ignore_patterns.is_empty() {
            self.ignore_patterns = other.ignore_patterns;
        }
        Ok(())
    }
}

/// Async I/O configuration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AsyncIoConfig {
    /// Buffer size for async file operations
    #[serde(default = "default_async_buffer_size")]
    pub buffer_size: usize,

    /// Maximum concurrent file operations
    #[serde(default = "default_max_concurrent_io")]
    pub max_concurrent_operations: usize,

    /// Timeout for individual I/O operations
    #[serde(default = "default_io_timeout", with = "humantime_serde")]
    pub operation_timeout: Duration,
}

impl Default for AsyncIoConfig {
    fn default() -> Self {
        Self {
            buffer_size: default_async_buffer_size(),
            max_concurrent_operations: default_max_concurrent_io(),
            operation_timeout: default_io_timeout(),
        }
    }
}

impl AsyncIoConfig {
    #[allow(clippy::unnecessary_wraps)]
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        if other.buffer_size > 0 {
            self.buffer_size = other.buffer_size;
        }
        if other.max_concurrent_operations > 0 {
            self.max_concurrent_operations = other.max_concurrent_operations;
        }
        if other.operation_timeout.as_secs() > 0 {
            self.operation_timeout = other.operation_timeout;
        }
        Ok(())
    }
}

/// Retry configuration for filesystem operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    #[serde(default = "default_max_retries")]
    pub max_attempts: u32,

    /// Initial delay between retries
    #[serde(default = "default_retry_delay", with = "humantime_serde")]
    pub initial_delay: Duration,

    /// Maximum delay between retries
    #[serde(default = "default_max_retry_delay", with = "humantime_serde")]
    pub max_delay: Duration,

    /// Exponential backoff multiplier
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: default_max_retries(),
            initial_delay: default_retry_delay(),
            max_delay: default_max_retry_delay(),
            backoff_multiplier: default_backoff_multiplier(),
        }
    }
}

impl RetryConfig {
    #[allow(clippy::unnecessary_wraps)]
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        if other.max_attempts > 0 {
            self.max_attempts = other.max_attempts;
        }
        if other.initial_delay.as_millis() > 0 {
            self.initial_delay = other.initial_delay;
        }
        if other.max_delay.as_secs() > 0 {
            self.max_delay = other.max_delay;
        }
        if other.backoff_multiplier > 0.0 {
            self.backoff_multiplier = other.backoff_multiplier;
        }
        Ok(())
    }
}

/// Project validation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Whether to require package.json at project root
    #[serde(default = "default_true")]
    pub require_package_json: bool,

    /// Required fields in package.json
    #[serde(default)]
    pub required_package_fields: Vec<String>,

    /// Whether to validate dependency versions
    #[serde(default = "default_true")]
    pub validate_dependencies: bool,

    /// Custom validation rules
    #[serde(default)]
    pub custom_rules: HashMap<String, serde_json::Value>,

    /// Whether to fail on validation warnings
    #[serde(default)]
    pub strict_mode: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            require_package_json: true,
            required_package_fields: Vec::new(),
            validate_dependencies: true,
            custom_rules: HashMap::new(),
            strict_mode: false,
        }
    }
}

impl ValidationConfig {
    #[allow(clippy::unnecessary_wraps)]
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.require_package_json = other.require_package_json;
        if !other.required_package_fields.is_empty() {
            self.required_package_fields = other.required_package_fields;
        }
        self.validate_dependencies = other.validate_dependencies;
        self.custom_rules.extend(other.custom_rules);
        self.strict_mode = other.strict_mode;
        Ok(())
    }
}

// Default value functions

fn default_version() -> String {
    "1.0".to_string()
}

fn default_detection_order() -> Vec<PackageManagerKind> {
    // Check environment variable for custom detection order
    if let Ok(env_order) = std::env::var("SUBLIME_PACKAGE_MANAGER_ORDER") {
        let mut order = Vec::new();
        for manager_name in env_order.split(',') {
            match manager_name.trim().to_lowercase().as_str() {
                "npm" => order.push(PackageManagerKind::Npm),
                "yarn" => order.push(PackageManagerKind::Yarn),
                "pnpm" => order.push(PackageManagerKind::Pnpm),
                "bun" => order.push(PackageManagerKind::Bun),
                "jsr" => order.push(PackageManagerKind::Jsr),
                _ => {} // Ignore unknown package managers
            }
        }
        if !order.is_empty() {
            return order;
        }
    }

    // Fallback to hardcoded default order
    vec![
        PackageManagerKind::Bun,
        PackageManagerKind::Pnpm,
        PackageManagerKind::Yarn,
        PackageManagerKind::Npm,
        PackageManagerKind::Jsr,
    ]
}

fn default_package_manager_env() -> String {
    "SUBLIME_PACKAGE_MANAGER".to_string()
}

fn default_workspace_patterns() -> Vec<String> {
    // Check environment variable for custom workspace patterns
    if let Ok(env_patterns) = std::env::var("SUBLIME_WORKSPACE_PATTERNS") {
        let patterns: Vec<String> = env_patterns
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect();
        if !patterns.is_empty() {
            return patterns;
        }
    }

    // Fallback to hardcoded default patterns
    vec![
        "packages/*".to_string(),
        "apps/*".to_string(),
        "libs/*".to_string(),
        "modules/*".to_string(),
        "components/*".to_string(),
        "services/*".to_string(),
    ]
}

fn default_package_directories() -> Vec<String> {
    // Check environment variable for custom package directories
    if let Ok(env_dirs) = std::env::var("SUBLIME_PACKAGE_DIRECTORIES") {
        let directories: Vec<String> =
            env_dirs.split(',').map(|d| d.trim().to_string()).filter(|d| !d.is_empty()).collect();
        if !directories.is_empty() {
            return directories;
        }
    }

    // Fallback to hardcoded default directories
    vec![
        "packages".to_string(),
        "apps".to_string(),
        "libs".to_string(),
        "components".to_string(),
        "modules".to_string(),
        "services".to_string(),
        "tools".to_string(),
        "shared".to_string(),
        "core".to_string(),
    ]
}

fn default_exclude_patterns() -> Vec<String> {
    // Check environment variable for custom exclude patterns
    if let Ok(env_excludes) = std::env::var("SUBLIME_EXCLUDE_PATTERNS") {
        let patterns: Vec<String> = env_excludes
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect();
        if !patterns.is_empty() {
            return patterns;
        }
    }

    // Fallback to hardcoded default exclude patterns
    vec![
        "node_modules".to_string(),
        ".git".to_string(),
        "dist".to_string(),
        "build".to_string(),
        "coverage".to_string(),
        ".next".to_string(),
        ".nuxt".to_string(),
        "out".to_string(),
    ]
}

fn default_max_depth() -> usize {
    // Check environment variable for custom max search depth
    if let Ok(env_depth) = std::env::var("SUBLIME_MAX_SEARCH_DEPTH")
        && let Ok(depth) = env_depth.trim().parse::<usize>()
        && (1..=20).contains(&depth)
    {
        // Reasonable bounds
        return depth;
    }

    // Fallback to hardcoded default depth
    5
}

fn default_command_timeout() -> Duration {
    // Check environment variable for custom command timeout
    if let Ok(env_timeout) = std::env::var("SUBLIME_COMMAND_TIMEOUT")
        && let Ok(seconds) = env_timeout.trim().parse::<u64>()
        && seconds > 0
        && seconds <= 3600
    {
        // Max 1 hour
        return Duration::from_secs(seconds);
    }

    // Fallback to hardcoded default timeout
    Duration::from_secs(30)
}

fn default_buffer_size() -> usize {
    // Check environment variable for custom buffer size
    if let Ok(env_buffer) = std::env::var("SUBLIME_BUFFER_SIZE")
        && let Ok(buffer_size) = env_buffer.trim().parse::<usize>()
            && (256..=65536).contains(&buffer_size) {
                // Reasonable bounds: 256B to 64KB
                return buffer_size;
            }

    // Fallback to hardcoded default buffer size
    1024
}

fn default_stream_timeout() -> Duration {
    Duration::from_secs(1)
}

fn default_max_concurrent() -> usize {
    // Check environment variable for custom max concurrent commands
    if let Ok(env_concurrent) = std::env::var("SUBLIME_MAX_CONCURRENT")
        && let Ok(max_concurrent) = env_concurrent.trim().parse::<usize>()
            && (1..=100).contains(&max_concurrent) {
                // Reasonable bounds
                return max_concurrent;
            }

    // Fallback to hardcoded default
    4
}

fn default_ignore_patterns() -> Vec<String> {
    // Check environment variable for custom ignore patterns
    if let Ok(env_ignores) = std::env::var("SUBLIME_IGNORE_PATTERNS") {
        let patterns: Vec<String> = env_ignores
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect();
        if !patterns.is_empty() {
            return patterns;
        }
    }

    // Fallback to hardcoded default ignore patterns
    vec![
        ".git".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        ".DS_Store".to_string(),
        "Thumbs.db".to_string(),
    ]
}

fn default_async_buffer_size() -> usize {
    // Check environment variable for custom async buffer size
    if let Ok(env_buffer) = std::env::var("SUBLIME_ASYNC_BUFFER_SIZE")
        && let Ok(buffer_size) = env_buffer.trim().parse::<usize>()
            && (1024..=1_048_576).contains(&buffer_size) {
                // Reasonable bounds: 1KB to 1MB
                return buffer_size;
            }

    // Fallback to hardcoded default async buffer size
    8192
}

fn default_max_concurrent_io() -> usize {
    // Check environment variable for custom max concurrent I/O operations
    if let Ok(env_io) = std::env::var("SUBLIME_MAX_CONCURRENT_IO")
        && let Ok(max_io) = env_io.trim().parse::<usize>()
            && (1..=1000).contains(&max_io) {
                // Reasonable bounds
                return max_io;
            }

    // Fallback to hardcoded default max concurrent I/O
    10
}

fn default_io_timeout() -> Duration {
    // Check environment variable for custom I/O timeout
    if let Ok(env_timeout) = std::env::var("SUBLIME_IO_TIMEOUT")
        && let Ok(seconds) = env_timeout.trim().parse::<u64>()
            && seconds > 0 && seconds <= 300 {
                // Max 5 minutes
                return Duration::from_secs(seconds);
            }

    // Fallback to hardcoded default I/O timeout
    Duration::from_secs(5)
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> Duration {
    Duration::from_millis(100)
}

fn default_max_retry_delay() -> Duration {
    Duration::from_secs(5)
}

fn default_backoff_multiplier() -> f64 {
    2.0
}

fn default_true() -> bool {
    true
}

fn default_custom_workspace_fields() -> Vec<String> {
    // Check environment variable for custom workspace fields
    if let Ok(env_fields) = std::env::var("SUBLIME_CUSTOM_WORKSPACE_FIELDS") {
        let fields: Vec<String> =
            env_fields.split(',').map(|f| f.trim().to_string()).filter(|f| !f.is_empty()).collect();
        if !fields.is_empty() {
            return fields;
        }
    }

    // Fallback to common workspace field patterns
    vec!["@myorg/".to_string()]
}

fn default_collection_window_ms() -> u64 {
    // Check environment variable for custom collection window
    if let Ok(env_window) = std::env::var("SUBLIME_COLLECTION_WINDOW_MS")
        && let Ok(window_ms) = env_window.trim().parse::<u64>()
            && (1..=1000).contains(&window_ms) {
                // Reasonable bounds: 1ms to 1s
                return window_ms;
            }

    // Fallback to hardcoded default
    5
}

fn default_collection_sleep_us() -> u64 {
    // Check environment variable for custom collection sleep
    if let Ok(env_sleep) = std::env::var("SUBLIME_COLLECTION_SLEEP_US")
        && let Ok(sleep_us) = env_sleep.trim().parse::<u64>()
            && (10..=10_000).contains(&sleep_us) {
                // Reasonable bounds: 10μs to 10ms
                return sleep_us;
            }

    // Fallback to hardcoded default
    100
}

fn default_idle_sleep_ms() -> u64 {
    // Check environment variable for custom idle sleep
    if let Ok(env_idle) = std::env::var("SUBLIME_IDLE_SLEEP_MS")
        && let Ok(idle_ms) = env_idle.trim().parse::<u64>()
            && (1..=1000).contains(&idle_ms) {
                // Reasonable bounds: 1ms to 1s
                return idle_ms;
            }

    // Fallback to hardcoded default
    10
}
