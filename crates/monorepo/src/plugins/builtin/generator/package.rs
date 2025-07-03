//! Package generation functionality for the generator plugin

use crate::error::Result;
use crate::plugins::builtin::common::success_with_timing;
use crate::plugins::types::{PluginContext, PluginResult};
use std::time::Instant;

impl super::GeneratorPlugin {
    /// Generate a new package with real file creation
    ///
    /// Creates actual package files in the monorepo using the file system service
    /// and following the project's package structure conventions.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the package to generate
    /// * `template` - Template type to use for generation
    /// * `context` - Plugin context with access to file system and configuration
    ///
    /// # Returns
    ///
    /// Result with details of actually created files and package structure
    pub(super) fn generate_package(
        name: &str,
        template: &str,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        let start_time = Instant::now();

        // Validate package name
        if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Ok(PluginResult::error(format!("Invalid package name: {name}. Use alphanumeric characters, hyphens, and underscores only.")));
        }

        // Check if package already exists
        for existing_package in context.packages {
            if existing_package.name() == name {
                return Ok(PluginResult::error(format!(
                    "Package '{name}' already exists at {}",
                    existing_package.path().display()
                )));
            }
        }

        // Determine package path based on monorepo structure
        let packages_dir = context.root_path.join("packages");
        let package_path = packages_dir.join(name);

        // Create package directory
        if let Err(e) = std::fs::create_dir_all(&package_path) {
            return Ok(PluginResult::error(format!("Failed to create package directory: {e}")));
        }

        let mut generated_files = Vec::new();

        // Generate package.json based on template
        let package_json = Self::create_package_json(name, template);

