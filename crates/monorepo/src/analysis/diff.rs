//! Diff analysis for monorepo changes detection
//!
//! This module provides comprehensive diff analysis capabilities for comparing branches,
//! detecting changes, and mapping them to affected packages with significance analysis.

use crate::core::MonorepoProject;
use crate::error::Result;
use std::collections::HashMap;
use std::path::Path;
use sublime_git_tools::{GitChangedFile, GitFileStatus};

// Import consistent types from changes module
use crate::changes::{ChangeSignificance, PackageChangeType};

// Import types from types/diff and changes
use super::types::diff::{
    BranchComparisonResult, ChangeAnalysis, ChangeAnalysisResult,
    ChangeAnalyzer, ChangeSignificanceResult, DiffAnalyzer,
};
use crate::changes::PackageChange;

impl<'a> DiffAnalyzer<'a> {
    /// Create a new diff analyzer with direct borrowing from project
    ///
    /// Uses borrowing instead of trait objects to eliminate Arc proliferation
    /// and work with Rust ownership principles.
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to monorepo project
    ///
    /// # Returns
    ///
    /// A new diff analyzer instance
    #[must_use]
    pub fn new(project: &'a MonorepoProject) -> Self {
        // Add built-in analyzers
        let analyzers: Vec<Box<dyn ChangeAnalyzer>> = vec![
            Box::new(PackageJsonAnalyzer),
            Box::new(SourceCodeAnalyzer),
            Box::new(ConfigurationAnalyzer),
            Box::new(DocumentationAnalyzer),
            Box::new(TestAnalyzer),
        ];

        Self {
            analyzers,
            repository: project.repository(),
            packages: &project.packages,
            root_path: project.root_path(),
        }
    }

    /// Create a new diff analyzer with custom analyzers
    #[must_use]
    pub fn with_analyzers(
        project: &'a MonorepoProject,
        analyzers: Vec<Box<dyn ChangeAnalyzer>>,
    ) -> Self {
        Self {
            analyzers,
            repository: project.repository(),
            packages: &project.packages,
            root_path: project.root_path(),
        }
    }

    /// Creates a new diff analyzer from an existing MonorepoProject
    ///
    /// Convenience method that wraps the `new` constructor for backward compatibility.
    /// Uses real direct borrowing following Rust ownership principles.
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to the monorepo project
    ///
    /// # Returns
    ///
    /// A new DiffAnalyzer instance with built-in analyzers and direct borrowing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::{DiffAnalyzer, MonorepoProject};
    ///
    /// let project = MonorepoProject::new("/path/to/monorepo")?;
    /// let diff_analyzer = DiffAnalyzer::from_project(&project);
    /// ```
    #[must_use]
    pub fn from_project(project: &'a MonorepoProject) -> Self {
        Self::new(project)
    }

    /// Compare two branches and analyze the differences
    pub fn compare_branches(
        &self,
        base_branch: &str,
        target_branch: &str,
    ) -> Result<BranchComparisonResult> {
        // Validate branch names
        if base_branch.is_empty() {
            return Err(crate::error::Error::Analysis(
                "Base branch name cannot be empty".to_string(),
            ));
        }
        if target_branch.is_empty() {
            return Err(crate::error::Error::Analysis(
                "Target branch name cannot be empty".to_string(),
            ));
        }

        // Validate branches exist using git repository
        // Check if base branch exists
        if !self.repository.branch_exists(base_branch).map_err(|e| {
            crate::error::Error::Analysis(format!("Failed to verify base branch: {e}"))
        })? {
            return Err(crate::error::Error::Analysis(format!(
                "Base branch '{base_branch}' does not exist"
            )));
        }

        // Check if target branch exists
        if !self.repository.branch_exists(target_branch).map_err(|e| {
            crate::error::Error::Analysis(format!("Failed to verify target branch: {e}"))
        })? {
            return Err(crate::error::Error::Analysis(format!(
                "Target branch '{target_branch}' does not exist"
            )));
        }

        // First, get the merge base between the branches
        let merge_base = self.repository.get_diverged_commit(base_branch)?;

        // Get all changed files between branches
        let changed_files =
            self.repository.get_all_files_changed_since_sha_with_status(base_branch)?;

        // Map changes to packages
        let affected_packages_analysis = self.identify_affected_packages(&changed_files)?;

        // Check for merge conflicts (simplified)
        let conflicts = self.detect_potential_conflicts(base_branch, target_branch)?;

        Ok(BranchComparisonResult {
            base_branch: base_branch.to_string(),
            target_branch: target_branch.to_string(),
            changed_files,
            affected_packages: affected_packages_analysis.directly_affected.clone(),
            merge_base,
            conflicts,
        })
    }

