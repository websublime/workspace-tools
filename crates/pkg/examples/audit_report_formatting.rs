//! Example demonstrating audit report formatting capabilities.
//!
//! This example shows how to:
//! 1. Create an audit report structure
//! 2. Format it as Markdown with different verbosity levels
//! 3. Export it as JSON
//! 4. Use formatting options for customization

use std::collections::HashMap;
use std::path::PathBuf;
use sublime_pkg_tools::audit::{
    AuditIssue, AuditReport, AuditReportExt, AuditSections, BreakingChangesAuditSection,
    CategorizationStats, DependencyAuditSection, DependencyCategorization, DeprecatedPackage,
    FormatOptions, IssueCategory, IssueSeverity, UpgradeAuditSection, Verbosity,
    VersionConsistencyAuditSection, VersionInconsistency, VersionUsage, format_markdown,
};

fn main() {
    println!("=== Audit Report Formatting Example ===\n");

    // Create a sample audit report
    let report = create_sample_report();

    // Example 1: Default Markdown formatting
    println!("1. Default Markdown Format:");
    println!("{}", "=".repeat(80));
    let markdown_default = report.to_markdown();
    println!("{}", markdown_default);
    println!();

    // Example 2: Minimal verbosity (summary only)
    println!("2. Minimal Verbosity (Summary Only):");
    println!("{}", "=".repeat(80));
    let options_minimal = FormatOptions::default().with_verbosity(Verbosity::Minimal);
    let markdown_minimal = format_markdown(&report, &options_minimal);
    println!("{}", markdown_minimal);
    println!();

    // Example 3: Detailed verbosity (all information)
    println!("3. Detailed Verbosity (All Information):");
    println!("{}", "=".repeat(80));
    let options_detailed = FormatOptions::default()
        .with_verbosity(Verbosity::Detailed)
        .with_suggestions(true)
        .with_metadata(true);
    let markdown_detailed = format_markdown(&report, &options_detailed);
    println!("{}", markdown_detailed);
    println!();

    // Example 4: Custom formatting options
    println!("4. Custom Formatting (No Suggestions, with Metadata):");
    println!("{}", "=".repeat(80));
    let options_custom = FormatOptions::default()
        .with_suggestions(false)
        .with_metadata(true)
        .with_verbosity(Verbosity::Normal);
    let markdown_custom = format_markdown(&report, &options_custom);
    println!("{}", markdown_custom);
    println!();

    // Example 5: JSON formatting
    println!("5. JSON Format:");
    println!("{}", "=".repeat(80));
    match report.to_json() {
        Ok(json) => {
            // Print first 500 chars to keep output manageable
            let preview =
                if json.len() > 500 { format!("{}...\n(truncated)", &json[..500]) } else { json };
            println!("{}", preview);
        }
        Err(e) => eprintln!("Failed to format as JSON: {}", e),
    }
    println!();

    // Example 6: Querying the report
    println!("6. Report Query Examples:");
    println!("{}", "=".repeat(80));
    println!("Health Score: {}/100", report.health_score);
    println!("Passed: {}", report.passed());
    println!("Total Issues: {}", report.total_issues());
    println!("Critical Issues: {}", report.critical_issues().len());
    println!("Warnings: {}", report.warnings().len());
    println!("Info Items: {}", report.info_items().len());
    println!();

    // Example 7: Filtering issues by severity
    println!("7. Issues by Severity:");
    println!("{}", "=".repeat(80));

    let critical = report.critical_issues();
    if !critical.is_empty() {
        println!("Critical Issues ({}):", critical.len());
        for issue in critical {
            println!("  - {}: {}", issue.title, issue.description);
        }
    }

    let warnings = report.warnings();
    if !warnings.is_empty() {
        println!("\nWarnings ({}):", warnings.len());
        for issue in warnings {
            println!("  - {}: {}", issue.title, issue.description);
        }
    }
    println!();

    // Example 8: Suggested actions
    println!("8. Suggested Actions:");
    println!("{}", "=".repeat(80));
    for (i, action) in report.summary.suggested_actions.iter().enumerate() {
        println!("{}. {}", i + 1, action);
    }
    println!();

    println!("=== Example Complete ===");
}

