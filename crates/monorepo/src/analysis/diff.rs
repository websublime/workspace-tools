//! Diff analysis for monorepo changes detection
//!
//! This module provides comprehensive diff analysis capabilities for comparing branches,
//! detecting changes, and mapping them to affected packages with significance analysis.

use crate::core::MonorepoProject;
use crate::error::Result;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use sublime_git_tools::{GitChangedFile, GitFileStatus};

// Import consistent types from changes module
use crate::changes::{ChangeSignificance, PackageChangeType};

// Import types from types/diff and changes
use super::types::diff::{
    AffectedPackagesAnalysis, BranchComparisonResult, ChangeAnalysis, ChangeAnalysisResult,
    ChangeAnalyzer, ChangeSignificanceResult, DiffAnalyzer,
};
use crate::changes::PackageChange;

impl DiffAnalyzer {
    /// Create a new diff analyzer with injected dependencies
    #[must_use]
    pub fn new(
        git_provider: Box<dyn crate::core::GitProvider>,
        package_provider: Box<dyn crate::core::PackageProvider>,
        file_system_provider: Box<dyn crate::core::FileSystemProvider>,
        package_discovery_provider: Box<dyn crate::core::interfaces::PackageDiscoveryProvider>,
    ) -> Self {
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
            git_provider,
            package_provider,
            file_system_provider,
            package_discovery_provider,
        }
    }

    /// Create a new diff analyzer from project (convenience method)
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_project(project: Arc<MonorepoProject>) -> Self {
        use crate::core::interfaces::DependencyFactory;

        // Create providers but don't store the project
        let git_provider = DependencyFactory::git_provider(Arc::clone(&project));
        let package_provider = DependencyFactory::package_provider(Arc::clone(&project));
        let file_system_provider = DependencyFactory::file_system_provider(Arc::clone(&project));
        let package_discovery_provider =
            DependencyFactory::package_discovery_provider(Arc::clone(&project));

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
            git_provider,
            package_provider,
            file_system_provider,
            package_discovery_provider,
        }
    }

    /// Create a new diff analyzer with custom analyzers
    #[must_use]
    pub fn with_analyzers(
        git_provider: Box<dyn crate::core::GitProvider>,
        package_provider: Box<dyn crate::core::PackageProvider>,
        file_system_provider: Box<dyn crate::core::FileSystemProvider>,
        package_discovery_provider: Box<dyn crate::core::interfaces::PackageDiscoveryProvider>,
        analyzers: Vec<Box<dyn ChangeAnalyzer>>,
    ) -> Self {
        Self {
            analyzers,
            git_provider,
            package_provider,
            file_system_provider,
            package_discovery_provider,
        }
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

        // Validate branches exist using git commands
        let repo_root = self.git_provider.repository_root();

        // Check if base branch exists
        let base_check = Command::new("git")
            .args(["rev-parse", "--verify", base_branch])
            .current_dir(repo_root)
            .output()
            .map_err(|e| {
                crate::error::Error::Analysis(format!("Failed to verify base branch: {e}"))
            })?;

        if !base_check.status.success() {
            return Err(crate::error::Error::Analysis(format!(
                "Base branch '{base_branch}' does not exist"
            )));
        }

        // Check if target branch exists
        let target_check = Command::new("git")
            .args(["rev-parse", "--verify", target_branch])
            .current_dir(repo_root)
            .output()
            .map_err(|e| {
                crate::error::Error::Analysis(format!("Failed to verify target branch: {e}"))
            })?;

        if !target_check.status.success() {
            return Err(crate::error::Error::Analysis(format!(
                "Target branch '{target_branch}' does not exist"
            )));
        }

        // First, get the merge base between the branches
        let merge_base = self.git_provider.get_diverged_commit(base_branch)?;

        // Get all changed files between branches
        let changed_files =
            self.git_provider.get_all_files_changed_since_sha_with_status(base_branch)?;

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
            self.git_provider.get_all_files_changed_since_sha_with_status(since_ref)?;

        // Map changes to packages
        let package_changes = self.map_changes_to_packages(&changed_files);

        // Identify affected packages with dependency analysis
        let affected_packages = self.identify_affected_packages(&changed_files)?;

        // Analyze significance of changes
        let significance_analysis = self.analyze_change_significance(&package_changes);

        Ok(ChangeAnalysis {
            from_ref: since_ref.to_string(),
            to_ref: to_ref.to_string(),
            changed_files,
            package_changes,
            affected_packages,
            significance_analysis,
        })
    }

    /// Map changed files to affected packages
    pub fn map_changes_to_packages(&self, changed_files: &[GitChangedFile]) -> Vec<PackageChange> {
        let mut package_changes: HashMap<String, PackageChangeBuilder> = HashMap::new();

        for file in changed_files {
            // Find which package this file belongs to
            let file_path = Path::new(&file.path);

            // Get the package discovery provider to access the descriptor
            let descriptor = self.package_discovery_provider.get_package_descriptor();
            // Get project root from the first package's parent directory
            let project_root = if let Some(first_pkg) = descriptor.packages().first() {
                // Get parent directory, fallback to package path if no parent exists
                let first_parent =
                    first_pkg.absolute_path.parent().unwrap_or(&first_pkg.absolute_path);
                // Get grandparent directory, fallback to first parent if no grandparent exists
                first_parent.parent().unwrap_or(first_parent)
            } else {
                Path::new(".")
            };

            // Resolve the file path relative to the project root if it's relative
            let full_file_path = if file_path.is_absolute() {
                file_path.to_path_buf()
            } else {
                project_root.join(file_path)
            };

            // Canonicalize the file path to handle symlinks like /private/var -> /var on macOS
            // For deleted files, canonicalization will fail, so we fall back to the full path
            let canonical_file_path = if file.status == sublime_git_tools::GitFileStatus::Deleted {
                // For deleted files, canonicalize the parent directory and append the filename
                if let Some(parent) = full_file_path.parent() {
                    if let Some(filename) = full_file_path.file_name() {
                        parent
                            .canonicalize()
                            .unwrap_or_else(|_| parent.to_path_buf())
                            .join(filename)
                    } else {
                        full_file_path
                    }
                } else {
                    full_file_path
                }
            } else {
                full_file_path.canonicalize().unwrap_or(full_file_path)
            };

            // Find package by manually checking canonicalized paths since find_package_for_path
            // might not handle symlinks properly
            let package = descriptor.packages().iter().find(|pkg| {
                let canonical_pkg_path =
                    pkg.absolute_path.canonicalize().unwrap_or_else(|_| pkg.absolute_path.clone());
                canonical_file_path.starts_with(&canonical_pkg_path)
            });

            if let Some(package) = package {
                let package_name = package.name.clone();

                // Get or create package change builder
                let change_builder = package_changes
                    .entry(package_name.clone())
                    .or_insert_with(|| PackageChangeBuilder::new(package_name));

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
    ) -> Result<AffectedPackagesAnalysis> {
        let direct_changes = self.map_changes_to_packages(changes);

        let mut directly_affected = std::collections::HashSet::new();
        let mut dependents_affected = std::collections::HashSet::new();
        let mut change_graph = HashMap::new();

        // Build change propagation graph
        for package_change in &direct_changes {
            directly_affected.insert(package_change.package_name.clone());

            // Find all packages that depend on this changed package
            let dependents = self.package_provider.get_dependents(&package_change.package_name);

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

        Ok(AffectedPackagesAnalysis {
            directly_affected,
            dependents_affected,
            change_propagation_graph: change_graph,
            impact_scores,
            total_affected_count,
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
                if let Some(package_info) = self.package_provider.get_package(&change.package_name)
                {
                    // Check if package has many dependents
                    if package_info.dependents.len() > 5 {
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
            if let Some(package_info) = self.package_provider.get_package(package_name) {
                #[allow(clippy::cast_precision_loss)]
                {
                    score += package_info.dependents.len() as f32 * 0.1;
                }
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
        use std::process::Command;

        // Get the repository root for running git commands
        let repo_root = self.git_provider.repository_root();

        // Get merge base between the two branches
        let merge_base_output = Command::new("git")
            .args(["merge-base", base_branch, target_branch])
            .current_dir(repo_root)
            .output()
            .map_err(|e| crate::error::Error::Analysis(format!("Failed to get merge base: {e}")))?;

        if !merge_base_output.status.success() {
            return Ok(vec![]); // No common ancestor or invalid branches
        }

        let merge_base = String::from_utf8_lossy(&merge_base_output.stdout).trim().to_string();

        // Get files changed from merge base to base branch
        let base_changes_output = Command::new("git")
            .args(["diff", "--name-only", &merge_base, base_branch])
            .current_dir(repo_root)
            .output()
            .map_err(|e| {
                crate::error::Error::Analysis(format!("Failed to get base changes: {e}"))
            })?;

        let base_changes: HashSet<String> = if base_changes_output.status.success() {
            String::from_utf8_lossy(&base_changes_output.stdout)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect()
        } else {
            HashSet::new()
        };

        // Get files changed from merge base to target branch
        let target_changes_output = Command::new("git")
            .args(["diff", "--name-only", &merge_base, target_branch])
            .current_dir(repo_root)
            .output()
            .map_err(|e| {
                crate::error::Error::Analysis(format!("Failed to get target changes: {e}"))
            })?;

        let target_changes: HashSet<String> = if target_changes_output.status.success() {
            String::from_utf8_lossy(&target_changes_output.stdout)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect()
        } else {
            HashSet::new()
        };

        // Find files that were modified in both branches
        let conflicts: Vec<String> = base_changes.intersection(&target_changes).cloned().collect();

        Ok(conflicts)
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
