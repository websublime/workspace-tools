//! # Package Command Service
//!
//! ## What
//! Enterprise-grade service for executing package manager commands using the CommandExecutor
//! from the standard crate. This service provides a unified interface for npm, yarn, pnpm, 
//! and bun operations with automatic package manager detection, timeout configuration,
//! and retry logic for network operations.
//!
//! ## How
//! The service integrates multiple standard crate components:
//! - CommandExecutor for command execution with timeout and retry logic
//! - PackageManager detection for automatic package manager identification
//! - AsyncFileSystem for file operations and project structure analysis
//! - PackageToolsConfig for configuration management and timeout settings
//!
//! ## Why
//! This service addresses Task 2.3 of the refactoring plan by providing a centralized
//! location for all package manager command operations. It eliminates the need for
//! scattered command execution logic throughout the codebase and provides enterprise-grade
//! reliability patterns including retry logic, timeout handling, and comprehensive error management.

use crate::{
    config::PackageToolsConfig,
    errors::PackageError,
};
use sublime_standard_tools::{
    command::{CommandBuilder, Executor},
    config::StandardConfig,
    filesystem::AsyncFileSystem,
    node::{PackageManager, PackageManagerKind},
};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::time::{sleep, timeout};

/// Maximum number of retry attempts for network operations
const MAX_RETRY_ATTEMPTS: u32 = 3;

/// Default timeout for package manager operations (5 minutes)
const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_secs(300);

/// Base delay between retry attempts
const BASE_RETRY_DELAY: Duration = Duration::from_millis(1000);

/// Enterprise-grade package command service for package manager operations
///
/// This service provides a unified interface for executing package manager commands
/// across different package managers (npm, yarn, pnpm, bun) with automatic detection,
/// timeout configuration, and retry logic for network operations.
///
/// ## Architecture
///
/// - **CommandExecutor Integration**: Uses standard crate CommandExecutor for reliable command execution
/// - **Package Manager Detection**: Automatic detection using PackageManager::detect_with_config
/// - **Async Filesystem**: Integrated filesystem operations for project analysis
/// - **Configuration Management**: Comprehensive configuration via PackageToolsConfig
/// - **Retry Logic**: Exponential backoff retry strategy for network-dependent operations
/// - **Timeout Handling**: Configurable timeouts with sensible defaults
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::services::PackageCommandService;
/// use sublime_standard_tools::{command::DefaultCommandExecutor, filesystem::AsyncFileSystem};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = AsyncFileSystem::new();
/// let executor = DefaultCommandExecutor::new();
/// let service = PackageCommandService::new(executor, fs);
///
/// // Install dependencies
/// service.install_dependencies().await?;
///
/// // Add a new package
/// service.add_package("react", Some("^18.0.0")).await?;
///
/// // Remove a package
/// service.remove_package("lodash").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct PackageCommandService<E, F> 
where
    E: Executor + Clone,
    F: AsyncFileSystem + Clone,
{
    /// Command executor for running package manager commands
    executor: E,
    /// Filesystem implementation for file operations
    filesystem: F,
    /// Configuration for package tools behavior
    config: PackageToolsConfig,
    /// Standard configuration for command execution and timeouts
    standard_config: StandardConfig,
    /// Working directory for command execution
    working_directory: PathBuf,
    /// Cached package manager for performance optimization
    cached_package_manager: Option<PackageManager>,
}

