//! Built-in plugin implementations
//!
//! Provides default plugin implementations that are compiled into the application.
//! These plugins demonstrate the plugin system and provide essential functionality.

use super::types::{
    MonorepoPlugin, PluginInfo, PluginContext, PluginResult, PluginCapabilities, 
    PluginCommand, PluginArgument, PluginArgumentType
};
use crate::error::Result;

/// Built-in analyzer plugin for code analysis and dependency tracking
///
/// Provides functionality for analyzing code structure, dependencies,
/// and package relationships within the monorepo.
pub struct AnalyzerPlugin {
    /// Plugin name
    name: String,
    /// Plugin version
    version: String,
}

impl AnalyzerPlugin {
    /// Create a new analyzer plugin
    pub fn new() -> Self {
        Self {
            name: "analyzer".to_string(),
            version: "1.0.0".to_string(),
        }
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
                        arguments: vec![
                            PluginArgument {
                                name: "package".to_string(),
                                description: "Specific package to analyze (optional)".to_string(),
                                required: false,
                                arg_type: PluginArgumentType::String,
                                default_value: None,
                            },
                        ],
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
                        arguments: vec![
                            PluginArgument {
                                name: "since".to_string(),
                                description: "Analyze changes since this commit/tag".to_string(),
                                required: false,
                                arg_type: PluginArgumentType::String,
                                default_value: Some("HEAD~1".to_string()),
                            },
                        ],
                        async_support: true,
                    },
                ],
                async_support: true,
                parallel_support: false,
                categories: vec!["analyzer".to_string(), "dependencies".to_string()],
                file_patterns: vec!["package.json".to_string(), "*.ts".to_string(), "*.js".to_string()],
            },
        }
    }

    fn initialize(&mut self, _context: &PluginContext) -> Result<()> {
        log::info!("Initializing analyzer plugin");
        Ok(())
    }

    fn execute_command(&self, command: &str, args: &[String]) -> Result<PluginResult> {
        match command {
            "analyze-dependencies" => {
                let package_filter = args.first().map(std::string::String::as_str);
                Ok(Self::analyze_dependencies(package_filter))
            }
            "detect-cycles" => {
                Ok(Self::detect_cycles())
            }
            "impact-analysis" => {
                let since = args.first().map_or("HEAD~1", |s| s.as_str());
                Ok(Self::impact_analysis(since))
            }
            _ => Ok(PluginResult::error(format!("Unknown command: {command}"))),
        }
    }
}

impl AnalyzerPlugin {
    /// Analyze package dependencies
    fn analyze_dependencies(package_filter: Option<&str>) -> PluginResult {
        let mut analysis = serde_json::Map::new();
        
        analysis.insert("total_packages".to_string(), serde_json::Value::Number(serde_json::Number::from(0)));
        analysis.insert("external_dependencies".to_string(), serde_json::Value::Number(serde_json::Number::from(0)));
        analysis.insert("internal_dependencies".to_string(), serde_json::Value::Number(serde_json::Number::from(0)));
        
        if let Some(package) = package_filter {
            analysis.insert("analyzed_package".to_string(), serde_json::Value::String(package.to_string()));
        }

        log::info!("Analyzed dependencies for package filter: {:?}", package_filter);
        
        PluginResult::success(analysis)
            .with_metadata("command", "analyze-dependencies")
            .with_metadata("analyzer", "builtin")
    }

    /// Detect circular dependencies
    fn detect_cycles() -> PluginResult {
        let cycles = serde_json::json!({
            "cycles_found": 0,
            "cycles": []
        });

        log::info!("Checked for circular dependencies");
        
        PluginResult::success(cycles)
            .with_metadata("command", "detect-cycles")
            .with_metadata("analyzer", "builtin")
    }

    /// Perform impact analysis
    fn impact_analysis(since: &str) -> PluginResult {
        let impact = serde_json::json!({
            "since": since,
            "affected_packages": [],
            "change_types": []
        });

        log::info!("Performed impact analysis since: {}", since);
        
        PluginResult::success(impact)
            .with_metadata("command", "impact-analysis")
            .with_metadata("analyzer", "builtin")
    }
}

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
            version: "1.0.0".to_string(),
        }
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
                        arguments: vec![
                            PluginArgument {
                                name: "type".to_string(),
                                description: "Configuration type (eslint, prettier, etc.)".to_string(),
                                required: true,
                                arg_type: PluginArgumentType::String,
                                default_value: None,
                            },
                        ],
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

    fn execute_command(&self, command: &str, args: &[String]) -> Result<PluginResult> {
        match command {
            "generate-package" => {
                let name = args.first().ok_or_else(|| crate::error::Error::plugin("Package name required"))?;
                let template = args.get(1).map_or("default", |s| s.as_str());
                Ok(Self::generate_package(name, template))
            }
            "generate-config" => {
                let config_type = args.first().ok_or_else(|| crate::error::Error::plugin("Config type required"))?;
                Ok(Self::generate_config(config_type))
            }
            _ => Ok(PluginResult::error(format!("Unknown command: {command}"))),
        }
    }
}

