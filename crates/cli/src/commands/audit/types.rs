//! Shared types for audit command implementations.
//!
//! This module provides types and utilities used across audit command implementations.
//!
//! # What
//!
//! Provides:
//! - Section parsing and validation
//! - Severity level parsing
//! - Verbosity level parsing
//! - Format option construction
//! - Output path utilities
//!
//! # How
//!
//! These types bridge the CLI argument layer with the core audit functionality
//! from `sublime-package-tools`. They handle parsing string arguments into
//! strongly-typed enums and structures.
//!
//! # Why
//!
//! Centralizing type conversion and validation:
//! - Ensures consistent behavior across all audit commands
//! - Provides clear error messages for invalid inputs
//! - Keeps command implementations clean and focused
//! - Enables easy testing of argument parsing

use crate::error::{CliError, Result};
use sublime_pkg_tools::audit::{FormatOptions, Verbosity};

/// Audit sections that can be executed.
///
/// This enum represents the different audit sections available.
/// Each section focuses on a specific aspect of project health.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::audit::types::AuditSection;
///
/// let section = AuditSection::parse("upgrades")?;
/// assert_eq!(section, AuditSection::Upgrades);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuditSection {
    /// All audit sections.
    All,
    /// Upgrade availability audit.
    Upgrades,
    /// Dependency health audit.
    Dependencies,
    /// Version consistency audit.
    VersionConsistency,
    /// Breaking changes audit.
    BreakingChanges,
}

impl AuditSection {
    /// Parses a section name from a string.
    ///
    /// # Arguments
    ///
    /// * `s` - The section name to parse
    ///
    /// # Returns
    ///
    /// Returns the parsed `AuditSection` or an error if the name is invalid.
    ///
    /// # Errors
    ///
    /// Returns an error if the section name is not recognized.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let section = AuditSection::parse("all")?;
    /// assert_eq!(section, AuditSection::All);
    /// ```
    pub(crate) fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "all" => Ok(Self::All),
            "upgrades" => Ok(Self::Upgrades),
            "dependencies" => Ok(Self::Dependencies),
            "version-consistency" => Ok(Self::VersionConsistency),
            "breaking-changes" => Ok(Self::BreakingChanges),
            _ => Err(CliError::validation(format!(
                "Invalid audit section '{s}'. Valid options: all, upgrades, dependencies, \
                 version-consistency, breaking-changes"
            ))),
        }
    }

    /// Checks if this section is "all".
    ///
    /// # Returns
    ///
    /// Returns `true` if this section represents all sections.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert!(AuditSection::All.is_all());
    /// assert!(!AuditSection::Upgrades.is_all());
    /// ```
    pub(crate) fn is_all(self) -> bool {
        matches!(self, Self::All)
    }
}

/// Parses a list of audit sections from strings.
///
/// # Arguments
///
/// * `sections` - The section names to parse
///
/// # Returns
///
/// Returns a vector of parsed sections, with "all" expanded or deduplicated.
///
/// # Errors
///
/// Returns an error if any section name is invalid.
///
/// # Examples
///
/// ```rust,ignore
/// let sections = parse_sections(&["all"])?;
/// assert_eq!(sections.len(), 1);
/// assert!(sections[0].is_all());
/// ```
pub(crate) fn parse_sections(sections: &[String]) -> Result<Vec<AuditSection>> {
    let mut parsed = Vec::new();

    for section_str in sections {
        let section = AuditSection::parse(section_str)?;

        // If "all" is found, return just that
        if section.is_all() {
            return Ok(vec![AuditSection::All]);
        }

        // Avoid duplicates
        if !parsed.contains(&section) {
            parsed.push(section);
        }
    }

    // If no sections specified (empty), default to "all"
    if parsed.is_empty() {
        parsed.push(AuditSection::All);
    }

    Ok(parsed)
}

/// Minimum severity level for filtering audit issues.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::audit::types::MinSeverity;
///
/// let severity = MinSeverity::parse("warning")?;
/// assert_eq!(severity, MinSeverity::Warning);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MinSeverity {
    /// Show only critical issues.
    Critical,
    /// Show warning and above issues.
    Warning,
    /// Show all issues including info.
    Info,
}

impl MinSeverity {
    /// Parses a severity level from a string.
    ///
    /// # Arguments
    ///
    /// * `s` - The severity level to parse
    ///
    /// # Returns
    ///
    /// Returns the parsed `MinSeverity` or an error if invalid.
    ///
    /// # Errors
    ///
    /// Returns an error if the severity level is not recognized.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let severity = MinSeverity::parse("critical")?;
    /// assert_eq!(severity, MinSeverity::Critical);
    /// ```
    pub(crate) fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "critical" => Ok(Self::Critical),
            "warning" => Ok(Self::Warning),
            "info" => Ok(Self::Info),
            _ => Err(CliError::validation(format!(
                "Invalid severity level '{s}'. Valid options: critical, warning, info"
            ))),
        }
    }
}

/// Parses verbosity level from a string.
///
/// # Arguments
///
/// * `s` - The verbosity level to parse
///
/// # Returns
///
/// Returns the parsed `Verbosity` from sublime-package-tools.
///
/// # Errors
///
/// Returns an error if the verbosity level is not recognized.
///
/// # Examples
///
/// ```rust,ignore
/// let verbosity = parse_verbosity("detailed")?;
/// ```
pub(crate) fn parse_verbosity(s: &str) -> Result<Verbosity> {
    match s.to_lowercase().as_str() {
        "minimal" => Ok(Verbosity::Minimal),
        "normal" => Ok(Verbosity::Normal),
        "detailed" => Ok(Verbosity::Detailed),
        _ => Err(CliError::validation(format!(
            "Invalid verbosity level '{s}'. Valid options: minimal, normal, detailed"
        ))),
    }
}

/// Builds format options for audit report generation.
///
/// # Arguments
///
/// * `verbosity` - The verbosity level
/// * `include_health_score` - Whether to include health score in the report
///
/// # Returns
///
/// Returns configured `FormatOptions` for report generation.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::Verbosity;
///
/// let options = build_format_options(Verbosity::Normal, true);
/// ```
#[allow(dead_code)] // TODO: will be used in story 8.3 for export formats
pub(crate) fn build_format_options(
    verbosity: Verbosity,
    _include_health_score: bool,
) -> FormatOptions {
    FormatOptions {
        colors: false, // Colors are handled by CLI output system
        verbosity,
        include_suggestions: true,
        include_metadata: true,
    }
}
