//! Workspace status command implementation.
//!
//! This module implements the `workspace status` command which displays comprehensive
//! information about the current workspace including repository type, package manager,
//! active branch, pending changesets, and all workspace packages.
//!
//! # What
//!
//! Provides the `execute_status` function that:
//! - Detects repository type (simple or monorepo with specific type)
//! - Detects the package manager in use (npm, yarn, pnpm, bun)
//! - Retrieves the current Git branch (graceful degradation if not a Git repo)
//! - Lists pending changesets (if changeset directory exists)
//! - Lists all workspace packages with their names, versions, and paths
//! - Displays results in table or JSON format
//!
//! # How
//!
//! The command flow:
//! 1. Detects if the workspace is a monorepo or simple project
//! 2. Detects the package manager from lock files
//! 3. Opens Git repository and retrieves current branch (optional)
//! 4. Loads pending changesets from the changeset manager
//! 5. Lists all workspace packages with metadata
//! 6. Formats output as styled sections/tables (human) or JSON (automation)
//!
//! # Why
//!
//! Workspace status provides essential information for:
//! - Understanding workspace structure at a glance
//! - Verifying package manager and repository configuration
//! - Checking active changesets before releases
//! - Scripting and CI/CD integration via JSON output
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::status::execute_status;
//! use sublime_cli_tools::cli::commands::StatusArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let args = StatusArgs {};
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_status(&args, &output, Path::new("."), None).await?;
//! # Ok(())
//! # }
//! ```

use crate::cli::commands::StatusArgs;
use crate::error::{CliError, Result};
use crate::output::table::{TableBuilder, TableTheme};
use crate::output::{JsonResponse, Output};
use serde::Serialize;
use std::path::Path;
use sublime_git_tools::Repo;
use sublime_pkg_tools::changeset::ChangesetManager;
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait, MonorepoKind};
use sublime_standard_tools::node::PackageManager;
use tracing::{debug, info, warn};

use super::find_and_load_config;

// ============================================================================
// JSON Response Types
// ============================================================================

/// JSON response for workspace status command.
///
/// Contains all workspace information in a structured format suitable for
/// automation and scripting.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusJsonResponse {
    /// Repository type information.
    repository: RepositoryInfoJson,
    /// Package manager information.
    package_manager: PackageManagerInfoJson,
    /// Current Git branch (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<BranchInfoJson>,
    /// List of pending changesets.
    changesets: Vec<ChangesetInfoJson>,
    /// List of workspace packages.
    packages: Vec<PackageInfoJson>,
}

/// Repository type information for JSON output.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepositoryInfoJson {
    /// Repository kind: "simple" or "monorepo".
    kind: String,
    /// Monorepo type if applicable (npm, yarn, pnpm, bun, deno, custom).
    #[serde(skip_serializing_if = "Option::is_none")]
    monorepo_type: Option<String>,
}

/// Package manager information for JSON output.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PackageManagerInfoJson {
    /// Package manager name (npm, yarn, pnpm, bun, jsr).
    name: String,
    /// Lock file name.
    lock_file: String,
}

/// Git branch information for JSON output.
#[derive(Debug, Clone, Serialize)]
struct BranchInfoJson {
    /// Branch name.
    name: String,
}

/// Changeset information for JSON output.
#[derive(Debug, Clone, Serialize)]
struct ChangesetInfoJson {
    /// Changeset ID (branch name).
    id: String,
}

/// Package information for JSON output.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PackageInfoJson {
    /// Package name (may include scope).
    name: String,
    /// Package version.
    version: String,
    /// Package path relative to workspace root.
    path: String,
}

// ============================================================================
// Command Implementation
// ============================================================================

