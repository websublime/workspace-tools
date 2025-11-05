//! Dependency audit command implementation.
//!
//! This module provides focused dependency audit functionality.
//!
//! # What
//!
//! Provides:
//! - `execute_dependency_audit` function - Main entry point for dependency-specific audits
//! - Circular dependency detection and reporting
//! - Version conflict detection and highlighting
//! - Dependency categorization analysis (internal/external/workspace/local)
//! - Actionable recommendations for dependency health
//!
//! # How
//!
//! The execution flow:
//! 1. Initialize audit manager and configuration
//! 2. Execute dependency graph analysis via AuditManager
//! 3. Detect circular dependencies using Tarjan's algorithm
//! 4. Identify version conflicts across packages
//! 5. Categorize all dependencies by type
//! 6. Generate detailed report with statistics and recommendations
//! 7. Display results via Output system or write to file
//!
//! # Why
//!
//! A dedicated dependency audit provides:
//! - Early detection of circular dependencies that can break builds
//! - Visibility into version mismatches that may cause runtime issues
//! - Clear understanding of dependency categorization for optimization
//! - Actionable insights for maintaining a healthy dependency graph

use crate::commands::audit::types::{MinSeverity, parse_verbosity};
use crate::error::{CliError, Result};
use crate::output::Output;
use std::path::Path;
use sublime_pkg_tools::audit::{AuditManager, IssueSeverity, Verbosity};

/// Executes a focused dependency audit.
///
/// This function provides detailed dependency graph analysis and reporting,
/// focusing specifically on circular dependencies, version conflicts, and
/// dependency categorization.
///
/// # Arguments
///
/// * `output` - The output context for formatting and display
/// * `workspace_root` - The workspace root directory
/// * `config_path` - Optional path to configuration file
/// * `min_severity` - Minimum severity level for filtering issues
/// * `verbosity` - Level of detail in the report
/// * `output_file` - Optional path to write report to file
///
/// # Returns
///
/// Returns `Ok(())` if the dependency audit completed successfully.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration file cannot be loaded or is invalid
/// - Workspace root is invalid
/// - Audit manager initialization fails
/// - Dependency graph analysis fails
/// - Report generation fails
/// - File I/O operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::audit::dependencies::execute_dependency_audit;
/// use sublime_cli_tools::commands::audit::types::MinSeverity;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::path::Path;
///
/// let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
/// let workspace_root = Path::new(".");
///
/// execute_dependency_audit(
///     &output,
///     workspace_root,
///     None,
///     MinSeverity::Info,
///     "normal",
///     None,
/// ).await?;
/// ```
pub async fn execute_dependency_audit(
    output: &Output,
    workspace_root: &Path,
    config_path: Option<&Path>,
    min_severity: MinSeverity,
    verbosity_str: &str,
    output_file: Option<&Path>,
) -> Result<()> {
    // Parse verbosity
    let verbosity = parse_verbosity(verbosity_str)?;

    // Load configuration
    let config = load_audit_config(config_path).await?;

    // Initialize audit manager
    output.info("Initializing dependency audit...")?;
    let audit_manager = AuditManager::new(workspace_root.to_path_buf(), config)
        .await
        .map_err(|e| CliError::execution(format!("Failed to initialize audit manager: {e}")))?;

    // Execute dependency audit
    output.info("Analyzing dependency graph...")?;
    let dependencies = audit_manager
        .audit_dependencies()
        .await
        .map_err(|e| CliError::execution(format!("Dependency audit failed: {e}")))?;

    // Execute categorization analysis
    output.info("Categorizing dependencies...")?;
    let categorization = audit_manager
        .categorize_dependencies()
        .await
        .map_err(|e| CliError::execution(format!("Dependency categorization failed: {e}")))?;

    // Display results
    format_dependency_report(&dependencies, &categorization, min_severity, verbosity, output)?;

    // Write to file if requested
    if let Some(file_path) = output_file {
        write_dependency_report_to_file(&dependencies, &categorization, file_path)?;
        output.success(&format!("Report written to {}", file_path.display()))?;
    }

    Ok(())
}

