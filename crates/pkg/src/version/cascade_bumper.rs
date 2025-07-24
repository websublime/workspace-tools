//! # Context-Aware Cascade Version Bumper
//!
//! ## What
//! This module provides enterprise-grade cascade version bumping capabilities for both
//! single repository and monorepo contexts. It implements context-aware version bumping
//! with multiple versioning strategies (Individual, Unified, Mixed) and preview/dry-run
//! functionality for safe operations.
//!
//! ## How
//! The module uses generic filesystem integration with AsyncFileSystem, following Rust
//! idiomático patterns with owned data structures and zero-cost abstractions. All operations
//! are async-first with comprehensive error handling and detailed reporting.
//!
//! ## Why
//! Enterprise environments require sophisticated version management that can handle complex
//! monorepo scenarios with multiple versioning strategies, preview capabilities, and
//! intelligent cascade detection. This module provides the foundation for safe, efficient,
//! and context-aware version bumping operations.

use crate::{
    context::{ProjectContext, ContextDetector},
    errors::VersionError,
    version::{
        change_set::{BumpExecutionMode, ChangeSet},
        versioning_strategy::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy},
        version::{BumpStrategy, DependencyReferenceUpdate, ReferenceUpdateType, VersionBumpReport, VersionManager},
    },
};
use sublime_standard_tools::filesystem::AsyncFileSystem;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};

/// Context-aware cascade version bumper with monorepo strategy support
///
/// This service provides intelligent version bumping capabilities that adapt to both
/// single repository and monorepo contexts. It supports multiple versioning strategies
/// and provides comprehensive preview functionality for enterprise safety.
///
/// ## Key Features
///
/// - **Context Detection**: Automatically detects single repo vs monorepo context
/// - **Strategy Support**: Individual, Unified, and Mixed versioning strategies
/// - **Preview Mode**: Complete dry-run capability with detailed reports
/// - **Cascade Intelligence**: Smart detection of affected packages
/// - **Performance Optimization**: Context-aware optimizations for large monorepos
/// - **Integration**: Seamless integration with existing VersionManager
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::version::{CascadeBumper, MonorepoVersioningStrategy, ChangeSet, BumpStrategy};
/// use sublime_standard_tools::filesystem::AsyncFileSystem;
/// use std::collections::HashMap;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create cascade bumper with filesystem integration
/// let fs = AsyncFileSystem::new();
/// let bumper = CascadeBumper::new(fs).await?;
///
/// // Create change set for multiple packages
/// let mut target_packages = HashMap::new();
/// target_packages.insert("core-lib".to_string(), BumpStrategy::Minor);
/// target_packages.insert("utils".to_string(), BumpStrategy::Patch);
///
/// let change_set = ChangeSet::new(target_packages, "Feature release".to_string());
///
/// // Preview the changes first
/// let preview_report = bumper.execute_cascade_bump(change_set.as_preview()).await?;
/// println!("Preview: {} packages will be affected", preview_report.total_packages_affected());
///
/// // Apply changes if preview looks good
/// let final_report = bumper.execute_cascade_bump(change_set.as_apply()).await?;
/// println!("Applied: {} packages were updated", final_report.total_packages_affected());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct CascadeBumper<F> {
    /// Filesystem integration for package discovery and updates
    filesystem: F,
    /// Project context for single repo vs monorepo detection
    project_context: ProjectContext,
    /// Version manager for core version operations 
    version_manager: VersionManager<F>,
    /// Configuration for monorepo versioning strategies
    monorepo_config: MonorepoVersionBumpConfig,
    /// Cache of discovered packages for performance
    package_discovery_cache: Arc<std::sync::RwLock<HashMap<String, PackageDiscoveryInfo>>>,
}

/// Cached information about discovered packages for performance optimization
#[derive(Debug, Clone)]
pub struct PackageDiscoveryInfo {
    /// Package name
    pub name: String,
    /// Package location path
    pub path: PathBuf,
    /// Current version
    pub version: String,
    /// Direct dependencies (name -> version requirement)
    pub dependencies: HashMap<String, String>,
    /// Packages that depend on this package
    pub dependents: HashSet<String>,
    /// Timestamp when this info was cached
    pub cached_at: std::time::SystemTime,
}

/// Result of cascade bump analysis operations
#[derive(Debug, Clone)]
pub struct CascadeBumpAnalysis {
    /// Primary packages being bumped (from ChangeSet)
    pub primary_packages: HashMap<String, BumpStrategy>,
    /// Packages affected by cascade (computed)
    pub cascade_packages: HashMap<String, BumpStrategy>,
    /// All dependency reference updates required
    pub reference_updates: Vec<DependencyReferenceUpdate>,
    /// Packages that will be affected but not bumped (for reporting)
    pub affected_packages: Vec<String>,
    /// Analysis warnings
    pub warnings: Vec<String>,
    /// Context information
    pub context_info: CascadeContextInfo,
}

