//! Change detection logic for monorepo analysis

use std::collections::HashSet;
use std::path::Path;
use sublime_git_tools::GitChangedFile;
use super::change_engine::ChangeDetectionEngine;

/// Analyzes changes in the repository to determine affected packages
pub struct ChangeDetector {
    /// Root path of the monorepo
    root_path: std::path::PathBuf,
    
    /// Configurable detection engine
    engine: ChangeDetectionEngine,
}

impl ChangeDetector {
    /// Create a new change detector with default rules
    pub fn new(root_path: impl AsRef<Path>) -> Self {
        Self { 
            root_path: root_path.as_ref().to_path_buf(),
            engine: ChangeDetectionEngine::new(),
        }
    }
    
    /// Create a new change detector with custom rules from config file
    pub fn with_config_file(root_path: impl AsRef<Path>, config_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            root_path: root_path.as_ref().to_path_buf(),
            engine: ChangeDetectionEngine::from_config_file(config_path)?,
        })
    }
    
    /// Create a new change detector with custom engine
    pub fn with_engine(root_path: impl AsRef<Path>, engine: ChangeDetectionEngine) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
            engine,
        }
    }

    /// Map changed files to affected packages
    pub fn map_changes_to_packages(
        &mut self,
        changed_files: &[GitChangedFile],
        packages: &[crate::core::MonorepoPackageInfo],
    ) -> Vec<PackageChange> {
        let mut package_changes = Vec::new();

        // Group changes by package
        for package in packages {
            let mut changes_for_package = Vec::new();
            let package_path = package.relative_path();

            for changed_file in changed_files {
                let file_path = Path::new(&changed_file.path);

                // Check if the changed file is within this package
                if file_path.starts_with(package_path) {
                    changes_for_package.push(changed_file.clone());
                }
            }

            if !changes_for_package.is_empty() {
                let change_type = self.engine.determine_change_type(&changes_for_package, package);
                let significance = self.engine.analyze_significance(&changes_for_package, package);
                let suggested_bump = self.engine.suggest_version_bump(&change_type, &significance, package);

                package_changes.push(PackageChange {
                    package_name: package.name().to_string(),
                    changed_files: changes_for_package,
                    change_type,
                    significance,
                    suggested_version_bump: suggested_bump,
                });
            }
        }

        package_changes
    }

    /// Get access to the underlying engine for configuration
    pub fn engine_mut(&mut self) -> &mut ChangeDetectionEngine {
        &mut self.engine
    }
    
    /// Get read-only access to the engine
    pub fn engine(&self) -> &ChangeDetectionEngine {
        &self.engine
    }

    /// Find all packages affected by changes (including dependents)
    pub fn find_affected_packages(
        &self,
        direct_changes: &[String],
        packages: &[crate::core::MonorepoPackageInfo],
    ) -> HashSet<String> {
        let mut affected = HashSet::new();

        // Add directly changed packages
        for package_name in direct_changes {
            affected.insert(package_name.clone());

            // Find and add all dependents
            self.add_dependents(package_name, packages, &mut affected);
        }

        affected
    }

    /// Recursively add dependent packages
    fn add_dependents(
        &self,
        package_name: &str,
        packages: &[crate::core::MonorepoPackageInfo],
        affected: &mut HashSet<String>,
    ) {
        for package in packages {
            if package.workspace_package.workspace_dependencies.contains(&package_name.to_string())
            {
                let dep_name = package.name().to_string();
                if affected.insert(dep_name.clone()) {
                    // Recursively add dependents of this package
                    self.add_dependents(&dep_name, packages, affected);
                }
            }
        }
    }
}

/// Represents a change to a package
#[derive(Debug, Clone)]
pub struct PackageChange {
    /// Name of the affected package
    pub package_name: String,

    /// Files changed in this package
    pub changed_files: Vec<GitChangedFile>,

    /// Type of changes
    pub change_type: PackageChangeType,

    /// Significance of the changes
    pub significance: ChangeSignificance,

    /// Suggested version bump
    pub suggested_version_bump: VersionBumpType,
}

// Re-export types from change_rules for convenience
pub use super::change_rules::{PackageChangeType, ChangeSignificance, VersionBumpType};
