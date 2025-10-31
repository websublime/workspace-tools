//! Command definitions and argument structures.
//!
//! This module defines all CLI commands and their associated argument structures.
//! Each command is represented as a variant in the `Commands` enum with its
//! corresponding arguments struct.
//!
//! # What
//!
//! Provides:
//! - `Commands` enum with all available commands
//! - Argument structures for each command
//! - Command-specific options and flags
//! - Subcommand hierarchies (e.g., config show/validate)
//!
//! # How
//!
//! Uses Clap's derive macros to define commands and their arguments.
//! Each command has a dedicated args struct that encapsulates all its options.
//!
//! # Why
//!
//! Separating command definitions from execution logic enables:
//! - Clear command structure and documentation
//! - Type-safe argument handling
//! - Automatic help generation
//! - Easy testing of command parsing
//!
//! # Examples
//!
//! ```rust
//! use clap::Parser;
//! use sublime_cli_tools::cli::{Cli, Commands};
//!
//! let cli = Cli::parse_from(["wnt", "version"]);
//! match cli.command {
//!     Commands::Version(_) => println!("Version command"),
//!     _ => {}
//! }
//! ```

use clap::{Args, Subcommand};
use std::path::PathBuf;

/// All available CLI commands.
///
/// Each variant represents a top-level command with its associated arguments.
///
/// # Examples
///
/// ```rust
/// use clap::Parser;
/// use sublime_cli_tools::cli::{Cli, Commands};
///
/// let cli = Cli::parse_from(["wnt", "init"]);
/// matches!(cli.command, Commands::Init(_));
/// ```
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize project configuration.
    ///
    /// Creates a new configuration file for changeset-based version management.
    /// Supports interactive and non-interactive modes.
    Init(InitArgs),

    /// Manage configuration.
    ///
    /// View, validate, and modify project configuration.
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Manage changesets.
    ///
    /// Create, update, list, and manage changesets for version control.
    #[command(subcommand)]
    Changeset(ChangesetCommands),

    /// Bump package versions based on changesets.
    ///
    /// Calculates and applies version bumps according to active changesets
    /// and the configured versioning strategy.
    Bump(BumpArgs),

    /// Manage dependency upgrades.
    ///
    /// Check for available upgrades and apply them to workspace packages.
    #[command(subcommand)]
    Upgrade(UpgradeCommands),

    /// Run project health audit.
    ///
    /// Analyzes project health including upgrades, dependencies,
    /// version consistency, and breaking changes.
    Audit(AuditArgs),

    /// Analyze changes in repository.
    ///
    /// Detects which packages are affected by changes in the working
    /// directory or between commits.
    Changes(ChangesArgs),

    /// Display version information.
    ///
    /// Shows the CLI version and optionally detailed build information.
    Version(VersionArgs),
}

// ============================================================================
// Init Command
// ============================================================================

/// Arguments for the `init` command.
///
/// # Examples
///
/// ```rust
/// use clap::Parser;
/// use sublime_cli_tools::cli::Cli;
///
/// let cli = Cli::parse_from(["wnt", "init", "--non-interactive"]);
/// ```
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Changeset directory path.
    ///
    /// Directory where changeset files will be stored.
    ///
    /// Default: .changesets/
    #[arg(long, value_name = "PATH", default_value = ".changesets")]
    pub changeset_path: PathBuf,

    /// Comma-separated list of environments.
    ///
    /// Example: "dev,staging,prod"
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub environments: Option<Vec<String>>,

    /// Comma-separated list of default environments.
    ///
    /// Example: "prod"
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub default_env: Option<Vec<String>>,

    /// Versioning strategy.
    ///
    /// - independent: Each package versions independently
    /// - unified: All packages share the same version
    #[arg(long, value_name = "STRATEGY")]
    pub strategy: Option<String>,

    /// NPM registry URL.
    ///
    /// Default: <https://registry.npmjs.org>
    #[arg(long, value_name = "URL", default_value = "https://registry.npmjs.org")]
    pub registry: String,

    /// Configuration file format.
    ///
    /// Options: json, toml, yaml
    #[arg(long = "config-format", value_name = "FORMAT")]
    pub config_format: Option<String>,

    /// Overwrite existing configuration.
    ///
    /// Forces initialization even if a config file already exists.
    #[arg(long)]
    pub force: bool,

    /// Non-interactive mode.
    ///
    /// Uses default values or provided flags without prompting.
    #[arg(long)]
    pub non_interactive: bool,
}

