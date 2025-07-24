//! # Context-Aware Dependency Classification
//!
//! This module provides dependency classification that adapts its behavior
//! based on the project context (single repository vs monorepo).
//!
//! ## Classification Strategies
//!
//! ### Single Repository Context
//! - **Simple Classification**: Only `file:` protocol dependencies are internal
//! - **External by Default**: All registry, git, and URL dependencies are external
//! - **No Workspace Support**: Workspace protocols are rejected
//!
//! ### Monorepo Context  
//! - **Name-Based Classification**: Dependencies are internal if they match workspace packages
//! - **Mixed References**: Support for different protocols referring to the same package
//! - **Workspace Protocol Support**: Full support for workspace: protocols
//!
//! ## Examples
//!
//! ```rust
//! use sublime_package_tools::context::{DependencyClassifier, ProjectContext, DependencyClass};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let context = ProjectContext::Single(Default::default());
//! let classifier = DependencyClassifier::new(context);
//!
//! // In single repository, only file: is internal
//! let dep_string = "file:../local-package";
//! let classification = classifier.classify_dependency(dep_string)?;
//! assert!(classification.class.is_internal());
//!
//! // Registry dependencies are external in single repo
//! let dep_string = "^1.0.0";
//! let classification = classifier.classify_dependency(dep_string)?;
//! assert_eq!(classification.class, DependencyClass::External);
//! # Ok(())
//! # }
//! ```

use crate::{
    context::{ProjectContext, SingleRepositoryContext, MonorepoContext, protocols::DependencyProtocol},
    errors::VersionError,
    Dependency,
};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Context-aware dependency classifier
///
/// The dependency classifier adapts its classification logic based on the
/// project context, providing different strategies for single repositories
/// and monorepos.
///
/// ## Classification Logic
///
/// - **Single Repository**: Protocol-based (only file: = internal)
/// - **Monorepo**: Name-based (package name in workspace = internal)
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::context::{DependencyClassifier, ProjectContext};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let context = ProjectContext::Single(Default::default());
/// let classifier = DependencyClassifier::new(context);
///
/// let result = classifier.classify_dependency("workspace:*")?;
/// // In single repo context, workspace protocols are rejected
/// assert!(result.has_errors());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct DependencyClassifier {
    /// Project context that determines classification strategy
    context: ProjectContext,
    /// Cache of classification results for performance
    classification_cache: HashMap<String, ClassificationResult>,
    /// Whether to enable strict validation
    strict_mode: bool,
}

impl DependencyClassifier {
    /// Create a new dependency classifier for the given context
    ///
    /// # Arguments
    ///
    /// * `context` - The project context that determines classification strategy
    ///
    /// # Returns
    ///
    /// A new dependency classifier instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{DependencyClassifier, ProjectContext};
    ///
    /// let context = ProjectContext::Single(Default::default());
    /// let classifier = DependencyClassifier::new(context);
    /// ```
    #[must_use]
    pub fn new(context: ProjectContext) -> Self {
        Self {
            context,
            classification_cache: HashMap::new(),
            strict_mode: true,
        }
    }

    /// Create a classifier with relaxed validation
    ///
    /// # Arguments
    ///
    /// * `context` - The project context
    ///
    /// # Returns
    ///
    /// A classifier that allows more permissive dependency formats
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{DependencyClassifier, ProjectContext};
    ///
    /// let context = ProjectContext::Monorepo(Default::default());
    /// let classifier = DependencyClassifier::relaxed(context);
    /// ```
    #[must_use]
    pub fn relaxed(context: ProjectContext) -> Self {
        Self {
            context,
            classification_cache: HashMap::new(),
            strict_mode: false,
        }
    }

    /// Classify a dependency based on its specification string
    ///
    /// This is the main classification method that determines whether a
    /// dependency is internal or external based on the project context.
    ///
    /// # Arguments
    ///
    /// * `dep_string` - The dependency specification string
    ///
    /// # Returns
    ///
    /// A classification result with the dependency class and metadata
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - The dependency string is invalid
    /// - Unsupported protocols are used in strict mode
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{DependencyClassifier, ProjectContext, DependencyClass};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let context = ProjectContext::Single(Default::default());
    /// let classifier = DependencyClassifier::new(context);
    ///
    /// let result = classifier.classify_dependency("^1.0.0")?;
    /// assert_eq!(result.class, DependencyClass::External);
    /// # Ok(())
    /// # }
    /// ```
    pub fn classify_dependency(&mut self, dep_string: &str) -> Result<ClassificationResult, VersionError> {
        // Check cache first
        if let Some(cached) = self.classification_cache.get(dep_string) {
            return Ok(cached.clone());
        }

        let result = match &self.context {
            ProjectContext::Single(config) => self.classify_for_single_repo(dep_string, config)?,
            ProjectContext::Monorepo(config) => self.classify_for_monorepo(dep_string, config)?,
        };

        // Cache the result
        self.classification_cache.insert(dep_string.to_string(), result.clone());
        Ok(result)
    }

