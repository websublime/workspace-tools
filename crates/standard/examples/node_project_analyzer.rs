//! # Node.js Project Analyzer
//!
//! This example demonstrates a practical application that combines multiple features
//! of the sublime_standard_tools crate to analyze Node.js projects, including:
//! - Project detection and validation
//! - Package manager operations
//! - Dependency analysis
//! - Script execution
//! - Result caching
//! - Configuration management
//!
//! Run with a path to a Node.js project: `cargo run --example node_project_analyzer -- /path/to/project`

#![allow(clippy::print_stdout)]
#![allow(clippy::uninlined_format_args)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

use std::{
    collections::HashMap,
    env,
    error::Error,
    path::{Path, PathBuf},
    time::Duration,
};
use sublime_standard_tools::{
    cache::{Cache, CacheConfig, CacheStrategy},
    command::{CommandBuilder, CommandExecutor, DefaultCommandExecutor},
    config::{ConfigManager, ConfigScope, ConfigValue},
    diagnostic::{DiagnosticCollector, DiagnosticLevel},
    error::{StandardError, StandardResult},
    project::{
        FileSystem, FileSystemManager, PackageManagerKind, PathExt, PathUtils, Project,
        ProjectConfig, ProjectManager, ValidationStatus,
    },
    validation::{ValidationResult, ValidationRule, Validator},
};

/// Configuration for the project analyzer
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
struct AnalyzerConfig {
    /// Maximum depth to scan for dependencies
    max_depth: usize,
    /// Whether to scan dev dependencies
    include_dev_deps: bool,
    /// Whether to execute npm scripts
    run_scripts: bool,
    /// Whether to scan node_modules content
    scan_node_modules: bool,
    /// Maximum execution time for scripts in seconds
    script_timeout: u64,
    /// Whether to cache results
    use_cache: bool,
    /// Cache TTL in seconds
    cache_ttl: u64,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            max_depth: 1,
            include_dev_deps: false,
            run_scripts: false,
            scan_node_modules: false,
            script_timeout: 30,
            use_cache: true,
            cache_ttl: 3600, // 1 hour
        }
    }
}

/// Analysis result for a Node.js project
#[derive(Debug, Clone)]
struct ProjectAnalysisResult {
    /// Project name
    name: String,
    /// Project version
    version: String,
    /// Project root path
    root: PathBuf,
    /// Package manager used
    package_manager: Option<PackageManagerKind>,
    /// Dependencies found
    dependencies: HashMap<String, String>,
    /// Development dependencies found
    dev_dependencies: HashMap<String, String>,
    /// Scripts defined in package.json
    scripts: HashMap<String, String>,
    /// Size statistics for the project
    stats: ProjectStats,
    /// Analysis timestamp
    timestamp: chrono::DateTime<chrono::Local>,
    /// Validation status
    validation: ValidationStatus,
    /// Script execution results (if requested)
    script_results: Option<HashMap<String, ScriptResult>>,
}

/// Statistics about project size
#[derive(Debug, Clone)]
struct ProjectStats {
    /// Total number of files
    total_files: usize,
    /// Total size in bytes
    total_size: u64,
    /// Number of JavaScript files
    js_files: usize,
    /// Number of TypeScript files
    ts_files: usize,
    /// Size of node_modules in bytes
    node_modules_size: Option<u64>,
}

/// Result of executing a script
#[derive(Debug, Clone)]
struct ScriptResult {
    /// Script name
    name: String,
    /// Script command
    command: String,
    /// Exit code
    exit_code: Option<i32>,
    /// Standard output
    stdout: String,
    /// Standard error
    stderr: String,
    /// Execution duration
    duration: Duration,
    /// Whether the script completed successfully
    success: bool,
}

/// Validation rule for package.json
struct PackageJsonValidator;

