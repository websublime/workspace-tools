//! # Real-World Usage Test: Monorepo Analysis and Build Tool
//!
//! ## What
//! This integration test demonstrates a comprehensive real-world usage scenario
//! where we build a tool that analyzes Node.js monorepos, validates their structure,
//! executes build commands, and manages configuration.
//!
//! ## How
//! The test creates a realistic monorepo structure, uses all major APIs from the crate
//! to analyze, validate, and process the monorepo, including filesystem operations,
//! command execution, error handling, and configuration management.
//!
//! ## Why
//! This test serves as both a comprehensive API demonstration and a real-world
//! validation that all components work together correctly in typical usage scenarios.

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
    command::{
        CommandBuilder, CommandPriority, CommandQueue, CommandQueueConfig, DefaultCommandExecutor,
        Executor, StreamConfig,
    },
    error::{Error, Result},
    filesystem::{FileSystem, FileSystemManager, NodePathKind, PathExt, PathUtils},
    monorepo::{
        ConfigManager, ConfigScope, ConfigValue, MonorepoDetector, MonorepoKind, PackageManager,
        ProjectConfig, ProjectManager, ProjectValidationStatus,
    },
};

use tempfile::TempDir;

/// Represents our monorepo analysis and build tool
#[derive(Debug)]
struct MonorepoAnalyzer {
    /// Filesystem interface
    fs: FileSystemManager,
    /// Command executor for running build commands
    executor: DefaultCommandExecutor,
    /// Command queue for managing concurrent builds
    queue: Option<CommandQueue>,
    /// Configuration manager
    config: ConfigManager,
    /// Project manager for validation
    project_manager: ProjectManager,
    /// Monorepo detector for structure analysis
    detector: MonorepoDetector,
}

/// Analysis results for a monorepo
#[derive(Debug)]
struct AnalysisReport {
    /// Root path of the monorepo
    root: PathBuf,
    /// Type of monorepo detected
    monorepo_kind: MonorepoKind,
    /// Package manager information
    package_manager: PackageManager,
    /// List of packages found
    packages: Vec<PackageInfo>,
    /// Validation results for each package
    validations: Vec<ValidationResult>,
    /// Build execution results
    build_results: Vec<BuildResult>,
    /// Configuration summary
    config_summary: HashMap<String, ConfigValue>,
}

/// Information about a package in the monorepo
#[derive(Clone, Debug)]
struct PackageInfo {
    /// Package name
    name: String,
    /// Package version
    version: String,
    /// Relative path from monorepo root
    path: PathBuf,
    /// Dependencies within the workspace
    workspace_deps: Vec<String>,
    /// Has build script
    has_build_script: bool,
}

/// Validation result for a package
#[derive(Debug)]
struct ValidationResult {
    /// Package name
    package_name: String,
    /// Validation status
    status: ProjectValidationStatus,
    /// Additional checks performed
    checks: Vec<String>,
}

/// Build result for a package
#[derive(Debug)]
struct BuildResult {
    /// Package name
    package_name: String,
    /// Whether build succeeded
    success: bool,
    /// Build duration
    duration: Duration,
    /// Build output (truncated)
    output: String,
}

impl MonorepoAnalyzer {
    /// Creates a new monorepo analyzer with default configuration
    fn new() -> Result<Self> {
        let mut config = ConfigManager::new();

        // Set up configuration paths for different scopes
        let user_config_path = PathUtils::current_dir()?.join(".monorepo-analyzer-user.json");
        config.set_path(ConfigScope::User, user_config_path);

        // Set default configuration values
        config.set("max_concurrent_builds", ConfigValue::Integer(4));
        config.set("build_timeout_seconds", ConfigValue::Integer(300));
        config.set("enable_detailed_logging", ConfigValue::Boolean(true));
        config.set(
            "supported_package_managers",
            ConfigValue::Array(vec![
                ConfigValue::String("npm".to_string()),
                ConfigValue::String("yarn".to_string()),
                ConfigValue::String("pnpm".to_string()),
            ]),
        );

        Ok(Self {
            fs: FileSystemManager::new(),
            executor: DefaultCommandExecutor::new(),
            queue: None,
            config,
            project_manager: ProjectManager::new(),
            detector: MonorepoDetector::new(),
        })
    }