/// Execute the workspace status command.
///
/// Displays comprehensive workspace information including repository type,
/// package manager, Git branch, pending changesets, and all workspace packages.
///
/// # Arguments
///
/// * `_args` - Command arguments (currently unused, uses global options)
/// * `output` - Output handler for formatting results
/// * `root` - Workspace root directory path
/// * `config_path` - Optional custom config file path
///
/// # Returns
///
/// Returns `Ok(())` if status display succeeds, or an error if:
/// - The path is not a valid Node.js project
/// - Package detection fails critically
///
/// # Errors
///
/// This function will return an error if:
/// - No package.json is found at the root
/// - Critical filesystem errors occur
///
/// Non-fatal errors (Git not available, no changesets) are handled gracefully
/// with appropriate fallback behavior.
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::commands::status::execute_status;
/// use sublime_cli_tools::cli::commands::StatusArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = StatusArgs {};
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// execute_status(&args, &output, Path::new("."), None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_status(
    _args: &StatusArgs,
    output: &Output,
    root: &Path,
    config_path: Option<&Path>,
) -> Result<()> {
    info!("Executing workspace status command");
    debug!("Workspace root: {}", root.display());

    let fs = FileSystemManager::new();

    // Check if root has package.json (required for Node.js project)
    let package_json_path = root.join("package.json");
    if !fs.exists(&package_json_path).await {
        return Err(CliError::validation("Not a valid Node.js project: package.json not found"));
    }

    // 1. Detect repository type (monorepo or simple)
    let detector = MonorepoDetector::new();
    let (repo_kind, monorepo_type, packages) = detect_repository_info(&detector, root).await;

    // 2. Detect package manager
    let (pm_name, pm_lock_file) = detect_package_manager_info(root);

    // 3. Get Git branch (graceful degradation)
    let branch_name = get_git_branch(root);

    // 4. Get pending changesets
    let changesets = get_pending_changesets(root, config_path, &fs).await;

    // Format output based on mode
    if output.format().is_json() {
        output_json(
            output,
            &repo_kind,
            monorepo_type.as_deref(),
            &pm_name,
            &pm_lock_file,
            branch_name.as_deref(),
            &changesets,
            &packages,
        )?;
    } else {
        output_human(
            output,
            &repo_kind,
            monorepo_type.as_deref(),
            &pm_name,
            &pm_lock_file,
            branch_name.as_deref(),
            &changesets,
            &packages,
        )?;
    }

    Ok(())
}

/// Detects repository type and returns packages if monorepo.
async fn detect_repository_info(
    detector: &MonorepoDetector,
    root: &Path,
) -> (String, Option<String>, Vec<PackageInfoJson>) {
    match detector.is_monorepo_root(root).await {
        Ok(Some(kind)) => {
            let monorepo_type = monorepo_kind_to_string(&kind);
            debug!("Detected monorepo: {}", monorepo_type);

            // Get packages for monorepo
            let packages = match detector.detect_packages(root).await {
                Ok(pkgs) => pkgs
                    .into_iter()
                    .map(|p| PackageInfoJson {
                        name: p.name.clone(),
                        version: p.version.clone(),
                        path: p.location.to_string_lossy().to_string(),
                    })
                    .collect(),
                Err(e) => {
                    warn!("Failed to detect packages: {}", e);
                    Vec::new()
                }
            };

            ("monorepo".to_string(), Some(monorepo_type), packages)
        }
        Ok(None) => {
            debug!("Detected simple repository");
            // For simple repo, try to get root package info
            let packages = get_root_package_info(root).await;
            ("simple".to_string(), None, packages)
        }
        Err(e) => {
            warn!("Failed to detect repository type: {}", e);
            ("unknown".to_string(), None, Vec::new())
        }
    }
}

/// Converts MonorepoKind to string representation.
fn monorepo_kind_to_string(kind: &MonorepoKind) -> String {
    match kind {
        MonorepoKind::NpmWorkSpace => "npm".to_string(),
        MonorepoKind::YarnWorkspaces => "yarn".to_string(),
        MonorepoKind::PnpmWorkspaces => "pnpm".to_string(),
        MonorepoKind::BunWorkspaces => "bun".to_string(),
        MonorepoKind::DenoWorkspaces => "deno".to_string(),
        MonorepoKind::Custom { name, .. } => name.clone(),
    }
}

/// Gets package info from root package.json for simple repositories.
async fn get_root_package_info(root: &Path) -> Vec<PackageInfoJson> {
    let fs = FileSystemManager::new();
    let package_json_path = root.join("package.json");

    match fs.read_file_string(&package_json_path).await {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(json) => {
                let name =
                    json.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed").to_string();
                let version =
                    json.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();

                vec![PackageInfoJson { name, version, path: ".".to_string() }]
            }
            Err(e) => {
                warn!("Failed to parse package.json: {}", e);
                Vec::new()
            }
        },
        Err(e) => {
            warn!("Failed to read package.json: {}", e);
            Vec::new()
        }
    }
}

/// Detects package manager and returns name and lock file.
fn detect_package_manager_info(root: &Path) -> (String, String) {
    match PackageManager::detect(root) {
        Ok(pm) => {
            let name = pm.kind().command().to_string();
            let lock_file = pm.kind().lock_file().to_string();
            debug!("Detected package manager: {} ({})", name, lock_file);
            (name, lock_file)
        }
        Err(e) => {
            warn!("Failed to detect package manager: {}", e);
            ("unknown".to_string(), "unknown".to_string())
        }
    }
}

