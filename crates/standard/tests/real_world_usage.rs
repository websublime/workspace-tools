//! # Real-World Usage Examples: Simple Repository and Monorepo Analysis
//!
//! ## What
//! This test demonstrates practical real-world usage scenarios of the sublime-standard-tools crate.
//! It showcases two main scenarios: analyzing a simple Node.js repository and a complex monorepo,
//! including error recovery scenarios like missing lock files.
//!
//! ## How
//! The test creates realistic project structures using temporary directories, then uses the crate's
//! APIs to detect project types, analyze package structures, validate configurations, and execute
//! scripts. It demonstrates error handling and recovery patterns that developers encounter in practice.
//!
//! ## Why
//! These examples serve as practical guidance for developers using the crate, showing clear patterns
//! for project detection, monorepo analysis, command execution, and error handling in real scenarios.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::print_stdout)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::unused_self)]
#![allow(dead_code)]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

use sublime_standard_tools::{
    command::{CommandBuilder, DefaultCommandExecutor, Executor},
    error::{Error, Result},
    filesystem::{AsyncFileSystem, FileSystemManager, NodePathKind, PathExt},
    monorepo::{MonorepoDetector, MonorepoDetectorTrait},
    node::PackageManager,
    project::{ProjectDetector, ProjectManager},
};

use tempfile::TempDir;

/// Example analyzer for simple Node.js repositories
#[derive(Debug)]
struct SimpleRepoAnalyzer {
    fs: FileSystemManager,
    executor: DefaultCommandExecutor,
    project_detector: ProjectDetector,
    project_manager: ProjectManager,
}

/// Example analyzer for monorepo structures
#[derive(Debug)]
struct MonorepoAnalyzer {
    fs: FileSystemManager,
    executor: DefaultCommandExecutor,
    monorepo_detector: MonorepoDetector,
}

/// Analysis results for a simple repository
#[derive(Debug)]
struct SimpleRepoInfo {
    name: String,
    version: String,
    root_path: PathBuf,
    package_manager: PackageManager,
    has_typescript: bool,
    scripts: Vec<String>,
    dependencies: Vec<String>,
    dev_dependencies: Vec<String>,
}

/// Analysis results for a monorepo
#[derive(Debug)]
struct MonorepoInfo {
    name: String,
    root_path: PathBuf,
    package_manager: PackageManager,
    workspace_packages: Vec<WorkspacePackageInfo>,
    dependency_graph: HashMap<String, Vec<String>>,
}

/// Information about a workspace package
#[derive(Clone, Debug)]
struct WorkspacePackageInfo {
    name: String,
    version: String,
    path: PathBuf,
    scripts: Vec<String>,
    dependencies: Vec<String>,
    workspace_dependencies: Vec<String>,
}

impl SimpleRepoAnalyzer {
    /// Creates a new simple repository analyzer
    fn new() -> Self {
        Self {
            fs: FileSystemManager::new(),
            executor: DefaultCommandExecutor::new(),
            project_detector: ProjectDetector::new(),
            project_manager: ProjectManager::new(),
        }
    }

    /// Creates a new simple repository analyzer with project-specific configuration
    async fn new_with_project_config(project_path: &Path) -> Result<Self> {
        Ok(Self {
            fs: FileSystemManager::new_with_project_config(project_path).await?,
            executor: DefaultCommandExecutor::new_with_project_config(project_path).await?,
            project_detector: ProjectDetector::new(),
            project_manager: ProjectManager::new(),
        })
    }

