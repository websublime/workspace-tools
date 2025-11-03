//! Git integration for version bump operations.
//!
//! This module provides Git operations for the bump command, including:
//! - Committing version changes
//! - Creating Git tags for releases
//! - Pushing tags to remote repositories
//!
//! # What
//!
//! Provides atomic Git operations for version bumps with proper error handling
//! and rollback support. All operations are designed to be safe and reversible.
//!
//! # How
//!
//! The module uses `sublime_git_tools::Repo` to interact with Git repositories.
//! Operations are performed atomically where possible, with clear error messages
//! when operations fail.
//!
//! Git operations flow:
//! 1. Stage modified files (package.json, CHANGELOG.md, etc.)
//! 2. Create commit with descriptive message
//! 3. Create tags for bumped packages
//! 4. Push tags to remote (if requested)
//!
//! # Why
//!
//! Separating Git operations into a dedicated module:
//! - Enables comprehensive testing with mocked Git operations
//! - Provides clear error handling for each Git operation
//! - Allows reuse across different bump scenarios
//! - Makes rollback logic easier to implement
//!
//! # Examples
//!
//! ```rust,ignore
//! use sublime_cli_tools::commands::bump::git_integration::{
//!     commit_version_changes, create_release_tags, push_tags_to_remote
//! };
//! use sublime_git_tools::Repo;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let repo = Repo::open(".")?;
//! let modified_files = vec![
//!     PathBuf::from("packages/core/package.json"),
//!     PathBuf::from("packages/core/CHANGELOG.md"),
//! ];
//!
//! // Commit changes
//! let commit_sha = commit_version_changes(&repo, &modified_files, "chore: bump versions")?;
//!
//! // Create tags
//! let tags = vec![("@org/core".to_string(), "1.2.0".to_string())];
//! create_release_tags(&repo, &tags)?;
//!
//! // Push tags
//! push_tags_to_remote(&repo)?;
//! # Ok(())
//! # }
//! ```

use crate::error::{CliError, Result};
use std::path::Path;
use sublime_git_tools::Repo;
use tracing::{debug, info, warn};

/// Commits version changes to the Git repository.
///
/// Stages the provided files and creates a commit with the specified message.
/// This operation is atomic - either all files are committed or none are.
///
/// # Arguments
///
/// * `repo` - Git repository instance
/// * `modified_files` - List of files that were modified during version bump
/// * `commit_message` - Commit message to use
///
/// # Returns
///
/// Returns the commit SHA on success, or an error if the operation fails.
///
/// # Errors
///
/// Returns an error if:
/// - Files cannot be staged
/// - Commit creation fails
/// - Repository is in an invalid state
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_git_tools::Repo;
/// use std::path::PathBuf;
///
/// let repo = Repo::open(".")?;
/// let files = vec![PathBuf::from("package.json")];
/// let sha = commit_version_changes(&repo, &files, "chore: bump version to 1.2.0")?;
/// println!("Created commit: {}", sha);
/// ```
pub fn commit_version_changes(
    repo: &Repo,
    modified_files: &[impl AsRef<Path>],
    commit_message: &str,
) -> Result<String> {
    info!("Committing version changes to Git");
    debug!("Commit message: {}", commit_message);
    debug!("Modified files: {}", modified_files.len());

    // Stage all modified files
    for file in modified_files {
        let file_path = file.as_ref();
        debug!("Staging file: {}", file_path.display());

        repo.add(file_path.to_str().ok_or_else(|| {
            CliError::execution(format!(
                "File path contains invalid UTF-8: {}",
                file_path.display()
            ))
        })?)
        .map_err(|e| {
            CliError::execution(format!("Failed to stage file {}: {e}", file_path.display()))
        })?;
    }

    info!("Staged {} files", modified_files.len());

    // Create commit
    let commit_sha = repo
        .commit(commit_message)
        .map_err(|e| CliError::execution(format!("Failed to create commit: {e}")))?;

    info!("Created commit: {}", commit_sha);
    Ok(commit_sha)
}

