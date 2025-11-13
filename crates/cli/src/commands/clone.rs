//! Clone command implementation.
//!
//! This module implements the repository cloning functionality with automatic
//! workspace setup. It provides a seamless developer onboarding experience by
//! cloning a Git repository and automatically initializing or validating
//! workspace configuration.
//!
//! # What
//!
//! Provides the `clone` command that:
//! - Clones Git repositories (HTTPS and SSH)
//! - Shows progress during clone operations
//! - Supports shallow clones with --depth flag
//! - Supports --force to overwrite existing directories
//! - Automatically detects workspace configuration (Story 11.3)
//! - Validates existing configuration (Story 11.3)
//! - Initializes workspace if no configuration exists (Story 11.4)
//!
//! # How
//!
//! The command follows this flow:
//! 1. Parse URL and determine destination directory
//! 2. Validate destination doesn't exist (unless --force)
//! 3. Remove destination if --force is set
//! 4. Clone repository with progress indication
//! 5. Detect workspace configuration (Story 11.3)
//! 6. Validate configuration or run init (Story 11.3/11.4)
//! 7. Display success message with next steps (Story 11.4)
//!
//! # Why
//!
//! Cloning with automatic setup:
//! - Reduces onboarding friction for new developers
//! - Ensures consistent workspace setup
//! - Validates configuration immediately
//! - Provides clear feedback and next steps
//!
//! # Examples
//!
//! ```bash
//! # Clone to default location (repository name)
//! workspace clone https://github.com/org/repo.git
//!
//! # Clone to specific directory
//! workspace clone https://github.com/org/repo.git ./my-dir
//!
//! # Clone with configuration overrides
//! workspace clone https://github.com/org/repo.git \
//!     --strategy independent \
//!     --environments "dev,staging,prod"
//!
//! # Force clone (remove existing directory)
//! workspace clone https://github.com/org/repo.git --force
//!
//! # Shallow clone
//! workspace clone https://github.com/org/repo.git --depth 1
//! ```

use crate::cli::commands::CloneArgs;
use crate::error::{CliError, Result};
use crate::output::OutputFormat;
use crate::output::progress::ProgressBar;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};

/// Determines the destination directory for cloning.
///
/// Extracts the repository name from the URL and uses it as the destination
/// if no explicit destination was provided. Supports both HTTPS and SSH URLs.
///
/// # URL Formats Supported
///
/// - HTTPS: `https://github.com/org/repo.git` → `repo`
/// - HTTPS (no .git): `https://github.com/org/repo` → `repo`
/// - SSH: `git@github.com:org/repo.git` → `repo`
/// - SSH (no .git): `git@github.com:org/repo` → `repo`
///
/// # Arguments
///
/// * `url` - The repository URL to clone from
/// * `destination` - Optional explicit destination directory
///
/// # Returns
///
/// Returns the destination `PathBuf` that should be used for cloning.
///
/// # Errors
///
/// Returns an error if:
/// - The URL format is invalid
/// - Cannot extract repository name from URL
///
/// # Examples
///
/// ```rust,ignore
/// // HTTPS URL
/// let dest = determine_destination(
///     "https://github.com/org/repo.git",
///     None
/// )?;
/// assert_eq!(dest, PathBuf::from("repo"));
///
/// // SSH URL
/// let dest = determine_destination(
///     "git@github.com:org/repo.git",
///     None
/// )?;
/// assert_eq!(dest, PathBuf::from("repo"));
///
/// // Explicit destination takes precedence
/// let dest = determine_destination(
///     "https://github.com/org/repo.git",
///     Some(&PathBuf::from("my-dir"))
/// )?;
/// assert_eq!(dest, PathBuf::from("my-dir"));
/// ```
pub(crate) fn determine_destination(url: &str, destination: Option<&PathBuf>) -> Result<PathBuf> {
    // If destination is explicitly provided, use it
    if let Some(dest) = destination {
        return Ok(dest.clone());
    }

    // Extract repository name from URL
    // Supports:
    // - HTTPS: https://github.com/org/repo.git
    // - SSH: git@github.com:org/repo.git
    // - Without .git extension

    // Try HTTPS format first: https://.../org/repo.git or https://.../org/repo
    let https_regex = Regex::new(r"https?://[^/]+/.*/([^/]+?)(\.git)?$")
        .map_err(|e| CliError::validation(format!("Invalid URL regex: {e}")))?;

    if let Some(captures) = https_regex.captures(url)
        && let Some(repo_name) = captures.get(1)
    {
        return Ok(PathBuf::from(repo_name.as_str()));
    }

    // Try SSH format: git@host:org/repo.git or git@host:org/repo
    let ssh_regex = Regex::new(r"^[^@]+@[^:]+:.*/([^/]+?)(\.git)?$")
        .map_err(|e| CliError::validation(format!("Invalid URL regex: {e}")))?;

    if let Some(captures) = ssh_regex.captures(url)
        && let Some(repo_name) = captures.get(1)
    {
        return Ok(PathBuf::from(repo_name.as_str()));
    }

    // If we couldn't parse the URL, return an error
    Err(CliError::validation(format!(
        "Unable to determine repository name from URL: {url}. \
         Please provide an explicit destination directory."
    )))
}

