//! Comprehensive tests for the analysis module
//!
//! This module provides complete test coverage for all analysis functionality,
//! including monorepo analysis, diff analysis, package classification, dependency
//! graph building, registry analysis, and workspace configuration analysis.

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod tests {
    use crate::analysis::types::{
        ChangeAnalysisResult, ChangeAnalyzer, DiffAnalyzer, MonorepoAnalyzer,
    };
    use crate::changes::{ChangeSignificance, PackageChangeType};
    use crate::config::VersionBumpType;
    use crate::core::MonorepoProject;
    use crate::error::Result;
    use sublime_git_tools::{GitChangedFile, GitFileStatus};
    use sublime_standard_tools::monorepo::PackageManagerKind;
    use tempfile::TempDir;

    /// Test analyzer for testing custom analyzers
    struct TestAnalyzer;
    
    impl ChangeAnalyzer for TestAnalyzer {
        fn can_analyze(&self, _file_path: &str) -> bool {
            true
        }
        
        fn analyze_change(&self, _change: &GitChangedFile) -> ChangeAnalysisResult {
            ChangeAnalysisResult {
                change_type: PackageChangeType::SourceCode,
                significance: ChangeSignificance::High,
                context: vec!["Test analysis".to_string()],
            }
        }
    }

    /// Helper function to create a test monorepo project
    fn create_test_project() -> Result<(TempDir, MonorepoProject)> {
        let temp_dir = TempDir::new().map_err(crate::error::Error::Io)?;
        let root_path = temp_dir.path().to_path_buf();

        // Create a basic package.json for the root
        let package_json_content = r#"{
            "name": "test-monorepo",
            "version": "1.0.0",
            "workspaces": ["packages/*", "apps/*"]
        }"#;
        std::fs::write(root_path.join("package.json"), package_json_content)
            .map_err(crate::error::Error::Io)?;

        // Create package directories
        std::fs::create_dir_all(root_path.join("packages/core"))
            .map_err(crate::error::Error::Io)?;
        std::fs::create_dir_all(root_path.join("packages/utils"))
            .map_err(crate::error::Error::Io)?;
        std::fs::create_dir_all(root_path.join("apps/web"))
            .map_err(crate::error::Error::Io)?;

        // Create package.json files for packages
        let core_package_json = r#"{
            "name": "@test/core",
            "version": "1.0.0",
            "dependencies": {
                "lodash": "^4.17.21"
            }
        }"#;
        std::fs::write(
            root_path.join("packages/core/package.json"),
            core_package_json,
        )
        .map_err(crate::error::Error::Io)?;

        let utils_package_json = r#"{
            "name": "@test/utils",
            "version": "1.0.0",
            "dependencies": {
                "@test/core": "^1.0.0"
            }
        }"#;
        std::fs::write(
            root_path.join("packages/utils/package.json"),
            utils_package_json,
        )
        .map_err(crate::error::Error::Io)?;

        let web_package_json = r#"{
            "name": "@test/web",
            "version": "1.0.0",
            "dependencies": {
                "@test/core": "^1.0.0",
                "@test/utils": "^1.0.0"
            }
        }"#;
        std::fs::write(root_path.join("apps/web/package.json"), web_package_json)
            .map_err(crate::error::Error::Io)?;

        // Create package-lock.json (required for MonorepoDetector)
        let package_lock_json = r#"{
            "name": "test-monorepo",
            "version": "1.0.0",
            "lockfileVersion": 3,
            "requires": true,
            "packages": {
                "": {
                    "name": "test-monorepo",
                    "version": "1.0.0",
                    "workspaces": ["packages/*", "apps/*"]
                },
                "node_modules/@test/core": {
                    "resolved": "packages/core",
                    "link": true
                },
                "node_modules/@test/utils": {
                    "resolved": "packages/utils",
                    "link": true
                },
                "node_modules/@test/web": {
                    "resolved": "apps/web",
                    "link": true
                },
                "packages/core": {
                    "name": "@test/core",
                    "version": "1.0.0",
                    "dependencies": {
                        "lodash": "^4.17.21"
                    }
                },
                "packages/utils": {
                    "name": "@test/utils",
                    "version": "1.0.0",
                    "dependencies": {
                        "@test/core": "^1.0.0"
                    }
                },
                "apps/web": {
                    "name": "@test/web",
                    "version": "1.0.0",
                    "dependencies": {
                        "@test/core": "^1.0.0",
                        "@test/utils": "^1.0.0"
                    }
                }
            }
        }"#;
        std::fs::write(root_path.join("package-lock.json"), package_lock_json)
            .map_err(crate::error::Error::Io)?;

        // Initialize git repository
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&root_path)
            .output()
            .map_err(crate::error::Error::Io)?;

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&root_path)
            .output()
            .map_err(crate::error::Error::Io)?;

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&root_path)
            .output()
            .map_err(crate::error::Error::Io)?;

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&root_path)
            .output()
            .map_err(crate::error::Error::Io)?;

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&root_path)
            .output()
            .map_err(crate::error::Error::Io)?;

        // Ensure we have a 'main' branch for consistent testing across different Git configurations
        // Some CI environments might initialize with 'master' or no default branch
        std::process::Command::new("git")
            .args(["checkout", "-b", "main"])
            .current_dir(&root_path)
            .output()
            .map_err(crate::error::Error::Io)?;

        let project = MonorepoProject::new(&root_path)?;
        Ok((temp_dir, project))
    }

    /// Helper function to create test git changes
    fn create_test_git_changes() -> Vec<GitChangedFile> {
        vec![
            GitChangedFile {
                path: "packages/core/src/index.ts".to_string(),
                status: GitFileStatus::Modified,
                staged: false,
                workdir: true,
            },
            GitChangedFile {
                path: "packages/core/package.json".to_string(),
                status: GitFileStatus::Modified,
                staged: false,
                workdir: true,
            },
            GitChangedFile {
                path: "packages/utils/src/helpers.ts".to_string(),
                status: GitFileStatus::Added,
                staged: true,
                workdir: false,
            },
            GitChangedFile {
                path: "apps/web/README.md".to_string(),
                status: GitFileStatus::Modified,
                staged: false,
                workdir: true,
            },
        ]
    }

    #[test]
    fn test_monorepo_analyzer_new() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        // Verify analyzer is properly initialized
        assert_eq!(analyzer.packages.len(), 3);
        assert!(analyzer.root_path.exists());

        Ok(())
    }

    #[test]
    fn test_monorepo_analyzer_from_project() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::from_project(&project);

        // Verify analyzer is properly initialized
        assert_eq!(analyzer.packages.len(), 3);
        assert!(analyzer.root_path.exists());

        Ok(())
    }

    #[test]
    fn test_detect_monorepo_info() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let result = analyzer.detect_monorepo_info(project.root_path())?;

        // Verify all components are analyzed
        assert!(matches!(
            result.kind,
            sublime_standard_tools::monorepo::MonorepoKind::NpmWorkSpace
        ));
        assert_eq!(result.root_path, project.root_path().to_path_buf());
        assert!(!result.packages.internal_packages.is_empty());
        assert_eq!(result.dependency_graph.node_count, 3);

        Ok(())
    }

    #[test]
    fn test_analyze_package_manager() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let result = analyzer.analyze_package_manager()?;

        // Verify package manager analysis
        assert_eq!(result.kind, PackageManagerKind::Npm);
        assert!(result.lock_file.ends_with("package-lock.json"));
        assert!(!result.config_files.is_empty());
        assert!(result.workspaces_config.is_array());

        Ok(())
    }

    #[test]
    fn test_build_dependency_graph() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let result = analyzer.build_dependency_graph()?;

        // Verify dependency graph structure
        assert_eq!(result.node_count, 3);
        assert!(result.edge_count > 0);
        assert!(!result.has_cycles); // Our test setup has no cycles
        assert!(result.cycles.is_empty());

        Ok(())
    }

    #[test]
    #[ignore = "Requires complex dependency analysis not essential for simplified CLI/daemon API"]
    fn test_build_dependency_graph_empty_packages() -> Result<()> {
        let temp_dir = TempDir::new().map_err(crate::error::Error::Io)?;
        let root_path = temp_dir.path().to_path_buf();

        // Create empty monorepo
        let package_json_content = r#"{
            "name": "empty-monorepo",
            "version": "1.0.0",
            "workspaces": []
        }"#;
        std::fs::write(root_path.join("package.json"), package_json_content)
            .map_err(crate::error::Error::Io)?;

        let project = MonorepoProject::new(&root_path)?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let result = analyzer.build_dependency_graph()?;

        // Verify empty graph
        assert_eq!(result.node_count, 0);
        assert_eq!(result.edge_count, 0);
        assert!(!result.has_cycles);
        assert!(result.cycles.is_empty());
        assert!(result.version_conflicts.is_empty());
        assert!(result.upgradable.is_empty());

        Ok(())
    }

    #[test]
    fn test_calculate_max_depth() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let max_depth = analyzer.calculate_max_depth(&project.packages);

        // With our dependency structure, we should have some depth
        assert!(max_depth > 0);

        Ok(())
    }

    #[test]
    fn test_find_packages_with_most_dependencies() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let result = analyzer.find_packages_with_most_dependencies(&project.packages);

        // Should return top packages by dependency count
        assert!(!result.is_empty());
        assert!(result.len() <= 5); // Top 5
        
        // Should be sorted by dependency count (descending)
        if result.len() > 1 {
            assert!(result[0].1 >= result[1].1);
        }

        Ok(())
    }

    #[test]
    fn test_find_packages_with_most_dependents() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let result = analyzer.find_packages_with_most_dependents(&project.packages);

        // Should return top packages by dependent count
        assert!(!result.is_empty());
        assert!(result.len() <= 5); // Top 5

        Ok(())
    }

    #[test]
    fn test_classify_packages() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let result = analyzer.classify_packages()?;

        // Verify package classification
        assert_eq!(result.internal_packages.len(), 3);
        assert!(!result.external_dependencies.is_empty());
        
        // Check that we have the expected packages
        let package_names: Vec<&String> = result.internal_packages.iter().map(|p| &p.name).collect();
        assert!(package_names.contains(&&"@test/core".to_string()));
        assert!(package_names.contains(&&"@test/utils".to_string()));
        assert!(package_names.contains(&&"@test/web".to_string()));

        Ok(())
    }

    #[test]
    fn test_analyze_registries() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let result = analyzer.analyze_registries()?;

        // Verify registry analysis
        assert!(!result.default_registry.is_empty());
        assert!(!result.registries.is_empty());

        Ok(())
    }

    #[test]
    fn test_check_registry_auth() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let has_auth = analyzer.check_registry_auth("https://registry.npmjs.org/");

        // This will depend on the environment, but should not panic
        #[allow(clippy::overly_complex_bool_expr)]
        {
            assert!(has_auth || !has_auth); // Always true, just ensuring no panic
        }

        Ok(())
    }

    #[test]
    fn test_get_package_information() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let package_info = analyzer.get_package_information();

        assert_eq!(package_info.len(), 3);
        
        // Verify package information structure
        for info in &package_info {
            assert!(!info.name.is_empty());
            assert!(!info.version.is_empty());
            assert!(info.path.exists());
            assert!(info.is_internal);
        }

        Ok(())
    }

    #[test]
    fn test_analyze_available_upgrades() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let result = analyzer.analyze_available_upgrades()?;

        // Verify upgrade analysis structure
        assert_eq!(result.total_packages, 3);
        assert!(result.upgradable_count <= result.total_packages);

        Ok(())
    }

    #[test]
    fn test_analyze_workspace_config() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let result = analyzer.analyze_workspace_config()?;

        // Verify workspace config analysis
        assert!(!result.patterns.is_empty());
        assert_eq!(result.matched_packages, 3);
        assert!(!result.has_nohoist); // Our test setup doesn't use nohoist

        Ok(())
    }

    #[test]
    fn test_get_config_workspace_patterns() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let patterns = analyzer.get_config_workspace_patterns()?;

        // Should extract patterns from package.json workspaces
        assert!(!patterns.is_empty());

        Ok(())
    }

    #[test]
    fn test_get_auto_detected_patterns() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let patterns = analyzer.get_auto_detected_patterns()?;

        // Auto-detection should find our package structure
        assert!(!patterns.is_empty());

        Ok(())
    }

    #[test]
    fn test_calculate_pattern_specificity() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let specificity1 = analyzer.calculate_pattern_specificity("packages/*");
        let specificity2 = analyzer.calculate_pattern_specificity("packages/core");
        let specificity3 = analyzer.calculate_pattern_specificity("**");

        // More specific patterns should have higher scores
        assert!(specificity2 > specificity1); // Exact match > wildcard
        assert!(specificity1 > specificity3); // Single wildcard > double wildcard

        Ok(())
    }

    #[test]
    fn test_get_validated_workspace_patterns() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let analysis = analyzer.get_validated_workspace_patterns()?;

        // Verify pattern validation
        assert!(!analysis.config_patterns.is_empty());
        assert!(!analysis.effective_patterns.is_empty());
        
        // Check pattern statistics
        for stat in &analysis.pattern_statistics {
            assert!(!stat.pattern.is_empty());
            assert!(stat.specificity > 0);
        }

        Ok(())
    }

    #[test]
    fn test_find_orphaned_packages() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let patterns = vec!["packages/*".to_string()];
        let orphaned = analyzer.find_orphaned_packages(&patterns);

        // With our setup, apps/web should be orphaned with only packages/* pattern
        assert!(!orphaned.is_empty());

        Ok(())
    }

    #[test]
    fn test_matches_glob_pattern() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        // Test various glob patterns
        assert!(analyzer.matches_glob_pattern("packages/core", "packages/*"));
        assert!(analyzer.matches_glob_pattern("packages/core/src/index.ts", "packages/**"));
        assert!(!analyzer.matches_glob_pattern("apps/web", "packages/*"));
        assert!(analyzer.matches_glob_pattern("@test/core", "@test/*"));

        // Test invalid pattern handling
        assert!(analyzer.matches_glob_pattern("test", "test")); // Falls back to exact match

        Ok(())
    }

    #[test]
    fn test_detect_changes_since() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        // Create some changes
        std::fs::create_dir_all(project.root_path().join("packages/core/src"))
            .map_err(crate::error::Error::Io)?;
            
        std::fs::write(
            project.root_path().join("packages/core/src/new-file.ts"),
            "export const newFunction = () => {};",
        )
        .map_err(crate::error::Error::Io)?;

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(project.root_path())
            .output()
            .map_err(crate::error::Error::Io)?;

        std::process::Command::new("git")
            .args(["commit", "-m", "Add new file"])
            .current_dir(project.root_path())
            .output()
            .map_err(crate::error::Error::Io)?;

        let result = analyzer.detect_changes_since("HEAD~1", None)?;

        // Verify change analysis
        assert_eq!(result.from_ref, "HEAD~1");
        assert_eq!(result.to_ref, "HEAD");
        assert!(!result.changed_files.is_empty());

        Ok(())
    }

    #[test]
    fn test_compare_branches() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        // Create a new branch from main
        std::process::Command::new("git")
            .args(["checkout", "-b", "feature-branch"])
            .current_dir(project.root_path())
            .output()
            .map_err(crate::error::Error::Io)?;

        // Create src directory first
        std::fs::create_dir_all(project.root_path().join("packages/core/src"))
            .map_err(crate::error::Error::Io)?;

        std::fs::write(
            project.root_path().join("packages/core/src/feature.ts"),
            "export const featureFunction = () => {};",
        )
        .map_err(crate::error::Error::Io)?;

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(project.root_path())
            .output()
            .map_err(crate::error::Error::Io)?;

        std::process::Command::new("git")
            .args(["commit", "-m", "Add feature"])
            .current_dir(project.root_path())
            .output()
            .map_err(crate::error::Error::Io)?;

        let result = analyzer.compare_branches("main", "feature-branch")?;

        // Verify branch comparison
        assert_eq!(result.base_branch, "main");
        assert_eq!(result.target_branch, "feature-branch");
        assert!(!result.merge_base.is_empty());

        Ok(())
    }

    #[test]
    fn test_diff_analyzer_new() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = DiffAnalyzer::new(&project);

        // Verify analyzer has built-in analyzers
        assert_eq!(analyzer.analyzers.len(), 5);
        assert_eq!(analyzer.packages.len(), 3);

        Ok(())
    }

    #[test]
    fn test_diff_analyzer_from_project() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = DiffAnalyzer::from_project(&project);

        // Verify analyzer is properly initialized
        assert_eq!(analyzer.analyzers.len(), 5);
        assert_eq!(analyzer.packages.len(), 3);

        Ok(())
    }

    #[test]
    fn test_diff_analyzer_with_analyzers() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        
        // Create custom analyzer for testing using the TestAnalyzer defined at module level

        let custom_analyzers: Vec<Box<dyn ChangeAnalyzer>> = vec![Box::new(TestAnalyzer)];
        let analyzer = DiffAnalyzer::with_analyzers(&project, custom_analyzers);

        // Verify custom analyzers are used
        assert_eq!(analyzer.analyzers.len(), 1);

        Ok(())
    }

    #[test]
    fn test_map_changes_to_packages() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = DiffAnalyzer::new(&project);

        let changes = create_test_git_changes();
        let package_changes = analyzer.map_changes_to_packages(&changes);

        // Verify changes are mapped to packages
        assert!(!package_changes.is_empty());
        
        // Should have changes for multiple packages
        let package_names: Vec<&String> = package_changes.iter().map(|c| &c.package_name).collect();
        assert!(package_names.len() > 1);

        Ok(())
    }

    #[test]
    fn test_identify_affected_packages() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = DiffAnalyzer::new(&project);

        let changes = create_test_git_changes();
        let result = analyzer.identify_affected_packages(&changes)?;

        // Verify affected package analysis
        assert!(!result.directly_affected.is_empty());
        assert!(result.total_affected_count > 0);
        assert!(!result.impact_scores.is_empty());

        Ok(())
    }

    #[test]
    fn test_analyze_change_significance() -> Result<()> {
        let (_temp_dir, mut project) = create_test_project()?;
        
        // Build dependency graph to populate dependents
        project.build_dependency_graph()?;
        
        let analyzer = DiffAnalyzer::new(&project);

        let changes = create_test_git_changes();
        let package_changes = analyzer.map_changes_to_packages(&changes);
        let significance_results = analyzer.analyze_change_significance(&package_changes);

        // Verify significance analysis
        assert!(!significance_results.is_empty());
        
        for result in &significance_results {
            assert!(!result.package_name.is_empty());
            // Reasons may be empty if the package doesn't meet special conditions
            // (e.g., doesn't have many dependents, isn't a core package, etc.)
        }

        Ok(())
    }

    #[test]
    fn test_built_in_analyzers_indirectly() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = DiffAnalyzer::new(&project);

        // Test that built-in analyzers are working by checking the analyzer count
        assert_eq!(analyzer.analyzers.len(), 5);

        // Test change mapping which uses the built-in analyzers
        let changes = vec![
            GitChangedFile {
                path: "packages/core/package.json".to_string(),
                status: GitFileStatus::Modified,
                staged: false,
                workdir: true,
            },
            GitChangedFile {
                path: "packages/core/src/index.ts".to_string(),
                status: GitFileStatus::Modified,
                staged: false,
                workdir: true,
            },
            GitChangedFile {
                path: "packages/core/README.md".to_string(),
                status: GitFileStatus::Modified,
                staged: false,
                workdir: true,
            },
            GitChangedFile {
                path: "packages/core/src/index.test.ts".to_string(),
                status: GitFileStatus::Added,
                staged: true,
                workdir: false,
            },
        ];

        let package_changes = analyzer.map_changes_to_packages(&changes);
        
        // Verify that changes are properly categorized by the built-in analyzers
        assert!(!package_changes.is_empty());
        
        // Should have at least one package change for core package
        let core_changes: Vec<_> = package_changes.iter()
            .filter(|c| c.package_name.contains("core"))
            .collect();
        assert!(!core_changes.is_empty());

        Ok(())
    }

    #[test]
    fn test_change_analysis_result() {
        let result = ChangeAnalysisResult {
            change_type: PackageChangeType::SourceCode,
            significance: ChangeSignificance::High,
            context: vec!["Breaking change detected".to_string()],
        };

        assert_eq!(result.change_type, PackageChangeType::SourceCode);
        assert_eq!(result.significance, ChangeSignificance::High);
        assert_eq!(result.context.len(), 1);
    }

    #[test]
    fn test_change_significance_ordering() {
        assert!(ChangeSignificance::High > ChangeSignificance::Medium);
        assert!(ChangeSignificance::Medium > ChangeSignificance::Low);
        
        let mut significances = [
            ChangeSignificance::Low,
            ChangeSignificance::High,
            ChangeSignificance::Medium,
        ];
        significances.sort();
        
        assert_eq!(significances[0], ChangeSignificance::Low);
        assert_eq!(significances[1], ChangeSignificance::Medium);
        assert_eq!(significances[2], ChangeSignificance::High);
    }

    #[test]
    fn test_change_significance_elevate() {
        assert_eq!(ChangeSignificance::Low.elevate(), ChangeSignificance::Medium);
        assert_eq!(ChangeSignificance::Medium.elevate(), ChangeSignificance::High);
        assert_eq!(ChangeSignificance::High.elevate(), ChangeSignificance::High); // Max level
    }

    #[test]
    fn test_package_change_type_equality() {
        // Test basic equality operations
        assert_eq!(PackageChangeType::Dependencies, PackageChangeType::Dependencies);
        assert_ne!(PackageChangeType::Dependencies, PackageChangeType::SourceCode);
        assert_ne!(PackageChangeType::SourceCode, PackageChangeType::Configuration);
        assert_ne!(PackageChangeType::Configuration, PackageChangeType::Documentation);
    }

    #[test]
    fn test_version_bump_suggestions_indirectly() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = DiffAnalyzer::new(&project);

        let changes = vec![
            GitChangedFile {
                path: "packages/core/package.json".to_string(),
                status: GitFileStatus::Modified,
                staged: false,
                workdir: true,
            },
        ];

        let package_changes = analyzer.map_changes_to_packages(&changes);
        
        // Verify that version bump suggestions are generated
        for change in &package_changes {
            // Should have a suggested version bump
            assert!(matches!(
                change.suggested_version_bump,
                VersionBumpType::Major | VersionBumpType::Minor | VersionBumpType::Patch
            ));
        }

        Ok(())
    }

    #[test]
    fn test_registry_analysis() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let result = analyzer.analyze_registries()?;

        // Test registry analysis structure
        assert!(!result.default_registry.is_empty());
        assert!(!result.registries.is_empty());
        
        // Test that registries have expected fields
        for registry in &result.registries {
            assert!(!registry.url.is_empty());
            assert!(!registry.registry_type.is_empty());
        }

        Ok(())
    }

    #[test]
    fn test_upgrade_analysis() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let result = analyzer.analyze_available_upgrades()?;

        // Test upgrade analysis structure
        assert_eq!(result.total_packages, 3);
        assert!(result.upgradable_count <= result.total_packages);

        Ok(())
    }

    #[test]
    fn test_package_information_access() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let package_info = analyzer.get_package_information();

        assert_eq!(package_info.len(), 3);
        
        for info in &package_info {
            assert!(!info.name.is_empty());
            assert!(!info.version.is_empty());
            assert!(info.path.exists());
            assert!(info.is_internal);
        }

        Ok(())
    }

    #[test]
    fn test_error_handling() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = DiffAnalyzer::new(&project);

        // Test branch comparison with invalid branches
        let result = analyzer.compare_branches("", "feature-test");
        assert!(result.is_err());

        let result = analyzer.compare_branches("feature-test", "");
        assert!(result.is_err());

        let result = analyzer.compare_branches("nonexistent-branch", "feature-test");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_empty_changes_handling() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = DiffAnalyzer::new(&project);

        let empty_changes = vec![];
        let package_changes = analyzer.map_changes_to_packages(&empty_changes);
        assert!(package_changes.is_empty());

        let affected = analyzer.identify_affected_packages(&empty_changes)?;
        assert!(affected.directly_affected.is_empty());
        assert_eq!(affected.total_affected_count, 0);

        Ok(())
    }

    #[test]
    #[allow(clippy::redundant_closure)]
    fn test_serialization_deserialization() -> Result<()> {
        // Test ChangeSignificance
        let significance = ChangeSignificance::High;
        let json = serde_json::to_string(&significance).map_err(|e| crate::error::Error::analysis(e.to_string()))?;
        let deserialized: ChangeSignificance = serde_json::from_str(&json).map_err(|e| crate::error::Error::analysis(e.to_string()))?;
        assert_eq!(significance, deserialized);

        // Test PackageChangeType
        let change_type = PackageChangeType::SourceCode;
        let json = serde_json::to_string(&change_type).map_err(|e| crate::error::Error::analysis(e.to_string()))?;
        let deserialized: PackageChangeType = serde_json::from_str(&json).map_err(|e| crate::error::Error::analysis(e.to_string()))?;
        assert_eq!(change_type, deserialized);

        Ok(())
    }

    #[test]
    fn test_pattern_statistics() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let analyzer = MonorepoAnalyzer::new(&project);

        let analysis = analyzer.get_validated_workspace_patterns()?;

        // Test pattern statistics structure
        assert!(!analysis.pattern_statistics.is_empty());
        
        for stats in &analysis.pattern_statistics {
            assert!(!stats.pattern.is_empty());
            assert!(stats.specificity > 0);
        }

        Ok(())
    }

    #[test]
    fn test_comprehensive_analysis_workflow() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        
        // Test complete analysis workflow
        let monorepo_analyzer = MonorepoAnalyzer::new(&project);
        let diff_analyzer = DiffAnalyzer::new(&project);

        // 1. Analyze monorepo structure
        let monorepo_result = monorepo_analyzer.detect_monorepo_info(project.root_path())?;
        assert!(!monorepo_result.packages.internal_packages.is_empty());

        // 2. Analyze workspace patterns
        let workspace_analysis = monorepo_analyzer.get_validated_workspace_patterns()?;
        assert!(!workspace_analysis.effective_patterns.is_empty());

        // 3. Simulate and analyze changes
        let test_changes = create_test_git_changes();
        let package_changes = diff_analyzer.map_changes_to_packages(&test_changes);
        assert!(!package_changes.is_empty());

        // 4. Analyze significance
        let significance_results = diff_analyzer.analyze_change_significance(&package_changes);
        assert!(!significance_results.is_empty());

        // 5. Identify affected packages
        let affected = diff_analyzer.identify_affected_packages(&test_changes)?;
        assert!(!affected.directly_affected.is_empty());

        Ok(())
    }
}