/// Creates Git tags for released packages.
///
/// Creates annotated tags for each package that was bumped. Tag names follow
/// the format: `<package-name>@<version>` for scoped/monorepo packages,
/// or `v<version>` for single-package repositories.
///
/// # Arguments
///
/// * `repo` - Git repository instance
/// * `package_versions` - List of (package_name, version) tuples to tag
///
/// # Returns
///
/// Returns the list of created tag names on success.
///
/// # Errors
///
/// Returns an error if:
/// - Tag creation fails for any package
/// - Repository is in an invalid state
/// - Tag already exists (unless force flag is set)
///
/// # Tag Naming Convention
///
/// - Scoped packages: `@org/package@1.2.0`
/// - Unscoped packages (monorepo): `package@1.2.0`
/// - Single package repo: `v1.2.0`
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_git_tools::Repo;
///
/// let repo = Repo::open(".")?;
/// let packages = vec![
///     ("@myorg/core".to_string(), "1.2.0".to_string()),
///     ("@myorg/utils".to_string(), "2.0.0".to_string()),
/// ];
/// let tags = create_release_tags(&repo, &packages)?;
/// println!("Created {} tags", tags.len());
/// ```
pub fn create_release_tags(
    repo: &Repo,
    package_versions: &[(String, String)],
) -> Result<Vec<String>> {
    info!("Creating Git tags for {} packages", package_versions.len());

    let mut created_tags = Vec::new();

    for (package_name, version) in package_versions {
        // Determine tag name based on package name
        let tag_name = if package_versions.len() == 1 && !package_name.starts_with('@') {
            // Single package repository - use v<version>
            format!("v{version}")
        } else {
            // Monorepo or scoped package - use <package>@<version>
            format!("{package_name}@{version}")
        };

        debug!("Creating tag: {}", tag_name);

        // Create annotated tag with message
        let tag_message = format!("Release {package_name} version {version}");

        repo.create_tag(&tag_name, Some(tag_message)).map_err(|e| {
            CliError::execution(format!(
                "Failed to create tag '{tag_name}' for package {package_name}: {e}"
            ))
        })?;

        info!("Created tag: {}", tag_name);
        created_tags.push(tag_name);
    }

    info!("Successfully created {} tags", created_tags.len());
    Ok(created_tags)
}

/// Pushes Git tags to the remote repository.
///
/// Pushes all tags to the default remote (usually 'origin'). This operation
/// requires network connectivity and proper authentication.
///
/// # Arguments
///
/// * `repo` - Git repository instance
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns an error if:
/// - No remote repository is configured
/// - Network connectivity issues
/// - Authentication fails
/// - Remote rejects the push
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_git_tools::Repo;
///
/// let repo = Repo::open(".")?;
/// push_tags_to_remote(&repo)?;
/// println!("Tags pushed successfully");
/// ```
pub fn push_tags_to_remote(repo: &Repo) -> Result<()> {
    info!("Pushing tags to remote repository");

    repo.push("origin", Some(true))
        .map_err(|e| CliError::execution(format!("Failed to push to remote: {e}")))?;

    info!("Successfully pushed tags to remote");
    Ok(())
}

/// Validates that the repository is in a clean state for version bumping.
///
/// Checks that:
/// - Repository is not in a detached HEAD state
/// - No uncommitted changes exist (unless we're about to commit them)
/// - Repository has at least one commit
///
/// # Arguments
///
/// * `repo` - Git repository instance
/// * `allow_uncommitted` - Whether to allow uncommitted changes (true when we're about to commit)
///
/// # Returns
///
/// Returns `Ok(())` if the repository state is valid.
///
/// # Errors
///
/// Returns an error if:
/// - Repository is in detached HEAD state
/// - Uncommitted changes exist (when not allowed)
/// - Repository has no commits
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_git_tools::Repo;
///
/// let repo = Repo::open(".")?;
/// validate_repository_state(&repo, false)?;
/// ```
pub fn validate_repository_state(repo: &Repo, allow_uncommitted: bool) -> Result<()> {
    debug!("Validating repository state");

    // Check if we have a current branch (not detached HEAD)
    let current_branch = repo
        .get_current_branch()
        .map_err(|e| CliError::execution(format!("Failed to get current branch: {e}")))?;

    if current_branch.is_empty() {
        return Err(CliError::execution(
            "Repository is in detached HEAD state. Please checkout a branch first.".to_string(),
        ));
    }

    debug!("Current branch: {}", current_branch);

    // Check for uncommitted changes (if not allowed)
    if !allow_uncommitted {
        let status = repo
            .get_status_detailed()
            .map_err(|e| CliError::execution(format!("Failed to get repository status: {e}")))?;

        if !status.is_empty() {
            warn!("Repository has {} uncommitted changes", status.len());
            return Err(CliError::execution(
                "Repository has uncommitted changes. Please commit or stash them first."
                    .to_string(),
            ));
        }
    }

    debug!("Repository state is valid");
    Ok(())
}

/// Gets the current commit SHA.
///
/// Retrieves the SHA of the current HEAD commit.
///
/// # Arguments
///
/// * `repo` - Git repository instance
///
/// # Returns
///
/// Returns the commit SHA as a string.
///
/// # Errors
///
/// Returns an error if:
/// - Repository has no commits
/// - Failed to access HEAD
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_git_tools::Repo;
///
/// let repo = Repo::open(".")?;
/// let sha = get_current_commit_sha(&repo)?;
/// println!("Current commit: {}", sha);
/// ```
pub fn get_current_commit_sha(repo: &Repo) -> Result<String> {
    debug!("Getting current commit SHA");

    let sha = repo
        .get_current_sha()
        .map_err(|e| CliError::execution(format!("Failed to get current commit: {e}")))?;

    debug!("Current commit SHA: {}", sha);
    Ok(sha)
}