    /// Detect changes since a specific reference (commit, tag, or branch)
    pub fn detect_changes_since(
        &self,
        since_ref: &str,
        until_ref: Option<&str>,
    ) -> Result<ChangeAnalysis> {
        let to_ref = until_ref.unwrap_or("HEAD");

        // Get changed files
        let changed_files =
            self.repository.get_all_files_changed_since_sha_with_status(since_ref)?;

        // Map changes to packages
        let package_changes = self.map_changes_to_packages(&changed_files);

        // Identify affected packages with dependency analysis
        let mut affected_packages = self.identify_affected_packages(&changed_files)?;

        // Analyze significance of changes
        let significance_analysis = self.analyze_change_significance(&package_changes);

        // Update the affected packages analysis with the correct refs
        affected_packages.from_ref = since_ref.to_string();
        affected_packages.to_ref = to_ref.to_string();
        affected_packages.changed_files = changed_files;
        affected_packages.package_changes = package_changes;
        affected_packages.significance_analysis = significance_analysis;

        Ok(affected_packages)
    }

    /// Map changed files to affected packages
    pub fn map_changes_to_packages(&self, changed_files: &[GitChangedFile]) -> Vec<PackageChange> {
        let mut package_changes: HashMap<String, PackageChangeBuilder> = HashMap::new();

        // Pre-canonicalize package paths once for better performance
        let canonical_packages: Vec<_> = self.packages.iter().map(|pkg| {
            let canonical_path = pkg.path().canonicalize().unwrap_or_else(|_| pkg.path().clone());
            (pkg, canonical_path)
        }).collect();

        for file in changed_files {
            // Find which package this file belongs to
            let file_path = Path::new(&file.path);

            // Use direct reference to project root
            let project_root = self.root_path;

            // Resolve the file path relative to the project root if it's relative
            let full_file_path = if file_path.is_absolute() {
                file_path.to_path_buf()
            } else {
                project_root.join(file_path)
            };

            // Optimize: Skip canonicalization for most cases, use string prefix matching first
            let package = canonical_packages.iter().find(|(pkg, canonical_pkg_path)| {
                // Fast path: try string-based prefix matching first (works for most cases)
                if file.path.starts_with(pkg.path().to_string_lossy().as_ref()) {
                    return true;
                }
                
                // Fallback: use canonicalized paths for edge cases (symlinks, etc.)
                let canonical_file_path = if file.status == sublime_git_tools::GitFileStatus::Deleted {
                    // For deleted files, canonicalize the parent directory and append the filename
                    if let Some(parent) = full_file_path.parent() {
                        if let Some(filename) = full_file_path.file_name() {
                            parent
                                .canonicalize()
                                .unwrap_or_else(|_| parent.to_path_buf())
                                .join(filename)
                        } else {
                            full_file_path.clone()
                        }
                    } else {
                        full_file_path.clone()
                    }
                } else {
                    full_file_path.canonicalize().unwrap_or(full_file_path.clone())
                };
                
                canonical_file_path.starts_with(canonical_pkg_path)
            }).map(|(pkg, _)| *pkg);

            if let Some(package) = package {
                let package_name = package.name();

                // Get or create package change builder (optimized to avoid clone in hot path)
                let change_builder = package_changes
                    .entry(package_name.to_string())
                    .or_insert_with(|| PackageChangeBuilder::new(package_name.to_string()));

                // Add the file change
                change_builder.add_file_change(file.clone());

                // Determine change type and significance using analyzers
                for analyzer in &self.analyzers {
                    if analyzer.can_analyze(&file.path) {
                        let analysis = analyzer.analyze_change(file);
                        change_builder.apply_analysis(analysis);
                    }
                }
            }
        }

        // Convert builders to final PackageChange objects
        package_changes.into_values().map(PackageChangeBuilder::build).collect()
    }