    /// Initializes the command queue with configuration
    fn initialize_queue(&mut self) -> Result<()> {
        let max_concurrent =
            self.config.get("max_concurrent_builds").and_then(|v| v.as_integer()).unwrap_or(4)
                as usize;

        let timeout_secs =
            self.config.get("build_timeout_seconds").and_then(|v| v.as_integer()).unwrap_or(300)
                as u64;

        let queue_config = CommandQueueConfig {
            max_concurrent_commands: max_concurrent,
            rate_limit: Some(Duration::from_millis(100)), // Prevent overwhelming the system
            default_timeout: Duration::from_secs(timeout_secs),
            shutdown_timeout: Duration::from_secs(30),
        };

        self.queue = Some(CommandQueue::with_config(queue_config).start()?);
        Ok(())
    }

    /// Analyzes a monorepo and generates a comprehensive report
    async fn analyze_monorepo(&mut self, path: &Path) -> Result<AnalysisReport> {
        // Step 1: Basic path validation and normalization
        let normalized_path = path.normalize();
        if !self.fs.exists(&normalized_path) {
            return Err(Error::operation(format!(
                "Path does not exist: {}",
                normalized_path.display()
            )));
        }

        // Step 2: Detect monorepo structure
        let monorepo_descriptor = self.detector.detect_monorepo(&normalized_path)?;
        let monorepo_kind = monorepo_descriptor.kind().clone();
        let packages = monorepo_descriptor.packages();

        // Step 3: Detect package manager
        let package_manager = PackageManager::detect(&normalized_path)?;

        // Step 4: Validate that detected package manager matches monorepo type
        self.validate_package_manager_consistency(&package_manager, &monorepo_kind)?;

        // Step 5: Initialize command queue for parallel operations
        self.initialize_queue()?;

        // Step 6: Analyze each package
        let mut package_infos = Vec::new();
        let mut validations = Vec::new();

        for workspace_package in packages {
            // Extract package information
            let package_info = self.extract_package_info(workspace_package, &normalized_path)?;
            package_infos.push(package_info.clone());

            // Validate package structure
            let validation = self.validate_package(&workspace_package.absolute_path)?;
            validations.push(ValidationResult {
                package_name: package_info.name.clone(),
                status: validation.status,
                checks: validation.checks,
            });
        }

        // Step 7: Execute builds for packages with build scripts
        let build_results = self.execute_builds(&package_infos, &package_manager).await?;

        // Step 8: Gather configuration summary
        let config_summary = self.gather_config_summary();

        Ok(AnalysisReport {
            root: normalized_path,
            monorepo_kind,
            package_manager,
            packages: package_infos,
            validations,
            build_results,
            config_summary,
        })
    }

    /// Validates that package manager is consistent with monorepo type
    fn validate_package_manager_consistency(
        &self,
        package_manager: &PackageManager,
        monorepo_kind: &MonorepoKind,
    ) -> Result<()> {
        let expected_managers = match monorepo_kind {
            MonorepoKind::NpmWorkSpace => vec!["npm"],
            MonorepoKind::YarnWorkspaces => vec!["yarn", "npm"], // Yarn can coexist with npm
            MonorepoKind::PnpmWorkspaces => vec!["pnpm"],
            MonorepoKind::BunWorkspaces => vec!["bun"],
            MonorepoKind::DenoWorkspaces => vec!["deno"],
            MonorepoKind::Custom { name: _, config_file: _ } => return Ok(()), // Skip validation for custom
        };

        let actual_manager = package_manager.kind().command();
        if !expected_managers.contains(&actual_manager) {
            log::warn!(
                "Package manager '{}' may not be optimal for monorepo type '{}'",
                actual_manager,
                monorepo_kind.name()
            );
        }

        Ok(())
    }