/// Formats and displays a dependency audit report.
///
/// This function formats dependency audit results with detailed analysis of
/// circular dependencies, version conflicts, and categorization statistics.
///
/// # Arguments
///
/// * `dependencies` - The dependency audit section results
/// * `categorization` - The dependency categorization results
/// * `min_severity` - Minimum severity level to display
/// * `verbosity` - Level of detail in the report
/// * `output` - Output context for formatting
///
/// # Returns
///
/// Returns `Ok(())` if the report was successfully formatted and displayed.
///
/// # Errors
///
/// Returns an error if output operations fail.
fn format_dependency_report(
    dependencies: &sublime_pkg_tools::audit::DependencyAuditSection,
    categorization: &sublime_pkg_tools::audit::DependencyCategorization,
    min_severity: MinSeverity,
    verbosity: Verbosity,
    output: &Output,
) -> Result<()> {
    // Display header
    output.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    output.info("            DEPENDENCY AUDIT REPORT                ")?;
    output.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    output.info("")?;

    // Filter issues by severity
    let filtered_issues: Vec<_> = dependencies
        .issues
        .iter()
        .filter(|issue| {
            matches!(
                (min_severity, &issue.severity),
                (MinSeverity::Critical, IssueSeverity::Critical)
                    | (MinSeverity::Warning, IssueSeverity::Critical | IssueSeverity::Warning)
                    | (MinSeverity::Info, _)
            )
        })
        .collect();

    // Display summary
    display_dependency_summary(dependencies, categorization, &filtered_issues, output)?;

    // Display circular dependencies (always shown if any exist)
    if !dependencies.circular_dependencies.is_empty() {
        output.info("")?;
        display_circular_dependencies(dependencies, output)?;
    }

    // Display version conflicts (always shown if any exist)
    if !dependencies.version_conflicts.is_empty() {
        output.info("")?;
        display_version_conflicts(dependencies, output)?;
    }

    // Display categorization details
    if matches!(verbosity, Verbosity::Normal | Verbosity::Detailed) {
        display_categorization_details(categorization, verbosity, output)?;
    }

    // Display recommendations
    if matches!(verbosity, Verbosity::Normal | Verbosity::Detailed) {
        display_dependency_recommendations(dependencies, categorization, output)?;
    }

    Ok(())
}

/// Displays the dependency audit summary.
///
/// # Arguments
///
/// * `dependencies` - The dependency audit section results
/// * `categorization` - The dependency categorization results
/// * `filtered_issues` - The filtered issues to count
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_dependency_summary(
    dependencies: &sublime_pkg_tools::audit::DependencyAuditSection,
    categorization: &sublime_pkg_tools::audit::DependencyCategorization,
    filtered_issues: &[&sublime_pkg_tools::audit::AuditIssue],
    output: &Output,
) -> Result<()> {
    output.info("━━━ Summary ━━━")?;

    // Overall health message
    if dependencies.circular_dependencies.is_empty() && dependencies.version_conflicts.is_empty() {
        output.success("✓ No critical dependency issues detected!")?;
    } else {
        if !dependencies.circular_dependencies.is_empty() {
            output.error(&format!(
                "⚠️  {} circular dependencies detected",
                dependencies.circular_dependencies.len()
            ))?;
        }
        if !dependencies.version_conflicts.is_empty() {
            output.warning(&format!(
                "⚠️  {} version conflicts detected",
                dependencies.version_conflicts.len()
            ))?;
        }
    }

    output.info("")?;

    // Issue counts
    let critical_count =
        filtered_issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).count();
    let warning_count =
        filtered_issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).count();
    let info_count =
        filtered_issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Info)).count();

    output.info(&format!("Total Issues: {}", filtered_issues.len()))?;
    if critical_count > 0 {
        output.error(&format!("  Critical: {critical_count}"))?;
    }
    if warning_count > 0 {
        output.warning(&format!("  Warnings: {warning_count}"))?;
    }
    if info_count > 0 {
        output.info(&format!("  Info: {info_count}"))?;
    }

    // Categorization stats
    output.info("")?;
    output.info("Dependency Categorization:")?;
    output.info(&format!("  Internal packages: {}", categorization.stats.internal_packages))?;
    output.info(&format!("  External packages: {}", categorization.stats.external_packages))?;
    output.info(&format!("  Workspace links: {}", categorization.stats.workspace_links))?;
    output.info(&format!("  Local links: {}", categorization.stats.local_links))?;

    Ok(())
}

/// Displays circular dependencies section.
///
/// # Arguments
///
/// * `dependencies` - The dependency audit section results
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_circular_dependencies(
    dependencies: &sublime_pkg_tools::audit::DependencyAuditSection,
    output: &Output,
) -> Result<()> {
    output.info("━━━ Circular Dependencies (CRITICAL) ━━━")?;
    output.info("")?;

    for circular_dep in &dependencies.circular_dependencies {
        output.error("🔁 Circular dependency detected:")?;
        output.info(&format!("   Cycle: {}", circular_dep.display_cycle()))?;
        output.info(&format!("   Length: {} packages", circular_dep.len()))?;
        output.info("")?;
    }

    output.info("💡 Circular dependencies can cause:")?;
    output.info("   - Version resolution failures")?;
    output.info("   - Build system infinite loops")?;
    output.info("   - Runtime initialization issues")?;

    Ok(())
}

