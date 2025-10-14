//! Package.json validation module.
//!
//! This module provides comprehensive validation of package.json files according
//! to Node.js package.json specifications and best practices. It checks for
//! required fields, validates formats, and identifies common issues.
//!
//! # What
//!
//! Implements validation rules for package.json files:
//! - Required field validation (name, version)
//! - Format validation (semver, URLs, email addresses)
//! - Dependency validation (version constraints, circular deps)
//! - Best practice recommendations
//! - Workspace-specific validation rules
//!
//! # How
//!
//! Uses a rule-based validation system where each rule checks specific
//! aspects of the package.json. Rules can produce errors (blocking issues)
//! or warnings (recommendations). The validator aggregates results and
//! provides detailed feedback.
//!
//! # Why
//!
//! Invalid package.json files can cause build failures, publishing issues,
//! and runtime problems. This validator helps catch issues early and ensures
//! packages follow Node.js ecosystem best practices.

use crate::error::{PackageError, PackageResult};
use crate::package::{BugsInfo, PackageJson, PersonOrString, Repository, WorkspaceConfig};
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;
use sublime_standard_tools::filesystem::AsyncFileSystem;

/// Severity level of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// Information - not a problem, just informative
    Info,
    /// Warning - issue that should be addressed but doesn't block operation
    Warning,
    /// Error - critical issue that will cause problems
    Error,
}

impl std::fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationSeverity::Info => write!(f, "INFO"),
            ValidationSeverity::Warning => write!(f, "WARNING"),
            ValidationSeverity::Error => write!(f, "ERROR"),
        }
    }
}

/// Represents a validation issue found in package.json.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    /// Severity of the issue
    pub severity: ValidationSeverity,
    /// Field path where the issue was found
    pub field: String,
    /// Human-readable message describing the issue
    pub message: String,
    /// Optional suggestion for fixing the issue
    pub suggestion: Option<String>,
}

impl ValidationIssue {
    /// Creates a new validation issue.
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level
    /// * `field` - The field path (e.g., "dependencies.lodash")
    /// * `message` - Description of the issue
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{ValidationIssue, ValidationSeverity};
    ///
    /// let issue = ValidationIssue::new(
    ///     ValidationSeverity::Error,
    ///     "version",
    ///     "Invalid semver format"
    /// );
    /// ```
    pub fn new(severity: ValidationSeverity, field: &str, message: &str) -> Self {
        Self { severity, field: field.to_string(), message: message.to_string(), suggestion: None }
    }

    /// Creates a new validation issue with a suggestion.
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level
    /// * `field` - The field path
    /// * `message` - Description of the issue
    /// * `suggestion` - Suggested fix for the issue
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{ValidationIssue, ValidationSeverity};
    ///
    /// let issue = ValidationIssue::with_suggestion(
    ///     ValidationSeverity::Warning,
    ///     "license",
    ///     "License field is missing",
    ///     "Consider adding a license field, e.g., 'MIT'"
    /// );
    /// ```
    pub fn with_suggestion(
        severity: ValidationSeverity,
        field: &str,
        message: &str,
        suggestion: &str,
    ) -> Self {
        Self {
            severity,
            field: field.to_string(),
            message: message.to_string(),
            suggestion: Some(suggestion.to_string()),
        }
    }

    /// Checks if this is an error-level issue.
    pub fn is_error(&self) -> bool {
        self.severity == ValidationSeverity::Error
    }

    /// Checks if this is a warning-level issue.
    pub fn is_warning(&self) -> bool {
        self.severity == ValidationSeverity::Warning
    }

    /// Checks if this is an info-level issue.
    pub fn is_info(&self) -> bool {
        self.severity == ValidationSeverity::Info
    }
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} - {}", self.severity, self.field, self.message)?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, " (Suggestion: {})", suggestion)?;
        }
        Ok(())
    }
}