impl ValidationRule<Project> for PackageJsonValidator {
    #[allow(clippy::single_char_pattern)]
    fn validate(&self, project: &Project) -> ValidationResult {
        // Get package.json if available
        let Some(package_json) = project.package_json() else {
            return ValidationResult::Error(vec!["Package.json not found".to_string()]);
        };

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate name
        if package_json.name.is_empty() {
            errors.push("Package name is empty".to_string());
        } else if package_json.name.contains(" ") {
            errors.push(format!("Invalid package name '{}' (contains spaces)", package_json.name));
        }

        // Validate version
        if package_json.version.is_empty() {
            errors.push("Package version is empty".to_string());
        } else if !package_json.version.split('.').all(|part| part.parse::<u32>().is_ok()) {
            errors.push(format!("Invalid version format: '{}'", package_json.version));
        }

        // Validate scripts
        if package_json.scripts.is_empty() {
            warnings.push("No scripts defined in package.json".to_string());
        } else if !package_json.scripts.contains_key("test") {
            warnings.push("No test script defined".to_string());
        }

        // Check for common dependencies
        let has_typescript = package_json.dependencies.contains_key("typescript")
            || package_json.dev_dependencies.contains_key("typescript");

        let has_types = package_json.dependencies.keys().any(|k| k.starts_with("@types/"))
            || package_json.dev_dependencies.keys().any(|k| k.starts_with("@types/"));

        if has_typescript && !has_types {
            warnings.push("TypeScript is used but no @types packages found".to_string());
        }

        if !has_typescript && has_types {
            warnings.push("@types packages found but TypeScript is not installed".to_string());
        }

        // Return validation result
        if !errors.is_empty() {
            ValidationResult::Error(errors)
        } else if !warnings.is_empty() {
            ValidationResult::Warning(warnings)
        } else {
            ValidationResult::Valid
        }
    }
}

/// Node.js project analyzer
struct ProjectAnalyzer {
    /// Project manager for project detection
    project_manager: ProjectManager,
    /// Command executor for running scripts
    command_executor: DefaultCommandExecutor,
    /// File system manager for file operations
    fs_manager: FileSystemManager,
    /// Cache for analysis results
    cache: Cache<PathBuf, ProjectAnalysisResult>,
    /// Configuration manager
    config_manager: ConfigManager,
    /// Diagnostic collector
    diagnostics: DiagnosticCollector,
    /// Project validator
    validator: Validator<Project>,
}

impl ProjectAnalyzer {
    /// Creates a new project analyzer
    #[allow(clippy::ignored_unit_patterns)]
    fn new() -> Self {
        // Initialize components
        let project_manager = ProjectManager::new();
        let command_executor = DefaultCommandExecutor::new();
        let fs_manager = FileSystemManager::new();
        let diagnostics = DiagnosticCollector::new();

        // Create a cache for analysis results
        let cache_config = CacheConfig {
            default_ttl: Duration::from_secs(3600), // 1 hour
            capacity: 50,                           // Store up to 50 results
            strategy: CacheStrategy::LRU,           // Least recently used eviction
        };
        let cache = Cache::<PathBuf, ProjectAnalysisResult>::with_config(cache_config);

        // Configure config manager
        let mut config_manager = ConfigManager::new();

        // Try to find configuration file locations
        let home_dir = dirs::home_dir();

        // Global config in user's home directory
        if let Some(home) = &home_dir {
            let global_config = home.join(".node-analyzer.json");
            config_manager.set_path(ConfigScope::Global, global_config);
        }

        // User config in current directory
        let user_config = PathBuf::from("node-analyzer.json");
        config_manager.set_path(ConfigScope::User, user_config);

        // Create default configuration if it doesn't exist
        Self::initialize_default_config(&config_manager);

        // Try to load configurations
        match config_manager.load_all() {
            Ok(_) => {
                diagnostics.info("config", "Configuration loaded successfully");
            }
            Err(e) => {
                diagnostics.warning("config", format!("Failed to load configuration: {}", e));
                // Continue with defaults
            }
        }

        // Create validator with custom rules
        let mut validator = Validator::<Project>::new();
        validator.add_rule(PackageJsonValidator);

        Self {
            project_manager,
            command_executor,
            fs_manager,
            cache,
            config_manager,
            diagnostics,
            validator,
        }
    }