/// Validates that the destination directory is suitable for cloning.
///
/// Checks that the destination either doesn't exist, or can be removed
/// if the --force flag is set.
///
/// # Arguments
///
/// * `destination` - The destination directory path
/// * `force` - Whether to allow overwriting existing directory
///
/// # Returns
///
/// Returns `Ok(())` if the destination is valid, or an error otherwise.
///
/// # Errors
///
/// Returns an error if:
/// - Destination exists and --force is not set
/// - Cannot access destination path
///
/// # Examples
///
/// ```rust,ignore
/// // New directory - should succeed
/// validate_destination(Path::new("./new-dir"), false)?;
///
/// // Existing directory without force - should fail
/// let result = validate_destination(Path::new("./existing"), false);
/// assert!(result.is_err());
///
/// // Existing directory with force - should succeed
/// validate_destination(Path::new("./existing"), true)?;
/// ```
pub(crate) fn validate_destination(destination: &Path, force: bool) -> Result<()> {
    // Check if destination exists
    if destination.exists() {
        if !force {
            return Err(CliError::validation(format!(
                "Destination already exists: {}. Use --force to overwrite.",
                destination.display()
            )));
        }

        // With --force, we'll allow removal (this will be handled by the execute function)
        // Just validate that it's accessible
        if !destination.is_dir() {
            return Err(CliError::validation(format!(
                "Destination exists but is not a directory: {}",
                destination.display()
            )));
        }
    }

    Ok(())
}

