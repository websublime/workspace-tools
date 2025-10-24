//! Version consistency audit section for analyzing internal dependency version alignment.
//!
//! **What**: Provides functionality to audit version consistency of internal (workspace)
//! dependencies, detecting cases where the same internal package is depended upon with
//! different version specifications across the workspace.
//!
//! **How**: Analyzes all packages in the workspace and their internal dependencies,
//! tracking which version specifications are used for each internal package. Identifies
//! inconsistencies where multiple different version specs are declared for the same
//! internal package, and recommends the most appropriate consistent version.
//!
//! **Why**: To ensure internal dependency versions are consistent across the workspace,
//! which helps prevent version conflicts, simplifies dependency management, and ensures
//! all packages work with compatible versions of internal dependencies.

use crate::audit::issue::{AuditIssue, IssueCategory, IssueSeverity};
use crate::audit::sections::dependencies::VersionUsage;
use crate::config::PackageToolsConfig;
use crate::error::{AuditError, AuditResult};
use crate::types::PackageInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Audit section containing version consistency analysis results.
///
/// Contains detailed information about version inconsistencies found across
/// internal dependencies in the workspace, with recommendations for resolution.
///
/// # Examples
///
/// ## Accessing consistency statistics
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::VersionConsistencyAuditSection;
///
/// # fn example(section: VersionConsistencyAuditSection) {
/// println!("Inconsistencies found: {}", section.inconsistencies.len());
/// println!("Issues found: {}", section.issues.len());
///
/// for inconsistency in &section.inconsistencies {
///     println!("Package {} has {} different versions used",
///         inconsistency.package_name,
///         inconsistency.versions_used.len());
/// }
/// # }
/// ```
///
/// ## Checking for critical issues
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::VersionConsistencyAuditSection;
///
/// # fn example(section: VersionConsistencyAuditSection) {
/// let critical_issues: Vec<_> = section.issues.iter()
///     .filter(|issue| issue.is_critical())
///     .collect();
///
/// if !critical_issues.is_empty() {
///     println!("Found {} critical version consistency issues", critical_issues.len());
/// }
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConsistencyAuditSection {
    /// List of version inconsistencies detected across internal dependencies.
    ///
    /// Each inconsistency represents an internal package that is depended upon
    /// with different version specifications across the workspace.
    pub inconsistencies: Vec<VersionInconsistency>,

    /// List of audit issues generated from the consistency analysis.
    ///
    /// Issues are created based on configuration:
    /// - Critical issues if `fail_on_inconsistency` is enabled
    /// - Warning issues if `warn_on_inconsistency` is enabled
    pub issues: Vec<AuditIssue>,
}

impl VersionConsistencyAuditSection {
    /// Creates an empty version consistency audit section.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::VersionConsistencyAuditSection;
    ///
    /// let section = VersionConsistencyAuditSection::empty();
    /// assert_eq!(section.inconsistencies.len(), 0);
    /// assert_eq!(section.issues.len(), 0);
    /// ```
    #[must_use]
    pub fn empty() -> Self {
        Self { inconsistencies: Vec::new(), issues: Vec::new() }
    }

    /// Returns whether any inconsistencies were found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::VersionConsistencyAuditSection;
    ///
    /// let section = VersionConsistencyAuditSection::empty();
    /// assert!(!section.has_inconsistencies());
    /// ```
    #[must_use]
    pub fn has_inconsistencies(&self) -> bool {
        !self.inconsistencies.is_empty()
    }

    /// Returns the number of critical issues found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::VersionConsistencyAuditSection;
    ///
    /// let section = VersionConsistencyAuditSection::empty();
    /// assert_eq!(section.critical_issue_count(), 0);
    /// ```
    #[must_use]
    pub fn critical_issue_count(&self) -> usize {
        self.issues.iter().filter(|issue| issue.is_critical()).count()
    }

    /// Returns the number of warning issues found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::VersionConsistencyAuditSection;
    ///
    /// let section = VersionConsistencyAuditSection::empty();
    /// assert_eq!(section.warning_issue_count(), 0);
    /// ```
    #[must_use]
    pub fn warning_issue_count(&self) -> usize {
        self.issues.iter().filter(|issue| issue.is_warning()).count()
    }

    /// Returns the number of informational issues found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::VersionConsistencyAuditSection;
    ///
    /// let section = VersionConsistencyAuditSection::empty();
    /// assert_eq!(section.info_issue_count(), 0);
    /// ```
    #[must_use]
    pub fn info_issue_count(&self) -> usize {
        self.issues.iter().filter(|issue| issue.is_info()).count()
    }

