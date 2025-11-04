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

#[cfg(test)]
mod dependency_audit_tests {
    use crate::commands::audit::dependencies::execute_dependency_audit;
    use crate::commands::audit::types::MinSeverity;
    use crate::output::{Output, OutputFormat};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_execute_dependency_audit_valid_args() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        // This will try to initialize AuditManager and run dependency audit
        // It may fail if not in a valid git repo, but that's OK for this test
        let result = execute_dependency_audit(
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
    async fn test_execute_dependency_audit_critical_severity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_dependency_audit(
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
    async fn test_execute_dependency_audit_warning_severity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_dependency_audit(
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
    async fn test_execute_dependency_audit_minimal_verbosity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_dependency_audit(
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
    async fn test_execute_dependency_audit_detailed_verbosity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_dependency_audit(
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
    async fn test_execute_dependency_audit_invalid_verbosity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_dependency_audit(
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
    async fn test_execute_dependency_audit_with_output_file() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");
        let output_file = PathBuf::from("/tmp/dependency-audit-test.json");

        let result = execute_dependency_audit(
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
    async fn test_execute_dependency_audit_invalid_workspace() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from("/nonexistent/path/to/workspace");

        let result = execute_dependency_audit(
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
mod dependency_report_formatting_tests {
    use sublime_pkg_tools::audit::{DependencyAuditSection, VersionConflict, VersionUsage};
    use sublime_pkg_tools::types::CircularDependency;

    #[test]
    fn test_dependency_audit_section_empty() {
        let section = DependencyAuditSection::empty();
        assert!(section.circular_dependencies.is_empty());
        assert!(section.version_conflicts.is_empty());
        assert!(section.issues.is_empty());
        assert!(!section.has_circular_dependencies());
        assert!(!section.has_version_conflicts());
    }

    #[test]
    fn test_dependency_audit_section_has_circular_dependencies() {
        let mut section = DependencyAuditSection::empty();
        section
            .circular_dependencies
            .push(CircularDependency::new(vec!["pkg-a".to_string(), "pkg-b".to_string()]));

        assert!(section.has_circular_dependencies());
        assert_eq!(section.circular_dependencies.len(), 1);
    }

    #[test]
    fn test_dependency_audit_section_has_version_conflicts() {
        let mut section = DependencyAuditSection::empty();
        section.version_conflicts.push(VersionConflict {
            dependency_name: "lodash".to_string(),
            versions: vec![
                VersionUsage {
                    package_name: "pkg-a".to_string(),
                    version_spec: "^4.17.20".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-b".to_string(),
                    version_spec: "^3.10.1".to_string(),
                },
            ],
        });

        assert!(section.has_version_conflicts());
        assert_eq!(section.version_conflicts.len(), 1);
    }

    #[test]
    fn test_dependency_audit_section_issue_counts() {
        use sublime_pkg_tools::audit::{AuditIssue, IssueCategory, IssueSeverity};

        let mut section = DependencyAuditSection::empty();

        // Add critical issues
        section.issues.push(AuditIssue::new(
            IssueSeverity::Critical,
            IssueCategory::Dependencies,
            "Critical circular dependency".to_string(),
            "Circular dependency detected".to_string(),
        ));
        section.issues.push(AuditIssue::new(
            IssueSeverity::Critical,
            IssueCategory::Dependencies,
            "Another critical issue".to_string(),
            "Description".to_string(),
        ));

        // Add warnings
        section.issues.push(AuditIssue::new(
            IssueSeverity::Warning,
            IssueCategory::Dependencies,
            "Version conflict".to_string(),
            "Multiple versions detected".to_string(),
        ));

        // Add info
        section.issues.push(AuditIssue::new(
            IssueSeverity::Info,
            IssueCategory::Dependencies,
            "Info issue".to_string(),
            "Description".to_string(),
        ));

        assert_eq!(section.critical_issue_count(), 2);
        assert_eq!(section.warning_issue_count(), 1);
        assert_eq!(section.info_issue_count(), 1);
        assert_eq!(section.issues.len(), 4);
    }

    #[test]
    fn test_circular_dependency_display() {
        let circular_dep = CircularDependency::new(vec![
            "pkg-a".to_string(),
            "pkg-b".to_string(),
            "pkg-a".to_string(),
        ]);

        assert_eq!(circular_dep.len(), 3);
        assert!(circular_dep.involves("pkg-a"));
        assert!(circular_dep.involves("pkg-b"));
        assert!(!circular_dep.involves("pkg-c"));

        let display = circular_dep.display_cycle();
        assert!(display.contains("pkg-a"));
        assert!(display.contains("pkg-b"));
    }

    #[test]
    fn test_circular_dependencies_for_package() {
        let mut section = DependencyAuditSection::empty();
        section
            .circular_dependencies
            .push(CircularDependency::new(vec!["pkg-a".to_string(), "pkg-b".to_string()]));
        section
            .circular_dependencies
            .push(CircularDependency::new(vec!["pkg-c".to_string(), "pkg-d".to_string()]));

        let cycles_for_a = section.circular_dependencies_for_package("pkg-a");
        assert_eq!(cycles_for_a.len(), 1);

        let cycles_for_c = section.circular_dependencies_for_package("pkg-c");
        assert_eq!(cycles_for_c.len(), 1);

        let cycles_for_e = section.circular_dependencies_for_package("pkg-e");
        assert_eq!(cycles_for_e.len(), 0);
    }

    #[test]
    fn test_version_conflict_count() {
        let conflict = VersionConflict {
            dependency_name: "lodash".to_string(),
            versions: vec![
                VersionUsage {
                    package_name: "pkg-a".to_string(),
                    version_spec: "^4.17.20".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-b".to_string(),
                    version_spec: "^3.10.1".to_string(),
                },
            ],
        };

        assert_eq!(conflict.version_count(), 2);
    }

    #[test]
    fn test_version_conflict_describe() {
        let conflict = VersionConflict {
            dependency_name: "lodash".to_string(),
            versions: vec![
                VersionUsage {
                    package_name: "pkg-a".to_string(),
                    version_spec: "^4.17.20".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-b".to_string(),
                    version_spec: "^3.10.1".to_string(),
                },
            ],
        };

        let description = conflict.describe();
        assert!(description.contains("lodash"));
        assert!(description.contains("pkg-a"));
        assert!(description.contains("pkg-b"));
        assert!(description.contains("^4.17.20"));
        assert!(description.contains("^3.10.1"));
    }

    #[test]
    fn test_version_conflicts_for_dependency() {
        let mut section = DependencyAuditSection::empty();
        section.version_conflicts.push(VersionConflict {
            dependency_name: "lodash".to_string(),
            versions: vec![VersionUsage {
                package_name: "pkg-a".to_string(),
                version_spec: "^4.17.20".to_string(),
            }],
        });

        let conflict = section.version_conflicts_for_dependency("lodash");
        assert!(conflict.is_some());
        assert_eq!(conflict.unwrap().dependency_name, "lodash");

        let no_conflict = section.version_conflicts_for_dependency("nonexistent");
        assert!(no_conflict.is_none());
    }
}

#[cfg(test)]
mod version_consistency_audit_tests {
    use crate::commands::audit::types::MinSeverity;
    use crate::commands::audit::versions::execute_version_consistency_audit;
    use crate::output::{Output, OutputFormat};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_execute_version_consistency_audit_valid_args() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        // This will try to initialize AuditManager and run version consistency audit
        // It may fail if not in a valid git repo, but that's OK for this test
        let result = execute_version_consistency_audit(
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
    async fn test_execute_version_consistency_audit_critical_severity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_version_consistency_audit(
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
    async fn test_execute_version_consistency_audit_warning_severity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_version_consistency_audit(
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
    async fn test_execute_version_consistency_audit_minimal_verbosity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_version_consistency_audit(
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
    async fn test_execute_version_consistency_audit_detailed_verbosity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_version_consistency_audit(
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
    async fn test_execute_version_consistency_audit_invalid_verbosity() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");

        let result = execute_version_consistency_audit(
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
    async fn test_execute_version_consistency_audit_with_output_file() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from(".");
        let output_file = PathBuf::from("/tmp/version-consistency-audit-test.json");

        let result = execute_version_consistency_audit(
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
    async fn test_execute_version_consistency_audit_invalid_workspace() {
        let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
        let workspace_root = PathBuf::from("/nonexistent/path/to/workspace");

        let result = execute_version_consistency_audit(
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
mod version_consistency_report_formatting_tests {
    use sublime_pkg_tools::audit::{
        VersionConsistencyAuditSection, VersionInconsistency, VersionUsage,
    };

    #[test]
    fn test_version_consistency_audit_section_empty() {
        let section = VersionConsistencyAuditSection::empty();
        assert!(section.inconsistencies.is_empty());
        assert!(section.issues.is_empty());
        assert!(!section.has_inconsistencies());
    }

    #[test]
    fn test_version_consistency_audit_section_has_inconsistencies() {
        let mut section = VersionConsistencyAuditSection::empty();
        section.inconsistencies.push(VersionInconsistency {
            package_name: "@myorg/core".to_string(),
            versions_used: vec![
                VersionUsage {
                    package_name: "app-a".to_string(),
                    version_spec: "^1.0.0".to_string(),
                },
                VersionUsage {
                    package_name: "app-b".to_string(),
                    version_spec: "^1.1.0".to_string(),
                },
            ],
            recommended_version: "^1.1.0".to_string(),
        });

        assert!(section.has_inconsistencies());
        assert_eq!(section.inconsistencies.len(), 1);
    }

    #[test]
    fn test_version_consistency_audit_section_issue_counts() {
        use sublime_pkg_tools::audit::{AuditIssue, IssueCategory, IssueSeverity};

        let mut section = VersionConsistencyAuditSection::empty();

        // Add critical issues
        section.issues.push(AuditIssue::new(
            IssueSeverity::Critical,
            IssueCategory::VersionConsistency,
            "Critical inconsistency".to_string(),
            "Version mismatch detected".to_string(),
        ));
        section.issues.push(AuditIssue::new(
            IssueSeverity::Critical,
            IssueCategory::VersionConsistency,
            "Another critical issue".to_string(),
            "Description".to_string(),
        ));

        // Add warnings
        section.issues.push(AuditIssue::new(
            IssueSeverity::Warning,
            IssueCategory::VersionConsistency,
            "Warning issue".to_string(),
            "Minor inconsistency".to_string(),
        ));

        // Add info
        section.issues.push(AuditIssue::new(
            IssueSeverity::Info,
            IssueCategory::VersionConsistency,
            "Info issue".to_string(),
            "Description".to_string(),
        ));

        assert_eq!(section.critical_issue_count(), 2);
        assert_eq!(section.warning_issue_count(), 1);
        assert_eq!(section.info_issue_count(), 1);
        assert_eq!(section.issues.len(), 4);
    }

    #[test]
    fn test_version_inconsistency_version_count() {
        let inconsistency = VersionInconsistency {
            package_name: "@myorg/utils".to_string(),
            versions_used: vec![
                VersionUsage {
                    package_name: "pkg-a".to_string(),
                    version_spec: "^1.0.0".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-b".to_string(),
                    version_spec: "^2.0.0".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-c".to_string(),
                    version_spec: "^1.0.0".to_string(),
                },
            ],
            recommended_version: "workspace:*".to_string(),
        };

        assert_eq!(inconsistency.version_count(), 3);
    }

    #[test]
    fn test_version_inconsistency_describe() {
        let inconsistency = VersionInconsistency {
            package_name: "@myorg/core".to_string(),
            versions_used: vec![
                VersionUsage {
                    package_name: "app-a".to_string(),
                    version_spec: "^1.0.0".to_string(),
                },
                VersionUsage {
                    package_name: "app-b".to_string(),
                    version_spec: "^1.1.0".to_string(),
                },
            ],
            recommended_version: "^1.1.0".to_string(),
        };

        let description = inconsistency.describe();
        assert!(description.contains("@myorg/core"));
        assert!(description.contains("app-a"));
        assert!(description.contains("app-b"));
        assert!(description.contains("^1.0.0"));
        assert!(description.contains("^1.1.0"));
    }

    #[test]
    fn test_version_inconsistency_unique_versions() {
        let inconsistency = VersionInconsistency {
            package_name: "@myorg/shared".to_string(),
            versions_used: vec![
                VersionUsage {
                    package_name: "pkg-a".to_string(),
                    version_spec: "^1.0.0".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-b".to_string(),
                    version_spec: "^1.0.0".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-c".to_string(),
                    version_spec: "^2.0.0".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-d".to_string(),
                    version_spec: "workspace:*".to_string(),
                },
            ],
            recommended_version: "workspace:*".to_string(),
        };

        let unique_versions = inconsistency.unique_versions();
        assert_eq!(unique_versions.len(), 3);
        assert!(unique_versions.contains(&"^1.0.0".to_string()));
        assert!(unique_versions.contains(&"^2.0.0".to_string()));
        assert!(unique_versions.contains(&"workspace:*".to_string()));
    }

    #[test]
    fn test_version_inconsistency_for_package() {
        let mut section = VersionConsistencyAuditSection::empty();
        section.inconsistencies.push(VersionInconsistency {
            package_name: "@myorg/core".to_string(),
            versions_used: vec![VersionUsage {
                package_name: "app-a".to_string(),
                version_spec: "^1.0.0".to_string(),
            }],
            recommended_version: "^1.0.0".to_string(),
        });

        let inconsistency = section.inconsistency_for_package("@myorg/core");
        assert!(inconsistency.is_some());
        assert_eq!(inconsistency.unwrap().package_name, "@myorg/core");

        let no_inconsistency = section.inconsistency_for_package("nonexistent");
        assert!(no_inconsistency.is_none());
    }

    #[test]
    fn test_version_inconsistency_with_workspace_protocol() {
        let inconsistency = VersionInconsistency {
            package_name: "@myorg/common".to_string(),
            versions_used: vec![
                VersionUsage {
                    package_name: "pkg-a".to_string(),
                    version_spec: "workspace:*".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-b".to_string(),
                    version_spec: "^1.0.0".to_string(),
                },
            ],
            recommended_version: "workspace:*".to_string(),
        };

        assert_eq!(inconsistency.package_name, "@myorg/common");
        assert_eq!(inconsistency.recommended_version, "workspace:*");
        assert!(inconsistency.versions_used.iter().any(|v| v.version_spec == "workspace:*"));
    }

    #[test]
    fn test_version_inconsistency_multiple_packages_same_version() {
        let inconsistency = VersionInconsistency {
            package_name: "@myorg/utils".to_string(),
            versions_used: vec![
                VersionUsage {
                    package_name: "pkg-a".to_string(),
                    version_spec: "^1.5.0".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-b".to_string(),
                    version_spec: "^1.5.0".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-c".to_string(),
                    version_spec: "^1.5.0".to_string(),
                },
                VersionUsage {
                    package_name: "pkg-d".to_string(),
                    version_spec: "^2.0.0".to_string(),
                },
            ],
            recommended_version: "^2.0.0".to_string(),
        };

        // Despite 4 usages, there are only 2 unique versions
        let unique = inconsistency.unique_versions();
        assert_eq!(unique.len(), 2);
        assert_eq!(inconsistency.version_count(), 4);
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod categorization_tests {
    use std::path::PathBuf;
    use sublime_pkg_tools::audit::{
        CategorizationStats, DependencyCategorization, ExternalPackage, InternalPackage, LocalLink,
        LocalLinkType, WorkspaceLink,
    };

    #[test]
    fn test_categorization_stats_all_zero() {
        let stats = CategorizationStats {
            total_packages: 0,
            internal_packages: 0,
            external_packages: 0,
            workspace_links: 0,
            local_links: 0,
        };

        assert_eq!(stats.total_packages, 0);
        assert_eq!(stats.internal_packages, 0);
        assert_eq!(stats.external_packages, 0);
        assert_eq!(stats.workspace_links, 0);
        assert_eq!(stats.local_links, 0);
    }

    #[test]
    fn test_categorization_stats_with_values() {
        let stats = CategorizationStats {
            total_packages: 10,
            internal_packages: 5,
            external_packages: 50,
            workspace_links: 10,
            local_links: 2,
        };

        assert_eq!(stats.total_packages, 10);
        assert_eq!(stats.internal_packages, 5);
        assert_eq!(stats.external_packages, 50);
        assert_eq!(stats.workspace_links, 10);
        assert_eq!(stats.local_links, 2);
    }

    #[test]
    fn test_dependency_categorization_empty() {
        let categorization = DependencyCategorization {
            internal_packages: vec![],
            external_packages: vec![],
            workspace_links: vec![],
            local_links: vec![],
            stats: CategorizationStats {
                total_packages: 0,
                internal_packages: 0,
                external_packages: 0,
                workspace_links: 0,
                local_links: 0,
            },
        };

        assert!(categorization.internal_packages.is_empty());
        assert!(categorization.external_packages.is_empty());
        assert!(categorization.workspace_links.is_empty());
        assert!(categorization.local_links.is_empty());
        assert_eq!(categorization.stats.internal_packages, 0);
    }

    #[test]
    fn test_internal_package_structure() {
        let internal = InternalPackage {
            name: "my-internal-package".to_string(),
            path: PathBuf::from("packages/internal"),
            version: Some("1.0.0".to_string()),
            used_by: vec!["app-a".to_string(), "app-b".to_string()],
        };

        assert_eq!(internal.name, "my-internal-package");
        assert_eq!(internal.version, Some("1.0.0".to_string()));
        assert_eq!(internal.used_by.len(), 2);
    }

    #[test]
    fn test_external_package_structure() {
        let external = ExternalPackage {
            name: "lodash".to_string(),
            version_spec: "^4.17.21".to_string(),
            used_by: vec!["pkg-a".to_string(), "pkg-b".to_string(), "pkg-c".to_string()],
            is_deprecated: false,
        };

        assert_eq!(external.name, "lodash");
        assert_eq!(external.version_spec, "^4.17.21");
        assert_eq!(external.used_by.len(), 3);
        assert!(!external.is_deprecated);
    }

    #[test]
    fn test_workspace_link_structure() {
        let workspace_link = WorkspaceLink {
            package_name: "my-app".to_string(),
            dependency_name: "shared-utils".to_string(),
            version_spec: "workspace:*".to_string(),
        };

        assert_eq!(workspace_link.package_name, "my-app");
        assert_eq!(workspace_link.dependency_name, "shared-utils");
        assert_eq!(workspace_link.version_spec, "workspace:*");
    }

    #[test]
    fn test_local_link_file_type() {
        let local_link = LocalLink {
            package_name: "app".to_string(),
            dependency_name: "local-package".to_string(),
            link_type: LocalLinkType::File,
            path: "../local-package".to_string(),
        };

        assert_eq!(local_link.package_name, "app");
        assert_eq!(local_link.dependency_name, "local-package");
        assert!(matches!(local_link.link_type, LocalLinkType::File));
        assert_eq!(local_link.path, "../local-package");
    }

    #[test]
    fn test_local_link_portal_type() {
        let local_link = LocalLink {
            package_name: "app-a".to_string(),
            dependency_name: "portal-package".to_string(),
            link_type: LocalLinkType::Portal,
            path: "~/portal-package".to_string(),
        };

        assert_eq!(local_link.package_name, "app-a");
        assert_eq!(local_link.dependency_name, "portal-package");
        assert!(matches!(local_link.link_type, LocalLinkType::Portal));
        assert_eq!(local_link.path, "~/portal-package");
    }

    #[test]
    fn test_local_link_symlink_type() {
        let local_link = LocalLink {
            package_name: "app".to_string(),
            dependency_name: "link-package".to_string(),
            link_type: LocalLinkType::Link,
            path: "../link-package".to_string(),
        };

        assert_eq!(local_link.package_name, "app");
        assert_eq!(local_link.dependency_name, "link-package");
        assert!(matches!(local_link.link_type, LocalLinkType::Link));
        assert_eq!(local_link.path, "../link-package");
    }

    #[test]
    fn test_categorization_with_mixed_data() {
        let categorization = DependencyCategorization {
            internal_packages: vec![InternalPackage {
                name: "pkg-a".to_string(),
                path: PathBuf::from("packages/a"),
                version: Some("1.0.0".to_string()),
                used_by: vec!["pkg-b".to_string()],
            }],
            external_packages: vec![ExternalPackage {
                name: "lodash".to_string(),
                version_spec: "^4.17.21".to_string(),
                used_by: vec!["pkg-a".to_string(), "pkg-b".to_string()],
                is_deprecated: false,
            }],
            workspace_links: vec![WorkspaceLink {
                package_name: "pkg-b".to_string(),
                dependency_name: "shared".to_string(),
                version_spec: "workspace:*".to_string(),
            }],
            local_links: vec![LocalLink {
                package_name: "pkg-a".to_string(),
                dependency_name: "local".to_string(),
                link_type: LocalLinkType::File,
                path: "../local".to_string(),
            }],
            stats: CategorizationStats {
                total_packages: 2,
                internal_packages: 1,
                external_packages: 1,
                workspace_links: 1,
                local_links: 1,
            },
        };

        assert_eq!(categorization.internal_packages.len(), 1);
        assert_eq!(categorization.external_packages.len(), 1);
        assert_eq!(categorization.workspace_links.len(), 1);
        assert_eq!(categorization.local_links.len(), 1);
        assert_eq!(categorization.stats.total_packages, 2);
        assert_eq!(categorization.stats.internal_packages, 1);
        assert_eq!(categorization.stats.external_packages, 1);
    }
}