// ============================================================================
// Config Commands
// ============================================================================

/// Subcommands for the `config` command.
///
/// # Examples
///
/// ```rust
/// use clap::Parser;
/// use sublime_cli_tools::cli::Cli;
///
/// let cli = Cli::parse_from(["wnt", "config", "show"]);
/// ```
#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Display current configuration.
    ///
    /// Shows all configuration values from the detected config file.
    Show(ConfigShowArgs),

    /// Validate configuration file.
    ///
    /// Checks that the configuration file is valid and all required
    /// fields are present.
    Validate(ConfigValidateArgs),
}

/// Arguments for the `config show` command.
#[derive(Debug, Args)]
pub struct ConfigShowArgs {
    // No additional args beyond global options
}

/// Arguments for the `config validate` command.
#[derive(Debug, Args)]
pub struct ConfigValidateArgs {
    // No additional args beyond global options
}

// ============================================================================
// Changeset Commands
// ============================================================================

/// Subcommands for the `changeset` command.
///
/// # Examples
///
/// ```rust
/// use clap::Parser;
/// use sublime_cli_tools::cli::Cli;
///
/// let cli = Cli::parse_from(["wnt", "changeset", "create", "--bump", "minor"]);
/// ```
#[derive(Debug, Subcommand)]
pub enum ChangesetCommands {
    /// Create a new changeset.
    ///
    /// Creates a changeset for the current branch with specified bump type
    /// and environments.
    Create(ChangesetCreateArgs),

    /// Update an existing changeset.
    ///
    /// Adds commits or packages to an existing changeset.
    Update(ChangesetUpdateArgs),

    /// List all changesets.
    ///
    /// Shows all active changesets with optional filtering and sorting.
    List(ChangesetListArgs),

    /// Show changeset details.
    ///
    /// Displays detailed information about a specific changeset.
    Show(ChangesetShowArgs),

    /// Edit a changeset in the user's editor.
    ///
    /// Opens the changeset file in $EDITOR for manual editing.
    Edit(ChangesetEditArgs),

    /// Delete a changeset.
    ///
    /// Removes a changeset from the active changesets.
    Delete(ChangesetDeleteArgs),

    /// Query changeset history.
    ///
    /// Searches archived changesets with filtering options.
    History(ChangesetHistoryArgs),

    /// Check if changeset exists.
    ///
    /// Checks if a changeset exists for the current or specified branch.
    /// Useful for Git hooks.
    Check(ChangesetCheckArgs),
}

/// Arguments for the `changeset create` command.
#[derive(Debug, Args)]
pub struct ChangesetCreateArgs {
    /// Bump type.
    ///
    /// Options: major, minor, patch
    #[arg(long, value_name = "TYPE")]
    pub bump: Option<String>,

    /// Comma-separated list of environments.
    ///
    /// Example: "staging,prod"
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub env: Option<Vec<String>>,

    /// Branch name.
    ///
    /// Defaults to current Git branch.
    #[arg(long, value_name = "NAME")]
    pub branch: Option<String>,

    /// Changeset message.
    ///
    /// Optional description of the changes.
    #[arg(long, value_name = "TEXT")]
    pub message: Option<String>,

    /// Comma-separated list of packages.
    ///
    /// Auto-detected from Git changes if not provided.
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub packages: Option<Vec<String>>,

    /// Non-interactive mode.
    ///
    /// Uses provided flags without prompting.
    #[arg(long)]
    pub non_interactive: bool,
}

/// Arguments for the `changeset update` command.
#[derive(Debug, Args)]
pub struct ChangesetUpdateArgs {
    /// Changeset ID or branch name.
    ///
    /// Defaults to current branch if not provided.
    #[arg(value_name = "ID")]
    pub id: Option<String>,

    /// Add specific commit hash.
    ///
    /// Adds a specific commit to the changeset.
    #[arg(long, value_name = "HASH")]
    pub commit: Option<String>,

    /// Comma-separated list of packages to add.
    ///
    /// Adds specific packages to the changeset.
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub packages: Option<Vec<String>>,

    /// Update bump type.
    ///
    /// Options: major, minor, patch
    #[arg(long, value_name = "TYPE")]
    pub bump: Option<String>,

    /// Comma-separated list of environments to add.
    ///
    /// Example: "staging,prod"
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub env: Option<Vec<String>>,
}

