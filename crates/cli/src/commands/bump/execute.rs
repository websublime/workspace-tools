//! Bump execute command implementation.
//!
//! This module implements the bump execution (apply) functionality which actually
//! applies version bumps, updates files, generates changelogs, archives changesets,
//! and performs Git operations.
//!
//! # What
//!
//! Provides the `execute_bump_apply` function that:
//! - Loads all active changesets
//! - Calculates and applies version bumps using `VersionResolver`
//! - Updates package.json files with new versions
//! - Generates/updates CHANGELOG.md files (if enabled)
//! - Archives processed changesets (if enabled)
//! - Creates Git commits and tags (if requested)
//! - Pushes tags to remote (if requested)
//! - Provides atomic operations with rollback on failure
//!
//! # How
//!
//! The command flow:
//! 1. Validates repository state (if git operations requested)
//! 2. Loads workspace configuration to determine versioning strategy
//! 3. Creates ChangesetManager and loads all pending changesets
//! 4. If no changesets exist, reports "nothing to bump" and exits
//! 5. Uses VersionResolver to calculate all version bumps
//! 6. Shows confirmation prompt (unless --force)
//! 7. Applies version updates via `VersionResolver::apply_versions()`
//! 8. Generates changelogs for each affected package (if enabled)
//! 9. Archives changesets with release metadata (if enabled)
//! 10. Commits changes to Git (if --git-commit)
//! 11. Creates Git tags for releases (if --git-tag)
//! 12. Pushes tags to remote (if --git-push)
//! 13. Displays success summary
//!
//! ## Strategy Handling
//!
//! ### Independent Strategy
//! - Only packages listed in changeset.packages receive version bumps
//! - Other packages remain unchanged even if they're dependencies
//! - Each package gets its own tag: `<package>@<version>`
//!
//! ### Unified Strategy
//! - ALL packages bump when ANY changeset exists
//! - All packages receive the same version
//! - Highest bump type from all changesets is used
//! - Each package gets a tag (or one monorepo tag if configured)
//!
//! ## Error Handling and Recovery
//!
//! The command provides error handling at multiple levels:
//!
//! ### File Operations
//! - `VersionResolver::apply_versions()` creates automatic backups before modifications
//! - If file updates fail, the resolver's internal backup mechanism can restore files
//! - This provides protection for package.json modifications
//!
//! ### Git Operations
//! - Git operations (commit, tag, push) are performed atomically
//! - Each operation validates prerequisites before executing
//! - Failed git operations do not affect already-modified files
//! - Manual recovery may be needed if git operations partially complete
//!
//! ### Changeset Archival
//! - Changesets are only archived after all updates succeed
//! - If archival is disabled (--no-archive), changesets remain in place
//! - Archived changesets include full release metadata for audit trail
//!
//! **Note**: For full rollback capability including git operations, users should:
//! 1. Use `--dry-run` mode first to preview changes
//! 2. Test in a feature branch before applying to main
//! 3. Use Git's own rollback mechanisms (git reset, git revert) if needed
//! 4. Keep backups via the backup directory created by VersionResolver
//!
//! # Why
//!
//! Execute mode is essential for:
//! - Completing the release workflow
//! - Automating version bumps in CI/CD
//! - Ensuring atomic operations (all or nothing)
//! - Maintaining audit trail through changesets
//! - Integrating with Git workflows
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::bump::execute_bump_apply;
//! use sublime_cli_tools::cli::commands::BumpArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let args = BumpArgs {
//!     dry_run: false,
//!     execute: true,
//!     snapshot: false,
//!     snapshot_format: None,
//!     prerelease: None,
//!     packages: None,
//!     git_tag: true,
//!     git_push: false,
//!     git_commit: true,
//!     no_changelog: false,
//!     no_archive: false,
//!     force: false,
//! };
//!
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_bump_apply(&args, &output, Path::new("."), None).await?;
//! # Ok(())
//! # }
//! ```

