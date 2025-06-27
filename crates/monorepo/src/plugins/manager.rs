//! Plugin manager implementation
//!
//! Central management system for loading, organizing, and executing plugins.
//! Handles plugin lifecycle, command execution, and error handling.

use super::types::{MonorepoPlugin, PluginContext, PluginInfo, PluginLifecycle, PluginResult};
use crate::error::{Error, Result};
use crate::logging::{log_operation, log_operation_start, log_operation_complete, log_operation_error, log_performance, ErrorContext};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Central plugin management system
///
/// Manages the lifecycle of all plugins including loading, initialization,
/// command execution, and cleanup. Provides a unified interface for
/// interacting with different types of plugins.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::plugins::PluginManager;
/// use sublime_monorepo_tools::core::MonorepoProject;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let project = Arc::new(MonorepoProject::new(".")?);
/// let mut manager = PluginManager::from_project(project)?;
///
/// // Load built-in plugins
/// manager.load_builtin_plugins()?;
///
/// // List available plugins
/// let plugins = manager.list_plugins();
/// for info in plugins {
///     println!("Plugin: {} v{}", info.name, info.version);
/// }
///
/// // Execute plugin command
/// if manager.has_plugin("analyzer") {
///     let result = manager.execute_plugin_command("analyzer", "custom-check", &[])?;
///     println!("Command result: {}", result.success);
/// }
/// # Ok(())
/// # }
/// ```
pub struct PluginManager {
    /// Reference to the monorepo project
    project: Arc<crate::core::MonorepoProject>,
    /// Loaded plugins indexed by name
    plugins: HashMap<String, Box<dyn MonorepoPlugin>>,
    /// Plugin states indexed by name
    plugin_states: HashMap<String, PluginLifecycle>,
    /// Plugin execution context
    context: PluginContext,
    /// Plugin execution metrics
    metrics: Arc<RwLock<PluginMetrics>>,
}

/// Plugin execution metrics and statistics
#[derive(Debug, Default)]
struct PluginMetrics {
    /// Number of commands executed per plugin
    command_counts: HashMap<String, u64>,
    /// Total execution times per plugin
    execution_times: HashMap<String, u64>,
    /// Error counts per plugin
    error_counts: HashMap<String, u64>,
}

impl PluginManager {
    /// Create a new plugin manager
    ///
    /// # Arguments
    ///
    /// * `project` - Monorepo project reference
    /// * `working_directory` - Working directory for plugin operations
    ///
    /// # Returns
    ///
    /// A new plugin manager instance
    pub fn new(
        project: Arc<crate::core::MonorepoProject>,
        working_directory: std::path::PathBuf,
    ) -> Self {
        let context = PluginContext::new(
            Arc::clone(&project),
            HashMap::new(),
            working_directory,
        );

        Self {
            project,
            plugins: HashMap::new(),
            plugin_states: HashMap::new(),
            context,
            metrics: Arc::new(RwLock::new(PluginMetrics::default())),
        }
    }

    /// Create plugin manager from project
    ///
    /// # Arguments
    ///
    /// * `project` - Monorepo project reference
    ///
    /// # Returns
    ///
    /// A new plugin manager instance using project root as working directory
    ///
    /// # Errors
    ///
    /// Returns an error if the project cannot be accessed
    pub fn from_project(project: Arc<crate::core::MonorepoProject>) -> Result<Self> {
        let working_directory = project.root_path().to_path_buf();
        Ok(Self::new(project, working_directory))
    }

    /// Load a plugin into the manager
    ///
    /// # Arguments
    ///
    /// * `plugin` - Plugin implementation to load
    ///
    /// # Returns
    ///
    /// The plugin info if successful
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be loaded or initialized
    pub fn load_plugin(&mut self, mut plugin: Box<dyn MonorepoPlugin>) -> Result<PluginInfo> {
        let info = plugin.info();
        let plugin_name = info.name.clone();

        // Check if plugin is already loaded
        if self.plugins.contains_key(&plugin_name) {
            return Err(Error::plugin(format!("Plugin {} is already loaded", plugin_name)));
        }

        // Set plugin state to loading
        self.plugin_states.insert(plugin_name.clone(), PluginLifecycle::Loading);

        // Initialize the plugin
        match plugin.initialize(&self.context) {
            Ok(()) => {
                // Plugin initialized successfully
                self.plugin_states.insert(plugin_name.clone(), PluginLifecycle::Active);
                self.plugins.insert(plugin_name, plugin);
                
                log_operation_complete("plugin_load", Some(&format!("{} v{}", info.name, info.version)));
                Ok(info)
            }
            Err(e) => {
                // Plugin initialization failed
                self.plugin_states.insert(plugin_name.clone(), PluginLifecycle::Errored);
                
                ErrorContext::new("plugin_initialize")
                    .with_detail("plugin", &plugin_name)
                    .with_detail("version", &info.version)
                    .log_error(&e);
                
                Err(Error::plugin(format!("Failed to initialize plugin {}: {}", plugin_name, e)))
            }
        }
    }

