//! Built-in generator plugin for code generation and templating
//!
//! Provides functionality for generating code, configuration files,
//! and project structures within the monorepo.

use crate::error::Result;
use crate::plugins::types::{
    MonorepoPlugin, PluginArgument, PluginArgumentType, PluginCapabilities, PluginCommand,
    PluginContext, PluginInfo, PluginResult,
};

mod package;
mod config;

/// Built-in generator plugin for code generation and templating
///
/// Provides functionality for generating code, configuration files,
/// and project structures within the monorepo.
pub struct GeneratorPlugin {
    /// Plugin name
    name: String,
    /// Plugin version
    version: String,
}

impl GeneratorPlugin {
    /// Create a new generator plugin
    pub fn new() -> Self {
        Self { 
            name: "generator".to_string(), 
            version: "1.0.0".to_string() 
        }
    }
}

impl Default for GeneratorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl MonorepoPlugin for GeneratorPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name.clone(),
            version: self.version.clone(),
            description: "Built-in code generation and templating plugin".to_string(),
            author: "Sublime Monorepo Tools".to_string(),
            capabilities: PluginCapabilities {
                commands: vec![
                    PluginCommand {
                        name: "generate-package".to_string(),
                        description: "Generate a new package from template".to_string(),
                        arguments: vec![
                            PluginArgument {
                                name: "name".to_string(),
                                description: "Package name".to_string(),
                                required: true,
                                arg_type: PluginArgumentType::String,
                                default_value: None,
                            },
                            PluginArgument {
                                name: "template".to_string(),
                                description: "Template to use".to_string(),
                                required: false,
                                arg_type: PluginArgumentType::String,
                                default_value: Some("default".to_string()),
                            },
                        ],
                        async_support: false,
                    },
                    PluginCommand {
                        name: "generate-config".to_string(),
                        description: "Generate configuration files".to_string(),
                        arguments: vec![PluginArgument {
                            name: "type".to_string(),
                            description: "Configuration type (eslint, prettier, etc.)".to_string(),
                            required: true,
                            arg_type: PluginArgumentType::String,
                            default_value: None,
                        }],
                        async_support: false,
                    },
                ],
                async_support: false,
                parallel_support: true,
                categories: vec!["generator".to_string(), "templates".to_string()],
                file_patterns: vec!["*.template".to_string(), "*.mustache".to_string()],
            },
        }
    }

    fn initialize(&mut self, _context: &PluginContext) -> Result<()> {
        log::info!("Initializing generator plugin");
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
            "generate-package" => {
                let name = args
                    .first()
                    .ok_or_else(|| crate::error::Error::plugin("Package name required"))?;
                let template = args.get(1).map_or("default", |s| s.as_str());
                Self::generate_package(name, template, context)
            }
            "generate-config" => {
                let config_type = args
                    .first()
                    .ok_or_else(|| crate::error::Error::plugin("Config type required"))?;
                Self::generate_config(config_type, context)
            }
            _ => Ok(unknown_command_error(command)),
        }
    }
}