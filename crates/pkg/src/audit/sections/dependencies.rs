//! Dependency audit section for analyzing dependency graph health and conflicts.
//!
//! **What**: Provides functionality to audit internal package dependencies for circular
//! dependencies, version conflicts, and other dependency graph issues.
//!
//! **How**: Uses the `DependencyGraph` from the version module to detect circular dependencies
//! using Tarjan's algorithm, and analyzes dependency version specifications across packages
//! to identify conflicts where the same dependency is required with incompatible versions.
//!
//! **Why**: To provide early detection of dependency issues that can cause build failures,
//! runtime errors, or maintenance problems, enabling teams to maintain a healthy dependency
//! graph and resolve conflicts before they impact production.

use crate::audit::issue::{AuditIssue, IssueCategory, IssueSeverity};
use crate::config::PackageToolsConfig;
use crate::error::{AuditError, AuditResult};
use crate::types::{CircularDependency, DependencyType, PackageInfo};
use crate::version::DependencyGraph;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Audit section containing dependency analysis results.
///
/// Contains detailed information about dependency graph issues including circular
/// dependencies, version conflicts, and generated audit issues.
///
/// # Examples
///
/// ## Accessing dependency statistics
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::DependencyAuditSection;
///
/// # fn example(section: DependencyAuditSection) {
/// println!("Circular dependencies found: {}", section.circular_dependencies.len());
/// println!("Version conflicts found: {}", section.version_conflicts.len());
/// println!("Issues found: {}", section.issues.len());
/// # }
/// ```
///
/// ## Checking for critical issues
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::DependencyAuditSection;
///
/// # fn example(section: DependencyAuditSection) {
/// let critical_issues: Vec<_> = section.issues.iter()
///     .filter(|issue| issue.is_critical())
///     .collect();
///
/// if !critical_issues.is_empty() {
///     println!("Found {} critical dependency issues", critical_issues.len());
///     for issue in critical_issues {
///         println!("  - {}: {}", issue.title, issue.description);
///     }
/// }
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAuditSection {
    /// List of circular dependencies detected in the workspace.
    ///
    /// Each circular dependency represents a cycle in the dependency graph
    /// that can cause issues with versioning and package resolution.
    pub circular_dependencies: Vec<CircularDependency>,

    /// List of version conflicts detected across packages.
    ///
    /// A version conflict occurs when multiple packages depend on the same
    /// external package but with potentially incompatible version specifications.
    pub version_conflicts: Vec<VersionConflict>,

    /// List of audit issues generated from the dependency analysis.
    ///
    /// Issues are created based on detected problems:
    /// - Circular dependencies generate Critical issues
    /// - Version conflicts generate Warning issues
    pub issues: Vec<AuditIssue>,
}

impl DependencyAuditSection {
    /// Creates an empty dependency audit section.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::sections::dependencies::DependencyAuditSection;
    ///
    /// let section = DependencyAuditSection::empty();
    /// assert_eq!(section.circular_dependencies.len(), 0);
    /// assert_eq!(section.version_conflicts.len(), 0);
    /// assert_eq!(section.issues.len(), 0);
    /// ```
    #[must_use]
    pub fn empty() -> Self {
        Self {
            circular_dependencies: Vec::new(),
            version_conflicts: Vec::new(),
            issues: Vec::new(),
        }
    }

    /// Returns whether any circular dependencies were detected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::sections::dependencies::DependencyAuditSection;
    ///
    /// let section = DependencyAuditSection::empty();
    /// assert!(!section.has_circular_dependencies());
    /// ```
    #[must_use]
    pub fn has_circular_dependencies(&self) -> bool {
        !self.circular_dependencies.is_empty()
    }

    /// Returns whether any version conflicts were detected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::sections::dependencies::DependencyAuditSection;
    ///
    /// let section = DependencyAuditSection::empty();
    /// assert!(!section.has_version_conflicts());
    /// ```
    #[must_use]
    pub fn has_version_conflicts(&self) -> bool {
        !self.version_conflicts.is_empty()
    }

    /// Returns the number of critical issues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::sections::dependencies::DependencyAuditSection;
    ///
    /// let section = DependencyAuditSection::empty();
    /// assert_eq!(section.critical_issue_count(), 0);
    /// ```
    #[must_use]
    pub fn critical_issue_count(&self) -> usize {
        self.issues.iter().filter(|i| i.is_critical()).count()
    }

    /// Returns the number of warning issues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::sections::dependencies::DependencyAuditSection;
    ///
    /// let section = DependencyAuditSection::empty();
    /// assert_eq!(section.warning_issue_count(), 0);
    /// ```
    #[must_use]
    pub fn warning_issue_count(&self) -> usize {
        self.issues.iter().filter(|i| i.is_warning()).count()
    }