    /// Returns inconsistencies for a specific internal package.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the internal package to find inconsistencies for
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::audit::VersionConsistencyAuditSection;
    ///
    /// # fn example(section: VersionConsistencyAuditSection) {
    /// if let Some(inconsistency) = section.inconsistency_for_package("@myorg/core") {
    ///     println!("Found inconsistency for @myorg/core");
    ///     println!("Recommended version: {}", inconsistency.recommended_version);
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn inconsistency_for_package(&self, package_name: &str) -> Option<&VersionInconsistency> {
        self.inconsistencies.iter().find(|i| i.package_name == package_name)
    }
}

/// Represents a version inconsistency for an internal package.
///
/// Contains information about different version specifications used across
/// the workspace for a single internal package, along with a recommended
/// version for consistency.
///
/// # Examples
///
/// ## Creating an inconsistency manually
///
/// ```rust
/// use sublime_pkg_tools::audit::{VersionInconsistency, VersionUsage};
///
/// let inconsistency = VersionInconsistency {
///     package_name: "@myorg/core".to_string(),
///     versions_used: vec![
///         VersionUsage {
///             package_name: "app-a".to_string(),
///             version_spec: "^1.0.0".to_string(),
///         },
///         VersionUsage {
///             package_name: "app-b".to_string(),
///             version_spec: "^1.1.0".to_string(),
///         },
///     ],
///     recommended_version: "^1.1.0".to_string(),
/// };
///
/// assert_eq!(inconsistency.version_count(), 2);
/// ```
///
/// ## Describing the inconsistency
///
/// ```rust
/// use sublime_pkg_tools::audit::{VersionInconsistency, VersionUsage};
///
/// let inconsistency = VersionInconsistency {
///     package_name: "@myorg/utils".to_string(),
///     versions_used: vec![
///         VersionUsage {
///             package_name: "pkg-a".to_string(),
///             version_spec: "workspace:*".to_string(),
///         },
///         VersionUsage {
///             package_name: "pkg-b".to_string(),
///             version_spec: "^2.0.0".to_string(),
///         },
///     ],
///     recommended_version: "workspace:*".to_string(),
/// };
///
/// let description = inconsistency.describe();
/// assert!(description.contains("@myorg/utils"));
/// assert!(description.contains("pkg-a"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionInconsistency {
    /// Name of the internal package with inconsistent versions.
    ///
    /// This is the package that is being depended upon with different
    /// version specifications across the workspace.
    pub package_name: String,

    /// List of version specifications used across different packages.
    ///
    /// Each entry represents a package and the version spec it uses
    /// for this internal dependency.
    pub versions_used: Vec<VersionUsage>,

    /// Recommended version specification for consistency.
    ///
    /// This is typically:
    /// - "workspace:*" if any package uses it (preferred for monorepos)
    /// - The most recent semver version if specified versions are used
    /// - The most commonly used version if no clear preference
    pub recommended_version: String,
}

impl VersionInconsistency {
    /// Returns the number of different version specifications.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{VersionInconsistency, VersionUsage};
    ///
    /// let inconsistency = VersionInconsistency {
    ///     package_name: "@myorg/core".to_string(),
    ///     versions_used: vec![
    ///         VersionUsage {
    ///             package_name: "app-a".to_string(),
    ///             version_spec: "^1.0.0".to_string(),
    ///         },
    ///         VersionUsage {
    ///             package_name: "app-b".to_string(),
    ///             version_spec: "^1.1.0".to_string(),
    ///         },
    ///     ],
    ///     recommended_version: "^1.1.0".to_string(),
    /// };
    ///
    /// assert_eq!(inconsistency.version_count(), 2);
    /// ```
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions_used.len()
    }

    /// Returns a human-readable description of the inconsistency.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{VersionInconsistency, VersionUsage};
    ///
    /// let inconsistency = VersionInconsistency {
    ///     package_name: "@myorg/utils".to_string(),
    ///     versions_used: vec![
    ///         VersionUsage {
    ///             package_name: "pkg-a".to_string(),
    ///             version_spec: "^1.0.0".to_string(),
    ///         },
    ///     ],
    ///     recommended_version: "^1.0.0".to_string(),
    /// };
    ///
    /// let description = inconsistency.describe();
    /// assert!(description.contains("@myorg/utils"));
    /// ```
    #[must_use]
    pub fn describe(&self) -> String {
        let version_details: Vec<String> = self
            .versions_used
            .iter()
            .map(|v| format!("{} ({})", v.package_name, v.version_spec))
            .collect();

        format!(
            "Package '{}' is used with {} different versions: {}",
            self.package_name,
            self.version_count(),
            version_details.join(", ")
        )
    }

    /// Returns the unique version specifications used.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{VersionInconsistency, VersionUsage};
    ///
    /// let inconsistency = VersionInconsistency {
    ///     package_name: "@myorg/core".to_string(),
    ///     versions_used: vec![
    ///         VersionUsage {
    ///             package_name: "app-a".to_string(),
    ///             version_spec: "^1.0.0".to_string(),
    ///         },
    ///         VersionUsage {
    ///             package_name: "app-b".to_string(),
    ///             version_spec: "^1.0.0".to_string(),
    ///         },
    ///         VersionUsage {
    ///             package_name: "app-c".to_string(),
    ///             version_spec: "^1.1.0".to_string(),
    ///         },
    ///     ],
    ///     recommended_version: "^1.1.0".to_string(),
    /// };
    ///
    /// let unique_versions = inconsistency.unique_versions();
    /// assert_eq!(unique_versions.len(), 2);
    /// assert!(unique_versions.contains(&"^1.0.0".to_string()));
    /// assert!(unique_versions.contains(&"^1.1.0".to_string()));
    /// ```
    #[must_use]
    pub fn unique_versions(&self) -> Vec<String> {
        let mut versions: Vec<String> =
            self.versions_used.iter().map(|v| v.version_spec.clone()).collect();
        versions.sort();
        versions.dedup();
        versions
    }
}

