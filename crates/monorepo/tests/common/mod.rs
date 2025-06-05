//! Common test utilities and fixtures
//!
//! This module provides shared utilities, fixtures, and helper functions
//! for integration tests across the monorepo tools.

use std::path::Path;
use std::fs;
use serde_json::json;

/// Set up a test monorepo structure for integration tests
///
/// Creates a realistic monorepo structure with:
/// - Root package.json with workspaces
/// - Multiple packages in packages/ directory
/// - Various file types for testing change detection
pub fn setup_test_monorepo(root_path: &Path) {
    // Create root package.json
    let root_package_json = json!({
        "name": "test-monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"],
        "devDependencies": {
            "typescript": "^5.0.0",
            "jest": "^29.0.0"
        }
    });
    
    fs::write(
        root_path.join("package.json"),
        serde_json::to_string_pretty(&root_package_json).expect("Should serialize JSON")
    ).expect("Should write root package.json");
    
    // Create packages directory
    let packages_dir = root_path.join("packages");
    fs::create_dir_all(&packages_dir).expect("Should create packages directory");
    
    // Create first package: @test/core
    create_test_package(&packages_dir, "core", "1.0.0", vec!["lodash"]);
    
    // Create second package: @test/utils  
    create_test_package(&packages_dir, "utils", "1.2.0", vec!["@test/core"]);
    
    // Create third package: @test/app
    create_test_package(&packages_dir, "app", "2.0.0", vec!["@test/core", "@test/utils", "react"]);
    
    // Create configuration files
    create_config_files(root_path);
    
    // Create some source files for change detection testing
    create_source_files(&packages_dir);
}

/// Create a test package with realistic structure
fn create_test_package(packages_dir: &Path, name: &str, version: &str, dependencies: Vec<&str>) {
    let package_dir = packages_dir.join(name);
    fs::create_dir_all(&package_dir).expect("Should create package directory");
    
    // Create package.json
    let mut deps_map = serde_json::Map::new();
    for dep in dependencies {
        if dep.starts_with('@') {
            deps_map.insert(dep.to_string(), json!("^1.0.0"));
        } else {
            deps_map.insert(dep.to_string(), json!("^4.0.0"));
        }
    }
    
    let package_json = json!({
        "name": format!("@test/{name}"),
        "version": version,
        "main": "dist/index.js",
        "types": "dist/index.d.ts",
        "scripts": {
            "build": "tsc",
            "test": "jest",
            "lint": "eslint src/"
        },
        "dependencies": deps_map,
        "devDependencies": {
            "@types/node": "^20.0.0"
        }
    });
    
    fs::write(
        package_dir.join("package.json"),
        serde_json::to_string_pretty(&package_json).expect("Should serialize JSON")
    ).expect("Should write package.json");
    
    // Create src directory and basic files
    let src_dir = package_dir.join("src");
    fs::create_dir_all(&src_dir).expect("Should create src directory");
    
    // Create index.ts
    let index_content = format!(
        "// Main entry point for @test/{name}

export const {name}Version = \"{version}\";

export function get{name}Info() {{
    return {{
        name: \"@test/{name}\",
        version: {name}Version,
    }};
}}

export default get{name}Info;
"
    );
    
    fs::write(src_dir.join("index.ts"), index_content)
        .expect("Should write index.ts");
    
    // Create a utility file
    let utils_content = format!(
        "// Utilities for {name} package

export function {name}Helper(input: string): string {{
    return `{name}: ${{input}}`;
}}

export const {name}Constants = {{
    DEFAULT_VALUE: \"default\",
    MAX_RETRIES: 3,
}};
"
    );
    
    fs::write(src_dir.join("utils.ts"), utils_content)
        .expect("Should write utils.ts");
    
    // Create test directory
    let test_dir = package_dir.join("tests");
    fs::create_dir_all(&test_dir).expect("Should create tests directory");
    
    let test_content = format!(
        "import {{ get{name}Info }} from '../src/index';

describe('{name} package', () => {{
    it('should return correct package info', () => {{
        const info = get{name}Info();
        expect(info.name).toBe('@test/{name}');
        expect(info.version).toBe('{version}');
    }});
}});
"
    );
    
    fs::write(test_dir.join("index.test.ts"), test_content)
        .expect("Should write test file");
}

