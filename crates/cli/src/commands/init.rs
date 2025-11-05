//! Init command implementation.
//!
//! This module implements the `wnt init` command which initializes a workspace
//! for changeset-based version management.
//!
//! # What
//!
//! Provides the init command that:
//! - Detects workspace structure (single package or monorepo)
//! - Collects configuration through interactive prompts or CLI flags
//! - Generates repo.config.[format] file
//! - Creates necessary directory structure (.changesets, .changesets/history, .wnt-backups)
//! - Sets up .gitignore entries
//! - Creates example changeset file with documentation
//!
//! # How
//!
//! The command flow:
//! 1. Validates the target directory is a Node.js project
//! 2. Detects if it's a monorepo or single package
//! 3. Checks for existing configuration
//! 4. Collects configuration via prompts or flags
//! 5. Generates configuration file
//! 6. Creates directory structure
//! 7. Updates .gitignore
//! 8. Creates example changeset
//! 9. Outputs success message
//!
//! # Why
//!
//! Initialization must be robust to handle various project structures,
//! provide helpful defaults, and ensure proper setup for the changeset workflow.
//! Clear output and documentation help users understand the setup.

use crate::cli::commands::InitArgs;
use crate::error::{CliError, Result};
use crate::output::{JsonResponse, OutputFormat};
use dialoguer::{Input, MultiSelect, Select};
use serde::Serialize;
use std::path::{Path, PathBuf};
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait, MonorepoKind};
use tracing::{debug, info};

/// Execute the init command.
///
/// Initializes a workspace for changeset-based version management by creating
/// configuration, directory structure, and example files.
///
/// # Arguments
///
/// * `args` - Command arguments from CLI
/// * `root` - Workspace root directory
/// * `format` - Output format for the command result
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if initialization fails.
///
/// # Errors
///
/// Returns an error if:
/// - The directory is not a Node.js project
/// - Configuration already exists and force flag is not set
/// - File system operations fail
/// - Configuration validation fails
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::init::execute_init;
/// use sublime_cli_tools::cli::commands::InitArgs;
/// use sublime_cli_tools::output::OutputFormat;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = InitArgs {
///     non_interactive: true,
///     strategy: Some("independent".to_string()),
///     ..Default::default()
/// };
///
/// execute_init(&args, Path::new("."), OutputFormat::Human).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_init(args: &InitArgs, root: &Path, format: OutputFormat) -> Result<()> {
    debug!("Initializing workspace at: {}", root.display());

    // Validate workspace is a Node.js project
    validate_nodejs_project(root).await?;

    // Detect workspace type
    let workspace_info = detect_workspace_type(root).await?;
    info!("Detected workspace type: {}", workspace_info.kind_description());

    // Check for existing configuration
    let config_path = find_existing_config(root).await?;
    if let Some(existing_path) = &config_path {
        if !args.force {
            return Err(CliError::configuration(format!(
                "Configuration file already exists: {}. Use --force to overwrite.",
                existing_path.display()
            )));
        }
        info!("Force flag set, will overwrite existing configuration");
    }

    // Collect configuration
    let init_config = if args.non_interactive {
        collect_config_non_interactive(args, &workspace_info)
    } else {
        collect_config_interactive(args, &workspace_info)?
    };

    // Validate configuration
    validate_init_config(&init_config)?;

    // Generate configuration file
    let config_file_path = generate_config_file(root, &init_config).await?;
    info!("Configuration file created: {}", config_file_path.display());

    // Create directory structure
    create_directory_structure(root, &init_config.changeset_path).await?;
    info!("Directory structure created");

    // Update .gitignore
    update_gitignore(root).await?;
    info!(".gitignore updated");

    // Create example changeset
    create_example_changeset(root, &init_config.changeset_path).await?;
    info!("Example changeset created");

    // Output result
    output_init_result(&config_file_path, &init_config, format)?;

    Ok(())
}

/// Workspace information detected during initialization.
#[derive(Debug)]
struct WorkspaceInfo {
    /// Whether this is a monorepo
    is_monorepo: bool,
    /// Monorepo kind if applicable
    monorepo_kind: Option<MonorepoKind>,
    /// Number of packages detected
    package_count: usize,
}

