//! # Project Type Definitions
//!
//! ## What
//! This file defines the core types and traits for representing Node.js projects
//! in a unified way, regardless of their structure (simple or monorepo).
//!
//! ## How
//! Types are defined as enums and traits that model project structures and their
//! characteristics. The `ProjectInfo` trait provides a common interface, while
//! `ProjectKind` and `ProjectDescriptor` handle type differentiation.
//!
//! ## Why
//! A well-defined type system ensures that all project types are represented
//! consistently and safely throughout the codebase, enabling uniform handling
//! of different project structures while maintaining type safety.

use crate::monorepo::{MonorepoDescriptor, MonorepoKind, PackageManager};
use package_json::PackageJson;
use std::path::{Path, PathBuf};

/// Represents the type of Node.js project.
///
/// This enum differentiates between simple repositories and various
/// types of monorepo structures, enabling type-specific processing
/// while maintaining a common interface.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ProjectKind;
/// use sublime_standard_tools::monorepo::MonorepoKind;
///
/// let simple = ProjectKind::Simple;
/// let yarn_mono = ProjectKind::Monorepo(MonorepoKind::YarnWorkspaces);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectKind {
    /// A simple Node.js project (single package.json)
    Simple,
    /// A monorepo project with specific type
    Monorepo(MonorepoKind),
}

impl ProjectKind {
    /// Returns a human-readable name for the project kind.
    ///
    /// # Returns
    ///
    /// A string describing the project type.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// assert_eq!(ProjectKind::Simple.name(), "simple");
    /// assert_eq!(ProjectKind::Monorepo(MonorepoKind::YarnWorkspaces).name(), "yarn monorepo");
    /// ```
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Simple => "simple".to_string(),
            Self::Monorepo(kind) => format!("{} monorepo", kind.name()),
        }
    }

    /// Checks if this is a monorepo project.
    ///
    /// # Returns
    ///
    /// `true` if the project is any type of monorepo, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// assert!(!ProjectKind::Simple.is_monorepo());
    /// assert!(ProjectKind::Monorepo(MonorepoKind::NpmWorkSpace).is_monorepo());
    /// ```
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        matches!(self, Self::Monorepo(_))
    }
}

/// Common interface for all Node.js project types.
///
/// This trait provides a unified API for accessing project information
/// regardless of whether it's a simple repository or a monorepo.
/// All project implementations must provide these basic capabilities.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ProjectInfo;
/// use std::path::Path;
///
/// fn print_project_info(project: &impl ProjectInfo) {
///     println!("Project root: {}", project.root().display());
///     println!("Project type: {}", project.kind().name());
///     
///     if let Some(pm) = project.package_manager() {
///         println!("Package manager: {:?}", pm.kind());
///     }
/// }
/// ```
pub trait ProjectInfo: Send + Sync {
    /// Returns the root directory of the project.
    ///
    /// # Returns
    ///
    /// A reference to the Path representing the project's root directory.
    fn root(&self) -> &Path;

    /// Returns the package manager for the project, if detected.
    ///
    /// # Returns
    ///
    /// * `Some(&PackageManager)` - If a package manager was detected
    /// * `None` - If no package manager was detected or detection was disabled
    fn package_manager(&self) -> Option<&PackageManager>;

    /// Returns the parsed package.json for the project, if available.
    ///
    /// # Returns
    ///
    /// * `Some(&PackageJson)` - If package.json was successfully parsed
    /// * `None` - If package.json was not found or could not be parsed
    fn package_json(&self) -> Option<&PackageJson>;

    /// Returns the validation status of the project.
    ///
    /// # Returns
    ///
    /// A reference to the `ProjectValidationStatus` indicating the validation state.
    fn validation_status(&self) -> &ProjectValidationStatus;

    /// Returns the kind of project.
    ///
    /// # Returns
    ///
    /// The `ProjectKind` enum value representing the type of project.
    fn kind(&self) -> ProjectKind;
}

/// Represents different types of Node.js projects with their specific data.
///
/// This enum serves as a container that can hold either a simple project
/// or a monorepo descriptor, providing type-safe access to project-specific
/// functionality while maintaining a common interface through the `ProjectInfo` trait.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::{ProjectDescriptor, ProjectInfo};
///
/// fn process_project(descriptor: ProjectDescriptor) {
///     match descriptor {
///         ProjectDescriptor::Simple(simple) => {
///             println!("Processing simple project at {}", simple.root().display());
///         }
///         ProjectDescriptor::Monorepo(monorepo) => {
///             println!("Processing {} with {} packages", 
///                      monorepo.kind().name(),
///                      monorepo.packages().len());
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub enum ProjectDescriptor {
    /// A simple Node.js project
    Simple(Box<super::SimpleProject>),
    /// A monorepo project
    Monorepo(Box<MonorepoDescriptor>),
}

impl ProjectDescriptor {
    /// Returns a reference to the project as a trait object.
    ///
    /// This method provides unified access to project information
    /// regardless of the underlying type.
    ///
    /// # Returns
    ///
    /// A reference to the project implementing the `ProjectInfo` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::{ProjectDescriptor, ProjectInfo};
    /// # fn example(descriptor: ProjectDescriptor) {
    /// let info = descriptor.as_project_info();
    /// println!("Project type: {}", info.kind().name());
    /// # }
    /// ```
    #[must_use]
    pub fn as_project_info(&self) -> &dyn ProjectInfo {
        match self {
            Self::Simple(project) => project.as_ref(),
            Self::Monorepo(monorepo) => monorepo.as_ref(),
        }
    }
}