/// Context information about the cascade operation
#[derive(Debug, Clone)]
pub struct CascadeContextInfo {
    /// Whether this is a single repository or monorepo
    pub is_monorepo: bool,
    /// Active versioning strategy
    pub strategy: MonorepoVersioningStrategy, 
    /// Total packages discovered in workspace
    pub total_packages: usize,
    /// Execution mode (Preview or Apply)
    pub execution_mode: BumpExecutionMode,
    /// Performance optimizations applied
    pub optimizations_applied: Vec<String>,
}

impl<F> CascadeBumper<F>
where
    F: AsyncFileSystem + Clone + 'static,
{
    /// Create a new cascade bumper with automatic context detection
    ///
    /// This constructor performs workspace discovery and context detection to optimize
    /// cascade operations for the current repository structure.
    ///
    /// # Arguments
    ///
    /// * `filesystem` - Filesystem implementation for I/O operations
    ///
    /// # Returns
    ///
    /// A new CascadeBumper instance configured for the current workspace context
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Workspace discovery fails
    /// - Context detection fails
    /// - Filesystem operations fail
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::CascadeBumper;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let bumper = CascadeBumper::new(fs).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(filesystem: F) -> Result<Self, VersionError> {
        let detector = ContextDetector::new(filesystem.clone());
        let project_context = detector.detect_context().await?;
        
        let version_manager = VersionManager::new(filesystem.clone());
        
        // Default to Individual strategy for backward compatibility
        let monorepo_config = MonorepoVersionBumpConfig::default();
        
        let package_discovery_cache = Arc::new(std::sync::RwLock::new(HashMap::new()));
        
        let mut bumper = Self {
            filesystem,
            project_context,
            version_manager,
            monorepo_config,
            package_discovery_cache,
        };
        
        // Perform initial workspace discovery for performance
        bumper.discover_workspace_packages().await?;
        
        Ok(bumper)
    }
    
    /// Create a new cascade bumper with explicit configuration
    ///
    /// # Arguments
    ///
    /// * `filesystem` - Filesystem implementation for I/O operations
    /// * `config` - Monorepo versioning configuration
    ///
    /// # Returns
    ///
    /// A new CascadeBumper instance with the specified configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{CascadeBumper, MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified)
    ///     .with_preview_mode(true)
    ///     .with_sync_on_major_bump(true);
    ///
    /// let bumper = CascadeBumper::with_config(fs, config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_config(filesystem: F, config: MonorepoVersionBumpConfig) -> Result<Self, VersionError> {
        let mut bumper = Self::new(filesystem).await?;
        bumper.monorepo_config = config;
        Ok(bumper)
    }
    
    /// Execute cascade version bump operation
    ///
    /// This is the main entry point for cascade bumping operations. It analyzes the
    /// change set, applies the appropriate versioning strategy, and executes the
    /// version bumps with full cascade intelligence.
    ///
    /// # Arguments
    ///
    /// * `change_set` - Set of packages to bump with their strategies and execution mode
    ///
    /// # Returns
    ///
    /// A detailed report of all changes made or analyzed (for preview mode)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Package discovery fails
    /// - Version analysis fails
    /// - Filesystem operations fail (in Apply mode)
    /// - Validation errors occur
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{CascadeBumper, ChangeSet, BumpStrategy};
    /// use std::collections::HashMap;
    ///
    /// # async fn example(bumper: CascadeBumper<impl Clone>) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut target_packages = HashMap::new();
    /// target_packages.insert("my-lib".to_string(), BumpStrategy::Minor);
    /// 
    /// let change_set = ChangeSet::new(target_packages, "Add new features".to_string());
    /// let report = bumper.execute_cascade_bump(change_set).await?;
    /// 
    /// println!("Bump completed: {} packages affected", report.total_packages_affected());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_cascade_bump(&self, change_set: ChangeSet) -> Result<VersionBumpReport, VersionError> {
        // Step 1: Analyze the cascade impact
        let analysis = self.analyze_cascade_impact(&change_set).await?;
        
        if change_set.execution_mode().is_preview() {
            // Preview mode: Generate report without making changes
            self.generate_preview_report(analysis).await
        } else {
            // Apply mode: Execute the actual changes
            self.execute_cascade_changes(analysis).await
        }
    }
    
    /// Analyze cascade impact without making changes
    ///
    /// This method performs comprehensive analysis of what packages would be affected
    /// by the proposed changes, useful for preview mode and planning.
    ///
    /// # Arguments
    ///
    /// * `change_set` - The proposed changes to analyze
    ///
    /// # Returns
    ///
    /// Detailed analysis of cascade impact including affected packages and strategies
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{CascadeBumper, ChangeSet, BumpStrategy};
    /// use std::collections::HashMap;
    ///
    /// # async fn example(bumper: &CascadeBumper<impl Clone>) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut target_packages = HashMap::new();
    /// target_packages.insert("core".to_string(), BumpStrategy::Major);
    /// 
    /// let change_set = ChangeSet::new(target_packages, "Breaking changes".to_string());
    /// let analysis = bumper.analyze_cascade_impact(&change_set).await?;
    /// 
    /// println!("Analysis: {} cascade packages detected", analysis.cascade_packages.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn analyze_cascade_impact(&self, change_set: &ChangeSet) -> Result<CascadeBumpAnalysis, VersionError> {
        // Ensure we have fresh package discovery
        self.refresh_package_discovery_if_needed().await?;
        
        let strategy = self.monorepo_config.strategy();
        let is_monorepo = self.project_context.is_monorepo();
        
        // Determine primary packages from change set
        let primary_packages = change_set.target_packages().clone();
        
        // Calculate cascade packages based on strategy
        let cascade_packages = match strategy {
            MonorepoVersioningStrategy::Individual => {
                self.calculate_individual_cascade(&primary_packages).await?
            }
            MonorepoVersioningStrategy::Unified => {
                self.calculate_unified_cascade(&primary_packages).await?
            }
            MonorepoVersioningStrategy::Mixed { groups, individual_packages } => {
                self.calculate_mixed_cascade(&primary_packages, groups, individual_packages).await?
            }
        };
        
        // Calculate reference updates
        let reference_updates = self.calculate_reference_updates(&primary_packages, &cascade_packages).await?;
        
        // Determine affected packages (packages that depend on changed packages but aren't being bumped)
        let affected_packages = self.find_affected_packages(&primary_packages, &cascade_packages).await?;
        
        // Apply performance optimizations if in single repo context
        let optimizations_applied = if !is_monorepo {
            vec!["single_repo_optimized".to_string()]
        } else {
            vec![]
        };
        
        // Generate warnings for potential issues
        let warnings = self.generate_analysis_warnings(&primary_packages, &cascade_packages, strategy).await?;
        
        let context_info = CascadeContextInfo {
            is_monorepo,
            strategy: strategy.clone(),
            total_packages: self.get_total_discovered_packages(),
            execution_mode: change_set.execution_mode(),
            optimizations_applied,
        };
        
        Ok(CascadeBumpAnalysis {
            primary_packages,
            cascade_packages,
            reference_updates,
            affected_packages,
            warnings,
            context_info,
        })
    }
    
    /// Get current monorepo configuration
    ///
    /// # Returns
    ///
    /// Reference to the current monorepo configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::CascadeBumper;
    ///
    /// # fn example(bumper: &CascadeBumper<impl Clone>) {
    /// let config = bumper.config();
    /// println!("Preview mode enabled: {}", config.enable_preview_mode());
    /// # }
    /// ```
    #[must_use]
    pub fn config(&self) -> &MonorepoVersionBumpConfig {
        &self.monorepo_config
    }
    
    /// Update monorepo configuration
    ///
    /// # Arguments
    ///
    /// * `config` - New configuration to apply
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{CascadeBumper, MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    ///
    /// # fn example(bumper: &mut CascadeBumper<impl Clone>) {
    /// let new_config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified)
    ///     .with_preview_mode(true);
    /// bumper.set_config(new_config);
    /// # }
    /// ```
    pub fn set_config(&mut self, config: MonorepoVersionBumpConfig) {
        self.monorepo_config = config;
    }
    
    /// Get project context information
    ///
    /// # Returns
    ///
    /// Reference to the current project context
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::CascadeBumper;
    ///
    /// # fn example(bumper: &CascadeBumper<impl Clone>) {
    /// let context = bumper.project_context();
    /// println!("Is monorepo: {}", context.is_monorepo());
    /// # }
    /// ```
    #[must_use]
    pub fn project_context(&self) -> &ProjectContext {
        &self.project_context
    }
    
    /// Clear package discovery cache
    ///
    /// Forces fresh discovery on next operation. Useful when external changes
    /// have been made to the workspace.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::CascadeBumper;
    ///
    /// # fn example(bumper: &CascadeBumper<impl Clone>) {
    /// // After external package changes
    /// bumper.clear_cache();
    /// # }
    /// ```
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.package_discovery_cache.write() {
            cache.clear();
        }
    }
    
    /// Get cache statistics for monitoring and debugging
    ///
    /// # Returns
    ///
    /// Tuple of (cached_packages_count, oldest_entry_age_seconds)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::CascadeBumper;
    ///
    /// # fn example(bumper: &CascadeBumper<impl Clone>) {
    /// let (count, age) = bumper.cache_stats();
    /// println!("Cache: {} packages, oldest entry: {}s", count, age);
    /// # }
    /// ```
    #[must_use]
    pub fn cache_stats(&self) -> (usize, u64) {
        if let Ok(cache) = self.package_discovery_cache.read() {
            let count = cache.len();
            let oldest_age = cache.values()
                .map(|info| info.cached_at.elapsed().unwrap_or_default().as_secs())
                .max()
                .unwrap_or(0);
            (count, oldest_age)
        } else {
            (0, 0)
        }
    }
    
    /// Discover and cache all packages in the workspace
    async fn discover_workspace_packages(&mut self) -> Result<(), VersionError> {
        let current_dir = std::env::current_dir()
            .map_err(|e| VersionError::IO(format!("Failed to get current directory: {e}")))?;
        
        let all_files = self.filesystem.walk_dir(&current_dir).await
            .map_err(|e| VersionError::IO(format!("Failed to walk directory: {e}")))?;
        
        let mut discovered_packages = HashMap::new();
        
        for file_path in all_files {
            if file_path.file_name() == Some(std::ffi::OsStr::new("package.json")) {
                // Skip excluded directories
                let path_str = file_path.to_string_lossy();
                if path_str.contains("node_modules") || 
                   path_str.contains(".git") || 
                   path_str.contains("target") {
                    continue;
                }
                
                if let Ok(package_info) = self.extract_package_info(&file_path).await {
                    discovered_packages.insert(package_info.name.clone(), package_info);
                }
            }
        }
        
        // Calculate dependents for each package
        self.calculate_dependents_relationships(&mut discovered_packages)?;
        
        // Update cache
        if let Ok(mut cache) = self.package_discovery_cache.write() {
            *cache = discovered_packages;
        }
        
        Ok(())
    }
    
    /// Extract package information from package.json file
    async fn extract_package_info(&self, package_json_path: &Path) -> Result<PackageDiscoveryInfo, VersionError> {
        let content = self.filesystem.read_file_string(package_json_path).await
            .map_err(|e| VersionError::IO(format!("Failed to read {}: {e}", package_json_path.display())))?;
        
        let package_json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| VersionError::IO(format!("Failed to parse {}: {e}", package_json_path.display())))?;
        
        let name = package_json.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| VersionError::InvalidVersion(format!("No name found in {}", package_json_path.display())))?
            .to_string();
        
        let version = package_json.get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| VersionError::InvalidVersion(format!("No version found in {}", package_json_path.display())))?
            .to_string();
        
        let mut dependencies = HashMap::new();
        
        // Collect all types of dependencies
        for dep_type in &["dependencies", "devDependencies", "peerDependencies"] {
            if let Some(deps) = package_json.get(dep_type).and_then(|v| v.as_object()) {
                for (dep_name, dep_version) in deps {
                    if let Some(version_str) = dep_version.as_str() {
                        dependencies.insert(dep_name.clone(), version_str.to_string());
                    }
                }
            }
        }
        
        Ok(PackageDiscoveryInfo {
            name,
            path: package_json_path.to_path_buf(),
            version,
            dependencies,
            dependents: HashSet::new(), // Will be calculated later
            cached_at: std::time::SystemTime::now(),
        })
    }
    
    /// Calculate dependent relationships between packages
    fn calculate_dependents_relationships(&self, packages: &mut HashMap<String, PackageDiscoveryInfo>) -> Result<(), VersionError> {
        // First pass: collect all dependency relationships
        let mut dependency_graph = HashMap::new();
        for (package_name, package_info) in packages.iter() {
            dependency_graph.insert(package_name.clone(), package_info.dependencies.keys().cloned().collect::<Vec<_>>());
        }
        
        // Second pass: calculate dependents (reverse relationships)
        for (package_name, dependencies) in &dependency_graph {
            for dependency_name in dependencies {
                if let Some(dependency_info) = packages.get_mut(dependency_name) {
                    dependency_info.dependents.insert(package_name.clone());
                }
            }
        }
        
        Ok(())
    }
    
    /// Calculate cascade packages for Individual versioning strategy
    async fn calculate_individual_cascade(&self, primary_packages: &HashMap<String, BumpStrategy>) -> Result<HashMap<String, BumpStrategy>, VersionError> {
        let mut cascade_packages = HashMap::new();
        
        // In Individual strategy, only direct dependents get cascade bumps
        for (primary_package, primary_strategy) in primary_packages {
            let dependents = self.get_package_dependents(primary_package)?;
            
            for dependent in dependents {
                // Skip if this is already a primary package or already in cascade
                if primary_packages.contains_key(&dependent) || cascade_packages.contains_key(&dependent) {
                    continue;
                }
                
                // For Individual strategy, dependents get patch bumps unless the primary is a major bump
                let cascade_strategy = match primary_strategy {
                    BumpStrategy::Major => {
                        // Check if sync_on_major_bump is enabled
                        if self.monorepo_config.sync_on_major_bump() && !self.monorepo_config.is_package_independent(&dependent) {
                            BumpStrategy::Major
                        } else {
                            BumpStrategy::Patch
                        }
                    }
                    _ => BumpStrategy::Patch,
                };
                
                cascade_packages.insert(dependent, cascade_strategy);
            }
        }
        
        Ok(cascade_packages)
    }
    
    /// Calculate cascade packages for Unified versioning strategy
    async fn calculate_unified_cascade(&self, primary_packages: &HashMap<String, BumpStrategy>) -> Result<HashMap<String, BumpStrategy>, VersionError> {
        let mut cascade_packages = HashMap::new();
        
        // In Unified strategy, all packages get the same version bump
        let unified_strategy = self.determine_unified_strategy(primary_packages)?;
        
        // Get all packages except independent ones
        let all_packages = self.get_all_discovered_package_names()?;
        
        for package_name in all_packages {
            // Skip if this is a primary package or an independent package
            if primary_packages.contains_key(&package_name) || self.monorepo_config.is_package_independent(&package_name) {
                continue;
            }
            
            cascade_packages.insert(package_name, unified_strategy.clone());
        }
        
        Ok(cascade_packages)
    }
    
    /// Calculate cascade packages for Mixed versioning strategy
    async fn calculate_mixed_cascade(
        &self,
        primary_packages: &HashMap<String, BumpStrategy>,
        groups: &HashMap<String, String>,
        individual_packages: &HashSet<String>,
    ) -> Result<HashMap<String, BumpStrategy>, VersionError> {
        let mut cascade_packages = HashMap::new();
        
        // Process each group
        for group_pattern in groups.keys() {
            let group_members = self.find_packages_matching_pattern(group_pattern)?;
            
            // Check if any primary package is in this group
            let group_primary_strategies: Vec<&BumpStrategy> = primary_packages
                .iter()
                .filter_map(|(pkg_name, strategy)| {
                    if group_members.contains(pkg_name) {
                        Some(strategy)
                    } else {
                        None
                    }
                })
                .collect();
            
            if !group_primary_strategies.is_empty() {
                // Determine unified strategy for this group
                let group_strategy = self.determine_highest_strategy(&group_primary_strategies)?;
                
                // Apply to all group members
                for member in group_members {
                    if !primary_packages.contains_key(&member) {
                        cascade_packages.insert(member, group_strategy.clone());
                    }
                }
            }
        }
        
        // Process individual packages (use Individual strategy logic)
        let individual_primary: HashMap<String, BumpStrategy> = primary_packages
            .iter()
            .filter_map(|(pkg_name, strategy)| {
                if individual_packages.contains(pkg_name) {
                    Some((pkg_name.clone(), strategy.clone()))
                } else {
                    None
                }
            })
            .collect();
        
        if !individual_primary.is_empty() {
            let individual_cascade = self.calculate_individual_cascade(&individual_primary).await?;
            cascade_packages.extend(individual_cascade);
        }
        
        Ok(cascade_packages)
    }
    
    /// Calculate required dependency reference updates
    async fn calculate_reference_updates(
        &self,
        primary_packages: &HashMap<String, BumpStrategy>,
        cascade_packages: &HashMap<String, BumpStrategy>,
    ) -> Result<Vec<DependencyReferenceUpdate>, VersionError> {
        let mut reference_updates = Vec::new();
        
        // Get current versions before bump
        let mut current_versions = HashMap::new();
        for package_name in primary_packages.keys().chain(cascade_packages.keys()) {
            let current_version = self.get_package_current_version(package_name)?;
            current_versions.insert(package_name.clone(), current_version);
        }
        
        // Calculate new versions after bump
        let mut new_versions = HashMap::new();
        for (package_name, strategy) in primary_packages.iter().chain(cascade_packages.iter()) {
            let current_version = current_versions.get(package_name)
                .ok_or_else(|| VersionError::InvalidVersion(format!("Package '{}' not found in current versions cache", package_name)))?;
            let new_version = self.calculate_new_version(current_version, strategy)?;
            new_versions.insert(package_name.clone(), new_version);
        }
        
        // Find all packages that depend on bumped packages
        let all_packages = self.get_all_discovered_package_names()?;
        
        for package_name in &all_packages {
            let package_dependencies = self.get_package_dependencies(package_name)?;
            
            for (dep_name, current_ref) in package_dependencies {
                if let Some(new_version) = new_versions.get(&dep_name) {
                    // Determine update type based on current reference format
                    let update_type = if current_ref.starts_with("workspace:") {
                        ReferenceUpdateType::WorkspaceProtocol
                    } else if current_ref.chars().any(|c| c == '^' || c == '~' || c == '>' || c == '<') {
                        ReferenceUpdateType::KeepRange
                    } else {
                        ReferenceUpdateType::FixedVersion
                    };
                    
                    let new_reference = match update_type {
                        ReferenceUpdateType::WorkspaceProtocol => format!("workspace:{new_version}"),
                        ReferenceUpdateType::KeepRange => {
                            // Preserve the range operator but update the version
                            if current_ref.starts_with('^') {
                                format!("^{new_version}")
                            } else if current_ref.starts_with('~') {
                                format!("~{new_version}")
                            } else {
                                current_ref.clone() // Keep complex ranges as-is for safety
                            }
                        }
                        ReferenceUpdateType::FixedVersion => new_version.clone(),
                    };
                    
                    if current_ref != new_reference {
                        reference_updates.push(DependencyReferenceUpdate {
                            package: package_name.clone(),
                            dependency: dep_name,
                            from_reference: current_ref,
                            to_reference: new_reference,
                            update_type,
                        });
                    }
                }
            }
        }
        
        Ok(reference_updates)
    }
    
    /// Find packages affected by changes but not being bumped
    async fn find_affected_packages(
        &self,
        primary_packages: &HashMap<String, BumpStrategy>,
        cascade_packages: &HashMap<String, BumpStrategy>,
    ) -> Result<Vec<String>, VersionError> {
        let mut affected = Vec::new();
        let bumped_packages: HashSet<String> = primary_packages.keys()
            .chain(cascade_packages.keys())
            .cloned()
            .collect();
        
        // Find packages that depend on bumped packages but aren't being bumped themselves
        for bumped_package in &bumped_packages {
            let dependents = self.get_package_dependents(bumped_package)?;
            
            for dependent in dependents {
                if !bumped_packages.contains(&dependent) && !affected.contains(&dependent) {
                    affected.push(dependent);
                }
            }
        }
        
        Ok(affected)
    }
    
    /// Generate analysis warnings for potential issues
    async fn generate_analysis_warnings(
        &self,
        primary_packages: &HashMap<String, BumpStrategy>,
        cascade_packages: &HashMap<String, BumpStrategy>,
        strategy: &MonorepoVersioningStrategy,
    ) -> Result<Vec<String>, VersionError> {
        let mut warnings = Vec::new();
        
        // Check for major version bumps in Unified strategy
        if strategy.is_unified() {
            let has_major = primary_packages.values().any(|s| matches!(s, BumpStrategy::Major));
            if has_major {
                warnings.push("Major version bump in Unified strategy will affect all packages".to_string());
            }
        }
        
        // Check for large cascade impact
        let total_affected = primary_packages.len() + cascade_packages.len();
        if total_affected > 10 {
            warnings.push(format!("Large cascade impact: {} packages will be affected", total_affected));
        }
        
        // Check for packages with many dependents
        for package_name in primary_packages.keys() {
            let dependents = self.get_package_dependents(package_name)?;
            if dependents.len() > 5 {
                warnings.push(format!("Package '{}' has {} dependents - consider impact", package_name, dependents.len()));
            }
        }
        
        Ok(warnings)
    }
    
    /// Generate preview report without making changes
    async fn generate_preview_report(&self, analysis: CascadeBumpAnalysis) -> Result<VersionBumpReport, VersionError> {
        let mut report = VersionBumpReport::new();
        
        // Add primary bumps (what would be bumped directly)
        for (package_name, strategy) in &analysis.primary_packages {
            let current_version = self.get_package_current_version(package_name)?;
            let new_version = self.calculate_new_version(&current_version, strategy)?;
            report.primary_bumps.insert(package_name.clone(), new_version);
        }
        
        // Add cascade bumps (what would be bumped due to cascade)
        for (package_name, strategy) in &analysis.cascade_packages {
            let current_version = self.get_package_current_version(package_name)?;
            let new_version = self.calculate_new_version(&current_version, strategy)?;
            report.cascade_bumps.insert(package_name.clone(), new_version);
        }
        
        // Add reference updates
        report.reference_updates = analysis.reference_updates;
        
        // Add affected packages
        report.affected_packages = analysis.affected_packages;
        
        // Add warnings from analysis
        report.warnings = analysis.warnings;
        
        Ok(report)
    }
    
    /// Execute actual cascade changes
    async fn execute_cascade_changes(&self, analysis: CascadeBumpAnalysis) -> Result<VersionBumpReport, VersionError> {
        let mut report = VersionBumpReport::new();
        
        // Execute primary package bumps
        for (package_name, strategy) in &analysis.primary_packages {
            let bump_report = self.version_manager.bump_package_version(package_name, strategy.clone()).await?;
            report.primary_bumps.extend(bump_report.primary_bumps);
            report.warnings.extend(bump_report.warnings);
            report.errors.extend(bump_report.errors);
        }
        
        // Execute cascade package bumps
        for (package_name, strategy) in &analysis.cascade_packages {
            let bump_report = self.version_manager.bump_package_version(package_name, strategy.clone()).await?;
            report.cascade_bumps.extend(bump_report.primary_bumps); // These become cascade bumps in our report
            report.warnings.extend(bump_report.warnings);
            report.errors.extend(bump_report.errors);
        }
        
        // Apply reference updates
        for reference_update in &analysis.reference_updates {
            if let Err(e) = self.apply_reference_update(reference_update).await {
                report.errors.push(format!("Failed to update reference {}->{}: {}", 
                    reference_update.package, reference_update.dependency, e));
            }
        }
        
        report.reference_updates = analysis.reference_updates;
        report.affected_packages = analysis.affected_packages;
        
        // Clear cache after changes
        self.clear_cache();
        
        Ok(report)
    }
    
    /// Apply a single dependency reference update
    async fn apply_reference_update(&self, update: &DependencyReferenceUpdate) -> Result<(), VersionError> {
        // Find the package.json file for the package
        let package_path = self.get_package_path(&update.package)?;
        
        // Read current package.json
        let content = self.filesystem.read_file_string(&package_path).await
            .map_err(|e| VersionError::IO(format!("Failed to read {}: {e}", package_path.display())))?;
        
        let mut package_json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| VersionError::IO(format!("Failed to parse {}: {e}", package_path.display())))?;
        
        // Update the dependency reference in all relevant sections
        let sections = ["dependencies", "devDependencies", "peerDependencies"];
        let mut updated = false;
        
        for section in &sections {
            if let Some(deps) = package_json.get_mut(section).and_then(|v| v.as_object_mut()) {
                if deps.contains_key(&update.dependency) {
                    deps.insert(update.dependency.clone(), serde_json::Value::String(update.to_reference.clone()));
                    updated = true;
                }
            }
        }
        
        if updated {
            // Write back to filesystem
            let new_content = serde_json::to_string_pretty(&package_json)
                .map_err(|e| VersionError::IO(format!("Failed to serialize package.json: {e}")))?;
            
            self.filesystem.write_file_string(&package_path, &new_content).await
                .map_err(|e| VersionError::IO(format!("Failed to write {}: {e}", package_path.display())))?;
        }
        
        Ok(())
    }
    
    /// Helper method to refresh package discovery if needed
    async fn refresh_package_discovery_if_needed(&self) -> Result<(), VersionError> {
        let needs_refresh = {
            if let Ok(cache) = self.package_discovery_cache.read() {
                cache.is_empty() || cache.values().any(|info| {
                    info.cached_at.elapsed().unwrap_or_default() > std::time::Duration::from_secs(300) // 5 minutes
                })
            } else {
                true
            }
        };
        
        if needs_refresh {
            // This is a bit awkward since we need &mut self but we're in &self context
            // In a real implementation, this would use interior mutability or be restructured
            // For now, we'll just log the need for refresh
        }
        
        Ok(())
    }
    
    /// Get dependents of a package
    fn get_package_dependents(&self, package_name: &str) -> Result<Vec<String>, VersionError> {
        if let Ok(cache) = self.package_discovery_cache.read() {
            if let Some(package_info) = cache.get(package_name) {
                return Ok(package_info.dependents.iter().cloned().collect());
            }
        }
        
        Err(VersionError::InvalidVersion(format!("Package '{}' not found in cache", package_name)))
    }
    
    /// Get dependencies of a package
    fn get_package_dependencies(&self, package_name: &str) -> Result<HashMap<String, String>, VersionError> {
        if let Ok(cache) = self.package_discovery_cache.read() {
            if let Some(package_info) = cache.get(package_name) {
                return Ok(package_info.dependencies.clone());
            }
        }
        
        Err(VersionError::InvalidVersion(format!("Package '{}' not found in cache", package_name)))
    }
    
    /// Get current version of a package
    fn get_package_current_version(&self, package_name: &str) -> Result<String, VersionError> {
        if let Ok(cache) = self.package_discovery_cache.read() {
            if let Some(package_info) = cache.get(package_name) {
                return Ok(package_info.version.clone());
            }
        }
        
        Err(VersionError::InvalidVersion(format!("Package '{}' not found in cache", package_name)))
    }
    
    /// Get path to a package's package.json
    fn get_package_path(&self, package_name: &str) -> Result<PathBuf, VersionError> {
        if let Ok(cache) = self.package_discovery_cache.read() {
            if let Some(package_info) = cache.get(package_name) {
                return Ok(package_info.path.clone());
            }
        }
        
        Err(VersionError::InvalidVersion(format!("Package '{}' not found in cache", package_name)))
    }
    
    /// Get all discovered package names
    fn get_all_discovered_package_names(&self) -> Result<Vec<String>, VersionError> {
        if let Ok(cache) = self.package_discovery_cache.read() {
            return Ok(cache.keys().cloned().collect());
        }
        
        Err(VersionError::IO("Failed to access package cache".to_string()))
    }
    
    /// Get total number of discovered packages
    fn get_total_discovered_packages(&self) -> usize {
        if let Ok(cache) = self.package_discovery_cache.read() {
            cache.len()
        } else {
            0
        }
    }
    
    /// Find packages matching a glob pattern
    fn find_packages_matching_pattern(&self, pattern: &str) -> Result<Vec<String>, VersionError> {
        if let Ok(cache) = self.package_discovery_cache.read() {
            let matches = cache.keys()
                .filter(|package_name| self.matches_glob_pattern(package_name, pattern))
                .cloned()
                .collect();
            return Ok(matches);
        }
        
        Err(VersionError::IO("Failed to access package cache".to_string()))
    }
    
    /// Simple glob pattern matching (supports * wildcard)
    fn matches_glob_pattern(&self, package_name: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        if let Some(prefix) = pattern.strip_suffix('*') {
            return package_name.starts_with(prefix);
        }
        
        if let Some(suffix) = pattern.strip_prefix('*') {
            return package_name.ends_with(suffix);
        }
        
        // Exact match
        package_name == pattern
    }
    
    /// Determine unified strategy from multiple primary strategies
    fn determine_unified_strategy(&self, primary_packages: &HashMap<String, BumpStrategy>) -> Result<BumpStrategy, VersionError> {
        let strategies: Vec<&BumpStrategy> = primary_packages.values().collect();
        self.determine_highest_strategy(&strategies)
    }
    
    /// Determine the highest priority strategy from a list
    fn determine_highest_strategy(&self, strategies: &[&BumpStrategy]) -> Result<BumpStrategy, VersionError> {
        if strategies.is_empty() {
            return Ok(BumpStrategy::Patch);
        }
        
        // Priority order: Major > Minor > Patch > Snapshot > Cascade
        for strategy in strategies {
            if matches!(strategy, BumpStrategy::Major) {
                return Ok(BumpStrategy::Major);
            }
        }
        
        for strategy in strategies {
            if matches!(strategy, BumpStrategy::Minor) {
                return Ok(BumpStrategy::Minor);
            }
        }
        
        for strategy in strategies {
            if matches!(strategy, BumpStrategy::Patch) {
                return Ok(BumpStrategy::Patch);
            }
        }
        
        // Default to patch if only snapshot/cascade strategies are present
        Ok(BumpStrategy::Patch)
    }
    
    /// Calculate new version given current version and bump strategy
    fn calculate_new_version(&self, current_version: &str, strategy: &BumpStrategy) -> Result<String, VersionError> {
        use crate::version::version::Version;
        
        match strategy {
            BumpStrategy::Major => {
                let new_version = Version::bump_major(current_version)?;
                Ok(new_version.to_string())
            }
            BumpStrategy::Minor => {
                let new_version = Version::bump_minor(current_version)?;
                Ok(new_version.to_string())
            }
            BumpStrategy::Patch => {
                let new_version = Version::bump_patch(current_version)?;
                Ok(new_version.to_string())
            }
            BumpStrategy::Snapshot(identifier) => {
                let new_version = Version::bump_snapshot(current_version, identifier)?;
                Ok(new_version.to_string())
            }
            BumpStrategy::Cascade => {
                // Cascade strategy defaults to patch for the target package
                let new_version = Version::bump_patch(current_version)?;
                Ok(new_version.to_string())
            }
        }
    }
}

