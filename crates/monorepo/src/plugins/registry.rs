//! Plugin registry for plugin discovery and metadata management
//!
//! Provides a centralized registry for discovering, registering, and managing
//! plugin metadata. Supports both built-in and external plugin discovery.

use super::types::{PluginInfo, PluginCapabilities};
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Central registry for plugin discovery and management
///
/// Manages plugin metadata, discovery paths, and provides functionality
/// for finding and loading plugins from various sources.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::plugins::PluginRegistry;
///
/// let mut registry = PluginRegistry::new();
///
/// // Add plugin discovery path
/// registry.add_discovery_path("/path/to/plugins");
///
/// // Register built-in plugin
/// registry.register_builtin("analyzer", "1.0.0", "Code analyzer plugin");
///
/// // Discover available plugins
/// let available = registry.discover_plugins()?;
/// for plugin in available {
///     println!("Found plugin: {} v{}", plugin.name, plugin.version);
/// }
/// ```
pub struct PluginRegistry {
    /// Registered plugin metadata
    plugins: HashMap<String, PluginRegistryEntry>,
    /// Plugin discovery paths
    discovery_paths: Vec<PathBuf>,
    /// Built-in plugin definitions
    builtin_plugins: HashMap<String, PluginInfo>,
}

/// Plugin registry entry with metadata and location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRegistryEntry {
    /// Plugin information
    pub info: PluginInfo,
    /// Plugin source type
    pub source: PluginSource,
    /// Plugin location (path or identifier)
    pub location: String,
    /// Whether plugin is currently available
    pub available: bool,
    /// Last discovery/validation timestamp
    pub last_checked: chrono::DateTime<chrono::Utc>,
}

/// Plugin source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginSource {
    /// Built-in plugin (compiled into the application)
    Builtin,
    /// External plugin (loaded from file system)
    External,
    /// Remote plugin (downloaded from registry)
    Remote,
}