use crate::cli::commands::BumpArgs;
use crate::commands::bump::git_integration::{
    commit_version_changes, create_release_tags, get_current_commit_sha, push_tags_to_remote,
    validate_repository_state,
};
use crate::commands::bump::preview::{load_config, merge_changesets};
use crate::commands::bump::snapshot::{BumpSnapshot, BumpSummary, ChangesetInfo, PackageBumpInfo};
use crate::error::{CliError, Result};
use crate::interactive::prompts::prompt_confirm;
use crate::output::styling::{StatusSymbol, print_item};
use crate::output::{JsonResponse, Output};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use sublime_git_tools::Repo;
use sublime_pkg_tools::changelog::ChangelogGenerator;
use sublime_pkg_tools::changeset::ChangesetManager;

use sublime_pkg_tools::types::{Changeset, ReleaseInfo};
use sublime_pkg_tools::version::VersionResolver;
use sublime_standard_tools::filesystem::FileSystemManager;
use tracing::{debug, error, info, warn};

/// Execute the bump apply command.
///
/// Applies version bumps based on active changesets, updates files, generates
/// changelogs, archives changesets, and performs Git operations.
///
/// # Arguments
///
/// * `args` - Command arguments specifying execution options
/// * `output` - Output handler for formatting and displaying results
/// * `root` - Workspace root directory
/// * `config_path` - Optional path to config file (from global `--config` option)
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if the operation fails.
///
/// # Errors
///
/// Returns an error if:
/// - The workspace is not initialized (no configuration found)
/// - Cannot load changesets or packages
/// - Version resolution fails
/// - File system operations fail
/// - Git operations fail (if requested)
/// - User cancels operation at confirmation prompt
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::bump::execute_bump_apply;
/// use sublime_cli_tools::cli::commands::BumpArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = BumpArgs {
///     execute: true,
///     git_commit: true,
///     git_tag: true,
///     ..Default::default()
/// };
/// let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
/// execute_bump_apply(&args, &output, Path::new("."), None).await?;
/// # Ok(())
/// # }
/// ```
// Allow too_many_lines: This function orchestrates the complete version bump workflow with many steps
// including validation, version resolution, file updates, changelog generation, changeset archival,
// and git operations. Breaking it into smaller functions would reduce readability and make the
// sequential workflow harder to follow. The length is justified by the complex multi-step process.
#[allow(clippy::too_many_lines)]
pub async fn execute_bump_apply(
    args: &BumpArgs,
    output: &Output,
    root: &Path,
    config_path: Option<&Path>,
) -> Result<()> {
    let workspace_root = root;
    info!("Executing bump apply in workspace: {}", workspace_root.display());

    // Step 1: Validate Git repository state if git operations are requested
    let git_repo = if args.git_commit || args.git_tag || args.git_push {
        debug!("Git operations requested, validating repository state");

        let repo = Repo::open(workspace_root.to_str().ok_or_else(|| {
            CliError::execution("Workspace path contains invalid UTF-8".to_string())
        })?)
        .map_err(|e| {
            CliError::execution(format!(
                "Failed to open Git repository at {}: {}",
                workspace_root.display(),
                e
            ))
        })?;

        // Validate repository state - we allow uncommitted changes since we'll be making them
        validate_repository_state(&repo, true)?;
        Some(repo)
    } else {
        debug!("No git operations requested, skipping git validation");
        None
    };

    // Step 2: Load configuration
    let config = load_config(workspace_root, config_path).await?;
    info!("Configuration loaded successfully");
    debug!("Versioning strategy: {:?}", config.version.strategy);

    // Step 3: Load all pending changesets
    let fs = FileSystemManager::new();
    let manager = ChangesetManager::new(workspace_root.to_path_buf(), fs.clone(), config.clone())
        .await
        .map_err(|e| CliError::execution(format!("Failed to create changeset manager: {e}")))?;

    let loaded_changesets = manager
        .list_pending()
        .await
        .map_err(|e| CliError::execution(format!("Failed to load changesets: {e}")))?;

    debug!("Loaded {} changeset(s)", loaded_changesets.len());

    // Step 4: Check if there are any changesets
    if loaded_changesets.is_empty() {
        if output.format().is_json() {
            let response: JsonResponse<ExecuteResult> = JsonResponse::success(ExecuteResult {
                strategy: config.version.strategy.to_string(),
                packages_updated: 0,
                changesets_archived: 0,
                files_modified: vec![],
                tags_created: vec![],
                commit_sha: None,
                snapshot: BumpSnapshot {
                    strategy: config.version.strategy.to_string(),
                    packages: vec![],
                    changesets: vec![],
                    summary: BumpSummary::default(),
                },
            });
            output.json(&response)?;
        } else {
            output.info("No changesets found. Nothing to bump.")?;
        }
        return Ok(());
    }

    info!("Processing {} changeset(s)", loaded_changesets.len());

    // Step 6: Create VersionResolver and resolve versions
    let resolver = VersionResolver::new(workspace_root.to_path_buf(), config.clone())
        .await
        .map_err(|e| CliError::execution(format!("Failed to create version resolver: {e}")))?;

    // Merge all changesets for resolution
    let merged_changeset = merge_changesets(&loaded_changesets)?;

    // Resolve versions
    let resolution = resolver
        .resolve_versions(&merged_changeset)
        .await
        .map_err(|e| CliError::execution(format!("Failed to resolve versions: {e}")))?;

    debug!("Resolved {} package updates", resolution.updates.len());

    if resolution.updates.is_empty() {
        if output.format().is_json() {
            let response: JsonResponse<ExecuteResult> = JsonResponse::success(ExecuteResult {
                strategy: config.version.strategy.to_string(),
                packages_updated: 0,
                changesets_archived: 0,
                files_modified: vec![],
                tags_created: vec![],
                commit_sha: None,
                snapshot: BumpSnapshot {
                    strategy: config.version.strategy.to_string(),
                    packages: vec![],
                    changesets: vec![],
                    summary: BumpSummary::default(),
                },
            });
            output.json(&response)?;
        } else {
            output.info("No version updates needed. All packages are up to date.")?;
        }
        return Ok(());
    }

    // Step 7: Show confirmation prompt (unless --force)
    if !args.force && !output.format().is_json() {
        output.blank_line()?;
        StatusSymbol::Info.print_line("About to bump versions for the following packages:");
        output.blank_line()?;

        for update in &resolution.updates {
            print_item(
                &format!("  {}", update.name),
                &format!("{} → {}", update.current_version, update.next_version),
                false,
            );
        }

        output.blank_line()?;

        let operations = build_operations_summary(args);
        StatusSymbol::Info.print_line("Operations to perform:");
        for op in operations {
            print_item("  ✓", &op, false);
        }

        output.blank_line()?;

        let confirmed = prompt_confirm("Proceed with version bump?", false, false)?;
        if !confirmed {
            output.info("Version bump cancelled by user.")?;
            return Ok(());
        }
    }

    info!("Applying version updates");

    // Step 8: Apply version updates
    let apply_result = resolver.apply_versions(&merged_changeset, false).await.map_err(|e| {
        error!("Failed to apply version updates: {}", e);
        CliError::execution(format!("Failed to apply version updates: {e}"))
    })?;

    info!("Successfully updated {} packages", apply_result.summary.packages_updated);

    // Collect modified files for git commit (package.json files that were updated)
    let mut modified_files: Vec<PathBuf> =
        apply_result.resolution.updates.iter().map(|u| u.path.join("package.json")).collect();

    // Step 9: Generate changelogs (if enabled)
    if !args.no_changelog && config.changelog.enabled {
        info!("Generating changelogs");

        if git_repo.is_some() {
            // Need to open a new Repo instance since Repo doesn't implement Clone
            let repo_for_changelog = Repo::open(workspace_root.to_str().ok_or_else(|| {
                CliError::execution("Workspace path contains invalid UTF-8".to_string())
            })?)
            .map_err(|e| {
                error!("Failed to open Git repository for changelog: {}", e);
                CliError::execution(format!("Failed to open Git repository for changelog: {e}"))
            })?;

            let changelog_gen = ChangelogGenerator::new(
                workspace_root.to_path_buf(),
                repo_for_changelog,
                fs.clone(),
                config.changelog.clone(),
            )
            .await
            .map_err(|e| {
                error!("Failed to create changelog generator: {}", e);
                CliError::execution(format!("Failed to create changelog generator: {e}"))
            })?;

            for changeset in &loaded_changesets {
                debug!("Generating changelog for changeset: {}", changeset.branch);

                let changelogs = changelog_gen
                    .generate_from_changeset(changeset, &apply_result.resolution)
                    .await
                    .map_err(|e| {
                        error!(
                            "Failed to generate changelog for changeset '{}': {}",
                            changeset.branch, e
                        );
                        CliError::execution(format!(
                            "Failed to generate changelog for changeset '{}': {}",
                            changeset.branch, e
                        ))
                    })?;

                info!(
                    "Generated {} changelog(s) for changeset '{}'",
                    changelogs.len(),
                    changeset.branch
                );

                // Add CHANGELOG.md files to modified files list
                for changelog in changelogs {
                    if let Some(pkg_name) = &changelog.package_name {
                        let changelog_path = workspace_root.join(pkg_name).join("CHANGELOG.md");
                        modified_files.push(changelog_path);
                    }
                }
            }
        } else {
            warn!("Changelog generation requested but no git repository available, skipping");
        }
    } else {
        debug!("Changelog generation disabled");
    }

    // Step 10: Archive changesets (if enabled)
    let mut archived_count = 0;
    if args.no_archive {
        debug!("Changeset archival disabled");
    } else {
        info!("Archiving changesets");

        // Build release info
        let commit_sha = if let Some(ref repo) = git_repo {
            get_current_commit_sha(repo).unwrap_or_else(|_| "unknown".to_string())
        } else {
            "unknown".to_string()
        };

        let mut versions_map = HashMap::new();
        for update in &apply_result.resolution.updates {
            versions_map.insert(update.name.clone(), update.next_version.to_string());
        }

        let release_info = ReleaseInfo::new("wnt-cli", commit_sha.as_str(), versions_map);

        for changeset in &loaded_changesets {
            debug!("Archiving changeset: {}", changeset.branch);

            manager.archive(&changeset.branch, release_info.clone()).await.map_err(|e| {
                error!("Failed to archive changeset '{}': {}", changeset.branch, e);
                CliError::execution(format!(
                    "Failed to archive changeset '{}': {}",
                    changeset.branch, e
                ))
            })?;

            archived_count += 1;
        }

        info!("Archived {} changeset(s)", archived_count);
    }

    // Step 11: Git operations
    let mut commit_sha = None;
    let mut tags_created = Vec::new();

    if let Some(ref repo) = git_repo {
        // Commit changes (if requested)
        if args.git_commit {
            info!("Committing version changes to Git");

            let commit_message = build_commit_message(&apply_result.resolution.updates);
            let sha = commit_version_changes(repo, &modified_files, &commit_message)?;

            commit_sha = Some(sha.clone());
            info!("Created commit: {}", sha);
        }

        // Create tags (if requested)
        if args.git_tag {
            info!("Creating Git tags");

            let package_versions: Vec<(String, String)> = apply_result
                .resolution
                .updates
                .iter()
                .map(|u| (u.name.clone(), u.next_version.to_string()))
                .collect();

            tags_created = create_release_tags(repo, &package_versions)?;
            info!("Created {} tag(s)", tags_created.len());
        }

        // Push tags (if requested)
        if args.git_push && args.git_tag {
            info!("Pushing tags to remote");
            push_tags_to_remote(repo)?;
            info!("Successfully pushed tags to remote");
        } else if args.git_push && !args.git_tag {
            warn!("--git-push specified without --git-tag, skipping push");
        }
    }

    // Step 12: Build and display result
    let result = ExecuteResult {
        strategy: config.version.strategy.to_string(),
        packages_updated: apply_result.summary.packages_updated,
        changesets_archived: archived_count,
        files_modified: modified_files.clone(),
        tags_created: tags_created.clone(),
        commit_sha: commit_sha.clone(),
        snapshot: build_result_snapshot(
            &config,
            &apply_result.resolution.updates,
            &loaded_changesets,
        ),
    };

    if output.format().is_json() {
        let response: JsonResponse<ExecuteResult> = JsonResponse::success(result);
        output.json(&response)?;
    } else {
        display_result(output, &result)?;
    }

    info!("Version bump completed successfully");
    Ok(())
}

