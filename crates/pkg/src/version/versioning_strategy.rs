//! # Monorepo Versioning Strategy Configuration
//!
//! ## What
//! This module provides configuration structures for managing different versioning
//! strategies in monorepo contexts. It supports Individual (each package independent),
//! Unified (all packages share version), and Mixed (groups of packages) strategies
//! with enterprise-grade type safety and performance.
//!
//! ## How
//! The module uses concrete Rust enums and structs with owned data, avoiding
//! Java-like abstractions. All configurations are designed for async contexts
//! with efficient serialization support for persistence.
//!
//! ## Why
//! Different monorepos require different versioning approaches. Some need
//! independent package versions, others require synchronized versions across
//! all packages, and complex monorepos need mixed strategies. This module
//! provides the configuration foundation for enterprise versioning workflows.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Versioning strategy for monorepo package management
///
/// Defines how package versions are managed within a monorepo context.
/// Each strategy optimizes for different use cases and organizational needs.
///
/// # Strategy Details
///
/// - **Individual**: Each package maintains independent semantic versioning
/// - **Unified**: All packages share the same version, bumped together
/// - **Mixed**: Combination approach with groups of unified packages and individual packages
///
/// # Examples
///
/// ```rust
/// use sublime_package_tools::version::MonorepoVersioningStrategy;
/// use std::collections::HashMap;
///
/// // Individual versioning - each package independent
/// let individual = MonorepoVersioningStrategy::Individual;
///
/// // Unified versioning - all packages same version
/// let unified = MonorepoVersioningStrategy::Unified;
///
/// // Mixed versioning with groups
/// let mut groups = HashMap::new();
/// groups.insert("core-*".to_string(), "1.0.0".to_string());
/// groups.insert("utils-*".to_string(), "2.1.0".to_string());
///
/// let mut individual_packages = std::collections::HashSet::new();
/// individual_packages.insert("example-app".to_string());
///
/// let mixed = MonorepoVersioningStrategy::Mixed {
///     groups,
///     individual_packages,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonorepoVersioningStrategy {
    /// Each package maintains its own independent version
    ///
    /// This strategy allows maximum flexibility where each package can have
    /// its own semantic version independent of others. Suitable for:
    /// - Libraries with different release cycles
    /// - Packages with varying stability levels
    /// - Legacy monorepos migrating to unified versioning
    ///
    /// Example: package-a@1.2.0, package-b@2.1.5, package-c@0.3.0
    Individual,
    
    /// All packages share the same version, bumped together
    ///
    /// This strategy ensures version consistency across the entire monorepo.
    /// When any package changes, all packages get the same version bump.
    /// Suitable for:
    /// - Tightly coupled packages that should stay in sync
    /// - Simplified release management
    /// - Consistent user experience across packages
    ///
    /// Example: package-a@1.0.0, package-b@1.0.0, package-c@1.0.0
    Unified,
    
    /// Mixed strategy with groups of unified packages and individual packages
    ///
    /// This advanced strategy allows grouping related packages under unified
    /// versioning while keeping others individual. Groups use glob patterns
    /// to match package names.
    ///
    /// # Fields
    ///
    /// * `groups` - Map of glob patterns to shared versions
    /// * `individual_packages` - Set of packages that remain individual
    ///
    /// Example: 
    /// - core-* packages → 1.0.0 (unified group)
    /// - utils-* packages → 2.1.0 (unified group)  
    /// - example-app → individual versioning
    Mixed {
        /// Groups of packages that share versions
        ///
        /// Key: Glob pattern to match package names (e.g., "core-*", "utils-*")
        /// Value: Current shared version for this group
        groups: HashMap<String, String>,
        
        /// Packages that maintain individual versioning
        ///
        /// These packages are excluded from any group and maintain
        /// independent semantic versioning regardless of group changes.
        individual_packages: HashSet<String>,
    },
}

impl Default for MonorepoVersioningStrategy {
    /// Default strategy is Individual for backward compatibility
    ///
    /// Individual versioning is the safest default as it maintains
    /// existing package independence while allowing opt-in to more
    /// advanced strategies.
    fn default() -> Self {
        Self::Individual
    }
}