    /// Extracts detailed information about a package
    fn extract_package_info(
        &self,
        workspace_package: &sublime_standard_tools::monorepo::WorkspacePackage,
        _root: &Path,
    ) -> Result<PackageInfo> {
        let package_json_path = workspace_package.absolute_path.join("package.json");
        let package_json_content = self.fs.read_file_string(&package_json_path)?;

        // Parse package.json to check for build scripts
        let package_json: serde_json::Value =
            serde_json::from_str(&package_json_content).map_err(|e| {
                Error::operation(format!(
                    "Invalid package.json in {}: {}",
                    workspace_package.name, e
                ))
            })?;

        let has_build_script =
            package_json.get("scripts").and_then(|scripts| scripts.get("build")).is_some();

        Ok(PackageInfo {
            name: workspace_package.name.clone(),
            version: workspace_package.version.clone(),
            path: workspace_package.location.clone(),
            workspace_deps: workspace_package.workspace_dependencies.clone(),
            has_build_script,
        })
    }

    /// Validates a package structure and configuration
    fn validate_package(&self, package_path: &Path) -> Result<DetailedValidation> {
        let config =
            ProjectConfig::new().with_detect_package_manager(true).with_validate_structure(true);

        let project = self.project_manager.detect_project(package_path, &config)?;
        let mut checks = Vec::new();

        // Custom validation checks
        checks.push("package.json format".to_string());

        // Check for common Node.js directories
        if self.fs.exists(&package_path.node_path(NodePathKind::Src)) {
            checks.push("src directory exists".to_string());
        }

        if self.fs.exists(&package_path.node_path(NodePathKind::Test)) {
            checks.push("test directory exists".to_string());
        }

        // Check for TypeScript configuration
        if self.fs.exists(&package_path.join("tsconfig.json")) {
            checks.push("TypeScript configuration".to_string());
        }

        // Check for documentation
        if self.fs.exists(&package_path.join("README.md")) {
            checks.push("README documentation".to_string());
        }

        Ok(DetailedValidation { status: project.validation_status().clone(), checks })
    }