/// Displays version conflicts section.
///
/// # Arguments
///
/// * `dependencies` - The dependency audit section results
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_version_conflicts(
    dependencies: &sublime_pkg_tools::audit::DependencyAuditSection,
    output: &Output,
) -> Result<()> {
    output.info("━━━ Version Conflicts (WARNING) ━━━")?;
    output.info("")?;

    for conflict in &dependencies.version_conflicts {
        output.warning(&format!("⚠️  {}", conflict.dependency_name))?;
        output.info(&format!("   {} different versions used:", conflict.versions.len()))?;

        for version_usage in &conflict.versions {
            output.info(&format!(
                "   - {} uses version {}",
                version_usage.package_name, version_usage.version_spec
            ))?;
        }
        output.info("")?;
    }

    output.info("💡 Version conflicts can lead to:")?;
    output.info("   - Multiple copies of the same package in node_modules")?;
    output.info("   - Increased bundle size")?;
    output.info("   - Potential runtime incompatibilities")?;

    Ok(())
}

/// Displays categorization details.
///
/// # Arguments
///
/// * `categorization` - The dependency categorization results
/// * `verbosity` - Level of detail to show
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_categorization_details(
    categorization: &sublime_pkg_tools::audit::DependencyCategorization,
    verbosity: Verbosity,
    output: &Output,
) -> Result<()> {
    output.info("")?;
    output.info("━━━ Dependency Categorization ━━━")?;
    output.info("")?;

    // Internal packages
    if !categorization.internal_packages.is_empty() {
        output.info(&format!("Internal Packages ({}):", categorization.internal_packages.len()))?;

        if matches!(verbosity, Verbosity::Detailed) {
            for pkg in &categorization.internal_packages {
                let version_display = pkg.version.as_deref().unwrap_or("unknown");
                output.info(&format!(
                    "  - {} (v{}) used by {} packages",
                    pkg.name,
                    version_display,
                    pkg.used_by.len()
                ))?;
            }
        } else {
            // Show just the count in normal mode
            let heavily_used: Vec<_> =
                categorization.internal_packages.iter().filter(|p| p.used_by.len() > 3).collect();

            if !heavily_used.is_empty() {
                output.info(&format!(
                    "  {} heavily-used packages (>3 dependents)",
                    heavily_used.len()
                ))?;
            }
        }
        output.info("")?;
    }

    // External packages
    if !categorization.external_packages.is_empty() {
        output.info(&format!("External Packages ({}):", categorization.external_packages.len()))?;

        if matches!(verbosity, Verbosity::Detailed) {
            // Show top 10 most-used external packages
            let mut sorted_external = categorization.external_packages.clone();
            sorted_external.sort_by(|a, b| b.used_by.len().cmp(&a.used_by.len()));

            for pkg in sorted_external.iter().take(10) {
                output.info(&format!(
                    "  - {} ({}) used by {} packages",
                    pkg.name,
                    pkg.version_spec,
                    pkg.used_by.len()
                ))?;
            }

            if sorted_external.len() > 10 {
                output.info(&format!("  ... and {} more", sorted_external.len() - 10))?;
            }
        }
        output.info("")?;
    }

    // Workspace links
    if !categorization.workspace_links.is_empty() {
        output.info(&format!(
            "Workspace Links ({}): Using workspace: protocol",
            categorization.workspace_links.len()
        ))?;

        if matches!(verbosity, Verbosity::Detailed) {
            for link in &categorization.workspace_links {
                output.info(&format!(
                    "  - {} depends on {} ({})",
                    link.package_name, link.dependency_name, link.version_spec
                ))?;
            }
        }
        output.info("")?;
    }

    // Local links
    if !categorization.local_links.is_empty() {
        output.info(&format!(
            "Local Links ({}): Using file:/link:/portal: protocols",
            categorization.local_links.len()
        ))?;

        if matches!(verbosity, Verbosity::Detailed) {
            for link in &categorization.local_links {
                output.info(&format!(
                    "  - {} depends on {} ({:?}) → {}",
                    link.package_name, link.dependency_name, link.link_type, link.path
                ))?;
            }
        }
        output.info("")?;
    }

    Ok(())
}

