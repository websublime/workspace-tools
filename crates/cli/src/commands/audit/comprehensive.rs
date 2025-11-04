//! Comprehensive audit command implementation.
//!
//! This module provides the main audit command execution logic.
//!
//! # What
//!
//! Provides:
//! - `execute_audit` function - Main entry point for audit command
//! - Configuration loading and validation
//! - Audit manager initialization
//! - Orchestration of all audit sections
//! - Health score calculation
//! - Report generation and display
//! - File output handling
//!
//! # How
//!
//! The execution flow:
//! 1. Load workspace configuration
//! 2. Parse and validate command arguments
//! 3. Initialize audit manager from sublime-package-tools
//! 4. Execute selected audit sections (or all by default)
//! 5. Aggregate results from all sections
//! 6. Calculate overall health score
//! 7. Apply severity filtering if requested
//! 8. Generate formatted report
//! 9. Display report via Output system or write to file
//!
//! # Why
//!
//! Centralizing audit execution:
//! - Provides consistent workflow across all audit types
//! - Ensures proper error handling and reporting
//! - Integrates seamlessly with other CLI commands
//! - Supports multiple output formats and destinations

use crate::cli::commands::AuditArgs;
use crate::commands::audit::report::format_audit_report;
use crate::commands::audit::types::{AuditSection, MinSeverity, parse_sections, parse_verbosity};
use crate::error::{CliError, Result};
use crate::output::Output;
use std::path::Path;
use sublime_pkg_tools::audit::AuditManager;
use sublime_pkg_tools::audit::{AuditIssue, IssueSeverity};
use sublime_pkg_tools::audit::{
    BreakingChangesAuditSection, DependencyAuditSection, UpgradeAuditSection,
    VersionConsistencyAuditSection,
};
use sublime_pkg_tools::config::ConfigLoader;

/// Aggregated results from all audit sections.
///
/// This structure collects results from individual audit sections
/// to provide a unified view of project health.
///
/// # Examples
///
/// ```rust,ignore
/// let results = AuditResults {
///     upgrades: Some(upgrades_section),
///     dependencies: Some(dependencies_section),
///     version_consistency: Some(version_section),
///     breaking_changes: None, // Not yet implemented
/// };
/// ```
#[derive(Debug)]
pub struct AuditResults {
    /// Results from upgrade audit section.
    pub upgrades: Option<UpgradeAuditSection>,

    /// Results from dependency audit section.
    pub dependencies: Option<DependencyAuditSection>,

    /// Results from version consistency audit section.
    pub version_consistency: Option<VersionConsistencyAuditSection>,

    /// Results from breaking changes audit section.
    pub breaking_changes: Option<BreakingChangesAuditSection>,
}

impl AuditResults {
    /// Collects all issues from all audit sections.
    ///
    /// # Returns
    ///
    /// A vector containing all audit issues from all sections.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let results = AuditResults { /* ... */ };
    /// let all_issues = results.all_issues();
    /// println!("Total issues: {}", all_issues.len());
    /// ```
    pub fn all_issues(&self) -> Vec<&AuditIssue> {
        let mut issues = Vec::new();

        if let Some(ref upgrades) = self.upgrades {
            issues.extend(upgrades.issues.iter());
        }

        if let Some(ref dependencies) = self.dependencies {
            issues.extend(dependencies.issues.iter());
        }

        if let Some(ref version_consistency) = self.version_consistency {
            issues.extend(version_consistency.issues.iter());
        }

        if let Some(ref breaking_changes) = self.breaking_changes {
            issues.extend(breaking_changes.issues.iter());
        }

        issues
    }

    /// Counts issues by severity level.
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level to count
    ///
    /// # Returns
    ///
    /// The number of issues with the specified severity.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let results = AuditResults { /* ... */ };
    /// let critical_count = results.count_by_severity(&IssueSeverity::Critical);
    /// println!("Critical issues: {}", critical_count);
    /// ```
    pub fn count_by_severity(&self, severity: &IssueSeverity) -> usize {
        self.all_issues().iter().filter(|issue| &issue.severity == severity).count()
    }

    /// Calculates an overall health score from all audit sections.
    ///
    /// The health score is calculated based on:
    /// - Number and severity of issues found
    /// - Upgrade availability metrics
    /// - Dependency health metrics
    ///
    /// # Returns
    ///
    /// A score from 0 to 100, where 100 is perfect health.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let results = AuditResults { /* ... */ };
    /// let score = results.calculate_health_score();
    /// println!("Project health: {}%", score);
    /// ```
    pub fn calculate_health_score(&self) -> u8 {
        let critical_issues = self.count_by_severity(&IssueSeverity::Critical);
        let warning_issues = self.count_by_severity(&IssueSeverity::Warning);
        let info_issues = self.count_by_severity(&IssueSeverity::Info);

        // Start with perfect score
        let mut score: i32 = 100;

        // Deduct points for issues by severity (capped to prevent overflow)
        let critical_deduction = critical_issues.min(6) * 15; // Cap at 6 to prevent excessive deduction
        let warning_deduction = warning_issues.min(20) * 5; // Cap at 20
        let info_deduction = info_issues.min(100); // Cap at 100, no multiplication needed

        score -= i32::try_from(critical_deduction).unwrap_or(90);
        score -= i32::try_from(warning_deduction).unwrap_or(100);
        score -= i32::try_from(info_deduction).unwrap_or(100);

        // Additional deductions based on upgrade metrics
        if let Some(ref upgrades) = self.upgrades {
            // Deduct for major upgrades available (potential breaking changes)
            let major_deduction = upgrades.major_upgrades.min(50) * 2;
            score -= i32::try_from(major_deduction).unwrap_or(100);

            // Deduct for deprecated packages
            let deprecated_deduction = upgrades.deprecated_packages.len().min(20) * 5;
            score -= i32::try_from(deprecated_deduction).unwrap_or(100);
        }

        // Additional deductions based on dependency metrics
        if let Some(ref dependencies) = self.dependencies {
            // Circular dependencies are serious issues
            let circular_deduction = dependencies.circular_dependencies.len().min(10) * 10;
            score -= i32::try_from(circular_deduction).unwrap_or(100);

            // Version conflicts are problematic
            let conflict_deduction = dependencies.version_conflicts.len().min(20) * 5;
            score -= i32::try_from(conflict_deduction).unwrap_or(100);
        }

        // Ensure score stays within 0-100 range
        u8::try_from(score.clamp(0, 100)).unwrap_or(0)
    }
}