/// Create configuration files for testing
fn create_config_files(root_path: &Path) {
    // Create TypeScript config
    let tsconfig = json!({
        "compilerOptions": {
            "target": "ES2020",
            "module": "commonjs",
            "declaration": true,
            "outDir": "./dist",
            "rootDir": "./src",
            "strict": true,
            "esModuleInterop": true
        },
        "include": ["src/**/*"],
        "exclude": ["node_modules", "dist", "tests"]
    });
    
    fs::write(
        root_path.join("tsconfig.json"),
        serde_json::to_string_pretty(&tsconfig).expect("Should serialize JSON")
    ).expect("Should write tsconfig.json");
    
    // Create ESLint config
    let eslint_config = json!({
        "extends": ["@typescript-eslint/recommended"],
        "parser": "@typescript-eslint/parser",
        "plugins": ["@typescript-eslint"],
        "rules": {
            "@typescript-eslint/no-unused-vars": "error",
            "@typescript-eslint/explicit-function-return-type": "warn"
        }
    });
    
    fs::write(
        root_path.join(".eslintrc.json"),
        serde_json::to_string_pretty(&eslint_config).expect("Should serialize JSON")
    ).expect("Should write .eslintrc.json");
    
    // Create README
    let readme_content = r"# Test Monorepo

This is a test monorepo structure for integration testing.

## Packages

- `@test/core` - Core functionality
- `@test/utils` - Utility functions  
- `@test/app` - Main application

## Development

```bash
npm install
npm run build
npm test
```
";
    
    fs::write(root_path.join("README.md"), readme_content)
        .expect("Should write README.md");
}

/// Create various source files for change detection testing
fn create_source_files(packages_dir: &Path) {
    // Create API files in core package for testing high-significance changes
    let core_dir = packages_dir.join("core");
    let api_dir = core_dir.join("src").join("api");
    fs::create_dir_all(&api_dir).expect("Should create api directory");
    
    let api_content = r"// Public API - changes here are high significance
export interface CoreAPI {
    initialize(config: CoreConfig): Promise<void>;
    process(data: unknown): Promise<ProcessResult>;
    shutdown(): Promise<void>;
}

export interface CoreConfig {
    apiKey: string;
    endpoint: string;
    timeout?: number;
}

export interface ProcessResult {
    success: boolean;
    data?: unknown;
    error?: string;
}

export const createCoreAPI = (): CoreAPI => {
    // Implementation
    return {} as CoreAPI;
};
";
    
    fs::write(api_dir.join("core.ts"), api_content)
        .expect("Should write API file");
    
    // Create documentation files for testing low-significance changes
    let docs_dir = packages_dir.join("utils").join("docs");
    fs::create_dir_all(&docs_dir).expect("Should create docs directory");
    
    let docs_content = r"# Utils Package Documentation

## Overview

The utils package provides common utility functions.

## Functions

### utilsHelper(input: string): string

Formats input with package prefix.

### Constants

- `DEFAULT_VALUE`: Default configuration value
- `MAX_RETRIES`: Maximum number of retry attempts
";
    
    fs::write(docs_dir.join("api.md"), docs_content)
        .expect("Should write docs file");
}

/// Create a test package change for testing change detection
#[allow(dead_code)]
pub fn create_package_change(package_dir: &Path, change_type: &str) {
    match change_type {
        "source" => {
            // Modify source code
            let src_file = package_dir.join("src").join("index.ts");
            if src_file.exists() {
                let content = fs::read_to_string(&src_file).expect("Should read file");
                let modified_content = format!("{content}\n// Modified for testing\n");
                fs::write(src_file, modified_content).expect("Should write modified file");
            }
        }
        "dependencies" => {
            // Modify package.json dependencies
            let package_json_path = package_dir.join("package.json");
            if package_json_path.exists() {
                let content = fs::read_to_string(&package_json_path).expect("Should read package.json");
                let mut package_json: serde_json::Value = serde_json::from_str(&content)
                    .expect("Should parse package.json");
                
                if let Some(deps) = package_json.get_mut("dependencies") {
                    if let Some(deps_obj) = deps.as_object_mut() {
                        deps_obj.insert("new-dependency".to_string(), json!("^1.0.0"));
                    }
                }
                
                fs::write(
                    package_json_path,
                    serde_json::to_string_pretty(&package_json).expect("Should serialize JSON")
                ).expect("Should write modified package.json");
            }
        }
        "documentation" => {
            // Add documentation change
            let readme_path = package_dir.join("README.md");
            let readme_content = "# Package Documentation\n\nThis package has been updated.\n";
            fs::write(readme_path, readme_content).expect("Should write README");
        }
        _ => {
            // Default: touch a file to indicate change
            let change_file = package_dir.join("CHANGED");
            fs::write(change_file, "Modified for testing").expect("Should write change marker");
        }
    }
}

/// Verify test monorepo structure was created correctly
#[allow(dead_code)]
pub fn verify_test_structure(root_path: &Path) -> bool {
    let required_files = vec![
        "package.json",
        "packages/core/package.json",
        "packages/utils/package.json", 
        "packages/app/package.json",
        "packages/core/src/index.ts",
        "packages/utils/src/index.ts",
        "packages/app/src/index.ts",
    ];
    
    for file in required_files {
        if !root_path.join(file).exists() {
            return false;
        }
    }
    
    true
}