/// Displays actionable dependency recommendations.
///
/// # Arguments
///
/// * `dependencies` - The dependency audit section results
/// * `categorization` - The dependency categorization results
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_dependency_recommendations(
    dependencies: &sublime_pkg_tools::audit::DependencyAuditSection,
    categorization: &sublime_pkg_tools::audit::DependencyCategorization,
    output: &Output,
) -> Result<()> {
    let mut recommendations = Vec::new();

    // Circular dependencies - highest priority
    if !dependencies.circular_dependencies.is_empty() {
        recommendations.push("🚨 Break circular dependencies immediately");
        recommendations.push("   - Extract shared code into a separate package");
        recommendations.push("   - Use dependency inversion (interfaces/abstractions)");
        recommendations.push("   - Restructure package dependencies");
    }

    // Version conflicts - important
    if !dependencies.version_conflicts.is_empty() {
        recommendations.push("⚠️  Resolve version conflicts to ensure consistency");
        recommendations.push("   - Align dependency versions across packages");
        recommendations.push("   - Use workspace: protocol for internal dependencies");
        recommendations.push("   - Consider using pnpm or yarn workspaces for hoisting");
    }

    // Optimization opportunities
    if categorization.stats.external_packages > 100 {
        recommendations.push("📦 Consider dependency optimization");
        recommendations.push("   - Review and remove unused dependencies");
        recommendations.push("   - Look for smaller alternative packages");
        recommendations.push("   - Use bundle analysis tools to identify large dependencies");
    }

    // Workspace protocol usage
    if categorization.stats.workspace_links == 0 && categorization.stats.internal_packages > 0 {
        recommendations.push("🔗 Consider using workspace: protocol");
        recommendations.push("   - Improves version consistency for internal packages");
        recommendations.push("   - Simplifies dependency management in monorepos");
        recommendations.push("   - Prevents version mismatch issues");
    }

    // Display recommendations if any
    if !recommendations.is_empty() {
        output.info("")?;
        output.info("━━━ Recommendations ━━━")?;
        output.info("")?;

        let mut rec_num = 1;
        for recommendation in &recommendations {
            if recommendation.starts_with("   ") {
                output.info(recommendation)?;
            } else {
                output.info(&format!("{rec_num}. {recommendation}"))?;
                rec_num += 1;
            }
        }

        output.info("")?;
        output.info("💡 Tip: Use --verbosity detailed for more specific package information")?;
    }

    Ok(())
}

/// Writes the dependency report to a file.
///
/// # Arguments
///
/// * `_dependencies` - The dependency audit section results
/// * `_categorization` - The dependency categorization results
/// * `_file_path` - Path to write the report file
///
/// # Returns
///
/// Returns `Ok(())` if the file was written successfully.
///
/// # Errors
///
/// Returns an error if file I/O operations fail or JSON serialization fails.
///
/// # Note
///
/// Currently a placeholder. Full JSON output will be implemented based on
/// the OutputFormat from the global CLI args.
fn write_dependency_report_to_file(
    dependencies: &sublime_pkg_tools::audit::DependencyAuditSection,
    categorization: &sublime_pkg_tools::audit::DependencyCategorization,
    file_path: &Path,
) -> Result<()> {
    use crate::output::export::{ExportFormat, export_data};
    use serde::Serialize;
    use std::collections::HashMap;

    #[derive(Serialize)]
    struct DependencyExportData {
        title: String,
        summary: HashMap<String, serde_json::Value>,
        circular_dependencies: Vec<Vec<String>>,
        version_conflicts: Vec<HashMap<String, serde_json::Value>>,
        categorization: HashMap<String, usize>,
    }

    let mut summary = HashMap::new();
    summary.insert(
        "circular_count".to_string(),
        serde_json::json!(dependencies.circular_dependencies.len()),
    );
    summary.insert(
        "conflict_count".to_string(),
        serde_json::json!(dependencies.version_conflicts.len()),
    );

    let data = DependencyExportData {
        title: "Dependency Audit Report".to_string(),
        summary,
        circular_dependencies: dependencies
            .circular_dependencies
            .iter()
            .map(|cd| cd.cycle.clone())
            .collect(),
        version_conflicts: dependencies
            .version_conflicts
            .iter()
            .map(|c| {
                let mut map = HashMap::new();
                map.insert(
                    "dependency_name".to_string(),
                    serde_json::json!(c.dependency_name.clone()),
                );
                map.insert("versions".to_string(), serde_json::json!(c.versions.clone()));
                map
            })
            .collect(),
        categorization: {
            let mut cat_map = HashMap::new();
            cat_map.insert("internal_packages".to_string(), categorization.internal_packages.len());
            cat_map.insert("external_packages".to_string(), categorization.external_packages.len());
            cat_map.insert("workspace_links".to_string(), categorization.workspace_links.len());
            cat_map.insert("local_links".to_string(), categorization.local_links.len());
            cat_map
        },
    };

    let format = match file_path.extension().and_then(|s| s.to_str()) {
        Some("md" | "markdown") => ExportFormat::Markdown,
        _ => ExportFormat::Html, // default to HTML for .html, .htm, and unknown extensions
    };

    export_data(&data, format, file_path)?;
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
    use sublime_pkg_tools::config::ConfigLoader;

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