    /// Initializes default configuration if none exists
    fn initialize_default_config(config_manager: &ConfigManager) {
        // Check if we already have configuration values
        if config_manager.get("analyzer").is_some() {
            return;
        }

        // Create default configuration
        let mut defaults = HashMap::new();

        // Analyzer settings
        let mut analyzer_defaults = HashMap::new();
        analyzer_defaults.insert("maxDepth".to_string(), ConfigValue::Integer(1));
        analyzer_defaults.insert("includeDevDeps".to_string(), ConfigValue::Boolean(false));
        analyzer_defaults.insert("runScripts".to_string(), ConfigValue::Boolean(false));
        analyzer_defaults.insert("scanNodeModules".to_string(), ConfigValue::Boolean(false));
        analyzer_defaults.insert("scriptTimeout".to_string(), ConfigValue::Integer(30));
        analyzer_defaults.insert("useCache".to_string(), ConfigValue::Boolean(true));
        analyzer_defaults.insert("cacheTTL".to_string(), ConfigValue::Integer(3600));

        defaults.insert("analyzer".to_string(), ConfigValue::Map(analyzer_defaults));

        // Scripts to run by default
        let default_scripts =
            vec![ConfigValue::String("test".to_string()), ConfigValue::String("build".to_string())];

        defaults.insert("defaultScripts".to_string(), ConfigValue::Array(default_scripts));

        // Set default values
        for (key, value) in defaults {
            config_manager.set(&key, value);
        }
    }

    /// Loads analyzer configuration from config manager
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn load_config(&self) -> AnalyzerConfig {
        let mut config = AnalyzerConfig::default();

        if let Some(analyzer_value) = self.config_manager.get("analyzer") {
            if let Some(analyzer_map) = analyzer_value.as_map() {
                // Update configuration from map
                if let Some(max_depth) = analyzer_map.get("maxDepth") {
                    if let Some(value) = max_depth.as_integer() {
                        config.max_depth = value as usize;
                    }
                }

                if let Some(include_dev) = analyzer_map.get("includeDevDeps") {
                    if let Some(value) = include_dev.as_boolean() {
                        config.include_dev_deps = value;
                    }
                }

                if let Some(run_scripts) = analyzer_map.get("runScripts") {
                    if let Some(value) = run_scripts.as_boolean() {
                        config.run_scripts = value;
                    }
                }

                if let Some(scan_modules) = analyzer_map.get("scanNodeModules") {
                    if let Some(value) = scan_modules.as_boolean() {
                        config.scan_node_modules = value;
                    }
                }

                if let Some(timeout) = analyzer_map.get("scriptTimeout") {
                    if let Some(value) = timeout.as_integer() {
                        config.script_timeout = value as u64;
                    }
                }

                if let Some(use_cache) = analyzer_map.get("useCache") {
                    if let Some(value) = use_cache.as_boolean() {
                        config.use_cache = value;
                    }
                }

                if let Some(cache_ttl) = analyzer_map.get("cacheTTL") {
                    if let Some(value) = cache_ttl.as_integer() {
                        config.cache_ttl = value as u64;
                    }
                }
            }
        }

        config
    }