impl MonorepoVersioningStrategy {
    /// Check if this strategy requires group management
    ///
    /// # Returns
    ///
    /// `true` if the strategy uses groups (Mixed), `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::MonorepoVersioningStrategy;
    /// use std::collections::HashMap;
    ///
    /// assert!(!MonorepoVersioningStrategy::Individual.has_groups());
    /// assert!(!MonorepoVersioningStrategy::Unified.has_groups());
    ///
    /// let mixed = MonorepoVersioningStrategy::Mixed {
    ///     groups: HashMap::new(),
    ///     individual_packages: std::collections::HashSet::new(),
    /// };
    /// assert!(mixed.has_groups());
    /// ```
    #[must_use]
    pub const fn has_groups(&self) -> bool {
        matches!(self, Self::Mixed { .. })
    }
    
    /// Check if this strategy synchronizes all packages
    ///
    /// # Returns
    ///
    /// `true` if all packages share versions (Unified), `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::MonorepoVersioningStrategy;
    /// use std::collections::HashMap;
    ///
    /// assert!(!MonorepoVersioningStrategy::Individual.is_unified());
    /// assert!(MonorepoVersioningStrategy::Unified.is_unified());
    ///
    /// let mixed = MonorepoVersioningStrategy::Mixed {
    ///     groups: HashMap::new(),
    ///     individual_packages: std::collections::HashSet::new(),
    /// };
    /// assert!(!mixed.is_unified());
    /// ```
    #[must_use]
    pub const fn is_unified(&self) -> bool {
        matches!(self, Self::Unified)
    }
    
    /// Check if this strategy allows individual package versioning
    ///
    /// # Returns
    ///
    /// `true` if some packages can have individual versions, `false` if all are synchronized
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::MonorepoVersioningStrategy;
    /// use std::collections::HashMap;
    ///
    /// assert!(MonorepoVersioningStrategy::Individual.allows_individual());
    /// assert!(!MonorepoVersioningStrategy::Unified.allows_individual());
    ///
    /// let mixed = MonorepoVersioningStrategy::Mixed {
    ///     groups: HashMap::new(),
    ///     individual_packages: std::collections::HashSet::new(),
    /// };
    /// assert!(mixed.allows_individual());
    /// ```
    #[must_use]
    pub const fn allows_individual(&self) -> bool {
        !matches!(self, Self::Unified)
    }

    /// Get the groups map for Mixed strategy
    ///
    /// # Returns
    ///
    /// Reference to groups HashMap if Mixed strategy, None otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::MonorepoVersioningStrategy;
    /// use std::collections::HashMap;
    ///
    /// assert!(MonorepoVersioningStrategy::Individual.groups().is_none());
    /// assert!(MonorepoVersioningStrategy::Unified.groups().is_none());
    ///
    /// let mixed = MonorepoVersioningStrategy::Mixed {
    ///     groups: HashMap::new(),
    ///     individual_packages: std::collections::HashSet::new(),
    /// };
    /// assert!(mixed.groups().is_some());
    /// ```
    #[must_use]
    pub fn groups(&self) -> Option<&HashMap<String, String>> {
        match self {
            Self::Mixed { groups, .. } => Some(groups),
            _ => None,
        }
    }

    /// Get the individual packages set for Mixed strategy
    ///
    /// # Returns
    ///
    /// Reference to individual packages HashSet if Mixed strategy, None otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::MonorepoVersioningStrategy;
    /// use std::collections::HashMap;
    ///
    /// assert!(MonorepoVersioningStrategy::Individual.individual_packages().is_none());
    /// assert!(MonorepoVersioningStrategy::Unified.individual_packages().is_none());
    ///
    /// let mixed = MonorepoVersioningStrategy::Mixed {
    ///     groups: HashMap::new(),
    ///     individual_packages: std::collections::HashSet::new(),
    /// };
    /// assert!(mixed.individual_packages().is_some());
    /// ```
    #[must_use]
    pub fn individual_packages(&self) -> Option<&HashSet<String>> {
        match self {
            Self::Mixed { individual_packages, .. } => Some(individual_packages),
            _ => None,
        }
    }
}

