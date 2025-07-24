//! # Project Context Definitions
//!
//! This module defines the core project context types that enable context-aware
//! architecture throughout the package tools system.
//!
//! ## Project Context Types
//!
//! - **Single Repository**: Traditional single-package project structure
//! - **Monorepo**: Multi-package workspace with internal dependencies
//!
//! Each context type has its own configuration, supported features, and protocols
//! to optimize the package tools experience for that specific environment.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::context::protocols::DependencyProtocol;

/// Project context defining the structure and capabilities of the current project
///
/// This enum is the core of the context-aware architecture, allowing the system
/// to adapt its behavior based on the detected project structure.
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::context::ProjectContext;
///
/// # fn example(context: ProjectContext) {
/// match context {
///     ProjectContext::Single(config) => {
///         println!("Single repository with {} protocols", config.supported_protocols.len());
///         // Optimize for network operations
///     }
///     ProjectContext::Monorepo(config) => {
///         println!("Monorepo with {} packages", config.workspace_packages.len());
///         // Enable workspace features and cascade operations
///     }
/// }
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectContext {
    /// Single repository context - traditional single-package project
    Single(SingleRepositoryContext),
    /// Monorepo context - multi-package workspace project  
    Monorepo(MonorepoContext),
}

impl ProjectContext {
    /// Check if this is a single repository context
    ///
    /// # Returns
    ///
    /// `true` if this is a single repository context, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{ProjectContext, SingleRepositoryContext};
    ///
    /// let context = ProjectContext::Single(SingleRepositoryContext::default());
    /// assert!(context.is_single());
    /// ```
    #[must_use]
    pub fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    /// Check if this is a monorepo context
    ///
    /// # Returns
    ///
    /// `true` if this is a monorepo context, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{ProjectContext, MonorepoContext};
    ///
    /// let context = ProjectContext::Monorepo(MonorepoContext::default());
    /// assert!(context.is_monorepo());
    /// ```
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        matches!(self, Self::Monorepo(_))
    }

    /// Get the supported dependency protocols for this context
    ///
    /// # Returns
    ///
    /// A slice of supported dependency protocols
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{ProjectContext, SingleRepositoryContext};
    ///
    /// let context = ProjectContext::Single(SingleRepositoryContext::default());
    /// let protocols = context.supported_protocols();
    /// println!("Supports {} protocols", protocols.len());
    /// ```
    #[must_use]
    pub fn supported_protocols(&self) -> &[DependencyProtocol] {
        match self {
            Self::Single(config) => &config.supported_protocols,
            Self::Monorepo(config) => &config.supported_protocols,
        }
    }
}

/// Configuration for single repository contexts
///
/// Single repositories are traditional single-package projects with simpler
/// dependency management needs. They don't support workspace protocols
/// and are optimized for network operations.
///
/// ## Features
///
/// - All dependency protocols except workspace
/// - Network-optimized operations
/// - Simplified dependency classification (only file: = internal)
/// - No cascade operations
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::context::{SingleRepositoryContext, SingleRepoFeatures};
///
/// let config = SingleRepositoryContext {
///     features_enabled: SingleRepoFeatures::basic(),
///     ..Default::default()
/// };
/// assert!(!config.features_enabled.cascade_bumping);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleRepositoryContext {
    /// Dependency protocols supported in single repository context
    pub supported_protocols: Vec<DependencyProtocol>,
    /// Features enabled for single repository operations
    pub features_enabled: SingleRepoFeatures,
    /// Internal dependency classification strategy
    pub internal_classification: InternalClassification,
}

impl Default for SingleRepositoryContext {
    fn default() -> Self {
        Self {
            supported_protocols: DependencyProtocol::all_except_workspace(),
            features_enabled: SingleRepoFeatures::basic(),
            internal_classification: InternalClassification::FileOnly,
        }
    }
}

/// Configuration for monorepo contexts
///
/// Monorepos are multi-package workspaces with complex internal dependency
/// relationships. They support all dependency protocols including workspace
/// and are optimized for filesystem operations.
///
/// ## Features
///
/// - All dependency protocols including workspace
/// - Filesystem-optimized operations
/// - Complex dependency classification (name-based + protocol-based)
/// - Cascade operations and workspace management
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::context::{MonorepoContext, MonorepoFeatures};
/// use std::collections::HashMap;
///
/// let mut workspace_packages = HashMap::new();
/// workspace_packages.insert("package-a".to_string(), "packages/a".to_string());
/// workspace_packages.insert("package-b".to_string(), "packages/b".to_string());
///
/// let config = MonorepoContext {
///     workspace_packages,
///     features_enabled: MonorepoFeatures::all(),
///     ..Default::default()
/// };
/// assert!(config.features_enabled.cascade_bumping);
/// assert!(config.features_enabled.workspace_protocols);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonorepoContext {
    /// Map of package names to their relative paths in the workspace
    pub workspace_packages: HashMap<String, String>,
    /// Dependency protocols supported in monorepo context (all protocols)
    pub supported_protocols: Vec<DependencyProtocol>,
    /// Features enabled for monorepo operations
    pub features_enabled: MonorepoFeatures,
    /// Internal dependency classification strategy
    pub internal_classification: InternalClassification,
}

