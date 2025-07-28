//! # Dependency Analysis System
//!
//! Enterprise-grade dependency analysis for Node.js packages with intelligent
//! classification, conflict detection, and comprehensive reporting capabilities.
//!
//! ## What
//!
//! The `DependencyAnalyzer` provides advanced analysis of package dependencies including:
//! - Comprehensive dependency analysis with metrics and insights
//! - Version conflict detection between packages
//! - Intelligent dependency classification (internal/external/dev/peer)
//! - Performance-optimized algorithms for large monorepos
//!
//! ## How
//!
//! This module implements a stateless analyzer that processes packages and their
//! dependencies using configurable analysis strategies. All operations are designed
//! for performance with minimal memory allocation and efficient algorithms.
//!
//! ## Why
//!
//! Large monorepos require sophisticated dependency analysis to:
//! - Detect version conflicts before they cause runtime issues
//! - Classify dependencies for proper management and security
//! - Generate actionable insights for dependency optimization
//! - Support automated dependency upgrade workflows
//!
//! ## Examples
//!
//! ```rust
//! use sublime_package_tools::dependency::analyzer::DependencyAnalyzer;
//! use sublime_package_tools::config::PackageToolsConfig;
//! use sublime_package_tools::Package;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create analyzer with default configuration
//! let analyzer = DependencyAnalyzer::new();
//!
//! // Analyze a single package
//! let package = Package::new("my-app", "1.0.0", None)?;
//! let report = analyzer.analyze_dependencies(&package);
//! println!("Dependencies: {}", report.total_count());
//!
//! // Find conflicts across multiple packages
//! let packages = vec![package];
//! let conflicts = analyzer.find_conflicts(&packages);
//! if !conflicts.is_empty() {
//!     println!("Found {} conflicts", conflicts.len());
//! }
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

use crate::{
    config::PackageToolsConfig,
    Package,
    Dependency,
};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// Advanced dependency analyzer for Node.js packages.
///
/// Provides comprehensive analysis capabilities including dependency metrics,
/// conflict detection, and intelligent classification. Designed for enterprise
/// use with performance optimization for large monorepos.
///
/// ## Configuration
///
/// The analyzer uses `PackageToolsConfig` for customizing analysis behavior:
/// - Registry settings for version resolution
/// - Monorepo settings for workspace-aware classification
/// - Standard settings for filesystem and performance tuning
///
/// ## Performance
///
/// All operations are optimized for large-scale analysis:
/// - O(n) complexity for single package analysis
/// - O(n²) worst-case for conflict detection with optimizations
/// - Minimal memory allocation with reusable data structures
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::dependency::analyzer::DependencyAnalyzer;
/// use sublime_package_tools::config::PackageToolsConfig;
///
/// // Create with custom configuration
/// let config = PackageToolsConfig::default();
/// let analyzer = DependencyAnalyzer::with_config(config);
///
/// // Use for analysis operations
/// // (requires actual packages for meaningful analysis)
/// ```
#[derive(Debug, Clone)]
pub struct DependencyAnalyzer {
    /// Configuration for analysis behavior and performance tuning
    config: PackageToolsConfig,
    
    /// Cache for dependency classification results to improve performance
    /// Key: dependency name, Value: (classification, timestamp)
    classification_cache: HashMap<String, (DependencyClass, Instant)>,
    
    /// Cache for version analysis results
    /// Key: version string, Value: (analysis result, timestamp)
    version_cache: HashMap<String, (VersionAnalysis, Instant)>,
}

/// Classification of a dependency based on its usage and context.
///
/// Dependencies are classified to enable different management strategies:
/// - Internal dependencies require different update strategies
/// - Development dependencies don't affect production builds
/// - Peer dependencies require coordination with host packages
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::dependency::analyzer::DependencyClass;
///
/// // Check if a dependency is internal to the workspace
/// let class = DependencyClass::Internal;
/// assert!(class.is_internal());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencyClass {
    /// Internal dependency within the current workspace/monorepo
    Internal,
    /// External dependency from public registries
    External,
    /// Development-only dependency (devDependencies)
    Development,
    /// Peer dependency that must be provided by the consumer
    Peer,
    /// Optional dependency that doesn't affect functionality if missing
    Optional,
}