    /// Identify all affected packages including dependents
    pub fn identify_affected_packages(
        &self,
        changes: &[GitChangedFile],
    ) -> Result<ChangeAnalysis> {
        let direct_changes = self.map_changes_to_packages(changes);

        let mut directly_affected = std::collections::HashSet::new();
        let mut dependents_affected = std::collections::HashSet::new();
        let mut change_graph = HashMap::new();

        // Build change propagation graph
        for package_change in &direct_changes {
            directly_affected.insert(package_change.package_name.clone());

            // Find all packages that depend on this changed package
            let dependents = self.get_dependents(&package_change.package_name);

            for dependent_pkg in dependents {
                let dependent_name = dependent_pkg.name().to_string();
                if !directly_affected.contains(&dependent_name) {
                    dependents_affected.insert(dependent_name.clone());
                }

                // Record the dependency relationship in the change graph
                change_graph
                    .entry(package_change.package_name.clone())
                    .or_insert_with(Vec::new)
                    .push(dependent_name);
            }
        }

        // Convert HashSets to Vecs for the final result
        let directly_affected: Vec<String> = directly_affected.into_iter().collect();
        let dependents_affected: Vec<String> = dependents_affected.into_iter().collect();

        // Calculate impact score based on dependency depth and breadth
        let impact_scores = self.calculate_impact_scores(&directly_affected, &dependents_affected);

        let total_affected_count = directly_affected.len() + dependents_affected.len();

        Ok(ChangeAnalysis {
            from_ref: "HEAD".to_string(),
            to_ref: "HEAD".to_string(),
            changed_files: changes.to_vec(),
            package_changes: direct_changes,
            directly_affected,
            dependents_affected,
            change_propagation_graph: change_graph,
            impact_scores,
            total_affected_count,
            significance_analysis: Vec::new(),
        })
    }

    /// Analyze the significance of package changes
    #[must_use]
    pub fn analyze_change_significance(
        &self,
        package_changes: &[PackageChange],
    ) -> Vec<ChangeSignificanceResult> {
        package_changes
            .iter()
            .map(|change| {
                let mut significance = change.significance;
                let mut reasons = Vec::new();

                // Enhance significance based on package role
                if let Some(package_info) =
                    self.packages.iter().find(|pkg| pkg.name() == change.package_name)
                {
                    // Check if package has many dependents
                    let dependents = self.get_dependents(&change.package_name);
                    if dependents.len() > 5 {
                        significance = significance.elevate();
                        reasons.push("Package has many dependents".to_string());
                    }

                    // Check if package is a core/shared library
                    if package_info.name().contains("core")
                        || package_info.name().contains("shared")
                        || package_info.name().contains("utils")
                    {
                        significance = significance.elevate();
                        reasons.push("Core/shared library package".to_string());
                    }

                    // Check version status
                    match &package_info.version_status {
                        crate::core::VersionStatus::Dirty => {
                            significance = significance.elevate();
                            reasons.push("Package has uncommitted changes".to_string());
                        }
                        crate::core::VersionStatus::Snapshot { .. } => {
                            reasons.push("Package is in snapshot mode".to_string());
                        }
                        _ => {}
                    }
                }

                // Analyze change patterns
                let breaking_change_indicators =
                    ["BREAKING", "breaking", "Breaking", "API", "interface", "contract"];

                for file_change in &change.changed_files {
                    for indicator in &breaking_change_indicators {
                        if file_change.path.contains(indicator) {
                            significance = ChangeSignificance::High;
                            reasons.push(format!(
                                "File path contains breaking change indicator: {indicator}"
                            ));
                            break;
                        }
                    }
                }

                ChangeSignificanceResult {
                    package_name: change.package_name.clone(),
                    original_significance: change.significance,
                    final_significance: significance,
                    reasons,
                    suggested_version_bump: Self::suggest_version_bump(
                        significance,
                        change.change_type,
                    ),
                }
            })
            .collect()
    }

    /// Suggest appropriate version bump based on change significance and type
    fn suggest_version_bump(
        significance: ChangeSignificance,
        change_type: PackageChangeType,
    ) -> crate::config::VersionBumpType {
        use crate::config::VersionBumpType;

        match (significance, change_type) {
            (ChangeSignificance::High, _) => VersionBumpType::Major,
            (
                ChangeSignificance::Medium,
                PackageChangeType::SourceCode | PackageChangeType::Dependencies,
            ) => VersionBumpType::Minor,
            (
                ChangeSignificance::Low,
                PackageChangeType::SourceCode | PackageChangeType::Dependencies,
            )
            | (
                _,
                PackageChangeType::Documentation
                | PackageChangeType::Tests
                | PackageChangeType::Configuration,
            ) => VersionBumpType::Patch,
        }
    }

