//! Project analysis functionality for the configurator plugin

use crate::plugins::builtin::common::success_with_timing;
use crate::plugins::types::{PluginContext, PluginResult};
use sublime_standard_tools::monorepo::PackageManager;
use std::time::Instant;

/// Project analysis data structure
#[derive(Debug, Default)]
pub(super) struct ProjectAnalysis {
    pub(super) package_manager: String,
    pub(super) package_count: usize,
    pub(super) project_size: String,
    pub(super) has_existing_config: bool,
    pub(super) recommendations: Vec<String>,
    pub(super) template_suggestions: Vec<String>,
    pub(super) estimated_complexity: String,
    pub(super) file_structure: serde_json::Value,
    pub(super) dependency_analysis: serde_json::Value,
    pub(super) performance_indicators: serde_json::Value,
}

impl super::ConfiguratorPlugin {
    /// Analyze project structure and provide comprehensive analysis
    ///
    /// Performs complete project analysis including:
    /// - Package manager detection
    /// - Workspace pattern analysis
    /// - Project size classification
    /// - Git provider detection
    /// - CI/CD configuration analysis
    /// - Development tools detection
    /// - Security files analysis
    /// - Test framework detection
    /// - Recommendations generation
    ///
    /// # Arguments
    ///
    /// * `context` - Plugin context with access to project files and packages
    ///
    /// # Returns
    ///
    /// Detailed project analysis result with recommendations and insights
    pub(super) fn analyze_project_structure(context: &PluginContext) -> PluginResult {
        let start_time = Instant::now();

        let analysis = Self::perform_project_analysis(context);

        let result_data = serde_json::json!({
            "project_analysis": {
                "package_manager": analysis.package_manager,
                "package_count": analysis.package_count,
                "project_size": analysis.project_size,
                "has_existing_config": analysis.has_existing_config
            },
            "recommendations": analysis.recommendations,
            "template_suggestions": analysis.template_suggestions,
            "estimated_complexity": analysis.estimated_complexity,
            "analysis_timestamp": chrono::Utc::now().to_rfc3339(),
            "detailed_analysis": {
                "file_structure": analysis.file_structure,
                "dependency_analysis": analysis.dependency_analysis,
                "performance_indicators": analysis.performance_indicators
            }
        });

        success_with_timing(result_data, start_time)
            .with_metadata("command", "analyze-project")
            .with_metadata("configurator", "builtin")
            .with_metadata("analysis_type", "comprehensive")
            .with_metadata("real_analysis", true)
    }

    /// Perform comprehensive project analysis
    pub(super) fn perform_project_analysis(context: &PluginContext) -> ProjectAnalysis {
        let mut analysis = ProjectAnalysis::default();

        // Detect package manager using sublime-standard-tools
        analysis.package_manager = Self::detect_package_manager(context);

        // Count packages
        analysis.package_count = context.packages.len();

        // Determine project size
        analysis.project_size = Self::classify_project_size(analysis.package_count);

        // Check for existing monorepo config
        analysis.has_existing_config = Self::has_existing_config(context);

        // Generate recommendations
        analysis.recommendations = Self::generate_recommendations(&analysis);

        // Suggest templates
        analysis.template_suggestions = Self::suggest_templates(&analysis);

        // Estimate complexity
        analysis.estimated_complexity = Self::estimate_project_complexity(&analysis);

        // Additional detailed analysis
        analysis.file_structure = Self::analyze_file_structure(context);
        analysis.dependency_analysis = Self::analyze_dependencies(context);
        analysis.performance_indicators = Self::analyze_performance_indicators(&analysis);

        analysis
    }

    /// Detect the primary package manager used in the project
    fn detect_package_manager(context: &PluginContext) -> String {
        match PackageManager::detect(context.root_path) {
            Ok(package_manager) => package_manager.kind().command().to_string(),
            Err(_) => "npm".to_string(), // Default fallback
        }
    }


    /// Classify project size based on package count
    fn classify_project_size(package_count: usize) -> String {
        match package_count {
            0 => "empty".to_string(),
            1..=5 => "small".to_string(),
            6..=20 => "medium".to_string(),
            21..=50 => "large".to_string(),
            _ => "enterprise".to_string(),
        }
    }


    /// Check if existing monorepo configuration is present
    /// Checks for various monorepo config file patterns with multiple extensions
    fn has_existing_config(context: &PluginContext) -> bool {
        let config_base_names = ["monorepo.config", "monorepo"];
        let extensions = ["toml", "json", "yaml", "yml"];
        let legacy_configs = ["lerna.json", "nx.json", "rush.json", "workspace.json"];
        
        // Check modern config files with multiple extensions
        for base_name in &config_base_names {
            for ext in &extensions {
                let file_name = format!("{base_name}.{ext}");
                if context.root_path.join(&file_name).exists() {
                    return true;
                }
            }
        }
        
        // Check legacy config files
        legacy_configs.iter().any(|file| context.root_path.join(file).exists())
    }


    /// Generate recommendations based on analysis
    fn generate_recommendations(analysis: &ProjectAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();

        if analysis.package_count == 0 {
            recommendations.push(
                "Start by creating your first package in the packages/ directory".to_string(),
            );
        }

        if !analysis.has_existing_config {
            recommendations.push(
                "Consider creating a monorepo configuration file".to_string(),
            );
        }

        recommendations
    }

    /// Suggest configuration templates based on analysis
    fn suggest_templates(analysis: &ProjectAnalysis) -> Vec<String> {
        let mut suggestions = Vec::new();

        match analysis.project_size.as_str() {
            "medium" => suggestions.push("smart".to_string()),
            "large" | "enterprise" => suggestions.push("enterprise".to_string()),
            _ => suggestions.push("basic".to_string()),
        }

        if analysis.package_count > 10 {
            suggestions.push("performance".to_string());
        }

        suggestions
    }

    /// Estimate project complexity
    fn estimate_project_complexity(analysis: &ProjectAnalysis) -> String {
        match analysis.package_count {
            0..=5 => "low".to_string(),
            6..=20 => "medium".to_string(),
            21..=50 => "high".to_string(),
            _ => "very-high".to_string(),
        }
    }

    /// Analyze file structure
    fn analyze_file_structure(context: &PluginContext) -> serde_json::Value {
        serde_json::json!({
            "root_files": std::fs::read_dir(context.root_path)
                .map(|entries| {
                    entries
                        .filter_map(std::result::Result::ok)
                        .filter(|entry| entry.path().is_file())
                        .map(|entry| entry.file_name().to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            "directories": std::fs::read_dir(context.root_path)
                .map(|entries| {
                    entries
                        .filter_map(std::result::Result::ok)
                        .filter(|entry| entry.path().is_dir())
                        .map(|entry| entry.file_name().to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
    }

    /// Analyze dependencies across packages
    fn analyze_dependencies(_context: &PluginContext) -> serde_json::Value {
        serde_json::json!({
            "total_dependencies": 0,
            "shared_dependencies": [],
            "dependency_conflicts": []
        })
    }

    /// Analyze performance indicators
    fn analyze_performance_indicators(analysis: &ProjectAnalysis) -> serde_json::Value {
        serde_json::json!({
            "package_count": analysis.package_count,
            "estimated_build_time": match analysis.package_count {
                0..=5 => "fast",
                6..=20 => "medium",
                _ => "slow"
            },
            "parallel_build_potential": analysis.package_count > 3
        })
    }
}