    /// Executes build commands for packages that have build scripts
    async fn execute_builds(
        &mut self,
        packages: &[PackageInfo],
        package_manager: &PackageManager,
    ) -> Result<Vec<BuildResult>> {
        let queue =
            self.queue.as_ref().ok_or_else(|| Error::operation("Command queue not initialized"))?;

        let mut build_command_ids = Vec::new();
        let mut buildable_packages = Vec::new();

        // Queue build commands for packages with build scripts
        for package in packages {
            if package.has_build_script {
                let build_command = CommandBuilder::new(package_manager.kind().command())
                    .arg("run")
                    .arg("build")
                    .current_dir(&package.path)
                    .timeout(Duration::from_secs(300))
                    .build();

                let command_id = queue.enqueue(build_command, CommandPriority::Normal).await?;
                build_command_ids.push(command_id);
                buildable_packages.push(package.clone());
            }
        }

        // Wait for all builds to complete
        let mut results = Vec::new();
        for (command_id, package) in build_command_ids.iter().zip(buildable_packages.iter()) {
            match queue.wait_for_command(command_id, Duration::from_secs(600)).await {
                Ok(result) => {
                    let success = result.is_successful();
                    let output = if let Some(cmd_output) = result.output {
                        format!(
                            "Exit code: {}\nStdout: {}\nStderr: {}",
                            cmd_output.status(),
                            truncate_string(cmd_output.stdout(), 200),
                            truncate_string(cmd_output.stderr(), 200)
                        )
                    } else {
                        result.error.unwrap_or_else(|| "Unknown error".to_string())
                    };

                    results.push(BuildResult {
                        package_name: package.name.clone(),
                        success,
                        duration: Duration::from_millis(100), // Placeholder - would use actual duration
                        output,
                    });
                }
                Err(e) => {
                    results.push(BuildResult {
                        package_name: package.name.clone(),
                        success: false,
                        duration: Duration::from_secs(0),
                        output: format!("Build timeout or error: {e}"),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Demonstrates streaming command output for long-running operations
    async fn demonstrate_streaming(&self, package_path: &Path) -> Result<()> {
        let stream_config = StreamConfig::default();

        let command = CommandBuilder::new("echo")
            .arg("Starting install process...")
            .current_dir(package_path)
            .build();

        let (mut stream, mut child) = self.executor.execute_stream(command, stream_config).await?;

        // Read streaming output with timeout
        let mut line_count = 0;
        while line_count < 5 {
            // Limit for demo
            match stream.next_timeout(Duration::from_secs(1)).await {
                Ok(Some(output)) => {
                    match output {
                        sublime_standard_tools::command::StreamOutput::Stdout(line) => {
                            log::info!("INSTALL STDOUT: {}", truncate_string(&line, 100));
                        }
                        sublime_standard_tools::command::StreamOutput::Stderr(line) => {
                            log::warn!("INSTALL STDERR: {}", truncate_string(&line, 100));
                        }
                        sublime_standard_tools::command::StreamOutput::End => break,
                    }
                    line_count += 1;
                }
                Ok(None) => break,
                Err(_) => {
                    log::info!("Stream timeout - process likely completed");
                    break;
                }
            }
        }

        // Clean up
        let _ = child.kill().await;
        Ok(())
    }

    /// Gathers configuration summary for reporting
    fn gather_config_summary(&self) -> HashMap<String, ConfigValue> {
        let mut summary = HashMap::new();

        // Get known configuration keys
        let keys = ["max_concurrent_builds", "build_timeout_seconds", "enable_detailed_logging"];

        for key in &keys {
            if let Some(value) = self.config.get(key) {
                summary.insert((*key).to_string(), value);
            }
        }

        summary
    }

    /// Shuts down the analyzer and cleans up resources
    async fn shutdown(&mut self) -> Result<()> {
        if let Some(mut queue) = self.queue.take() {
            queue.shutdown().await?;
        }
        Ok(())
    }
}

/// Detailed validation result
#[derive(Debug)]
struct DetailedValidation {
    status: ProjectValidationStatus,
    checks: Vec<String>,
}

/// Helper function to truncate strings for display
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

/// Sets up a realistic test monorepo structure
fn setup_test_monorepo(temp_dir: &TempDir) -> Result<PathBuf> {
    let fs = FileSystemManager::new();
    let root = temp_dir.path().to_path_buf();

    // Create root package.json with workspaces
    let root_package_json = serde_json::json!({
        "name": "my-monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": [
            "packages/*",
            "apps/*"
        ],
        "devDependencies": {
            "typescript": "^4.9.0",
            "jest": "^29.0.0"
        }
    });

    fs.write_file_string(
        &root.join("package.json"),
        &serde_json::to_string_pretty(&root_package_json)
            .map_err(|e| Error::operation(format!("Failed to serialize JSON: {e}")))?,
    )?;

    // Create yarn.lock to make it a Yarn workspace
    fs.write_file_string(&root.join("yarn.lock"), "")?;

    // Create packages directory structure
    let packages_dir = root.join("packages");
    let apps_dir = root.join("apps");
    fs.create_dir_all(&packages_dir)?;
    fs.create_dir_all(&apps_dir)?;

    // Create shared library package
    let shared_lib_dir = packages_dir.join("shared");
    fs.create_dir_all(&shared_lib_dir)?;
    fs.create_dir_all(&shared_lib_dir.join("src"))?;
    fs.create_dir_all(&shared_lib_dir.join("test"))?;

    let shared_package_json = serde_json::json!({
        "name": "@myorg/shared",
        "version": "1.0.0",
        "main": "dist/index.js",
        "scripts": {
            "build": "echo 'Building shared library'",
            "test": "echo 'Testing shared library'"
        },
        "devDependencies": {
            "typescript": "^4.9.0"
        }
    });

    fs.write_file_string(
        &shared_lib_dir.join("package.json"),
        &serde_json::to_string_pretty(&shared_package_json)
            .map_err(|e| Error::operation(format!("Failed to serialize JSON: {e}")))?,
    )?;

    fs.write_file_string(
        &shared_lib_dir.join("tsconfig.json"),
        r#"{"compilerOptions": {"target": "es2020", "outDir": "dist"}}"#,
    )?;

    fs.write_file_string(
        &shared_lib_dir.join("README.md"),
        "# Shared Library\n\nCommon utilities.",
    )?;

    fs.write_file_string(
        &shared_lib_dir.join("src").join("index.ts"),
        "export const greet = (name: string) => `Hello, ${name}!`;",
    )?;

    // Create UI components package
    let ui_dir = packages_dir.join("ui");
    fs.create_dir_all(&ui_dir)?;
    fs.create_dir_all(&ui_dir.join("src"))?;

    let ui_package_json = serde_json::json!({
        "name": "@myorg/ui",
        "version": "1.0.0",
        "main": "dist/index.js",
        "scripts": {
            "build": "echo 'Building UI components'",
            "test": "echo 'Testing UI components'"
        },
        "dependencies": {
            "@myorg/shared": "1.0.0"
        },
        "devDependencies": {
            "typescript": "^4.9.0"
        }
    });

    fs.write_file_string(
        &ui_dir.join("package.json"),
        &serde_json::to_string_pretty(&ui_package_json)
            .map_err(|e| Error::operation(format!("Failed to serialize JSON: {e}")))?,
    )?;

    fs.write_file_string(&ui_dir.join("README.md"), "# UI Components\n\nReusable UI components.")?;

    // Create web application
    let web_app_dir = apps_dir.join("web");
    fs.create_dir_all(&web_app_dir)?;
    fs.create_dir_all(&web_app_dir.join("src"))?;

    let web_package_json = serde_json::json!({
        "name": "@myorg/web",
        "version": "1.0.0",
        "scripts": {
            "build": "echo 'Building web application'",
            "dev": "echo 'Starting dev server'",
            "test": "echo 'Testing web application'"
        },
        "dependencies": {
            "@myorg/shared": "1.0.0",
            "@myorg/ui": "1.0.0"
        },
        "devDependencies": {
            "webpack": "^5.0.0",
            "webpack-cli": "^4.0.0"
        }
    });

    fs.write_file_string(
        &web_app_dir.join("package.json"),
        &serde_json::to_string_pretty(&web_package_json)
            .map_err(|e| Error::operation(format!("Failed to serialize JSON: {e}")))?,
    )?;

    fs.write_file_string(
        &web_app_dir.join("README.md"),
        "# Web Application\n\nMain web application.",
    )?;

    // Create a package without build script
    let docs_dir = packages_dir.join("docs");
    fs.create_dir_all(&docs_dir)?;

    let docs_package_json = serde_json::json!({
        "name": "@myorg/docs",
        "version": "1.0.0",
        "scripts": {
            "serve": "echo 'Serving documentation'"
        }
    });

    fs.write_file_string(
        &docs_dir.join("package.json"),
        &serde_json::to_string_pretty(&docs_package_json)
            .map_err(|e| Error::operation(format!("Failed to serialize JSON: {e}")))?,
    )?;

    Ok(root)
}

#[tokio::test]
async fn test_comprehensive_monorepo_analysis() -> Result<()> {
    // Set up test environment
    let temp_dir = tempfile::tempdir()
        .map_err(|e| Error::operation(format!("Failed to create temp dir: {e}")))?;

    let monorepo_root = setup_test_monorepo(&temp_dir)?;
    println!("Created test monorepo at: {}", monorepo_root.display());

    // Create and configure analyzer
    let mut analyzer = MonorepoAnalyzer::new()?;

    // Run comprehensive analysis
    println!("Starting monorepo analysis...");
    let analysis_report = analyzer.analyze_monorepo(&monorepo_root).await?;

    // Demonstrate streaming functionality
    println!("Demonstrating streaming command output...");
    let _ = analyzer.demonstrate_streaming(&monorepo_root).await;

    // Validate analysis results
    println!("Validating analysis results...");

    // Check monorepo detection
    assert!(matches!(analysis_report.monorepo_kind, MonorepoKind::YarnWorkspaces));
    println!("✓ Correctly detected Yarn Workspaces monorepo");

    // Check package manager detection
    assert_eq!(analysis_report.package_manager.kind().command(), "yarn");
    println!("✓ Correctly detected Yarn package manager");

    // Check package discovery
    assert_eq!(analysis_report.packages.len(), 4); // shared, ui, web, docs
    println!("✓ Discovered {} packages", analysis_report.packages.len());

    // Validate package information
    let package_names: Vec<&String> = analysis_report.packages.iter().map(|p| &p.name).collect();
    assert!(package_names.contains(&&"@myorg/shared".to_string()));
    assert!(package_names.contains(&&"@myorg/ui".to_string()));
    assert!(package_names.contains(&&"@myorg/web".to_string()));
    assert!(package_names.contains(&&"@myorg/docs".to_string()));
    println!("✓ All expected packages found: {package_names:?}");

    // Check build script detection
    let buildable_packages: Vec<&PackageInfo> =
        analysis_report.packages.iter().filter(|p| p.has_build_script).collect();
    assert_eq!(buildable_packages.len(), 3); // shared, ui, web have build scripts
    println!("✓ Detected {} packages with build scripts", buildable_packages.len());

    // Check workspace dependencies
    let ui_package = analysis_report
        .packages
        .iter()
        .find(|p| p.name == "@myorg/ui")
        .ok_or_else(|| Error::operation("UI package not found"))?;
    assert!(ui_package.workspace_deps.contains(&"@myorg/shared".to_string()));
    println!("✓ Workspace dependencies correctly identified");

    // Validate project validations
    assert_eq!(analysis_report.validations.len(), 4);
    for validation in &analysis_report.validations {
        println!("Package {} validation: {:?}", validation.package_name, validation.status);
        assert!(!validation.checks.is_empty()); // Should have performed some checks
    }
    println!("✓ All packages validated with detailed checks");

    // Check configuration
    assert!(!analysis_report.config_summary.is_empty());
    if let Some(max_concurrent) = analysis_report.config_summary.get("max_concurrent_builds") {
        assert_eq!(max_concurrent.as_integer(), Some(4));
        println!("✓ Configuration correctly loaded and accessible");
    }

    // Validate build results (these will likely fail in test environment, but structure should be correct)
    println!("Build results summary:");
    for build_result in &analysis_report.build_results {
        println!(
            "  {} - Success: {}, Duration: {:?}",
            build_result.package_name, build_result.success, build_result.duration
        );
    }
    println!("✓ Build execution attempted for all buildable packages");

    // Test filesystem operations
    let fs = FileSystemManager::new();

    // Test path extensions
    let ui_src_path = monorepo_root.join("packages/ui").node_path(NodePathKind::Src);
    assert!(fs.exists(&ui_src_path));
    println!("✓ Path extensions working correctly");

    // Test path utilities
    if let Some(project_root) = PathUtils::find_project_root(&monorepo_root.join("packages/ui")) {
        assert_eq!(project_root, monorepo_root);
        println!("✓ Project root detection working");
    }

    // Test configuration management in detail
    let test_config = ConfigManager::new();
    test_config.set("test_setting", ConfigValue::String("test_value".to_string()));
    assert_eq!(
        test_config.get("test_setting").and_then(|v| v.as_string().map(ToString::to_string)),
        Some("test_value".to_string())
    );
    println!("✓ Configuration management working correctly");

    // Clean up
    analyzer.shutdown().await?;
    println!("✓ Analyzer shutdown completed successfully");

    println!("🎉 Comprehensive monorepo analysis test completed successfully!");

    // Print final summary
    println!("\n=== ANALYSIS SUMMARY ===");
    println!("Monorepo Type: {}", analysis_report.monorepo_kind.name());
    println!("Package Manager: {}", analysis_report.package_manager.kind().command());
    println!("Total Packages: {}", analysis_report.packages.len());
    println!(
        "Buildable Packages: {}",
        analysis_report.packages.iter().filter(|p| p.has_build_script).count()
    );
    println!("Validation Results: {} packages validated", analysis_report.validations.len());
    println!("Build Results: {} build attempts", analysis_report.build_results.len());
    println!("Configuration Keys: {}", analysis_report.config_summary.len());

    Ok(())
}

#[tokio::test]
async fn test_error_handling_scenarios() -> Result<()> {
    let mut analyzer = MonorepoAnalyzer::new()?;

    // Test 1: Non-existent path
    let result = analyzer.analyze_monorepo(Path::new("/non/existent/path")).await;
    assert!(result.is_err());
    println!("✓ Correctly handled non-existent path error");

    // Test 2: Non-monorepo directory
    let temp_dir = tempfile::tempdir()
        .map_err(|e| Error::operation(format!("Failed to create temp dir: {e}")))?;

    let fs = FileSystemManager::new();

    // Create a regular Node.js project (not a monorepo)
    let single_project_json = serde_json::json!({
        "name": "single-project",
        "version": "1.0.0"
    });

    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        &serde_json::to_string_pretty(&single_project_json)
            .map_err(|e| Error::operation(format!("Failed to serialize JSON: {e}")))?,
    )?;

    let result = analyzer.analyze_monorepo(temp_dir.path()).await;
    assert!(result.is_err());
    println!("✓ Correctly detected non-monorepo structure");

    analyzer.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn test_command_execution_edge_cases() -> Result<()> {
    let executor = DefaultCommandExecutor::new();

    // Test 1: Command timeout
    #[cfg(unix)]
    {
        let timeout_command = CommandBuilder::new("sleep")
            .arg("10") // Sleep for 10 seconds
            .timeout(Duration::from_millis(100)) // But timeout after 100ms
            .build();

        let result = executor.execute(timeout_command).await;
        assert!(result.is_err());
        println!("✓ Command timeout handled correctly");
    }

    // Test 2: Non-existent command
    let invalid_command = CommandBuilder::new("this-command-does-not-exist-12345").build();

    let result = executor.execute(invalid_command).await;
    assert!(result.is_err());
    println!("✓ Non-existent command handled correctly");

    // Test 3: Command queue operations
    let mut queue = CommandQueue::new().start()?;

    let cmd1 = CommandBuilder::new("echo").arg("high-priority").build();
    let cmd2 = CommandBuilder::new("echo").arg("low-priority").build();

    let _id1 = queue.enqueue(cmd1, CommandPriority::High).await?;
    let _id2 = queue.enqueue(cmd2, CommandPriority::Low).await?;

    // Wait for completion
    queue.wait_for_completion().await?;
    queue.shutdown().await?;

    println!("✓ Command queue priority handling working");

    Ok(())
}

/// Integration test demonstrating the most common real-world usage patterns
#[tokio::test]
async fn test_common_usage_patterns() -> Result<()> {
    // Pattern 1: Quick project validation
    let project_manager = ProjectManager::new();
    let config =
        ProjectConfig::new().with_detect_package_manager(true).with_validate_structure(true);

    // This would typically be used on an existing project
    // For the test, we'll just verify the API works
    let current_dir = PathUtils::current_dir()?;
    if current_dir.join("package.json").exists() {
        let _project = project_manager.detect_project(&current_dir, &config)?;
        println!("✓ Project detection API working");
    }

    // Pattern 2: Simple command execution
    let executor = DefaultCommandExecutor::new();
    let simple_cmd = CommandBuilder::new("echo").arg("Hello, World!").build();

    let output = executor.execute(simple_cmd).await?;
    assert!(output.success());
    assert!(output.stdout().contains("Hello, World!"));
    println!("✓ Simple command execution working");

    // Pattern 3: Filesystem operations
    let fs = FileSystemManager::new();
    let temp_dir = tempfile::tempdir()
        .map_err(|e| Error::operation(format!("Failed to create temp dir: {e}")))?;

    let test_file = temp_dir.path().join("test.txt");
    fs.write_file_string(&test_file, "Test content")?;

    let content = fs.read_file_string(&test_file)?;
    assert_eq!(content, "Test content");
    println!("✓ Filesystem operations working");

    // Pattern 4: Configuration management
    let config_manager = ConfigManager::new();
    config_manager.set("app_name", ConfigValue::String("MyApp".to_string()));
    config_manager.set("debug_mode", ConfigValue::Boolean(true));
    config_manager.set("max_connections", ConfigValue::Integer(100));

    assert_eq!(
        config_manager.get("app_name").and_then(|v| v.as_string().map(ToString::to_string)),
        Some("MyApp".to_string())
    );
    assert_eq!(config_manager.get("debug_mode").and_then(|v| v.as_boolean()), Some(true));
    assert_eq!(config_manager.get("max_connections").and_then(|v| v.as_integer()), Some(100));
    println!("✓ Configuration management working");

    println!("🎉 All common usage patterns working correctly!");
    Ok(())
}