    /// Classify a structured dependency object
    ///
    /// # Arguments
    ///
    /// * `dependency` - The dependency object to classify
    ///
    /// # Returns
    ///
    /// A classification result
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{Dependency, context::{DependencyClassifier, ProjectContext}};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dependency = Dependency::new("react", "^18.0.0")?;
    /// let context = ProjectContext::Single(Default::default());
    /// let mut classifier = DependencyClassifier::new(context);
    ///
    /// let result = classifier.classify_dependency_object(&dependency)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn classify_dependency_object(&mut self, dependency: &Dependency) -> Result<ClassificationResult, VersionError> {
        let dep_string = format!("{}@{}", dependency.name(), dependency.version());
        self.classify_dependency(&dep_string)
    }

    /// Clear the classification cache
    ///
    /// Useful when the project context or workspace packages change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{DependencyClassifier, ProjectContext};
    ///
    /// let context = ProjectContext::Single(Default::default());
    /// let mut classifier = DependencyClassifier::new(context);
    /// classifier.clear_cache();
    /// ```
    pub fn clear_cache(&mut self) {
        self.classification_cache.clear();
    }

    /// Get cache statistics
    ///
    /// # Returns
    ///
    /// The number of cached classification results
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{DependencyClassifier, ProjectContext};
    ///
    /// let context = ProjectContext::Single(Default::default());
    /// let classifier = DependencyClassifier::new(context);
    /// println!("Cache size: {}", classifier.cache_size());
    /// ```
    #[must_use]
    pub fn cache_size(&self) -> usize {
        self.classification_cache.len()
    }

    /// Classify dependency for single repository context
    fn classify_for_single_repo(
        &self, 
        dep_string: &str, 
        _config: &SingleRepositoryContext
    ) -> Result<ClassificationResult, VersionError> {
        let protocol = DependencyProtocol::parse(dep_string);
        let warnings = Vec::new();
        let mut errors = Vec::new();

        // In single repositories, reject workspace protocols
        if protocol.is_workspace_only() {
            let error = "Workspace protocols are not supported in single repository contexts".to_string();
            if self.strict_mode {
                return Err(VersionError::InvalidVersion(error));
            }
            errors.push(error);
        }

        // Simple classification: only file: = internal
        let class = match protocol {
            DependencyProtocol::File => DependencyClass::Internal {
                reference_type: InternalReferenceType::LocalFile,
                warning: None,
            },
            _ => DependencyClass::External,
        };

        Ok(ClassificationResult {
            class,
            protocol,
            strategy: ClassificationStrategy::ProtocolBased,
            confidence: 1.0, // Simple logic, high confidence
            warnings,
            errors,
        })
    }

    /// Classify dependency for monorepo context
    fn classify_for_monorepo(
        &self, 
        dep_string: &str, 
        config: &MonorepoContext
    ) -> Result<ClassificationResult, VersionError> {
        let protocol = DependencyProtocol::parse(dep_string);
        let mut warnings = Vec::new();
        let errors = Vec::new();

        // Extract package name from dependency string
        let package_name = self.extract_package_name(dep_string, &protocol);

        // Check if package name is in workspace
        let package_name_present = package_name.is_some();
        let class = if let Some(pkg_name) = package_name {
            if config.workspace_packages.contains_key(&pkg_name) {
                // This is an internal package - determine reference type and warnings
                let (reference_type, warning) = self.analyze_internal_reference(dep_string, &protocol);
                
                // Add protocol-specific warnings
                if !protocol.is_filesystem_based() && !protocol.is_workspace_only() {
                    warnings.push(format!(
                        "Package '{}' is internal but uses '{}' protocol. Consider using 'workspace:' for consistency.",
                        pkg_name, protocol
                    ));
                }
                
                DependencyClass::Internal {
                    reference_type,
                    warning,
                }
            } else {
                DependencyClass::External
            }
        } else {
            // Fallback to protocol-based classification with context-aware warnings
            match protocol {
                DependencyProtocol::File => {
                    let warning = if self.context.is_monorepo() {
                        Some("Consider using workspace: protocol for better consistency in monorepo".to_string())
                    } else {
                        None
                    };
                    DependencyClass::Internal {
                        reference_type: InternalReferenceType::LocalFile,
                        warning,
                    }
                }
                DependencyProtocol::Workspace => DependencyClass::Internal {
                    reference_type: InternalReferenceType::WorkspaceProtocol,
                    warning: None,
                },
                _ => DependencyClass::External,
            }
        };

        Ok(ClassificationResult {
            class,
            protocol,
            strategy: ClassificationStrategy::NameBased,
            confidence: if package_name_present { 0.9 } else { 0.6 },
            warnings,
            errors,
        })
    }

    /// Analyze internal reference to determine type and warnings
    fn analyze_internal_reference(&self, dep_string: &str, protocol: &DependencyProtocol) -> (InternalReferenceType, Option<String>) {
        match protocol {
            DependencyProtocol::Workspace => {
                (InternalReferenceType::WorkspaceProtocol, None)
            }
            DependencyProtocol::File => {
                let warning = if self.context.is_monorepo() {
                    Some("Consider using workspace: protocol for better consistency in monorepo".to_string())
                } else {
                    None
                };
                (InternalReferenceType::LocalFile, warning)
            }
            DependencyProtocol::Registry | DependencyProtocol::Scoped => {
                // Extract version from dependency string
                let version = self.extract_version_from_dep_string(dep_string)
                    .unwrap_or_else(|| "unknown".to_string());
                
                let warning = if self.context.is_monorepo() {
                    Some("Consider using workspace: protocol for internal dependencies".to_string())
                } else {
                    None
                };
                
                (InternalReferenceType::RegistryVersion(version), warning)
            }
            _ => {
                let warning = Some("Unusual reference type for internal package".to_string());
                (InternalReferenceType::Other, warning)
            }
        }
    }

    /// Extract version from dependency string
    fn extract_version_from_dep_string(&self, dep_string: &str) -> Option<String> {
        if let Some(at_pos) = dep_string.rfind('@') {
            // Handle scoped packages: @scope/package@version
            if dep_string.starts_with('@') && dep_string[1..at_pos].contains('/') {
                Some(dep_string[at_pos + 1..].to_string())
            } else if !dep_string.starts_with('@') {
                // Regular package@version
                Some(dep_string[at_pos + 1..].to_string())
            } else {
                None
            }
        } else {
            // Assume this is just a version spec
            Some(dep_string.to_string())
        }
    }

    /// Extract package name from dependency string
    fn extract_package_name(&self, dep_string: &str, protocol: &DependencyProtocol) -> Option<String> {
        match protocol {
            DependencyProtocol::Workspace => {
                // workspace:package-name or workspace:*
                if let Some(name) = dep_string.strip_prefix("workspace:") {
                    if name == "*" {
                        None // Cannot determine package name from workspace:*
                    } else {
                        Some(name.to_string())
                    }
                } else {
                    None
                }
            }
            DependencyProtocol::Scoped => {
                // @scope/package@version -> @scope/package
                if let Some(at_pos) = dep_string.rfind('@') {
                    Some(dep_string[..at_pos].to_string())
                } else {
                    Some(dep_string.to_string())
                }
            }
            _ => {
                // package@version -> package
                if let Some(at_pos) = dep_string.find('@') {
                    Some(dep_string[..at_pos].to_string())
                } else if dep_string.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                    // Looks like a package name without version
                    Some(dep_string.to_string())
                } else {
                    None
                }
            }
        }
    }
}