/// Plugin discovery result
#[derive(Debug, Clone)]
pub struct PluginDiscoveryResult {
    /// Total plugins found
    pub total_found: usize,
    /// Available plugins
    pub available_plugins: Vec<PluginInfo>,
    /// Unavailable plugins with reasons
    pub unavailable_plugins: Vec<(String, String)>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            discovery_paths: Vec::new(),
            builtin_plugins: HashMap::new(),
        }
    }

    /// Add a directory path for plugin discovery
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path to search for plugins
    pub fn add_discovery_path<P: Into<PathBuf>>(&mut self, path: P) {
        let path = path.into();
        if !self.discovery_paths.contains(&path) {
            self.discovery_paths.push(path);
        }
    }

    /// Register a built-in plugin
    ///
    /// # Arguments
    ///
    /// * `name` - Plugin name
    /// * `version` - Plugin version
    /// * `description` - Plugin description
    pub fn register_builtin(
        &mut self,
        name: &str,
        version: &str,
        description: &str,
    ) {
        let info = PluginInfo {
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            author: "Sublime Monorepo Tools".to_string(),
            capabilities: PluginCapabilities::default(),
        };

        self.builtin_plugins.insert(name.to_string(), info.clone());

        let entry = PluginRegistryEntry {
            info,
            source: PluginSource::Builtin,
            location: "builtin".to_string(),
            available: true,
            last_checked: chrono::Utc::now(),
        };

        self.plugins.insert(name.to_string(), entry);
    }

    /// Register a plugin with full metadata
    ///
    /// # Arguments
    ///
    /// * `info` - Plugin information
    /// * `source` - Plugin source type
    /// * `location` - Plugin location
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin is already registered
    pub fn register_plugin(
        &mut self,
        info: PluginInfo,
        source: PluginSource,
        location: String,
    ) -> Result<()> {
        if self.plugins.contains_key(&info.name) {
            return Err(Error::plugin(format!("Plugin {plugin_name} is already registered", plugin_name = info.name)));
        }

        let entry = PluginRegistryEntry {
            info,
            source,
            location,
            available: true,
            last_checked: chrono::Utc::now(),
        };

        self.plugins.insert(entry.info.name.clone(), entry);
        Ok(())
    }

    /// Discover all available plugins
    ///
    /// Searches all discovery paths and validates plugin availability.
    ///
    /// # Returns
    ///
    /// Plugin discovery results
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails
    pub fn discover_plugins(&mut self) -> Result<PluginDiscoveryResult> {
        let mut available_plugins = Vec::new();
        let mut unavailable_plugins = Vec::new();

        // Include built-in plugins
        for (name, info) in &self.builtin_plugins {
            available_plugins.push(info.clone());
            log::debug!("Discovered built-in plugin: {}", name);
        }

        // Discover external plugins
        for path in &self.discovery_paths.clone() {
            match Self::discover_plugins_in_path(path) {
                Ok(plugins) => {
                    available_plugins.extend(plugins);
                }
                Err(e) => {
                    log::warn!("Failed to discover plugins in {}: {}", path.display(), e);
                    unavailable_plugins.push((
                        path.display().to_string(),
                        format!("Discovery failed: {e}"),
                    ));
                }
            }
        }

        // Validate all registered plugins
        self.validate_registered_plugins(&mut available_plugins, &mut unavailable_plugins);

        let total_found = available_plugins.len() + unavailable_plugins.len();

        Ok(PluginDiscoveryResult {
            total_found,
            available_plugins,
            unavailable_plugins,
        })
    }

    /// Get plugin information by name
    ///
    /// # Arguments
    ///
    /// * `name` - Plugin name
    ///
    /// # Returns
    ///
    /// Plugin information if found
    pub fn get_plugin(&self, name: &str) -> Option<&PluginRegistryEntry> {
        self.plugins.get(name)
    }

    /// List all registered plugins
    ///
    /// # Returns
    ///
    /// Iterator over all registered plugin entries
    pub fn list_plugins(&self) -> impl Iterator<Item = &PluginRegistryEntry> {
        self.plugins.values()
    }

    /// Check if a plugin is registered
    ///
    /// # Arguments
    ///
    /// * `name` - Plugin name
    ///
    /// # Returns
    ///
    /// True if the plugin is registered
    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Get plugins by category
    ///
    /// # Arguments
    ///
    /// * `category` - Plugin category to filter by
    ///
    /// # Returns
    ///
    /// Vector of plugin entries matching the category
    pub fn get_plugins_by_category(&self, category: &str) -> Vec<&PluginRegistryEntry> {
        self.plugins
            .values()
            .filter(|entry| entry.info.capabilities.categories.contains(&category.to_string()))
            .collect()
    }

    /// Update plugin availability status
    ///
    /// # Arguments
    ///
    /// * `name` - Plugin name
    /// * `available` - New availability status
    pub fn update_plugin_availability(&mut self, name: &str, available: bool) {
        if let Some(entry) = self.plugins.get_mut(name) {
            entry.available = available;
            entry.last_checked = chrono::Utc::now();
        }
    }

    /// Remove a plugin from the registry
    ///
    /// # Arguments
    ///
    /// * `name` - Plugin name to remove
    ///
    /// # Returns
    ///
    /// True if the plugin was removed
    pub fn remove_plugin(&mut self, name: &str) -> bool {
        self.plugins.remove(name).is_some()
    }

    /// Discover plugins in a specific directory path
    fn discover_plugins_in_path(path: &PathBuf) -> Result<Vec<PluginInfo>> {
        let mut plugins = Vec::new();

        if !path.exists() {
            return Ok(plugins);
        }

        if !path.is_dir() {
            return Err(Error::plugin(format!("Path is not a directory: {path}", path = path.display())));
        }

        // Read directory entries
        let entries = std::fs::read_dir(path)
            .map_err(|e| Error::plugin(format!("Cannot read directory {path}: {e}", path = path.display())))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| Error::plugin(format!("Cannot read directory entry: {e}")))?;
            
            let entry_path = entry.path();
            
            // Look for plugin manifest files
            if entry_path.is_file() && entry_path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(file_name) = entry_path.file_stem().and_then(|s| s.to_str()) {
                    if file_name.ends_with(".plugin") {
                        match Self::load_plugin_manifest(&entry_path) {
                            Ok(info) => {
                                plugins.push(info);
                                log::debug!("Discovered external plugin: {} at {}", file_name, entry_path.display());
                            }
                            Err(e) => {
                                log::warn!("Failed to load plugin manifest {}: {}", entry_path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// Load plugin information from a manifest file
    fn load_plugin_manifest(manifest_path: &PathBuf) -> Result<PluginInfo> {
        let content = std::fs::read_to_string(manifest_path)
            .map_err(|e| Error::plugin(format!("Cannot read manifest file: {e}")))?;

        let info: PluginInfo = serde_json::from_str(&content)
            .map_err(|e| Error::plugin(format!("Invalid plugin manifest format: {e}")))?;

        Ok(info)
    }

    /// Validate all registered plugins
    fn validate_registered_plugins(
        &mut self,
        _available: &mut Vec<PluginInfo>,
        unavailable: &mut Vec<(String, String)>,
    ) {
        let plugin_names: Vec<String> = self.plugins.keys().cloned().collect();
        
        for name in plugin_names {
            if let Some(entry) = self.plugins.get_mut(&name) {
                match &entry.source {
                    PluginSource::Builtin => {
                        // Built-in plugins are always available
                        entry.available = true;
                    }
                    PluginSource::External => {
                        // Validate external plugin availability
                        let plugin_path = PathBuf::from(&entry.location);
                        if plugin_path.exists() {
                            entry.available = true;
                        } else {
                            entry.available = false;
                            unavailable.push((
                                name.clone(),
                                format!("Plugin file not found: {location}", location = entry.location),
                            ));
                        }
                    }
                    PluginSource::Remote => {
                        // For now, assume remote plugins are available
                        // In a full implementation, we would check remote availability
                        entry.available = true;
                    }
                }

                entry.last_checked = chrono::Utc::now();
            }
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        
        // Register built-in plugins
        registry.register_builtin(
            "analyzer",
            "1.0.0",
            "Built-in code analysis and dependency tracking plugin",
        );
        
        registry.register_builtin(
            "generator",
            "1.0.0",
            "Built-in code generation and templating plugin",
        );
        
        registry.register_builtin(
            "validator",
            "1.0.0",
            "Built-in validation and quality assurance plugin",
        );

        registry
    }
}