/// Clones a Git repository with progress indication.
///
/// This function performs the actual Git clone operation using `sublime_git_tools`,
/// displaying progress feedback to the user via a spinner. It supports both
/// full and shallow clones.
///
/// # Arguments
///
/// * `url` - The repository URL to clone from
/// * `destination` - The local path where the repository should be cloned
/// * `depth` - Optional depth for shallow clone (None for full clone)
/// * `format` - Output format to control progress display
///
/// # Returns
///
/// Returns the cloned `Repo` instance on success.
///
/// # Errors
///
/// Returns an error if:
/// - Network connection fails
/// - Authentication fails (for private repositories)
/// - Insufficient disk space
/// - Permission denied
/// - Invalid URL or destination
///
/// # Examples
///
/// ```rust,ignore
/// // Full clone
/// let repo = clone_with_progress(
///     "https://github.com/org/repo.git",
///     Path::new("./repo"),
///     None,
///     OutputFormat::Human
/// )?;
///
/// // Shallow clone
/// let repo = clone_with_progress(
///     "https://github.com/org/repo.git",
///     Path::new("./repo"),
///     Some(1),
///     OutputFormat::Human
/// )?;
/// ```
///
/// # Implementation Notes
///
/// Currently uses `Repo::clone()` which doesn't support progress callbacks.
/// The spinner provides visual feedback that the operation is in progress.
///
/// TODO: Story 11.2 - Enhance `sublime_git_tools` to support `RemoteCallbacks`
/// for real-time progress tracking (receiving objects, resolving deltas).
pub(crate) fn clone_with_progress(
    url: &str,
    destination: &Path,
    depth: Option<u32>,
    format: OutputFormat,
) -> Result<Repo> {
    let destination_str = destination.to_str().ok_or_else(|| {
        CliError::validation(format!("Invalid destination path: {}", destination.display()))
    })?;

    // Convert depth from u32 to i32 if specified
    let depth_i32 = if let Some(depth_value) = depth {
        Some(i32::try_from(depth_value).map_err(|_| {
            CliError::validation(format!(
                "Depth value {} is too large (maximum: {})",
                depth_value,
                i32::MAX
            ))
        })?)
    } else {
        None
    };

    // Create progress bar for tracking clone progress
    // Start with 0 total - will be updated when we get the first progress callback
    let progress = Arc::new(Mutex::new(ProgressBar::new_with_format(0, format)));
    let progress_clone = Arc::clone(&progress);

    // Start with a spinner message until we know the total
    {
        let pb = progress
            .lock()
            .map_err(|e| CliError::execution(format!("Failed to lock progress bar: {e}")))?;
        pb.set_message(format!("Cloning repository from {url}..."));
    }

    // Perform clone operation with progress tracking
    let result =
        Repo::clone_with_progress(url, destination_str, depth_i32, move |current, total| {
            if let Ok(pb) = progress_clone.lock()
                && total > 0
            {
                // Update progress bar with current progress
                // Casting is intentional: converting object counts to percentage display
                #[allow(
                    clippy::cast_precision_loss,
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss
                )]
                let percentage = (current as f64 / total as f64 * 100.0) as u64;
                pb.set_position(current as u64);
                pb.set_message(format!("Receiving objects: {percentage}% ({current}/{total})"));
            }
        });

    // Handle result
    let repo = match result {
        Ok(repo) => {
            // Clone succeeded
            let pb = progress
                .lock()
                .map_err(|e| CliError::execution(format!("Failed to lock progress bar: {e}")))?;
            pb.finish_with_message("✓ Clone complete");
            repo
        }
        Err(e) => {
            // Clone failed
            let pb = progress
                .lock()
                .map_err(|e| CliError::execution(format!("Failed to lock progress bar: {e}")))?;
            pb.abandon_with_message("✗ Clone failed");
            return Err(map_git_error(&e, url));
        }
    };

    Ok(repo)
}

/// Maps Git errors to user-friendly CLI errors with actionable messages.
///
/// Analyzes the Git error and provides context-specific error messages
/// with suggestions for fixing common issues.
///
/// # Arguments
///
/// * `error` - The Git error from `sublime_git_tools`
/// * `url` - The repository URL that was being cloned
///
/// # Returns
///
/// Returns a `CliError` with a user-friendly message and suggestions.
///
/// # Error Categories
///
/// - Network errors: Connection failed, timeout, DNS resolution
/// - Authentication errors: Invalid credentials, SSH key issues, missing token
/// - Disk space errors: Insufficient space, quota exceeded
/// - Permission errors: Access denied, read-only filesystem
/// - Repository errors: Not found, invalid URL, empty repository
///
/// # Examples
///
/// ```rust,ignore
/// let repo = Repo::clone(url, dest).map_err(|e| map_git_error(e, url))?;
/// ```
fn map_git_error(error: &sublime_git_tools::RepoError, url: &str) -> CliError {
    use sublime_git_tools::RepoError;

    let error_msg = error.to_string();
    let lowercase_msg = error_msg.to_lowercase();

    // Network errors
    if lowercase_msg.contains("failed to resolve")
        || lowercase_msg.contains("could not resolve")
        || lowercase_msg.contains("name or service not known")
    {
        return CliError::git(format!(
            "Network error: Could not resolve host for URL: {url}\n\
             \n\
             Suggestions:\n\
             - Check your internet connection\n\
             - Verify the repository URL is correct\n\
             - Check if the hostname is accessible"
        ));
    }

    if lowercase_msg.contains("connection") || lowercase_msg.contains("timeout") {
        return CliError::git(format!(
            "Network error: Connection failed for URL: {url}\n\
             \n\
             Suggestions:\n\
             - Check your internet connection\n\
             - Verify you can access the repository host\n\
             - Try again later if the service is temporarily unavailable"
        ));
    }

    // Authentication errors
    if lowercase_msg.contains("authentication")
        || lowercase_msg.contains("credential")
        || lowercase_msg.contains("permission denied")
    {
        let suggestion = if url.starts_with("git@") || url.contains("ssh://") {
            "Suggestions:\n\
             - Ensure your SSH key is added to your SSH agent\n\
             - Verify your SSH key is registered with the git hosting service\n\
             - Check SSH key permissions (should be 600 for private key)\n\
             - Try: ssh -T git@<hostname> to test SSH connection"
        } else {
            "Suggestions:\n\
             - Verify you have access to this repository\n\
             - For private repositories, ensure authentication is configured\n\
             - Check if your access token or credentials are valid\n\
             - Try using SSH URL instead of HTTPS"
        };

        return CliError::git(format!(
            "Authentication error: Failed to authenticate for URL: {url}\n\
             \n\
             {suggestion}"
        ));
    }

    // Disk space errors
    if lowercase_msg.contains("no space")
        || lowercase_msg.contains("disk full")
        || lowercase_msg.contains("quota exceeded")
    {
        return CliError::io(
            "Disk space error: Insufficient disk space to clone repository\n\
             \n\
             Suggestions:\n\
             - Free up disk space\n\
             - Choose a different destination with more space\n\
             - Consider using --depth 1 for a shallow clone (requires less space)"
                .to_string(),
        );
    }

    // Repository not found
    if lowercase_msg.contains("not found")
        || lowercase_msg.contains("repository not found")
        || lowercase_msg.contains("404")
    {
        return CliError::git(format!(
            "Repository error: Repository not found: {url}\n\
             \n\
             Suggestions:\n\
             - Verify the repository URL is correct\n\
             - Check if the repository exists\n\
             - Ensure you have access to the repository if it's private"
        ));
    }

    // Generic git error with context
    match &error {
        RepoError::CloneRepoFailure(e) => CliError::git(format!(
            "Failed to clone repository from {url}: {error}\n\
             \n\
             Original error: {e}"
        )),
        _ => CliError::git(format!("Git operation failed: {error}")),
    }
}