/// Gets current Git branch name.
fn get_git_branch(root: &Path) -> Option<String> {
    match Repo::open(root.to_string_lossy().as_ref()) {
        Ok(repo) => match repo.get_current_branch() {
            Ok(branch) => {
                debug!("Current Git branch: {}", branch);
                Some(branch)
            }
            Err(e) => {
                debug!("Failed to get current branch: {}", e);
                None
            }
        },
        Err(e) => {
            debug!("Not a Git repository or failed to open: {}", e);
            None
        }
    }
}

/// Gets pending changesets from the changeset manager.
async fn get_pending_changesets(
    root: &Path,
    config_path: Option<&Path>,
    fs: &FileSystemManager,
) -> Vec<ChangesetInfoJson> {
    // Try to load config, use default if not found
    let config = match find_and_load_config(root, config_path).await {
        Ok(Some(config)) => config,
        Ok(None) => PackageToolsConfig::default(),
        Err(e) => {
            debug!("Failed to load config for changesets: {}", e);
            PackageToolsConfig::default()
        }
    };

    // Create changeset manager and list pending
    match ChangesetManager::new(root, fs.clone(), config).await {
        Ok(manager) => match manager.list_pending().await {
            Ok(changesets) => {
                debug!("Found {} pending changesets", changesets.len());
                changesets.into_iter().map(|c| ChangesetInfoJson { id: c.branch }).collect()
            }
            Err(e) => {
                debug!("Failed to list changesets: {}", e);
                Vec::new()
            }
        },
        Err(e) => {
            debug!("Failed to create changeset manager: {}", e);
            Vec::new()
        }
    }
}

// ============================================================================
// Output Formatting
// ============================================================================

/// Outputs status in JSON format.
#[allow(clippy::too_many_arguments)]
fn output_json(
    output: &Output,
    repo_kind: &str,
    monorepo_type: Option<&str>,
    pm_name: &str,
    pm_lock_file: &str,
    branch_name: Option<&str>,
    changesets: &[ChangesetInfoJson],
    packages: &[PackageInfoJson],
) -> Result<()> {
    let response = StatusJsonResponse {
        repository: RepositoryInfoJson {
            kind: repo_kind.to_string(),
            monorepo_type: monorepo_type.map(String::from),
        },
        package_manager: PackageManagerInfoJson {
            name: pm_name.to_string(),
            lock_file: pm_lock_file.to_string(),
        },
        branch: branch_name.map(|name| BranchInfoJson { name: name.to_string() }),
        changesets: changesets.to_vec(),
        packages: packages.to_vec(),
    };

    output.json(&JsonResponse::success(response))?;
    Ok(())
}

/// Outputs status in human-readable format.
#[allow(clippy::too_many_arguments)]
fn output_human(
    output: &Output,
    repo_kind: &str,
    monorepo_type: Option<&str>,
    pm_name: &str,
    pm_lock_file: &str,
    branch_name: Option<&str>,
    changesets: &[ChangesetInfoJson],
    packages: &[PackageInfoJson],
) -> Result<()> {
    output.blank_line()?;
    output.info("Workspace Status")?;
    output.plain("================")?;
    output.blank_line()?;

    // Repository section
    output.info("Repository")?;
    if let Some(mono_type) = monorepo_type {
        output.plain(&format!("  Type: {repo_kind} ({mono_type})"))?;
    } else {
        output.plain(&format!("  Type: {repo_kind}"))?;
    }
    output.blank_line()?;

    // Package Manager section
    output.info("Package Manager")?;
    output.plain(&format!("  Name: {pm_name}"))?;
    output.plain(&format!("  Lock file: {pm_lock_file}"))?;
    output.blank_line()?;

    // Git Branch section
    output.info("Git Branch")?;
    if let Some(branch) = branch_name {
        output.plain(&format!("  Current: {branch}"))?;
    } else {
        output.plain("  Current: (not a Git repository)")?;
    }
    output.blank_line()?;

    // Changesets section
    let changeset_count = changesets.len();
    output.info(&format!("Active Changesets ({changeset_count})"))?;
    if changesets.is_empty() {
        output.plain("  (none)")?;
    } else {
        for changeset in changesets {
            let id = &changeset.id;
            output.plain(&format!("  - {id}"))?;
        }
    }
    output.blank_line()?;

    // Packages section
    let package_count = packages.len();
    output.info(&format!("Packages ({package_count})"))?;
    if packages.is_empty() {
        output.plain("  (none)")?;
    } else {
        let mut table = TableBuilder::new()
            .theme(TableTheme::Default)
            .columns(&["Name", "Version", "Path"])
            .build();

        for pkg in packages {
            table.add_row(&[&pkg.name, &pkg.version, &pkg.path]);
        }

        output.table(&mut table)?;
    }

    output.blank_line()?;
    Ok(())
}