/// Arguments for the `changeset list` command.
#[derive(Debug, Args)]
pub struct ChangesetListArgs {
    /// Filter by package name.
    #[arg(long, value_name = "NAME")]
    pub filter_package: Option<String>,

    /// Filter by bump type.
    ///
    /// Options: major, minor, patch
    #[arg(long, value_name = "TYPE")]
    pub filter_bump: Option<String>,

    /// Filter by environment.
    #[arg(long, value_name = "ENV")]
    pub filter_env: Option<String>,

    /// Sort by field.
    ///
    /// Options: date, bump, branch
    #[arg(long, value_name = "FIELD", default_value = "date")]
    pub sort: String,
}

/// Arguments for the `changeset show` command.
#[derive(Debug, Args)]
pub struct ChangesetShowArgs {
    /// Branch name or changeset ID.
    #[arg(value_name = "BRANCH")]
    pub branch: String,
}

/// Arguments for the `changeset edit` command.
#[derive(Debug, Args)]
pub struct ChangesetEditArgs {
    /// Branch name to edit changeset for.
    ///
    /// Defaults to current Git branch if not provided.
    #[arg(value_name = "BRANCH")]
    pub branch: Option<String>,
}

/// Arguments for the `changeset delete` command.
#[derive(Debug, Args)]
pub struct ChangesetDeleteArgs {
    /// Branch name to delete changeset for.
    #[arg(value_name = "BRANCH")]
    pub branch: String,

    /// Skip confirmation prompt.
    #[arg(long)]
    pub force: bool,
}

/// Arguments for the `changeset history` command.
#[derive(Debug, Args)]
pub struct ChangesetHistoryArgs {
    /// Filter by package name.
    #[arg(long, value_name = "NAME")]
    pub package: Option<String>,

    /// Since date (ISO 8601).
    ///
    /// Example: "2024-01-01"
    #[arg(long, value_name = "DATE")]
    pub since: Option<String>,

    /// Until date (ISO 8601).
    ///
    /// Example: "2024-12-31"
    #[arg(long, value_name = "DATE")]
    pub until: Option<String>,

    /// Filter by environment.
    #[arg(long, value_name = "ENV")]
    pub env: Option<String>,

    /// Filter by bump type.
    ///
    /// Options: major, minor, patch
    #[arg(long, value_name = "TYPE")]
    pub bump: Option<String>,

    /// Limit number of results.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
}

/// Arguments for the `changeset check` command.
#[derive(Debug, Args)]
pub struct ChangesetCheckArgs {
    /// Branch name to check.
    ///
    /// Defaults to current Git branch.
    #[arg(long, value_name = "NAME")]
    pub branch: Option<String>,
}

// ============================================================================
// Bump Command
// ============================================================================

/// Arguments for the `bump` command.
///
/// # Examples
///
/// ```rust
/// use clap::Parser;
/// use sublime_cli_tools::cli::Cli;
///
/// let cli = Cli::parse_from(["wnt", "bump", "--dry-run"]);
/// ```
#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct BumpArgs {
    /// Preview changes without applying.
    ///
    /// Shows what would be changed without modifying any files.
    #[arg(long)]
    pub dry_run: bool,

    /// Apply version changes.
    ///
    /// Must be explicitly specified to apply changes.
    /// Cannot be used with --dry-run.
    #[arg(long, conflicts_with = "dry_run")]
    pub execute: bool,

    /// Generate snapshot versions.
    ///
    /// Creates snapshot versions based on the snapshot format template.
    #[arg(long)]
    pub snapshot: bool,

    /// Snapshot format template.
    ///
    /// Variables: {version}, {branch}, {short_commit}, {commit}
    /// Example: "{version}-{branch}.{short_commit}"
    #[arg(long, value_name = "FORMAT")]
    pub snapshot_format: Option<String>,

    /// Pre-release tag.
    ///
    /// Options: alpha, beta, rc
    #[arg(long, value_name = "TAG")]
    pub prerelease: Option<String>,

    /// Comma-separated list of packages to bump.
    ///
    /// Overrides changeset packages.
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub packages: Option<Vec<String>>,

    /// Create Git tags for releases.
    ///
    /// Tags are created in the format: package@version
    #[arg(long)]
    pub git_tag: bool,

    /// Push Git tags to remote.
    ///
    /// Requires --git-tag to be set.
    #[arg(long, requires = "git_tag")]
    pub git_push: bool,

    /// Commit version changes.
    ///
    /// Creates a commit with all version changes.
    #[arg(long)]
    pub git_commit: bool,

    /// Don't update changelogs.
    ///
    /// Skips changelog generation/updates.
    #[arg(long)]
    pub no_changelog: bool,

    /// Don't archive changesets.
    ///
    /// Keeps changesets active after bump.
    #[arg(long)]
    pub no_archive: bool,

    /// Skip confirmations.
    ///
    /// Automatically confirms all prompts.
    #[arg(long)]
    pub force: bool,
}

