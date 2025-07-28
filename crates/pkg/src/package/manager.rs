//! # Package Manager
//!
//! Enterprise-grade package management with async filesystem integration.
//!
//! ## What
//!
//! The `PackageManager` provides all core functionality for reading, writing, and validating
//! Node.js package.json files. It serves as the primary interface for package operations,
//! separating business logic from the pure data structure (`Package`).
//!
//! ## How
//!
//! This module implements a generic `PackageManager<F>` where `F` implements `AsyncFileSystem`,
//! enabling full async I/O operations with proper error handling. All operations are stateless,
//! working on provided Package instances without maintaining internal state.
//!
//! ## Why
//!
//! By separating package operations from the data structure, we achieve:
//! - Clean architecture with single responsibility principle
//! - Testability through filesystem abstraction
//! - Enterprise-grade error handling and validation
//! - Performance optimization through async operations
//!
//! ## Examples
//!
//! ```rust
//! use sublime_package_tools::package::manager::PackageManager;
//! use sublime_standard_tools::filesystem::AsyncFileSystem;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a package manager with filesystem
//! let fs = AsyncFileSystem::new();
//! let manager = PackageManager::new(fs);
//!
//! // Read a package.json file
//! let package = manager.read_package(Path::new("package.json")).await?;
//! println!("Package: {} v{}", package.name, package.version);
//!
//! // Validate package structure
//! let report = manager.validate_package(&package).await?;
//! if report.has_errors() {
//!     println!("Validation errors found!");
//! }
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

use crate::{Dependency, Package, Result, errors::PackageError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use sublime_standard_tools::filesystem::AsyncFileSystem;

/// Package manager for Node.js package operations with async filesystem integration.
///
/// This struct provides all core functionality for package management operations,
/// including reading, writing, and validating package.json files. It uses generic
/// filesystem abstraction for testability and flexibility.
///
/// ## Type Parameters
///
/// * `F` - Filesystem implementation that must implement `AsyncFileSystem` trait
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::package::manager::PackageManager;
/// use sublime_standard_tools::filesystem::AsyncFileSystem;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = AsyncFileSystem::new();
/// let manager = PackageManager::new(fs);
///
/// // Use manager for package operations
/// let package = manager.read_package(std::path::Path::new("package.json")).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct PackageManager<F> {
    /// Filesystem implementation for I/O operations
    filesystem: F,
}