/// Audits version consistency of internal dependencies across the workspace.
///
/// This function analyzes all internal dependencies across packages in the workspace
/// and identifies cases where the same internal package is depended upon with different
/// version specifications. It generates issues based on configuration settings.
///
/// # Arguments
///
/// * `packages` - List of all packages in the workspace
/// * `internal_package_names` - Set of internal package names to check for consistency
/// * `config` - Configuration controlling issue severity
///
/// # Returns
///
/// Returns a `VersionConsistencyAuditSection` containing detected inconsistencies and
/// generated audit issues.
///
/// # Errors
///
/// This function is currently infallible and returns `Ok` in all cases. The error
/// return type is maintained for API consistency with other audit functions.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::audit_version_consistency;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use std::collections::HashSet;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let packages = vec![/* discovered packages */];
/// let internal_names: HashSet<String> = packages.iter()
///     .map(|p| p.name().to_string())
///     .collect();
/// let config = PackageToolsConfig::default();
///
/// let section = audit_version_consistency(&packages, &internal_names, &config).await?;
///
/// println!("Found {} inconsistencies", section.inconsistencies.len());
/// for inconsistency in &section.inconsistencies {
///     println!("  - {}", inconsistency.describe());
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## With custom configuration
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::audit_version_consistency;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use std::collections::HashSet;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let packages = vec![/* discovered packages */];
/// let internal_names: HashSet<String> = packages.iter()
///     .map(|p| p.name().to_string())
///     .collect();
///
/// let mut config = PackageToolsConfig::default();
/// config.audit.version_consistency.fail_on_inconsistency = true;
///
/// let section = audit_version_consistency(&packages, &internal_names, &config).await?;
///
/// // Critical issues will be generated with fail_on_inconsistency = true
/// if section.critical_issue_count() > 0 {
///     eprintln!("Critical version consistency issues found!");
/// }
/// # Ok(())
/// # }
/// ```
pub async fn audit_version_consistency(
    packages: &[PackageInfo],
    internal_package_names: &std::collections::HashSet<String>,
    config: &PackageToolsConfig,
) -> AuditResult<VersionConsistencyAuditSection> {
    // If the section is disabled, return empty
    if !config.audit.sections.version_consistency {
        return Err(AuditError::SectionDisabled { section: "version_consistency".to_string() });
    }

    // Track internal dependency usage across all packages
    let internal_usage = collect_internal_dependency_usage(packages, internal_package_names);

    // Detect inconsistencies
    let inconsistencies = detect_inconsistencies(internal_usage);

    // Generate issues based on configuration
    let issues = generate_issues(&inconsistencies, config);

    Ok(VersionConsistencyAuditSection { inconsistencies, issues })
}

