//! Unit tests for changes module

#[cfg(test)]
mod tests {
    use crate::changes::*;
    use crate::core::MonorepoPackageInfo;
    use sublime_git_tools::{GitChangedFile, GitFileStatus};
    use sublime_package_tools::{Package, PackageInfo};
    use sublime_standard_tools::monorepo::WorkspacePackage;
    use std::path::PathBuf;

    /// Helper to create test package info
    fn create_test_package_info(name: &str, version: &str, location: &str) -> MonorepoPackageInfo {
        let package = Package::new(name, version, None).expect("Failed to create package");
        let package_json_path = format!("{location}/package.json");
        
        let package_info = PackageInfo::new(
            package,
            package_json_path,
            location.to_string(),
            location.to_string(),
            serde_json::json!({
                "name": name,
                "version": version,
            }),
        );
        
        let workspace_package = WorkspacePackage {
            name: name.to_string(),
            version: version.to_string(),
            location: PathBuf::from(location),
            absolute_path: PathBuf::from(format!("/monorepo/{location}")),
            workspace_dependencies: vec![],
            workspace_dev_dependencies: vec![],
        };
        
        MonorepoPackageInfo::new(package_info, workspace_package, true)
    }

    /// Helper to create test changed file
    fn create_changed_file(path: &str, status: GitFileStatus) -> GitChangedFile {
        GitChangedFile {
            path: path.to_string(),
            status,
        }
    }

    #[test]
    fn test_change_significance_ordering() {
        assert!(ChangeSignificance::High > ChangeSignificance::Medium);
        assert!(ChangeSignificance::Medium > ChangeSignificance::Low);
    }

    #[test]
    fn test_change_significance_elevation() {
        assert_eq!(ChangeSignificance::Low.elevate(), ChangeSignificance::Medium);
        assert_eq!(ChangeSignificance::Medium.elevate(), ChangeSignificance::High);
        assert_eq!(ChangeSignificance::High.elevate(), ChangeSignificance::High);
    }

    #[test]
    fn test_package_change_type_variants() {
        let source = PackageChangeType::SourceCode;
        let deps = PackageChangeType::Dependencies;
        let config = PackageChangeType::Configuration;
        let docs = PackageChangeType::Documentation;
        let tests = PackageChangeType::Tests;
        
        assert_eq!(source, PackageChangeType::SourceCode);
        assert_ne!(source, deps);
        assert_ne!(deps, config);
        assert_ne!(config, docs);
        assert_ne!(docs, tests);
    }

    #[test]
    fn test_change_detector_creation() {
        let detector = ChangeDetector::new("/test/monorepo");
        
        // Test that detector is created successfully
        assert!(detector.engine().validate_rules().is_empty());
    }

    #[test]
    fn test_change_detector_with_custom_engine() {
        let custom_engine = ChangeDetectionEngine::new();
        let detector = ChangeDetector::with_engine("/test/path", custom_engine);
        
        // Test that detector can be created with custom engine
        assert!(detector.engine().validate_rules().is_empty());
    }

    #[test]
    fn test_change_detection_engine_validation() {
        let engine = ChangeDetectionEngine::new();
        let validation_errors = engine.validate_rules();
        
        // Default rules should be valid
        assert!(validation_errors.is_empty());
    }

    #[test]
    fn test_change_detection_rules_creation() {
        let rules = ChangeDetectionRules::default();
        
        // Test that default rules exist
        assert!(!rules.change_type_rules.is_empty());
        assert!(!rules.significance_rules.is_empty());
        assert!(!rules.version_bump_rules.is_empty());
        assert!(rules.project_overrides.is_empty());
    }