impl Default for MonorepoContext {
    fn default() -> Self {
        Self {
            workspace_packages: HashMap::new(),
            supported_protocols: DependencyProtocol::all(),
            features_enabled: MonorepoFeatures::all(),
            internal_classification: InternalClassification::NameBased,
        }
    }
}

/// Features available in single repository context
///
/// Single repositories have a simplified feature set focused on
/// network operations and basic dependency management.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SingleRepoFeatures {
    /// Whether version bumping is enabled
    pub version_bumping: bool,
    /// Whether dependency resolution is enabled
    pub dependency_resolution: bool,
    /// Whether upgrade checking is enabled
    pub upgrade_checking: bool,
    /// Cascade bumping is always disabled in single repositories
    pub cascade_bumping: bool,
    /// Workspace protocols are not supported in single repositories
    pub workspace_protocols: bool,
    /// Whether network caching is enabled
    pub network_caching: bool,
}

impl SingleRepoFeatures {
    /// Create basic single repository features configuration
    ///
    /// # Returns
    ///
    /// A basic configuration suitable for most single repository projects
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::SingleRepoFeatures;
    ///
    /// let features = SingleRepoFeatures::basic();
    /// assert!(features.version_bumping);
    /// assert!(!features.cascade_bumping);
    /// assert!(!features.workspace_protocols);
    /// ```
    #[must_use]
    pub fn basic() -> Self {
        Self {
            version_bumping: true,
            dependency_resolution: true,
            upgrade_checking: true,
            cascade_bumping: false,           // Never enabled in single repos
            workspace_protocols: false,      // Never supported in single repos
            network_caching: true,
        }
    }
}

impl Default for SingleRepoFeatures {
    fn default() -> Self {
        Self::basic()
    }
}

/// Features available in monorepo context
///
/// Monorepos have access to the full feature set including advanced
/// workspace management and cascade operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonorepoFeatures {
    /// Whether version bumping is enabled
    pub version_bumping: bool,
    /// Whether dependency resolution is enabled
    pub dependency_resolution: bool,
    /// Whether upgrade checking is enabled
    pub upgrade_checking: bool,
    /// Whether cascade bumping is enabled (bump dependents automatically)
    pub cascade_bumping: bool,
    /// Whether workspace protocols are supported
    pub workspace_protocols: bool,
    /// Whether internal dependency classification is enabled
    pub internal_classification: bool,
    /// Whether mixed reference support is enabled (A→B semver, B→C workspace)
    pub mixed_references: bool,
    /// Whether filesystem caching is enabled
    pub filesystem_caching: bool,
    /// Whether concurrent processing is enabled
    pub concurrent_processing: bool,
}

impl MonorepoFeatures {
    /// Create full monorepo features configuration
    ///
    /// # Returns
    ///
    /// A full configuration with all monorepo features enabled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::MonorepoFeatures;
    ///
    /// let features = MonorepoFeatures::all();
    /// assert!(features.cascade_bumping);
    /// assert!(features.workspace_protocols);
    /// assert!(features.mixed_references);
    /// ```
    #[must_use]
    pub fn all() -> Self {
        Self {
            version_bumping: true,
            dependency_resolution: true,
            upgrade_checking: true,
            cascade_bumping: true,
            workspace_protocols: true,
            internal_classification: true,
            mixed_references: true,
            filesystem_caching: true,
            concurrent_processing: true,
        }
    }

    /// Create basic monorepo features configuration
    ///
    /// # Returns
    ///
    /// A basic configuration with core monorepo features enabled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::MonorepoFeatures;
    ///
    /// let features = MonorepoFeatures::basic();
    /// assert!(features.workspace_protocols);
    /// assert!(!features.concurrent_processing);
    /// ```
    #[must_use]
    pub fn basic() -> Self {
        Self {
            version_bumping: true,
            dependency_resolution: true,
            upgrade_checking: true,
            cascade_bumping: true,
            workspace_protocols: true,
            internal_classification: true,
            mixed_references: false,
            filesystem_caching: true,
            concurrent_processing: false,
        }
    }
}

impl Default for MonorepoFeatures {
    fn default() -> Self {
        Self::all()
    }
}

/// Internal dependency classification strategy
///
/// Defines how dependencies are classified as internal vs external
/// based on the project context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InternalClassification {
    /// Only file: protocol dependencies are considered internal (single repo)
    FileOnly,
    /// Classification based on package name presence in workspace (monorepo)
    NameBased,
    /// Hybrid approach using both protocol and name-based classification
    Hybrid,
}

impl Default for InternalClassification {
    fn default() -> Self {
        Self::FileOnly
    }
}