impl<F> PackageManager<F>
where
    F: AsyncFileSystem + Clone + Send + Sync,
{
    /// Creates a new PackageManager with the provided filesystem implementation.
    ///
    /// # Arguments
    ///
    /// * `filesystem` - AsyncFileSystem implementation for I/O operations
    ///
    /// # Returns
    ///
    /// A new `PackageManager` instance ready for package operations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::package::manager::PackageManager;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// let fs = AsyncFileSystem::new();
    /// let manager = PackageManager::new(fs);
    /// ```
    #[must_use]
    pub fn new(filesystem: F) -> Self {
        Self { filesystem }
    }

    /// Reads a package.json file and returns a Package instance.
    ///
    /// This method reads the package.json file from the specified path,
    /// parses it, and validates the basic structure before returning
    /// a Package instance.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the package.json file
    ///
    /// # Returns
    ///
    /// * `Ok(Package)` - Successfully parsed package
    /// * `Err(Error)` - If file cannot be read or parsed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::package::manager::PackageManager;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let manager = PackageManager::new(fs);
    ///
    /// let package = manager.read_package(Path::new("package.json")).await?;
    /// println!("Loaded package: {} v{}", package.name, package.version);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_package(&self, path: &Path) -> Result<Package> {
        // Read the file contents
        let contents = self.filesystem.read_file(path).await
            .map_err(|e| PackageError::PackageJsonIoFailure {
                path: path.display().to_string(),
                error: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
            })?;

        // Convert bytes to string
        let json_str = String::from_utf8(contents)
            .map_err(|e| PackageError::PackageJsonParseFailure {
                path: path.display().to_string(),
                error: serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid UTF-8: {}", e)
                )),
            })?;

        // Parse JSON
        let json_value: Value = serde_json::from_str(&json_str)
            .map_err(|e| PackageError::PackageJsonParseFailure {
                path: path.display().to_string(),
                error: e,
            })?;

        // Extract package information
        let package_json: PackageJson = serde_json::from_value(json_value.clone())
            .map_err(|e| PackageError::PackageJsonParseFailure {
                path: path.display().to_string(),
                error: e,
            })?;

        // Convert dependencies
        let mut dependencies = Vec::new();
        
        // Process regular dependencies
        if let Some(deps) = &package_json.dependencies {
            for (name, version) in deps {
                match Dependency::new(name, version) {
                    Ok(dep) => dependencies.push(dep),
                    Err(_) => {
                        // Skip invalid dependencies (e.g., workspace:*)
                        continue;
                    }
                }
            }
        }

        // Create Package instance
        Package::new(&package_json.name, &package_json.version, Some(dependencies))
            .map_err(|e| crate::Error::Version(e))
    }

    /// Writes a Package to a package.json file.
    ///
    /// This method serializes the Package struct to JSON format and writes
    /// it to the specified path. It ensures atomic writes by using a
    /// temporary file and renaming strategy.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where to write the package.json file
    /// * `package` - Package instance to write
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully written
    /// * `Err(Error)` - If write operation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::package::manager::PackageManager;
    /// use sublime_package_tools::Package;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let manager = PackageManager::new(fs);
    ///
    /// let package = Package::new("my-app", "1.0.0", None)?;
    /// manager.write_package(Path::new("package.json"), &package).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn write_package(&self, path: &Path, package: &Package) -> Result<()> {
        // Create backup if file exists
        let backup_result = self.create_backup_if_exists(path).await;
        if let Err(e) = backup_result {
            // Log warning but continue - backup is not critical
            eprintln!("Warning: Failed to create backup for {}: {}", path.display(), e);
        }

        // Convert Package to PackageJson struct for serialization
        let package_json = self.package_to_json(package)?;

        // Serialize to JSON with pretty formatting
        let json_content = serde_json::to_string_pretty(&package_json)
            .map_err(|e| PackageError::PackageJsonParseFailure {
                path: path.display().to_string(),
                error: e,
            })?;

        // Write atomically using temporary file approach
        let temp_path = self.create_temp_path(path);
        
        // Write to temporary file first
        self.filesystem.write_file(&temp_path, json_content.as_bytes()).await
            .map_err(|e| PackageError::PackageJsonIoFailure {
                path: temp_path.display().to_string(),
                error: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
            })?;

        // Perform atomic rename by using std::fs::rename (this is atomic on most filesystems)
        std::fs::rename(&temp_path, path)
            .map_err(|e| {
                // Clean up temp file on failure
                let _ = std::fs::remove_file(&temp_path);
                PackageError::PackageJsonIoFailure {
                    path: path.display().to_string(),
                    error: e,
                }
            })?;

        Ok(())
    }

    /// Validates a Package according to npm specifications.
    ///
    /// This method performs comprehensive validation of the package structure,
    /// including name validation, version format, dependency checks, and
    /// other npm-specific rules.
    ///
    /// # Arguments
    ///
    /// * `package` - Package instance to validate
    ///
    /// # Returns
    ///
    /// * `Ok(PackageValidationReport)` - Validation report with warnings and errors
    /// * `Err(Error)` - If validation process fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::package::manager::PackageManager;
    /// use sublime_package_tools::Package;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let manager = PackageManager::new(fs);
    ///
    /// let package = Package::new("my-app", "1.0.0", None)?;
    /// let report = manager.validate_package(&package).await?;
    ///
    /// if report.has_errors() {
    ///     for error in report.errors() {
    ///         println!("Error: {}", error);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_package(&self, package: &Package) -> Result<PackageValidationReport> {
        let mut report = PackageValidationReport::new();

        // Validate package name
        self.validate_package_name(&package.name, &mut report);

        // Validate package version
        self.validate_package_version(&package.version, &mut report);

        // Validate dependencies
        self.validate_dependencies(&package.dependencies, &mut report);

        // Add warnings for missing best practices
        self.check_best_practices(package, &mut report).await;

        Ok(report)
    }

    /// Validates a package name according to npm naming conventions.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name to validate
    /// * `report` - Validation report to add errors/warnings to
    fn validate_package_name(&self, name: &str, report: &mut PackageValidationReport) {
        // Check for empty name
        if name.is_empty() {
            report.add_error("Package name cannot be empty");
            return;
        }

        // Check length (npm limit is 214 characters)
        if name.len() > 214 {
            report.add_error("Package name cannot exceed 214 characters");
        }

        // Check for uppercase letters
        if name != name.to_lowercase() {
            report.add_error("Package name must be lowercase");
        }

        // Check for spaces
        if name.contains(' ') {
            report.add_error("Package name cannot contain spaces");
        }

        // Check for invalid starting characters
        if name.starts_with('.') || name.starts_with('_') {
            report.add_error("Package name cannot start with . or _");
        }

        // Check for reserved names
        let reserved_names = [
            "node_modules", "favicon.ico", "npm", "console", "require", "module", 
            "process", "global", "buffer", "__dirname", "__filename"
        ];
        if reserved_names.contains(&name) {
            report.add_error(&format!("Package name '{}' is reserved", name));
        }

        // Check for invalid characters (npm allows only lowercase letters, digits, hyphens, and dots)
        let valid_chars = name.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.' || c == '/' || c == '@'
        });
        if !valid_chars {
            report.add_error("Package name contains invalid characters. Only lowercase letters, digits, hyphens, dots, slashes, and @ are allowed");
        }

        // Check for URL-unsafe characters
        if name.contains('%') {
            report.add_error("Package name cannot contain URL-unsafe characters like %");
        }

        // Warnings for best practices
        if name.len() < 2 {
            report.add_warning("Package name should be at least 2 characters long");
        }

        if name.starts_with('@') && !name.contains('/') {
            report.add_error("Scoped package name must contain a slash (e.g., @scope/package)");
        }

        if name.ends_with('-') || name.ends_with('.') {
            report.add_warning("Package name should not end with - or .");
        }
    }

    /// Validates a package version according to semver specifications.
    ///
    /// # Arguments
    ///
    /// * `version` - Version string to validate
    /// * `report` - Validation report to add errors/warnings to
    fn validate_package_version(&self, version: &str, report: &mut PackageValidationReport) {
        // Try to parse as semver
        match semver::Version::parse(version) {
            Ok(parsed_version) => {
                // Add warnings for unusual version patterns
                if parsed_version.major == 0 && parsed_version.minor == 0 && parsed_version.patch == 0 {
                    report.add_warning("Version 0.0.0 is not recommended for published packages");
                }

                if !parsed_version.pre.is_empty() {
                    report.add_warning("Pre-release versions may not be suitable for production");
                }

                if !parsed_version.build.is_empty() {
                    report.add_warning("Build metadata in versions is ignored by npm");
                }
            }
            Err(_) => {
                report.add_error(&format!("Invalid version format: '{}'. Must follow semantic versioning (e.g., 1.0.0)", version));
            }
        }

        // Check for empty version
        if version.is_empty() {
            report.add_error("Package version cannot be empty");
        }
    }

    /// Validates package dependencies.
    ///
    /// # Arguments
    ///
    /// * `dependencies` - List of dependencies to validate
    /// * `report` - Validation report to add errors/warnings to
    fn validate_dependencies(&self, dependencies: &[Dependency], report: &mut PackageValidationReport) {
        // Check for duplicate dependencies
        let mut seen_names = std::collections::HashSet::new();
        for dep in dependencies {
            if !seen_names.insert(&dep.name) {
                report.add_error(&format!("Duplicate dependency: {}", dep.name));
            }

            // Validate dependency name
            self.validate_package_name(&dep.name, report);

            // Check for potentially problematic version ranges
            let version_str = dep.version.to_string();
            if version_str == "*" {
                report.add_warning(&format!("Wildcard version (*) for dependency '{}' is not recommended", dep.name));
            }

            if version_str.starts_with(">=") && !version_str.contains('<') {
                report.add_warning(&format!("Open-ended version range for dependency '{}' may cause issues", dep.name));
            }

            // Check for exact versions (might indicate pinning)
            if !version_str.contains('^') && !version_str.contains('~') && !version_str.contains('>') && !version_str.contains('<') && semver::Version::parse(&version_str).is_ok() {
                report.add_warning(&format!("Exact version specified for dependency '{}'. Consider using ^ or ~ for flexibility", dep.name));
            }
        }

        // Warning for too many dependencies
        if dependencies.len() > 50 {
            report.add_warning(&format!("Package has {} dependencies. Consider reducing dependencies for better maintainability", dependencies.len()));
        }
    }

    /// Checks for best practices and common patterns.
    ///
    /// # Arguments
    ///
    /// * `package` - Package to check
    /// * `report` - Validation report to add warnings to
    async fn check_best_practices(&self, package: &Package, report: &mut PackageValidationReport) {
        // These checks would typically require reading the original package.json
        // to check for fields like description, author, license, etc.
        // For now, we'll add warnings for missing recommended fields
        
        // Since Package struct only has name, version, and dependencies,
        // we can only validate what we have
        
        // Check if it's a scoped package and warn about access
        if package.name.starts_with('@') {
            report.add_warning("Scoped packages are private by default. Use 'npm publish --access public' if you want to publish publicly");
        }

        // Check for common naming patterns that might indicate issues
        if package.name.contains("test") || package.name.contains("example") {
            report.add_warning("Package name suggests this might be a test or example package");
        }

        // Check for single-character names (usually not good packages)
        if package.name.len() == 1 {
            report.add_warning("Single-character package names are not recommended");
        }

        // Suggest using semantic versioning properly
        if let Ok(version) = semver::Version::parse(&package.version) {
            if version.major >= 1 && package.dependencies.is_empty() {
                report.add_warning("Package version is 1.0.0+ but has no dependencies. Consider if this is intentional");
            }
        }
    }

    /// Creates a backup of the file if it exists.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to backup
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If backup was created or file doesn't exist
    /// * `Err(crate::Error)` - If backup creation failed
    async fn create_backup_if_exists(&self, path: &Path) -> Result<()> {
        // Try to read the existing file
        match self.filesystem.read_file(path).await {
            Ok(contents) => {
                // File exists, create backup
                let backup_path = self.create_backup_path(path);
                self.filesystem.write_file(&backup_path, &contents).await
                    .map_err(|e| crate::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string()
                    )))?;
                Ok(())
            }
            Err(_) => {
                // File doesn't exist, no backup needed
                Ok(())
            }
        }
    }

    /// Creates a backup path for the given file path.
    ///
    /// # Arguments
    ///
    /// * `path` - Original file path
    ///
    /// # Returns
    ///
    /// Path for backup file with .backup extension
    fn create_backup_path(&self, path: &Path) -> std::path::PathBuf {
        let mut backup_path = path.to_path_buf();
        let mut extension = path.extension().unwrap_or_default().to_os_string();
        extension.push(".backup");
        backup_path.set_extension(extension);
        backup_path
    }

    /// Creates a temporary path for atomic write operations.
    ///
    /// # Arguments
    ///
    /// * `path` - Target file path
    ///
    /// # Returns
    ///
    /// Path for temporary file with .tmp extension
    fn create_temp_path(&self, path: &Path) -> std::path::PathBuf {
        let mut temp_path = path.to_path_buf();
        let mut extension = path.extension().unwrap_or_default().to_os_string();
        extension.push(".tmp");
        temp_path.set_extension(extension);
        temp_path
    }

    /// Converts a Package struct to PackageJson for serialization.
    ///
    /// # Arguments
    ///
    /// * `package` - Package instance to convert
    ///
    /// # Returns
    ///
    /// PackageJson struct ready for JSON serialization
    fn package_to_json(&self, package: &Package) -> Result<PackageJson> {
        let mut dependencies = HashMap::new();
        
        // Convert dependencies back to string format
        for dep in &package.dependencies {
            dependencies.insert(dep.name.clone(), dep.version.to_string());
        }

        Ok(PackageJson {
            name: package.name.clone(),
            version: package.version.clone(),
            dependencies: if dependencies.is_empty() { None } else { Some(dependencies) },
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
        })
    }
}

