use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    error::VersionError,
    version::{
        range::VersionRange,
        versioning::{Version, VersionComparison},
    },
    PackageResult,
};

/// Advanced version parser with validation and analysis capabilities.
///
/// Provides comprehensive parsing of version strings, ranges, and constraints
/// with detailed validation and normalization. Supports various package manager
/// formats and conventions.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::version::VersionParser;
///
/// let parser = VersionParser::new();
/// let analysis = parser.parse_and_analyze("^1.2.3")?;
///
/// println!("Type: {:?}", analysis.version_type);
/// println!("Range: {}", analysis.range);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct VersionParser {
    /// Validation configuration
    config: VersionParserConfig,
}

/// Configuration for version parser behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionParserConfig {
    /// Allow pre-release versions
    pub allow_prerelease: bool,
    /// Allow build metadata
    pub allow_build_metadata: bool,
    /// Strict semver compliance
    pub strict_semver: bool,
    /// Maximum allowed major version
    pub max_major_version: Option<u64>,
    /// Allow leading 'v' prefix
    pub allow_v_prefix: bool,
    /// Normalize versions during parsing
    pub normalize_versions: bool,
}

/// Version analysis result containing parsed information and metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionAnalysis {
    /// Original input string
    pub original: String,
    /// Normalized version string
    pub normalized: String,
    /// Parsed version range
    pub range: VersionRange,
    /// Version type classification
    pub version_type: VersionType,
    /// Validation results
    pub validation: ValidationResult,
    /// Extracted metadata
    pub metadata: VersionMetadata,
}

/// Version type classification for analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VersionType {
    /// Exact version (1.2.3)
    Exact,
    /// Caret range (^1.2.3)
    Caret,
    /// Tilde range (~1.2.3)
    Tilde,
    /// Comparison range (>=1.2.3, <2.0.0)
    Comparison,
    /// Wildcard pattern (1.2.*)
    Wildcard,
    /// Range specification (1.0.0 - 2.0.0)
    Range,
    /// Any version (*)
    Any,
    /// Invalid format
    Invalid,
}

/// Validation result for parsed versions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    /// Whether the version is valid
    pub is_valid: bool,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Validation errors
    pub errors: Vec<String>,
    /// Suggested fixes
    pub suggestions: Vec<String>,
}

/// Metadata extracted from version strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionMetadata {
    /// Whether version has pre-release component
    pub has_prerelease: bool,
    /// Whether version has build metadata
    pub has_build_metadata: bool,
    /// Pre-release identifiers
    pub prerelease_parts: Vec<String>,
    /// Build metadata parts
    pub build_parts: Vec<String>,
    /// Detected package manager format
    pub package_manager_format: Option<PackageManagerFormat>,
    /// Version stability classification
    pub stability: VersionStability,
}

/// Package manager format detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageManagerFormat {
    /// NPM-style versioning
    Npm,
    /// Yarn-style versioning
    Yarn,
    /// PNPM-style versioning
    Pnpm,
    /// Semantic versioning standard
    Semver,
}

/// Version stability classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VersionStability {
    /// Stable release version
    Stable,
    /// Pre-release version
    PreRelease,
    /// Development/snapshot version
    Development,
    /// Experimental version
    Experimental,
}

impl Default for VersionParserConfig {
    fn default() -> Self {
        Self {
            allow_prerelease: true,
            allow_build_metadata: true,
            strict_semver: false,
            max_major_version: None,
            allow_v_prefix: true,
            normalize_versions: true,
        }
    }
}

