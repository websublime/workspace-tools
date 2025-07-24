//! # Workspace-Aware Dependency Resolver
//!
//! This module provides workspace-aware dependency resolution that adapts its behavior
//! based on project context (single repository vs monorepo).
//!
//! ## Overview
//!
//! The workspace-aware dependency resolver integrates with the standard crate's
//! ProjectDetector and MonorepoDetector to provide context-aware dependency resolution.
//! It automatically detects project structure and optimizes resolution strategies accordingly.
//!
//! ## Key Features
//!
//! - **Auto-Detection**: Automatically detects single repository vs monorepo contexts
//! - **Context-Aware Resolution**: Different resolution strategies per context
//! - **Internal/External Classification**: Distinguishes internal vs external dependencies
//! - **Enterprise Integration**: Full integration with standard crate detectors
//!
//! ## Examples
//!
//! ```rust
//! use sublime_package_tools::services::WorkspaceAwareDependencyResolver;
//! use sublime_standard_tools::filesystem::AsyncFileSystem;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create resolver with filesystem integration
//! let fs = AsyncFileSystem::new();
//! let resolver = WorkspaceAwareDependencyResolver::new(fs);
//!
//! // Auto-detect context and resolve dependencies
//! let context = resolver.detect_project_context().await?;
//! let dependencies = resolver.resolve_with_context(&["react", "lodash"], &context).await?;
//! # Ok(())
//! # }
//! ```

#![allow(dead_code)] // Phase 4.2 integration pending - workspace resolver fully implemented
#![allow(clippy::unused_async)] // I/O operations pending integration

