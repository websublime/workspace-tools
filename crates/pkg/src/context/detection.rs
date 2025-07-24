//! # Context Detection
//!
//! This module provides automatic detection of project context (single repository vs monorepo)
//! by analyzing the project structure and configuration files.
//!
//! ## Detection Strategy
//!
//! The context detector uses multiple heuristics to determine the project type:
//!
//! 1. **Workspace Configuration**: Checks for workspace configuration in package.json
//! 2. **Multiple Packages**: Looks for multiple package.json files in subdirectories  
//! 3. **Monorepo Tools**: Detects configuration files from monorepo tools (lerna, nx, etc.)
//! 4. **Directory Structure**: Analyzes directory structure patterns
//!
//! ## Examples
//!
//! ```rust
//! use sublime_package_tools::context::ContextDetector;
//! use sublime_standard_tools::filesystem::AsyncFileSystem;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let fs = AsyncFileSystem::new();
//! let detector = ContextDetector::new(fs);
//!
//! let context = detector.detect_context().await?;
//! println!("Detected context: {:?}", context);
//! # Ok(())
//! # }
//! ```

use crate::{
    context::{ProjectContext, SingleRepositoryContext, MonorepoContext},
    errors::VersionError,
};
use sublime_standard_tools::{
    filesystem::AsyncFileSystem,
    project::{ProjectDetector, ProjectKind},
    monorepo::{MonorepoDetector, MonorepoDetectorTrait},
    config::StandardConfig,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Context detector for automatically determining project type
///
/// The context detector analyzes the project structure to determine whether
/// it's a single repository or monorepo, and configures the appropriate
/// context settings.
///
/// ## Detection Methods
///
/// - **package.json analysis**: Checks for workspace configuration
/// - **Directory scanning**: Looks for multiple package.json files  
/// - **Tool detection**: Identifies monorepo tool configuration files
/// - **Heuristic analysis**: Uses patterns to classify ambiguous cases
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::context::ContextDetector;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let filesystem = ();
/// let detector = ContextDetector::new(filesystem);
/// 
/// // Auto-detect context
/// let context = detector.detect_context().await?;
/// 
/// // Force detection as monorepo
/// let monorepo_context = detector.detect_as_monorepo().await?;
/// 
/// // Force detection as single repository
/// let single_context = detector.detect_as_single().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ContextDetector<F: AsyncFileSystem> {
    /// Standard crate project detector for unified project detection
    project_detector: ProjectDetector<F>,
    /// Standard crate monorepo detector for workspace detection  
    monorepo_detector: MonorepoDetector<F>,
    /// Current working directory for detection
    working_directory: PathBuf,
    /// Whether to use strict detection (require explicit workspace config)
    strict_mode: bool,
    /// Configuration for detection behavior
    config: Option<StandardConfig>,
}