    /// Returns the number of informational issues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::sections::dependencies::DependencyAuditSection;
    ///
    /// let section = DependencyAuditSection::empty();
    /// assert_eq!(section.info_issue_count(), 0);
    /// ```
    #[must_use]
    pub fn info_issue_count(&self) -> usize {
        self.issues.iter().filter(|i| i.is_info()).count()
    }

    /// Returns circular dependencies that involve the specified package.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The name of the package to check
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::sections::dependencies::DependencyAuditSection;
    ///
    /// let section = DependencyAuditSection::empty();
    /// let cycles = section.circular_dependencies_for_package("my-package");
    /// assert_eq!(cycles.len(), 0);
    /// ```
    #[must_use]
    pub fn circular_dependencies_for_package(
        &self,
        package_name: &str,
    ) -> Vec<&CircularDependency> {
        self.circular_dependencies.iter().filter(|cd| cd.involves(package_name)).collect()
    }

    /// Returns version conflicts for the specified dependency.
    ///
    /// # Arguments
    ///
    /// * `dependency_name` - The name of the dependency to check
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::sections::dependencies::DependencyAuditSection;
    ///
    /// let section = DependencyAuditSection::empty();
    /// let conflicts = section.version_conflicts_for_dependency("lodash");
    /// assert!(conflicts.is_none());
    /// ```
    #[must_use]
    pub fn version_conflicts_for_dependency(
        &self,
        dependency_name: &str,
    ) -> Option<&VersionConflict> {
        self.version_conflicts.iter().find(|vc| vc.dependency_name == dependency_name)
    }
}

/// Represents a version conflict for a specific dependency.
///
/// A version conflict occurs when multiple packages in the workspace depend on
/// the same external package but specify potentially incompatible version ranges.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::audit::sections::dependencies::{VersionConflict, VersionUsage};
///
/// let conflict = VersionConflict {
///     dependency_name: "lodash".to_string(),
///     versions: vec![
///         VersionUsage {
///             package_name: "pkg-a".to_string(),
///             version_spec: "^4.17.20".to_string(),
///         },
///         VersionUsage {
///             package_name: "pkg-b".to_string(),
///             version_spec: "^3.10.1".to_string(),
///         },
///     ],
/// };
///
/// assert_eq!(conflict.dependency_name, "lodash");
/// assert_eq!(conflict.versions.len(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionConflict {
    /// Name of the dependency that has conflicting versions.
    pub dependency_name: String,

    /// List of version specifications used across different packages.
    pub versions: Vec<VersionUsage>,
}

impl VersionConflict {
    /// Returns the number of different version specifications.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::sections::dependencies::{VersionConflict, VersionUsage};
    ///
    /// let conflict = VersionConflict {
    ///     dependency_name: "lodash".to_string(),
    ///     versions: vec![
    ///         VersionUsage {
    ///             package_name: "pkg-a".to_string(),
    ///             version_spec: "^4.17.20".to_string(),
    ///         },
    ///         VersionUsage {
    ///             package_name: "pkg-b".to_string(),
    ///             version_spec: "^3.10.1".to_string(),
    ///         },
    ///     ],
    /// };
    ///
    /// assert_eq!(conflict.version_count(), 2);
    /// ```
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    /// Returns a formatted description of the conflict.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::sections::dependencies::{VersionConflict, VersionUsage};
    ///
    /// let conflict = VersionConflict {
    ///     dependency_name: "lodash".to_string(),
    ///     versions: vec![
    ///         VersionUsage {
    ///             package_name: "pkg-a".to_string(),
    ///             version_spec: "^4.17.20".to_string(),
    ///         },
    ///     ],
    /// };
    ///
    /// let description = conflict.describe();
    /// assert!(description.contains("lodash"));
    /// assert!(description.contains("pkg-a"));
    /// ```
    #[must_use]
    pub fn describe(&self) -> String {
        let version_details: Vec<String> = self
            .versions
            .iter()
            .map(|v| format!("{} ({})", v.package_name, v.version_spec))
            .collect();

        format!("{} used by: {}", self.dependency_name, version_details.join(", "))
    }
}

/// Represents a specific version usage of a dependency by a package.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::audit::sections::dependencies::VersionUsage;
///
/// let usage = VersionUsage {
///     package_name: "my-app".to_string(),
///     version_spec: "^1.2.3".to_string(),
/// };
///
/// assert_eq!(usage.package_name, "my-app");
/// assert_eq!(usage.version_spec, "^1.2.3");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionUsage {
    /// Name of the package that declares this dependency.
    pub package_name: String,

    /// Version specification declared in package.json.
    pub version_spec: String,
}