    /// Analyzes a Node.js project at the given path
    async fn analyze_project(&self, path: &Path) -> StandardResult<ProjectAnalysisResult> {
        let config = self.load_config();
        self.diagnostics.info("analyzer", format!("Analyzing project at: {}", path.display()));

        // Normalize path
        let path = path.normalize();

        // Check if we have a cached result
        if config.use_cache {
            if let Some(cached_result) = self.cache.get(&path) {
                self.diagnostics
                    .info("cache", format!("Using cached analysis for {}", path.display()));
                return Ok(cached_result);
            }
        }

        // Detect the project
        let project_config =
            ProjectConfig::new().detect_package_manager(true).validate_structure(true);

        let project = self.project_manager.detect_project(&path, &project_config)?;

        // Validate the project
        let validation_result = self.validator.validate(&project);
        if validation_result.has_errors() {
            if let ValidationResult::Error(errors) = &validation_result {
                for error in errors {
                    self.diagnostics.error("validation", error.clone());
                }
            }
        } else if validation_result.has_warnings() {
            if let ValidationResult::Warning(warnings) = &validation_result {
                for warning in warnings {
                    self.diagnostics.warning("validation", warning.clone());
                }
            }
        }

        // Get package.json data
        let package_json = project.package_json().ok_or_else(|| {
            StandardError::operation("Package.json not found or invalid".to_string())
        })?;

        // Collect project stats
        let stats = self.collect_project_stats(&project, &config)?;

        // Prepare analysis result
        let mut result = ProjectAnalysisResult {
            name: package_json.name.clone(),
            version: package_json.version.clone(),
            root: project.root().to_path_buf(),
            package_manager: project
                .package_manager()
                .map(sublime_standard_tools::project::PackageManager::kind),
            dependencies: package_json.dependencies.clone(),
            dev_dependencies: package_json.dev_dependencies.clone(),
            scripts: package_json.scripts.clone(),
            stats,
            timestamp: chrono::Local::now(),
            validation: project.validation_status().clone(),
            script_results: None,
        };

        // Execute scripts if requested
        if config.run_scripts && !package_json.scripts.is_empty() {
            let script_results = self.execute_scripts(&project, &config).await?;
            result.script_results = Some(script_results);
        }

        // Cache the result if caching is enabled
        if config.use_cache {
            self.cache.put_with_ttl(
                path.clone(),
                result.clone(),
                Duration::from_secs(config.cache_ttl),
            );
        }

        Ok(result)
    }

    /// Collects statistics about the project
    fn collect_project_stats(
        &self,
        project: &Project,
        config: &AnalyzerConfig,
    ) -> StandardResult<ProjectStats> {
        self.diagnostics.info(
            "stats",
            format!("Collecting statistics for project at {}", project.root().display()),
        );

        let root = project.root();
        let mut stats = ProjectStats {
            total_files: 0,
            total_size: 0,
            js_files: 0,
            ts_files: 0,
            node_modules_size: None,
        };

        // Get all files in the project (excluding node_modules unless configured)
        let paths = match self.fs_manager.walk_dir(root) {
            Ok(paths) => paths,
            Err(e) => {
                self.diagnostics
                    .error("stats", format!("Failed to walk directory {}: {}", root.display(), e));
                return Err(e.into());
            }
        };

        // Process each file
        for path in paths {
            // Skip node_modules if not configured to scan
            if !config.scan_node_modules && path.starts_with(root.join("node_modules")) {
                continue;
            }

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Count file
            stats.total_files += 1;

            // Get file size
            if let Ok(metadata) = self.fs_manager.metadata(&path) {
                let file_size = metadata.len();
                stats.total_size += file_size;

                // Track node_modules size separately
                if path.starts_with(root.join("node_modules")) {
                    stats.node_modules_size =
                        Some(stats.node_modules_size.unwrap_or(0) + file_size);
                }
            }

            // Count by file type
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if ext == "js" || ext == "jsx" || ext == "mjs" {
                    stats.js_files += 1;
                } else if ext == "ts" || ext == "tsx" {
                    stats.ts_files += 1;
                }
            }
        }

        self.diagnostics.info(
            "stats",
            format!(
                "Project statistics: {} files, {} bytes total, {} JS files, {} TS files",
                stats.total_files, stats.total_size, stats.js_files, stats.ts_files
            ),
        );

