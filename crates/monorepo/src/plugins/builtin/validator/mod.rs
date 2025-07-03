//! Built-in validator plugin for validation and quality assurance
//!
//! Provides functionality for validating code quality, style,
//! and adherence to monorepo policies.

use crate::error::Result;
use crate::plugins::types::{
    MonorepoPlugin, PluginArgument, PluginArgumentType, PluginCapabilities, PluginCommand,
    PluginContext, PluginInfo, PluginResult,
};

mod commits;
mod dependencies;
mod structure;

/// Built-in validator plugin for validation and quality assurance
///
/// Provides functionality for validating code quality, style,
/// and adherence to monorepo policies.
pub struct ValidatorPlugin {
    /// Plugin name
    name: String,
    /// Plugin version
    version: String,
}

impl ValidatorPlugin {
    /// Create a new validator plugin
    pub fn new() -> Self {
        Self { 
            name: "validator".to_string(), 
            version: "1.0.0".to_string() 
        }
    }
}

impl Default for ValidatorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl MonorepoPlugin for ValidatorPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name.clone(),
            version: self.version.clone(),
            description: "Built-in validation and quality assurance plugin".to_string(),
            author: "Sublime Monorepo Tools".to_string(),
            capabilities: PluginCapabilities {
                commands: vec![
                    PluginCommand {
                        name: "validate-structure".to_string(),
                        description: "Validate monorepo structure and conventions".to_string(),
                        arguments: vec![],
                        async_support: false,
                    },
                    PluginCommand {
                        name: "validate-dependencies".to_string(),
                        description: "Validate dependency constraints and versions".to_string(),
                        arguments: vec![PluginArgument {
                            name: "strict".to_string(),
                            description: "Enable strict validation mode".to_string(),
                            required: false,
                            arg_type: PluginArgumentType::Boolean,
                            default_value: Some("false".to_string()),
                        }],
                        async_support: true,
                    },
                    PluginCommand {
                        name: "validate-commits".to_string(),
                        description: "Validate commit messages against conventions".to_string(),
                        arguments: vec![PluginArgument {
                            name: "count".to_string(),
                            description: "Number of recent commits to validate".to_string(),
                            required: false,
                            arg_type: PluginArgumentType::Integer,
                            default_value: Some("10".to_string()),
                        }],
                        async_support: false,
                    },
                ],
                async_support: true,
                parallel_support: true,
                categories: vec!["validator".to_string(), "quality".to_string()],
                file_patterns: vec![
                    "package.json".to_string(),
                    "*.js".to_string(),
                    "*.ts".to_string(),
                ],
            },
        }
    }

    fn initialize(&mut self, _context: &PluginContext) -> Result<()> {
        log::info!("Initializing validator plugin");
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
            "validate-structure" => Ok(Self::validate_structure(context)),
            "validate-dependencies" => {
                let strict = args.first().and_then(|s| s.parse().ok()).unwrap_or(false);
                Self::validate_dependencies(strict, context)
            }
            "validate-commits" => {
                let count = args.first().and_then(|s| s.parse().ok()).unwrap_or(10);
                Self::validate_commits(count, context)
            }
            _ => Ok(unknown_command_error(command)),
        }
    }
}