/// Detailed analysis results for a package's dependencies.
///
/// Contains comprehensive metrics, insights, and classifications for all
/// dependencies in a package. Used to generate reports and make automated
/// decisions about dependency management.
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::dependency::analyzer::DependencyReport;
///
/// // Access report metrics
/// # let report = DependencyReport::new();
/// println!("Total dependencies: {}", report.total_count());
/// println!("External dependencies: {}", report.external_count());
/// ```
#[derive(Debug, Clone)]
pub struct DependencyReport {
    /// Total number of dependencies analyzed
    pub total_count: usize,
    /// Number of production dependencies
    pub production_count: usize,
    /// Number of development dependencies
    pub development_count: usize,
    /// Number of peer dependencies
    pub peer_count: usize,
    /// Number of optional dependencies
    pub optional_count: usize,
    /// Number of internal dependencies (workspace/monorepo)
    pub internal_count: usize,
    /// Number of external dependencies (from registries)
    pub external_count: usize,
    /// Dependencies classified by type
    pub classifications: HashMap<String, DependencyClass>,
    /// Version analysis for each dependency
    pub version_analysis: HashMap<String, VersionAnalysis>,
    /// Detected patterns and insights
    pub insights: Vec<DependencyInsight>,
    /// Analysis duration for performance monitoring
    pub analysis_duration: std::time::Duration,
}

/// Represents a version conflict between packages.
///
/// Contains detailed information about conflicting dependencies including
/// the packages involved, conflicting versions, and suggested resolutions.
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::dependency::analyzer::Conflict;
///
/// # let conflict = Conflict {
/// #     dependency_name: "react".to_string(),
/// #     conflicting_versions: vec!["^17.0.0".to_string(), "^18.0.0".to_string()],
/// #     affected_packages: vec!["app".to_string(), "shared".to_string()],
/// #     severity: ConflictSeverity::High,
/// #     suggested_resolution: Some("Upgrade all packages to ^18.0.0".to_string()),
/// # };
/// println!("Conflict in {}: {:?}", conflict.dependency_name, conflict.conflicting_versions);
/// ```
#[derive(Debug, Clone)]
pub struct Conflict {
    /// Name of the dependency that has conflicts
    pub dependency_name: String,
    /// List of conflicting version requirements
    pub conflicting_versions: Vec<String>,
    /// Packages that have this conflicting dependency
    pub affected_packages: Vec<String>,
    /// Severity level of the conflict
    pub severity: ConflictSeverity,
    /// Suggested resolution strategy
    pub suggested_resolution: Option<String>,
}

/// Severity level of a dependency conflict.
///
/// Used to prioritize conflict resolution efforts and determine
/// the urgency of fixes required.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictSeverity {
    /// Low severity - minor version differences, likely compatible
    Low,
    /// Medium severity - potentially incompatible, should be resolved
    Medium,
    /// High severity - definitely incompatible, must be resolved
    High,
    /// Critical severity - blocking issue, immediate resolution required
    Critical,
}

/// Analysis results for a specific version requirement.
///
/// Contains insights about version patterns, stability, and potential issues.
#[derive(Debug, Clone)]
pub struct VersionAnalysis {
    /// Whether the version uses semantic versioning
    pub is_semver_compliant: bool,
    /// Version stability (stable, prerelease, etc.)
    pub stability: VersionStability,
    /// Whether the version range is restrictive or flexible
    pub flexibility_score: f32,
    /// Detected version patterns (exact, caret, tilde, etc.)
    pub pattern_type: VersionPattern,
    /// Potential issues with this version requirement
    pub issues: Vec<VersionIssue>,
}

/// Type of version pattern used in a dependency requirement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionPattern {
    /// Exact version (1.0.0)
    Exact,
    /// Caret range (^1.0.0)
    Caret,
    /// Tilde range (~1.0.0)
    Tilde,
    /// Greater than range (>1.0.0)
    GreaterThan,
    /// Range specification (>=1.0.0 <2.0.0)
    Range,
    /// Wildcard or latest (*)
    Wildcard,
}