impl WorkspaceInfo {
    /// Returns a human-readable description of the workspace kind.
    fn kind_description(&self) -> &str {
        if self.is_monorepo {
            match &self.monorepo_kind {
                Some(MonorepoKind::NpmWorkSpace) => "npm workspaces monorepo",
                Some(MonorepoKind::YarnWorkspaces) => "yarn workspaces monorepo",
                Some(MonorepoKind::PnpmWorkspaces) => "pnpm workspaces monorepo",
                Some(MonorepoKind::BunWorkspaces) => "bun workspaces monorepo",
                Some(MonorepoKind::DenoWorkspaces) => "deno workspaces monorepo",
                Some(MonorepoKind::Custom { .. }) => "custom monorepo",
                None => "monorepo",
            }
        } else {
            "single package project"
        }
    }

    /// Returns the recommended default strategy.
    fn recommended_strategy(&self) -> &str {
        if self.is_monorepo && self.package_count > 3 {
            "independent"
        } else if self.is_monorepo {
            "unified"
        } else {
            "independent"
        }
    }
}

/// Configuration collected during initialization.
#[derive(Debug, Serialize)]
struct InitConfig {
    /// Changeset directory path
    changeset_path: String,
    /// Available environments
    environments: Vec<String>,
    /// Default environments
    default_environments: Vec<String>,
    /// Versioning strategy
    strategy: String,
    /// NPM registry URL
    registry: String,
    /// Configuration file format
    config_format: String,
}

/// Validates that the directory is a Node.js project.
async fn validate_nodejs_project(root: &Path) -> Result<()> {
    let fs = FileSystemManager::new();
    let package_json = root.join("package.json");

    if !fs.exists(&package_json).await {
        return Err(CliError::validation(
            "No package.json found. This does not appear to be a Node.js project.",
        ));
    }

    Ok(())
}

/// Detects the workspace type (monorepo or single package).
///
/// A project is considered a monorepo if:
/// 1. The package.json contains a "workspaces" field (even if empty), OR
/// 2. The detector finds multiple packages
///
/// This ensures that newly created monorepos with empty workspace arrays
/// are correctly identified as monorepos.
async fn detect_workspace_type(root: &Path) -> Result<WorkspaceInfo> {
    let detector = MonorepoDetector::new();
    let fs = FileSystemManager::new();

    // First, check if package.json explicitly declares workspaces
    let has_workspace_field = check_has_workspace_field(root, &fs).await?;

    // Then check with the detector
    match detector.is_monorepo_root(root).await {
        Ok(Some(kind)) => {
            // It's a monorepo, try to get descriptor for package count
            match detector.detect_monorepo(root).await {
                Ok(descriptor) => Ok(WorkspaceInfo {
                    is_monorepo: true,
                    monorepo_kind: Some(kind),
                    package_count: descriptor.packages().len(),
                }),
                Err(_) => {
                    // Fallback if descriptor detection fails
                    Ok(WorkspaceInfo {
                        is_monorepo: true,
                        monorepo_kind: Some(kind),
                        package_count: 0,
                    })
                }
            }
        }
        Ok(None) => {
            // Detector says not a monorepo, but check if workspaces field exists
            if has_workspace_field {
                // package.json has workspaces field, so it IS a monorepo
                // (even if the array is empty - newly created monorepo)
                debug!(
                    "Detected monorepo from workspaces field in package.json (detector returned None)"
                );
                Ok(WorkspaceInfo {
                    is_monorepo: true,
                    monorepo_kind: Some(MonorepoKind::NpmWorkSpace), // Default to npm
                    package_count: 0,                                // No packages yet
                })
            } else {
                // Single package project
                Ok(WorkspaceInfo { is_monorepo: false, monorepo_kind: None, package_count: 1 })
            }
        }
        Err(e) => Err(CliError::execution(format!("Failed to detect workspace type: {e}"))),
    }
}