    /// Calculate impact scores for affected packages
    fn calculate_impact_scores(
        &self,
        directly_affected: &[String],
        dependents_affected: &[String],
    ) -> HashMap<String, f32> {
        let mut scores = HashMap::new();

        // Direct changes get base score
        for package_name in directly_affected {
            let mut score = 1.0;

            // Boost score based on number of dependents
            let dependents = self.get_dependents(package_name);
            #[allow(clippy::cast_precision_loss)]
            {
                score += dependents.len() as f32 * 0.1;
            }

            scores.insert(package_name.clone(), score);
        }

        // Dependent changes get lower score
        for package_name in dependents_affected {
            scores.insert(package_name.clone(), 0.5);
        }

        scores
    }

    /// Detect potential merge conflicts between branches
    fn detect_potential_conflicts(
        &self,
        base_branch: &str,
        target_branch: &str,
    ) -> Result<Vec<String>> {
        use std::collections::HashSet;

        // Get merge base between the two branches using git repository
        let merge_base = self.repository.get_merge_base(base_branch, target_branch)
            .map_err(|e| crate::error::Error::Analysis(format!("Failed to get merge base: {e}")))?;

        // Get files changed from merge base to base branch
        let base_changes = self.repository.get_files_changed_between(&merge_base, base_branch)
            .map_err(|e| crate::error::Error::Analysis(format!("Failed to get base changes: {e}")))?;
        
        let base_changes: HashSet<String> = base_changes.into_iter().map(|f| f.path).collect();

        // Get files changed from merge base to target branch
        let target_changes = self.repository.get_files_changed_between(&merge_base, target_branch)
            .map_err(|e| crate::error::Error::Analysis(format!("Failed to get target changes: {e}")))?;
            
        let target_changes: HashSet<String> = target_changes.into_iter().map(|f| f.path).collect();

        // Find files that were modified in both branches
        let conflicts: Vec<String> = base_changes.intersection(&target_changes).cloned().collect();

        Ok(conflicts)
    }