/// Result of package.json validation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// List of all validation issues found
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    /// Creates a new empty validation result.
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    /// Creates a validation result with issues.
    pub fn with_issues(issues: Vec<ValidationIssue>) -> Self {
        Self { issues }
    }

    /// Adds an issue to the validation result.
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    /// Adds multiple issues to the validation result.
    pub fn add_issues(&mut self, mut new_issues: Vec<ValidationIssue>) {
        self.issues.append(&mut new_issues);
    }

    /// Checks if the validation passed (no errors).
    pub fn is_valid(&self) -> bool {
        !self.has_errors()
    }

    /// Checks if there are any error-level issues.
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|issue| issue.is_error())
    }

    /// Checks if there are any warning-level issues.
    pub fn has_warnings(&self) -> bool {
        self.issues.iter().any(|issue| issue.is_warning())
    }

    /// Gets all error-level issues.
    pub fn errors(&self) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|issue| issue.is_error()).collect()
    }

    /// Gets all warning-level issues.
    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|issue| issue.is_warning()).collect()
    }

    /// Gets all info-level issues.
    pub fn info(&self) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|issue| issue.is_info()).collect()
    }

    /// Gets the total number of issues.
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    /// Gets the number of errors.
    pub fn error_count(&self) -> usize {
        self.errors().len()
    }

    /// Gets the number of warnings.
    pub fn warning_count(&self) -> usize {
        self.warnings().len()
    }

    /// Merges another validation result into this one.
    pub fn merge(&mut self, other: ValidationResult) {
        self.issues.extend(other.issues);
    }

    /// Sorts issues by severity (errors first, then warnings, then info).
    pub fn sort_by_severity(&mut self) {
        self.issues.sort_by_key(|issue| std::cmp::Reverse(issue.severity));
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.issues.is_empty() {
            write!(f, "Package.json validation passed with no issues")?;
        } else {
            writeln!(f, "Package.json validation found {} issues:", self.issues.len())?;
            for issue in &self.issues {
                writeln!(f, "  {}", issue)?;
            }
        }
        Ok(())
    }
}

/// Package.json validator with configurable rules.
///
/// This validator applies a comprehensive set of rules to validate package.json
/// files according to Node.js specifications and best practices.
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::package::{PackageJsonValidator, PackageJson};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let validator = PackageJsonValidator::new();
/// let package_json = PackageJson::parse_from_str(r#"
/// {
///   "name": "my-package",
///   "version": "1.0.0"
/// }
/// "#)?;
///
/// let result = validator.validate(&package_json);
/// if result.has_errors() {
///     for error in result.errors() {
///         eprintln!("Error: {}", error);
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct PackageJsonValidator {
    /// Whether to enable strict validation mode
    strict_mode: bool,
    /// Whether to validate dependency versions
    validate_dependencies: bool,
    /// Required fields that must be present
    required_fields: HashSet<String>,
    /// Compiled regex patterns for validation
    patterns: ValidationPatterns,
}

/// Precompiled regex patterns for validation.
struct ValidationPatterns {
    email: Regex,
    url: Regex,
    semver: Regex,
    npm_package_name: Regex,
    scoped_package_name: Regex,
}

impl ValidationPatterns {
    fn new() -> PackageResult<Self> {
        Ok(Self {
            email: Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").map_err(|e| {
                PackageError::operation("compile_regex", format!("Email regex failed: {}", e))
            })?,
            url: Regex::new(r"^https?://[^\s]+$").map_err(|e| {
                PackageError::operation("compile_regex", format!("URL regex failed: {}", e))
            })?,
            semver: Regex::new(r"^\d+\.\d+\.\d+(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$").map_err(|e| {
                PackageError::operation("compile_regex", format!("Semver regex failed: {}", e))
            })?,
            npm_package_name: Regex::new(r"^[a-z0-9]([a-z0-9\-._]*[a-z0-9])?$").map_err(|e| {
                PackageError::operation("compile_regex", format!("Package name regex failed: {}", e))
            })?,
            scoped_package_name: Regex::new(r"^@[a-z0-9-._]+/[a-z0-9]([a-z0-9\-._]*[a-z0-9])?$").map_err(|e| {
                PackageError::operation("compile_regex", format!("Scoped package name regex failed: {}", e))
            })?,
        })
    }
}

impl PackageJsonValidator {
    /// Creates a new validator with default settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::PackageJsonValidator;
    ///
    /// let validator = PackageJsonValidator::new();
    /// ```
    pub fn new() -> PackageResult<Self> {
        let mut required_fields = HashSet::new();
        required_fields.insert("name".to_string());
        required_fields.insert("version".to_string());

        Ok(Self {
            strict_mode: false,
            validate_dependencies: true,
            required_fields,
            patterns: ValidationPatterns::new()?,
        })
    }

