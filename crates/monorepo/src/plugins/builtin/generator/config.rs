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
            "eslint" => {
                Self::generate_eslint_config(context, &mut generated_files)?;
            }
            "prettier" => {
                Self::generate_prettier_config(context, &mut generated_files)?;
            }
            "typescript" => {
                Self::generate_typescript_config(context, &mut generated_files)?;
            }
            "jest" => {
                Self::generate_jest_config(context, &mut generated_files)?;
            }
            "gitignore" => {
                Self::generate_gitignore_config(context, &mut generated_files)?;
            }
            _ => {
                return Ok(PluginResult::error(format!(
                    "Unknown config type: {config_type}. Supported types: eslint, prettier, typescript, jest, gitignore"
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

    fn generate_eslint_config(context: &PluginContext, generated_files: &mut Vec<String>) -> Result<()> {
        let eslint_config = serde_json::json!({
            "env": {
                "browser": true,
                "es2021": true,
                "node": true
            },
            "extends": [
                "eslint:recommended",
                "@typescript-eslint/recommended"
            ],
            "parser": "@typescript-eslint/parser",
            "parserOptions": {
                "ecmaVersion": "latest",
                "sourceType": "module"
            },
            "plugins": ["@typescript-eslint"],
            "rules": {
                "indent": ["error", 2],
                "linebreak-style": ["error", "unix"],
                "quotes": ["error", "single"],
                "semi": ["error", "always"],
                "@typescript-eslint/no-unused-vars": "error",
                "@typescript-eslint/explicit-function-return-type": "warn"
            },
            "ignorePatterns": ["dist/", "node_modules/", "*.js"]
        });

        let eslint_path = context.root_path.join(".eslintrc.json");
        let eslint_content = serde_json::to_string_pretty(&eslint_config).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to serialize eslint config: {e}"))
        })?;

        std::fs::write(&eslint_path, eslint_content).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to write .eslintrc.json: {e}"))
        })?;
        generated_files.push(".eslintrc.json".to_string());
        Ok(())
    }

    fn generate_prettier_config(context: &PluginContext, generated_files: &mut Vec<String>) -> Result<()> {
        let prettier_config = serde_json::json!({
            "semi": true,
            "trailingComma": "es5",
            "singleQuote": true,
            "printWidth": 80,
            "tabWidth": 2,
            "useTabs": false,
            "endOfLine": "lf",
            "arrowParens": "avoid",
            "bracketSpacing": true,
            "bracketSameLine": false
        });

        let prettier_path = context.root_path.join(".prettierrc.json");
        let prettier_content = serde_json::to_string_pretty(&prettier_config).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to serialize prettier config: {e}"))
        })?;

        std::fs::write(&prettier_path, prettier_content).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to write .prettierrc.json: {e}"))
        })?;
        generated_files.push(".prettierrc.json".to_string());

        // Also generate .prettierignore
        let prettier_ignore = "dist/\nnode_modules/\n*.min.js\n*.bundle.js\ncoverage/\n.nyc_output/\n";
        let prettier_ignore_path = context.root_path.join(".prettierignore");

        std::fs::write(&prettier_ignore_path, prettier_ignore).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to write .prettierignore: {e}"))
        })?;
        generated_files.push(".prettierignore".to_string());
        Ok(())
    }

    fn generate_typescript_config(context: &PluginContext, generated_files: &mut Vec<String>) -> Result<()> {
        let tsconfig = serde_json::json!({
            "compilerOptions": {
                "target": "ES2020",
                "lib": ["ES2020", "DOM"],
                "module": "commonjs",
                "moduleResolution": "node",
                "outDir": "./dist",
                "rootDir": "./src",
                "strict": true,
                "esModuleInterop": true,
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
            "include": ["src/**/*", "tests/**/*"],
            "exclude": ["node_modules", "dist", "**/*.spec.ts", "**/*.test.ts"],
            "compileOnSave": true
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

    fn generate_jest_config(context: &PluginContext, generated_files: &mut Vec<String>) -> Result<()> {
        let jest_config = serde_json::json!({
            "preset": "ts-jest",
            "testEnvironment": "node",
            "roots": ["<rootDir>/src", "<rootDir>/tests"],
            "testMatch": ["**/__tests__/**/*.ts", "**/?(*.)+(spec|test).ts"],
            "transform": {
                "^.+\\.ts$": "ts-jest"
            },
            "collectCoverageFrom": [
                "src/**/*.ts",
                "!src/**/*.d.ts",
                "!src/**/*.test.ts",
                "!src/**/*.spec.ts"
            ],
            "coverageDirectory": "coverage",
            "coverageReporters": ["text", "lcov", "html"],
            "coverageThreshold": {
                "global": {
                    "branches": 80,
                    "functions": 80,
                    "lines": 80,
                    "statements": 80
                }
            },
            "moduleFileExtensions": ["ts", "js", "json"],
            "setupFilesAfterEnv": [],
            "verbose": true
        });

        let jest_path = context.root_path.join("jest.config.json");
        let jest_content = serde_json::to_string_pretty(&jest_config).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to serialize jest config: {e}"))
        })?;

        std::fs::write(&jest_path, jest_content).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to write jest.config.json: {e}"))
        })?;
        generated_files.push("jest.config.json".to_string());
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