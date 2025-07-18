//! # Project Validation Types
//!
//! ## What
//! This module defines types for project validation status and related
//! validation operations.
//!
//! ## How
//! The `ProjectValidationStatus` enum represents different validation
//! states, with helper methods to check validation results.
//!
//! ## Why
//! Separate validation types enable comprehensive project validation
//! with clear status reporting and error handling.

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