/// Configuration for monorepo version bumping operations
///
/// Provides comprehensive configuration for how version bumping should behave
/// in monorepo contexts, including strategy selection, synchronization rules,
/// and preview functionality.
///
/// # Examples
///
/// ```rust
/// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
/// use std::collections::HashSet;
///
/// // Basic individual versioning configuration  
/// let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Individual);
/// assert!(!config.sync_on_major_bump());
/// assert!(!config.enable_preview_mode());
///
/// // Advanced unified configuration with preview
/// let mut advanced = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified)
///     .with_sync_on_major_bump(true)
///     .with_preview_mode(true)
///     .with_snapshot_template("alpha-{sha}".to_string());
///
/// let mut independent = HashSet::new();
/// independent.insert("legacy-package".to_string());
/// advanced = advanced.with_independent_packages(independent);
///
/// assert!(advanced.sync_on_major_bump());
/// assert!(advanced.enable_preview_mode());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonorepoVersionBumpConfig {
    /// Primary versioning strategy for the monorepo
    strategy: MonorepoVersioningStrategy,
    
    /// Force unified versioning on major bumps regardless of strategy
    ///
    /// When enabled, major version bumps will synchronize all packages
    /// to the same version even in Individual or Mixed strategies.
    /// This prevents major version fragmentation across the monorepo.
    sync_on_major_bump: bool,
    
    /// Packages that never participate in unified versioning
    ///
    /// These packages maintain individual versioning even when
    /// sync_on_major_bump is enabled or in Unified strategy.
    /// Useful for legacy packages or external dependencies.
    independent_packages: HashSet<String>,
    
    /// Enable preview mode for version bumping operations
    ///
    /// When enabled, all version bump operations default to preview mode,
    /// requiring explicit confirmation to apply changes. Essential for
    /// enterprise environments with change control processes.
    enable_preview_mode: bool,
    
    /// Template for snapshot versions in unified mode
    ///
    /// Defines how snapshot versions should be formatted when using
    /// unified versioning with snapshot bumps. Supports placeholders:
    /// - {sha}: Git commit SHA
    /// - {timestamp}: Unix timestamp
    /// - {branch}: Current branch name
    unified_snapshot_template: String,
}

impl MonorepoVersionBumpConfig {
    /// Create a new configuration with the specified strategy
    ///
    /// # Arguments
    ///
    /// * `strategy` - The versioning strategy to use
    ///
    /// # Returns
    ///
    /// A new configuration with default settings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    ///
    /// let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified);
    /// assert!(config.strategy().is_unified());
    /// ```
    #[must_use]
    pub fn new(strategy: MonorepoVersioningStrategy) -> Self {
        Self {
            strategy,
            sync_on_major_bump: false,
            independent_packages: HashSet::new(),
            enable_preview_mode: false,
            unified_snapshot_template: "{sha}".to_string(),
        }
    }
    
    /// Set whether to sync all packages on major bumps
    ///
    /// # Arguments
    ///
    /// * `sync` - Whether to enable synchronization on major bumps
    ///
    /// # Returns
    ///
    /// Updated configuration (builder pattern)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    ///
    /// let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Individual)
    ///     .with_sync_on_major_bump(true);
    /// assert!(config.sync_on_major_bump());
    /// ```
    #[must_use]
    pub fn with_sync_on_major_bump(mut self, sync: bool) -> Self {
        self.sync_on_major_bump = sync;
        self
    }
    
    /// Set packages that should remain independent
    ///
    /// # Arguments
    ///
    /// * `packages` - Set of package names to keep independent
    ///
    /// # Returns
    ///
    /// Updated configuration (builder pattern)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    /// use std::collections::HashSet;
    ///
    /// let mut independent = HashSet::new();
    /// independent.insert("legacy-pkg".to_string());
    ///
    /// let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified)
    ///     .with_independent_packages(independent);
    /// assert!(config.independent_packages().contains("legacy-pkg"));
    /// ```
    #[must_use]
    pub fn with_independent_packages(mut self, packages: HashSet<String>) -> Self {
        self.independent_packages = packages;
        self
    }
    
    /// Set whether to enable preview mode by default
    ///
    /// # Arguments
    ///
    /// * `enable` - Whether to enable preview mode
    ///
    /// # Returns
    ///
    /// Updated configuration (builder pattern)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    ///
    /// let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Individual)
    ///     .with_preview_mode(true);
    /// assert!(config.enable_preview_mode());
    /// ```
    #[must_use]
    pub fn with_preview_mode(mut self, enable: bool) -> Self {
        self.enable_preview_mode = enable;
        self
    }
    