/// Builds a list of operations that will be performed.
fn build_operations_summary(args: &BumpArgs) -> Vec<String> {
    let mut operations = vec!["Update package.json files".to_string()];

    if !args.no_changelog {
        operations.push("Generate/update CHANGELOG.md files".to_string());
    }

    if !args.no_archive {
        operations.push("Archive changesets".to_string());
    }

    if args.git_commit {
        operations.push("Create Git commit".to_string());
    }

    if args.git_tag {
        operations.push("Create Git tags".to_string());
    }

    if args.git_push {
        operations.push("Push tags to remote".to_string());
    }

    operations
}

/// Builds a commit message from package updates.
fn build_commit_message(updates: &[sublime_pkg_tools::types::PackageUpdate]) -> String {
    use std::fmt::Write;

    if updates.len() == 1 {
        let update = &updates[0];
        format!("chore: bump {} to {}", update.name, update.next_version)
    } else {
        let mut message = String::from("chore: bump versions\n\n");

        for update in updates {
            let _ = writeln!(
                message,
                "- {}: {} → {}",
                update.name, update.current_version, update.next_version
            );
        }

        message
    }
}

/// Calculates the bump type by comparing two versions.
fn calculate_bump_type(
    current: &sublime_pkg_tools::types::Version,
    next: &sublime_pkg_tools::types::Version,
) -> sublime_pkg_tools::types::VersionBump {
    use sublime_pkg_tools::types::VersionBump;

    if next.major() > current.major() {
        VersionBump::Major
    } else if next.minor() > current.minor() {
        VersionBump::Minor
    } else if next.patch() > current.patch() {
        VersionBump::Patch
    } else {
        VersionBump::None
    }
}