impl VersionParser {
    /// Creates a new version parser with default configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionParser;
    ///
    /// let parser = VersionParser::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(VersionParserConfig::default())
    }

    /// Creates a version parser with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Parser configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionParser, VersionParserConfig};
    ///
    /// let config = VersionParserConfig {
    ///     strict_semver: true,
    ///     allow_prerelease: false,
    ///     ..Default::default()
    /// };
    /// let parser = VersionParser::with_config(config);
    /// ```
    #[must_use]
    pub fn with_config(config: VersionParserConfig) -> Self {
        Self { config }
    }

    /// Parses and analyzes a version string.
    ///
    /// # Arguments
    ///
    /// * `input` - Version string to parse and analyze
    ///
    /// # Returns
    ///
    /// Comprehensive analysis of the version string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionParser;
    ///
    /// let parser = VersionParser::new();
    /// let analysis = parser.parse_and_analyze("^1.2.3-alpha.1")?;
    ///
    /// assert!(analysis.metadata.has_prerelease);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse_and_analyze(&self, input: &str) -> PackageResult<VersionAnalysis> {
        let normalized = self.normalize_input(input);
        let version_type = self.classify_version_type(&normalized);

        // Try to parse the range, but handle failures gracefully for analysis
        let (range, validation) = match self.parse_range(&normalized) {
            Ok(range) => {
                let validation = self.validate_input(&normalized, &range);
                (range, validation)
            }
            Err(_) => {
                // If parsing fails, create a dummy range and mark as invalid
                let range = VersionRange::Any; // Placeholder
                let mut validation = ValidationResult {
                    is_valid: false,
                    warnings: Vec::new(),
                    errors: vec!["Failed to parse version range".to_string()],
                    suggestions: Vec::new(),
                };
                validation.errors.extend(self.suggest_corrections(&normalized).unwrap_or_default());
                (range, validation)
            }
        };

        let metadata = self.extract_metadata(&normalized, &range);

        Ok(VersionAnalysis {
            original: input.to_string(),
            normalized,
            range,
            version_type,
            validation,
            metadata,
        })
    }

    /// Parses a version range from a string.
    ///
    /// # Arguments
    ///
    /// * `input` - Version range string
    ///
    /// # Returns
    ///
    /// Parsed version range
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionParser;
    ///
    /// let parser = VersionParser::new();
    /// let range = parser.parse_range("^1.2.3")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse_range(&self, input: &str) -> PackageResult<VersionRange> {
        VersionRange::from_str(input).map_err(Into::into)
    }

    /// Parses an exact version from a string.
    ///
    /// # Arguments
    ///
    /// * `input` - Version string
    ///
    /// # Returns
    ///
    /// Parsed version
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionParser;
    ///
    /// let parser = VersionParser::new();
    /// let version = parser.parse_version("1.2.3-alpha.1")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse_version(&self, input: &str) -> PackageResult<Version> {
        let normalized = self.normalize_input(input);
        self.validate_version_format(&normalized)?;
        Version::from_str(&normalized).map_err(Into::into)
    }

    /// Validates multiple version strings and returns a summary.
    ///
    /// # Arguments
    ///
    /// * `versions` - Collection of version strings to validate
    ///
    /// # Returns
    ///
    /// Validation summary with statistics and issues
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionParser;
    ///
    /// let parser = VersionParser::new();
    /// let versions = vec!["1.2.3", "^1.2.3", "invalid"];
    /// let summary = parser.validate_multiple(&versions)?;
    ///
    /// println!("Valid: {}, Invalid: {}", summary.valid_count, summary.invalid_count);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn validate_multiple(&self, versions: &[&str]) -> PackageResult<ValidationSummary> {
        let mut valid_count = 0;
        let mut invalid_count = 0;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut issues = HashMap::new();

        for (index, version_str) in versions.iter().enumerate() {
            match self.parse_and_analyze(version_str) {
                Ok(analysis) => {
                    if analysis.validation.is_valid {
                        valid_count += 1;
                    } else {
                        invalid_count += 1;
                    }

                    if !analysis.validation.warnings.is_empty() {
                        warnings.extend(analysis.validation.warnings.clone());
                    }

                    if !analysis.validation.errors.is_empty() {
                        errors.extend(analysis.validation.errors.clone());
                        issues.insert(index, analysis.validation.errors);
                    }
                }
                Err(err) => {
                    invalid_count += 1;
                    errors.push(format!("Version {}: {}", index, err));
                    issues.insert(index, vec![err.to_string()]);
                }
            }
        }

        Ok(ValidationSummary {
            total_count: versions.len(),
            valid_count,
            invalid_count,
            warnings,
            errors,
            issues,
        })
    }

    /// Compares two version strings and returns detailed comparison.
    ///
    /// # Arguments
    ///
    /// * `left` - First version string
    /// * `right` - Second version string
    ///
    /// # Returns
    ///
    /// Detailed comparison result
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionParser;
    ///
    /// let parser = VersionParser::new();
    /// let comparison = parser.compare_versions("1.2.3", "1.2.4")?;
    ///
    /// println!("Result: {:?}", comparison);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn compare_versions(&self, left: &str, right: &str) -> PackageResult<VersionComparison> {
        let left_version = self.parse_version(left)?;
        let right_version = self.parse_version(right)?;

        Ok(left_version.compare(&right_version))
    }

    /// Suggests corrections for invalid version strings.
    ///
    /// # Arguments
    ///
    /// * `input` - Invalid version string
    ///
    /// # Returns
    ///
    /// List of suggested corrections
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionParser;
    ///
    /// let parser = VersionParser::new();
    /// let suggestions = parser.suggest_corrections("1.2")?;
    ///
    /// assert!(!suggestions.is_empty());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn suggest_corrections(&self, input: &str) -> PackageResult<Vec<String>> {
        let mut suggestions = Vec::new();
        let normalized = self.normalize_input(input);

        // Common correction patterns
        if let Some(suggestion) = self.suggest_missing_patch(&normalized) {
            suggestions.push(suggestion);
        }

        if let Some(suggestion) = self.suggest_remove_prefix(&normalized) {
            suggestions.push(suggestion);
        }

        if let Some(suggestion) = self.suggest_fix_prerelease(&normalized) {
            suggestions.push(suggestion);
        }

        // If no specific suggestions, try generic fixes
        if suggestions.is_empty() {
            suggestions.extend(self.generate_generic_suggestions(&normalized));
        }

        Ok(suggestions)
    }

    /// Normalizes input version string according to configuration.
    fn normalize_input(&self, input: &str) -> String {
        let mut normalized = input.trim().to_string();

        if self.config.normalize_versions {
            // Remove leading 'v' if allowed
            if self.config.allow_v_prefix && normalized.starts_with('v') {
                normalized = normalized[1..].to_string();
            }

            // Normalize whitespace
            normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
        }

        normalized
    }

    /// Classifies the type of version string.
    fn classify_version_type(&self, input: &str) -> VersionType {
        if input == "*" {
            return VersionType::Any;
        }

        if input.contains(" - ") {
            return VersionType::Range;
        }

        if input.contains('*') {
            return VersionType::Wildcard;
        }

        if input.starts_with('^') {
            return VersionType::Caret;
        }

        if input.starts_with('~') {
            return VersionType::Tilde;
        }

        if input.starts_with(">=")
            || input.starts_with("<=")
            || input.starts_with('>')
            || input.starts_with('<')
        {
            return VersionType::Comparison;
        }

        // Try to parse as exact version
        if Version::from_str(input).is_ok() {
            VersionType::Exact
        } else {
            VersionType::Invalid
        }
    }

    /// Validates input string and range.
    fn validate_input(&self, input: &str, range: &VersionRange) -> ValidationResult {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut suggestions = Vec::new();

        // Check for common issues
        if input.is_empty() {
            errors.push("Version string cannot be empty".to_string());
            suggestions.push("Provide a valid version string like '1.0.0'".to_string());
        }

        // Validate against configuration
        if let Err(validation_errors) = self.validate_against_config(input, range) {
            errors.extend(validation_errors);
        }

        // Check for potential issues
        warnings.extend(self.check_for_warnings(input, range));

        let is_valid = errors.is_empty();

        ValidationResult { is_valid, warnings, errors, suggestions }
    }

    /// Extracts metadata from version string and range.
    fn extract_metadata(&self, input: &str, _range: &VersionRange) -> VersionMetadata {
        let has_prerelease = input.contains('-') && !input.contains(" - ");
        let has_build_metadata = input.contains('+');

        let prerelease_parts =
            if has_prerelease { self.extract_prerelease_parts(input) } else { Vec::new() };

        let build_parts =
            if has_build_metadata { self.extract_build_parts(input) } else { Vec::new() };

        let package_manager_format = self.detect_package_manager_format(input);
        let stability = self.classify_stability(input, &prerelease_parts);

        VersionMetadata {
            has_prerelease,
            has_build_metadata,
            prerelease_parts,
            build_parts,
            package_manager_format,
            stability,
        }
    }

    /// Validates version format against semver rules.
    fn validate_version_format(&self, input: &str) -> PackageResult<()> {
        // Use simple validation by trying to parse with semver crate
        if Version::from_str(input).is_err() {
            return Err(VersionError::InvalidFormat {
                version: input.to_string(),
                reason: "Does not match semantic versioning format".to_string(),
            }
            .into());
        }

        Ok(())
    }

    /// Validates against parser configuration.
    fn validate_against_config(
        &self,
        input: &str,
        range: &VersionRange,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.config.strict_semver {
            if let Err(err) = self.validate_version_format(input) {
                errors.push(format!("Strict semver validation failed: {}", err));
            }
        }

        if !self.config.allow_prerelease && input.contains('-') && !input.contains(" - ") {
            errors.push("Pre-release versions are not allowed".to_string());
        }

        if !self.config.allow_build_metadata && input.contains('+') {
            errors.push("Build metadata is not allowed".to_string());
        }

        if let Some(max_major) = self.config.max_major_version {
            if let Some(min_version) = range.min_version() {
                if min_version.major() > max_major {
                    errors.push(format!(
                        "Major version {} exceeds maximum {}",
                        min_version.major(),
                        max_major
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Checks for potential warnings.
    fn check_for_warnings(&self, input: &str, _range: &VersionRange) -> Vec<String> {
        let mut warnings = Vec::new();

        if input.starts_with('v') && !self.config.allow_v_prefix {
            warnings.push("Version string starts with 'v' which may not be supported".to_string());
        }

        if input.contains("  ") {
            warnings.push("Version string contains multiple spaces".to_string());
        }

        warnings
    }

    /// Extracts pre-release parts from version string.
    fn extract_prerelease_parts(&self, input: &str) -> Vec<String> {
        if let Some(dash_pos) = input.find('-') {
            let prerelease_part = &input[dash_pos + 1..];

            // Remove build metadata if present
            let prerelease_clean = if let Some(plus_pos) = prerelease_part.find('+') {
                &prerelease_part[..plus_pos]
            } else {
                prerelease_part
            };

            prerelease_clean.split('.').map(String::from).collect()
        } else {
            Vec::new()
        }
    }

    /// Extracts build metadata parts from version string.
    fn extract_build_parts(&self, input: &str) -> Vec<String> {
        if let Some(plus_pos) = input.find('+') {
            let build_part = &input[plus_pos + 1..];
            build_part.split('.').map(String::from).collect()
        } else {
            Vec::new()
        }
    }

    /// Detects package manager format from version string.
    fn detect_package_manager_format(&self, _input: &str) -> Option<PackageManagerFormat> {
        // For now, default to NPM format
        // Could be enhanced to detect specific formats based on patterns
        Some(PackageManagerFormat::Npm)
    }

    /// Classifies version stability.
    fn classify_stability(&self, _input: &str, prerelease_parts: &[String]) -> VersionStability {
        if prerelease_parts.is_empty() {
            return VersionStability::Stable;
        }

        // Check for common pre-release identifiers
        for part in prerelease_parts {
            let part_lower = part.to_lowercase();
            match part_lower.as_str() {
                "alpha" | "a" => return VersionStability::Experimental,
                "beta" | "b" => return VersionStability::PreRelease,
                "rc" | "release-candidate" => return VersionStability::PreRelease,
                "dev" | "snapshot" => return VersionStability::Development,
                _ => {}
            }
        }

        VersionStability::PreRelease
    }

    /// Suggests adding missing patch version.
    fn suggest_missing_patch(&self, input: &str) -> Option<String> {
        let parts: Vec<&str> = input.split('.').collect();
        if parts.len() == 2 && parts[0].parse::<u64>().is_ok() && parts[1].parse::<u64>().is_ok() {
            return Some(format!("{}.0", input));
        }
        None
    }

    /// Suggests removing invalid prefix.
    fn suggest_remove_prefix(&self, input: &str) -> Option<String> {
        if input.starts_with('v') && input.len() > 1 {
            let without_v = &input[1..];
            if Version::from_str(without_v).is_ok() {
                return Some(without_v.to_string());
            }
        }

        // Also try removing other common prefixes
        for prefix in ["version", "ver", "release", "rel"] {
            if input.to_lowercase().starts_with(prefix) && input.len() > prefix.len() {
                let without_prefix = &input[prefix.len()..].trim_start_matches(['-', '_', ' ']);
                if Version::from_str(without_prefix).is_ok() {
                    return Some(without_prefix.to_string());
                }
            }
        }

        None
    }

    /// Suggests fixing pre-release format.
    fn suggest_fix_prerelease(&self, input: &str) -> Option<String> {
        // Look for common pre-release format issues
        if input.contains("_") {
            let fixed = input.replace('_', "-");
            if Version::from_str(&fixed).is_ok() {
                return Some(fixed);
            }
        }
        None
    }

    /// Generates generic suggestions for invalid versions.
    fn generate_generic_suggestions(&self, input: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Try common version patterns
        suggestions.push("1.0.0".to_string());
        suggestions.push("0.1.0".to_string());

        // If input looks like it might be a partial version
        if input.chars().all(|c| c.is_ascii_digit() || c == '.') {
            let parts: Vec<&str> = input.split('.').collect();
            match parts.len() {
                1 => suggestions.push(format!("{}.0.0", parts[0])),
                2 => suggestions.push(format!("{}.0", input)),
                _ => {}
            }
        }

        suggestions
    }
}

impl Default for VersionParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of validation results for multiple versions.
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    /// Total number of versions validated
    pub total_count: usize,
    /// Number of valid versions
    pub valid_count: usize,
    /// Number of invalid versions
    pub invalid_count: usize,
    /// All validation warnings
    pub warnings: Vec<String>,
    /// All validation errors
    pub errors: Vec<String>,
    /// Issues mapped by version index
    pub issues: HashMap<usize, Vec<String>>,
}

impl ValidationSummary {
    /// Gets the validation success rate as a percentage.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionParser, ValidationSummary};
    /// use std::collections::HashMap;
    ///
    /// let summary = ValidationSummary {
    ///     total_count: 10,
    ///     valid_count: 8,
    ///     invalid_count: 2,
    ///     warnings: Vec::new(),
    ///     errors: Vec::new(),
    ///     issues: HashMap::new(),
    /// };
    ///
    /// assert_eq!(summary.success_rate(), 80.0);
    /// ```
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            (self.valid_count as f64 / self.total_count as f64) * 100.0
        }
    }

    /// Checks if all versions are valid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::ValidationSummary;
    /// use std::collections::HashMap;
    ///
    /// let summary = ValidationSummary {
    ///     total_count: 5,
    ///     valid_count: 5,
    ///     invalid_count: 0,
    ///     warnings: Vec::new(),
    ///     errors: Vec::new(),
    ///     issues: HashMap::new(),
    /// };
    ///
    /// assert!(summary.all_valid());
    /// ```
    #[must_use]
    pub fn all_valid(&self) -> bool {
        self.invalid_count == 0 && self.total_count > 0
    }

    /// Gets issues for a specific version index.
    ///
    /// # Arguments
    ///
    /// * `index` - Version index to get issues for
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::ValidationSummary;
    /// use std::collections::HashMap;
    ///
    /// let mut issues = HashMap::new();
    /// issues.insert(0, vec!["Invalid format".to_string()]);
    ///
    /// let summary = ValidationSummary {
    ///     total_count: 1,
    ///     valid_count: 0,
    ///     invalid_count: 1,
    ///     warnings: Vec::new(),
    ///     errors: Vec::new(),
    ///     issues,
    /// };
    ///
    /// assert_eq!(summary.issues_for(0), Some(&vec!["Invalid format".to_string()]));
    /// ```
    #[must_use]
    pub fn issues_for(&self, index: usize) -> Option<&Vec<String>> {
        self.issues.get(&index)
    }
}