impl<F> ContextDetector<F>
where
    F: AsyncFileSystem + Clone + 'static,
{
    /// Create a new context detector
    ///
    /// # Arguments
    ///
    /// * `filesystem` - Filesystem implementation for reading files
    ///
    /// # Returns
    ///
    /// A new context detector instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::ContextDetector;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// let fs = AsyncFileSystem::new();
    /// let detector = ContextDetector::new(fs);
    /// ```
    #[must_use]
    pub fn new(filesystem: F) -> Self {
        let project_detector = ProjectDetector::with_filesystem(filesystem.clone());
        let monorepo_detector = MonorepoDetector::with_filesystem(filesystem.clone());
        
        Self {
            project_detector,
            monorepo_detector,
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            strict_mode: false,
            config: None,
        }
    }

    /// Create a context detector with a specific working directory
    ///
    /// # Arguments
    ///
    /// * `filesystem` - Filesystem implementation for reading files
    /// * `working_directory` - Directory to use as the project root
    ///
    /// # Returns
    ///
    /// A new context detector instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::ContextDetector;
    /// use std::path::PathBuf;
    ///
    /// # let filesystem = ();
    /// let detector = ContextDetector::with_directory(
    ///     filesystem,
    ///     PathBuf::from("/path/to/project")
    /// );
    /// ```
    #[must_use]
    pub fn with_directory(filesystem: F, working_directory: PathBuf) -> Self {
        let project_detector = ProjectDetector::with_filesystem(filesystem.clone());
        let monorepo_detector = MonorepoDetector::with_filesystem(filesystem.clone());
        
        Self {
            project_detector,
            monorepo_detector,
            working_directory,
            strict_mode: false,
            config: None,
        }
    }

    /// Create a context detector with custom configuration
    ///
    /// # Arguments
    ///
    /// * `filesystem` - Filesystem implementation for reading files
    /// * `config` - Standard configuration for detection behavior
    ///
    /// # Returns
    ///
    /// A new context detector instance with configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::ContextDetector;
    /// use sublime_standard_tools::config::StandardConfig;
    ///
    /// # let filesystem = ();
    /// # let config = StandardConfig::default();
    /// let detector = ContextDetector::with_config(filesystem, config);
    /// ```
    #[must_use]
    pub fn with_config(filesystem: F, config: StandardConfig) -> Self {
        let project_detector = ProjectDetector::with_filesystem(filesystem.clone());
        let monorepo_detector = MonorepoDetector::with_filesystem(filesystem.clone());
        
        Self {
            project_detector,
            monorepo_detector,
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            strict_mode: false,
            config: Some(config),
        }
    }

    /// Enable strict mode detection
    ///
    /// In strict mode, the detector requires explicit workspace configuration
    /// to classify a project as a monorepo, rather than using heuristics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::ContextDetector;
    ///
    /// # let filesystem = ();
    /// let detector = ContextDetector::new(filesystem).with_strict_mode();
    /// ```
    #[must_use]
    pub fn with_strict_mode(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    /// Automatically detect the project context
    ///
    /// This is the main entry point for context detection. It analyzes the
    /// project structure and returns the appropriate context configuration.
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
    /// use sublime_package_tools::context::{ContextDetector, ProjectContext};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let filesystem = ();
    /// let detector = ContextDetector::new(filesystem);
    /// let context = detector.detect_context().await?;
    ///
    /// match context {
    ///     ProjectContext::Single(_) => println!("Single repository detected"),
    ///     ProjectContext::Monorepo(_) => println!("Monorepo detected"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_context(&self) -> Result<ProjectContext, VersionError> {
        // Use standard crate MonorepoDetector for robust detection
        let monorepo_result = self.monorepo_detector
            .is_monorepo_root(&self.working_directory)
            .await
            .map_err(|e| VersionError::IO(format!("Failed to detect monorepo: {e}")))?;

        match monorepo_result {
            Some(_monorepo_kind) => {
                // Detected as monorepo - build monorepo context
                Ok(ProjectContext::Monorepo(self.build_monorepo_context().await?))
            }
            None => {
                // In strict mode, ensure explicit workspace detection
                if self.strict_mode {
                    // Use project detector to verify it's a valid single repo
                    let project_kind = self.project_detector
                        .detect_kind(&self.working_directory)
                        .await
                        .map_err(|e| VersionError::IO(format!("Failed to detect project kind: {e}")))?;
                    
                    match project_kind {
                        ProjectKind::Repository(repo_kind) => {
                            if repo_kind.is_monorepo() {
                                Ok(ProjectContext::Monorepo(self.build_monorepo_context().await?))
                            } else {
                                Ok(ProjectContext::Single(self.build_single_context().await?))
                            }
                        }
                    }
                } else {
                    // Non-strict mode: default to single repository
                    Ok(ProjectContext::Single(self.build_single_context().await?))
                }
            }
        }
    }

    /// Force detection as a monorepo
    ///
    /// This method bypasses auto-detection and builds a monorepo context
    /// by scanning for workspace packages.
    ///
    /// # Returns
    ///
    /// A monorepo project context
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{ContextDetector, ProjectContext};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let filesystem = ();
    /// let detector = ContextDetector::new(filesystem);
    /// let context = detector.detect_as_monorepo().await?;
    ///
    /// if let ProjectContext::Monorepo(config) = context {
    ///     println!("Found {} workspace packages", config.workspace_packages.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_as_monorepo(&self) -> Result<ProjectContext, VersionError> {
        Ok(ProjectContext::Monorepo(self.build_monorepo_context().await?))
    }

    /// Force detection as a single repository
    ///
    /// This method bypasses auto-detection and builds a single repository context.
    ///
    /// # Returns
    ///
    /// A single repository project context
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{ContextDetector, ProjectContext};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let filesystem = ();
    /// let detector = ContextDetector::new(filesystem);
    /// let context = detector.detect_as_single().await?;
    ///
    /// if let ProjectContext::Single(config) = context {
    ///     println!("Single repository context configured");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_as_single(&self) -> Result<ProjectContext, VersionError> {
        Ok(ProjectContext::Single(self.build_single_context().await?))
    }


    /// Build a monorepo context configuration
    async fn build_monorepo_context(&self) -> Result<MonorepoContext, VersionError> {
        // Use standard crate MonorepoDetector to get full monorepo analysis
        let monorepo_descriptor = self.monorepo_detector
            .detect_monorepo(&self.working_directory)
            .await
            .map_err(|e| VersionError::IO(format!("Failed to analyze monorepo: {e}")))?;
        
        // Convert MonorepoDescriptor to workspace_packages map
        let workspace_packages: HashMap<String, String> = monorepo_descriptor
            .packages()
            .iter()
            .map(|pkg| (pkg.name.clone(), pkg.location.to_string_lossy().to_string()))
            .collect();
        
        Ok(MonorepoContext {
            workspace_packages,
            ..MonorepoContext::default()
        })
    }

    /// Build a single repository context configuration
    async fn build_single_context(&self) -> Result<SingleRepositoryContext, VersionError> {
        Ok(SingleRepositoryContext::default())
    }

}