/// Dependency classification result
///
/// Contains the classification decision along with metadata about
/// how the decision was made and any issues encountered.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassificationResult {
    /// The dependency classification (internal or external)
    pub class: DependencyClass,
    /// The detected dependency protocol
    pub protocol: DependencyProtocol,
    /// The strategy used for classification
    pub strategy: ClassificationStrategy,
    /// Confidence level of the classification (0.0 to 1.0)
    pub confidence: f64,
    /// Warnings generated during classification
    pub warnings: Vec<String>,
    /// Errors encountered during classification
    pub errors: Vec<String>,
}

impl ClassificationResult {
    /// Check if the classification has high confidence
    ///
    /// # Returns
    ///
    /// `true` if confidence is above 0.8, `false` otherwise
    #[must_use]
    pub fn is_high_confidence(&self) -> bool {
        self.confidence > 0.8
    }

    /// Check if there are warnings
    ///
    /// # Returns
    ///
    /// `true` if there are warnings, `false` otherwise
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Check if there are errors
    ///
    /// # Returns
    ///
    /// `true` if there are errors, `false` otherwise
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if the dependency is internal
    ///
    /// # Returns
    ///
    /// `true` if classified as internal, `false` otherwise
    #[must_use]
    pub fn is_internal(&self) -> bool {
        self.class.is_internal()
    }

