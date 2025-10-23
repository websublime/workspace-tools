//! Dependency categorization section for analyzing dependency types and structure.
//!
//! **What**: Provides functionality to categorize all dependencies in a workspace into
//! internal packages, external packages, workspace links, and local links, with detailed
//! statistics and usage information for each category.
//!
//! **How**: Analyzes all packages in the workspace and their dependencies, using version
//! specification patterns to identify workspace protocols (workspace:*), local protocols
//! (file:, link:, portal:), internal workspace packages, and external registry packages.
//! Builds comprehensive lists of each category with usage tracking.
//!
//! **Why**: To provide clear visibility into the dependency structure of a project,
//! enabling teams to understand the balance between internal and external dependencies,
//! identify workspace protocol usage, and make informed decisions about dependency
//! management and architecture.

use crate::audit::issue::{AuditIssue, IssueCategory, IssueSeverity};
use crate::config::PackageToolsConfig;
use crate::error::AuditResult;
use crate::types::PackageInfo;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Categorization of all dependencies in a workspace.
///
/// Contains comprehensive information about dependency categories including
/// internal packages, external packages, workspace links, and local links,
/// along with summary statistics.
///
/// # Examples
///
/// ## Accessing categorization statistics
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::DependencyCategorization;
///
/// # fn example(categorization: DependencyCategorization) {
/// println!("Total packages: {}", categorization.stats.total_packages);
/// println!("Internal packages: {}", categorization.stats.internal_packages);
/// println!("External packages: {}", categorization.stats.external_packages);
/// println!("Workspace links: {}", categorization.stats.workspace_links);
/// println!("Local links: {}", categorization.stats.local_links);
/// # }
/// ```
///
/// ## Finding packages using a specific internal package
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::DependencyCategorization;
///
/// # fn example(categorization: DependencyCategorization) {
/// for internal_pkg in &categorization.internal_packages {
///     if internal_pkg.used_by.len() > 5 {
///         println!("{} is used by {} packages", internal_pkg.name, internal_pkg.used_by.len());
///     }
/// }
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCategorization {
    /// Internal packages (workspace packages).
    ///
    /// These are packages that exist within the current workspace and are
    /// depended upon by other packages in the workspace.
    pub internal_packages: Vec<InternalPackage>,

    /// External packages (from registries).
    ///
    /// These are packages from npm or other registries that are not part
    /// of the current workspace.
    pub external_packages: Vec<ExternalPackage>,

    /// Workspace links (workspace:*, workspace:^, etc.).
    ///
    /// These are dependencies using the workspace protocol, which references
    /// packages within the workspace using workspace-relative versioning.
    pub workspace_links: Vec<WorkspaceLink>,

    /// Local links (file:, link:, portal:).
    ///
    /// These are dependencies using local filesystem protocols to reference
    /// packages via relative or absolute paths.
    pub local_links: Vec<LocalLink>,

    /// Summary statistics for all categories.
    pub stats: CategorizationStats,
}

impl DependencyCategorization {
    /// Creates an empty categorization result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::DependencyCategorization;
    ///
    /// let categorization = DependencyCategorization::empty();
    /// assert_eq!(categorization.stats.total_packages, 0);
    /// assert_eq!(categorization.internal_packages.len(), 0);
    /// ```
    #[must_use]
    pub fn empty() -> Self {
        Self {
            internal_packages: Vec::new(),
            external_packages: Vec::new(),
            workspace_links: Vec::new(),
            local_links: Vec::new(),
            stats: CategorizationStats::default(),
        }
    }

    /// Returns whether any internal packages were found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::DependencyCategorization;
    ///
    /// let categorization = DependencyCategorization::empty();
    /// assert!(!categorization.has_internal_packages());
    /// ```
    #[must_use]
    pub fn has_internal_packages(&self) -> bool {
        !self.internal_packages.is_empty()
    }

    /// Returns whether any external packages were found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::DependencyCategorization;
    ///
    /// let categorization = DependencyCategorization::empty();
    /// assert!(!categorization.has_external_packages());
    /// ```
    #[must_use]
    pub fn has_external_packages(&self) -> bool {
        !self.external_packages.is_empty()
    }