// ============================================================================
// Upgrade Commands
// ============================================================================

/// Subcommands for the `upgrade` command.
///
/// # Examples
///
/// ```rust
/// use clap::Parser;
/// use sublime_cli_tools::cli::Cli;
///
/// let cli = Cli::parse_from(["wnt", "upgrade", "check"]);
/// ```
#[derive(Debug, Subcommand)]
pub enum UpgradeCommands {
    /// Check for available upgrades.
    ///
    /// Detects outdated dependencies in workspace packages.
    Check(UpgradeCheckArgs),

    /// Apply dependency upgrades.
    ///
    /// Updates dependencies to newer versions.
    Apply(UpgradeApplyArgs),

    /// Manage upgrade backups.
    ///
    /// List, restore, or clean upgrade backups.
    #[command(subcommand)]
    Backups(UpgradeBackupCommands),
}

/// Arguments for the `upgrade check` command.
#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct UpgradeCheckArgs {
    /// Include major version upgrades.
    ///
    /// Default: true
    #[arg(long, default_value = "true")]
    pub major: bool,

    /// Don't include major version upgrades.
    #[arg(long, conflicts_with = "major")]
    pub no_major: bool,

    /// Include minor version upgrades.
    ///
    /// Default: true
    #[arg(long, default_value = "true")]
    pub minor: bool,

    /// Don't include minor version upgrades.
    #[arg(long, conflicts_with = "minor")]
    pub no_minor: bool,

    /// Include patch version upgrades.
    ///
    /// Default: true
    #[arg(long, default_value = "true")]
    pub patch: bool,

    /// Don't include patch version upgrades.
    #[arg(long, conflicts_with = "patch")]
    pub no_patch: bool,

    /// Include dev dependencies.
    ///
    /// Default: true
    #[arg(long, default_value = "true")]
    pub dev: bool,

    /// Include peer dependencies.
    ///
    /// Default: false
    #[arg(long)]
    pub peer: bool,

    /// Comma-separated list of packages to check.
    ///
    /// Only checks specified packages.
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub packages: Option<Vec<String>>,

    /// Override registry URL.
    ///
    /// Uses this registry instead of the configured one.
    #[arg(long, value_name = "URL")]
    pub registry: Option<String>,
}

/// Arguments for the `upgrade apply` command.
#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct UpgradeApplyArgs {
    /// Preview without applying.
    ///
    /// Shows what would be upgraded without making changes.
    #[arg(long)]
    pub dry_run: bool,

    /// Only apply patch upgrades.
    ///
    /// Restricts upgrades to patch versions only.
    #[arg(long, conflicts_with_all = ["minor_and_patch"])]
    pub patch_only: bool,

    /// Only apply minor and patch upgrades.
    ///
    /// Restricts upgrades to non-breaking versions.
    #[arg(long, conflicts_with = "patch_only")]
    pub minor_and_patch: bool,

    /// Comma-separated list of packages to upgrade.
    ///
    /// Only upgrades specified packages.
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub packages: Option<Vec<String>>,

    /// Automatically create changeset.
    ///
    /// Creates a changeset for the upgrades.
    #[arg(long)]
    pub auto_changeset: bool,

    /// Changeset bump type.
    ///
    /// Options: major, minor, patch
    /// Default: patch
    #[arg(long, value_name = "TYPE", default_value = "patch")]
    pub changeset_bump: String,

    /// Skip backup creation.
    ///
    /// Does not create a backup before upgrading.
    #[arg(long)]
    pub no_backup: bool,

    /// Skip confirmations.
    ///
    /// Automatically confirms all prompts.
    #[arg(long)]
    pub force: bool,
}

/// Subcommands for `upgrade backups`.
#[derive(Debug, Subcommand)]
pub enum UpgradeBackupCommands {
    /// List all backups.
    ///
    /// Shows available upgrade backups.
    List(UpgradeBackupListArgs),