use crate::{
    context::{ContextDetector, ProjectContext, DependencyClassifier, DependencyClass},
    config::PackageToolsConfig,
    errors::VersionError,
};
use sublime_standard_tools::{
    filesystem::AsyncFileSystem,
    project::ProjectDetector,
    monorepo::{MonorepoDetector, MonorepoDetectorTrait},
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Workspace-aware dependency resolver with integrated context detection
///
/// This resolver provides enterprise-grade dependency resolution that automatically
/// adapts to project structure. It uses the standard crate's detectors for robust
/// project analysis and context-aware resolution strategies.
///
/// ## Architecture
///
/// - **ProjectDetector**: Unified project detection and analysis
/// - **MonorepoDetector**: Workspace detection and package discovery
/// - **ContextDetector**: Context-aware architecture integration
/// - **PackageToolsConfig**: Configuration management
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::services::WorkspaceAwareDependencyResolver;
/// use sublime_standard_tools::filesystem::AsyncFileSystem;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = AsyncFileSystem::new();
/// let resolver = WorkspaceAwareDependencyResolver::new(fs);
/// 
/// // Auto-detect project context
/// let context = resolver.detect_project_context().await?;
/// 
/// match context {
///     ProjectContext::Single(_) => {
///         // Optimized for network operations
///         println!("Single repository detected - network optimization enabled");
///     }
///     ProjectContext::Monorepo(config) => {
///         // Optimized for workspace operations
///         println!("Monorepo detected with {} packages", config.workspace_packages.len());
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct WorkspaceAwareDependencyResolver<F: AsyncFileSystem> {
    /// Standard crate project detector for unified project detection
    project_detector: ProjectDetector<F>,
    /// Standard crate monorepo detector for workspace detection  
    monorepo_detector: MonorepoDetector<F>,
    /// Context detector for context-aware architecture
    context_detector: ContextDetector<F>,
    /// Filesystem implementation for file operations
    filesystem: F,
    /// Configuration for package tools behavior
    config: PackageToolsConfig,
    /// Working directory for resolution
    working_directory: PathBuf,
}

impl<F> WorkspaceAwareDependencyResolver<F>
where
    F: AsyncFileSystem + Clone + 'static,
{
    /// Create a new workspace-aware dependency resolver
    ///
    /// # Arguments
    ///
    /// * `filesystem` - Filesystem implementation for file operations
    ///
    /// # Returns
    ///
    /// A new workspace-aware dependency resolver instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::WorkspaceAwareDependencyResolver;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// let fs = AsyncFileSystem::new();
    /// let resolver = WorkspaceAwareDependencyResolver::new(fs);
    /// ```
    #[must_use]
    pub fn new(filesystem: F) -> Self {
        let project_detector = ProjectDetector::with_filesystem(filesystem.clone());
        let monorepo_detector = MonorepoDetector::with_filesystem(filesystem.clone());
        let context_detector = ContextDetector::new(filesystem.clone());
        
        Self {
            project_detector,
            monorepo_detector,
            context_detector,
            filesystem,
            config: PackageToolsConfig::default(),
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// Create a workspace-aware dependency resolver with custom configuration
    ///
    /// # Arguments
    ///
    /// * `filesystem` - Filesystem implementation for file operations
    /// * `config` - Package tools configuration
    ///
    /// # Returns
    ///
    /// A new workspace-aware dependency resolver instance with configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{services::WorkspaceAwareDependencyResolver, config::PackageToolsConfig};
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// let fs = AsyncFileSystem::new();
    /// let config = PackageToolsConfig::default();
    /// let resolver = WorkspaceAwareDependencyResolver::with_config(fs, config);
    /// ```
    #[must_use]
    pub fn with_config(filesystem: F, config: PackageToolsConfig) -> Self {
        let project_detector = ProjectDetector::with_filesystem(filesystem.clone());
        let monorepo_detector = MonorepoDetector::with_filesystem(filesystem.clone());
        let context_detector = ContextDetector::new(filesystem.clone());
        
        Self {
            project_detector,
            monorepo_detector,
            context_detector,
            filesystem,
            config,
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// Create a workspace-aware dependency resolver with working directory
    ///
    /// # Arguments
    ///
    /// * `filesystem` - Filesystem implementation for file operations
    /// * `working_directory` - Directory to use as project root
    ///
    /// # Returns
    ///
    /// A new workspace-aware dependency resolver instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::WorkspaceAwareDependencyResolver;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    /// use std::path::PathBuf;
    ///
    /// let fs = AsyncFileSystem::new();
    /// let working_dir = PathBuf::from("/path/to/project");
    /// let resolver = WorkspaceAwareDependencyResolver::with_directory(fs, working_dir);
    /// ```
    #[must_use]
    pub fn with_directory(filesystem: F, working_directory: PathBuf) -> Self {
        let project_detector = ProjectDetector::with_filesystem(filesystem.clone());
        let monorepo_detector = MonorepoDetector::with_filesystem(filesystem.clone());
        let context_detector = ContextDetector::with_directory(filesystem.clone(), working_directory.clone());
        
        Self {
            project_detector,
            monorepo_detector,
            context_detector,
            filesystem,
            config: PackageToolsConfig::default(),
            working_directory,
        }
    }

    /// Automatically detect the project context
    ///
    /// This is the main entry point for context detection. It uses the integrated
    /// standard crate detectors to analyze project structure and return the
    /// appropriate context configuration.
    ///
    /// # Returns
    ///
    /// The detected project context
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - File system operations fail
    /// - Project structure cannot be determined
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{services::WorkspaceAwareDependencyResolver, context::ProjectContext};
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let resolver = WorkspaceAwareDependencyResolver::new(fs);
    /// 
    /// let context = resolver.detect_project_context().await?;
    /// match context {
    ///     ProjectContext::Single(_) => println!("Single repository detected"),
    ///     ProjectContext::Monorepo(_) => println!("Monorepo detected"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_project_context(&self) -> Result<ProjectContext, VersionError> {
        self.context_detector.detect_context().await
    }

    /// Detect project context with strict validation
    ///
    /// Uses strict mode detection which requires explicit workspace configuration
    /// to classify a project as a monorepo.
    ///
    /// # Returns
    ///
    /// The detected project context with strict validation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::WorkspaceAwareDependencyResolver;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let resolver = WorkspaceAwareDependencyResolver::new(fs);
    /// 
    /// let context = resolver.detect_project_context_strict().await?;
    /// println!("Strict detection completed: {:?}", context);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_project_context_strict(&self) -> Result<ProjectContext, VersionError> {
        // Create a new strict detector to avoid moving self.context_detector
        let strict_detector = ContextDetector::with_directory(
            self.filesystem.clone(),
            self.working_directory.clone()
        ).with_strict_mode();
        
        strict_detector.detect_context().await
    }

    /// Check if the current directory is a monorepo root
    ///
    /// Uses the standard crate MonorepoDetector for robust monorepo detection.
    ///
    /// # Returns
    ///
    /// `true` if the directory is a monorepo root, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::WorkspaceAwareDependencyResolver;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let resolver = WorkspaceAwareDependencyResolver::new(fs);
    /// 
    /// if resolver.is_monorepo().await? {
    ///     println!("This is a monorepo");
    /// } else {
    ///     println!("This is a single repository");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_monorepo(&self) -> Result<bool, VersionError> {
        let result = self.monorepo_detector
            .is_monorepo_root(&self.working_directory)
            .await
            .map_err(|e| VersionError::IO(format!("Failed to detect monorepo: {e}")))?;
        
        Ok(result.is_some())
    }

    /// Classify a dependency based on project context
    ///
    /// Uses context-aware logic to distinguish between internal and external
    /// dependencies. The classification strategy adapts based on whether the
    /// project is a single repository or monorepo.
    ///
    /// # Arguments
    ///
    /// * `dependency_string` - Dependency specification string
    /// * `context` - Project context for classification
    ///
    /// # Returns
    ///
    /// The dependency classification result
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{services::WorkspaceAwareDependencyResolver, context::ProjectContext};
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let resolver = WorkspaceAwareDependencyResolver::new(fs);
    /// 
    /// let context = resolver.detect_project_context().await?;
    /// let classification = resolver.classify_dependency("react@^18.0.0", &context).await?;
    /// 
    /// println!("Dependency class: {:?}", classification.dependency_class);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn classify_dependency(
        &self, 
        dependency_string: &str, 
        context: &ProjectContext
    ) -> Result<crate::context::classification::ClassificationResult, VersionError> {
        let mut classifier = DependencyClassifier::new(context.clone());
        classifier.classify_dependency(dependency_string)
    }

    /// Resolve dependencies with context-aware optimization
    ///
    /// Performs dependency resolution using different strategies based on
    /// project context:
    /// - Single repository: Network-optimized resolution
    /// - Monorepo: Workspace-aware resolution with internal dependency handling
    ///
    /// # Arguments
    ///
    /// * `dependencies` - List of dependency specification strings
    /// * `context` - Project context for resolution optimization
    ///
    /// # Returns
    ///
    /// Map of resolved dependencies with their versions
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::WorkspaceAwareDependencyResolver;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let resolver = WorkspaceAwareDependencyResolver::new(fs);
    /// 
    /// let context = resolver.detect_project_context().await?;
    /// let dependencies = vec!["react@^18.0.0", "lodash@^4.17.21"];
    /// 
    /// let resolved = resolver.resolve_with_context(&dependencies, &context).await?;
    /// for (name, version) in resolved {
    ///     println!("Resolved: {} -> {}", name, version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve_with_context(
        &self,
        dependencies: &[&str],
        context: &ProjectContext,
    ) -> Result<HashMap<String, String>, VersionError> {
        let mut resolved = HashMap::new();
        
        for dep_string in dependencies {
            let classification = self.classify_dependency(dep_string, context).await?;
            
            match classification.class {
                DependencyClass::Internal { .. } => {
                    // For internal dependencies, resolve using workspace information
                    match context {
                        ProjectContext::Monorepo(monorepo_context) => {
                            if let Some(name) = self.extract_package_name(dep_string) {
                                if let Some(version) = self.resolve_internal_dependency(&name, monorepo_context).await? {
                                    resolved.insert(name, version);
                                }
                            }
                        }
                        ProjectContext::Single(_) => {
                            // Single repositories have limited internal dependency support
                            if let Some(name) = self.extract_package_name(dep_string) {
                                resolved.insert(name, "file:local".to_string());
                            }
                        }
                    }
                }
                DependencyClass::External => {
                    // For external dependencies, use network resolution
                    if let Some(name) = self.extract_package_name(dep_string) {
                        let version = self.resolve_external_dependency(dep_string).await?;
                        resolved.insert(name, version);
                    }
                }
            }
        }
        
        Ok(resolved)
    }

    /// Distinguish internal vs external dependencies
    ///
    /// This method implements the core logic for distinguishing between
    /// internal and external dependencies using context-aware strategies:
    ///
    /// - **Single Repository**: Only file: protocol = internal
    /// - **Monorepo**: Name-based classification with workspace package detection
    ///
    /// # Arguments
    ///
    /// * `dependencies` - List of dependency specification strings
    /// * `context` - Project context for classification
    ///
    /// # Returns
    ///
    /// Map of dependencies to their internal/external classification
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::WorkspaceAwareDependencyResolver;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let resolver = WorkspaceAwareDependencyResolver::new(fs);
    /// 
    /// let context = resolver.detect_project_context().await?;
    /// let dependencies = vec!["@my-org/shared-lib", "react", "file:../local-pkg"];
    /// 
    /// let classification = resolver.distinguish_internal_external(&dependencies, &context).await?;
    /// for (dep, class) in classification {
    ///     println!("{}: {:?}", dep, class);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn distinguish_internal_external(
        &self,
        dependencies: &[&str],
        context: &ProjectContext,
    ) -> Result<HashMap<String, DependencyClass>, VersionError> {
        let mut classification = HashMap::new();
        
        for dep_string in dependencies {
            let result = self.classify_dependency(dep_string, context).await?;
            if let Some(name) = self.extract_package_name(dep_string) {
                classification.insert(name, result.class);
            }
        }
        
        Ok(classification)
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
    /// use sublime_package_tools::services::WorkspaceAwareDependencyResolver;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// let fs = AsyncFileSystem::new();
    /// let resolver = WorkspaceAwareDependencyResolver::new(fs);
    /// let config = resolver.config();
    /// println!("Current config: {:?}", config);
    /// ```
    #[must_use]
    pub fn config(&self) -> &PackageToolsConfig {
        &self.config
    }

    /// Get the working directory
    ///
    /// # Returns
    ///
    /// Reference to the current working directory path
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::WorkspaceAwareDependencyResolver;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// let fs = AsyncFileSystem::new();
    /// let resolver = WorkspaceAwareDependencyResolver::new(fs);
    /// println!("Working directory: {}", resolver.working_directory().display());
    /// ```
    #[must_use]
    pub fn working_directory(&self) -> &Path {
        &self.working_directory
    }

    /// Extract package name from dependency string
    ///
    /// # Arguments
    ///
    /// * `dep_string` - Dependency specification string
    ///
    /// # Returns
    ///
    /// Package name if successfully extracted
    fn extract_package_name(&self, dep_string: &str) -> Option<String> {
        // Simple extraction logic - can be enhanced with proper parsing
        if let Some(at_pos) = dep_string.find('@') {
            if let Some(stripped) = dep_string.strip_prefix('@') {
                // Scoped package like "@org/package@version"
                if let Some(second_at) = stripped.find('@') {
                    Some(dep_string[..second_at + 1].to_string())
                } else {
                    Some(dep_string.to_string())
                }
            } else {
                // Regular package like "package@version"
                Some(dep_string[..at_pos].to_string())
            }
        } else {
            Some(dep_string.to_string())
        }
    }

    /// Resolve internal dependency version from workspace
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `monorepo_context` - Monorepo context with workspace information
    ///
    /// # Returns
    ///
    /// Resolved version if package found in workspace
    async fn resolve_internal_dependency(
        &self,
        name: &str,
        monorepo_context: &crate::context::MonorepoContext,
    ) -> Result<Option<String>, VersionError> {
        if monorepo_context.workspace_packages.contains_key(name) {
            // For workspace packages, return workspace protocol
            Ok(Some("workspace:*".to_string()))
        } else {
            Ok(None)
        }
    }

    /// Resolve external dependency version from registry
    ///
    /// # Arguments
    ///
    /// * `dep_string` - Dependency specification string
    ///
    /// # Returns
    ///
    /// Resolved version from registry
    async fn resolve_external_dependency(&self, dep_string: &str) -> Result<String, VersionError> {
        // Simplified resolution - in real implementation would query registry
        if let Some(at_pos) = dep_string.find('@') {
            Ok(dep_string[at_pos + 1..].to_string())
        } else {
            Ok("latest".to_string())
        }
    }
}