/// Audits dependency graph health for circular dependencies and version conflicts.
///
/// This function performs comprehensive dependency analysis by building a dependency
/// graph and checking for common issues that can cause problems in monorepo setups.
///
/// # Arguments
///
/// * `workspace_root` - Root directory of the workspace
/// * `packages` - List of all packages in the workspace
/// * `config` - Configuration controlling which checks to perform
///
/// # Returns
///
/// Returns a `DependencyAuditSection` containing detected issues and generated audit issues.
///
/// # Errors
///
/// Returns an error if:
/// - The dependency graph cannot be constructed
/// - Package information is invalid
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::sections::dependencies::audit_dependencies;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use sublime_pkg_tools::types::PackageInfo;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
/// let packages: Vec<PackageInfo> = vec![/* ... */];
/// let config = PackageToolsConfig::default();
///
/// let section = audit_dependencies(&workspace_root, &packages, &config).await?;
///
/// println!("Circular dependencies: {}", section.circular_dependencies.len());
/// println!("Version conflicts: {}", section.version_conflicts.len());
/// println!("Total issues: {}", section.issues.len());
/// # Ok(())
/// # }
/// ```
pub async fn audit_dependencies(
    _workspace_root: &std::path::Path,
    packages: &[PackageInfo],
    config: &PackageToolsConfig,
) -> AuditResult<DependencyAuditSection> {
    let mut section = DependencyAuditSection::empty();

    // Early return if dependencies section is disabled
    if !config.audit.sections.dependencies {
        return Ok(section);
    }

    // Build dependency graph
    let graph = DependencyGraph::from_packages(packages).map_err(|e| {
        AuditError::DependencyGraphFailed {
            reason: format!("Failed to build dependency graph: {}", e),
        }
    })?;

    // Check for circular dependencies if configured
    if config.audit.dependencies.check_circular {
        section.circular_dependencies = graph.detect_cycles();

        // Generate critical issues for each circular dependency
        for circular_dep in &section.circular_dependencies {
            let mut issue = AuditIssue::new(
                IssueSeverity::Critical,
                IssueCategory::Dependencies,
                "Circular dependency detected".to_string(),
                format!(
                    "A circular dependency exists in the workspace: {}. \
                     This can cause issues with version resolution and may lead to infinite loops.",
                    circular_dep.display_cycle()
                ),
            );

            // Add all packages in the cycle as affected packages
            for package_name in &circular_dep.cycle {
                issue.add_affected_package(package_name.clone());
            }

            issue.set_suggestion(
                "Break the circular dependency by restructuring package dependencies. \
                 Consider extracting shared functionality into a separate package or \
                 using dependency inversion."
                    .to_string(),
            );

            issue.add_metadata("cycle".to_string(), circular_dep.display_cycle());
            issue.add_metadata("cycle_length".to_string(), circular_dep.len().to_string());

            section.issues.push(issue);
        }
    }

    // Check for version conflicts if configured
    if config.audit.dependencies.check_version_conflicts {
        section.version_conflicts = detect_version_conflicts(packages);

        // Generate warning issues for each version conflict
        for conflict in &section.version_conflicts {
            let mut issue = AuditIssue::new(
                IssueSeverity::Warning,
                IssueCategory::Dependencies,
                format!("Version conflict for dependency '{}'", conflict.dependency_name),
                format!(
                    "Multiple packages depend on '{}' with different version specifications. \
                     This may cause unexpected behavior or installation issues. {}",
                    conflict.dependency_name,
                    conflict.describe()
                ),
            );

            // Add all packages that use this dependency as affected packages
            for version_usage in &conflict.versions {
                issue.add_affected_package(version_usage.package_name.clone());
            }

            issue.set_suggestion(format!(
                "Align version specifications for '{}' across all packages. \
                     Consider using workspace protocol (workspace:*) for internal dependencies \
                     or ensure compatible version ranges for external dependencies.",
                conflict.dependency_name
            ));

            issue.add_metadata("dependency".to_string(), conflict.dependency_name.clone());
            issue.add_metadata("conflict_count".to_string(), conflict.version_count().to_string());

            // Add version details as metadata
            for (idx, version_usage) in conflict.versions.iter().enumerate() {
                issue.add_metadata(
                    format!("version_{}", idx),
                    format!("{}={}", version_usage.package_name, version_usage.version_spec),
                );
            }

            section.issues.push(issue);
        }
    }

    // TODO: will be implemented in story 10.4 (check_missing)
    // Missing dependencies require source code analysis to detect imports
    // that don't have corresponding package.json entries

    // TODO: will be implemented in story 10.4 (check_unused)
    // Unused dependencies require source code analysis to determine if
    // declared dependencies are actually imported in the codebase

    Ok(section)
}