/// Executes the clone command.
///
/// This function orchestrates the complete clone workflow including:
/// 1. Determining the destination directory
/// 2. Validating the destination
/// 3. Removing existing destination if --force is set
/// 4. Cloning the repository with progress
/// 5. Detecting and validating workspace configuration (Story 11.3)
/// 6. Initializing workspace if needed (Story 11.4)
///
/// # Arguments
///
/// * `args` - Clone command arguments containing URL, destination, and options
/// * `format` - Output format for progress and messages
///
/// # Returns
///
/// Returns `Ok(())` on successful clone and setup, or an error if any step fails.
///
/// # Errors
///
/// Returns an error if:
/// - URL parsing fails
/// - Destination validation fails
/// - Clone operation fails (network, auth, disk space, etc.)
/// - Workspace setup fails (Stories 11.3/11.4)
///
/// # Examples
///
/// ```rust,ignore
/// let args = CloneArgs {
///     url: "https://github.com/org/repo.git".to_string(),
///     destination: None,
///     force: false,
///     depth: None,
///     // ... other fields
/// };
///
/// execute_clone(&args, OutputFormat::Human).await?;
/// ```
pub async fn execute_clone(args: &CloneArgs, format: OutputFormat) -> Result<()> {
    // Step 1: Determine destination directory
    let destination = determine_destination(&args.url, args.destination.as_ref())?;

    // Step 2: Validate destination
    validate_destination(&destination, args.force)?;

    // Step 3: Remove existing destination if --force is set
    if args.force && destination.exists() {
        let fs = FileSystemManager::new();
        fs.remove(&destination).await.map_err(|e| {
            CliError::io(format!(
                "Failed to remove existing destination {}: {}",
                destination.display(),
                e
            ))
        })?;
    }

    // Step 4: Clone repository with progress
    let _repo = clone_with_progress(&args.url, &destination, args.depth, format)?;

    // TODO: Story 11.3 - Implement configuration detection and validation
    // - Detect if workspace configuration exists
    // - If exists, validate configuration (unless --skip-validation)
    // - Report validation results

    // TODO: Story 11.4 - Implement init integration and output
    // - If no config detected, run execute_init with merged settings
    // - Convert CloneArgs to InitArgs with proper priority
    // - Display success message with next steps

    // Note: #[allow(clippy::todo)] cannot be used on macro invocations
    // This will be implemented as part of Stories 11.3 and 11.4
    todo!(
        "Configuration detection and validation - Story 11.3\nInit integration and output - Story 11.4"
    )
}