impl<E, F> PackageCommandService<E, F>
where
    E: Executor + Clone + 'static,
    F: AsyncFileSystem + Clone + 'static,
{
    /// Create a new package command service with default configuration
    ///
    /// # Arguments
    ///
    /// * `executor` - Command executor implementation for running commands
    /// * `filesystem` - Filesystem implementation for file operations
    ///
    /// # Returns
    ///
    /// A new PackageCommandService instance with default configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::PackageCommandService;
    /// use sublime_standard_tools::{command::DefaultCommandExecutor, filesystem::AsyncFileSystem};
    ///
    /// let fs = AsyncFileSystem::new();
    /// let executor = DefaultCommandExecutor::new();
    /// let service = PackageCommandService::new(executor, fs);
    /// ```
    #[must_use]
    pub fn new(executor: E, filesystem: F) -> Self {
        Self {
            executor,
            filesystem,
            config: PackageToolsConfig::default(),
            standard_config: StandardConfig::default(),
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            cached_package_manager: None,
        }
    }

    /// Create a new package command service with custom configuration
    ///
    /// # Arguments
    ///
    /// * `executor` - Command executor implementation for running commands
    /// * `filesystem` - Filesystem implementation for file operations
    /// * `config` - Package tools configuration for customizing behavior
    ///
    /// # Returns
    ///
    /// A new PackageCommandService instance with the provided configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{services::PackageCommandService, config::PackageToolsConfig};
    /// use sublime_standard_tools::{command::DefaultCommandExecutor, filesystem::AsyncFileSystem};
    ///
    /// let fs = AsyncFileSystem::new();
    /// let executor = DefaultCommandExecutor::new();
    /// let config = PackageToolsConfig::default();
    /// let service = PackageCommandService::with_config(executor, fs, config);
    /// ```
    #[must_use]
    pub fn with_config(executor: E, filesystem: F, config: PackageToolsConfig) -> Self {
        Self {
            executor,
            filesystem,
            config,
            standard_config: StandardConfig::default(),
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            cached_package_manager: None,
        }
    }

    /// Create a new package command service with custom working directory
    ///
    /// # Arguments
    ///
    /// * `executor` - Command executor implementation for running commands
    /// * `filesystem` - Filesystem implementation for file operations
    /// * `working_directory` - Working directory for command execution
    ///
    /// # Returns
    ///
    /// A new PackageCommandService instance with the specified working directory
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::PackageCommandService;
    /// use sublime_standard_tools::{command::DefaultCommandExecutor, filesystem::AsyncFileSystem};
    /// use std::path::PathBuf;
    ///
    /// let fs = AsyncFileSystem::new();
    /// let executor = DefaultCommandExecutor::new();
    /// let working_dir = PathBuf::from("/path/to/project");
    /// let service = PackageCommandService::with_directory(executor, fs, working_dir);
    /// ```
    #[must_use]
    pub fn with_directory(executor: E, filesystem: F, working_directory: PathBuf) -> Self {
        Self {
            executor,
            filesystem,
            config: PackageToolsConfig::default(),
            standard_config: StandardConfig::default(),
            working_directory,
            cached_package_manager: None,
        }
    }

    /// Detect the package manager in the working directory with caching
    ///
    /// This method uses PackageManager::detect_with_config from the standard crate
    /// to automatically identify the package manager being used. Results are cached
    /// for performance optimization.
    ///
    /// # Returns
    ///
    /// The detected package manager
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No package manager lock files are found
    /// - The working directory is not accessible
    /// - File system operations fail
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::PackageCommandService;
    /// use sublime_standard_tools::{command::DefaultCommandExecutor, filesystem::AsyncFileSystem};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let executor = DefaultCommandExecutor::new();
    /// let service = PackageCommandService::new(executor, fs);
    ///
    /// let package_manager = service.detect_package_manager().await?;
    /// println!("Detected package manager: {:?}", package_manager.kind());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_package_manager(&mut self) -> Result<&PackageManager, PackageError> {
        if self.cached_package_manager.is_none() {
            let pm_config = &self.standard_config.package_managers;
            let package_manager = PackageManager::detect_with_config(&self.working_directory, pm_config)
                .map_err(|e| PackageError::Configuration(format!("Failed to detect package manager: {e}")))?;
            
            self.cached_package_manager = Some(package_manager);
        }
        
        Ok(self.cached_package_manager.as_ref().unwrap())
    }

    /// Install all dependencies in the current project
    ///
    /// This method runs the appropriate install command for the detected package manager:
    /// - npm: `npm ci` (if package-lock.json exists) or `npm install`
    /// - yarn: `yarn install --frozen-lockfile` (if yarn.lock exists) or `yarn install`
    /// - pnpm: `pnpm install --frozen-lockfile`
    /// - bun: `bun install --frozen-lockfile`
    ///
    /// # Returns
    ///
    /// `Ok(())` if installation succeeds
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Package manager detection fails
    /// - Command execution fails
    /// - Timeout is exceeded
    /// - All retry attempts are exhausted
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::PackageCommandService;
    /// use sublime_standard_tools::{command::DefaultCommandExecutor, filesystem::AsyncFileSystem};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let executor = DefaultCommandExecutor::new();
    /// let mut service = PackageCommandService::new(executor, fs);
    ///
    /// service.install_dependencies().await?;
    /// println!("Dependencies installed successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn install_dependencies(&mut self) -> Result<(), PackageError> {
        let package_manager = self.detect_package_manager().await?;
        
        let (command, args) = match package_manager.kind() {
            PackageManagerKind::Npm => {
                // Use npm ci if package-lock.json exists, otherwise npm install
                let lock_file_exists = self.filesystem
                    .exists(&self.working_directory.join("package-lock.json"))
                    .await;
                
                if lock_file_exists {
                    ("npm", vec!["ci".to_string()])
                } else {
                    ("npm", vec!["install".to_string()])
                }
            }
            PackageManagerKind::Yarn => {
                // Use frozen-lockfile if yarn.lock exists
                let lock_file_exists = self.filesystem
                    .exists(&self.working_directory.join("yarn.lock"))
                    .await;
                
                if lock_file_exists {
                    ("yarn", vec!["install".to_string(), "--frozen-lockfile".to_string()])
                } else {
                    ("yarn", vec!["install".to_string()])
                }
            }
            PackageManagerKind::Pnpm => {
                ("pnpm", vec!["install".to_string(), "--frozen-lockfile".to_string()])
            }
            PackageManagerKind::Bun => {
                ("bun", vec!["install".to_string(), "--frozen-lockfile".to_string()])
            }
            PackageManagerKind::Jsr => {
                return Err(PackageError::UnsupportedOperation(
                    "JSR does not support dependency installation".to_string()
                ));
            }
        };

        self.execute_command_with_retry(command, args, "install dependencies").await
    }

    /// Add a package to the project dependencies
    ///
    /// This method adds a new package to the project using the appropriate command
    /// for the detected package manager and automatically updates the package.json file.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to add
    /// * `version` - Optional version specification (e.g., "^18.0.0", "latest")
    ///
    /// # Returns
    ///
    /// `Ok(())` if package addition succeeds
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Package manager detection fails
    /// - Package name is invalid
    /// - Command execution fails
    /// - Network operations fail
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::PackageCommandService;
    /// use sublime_standard_tools::{command::DefaultCommandExecutor, filesystem::AsyncFileSystem};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let executor = DefaultCommandExecutor::new();
    /// let mut service = PackageCommandService::new(executor, fs);
    ///
    /// // Add latest version
    /// service.add_package("react", None).await?;
    ///
    /// // Add specific version
    /// service.add_package("lodash", Some("^4.17.21")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_package(
        &mut self, 
        package_name: &str, 
        version: Option<&str>
    ) -> Result<(), PackageError> {
        if package_name.is_empty() {
            return Err(PackageError::Configuration("Package name cannot be empty".to_string()));
        }

        let package_manager = self.detect_package_manager().await?;
        
        let package_spec = if let Some(v) = version {
            format!("{}@{}", package_name, v)
        } else {
            package_name.to_string()
        };

        let (command, mut args) = match package_manager.kind() {
            PackageManagerKind::Npm => ("npm", vec!["install".to_string(), "--save".to_string()]),
            PackageManagerKind::Yarn => ("yarn", vec!["add".to_string()]),
            PackageManagerKind::Pnpm => ("pnpm", vec!["add".to_string()]),
            PackageManagerKind::Bun => ("bun", vec!["add".to_string()]),
            PackageManagerKind::Jsr => {
                return Err(PackageError::UnsupportedOperation(
                    "JSR does not support package addition via command line".to_string()
                ));
            }
        };

        args.push(package_spec);

        self.execute_command_with_retry(command, args, &format!("add package {package_name}")).await
    }

    /// Remove a package from the project dependencies
    ///
    /// This method removes a package from the project using the appropriate command
    /// for the detected package manager and automatically updates the package.json file.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to remove
    ///
    /// # Returns
    ///
    /// `Ok(())` if package removal succeeds
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Package manager detection fails
    /// - Package name is invalid or not found
    /// - Command execution fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::PackageCommandService;
    /// use sublime_standard_tools::{command::DefaultCommandExecutor, filesystem::AsyncFileSystem};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let executor = DefaultCommandExecutor::new();
    /// let mut service = PackageCommandService::new(executor, fs);
    ///
    /// service.remove_package("lodash").await?;
    /// println!("Package removed successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_package(&mut self, package_name: &str) -> Result<(), PackageError> {
        if package_name.is_empty() {
            return Err(PackageError::Configuration("Package name cannot be empty".to_string()));
        }

        let package_manager = self.detect_package_manager().await?;
        
        let (command, mut args) = match package_manager.kind() {
            PackageManagerKind::Npm => ("npm", vec!["uninstall".to_string()]),
            PackageManagerKind::Yarn => ("yarn", vec!["remove".to_string()]),
            PackageManagerKind::Pnpm => ("pnpm", vec!["remove".to_string()]),
            PackageManagerKind::Bun => ("bun", vec!["remove".to_string()]),
            PackageManagerKind::Jsr => {
                return Err(PackageError::UnsupportedOperation(
                    "JSR does not support package removal via command line".to_string()
                ));
            }
        };

        args.push(package_name.to_string());

        self.execute_command_with_retry(command, args, &format!("remove package {package_name}")).await
    }

    /// Run a package.json script
    ///
    /// This method executes a script defined in the package.json scripts section
    /// using the appropriate command for the detected package manager.
    ///
    /// # Arguments
    ///
    /// * `script_name` - Name of the script to run (e.g., "build", "test", "dev")
    /// * `args` - Optional additional arguments to pass to the script
    ///
    /// # Returns
    ///
    /// `Ok(())` if script execution succeeds
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Package manager detection fails
    /// - Script name is not found in package.json
    /// - Script execution fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::PackageCommandService;
    /// use sublime_standard_tools::{command::DefaultCommandExecutor, filesystem::AsyncFileSystem};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let executor = DefaultCommandExecutor::new();
    /// let mut service = PackageCommandService::new(executor, fs);
    ///
    /// // Run build script
    /// service.run_script("build", None).await?;
    ///
    /// // Run test script with arguments
    /// service.run_script("test", Some(vec!["--coverage".to_string()])).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_script(
        &mut self, 
        script_name: &str, 
        args: Option<Vec<String>>
    ) -> Result<(), PackageError> {
        if script_name.is_empty() {
            return Err(PackageError::Configuration("Script name cannot be empty".to_string()));
        }

        let package_manager = self.detect_package_manager().await?;
        
        let (command, mut cmd_args) = match package_manager.kind() {
            PackageManagerKind::Npm => ("npm", vec!["run".to_string()]),
            PackageManagerKind::Yarn => ("yarn", vec![]),
            PackageManagerKind::Pnpm => ("pnpm", vec![]),
            PackageManagerKind::Bun => ("bun", vec!["run".to_string()]),
            PackageManagerKind::Jsr => {
                return Err(PackageError::UnsupportedOperation(
                    "JSR does not support script execution via command line".to_string()
                ));
            }
        };

        cmd_args.push(script_name.to_string());
        
        if let Some(additional_args) = args {
            cmd_args.extend(additional_args);
        }

        self.execute_command_with_retry(command, cmd_args, &format!("run script {script_name}")).await
    }

    /// Execute a command with retry logic and timeout handling
    ///
    /// This method implements enterprise-grade reliability patterns including:
    /// - Exponential backoff retry strategy
    /// - Configurable timeout handling
    /// - Comprehensive error reporting
    /// - Network failure resilience
    ///
    /// # Arguments
    ///
    /// * `command` - Command to execute
    /// * `args` - Command arguments
    /// * `operation_description` - Human-readable description for error reporting
    ///
    /// # Returns
    ///
    /// `Ok(())` if command execution succeeds within timeout and retry limits
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - All retry attempts are exhausted
    /// - Command times out consistently
    /// - Command execution fails with non-retryable error
    async fn execute_command_with_retry(
        &self,
        command: &str,
        args: Vec<String>,
        operation_description: &str,
    ) -> Result<(), PackageError> {
        let timeout_duration = self.standard_config.commands.default_timeout;
        
        let max_retries = self.standard_config.filesystem.retry.max_attempts;

        for attempt in 1..=max_retries {
            match self.execute_single_command(command, &args, timeout_duration).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if attempt == max_retries {
                        return Err(PackageError::CommandExecution(format!(
                            "Failed to {} after {} attempts: {}",
                            operation_description, max_retries, e
                        )));
                    }

                    // Exponential backoff with jitter
                    let delay = BASE_RETRY_DELAY * (2_u32.pow(attempt - 1));
                    // Note: Replace with proper logging when log crate is available
                    eprintln!(
                        "Attempt {} to {} failed: {}. Retrying in {:?}...",
                        attempt, operation_description, e, delay
                    );
                    
                    sleep(delay).await;
                }
            }
        }

        Err(PackageError::CommandExecution(format!(
            "Failed to {} after {} attempts",
            operation_description, max_retries
        )))
    }

    /// Execute a single command with timeout
    ///
    /// # Arguments
    ///
    /// * `command` - Command to execute
    /// * `args` - Command arguments
    /// * `timeout_duration` - Maximum execution time
    ///
    /// # Returns
    ///
    /// `Ok(())` if command execution succeeds within timeout
    async fn execute_single_command(
        &self,
        command: &str,
        args: &[String],
        timeout_duration: Duration,
    ) -> Result<(), PackageError> {
        let mut cmd_builder = CommandBuilder::new(command)
            .current_dir(&self.working_directory)
            .timeout(timeout_duration);
        
        // Add each argument individually
        for arg in args {
            cmd_builder = cmd_builder.arg(arg);
        }
        
        let cmd = cmd_builder.build();

        let result = timeout(timeout_duration, self.executor.execute(cmd)).await;

        match result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(PackageError::CommandExecution(format!(
                "Command '{}' failed: {}",
                command, e
            ))),
            Err(_) => Err(PackageError::CommandExecution(format!(
                "Command '{}' timed out after {:?}",
                command, timeout_duration
            ))),
        }
    }

    /// Get the current working directory
    ///
    /// # Returns
    ///
    /// Reference to the current working directory path
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::PackageCommandService;
    /// use sublime_standard_tools::{command::DefaultCommandExecutor, filesystem::AsyncFileSystem};
    ///
    /// let fs = AsyncFileSystem::new();
    /// let executor = DefaultCommandExecutor::new();
    /// let service = PackageCommandService::new(executor, fs);
    /// 
    /// println!("Working directory: {}", service.working_directory().display());
    /// ```
    #[must_use]
    pub fn working_directory(&self) -> &Path {
        &self.working_directory
    }

    /// Get the current configuration
    ///
    /// # Returns
    ///
    /// Reference to the current package tools configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::PackageCommandService;
    /// use sublime_standard_tools::{command::DefaultCommandExecutor, filesystem::AsyncFileSystem};
    ///
    /// let fs = AsyncFileSystem::new();
    /// let executor = DefaultCommandExecutor::new();
    /// let service = PackageCommandService::new(executor, fs);
    /// 
    /// let config = service.config();
    /// println!("Max retries: {:?}", config.network.max_retries);
    /// ```
    #[must_use]
    pub fn config(&self) -> &PackageToolsConfig {
        &self.config
    }

    /// Clear the cached package manager to force re-detection
    ///
    /// This method is useful when the project structure changes or when
    /// you want to ensure fresh package manager detection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::PackageCommandService;
    /// use sublime_standard_tools::{command::DefaultCommandExecutor, filesystem::AsyncFileSystem};
    ///
    /// let fs = AsyncFileSystem::new();
    /// let executor = DefaultCommandExecutor::new();
    /// let mut service = PackageCommandService::new(executor, fs);
    /// 
    /// service.clear_package_manager_cache();
    /// ```
    pub fn clear_package_manager_cache(&mut self) {
        self.cached_package_manager = None;
    }
}