    /// Check if the dependency is external
    ///
    /// # Returns
    ///
    /// `true` if classified as external, `false` otherwise
    #[must_use]
    pub fn is_external(&self) -> bool {
        self.class.is_external()
    }

    /// Get the reference type for internal dependencies
    ///
    /// # Returns
    ///
    /// Optional reference type if this is an internal dependency
    #[must_use]
    pub fn reference_type(&self) -> Option<&InternalReferenceType> {
        self.class.reference_type()
    }

    /// Get all warnings including both classification warnings and class warnings
    ///
    /// # Returns
    ///
    /// Iterator over all warning messages
    pub fn all_warnings(&self) -> impl Iterator<Item = &str> {
        self.warnings.iter().map(|s| s.as_str())
            .chain(self.class.warning())
    }
}

/// Dependency classification types with detailed metadata
///
/// Represents whether a dependency is internal to the project/workspace
/// or external from a registry or other source, including detailed
/// information about the reference type and any warnings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyClass {
    /// Internal dependency (part of the same project/workspace)
    Internal {
        /// Type of reference used for this internal dependency
        reference_type: InternalReferenceType,
        /// Optional warning about the reference type
        warning: Option<String>,
    },
    /// External dependency (from registry, git, URL, etc.)
    External,
}

impl DependencyClass {
    /// Check if this is an internal dependency
    ///
    /// # Returns
    ///
    /// `true` if this is an internal dependency, `false` otherwise
    #[must_use]
    pub fn is_internal(&self) -> bool {
        matches!(self, Self::Internal { .. })
    }

    /// Check if this is an external dependency
    ///
    /// # Returns
    ///
    /// `true` if this is an external dependency, `false` otherwise
    #[must_use]
    pub fn is_external(&self) -> bool {
        matches!(self, Self::External)
    }

    /// Get the warning message for internal dependencies
    ///
    /// # Returns
    ///
    /// Optional warning message for internal dependencies
    #[must_use]
    pub fn warning(&self) -> Option<&str> {
        match self {
            Self::Internal { warning, .. } => warning.as_deref(),
            Self::External => None,
        }
    }

    /// Get the reference type for internal dependencies
    ///
    /// # Returns
    ///
    /// Optional reference type for internal dependencies
    #[must_use]
    pub fn reference_type(&self) -> Option<&InternalReferenceType> {
        match self {
            Self::Internal { reference_type, .. } => Some(reference_type),
            Self::External => None,
        }
    }
}

impl std::fmt::Display for DependencyClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal { reference_type, .. } => write!(f, "internal({})", reference_type),
            Self::External => write!(f, "external"),
        }
    }
}

/// Classification strategy used to determine dependency class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClassificationStrategy {
    /// Protocol-based classification (single repository)
    ProtocolBased,
    /// Name-based classification (monorepo)
    NameBased,
    /// Hybrid classification using both protocol and name
    Hybrid,
}

impl std::fmt::Display for ClassificationStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProtocolBased => write!(f, "protocol-based"),
            Self::NameBased => write!(f, "name-based"),
            Self::Hybrid => write!(f, "hybrid"),
        }
    }
}

/// Types of internal dependency references
///
/// Represents different ways internal dependencies can be referenced
/// within a project or workspace, with implications for consistency
/// and maintainability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InternalReferenceType {
    /// Workspace protocol reference (`workspace:*`, `workspace:^`)
    ///
    /// This is the ideal way to reference internal dependencies in monorepos.
    /// Provides automatic version resolution and workspace-aware handling.
    WorkspaceProtocol,
    
    /// Local file path reference (`file:../local-package`)
    ///
    /// Direct file system reference to a local package. Works in both
    /// single repositories and monorepos, but workspace protocol is
    /// preferred in monorepos for better consistency.
    LocalFile,
    
    /// Registry version reference (`^1.0.0`, `~2.1.0`)
    ///
    /// Standard semver reference that happens to point to an internal
    /// package. Works but can be inconsistent in monorepos where
    /// workspace protocol would be more appropriate.
    RegistryVersion(String),
    
    /// Other reference types (git, jsr, URL, etc.)
    ///
    /// Uncommon but possible ways to reference internal packages,
    /// such as git repositories or alternative registries.
    Other,
}

impl std::fmt::Display for InternalReferenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkspaceProtocol => write!(f, "workspace"),
            Self::LocalFile => write!(f, "file"),
            Self::RegistryVersion(version) => write!(f, "registry:{}", version),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Internal dependency classification method (from project.rs)
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