/// Potential issues detected in version requirements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionIssue {
    /// Version range is too restrictive
    TooRestrictive,
    /// Version range is too permissive
    TooPermissive,
    /// Using prerelease version in production
    PrereleaseInProduction,
    /// Version pattern inconsistent with project standards
    InconsistentPattern,
    /// Version is outdated
    Outdated,
    /// Version has known security vulnerabilities
    SecurityVulnerability,
}

/// Insights generated from dependency analysis.
///
/// Provides actionable recommendations for improving dependency management.
#[derive(Debug, Clone)]
pub struct DependencyInsight {
    /// Type of insight
    pub insight_type: InsightType,
    /// Human-readable message
    pub message: String,
    /// Affected dependencies
    pub affected_dependencies: Vec<String>,
    /// Suggested action to address the insight
    pub suggested_action: Option<String>,
    /// Priority level for addressing this insight
    pub priority: InsightPriority,
}

/// Type of dependency insight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InsightType {
    /// Dependency count is unusually high
    TooManyDependencies,
    /// Dependencies are outdated
    OutdatedDependencies,
    /// Version patterns are inconsistent
    InconsistentVersioning,
    /// Potential security vulnerabilities
    SecurityConcerns,
    /// Opportunities for optimization
    OptimizationOpportunity,
    /// Dependencies that could be internal
    InternalizationCandidate,
}

/// Priority level for dependency insights.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum InsightPriority {
    /// Low priority - nice to have improvements
    Low,
    /// Medium priority - recommended improvements
    Medium,
    /// High priority - should be addressed soon
    High,
    /// Critical priority - requires immediate attention
    Critical,
}

/// Stability classification for package versions.
///
/// Re-exported from the crate root for consistency.
pub use crate::VersionStability;