    /// Analyzes a simple Node.js repository
    async fn analyze_simple_repo(&self, path: &Path) -> Result<SimpleRepoInfo> {
        println!("🔍 Analyzing simple repository at: {}", path.display());
        
        // Demonstrate configuration usage
        if self.fs.exists(&path.join("repo.config.toml")).await {
            println!("📋 Found repo.config.toml - using project-specific configuration");
        } else {
            println!("📋 No repo.config.toml found - using default configuration");
        }

        // Step 1: Detect project type
        let project = self.project_detector.detect(path, None).await?;
        let project_info = project.as_project_info();
        
        println!("📦 Detected project type: {}", project_info.kind().name());

        // Step 2: Validate project structure
        let validation_config = None;
        let project_descriptor = self.project_manager.create_project(path, validation_config).await?;
        let validation_status = project_descriptor.as_project_info().validation_status();
        
        println!("✅ Project validation status: {:?}", validation_status);

        // Step 3: Read and parse package.json
        let package_json_path = path.join("package.json");
        if !self.fs.exists(&package_json_path).await {
            return Err(Error::operation("package.json not found in repository"));
        }

        let package_content = self.fs.read_file_string(&package_json_path).await?;
        let package_json: serde_json::Value = serde_json::from_str(&package_content)
            .map_err(|e| Error::operation(format!("Invalid package.json: {}", e)))?;

        // Step 4: Extract package information
        let name = package_json.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let version = package_json.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        // Step 5: Detect package manager
        let package_manager = PackageManager::detect(path)?;
        println!("📋 Using package manager: {}", package_manager.kind().command());

        // Step 6: Check for TypeScript
        let has_typescript = self.fs.exists(&path.join("tsconfig.json")).await ||
            package_json.get("devDependencies")
                .and_then(|deps| deps.get("typescript"))
                .is_some();

        // Step 7: Extract scripts
        let scripts = package_json.get("scripts")
            .and_then(|s| s.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();

        // Step 8: Extract dependencies
        let dependencies = package_json.get("dependencies")
            .and_then(|deps| deps.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();

        let dev_dependencies = package_json.get("devDependencies")
            .and_then(|deps| deps.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();

        Ok(SimpleRepoInfo {
            name,
            version,
            root_path: path.to_path_buf(),
            package_manager,
            has_typescript,
            scripts,
            dependencies,
            dev_dependencies,
        })
    }

    /// Demonstrates running scripts in a simple repository
    async fn run_scripts(&self, repo_info: &SimpleRepoInfo, scripts_to_run: &[&str]) -> Result<Vec<(String, bool)>> {
        println!("🚀 Running scripts in simple repository...");

        let mut results = Vec::new();

        for script_name in scripts_to_run {
            if repo_info.scripts.contains(&script_name.to_string()) {
                println!("  Running script: {}", script_name);
                
                let command = CommandBuilder::new(repo_info.package_manager.kind().command())
                    .arg("run")
                    .arg(*script_name)
                    .current_dir(&repo_info.root_path)
                    .timeout(Duration::from_secs(30))
                    .build();

                match self.executor.execute(command).await {
                    Ok(output) => {
                        let success = output.success();
                        println!("    ✅ Script '{}' completed (success: {})", script_name, success);
                        if !success {
                            println!("    Output: {}", output.stderr().trim());
                        }
                        results.push((script_name.to_string(), success));
                    }
                    Err(e) => {
                        println!("    ❌ Script '{}' failed: {}", script_name, e);
                        results.push((script_name.to_string(), false));
                    }
                }
            } else {
                println!("  ⚠️ Script '{}' not found", script_name);
                results.push((script_name.to_string(), false));
            }
        }

        Ok(results)
    }
}

impl MonorepoAnalyzer {
    /// Creates a new monorepo analyzer
    fn new() -> Self {
        Self {
            fs: FileSystemManager::new(),
            executor: DefaultCommandExecutor::new(),
            monorepo_detector: MonorepoDetector::new(),
        }
    }

    /// Creates a new monorepo analyzer with project-specific configuration
    async fn new_with_project_config(project_path: &Path) -> Result<Self> {        
        Ok(Self {
            fs: FileSystemManager::new_with_project_config(project_path).await?,
            executor: DefaultCommandExecutor::new_with_project_config(project_path).await?,
            monorepo_detector: MonorepoDetector::new_with_project_config(project_path).await?,
        })
    }

    /// Analyzes a monorepo structure
    async fn analyze_monorepo(&self, path: &Path) -> Result<MonorepoInfo> {
        println!("🔍 Analyzing monorepo at: {}", path.display());
        
        // Demonstrate configuration usage
        if self.fs.exists(&path.join("repo.config.toml")).await {
            println!("📋 Found repo.config.toml - using monorepo-specific configuration");
            println!("    Custom workspace fields: [@scope/] pattern enabled");
            println!("    Extended workspace patterns and higher concurrency configured");
        } else {
            println!("📋 No repo.config.toml found - using default configuration");
        }

        // Step 1: Check if this is actually a monorepo
        let monorepo_kind = self.monorepo_detector.is_monorepo_root(path).await?
            .ok_or_else(|| Error::operation("Directory is not a monorepo"))?;

        println!("📦 Detected monorepo type: {}", monorepo_kind.name());

        // Step 2: Detect full monorepo structure
        let monorepo_descriptor = self.monorepo_detector.detect_monorepo(path).await?;
        let packages = monorepo_descriptor.packages();

        println!("📋 Found {} workspace packages", packages.len());

        // Step 3: Detect package manager
        let package_manager = PackageManager::detect(path)?;
        println!("📋 Using package manager: {}", package_manager.kind().command());

        // Step 4: Read root package.json for monorepo name
        let root_package_path = path.join("package.json");
        let root_package_content = self.fs.read_file_string(&root_package_path).await?;
        let root_package_json: serde_json::Value = serde_json::from_str(&root_package_content)
            .map_err(|e| Error::operation(format!("Invalid root package.json: {}", e)))?;

        let monorepo_name = root_package_json.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown-monorepo")
            .to_string();

        // Step 5: Analyze each workspace package
        let mut workspace_packages = Vec::new();
        
        for package in packages {
            let package_info = self.analyze_workspace_package(package).await?;
            workspace_packages.push(package_info);
        }

        // Step 6: Build dependency graph
        let raw_graph = monorepo_descriptor.get_dependency_graph();
        let dependency_graph: HashMap<String, Vec<String>> = raw_graph
            .into_iter()
            .map(|(pkg, deps)| (pkg.to_string(), deps.iter().map(|d| d.name.clone()).collect()))
            .collect();

        Ok(MonorepoInfo {
            name: monorepo_name,
            root_path: path.to_path_buf(),
            package_manager,
            workspace_packages,
            dependency_graph,
        })
    }

    /// Analyzes a single workspace package
    async fn analyze_workspace_package(&self, package: &sublime_standard_tools::monorepo::WorkspacePackage) -> Result<WorkspacePackageInfo> {
        let package_json_path = package.absolute_path.join("package.json");
        let package_content = self.fs.read_file_string(&package_json_path).await?;
        let package_json: serde_json::Value = serde_json::from_str(&package_content)
            .map_err(|e| Error::operation(format!("Invalid package.json in {}: {}", package.name, e)))?;


        // Extract scripts
        let scripts = package_json.get("scripts")
            .and_then(|s| s.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();

        // Extract dependencies
        let dependencies = package_json.get("dependencies")
            .and_then(|deps| deps.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();

        Ok(WorkspacePackageInfo {
            name: package.name.clone(),
            version: package.version.clone(),
            path: package.location.clone(),
            scripts,
            dependencies,
            workspace_dependencies: package.workspace_dependencies.clone(),
        })
    }

    /// Demonstrates running scripts across monorepo packages
    async fn run_workspace_scripts(&self, monorepo_info: &MonorepoInfo, script_name: &str) -> Result<Vec<(String, bool)>> {
        println!("🚀 Running '{}' script across workspace packages...", script_name);

        let mut results = Vec::new();

        for package in &monorepo_info.workspace_packages {
            if package.scripts.contains(&script_name.to_string()) {
                println!("  Running '{}' for package: {}", script_name, package.name);
                
                let command = CommandBuilder::new(monorepo_info.package_manager.kind().command())
                    .arg("run")
                    .arg(script_name)
                    .current_dir(&monorepo_info.root_path.join(&package.path))
                    .timeout(Duration::from_secs(60))
                    .build();

                match self.executor.execute(command).await {
                    Ok(output) => {
                        let success = output.success();
                        println!("    ✅ '{}' completed for {} (success: {})", script_name, package.name, success);
                        if !output.stdout().trim().is_empty() {
                            println!("    Output: {}", output.stdout().trim());
                        }
                        results.push((package.name.clone(), success));
                    }
                    Err(e) => {
                        println!("    ❌ '{}' failed for {}: {}", script_name, package.name, e);
                        results.push((package.name.clone(), false));
                    }
                }
            } else {
                println!("  ⚠️ Script '{}' not found in package {}", script_name, package.name);
            }
        }

        Ok(results)
    }

    /// Demonstrates error recovery scenario: missing lock file
    async fn simulate_missing_lock_file_recovery(&self, path: &Path) -> Result<()> {
        println!("🔧 Simulating missing lock file recovery scenario...");

        let lock_file_path = path.join("pnpm-lock.yaml");

        // Step 1: Remove lock file to simulate missing state
        if self.fs.exists(&lock_file_path).await {
            println!("  Temporarily removing lock file to simulate missing state...");
            self.fs.remove(&lock_file_path).await?;
        }

        // Step 2: Try to detect package manager (should still work)
        match PackageManager::detect(path) {
            Ok(pm) => {
                println!("  ✅ Package manager detection still works: {}", pm.kind().command());
            }
            Err(e) => {
                println!("  ⚠️ Package manager detection affected by missing lock file: {}", e);
            }
        }

        // Step 3: Try to analyze monorepo (should work but may show warnings)
        match self.analyze_monorepo(path).await {
            Ok(_) => {
                println!("  ✅ Monorepo analysis works even without lock file");
            }
            Err(e) => {
                println!("  ⚠️ Monorepo analysis affected: {}", e);
            }
        }

        // Step 4: Simulate running install to create lock file
        println!("  Running 'pnpm install' to recreate lock file...");
        
        let install_command = CommandBuilder::new("echo")
            .arg("Simulating: pnpm install completed successfully")
            .current_dir(path)
            .build();

        match self.executor.execute(install_command).await {
            Ok(output) => {
                println!("  ✅ Install simulation: {}", output.stdout().trim());
                
                // Create a mock lock file
                self.fs.write_file_string(&lock_file_path, "# Mock pnpm lock file\nlockfileVersion: 5.4").await?;
                println!("  ✅ Lock file recreated");
            }
            Err(e) => {
                println!("  ❌ Install simulation failed: {}", e);
            }
        }

        // Step 5: Verify recovery
        match self.analyze_monorepo(path).await {
            Ok(info) => {
                println!("  ✅ Recovery successful! Monorepo analysis working with {} packages", info.workspace_packages.len());
            }
            Err(e) => {
                println!("  ❌ Recovery failed: {}", e);
            }
        }

        Ok(())
    }
}

/// Sets up a realistic simple repository structure (@scope/simple v0.4.0)
async fn setup_simple_repo(temp_dir: &TempDir) -> Result<PathBuf> {
    let fs = FileSystemManager::new();
    let root = temp_dir.path().to_path_buf();

    println!("🏗️ Setting up simple repository: @scope/simple v0.4.0");

    // Create package.json
    let package_json = serde_json::json!({
        "name": "@scope/simple",
        "version": "0.4.0",
        "description": "Simple TypeScript project with API fetch functionality",
        "main": "dist/index.js",
        "types": "dist/index.d.ts",
        "scripts": {
            "build": "echo 'Building @scope/simple...'",
            "lint": "echo 'Linting @scope/simple...'",
            "dev": "echo 'Starting dev server for @scope/simple...'"
        },
        "dependencies": {
            "node-fetch": "^3.3.0"
        },
        "devDependencies": {
            "typescript": "^4.9.0",
            "@types/node": "^18.0.0"
        }
    });

    fs.write_file_string(
        &root.join("package.json"),
        &serde_json::to_string_pretty(&package_json)
            .map_err(|e| Error::operation(format!("Failed to serialize package.json: {}", e)))?,
    ).await?;

    // Create TypeScript config
    let tsconfig = serde_json::json!({
        "compilerOptions": {
            "target": "ES2020",
            "module": "commonjs",
            "outDir": "dist",
            "rootDir": "src",
            "strict": true,
            "esModuleInterop": true,
            "skipLibCheck": true,
            "forceConsistentCasingInFileNames": true
        },
        "include": ["src/**/*"],
        "exclude": ["node_modules", "dist"]
    });

    fs.write_file_string(
        &root.join("tsconfig.json"),
        &serde_json::to_string_pretty(&tsconfig)
            .map_err(|e| Error::operation(format!("Failed to serialize tsconfig.json: {}", e)))?,
    ).await?;

    // Create src directory and main file
    let src_dir = root.node_path(NodePathKind::Src);
    fs.create_dir_all(&src_dir).await?;

    let main_code = r#"import fetch from 'node-fetch';

/**
 * Simple API client for demonstration purposes
 */
export class ApiClient {
    private baseUrl: string;

    constructor(baseUrl: string) {
        this.baseUrl = baseUrl;
    }

    /**
     * Fetches data from the specified endpoint
     * @param endpoint - API endpoint to fetch from
     * @returns Promise with the response data
     */
    async fetchData(endpoint: string): Promise<any> {
        try {
            const response = await fetch(`${this.baseUrl}${endpoint}`);
            
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            
            return await response.json();
        } catch (error) {
            console.error('API fetch error:', error);
            throw error;
        }
    }
}

/**
 * Default API client instance
 */
export const apiClient = new ApiClient('https://api.example.com');

/**
 * Convenience function for quick API calls
 * @param endpoint - API endpoint
 * @returns Promise with the response data
 */
export async function fetchFromApi(endpoint: string): Promise<any> {
    return apiClient.fetchData(endpoint);
}
"#;

    fs.write_file_string(&src_dir.join("index.ts"), main_code).await?;

    // Create README
    let readme = r#"# @scope/simple

A simple TypeScript project demonstrating API fetch functionality.

## Installation

```bash
npm install
```

## Usage

```typescript
import { fetchFromApi, ApiClient } from '@scope/simple';

// Using the convenience function
const data = await fetchFromApi('/users');

// Using the client class
const client = new ApiClient('https://my-api.com');
const result = await client.fetchData('/posts');
```

## Scripts

- `npm run build` - Build the project
- `npm run lint` - Lint the code
- `npm run dev` - Start development server
"#;

    fs.write_file_string(&root.join("README.md"), readme).await?;

    // Create npm lock file
    fs.write_file_string(&root.join("package-lock.json"), r#"{"name": "@scope/simple", "version": "0.4.0"}"#).await?;

    // Create repo.config.toml with custom configuration
    let repo_config = r#"# Configuration for @scope/simple project
version = "1.0"

[package_managers]
# Prefer npm for simple repositories
detection_order = ["Npm", "Yarn", "Pnpm", "Bun", "Jsr"]
detect_from_env = true
fallback = "Npm"

[commands]
# Faster timeouts for simple projects  
max_concurrent_commands = 2

[validation]
# Strict validation for production projects
strict_mode = false
require_package_json = true
validate_dependencies = true

[filesystem]
# Simple project ignore patterns
ignore_patterns = [
    ".git",
    "node_modules", 
    "dist",
    ".DS_Store"
]
"#;

    fs.write_file_string(&root.join("repo.config.toml"), repo_config).await?;

    println!("✅ Simple repository setup completed with custom configuration");
    Ok(root)
}

/// Sets up a realistic monorepo structure with pnpm
async fn setup_monorepo(temp_dir: &TempDir) -> Result<PathBuf> {
    let fs = FileSystemManager::new();
    let root = temp_dir.path().to_path_buf();

    println!("🏗️ Setting up monorepo with pnpm workspace...");

    // Create root package.json
    let root_package_json = serde_json::json!({
        "name": "@scope/monorepo",
        "version": "1.0.0",
        "private": true,
        "scripts": {
            "build": "echo 'Building all packages...'",
            "lint": "echo 'Linting all packages...'",
            "dev": "echo 'Starting all dev servers...'"
        },
        "devDependencies": {
            "typescript": "^4.9.0"
        }
    });

    fs.write_file_string(
        &root.join("package.json"),
        &serde_json::to_string_pretty(&root_package_json)
            .map_err(|e| Error::operation(format!("Failed to serialize root package.json: {}", e)))?,
    ).await?;

    // Create pnpm-workspace.yaml
    let pnpm_workspace = r#"packages:
  - 'packages/*'
  - 'apps/*'
"#;

    fs.write_file_string(&root.join("pnpm-workspace.yaml"), pnpm_workspace).await?;

    // Create .npmrc with custom pnpm config
    let npmrc = r#"# Custom pnpm configuration
auto-install-peers=true
shamefully-hoist=false
strict-peer-dependencies=true
"#;

    fs.write_file_string(&root.join(".npmrc"), npmrc).await?;

    // Create packages directory
    let packages_dir = root.join("packages");
    let apps_dir = root.join("apps");
    fs.create_dir_all(&packages_dir).await?;
    fs.create_dir_all(&apps_dir).await?;

    // Create @scope/lib package
    let lib_dir = packages_dir.join("lib");
    fs.create_dir_all(&lib_dir).await?;
    fs.create_dir_all(&lib_dir.join("src")).await?;

    let lib_package_json = serde_json::json!({
        "name": "@scope/lib",
        "version": "1.0.0",
        "main": "dist/index.js",
        "types": "dist/index.d.ts",
        "scripts": {
            "build": "echo 'Building @scope/lib...'",
            "lint": "echo 'Linting @scope/lib...'",
            "dev": "echo 'Starting dev mode for @scope/lib...'"
        },
        "dependencies": {
            "signals": "^1.0.0"
        },
        "devDependencies": {
            "typescript": "^4.9.0"
        }
    });

    fs.write_file_string(
        &lib_dir.join("package.json"),
        &serde_json::to_string_pretty(&lib_package_json)
            .map_err(|e| Error::operation(format!("Failed to serialize lib package.json: {}", e)))?,
    ).await?;

    fs.write_file_string(&lib_dir.join("src/index.ts"), r#"// Library functionality using signals
export * from './signals';
export * from './utils';
"#).await?;

    // Create @scope/ui package
    let ui_dir = packages_dir.join("ui");
    fs.create_dir_all(&ui_dir).await?;
    fs.create_dir_all(&ui_dir.join("src")).await?;

    let ui_package_json = serde_json::json!({
        "name": "@scope/ui",
        "version": "1.0.0",
        "main": "dist/index.js",
        "types": "dist/index.d.ts",
        "scripts": {
            "build": "echo 'Building @scope/ui...'",
            "lint": "echo 'Linting @scope/ui...'",
            "dev": "echo 'Starting dev mode for @scope/ui...'"
        },
        "dependencies": {
            "zag-js": "^0.10.0"
        },
        "devDependencies": {
            "typescript": "^4.9.0"
        }
    });

    fs.write_file_string(
        &ui_dir.join("package.json"),
        &serde_json::to_string_pretty(&ui_package_json)
            .map_err(|e| Error::operation(format!("Failed to serialize ui package.json: {}", e)))?,
    ).await?;

    fs.write_file_string(&ui_dir.join("src/index.ts"), r#"// UI components using zag-js
export * from './components';
export * from './hooks';
"#).await?;

    // Create @scope/app package
    let app_dir = apps_dir.join("app");
    fs.create_dir_all(&app_dir).await?;
    fs.create_dir_all(&app_dir.join("src")).await?;

    let app_package_json = serde_json::json!({
        "name": "@scope/app",
        "version": "1.0.0",
        "scripts": {
            "build": "echo 'Building @scope/app...'",
            "lint": "echo 'Linting @scope/app...'",
            "dev": "echo 'Starting dev server for @scope/app...'"
        },
        "dependencies": {
            "@scope/lib": "1.0.0",
            "@scope/ui": "workspace:*",
            "react": "^18.0.0"
        },
        "devDependencies": {
            "typescript": "^4.9.0",
            "@types/react": "^18.0.0"
        }
    });

    fs.write_file_string(
        &app_dir.join("package.json"),
        &serde_json::to_string_pretty(&app_package_json)
            .map_err(|e| Error::operation(format!("Failed to serialize app package.json: {}", e)))?,
    ).await?;

    fs.write_file_string(&app_dir.join("src/index.ts"), r#"// Main application using lib and ui packages
import { someFunction } from '@scope/lib';
import { SomeComponent } from '@scope/ui';

console.log('Application started');
"#).await?;

    // Create pnpm-lock.yaml (will be removed/recreated in error recovery test)
    fs.write_file_string(&root.join("pnpm-lock.yaml"), r#"lockfileVersion: 5.4

specifiers:
  '@scope/app': workspace:*
  '@scope/lib': 1.0.0
  '@scope/ui': workspace:*
  typescript: ^4.9.0

devDependencies:
  typescript: 4.9.5
"#).await?;

    // Create repo.config.toml with monorepo-specific configuration
    let repo_config = r#"# Configuration for @scope/monorepo project
version = "1.0"

[package_managers]
# Prefer pnpm for monorepos with custom workspace detection
detection_order = ["Pnpm", "Yarn", "Npm", "Bun"]
detect_from_env = true
fallback = "Pnpm"

# Custom workspace field patterns for @scope/ packages
custom_workspace_fields = ["@scope/"]

[monorepo]
# Extended workspace patterns for our monorepo structure
workspace_patterns = [
    "packages/*",
    "apps/*", 
    "libs/*",
    "tools/*"
]

# Additional package directories
package_directories = [
    "packages",
    "apps",
    "libs", 
    "tools",
    "services"
]

# Exclude patterns specific to our monorepo
exclude_patterns = [
    "node_modules",
    ".git",
    "dist", 
    "build",
    ".next",
    ".nuxt",
    "coverage",
    ".pnpm-store"
]

# Search configuration
max_search_depth = 6
follow_symlinks = false

[commands]
# Higher concurrency for monorepo builds
max_concurrent_commands = 6
queue_collection_window_ms = 10
queue_collection_sleep_us = 50

[filesystem]
# Monorepo-specific ignore patterns
ignore_patterns = [
    ".git",
    "node_modules",
    ".pnpm-store", 
    "dist",
    "build",
    ".DS_Store"
]

[validation]
# Flexible validation for development monorepos
strict_mode = false
require_package_json = true
validate_dependencies = true
"#;

    fs.write_file_string(&root.join("repo.config.toml"), repo_config).await?;

    println!("✅ Monorepo setup completed with custom configuration");
    Ok(root)
}

#[tokio::test]
async fn test_simple_repository_analysis() -> Result<()> {
    println!("🚀 Testing Simple Repository Analysis");
    println!("=====================================");

    // Setup
    let temp_dir = tempfile::tempdir()
        .map_err(|e| Error::operation(format!("Failed to create temp dir: {}", e)))?;
    
    let repo_path = setup_simple_repo(&temp_dir).await?;
    
    // Create analyzer with project-specific configuration from repo.config.toml
    println!("🔧 Loading project-specific configuration from repo.config.toml...");
    let analyzer = SimpleRepoAnalyzer::new_with_project_config(&repo_path).await
        .unwrap_or_else(|_| {
            println!("⚠️ Failed to load project config, falling back to defaults");
            SimpleRepoAnalyzer::new()
        });

    // Analyze the repository
    let repo_info = analyzer.analyze_simple_repo(&repo_path).await?;

    // Verify analysis results
    println!("\n📊 Analysis Results:");
    println!("  Name: {}", repo_info.name);
    println!("  Version: {}", repo_info.version);
    println!("  Package Manager: {}", repo_info.package_manager.kind().command());
    println!("  Has TypeScript: {}", repo_info.has_typescript);
    println!("  Scripts: {:?}", repo_info.scripts);
    println!("  Dependencies: {:?}", repo_info.dependencies);
    println!("  Dev Dependencies: {:?}", repo_info.dev_dependencies);

    // Assertions
    assert_eq!(repo_info.name, "@scope/simple");
    assert_eq!(repo_info.version, "0.4.0");
    assert!(repo_info.has_typescript);
    assert!(repo_info.scripts.contains(&"build".to_string()));
    assert!(repo_info.scripts.contains(&"lint".to_string()));
    assert!(repo_info.scripts.contains(&"dev".to_string()));
    assert!(repo_info.dependencies.contains(&"node-fetch".to_string()));
    assert!(repo_info.dev_dependencies.contains(&"typescript".to_string()));

    // Test script execution
    println!("\n🏃 Testing Script Execution:");
    let script_results = analyzer.run_scripts(&repo_info, &["build", "lint", "dev"]).await?;
    
    for (script, success) in &script_results {
        println!("  {} - {}", script, if *success { "✅" } else { "❌" });
    }

    // Verify all scripts ran successfully (they're just echo commands)
    assert_eq!(script_results.len(), 3);
    assert!(script_results.iter().all(|(_, success)| *success));

    println!("\n✅ Simple repository analysis test passed!");
    Ok(())
}

#[tokio::test]
async fn test_monorepo_analysis() -> Result<()> {
    println!("🚀 Testing Monorepo Analysis");
    println!("============================");

    // Setup
    let temp_dir = tempfile::tempdir()
        .map_err(|e| Error::operation(format!("Failed to create temp dir: {}", e)))?;
    
    let monorepo_path = setup_monorepo(&temp_dir).await?;
    let analyzer = MonorepoAnalyzer::new();

    // Analyze the monorepo
    let monorepo_info = analyzer.analyze_monorepo(&monorepo_path).await?;

    // Verify analysis results
    println!("\n📊 Monorepo Analysis Results:");
    println!("  Name: {}", monorepo_info.name);
    println!("  Package Manager: {}", monorepo_info.package_manager.kind().command());
    println!("  Workspace Packages: {}", monorepo_info.workspace_packages.len());
    
    for package in &monorepo_info.workspace_packages {
        println!("    - {} v{} ({})", package.name, package.version, package.path.display());
        println!("      Scripts: {:?}", package.scripts);
        println!("      Dependencies: {:?}", package.dependencies);
        if !package.workspace_dependencies.is_empty() {
            println!("      Workspace Deps: {:?}", package.workspace_dependencies);
        }
    }

    println!("\n🔗 Dependency Graph:");
    for (package, deps) in &monorepo_info.dependency_graph {
        if !deps.is_empty() {
            println!("  {} depends on:", package);
            for dep in deps {
                println!("    - {}", dep);
            }
        }
    }

    // Assertions
    assert_eq!(monorepo_info.name, "@scope/monorepo");
    assert_eq!(monorepo_info.workspace_packages.len(), 3);
    
    let package_names: Vec<&String> = monorepo_info.workspace_packages.iter().map(|p| &p.name).collect();
    assert!(package_names.contains(&&"@scope/lib".to_string()));
    assert!(package_names.contains(&&"@scope/ui".to_string()));
    assert!(package_names.contains(&&"@scope/app".to_string()));

    // Verify dependencies
    let app_package = monorepo_info.workspace_packages.iter()
        .find(|p| p.name == "@scope/app")
        .expect("@scope/app package should exist");
    
    assert!(app_package.workspace_dependencies.contains(&"@scope/lib".to_string()));
    assert!(app_package.workspace_dependencies.contains(&"@scope/ui".to_string()));

    let lib_package = monorepo_info.workspace_packages.iter()
        .find(|p| p.name == "@scope/lib")
        .expect("@scope/lib package should exist");
    
    assert!(lib_package.dependencies.contains(&"signals".to_string()));

    let ui_package = monorepo_info.workspace_packages.iter()
        .find(|p| p.name == "@scope/ui")
        .expect("@scope/ui package should exist");
    
    assert!(ui_package.dependencies.contains(&"zag-js".to_string()));

    // Test workspace script execution
    println!("\n🏃 Testing Workspace Script Execution:");
    let build_results = analyzer.run_workspace_scripts(&monorepo_info, "build").await?;
    
    for (package, success) in &build_results {
        println!("  {} build - {}", package, if *success { "✅" } else { "❌" });
    }

    // Verify all build scripts ran successfully
    assert_eq!(build_results.len(), 3);
    assert!(build_results.iter().all(|(_, success)| *success));

    println!("\n✅ Monorepo analysis test passed!");
    Ok(())
}

#[tokio::test]
async fn test_error_recovery_missing_lock_file() -> Result<()> {
    println!("🚀 Testing Error Recovery: Missing Lock File");
    println!("============================================");

    // Setup
    let temp_dir = tempfile::tempdir()
        .map_err(|e| Error::operation(format!("Failed to create temp dir: {}", e)))?;
    
    let monorepo_path = setup_monorepo(&temp_dir).await?;
    let analyzer = MonorepoAnalyzer::new();

    // Test the missing lock file recovery scenario
    analyzer.simulate_missing_lock_file_recovery(&monorepo_path).await?;

    println!("\n✅ Error recovery test completed!");
    Ok(())
}

#[tokio::test]
async fn test_project_type_detection() -> Result<()> {
    println!("🚀 Testing Project Type Detection");
    println!("=================================");

    // Setup both project types
    let simple_temp = tempfile::tempdir()
        .map_err(|e| Error::operation(format!("Failed to create temp dir: {}", e)))?;
    let monorepo_temp = tempfile::tempdir()
        .map_err(|e| Error::operation(format!("Failed to create temp dir: {}", e)))?;

    let simple_path = setup_simple_repo(&simple_temp).await?;
    let monorepo_path = setup_monorepo(&monorepo_temp).await?;

    let project_detector = ProjectDetector::new();
    
    // Create monorepo detector with project-specific configuration for better detection
    let monorepo_detector = MonorepoDetector::new_with_project_config(&monorepo_path).await
        .unwrap_or_else(|_| {
            println!("⚠️ Using default monorepo detector config");
            MonorepoDetector::new()
        });

    // Test simple repository detection
    println!("\n🔍 Testing simple repository detection:");
    let simple_project = project_detector.detect(&simple_path, None).await?;
    let simple_info = simple_project.as_project_info();
    println!("  Simple project type: {}", simple_info.kind().name());
    
    let is_simple_monorepo = monorepo_detector.is_monorepo_root(&simple_path).await?;
    println!("  Is simple repo a monorepo: {:?}", is_simple_monorepo);
    
    assert!(is_simple_monorepo.is_none()); // Should not be detected as monorepo

    // Test monorepo detection
    println!("\n🔍 Testing monorepo detection:");
    let monorepo_project = project_detector.detect(&monorepo_path, None).await?;
    let monorepo_project_info = monorepo_project.as_project_info();
    println!("  Monorepo project type: {}", monorepo_project_info.kind().name());
    
    let monorepo_kind = monorepo_detector.is_monorepo_root(&monorepo_path).await?;
    println!("  Monorepo type: {:?}", monorepo_kind);
    
    assert!(monorepo_kind.is_some()); // Should be detected as monorepo
    if let Some(kind) = monorepo_kind {
        println!("  Detected monorepo kind: {}", kind.name());
    }

    println!("\n✅ Project type detection test passed!");
    Ok(())
}

#[tokio::test]
async fn test_comprehensive_real_world_scenario() -> Result<()> {
    println!("🚀 Comprehensive Real-World Scenario Test");
    println!("==========================================");

    // This test demonstrates a complete workflow a developer might follow

    // Step 1: Set up both project types
    println!("\n📁 Step 1: Setting up test projects...");
    
    let simple_temp = tempfile::tempdir()
        .map_err(|e| Error::operation(format!("Failed to create temp dir: {}", e)))?;
    let monorepo_temp = tempfile::tempdir()
        .map_err(|e| Error::operation(format!("Failed to create temp dir: {}", e)))?;

    let simple_path = setup_simple_repo(&simple_temp).await?;
    let monorepo_path = setup_monorepo(&monorepo_temp).await?;

    // Step 2: Project discovery and analysis
    println!("\n🔍 Step 2: Discovering and analyzing projects...");
    
    // Create analyzers with project-specific configurations
    println!("🔧 Loading project-specific configurations...");
    let simple_analyzer = SimpleRepoAnalyzer::new_with_project_config(&simple_path).await
        .unwrap_or_else(|_| {
            println!("⚠️ Simple repo config load failed, using defaults");
            SimpleRepoAnalyzer::new()
        });
    
    let monorepo_analyzer = MonorepoAnalyzer::new_with_project_config(&monorepo_path).await
        .unwrap_or_else(|_| {
            println!("⚠️ Monorepo config load failed, using defaults");
            MonorepoAnalyzer::new()
        });

    let simple_info = simple_analyzer.analyze_simple_repo(&simple_path).await?;
    let monorepo_info = monorepo_analyzer.analyze_monorepo(&monorepo_path).await?;

    println!("  Simple project: {} v{}", simple_info.name, simple_info.version);
    println!("  Monorepo: {} with {} packages", monorepo_info.name, monorepo_info.workspace_packages.len());

    // Step 3: Validation and health checks
    println!("\n✅ Step 3: Running validation and health checks...");
    
    // Check that all expected files exist
    let fs = FileSystemManager::new();
    
    // Simple repo checks
    assert!(fs.exists(&simple_path.join("package.json")).await);
    assert!(fs.exists(&simple_path.join("tsconfig.json")).await);
    assert!(fs.exists(&simple_path.node_path(NodePathKind::Src)).await);
    println!("  Simple repository structure: ✅");

    // Monorepo checks
    assert!(fs.exists(&monorepo_path.join("pnpm-workspace.yaml")).await);
    assert!(fs.exists(&monorepo_path.join("packages/lib/package.json")).await);
    assert!(fs.exists(&monorepo_path.join("packages/ui/package.json")).await);
    assert!(fs.exists(&monorepo_path.join("apps/app/package.json")).await);
    println!("  Monorepo structure: ✅");

    // Step 4: Dependency analysis
    println!("\n🔗 Step 4: Analyzing dependencies...");
    
    // Simple repo dependencies
    println!("  Simple repo dependencies: {:?}", simple_info.dependencies);
    assert!(simple_info.dependencies.contains(&"node-fetch".to_string()));

    // Monorepo workspace dependencies
    let app_package = monorepo_info.workspace_packages.iter()
        .find(|p| p.name == "@scope/app")
        .expect("App package should exist");
    
    println!("  App workspace dependencies: {:?}", app_package.workspace_dependencies);
    assert!(app_package.workspace_dependencies.contains(&"@scope/lib".to_string()));
    assert!(app_package.workspace_dependencies.contains(&"@scope/ui".to_string()));

    // Step 5: Script execution simulation
    println!("\n🏃 Step 5: Executing development scripts...");
    
    // Run scripts in simple repo
    let simple_results = simple_analyzer.run_scripts(&simple_info, &["build", "lint"]).await?;
    let successful_simple = simple_results.iter().filter(|(_, success)| *success).count();
    println!("  Simple repo: {}/{} scripts succeeded", successful_simple, simple_results.len());

    // Run scripts in monorepo
    let monorepo_results = monorepo_analyzer.run_workspace_scripts(&monorepo_info, "build").await?;
    let successful_monorepo = monorepo_results.iter().filter(|(_, success)| *success).count();
    println!("  Monorepo: {}/{} package builds succeeded", successful_monorepo, monorepo_results.len());

    // Step 6: Error recovery demonstration
    println!("\n🔧 Step 6: Demonstrating error recovery...");
    monorepo_analyzer.simulate_missing_lock_file_recovery(&monorepo_path).await?;

    // Step 7: Final verification
    println!("\n🎯 Step 7: Final verification...");
    
    // Verify we can still detect and analyze after recovery
    let recovered_info = monorepo_analyzer.analyze_monorepo(&monorepo_path).await?;
    assert_eq!(recovered_info.workspace_packages.len(), monorepo_info.workspace_packages.len());
    println!("  Post-recovery analysis: ✅");

    println!("\n🎉 Comprehensive real-world scenario test completed successfully!");
    
    // Summary
    println!("\n=== FINAL SUMMARY ===");
    println!("Simple Repository:");
    println!("  Name: {}", simple_info.name);
    println!("  Version: {}", simple_info.version);
    println!("  Package Manager: {}", simple_info.package_manager.kind().command());
    println!("  TypeScript: {}", simple_info.has_typescript);
    
    println!("\nMonorepo:");
    println!("  Name: {}", monorepo_info.name);
    println!("  Package Manager: {}", monorepo_info.package_manager.kind().command());
    println!("  Packages: {}", monorepo_info.workspace_packages.len());
    for pkg in &monorepo_info.workspace_packages {
        println!("    - {} v{}", pkg.name, pkg.version);
    }

    Ok(())
}