    /// Returns whether any workspace links were found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::DependencyCategorization;
    ///
    /// let categorization = DependencyCategorization::empty();
    /// assert!(!categorization.has_workspace_links());
    /// ```
    #[must_use]
    pub fn has_workspace_links(&self) -> bool {
        !self.workspace_links.is_empty()
    }

    /// Returns whether any local links were found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::DependencyCategorization;
    ///
    /// let categorization = DependencyCategorization::empty();
    /// assert!(!categorization.has_local_links());
    /// ```
    #[must_use]
    pub fn has_local_links(&self) -> bool {
        !self.local_links.is_empty()
    }

    /// Returns the percentage of internal packages relative to total unique dependencies.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::DependencyCategorization;
    ///
    /// let categorization = DependencyCategorization::empty();
    /// assert_eq!(categorization.internal_percentage(), 0.0);
    /// ```
    #[must_use]
    pub fn internal_percentage(&self) -> f64 {
        let total = self.stats.internal_packages + self.stats.external_packages;
        if total == 0 {
            0.0
        } else {
            (self.stats.internal_packages as f64 / total as f64) * 100.0
        }
    }

    /// Returns the percentage of external packages relative to total unique dependencies.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::DependencyCategorization;
    ///
    /// let categorization = DependencyCategorization::empty();
    /// assert_eq!(categorization.external_percentage(), 0.0);
    /// ```
    #[must_use]
    pub fn external_percentage(&self) -> f64 {
        let total = self.stats.internal_packages + self.stats.external_packages;
        if total == 0 {
            0.0
        } else {
            (self.stats.external_packages as f64 / total as f64) * 100.0
        }
    }
}

/// Internal package information.
///
/// Represents a package that exists within the current workspace and is
/// depended upon by other packages in the workspace.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::InternalPackage;
///
/// # fn example(pkg: InternalPackage) {
/// println!("Package: {} (v{})", pkg.name, pkg.version.as_ref().map_or("unknown", |v| v.as_str()));
/// println!("Used by {} packages", pkg.used_by.len());
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalPackage {
    /// Package name.
    pub name: String,

    /// Package path relative to workspace root.
    pub path: PathBuf,

    /// Current version of the package.
    ///
    /// May be `None` if the package.json doesn't specify a version.
    pub version: Option<String>,

    /// List of package names that depend on this package.
    ///
    /// Each entry is the name of a package in the workspace that lists
    /// this package as a dependency.
    pub used_by: Vec<String>,
}

/// External package information.
///
/// Represents a package from a registry (npm, etc.) that is not part of
/// the current workspace.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::ExternalPackage;
///
/// # fn example(pkg: ExternalPackage) {
/// println!("Package: {} ({})", pkg.name, pkg.version_spec);
/// println!("Used by {} packages", pkg.used_by.len());
/// if pkg.is_deprecated {
///     println!("WARNING: This package is deprecated!");
/// }
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalPackage {
    /// Package name.
    pub name: String,

    /// Version specification as it appears in package.json.
    ///
    /// Examples: "^1.0.0", "~2.3.4", "latest", "1.x"
    pub version_spec: String,

    /// List of package names that use this external package.
    ///
    /// Each entry is the name of a package in the workspace that lists
    /// this external package as a dependency.
    pub used_by: Vec<String>,

    /// Whether this package is marked as deprecated.
    ///
    /// Note: This field is always `false` during categorization as deprecation
    /// checking requires registry API calls. Use the upgrade audit section
    /// (via `AuditManager::audit_upgrades()`) to detect deprecated packages.
    pub is_deprecated: bool,
}