        Ok(stats)
    }

    /// Executes scripts from package.json
    async fn execute_scripts(
        &self,
        project: &Project,
        config: &AnalyzerConfig,
    ) -> StandardResult<HashMap<String, ScriptResult>> {
        let package_json = project
            .package_json()
            .ok_or_else(|| StandardError::operation("Package.json not found".to_string()))?;

        let mut results = HashMap::new();
        let scripts_to_run = self.get_scripts_to_run(package_json);

        self.diagnostics.info("scripts", format!("Executing {} scripts", scripts_to_run.len()));

        // Determine package manager command
        let package_manager = if let Some(pm) = project.package_manager() {
            pm.kind().command()
        } else {
            "npm" // Default to npm if no package manager detected
        };

        // Execute each script
        for (name, command) in scripts_to_run {
            self.diagnostics.info("scripts", format!("Executing script '{}': {}", name, command));

            let start_time = std::time::Instant::now();

            // Build command
            let cmd = CommandBuilder::new(package_manager)
                .arg("run")
                .arg(&name)
                .current_dir(project.root())
                .timeout(Duration::from_secs(config.script_timeout))
                .build();

            // Execute command
            let result = self.command_executor.execute(cmd).await;
            let duration = start_time.elapsed();

            match result {
                Ok(output) => {
                    self.diagnostics.info(
                        "scripts",
                        format!("Script '{}' completed successfully in {:?}", name, duration),
                    );

                    results.insert(
                        name.clone(),
                        ScriptResult {
                            name: name.clone(),
                            command: command.clone(),
                            exit_code: Some(output.status()),
                            stdout: output.stdout().to_string(),
                            stderr: output.stderr().to_string(),
                            duration,
                            success: true,
                        },
                    );
                }
                Err(e) => {
                    self.diagnostics.error("scripts", format!("Script '{}' failed: {}", name, e));

                    results.insert(
                        name.clone(),
                        ScriptResult {
                            name: name.clone(),
                            command: command.clone(),
                            exit_code: None,
                            stdout: String::new(),
                            stderr: format!("Execution failed: {}", e),
                            duration,
                            success: false,
                        },
                    );
                }
            }
        }

        Ok(results)
    }

    /// Gets the list of scripts to run from package.json
    fn get_scripts_to_run(
        &self,
        package_json: &sublime_standard_tools::project::PackageJson,
    ) -> HashMap<String, String> {
        let mut scripts_to_run = HashMap::new();

        // Get default scripts from config
        let mut default_scripts = Vec::new();
        if let Some(scripts_value) = self.config_manager.get("defaultScripts") {
            if let Some(scripts_array) = scripts_value.as_array() {
                for script in scripts_array {
                    if let Some(script_name) = script.as_string() {
                        default_scripts.push(script_name.to_string());
                    }
                }
            }
        }

        // Add scripts that exist in package.json
        for script_name in default_scripts {
            if let Some(script_cmd) = package_json.scripts.get(&script_name) {
                scripts_to_run.insert(script_name, script_cmd.clone());
            }
        }

        scripts_to_run
    }

    /// Generates a report for the analysis result
    #[allow(clippy::unused_self)]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::single_char_add_str)]
    fn generate_report(&self, result: &ProjectAnalysisResult) -> String {
        let mut report = String::new();

        // Project header
        report.push_str(&format!("# Project Analysis: {}\n\n", result.name));
        report.push_str(&format!("Version: {}\n", result.version));
        report.push_str(&format!("Analyzed at: {}\n", result.timestamp));
        report.push_str(&format!("Path: {}\n\n", result.root.display()));

        // Package manager
        if let Some(pm) = result.package_manager {
            report.push_str(&format!("Package Manager: {}\n\n", pm.command()));
        } else {
            report.push_str("Package Manager: Not detected\n\n");
        }

        // Validation status
        report.push_str("## Validation Status\n\n");
        match &result.validation {
            ValidationStatus::Valid => {
                report.push_str("Status: ✅ Valid\n\n");
            }
            ValidationStatus::Warning(warnings) => {
                report.push_str("Status: ⚠️ Warning\n\n");
                for warning in warnings {
                    report.push_str(&format!("- {}\n", warning));
                }
                report.push_str("\n");
            }
            ValidationStatus::Error(errors) => {
                report.push_str("Status: ❌ Error\n\n");
                for error in errors {
                    report.push_str(&format!("- {}\n", error));
                }
                report.push_str("\n");
            }
            ValidationStatus::NotValidated => {
                report.push_str("Status: Not validated\n\n");
            }
        }

        // Project stats
        report.push_str("## Project Statistics\n\n");
        report.push_str(&format!("Total Files: {}\n", result.stats.total_files));
        report.push_str(&format!("Total Size: {} bytes\n", result.stats.total_size));
        report.push_str(&format!("JavaScript Files: {}\n", result.stats.js_files));
        report.push_str(&format!("TypeScript Files: {}\n", result.stats.ts_files));

        if let Some(nm_size) = result.stats.node_modules_size {
            report.push_str(&format!("node_modules Size: {} bytes\n", nm_size));
        }
        report.push_str("\n");

        // Dependencies
        report.push_str("## Dependencies\n\n");
        if result.dependencies.is_empty() {
            report.push_str("No dependencies found.\n\n");
        } else {
            report.push_str(&format!("Total Dependencies: {}\n\n", result.dependencies.len()));
            for (name, version) in &result.dependencies {
                report.push_str(&format!("- {}: {}\n", name, version));
            }
            report.push_str("\n");
        }

        // Dev Dependencies
        report.push_str("## Dev Dependencies\n\n");
        if result.dev_dependencies.is_empty() {
            report.push_str("No dev dependencies found.\n\n");
        } else {
            report.push_str(&format!(
                "Total Dev Dependencies: {}\n\n",
                result.dev_dependencies.len()
            ));
            for (name, version) in &result.dev_dependencies {
                report.push_str(&format!("- {}: {}\n", name, version));
            }
            report.push_str("\n");
        }

        // Scripts
        report.push_str("## Scripts\n\n");
        if result.scripts.is_empty() {
            report.push_str("No scripts found.\n\n");
        } else {
            report.push_str(&format!("Total Scripts: {}\n\n", result.scripts.len()));
            for (name, command) in &result.scripts {
                report.push_str(&format!("- {}: `{}`\n", name, command));
            }
            report.push_str("\n");
        }

        // Script execution results
        if let Some(script_results) = &result.script_results {
            if !script_results.is_empty() {
                report.push_str("## Script Execution Results\n\n");

                for result in script_results.values() {
                    report.push_str(&format!("### Script: {}\n\n", result.name));
                    report.push_str(&format!("Command: `{}`\n", result.command));
                    report.push_str(&format!(
                        "Status: {}\n",
                        if result.success { "✅ Success" } else { "❌ Failed" }
                    ));
                    report.push_str(&format!("Duration: {:?}\n", result.duration));

                    if let Some(code) = result.exit_code {
                        report.push_str(&format!("Exit Code: {}\n", code));
                    }

                    if !result.stdout.is_empty() {
                        report.push_str("\nOutput:\n```\n");
                        report.push_str(&result.stdout);
                        report.push_str("\n```\n");
                    }

                    if !result.stderr.is_empty() {
                        report.push_str("\nErrors:\n```\n");
                        report.push_str(&result.stderr);
                        report.push_str("\n```\n");
                    }

                    report.push_str("\n");
                }
            }
        }

        report
    }

    /// Prints diagnostic information
    fn print_diagnostics(&self) {
        println!("\n=== Diagnostic Information ===\n");

        let entries = self.diagnostics.entries();
        if entries.is_empty() {
            println!("No diagnostics collected.");
            return;
        }

        for entry in &entries {
            let level_str = match entry.level {
                DiagnosticLevel::Info => "INFO",
                DiagnosticLevel::Warning => "WARNING",
                DiagnosticLevel::Error => "ERROR",
                DiagnosticLevel::Critical => "CRITICAL",
            };

            println!("[{}] {}: {}", level_str, entry.context, entry.message);

            if !entry.data.is_empty() {
                println!("  Data:");
                for (key, value) in &entry.data {
                    println!("    {}: {}", key, value);
                }
            }

            if let Some(duration) = entry.duration {
                println!("  Duration: {:?}", duration);
            }

            println!();
        }

        // Print stats
        let warnings = self.diagnostics.entries_with_level_at_or_above(DiagnosticLevel::Warning);
        let errors = self.diagnostics.entries_with_level_at_or_above(DiagnosticLevel::Error);

        println!("Total Diagnostics: {}", entries.len());
        println!("Warnings: {}", warnings.len() - errors.len());
        println!("Errors: {}", errors.len());
    }
}