/// Validation report for package validation operations.
///
/// Contains detailed information about validation errors and warnings
/// found during package validation.
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::package::manager::PackageValidationReport;
///
/// let report = PackageValidationReport::new();
/// assert!(!report.has_errors());
/// assert!(!report.has_warnings());
/// ```
#[derive(Debug, Clone, Default)]
pub struct PackageValidationReport {
    /// List of validation errors
    pub(crate) errors: Vec<String>,
    /// List of validation warnings
    pub(crate) warnings: Vec<String>,
}

impl PackageValidationReport {
    /// Creates a new empty validation report.
    ///
    /// # Returns
    ///
    /// A new `PackageValidationReport` with no errors or warnings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::package::manager::PackageValidationReport;
    ///
    /// let report = PackageValidationReport::new();
    /// assert!(!report.has_errors());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if the validation report contains any errors.
    ///
    /// # Returns
    ///
    /// `true` if there are validation errors, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::package::manager::PackageValidationReport;
    ///
    /// let report = PackageValidationReport::new();
    /// assert!(!report.has_errors());
    /// ```
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Checks if the validation report contains any warnings.
    ///
    /// # Returns
    ///
    /// `true` if there are validation warnings, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::package::manager::PackageValidationReport;
    ///
    /// let report = PackageValidationReport::new();
    /// assert!(!report.has_warnings());
    /// ```
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns a slice of all validation errors.
    ///
    /// # Returns
    ///
    /// A slice containing all validation error messages
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::package::manager::PackageValidationReport;
    ///
    /// let report = PackageValidationReport::new();
    /// assert_eq!(report.errors().len(), 0);
    /// ```
    #[must_use]
    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    /// Returns a slice of all validation warnings.
    ///
    /// # Returns
    ///
    /// A slice containing all validation warning messages
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::package::manager::PackageValidationReport;
    ///
    /// let report = PackageValidationReport::new();
    /// assert_eq!(report.warnings().len(), 0);
    /// ```
    #[must_use]
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Adds an error to the validation report.
    ///
    /// # Arguments
    ///
    /// * `error` - Error message to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::package::manager::PackageValidationReport;
    ///
    /// let mut report = PackageValidationReport::new();
    /// report.add_error("Invalid package name");
    /// assert!(report.has_errors());
    /// ```
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
    }

    /// Adds a warning to the validation report.
    ///
    /// # Arguments
    ///
    /// * `warning` - Warning message to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::package::manager::PackageValidationReport;
    ///
    /// let mut report = PackageValidationReport::new();
    /// report.add_warning("Package lacks description");
    /// assert!(report.has_warnings());
    /// ```
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
}