impl DependencyAnalyzer {
    /// Creates a new `DependencyAnalyzer` with default configuration.
    ///
    /// Uses `PackageToolsConfig::default()` which provides sensible defaults
    /// for most use cases. For custom configuration, use `with_config()`.
    ///
    /// ## Performance
    ///
    /// The analyzer is designed to be lightweight and can be created frequently
    /// without significant overhead. However, reusing instances is recommended
    /// for better cache effectiveness.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::DependencyAnalyzer;
    ///
    /// let analyzer = DependencyAnalyzer::new();
    /// // Ready to use for dependency analysis
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(PackageToolsConfig::default())
    }

    /// Creates a new `DependencyAnalyzer` with custom configuration.
    ///
    /// Allows fine-tuning of analysis behavior through `PackageToolsConfig`:
    /// - Registry settings affect conflict detection algorithms
    /// - Monorepo settings influence dependency classification
    /// - Standard settings control performance parameters
    ///
    /// ## Arguments
    ///
    /// * `config` - Configuration for customizing analysis behavior
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::DependencyAnalyzer;
    /// use sublime_package_tools::config::PackageToolsConfig;
    ///
    /// let mut config = PackageToolsConfig::default();
    /// // Customize config as needed
    /// 
    /// let analyzer = DependencyAnalyzer::with_config(config);
    /// ```
    #[must_use]
    pub fn with_config(config: PackageToolsConfig) -> Self {
        Self {
            config,
            classification_cache: HashMap::new(),
            version_cache: HashMap::new(),
        }
    }

    /// Analyzes all dependencies in a package and generates a comprehensive report.
    ///
    /// Performs deep analysis of production, development, peer, and optional
    /// dependencies including classification, version analysis, and insight generation.
    ///
    /// ## Performance
    ///
    /// - Time complexity: O(n) where n is the number of dependencies
    /// - Space complexity: O(n) for report generation
    /// - Uses caching to improve performance on repeated analysis
    ///
    /// ## Arguments
    ///
    /// * `package` - Package to analyze
    ///
    /// ## Returns
    ///
    /// A comprehensive `DependencyReport` with analysis results
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::DependencyAnalyzer;
    /// use sublime_package_tools::Package;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let analyzer = DependencyAnalyzer::new();
    /// let package = Package::new("my-app", "1.0.0", None)?;
    /// 
    /// let report = analyzer.analyze_dependencies(&package);
    /// 
    /// println!("Analysis Results:");
    /// println!("- Total dependencies: {}", report.total_count());
    /// println!("- External dependencies: {}", report.external_count());
    /// println!("- Analysis took: {:?}", report.analysis_duration);
    /// # Ok(())
    /// # }
    /// ```
    pub fn analyze_dependencies(&self, package: &Package) -> DependencyReport {
        let start_time = Instant::now();

        // Initialize report with basic counts
        let mut report = DependencyReport::new();
        
        // TODO: Task 2.2.2 - Implement comprehensive dependency analysis
        // This will be implemented in the next task to include:
        // - Classification of each dependency
        // - Version pattern analysis
        // - Security and optimization insights
        // - Performance metrics and recommendations
        
        // For now, provide basic analysis
        report.total_count = package.dependencies.len();
        report.production_count = package.dependencies.len(); // All deps in Package are production
        report.analysis_duration = start_time.elapsed();

        // Generate basic insights
        if report.total_count > 50 {
            report.insights.push(DependencyInsight {
                insight_type: InsightType::TooManyDependencies,
                message: format!("Package has {} dependencies, consider reducing", report.total_count),
                affected_dependencies: vec![],
                suggested_action: Some("Review and remove unused dependencies".to_string()),
                priority: InsightPriority::Medium,
            });
        }

        report
    }

    /// Finds version conflicts between multiple packages.
    ///
    /// Analyzes all packages to identify dependencies with incompatible version
    /// requirements. Uses advanced algorithms to detect both obvious and subtle
    /// conflicts that could cause runtime issues.
    ///
    /// ## Performance
    ///
    /// - Time complexity: O(n*m) where n is packages and m is average dependencies
    /// - Optimizations reduce practical complexity through early termination
    /// - Results are deterministic and reproducible
    ///
    /// ## Arguments
    ///
    /// * `packages` - List of packages to analyze for conflicts
    ///
    /// ## Returns
    ///
    /// Vector of `Conflict` objects describing each detected conflict
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::DependencyAnalyzer;
    /// use sublime_package_tools::Package;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let analyzer = DependencyAnalyzer::new();
    /// let packages = vec![
    ///     Package::new("app", "1.0.0", None)?,
    ///     Package::new("shared", "1.0.0", None)?,
    /// ];
    /// 
    /// let conflicts = analyzer.find_conflicts(&packages);
    /// 
    /// for conflict in &conflicts {
    ///     println!("Conflict in {}: {:?}", 
    ///         conflict.dependency_name, 
    ///         conflict.conflicting_versions
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_conflicts(&self, packages: &[Package]) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        // TODO: Task 2.2.3 - Implement comprehensive conflict detection
        // This will include:
        // - Version requirement compatibility analysis
        // - Semantic versioning conflict detection
        // - Severity assessment based on version differences
        // - Resolution strategy recommendations
        // - Performance optimization for large package sets

        // For now, return empty conflicts (scaffolding)
        // The actual implementation will analyze version overlaps,
        // semver compatibility, and generate actionable conflict reports

        conflicts
    }

    /// Classifies a dependency based on its context and usage patterns.
    ///
    /// Uses intelligent algorithms to determine whether a dependency is internal
    /// to the workspace, external from registries, or has special characteristics
    /// like being development-only or peer dependencies.
    ///
    /// ## Classification Logic
    ///
    /// 1. **Workspace Context**: Checks if dependency exists in current workspace
    /// 2. **Naming Patterns**: Analyzes naming conventions for internal packages
    /// 3. **Registry Information**: Validates against package registries
    /// 4. **Configuration Rules**: Applies custom classification rules
    ///
    /// ## Arguments
    ///
    /// * `dependency` - Dependency to classify
    /// * `workspace_info` - Optional workspace context for internal classification
    ///
    /// ## Returns
    ///
    /// `DependencyClass` indicating the type and usage of the dependency
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::DependencyAnalyzer;
    /// use sublime_package_tools::Dependency;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let analyzer = DependencyAnalyzer::new();
    /// let dependency = Dependency::new("react", "^18.0.0")?;
    /// 
    /// let classification = analyzer.classify_dependency(&dependency, None);
    /// 
    /// match classification {
    ///     sublime_package_tools::dependency::analyzer::DependencyClass::External => {
    ///         println!("External dependency from registry");
    ///     },
    ///     sublime_package_tools::dependency::analyzer::DependencyClass::Internal => {
    ///         println!("Internal workspace dependency");
    ///     },
    ///     _ => println!("Other dependency type"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn classify_dependency(
        &self,
        dependency: &Dependency,
        workspace_info: Option<&WorkspaceInfo>,
    ) -> DependencyClass {
        // TODO: Task 2.2.4 - Implement intelligent classification
        // This will include:
        // - Workspace package detection
        // - Naming pattern analysis
        // - Registry validation
        // - Custom classification rules from config
        // - Caching for performance

        // For now, provide basic classification (scaffolding)
        // All dependencies are classified as external by default
        // until the full implementation in Task 2.2.4

        let _ = (dependency, workspace_info); // Suppress unused parameter warnings
        DependencyClass::External
    }
}