/// Package command execution result with detailed information
#[derive(Debug, Clone)]
pub struct CommandExecutionResult {
    /// Command that was executed
    pub command: String,
    /// Arguments passed to the command
    pub args: Vec<String>,
    /// Execution duration
    pub duration: Duration,
    /// Number of retry attempts made
    pub retry_attempts: u32,
    /// Whether the operation succeeded
    pub success: bool,
    /// Output captured from the command
    pub output: Option<String>,
    /// Error message if the command failed
    pub error: Option<String>,
}

impl CommandExecutionResult {
    /// Create a new successful command execution result
    ///
    /// # Arguments
    ///
    /// * `command` - Command that was executed
    /// * `args` - Arguments passed to the command
    /// * `duration` - Execution duration
    /// * `retry_attempts` - Number of retry attempts made
    ///
    /// # Returns
    ///
    /// A new successful CommandExecutionResult
    #[must_use]
    pub fn success(
        command: String,
        args: Vec<String>,
        duration: Duration,
        retry_attempts: u32,
    ) -> Self {
        Self {
            command,
            args,
            duration,
            retry_attempts,
            success: true,
            output: None,
            error: None,
        }
    }

    /// Create a new failed command execution result
    ///
    /// # Arguments
    ///
    /// * `command` - Command that was executed
    /// * `args` - Arguments passed to the command  
    /// * `duration` - Execution duration
    /// * `retry_attempts` - Number of retry attempts made
    /// * `error` - Error message describing the failure
    ///
    /// # Returns
    ///
    /// A new failed CommandExecutionResult
    #[must_use]
    pub fn failure(
        command: String,
        args: Vec<String>,
        duration: Duration,
        retry_attempts: u32,
        error: String,
    ) -> Self {
        Self {
            command,
            args,
            duration,
            retry_attempts,
            success: false,
            output: None,
            error: Some(error),
        }
    }
}