/// Collects all internal dependency usage across packages.
///
/// Returns a map from internal package name to all version usages across the workspace.
fn collect_internal_dependency_usage(
    packages: &[PackageInfo],
    internal_package_names: &std::collections::HashSet<String>,
) -> HashMap<String, Vec<VersionUsage>> {
    let mut usage_map: HashMap<String, Vec<VersionUsage>> = HashMap::new();

    for package in packages {
        let package_name = package.name();

        // Get all dependencies from package.json - we need to access raw dependencies
        // to include workspace protocol dependencies which are filtered out by all_dependencies()
        let mut all_deps: Vec<(String, String)> = Vec::new();

        // Collect from all dependency types
        if let Some(deps) = &package.package_json().dependencies {
            all_deps.extend(deps.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        if let Some(deps) = &package.package_json().dev_dependencies {
            all_deps.extend(deps.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        if let Some(deps) = &package.package_json().peer_dependencies {
            all_deps.extend(deps.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        if let Some(deps) = &package.package_json().optional_dependencies {
            all_deps.extend(deps.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        for (dep_name, version_spec) in all_deps {
            // Only track internal dependencies
            if internal_package_names.contains(&dep_name) {
                // Skip self-references
                if dep_name == package_name {
                    continue;
                }

                usage_map.entry(dep_name.clone()).or_default().push(VersionUsage {
                    package_name: package_name.to_string(),
                    version_spec: version_spec.clone(),
                });
            }
        }
    }

    usage_map
}

/// Detects inconsistencies in internal dependency versions.
///
/// An inconsistency exists when an internal package is referenced with more than
/// one unique version specification across the workspace.
fn detect_inconsistencies(
    usage_map: HashMap<String, Vec<VersionUsage>>,
) -> Vec<VersionInconsistency> {
    let mut inconsistencies = Vec::new();

    for (package_name, usages) in usage_map {
        // Get unique version specifications
        let unique_versions: std::collections::HashSet<String> =
            usages.iter().map(|u| u.version_spec.clone()).collect();

        // If more than one unique version, it's an inconsistency
        if unique_versions.len() > 1 {
            // Determine recommended version
            let recommended_version = determine_recommended_version(&usages, &unique_versions);

            inconsistencies.push(VersionInconsistency {
                package_name,
                versions_used: usages,
                recommended_version,
            });
        }
    }

    // Sort by package name for consistent output
    inconsistencies.sort_by(|a, b| a.package_name.cmp(&b.package_name));

    inconsistencies
}

/// Determines the recommended version for consistency.
///
/// The logic prioritizes:
/// 1. "workspace:*" if any package uses it (best practice for monorepos)
/// 2. Most commonly used version
/// 3. Alphabetically first version as fallback
fn determine_recommended_version(
    usages: &[VersionUsage],
    unique_versions: &std::collections::HashSet<String>,
) -> String {
    // Prefer workspace protocol if any package uses it
    if unique_versions.contains("workspace:*") {
        return "workspace:*".to_string();
    }

    // Check for any workspace protocol variant
    for version in unique_versions {
        if version.starts_with("workspace:") {
            return version.clone();
        }
    }

    // Count occurrences of each version
    let mut version_counts: HashMap<String, usize> = HashMap::new();
    for usage in usages {
        *version_counts.entry(usage.version_spec.clone()).or_insert(0) += 1;
    }

    // Find the most commonly used version
    let most_common =
        version_counts.iter().max_by_key(|(_, count)| *count).map(|(version, _)| version.clone());

    // Return most common, or fallback to first alphabetically
    most_common.unwrap_or_else(|| {
        let mut versions: Vec<String> = unique_versions.iter().cloned().collect();
        versions.sort();
        versions.first().cloned().unwrap_or_default()
    })
}

/// Generates audit issues from detected inconsistencies.
///
/// Issue severity is determined by configuration:
/// - Critical if `fail_on_inconsistency` is true
/// - Warning if `warn_on_inconsistency` is true
/// - No issues if both are false
fn generate_issues(
    inconsistencies: &[VersionInconsistency],
    config: &PackageToolsConfig,
) -> Vec<AuditIssue> {
    let mut issues = Vec::new();

    let severity = if config.audit.version_consistency.fail_on_inconsistency {
        IssueSeverity::Critical
    } else if config.audit.version_consistency.warn_on_inconsistency {
        IssueSeverity::Warning
    } else {
        // If both are false, don't generate issues
        return issues;
    };

    for inconsistency in inconsistencies {
        let mut issue = AuditIssue::new(
            severity,
            IssueCategory::VersionConsistency,
            format!(
                "Inconsistent versions for internal package '{}'",
                inconsistency.package_name
            ),
            format!(
                "The internal package '{}' is referenced with {} different version specifications across the workspace. \
                 This can lead to confusion and potential runtime issues.",
                inconsistency.package_name,
                inconsistency.version_count()
            ),
        );

        // Add all affected packages
        for usage in &inconsistency.versions_used {
            issue.add_affected_package(usage.package_name.clone());
        }

        // Add suggestion
        issue.set_suggestion(format!(
            "Update all references to '{}' to use '{}' for consistency. \
             The workspace protocol (workspace:*) is recommended for internal dependencies in monorepos.",
            inconsistency.package_name, inconsistency.recommended_version
        ));

        // Add metadata
        issue.add_metadata("internal_package".to_string(), inconsistency.package_name.clone());
        issue.add_metadata(
            "recommended_version".to_string(),
            inconsistency.recommended_version.clone(),
        );
        issue.add_metadata(
            "unique_version_count".to_string(),
            inconsistency.unique_versions().len().to_string(),
        );

        // Add version details
        for (idx, usage) in inconsistency.versions_used.iter().enumerate() {
            issue.add_metadata(format!("version_{}_package", idx), usage.package_name.clone());
            issue.add_metadata(format!("version_{}_spec", idx), usage.version_spec.clone());
        }

        issues.push(issue);
    }

    issues
}