/// Detects version conflicts for external dependencies across packages.
///
/// This function analyzes all external dependencies (non-workspace, non-local) used
/// by packages in the workspace and identifies cases where the same dependency is
/// declared with different version specifications.
///
/// # Arguments
///
/// * `packages` - List of all packages in the workspace
///
/// # Returns
///
/// A vector of `VersionConflict` structures, one for each dependency that has
/// conflicting version specifications.
///
/// # Algorithm
///
/// 1. Collect all external dependencies from all packages
/// 2. Group by dependency name
/// 3. For each dependency, collect all unique version specifications
/// 4. If more than one unique version spec exists, create a conflict entry
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::sections::dependencies::detect_version_conflicts;
/// use sublime_pkg_tools::types::PackageInfo;
///
/// let packages: Vec<PackageInfo> = vec![/* ... */];
/// let conflicts = detect_version_conflicts(&packages);
///
/// for conflict in conflicts {
///     println!("Conflict for {}: {} versions", conflict.dependency_name, conflict.version_count());
/// }
/// ```
fn detect_version_conflicts(packages: &[PackageInfo]) -> Vec<VersionConflict> {
    // Map from dependency name to list of (package_name, version_spec) pairs
    let mut dependency_usage: HashMap<String, Vec<(String, String)>> = HashMap::new();

    // Collect all external dependencies from all packages
    for package in packages {
        let package_name = package.name().to_string();

        // Get all dependencies (regular, dev, peer, optional)
        for (dep_name, version_spec, dep_type) in package.all_dependencies() {
            // Skip workspace and local protocol dependencies
            if is_workspace_or_local_protocol(&version_spec) {
                continue;
            }

            // Skip dependencies to other packages in the workspace
            // (these are internal, not external)
            let is_internal = packages.iter().any(|p| p.name() == dep_name);
            if is_internal {
                continue;
            }

            // Only track regular dependencies and peer dependencies for conflicts
            // Dev dependencies can safely vary across packages
            if matches!(dep_type, DependencyType::Regular | DependencyType::Peer) {
                dependency_usage
                    .entry(dep_name)
                    .or_default()
                    .push((package_name.clone(), version_spec));
            }
        }
    }

    // Find conflicts: dependencies with multiple different version specs
    let mut conflicts = Vec::new();

    for (dep_name, usages) in dependency_usage {
        // Group usages by version spec to find unique versions
        let mut version_map: HashMap<String, Vec<String>> = HashMap::new();

        for (package_name, version_spec) in usages {
            version_map.entry(version_spec.clone()).or_default().push(package_name);
        }

        // If there are multiple different version specs, it's a conflict
        if version_map.len() > 1 {
            let mut versions = Vec::new();

            for (version_spec, package_names) in version_map {
                for package_name in package_names {
                    versions
                        .push(VersionUsage { package_name, version_spec: version_spec.clone() });
                }
            }

            // Sort for consistent output
            versions.sort_by(|a, b| {
                a.package_name
                    .cmp(&b.package_name)
                    .then_with(|| a.version_spec.cmp(&b.version_spec))
            });

            conflicts.push(VersionConflict { dependency_name: dep_name, versions });
        }
    }

    // Sort conflicts by dependency name for consistent output
    conflicts.sort_by(|a, b| a.dependency_name.cmp(&b.dependency_name));

    conflicts
}

/// Checks if a version specification uses workspace or local protocol.
///
/// # Arguments
///
/// * `version_spec` - The version specification string
///
/// # Returns
///
/// `true` if the version spec uses workspace:, file:, link:, or portal: protocol.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::sections::dependencies::is_workspace_or_local_protocol;
///
/// assert!(is_workspace_or_local_protocol("workspace:*"));
/// assert!(is_workspace_or_local_protocol("file:../local-lib"));
/// assert!(is_workspace_or_local_protocol("link:../shared"));
/// assert!(!is_workspace_or_local_protocol("^1.2.3"));
/// assert!(!is_workspace_or_local_protocol("1.2.3"));
/// ```
fn is_workspace_or_local_protocol(version_spec: &str) -> bool {
    version_spec.starts_with("workspace:")
        || version_spec.starts_with("file:")
        || version_spec.starts_with("link:")
        || version_spec.starts_with("portal:")
}
