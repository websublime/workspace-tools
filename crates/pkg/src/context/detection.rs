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
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
pub struct ContextDetector<F> {
    /// Filesystem implementation for reading project files
    filesystem: F,
    /// Current working directory for detection
    working_directory: PathBuf,
    /// Whether to use strict detection (require explicit workspace config)
    strict_mode: bool,
}

impl<F> ContextDetector<F>
where
    F: Clone,
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
        Self {
            filesystem,
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            strict_mode: false,
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
        Self {
            filesystem,
            working_directory,
            strict_mode: false,
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
        // Step 1: Check for explicit workspace configuration
        if self.has_workspace_config().await? {
            return Ok(ProjectContext::Monorepo(self.build_monorepo_context().await?));
        }

        // Step 2: Check for monorepo tool configurations
        if self.has_monorepo_tools().await? {
            return Ok(ProjectContext::Monorepo(self.build_monorepo_context().await?));
        }

        // Step 3: Check for multiple packages (if not in strict mode)
        if !self.strict_mode && self.has_multiple_packages().await? {
            return Ok(ProjectContext::Monorepo(self.build_monorepo_context().await?));
        }

        // Default: Single repository
        Ok(ProjectContext::Single(self.build_single_context().await?))
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

    /// Check if the project has explicit workspace configuration
    async fn has_workspace_config(&self) -> Result<bool, VersionError> {
        let package_json_path = self.working_directory.join("package.json");
        
        // TODO: Implement actual file reading with self.filesystem
        // For now, simulate the check
        let _exists = true; // self.filesystem.exists(&package_json_path).await?;
        
        // TODO: Parse package.json and check for workspaces field
        // let content = self.filesystem.read_to_string(&package_json_path).await?;
        // let package_json: serde_json::Value = serde_json::from_str(&content)?;
        // Ok(package_json.get("workspaces").is_some())
        
        // Placeholder implementation
        Ok(false)
    }

    /// Check if the project has monorepo tool configurations
    async fn has_monorepo_tools(&self) -> Result<bool, VersionError> {
        let monorepo_files = &[
            "lerna.json",
            "nx.json", 
            "rush.json",
            "pnpm-workspace.yaml",
            "yarn.lock",
        ];

        for file in monorepo_files {
            let file_path = self.working_directory.join(file);
            // TODO: Check if file exists using self.filesystem
            // if self.filesystem.exists(&file_path).await? {
            //     return Ok(true);
            // }
            let _file_path = file_path; // Suppress unused variable warning
        }

        Ok(false) // Placeholder implementation
    }

    /// Check if the project has multiple package.json files
    async fn has_multiple_packages(&self) -> Result<bool, VersionError> {
        // TODO: Implement directory traversal to find package.json files
        // let package_files = self.find_package_files().await?;
        // Ok(package_files.len() > 1)
        
        Ok(false) // Placeholder implementation
    }

    /// Build a monorepo context configuration
    async fn build_monorepo_context(&self) -> Result<MonorepoContext, VersionError> {
        let workspace_packages = self.discover_workspace_packages().await?;
        
        Ok(MonorepoContext {
            workspace_packages,
            ..MonorepoContext::default()
        })
    }

    /// Build a single repository context configuration
    async fn build_single_context(&self) -> Result<SingleRepositoryContext, VersionError> {
        Ok(SingleRepositoryContext::default())
    }

    /// Discover workspace packages in a monorepo
    async fn discover_workspace_packages(&self) -> Result<HashMap<String, String>, VersionError> {
        let mut packages = HashMap::new();
        
        // TODO: Implement actual package discovery
        // This would involve:
        // 1. Reading workspace configuration from package.json
        // 2. Scanning directory patterns for package.json files
        // 3. Reading package names from each package.json
        // 4. Building a map of package name -> relative path
        
        // For now, return empty map
        Ok(packages)
    }

    /// Find all package.json files in the project
    async fn _find_package_files(&self) -> Result<Vec<PathBuf>, VersionError> {
        let mut packages = Vec::new();
        
        // TODO: Implement recursive directory traversal
        // This would use self.filesystem to:
        // 1. Recursively scan directories
        // 2. Find all package.json files
        // 3. Filter out node_modules and other excluded directories
        
        Ok(packages)
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