    /// Get dependents of a package
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to find dependents for
    ///
    /// # Returns
    ///
    /// Vector of packages that depend on the given package
    fn get_dependents(&self, package_name: &str) -> Vec<&crate::core::MonorepoPackageInfo> {
        if let Some(package) = self.packages.iter().find(|pkg| pkg.name() == package_name) {
            // Use the dependents field from the package info to find dependent packages
            package
                .dependents
                .iter()
                .filter_map(|dependent_name| {
                    self.packages.iter().find(|pkg| pkg.name() == dependent_name)
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// Helper for building `PackageChange` objects
struct PackageChangeBuilder {
    package_name: String,
    changed_files: Vec<GitChangedFile>,
    change_types: Vec<PackageChangeType>,
    significance: ChangeSignificance,
    contexts: Vec<String>,
}

impl PackageChangeBuilder {
    fn new(package_name: String) -> Self {
        Self {
            package_name,
            changed_files: Vec::new(),
            change_types: Vec::new(),
            significance: ChangeSignificance::Low,
            contexts: Vec::new(),
        }
    }

    fn add_file_change(&mut self, file: GitChangedFile) {
        self.changed_files.push(file);
    }

    fn apply_analysis(&mut self, analysis: ChangeAnalysisResult) {
        if !self.change_types.contains(&analysis.change_type) {
            self.change_types.push(analysis.change_type);
        }

        if analysis.significance > self.significance {
            self.significance = analysis.significance;
        }

        self.contexts.extend(analysis.context);
    }

    fn build(self) -> PackageChange {
        // Determine primary change type - prioritize most specific types first
        let change_type = if self.change_types.contains(&PackageChangeType::Dependencies) {
            PackageChangeType::Dependencies
        } else if self.change_types.contains(&PackageChangeType::Tests) {
            // Tests should take priority over SourceCode for test files
            PackageChangeType::Tests
        } else if self.change_types.contains(&PackageChangeType::SourceCode) {
            PackageChangeType::SourceCode
        } else if self.change_types.contains(&PackageChangeType::Configuration) {
            PackageChangeType::Configuration
        } else {
            PackageChangeType::Documentation
        };

        let suggested_version_bump =
            DiffAnalyzer::suggest_version_bump(self.significance, change_type);

        // Build metadata from contexts and analysis
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("contexts".to_string(), self.contexts.join(", "));
        metadata.insert("total_files".to_string(), self.changed_files.len().to_string());
        metadata.insert("change_types_analyzed".to_string(), format!("{:?}", self.change_types));

        PackageChange {
            package_name: self.package_name,
            change_type,
            significance: self.significance,
            changed_files: self.changed_files,
            suggested_version_bump,
            metadata,
        }
    }
}

// Built-in analyzers

/// Analyzer for package.json changes
struct PackageJsonAnalyzer;

impl ChangeAnalyzer for PackageJsonAnalyzer {
    fn can_analyze(&self, file_path: &str) -> bool {
        file_path.ends_with("package.json")
    }

    fn analyze_change(&self, _change: &GitChangedFile) -> ChangeAnalysisResult {
        ChangeAnalysisResult {
            change_type: PackageChangeType::Dependencies,
            significance: ChangeSignificance::Medium,
            context: vec!["Package.json change may affect dependencies".to_string()],
        }
    }
}

/// Analyzer for source code changes
struct SourceCodeAnalyzer;

impl ChangeAnalyzer for SourceCodeAnalyzer {
    fn can_analyze(&self, file_path: &str) -> bool {
        let source_extensions =
            [".js", ".ts", ".mts", ".jsx", ".tsx", ".mjs", ".cjs", ".vue", ".svelte"];
        source_extensions.iter().any(|ext| file_path.ends_with(ext))
    }

    fn analyze_change(&self, change: &GitChangedFile) -> ChangeAnalysisResult {
        let significance = match change.status {
            GitFileStatus::Added | GitFileStatus::Deleted => ChangeSignificance::Medium,
            GitFileStatus::Modified | GitFileStatus::Untracked => ChangeSignificance::Low,
        };

        ChangeAnalysisResult {
            change_type: PackageChangeType::SourceCode,
            significance,
            context: vec![format!(
                "Source code {} in {}",
                match change.status {
                    GitFileStatus::Added => "added",
                    GitFileStatus::Modified => "modified",
                    GitFileStatus::Deleted => "deleted",
                    GitFileStatus::Untracked => "untracked",
                },
                change.path
            )],
        }
    }
}

/// Analyzer for configuration changes
struct ConfigurationAnalyzer;

impl ChangeAnalyzer for ConfigurationAnalyzer {
    fn can_analyze(&self, file_path: &str) -> bool {
        let config_files = [
            ".json",
            ".yaml",
            ".yml",
            ".toml",
            ".ini",
            ".env",
            "tsconfig.json",
            "babel.config.",
            "webpack.config.",
            "rollup.config.",
            "rolldown.config.",
            "vite.config.",
            "jest.config.",
            ".eslintrc",
            ".prettierrc",
            "Dockerfile",
        ];
        config_files.iter().any(|pattern| file_path.contains(pattern))
    }

    fn analyze_change(&self, _change: &GitChangedFile) -> ChangeAnalysisResult {
        ChangeAnalysisResult {
            change_type: PackageChangeType::Configuration,
            significance: ChangeSignificance::Low,
            context: vec!["Configuration file change".to_string()],
        }
    }
}

/// Analyzer for documentation changes
struct DocumentationAnalyzer;

impl ChangeAnalyzer for DocumentationAnalyzer {
    fn can_analyze(&self, file_path: &str) -> bool {
        let doc_extensions = [".md", ".rst", ".txt", ".adoc"];
        doc_extensions.iter().any(|ext| file_path.ends_with(ext))
            || file_path.contains("README")
            || file_path.contains("CHANGELOG")
            || file_path.contains("docs/")
    }

    fn analyze_change(&self, _change: &GitChangedFile) -> ChangeAnalysisResult {
        ChangeAnalysisResult {
            change_type: PackageChangeType::Documentation,
            significance: ChangeSignificance::Low,
            context: vec!["Documentation change".to_string()],
        }
    }
}

/// Analyzer for test changes
struct TestAnalyzer;

impl ChangeAnalyzer for TestAnalyzer {
    fn can_analyze(&self, file_path: &str) -> bool {
        file_path.contains("test")
            || file_path.contains("spec")
            || file_path.contains("__tests__")
            || file_path.contains("tests")
            || file_path.ends_with(".test.js")
            || file_path.ends_with(".test.mjs")
            || file_path.ends_with(".test.cjs")
            || file_path.ends_with(".test.ts")
            || file_path.ends_with(".test.mts")
            || file_path.ends_with(".spec.js")
            || file_path.ends_with(".spec.mjs")
            || file_path.ends_with(".spec.cjs")
            || file_path.ends_with(".spec.ts")
            || file_path.ends_with(".spec.mts")
    }

    fn analyze_change(&self, _change: &GitChangedFile) -> ChangeAnalysisResult {
        ChangeAnalysisResult {
            change_type: PackageChangeType::Tests,
            significance: ChangeSignificance::Low,
            context: vec!["Test file change".to_string()],
        }
    }
}