    /// Unload a plugin from the manager
    ///
    /// # Arguments
    ///
    /// * `plugin_name` - Name of the plugin to unload
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be unloaded
    pub fn unload_plugin(&mut self, plugin_name: &str) -> Result<()> {
        if let Some(mut plugin) = self.plugins.remove(plugin_name) {
            self.plugin_states.insert(plugin_name.to_string(), PluginLifecycle::Unloading);
            
            // Clean up plugin resources
            if let Err(e) = plugin.cleanup() {
                log_operation_error("plugin_cleanup", &e, Some(plugin_name));
            }

            self.plugin_states.remove(plugin_name);
            log_operation_complete("plugin_unload", Some(plugin_name));
            Ok(())
        } else {
            Err(Error::plugin(format!("Plugin {} is not loaded", plugin_name)))
        }
    }

    /// Check if a plugin is loaded
    ///
    /// # Arguments
    ///
    /// * `plugin_name` - Name of the plugin to check
    ///
    /// # Returns
    ///
    /// True if the plugin is loaded and active
    pub fn has_plugin(&self, plugin_name: &str) -> bool {
        self.plugins.contains_key(plugin_name) &&
        matches!(self.plugin_states.get(plugin_name), Some(PluginLifecycle::Active))
    }

    /// Get information about a loaded plugin
    ///
    /// # Arguments
    ///
    /// * `plugin_name` - Name of the plugin
    ///
    /// # Returns
    ///
    /// Plugin information if found
    pub fn get_plugin_info(&self, plugin_name: &str) -> Option<PluginInfo> {
        self.plugins.get(plugin_name).map(|plugin| plugin.info())
    }