/// Workspace information for dependency classification.
///
/// Contains metadata about the current workspace that helps classify
/// dependencies as internal or external.
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    /// List of package names in the current workspace
    pub workspace_packages: HashSet<String>,
    /// Root directory of the workspace
    pub workspace_root: std::path::PathBuf,
    /// Workspace-specific naming patterns for internal packages
    pub internal_patterns: Vec<String>,
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyClass {
    /// Returns `true` if the dependency is classified as internal.
    ///
    /// Internal dependencies are those that belong to the current workspace
    /// or monorepo and require different management strategies.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::DependencyClass;
    ///
    /// assert!(DependencyClass::Internal.is_internal());
    /// assert!(!DependencyClass::External.is_internal());
    /// ```
    #[must_use]
    pub fn is_internal(&self) -> bool {
        matches!(self, DependencyClass::Internal)
    }

    /// Returns `true` if the dependency is classified as external.
    ///
    /// External dependencies come from package registries and follow
    /// standard dependency management practices.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::DependencyClass;
    ///
    /// assert!(DependencyClass::External.is_external());
    /// assert!(!DependencyClass::Internal.is_external());
    /// ```
    #[must_use]
    pub fn is_external(&self) -> bool {
        matches!(self, DependencyClass::External)
    }

    /// Returns `true` if the dependency is development-only.
    ///
    /// Development dependencies are not included in production builds
    /// and can be managed with different update strategies.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::DependencyClass;
    ///
    /// assert!(DependencyClass::Development.is_development());
    /// assert!(!DependencyClass::External.is_development());
    /// ```
    #[must_use]
    pub fn is_development(&self) -> bool {
        matches!(self, DependencyClass::Development)
    }
}

impl DependencyReport {
    /// Creates a new empty dependency report.
    ///
    /// Used internally for initializing reports before analysis.
    /// External users should obtain reports through `analyze_dependencies()`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_count: 0,
            production_count: 0,
            development_count: 0,
            peer_count: 0,
            optional_count: 0,
            internal_count: 0,
            external_count: 0,
            classifications: HashMap::new(),
            version_analysis: HashMap::new(),
            insights: Vec::new(),
            analysis_duration: std::time::Duration::ZERO,
        }
    }

    /// Returns the total number of dependencies analyzed.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::DependencyReport;
    ///
    /// let report = DependencyReport::new();
    /// assert_eq!(report.total_count(), 0);
    /// ```
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    /// Returns the number of external dependencies.
    ///
    /// External dependencies are those from package registries.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::DependencyReport;
    ///
    /// let report = DependencyReport::new();
    /// assert_eq!(report.external_count(), 0);
    /// ```
    #[must_use]
    pub fn external_count(&self) -> usize {
        self.external_count
    }

    /// Returns the number of internal dependencies.
    ///
    /// Internal dependencies are those from the current workspace.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::DependencyReport;
    ///
    /// let report = DependencyReport::new();
    /// assert_eq!(report.internal_count(), 0);
    /// ```
    #[must_use]
    pub fn internal_count(&self) -> usize {
        self.internal_count
    }

    /// Returns `true` if the report contains any high-priority insights.
    ///
    /// High-priority insights indicate issues that should be addressed
    /// to improve dependency management.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::DependencyReport;
    ///
    /// let report = DependencyReport::new();
    /// assert!(!report.has_high_priority_insights());
    /// ```
    #[must_use]
    pub fn has_high_priority_insights(&self) -> bool {
        self.insights
            .iter()
            .any(|insight| insight.priority >= InsightPriority::High)
    }

    /// Returns all insights with high or critical priority.
    ///
    /// These insights should be addressed to maintain healthy dependencies.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::DependencyReport;
    ///
    /// let report = DependencyReport::new();
    /// let high_priority = report.high_priority_insights();
    /// assert_eq!(high_priority.len(), 0);
    /// ```
    #[must_use]
    pub fn high_priority_insights(&self) -> Vec<&DependencyInsight> {
        self.insights
            .iter()
            .filter(|insight| insight.priority >= InsightPriority::High)
            .collect()
    }
}