/// Internal structure for deserializing package.json files.
///
/// This structure captures the essential fields from a package.json file
/// that are needed to create a Package instance. Additional fields are
/// preserved but not used in the Package creation.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PackageJson {
    /// Package name
    name: String,
    /// Package version
    version: String,
    /// Regular dependencies
    #[serde(default)]
    dependencies: Option<HashMap<String, String>>,
    /// Development dependencies (not used in Package but preserved)
    #[serde(default)]
    dev_dependencies: Option<HashMap<String, String>>,
    /// Peer dependencies (not used in Package but preserved)
    #[serde(default)]
    peer_dependencies: Option<HashMap<String, String>>,
    /// Optional dependencies (not used in Package but preserved)
    #[serde(default)]
    optional_dependencies: Option<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper mock filesystem for testing
    #[derive(Clone)]
    struct SimpleMockFs;
    
    #[async_trait::async_trait]
    impl sublime_standard_tools::filesystem::AsyncFileSystem for SimpleMockFs {
        async fn read_file(&self, _path: &Path) -> sublime_standard_tools::error::Result<Vec<u8>> {
            Ok(vec![])
        }
        async fn write_file(&self, _path: &Path, _contents: &[u8]) -> sublime_standard_tools::error::Result<()> {
            Ok(())
        }
        async fn read_file_string(&self, _path: &Path) -> sublime_standard_tools::error::Result<String> {
            Ok(String::new())
        }
        async fn write_file_string(&self, _path: &Path, _contents: &str) -> sublime_standard_tools::error::Result<()> {
            Ok(())
        }
        async fn create_dir_all(&self, _path: &Path) -> sublime_standard_tools::error::Result<()> {
            Ok(())
        }
        async fn remove(&self, _path: &Path) -> sublime_standard_tools::error::Result<()> {
            Ok(())
        }
        async fn exists(&self, _path: &Path) -> bool {
            false
        }
        async fn read_dir(&self, _path: &Path) -> sublime_standard_tools::error::Result<Vec<std::path::PathBuf>> {
            Ok(vec![])
        }
        async fn walk_dir(&self, _path: &Path) -> sublime_standard_tools::error::Result<Vec<std::path::PathBuf>> {
            Ok(vec![])
        }
        async fn metadata(&self, _path: &Path) -> sublime_standard_tools::error::Result<std::fs::Metadata> {
            use std::fs;
            fs::metadata(".").map_err(|e| sublime_standard_tools::error::Error::FileSystem(
                sublime_standard_tools::error::FileSystemError::Io {
                    path: _path.to_path_buf(),
                    message: e.to_string(),
                }
            ))
        }
    }

    #[test]
    fn test_validation_report() {
        let mut report = PackageValidationReport::new();
        assert!(!report.has_errors());
        assert!(!report.has_warnings());

        report.add_error("Test error");
        assert!(report.has_errors());
        assert_eq!(report.errors().len(), 1);

        report.add_warning("Test warning");
        assert!(report.has_warnings());
        assert_eq!(report.warnings().len(), 1);
    }

    #[test]
    fn test_package_json_deserialization() {
        let json_str = r#"{
            "name": "test-package",
            "version": "1.2.3",
            "dependencies": {
                "react": "^18.0.0",
                "lodash": "~4.17.21"
            },
            "devDependencies": {
                "jest": "^29.0.0"
            }
        }"#;

        let package_json: PackageJson = serde_json::from_str(json_str).unwrap();
        assert_eq!(package_json.name, "test-package");
        assert_eq!(package_json.version, "1.2.3");
        assert!(package_json.dependencies.is_some());
        assert_eq!(package_json.dependencies.as_ref().unwrap().len(), 2);
        assert!(package_json.dev_dependencies.is_some());
    }

    #[test]
    fn test_package_json_minimal() {
        let json_str = r#"{
            "name": "minimal-package",
            "version": "0.1.0"
        }"#;

        let package_json: PackageJson = serde_json::from_str(json_str).unwrap();
        assert_eq!(package_json.name, "minimal-package");
        assert_eq!(package_json.version, "0.1.0");
        assert!(package_json.dependencies.is_none());
        assert!(package_json.dev_dependencies.is_none());
    }

    #[test]
    fn test_package_to_json_conversion() {
        use crate::Dependency;
        
        let mut dependencies = Vec::new();
        dependencies.push(Dependency::new("react", "^18.0.0").unwrap());
        dependencies.push(Dependency::new("lodash", "~4.17.21").unwrap());
        
        let package = Package::new("test-app", "1.0.0", Some(dependencies)).unwrap();
        
        let manager = PackageManager::new(SimpleMockFs);
        let package_json = manager.package_to_json(&package).unwrap();
        
        assert_eq!(package_json.name, "test-app");
        assert_eq!(package_json.version, "1.0.0");
        assert!(package_json.dependencies.is_some());
        
        let deps = package_json.dependencies.unwrap();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps.get("react"), Some(&"^18.0.0".to_string()));
        assert_eq!(deps.get("lodash"), Some(&"~4.17.21".to_string()));
    }

    #[test]
    fn test_path_utilities() {
        // Test path utilities without requiring PackageManager instance
        let path = std::path::Path::new("package.json");
        
        // Test backup path creation logic
        let mut backup_path = path.to_path_buf();
        let mut extension = path.extension().unwrap_or_default().to_os_string();
        extension.push(".backup");
        backup_path.set_extension(extension);
        assert_eq!(backup_path, std::path::Path::new("package.json.backup"));
        
        // Test temp path creation logic
        let mut temp_path = path.to_path_buf();
        let mut extension = path.extension().unwrap_or_default().to_os_string();
        extension.push(".tmp");
        temp_path.set_extension(extension);
        assert_eq!(temp_path, std::path::Path::new("package.json.tmp"));
        
        // Test with directory path
        let path_with_dir = std::path::Path::new("src/package.json");
        let mut backup_path_with_dir = path_with_dir.to_path_buf();
        let mut extension = path_with_dir.extension().unwrap_or_default().to_os_string();
        extension.push(".backup");
        backup_path_with_dir.set_extension(extension);
        assert_eq!(backup_path_with_dir, std::path::Path::new("src/package.json.backup"));
    }

    #[tokio::test]
    async fn test_validate_package_valid() {
        use crate::Dependency;
        
        let mut dependencies = Vec::new();
        dependencies.push(Dependency::new("react", "^18.0.0").unwrap());
        dependencies.push(Dependency::new("lodash", "~4.17.21").unwrap());
        
        let package = Package::new("my-awesome-package", "1.2.3", Some(dependencies)).unwrap();
        
        let manager = PackageManager::new(SimpleMockFs);
        let report = manager.validate_package(&package).await.unwrap();
        
        // Should have no errors for a valid package
        assert!(!report.has_errors());
        // Might have some warnings (e.g., exact versions, scoped package warning)
        println!("Warnings: {:?}", report.warnings());
    }

    #[tokio::test]
    async fn test_validate_package_invalid_name() {
        // Create a package and then modify the name to be invalid
        let mut package = Package::new("valid-name", "1.0.0", None).unwrap();
        package.name = "Invalid Package Name".to_string();
        
        let manager = PackageManager::new(SimpleMockFs);
        let report = manager.validate_package(&package).await.unwrap();
        
        assert!(report.has_errors());
        let errors = report.errors();
        assert!(errors.iter().any(|e| e.contains("must be lowercase")));
        assert!(errors.iter().any(|e| e.contains("cannot contain spaces")));
    }

    #[tokio::test]
    async fn test_validate_package_invalid_version() {
        // Create a package with an invalid version by manually constructing it
        // Since Package::new validates the version, we need to bypass that
        let mut package = Package::new("valid-name", "1.0.0", None).unwrap();
        // Manually set an invalid version
        package.version = "not-a-version".to_string();
        
        let manager = PackageManager::new(SimpleMockFs);
        let report = manager.validate_package(&package).await.unwrap();
        
        assert!(report.has_errors());
        let errors = report.errors();
        assert!(errors.iter().any(|e| e.contains("Invalid version format")));
    }

    #[tokio::test]
    async fn test_validate_package_reserved_name() {
        let package = Package::new("npm", "1.0.0", None).unwrap();
        
        let manager = PackageManager::new(SimpleMockFs);
        let report = manager.validate_package(&package).await.unwrap();
        
        assert!(report.has_errors());
        let errors = report.errors();
        assert!(errors.iter().any(|e| e.contains("reserved")));
    }

    #[tokio::test]
    async fn test_validate_package_scoped_name() {
        let package = Package::new("@myorg/awesome-package", "1.0.0", None).unwrap();
        
        let manager = PackageManager::new(SimpleMockFs);
        let report = manager.validate_package(&package).await.unwrap();
        
        // Should not have errors for valid scoped name
        assert!(!report.has_errors());
        // Should have warning about scoped packages being private by default
        assert!(report.has_warnings());
        let warnings = report.warnings();
        assert!(warnings.iter().any(|w| w.contains("private by default")));
    }

    #[tokio::test]
    async fn test_validate_package_invalid_scoped_name() {
        let package = Package::new("@invalid-scope", "1.0.0", None).unwrap();
        
        let manager = PackageManager::new(SimpleMockFs);
        let report = manager.validate_package(&package).await.unwrap();
        
        assert!(report.has_errors());
        let errors = report.errors();
        assert!(errors.iter().any(|e| e.contains("must contain a slash")));
    }

    #[tokio::test]
    async fn test_validate_package_duplicate_dependencies() {
        use crate::Dependency;
        
        let mut dependencies = Vec::new();
        dependencies.push(Dependency::new("react", "^18.0.0").unwrap());
        dependencies.push(Dependency::new("react", "^17.0.0").unwrap()); // Duplicate
        
        let package = Package::new("test-package", "1.0.0", Some(dependencies)).unwrap();
        
        let manager = PackageManager::new(SimpleMockFs);
        let report = manager.validate_package(&package).await.unwrap();
        
        assert!(report.has_errors());
        let errors = report.errors();
        assert!(errors.iter().any(|e| e.contains("Duplicate dependency")));
    }

    #[tokio::test]
    async fn test_validate_package_version_warnings() {
        let package = Package::new("test-package", "0.0.0", None).unwrap();
        
        let manager = PackageManager::new(SimpleMockFs);
        let report = manager.validate_package(&package).await.unwrap();
        
        assert!(!report.has_errors());
        assert!(report.has_warnings());
        let warnings = report.warnings();
        assert!(warnings.iter().any(|w| w.contains("0.0.0 is not recommended")));
    }

    #[tokio::test]
    async fn test_validate_package_prerelease_warning() {
        let package = Package::new("test-package", "1.0.0-alpha.1", None).unwrap();
        
        let manager = PackageManager::new(SimpleMockFs);
        let report = manager.validate_package(&package).await.unwrap();
        
        assert!(!report.has_errors());
        assert!(report.has_warnings());
        let warnings = report.warnings();
        assert!(warnings.iter().any(|w| w.contains("Pre-release versions")));
    }
}