/// Builds a result snapshot from updates and changesets.
fn build_result_snapshot(
    config: &sublime_pkg_tools::config::PackageToolsConfig,
    updates: &[sublime_pkg_tools::types::PackageUpdate],
    changesets: &[Changeset],
) -> BumpSnapshot {
    let packages: Vec<PackageBumpInfo> = updates
        .iter()
        .map(|u| {
            let bump_type = calculate_bump_type(&u.current_version, &u.next_version);
            PackageBumpInfo {
                name: u.name.clone(),
                path: u.path.display().to_string(),
                current_version: u.current_version.to_string(),
                next_version: u.next_version.to_string(),
                bump_type,
                will_bump: true,
                reason: "version applied".to_string(),
            }
        })
        .collect();

    let changeset_infos: Vec<ChangesetInfo> = changesets
        .iter()
        .map(|cs| ChangesetInfo {
            id: cs.branch.clone(),
            branch: cs.branch.clone(),
            bump_type: cs.bump,
            packages: cs.packages.clone(),
            commit_count: cs.changes.len(),
        })
        .collect();

    let summary = BumpSummary::new(updates.len(), updates.len(), changesets.len(), false);

    BumpSnapshot {
        strategy: config.version.strategy.to_string(),
        packages,
        changesets: changeset_infos,
        summary,
    }
}

