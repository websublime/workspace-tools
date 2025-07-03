//! Built-in configurator plugin for configuration generation and project analysis
//!
//! Provides functionality for generating comprehensive monorepo configurations,
//! analyzing project structure, and providing setup recommendations.

use crate::error::Result;
use crate::plugins::types::{
    MonorepoPlugin, PluginArgument, PluginArgumentType, PluginCapabilities, PluginCommand,
    PluginContext, PluginInfo, PluginResult,
};

mod analysis;
mod generation;
mod validation;

// Re-export for internal use
use analysis::ProjectAnalysis;

/// Built-in configurator plugin for configuration generation and project analysis
///
/// Provides functionality for generating comprehensive monorepo configurations,
/// analyzing project structure, and providing setup recommendations.
pub struct ConfiguratorPlugin {
    /// Plugin name
    name: String,
    /// Plugin version
    version: String,
}

impl ConfiguratorPlugin {
    /// Create a new configurator plugin
    pub fn new() -> Self {
        Self { 
            name: "configurator".to_string(), 
            version: "1.0.0".to_string() 
        }
    }
}

impl Default for ConfiguratorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl MonorepoPlugin for ConfiguratorPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name.clone(),
            version: self.version.clone(),
            description: "Built-in configuration generation and project analysis plugin".to_string(),
            author: "Sublime Monorepo Tools".to_string(),
            capabilities: PluginCapabilities {
                commands: vec![
                    PluginCommand {
                        name: "generate-config".to_string(),
                        description: "Generate comprehensive monorepo configuration".to_string(),
                        arguments: vec![
                            PluginArgument {
                                name: "template".to_string(),
                                description: "Configuration template (basic, enterprise, performance, ci-cd, smart)".to_string(),
                                required: false,
                                arg_type: PluginArgumentType::String,
                                default_value: Some("smart".to_string()),
                            },
                            PluginArgument {
                                name: "output".to_string(),
                                description: "Output filename for configuration".to_string(),
                                required: false,
                                arg_type: PluginArgumentType::String,
                                default_value: Some("monorepo.config.toml".to_string()),
                            },
                        ],
                        async_support: false,
                    },
                    PluginCommand {
                        name: "analyze-project".to_string(),
                        description: "Analyze project structure and configuration needs".to_string(),
                        arguments: vec![],
                        async_support: false,
                    },
                    PluginCommand {
                        name: "validate-config".to_string(),
                        description: "Validate existing configuration files".to_string(),
                        arguments: vec![PluginArgument {
                            name: "config-path".to_string(),
                            description: "Path to configuration file to validate".to_string(),
                            required: false,
                            arg_type: PluginArgumentType::String,
                            default_value: Some("monorepo.config.toml".to_string()),
                        }],
                        async_support: false,
                    },
                ],
                async_support: false,
                parallel_support: false,
                categories: vec!["configurator".to_string(), "analysis".to_string(), "setup".to_string()],
                file_patterns: vec![
                    "package.json".to_string(),
                    "*.config.{js,ts,json,toml}".to_string(),
                    "package-lock.json".to_string(),
                    "yarn.lock".to_string(),
                    "pnpm-lock.yaml".to_string(),
                    "*.toml".to_string(),
                ],
            },
        }
    }

    fn initialize(&mut self, _context: &PluginContext) -> Result<()> {
        log::info!("Initializing configurator plugin");
        Ok(())
    }

    fn execute_command(
        &self,
        command: &str,
        args: &[String],
        context: &PluginContext,
    ) -> Result<PluginResult> {
        use crate::plugins::builtin::common::unknown_command_error;
        
        match command {
            "generate-config" => {
                let template = args.first().map_or("smart", |s| s.as_str());
                let output = args.get(1).map_or("monorepo.config.toml", |s| s.as_str());
                Self::generate_configuration(template, output, context)
            }
            "analyze-project" => Ok(Self::analyze_project_structure(context)),
            "validate-config" => {
                let config_path = args.first().map_or("monorepo.config.toml", |s| s.as_str());
                Ok(Self::validate_configuration(config_path, context))
            }
            _ => Ok(unknown_command_error(command)),
        }
    }
}