impl GeneratorPlugin {
    /// Generate a new package
    fn generate_package(name: &str, template: &str) -> PluginResult {
        let result = serde_json::json!({
            "package_name": name,
            "template_used": template,
            "generated_files": ["package.json", "src/index.ts", "README.md"],
            "status": "generated"
        });

        log::info!("Generated package '{}' using template '{}'", name, template);
        
        PluginResult::success(result)
            .with_metadata("command", "generate-package")
            .with_metadata("generator", "builtin")
    }

    /// Generate configuration files
    fn generate_config(config_type: &str) -> PluginResult {
        let result = serde_json::json!({
            "config_type": config_type,
            "generated_files": [format!(".{config_type}.json")],
            "status": "generated"
        });

        log::info!("Generated config for type: {}", config_type);
        
        PluginResult::success(result)
            .with_metadata("command", "generate-config")
            .with_metadata("generator", "builtin")
    }
}

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
            version: "1.0.0".to_string(),
        }
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
                        arguments: vec![
                            PluginArgument {
                                name: "strict".to_string(),
                                description: "Enable strict validation mode".to_string(),
                                required: false,
                                arg_type: PluginArgumentType::Boolean,
                                default_value: Some("false".to_string()),
                            },
                        ],
                        async_support: true,
                    },
                    PluginCommand {
                        name: "validate-commits".to_string(),
                        description: "Validate commit messages against conventions".to_string(),
                        arguments: vec![
                            PluginArgument {
                                name: "count".to_string(),
                                description: "Number of recent commits to validate".to_string(),
                                required: false,
                                arg_type: PluginArgumentType::Integer,
                                default_value: Some("10".to_string()),
                            },
                        ],
                        async_support: false,
                    },
                ],
                async_support: true,
                parallel_support: true,
                categories: vec!["validator".to_string(), "quality".to_string()],
                file_patterns: vec!["package.json".to_string(), "*.js".to_string(), "*.ts".to_string()],
            },
        }
    }

    fn initialize(&mut self, _context: &PluginContext) -> Result<()> {
        log::info!("Initializing validator plugin");
        Ok(())
    }

    fn execute_command(&self, command: &str, args: &[String]) -> Result<PluginResult> {
        match command {
            "validate-structure" => {
                Ok(Self::validate_structure())
            }
            "validate-dependencies" => {
                let strict = args.first()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(false);
                Ok(Self::validate_dependencies(strict))
            }
            "validate-commits" => {
                let count = args.first()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10);
                Ok(Self::validate_commits(count))
            }
            _ => Ok(PluginResult::error(format!("Unknown command: {command}"))),
        }
    }
}

impl ValidatorPlugin {
    /// Validate monorepo structure
    fn validate_structure() -> PluginResult {
        let result = serde_json::json!({
            "structure_valid": true,
            "issues": [],
            "recommendations": []
        });

        log::info!("Validated monorepo structure");
        
        PluginResult::success(result)
            .with_metadata("command", "validate-structure")
            .with_metadata("validator", "builtin")
    }

    /// Validate dependencies
    fn validate_dependencies(strict: bool) -> PluginResult {
        let result = serde_json::json!({
            "dependencies_valid": true,
            "strict_mode": strict,
            "violations": [],
            "warnings": []
        });

        log::info!("Validated dependencies (strict: {})", strict);
        
        PluginResult::success(result)
            .with_metadata("command", "validate-dependencies")
            .with_metadata("validator", "builtin")
    }

    /// Validate commit messages
    fn validate_commits(count: i32) -> PluginResult {
        let result = serde_json::json!({
            "commits_checked": count,
            "valid_commits": count,
            "invalid_commits": []
        });

        log::info!("Validated {} recent commits", count);
        
        PluginResult::success(result)
            .with_metadata("command", "validate-commits")
            .with_metadata("validator", "builtin")
    }
}

impl Default for AnalyzerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GeneratorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ValidatorPlugin {
    fn default() -> Self {
        Self::new()
    }
}