    #[test]
    fn test_file_pattern_types() {
        let glob_pattern = FilePattern {
            pattern_type: PatternType::Glob,
            pattern: "**/*.ts".to_string(),
            exclude: false,
        };
        
        let regex_pattern = FilePattern {
            pattern_type: PatternType::Regex,
            pattern: r"\.ts$".to_string(),
            exclude: false,
        };
        
        let exact_pattern = FilePattern {
            pattern_type: PatternType::Exact,
            pattern: "package.json".to_string(),
            exclude: false,
        };
        
        assert_eq!(glob_pattern.pattern, "**/*.ts");
        assert!(!glob_pattern.exclude);
        assert_eq!(regex_pattern.pattern, r"\.ts$");
        assert_eq!(exact_pattern.pattern, "package.json");
    }

    #[test]
    fn test_change_type_rule_creation() {
        let rule = ChangeTypeRule {
            name: "dependency_changes".to_string(),
            priority: 100,
            patterns: vec![FilePattern {
                pattern_type: PatternType::Exact,
                pattern: "package.json".to_string(),
                exclude: false,
            }],
            change_type: PackageChangeType::Dependencies,
            conditions: None,
        };
        
        assert_eq!(rule.name, "dependency_changes");
        assert_eq!(rule.priority, 100);
        assert_eq!(rule.change_type, PackageChangeType::Dependencies);
        assert_eq!(rule.patterns.len(), 1);
    }

    #[test]
    fn test_significance_rule_creation() {
        let rule = SignificanceRule {
            name: "public_api_changes".to_string(),
            priority: 100,
            patterns: vec![FilePattern {
                pattern_type: PatternType::Glob,
                pattern: "src/index.{ts,js}".to_string(),
                exclude: false,
            }],
            git_status: Some(vec![GitFileStatus::Modified, GitFileStatus::Added]),
            significance: ChangeSignificance::High,
            conditions: None,
        };
        
        assert_eq!(rule.significance, ChangeSignificance::High);
        assert!(rule.git_status.is_some());
        assert_eq!(rule.git_status.expect("git_status should be Some").len(), 2);
    }

    #[test]
    fn test_version_bump_rule_creation() {
        use crate::config::VersionBumpType;
        
        let rule = VersionBumpRule {
            name: "major_bump".to_string(),
            change_type: Some(PackageChangeType::SourceCode),
            significance: Some(ChangeSignificance::High),
            version_bump: VersionBumpType::Major,
            priority: 100,
        };
        
        assert_eq!(rule.version_bump, VersionBumpType::Major);
        assert_eq!(rule.change_type, Some(PackageChangeType::SourceCode));
        assert_eq!(rule.significance, Some(ChangeSignificance::High));
    }

    #[test]
    fn test_affected_packages_detection() {
        let detector = ChangeDetector::new("/test/monorepo");
        let packages = vec![
            create_test_package_info("pkg-a", "1.0.0", "packages/pkg-a"),
            create_test_package_info("pkg-b", "1.0.0", "packages/pkg-b"),
        ];
        
        let direct_changes = vec!["pkg-a".to_string()];
        let affected = detector.find_affected_packages(&direct_changes, &packages);
        
        // At minimum, directly changed package should be affected
        assert!(affected.contains("pkg-a"));
    }

    #[test]
    fn test_git_file_status_variants() {
        let modified = GitFileStatus::Modified;
        let added = GitFileStatus::Added;
        let deleted = GitFileStatus::Deleted;
        
        assert_ne!(modified, added);
        assert_ne!(added, deleted);
        assert_ne!(modified, deleted);
    }

    #[test]
    fn test_change_detector_mapping() {
        let mut detector = ChangeDetector::new("/test/monorepo");
        let packages = vec![
            create_test_package_info("test-pkg", "1.0.0", "packages/test-pkg"),
        ];
        
        let changed_files = vec![
            create_changed_file("packages/test-pkg/src/index.ts", GitFileStatus::Modified),
        ];
        
        let package_changes = detector.map_changes_to_packages(&changed_files, &packages);
        
        // Should detect changes in the package
        assert_eq!(package_changes.len(), 1);
        assert_eq!(package_changes[0].package_name, "test-pkg");
        assert_eq!(package_changes[0].changed_files.len(), 1);
    }
}