/// Workspace link information.
///
/// Represents a dependency using the workspace protocol (workspace:*, workspace:^, etc.)
/// which references packages within the workspace using workspace-relative versioning.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::WorkspaceLink;
///
/// # fn example(link: WorkspaceLink) {
/// println!("Package {} depends on {} via {}",
///     link.package_name, link.dependency_name, link.version_spec);
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLink {
    /// Name of the package that has the workspace dependency.
    pub package_name: String,

    /// Name of the dependency package.
    pub dependency_name: String,

    /// Version specification using workspace protocol.
    ///
    /// Examples: "workspace:*", "workspace:^", "workspace:~", "workspace:^1.0.0"
    pub version_spec: String,
}

/// Local link information.
///
/// Represents a dependency using local filesystem protocols (file:, link:, portal:)
/// to reference packages via relative or absolute paths.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::LocalLink;
///
/// # fn example(link: LocalLink) {
/// println!("Package {} links to {} via {} protocol",
///     link.package_name, link.dependency_name, link.link_type.as_str());
/// println!("Path: {}", link.path);
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalLink {
    /// Name of the package that has the local dependency.
    pub package_name: String,

    /// Name of the dependency package.
    pub dependency_name: String,

    /// Type of local link protocol used.
    pub link_type: LocalLinkType,

    /// Path specification as it appears in the version spec.
    ///
    /// This is the path portion after the protocol (e.g., "../utils" from "file:../utils")
    pub path: String,
}

/// Type of local link protocol.
///
/// Represents different protocols for linking to local filesystem packages.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::audit::LocalLinkType;
///
/// let file_type = LocalLinkType::File;
/// assert_eq!(file_type.as_str(), "file");
/// assert_eq!(file_type.protocol_prefix(), "file:");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalLinkType {
    /// file: protocol - Links to a local directory.
    File,

    /// link: protocol - Creates a symlink to a local directory.
    Link,

    /// portal: protocol - Used by Yarn Berry for portals.
    Portal,
}

impl LocalLinkType {
    /// Returns the string representation of the link type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::LocalLinkType;
    ///
    /// assert_eq!(LocalLinkType::File.as_str(), "file");
    /// assert_eq!(LocalLinkType::Link.as_str(), "link");
    /// assert_eq!(LocalLinkType::Portal.as_str(), "portal");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Link => "link",
            Self::Portal => "portal",
        }
    }

    /// Returns the protocol prefix including the colon.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::LocalLinkType;
    ///
    /// assert_eq!(LocalLinkType::File.protocol_prefix(), "file:");
    /// assert_eq!(LocalLinkType::Link.protocol_prefix(), "link:");
    /// assert_eq!(LocalLinkType::Portal.protocol_prefix(), "portal:");
    /// ```
    #[must_use]
    pub fn protocol_prefix(&self) -> &'static str {
        match self {
            Self::File => "file:",
            Self::Link => "link:",
            Self::Portal => "portal:",
        }
    }

    /// Parses a link type from a version specification.
    ///
    /// Returns `None` if the version spec doesn't use a recognized local protocol.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::LocalLinkType;
    ///
    /// assert_eq!(LocalLinkType::from_version_spec("file:../utils"), Some(LocalLinkType::File));
    /// assert_eq!(LocalLinkType::from_version_spec("link:./packages/core"), Some(LocalLinkType::Link));
    /// assert_eq!(LocalLinkType::from_version_spec("portal:../shared"), Some(LocalLinkType::Portal));
    /// assert_eq!(LocalLinkType::from_version_spec("^1.0.0"), None);
    /// ```
    #[must_use]
    pub fn from_version_spec(version_spec: &str) -> Option<Self> {
        if version_spec.starts_with("file:") {
            Some(Self::File)
        } else if version_spec.starts_with("link:") {
            Some(Self::Link)
        } else if version_spec.starts_with("portal:") {
            Some(Self::Portal)
        } else {
            None
        }
    }
}

impl std::fmt::Display for LocalLinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Categorization statistics.
///
/// Contains summary counts for each category of dependencies.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::audit::CategorizationStats;
///
/// let stats = CategorizationStats::default();
/// assert_eq!(stats.total_packages, 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategorizationStats {
    /// Total number of packages in the workspace.
    pub total_packages: usize,

    /// Number of unique internal packages (workspace packages that are depended upon).
    pub internal_packages: usize,

    /// Number of unique external packages (registry packages).
    pub external_packages: usize,

    /// Number of workspace protocol links.
    pub workspace_links: usize,

    /// Number of local filesystem protocol links.
    pub local_links: usize,
}