/// Executes the comprehensive audit command.
///
/// This function orchestrates the entire audit process from configuration loading
/// through report generation and display.
///
/// # Arguments
///
/// * `args` - The audit command arguments
/// * `output` - The output context for formatting and display
/// * `workspace_root` - The workspace root directory
/// * `config_path` - Optional path to configuration file
///
/// # Returns
///
/// Returns `Ok(())` if the audit completed successfully.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration file cannot be loaded or is invalid
/// - Workspace root is invalid
/// - Audit manager initialization fails
/// - Audit execution fails
/// - Report generation fails
/// - File I/O operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::cli::commands::AuditArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::path::Path;
///
/// let args = AuditArgs {
///     sections: vec!["all".to_string()],
///     output: None,
///     min_severity: "info".to_string(),
///     verbosity: "normal".to_string(),
///     no_health_score: false,
/// };
///
/// let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
/// let workspace_root = Path::new(".");
///
/// execute_audit(&args, &output, workspace_root, None).await?;
/// ```
pub async fn execute_audit(
    args: &AuditArgs,
    output: &Output,
    workspace_root: &Path,
    config_path: Option<&Path>,
) -> Result<()> {
    // Parse and validate arguments
    let sections = parse_sections(&args.sections)?;
    let min_severity = MinSeverity::parse(&args.min_severity)?;
    let verbosity = parse_verbosity(&args.verbosity)?;

    // Load configuration
    let config = load_audit_config(config_path).await?;

    // Initialize audit manager
    let audit_manager = AuditManager::new(workspace_root.to_path_buf(), config)
        .await
        .map_err(|e| CliError::execution(format!("Failed to initialize audit manager: {e}")))?;

    // Execute audit sections based on selection
    let mut results = AuditResults {
        upgrades: None,
        dependencies: None,
        version_consistency: None,
        breaking_changes: None,
    };

    // Determine which sections to run
    let run_all = sections.iter().any(|s| s.is_all());

    // Run upgrade audit if requested or all
    if run_all || sections.contains(&AuditSection::Upgrades) {
        output.info("Running upgrade audit...")?;
        let upgrades = audit_manager
            .audit_upgrades()
            .await
            .map_err(|e| CliError::execution(format!("Upgrade audit failed: {e}")))?;
        results.upgrades = Some(upgrades);
    }

    // Run dependency audit if requested or all
    if run_all || sections.contains(&AuditSection::Dependencies) {
        output.info("Running dependency audit...")?;
        let dependencies = audit_manager
            .audit_dependencies()
            .await
            .map_err(|e| CliError::execution(format!("Dependency audit failed: {e}")))?;
        results.dependencies = Some(dependencies);
    }

    // Run version consistency audit if requested or all
    if run_all || sections.contains(&AuditSection::VersionConsistency) {
        output.info("Running version consistency audit...")?;
        let version_consistency = audit_manager
            .audit_version_consistency()
            .await
            .map_err(|e| CliError::execution(format!("Version consistency audit failed: {e}")))?;
        results.version_consistency = Some(version_consistency);
    }

    // Run breaking changes audit if requested or all
    if run_all || sections.contains(&AuditSection::BreakingChanges) {
        output.info("Running breaking changes audit...")?;
        let breaking_changes = audit_manager
            .audit_breaking_changes()
            .await
            .map_err(|e| CliError::execution(format!("Breaking changes audit failed: {e}")))?;
        results.breaking_changes = Some(breaking_changes);
    }

    // Calculate health score
    let health_score =
        if args.no_health_score { None } else { Some(results.calculate_health_score()) };

    // Format and display report
    format_audit_report(
        &results,
        health_score,
        min_severity,
        verbosity,
        output,
        args.output.as_deref(),
    )
    .await?;

    Ok(())
}

/// Loads audit configuration from workspace.
///
/// # Arguments
///
/// * `config_path` - Optional path to configuration file
///
/// # Returns
///
/// Returns the loaded `PackageToolsConfig`.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration file cannot be found
/// - Configuration file cannot be parsed
/// - Configuration is invalid
async fn load_audit_config(
    config_path: Option<&Path>,
) -> Result<sublime_pkg_tools::config::PackageToolsConfig> {
    let config = if let Some(path) = config_path {
        ConfigLoader::load_from_file(path).await.map_err(|e| {
            CliError::configuration(format!("Failed to load config from {}: {e}", path.display()))
        })?
    } else {
        ConfigLoader::load_defaults()
            .await
            .map_err(|e| CliError::configuration(format!("Failed to load default config: {e}")))?
    };

    Ok(config)
}