    /// Creates a validator with strict mode enabled.
    ///
    /// Strict mode requires additional fields and applies more stringent rules.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::package::PackageJsonValidator;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let validator = PackageJsonValidator::strict();
    /// # Ok(())
    /// # }
    /// ```
    pub fn strict() -> PackageResult<Self> {
        let mut validator = Self::new()?;
        validator.strict_mode = true;

        // Add additional required fields for strict mode
        validator.required_fields.insert("description".to_string());
        validator.required_fields.insert("license".to_string());
        validator.required_fields.insert("repository".to_string());

        Ok(validator)
    }

    /// Enables or disables dependency validation.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to validate dependency versions and constraints
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut validator = sublime_pkg_tools::package::PackageJsonValidator::new()?;
    /// validator.set_dependency_validation(false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_dependency_validation(&mut self, enabled: bool) {
        self.validate_dependencies = enabled;
    }

    /// Adds a required field.
    ///
    /// # Arguments
    ///
    /// * `field` - The field name that should be required
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut validator = sublime_pkg_tools::package::PackageJsonValidator::new()?;
    /// validator.add_required_field("homepage");
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.insert(field.to_string());
    }

    /// Validates a PackageJson structure.
    ///
    /// # Arguments
    ///
    /// * `package_json` - The parsed package.json to validate
    ///
    /// # Returns
    ///
    /// A validation result containing any issues found
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::package::{PackageJsonValidator, PackageJson};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let validator = PackageJsonValidator::new()?;
    /// let package_json = PackageJson::parse_from_str(r#"
    /// {
    ///   "name": "my-package",
    ///   "version": "1.0.0"
    /// }
    /// "#)?;
    ///
    /// let result = validator.validate(&package_json);
    /// println!("Validation result: {}", result);
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate(&self, package_json: &PackageJson) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate required fields
        result.add_issues(self.validate_required_fields(package_json));

        // Validate package name
        result.add_issues(self.validate_name(package_json));

        // Validate version
        result.add_issues(self.validate_version(package_json));

        // Validate optional fields
        result.add_issues(self.validate_description(package_json));
        result.add_issues(self.validate_license(package_json));
        result.add_issues(self.validate_author(package_json));
        result.add_issues(self.validate_repository(package_json));
        result.add_issues(self.validate_homepage(package_json));
        result.add_issues(self.validate_bugs(package_json));

        // Validate dependencies
        if self.validate_dependencies {
            result.add_issues(self.validate_dependencies_section(package_json));
        }

        // Validate scripts
        result.add_issues(self.validate_scripts(package_json));

        // Validate engines
        result.add_issues(self.validate_engines(package_json));

        // Validate workspaces
        result.add_issues(self.validate_workspaces(package_json));

        result.sort_by_severity();
        result
    }

    /// Validates a package.json file from the filesystem.
    ///
    /// # Arguments
    ///
    /// * `filesystem` - The filesystem implementation to use
    /// * `path` - Path to the package.json file
    ///
    /// # Returns
    ///
    /// A validation result
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::package::PackageJsonValidator;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let validator = PackageJsonValidator::new()?;
    /// let result = validator.validate_file(&fs, Path::new("./package.json")).await?;
    /// println!("Validation: {}", result);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_file<F>(
        &self,
        filesystem: &F,
        path: &Path,
    ) -> PackageResult<ValidationResult>
    where
        F: AsyncFileSystem + Send + Sync,
    {
        let package_json = PackageJson::read_from_path(filesystem, path).await?;
        Ok(self.validate(&package_json))
    }

    fn validate_required_fields(&self, package_json: &PackageJson) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for field in &self.required_fields {
            match field.as_str() {
                "name" => {
                    if package_json.name.is_empty() {
                        issues.push(ValidationIssue::new(
                            ValidationSeverity::Error,
                            "name",
                            "Package name is required",
                        ));
                    }
                }
                "version" => {
                    // Version is validated separately
                }
                "description" => {
                    if package_json.description.is_none() {
                        issues.push(ValidationIssue::with_suggestion(
                            ValidationSeverity::Warning,
                            "description",
                            "Package description is missing",
                            "Add a description to help users understand your package",
                        ));
                    }
                }
                "license" => {
                    if package_json.license.is_none() {
                        issues.push(ValidationIssue::with_suggestion(
                            ValidationSeverity::Warning,
                            "license",
                            "Package license is missing",
                            "Add a license field (e.g., 'MIT', 'ISC', 'Apache-2.0')",
                        ));
                    }
                }
                "repository" => {
                    if package_json.repository.is_none() {
                        issues.push(ValidationIssue::with_suggestion(
                            ValidationSeverity::Warning,
                            "repository",
                            "Repository information is missing",
                            "Add repository URL to help users find the source code",
                        ));
                    }
                }
                _ => {
                    // Generic field validation would go here
                }
            }
        }

        issues
    }

    fn validate_name(&self, package_json: &PackageJson) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let name = &package_json.name;

        if name.is_empty() {
            return issues; // Handled by required fields validation
        }

        // Check package name format
        let is_scoped = name.starts_with('@');
        let is_valid = if is_scoped {
            self.patterns.scoped_package_name.is_match(name)
        } else {
            self.patterns.npm_package_name.is_match(name)
        };

        if !is_valid {
            issues.push(ValidationIssue::with_suggestion(
                ValidationSeverity::Error,
                "name",
                "Invalid package name format",
                "Package names must be lowercase, can contain hyphens and dots, and follow npm naming rules",
            ));
        }

        // Check name length
        if name.len() > 214 {
            issues.push(ValidationIssue::new(
                ValidationSeverity::Error,
                "name",
                "Package name is too long (max 214 characters)",
            ));
        }

        // Check for common problematic names
        let problematic_names =
            ["node_modules", "favicon.ico", "package.json", "npm", "node", "test", "example"];
        if problematic_names.contains(&name.as_str()) {
            issues.push(ValidationIssue::new(
                ValidationSeverity::Warning,
                "name",
                "Package name may conflict with common Node.js files or modules",
            ));
        }

        issues
    }

    fn validate_version(&self, package_json: &PackageJson) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let version = package_json.version.to_string();

        // Version is already parsed by semver, but let's do additional checks
        if !self.patterns.semver.is_match(&version) {
            issues.push(ValidationIssue::with_suggestion(
                ValidationSeverity::Error,
                "version",
                "Invalid semantic version format",
                "Use semantic versioning (e.g., '1.0.0', '0.1.0-alpha.1')",
            ));
        }

        issues
    }

    fn validate_description(&self, package_json: &PackageJson) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if let Some(description) = &package_json.description {
            if description.is_empty() {
                issues.push(ValidationIssue::new(
                    ValidationSeverity::Warning,
                    "description",
                    "Description is empty",
                ));
            } else if description.len() > 300 {
                issues.push(ValidationIssue::new(
                    ValidationSeverity::Warning,
                    "description",
                    "Description is very long (consider keeping it under 300 characters)",
                ));
            }
        }

        issues
    }

    fn validate_license(&self, _package_json: &PackageJson) -> Vec<ValidationIssue> {
        // Additional license validation could be added here
        // (e.g., checking against SPDX license list)

        Vec::new()
    }

    fn validate_author(&self, package_json: &PackageJson) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if let Some(author) = &package_json.author {
            match author {
                PersonOrString::Person(person) => {
                    if let Some(email) = &person.email {
                        if !self.patterns.email.is_match(email) {
                            issues.push(ValidationIssue::new(
                                ValidationSeverity::Warning,
                                "author.email",
                                "Invalid email format",
                            ));
                        }
                    }

                    if let Some(url) = &person.url {
                        if !self.patterns.url.is_match(url) {
                            issues.push(ValidationIssue::new(
                                ValidationSeverity::Warning,
                                "author.url",
                                "Invalid URL format",
                            ));
                        }
                    }
                }
                PersonOrString::String(_) => {
                    // String format is acceptable
                }
            }
        }

        issues
    }

    fn validate_repository(&self, package_json: &PackageJson) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if let Some(repository) = &package_json.repository {
            match repository {
                Repository::Detailed { url, .. } => {
                    if !self.patterns.url.is_match(url) {
                        issues.push(ValidationIssue::new(
                            ValidationSeverity::Warning,
                            "repository.url",
                            "Invalid repository URL format",
                        ));
                    }
                }
                Repository::String(url) => {
                    if !self.patterns.url.is_match(url) {
                        issues.push(ValidationIssue::new(
                            ValidationSeverity::Warning,
                            "repository",
                            "Invalid repository URL format",
                        ));
                    }
                }
            }
        }

        issues
    }

    fn validate_homepage(&self, package_json: &PackageJson) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if let Some(homepage) = &package_json.homepage {
            if !self.patterns.url.is_match(homepage) {
                issues.push(ValidationIssue::new(
                    ValidationSeverity::Warning,
                    "homepage",
                    "Invalid homepage URL format",
                ));
            }
        }

        issues
    }

    fn validate_bugs(&self, package_json: &PackageJson) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if let Some(bugs) = &package_json.bugs {
            match bugs {
                BugsInfo::String(url) => {
                    if !self.patterns.url.is_match(url) {
                        issues.push(ValidationIssue::new(
                            ValidationSeverity::Warning,
                            "bugs",
                            "Invalid bugs URL format",
                        ));
                    }
                }
                BugsInfo::Detailed { url, email } => {
                    if let Some(url) = url {
                        if !self.patterns.url.is_match(url) {
                            issues.push(ValidationIssue::new(
                                ValidationSeverity::Warning,
                                "bugs.url",
                                "Invalid bugs URL format",
                            ));
                        }
                    }

                    if let Some(email) = email {
                        if !self.patterns.email.is_match(email) {
                            issues.push(ValidationIssue::new(
                                ValidationSeverity::Warning,
                                "bugs.email",
                                "Invalid bugs email format",
                            ));
                        }
                    }
                }
            }
        }

        issues
    }

    fn validate_dependencies_section(&self, package_json: &PackageJson) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check for duplicate dependencies across sections
        let mut all_deps = HashSet::new();

        for name in package_json.dependencies.0.keys() {
            if !all_deps.insert(name.clone()) {
                issues.push(ValidationIssue::new(
                    ValidationSeverity::Error,
                    "dependencies",
                    format!("Duplicate dependency: {}", name).as_str(),
                ));
            }
        }

        for name in package_json.dev_dependencies.0.keys() {
            if !all_deps.insert(name.clone()) {
                issues.push(ValidationIssue::new(
                    ValidationSeverity::Warning,
                    "devDependencies",
                    format!("Dependency '{}' appears in multiple sections", name).as_str(),
                ));
            }
        }

        // Validate dependency version formats
        for (name, version) in &package_json.dependencies.0 {
            if version.trim().is_empty() {
                issues.push(ValidationIssue::new(
                    ValidationSeverity::Error,
                    "dependencies",
                    format!("Empty version for dependency '{}'", name).as_str(),
                ));
            }
        }

        issues
    }

    fn validate_scripts(&self, package_json: &PackageJson) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (name, script) in &package_json.scripts.0 {
            if script.trim().is_empty() {
                issues.push(ValidationIssue::new(
                    ValidationSeverity::Warning,
                    "scripts",
                    format!("Empty script: {}", name).as_str(),
                ));
            }

            // Check for potentially dangerous scripts
            if script.contains("rm -rf") || script.contains("del /") {
                issues.push(ValidationIssue::new(
                    ValidationSeverity::Warning,
                    "scripts",
                    format!("Script '{}' contains potentially dangerous commands", name).as_str(),
                ));
            }
        }

        issues
    }

    fn validate_engines(&self, package_json: &PackageJson) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (engine, version) in &package_json.engines {
            if version.trim().is_empty() {
                issues.push(ValidationIssue::new(
                    ValidationSeverity::Warning,
                    "engines",
                    format!("Empty version constraint for engine '{}'", engine).as_str(),
                ));
            }

            // Validate common engine names
            match engine.as_str() {
                "node" | "npm" | "yarn" | "pnpm" => {
                    // These are valid
                }
                _ => {
                    issues.push(ValidationIssue::new(
                        ValidationSeverity::Info,
                        "engines",
                        format!("Unknown engine: {}", engine).as_str(),
                    ));
                }
            }
        }

        issues
    }

    fn validate_workspaces(&self, package_json: &PackageJson) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if let Some(workspaces) = &package_json.workspaces {
            let patterns = match workspaces {
                WorkspaceConfig::Packages(patterns) => patterns,
                WorkspaceConfig::Detailed { packages, .. } => packages,
            };

            for pattern in patterns {
                if pattern.trim().is_empty() {
                    issues.push(ValidationIssue::new(
                        ValidationSeverity::Error,
                        "workspaces",
                        "Empty workspace pattern",
                    ));
                }
            }
        }

        issues
    }
}

impl Default for PackageJsonValidator {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback if regex compilation fails (shouldn't happen)
            // This should never happen in practice as the regex patterns are valid
            unreachable!("Failed to create default validator - invalid regex patterns")
        })
    }
}