/// Categorizes all dependencies in the workspace.
///
/// This function analyzes all packages and their dependencies, categorizing them into:
/// - Internal packages: Workspace packages that are used by other packages
/// - External packages: Registry packages from npm, etc.
/// - Workspace links: Dependencies using workspace: protocol
/// - Local links: Dependencies using file:, link:, or portal: protocols
///
/// # Arguments
///
/// * `packages` - All packages in the workspace to analyze
/// * `_config` - Configuration for categorization (currently unused, reserved for future use)
///
/// # Returns
///
/// Returns a `DependencyCategorization` containing all categorized dependencies and statistics.
///
/// # Errors
///
/// Currently does not return errors, but the error type is preserved for future validation.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::categorize_dependencies;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use sublime_pkg_tools::types::PackageInfo;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let packages: Vec<PackageInfo> = vec![/* ... */];
/// let config = PackageToolsConfig::default();
///
/// let categorization = categorize_dependencies(&packages, &config).await?;
///
/// println!("Found {} internal packages", categorization.stats.internal_packages);
/// println!("Found {} external packages", categorization.stats.external_packages);
/// # Ok(())
/// # }
/// ```
pub async fn categorize_dependencies(
    packages: &[PackageInfo],
    _config: &PackageToolsConfig,
) -> AuditResult<DependencyCategorization> {
    // Build a set of all package names in the workspace for quick lookup
    let workspace_packages: HashSet<String> =
        packages.iter().map(|p| p.name().to_string()).collect();

    // Maps to track unique dependencies and their usage
    let mut internal_map: HashMap<String, InternalPackage> = HashMap::new();
    let mut external_map: HashMap<String, ExternalPackage> = HashMap::new();
    let mut workspace_links: Vec<WorkspaceLink> = Vec::new();
    let mut local_links: Vec<LocalLink> = Vec::new();

    // Analyze each package's dependencies
    for package in packages {
        let package_name = package.name().to_string();

        // We need to check raw dependencies to catch workspace: and local protocols
        // because all_dependencies() filters them out
        let package_json = package.package_json();

        // Collect all dependency types
        let mut all_raw_deps = Vec::new();

        if let Some(deps) = &package_json.dependencies {
            for (name, version) in deps {
                all_raw_deps.push((name.clone(), version.clone()));
            }
        }
        if let Some(deps) = &package_json.dev_dependencies {
            for (name, version) in deps {
                all_raw_deps.push((name.clone(), version.clone()));
            }
        }
        if let Some(deps) = &package_json.peer_dependencies {
            for (name, version) in deps {
                all_raw_deps.push((name.clone(), version.clone()));
            }
        }
        if let Some(deps) = &package_json.optional_dependencies {
            for (name, version) in deps {
                all_raw_deps.push((name.clone(), version.clone()));
            }
        }

        for (dep_name, version_spec) in all_raw_deps {
            // Check for workspace protocol
            if version_spec.starts_with("workspace:") {
                workspace_links.push(WorkspaceLink {
                    package_name: package_name.clone(),
                    dependency_name: dep_name.clone(),
                    version_spec: version_spec.clone(),
                });
                continue;
            }

            // Check for local protocols
            if let Some(link_type) = LocalLinkType::from_version_spec(&version_spec) {
                let path = version_spec
                    .strip_prefix(link_type.protocol_prefix())
                    .unwrap_or(&version_spec)
                    .to_string();

                local_links.push(LocalLink {
                    package_name: package_name.clone(),
                    dependency_name: dep_name.clone(),
                    link_type,
                    path,
                });
                continue;
            }

            // Check if it's an internal workspace package
            if workspace_packages.contains(&dep_name) {
                internal_map
                    .entry(dep_name.clone())
                    .and_modify(|pkg| {
                        if !pkg.used_by.contains(&package_name) {
                            pkg.used_by.push(package_name.clone());
                        }
                    })
                    .or_insert_with(|| {
                        // Find the package info for this internal dependency
                        let dep_package = packages.iter().find(|p| p.name() == dep_name);

                        InternalPackage {
                            name: dep_name.clone(),
                            path: dep_package.map(|p| p.path().to_path_buf()).unwrap_or_default(),
                            version: dep_package.map(|p| format!("{}", p.version())),
                            used_by: vec![package_name.clone()],
                        }
                    });
            } else {
                // It's an external package
                external_map
                    .entry(dep_name.clone())
                    .and_modify(|pkg| {
                        if !pkg.used_by.contains(&package_name) {
                            pkg.used_by.push(package_name.clone());
                        }
                    })
                    .or_insert_with(|| ExternalPackage {
                        name: dep_name.clone(),
                        version_spec: version_spec.clone(),
                        used_by: vec![package_name.clone()],
                        // Note: Deprecated detection requires registry API calls and is
                        // performed by the upgrade audit section (story 10.2).
                        // Categorization focuses only on classifying dependency types.
                        is_deprecated: false,
                    });
            }
        }
    }

    // Convert maps to sorted vectors
    let mut internal_packages: Vec<InternalPackage> = internal_map.into_values().collect();
    internal_packages.sort_by(|a, b| a.name.cmp(&b.name));

    let mut external_packages: Vec<ExternalPackage> = external_map.into_values().collect();
    external_packages.sort_by(|a, b| a.name.cmp(&b.name));

    // Sort links for consistent output
    workspace_links.sort_by(|a, b| {
        a.package_name.cmp(&b.package_name).then(a.dependency_name.cmp(&b.dependency_name))
    });

    local_links.sort_by(|a, b| {
        a.package_name.cmp(&b.package_name).then(a.dependency_name.cmp(&b.dependency_name))
    });

    // Calculate statistics
    let stats = CategorizationStats {
        total_packages: packages.len(),
        internal_packages: internal_packages.len(),
        external_packages: external_packages.len(),
        workspace_links: workspace_links.len(),
        local_links: local_links.len(),
    };

    Ok(DependencyCategorization {
        internal_packages,
        external_packages,
        workspace_links,
        local_links,
        stats,
    })
}