/// Checks if the package.json contains a "workspaces" field.
///
/// Returns true if the field exists, regardless of whether it's empty or not.
/// This is critical for detecting newly created monorepos that don't have
/// any packages defined yet.
async fn check_has_workspace_field(root: &Path, fs: &FileSystemManager) -> Result<bool> {
    let package_json_path = root.join("package.json");

    if !fs.exists(&package_json_path).await {
        return Ok(false);
    }

    let content = fs.read_file_string(&package_json_path).await.map_err(|e| {
        CliError::io(format!(
            "Failed to read package.json at {}: {}",
            package_json_path.display(),
            e
        ))
    })?;

    let package_json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        CliError::validation(format!(
            "Failed to parse package.json at {}: {}",
            package_json_path.display(),
            e
        ))
    })?;

    Ok(package_json.get("workspaces").is_some())
}

/// Finds existing configuration file if present.
async fn find_existing_config(root: &Path) -> Result<Option<PathBuf>> {
    let fs = FileSystemManager::new();
    let formats = ["toml", "yaml", "yml", "json"];

    for format in &formats {
        let path = root.join(format!("repo.config.{format}"));
        if fs.exists(&path).await {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

/// Collects configuration in non-interactive mode.
fn collect_config_non_interactive(args: &InitArgs, workspace_info: &WorkspaceInfo) -> InitConfig {
    // Use provided values or defaults
    let changeset_path = args.changeset_path.to_string_lossy().to_string();

    let environments = args.environments.clone().unwrap_or_else(|| {
        vec!["dev".to_string(), "staging".to_string(), "production".to_string()]
    });

    let default_environments =
        args.default_env.clone().unwrap_or_else(|| vec!["production".to_string()]);

    let strategy =
        args.strategy.clone().unwrap_or_else(|| workspace_info.recommended_strategy().to_string());

    let registry = args.registry.clone();

    let config_format = args.config_format.clone().unwrap_or_else(|| "toml".to_string());

    InitConfig {
        changeset_path,
        environments,
        default_environments,
        strategy,
        registry,
        config_format,
    }
}

/// Collects configuration through interactive prompts.
fn collect_config_interactive(
    args: &InitArgs,
    workspace_info: &WorkspaceInfo,
) -> Result<InitConfig> {
    eprintln!("\n🚀 Initialize Workspace Node Tools\n");
    eprintln!(
        "Detected: {} with {} package(s)\n",
        workspace_info.kind_description(),
        workspace_info.package_count
    );

    // Changeset path - args.changeset_path has a default value, so just use it
    let changeset_path = args.changeset_path.to_string_lossy().to_string();

    // Available environments
    let environments = if let Some(envs) = &args.environments {
        envs.clone()
    } else {
        let default = ["dev", "staging", "production"];
        let input: String = Input::new()
            .with_prompt("Available environments (comma-separated)")
            .default(default.join(", "))
            .interact_text()
            .map_err(|e| CliError::user(format!("Failed to read input: {e}")))?;
        parse_comma_separated(&input)
    };

    // Default environments
    let default_environments = if let Some(default_env) = &args.default_env {
        default_env.clone()
    } else {
        // Create defaults array: true for last item (production), false for others
        let defaults: Vec<bool> =
            (0..environments.len()).map(|i| i == environments.len().saturating_sub(1)).collect();

        let indices = MultiSelect::new()
            .with_prompt("Select default environments (use space to select, enter to confirm)")
            .items(&environments)
            .defaults(&defaults)
            .interact()
            .map_err(|e| CliError::user(format!("Failed to read selection: {e}")))?;

        indices.iter().map(|&i| environments[i].clone()).collect()
    };

    // Versioning strategy
    let strategy = if let Some(strat) = &args.strategy {
        strat.clone()
    } else {
        let strategies = vec!["independent", "unified"];
        let recommended_idx = usize::from(workspace_info.recommended_strategy() == "unified");

        let selection = Select::new()
            .with_prompt("Versioning strategy")
            .items(&strategies)
            .default(recommended_idx)
            .interact()
            .map_err(|e| CliError::user(format!("Failed to read selection: {e}")))?;

        strategies[selection].to_string()
    };

    // NPM registry - args.registry has a default value, so just use it
    let registry = args.registry.clone();

    // Configuration format
    let config_format = if let Some(fmt) = &args.config_format {
        fmt.clone()
    } else {
        let formats = vec!["toml", "yaml", "json"];
        let selection = Select::new()
            .with_prompt("Configuration format")
            .items(&formats)
            .default(0)
            .interact()
            .map_err(|e| CliError::user(format!("Failed to read selection: {e}")))?;

        formats[selection].to_string()
    };

    Ok(InitConfig {
        changeset_path,
        environments,
        default_environments,
        strategy,
        registry,
        config_format,
    })
}

/// Validates the collected configuration.
fn validate_init_config(config: &InitConfig) -> Result<()> {
    // Validate strategy
    if config.strategy != "independent" && config.strategy != "unified" {
        return Err(CliError::validation(format!(
            "Invalid strategy '{}'. Must be 'independent' or 'unified'",
            config.strategy
        )));
    }

    // Validate format
    let valid_formats = ["toml", "yaml", "yml", "json"];
    if !valid_formats.contains(&config.config_format.as_str()) {
        return Err(CliError::validation(format!(
            "Invalid config format '{}'. Must be one of: toml, yaml, yml, json",
            config.config_format
        )));
    }

    // Validate environments not empty
    if config.environments.is_empty() {
        return Err(CliError::validation("At least one environment must be specified"));
    }

    // Validate default environments are subset of environments
    for default_env in &config.default_environments {
        if !config.environments.contains(default_env) {
            return Err(CliError::validation(format!(
                "Default environment '{default_env}' is not in available environments"
            )));
        }
    }

    // Validate changeset path is not empty
    if config.changeset_path.trim().is_empty() {
        return Err(CliError::validation("Changeset path cannot be empty"));
    }

    // Validate registry URL format
    if !config.registry.starts_with("http://") && !config.registry.starts_with("https://") {
        return Err(CliError::validation("Registry URL must start with http:// or https://"));
    }

    Ok(())
}

/// Generates the configuration file.
async fn generate_config_file(root: &Path, config: &InitConfig) -> Result<PathBuf> {
    let fs = FileSystemManager::new();

    // Create PackageToolsConfig with user settings
    let mut pkg_config = PackageToolsConfig::default();

    // Extract workspace patterns from package.json if it's a monorepo
    let workspace_patterns = extract_workspace_patterns(root, &fs).await?;

    // Set workspace config if this is a monorepo
    if workspace_patterns.is_empty() {
        // Check if it should be a monorepo (has workspaces field even if empty)
        let has_workspace_field = check_has_workspace_field(root, &fs).await?;
        if has_workspace_field {
            // Monorepo with empty patterns - still need to include workspace config
            pkg_config.workspace = Some(sublime_pkg_tools::config::WorkspaceConfig::empty());
            debug!("Added empty workspace config for monorepo with no patterns yet");
        }
        // If no workspaces field, workspace remains None (single-package project)
    } else {
        pkg_config.workspace =
            Some(sublime_pkg_tools::config::WorkspaceConfig::new(workspace_patterns.clone()));
        debug!("Added workspace patterns to config: {:?}", workspace_patterns);
    }

    // Set changeset config
    pkg_config.changeset.path.clone_from(&config.changeset_path);
    pkg_config.changeset.history_path = format!("{}/history", config.changeset_path);
    pkg_config.changeset.available_environments.clone_from(&config.environments);
    pkg_config.changeset.default_environments.clone_from(&config.default_environments);

    // Set version config
    pkg_config.version.strategy = if config.strategy == "unified" {
        sublime_pkg_tools::types::VersioningStrategy::Unified
    } else {
        sublime_pkg_tools::types::VersioningStrategy::Independent
    };

    // Set upgrade config registry
    pkg_config.upgrade.registry.default_registry.clone_from(&config.registry);

    // Serialize based on format
    let config_content = match config.config_format.as_str() {
        "json" => serde_json::to_string_pretty(&pkg_config)
            .map_err(|e| CliError::execution(format!("Failed to serialize JSON: {e}")))?,
        "yaml" | "yml" => serde_yaml::to_string(&pkg_config)
            .map_err(|e| CliError::execution(format!("Failed to serialize YAML: {e}")))?,
        "toml" => toml::to_string_pretty(&pkg_config)
            .map_err(|e| CliError::execution(format!("Failed to serialize TOML: {e}")))?,
        _ => {
            return Err(CliError::validation(format!(
                "Unsupported format: {}",
                config.config_format
            )));
        }
    };

    // Write to file
    let config_path = root.join(format!("repo.config.{}", config.config_format));
    fs.write_file(&config_path, config_content.as_bytes()).await.map_err(|e| {
        CliError::io(format!("Failed to write configuration file {}: {}", config_path.display(), e))
    })?;

    Ok(config_path)
}

/// Extracts workspace patterns from package.json.
///
/// Returns a vector of workspace patterns (e.g., `["packages/*", "apps/*"]`).
/// Returns an empty vector if no patterns are found or if it's not a monorepo.
///
/// Supports multiple formats:
/// - String array: `"workspaces": ["packages/*"]`
/// - Object with packages: `"workspaces": { "packages": ["packages/*"] }`
async fn extract_workspace_patterns(root: &Path, fs: &FileSystemManager) -> Result<Vec<String>> {
    let package_json_path = root.join("package.json");

    if !fs.exists(&package_json_path).await {
        return Ok(vec![]);
    }

    let content = fs.read_file_string(&package_json_path).await.map_err(|e| {
        CliError::io(format!(
            "Failed to read package.json at {}: {}",
            package_json_path.display(),
            e
        ))
    })?;

    let package_json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        CliError::validation(format!(
            "Failed to parse package.json at {}: {}",
            package_json_path.display(),
            e
        ))
    })?;

    // Check for workspaces field
    if let Some(workspaces) = package_json.get("workspaces") {
        // Handle array format: "workspaces": ["packages/*"]
        if let Some(arr) = workspaces.as_array() {
            let patterns: Vec<String> =
                arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
            debug!("Extracted workspace patterns (array format): {:?}", patterns);
            return Ok(patterns);
        }

        // Handle object format: "workspaces": { "packages": ["packages/*"] }
        if let Some(obj) = workspaces.as_object()
            && let Some(packages) = obj.get("packages")
            && let Some(arr) = packages.as_array()
        {
            let patterns: Vec<String> =
                arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
            debug!("Extracted workspace patterns (object format): {:?}", patterns);
            return Ok(patterns);
        }
    }

    Ok(vec![])
}

/// Creates the directory structure for changesets and backups.
async fn create_directory_structure(root: &Path, changeset_path: &str) -> Result<()> {
    let fs = FileSystemManager::new();

    // Create .changesets directory
    let changesets_dir = root.join(changeset_path);
    fs.create_dir_all(&changesets_dir).await.map_err(|e| {
        CliError::io(format!("Failed to create directory {}: {}", changesets_dir.display(), e))
    })?;

    // Create .changesets/history directory
    let history_dir = changesets_dir.join("history");
    fs.create_dir_all(&history_dir).await.map_err(|e| {
        CliError::io(format!("Failed to create directory {}: {}", history_dir.display(), e))
    })?;

    // Create .gitkeep in history directory
    let gitkeep_path = history_dir.join(".gitkeep");
    fs.write_file(&gitkeep_path, b"").await.map_err(|e| {
        CliError::io(format!("Failed to create .gitkeep file {}: {}", gitkeep_path.display(), e))
    })?;

    // Create .wnt-backups directory
    let backups_dir = root.join(".wnt-backups");
    fs.create_dir_all(&backups_dir).await.map_err(|e| {
        CliError::io(format!("Failed to create directory {}: {}", backups_dir.display(), e))
    })?;

    Ok(())
}

/// Updates .gitignore with necessary entries.
async fn update_gitignore(root: &Path) -> Result<()> {
    let fs = FileSystemManager::new();
    let gitignore_path = root.join(".gitignore");

    // Read existing .gitignore or create new content
    let existing_content = if fs.exists(&gitignore_path).await {
        match fs.read_file_string(&gitignore_path).await {
            Ok(content) => content,
            Err(e) => {
                return Err(CliError::io(format!(
                    "Failed to read .gitignore {}: {}",
                    gitignore_path.display(),
                    e
                )));
            }
        }
    } else {
        String::new()
    };

    // Check if we need to add entries
    let needs_backups = !existing_content.contains(".wnt-backups");
    let needs_comment = !existing_content.contains("# Workspace Node Tools");

    if !needs_backups && !needs_comment {
        // Nothing to add
        return Ok(());
    }

    // Build new content
    let mut new_content = existing_content;

    // Ensure trailing newline
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    // Add comment and entries if needed
    if needs_comment || needs_backups {
        if !new_content.is_empty() {
            new_content.push('\n');
        }
        new_content.push_str("# Workspace Node Tools\n");
        new_content.push_str("# Note: .changesets/ and repo.config.* should be versioned in git\n");
        new_content.push_str("# Only backups are excluded\n");
    }

    if needs_backups {
        new_content.push_str(".wnt-backups/\n");
    }

    // Write updated .gitignore
    fs.write_file(&gitignore_path, new_content.as_bytes()).await.map_err(|e| {
        CliError::io(format!("Failed to write .gitignore {}: {}", gitignore_path.display(), e))
    })?;

    Ok(())
}

/// Creates an example changeset file with documentation.
async fn create_example_changeset(root: &Path, changeset_path: &str) -> Result<()> {
    let fs = FileSystemManager::new();

    let example_content = r#"# Example Changeset
#
# This is an example changeset file to help you understand the format.
# When you're ready to create real changesets, use: wnt changeset add
#
# Important: Changesets should be committed to git as part of your PR.
# They tell the CI/CD pipeline what version bumps to perform on merge.
#
# Workflow:
# 1. Create a changeset: wnt changeset add
# 2. Commit it with your changes: git add .changesets/your-changeset.yaml
# 3. Push your branch: git push
# 4. When merged to main, CI runs: wnt bump --execute
# 5. Versions are bumped, changesets move to history/
#
# You can delete this example file when you're ready.

# Changeset ID (typically branch name or custom identifier)
id: example-feature

# Git branch this changeset is associated with
branch: main

# Version bump type: patch, minor, or major
bump: minor

# Packages affected (for monorepos, list package names; for single packages, use the package name)
packages:
  - "@myorg/core"
  - "@myorg/utils"

# Target environments (e.g., dev, staging, production)
environments:
  - production

# Commits included in this changeset (optional, auto-tracked)
commits:
  - abc123: "feat: add new feature"
  - def456: "fix: resolve issue"

# Timestamp when changeset was created
createdAt: "2024-01-01T00:00:00Z"

# Timestamp when changeset was last updated
updatedAt: "2024-01-01T00:00:00Z"
"#;

    let example_path = root.join(changeset_path).join("README-example.yaml");
    fs.write_file(&example_path, example_content.as_bytes()).await.map_err(|e| {
        CliError::io(format!(
            "Failed to create example changeset {}: {}",
            example_path.display(),
            e
        ))
    })?;

    Ok(())
}

/// Outputs the initialization result.
#[allow(clippy::print_stdout)]
fn output_init_result(config_path: &Path, config: &InitConfig, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json | OutputFormat::JsonCompact => {
            #[derive(Serialize)]
            #[allow(non_snake_case)]
            struct InitResult {
                configFile: String,
                configFormat: String,
                strategy: String,
                changesetPath: String,
                environments: Vec<String>,
                defaultEnvironments: Vec<String>,
                registry: String,
            }

            let result = InitResult {
                configFile: config_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("repo.config")
                    .to_string(),
                configFormat: config.config_format.clone(),
                strategy: config.strategy.clone(),
                changesetPath: config.changeset_path.clone(),
                environments: config.environments.clone(),
                defaultEnvironments: config.default_environments.clone(),
                registry: config.registry.clone(),
            };

            let response = JsonResponse::success(result);

            let json = if format == OutputFormat::JsonCompact {
                serde_json::to_string(&response)
            } else {
                serde_json::to_string_pretty(&response)
            }
            .map_err(|e| CliError::execution(format!("Failed to serialize JSON: {e}")))?;

            println!("{json}");
        }
        OutputFormat::Quiet => {
            // Minimal output
            println!("Configuration initialized");
        }
        OutputFormat::Human => {
            println!("\n✓ Configuration initialized successfully\n");
            println!(
                "  Config file: {}",
                config_path.file_name().and_then(|n| n.to_str()).unwrap_or("repo.config")
            );
            println!("  Strategy: {}", config.strategy);
            println!("  Changesets: {}", config.changeset_path);
            println!("  Environments: {}", config.environments.join(", "));
            println!("  Default: {}", config.default_environments.join(", "));
            println!();
        }
    }

    Ok(())
}

/// Parses comma-separated values into a vector of trimmed strings.
fn parse_comma_separated(input: &str) -> Vec<String> {
    input.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}