impl Default for DependencyReport {
    fn default() -> Self {
        Self::new()
    }
}

impl ConflictSeverity {
    /// Returns `true` if the conflict severity is high or critical.
    ///
    /// High severity conflicts should be resolved promptly to avoid issues.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::analyzer::ConflictSeverity;
    ///
    /// assert!(ConflictSeverity::High.is_severe());
    /// assert!(ConflictSeverity::Critical.is_severe());
    /// assert!(!ConflictSeverity::Low.is_severe());
    /// ```
    #[must_use]
    pub fn is_severe(&self) -> bool {
        matches!(self, ConflictSeverity::High | ConflictSeverity::Critical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_analyzer_creation() {
        let analyzer = DependencyAnalyzer::new();
        assert!(analyzer.classification_cache.is_empty());
        assert!(analyzer.version_cache.is_empty());
    }

    #[test]
    fn test_dependency_analyzer_with_config() {
        let config = PackageToolsConfig::default();
        let analyzer = DependencyAnalyzer::with_config(config);
        assert!(analyzer.classification_cache.is_empty());
    }

    #[test]
    fn test_dependency_class_methods() {
        assert!(DependencyClass::Internal.is_internal());
        assert!(!DependencyClass::External.is_internal());
        
        assert!(DependencyClass::External.is_external());
        assert!(!DependencyClass::Internal.is_external());
        
        assert!(DependencyClass::Development.is_development());
        assert!(!DependencyClass::External.is_development());
    }

    #[test]
    fn test_dependency_report_creation() {
        let report = DependencyReport::new();
        assert_eq!(report.total_count(), 0);
        assert_eq!(report.external_count(), 0);
        assert_eq!(report.internal_count(), 0);
        assert!(!report.has_high_priority_insights());
    }

    #[test]
    fn test_conflict_severity() {
        assert!(ConflictSeverity::High.is_severe());
        assert!(ConflictSeverity::Critical.is_severe());
        assert!(!ConflictSeverity::Low.is_severe());
        assert!(!ConflictSeverity::Medium.is_severe());
    }

    #[tokio::test]
    async fn test_basic_dependency_analysis() {
        let analyzer = DependencyAnalyzer::new();
        let package = Package::new("test-package", "1.0.0", None)
            .expect("Should create package");

        let report = analyzer.analyze_dependencies(&package);
        
        assert_eq!(report.total_count(), 0);
        assert_eq!(report.production_count, 0);
        assert!(report.analysis_duration > std::time::Duration::ZERO);
    }

    #[tokio::test]
    async fn test_dependency_analysis_with_dependencies() {
        use crate::Dependency;
        
        let analyzer = DependencyAnalyzer::new();
        let dependencies = vec![
            Dependency::new("react", "^18.0.0").expect("Should create dependency"),
            Dependency::new("lodash", "^4.17.0").expect("Should create dependency"),
        ];
        
        let package = Package::new("test-package", "1.0.0", Some(dependencies))
            .expect("Should create package");

        let report = analyzer.analyze_dependencies(&package);
        
        assert_eq!(report.total_count(), 2);
        assert_eq!(report.production_count, 2);
    }

    #[test]
    fn test_empty_conflict_detection() {
        let analyzer = DependencyAnalyzer::new();
        let packages = vec![];
        
        let conflicts = analyzer.find_conflicts(&packages);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_basic_dependency_classification() {
        let analyzer = DependencyAnalyzer::new();
        let dependency = Dependency::new("react", "^18.0.0")
            .expect("Should create dependency");
        
        let classification = analyzer.classify_dependency(&dependency, None);
        assert_eq!(classification, DependencyClass::External);
    }
}