//! Upgrade audit section for detecting and reporting available package upgrades.
//!
//! **What**: Provides functionality to audit external package dependencies for available
//! upgrades, categorize them by type (major, minor, patch), and identify deprecated packages.
//!
//! **How**: Uses the `UpgradeManager` to detect available upgrades, then analyzes the results
//! to create issues based on severity (deprecated packages are critical, major upgrades are
//! warnings, minor/patch are informational).
//!
//! **Why**: To provide actionable insights about dependency updates, helping teams stay
//! current with security patches and new features while identifying critical issues like
//! deprecated dependencies.

use crate::audit::issue::{AuditIssue, IssueCategory, IssueSeverity};
use crate::config::PackageToolsConfig;
use crate::error::{AuditError, AuditResult};
use crate::upgrade::{DependencyUpgrade, DetectionOptions, UpgradeManager, UpgradeType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Audit section containing upgrade analysis results.
///
/// Contains detailed information about available package upgrades, including
/// counts by upgrade type, deprecated packages, and generated audit issues.
///
/// # Examples
///
/// ## Accessing upgrade statistics
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::UpgradeAuditSection;
///
/// # fn example(section: UpgradeAuditSection) {
/// println!("Total upgrades available: {}", section.total_upgrades);
/// println!("Major upgrades: {}", section.major_upgrades);
/// println!("Minor upgrades: {}", section.minor_upgrades);
/// println!("Patch upgrades: {}", section.patch_upgrades);
/// println!("Deprecated packages: {}", section.deprecated_packages.len());
/// println!("Issues found: {}", section.issues.len());
/// # }
/// ```
///
/// ## Checking for critical issues
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::UpgradeAuditSection;
///
/// # fn example(section: UpgradeAuditSection) {
/// let critical_issues: Vec<_> = section.issues.iter()
///     .filter(|issue| issue.is_critical())
///     .collect();
///
/// if !critical_issues.is_empty() {
///     println!("Found {} critical upgrade issues", critical_issues.len());
///     for issue in critical_issues {
///         println!("  - {}: {}", issue.title, issue.description);
///     }
/// }
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeAuditSection {
    /// Total number of upgrades available.
    ///
    /// Sum of all major, minor, and patch upgrades across all packages.
    pub total_upgrades: usize,

    /// Number of major version upgrades available.
    ///
    /// Major upgrades may contain breaking changes according to semver.
    pub major_upgrades: usize,

    /// Number of minor version upgrades available.
    ///
    /// Minor upgrades add new features but should be backward compatible.
    pub minor_upgrades: usize,

    /// Number of patch version upgrades available.
    ///
    /// Patch upgrades contain bug fixes and should be backward compatible.
    pub patch_upgrades: usize,

    /// List of deprecated packages detected.
    ///
    /// Deprecated packages should be replaced as soon as possible.
    pub deprecated_packages: Vec<DeprecatedPackage>,

    /// Available upgrades grouped by package name.
    ///
    /// Maps from package name (from package.json) to list of upgrades
    /// available for that package's dependencies.
    pub upgrades_by_package: HashMap<String, Vec<DependencyUpgrade>>,

    /// List of audit issues generated from the upgrade analysis.
    ///
    /// Issues are created based on upgrade types and deprecated packages:
    /// - Deprecated packages generate Critical issues
    /// - Major upgrades generate Warning issues
    /// - Minor/Patch upgrades generate Info issues
    pub issues: Vec<AuditIssue>,
}

impl UpgradeAuditSection {
    /// Creates an empty upgrade audit section.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::UpgradeAuditSection;
    ///
    /// let section = UpgradeAuditSection::empty();
    /// assert_eq!(section.total_upgrades, 0);
    /// assert!(section.issues.is_empty());
    /// ```
    #[must_use]
    pub fn empty() -> Self {
        Self {
            total_upgrades: 0,
            major_upgrades: 0,
            minor_upgrades: 0,
            patch_upgrades: 0,
            deprecated_packages: Vec::new(),
            upgrades_by_package: HashMap::new(),
            issues: Vec::new(),
        }
    }

    /// Returns whether any upgrades were found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::UpgradeAuditSection;
    ///
    /// let empty = UpgradeAuditSection::empty();
    /// assert!(!empty.has_upgrades());
    /// ```
    #[must_use]
    pub fn has_upgrades(&self) -> bool {
        self.total_upgrades > 0
    }

    /// Returns whether any deprecated packages were found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::UpgradeAuditSection;
    ///
    /// let section = UpgradeAuditSection::empty();
    /// assert!(!section.has_deprecated_packages());
    /// ```
    #[must_use]
    pub fn has_deprecated_packages(&self) -> bool {
        !self.deprecated_packages.is_empty()
    }

    /// Returns the number of critical issues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::UpgradeAuditSection;
    ///
    /// let section = UpgradeAuditSection::empty();
    /// assert_eq!(section.critical_issue_count(), 0);
    /// ```
    #[must_use]
    pub fn critical_issue_count(&self) -> usize {
        self.issues.iter().filter(|issue| issue.is_critical()).count()
    }

    /// Returns the number of warning issues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::UpgradeAuditSection;
    ///
    /// let section = UpgradeAuditSection::empty();
    /// assert_eq!(section.warning_issue_count(), 0);
    /// ```
    #[must_use]
    pub fn warning_issue_count(&self) -> usize {
        self.issues.iter().filter(|issue| issue.is_warning()).count()
    }

    /// Returns the number of informational issues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::UpgradeAuditSection;
    ///
    /// let section = UpgradeAuditSection::empty();
    /// assert_eq!(section.info_issue_count(), 0);
    /// ```
    #[must_use]
    pub fn info_issue_count(&self) -> usize {
        self.issues.iter().filter(|issue| issue.is_info()).count()
    }

    /// Returns all upgrades for a specific package.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to get upgrades for
    ///
    /// # Returns
    ///
    /// A slice of upgrades for the package, or an empty slice if not found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::UpgradeAuditSection;
    ///
    /// let section = UpgradeAuditSection::empty();
    /// let upgrades = section.upgrades_for_package("my-app");
    /// assert!(upgrades.is_empty());
    /// ```
    #[must_use]
    pub fn upgrades_for_package(&self, package_name: &str) -> &[DependencyUpgrade] {
        self.upgrades_by_package.get(package_name).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

/// Information about a deprecated package.
///
/// Represents a package that has been marked as deprecated in the npm registry.
/// Deprecated packages should be replaced with alternatives as soon as possible.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::audit::DeprecatedPackage;
///
/// let deprecated = DeprecatedPackage {
///     name: "old-parser".to_string(),
///     current_version: "1.2.3".to_string(),
///     deprecation_message: "This package is no longer maintained".to_string(),
///     alternative: Some("new-parser".to_string()),
/// };
///
/// println!("Package '{}' is deprecated", deprecated.name);
/// if let Some(alt) = &deprecated.alternative {
///     println!("Consider using '{}' instead", alt);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeprecatedPackage {
    /// Name of the deprecated package.
    pub name: String,

    /// Current version installed/specified.
    pub current_version: String,

    /// Deprecation message from the registry.
    ///
    /// Usually explains why the package is deprecated and what to use instead.
    pub deprecation_message: String,

    /// Suggested alternative package (if available).
    ///
    /// Some deprecation messages include a recommended replacement package.
    pub alternative: Option<String>,
}

/// Performs an upgrade audit using the provided upgrade manager.
///
/// This function:
/// 1. Detects all available upgrades using the upgrade manager
/// 2. Categorizes upgrades by type (major, minor, patch)
/// 3. Identifies deprecated packages
/// 4. Generates audit issues based on severity:
///    - Deprecated packages → Critical
///    - Major upgrades → Warning
///    - Minor/Patch upgrades → Info
///
/// # Arguments
///
/// * `upgrade_manager` - The upgrade manager to use for detection
/// * `config` - Configuration specifying which upgrade types to include
///
/// # Returns
///
/// An `UpgradeAuditSection` containing all upgrade analysis results.
///
/// # Errors
///
/// Returns `AuditError` if:
/// - The upgrade audit section is disabled in configuration
/// - Upgrade detection fails (network issues, registry errors, etc.)
/// - Package analysis fails
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::{audit_upgrades, UpgradeAuditSection};
/// use sublime_pkg_tools::upgrade::UpgradeManager;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
/// let config = PackageToolsConfig::default();
///
/// let upgrade_manager = UpgradeManager::new(workspace_root, config.upgrade.clone()).await?;
///
/// let section = audit_upgrades(&upgrade_manager, &config).await?;
///
/// println!("Found {} total upgrades", section.total_upgrades);
/// println!("Critical issues: {}", section.critical_issue_count());
/// println!("Warnings: {}", section.warning_issue_count());
/// # Ok(())
/// # }
/// ```
pub async fn audit_upgrades(
    upgrade_manager: &UpgradeManager,
    config: &PackageToolsConfig,
) -> AuditResult<UpgradeAuditSection> {
    // Check if upgrades section is enabled
    if !config.audit.sections.upgrades {
        return Err(AuditError::SectionDisabled { section: "upgrades".to_string() });
    }

    // Build detection options based on audit configuration
    let options = build_detection_options(config);

    // Detect available upgrades
    let preview = upgrade_manager.detect_upgrades(options).await.map_err(|e| {
        AuditError::UpgradeDetectionFailed { reason: format!("Failed to detect upgrades: {}", e) }
    })?;

    // Initialize counters
    let mut major_count = 0;
    let mut minor_count = 0;
    let mut patch_count = 0;
    let mut deprecated_packages = Vec::new();
    let mut upgrades_by_package: HashMap<String, Vec<DependencyUpgrade>> = HashMap::new();
    let mut issues = Vec::new();

    // Process each package's upgrades
    for package_upgrades in preview.packages {
        let package_name = package_upgrades.package_name;
        let package_upgrades_list = package_upgrades.upgrades;

        if package_upgrades_list.is_empty() {
            continue;
        }

        // Store upgrades for this package
        upgrades_by_package.insert(package_name.clone(), package_upgrades_list.clone());

        // Process each upgrade
        for upgrade in &package_upgrades_list {
            // Count by type
            match upgrade.upgrade_type {
                UpgradeType::Major => major_count += 1,
                UpgradeType::Minor => minor_count += 1,
                UpgradeType::Patch => patch_count += 1,
            }

            // Check for deprecated packages
            if let Some(deprecation_msg) = &upgrade.version_info.deprecated {
                let deprecated = DeprecatedPackage {
                    name: upgrade.name.clone(),
                    current_version: upgrade.current_version.clone(),
                    deprecation_message: deprecation_msg.clone(),
                    alternative: extract_alternative(deprecation_msg),
                };

                deprecated_packages.push(deprecated.clone());

                // Create critical issue for deprecated package
                let mut issue = AuditIssue::new(
                    IssueSeverity::Critical,
                    IssueCategory::Upgrades,
                    format!("Deprecated package: {}", upgrade.name),
                    format!(
                        "Package '{}' (v{}) is deprecated. {}",
                        upgrade.name, upgrade.current_version, deprecation_msg
                    ),
                );
                issue.add_affected_package(package_name.clone());
                issue.add_metadata("package".to_string(), upgrade.name.clone());
                issue.add_metadata("current_version".to_string(), upgrade.current_version.clone());
                issue.add_metadata("deprecation_message".to_string(), deprecation_msg.clone());

                if let Some(alt) = &deprecated.alternative {
                    issue.set_suggestion(format!("Consider migrating to '{}'", alt));
                    issue.add_metadata("alternative".to_string(), alt.clone());
                }

                issues.push(issue);
            } else {
                // Create issue based on upgrade type
                let (severity, title, description, suggestion) = match upgrade.upgrade_type {
                    UpgradeType::Major => (
                        IssueSeverity::Warning,
                        format!("Major upgrade available: {}", upgrade.name),
                        format!(
                            "Package '{}' has a major version upgrade available ({} → {}). \
                             This may include breaking changes.",
                            upgrade.name, upgrade.current_version, upgrade.latest_version
                        ),
                        Some(
                            "Review the changelog for breaking changes before upgrading"
                                .to_string(),
                        ),
                    ),
                    UpgradeType::Minor => (
                        IssueSeverity::Info,
                        format!("Minor upgrade available: {}", upgrade.name),
                        format!(
                            "Package '{}' has a minor version upgrade available ({} → {}). \
                             This should be backward compatible.",
                            upgrade.name, upgrade.current_version, upgrade.latest_version
                        ),
                        Some("Consider upgrading to get new features".to_string()),
                    ),
                    UpgradeType::Patch => (
                        IssueSeverity::Info,
                        format!("Patch upgrade available: {}", upgrade.name),
                        format!(
                            "Package '{}' has a patch version upgrade available ({} → {}). \
                             This contains bug fixes.",
                            upgrade.name, upgrade.current_version, upgrade.latest_version
                        ),
                        Some("Consider upgrading to get bug fixes".to_string()),
                    ),
                };

                let mut issue =
                    AuditIssue::new(severity, IssueCategory::Upgrades, title, description);
                issue.add_affected_package(package_name.clone());
                issue.add_metadata("package".to_string(), upgrade.name.clone());
                issue.add_metadata("current_version".to_string(), upgrade.current_version.clone());
                issue.add_metadata("latest_version".to_string(), upgrade.latest_version.clone());
                issue.add_metadata(
                    "upgrade_type".to_string(),
                    format!("{:?}", upgrade.upgrade_type),
                );

                if let Some(sugg) = suggestion {
                    issue.set_suggestion(sugg);
                }

                issues.push(issue);
            }
        }
    }

    let total_upgrades = major_count + minor_count + patch_count;

    Ok(UpgradeAuditSection {
        total_upgrades,
        major_upgrades: major_count,
        minor_upgrades: minor_count,
        patch_upgrades: patch_count,
        deprecated_packages,
        upgrades_by_package,
        issues,
    })
}

/// Builds detection options from audit configuration.
///
/// Configures which dependency types to check based on the audit configuration.
fn build_detection_options(_config: &PackageToolsConfig) -> DetectionOptions {
    DetectionOptions {
        include_dependencies: true,
        include_dev_dependencies: true,
        include_peer_dependencies: true,
        include_optional_dependencies: true,
        package_filter: None,
        dependency_filter: None,
        include_prereleases: false,
        concurrency: 10,
    }
}

/// Attempts to extract an alternative package name from a deprecation message.
///
/// Looks for common patterns like "use X instead" or "migrate to X" in the
/// deprecation message.
///
/// # Arguments
///
/// * `message` - The deprecation message to parse
///
/// # Returns
///
/// The extracted alternative package name, if found.
pub(crate) fn extract_alternative(message: &str) -> Option<String> {
    let message_lower = message.to_lowercase();

    // Common patterns to look for
    let patterns = ["use ", "migrate to ", "replaced by ", "use the ", "switch to "];

    for pattern in &patterns {
        if let Some(start_idx) = message_lower.find(pattern) {
            let start = start_idx + pattern.len();
            let remaining = &message[start..];

            // Try to extract the package name (stop at space, comma, period, etc.)
            if let Some(name) = remaining
                .split(|c: char| c.is_whitespace() || c == ',' || c == '.' || c == '!')
                .next()
            {
                let trimmed = name.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    None
}