    /// Restore a backup.
    ///
    /// Restores package.json files from a backup.
    Restore(UpgradeBackupRestoreArgs),

    /// Clean old backups.
    ///
    /// Removes old backup files.
    Clean(UpgradeBackupCleanArgs),
}

/// Arguments for the `upgrade backups list` command.
#[derive(Debug, Args)]
pub struct UpgradeBackupListArgs {
    // No additional args beyond global options
}

/// Arguments for the `upgrade backups restore` command.
#[derive(Debug, Args)]
pub struct UpgradeBackupRestoreArgs {
    /// Backup ID to restore.
    ///
    /// Example: backup_20240115_103045
    #[arg(value_name = "ID")]
    pub id: String,

    /// Skip confirmation prompt.
    #[arg(long)]
    pub force: bool,
}

/// Arguments for the `upgrade backups clean` command.
#[derive(Debug, Args)]
pub struct UpgradeBackupCleanArgs {
    /// Number of recent backups to keep.
    ///
    /// Default: 5
    #[arg(long, value_name = "N", default_value = "5")]
    pub keep: usize,

    /// Skip confirmation prompt.
    #[arg(long)]
    pub force: bool,
}

// ============================================================================
// Audit Command
// ============================================================================

/// Arguments for the `audit` command.
///
/// # Examples
///
/// ```rust
/// use clap::Parser;
/// use sublime_cli_tools::cli::Cli;
///
/// let cli = Cli::parse_from(["wnt", "audit"]);
/// ```
#[derive(Debug, Args)]
pub struct AuditArgs {
    /// Comma-separated list of sections to audit.
    ///
    /// Options: all, upgrades, dependencies, version-consistency, breaking-changes
    /// Default: all
    #[arg(long, value_name = "LIST", value_delimiter = ',', default_value = "all")]
    pub sections: Vec<String>,

    /// Write output to file.
    ///
    /// If not specified, writes to stdout.
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Minimum severity level.
    ///
    /// Options: critical, high, medium, low, info
    /// Default: info
    #[arg(long, value_name = "LEVEL", default_value = "info")]
    pub min_severity: String,

    /// Detail level.
    ///
    /// Options: minimal, normal, detailed
    /// Default: normal
    #[arg(long, value_name = "LEVEL", default_value = "normal")]
    pub verbosity: String,

    /// Skip health score calculation.
    ///
    /// Disables the overall health score in the output.
    #[arg(long)]
    pub no_health_score: bool,
}

// ============================================================================
// Changes Command
// ============================================================================

/// Arguments for the `changes` command.
///
/// # Examples
///
/// ```rust
/// use clap::Parser;
/// use sublime_cli_tools::cli::Cli;
///
/// let cli = Cli::parse_from(["wnt", "changes", "--since", "HEAD~1"]);
/// ```
#[derive(Debug, Args)]
pub struct ChangesArgs {
    /// Since commit/branch/tag.
    ///
    /// Analyzes changes since this Git reference.
    /// If not provided, analyzes working directory changes.
    #[arg(long, value_name = "REF")]
    pub since: Option<String>,

    /// Until commit/branch/tag.
    ///
    /// Analyzes changes until this Git reference.
    /// Default: HEAD
    #[arg(long, value_name = "REF")]
    pub until: Option<String>,

    /// Compare against branch.
    ///
    /// Compares current branch against specified branch.
    #[arg(long, value_name = "NAME")]
    pub branch: Option<String>,

    /// Only staged changes.
    ///
    /// Analyzes only files in the Git staging area.
    #[arg(long, conflicts_with = "unstaged")]
    pub staged: bool,

    /// Only unstaged changes.
    ///
    /// Analyzes only files not in the Git staging area.
    #[arg(long, conflicts_with = "staged")]
    pub unstaged: bool,

    /// Comma-separated list of packages to filter.
    ///
    /// Only shows changes for specified packages.
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub packages: Option<Vec<String>>,
}

// ============================================================================
// Version Command
// ============================================================================

/// Arguments for the `version` command.
///
/// # Examples
///
/// ```rust
/// use clap::Parser;
/// use sublime_cli_tools::cli::Cli;
///
/// let cli = Cli::parse_from(["wnt", "version", "--verbose"]);
/// ```
#[derive(Debug, Args)]
pub struct VersionArgs {
    /// Show detailed version information.
    ///
    /// Includes Rust version, dependencies, and build information.
    #[arg(long)]
    pub verbose: bool,
}