/// Detection result with additional metadata
///
/// Provides detailed information about the detection process
/// and confidence level of the detected context.
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// The detected project context
    pub context: ProjectContext,
    /// Confidence level of the detection (0.0 to 1.0)
    pub confidence: f64,
    /// Evidence that led to this detection
    pub evidence: Vec<DetectionEvidence>,
    /// Warnings generated during detection
    pub warnings: Vec<String>,
}

/// Evidence used in context detection
#[derive(Debug, Clone)]
pub struct DetectionEvidence {
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// Description of the evidence
    pub description: String,
    /// Weight of this evidence in the decision (0.0 to 1.0)
    pub weight: f64,
}

/// Types of evidence used in detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceType {
    /// Workspace configuration in package.json
    WorkspaceConfig,
    /// Monorepo tool configuration files
    MonorepoTools,
    /// Multiple package.json files found
    MultiplePackages,
    /// Directory structure patterns
    DirectoryStructure,
    /// Package naming patterns
    NamingPatterns,
}

impl DetectionResult {
    /// Check if the detection has high confidence
    ///
    /// # Returns
    ///
    /// `true` if confidence is above 0.8, `false` otherwise
    #[must_use]
    pub fn is_high_confidence(&self) -> bool {
        self.confidence > 0.8
    }

    /// Check if the detection has warnings
    ///
    /// # Returns
    ///
    /// `true` if there are warnings, `false` otherwise
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get the strongest evidence for this detection
    ///
    /// # Returns
    ///
    /// The evidence with the highest weight, if any
    #[must_use]
    pub fn strongest_evidence(&self) -> Option<&DetectionEvidence> {
        self.evidence.iter().max_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap())
    }
}