/// Creates a sample audit report for demonstration purposes.
fn create_sample_report() -> AuditReport {
    // Create upgrade section with deprecated packages
    let upgrades = UpgradeAuditSection {
        total_upgrades: 12,
        major_upgrades: 3,
        minor_upgrades: 6,
        patch_upgrades: 3,
        deprecated_packages: vec![
            DeprecatedPackage {
                name: "request".to_string(),
                current_version: "2.88.2".to_string(),
                deprecation_message: "This package is deprecated and no longer maintained"
                    .to_string(),
                alternative: Some("axios".to_string()),
            },
            DeprecatedPackage {
                name: "mkdirp".to_string(),
                current_version: "0.5.5".to_string(),
                deprecation_message: "Functionality is now built into Node.js".to_string(),
                alternative: Some("fs.mkdir with recursive option".to_string()),
            },
        ],
        upgrades_by_package: HashMap::new(),
        issues: vec![
            AuditIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::Upgrades,
                title: "Major upgrades available".to_string(),
                description: "3 packages have major version upgrades available".to_string(),
                affected_packages: vec![
                    "react".to_string(),
                    "webpack".to_string(),
                    "typescript".to_string(),
                ],
                suggestion: Some(
                    "Review breaking changes and migration guides before upgrading".to_string(),
                ),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("upgrade_count".to_string(), "3".to_string());
                    map
                },
            },
            AuditIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::Upgrades,
                title: "Deprecated packages in use".to_string(),
                description: "2 deprecated packages are being used in the project".to_string(),
                affected_packages: vec!["request".to_string(), "mkdirp".to_string()],
                suggestion: Some(
                    "Replace deprecated packages with modern alternatives".to_string(),
                ),
                metadata: HashMap::new(),
            },
        ],
    };

    // Create dependency section
    let dependencies = DependencyAuditSection {
        circular_dependencies: vec![],
        version_conflicts: vec![],
        issues: vec![AuditIssue {
            severity: IssueSeverity::Warning,
            category: IssueCategory::Dependencies,
            title: "Version conflicts detected".to_string(),
            description: "Multiple versions of lodash are in use across packages".to_string(),
            affected_packages: vec!["pkg-a".to_string(), "pkg-b".to_string()],
            suggestion: Some("Align all packages to use the same version of lodash".to_string()),
            metadata: HashMap::new(),
        }],
    };

    // Create breaking changes section
    let breaking_changes = BreakingChangesAuditSection {
        packages_with_breaking: vec![],
        total_breaking_changes: 0,
        issues: vec![],
    };

    // Create categorization section
    let categorization = DependencyCategorization {
        internal_packages: vec![],
        external_packages: vec![],
        workspace_links: vec![],
        local_links: vec![],
        stats: CategorizationStats {
            total_packages: 25,
            internal_packages: 8,
            external_packages: 120,
            workspace_links: 5,
            local_links: 2,
        },
    };

    // Create version consistency section
    let version_consistency = VersionConsistencyAuditSection {
        inconsistencies: vec![VersionInconsistency {
            package_name: "lodash".to_string(),
            versions_used: vec![
                VersionUsage {
                    package_name: "pkg-a".to_string(),
                    version_spec: "^4.17.21".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-b".to_string(),
                    version_spec: "^4.16.0".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-c".to_string(),
                    version_spec: "^4.17.0".to_string(),
                },
            ],
            recommended_version: "^4.17.21".to_string(),
        }],
        issues: vec![AuditIssue {
            severity: IssueSeverity::Info,
            category: IssueCategory::VersionConsistency,
            title: "Version inconsistency found".to_string(),
            description: "lodash is used with different version specifications".to_string(),
            affected_packages: vec!["pkg-a".to_string(), "pkg-b".to_string(), "pkg-c".to_string()],
            suggestion: Some("Use ^4.17.21 across all packages".to_string()),
            metadata: HashMap::new(),
        }],
    };

    // Combine all sections
    let sections = AuditSections {
        upgrades,
        dependencies,
        breaking_changes,
        categorization,
        version_consistency,
    };

    // Create the report
    AuditReport::new(
        PathBuf::from("/example/workspace"),
        true, // is_monorepo
        sections,
        72, // health_score
    )
}
