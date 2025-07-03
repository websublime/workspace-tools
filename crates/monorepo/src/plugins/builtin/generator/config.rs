//! Configuration file generation functionality for the generator plugin

use crate::error::Result;
use crate::plugins::builtin::common::success_with_timing;
use crate::plugins::types::{PluginContext, PluginResult};
use std::time::Instant;

impl super::GeneratorPlugin {
    /// Generate configuration files with real file creation
    ///
    /// Creates actual configuration files in the monorepo root or specified location
    /// based on the configuration type and best practices for the ecosystem.
    ///
    /// # Arguments
    ///
    /// * `config_type` - Type of configuration to generate (eslint, prettier, typescript, jest, etc.)
    /// * `context` - Plugin context with access to file system and project structure
    ///
    /// # Returns
    ///
    /// Result with details of actually created configuration files
    pub(super) fn generate_config(config_type: &str, context: &PluginContext) -> Result<PluginResult> {
        let start_time = Instant::now();
        let mut generated_files = Vec::new();

        match config_type {
            "typescript" => {
                Self::generate_typescript_config(context, &mut generated_files)?;
            }
            "gitignore" => {
                Self::generate_gitignore_config(context, &mut generated_files)?;
            }
            _ => {
                return Ok(PluginResult::error(format!(
                    "Unknown config type: {config_type}. Supported types: typescript, gitignore"
                )));
            }
        }

        let result = serde_json::json!({
            "config_type": config_type,
            "generated_files": generated_files,
            "file_count": generated_files.len(),
            "status": "successfully_generated",
            "location": context.root_path.to_string_lossy()
        });

        Ok(success_with_timing(result, start_time)
            .with_metadata("command", "generate-config")
            .with_metadata("generator", "builtin")
            .with_metadata("real_generation", true)
            .with_metadata("config_location", context.root_path.to_string_lossy()))
    }


    fn generate_typescript_config(context: &PluginContext, generated_files: &mut Vec<String>) -> Result<()> {
        let tsconfig = serde_json::json!({
            "compilerOptions": {
                "target": "ES2020",
                "lib": ["ES2020", "DOM"],
                "module": "ESNext",
                "moduleResolution": "node",
                "outDir": "./dist",
                "rootDir": "./src",
                "strict": true,
                "esModuleInterop": true,
                "allowSyntheticDefaultImports": true,
                "skipLibCheck": true,
                "forceConsistentCasingInFileNames": true,
                "declaration": true,
                "declarationMap": true,
                "sourceMap": true,
                "removeComments": true,
                "noUnusedLocals": true,
                "noUnusedParameters": true,
                "noImplicitReturns": true,
                "noFallthroughCasesInSwitch": true
            },
            "include": ["src/**/*"],
            "exclude": ["node_modules", "dist"]
        });

        let tsconfig_path = context.root_path.join("tsconfig.json");
        let tsconfig_content = serde_json::to_string_pretty(&tsconfig).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to serialize tsconfig: {e}"))
        })?;

        std::fs::write(&tsconfig_path, tsconfig_content).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to write tsconfig.json: {e}"))
        })?;
        generated_files.push("tsconfig.json".to_string());
        Ok(())
    }


    fn generate_gitignore_config(context: &PluginContext, generated_files: &mut Vec<String>) -> Result<()> {
        let gitignore_content = "# Dependencies\nnode_modules/\nnpm-debug.log*\nyarn-debug.log*\nyarn-error.log*\n\n# Runtime data\npids\n*.pid\n*.seed\n*.pid.lock\n\n# Coverage\ncoverage/\n.nyc_output/\n\n# Build outputs\ndist/\nbuild/\n*.tsbuildinfo\n\n# Environment\n.env\n.env.local\n.env.development.local\n.env.test.local\n.env.production.local\n\n# Editor\n.vscode/\n.idea/\n*.swp\n*.swo\n*~\n\n# OS\n.DS_Store\nThumbs.db\n\n# Logs\nlogs\n*.log\n\n# Cache\n.cache/\n.parcel-cache/\n";

        let gitignore_path = context.root_path.join(".gitignore");
        std::fs::write(&gitignore_path, gitignore_content).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to write .gitignore: {e}"))
        })?;
        generated_files.push(".gitignore".to_string());
        Ok(())
    }
}