/// Generates audit issues based on dependency categorization.
///
/// Analyzes the categorization results and generates informational issues
/// about the dependency structure, such as highly-used packages and
/// workspace protocol usage.
///
/// # Arguments
///
/// * `categorization` - The categorization results to analyze
///
/// # Returns
///
/// Returns a vector of `AuditIssue` instances representing findings from the analysis.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::{categorize_dependencies, generate_categorization_issues};
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use sublime_pkg_tools::types::PackageInfo;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let packages: Vec<PackageInfo> = vec![/* ... */];
/// let config = PackageToolsConfig::default();
///
/// let categorization = categorize_dependencies(&packages, &config).await?;
/// let issues = generate_categorization_issues(&categorization);
///
/// for issue in issues {
///     println!("{}: {}", issue.title, issue.description);
/// }
/// # Ok(())
/// # }
/// ```
#[must_use]
pub fn generate_categorization_issues(
    categorization: &DependencyCategorization,
) -> Vec<AuditIssue> {
    let mut issues = Vec::new();

    // Generate issue for highly-used internal packages
    let highly_used_threshold = 5;
    for internal_pkg in &categorization.internal_packages {
        if internal_pkg.used_by.len() >= highly_used_threshold {
            issues.push(AuditIssue {
                severity: IssueSeverity::Info,
                category: IssueCategory::Dependencies,
                title: format!("Highly-used internal package: {}", internal_pkg.name),
                description: format!(
                    "Package '{}' is used by {} packages: {}. This indicates it's a core dependency.",
                    internal_pkg.name,
                    internal_pkg.used_by.len(),
                    internal_pkg.used_by.join(", ")
                ),
                affected_packages: vec![internal_pkg.name.clone()],
                suggestion: Some(format!(
                    "Consider carefully managing changes to '{}' as it impacts {} packages. \
                     Ensure proper versioning and changelog maintenance.",
                    internal_pkg.name,
                    internal_pkg.used_by.len()
                )),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("used_by_count".to_string(), internal_pkg.used_by.len().to_string());
                    meta.insert("used_by".to_string(), internal_pkg.used_by.join(", "));
                    meta
                },
            });
        }
    }

    // Generate issue for workspace protocol usage
    if !categorization.workspace_links.is_empty() {
        issues.push(AuditIssue {
            severity: IssueSeverity::Info,
            category: IssueCategory::Dependencies,
            title: format!(
                "Workspace protocol in use ({} links)",
                categorization.workspace_links.len()
            ),
            description: format!(
                "Found {} dependencies using workspace: protocol. This ensures packages \
                 always use the workspace version of internal dependencies.",
                categorization.workspace_links.len()
            ),
            affected_packages: categorization
                .workspace_links
                .iter()
                .map(|link| link.package_name.clone())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
            suggestion: Some(
                "Workspace protocol is recommended for internal dependencies in monorepos. \
                 It ensures consistency and prevents version mismatches."
                    .to_string(),
            ),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("count".to_string(), categorization.workspace_links.len().to_string());
                meta
            },
        });
    }

    // Generate issue for local protocol usage
    if !categorization.local_links.is_empty() {
        let file_count = categorization
            .local_links
            .iter()
            .filter(|link| link.link_type == LocalLinkType::File)
            .count();
        let link_count = categorization
            .local_links
            .iter()
            .filter(|link| link.link_type == LocalLinkType::Link)
            .count();
        let portal_count = categorization
            .local_links
            .iter()
            .filter(|link| link.link_type == LocalLinkType::Portal)
            .count();

        issues.push(AuditIssue {
            severity: IssueSeverity::Warning,
            category: IssueCategory::Dependencies,
            title: format!(
                "Local filesystem protocols in use ({} links)",
                categorization.local_links.len()
            ),
            description: format!(
                "Found {} dependencies using local filesystem protocols (file: {}, link: {}, portal: {}). \
                 These dependencies reference packages via filesystem paths.",
                categorization.local_links.len(),
                file_count,
                link_count,
                portal_count
            ),
            affected_packages: categorization
                .local_links
                .iter()
                .map(|link| link.package_name.clone())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
            suggestion: Some(
                "Consider using workspace: protocol for internal dependencies instead of file:, link:, or portal: \
                 protocols. Local protocols can cause issues with portability and package managers."
                    .to_string(),
            ),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("file_count".to_string(), file_count.to_string());
                meta.insert("link_count".to_string(), link_count.to_string());
                meta.insert("portal_count".to_string(), portal_count.to_string());
                meta.insert("total".to_string(), categorization.local_links.len().to_string());
                meta
            },
        });
    }

    // Generate summary issue
    issues.push(AuditIssue {
        severity: IssueSeverity::Info,
        category: IssueCategory::Dependencies,
        title: "Dependency categorization summary".to_string(),
        description: format!(
            "Workspace contains {} packages with {} unique internal packages and {} unique external packages. \
             Internal/External ratio: {:.1}%/{:.1}%.",
            categorization.stats.total_packages,
            categorization.stats.internal_packages,
            categorization.stats.external_packages,
            categorization.internal_percentage(),
            categorization.external_percentage()
        ),
        affected_packages: Vec::new(),
        suggestion: None,
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("total_packages".to_string(), categorization.stats.total_packages.to_string());
            meta.insert("internal_packages".to_string(), categorization.stats.internal_packages.to_string());
            meta.insert("external_packages".to_string(), categorization.stats.external_packages.to_string());
            meta.insert("workspace_links".to_string(), categorization.stats.workspace_links.to_string());
            meta.insert("local_links".to_string(), categorization.stats.local_links.to_string());
            meta.insert("internal_percentage".to_string(), format!("{:.1}", categorization.internal_percentage()));
            meta.insert("external_percentage".to_string(), format!("{:.1}", categorization.external_percentage()));
            meta
        },
    });

    issues
}