        // Write package.json
        let package_json_path = package_path.join("package.json");
        let package_json_content = serde_json::to_string_pretty(&package_json).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to serialize package.json: {e}"))
        })?;

        if let Err(e) = std::fs::write(&package_json_path, package_json_content) {
            return Ok(PluginResult::error(format!("Failed to write package.json: {e}")));
        }
        generated_files.push("package.json".to_string());

        // Create src directory and main file
        let src_dir = package_path.join("src");
        if let Err(e) = std::fs::create_dir_all(&src_dir) {
            return Ok(PluginResult::error(format!("Failed to create src directory: {e}")));
        }

        // Generate main file based on template
        let (main_file, main_content) = Self::create_main_file(name, template);

        let main_file_path = src_dir.join(&main_file);
        if let Err(e) = std::fs::write(&main_file_path, main_content) {
            return Ok(PluginResult::error(format!("Failed to write main file: {e}")));
        }
        generated_files.push(format!("src/{main_file}"));

        // Generate README.md
        let readme_content = Self::create_readme(name, template);
        let readme_path = package_path.join("README.md");
        if let Err(e) = std::fs::write(&readme_path, readme_content) {
            return Ok(PluginResult::error(format!("Failed to write README.md: {e}")));
        }
        generated_files.push("README.md".to_string());

        // Generate TypeScript config if applicable
        if template == "library" || template == "app" {
            let tsconfig = Self::create_tsconfig();
            let tsconfig_path = package_path.join("tsconfig.json");
            let tsconfig_content = serde_json::to_string_pretty(&tsconfig).map_err(|e| {
                crate::error::Error::plugin(format!("Failed to serialize tsconfig.json: {e}"))
            })?;

            if let Err(e) = std::fs::write(&tsconfig_path, tsconfig_content) {
                return Ok(PluginResult::error(format!("Failed to write tsconfig.json: {e}")));
            }
            generated_files.push("tsconfig.json".to_string());
        }

        let result = serde_json::json!({
            "package_name": name,
            "template_used": template,
            "package_path": package_path.to_string_lossy(),
            "generated_files": generated_files,
            "status": "successfully_generated",
            "file_count": generated_files.len()
        });

        Ok(success_with_timing(result, start_time)
            .with_metadata("command", "generate-package")
            .with_metadata("generator", "builtin")
            .with_metadata("real_generation", true)
            .with_metadata("package_path", package_path.to_string_lossy()))
    }

    fn create_package_json(name: &str, template: &str) -> serde_json::Value {
        match template {
            "library" => serde_json::json!({
                "name": name,
                "version": "0.1.0",
                "description": format!("Generated library package: {name}"),
                "main": "dist/index.js",
                "types": "dist/index.d.ts",
                "scripts": {
                    "build": "tsc",
                    "test": "jest",
                    "lint": "eslint src/**/*.ts",
                    "clean": "rm -rf dist"
                },
                "keywords": [name],
                "author": "Generated by Sublime Monorepo Tools",
                "license": "MIT",
                "devDependencies": {
                    "typescript": "^5.0.0",
                    "@types/node": "^20.0.0",
                    "jest": "^29.0.0",
                    "eslint": "^8.0.0"
                }
            }),
            "app" => serde_json::json!({
                "name": name,
                "version": "0.1.0",
                "description": format!("Generated application package: {name}"),
                "main": "dist/app.js",
                "scripts": {
                    "build": "tsc",
                    "start": "node dist/app.js",
                    "dev": "ts-node src/app.ts",
                    "test": "jest",
                    "lint": "eslint src/**/*.ts"
                },
                "keywords": [name, "application"],
                "author": "Generated by Sublime Monorepo Tools",
                "license": "MIT",
                "dependencies": {
                    "express": "^4.18.0"
                },
                "devDependencies": {
                    "typescript": "^5.0.0",
                    "@types/node": "^20.0.0",
                    "@types/express": "^4.17.0",
                    "ts-node": "^10.0.0",
                    "jest": "^29.0.0",
                    "eslint": "^8.0.0"
                }
            }),
            _ => serde_json::json!({
                "name": name,
                "version": "0.1.0",
                "description": format!("Generated package: {name}"),
                "main": "dist/index.js",
                "scripts": {
                    "build": "tsc",
                    "test": "jest"
                },
                "keywords": [name],
                "author": "Generated by Sublime Monorepo Tools",
                "license": "MIT"
            }),
        }
    }

    fn create_main_file(name: &str, template: &str) -> (String, String) {
        match template {
            "library" => ("index.ts".to_string(), format!(
                "/**\n * {name} library\n * Generated by Sublime Monorepo Tools\n */\n\nexport function hello(): string {{\n    return 'Hello from {name}!';\n}}\n\nexport default hello;\n"
            )),
            "app" => ("app.ts".to_string(), format!(
                "/**\n * {name} application\n * Generated by Sublime Monorepo Tools\n */\n\nimport express from 'express';\n\nconst app = express();\nconst port = process.env.PORT || 3000;\n\napp.get('/', (req, res) => {{\n    res.json({{ message: 'Hello from {name}!' }});\n}});\n\napp.listen(port, () => {{\n    console.log(`{name} is running on port ${{port}}`);\n}});\n"
            )),
            _ => ("index.ts".to_string(), format!(
                "/**\n * {name}\n * Generated by Sublime Monorepo Tools\n */\n\nexport function greet(name: string): string {{\n    return `Hello, ${{name}}!`;\n}}\n\nconsole.log(greet('{name}'));\n"
            ))
        }
    }

    fn create_readme(name: &str, template: &str) -> String {
        format!(
            "# {name}\n\nGenerated package by Sublime Monorepo Tools\n\n## Template: {template}\n\n## Installation\n\n```bash\nnpm install\n```\n\n## Build\n\n```bash\nnpm run build\n```\n\n## Test\n\n```bash\nnpm test\n```\n"
        )
    }

    fn create_tsconfig() -> serde_json::Value {
        serde_json::json!({
            "compilerOptions": {
                "target": "ES2020",
                "module": "commonjs",
                "outDir": "./dist",
                "rootDir": "./src",
                "strict": true,
                "esModuleInterop": true,
                "skipLibCheck": true,
                "forceConsistentCasingInFileNames": true,
                "declaration": true,
                "declarationMap": true,
                "sourceMap": true
            },
            "include": ["src/**/*"],
            "exclude": ["node_modules", "dist"]
        })
    }
}