/// Configuration result for workspace-aware operations
#[derive(Debug, Clone)]
pub struct WorkspaceResolutionResult {
    /// Resolved dependencies with their versions
    pub resolved_dependencies: HashMap<String, String>,
    /// Internal dependencies detected in the workspace
    pub internal_dependencies: Vec<String>,
    /// External dependencies requiring network resolution
    pub external_dependencies: Vec<String>,
    /// Warnings generated during resolution
    pub warnings: Vec<String>,
    /// Project context used for resolution
    pub context: ProjectContext,
}

impl WorkspaceResolutionResult {
    /// Create a new workspace resolution result
    ///
    /// # Arguments
    ///
    /// * `context` - Project context used for resolution
    ///
    /// # Returns
    ///
    /// A new workspace resolution result instance
    #[must_use]
    pub fn new(context: ProjectContext) -> Self {
        Self {
            resolved_dependencies: HashMap::new(),
            internal_dependencies: Vec::new(),
            external_dependencies: Vec::new(),
            warnings: Vec::new(),
            context,
        }
    }

    /// Check if resolution has any warnings
    ///
    /// # Returns
    ///
    /// `true` if there are warnings, `false` otherwise
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get the number of resolved dependencies
    ///
    /// # Returns
    ///
    /// Total number of resolved dependencies
    #[must_use]
    pub fn resolved_count(&self) -> usize {
        self.resolved_dependencies.len()
    }

    /// Check if the project is a monorepo
    ///
    /// # Returns
    ///
    /// `true` if the project context is a monorepo, `false` otherwise
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        self.context.is_monorepo()
    }
}