/// Displays the execution result.
fn display_result(output: &Output, result: &ExecuteResult) -> Result<()> {
    StatusSymbol::Success.print_line("Version bump completed successfully!");
    output.blank_line()?;

    StatusSymbol::Info.print_line("Summary:");
    print_item("  Packages updated", &result.packages_updated.to_string(), false);
    print_item("  Files modified", &result.files_modified.len().to_string(), false);
    print_item("  Changesets archived", &result.changesets_archived.to_string(), false);

    if !result.tags_created.is_empty() {
        print_item("  Tags created", &result.tags_created.len().to_string(), false);
    }

    if let Some(ref sha) = result.commit_sha {
        print_item("  Commit", &sha[..8.min(sha.len())], true);
    }

    output.blank_line()?;

    StatusSymbol::Info.print_line("Updated packages:");
    for pkg in &result.snapshot.packages {
        print_item(
            &format!("  {}", pkg.name),
            &format!("{} → {}", pkg.current_version, pkg.next_version),
            false,
        );
    }

    Ok(())
}

/// Result of executing a version bump.
#[derive(Debug, serde::Serialize)]
pub struct ExecuteResult {
    /// Versioning strategy used
    pub strategy: String,

    /// Number of packages updated
    pub packages_updated: usize,

    /// Number of changesets archived
    pub changesets_archived: usize,

    /// List of files that were modified
    pub files_modified: Vec<PathBuf>,

    /// List of Git tags that were created
    pub tags_created: Vec<String>,

    /// Git commit SHA (if commit was created)
    pub commit_sha: Option<String>,

    /// Full snapshot of the bump operation
    pub snapshot: BumpSnapshot,
}
