//! Impact analysis functionality for the analyzer plugin

use crate::error::Result;
use crate::plugins::builtin::common::success_with_timing;
use crate::plugins::types::{PluginContext, PluginResult};
use std::time::Instant;

impl super::AnalyzerPlugin {
    /// Perform comprehensive impact analysis using real Git and dependency analysis
    ///
    /// Analyzes the impact of changes since a given reference using:
    /// - Real Git operations to detect changed files
    /// - Dependency analysis to understand propagation
    /// - Package change analysis for comprehensive impact assessment
    ///
    /// # Arguments
    ///
    /// * `since` - Git reference to analyze changes from (commit, tag, or branch)
    /// * `context` - Plugin context with access to monorepo services
    ///
    /// # Returns
    ///
    /// Comprehensive impact analysis including:
    /// - Changed files and packages
    /// - Dependency propagation analysis
    /// - Affected packages through dependency chains
    /// - Severity and risk assessment
    #[allow(clippy::too_many_lines)]
    pub(super) fn impact_analysis(since: &str, context: &PluginContext) -> Result<PluginResult> {
        let start_time = Instant::now();

        // Get changed files using real Git operations
        let changed_files = context
            .repository
            .get_all_files_changed_since_sha_with_status(since)
            .map_err(|e| {
            crate::error::Error::plugin(format!("Failed to get changed files since {since}: {e}"))
        })?;

        // Analyze which packages are directly affected
        let mut directly_affected = std::collections::HashSet::new();
        let mut file_changes_by_package = std::collections::HashMap::new();

        for changed_file in &changed_files {
            for package in context.packages {
                let package_path = package.path().to_string_lossy();
                if changed_file.path.starts_with(package_path.as_ref()) {
                    directly_affected.insert(package.name().to_string());

                    file_changes_by_package
                        .entry(package.name().to_string())
                        .or_insert_with(Vec::new)
                        .push(serde_json::json!({
                            "file": changed_file.path,
                            "status": match changed_file.status {
                                sublime_git_tools::GitFileStatus::Added => "added",
                                sublime_git_tools::GitFileStatus::Modified => "modified",
                                sublime_git_tools::GitFileStatus::Deleted => "deleted",
                                sublime_git_tools::GitFileStatus::Untracked => "untracked",
                            }
                        }));
                    break;
                }
            }
        }

        // Create dependency service for propagation analysis
        let file_system_service = crate::core::services::FileSystemService::new(context.root_path)
            .map_err(|e| {
                crate::error::Error::plugin(format!("Failed to create file system service: {e}"))
            })?;

        let package_service = crate::core::services::PackageDiscoveryService::new(
            context.root_path,
            &file_system_service,
            context.config_ref,
        )
        .map_err(|e| {
            crate::error::Error::plugin(format!("Failed to create package service: {e}"))
        })?;

        let mut dependency_service = crate::core::services::DependencyAnalysisService::new(
            &package_service,
            context.config_ref,
        )
        .map_err(|e| {
            crate::error::Error::plugin(format!("Failed to create dependency service: {e}"))
        })?;

        // Get packages affected through dependencies (propagation)
        let changed_package_names: Vec<String> = directly_affected.iter().cloned().collect();
        let all_affected =
            dependency_service.get_affected_packages(&changed_package_names).map_err(|e| {
                crate::error::Error::plugin(format!("Failed to get affected packages: {e}"))
            })?;

        // Separate directly vs indirectly affected
        let indirectly_affected: Vec<String> =
            all_affected.iter().filter(|pkg| !directly_affected.contains(*pkg)).cloned().collect();

        // Analyze change types and severity
        let mut change_types = std::collections::HashMap::new();
        for changed_file in &changed_files {
            let change_type = if changed_file.path.contains("package.json") {
                "dependency_change"
            } else if changed_file.path.contains(".config") || changed_file.path.contains("config")
            {
                "configuration_change"
            } else if changed_file.path.contains(".test.")
                || changed_file.path.contains("test/")
                || changed_file.path.contains("__tests__")
            {
                "test_change"
            } else if changed_file.path.contains(".md") || changed_file.path.contains("README") {
                "documentation_change"
            } else {
                "source_code_change"
            };

            *change_types.entry(change_type.to_string()).or_insert(0) += 1;
        }

        // Calculate risk assessment
        let total_packages = context.packages.len();
        let impact_percentage = if total_packages > 0 {
            #[allow(clippy::cast_precision_loss)]
            let percentage = (all_affected.len() as f64 / total_packages.max(1) as f64) * 100.0;
            percentage
        } else {
            0.0
        };

        let risk_level = match impact_percentage {
            p if p >= 50.0 => "high",
            p if p >= 20.0 => "medium",
            p if p > 0.0 => "low",
            _ => "none",
        };

        // Get commit information for better context
        let commits =
            context.repository.get_commits_since(Some(since.to_string()), &None).map_err(|e| {
                crate::error::Error::plugin(format!("Failed to get commits since {since}: {e}"))
            })?;

        let impact = serde_json::json!({
            "since": since,
            "analysis_timestamp": chrono::Utc::now().to_rfc3339(),
            "changed_files": {
                "total": changed_files.len(),
                "by_package": file_changes_by_package,
                "change_types": change_types
            },
            "affected_packages": {
                "directly_affected": directly_affected.into_iter().collect::<Vec<_>>(),
                "indirectly_affected": indirectly_affected,
                "total_affected": all_affected.len(),
                "directly_affected_count": changed_package_names.len(),
                "indirectly_affected_count": all_affected.len() - changed_package_names.len()
            },
            "impact_assessment": {
                "risk_level": risk_level,
                "impact_percentage": format!("{:.1}%", impact_percentage),
                "total_packages": total_packages,
                "propagation_factor": if changed_package_names.is_empty() {
                    0.0
                } else {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        all_affected.len() as f64 / changed_package_names.len() as f64
                    }
                }
            },
            "commit_analysis": {
                "commit_count": commits.len(),
                "recent_commits": commits.into_iter().take(5).map(|commit| serde_json::json!({
                    "hash": commit.hash[0..8].to_string(),
                    "message": commit.message.lines().next().unwrap_or("No message"),
                    "author": commit.author_name,
                    "date": commit.author_date
                })).collect::<Vec<_>>()
            },
            "recommendations": match risk_level {
                "high" => "High impact changes detected. Consider careful testing and staged deployment.",
                "medium" => "Medium impact changes. Verify affected packages and run comprehensive tests.",
                "low" => "Low impact changes. Standard testing procedures should be sufficient.",
                "none" => "No package impact detected. Changes may be documentation or configuration only.",
                _ => "Review changes carefully"
            }
        });

        Ok(success_with_timing(impact, start_time)
            .with_metadata("command", "impact-analysis")
            .with_metadata("analyzer", "builtin")
            .with_metadata("real_analysis", true)
            .with_metadata("risk_level", risk_level)
            .with_metadata("files_analyzed", changed_files.len()))
    }
}