    /// List all loaded plugins
    ///
    /// # Returns
    ///
    /// Vector of plugin information for all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.values().map(|plugin| plugin.info()).collect()
    }

    /// Execute a command on a specific plugin
    ///
    /// # Arguments
    ///
    /// * `plugin_name` - Name of the plugin
    /// * `command` - Command to execute
    /// * `args` - Command arguments
    ///
    /// # Returns
    ///
    /// Plugin execution result
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin is not found or command execution fails
    pub fn execute_plugin_command(
        &self,
        plugin_name: &str,
        command: &str,
        args: &[String],
    ) -> Result<PluginResult> {
        let plugin = self.plugins.get(plugin_name)
            .ok_or_else(|| Error::plugin(format!("Plugin {} not found", plugin_name)))?;

        // Check plugin state
        match self.plugin_states.get(plugin_name) {
            Some(PluginLifecycle::Active) => {},
            Some(state) => {
                return Err(Error::plugin(format!(
                    "Plugin {} is not active (state: {})", 
                    plugin_name, 
                    state
                )));
            }
            None => {
                return Err(Error::plugin(format!("Plugin {} state unknown", plugin_name)));
            }
        }

        let start_time = Instant::now();
        
        // Execute the command
        let result = match plugin.execute_command(command, args) {
            Ok(mut result) => {
                result.execution_time_ms = start_time.elapsed().as_millis() as u64;
                
                // Update metrics
                self.update_metrics(plugin_name, result.execution_time_ms, false);
                
                log_performance(
                    &format!("plugin_command:{}", command), 
                    result.execution_time_ms, 
                    None
                );
                
                result
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                // Update error metrics
                self.update_metrics(plugin_name, execution_time, true);
                
                ErrorContext::new("plugin_command")
                    .with_detail("plugin", plugin_name)
                    .with_detail("command", command)
                    .with_detail("execution_time_ms", &execution_time.to_string())
                    .log_error(&e);
                
                PluginResult::error(format!("Command execution failed: {}", e))
                    .with_metadata("execution_time_ms", execution_time)
                    .with_metadata("plugin_name", plugin_name)
                    .with_metadata("command", command)
            }
        };

        Ok(result)
    }

    /// Load built-in plugins
    ///
    /// Loads all built-in plugins that are available in the system.
    ///
    /// # Errors
    ///
    /// Returns an error if any built-in plugin fails to load
    pub fn load_builtin_plugins(&mut self) -> Result<Vec<PluginInfo>> {
        let mut loaded_plugins = Vec::new();

        log_operation_start("load_builtin_plugins", None);
        
        // Load analyzer plugin
        let analyzer_plugin = super::builtin::AnalyzerPlugin::new();
        match self.load_plugin(Box::new(analyzer_plugin)) {
            Ok(info) => {
                log_operation("load_builtin_plugin", "Loaded analyzer plugin", Some(&info.name));
                loaded_plugins.push(info);
            }
            Err(e) => {
                ErrorContext::new("load_builtin_plugin")
                    .with_detail("plugin_type", "analyzer")
                    .log_error(&e);
            }
        }

        // Load generator plugin
        let generator_plugin = super::builtin::GeneratorPlugin::new();
        match self.load_plugin(Box::new(generator_plugin)) {
            Ok(info) => {
                log_operation("load_builtin_plugin", "Loaded generator plugin", Some(&info.name));
                loaded_plugins.push(info);
            }
            Err(e) => {
                ErrorContext::new("load_builtin_plugin")
                    .with_detail("plugin_type", "generator")
                    .log_error(&e);
            }
        }

        // Load validator plugin
        let validator_plugin = super::builtin::ValidatorPlugin::new();
        match self.load_plugin(Box::new(validator_plugin)) {
            Ok(info) => {
                log_operation("load_builtin_plugin", "Loaded validator plugin", Some(&info.name));
                loaded_plugins.push(info);
            }
            Err(e) => {
                ErrorContext::new("load_builtin_plugin")
                    .with_detail("plugin_type", "validator")
                    .log_error(&e);
            }
        }

        log_operation_complete("load_builtin_plugins", Some(&format!("{} plugins", loaded_plugins.len())));
        Ok(loaded_plugins)
    }

    /// Get plugin execution metrics
    ///
    /// # Returns
    ///
    /// Snapshot of current plugin metrics
    pub fn get_metrics(&self) -> HashMap<String, serde_json::Value> {
        let metrics = self.metrics.read().unwrap();
        let mut result = HashMap::new();

        result.insert("command_counts".to_string(), 
                     serde_json::to_value(&metrics.command_counts).unwrap_or_default());
        result.insert("execution_times".to_string(), 
                     serde_json::to_value(&metrics.execution_times).unwrap_or_default());
        result.insert("error_counts".to_string(), 
                     serde_json::to_value(&metrics.error_counts).unwrap_or_default());

        result
    }

    /// Update plugin execution metrics
    fn update_metrics(&self, plugin_name: &str, execution_time: u64, is_error: bool) {
        if let Ok(mut metrics) = self.metrics.write() {
            // Update command count
            *metrics.command_counts.entry(plugin_name.to_string()).or_insert(0) += 1;
            
            // Update execution time
            *metrics.execution_times.entry(plugin_name.to_string()).or_insert(0) += execution_time;
            
            // Update error count if applicable
            if is_error {
                *metrics.error_counts.entry(plugin_name.to_string()).or_insert(0) += 1;
            }
        }
    }

    /// Suspend a plugin (temporarily disable)
    ///
    /// # Arguments
    ///
    /// * `plugin_name` - Name of the plugin to suspend
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be suspended
    pub fn suspend_plugin(&mut self, plugin_name: &str) -> Result<()> {
        if self.plugins.contains_key(plugin_name) {
            self.plugin_states.insert(plugin_name.to_string(), PluginLifecycle::Suspended);
            log_operation("plugin_suspend", "Plugin suspended", Some(plugin_name));
            Ok(())
        } else {
            Err(Error::plugin(format!("Plugin {} not found", plugin_name)))
        }
    }

    /// Resume a suspended plugin
    ///
    /// # Arguments
    ///
    /// * `plugin_name` - Name of the plugin to resume
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be resumed
    pub fn resume_plugin(&mut self, plugin_name: &str) -> Result<()> {
        match self.plugin_states.get(plugin_name) {
            Some(PluginLifecycle::Suspended) => {
                self.plugin_states.insert(plugin_name.to_string(), PluginLifecycle::Active);
                log_operation("plugin_resume", "Plugin resumed", Some(plugin_name));
                Ok(())
            }
            Some(state) => {
                Err(Error::plugin(format!(
                    "Plugin {} cannot be resumed (current state: {})", 
                    plugin_name, 
                    state
                )))
            }
            None => {
                Err(Error::plugin(format!("Plugin {} not found", plugin_name)))
            }
        }
    }

    /// Get the current state of a plugin
    ///
    /// # Arguments
    ///
    /// * `plugin_name` - Name of the plugin
    ///
    /// # Returns
    ///
    /// Current plugin lifecycle state if found
    pub fn get_plugin_state(&self, plugin_name: &str) -> Option<PluginLifecycle> {
        self.plugin_states.get(plugin_name).copied()
    }
}