    /// Set the snapshot template for unified versioning
    ///
    /// # Arguments
    ///
    /// * `template` - Template string with placeholders
    ///
    /// # Returns
    ///
    /// Updated configuration (builder pattern)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    ///
    /// let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified)
    ///     .with_snapshot_template("beta-{sha}".to_string());
    /// assert_eq!(config.unified_snapshot_template(), "beta-{sha}");
    /// ```
    #[must_use]
    pub fn with_snapshot_template(mut self, template: String) -> Self {
        self.unified_snapshot_template = template;
        self
    }
    
    /// Get the versioning strategy
    ///
    /// # Returns
    ///
    /// Reference to the current versioning strategy
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    ///
    /// let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified);
    /// assert!(config.strategy().is_unified());
    /// ```
    #[must_use]
    pub fn strategy(&self) -> &MonorepoVersioningStrategy {
        &self.strategy
    }
    
    /// Check if synchronization on major bumps is enabled
    ///
    /// # Returns
    ///
    /// `true` if major bumps synchronize all packages
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    ///
    /// let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Individual);
    /// assert!(!config.sync_on_major_bump());
    /// ```
    #[must_use]
    pub fn sync_on_major_bump(&self) -> bool {
        self.sync_on_major_bump
    }
    
    /// Get the set of independent packages
    ///
    /// # Returns
    ///
    /// Reference to the set of package names that remain independent
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    ///
    /// let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified);
    /// assert!(config.independent_packages().is_empty());
    /// ```
    #[must_use]
    pub fn independent_packages(&self) -> &HashSet<String> {
        &self.independent_packages
    }
    
    /// Check if preview mode is enabled by default
    ///
    /// # Returns
    ///
    /// `true` if operations default to preview mode
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    ///
    /// let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Individual);
    /// assert!(!config.enable_preview_mode());
    /// ```
    #[must_use]
    pub fn enable_preview_mode(&self) -> bool {
        self.enable_preview_mode
    }
    
    /// Get the snapshot template for unified versioning
    ///
    /// # Returns
    ///
    /// String slice containing the template
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    ///
    /// let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified);
    /// assert_eq!(config.unified_snapshot_template(), "{sha}");
    /// ```
    #[must_use]
    pub fn unified_snapshot_template(&self) -> &str {
        &self.unified_snapshot_template
    }
    
    /// Add a package to the independent packages set
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to add
    ///
    /// # Returns
    ///
    /// `true` if the package was newly added, `false` if it already existed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    ///
    /// let mut config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified);
    /// assert!(config.add_independent_package("legacy-pkg".to_string()));
    /// assert!(!config.add_independent_package("legacy-pkg".to_string()));
    /// ```
    pub fn add_independent_package(&mut self, package_name: String) -> bool {
        self.independent_packages.insert(package_name)
    }
    
    /// Remove a package from the independent packages set
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to remove
    ///
    /// # Returns
    ///
    /// `true` if the package was removed, `false` if it wasn't present
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    ///
    /// let mut config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified);
    /// config.add_independent_package("test-pkg".to_string());
    /// assert!(config.remove_independent_package("test-pkg"));
    /// assert!(!config.remove_independent_package("test-pkg"));
    /// ```
    pub fn remove_independent_package(&mut self, package_name: &str) -> bool {
        self.independent_packages.remove(package_name)
    }
    
    /// Check if a package is marked as independent
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to check
    ///
    /// # Returns
    ///
    /// `true` if the package is in the independent set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
    ///
    /// let mut config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified);
    /// config.add_independent_package("legacy-pkg".to_string());
    /// assert!(config.is_package_independent("legacy-pkg"));
    /// assert!(!config.is_package_independent("normal-pkg"));
    /// ```
    #[must_use]
    pub fn is_package_independent(&self, package_name: &str) -> bool {
        self.independent_packages.contains(package_name)
    }
}

impl Default for MonorepoVersionBumpConfig {
    /// Default configuration uses Individual strategy with safe defaults
    ///
    /// - Strategy: Individual (backward compatible)
    /// - Sync on major: false (maintain independence)
    /// - Preview mode: false (user choice)
    /// - Independent packages: empty set
    /// - Snapshot template: "{sha}" (simple default)
    fn default() -> Self {
        Self::new(MonorepoVersioningStrategy::default())
    }
}

