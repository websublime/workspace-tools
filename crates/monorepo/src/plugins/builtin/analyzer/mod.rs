//! Built-in analyzer plugin for code analysis and dependency tracking
//!
//! Provides functionality for analyzing code structure, dependencies,
//! and package relationships within the monorepo.

use crate::error::Result;
use crate::plugins::types::{
    MonorepoPlugin, PluginArgument, PluginArgumentType, PluginCapabilities, PluginCommand,
    PluginContext, PluginInfo, PluginResult,
};

mod dependencies;
mod cycles;
mod impact;

/// Built-in analyzer plugin for code analysis and dependency tracking
///
/// Provides functionality for analyzing code structure, dependencies,
/// and package relationships within the monorepo.
pub struct AnalyzerPlugin {
    /// Plugin name
    name: String,
    /// Plugin version
    version: String,
    /// Whether the plugin is initialized
    initialized: bool,
}

impl AnalyzerPlugin {
    /// Create a new analyzer plugin
    pub fn new() -> Self {
        Self { 
            name: "analyzer".to_string(), 
            version: "1.0.0".to_string(), 
            initialized: false 
        }
    }
}

impl Default for AnalyzerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl MonorepoPlugin for AnalyzerPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name.clone(),
            version: self.version.clone(),
            description: "Built-in code analysis and dependency tracking plugin".to_string(),
            author: "Sublime Monorepo Tools".to_string(),
            capabilities: PluginCapabilities {
                commands: vec![
                    PluginCommand {
                        name: "analyze-dependencies".to_string(),
                        description: "Analyze package dependencies and relationships".to_string(),
                        arguments: vec![PluginArgument {
                            name: "package".to_string(),
                            description: "Specific package to analyze (optional)".to_string(),
                            required: false,
                            arg_type: PluginArgumentType::String,
                            default_value: None,
                        }],
                        async_support: true,
                    },
                    PluginCommand {
                        name: "detect-cycles".to_string(),
                        description: "Detect circular dependencies in the monorepo".to_string(),
                        arguments: vec![],
                        async_support: false,
                    },
                    PluginCommand {
                        name: "impact-analysis".to_string(),
                        description: "Analyze change impact across packages".to_string(),
                        arguments: vec![PluginArgument {
                            name: "since".to_string(),
                            description: "Analyze changes since this commit/tag".to_string(),
                            required: false,
                            arg_type: PluginArgumentType::String,
                            default_value: Some("HEAD~1".to_string()),
                        }],
                        async_support: true,
                    },
                ],
                async_support: true,
                parallel_support: false,
                categories: vec!["analyzer".to_string(), "dependencies".to_string()],
                file_patterns: vec![
                    "package.json".to_string(),
                    "*.ts".to_string(),
                    "*.js".to_string(),
                ],
            },
        }
    }

    fn initialize(&mut self, _context: &PluginContext) -> Result<()> {
        log::info!("Initializing analyzer plugin with access to monorepo services");
        self.initialized = true;
        Ok(())
    }

    fn execute_command(
        &self,
        command: &str,
        args: &[String],
        context: &PluginContext,
    ) -> Result<PluginResult> {
        use crate::plugins::builtin::common::{plugin_not_initialized_error, unknown_command_error};
        
        if !self.initialized {
            return Ok(plugin_not_initialized_error());
        }

        match command {
            "analyze-dependencies" => {
                let package_filter = args.first().map(std::string::String::as_str);
                Self::analyze_dependencies(package_filter, context)
            }
            "detect-cycles" => Self::detect_cycles(context),
            "impact-analysis" => {
                let since = args.first().map_or("HEAD~1", |s| s.as_str());
                Self::impact_analysis(since, context)
            }
            _ => Ok(unknown_command_error(command)),
        }
    }
}