/// Status of a project validation operation.
///
/// This enum represents the different states a project can be in
/// after validation, providing detailed information about any
/// issues found during the validation process.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ProjectValidationStatus;
///
/// let status = ProjectValidationStatus::Valid;
/// assert!(status.is_valid());
///
/// let warnings = ProjectValidationStatus::Warning(vec!["Missing LICENSE".to_string()]);
/// assert!(!warnings.is_valid());
/// assert!(warnings.has_warnings());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectValidationStatus {
    /// Project structure is valid
    Valid,
    /// Project has warnings but is usable
    Warning(Vec<String>),
    /// Project has errors that need to be fixed
    Error(Vec<String>),
    /// Project has not been validated
    NotValidated,
}

impl ProjectValidationStatus {
    /// Checks if the project validation passed without errors.
    ///
    /// # Returns
    ///
    /// `true` if the status is Valid, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectValidationStatus;
    ///
    /// assert!(ProjectValidationStatus::Valid.is_valid());
    /// assert!(!ProjectValidationStatus::Error(vec!["Missing package.json".to_string()]).is_valid());
    /// ```
    #[must_use]
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    /// Checks if the project has warnings.
    ///
    /// # Returns
    ///
    /// `true` if the status contains warnings, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectValidationStatus;
    ///
    /// let warnings = ProjectValidationStatus::Warning(vec!["Old dependencies".to_string()]);
    /// assert!(warnings.has_warnings());
    /// assert!(!ProjectValidationStatus::Valid.has_warnings());
    /// ```
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        matches!(self, Self::Warning(_))
    }

    /// Checks if the project has errors.
    ///
    /// # Returns
    ///
    /// `true` if the status contains errors, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectValidationStatus;
    ///
    /// let errors = ProjectValidationStatus::Error(vec!["Invalid package.json".to_string()]);
    /// assert!(errors.has_errors());
    /// assert!(!ProjectValidationStatus::Valid.has_errors());
    /// ```
    #[must_use]
    pub fn has_errors(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Gets the list of warnings if any.
    ///
    /// # Returns
    ///
    /// * `Some(&[String])` - If the status contains warnings
    /// * `None` - Otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectValidationStatus;
    ///
    /// let warnings = ProjectValidationStatus::Warning(vec!["Missing README".to_string()]);
    /// assert_eq!(warnings.warnings(), Some(&["Missing README".to_string()][..]));
    /// ```
    #[must_use]
    pub fn warnings(&self) -> Option<&[String]> {
        match self {
            Self::Warning(warnings) => Some(warnings),
            _ => None,
        }
    }

    /// Gets the list of errors if any.
    ///
    /// # Returns
    ///
    /// * `Some(&[String])` - If the status contains errors
    /// * `None` - Otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectValidationStatus;
    ///
    /// let errors = ProjectValidationStatus::Error(vec!["Missing package.json".to_string()]);
    /// assert_eq!(errors.errors(), Some(&["Missing package.json".to_string()][..]));
    /// ```
    #[must_use]
    pub fn errors(&self) -> Option<&[String]> {
        match self {
            Self::Error(errors) => Some(errors),
            _ => None,
        }
    }
}

/// Configuration options for project detection and validation.
///
/// This struct controls how projects are detected and validated,
/// allowing fine-grained control over the detection process.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ProjectConfig;
///
/// let config = ProjectConfig::new()
///     .with_detect_package_manager(true)
///     .with_validate_structure(true)
///     .with_detect_monorepo(true);
/// ```
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    /// Root directory for project detection (None uses current directory)
    pub(crate) root: Option<PathBuf>,
    /// Whether to detect the package manager
    pub(crate) detect_package_manager: bool,
    /// Whether to validate project structure
    pub(crate) validate_structure: bool,
    /// Whether to detect monorepo structures
    pub(crate) detect_monorepo: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectConfig {
    /// Creates a new `ProjectConfig` with default values.
    ///
    /// Default settings:
    /// - No specified root (uses current directory)
    /// - Package manager detection enabled
    /// - Structure validation enabled
    /// - Monorepo detection enabled
    ///
    /// # Returns
    ///
    /// A new `ProjectConfig` with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: None,
            detect_package_manager: true,
            validate_structure: true,
            detect_monorepo: true,
        }
    }

    /// Sets the root directory for project detection.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory to use
    ///
    /// # Returns
    ///
    /// Self with the root directory updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_root("/path/to/project");
    /// ```
    #[must_use]
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }

    /// Sets whether to detect package managers.
    ///
    /// # Arguments
    ///
    /// * `detect` - Whether to detect package managers
    ///
    /// # Returns
    ///
    /// Self with the package manager detection setting updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_detect_package_manager(false);
    /// ```
    #[must_use]
    pub fn with_detect_package_manager(mut self, detect: bool) -> Self {
        self.detect_package_manager = detect;
        self
    }

    /// Sets whether to validate project structure.
    ///
    /// # Arguments
    ///
    /// * `validate` - Whether to validate project structure
    ///
    /// # Returns
    ///
    /// Self with the validation setting updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_validate_structure(false);
    /// ```
    #[must_use]
    pub fn with_validate_structure(mut self, validate: bool) -> Self {
        self.validate_structure = validate;
        self
    }

    /// Sets whether to detect monorepo structures.
    ///
    /// # Arguments
    ///
    /// * `detect` - Whether to detect monorepo structures
    ///
    /// # Returns
    ///
    /// Self with the monorepo detection setting updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_detect_monorepo(false);
    /// ```
    #[must_use]
    pub fn with_detect_monorepo(mut self, detect: bool) -> Self {
        self.detect_monorepo = detect;
        self
    }
}