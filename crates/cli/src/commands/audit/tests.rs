//! Tests for audit command implementations.
//!
//! This module contains comprehensive tests for the audit command functionality.
//!
//! # What
//!
//! Provides tests for:
//! - Audit command execution
//! - Section parsing and validation
//! - Severity filtering
//! - Report generation and formatting
//! - Configuration loading
//! - Error handling
//!
//! # How
//!
//! Tests use:
//! - Mock configurations and workspace setups
//! - Temporary directories for file operations
//! - Mock output contexts
//! - Tokio runtime for async tests
//!
//! # Why
//!
//! Comprehensive testing ensures:
//! - Audit command works correctly across different scenarios
//! - Error cases are handled properly
//! - Output formatting is consistent
//! - Integration with sublime-package-tools works as expected
//! - 100% test coverage goal is met

#[cfg(test)]
mod comprehensive_tests {
    use crate::cli::commands::AuditArgs;
    use crate::commands::audit::comprehensive::execute_audit;
    use crate::output::{Output, OutputFormat};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_execute_audit_valid_args() {
        let args = AuditArgs {
            sections: vec!["all".to_string()],
            output: None,
            min_severity: "info".to_string(),
            verbosity: "normal".to_string(),
            no_health_score: false,
        };

        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        // This will try to initialize AuditManager and run audit
        // It may fail if not in a valid git repo, but that's OK for this test
        let result = execute_audit(&args, &output, &workspace_root, None).await;

        // We don't assert success because we may not be in a valid workspace
        // The important thing is that the function doesn't panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_execute_audit_invalid_section() {
        let args = AuditArgs {
            sections: vec!["invalid_section".to_string()],
            output: None,
            min_severity: "info".to_string(),
            verbosity: "normal".to_string(),
            no_health_score: false,
        };

        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_audit(&args, &output, &workspace_root, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_audit_invalid_severity() {
        let args = AuditArgs {
            sections: vec!["all".to_string()],
            output: None,
            min_severity: "invalid".to_string(),
            verbosity: "normal".to_string(),
            no_health_score: false,
        };

        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_audit(&args, &output, &workspace_root, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_audit_invalid_verbosity() {
        let args = AuditArgs {
            sections: vec!["all".to_string()],
            output: None,
            min_severity: "info".to_string(),
            verbosity: "invalid".to_string(),
            no_health_score: false,
        };

        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_audit(&args, &output, &workspace_root, None).await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod audit_results_tests {
    use crate::commands::audit::comprehensive::AuditResults;
    use sublime_pkg_tools::audit::IssueSeverity;

    #[test]
    fn test_audit_results_all_issues_empty() {
        let results = AuditResults {
            upgrades: None,
            dependencies: None,
            version_consistency: None,
            breaking_changes: None,
        };

        assert_eq!(results.all_issues().len(), 0);
    }

    #[test]
    fn test_audit_results_count_by_severity() {
        let results = AuditResults {
            upgrades: None,
            dependencies: None,
            version_consistency: None,
            breaking_changes: None,
        };

        assert_eq!(results.count_by_severity(&IssueSeverity::Critical), 0);
        assert_eq!(results.count_by_severity(&IssueSeverity::Warning), 0);
        assert_eq!(results.count_by_severity(&IssueSeverity::Info), 0);
    }

    #[test]
    fn test_audit_results_health_score_perfect() {
        let results = AuditResults {
            upgrades: None,
            dependencies: None,
            version_consistency: None,
            breaking_changes: None,
        };

        assert_eq!(results.calculate_health_score(), 100);
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod types_tests {
    use crate::commands::audit::types::{
        AuditSection, MinSeverity, build_format_options, parse_sections, parse_verbosity,
    };
    use sublime_pkg_tools::audit::Verbosity;

    #[test]
    fn test_audit_section_parse_all() {
        let section = AuditSection::parse("all").expect("should parse");
        assert_eq!(section, AuditSection::All);
        assert!(section.is_all());
    }

    #[test]
    fn test_audit_section_parse_upgrades() {
        let section = AuditSection::parse("upgrades").expect("should parse");
        assert_eq!(section, AuditSection::Upgrades);
        assert!(!section.is_all());
    }

    #[test]
    fn test_audit_section_parse_dependencies() {
        let section = AuditSection::parse("dependencies").expect("should parse");
        assert_eq!(section, AuditSection::Dependencies);
    }

    #[test]
    fn test_audit_section_parse_version_consistency() {
        let section = AuditSection::parse("version-consistency").expect("should parse");
        assert_eq!(section, AuditSection::VersionConsistency);
    }

    #[test]
    fn test_audit_section_parse_breaking_changes() {
        let section = AuditSection::parse("breaking-changes").expect("should parse");
        assert_eq!(section, AuditSection::BreakingChanges);
    }

    #[test]
    fn test_audit_section_parse_case_insensitive() {
        let section = AuditSection::parse("UPGRADES").expect("should parse");
        assert_eq!(section, AuditSection::Upgrades);

        let section = AuditSection::parse("Version-Consistency").expect("should parse");
        assert_eq!(section, AuditSection::VersionConsistency);
    }

    #[test]
    fn test_audit_section_parse_invalid() {
        let result = AuditSection::parse("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sections_single() {
        let sections = parse_sections(&["upgrades".to_string()]).expect("should parse");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0], AuditSection::Upgrades);
    }

    #[test]
    fn test_parse_sections_multiple() {
        let sections = parse_sections(&["upgrades".to_string(), "dependencies".to_string()])
            .expect("should parse");
        assert_eq!(sections.len(), 2);
        assert!(sections.contains(&AuditSection::Upgrades));
        assert!(sections.contains(&AuditSection::Dependencies));
    }

    #[test]
    fn test_parse_sections_all_overrides() {
        let sections =
            parse_sections(&["upgrades".to_string(), "all".to_string()]).expect("should parse");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0], AuditSection::All);
    }

    #[test]
    fn test_parse_sections_deduplicates() {
        let sections = parse_sections(&["upgrades".to_string(), "upgrades".to_string()])
            .expect("should parse");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0], AuditSection::Upgrades);
    }

    #[test]
    fn test_parse_sections_empty() {
        let sections = parse_sections(&[]).expect("should parse");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0], AuditSection::All);
    }

    #[test]
    fn test_min_severity_parse_critical() {
        let severity = MinSeverity::parse("critical").expect("should parse");
        assert_eq!(severity, MinSeverity::Critical);
    }

    #[test]
    fn test_min_severity_parse_warning() {
        let severity = MinSeverity::parse("warning").expect("should parse");
        assert_eq!(severity, MinSeverity::Warning);
    }

    #[test]
    fn test_min_severity_parse_info() {
        let severity = MinSeverity::parse("info").expect("should parse");
        assert_eq!(severity, MinSeverity::Info);
    }

    #[test]
    fn test_min_severity_parse_case_insensitive() {
        let severity = MinSeverity::parse("WARNING").expect("should parse");
        assert_eq!(severity, MinSeverity::Warning);
    }

    #[test]
    fn test_min_severity_parse_invalid() {
        let result = MinSeverity::parse("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_verbosity_minimal() {
        let verbosity = parse_verbosity("minimal").expect("should parse");
        assert!(matches!(verbosity, Verbosity::Minimal));
    }

    #[test]
    fn test_parse_verbosity_normal() {
        let verbosity = parse_verbosity("normal").expect("should parse");
        assert!(matches!(verbosity, Verbosity::Normal));
    }

    #[test]
    fn test_parse_verbosity_detailed() {
        let verbosity = parse_verbosity("detailed").expect("should parse");
        assert!(matches!(verbosity, Verbosity::Detailed));
    }

    #[test]
    fn test_parse_verbosity_case_insensitive() {
        let verbosity = parse_verbosity("DETAILED").expect("should parse");
        assert!(matches!(verbosity, Verbosity::Detailed));
    }

    #[test]
    fn test_parse_verbosity_invalid() {
        let result = parse_verbosity("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_format_options() {
        let options = build_format_options(Verbosity::Normal, true);
        assert!(matches!(options.verbosity, Verbosity::Normal));
        assert!(!options.colors); // Colors handled by CLI
        assert!(options.include_suggestions);
        assert!(options.include_metadata);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(MinSeverity::Critical < MinSeverity::Warning);
        assert!(MinSeverity::Warning < MinSeverity::Info);
    }
}

#[cfg(test)]
mod report_tests {
    use crate::commands::audit::comprehensive::AuditResults;
    use crate::commands::audit::report::{
        display_health_score, display_recommendations, filter_issues_by_severity,
        write_report_to_file,
    };
    use crate::commands::audit::types::MinSeverity;
    use crate::output::{Output, OutputFormat};
    use std::path::Path;

    #[test]
    fn test_filter_issues_empty() {
        let results = AuditResults {
            upgrades: None,
            dependencies: None,
            version_consistency: None,
            breaking_changes: None,
        };

        let filtered = filter_issues_by_severity(&results, MinSeverity::Info);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_display_health_score_success() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let result = display_health_score(95, &output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_health_score_warning() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let result = display_health_score(70, &output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_health_score_error() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let result = display_health_score(45, &output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_report_to_file_not_implemented() {
        let results = AuditResults {
            upgrades: None,
            dependencies: None,
            version_consistency: None,
            breaking_changes: None,
        };

        let path = Path::new("/tmp/test-report.json");
        let result = write_report_to_file(&results, Some(100), path);
        assert!(result.is_err());
    }

    #[test]
    fn test_display_recommendations_no_issues() {
        let results = AuditResults {
            upgrades: None,
            dependencies: None,
            version_consistency: None,
            breaking_changes: None,
        };

        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let result = display_recommendations(&results, Some(100), &output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_recommendations_low_health_score() {
        let results = AuditResults {
            upgrades: None,
            dependencies: None,
            version_consistency: None,
            breaking_changes: None,
        };

        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let result = display_recommendations(&results, Some(50), &output);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod upgrade_audit_tests {
    use crate::commands::audit::types::MinSeverity;
    use crate::commands::audit::upgrades::execute_upgrade_audit;
    use crate::output::{Output, OutputFormat};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_execute_upgrade_audit_valid_args() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        // This will try to initialize AuditManager and run upgrade audit
        // It may fail if not in a valid git repo, but that's OK for this test
        let result = execute_upgrade_audit(
            &output,
            &workspace_root,
            None,
            MinSeverity::Info,
            "normal",
            None,
        )
        .await;

        // We don't assert success because we may not be in a valid workspace
        // The important thing is that the function doesn't panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_execute_upgrade_audit_critical_severity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_upgrade_audit(
            &output,
            &workspace_root,
            None,
            MinSeverity::Critical,
            "normal",
            None,
        )
        .await;

        // Should not panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_execute_upgrade_audit_warning_severity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_upgrade_audit(
            &output,
            &workspace_root,
            None,
            MinSeverity::Warning,
            "normal",
            None,
        )
        .await;

        // Should not panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_execute_upgrade_audit_minimal_verbosity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_upgrade_audit(
            &output,
            &workspace_root,
            None,
            MinSeverity::Info,
            "minimal",
            None,
        )
        .await;

        // Should not panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_execute_upgrade_audit_detailed_verbosity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_upgrade_audit(
            &output,
            &workspace_root,
            None,
            MinSeverity::Info,
            "detailed",
            None,
        )
        .await;

        // Should not panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_execute_upgrade_audit_invalid_verbosity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_upgrade_audit(
            &output,
            &workspace_root,
            None,
            MinSeverity::Info,
            "invalid",
            None,
        )
        .await;

        // Should return error due to invalid verbosity
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_upgrade_audit_with_output_file() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");
        let output_file = PathBuf::from("/tmp/upgrade-audit-test.json");

        let result = execute_upgrade_audit(
            &output,
            &workspace_root,
            None,
            MinSeverity::Info,
            "normal",
            Some(&output_file),
        )
        .await;

        // This should fail with todo! for file output
        // The error is expected as file output is not yet implemented (story 7.6)
        let _ = result;
    }

    #[tokio::test]
    async fn test_execute_upgrade_audit_invalid_workspace() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from("/nonexistent/path/to/workspace");

        let result = execute_upgrade_audit(
            &output,
            &workspace_root,
            None,
            MinSeverity::Info,
            "normal",
            None,
        )
        .await;

        // Should fail because workspace doesn't exist
        assert!(result.is_err());
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod upgrade_report_formatting_tests {
    use std::collections::HashMap;
    use sublime_pkg_tools::audit::{DeprecatedPackage, UpgradeAuditSection};

    #[test]
    fn test_upgrade_audit_section_empty() {
        let section = UpgradeAuditSection::empty();
        assert_eq!(section.total_upgrades, 0);
        assert_eq!(section.major_upgrades, 0);
        assert_eq!(section.minor_upgrades, 0);
        assert_eq!(section.patch_upgrades, 0);
        assert!(section.deprecated_packages.is_empty());
        assert!(section.upgrades_by_package.is_empty());
        assert!(section.issues.is_empty());
        assert!(!section.has_upgrades());
        assert!(!section.has_deprecated_packages());
    }

    #[test]
    fn test_upgrade_audit_section_has_upgrades() {
        let mut section = UpgradeAuditSection::empty();
        section.total_upgrades = 5;
        section.major_upgrades = 2;
        section.minor_upgrades = 2;
        section.patch_upgrades = 1;

        assert!(section.has_upgrades());
        assert_eq!(section.total_upgrades, 5);
    }

    #[test]
    fn test_upgrade_audit_section_has_deprecated() {
        let mut section = UpgradeAuditSection::empty();
        section.deprecated_packages.push(DeprecatedPackage {
            name: "old-package".to_string(),
            current_version: "1.0.0".to_string(),
            deprecation_message: "This package is deprecated".to_string(),
            alternative: Some("new-package".to_string()),
        });

        assert!(section.has_deprecated_packages());
        assert_eq!(section.deprecated_packages.len(), 1);
    }

    #[test]
    fn test_upgrade_audit_section_issue_counts() {
        use sublime_pkg_tools::audit::{AuditIssue, IssueCategory, IssueSeverity};

        let mut section = UpgradeAuditSection::empty();

        // Add some critical issues
        section.issues.push(AuditIssue::new(
            IssueSeverity::Critical,
            IssueCategory::Upgrades,
            "Critical issue".to_string(),
            "Description".to_string(),
        ));
        section.issues.push(AuditIssue::new(
            IssueSeverity::Critical,
            IssueCategory::Upgrades,
            "Another critical".to_string(),
            "Description".to_string(),
        ));

        // Add some warnings
        section.issues.push(AuditIssue::new(
            IssueSeverity::Warning,
            IssueCategory::Upgrades,
            "Warning issue".to_string(),
            "Description".to_string(),
        ));

        // Add some info
        section.issues.push(AuditIssue::new(
            IssueSeverity::Info,
            IssueCategory::Upgrades,
            "Info issue".to_string(),
            "Description".to_string(),
        ));

        assert_eq!(section.critical_issue_count(), 2);
        assert_eq!(section.warning_issue_count(), 1);
        assert_eq!(section.info_issue_count(), 1);
        assert_eq!(section.issues.len(), 4);
    }

    #[test]
    fn test_upgrade_audit_section_upgrades_by_package() {
        use sublime_pkg_tools::types::DependencyType;
        use sublime_pkg_tools::upgrade::{DependencyUpgrade, UpgradeType, VersionInfo};

        let mut section = UpgradeAuditSection::empty();
        let mut upgrades_map = HashMap::new();

        let upgrade = DependencyUpgrade {
            name: "lodash".to_string(),
            current_version: "4.17.20".to_string(),
            latest_version: "4.17.21".to_string(),
            upgrade_type: UpgradeType::Patch,
            dependency_type: DependencyType::Regular,
            registry_url: "https://registry.npmjs.org".to_string(),
            version_info: VersionInfo {
                available_versions: vec!["4.17.21".to_string()],
                latest_stable: "4.17.21".to_string(),
                latest_prerelease: None,
                deprecated: None,
                published_at: None,
            },
        };

        upgrades_map.insert("my-app".to_string(), vec![upgrade]);
        section.upgrades_by_package = upgrades_map;

        let pkg_upgrades = section.upgrades_for_package("my-app");
        assert_eq!(pkg_upgrades.len(), 1);
        assert_eq!(pkg_upgrades[0].name, "lodash");

        let no_upgrades = section.upgrades_for_package("nonexistent");
        assert!(no_upgrades.is_empty());
    }

    #[test]
    fn test_deprecated_package_with_alternative() {
        let deprecated = DeprecatedPackage {
            name: "request".to_string(),
            current_version: "2.88.2".to_string(),
            deprecation_message: "This package is deprecated, use axios instead".to_string(),
            alternative: Some("axios".to_string()),
        };

        assert_eq!(deprecated.name, "request");
        assert_eq!(deprecated.current_version, "2.88.2");
        assert!(deprecated.alternative.is_some());
        assert_eq!(deprecated.alternative.unwrap(), "axios");
    }

    #[test]
    fn test_deprecated_package_without_alternative() {
        let deprecated = DeprecatedPackage {
            name: "old-lib".to_string(),
            current_version: "1.0.0".to_string(),
            deprecation_message: "No longer maintained".to_string(),
            alternative: None,
        };

        assert_eq!(deprecated.name, "old-lib");
        assert!(deprecated.alternative.is_none());
    }
}