// Helper function to add functionality to ProjectAnalyzer for file metadata
trait FileMetadata {
    /// Gets file metadata
    fn metadata(&self, path: &Path) -> std::io::Result<std::fs::Metadata>;
}

impl FileMetadata for FileSystemManager {
    fn metadata(&self, path: &Path) -> std::io::Result<std::fs::Metadata> {
        std::fs::metadata(path)
    }
}

#[allow(clippy::ignored_unit_patterns)]
#[tokio::main]
async fn main() -> StandardResult<()> {
    // Get the target path from command line arguments or use current directory
    let args: Vec<String> = env::args().collect();
    let target_path =
        if args.len() > 1 { PathBuf::from(&args[1]) } else { PathUtils::current_dir()? };

    println!("=== Node.js Project Analyzer ===");
    println!("Target path: {}", target_path.display());

    // Create the project analyzer
    let analyzer = ProjectAnalyzer::new();

    // Analyze the project
    match analyzer.analyze_project(&target_path).await {
        Ok(result) => {
            println!("\n=== Analysis Results ===\n");

            // Print basic information
            println!("Project: {} v{}", result.name, result.version);
            println!("Path: {}", result.root.display());

            if let Some(pm) = result.package_manager {
                println!("Package Manager: {}", pm.command());
            } else {
                println!("Package Manager: Not detected");
            }

            // Print dependencies count
            println!("\nDependencies: {}", result.dependencies.len());
            println!("Dev Dependencies: {}", result.dev_dependencies.len());

            // Print statistics
            println!("\nProject Statistics:");
            println!("  Total Files: {}", result.stats.total_files);
            println!("  Total Size: {} bytes", result.stats.total_size);
            println!("  JavaScript Files: {}", result.stats.js_files);
            println!("  TypeScript Files: {}", result.stats.ts_files);

            if let Some(nm_size) = result.stats.node_modules_size {
                println!("  node_modules Size: {} bytes", nm_size);
            }

            // Print validation status
            println!("\nValidation Status:");
            match &result.validation {
                ValidationStatus::Valid => {
                    println!("  Status: ✅ Valid");
                }
                ValidationStatus::Warning(warnings) => {
                    println!("  Status: ⚠️ Warning");
                    for warning in warnings {
                        println!("  - {}", warning);
                    }
                }
                ValidationStatus::Error(errors) => {
                    println!("  Status: ❌ Error");
                    for error in errors {
                        println!("  - {}", error);
                    }
                }
                ValidationStatus::NotValidated => {
                    println!("  Status: Not validated");
                }
            }

            // Print script execution results if available
            if let Some(script_results) = &result.script_results {
                if !script_results.is_empty() {
                    println!("\nScript Execution Results:");

                    for result in script_results.values() {
                        println!("\n  Script: {}", result.name);
                        println!("  Command: {}", result.command);
                        println!(
                            "  Status: {}",
                            if result.success { "✅ Success" } else { "❌ Failed" }
                        );
                        println!("  Duration: {:?}", result.duration);

                        if let Some(code) = result.exit_code {
                            println!("  Exit Code: {}", code);
                        }

                        if !result.stdout.is_empty() {
                            println!("\n  Output: (first 3 lines)");
                            for (i, line) in result.stdout.lines().take(3).enumerate() {
                                println!("    {}: {}", i + 1, line);
                            }
                            if result.stdout.lines().count() > 3 {
                                println!("    ...");
                            }
                        }
                    }
                }
            }

            // Generate and save full report
            let report = analyzer.generate_report(&result);
            let report_path = result.root.join("node-analyzer-report.md");

            println!("\nSaving full report to: {}", report_path.display());
            match std::fs::write(&report_path, report) {
                Ok(_) => println!("Report saved successfully"),
                Err(e) => println!("Failed to save report: {}", e),
            }
        }
        Err(e) => {
            println!("\n=== Analysis Failed ===");
            println!("Error: {}", e);

            // Print error chain
            let mut source = e.source(); // This will now work with Error trait in scope
            let mut depth = 1;
            while let Some(err) = source {
                println!("Cause {}: {}", depth, err);
                source = err.source();
                depth += 1;
            }
        }
    }

    // Print diagnostics
    analyzer.print_diagnostics();

    Ok(())
}
