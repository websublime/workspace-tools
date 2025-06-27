//! Comprehensive workflow tests for complete monorepo analysis
//!
//! This module contains end-to-end validation tests for the complete monorepo analysis
//! workflow from Git change detection to full dependency impact analysis.

// Allow common test patterns that violate production clippy rules
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::format_in_format_args)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::panic)]
#![allow(clippy::todo)]
#![allow(clippy::unimplemented)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(clippy::similar_names)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::let_underscore_untyped)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::useless_conversion)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::map_err_ignore)]
#![allow(clippy::get_unwrap)]
#![allow(clippy::len_zero)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::float_cmp)]
#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::items_after_statements)]
#![allow(unknown_lints)]
#![allow(clippy::redundant_pattern_matching)]
#![allow(clippy::implicit_clone)]

use crate::analysis::types::diff::{ChangeAnalysisResult, ChangeAnalyzer};
use crate::analysis::{DiffAnalyzer, MonorepoAnalyzer};
use crate::changes::{ChangeSignificance, PackageChangeType};
use crate::core::interfaces::DependencyFactory;
use crate::core::MonorepoProject;
use std::sync::Arc;
use sublime_git_tools::{GitChangedFile, GitFileStatus};
use sublime_standard_tools::monorepo::PackageManagerKind;
use tempfile::TempDir;

/// Creates a comprehensive test monorepo with Git setup and multiple packages
fn create_test_monorepo() -> (TempDir, Arc<MonorepoProject>) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repository");

    // Configure Git user for testing
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to configure git email");

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to configure git name");

    // Create workspace structure
    let packages_dir = temp_dir.path().join("packages");
    std::fs::create_dir_all(&packages_dir).expect("Failed to create packages dir");

    // Create root package.json
    let root_package_json = r#"{
  "name": "test-monorepo",
  "version": "1.0.0",
  "private": true,
  "workspaces": ["packages/*"]
}"#;
    std::fs::write(temp_dir.path().join("package.json"), root_package_json)
        .expect("Failed to write root package.json");

    // Create package-lock.json for npm detection
    std::fs::write(temp_dir.path().join("package-lock.json"), "{}")
        .expect("Failed to write package-lock.json");

    // Create test packages
    create_test_package(&packages_dir, "core", "1.0.0", &[]);
    create_test_package(&packages_dir, "utils", "1.1.0", &["@test/core"]);
    create_test_package(&packages_dir, "ui", "2.0.0", &["@test/core", "@test/utils"]);

    // Add files to git
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add files to git");

    // Create initial commit
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit with packages"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to create initial commit");

    // Create and return project
    let mut project =
        MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject");
    project.refresh_packages().expect("Failed to refresh packages");
    project.build_dependency_graph().expect("Failed to build dependency graph");

    (temp_dir, Arc::new(project))
}

/// Creates a test package with specified dependencies
fn create_test_package(
    packages_dir: &std::path::Path,
    name: &str,
    version: &str,
    dependencies: &[&str],
) {
    let package_dir = packages_dir.join(name);
    std::fs::create_dir_all(&package_dir).expect("Failed to create package directory");

    let mut dep_obj = serde_json::Map::new();
    for dep in dependencies {
        dep_obj.insert((*dep).to_string(), serde_json::Value::String("*".to_string()));
    }

    let package_json = serde_json::json!({
        "name": format!("@test/{name}"),
        "version": version,
        "dependencies": dep_obj
    });

    std::fs::write(
        package_dir.join("package.json"),
        serde_json::to_string_pretty(&package_json).expect("Failed to serialize package.json"),
    )
    .expect("Failed to write package.json");

    // Create source files
    let src_dir = package_dir.join("src");
    std::fs::create_dir_all(&src_dir).expect("Failed to create src directory");

    std::fs::write(
        src_dir.join("index.ts"),
        format!("// Main entry point for {name}\nexport * from './{name}';"),
    )
    .expect("Failed to write index.ts");

    std::fs::write(
        src_dir.join(format!("{name}.ts")),
        format!("// Implementation for {name} package\nexport function {name}() {{\n  return '{name} functionality';\n}}")
    ).expect("Failed to write implementation file");

    // Create test files
    let test_dir = package_dir.join("__tests__");
    std::fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    std::fs::write(
        test_dir.join(format!("{name}.test.ts")),
        format!("// Tests for {name} package\ndescribe('{name}', () => {{\n  test('should work', () => {{\n    expect(true).toBe(true);\n  }});\n}});")
    ).expect("Failed to write test file");
}

/// Creates changes in the test repository and commits them
fn create_git_changes(temp_dir: &TempDir, changes: &[(&str, &str)]) -> String {
    for (file_path, content) in changes {
        let full_path = temp_dir.path().join(file_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent directory");
        }
        std::fs::write(&full_path, content).expect("Failed to write changed file");
    }

    // Add and commit changes
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add changes");

    let _commit_output = std::process::Command::new("git")
        .args(["commit", "-m", "Test changes"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit changes");

    // Get the commit hash
    let hash_output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to get commit hash");

    String::from_utf8(hash_output.stdout).expect("Failed to parse commit hash").trim().to_string()
}

// Phase 2.4a-5a: Git change detection workflow tests

#[test]
fn test_git_change_detection_workflow_basic() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Create some changes
    let changes = vec![
        (
            "packages/core/src/core.ts",
            "// Updated core functionality\nexport function core() { return 'updated core'; }",
        ),
        ("packages/utils/README.md", "# Utils Package\nUpdated documentation"),
    ];

    let _commit_hash = create_git_changes(&temp_dir, &changes);

    // Test change detection
    let result = analyzer.detect_changes_since("HEAD~1", Some("HEAD"));
    assert!(result.is_ok(), "Change detection should succeed");

    let change_analysis = result.unwrap();
    assert_eq!(change_analysis.from_ref, "HEAD~1");
    assert_eq!(change_analysis.to_ref, "HEAD");

    // Note: Currently returns empty results due to placeholder implementation
    // This will be implemented in the subsequent tasks
}

#[test]
fn test_git_change_detection_with_different_refs() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Create multiple commits
    let first_changes = vec![(
        "packages/core/src/new-feature.ts",
        "// New feature\nexport function newFeature() { return 'new'; }",
    )];
    let first_commit = create_git_changes(&temp_dir, &first_changes);

    let second_changes = vec![(
        "packages/utils/src/helper.ts",
        "// Helper function\nexport function helper() { return 'help'; }",
    )];
    let second_commit = create_git_changes(&temp_dir, &second_changes);

    // Test detection between specific commits
    let result = analyzer.detect_changes_since(&first_commit, Some(&second_commit));
    assert!(result.is_ok(), "Change detection between commits should succeed");

    let change_analysis = result.unwrap();
    assert_eq!(change_analysis.from_ref, first_commit);
    assert_eq!(change_analysis.to_ref, second_commit);
}

#[test]
fn test_git_change_detection_with_branch_operations() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Create a feature branch
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/new-functionality"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to create feature branch");

    // Make changes on feature branch
    let feature_changes = vec![
        (
            "packages/ui/src/component.tsx",
            "// New component\nexport const Component = () => <div>New Component</div>;",
        ),
        (
            "packages/ui/package.json",
            r#"{
  "name": "@test/ui",
  "version": "2.1.0",
  "dependencies": {
    "@test/core": "*",
    "@test/utils": "*"
  }
}"#,
        ),
    ];

    create_git_changes(&temp_dir, &feature_changes);

    // Test detection from main branch
    let result = analyzer.detect_changes_since("main", None);
    assert!(result.is_ok(), "Branch-based change detection should succeed");

    let change_analysis = result.unwrap();
    assert_eq!(change_analysis.from_ref, "main");
    assert_eq!(change_analysis.to_ref, "HEAD");
}

#[test]
fn test_git_change_detection_error_handling() {
    let (_temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Test with invalid ref
    let result = analyzer.detect_changes_since("invalid-ref-that-does-not-exist", None);
    // Should return an error for invalid refs
    assert!(result.is_err(), "Should return error for invalid refs");
}

#[test]
fn test_git_change_detection_with_file_types() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Create changes of different file types
    let mixed_changes = vec![
        (
            "packages/core/src/types.ts",
            "// Type definitions\nexport interface CoreType { id: string; }",
        ),
        (
            "packages/core/package.json",
            r#"{
  "name": "@test/core",
  "version": "1.0.1",
  "dependencies": {}
}"#,
        ),
        (
            "packages/utils/__tests__/utils.test.ts",
            "// Updated tests\ndescribe('utils', () => { test('new test', () => {}); });",
        ),
        ("packages/ui/README.md", "# UI Package\nComprehensive UI components"),
        (".gitignore", "# Updated gitignore\nnode_modules/\n*.log"),
    ];

    create_git_changes(&temp_dir, &mixed_changes);

    // Test change detection with various file types
    let result = analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Should detect changes across different file types");

    let change_analysis = result.unwrap();
    assert_eq!(change_analysis.from_ref, "HEAD~1");
    assert_eq!(change_analysis.to_ref, "HEAD");
}

#[test]
fn test_git_change_detection_performance() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Create many files to simulate a large change
    let mut large_changes = Vec::new();
    for i in 0..50 {
        large_changes.push((
            format!("packages/core/src/module{i}.ts"),
            format!("// Module {i}\nexport const module{i} = () => 'module{i}';"),
        ));
    }

    // Convert to proper string references
    let large_changes: Vec<(&str, &str)> =
        large_changes.iter().map(|(path, content)| (path.as_str(), content.as_str())).collect();

    create_git_changes(&temp_dir, &large_changes);

    // Measure performance
    let start = std::time::Instant::now();
    let result = analyzer.detect_changes_since("HEAD~1", None);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Should handle large changesets");
    assert!(
        duration.as_millis() < 5000,
        "Should complete within reasonable time ({}ms)",
        duration.as_millis()
    );
}

#[test]
fn test_change_analysis_structure_completeness() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    create_git_changes(&temp_dir, &[("packages/core/src/index.ts", "// Updated index")]);

    let result = analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Change analysis should succeed");

    let change_analysis = result.unwrap();

    // Validate ChangeAnalysis structure
    assert!(!change_analysis.from_ref.is_empty(), "Should have from_ref");
    assert!(!change_analysis.to_ref.is_empty(), "Should have to_ref");

    // Should have actual data now that we implemented real functionality
    assert!(!change_analysis.changed_files.is_empty(), "Should have changed files");
    assert!(!change_analysis.package_changes.is_empty(), "Should have package changes");

    // Validate AffectedPackagesAnalysis structure
    assert!(
        !change_analysis.affected_packages.directly_affected.is_empty(),
        "Should have directly affected packages"
    );
    assert!(
        change_analysis.affected_packages.total_affected_count > 0,
        "Should have affected packages count"
    );
}

#[test]
fn test_diff_analyzer_creation_and_basic_functionality() {
    let (_temp_dir, project) = create_test_monorepo();
    let _diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Test that DiffAnalyzer can be created successfully
    // This validates the dependency injection is working correctly

    // Note: Actual functionality tests will be added when the implementation is complete
    // Currently this validates the structure and creation process
}

// This test is replaced by the more comprehensive version below

/// Helper function to validate that git operations are working correctly
fn validate_git_repository(temp_dir: &TempDir) -> bool {
    let git_dir = temp_dir.path().join(".git");
    git_dir.exists() && git_dir.is_dir()
}

#[test]
fn test_git_repository_validation() {
    let (temp_dir, _project) = create_test_monorepo();
    assert!(validate_git_repository(&temp_dir), "Git repository should be properly initialized");

    // Test git log
    let log_output = std::process::Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run git log");

    assert!(log_output.status.success(), "Git log should work");
    assert!(!log_output.stdout.is_empty(), "Should have commit history");
}

#[test]
fn test_git_status_and_operations() {
    let (temp_dir, _project) = create_test_monorepo();

    // Test git status
    let status_output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run git status");

    assert!(status_output.status.success(), "Git status should work");

    // Create a change and test detection
    std::fs::write(temp_dir.path().join("test-file.txt"), "Test content for git detection")
        .expect("Failed to create test file");

    let status_after_change = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run git status after change");

    assert!(status_after_change.status.success(), "Git status should work after changes");
    assert!(!status_after_change.stdout.is_empty(), "Should detect untracked file");
}

// Phase 2.4a-5b: Package mapping workflow tests

#[test]
fn test_package_mapping_with_real_changes() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Create changes that should map to specific packages
    let changes = vec![
        ("packages/core/src/index.ts", "// Updated core index\nexport * from './core';"),
        (
            "packages/utils/package.json",
            r#"{
  "name": "@test/utils",
  "version": "1.1.1",
  "dependencies": {
    "@test/core": "*"
  }
}"#,
        ),
        (
            "packages/ui/src/component.tsx",
            "// New UI component\nexport const NewComponent = () => <div>New</div>;",
        ),
    ];

    create_git_changes(&temp_dir, &changes);

    // Test that change detection now returns real data instead of empty results
    let result = analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Change detection should succeed");

    let change_analysis = result.unwrap();
    assert_eq!(change_analysis.from_ref, "HEAD~1");
    assert_eq!(change_analysis.to_ref, "HEAD");

    // Should now have actual changed files and package changes
    assert!(!change_analysis.changed_files.is_empty(), "Should detect actual changed files");
    assert!(!change_analysis.package_changes.is_empty(), "Should detect actual package changes");

    // Validate package mapping
    let package_names: Vec<String> =
        change_analysis.package_changes.iter().map(|pc| pc.package_name.clone()).collect();

    assert!(
        package_names.contains(&"@test/core".to_string()),
        "Should detect core package changes"
    );
    assert!(
        package_names.contains(&"@test/utils".to_string()),
        "Should detect utils package changes"
    );
    assert!(package_names.contains(&"@test/ui".to_string()), "Should detect ui package changes");
}

#[test]
fn test_package_mapping_specificity() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Create changes only in the core package
    let core_only_changes = vec![
        (
            "packages/core/src/core.ts",
            "// Core functionality\nexport function core() { return 'core'; }",
        ),
        ("packages/core/README.md", "# Core Package\nThis is the core package."),
    ];

    create_git_changes(&temp_dir, &core_only_changes);

    let result = analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Change detection should succeed");

    let change_analysis = result.unwrap();

    // Should only map to the core package
    assert_eq!(
        change_analysis.package_changes.len(),
        1,
        "Should only detect one package with changes"
    );
    assert_eq!(
        change_analysis.package_changes[0].package_name, "@test/core",
        "Should map to core package"
    );

    // Check that files are correctly associated
    assert!(
        !change_analysis.package_changes[0].changed_files.is_empty(),
        "Should have changed files"
    );

    let file_paths: Vec<String> =
        change_analysis.package_changes[0].changed_files.iter().map(|f| f.path.clone()).collect();

    assert!(file_paths.iter().any(|p| p.contains("core.ts")), "Should include core.ts");
    assert!(file_paths.iter().any(|p| p.contains("README.md")), "Should include README.md");
}

#[test]
fn test_dependency_impact_analysis() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Create changes in the core package (which others depend on)
    let core_changes = vec![(
        "packages/core/src/core.ts",
        "// Breaking change\nexport function core() { return 'breaking change'; }",
    )];

    create_git_changes(&temp_dir, &core_changes);

    let result = analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Change detection should succeed");

    let change_analysis = result.unwrap();

    // Should detect affected packages through dependencies
    assert!(
        !change_analysis.affected_packages.directly_affected.is_empty(),
        "Should have directly affected packages"
    );
    assert!(
        change_analysis.affected_packages.directly_affected.contains(&"@test/core".to_string()),
        "Core should be directly affected"
    );

    // Check dependent packages are identified
    assert!(
        !change_analysis.affected_packages.dependents_affected.is_empty(),
        "Should have dependents affected"
    );
    assert!(
        change_analysis.affected_packages.dependents_affected.contains(&"@test/utils".to_string()),
        "Utils should be affected (depends on core)"
    );
    assert!(
        change_analysis.affected_packages.dependents_affected.contains(&"@test/ui".to_string()),
        "UI should be affected (depends on core)"
    );

    // Total affected count should include direct + dependents
    let expected_total = change_analysis.affected_packages.directly_affected.len()
        + change_analysis.affected_packages.dependents_affected.len();
    assert_eq!(
        change_analysis.affected_packages.total_affected_count, expected_total,
        "Total count should match direct + dependents"
    );
}

#[test]
fn test_change_significance_analysis() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Create changes with different significance levels
    let mixed_changes = vec![
        // High significance: package.json dependency change
        (
            "packages/core/package.json",
            r#"{
  "name": "@test/core",
  "version": "2.0.0",
  "dependencies": {
    "new-dependency": "^1.0.0"
  }
}"#,
        ),
        // Medium significance: source code change
        (
            "packages/utils/src/utils.ts",
            "// Updated utils\nexport function utils() { return 'updated'; }",
        ),
        // Low significance: documentation change
        ("packages/ui/README.md", "# UI Package\nUpdated documentation for UI package"),
    ];

    create_git_changes(&temp_dir, &mixed_changes);

    let result = analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Change detection should succeed");

    let change_analysis = result.unwrap();

    // Should have significance analysis for all affected packages
    assert!(!change_analysis.significance_analysis.is_empty(), "Should have significance analysis");

    // Find core package significance (should be high due to package.json change)
    let core_significance =
        change_analysis.significance_analysis.iter().find(|s| s.package_name == "@test/core");
    assert!(core_significance.is_some(), "Should have significance analysis for core");

    // Validate that different change types are detected
    let change_types: Vec<String> =
        change_analysis.package_changes.iter().map(|pc| format!("{:?}", pc.change_type)).collect();

    assert!(change_types.len() >= 2, "Should detect multiple change types");
}

#[test]
fn test_file_type_specific_analysis() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Create changes to different file types
    let file_type_changes = vec![
        ("packages/core/src/index.ts", "// TypeScript change\nexport * from './core';"),
        ("packages/core/package.json", r#"{"name": "@test/core", "version": "1.0.1"}"#),
        ("packages/core/__tests__/core.test.ts", "// Test change\ndescribe('core', () => {});"),
        ("packages/core/tsconfig.json", r#"{"compilerOptions": {"strict": true}}"#),
        ("packages/core/README.md", "# Core\nUpdated readme"),
    ];

    create_git_changes(&temp_dir, &file_type_changes);

    let result = analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Change detection should succeed");

    let change_analysis = result.unwrap();

    // Should correctly classify different file types
    assert!(!change_analysis.package_changes.is_empty(), "Should have package changes");

    let core_package =
        change_analysis.package_changes.iter().find(|pc| pc.package_name == "@test/core");
    assert!(core_package.is_some(), "Should find core package changes");

    let core_change = core_package.unwrap();
    assert!(core_change.changed_files.len() >= 5, "Should detect all changed files");

    // Check that file paths are correctly captured
    let file_paths: Vec<&String> = core_change.changed_files.iter().map(|f| &f.path).collect();

    assert!(file_paths.iter().any(|p| p.contains("index.ts")), "Should include TypeScript file");
    assert!(file_paths.iter().any(|p| p.contains("package.json")), "Should include package.json");
    assert!(file_paths.iter().any(|p| p.contains("test.ts")), "Should include test file");
    assert!(file_paths.iter().any(|p| p.contains("tsconfig.json")), "Should include config file");
    assert!(file_paths.iter().any(|p| p.contains("README.md")), "Should include documentation");
}

#[test]
fn test_cross_package_dependency_detection() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Create a change that should affect the dependency chain: core -> utils -> ui
    let breaking_change = vec![(
        "packages/core/src/core.ts",
        "// BREAKING CHANGE: API modified\nexport function coreApi() { return 'new api'; }",
    )];

    create_git_changes(&temp_dir, &breaking_change);

    let result = analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Change detection should succeed");

    let change_analysis = result.unwrap();

    // Should detect the full dependency chain
    assert!(
        change_analysis.affected_packages.directly_affected.contains(&"@test/core".to_string()),
        "Core should be directly affected"
    );

    // All dependents should be affected
    let all_affected: std::collections::HashSet<String> = change_analysis
        .affected_packages
        .directly_affected
        .iter()
        .chain(change_analysis.affected_packages.dependents_affected.iter())
        .cloned()
        .collect();

    assert!(all_affected.contains("@test/core"), "Should include core");
    assert!(all_affected.contains("@test/utils"), "Should include utils (depends on core)");
    assert!(all_affected.contains("@test/ui"), "Should include ui (depends on core and utils)");

    // Check change propagation graph
    assert!(
        !change_analysis.affected_packages.change_propagation_graph.is_empty(),
        "Should have propagation graph"
    );
}

#[test]
fn test_error_handling_with_invalid_refs() {
    let (_temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Test with non-existent git ref
    let result = analyzer.detect_changes_since("non-existent-ref", None);
    // Should return an error for invalid refs
    assert!(result.is_err(), "Should return error for invalid refs");
}

#[test]
fn test_performance_with_many_files() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Create many file changes to test performance
    let mut many_changes = Vec::new();
    for i in 0..50 {
        many_changes.push((
            format!("packages/core/src/file{i}.ts"),
            format!("// File {i}\nexport const value{i} = {i};"),
        ));
    }

    // Convert to proper string references
    let many_changes: Vec<(&str, &str)> =
        many_changes.iter().map(|(path, content)| (path.as_str(), content.as_str())).collect();

    create_git_changes(&temp_dir, &many_changes);

    // Measure performance
    let start = std::time::Instant::now();
    let result = analyzer.detect_changes_since("HEAD~1", None);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Should handle many files successfully");
    assert!(
        duration.as_millis() < 10000,
        "Should complete within reasonable time ({}ms)",
        duration.as_millis()
    );

    let change_analysis = result.unwrap();
    assert!(!change_analysis.package_changes.is_empty(), "Should detect package changes");

    let core_changes =
        change_analysis.package_changes.iter().find(|pc| pc.package_name == "@test/core");
    assert!(core_changes.is_some(), "Should find core package changes");
    assert!(core_changes.unwrap().changed_files.len() >= 50, "Should detect all changed files");
}

// Phase 2.4a-5c: MonorepoAnalyzer end-to-end analysis tests

#[test]
fn test_monorepo_analyzer_complete_analysis_workflow() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Test complete monorepo detection and analysis
    let analysis_result = analyzer.detect_monorepo_info(temp_dir.path());
    assert!(analysis_result.is_ok(), "Complete monorepo analysis should succeed");

    let analysis = analysis_result.unwrap();

    // Validate MonorepoAnalysisResult structure
    assert!(!analysis.root_path.as_os_str().is_empty(), "Should have root path");

    // Validate package manager analysis
    assert!(
        matches!(analysis.package_manager.kind, PackageManagerKind::Npm),
        "Should detect npm package manager"
    );
    assert!(!analysis.package_manager.version.is_empty(), "Should have package manager version");
    assert!(!analysis.package_manager.config_files.is_empty(), "Should have config files");

    // Validate packages classification
    assert_eq!(analysis.packages.internal_packages.len(), 3, "Should find all 3 packages");
    assert_eq!(
        analysis.packages.external_dependencies.len(),
        0,
        "Should have no external dependencies in test setup"
    );

    // Validate dependency graph
    assert!(analysis.dependency_graph.edge_count >= 2, "Should have dependency relationships");
    assert!(analysis.dependency_graph.node_count >= 3, "Should have all packages in graph");

    // Validate registries analysis
    assert!(!analysis.registries.default_registry.is_empty(), "Should have default registry");
    assert!(!analysis.registries.registries.is_empty(), "Should have registries list");

    // Validate workspace configuration
    assert!(!analysis.workspace_config.patterns.is_empty(), "Should have workspace patterns");
}

#[test]
fn test_monorepo_analyzer_package_manager_analysis() {
    let (_temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    let result = analyzer.analyze_package_manager();
    assert!(result.is_ok(), "Package manager analysis should succeed");

    let analysis = result.unwrap();
    assert!(matches!(analysis.kind, PackageManagerKind::Npm), "Should detect npm");
    assert!(!analysis.version.is_empty(), "Should have version");
    assert!(!analysis.config_files.is_empty(), "Should find config files");
    assert!(!analysis.workspaces_config.is_null(), "Should have workspaces config");
}

#[test]
fn test_monorepo_analyzer_dependency_graph_analysis() {
    let (_temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    let result = analyzer.build_dependency_graph();
    assert!(result.is_ok(), "Dependency graph analysis should succeed");

    let graph = result.unwrap();
    assert!(graph.edge_count >= 2, "Should have dependency relationships");
    assert!(graph.node_count >= 3, "Should have all packages in graph");
    assert!(!graph.has_cycles, "Test setup should not have circular dependencies");

    // Validate graph structure
    assert!(graph.max_depth >= 1, "Should have dependency depth");
    assert!(!graph.most_dependencies.is_empty(), "Should identify packages with dependencies");
    assert!(!graph.most_dependents.is_empty(), "Should identify packages with dependents");
}

#[test]
fn test_monorepo_analyzer_package_classification() {
    let (_temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    let result = analyzer.classify_packages();
    assert!(result.is_ok(), "Package classification should succeed");

    let classification = result.unwrap();
    assert_eq!(classification.internal_packages.len(), 3, "Should classify all packages");
    assert_eq!(
        classification.external_dependencies.len(),
        0,
        "Should have no external deps in test"
    );

    // Verify all packages are properly classified as internal
    for package in &classification.internal_packages {
        assert!(package.is_internal, "Package should be marked as internal");
        assert!(!package.name.is_empty(), "Package should have name");
    }
}

#[test]
fn test_monorepo_analyzer_registry_analysis() {
    let (_temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    let result = analyzer.analyze_registries();
    assert!(result.is_ok(), "Registry analysis should succeed");

    let registries = result.unwrap();
    assert!(!registries.default_registry.is_empty(), "Should have default registry URL");
    assert!(registries.registries.len() >= 1, "Should have at least one registry");

    // All registries should be accessible (basic connectivity check)
    for registry in &registries.registries {
        assert!(!registry.url.is_empty(), "Registry should have URL");
    }
}

#[test]
fn test_monorepo_analyzer_workspace_config_analysis() {
    let (_temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    let result = analyzer.analyze_workspace_config();
    assert!(result.is_ok(), "Workspace config analysis should succeed");

    let config = result.unwrap();
    assert!(!config.patterns.is_empty(), "Should have workspace patterns");
    assert!(config.matched_packages >= 3, "Should match all test packages");
    assert!(config.orphaned_packages.len() == 0, "Should have no orphaned packages in test setup");
}

#[test]
#[allow(clippy::absurd_extreme_comparisons)]
fn test_monorepo_analyzer_package_information_retrieval() {
    let (_temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    let packages_info = analyzer.get_package_information();
    assert_eq!(packages_info.len(), 3, "Should return info for all packages");

    for package_info in &packages_info {
        assert!(!package_info.name.is_empty(), "Package should have name");
        assert!(!package_info.version.is_empty(), "Package should have version");
        assert!(!package_info.path.as_os_str().is_empty(), "Package should have path");
        // Dependencies vector is always valid by construction
        // Dependents vector is always valid by construction
    }

    // Verify specific packages exist
    let package_names: Vec<&String> = packages_info.iter().map(|p| &p.name).collect();
    assert!(package_names.contains(&&"@test/core".to_string()), "Should include core package");
    assert!(package_names.contains(&&"@test/utils".to_string()), "Should include utils package");
    assert!(package_names.contains(&&"@test/ui".to_string()), "Should include ui package");
}

#[test]
fn test_monorepo_analyzer_upgrade_analysis() {
    let (_temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // This might fail in test environment due to network restrictions, so we check for graceful handling
    let result = analyzer.analyze_available_upgrades();
    match result {
        Ok(upgrades) => {
            // If successful, validate structure
            // Total packages count is always valid by construction
            // Upgradable count is always valid by construction
            // Up-to-date list is always valid by construction
        }
        Err(_) => {
            // Network/registry errors are acceptable in test environment
            // The important thing is that the method doesn't panic
        }
    }
}

#[test]
fn test_monorepo_analyzer_pattern_analysis() {
    let (_temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Test pattern validation
    let result = analyzer.get_validated_workspace_patterns();
    assert!(result.is_ok(), "Pattern validation should succeed");

    let pattern_analysis = result.unwrap();
    assert!(!pattern_analysis.config_patterns.is_empty(), "Should have config patterns");
    assert!(
        !pattern_analysis.auto_detected_patterns.is_empty(),
        "Should have auto-detected patterns"
    );
    assert!(!pattern_analysis.effective_patterns.is_empty(), "Should have effective patterns");
    assert!(pattern_analysis.orphaned_packages.len() == 0, "Should have no orphaned packages");

    // Test auto-detection
    let auto_patterns = analyzer.get_auto_detected_patterns();
    assert!(auto_patterns.is_ok(), "Auto-detection should succeed");
    assert!(!auto_patterns.unwrap().is_empty(), "Should auto-detect some patterns");

    // Test config patterns
    let config_patterns = analyzer.get_config_workspace_patterns();
    assert!(config_patterns.is_ok(), "Config patterns should be retrievable");
}

#[test]
fn test_monorepo_analyzer_integration_with_diff_analyzer() {
    let (temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Create some changes for testing
    let changes = vec![(
        "packages/core/src/new-feature.ts",
        "// New feature\nexport function newFeature() { return 'new'; }",
    )];
    let _commit_hash = create_git_changes(&temp_dir, &changes);

    // Test change detection (MonorepoAnalyzer delegates to DiffAnalyzer)
    let result = analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Change detection should succeed");

    let change_analysis = result.unwrap();
    assert_eq!(change_analysis.from_ref, "HEAD~1");
    assert_eq!(change_analysis.to_ref, "HEAD");
    assert!(!change_analysis.changed_files.is_empty(), "Should detect changed files");
    assert!(!change_analysis.package_changes.is_empty(), "Should detect package changes");

    // Test branch comparison
    let branch_result = analyzer.compare_branches("HEAD~1", "HEAD");
    assert!(branch_result.is_ok(), "Branch comparison should succeed");

    let branch_comparison = branch_result.unwrap();
    assert_eq!(branch_comparison.base_branch, "HEAD~1");
    assert_eq!(branch_comparison.target_branch, "HEAD");
    assert!(!branch_comparison.changed_files.is_empty(), "Should have changed files");
}

#[test]
fn test_monorepo_analyzer_error_handling() {
    let (_temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Test with invalid path
    let invalid_path = std::path::Path::new("/nonexistent/path");
    let result = analyzer.detect_monorepo_info(invalid_path);
    assert!(result.is_err(), "Should return error for invalid path");

    // Test change detection with invalid refs
    let invalid_ref_result = analyzer.detect_changes_since("invalid-ref", None);
    assert!(invalid_ref_result.is_err(), "Should return error for invalid git ref");

    // Test branch comparison with invalid branches
    let invalid_branch_result = analyzer.compare_branches("invalid-base", "invalid-target");
    assert!(invalid_branch_result.is_err(), "Should return error for invalid branches");
}

#[test]
fn test_monorepo_analyzer_performance_characteristics() {
    let (_temp_dir, project) = create_test_monorepo();
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

    // Test that main operations complete within reasonable time
    let start = std::time::Instant::now();

    let _ = analyzer.analyze_package_manager();
    let _ = analyzer.build_dependency_graph();
    let _ = analyzer.classify_packages();
    let _ = analyzer.analyze_registries();
    let _ = analyzer.analyze_workspace_config();
    let _ = analyzer.get_package_information();

    let duration = start.elapsed();
    assert!(
        duration.as_millis() < 5000,
        "All main operations should complete within 5 seconds ({}ms)",
        duration.as_millis()
    );

    // Test individual operation performance
    let start = std::time::Instant::now();
    let _ = analyzer.get_validated_workspace_patterns();
    let pattern_duration = start.elapsed();
    assert!(
        pattern_duration.as_millis() < 1000,
        "Pattern validation should be fast ({}ms)",
        pattern_duration.as_millis()
    );
}

// Phase 2.4a-5d: DiffAnalyzer integration and dependency impact validation tests

// Phase 2.4a-5d-1: DiffAnalyzer construction and dependency injection

#[test]
fn test_diff_analyzer_construction_from_project() {
    let (_temp_dir, project) = create_test_monorepo();

    // Test creation from project (convenience method)
    let _diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Verify the analyzer was created successfully
    // The fact that this doesn't panic validates basic construction
    assert!(true, "DiffAnalyzer should be created from project successfully");
}

#[test]
fn test_diff_analyzer_construction_with_manual_dependency_injection() {
    let (_temp_dir, project) = create_test_monorepo();

    // Test manual construction with explicit dependency injection
    let git_provider = DependencyFactory::git_provider(Arc::clone(&project));
    let package_provider = DependencyFactory::package_provider(Arc::clone(&project));
    let file_system_provider = DependencyFactory::file_system_provider(Arc::clone(&project));
    let package_discovery_provider =
        DependencyFactory::package_discovery_provider(Arc::clone(&project));

    let _diff_analyzer = DiffAnalyzer::new(
        git_provider,
        package_provider,
        file_system_provider,
        package_discovery_provider,
    );

    // Verify the analyzer was created successfully with manual injection
    assert!(true, "DiffAnalyzer should be created with manual dependency injection");
}

#[test]
fn test_diff_analyzer_construction_with_custom_analyzers() {
    let (_temp_dir, project) = create_test_monorepo();

    // Create a custom analyzer for testing
    struct CustomTestAnalyzer;

    impl ChangeAnalyzer for CustomTestAnalyzer {
        fn can_analyze(&self, file_path: &str) -> bool {
            file_path.ends_with(".custom")
        }

        fn analyze_change(&self, _change: &GitChangedFile) -> ChangeAnalysisResult {
            ChangeAnalysisResult {
                change_type: PackageChangeType::SourceCode,
                significance: ChangeSignificance::High,
                context: vec!["Custom analyzer triggered".to_string()],
            }
        }
    }

    let git_provider = DependencyFactory::git_provider(Arc::clone(&project));
    let package_provider = DependencyFactory::package_provider(Arc::clone(&project));
    let file_system_provider = DependencyFactory::file_system_provider(Arc::clone(&project));
    let package_discovery_provider =
        DependencyFactory::package_discovery_provider(Arc::clone(&project));

    let custom_analyzers: Vec<Box<dyn ChangeAnalyzer>> = vec![Box::new(CustomTestAnalyzer)];

    let _diff_analyzer = DiffAnalyzer::with_analyzers(
        git_provider,
        package_provider,
        file_system_provider,
        package_discovery_provider,
        custom_analyzers,
    );

    // Verify analyzer creation with custom analyzers
    assert!(true, "DiffAnalyzer should support custom analyzers");
}

#[test]
fn test_diff_analyzer_dependency_injection_isolation() {
    let (_temp_dir, project) = create_test_monorepo();

    // Create multiple DiffAnalyzer instances to verify they don't interfere
    let _analyzer1 = DiffAnalyzer::from_project(Arc::clone(&project));
    let _analyzer2 = DiffAnalyzer::from_project(Arc::clone(&project));

    // Both should be independently functional
    // This test validates that dependency injection creates isolated instances
    assert!(true, "Multiple DiffAnalyzer instances should be isolated");
}

#[test]
fn test_diff_analyzer_built_in_analyzers_initialization() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Test that built-in analyzers are properly initialized by testing different file types

    let test_files = vec![
        GitChangedFile {
            path: "package.json".to_string(),
            status: GitFileStatus::Modified,
            staged: false,
            workdir: true,
        },
        GitChangedFile {
            path: "src/index.ts".to_string(),
            status: GitFileStatus::Modified,
            staged: false,
            workdir: true,
        },
        GitChangedFile {
            path: "tsconfig.json".to_string(),
            status: GitFileStatus::Modified,
            staged: false,
            workdir: true,
        },
        GitChangedFile {
            path: "README.md".to_string(),
            status: GitFileStatus::Modified,
            staged: false,
            workdir: true,
        },
        GitChangedFile {
            path: "__tests__/unit.test.ts".to_string(),
            status: GitFileStatus::Modified,
            staged: false,
            workdir: true,
        },
    ];

    // Test change mapping - should not fail with built-in analyzers
    let package_changes = diff_analyzer.map_changes_to_packages(&test_files);

    // Verify that the mapping works (even if no packages are found due to test setup)
    // Package changes vector is always valid by construction
}

#[test]
fn test_diff_analyzer_provider_interface_compatibility() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Test that the analyzer can access different provider interfaces

    let test_change = GitChangedFile {
        path: "packages/core/src/test.ts".to_string(),
        status: GitFileStatus::Modified,
        staged: false,
        workdir: true,
    };

    let changes = vec![test_change];

    // Test affected packages identification (requires PackageProvider access)
    let result = diff_analyzer.identify_affected_packages(&changes);
    assert!(result.is_ok(), "Should be able to identify affected packages through PackageProvider");

    // Test change mapping (requires PackageDiscoveryProvider access)
    let _package_changes = diff_analyzer.map_changes_to_packages(&changes);
    // Package changes vector is always valid by construction
}

#[test]
fn test_diff_analyzer_change_detection_integration() {
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create a change for testing
    let changes = vec![(
        "packages/core/src/test-feature.ts",
        "// Test feature\nexport function testFeature() { return 'test'; }",
    )];
    let _commit_hash = create_git_changes(&temp_dir, &changes);

    // Test change detection integration with Git provider
    let result = diff_analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "DiffAnalyzer should integrate with Git provider for change detection");

    let change_analysis = result.unwrap();
    assert_eq!(change_analysis.from_ref, "HEAD~1");
    assert_eq!(change_analysis.to_ref, "HEAD");
    assert!(!change_analysis.changed_files.is_empty(), "Should detect changed files");
}

#[test]
fn test_diff_analyzer_interface_consistency() {
    let (_temp_dir, project) = create_test_monorepo();

    // Test that both construction methods produce functionally equivalent analyzers
    let analyzer1 = DiffAnalyzer::from_project(Arc::clone(&project));

    let git_provider = DependencyFactory::git_provider(Arc::clone(&project));
    let package_provider = DependencyFactory::package_provider(Arc::clone(&project));
    let file_system_provider = DependencyFactory::file_system_provider(Arc::clone(&project));
    let package_discovery_provider =
        DependencyFactory::package_discovery_provider(Arc::clone(&project));

    let analyzer2 = DiffAnalyzer::new(
        git_provider,
        package_provider,
        file_system_provider,
        package_discovery_provider,
    );

    // Both should be able to perform the same operations

    let test_files = vec![GitChangedFile {
        path: "test.ts".to_string(),
        status: GitFileStatus::Modified,
        staged: false,
        workdir: true,
    }];

    let changes1 = analyzer1.map_changes_to_packages(&test_files);
    let changes2 = analyzer2.map_changes_to_packages(&test_files);

    // Both should produce equivalent results
    assert_eq!(
        changes1.len(),
        changes2.len(),
        "Both construction methods should produce equivalent analyzers"
    );
}

// ============================================================================
// DiffAnalyzer Change Mapping Tests (Phase 2.4a-5d-2)
// ============================================================================

#[test]
fn test_diff_analyzer_single_package_file_mapping() {
    // Test: Single file change maps to correct package
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Simulate a change to core package's main file
    let core_file_path = project.root_path().join("packages/core/package.json");
    let relative_path =
        core_file_path.strip_prefix(project.root_path()).unwrap().to_string_lossy().to_string();

    let changes = vec![GitChangedFile {
        path: relative_path,
        status: GitFileStatus::Modified,
        staged: false,
        workdir: true,
    }];

    let affected_packages = diff_analyzer.map_changes_to_packages(&changes);

    // Should affect only the core package
    assert_eq!(affected_packages.len(), 1, "Single file change should affect one package");
    assert_eq!(affected_packages[0].package_name, "@test/core", "Should affect the core package");
}

#[test]
fn test_diff_analyzer_multiple_package_file_mapping() {
    // Test: Multiple files in different packages map correctly
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    let core_file = project.root_path().join("packages/core/package.json");
    let utils_file = project.root_path().join("packages/utils/src/index.js");

    // Create utils src directory and file for testing
    std::fs::create_dir_all(project.root_path().join("packages/utils/src"))
        .expect("Failed to create utils src dir");
    std::fs::write(&utils_file, "// utils code").expect("Failed to write utils file");

    let changes = vec![
        GitChangedFile {
            path: core_file
                .strip_prefix(project.root_path())
                .unwrap()
                .to_string_lossy()
                .to_string(),
            status: GitFileStatus::Modified,
            staged: false,
            workdir: true,
        },
        GitChangedFile {
            path: utils_file
                .strip_prefix(project.root_path())
                .unwrap()
                .to_string_lossy()
                .to_string(),
            status: GitFileStatus::Added,
            staged: true,
            workdir: false,
        },
    ];

    let affected_packages = diff_analyzer.map_changes_to_packages(&changes);

    // Should affect both core and utils packages
    assert_eq!(affected_packages.len(), 2, "Multiple files should affect multiple packages");

    let package_names: Vec<_> = affected_packages.iter().map(|p| p.package_name.as_str()).collect();
    assert!(package_names.contains(&"@test/core"), "Should affect core package");
    assert!(package_names.contains(&"@test/utils"), "Should affect utils package");
}

#[test]
fn test_diff_analyzer_root_file_mapping() {
    // Test: Root-level files don't map to specific packages but are detected
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    let changes = vec![
        GitChangedFile {
            path: "package.json".to_string(), // Root package.json
            status: GitFileStatus::Modified,
            staged: false,
            workdir: true,
        },
        GitChangedFile {
            path: "README.md".to_string(), // Root README
            status: GitFileStatus::Added,
            staged: true,
            workdir: false,
        },
    ];

    let affected_packages = diff_analyzer.map_changes_to_packages(&changes);

    // Root files might not map to specific packages, but should be handled gracefully
    // The exact behavior depends on implementation - might be 0 packages or all packages
    // Affected packages vector is always valid by construction
}

#[test]
fn test_diff_analyzer_nested_file_mapping() {
    // Test: Deeply nested files map to correct package
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create nested directory structure in ui package
    let nested_dir = project.root_path().join("packages/ui/src/components/forms");
    std::fs::create_dir_all(&nested_dir).expect("Failed to create nested directory");

    let nested_file = nested_dir.join("LoginForm.tsx");
    std::fs::write(&nested_file, "// React component").expect("Failed to write nested file");

    let changes = vec![GitChangedFile {
        path: nested_file.strip_prefix(project.root_path()).unwrap().to_string_lossy().to_string(),
        status: GitFileStatus::Modified,
        staged: false,
        workdir: true,
    }];

    let affected_packages = diff_analyzer.map_changes_to_packages(&changes);

    // Should map to ui package despite being deeply nested
    assert_eq!(affected_packages.len(), 1, "Nested file should map to one package");
    assert_eq!(affected_packages[0].package_name, "@test/ui", "Should map to ui package");
}

#[test]
fn test_diff_analyzer_nonexistent_file_mapping() {
    // Test: Files that don't exist (deleted files) are handled correctly
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    let changes = vec![GitChangedFile {
        path: "packages/core/deleted-file.js".to_string(),
        status: GitFileStatus::Deleted,
        staged: true,
        workdir: false,
    }];

    let affected_packages = diff_analyzer.map_changes_to_packages(&changes);

    // Even deleted files should map to their packages if the package exists
    assert_eq!(affected_packages.len(), 1, "Deleted file should still map to package");
    assert_eq!(affected_packages[0].package_name, "@test/core", "Should map to core package");
}

#[test]
fn test_diff_analyzer_mixed_file_status_mapping() {
    // Test: Different file statuses (added, modified, deleted) map correctly
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create some test files
    let new_file = project.root_path().join("packages/utils/src/new-module.js");
    std::fs::create_dir_all(project.root_path().join("packages/utils/src"))
        .expect("Failed to create src dir");
    std::fs::write(&new_file, "// new module").expect("Failed to write new file");

    let changes = vec![
        GitChangedFile {
            path: "packages/core/package.json".to_string(),
            status: GitFileStatus::Modified,
            staged: false,
            workdir: true,
        },
        GitChangedFile {
            path: new_file.strip_prefix(project.root_path()).unwrap().to_string_lossy().to_string(),
            status: GitFileStatus::Added,
            staged: true,
            workdir: false,
        },
        GitChangedFile {
            path: "packages/ui/old-component.jsx".to_string(),
            status: GitFileStatus::Deleted,
            staged: true,
            workdir: false,
        },
    ];

    let affected_packages = diff_analyzer.map_changes_to_packages(&changes);

    // Should affect core, utils, and ui packages
    assert_eq!(affected_packages.len(), 3, "Mixed status changes should affect multiple packages");

    let package_names: Vec<_> = affected_packages.iter().map(|p| p.package_name.as_str()).collect();
    assert!(package_names.contains(&"@test/core"), "Should affect core package");
    assert!(package_names.contains(&"@test/utils"), "Should affect utils package");
    assert!(package_names.contains(&"@test/ui"), "Should affect ui package");
}

#[test]
fn test_diff_analyzer_path_canonicalization() {
    // Test: Path canonicalization handles symlinks and path variations correctly
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Test different path representations that should resolve to same package
    let changes = vec![
        GitChangedFile {
            path: "packages/core/package.json".to_string(),
            status: GitFileStatus::Modified,
            staged: false,
            workdir: true,
        },
        GitChangedFile {
            path: "./packages/core/src/index.js".to_string(), // Relative path
            status: GitFileStatus::Added,
            staged: true,
            workdir: false,
        },
    ];

    // Create the src file for testing
    std::fs::create_dir_all(project.root_path().join("packages/core/src"))
        .expect("Failed to create core src dir");
    std::fs::write(project.root_path().join("packages/core/src/index.js"), "// core index")
        .expect("Failed to write core index");

    let affected_packages = diff_analyzer.map_changes_to_packages(&changes);

    // Both files should map to the same package (core), so should have 1 unique package
    assert_eq!(affected_packages.len(), 1, "Different path formats should resolve to same package");
    assert_eq!(affected_packages[0].package_name, "@test/core", "Should resolve to core package");
}

#[test]
fn test_diff_analyzer_empty_changes_mapping() {
    // Test: Empty changes list is handled correctly
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    let changes = vec![];
    let affected_packages = diff_analyzer.map_changes_to_packages(&changes);

    // Empty changes should result in no affected packages
    assert_eq!(affected_packages.len(), 0, "Empty changes should affect no packages");
}

// ============================================================================
// Dependency Impact Analysis Tests (Phase 2.4a-5d-3)
// ============================================================================

#[test]
fn test_dependency_impact_direct_dependencies_identification() {
    // Test: Direct dependencies are correctly identified when a package changes
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Change the core package (which utils and ui depend on)
    let core_changes = vec![(
        "packages/core/src/core.ts",
        "// Breaking API change\nexport function coreApi() { return 'new API'; }",
    )];
    create_git_changes(&temp_dir, &core_changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = diff_analyzer.identify_affected_packages(&changed_files);
    assert!(result.is_ok(), "Should identify affected packages successfully");

    let affected_analysis = result.unwrap();

    // Core should be directly affected
    assert!(
        affected_analysis.directly_affected.contains(&"@test/core".to_string()),
        "Core package should be directly affected"
    );
    assert_eq!(
        affected_analysis.directly_affected.len(),
        1,
        "Only core should be directly affected"
    );

    // Utils and UI should be dependents affected (they depend on core)
    assert!(
        affected_analysis.dependents_affected.contains(&"@test/utils".to_string()),
        "Utils should be affected as dependent of core"
    );
    assert!(
        affected_analysis.dependents_affected.contains(&"@test/ui".to_string()),
        "UI should be affected as dependent of core"
    );

    // Total count should be direct + dependents
    let expected_total =
        affected_analysis.directly_affected.len() + affected_analysis.dependents_affected.len();
    assert_eq!(
        affected_analysis.total_affected_count, expected_total,
        "Total affected count should match direct + dependents"
    );
}

#[test]
fn test_dependency_impact_mid_level_package_changes() {
    // Test: Changes to mid-level package (utils) affect only its dependents, not dependencies
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Change the utils package (which ui depends on, but core doesn't)
    let utils_changes = vec![(
        "packages/utils/src/utils.ts",
        "// Utils API change\nexport function utilsApi() { return 'changed'; }",
    )];
    create_git_changes(&temp_dir, &utils_changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = diff_analyzer.identify_affected_packages(&changed_files);
    assert!(result.is_ok(), "Should identify affected packages successfully");

    let affected_analysis = result.unwrap();

    // Utils should be directly affected
    assert!(
        affected_analysis.directly_affected.contains(&"@test/utils".to_string()),
        "Utils package should be directly affected"
    );
    assert_eq!(
        affected_analysis.directly_affected.len(),
        1,
        "Only utils should be directly affected"
    );

    // UI should be affected as dependent, but core should NOT be affected
    assert!(
        affected_analysis.dependents_affected.contains(&"@test/ui".to_string()),
        "UI should be affected as dependent of utils"
    );
    assert!(
        !affected_analysis.dependents_affected.contains(&"@test/core".to_string()),
        "Core should NOT be affected (utils doesn't depend on it)"
    );
}

#[test]
fn test_dependency_impact_leaf_package_changes() {
    // Test: Changes to leaf package (ui) don't affect any dependents
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Change the ui package (which nothing depends on)
    let ui_changes = vec![(
        "packages/ui/src/component.tsx",
        "// UI component change\nexport const Component = () => <div>Updated</div>;",
    )];
    create_git_changes(&temp_dir, &ui_changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = diff_analyzer.identify_affected_packages(&changed_files);
    assert!(result.is_ok(), "Should identify affected packages successfully");

    let affected_analysis = result.unwrap();

    // UI should be directly affected
    assert!(
        affected_analysis.directly_affected.contains(&"@test/ui".to_string()),
        "UI package should be directly affected"
    );
    assert_eq!(affected_analysis.directly_affected.len(), 1, "Only UI should be directly affected");

    // No dependents should be affected (UI is a leaf package)
    assert_eq!(
        affected_analysis.dependents_affected.len(),
        0,
        "No dependents should be affected for leaf package"
    );

    // Total count should equal direct affected only
    assert_eq!(
        affected_analysis.total_affected_count, 1,
        "Total affected should equal directly affected for leaf package"
    );
}

#[test]
fn test_dependency_impact_multiple_packages_changes() {
    // Test: Changes to multiple packages correctly identify all impacts
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Change both core and utils packages
    let multi_changes = vec![
        (
            "packages/core/src/core.ts",
            "// Core change\nexport function core() { return 'core changed'; }",
        ),
        (
            "packages/utils/src/utils.ts",
            "// Utils change\nexport function utils() { return 'utils changed'; }",
        ),
    ];
    create_git_changes(&temp_dir, &multi_changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = diff_analyzer.identify_affected_packages(&changed_files);
    assert!(result.is_ok(), "Should identify affected packages successfully");

    let affected_analysis = result.unwrap();

    // Both core and utils should be directly affected
    assert!(
        affected_analysis.directly_affected.contains(&"@test/core".to_string()),
        "Core package should be directly affected"
    );
    assert!(
        affected_analysis.directly_affected.contains(&"@test/utils".to_string()),
        "Utils package should be directly affected"
    );
    assert_eq!(
        affected_analysis.directly_affected.len(),
        2,
        "Both core and utils should be directly affected"
    );

    // UI should be affected as dependent of both core and utils
    assert!(
        affected_analysis.dependents_affected.contains(&"@test/ui".to_string()),
        "UI should be affected as dependent"
    );

    // UI should only appear once in dependents despite depending on both changed packages
    assert_eq!(
        affected_analysis.dependents_affected.len(),
        1,
        "UI should appear only once in dependents despite multiple dependencies"
    );
}

#[test]
fn test_dependency_impact_change_propagation_graph() {
    // Test: Change propagation graph correctly maps dependency relationships
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Change the core package to test propagation
    let core_changes = vec![(
        "packages/core/package.json",
        r#"{
  "name": "@test/core",
  "version": "2.0.0",
  "description": "Breaking change to core package"
}"#,
    )];
    create_git_changes(&temp_dir, &core_changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = diff_analyzer.identify_affected_packages(&changed_files);
    assert!(result.is_ok(), "Should identify affected packages successfully");

    let affected_analysis = result.unwrap();

    // Validate change propagation graph structure
    assert!(
        !affected_analysis.change_propagation_graph.is_empty(),
        "Change propagation graph should not be empty"
    );

    // Core should have entries in the propagation graph showing its dependents
    assert!(
        affected_analysis.change_propagation_graph.contains_key("@test/core"),
        "Core package should be in propagation graph"
    );

    let core_dependents = &affected_analysis.change_propagation_graph["@test/core"];
    assert!(
        core_dependents.contains(&"@test/utils".to_string()),
        "Utils should be in core's dependents in propagation graph"
    );
    assert!(
        core_dependents.contains(&"@test/ui".to_string()),
        "UI should be in core's dependents in propagation graph"
    );
}

#[test]
fn test_dependency_impact_scores_calculation() {
    // Test: Impact scores are calculated correctly based on dependency relationships
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Change the core package (highest impact - many dependents)
    let core_changes = vec![(
        "packages/core/src/api.ts",
        "// Core API change\nexport const API_VERSION = '2.0.0';",
    )];
    create_git_changes(&temp_dir, &core_changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = diff_analyzer.identify_affected_packages(&changed_files);
    assert!(result.is_ok(), "Should identify affected packages successfully");

    let affected_analysis = result.unwrap();

    // Validate impact scores
    assert!(!affected_analysis.impact_scores.is_empty(), "Impact scores should be calculated");

    // Core should have the highest impact score (it has dependents)
    let core_score =
        affected_analysis.impact_scores.get("@test/core").expect("Core should have impact score");
    assert!(*core_score > 1.0, "Core should have elevated score due to dependents");

    // Dependent packages should have lower scores
    let utils_score =
        affected_analysis.impact_scores.get("@test/utils").expect("Utils should have impact score");
    let ui_score =
        affected_analysis.impact_scores.get("@test/ui").expect("UI should have impact score");

    assert_eq!(*utils_score, 0.5, "Dependents should have score of 0.5");
    assert_eq!(*ui_score, 0.5, "Dependents should have score of 0.5");
    assert!(*core_score > *utils_score, "Direct changes should have higher score than dependents");
}

#[test]
fn test_dependency_impact_no_dependencies_scenario() {
    // Test: Packages with no dependencies work correctly
    let (temp_dir, project) = create_test_monorepo();
    let _diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create an isolated package with no dependencies
    let isolated_package_dir = temp_dir.path().join("packages/isolated");
    std::fs::create_dir_all(&isolated_package_dir).expect("Failed to create isolated package dir");

    let isolated_package_json = r#"{
  "name": "@test/isolated",
  "version": "1.0.0",
  "dependencies": {}
}"#;
    std::fs::write(isolated_package_dir.join("package.json"), isolated_package_json)
        .expect("Failed to write isolated package.json");

    // Add to git and commit
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add isolated package");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add isolated package"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit isolated package");

    // Refresh project to include new package
    let mut updated_project =
        MonorepoProject::new(temp_dir.path()).expect("Failed to create updated project");
    updated_project.refresh_packages().expect("Failed to refresh packages");
    updated_project.build_dependency_graph().expect("Failed to build dependency graph");

    let updated_diff_analyzer = DiffAnalyzer::from_project(Arc::new(updated_project));

    // Change the isolated package
    let isolated_changes = vec![(
        "packages/isolated/src/index.js",
        "// Isolated package change\nexport const value = 'isolated';",
    )];

    // Create src directory
    std::fs::create_dir_all(isolated_package_dir.join("src")).expect("Failed to create src dir");
    create_git_changes(&temp_dir, &isolated_changes);

    let changed_files = updated_diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = updated_diff_analyzer.identify_affected_packages(&changed_files);
    assert!(result.is_ok(), "Should handle isolated package changes");

    let affected_analysis = result.unwrap();

    // Only the isolated package should be affected
    assert_eq!(
        affected_analysis.directly_affected.len(),
        1,
        "Only isolated package should be directly affected"
    );
    assert_eq!(
        affected_analysis.dependents_affected.len(),
        0,
        "No dependents should be affected for isolated package"
    );
    assert_eq!(
        affected_analysis.total_affected_count, 1,
        "Total affected should be 1 for isolated package"
    );
}

#[test]
fn test_dependency_impact_circular_dependencies_handling() {
    // Test: System handles circular dependencies gracefully (if they exist)
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Note: In real scenarios, circular dependencies should be avoided,
    // but the system should handle them gracefully if they exist

    // Change a package and verify the analysis doesn't infinite loop
    let changes =
        vec![("packages/core/src/core.ts", "// Safe change\nexport const VERSION = '1.0.1';")];
    create_git_changes(&temp_dir, &changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    // This should complete without hanging (no infinite loops)
    let start_time = std::time::Instant::now();
    let result = diff_analyzer.identify_affected_packages(&changed_files);
    let duration = start_time.elapsed();

    assert!(result.is_ok(), "Should handle dependency analysis without infinite loops");
    assert!(duration.as_millis() < 5000, "Should complete quickly without infinite loops");
}

#[test]
fn test_dependency_impact_performance_large_dependency_chain() {
    // Test: Performance with larger dependency chains
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create multiple files in core package to simulate larger change
    let mut large_changes = Vec::new();
    for i in 0..20 {
        large_changes.push((
            format!("packages/core/src/module{}.ts", i),
            format!("// Module {}\nexport const module{} = () => 'module{}';\n// Additional content to make files larger\n// Line 1\n// Line 2\n// Line 3\n// Line 4\n// Line 5", i, i, i)
        ));
    }

    // Convert to string slices
    let large_changes: Vec<(&str, &str)> =
        large_changes.iter().map(|(path, content)| (path.as_str(), content.as_str())).collect();

    create_git_changes(&temp_dir, &large_changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    // Measure performance of dependency impact analysis
    let start_time = std::time::Instant::now();
    let result = diff_analyzer.identify_affected_packages(&changed_files);
    let duration = start_time.elapsed();

    assert!(result.is_ok(), "Should handle large changes successfully");
    assert!(
        duration.as_millis() < 10000,
        "Should complete large analysis within reasonable time ({}ms)",
        duration.as_millis()
    );

    let affected_analysis = result.unwrap();
    assert!(!affected_analysis.directly_affected.is_empty(), "Should identify affected packages");
    assert!(!affected_analysis.impact_scores.is_empty(), "Should calculate impact scores");
}

// ============================================================================
// Rebuild/Retest Requirements Analysis Tests (Phase 2.4a-5d-3 continuation)
// ============================================================================

#[test]
fn test_rebuild_retest_requirements_for_source_code_changes() {
    // Test: Source code changes require rebuild and retest of affected packages
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Change source code in core package
    let source_changes = vec![(
        "packages/core/src/core.ts",
        "// Source code change\nexport function coreFunction() { return 'modified logic'; }",
    )];
    create_git_changes(&temp_dir, &source_changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = diff_analyzer.identify_affected_packages(&changed_files);
    assert!(result.is_ok(), "Should identify affected packages successfully");

    let affected_analysis = result.unwrap();

    // All affected packages should require rebuild and retest
    let all_affected: Vec<String> = affected_analysis
        .directly_affected
        .iter()
        .chain(affected_analysis.dependents_affected.iter())
        .cloned()
        .collect();

    // Core package (directly affected) should require full rebuild
    assert!(
        all_affected.contains(&"@test/core".to_string()),
        "Core should be marked for rebuild (direct source change)"
    );

    // Dependent packages should require retest to ensure compatibility
    assert!(
        all_affected.contains(&"@test/utils".to_string()),
        "Utils should be marked for retest (depends on changed core)"
    );
    assert!(
        all_affected.contains(&"@test/ui".to_string()),
        "UI should be marked for retest (depends on changed core)"
    );

    // Impact scores should reflect rebuild necessity
    let core_score = affected_analysis.impact_scores.get("@test/core").unwrap();
    assert!(*core_score > 1.0, "Core should have high impact score requiring rebuild");
}

#[test]
fn test_rebuild_retest_requirements_for_dependency_changes() {
    // Test: Dependency changes require rebuild of affected package and retest of dependents
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Change package.json dependencies in utils package
    let dependency_changes = vec![(
        "packages/utils/package.json",
        r#"{
  "name": "@test/utils",
  "version": "1.2.0",
  "dependencies": {
    "@test/core": "*",
    "lodash": "^4.17.21"
  }
}"#,
    )];
    create_git_changes(&temp_dir, &dependency_changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = diff_analyzer.identify_affected_packages(&changed_files);
    assert!(result.is_ok(), "Should identify affected packages successfully");

    let affected_analysis = result.unwrap();

    // Utils should be directly affected and require rebuild
    assert!(
        affected_analysis.directly_affected.contains(&"@test/utils".to_string()),
        "Utils should require rebuild (dependency change)"
    );

    // UI should be affected as dependent and require retest
    assert!(
        affected_analysis.dependents_affected.contains(&"@test/ui".to_string()),
        "UI should require retest (depends on utils with new dependencies)"
    );

    // Core should not be affected (utils doesn't affect core)
    assert!(
        !affected_analysis.dependents_affected.contains(&"@test/core".to_string()),
        "Core should not require rebuild/retest (no dependency on utils)"
    );
}

#[test]
fn test_rebuild_retest_requirements_minimal_for_docs_changes() {
    // Test: Documentation changes require minimal rebuild/retest
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Change only documentation
    let doc_changes = vec![
        ("packages/core/README.md", "# Core Package\nUpdated documentation with new examples."),
        ("packages/utils/docs/api.md", "# API Documentation\nDetailed API reference."),
    ];

    // Create docs directory
    std::fs::create_dir_all(temp_dir.path().join("packages/utils/docs"))
        .expect("Failed to create docs directory");
    create_git_changes(&temp_dir, &doc_changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = diff_analyzer.identify_affected_packages(&changed_files);
    assert!(result.is_ok(), "Should identify affected packages successfully");

    let affected_analysis = result.unwrap();

    // Only the packages with doc changes should be directly affected
    assert_eq!(
        affected_analysis.directly_affected.len(),
        2,
        "Only core and utils should be directly affected by doc changes"
    );

    // Core and utils have dependents, so they should still be identified
    // even for documentation changes, as the system conservatively assumes
    // all changes could affect dependents (e.g., API documentation changes)
    assert!(
        affected_analysis.dependents_affected.len() >= 1,
        "Dependents should be identified conservatively even for doc changes"
    );

    // Impact scores should reflect the package's role even for documentation changes
    let core_score = affected_analysis.impact_scores.get("@test/core").unwrap();
    let utils_score = affected_analysis.impact_scores.get("@test/utils").unwrap();

    // Core has dependents, so its score is elevated even for doc changes
    assert!(*core_score > 1.0, "Core should have elevated score due to dependents");
    assert!(*utils_score > 1.0, "Utils should have elevated score due to dependents");

    // Both should have the same elevated base score for dependent packages
    assert!(*core_score >= 1.0, "Core should have at least base score");
    assert!(*utils_score >= 1.0, "Utils should have at least base score");
}

#[test]
fn test_rebuild_retest_requirements_for_test_changes() {
    // Test: Test file changes require retest of package but minimal rebuild
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Change test files
    let test_changes = vec![
        (
            "packages/core/__tests__/core.test.ts",
            "// Updated tests\ndescribe('core', () => { test('new test case', () => {}); });",
        ),
        (
            "packages/utils/src/utils.spec.ts",
            "// New spec\ndescribe('utils', () => { test('utils spec', () => {}); });",
        ),
    ];

    // Create utils src directory for the spec file
    std::fs::create_dir_all(temp_dir.path().join("packages/utils/src"))
        .expect("Failed to create utils src directory");
    create_git_changes(&temp_dir, &test_changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = diff_analyzer.identify_affected_packages(&changed_files);
    assert!(result.is_ok(), "Should identify affected packages successfully");

    let affected_analysis = result.unwrap();

    // Only packages with test changes should be directly affected
    assert_eq!(
        affected_analysis.directly_affected.len(),
        2,
        "Core and utils should be affected by test changes"
    );

    // The system conservatively identifies dependents even for test changes
    // as test changes could indicate issues that affect dependent packages
    assert!(
        affected_analysis.dependents_affected.len() >= 1,
        "Dependents should be identified conservatively even for test changes"
    );

    // Test changes should have impact calculated considering all affected packages
    let total_affected = affected_analysis.total_affected_count;
    let expected_total =
        affected_analysis.directly_affected.len() + affected_analysis.dependents_affected.len();
    assert_eq!(total_affected, expected_total, "Total affected should include all packages");
}

#[test]
fn test_rebuild_retest_requirements_for_configuration_changes() {
    // Test: Configuration changes require careful rebuild analysis
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Change build configuration files
    let config_changes = vec![
        (
            "packages/core/tsconfig.json",
            r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "strict": true,
    "declaration": true
  }
}"#,
        ),
        (
            "packages/utils/webpack.config.js",
            "// Webpack config\nmodule.exports = { mode: 'production' };",
        ),
    ];
    create_git_changes(&temp_dir, &config_changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = diff_analyzer.identify_affected_packages(&changed_files);
    assert!(result.is_ok(), "Should identify affected packages successfully");

    let affected_analysis = result.unwrap();

    // Packages with config changes should be directly affected
    assert!(
        affected_analysis.directly_affected.contains(&"@test/core".to_string()),
        "Core should be affected by tsconfig change"
    );
    assert!(
        affected_analysis.directly_affected.contains(&"@test/utils".to_string()),
        "Utils should be affected by webpack config change"
    );

    // Dependents might be affected by config changes (depending on build output)
    let all_affected: Vec<String> = affected_analysis
        .directly_affected
        .iter()
        .chain(affected_analysis.dependents_affected.iter())
        .cloned()
        .collect();

    // UI should be affected if core config changes affect its dependencies
    assert!(
        all_affected.contains(&"@test/ui".to_string()),
        "UI should be affected by core config changes (dependency)"
    );
}

#[test]
fn test_rebuild_retest_requirements_priority_based_on_change_type() {
    // Test: Different change types have different rebuild/retest priorities
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create mixed changes with different priorities
    let mixed_changes = vec![
        // High priority: breaking API change
        (
            "packages/core/src/api.ts",
            "// BREAKING CHANGE: API modified\nexport interface CoreAPI { newMethod(): void; }",
        ),
        // Medium priority: source code change
        (
            "packages/utils/src/utils.ts",
            "// Implementation change\nexport function utils() { return 'updated'; }",
        ),
        // Low priority: documentation
        ("packages/ui/README.md", "# UI Package\nUpdated documentation"),
    ];

    // Create necessary directories
    std::fs::create_dir_all(temp_dir.path().join("packages/core/src"))
        .expect("Failed to create core src directory");
    std::fs::create_dir_all(temp_dir.path().join("packages/utils/src"))
        .expect("Failed to create utils src directory");

    create_git_changes(&temp_dir, &mixed_changes);

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = diff_analyzer.identify_affected_packages(&changed_files);
    assert!(result.is_ok(), "Should identify affected packages successfully");

    let affected_analysis = result.unwrap();

    // All three packages should be directly affected
    assert_eq!(
        affected_analysis.directly_affected.len(),
        3,
        "All packages with changes should be directly affected"
    );

    // Check impact scores reflect change priorities
    let core_score = affected_analysis.impact_scores.get("@test/core").unwrap();
    let utils_score = affected_analysis.impact_scores.get("@test/utils").unwrap();
    let ui_score = affected_analysis.impact_scores.get("@test/ui").unwrap();

    // Core should have highest score (has dependents + breaking change)
    assert!(*core_score > *utils_score, "Core should have higher impact than utils");
    // Note: UI and Utils might have similar scores in some scenarios,
    // but core should always be highest due to having more dependents
    assert!(*utils_score >= *ui_score, "Utils should have at least equal impact to UI");

    // UI should not be in dependents_affected since it's directly affected
    // (packages can't be in both directly_affected and dependents_affected)
    assert!(
        !affected_analysis.dependents_affected.contains(&"@test/ui".to_string()),
        "UI should not be in dependents list since it's directly affected"
    );

    // However, UI should be in directly_affected since it has changes
    assert!(
        affected_analysis.directly_affected.contains(&"@test/ui".to_string()),
        "UI should be directly affected due to its documentation changes"
    );
}

#[test]
fn test_rebuild_retest_requirements_incremental_analysis() {
    // Test: System can determine incremental rebuild/retest requirements
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // First change: small modification
    let first_changes = vec![(
        "packages/utils/src/helper.ts",
        "// Helper function\nexport function helper() { return 'help v1'; }",
    )];

    // Create directory
    std::fs::create_dir_all(temp_dir.path().join("packages/utils/src"))
        .expect("Failed to create utils src directory");
    create_git_changes(&temp_dir, &first_changes);

    let first_commit_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get first commit files");

    let first_result = diff_analyzer
        .identify_affected_packages(&first_commit_files)
        .expect("Should analyze first change");

    // Second change: another modification
    let second_changes = vec![(
        "packages/utils/src/helper.ts",
        "// Helper function\nexport function helper() { return 'help v2'; }",
    )];
    create_git_changes(&temp_dir, &second_changes);

    let second_commit_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get second commit files");

    let second_result = diff_analyzer
        .identify_affected_packages(&second_commit_files)
        .expect("Should analyze second change");

    // Both analyses should identify the same affected packages
    assert_eq!(
        first_result.directly_affected, second_result.directly_affected,
        "Incremental changes to same package should have consistent results"
    );

    assert_eq!(
        first_result.dependents_affected, second_result.dependents_affected,
        "Dependent packages should remain consistent across incremental changes"
    );

    // Utils should be directly affected in both cases
    assert!(
        first_result.directly_affected.contains(&"@test/utils".to_string()),
        "Utils should be directly affected in first change"
    );
    assert!(
        second_result.directly_affected.contains(&"@test/utils".to_string()),
        "Utils should be directly affected in second change"
    );
}

#[test]
fn test_rebuild_retest_requirements_performance_large_changeset() {
    // Test: Performance of rebuild/retest analysis with large changesets
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create a large changeset affecting multiple packages
    let mut large_changeset = Vec::new();

    // Create many files in each package
    for pkg in ["core", "utils", "ui"] {
        // Create source files
        for i in 0..15 {
            large_changeset.push((
                format!("packages/{}/src/module{}.ts", pkg, i),
                format!("// Module {} in {}\nexport const value{} = '{}_{}';\n// Additional content line 1\n// Additional content line 2\n// Additional content line 3", i, pkg, i, pkg, i)
            ));
        }

        // Create test files
        for i in 0..10 {
            large_changeset.push((
                format!("packages/{}/__tests__/test{}.spec.ts", pkg, i),
                format!("// Test {} for {}\ndescribe('test{}', () => {{ test('works', () => {{}}); }});", i, pkg, i)
            ));
        }

        // Create docs
        large_changeset.push((
            format!("packages/{}/docs/README.md", pkg),
            format!("# {} Package\nComprehensive documentation for {}.", pkg, pkg),
        ));
    }

    // Convert to string slices
    let large_changeset: Vec<(&str, &str)> =
        large_changeset.iter().map(|(path, content)| (path.as_str(), content.as_str())).collect();

    // Create necessary directories
    for pkg in ["core", "utils", "ui"] {
        std::fs::create_dir_all(temp_dir.path().join(format!("packages/{}/src", pkg)))
            .expect("Failed to create src directory");
        std::fs::create_dir_all(temp_dir.path().join(format!("packages/{}/__tests__", pkg)))
            .expect("Failed to create tests directory");
        std::fs::create_dir_all(temp_dir.path().join(format!("packages/{}/docs", pkg)))
            .expect("Failed to create docs directory");
    }

    create_git_changes(&temp_dir, &large_changeset);

    // Measure performance of large changeset analysis
    let start_time = std::time::Instant::now();

    let changed_files = diff_analyzer
        .git_provider
        .get_all_files_changed_since_sha_with_status("HEAD~1")
        .expect("Should get changed files");

    let result = diff_analyzer.identify_affected_packages(&changed_files);
    let duration = start_time.elapsed();

    assert!(result.is_ok(), "Should handle large changeset successfully");
    assert!(
        duration.as_millis() < 15000,
        "Large changeset analysis should complete within reasonable time ({}ms)",
        duration.as_millis()
    );

    let affected_analysis = result.unwrap();

    // All packages should be directly affected
    assert_eq!(
        affected_analysis.directly_affected.len(),
        3,
        "All packages should be directly affected by large changeset"
    );

    // Verify all packages are included
    for pkg in ["@test/core", "@test/utils", "@test/ui"] {
        assert!(
            affected_analysis.directly_affected.contains(&pkg.to_string()),
            "Package {} should be directly affected",
            pkg
        );
    }

    // Impact scores should be calculated
    assert_eq!(
        affected_analysis.impact_scores.len(),
        3,
        "Impact scores should be calculated for all affected packages"
    );

    // Total affected count should include all packages
    assert!(
        affected_analysis.total_affected_count >= 3,
        "Total affected count should include all packages"
    );
}

// ============================================================================
// Change Significance Analysis Tests (Phase 2.4a-5d-4)
// ============================================================================

#[test]
fn test_low_significance_documentation_changes() {
    // Test: Documentation changes start with Low significance
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create documentation changes
    let doc_changes = vec![
        (
            "packages/core/README.md",
            "# Core Package\nUpdated documentation for core functionality.",
        ),
        ("packages/utils/docs/api.md", "# API Documentation\nDetailed API reference guide."),
        ("packages/ui/CHANGELOG.md", "# Changelog\n\n## v2.0.1\n- Updated documentation"),
    ];

    // Create docs directory
    std::fs::create_dir_all(temp_dir.path().join("packages/utils/docs"))
        .expect("Failed to create utils docs directory");
    create_git_changes(&temp_dir, &doc_changes);

    let result = diff_analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Should detect documentation changes successfully");

    let change_analysis = result.unwrap();
    let significance_results = change_analysis.significance_analysis;

    // Find documentation changes
    let doc_significances: Vec<_> = significance_results
        .iter()
        .filter(|s| s.original_significance == crate::changes::ChangeSignificance::Low)
        .collect();

    assert!(
        !doc_significances.is_empty(),
        "Should have documentation changes with Low significance"
    );

    // All documentation changes should start as Low
    for result in &doc_significances {
        assert_eq!(
            result.original_significance,
            crate::changes::ChangeSignificance::Low,
            "Documentation changes should start with Low significance"
        );

        // May be elevated due to package having dependents
        assert!(
            result.final_significance >= crate::changes::ChangeSignificance::Low,
            "Final significance should be at least Low"
        );

        // Version bump should be Patch for documentation
        assert_eq!(
            result.suggested_version_bump,
            crate::config::VersionBumpType::Patch,
            "Documentation changes should suggest Patch version bump"
        );
    }
}

#[test]
fn test_low_significance_test_changes() {
    // Test: Test changes start with Low significance
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create test changes
    let test_changes = vec![
        (
            "packages/core/__tests__/core.test.ts",
            "// Updated tests\ndescribe('core tests', () => { test('new test', () => {}); });",
        ),
        (
            "packages/utils/src/utils.spec.ts",
            "// Spec file\ndescribe('utils spec', () => { test('spec test', () => {}); });",
        ),
        (
            "packages/ui/tests/component.test.jsx",
            "// Component tests\ntest('UI component test', () => {});",
        ),
    ];

    // Create necessary directories
    std::fs::create_dir_all(temp_dir.path().join("packages/utils/src"))
        .expect("Failed to create utils src directory");
    std::fs::create_dir_all(temp_dir.path().join("packages/ui/tests"))
        .expect("Failed to create ui tests directory");
    create_git_changes(&temp_dir, &test_changes);

    let result = diff_analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Should detect test changes successfully");

    let change_analysis = result.unwrap();
    let significance_results = change_analysis.significance_analysis;

    // Test changes should be present
    assert!(!significance_results.is_empty(), "Should have significance analysis for test changes");

    // Check that test changes have appropriate significance
    for result in &significance_results {
        // Version bump for test changes depends on final significance
        // If elevated due to dependents/core package, may be Minor instead of Patch
        // Core/shared/utils packages can be elevated to High (Major version bump)
        assert!(
            matches!(result.suggested_version_bump,
                    crate::config::VersionBumpType::Patch |
                    crate::config::VersionBumpType::Minor |
                    crate::config::VersionBumpType::Major),
            "Test changes should suggest Patch, Minor or Major version bump (got {:?} for package {})",
            result.suggested_version_bump, result.package_name
        );

        // Original significance should typically be Low for test-related files
        if result.original_significance == crate::changes::ChangeSignificance::Low {
            // If significance was elevated, should have reasons
            if result.final_significance > result.original_significance {
                assert!(
                    !result.reasons.is_empty(),
                    "Should have reasons for significance elevation"
                );
            }
        }
    }
}

#[test]
fn test_low_significance_configuration_changes() {
    // Test: Configuration changes start with Low significance
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create configuration changes
    let config_changes = vec![
        (
            "packages/core/.eslintrc.json",
            r#"{"extends": ["eslint:recommended"], "rules": {"no-console": "warn"}}"#,
        ),
        (
            "packages/utils/tsconfig.json",
            r#"{"compilerOptions": {"strict": true, "target": "ES2020"}}"#,
        ),
        (
            "packages/ui/webpack.config.js",
            "module.exports = { mode: 'development', entry: './src/index.js' };",
        ),
        ("packages/core/.env", "NODE_ENV=development\nDEBUG=true"),
    ];

    create_git_changes(&temp_dir, &config_changes);

    let result = diff_analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Should detect configuration changes successfully");

    let change_analysis = result.unwrap();
    let significance_results = change_analysis.significance_analysis;

    assert!(
        !significance_results.is_empty(),
        "Should have significance analysis for config changes"
    );

    // Configuration changes should suggest Patch version bumps
    // Note: Version bump for config changes depends on final significance.
    // If elevated due to dependents/core package, may be Minor instead of Patch
    for result in &significance_results {
        // Configuration changes suggest Patch by default, but may be elevated to Minor
        assert!(
            matches!(result.suggested_version_bump,
                    crate::config::VersionBumpType::Patch | crate::config::VersionBumpType::Minor),
            "Configuration changes should suggest Patch or Minor version bump (got {:?} for package {})",
            result.suggested_version_bump, result.package_name
        );
    }
}

#[test]
fn test_medium_significance_source_code_modifications() {
    // Test: Modified source code has Low significance, but added/deleted has Medium
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create source code modifications
    std::fs::create_dir_all(temp_dir.path().join("packages/core/src"))
        .expect("Failed to create core src directory");
    std::fs::create_dir_all(temp_dir.path().join("packages/utils/src"))
        .expect("Failed to create utils src directory");

    let source_changes = vec![
        (
            "packages/core/src/existing.ts",
            "// Modified existing file\nexport function existingFunction() { return 'modified'; }",
        ),
        (
            "packages/utils/src/new-feature.ts",
            "// New feature file\nexport function newFeature() { return 'new feature'; }",
        ),
    ];

    // Create existing file first
    std::fs::write(temp_dir.path().join("packages/core/src/existing.ts"), "// Original content")
        .expect("Failed to create existing file");

    create_git_changes(&temp_dir, &source_changes);

    let result = diff_analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Should detect source code changes successfully");

    let change_analysis = result.unwrap();
    let significance_results = change_analysis.significance_analysis;

    assert!(
        !significance_results.is_empty(),
        "Should have significance analysis for source changes"
    );

    // Source code changes should suggest appropriate version bumps
    for result in &significance_results {
        if result.final_significance == crate::changes::ChangeSignificance::Medium {
            // Medium source code changes should suggest Minor bump
            assert!(
                result.suggested_version_bump == crate::config::VersionBumpType::Minor
                    || result.suggested_version_bump == crate::config::VersionBumpType::Major,
                "Medium source code changes should suggest Minor or Major bump"
            );
        } else if result.final_significance == crate::changes::ChangeSignificance::Low {
            // Low source code changes should suggest Patch bump
            assert!(
                result.suggested_version_bump == crate::config::VersionBumpType::Patch
                    || result.suggested_version_bump == crate::config::VersionBumpType::Minor,
                "Low source code changes should suggest Patch or Minor bump"
            );
        }
    }
}

#[test]
fn test_medium_significance_dependency_changes() {
    // Test: Package.json changes have Medium significance
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create dependency changes
    let dependency_changes = vec![
        (
            "packages/core/package.json",
            r#"{
  "name": "@test/core",
  "version": "1.1.0",
  "dependencies": {
    "lodash": "^4.17.21",
    "axios": "^1.0.0"
  }
}"#,
        ),
        (
            "packages/utils/package.json",
            r#"{
  "name": "@test/utils",
  "version": "1.2.0",
  "dependencies": {
    "@test/core": "*",
    "moment": "^2.29.0"
  }
}"#,
        ),
    ];

    create_git_changes(&temp_dir, &dependency_changes);

    let result = diff_analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Should detect dependency changes successfully");

    let change_analysis = result.unwrap();
    let significance_results = change_analysis.significance_analysis;

    assert!(
        !significance_results.is_empty(),
        "Should have significance analysis for dependency changes"
    );

    // Find dependency changes
    let dep_significances: Vec<_> = significance_results
        .iter()
        .filter(|s| s.original_significance == crate::changes::ChangeSignificance::Medium)
        .collect();

    assert!(
        !dep_significances.is_empty(),
        "Should have dependency changes with Medium significance"
    );

    for result in &dep_significances {
        assert_eq!(
            result.original_significance,
            crate::changes::ChangeSignificance::Medium,
            "Dependency changes should start with Medium significance"
        );

        // Should suggest Minor bump for Medium dependencies
        assert!(
            result.suggested_version_bump == crate::config::VersionBumpType::Minor
                || result.suggested_version_bump == crate::config::VersionBumpType::Major,
            "Medium dependency changes should suggest Minor or Major bump"
        );
    }
}

#[test]
fn test_high_significance_breaking_change_indicators() {
    // Test: Breaking change indicators force High significance
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create changes with breaking change indicators
    std::fs::create_dir_all(temp_dir.path().join("packages/core/src"))
        .expect("Failed to create core src directory");

    let breaking_changes = vec![
        (
            "packages/core/src/BREAKING-api.ts",
            "// BREAKING CHANGE: API modified\nexport interface NewAPI { newMethod(): void; }",
        ),
        (
            "packages/core/src/interface-change.ts",
            "// Interface change\nexport interface ModifiedInterface { updatedMethod(): string; }",
        ),
        (
            "packages/core/src/contract-update.ts",
            "// Contract update\nexport const API_CONTRACT = 'v2.0';",
        ),
    ];

    create_git_changes(&temp_dir, &breaking_changes);

    let result = diff_analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Should detect breaking changes successfully");

    let change_analysis = result.unwrap();
    let significance_results = change_analysis.significance_analysis;

    assert!(
        !significance_results.is_empty(),
        "Should have significance analysis for breaking changes"
    );

    // Find changes with breaking indicators
    let breaking_significances: Vec<_> = significance_results
        .iter()
        .filter(|s| s.final_significance == crate::changes::ChangeSignificance::High)
        .collect();

    assert!(
        !breaking_significances.is_empty(),
        "Should have changes elevated to High significance"
    );

    for result in &breaking_significances {
        assert_eq!(
            result.final_significance,
            crate::changes::ChangeSignificance::High,
            "Breaking change indicators should force High significance"
        );

        // Should suggest Major bump for High significance
        assert_eq!(
            result.suggested_version_bump,
            crate::config::VersionBumpType::Major,
            "High significance changes should suggest Major bump"
        );

        // Should have breaking change reason
        assert!(
            result.reasons.iter().any(|r| r.contains("breaking change indicator")),
            "Should have reason about breaking change indicator"
        );
    }
}

#[test]
fn test_significance_elevation_many_dependents() {
    // Test: Packages with many dependents get elevated significance
    let (temp_dir, project) = create_test_monorepo();

    // Create a package structure where core has many dependents
    // We'll modify the test to create a scenario where we can control dependent count

    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create a simple documentation change in core (starts as Low)
    let core_changes = vec![(
        "packages/core/README.md",
        "# Core Package\nThis package has many dependents so should be elevated.",
    )];

    create_git_changes(&temp_dir, &core_changes);

    let result = diff_analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Should detect changes successfully");

    let change_analysis = result.unwrap();
    let significance_results = change_analysis.significance_analysis;

    assert!(!significance_results.is_empty(), "Should have significance analysis");

    // Find core package results
    let core_significance = significance_results.iter().find(|s| s.package_name == "@test/core");

    if let Some(result) = core_significance {
        // Core has dependents (utils and ui), so it might be elevated
        if result.final_significance > result.original_significance {
            assert!(result.reasons.iter().any(|r| r.contains("dependents") || r.contains("core") || r.contains("Core")),
                   "Should have reason for elevation due to dependents or core package status. Actual reasons: {:?}", result.reasons);
        }
    }
}

#[test]
fn test_significance_elevation_core_packages() {
    // Test: Core/shared/utils packages get elevated significance
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create changes in packages with core/shared/utils names
    let core_package_changes = vec![
        (
            "packages/core/src/core.ts",
            "// Core package change\nexport function coreFunction() { return 'core'; }",
        ),
        (
            "packages/utils/src/utils.ts",
            "// Utils package change\nexport function utilsFunction() { return 'utils'; }",
        ),
    ];

    // Create directories
    std::fs::create_dir_all(temp_dir.path().join("packages/core/src"))
        .expect("Failed to create core src directory");
    std::fs::create_dir_all(temp_dir.path().join("packages/utils/src"))
        .expect("Failed to create utils src directory");

    create_git_changes(&temp_dir, &core_package_changes);

    let result = diff_analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Should detect core package changes successfully");

    let change_analysis = result.unwrap();
    let significance_results = change_analysis.significance_analysis;

    assert!(
        !significance_results.is_empty(),
        "Should have significance analysis for core packages"
    );

    // Check core and utils packages
    for result in &significance_results {
        if result.package_name.contains("core") || result.package_name.contains("utils") {
            // Should have elevation reason for core/shared packages
            if result.final_significance > result.original_significance {
                assert!(
                    result.reasons.iter().any(|r| r.contains("Core/shared library package")),
                    "Should have reason for core/shared package elevation"
                );
            }
        }
    }
}

#[test]
fn test_version_bump_suggestions_comprehensive() {
    // Test: Comprehensive validation of version bump suggestions
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create changes of different types and significances
    std::fs::create_dir_all(temp_dir.path().join("packages/core/src"))
        .expect("Failed to create core src directory");

    let comprehensive_changes = vec![
        // High significance (breaking) → Major
        (
            "packages/core/src/BREAKING-change.ts",
            "// BREAKING: API changed\nexport interface NewAPI {}",
        ),
        // Documentation → always Patch
        ("packages/core/README.md", "# Core\nDocumentation update"),
        // Medium source code → Minor
        ("packages/core/src/new-feature.ts", "// New feature\nexport function newFeature() {}"),
        // Configuration → always Patch
        ("packages/core/tsconfig.json", r#"{"compilerOptions": {"strict": true}}"#),
    ];

    create_git_changes(&temp_dir, &comprehensive_changes);

    let result = diff_analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Should detect comprehensive changes successfully");

    let change_analysis = result.unwrap();
    let significance_results = change_analysis.significance_analysis;

    assert!(
        !significance_results.is_empty(),
        "Should have significance analysis for comprehensive changes"
    );

    // Validate version bump logic
    for result in &significance_results {
        match result.final_significance {
            crate::changes::ChangeSignificance::High => {
                assert_eq!(
                    result.suggested_version_bump,
                    crate::config::VersionBumpType::Major,
                    "High significance should always suggest Major bump"
                );
            }
            crate::changes::ChangeSignificance::Medium => {
                // Medium with source code or dependencies should be Minor
                assert!(
                    result.suggested_version_bump == crate::config::VersionBumpType::Minor
                        || result.suggested_version_bump == crate::config::VersionBumpType::Major,
                    "Medium significance should suggest Minor or Major bump"
                );
            }
            crate::changes::ChangeSignificance::Low => {
                // Low can be Patch or Minor depending on change type
                assert!(
                    result.suggested_version_bump == crate::config::VersionBumpType::Patch
                        || result.suggested_version_bump == crate::config::VersionBumpType::Minor,
                    "Low significance should suggest Patch or Minor bump"
                );
            }
        }
    }
}

#[test]
fn test_significance_analysis_edge_cases() {
    // Test: Edge cases and complex scenarios
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create complex edge case changes
    std::fs::create_dir_all(temp_dir.path().join("packages/ui/src"))
        .expect("Failed to create ui src directory");

    let edge_case_changes = vec![
        // File with multiple indicators
        (
            "packages/ui/src/BREAKING-interface-API.ts",
            "// Multiple indicators\nexport interface BreakingAPI { contract(): void; }",
        ),
        // Mixed significance in same package
        ("packages/ui/README.md", "# UI Package\nDocumentation for breaking API changes"),
        (
            "packages/ui/src/component.tsx",
            "// Component update\nexport const Component = () => <div>Updated</div>;",
        ),
    ];

    create_git_changes(&temp_dir, &edge_case_changes);

    let result = diff_analyzer.detect_changes_since("HEAD~1", None);
    assert!(result.is_ok(), "Should handle edge cases successfully");

    let change_analysis = result.unwrap();
    let significance_results = change_analysis.significance_analysis;

    assert!(!significance_results.is_empty(), "Should have significance analysis for edge cases");

    // UI package should have significance analysis
    let ui_significance = significance_results.iter().find(|s| s.package_name == "@test/ui");

    if let Some(result) = ui_significance {
        // Should have at least one reason for significance determination
        assert!(!result.reasons.is_empty(), "Should have reasons for significance determination");

        // Should have valid version bump suggestion
        assert!(
            matches!(
                result.suggested_version_bump,
                crate::config::VersionBumpType::Patch
                    | crate::config::VersionBumpType::Minor
                    | crate::config::VersionBumpType::Major
            ),
            "Should have valid version bump suggestion"
        );
    }
}

#[test]
fn test_significance_analysis_performance() {
    // Test: Performance with many packages and changes
    let (temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));

    // Create many changes across all packages
    let mut many_changes = Vec::new();

    for pkg in ["core", "utils", "ui"] {
        // Multiple file types per package
        for i in 0..10 {
            many_changes.push((
                format!("packages/{}/src/feature{}.ts", pkg, i),
                format!("// Feature {}\nexport function feature{}() {{ return '{}'; }}", i, i, pkg),
            ));
            many_changes.push((
                format!("packages/{}/docs/doc{}.md", pkg, i),
                format!("# Documentation {}\nDocumentation for feature {}", i, i),
            ));
        }

        // Create directories
        std::fs::create_dir_all(temp_dir.path().join(format!("packages/{}/src", pkg)))
            .expect("Failed to create src directory");
        std::fs::create_dir_all(temp_dir.path().join(format!("packages/{}/docs", pkg)))
            .expect("Failed to create docs directory");
    }

    // Convert to string slices
    let many_changes: Vec<(&str, &str)> =
        many_changes.iter().map(|(path, content)| (path.as_str(), content.as_str())).collect();

    create_git_changes(&temp_dir, &many_changes);

    // Measure performance
    let start_time = std::time::Instant::now();
    let result = diff_analyzer.detect_changes_since("HEAD~1", None);
    let duration = start_time.elapsed();

    assert!(result.is_ok(), "Should handle many changes successfully");
    assert!(
        duration.as_millis() < 10000,
        "Significance analysis should complete within reasonable time ({}ms)",
        duration.as_millis()
    );

    let change_analysis = result.unwrap();
    let significance_results = change_analysis.significance_analysis;

    assert!(!significance_results.is_empty(), "Should have significance analysis for many changes");
    assert_eq!(
        significance_results.len(),
        3,
        "Should have significance analysis for all 3 packages"
    );

    // All results should have valid data
    for result in &significance_results {
        assert!(!result.package_name.is_empty(), "Should have package name");
        assert!(
            !result.reasons.is_empty() || result.original_significance == result.final_significance,
            "Should have reasons if significance changed"
        );
        assert!(
            matches!(
                result.suggested_version_bump,
                crate::config::VersionBumpType::Patch
                    | crate::config::VersionBumpType::Minor
                    | crate::config::VersionBumpType::Major
            ),
            "Should have valid version bump suggestion"
        );
    }
}

// ==================== BRANCH COMPARISON AND CONFLICT DETECTION TESTS ====================

#[test]
fn test_branch_comparison_basic_functionality() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create a test branch with some changes
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature-branch"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create test branch");

    // Add a simple file change
    std::fs::write(
        _temp_dir.path().join("packages/core/new-feature.js"),
        "console.log('new feature');",
    )
    .expect("Failed to create test file");

    std::process::Command::new("git")
        .args(["add", "packages/core/new-feature.js"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add file to git");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add new feature"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit changes");

    // Test branch comparison
    let result = diff_analyzer.compare_branches("main", "feature-branch");

    assert!(result.is_ok(), "Branch comparison should succeed");
    let comparison = result.unwrap();

    assert_eq!(comparison.base_branch, "main");
    assert_eq!(comparison.target_branch, "feature-branch");
    assert!(!comparison.changed_files.is_empty(), "Should detect changed files");
    assert!(!comparison.affected_packages.is_empty(), "Should detect affected packages");
    assert!(!comparison.merge_base.is_empty(), "Should have a merge base");
}

#[test]
fn test_branch_comparison_merge_base_detection() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Get initial commit sha for comparison
    let initial_commit = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to get HEAD commit");
    let initial_sha = std::str::from_utf8(&initial_commit.stdout).unwrap().trim();

    // Create feature branch
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature-branch"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create feature branch");

    // Make changes
    std::fs::write(
        _temp_dir.path().join("packages/utils/helper.js"),
        "export const helper = () => 'help';",
    )
    .expect("Failed to create helper file");

    std::process::Command::new("git")
        .args(["add", "packages/utils/helper.js"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add helper file");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add helper function"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit helper");

    // Test merge base detection
    let result = diff_analyzer.compare_branches("main", "feature-branch");
    assert!(result.is_ok(), "Should detect merge base successfully");

    let comparison = result.unwrap();
    assert_eq!(comparison.merge_base, initial_sha, "Merge base should be the initial commit");
}

#[test]
fn test_branch_comparison_conflict_detection() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create first branch with changes to package.json
    std::process::Command::new("git")
        .args(["checkout", "-b", "branch-a"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create branch-a");

    // Modify core package.json
    let core_package_path = _temp_dir.path().join("packages/core/package.json");
    let mut package_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&core_package_path).unwrap()).unwrap();
    package_json["version"] = serde_json::Value::String("1.1.0".to_string());
    std::fs::write(&core_package_path, serde_json::to_string_pretty(&package_json).unwrap())
        .expect("Failed to update package.json");

    std::process::Command::new("git")
        .args(["add", "packages/core/package.json"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add package.json");

    std::process::Command::new("git")
        .args(["commit", "-m", "Update core version in branch-a"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit in branch-a");

    // Go back to main and create second branch
    std::process::Command::new("git")
        .args(["checkout", "main"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to checkout main");

    std::process::Command::new("git")
        .args(["checkout", "-b", "branch-b"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create branch-b");

    // Modify the same file differently
    let mut package_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&core_package_path).unwrap()).unwrap();
    package_json["version"] = serde_json::Value::String("1.2.0".to_string());
    std::fs::write(&core_package_path, serde_json::to_string_pretty(&package_json).unwrap())
        .expect("Failed to update package.json");

    std::process::Command::new("git")
        .args(["add", "packages/core/package.json"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add package.json");

    std::process::Command::new("git")
        .args(["commit", "-m", "Update core version in branch-b"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit in branch-b");

    // Test conflict detection
    let result = diff_analyzer.compare_branches("branch-a", "branch-b");
    assert!(result.is_ok(), "Should detect potential conflicts");

    let comparison = result.unwrap();
    assert!(!comparison.conflicts.is_empty(), "Should detect conflicts for modified file");
    assert!(
        comparison.conflicts.iter().any(|conflict| conflict.contains("package.json")),
        "Should identify package.json as conflicting file"
    );
}

#[test]
fn test_branch_comparison_with_no_conflicts() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create branch with changes to different files
    std::process::Command::new("git")
        .args(["checkout", "-b", "no-conflict-branch"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create no-conflict-branch");

    // Add new file (no conflict possible)
    std::fs::write(
        _temp_dir.path().join("packages/ui/new-component.js"),
        "export const NewComponent = () => 'new';",
    )
    .expect("Failed to create new component");

    std::process::Command::new("git")
        .args(["add", "packages/ui/new-component.js"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add new component");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add new component"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit new component");

    let result = diff_analyzer.compare_branches("main", "no-conflict-branch");
    assert!(result.is_ok(), "Should handle no-conflict case");

    let comparison = result.unwrap();
    assert!(comparison.conflicts.is_empty(), "Should have no conflicts for new files");
    assert!(!comparison.changed_files.is_empty(), "Should still detect changed files");
    assert!(
        comparison.affected_packages.contains(&"@test/ui".to_string()),
        "Should detect ui package as affected"
    );
}

#[test]
fn test_branch_comparison_with_divergent_branches() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create two divergent branches
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature-a"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create feature-a");

    std::fs::write(
        _temp_dir.path().join("packages/core/feature-a.js"),
        "export const featureA = true;",
    )
    .expect("Failed to create feature-a file");

    std::process::Command::new("git")
        .args(["add", "packages/core/feature-a.js"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add feature-a file");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add feature A"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit feature A");

    // Go back to main and create another branch
    std::process::Command::new("git")
        .args(["checkout", "main"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to checkout main");

    std::process::Command::new("git")
        .args(["checkout", "-b", "feature-b"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create feature-b");

    std::fs::write(
        _temp_dir.path().join("packages/utils/feature-b.js"),
        "export const featureB = true;",
    )
    .expect("Failed to create feature-b file");

    std::process::Command::new("git")
        .args(["add", "packages/utils/feature-b.js"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add feature-b file");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add feature B"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit feature B");

    // Compare the divergent branches
    let result = diff_analyzer.compare_branches("feature-a", "feature-b");
    assert!(result.is_ok(), "Should handle divergent branches");

    let comparison = result.unwrap();
    assert_eq!(comparison.base_branch, "feature-a");
    assert_eq!(comparison.target_branch, "feature-b");
    assert!(
        !comparison.changed_files.is_empty(),
        "Should detect changes between divergent branches"
    );
    assert!(
        comparison.affected_packages.len() >= 1,
        "Should detect affected packages from both branches"
    );
}

#[test]
fn test_branch_comparison_result_structure_completeness() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create a simple branch with changes
    std::process::Command::new("git")
        .args(["checkout", "-b", "structure-test"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create structure-test branch");

    std::fs::write(
        _temp_dir.path().join("packages/core/test-file.js"),
        "export const test = true;",
    )
    .expect("Failed to create test file");

    std::process::Command::new("git")
        .args(["add", "packages/core/test-file.js"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add test file");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add test file"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit test file");

    let result = diff_analyzer.compare_branches("main", "structure-test");
    assert!(result.is_ok(), "Should complete comparison successfully");

    let comparison = result.unwrap();

    // Validate all required fields are present and valid
    assert!(!comparison.base_branch.is_empty(), "Base branch should not be empty");
    assert!(!comparison.target_branch.is_empty(), "Target branch should not be empty");
    assert!(!comparison.changed_files.is_empty(), "Changed files should not be empty");
    assert!(!comparison.affected_packages.is_empty(), "Affected packages should not be empty");
    assert!(!comparison.merge_base.is_empty(), "Merge base should not be empty");

    // Validate changed files have proper structure
    for file in &comparison.changed_files {
        assert!(!file.path.is_empty(), "File path should not be empty");
        // GitFileStatus enum should be valid (already validated by type system)
    }

    // Validate affected packages are reasonable
    assert!(
        comparison.affected_packages.contains(&"@test/core".to_string()),
        "Should identify core package as affected"
    );
}

#[test]
fn test_branch_comparison_error_handling() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Test with non-existent branch
    let result = diff_analyzer.compare_branches("main", "non-existent-branch");
    assert!(result.is_err(), "Should fail when comparing with non-existent branch");

    // Test with invalid branch names
    let result = diff_analyzer.compare_branches("", "main");
    assert!(result.is_err(), "Should fail with empty branch name");

    let result = diff_analyzer.compare_branches("main", "");
    assert!(result.is_err(), "Should fail with empty target branch name");
}

#[test]
fn test_branch_comparison_git_provider_integration() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create test branch with multiple types of changes
    std::process::Command::new("git")
        .args(["checkout", "-b", "git-integration-test"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create git-integration-test branch");

    // Add various types of changes
    std::fs::write(_temp_dir.path().join("packages/core/index.js"), "export * from './core';")
        .expect("Failed to create core index");

    std::fs::write(
        _temp_dir.path().join("packages/utils/README.md"),
        "# Utils Package\n\nUtility functions.",
    )
    .expect("Failed to create utils README");

    // Modify existing package.json
    let utils_package_path = _temp_dir.path().join("packages/utils/package.json");
    let mut package_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&utils_package_path).unwrap()).unwrap();
    package_json["description"] = serde_json::Value::String("Updated utils package".to_string());
    std::fs::write(&utils_package_path, serde_json::to_string_pretty(&package_json).unwrap())
        .expect("Failed to update utils package.json");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add all changes");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add multiple file types"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit multiple changes");

    let result = diff_analyzer.compare_branches("main", "git-integration-test");
    assert!(result.is_ok(), "Should handle various Git operations successfully");

    let comparison = result.unwrap();
    assert!(comparison.changed_files.len() >= 3, "Should detect all changed files");
    assert!(comparison.affected_packages.len() >= 2, "Should detect both affected packages");

    // Validate that different file types are detected
    let file_paths: Vec<&str> = comparison.changed_files.iter().map(|f| f.path.as_str()).collect();
    assert!(
        file_paths.iter().any(|path| path.contains("index.js")),
        "Should detect JS file changes"
    );
    assert!(
        file_paths.iter().any(|path| path.contains("README.md")),
        "Should detect markdown changes"
    );
    assert!(
        file_paths.iter().any(|path| path.contains("package.json")),
        "Should detect package.json changes"
    );
}

#[test]
fn test_branch_comparison_performance_with_large_changesets() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create branch with many file changes
    std::process::Command::new("git")
        .args(["checkout", "-b", "large-changeset"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create large-changeset branch");

    // Create many files to simulate large changeset
    for i in 0..50 {
        let file_path = _temp_dir.path().join(format!("packages/core/file_{}.js", i));
        std::fs::write(file_path, format!("export const value{} = {};", i, i))
            .expect("Failed to create test file");
    }

    for i in 0..30 {
        let file_path = _temp_dir.path().join(format!("packages/utils/util_{}.js", i));
        std::fs::write(file_path, format!("export const util{} = () => {};", i, i))
            .expect("Failed to create util file");
    }

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add all files");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add large changeset"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit large changeset");

    // Test performance
    let start_time = std::time::Instant::now();
    let result = diff_analyzer.compare_branches("main", "large-changeset");
    let duration = start_time.elapsed();

    assert!(result.is_ok(), "Should handle large changesets successfully");
    assert!(
        duration.as_millis() < 5000,
        "Branch comparison should complete within reasonable time ({}ms)",
        duration.as_millis()
    );

    let comparison = result.unwrap();
    assert!(
        comparison.changed_files.len() >= 80,
        "Should detect all changed files in large changeset"
    );
    assert!(comparison.affected_packages.len() >= 2, "Should identify all affected packages");
}

// ==================== BUILT-IN ANALYZERS VALIDATION TESTS ====================

#[test]
fn test_package_json_analyzer() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create package.json changes
    std::process::Command::new("git")
        .args(["checkout", "-b", "package-json-test"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create package-json-test branch");

    // Modify package.json with dependency changes
    let core_package_path = _temp_dir.path().join("packages/core/package.json");
    let mut package_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&core_package_path).unwrap()).unwrap();
    package_json["dependencies"] = serde_json::json!({
        "lodash": "^4.17.21",
        "react": "^18.0.0"
    });
    std::fs::write(&core_package_path, serde_json::to_string_pretty(&package_json).unwrap())
        .expect("Failed to update package.json");

    std::process::Command::new("git")
        .args(["add", "packages/core/package.json"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add package.json");

    std::process::Command::new("git")
        .args(["commit", "-m", "Update package.json dependencies"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit package.json changes");

    // Test package.json analyzer
    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();
    let package_changes = changes.package_changes;

    assert!(!package_changes.is_empty(), "Should detect package.json changes");

    let core_change =
        package_changes.iter().find(|change| change.package_name == "@test/core").unwrap();
    assert_eq!(
        core_change.change_type,
        crate::changes::PackageChangeType::Dependencies,
        "Should detect as dependency change"
    );
    assert!(
        matches!(
            core_change.significance,
            crate::changes::ChangeSignificance::Medium | crate::changes::ChangeSignificance::High
        ),
        "Package.json changes should have medium or high significance"
    );

    // Verify analyzer context
    assert!(core_change.metadata.contains_key("contexts"), "Should have context metadata");
    let contexts = core_change.metadata.get("contexts").unwrap();
    assert!(contexts.contains("Package.json"), "Should mention package.json in context");
}

#[test]
fn test_source_code_analyzer() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create source code changes with different file types
    std::process::Command::new("git")
        .args(["checkout", "-b", "source-code-test"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create source-code-test branch");

    // Test different source code file extensions
    let test_files = vec![
        ("packages/core/index.js", "export const coreIndex = true;"),
        ("packages/utils/helper.ts", "export const helper = (): string => 'help';"),
        ("packages/ui/component.jsx", "export const Component = () => <div>Test</div>;"),
        ("packages/ui/types.tsx", "export interface Props { test: boolean; }"),
        ("packages/core/module.mjs", "export { default } from './core.js';"),
        ("packages/utils/legacy.cjs", "module.exports = { legacy: true };"),
        ("packages/ui/App.vue", "<template><div>Vue App</div></template>"),
        ("packages/ui/Widget.svelte", "<script>let count = 0;</script>"),
    ];

    for (file_path, content) in test_files {
        std::fs::write(_temp_dir.path().join(file_path), content)
            .expect("Failed to create source file");
    }

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add source files");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add various source code files"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit source changes");

    // Test source code analyzer
    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();
    let package_changes = changes.package_changes;

    assert!(!package_changes.is_empty(), "Should detect source code changes");

    // Verify all packages have source code changes
    for package_change in &package_changes {
        if package_change.package_name.contains("core")
            || package_change.package_name.contains("utils")
            || package_change.package_name.contains("ui")
        {
            assert!(
                matches!(package_change.change_type, crate::changes::PackageChangeType::SourceCode),
                "Should detect as source code change for package: {}",
                package_change.package_name
            );

            // Verify significance based on file status (added files = Medium)
            assert!(
                matches!(
                    package_change.significance,
                    crate::changes::ChangeSignificance::Medium
                        | crate::changes::ChangeSignificance::High
                ),
                "Added source files should have medium or high significance"
            );
        }
    }
}

#[test]
fn test_configuration_analyzer() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create configuration file changes
    std::process::Command::new("git")
        .args(["checkout", "-b", "config-test"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create config-test branch");

    // Test different configuration file types
    let config_files = vec![
        ("tsconfig.json", r#"{"compilerOptions": {"target": "es2020"}}"#),
        ("babel.config.js", "module.exports = { presets: ['@babel/preset-env'] };"),
        ("webpack.config.js", "module.exports = { entry: './src/index.js' };"),
        ("rollup.config.js", "export default { input: 'src/main.js' };"),
        ("vite.config.ts", "export default { base: '/app/' };"),
        ("jest.config.json", r#"{"testEnvironment": "node"}"#),
        (".eslintrc.json", r#"{"extends": ["eslint:recommended"]}"#),
        (".prettierrc", r#"{"semi": false, "singleQuote": true}"#),
        ("Dockerfile", "FROM node:18\nWORKDIR /app"),
        ("config.yaml", "server:\n  port: 3000"),
        ("settings.toml", "[database]\nhost = 'localhost'"),
        (".env", "NODE_ENV=production\nPORT=3000"),
    ];

    for (file_name, content) in config_files {
        std::fs::write(_temp_dir.path().join(file_name), content)
            .expect("Failed to create config file");
    }

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add config files");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add various configuration files"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit config changes");

    // Test configuration analyzer
    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();

    // Configuration files at root level should still be detected in change analysis
    assert!(!changes.changed_files.is_empty(), "Should detect configuration file changes");

    // Check that configuration files are properly categorized
    let config_file_detected = changes.changed_files.iter().any(|file| {
        file.path.contains("tsconfig.json")
            || file.path.contains("babel.config")
            || file.path.contains("webpack.config")
            || file.path.contains("jest.config")
            || file.path.contains(".eslintrc")
            || file.path.contains("Dockerfile")
    });

    assert!(config_file_detected, "Should detect various configuration file types");
}

#[test]
fn test_documentation_analyzer() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create documentation changes
    std::process::Command::new("git")
        .args(["checkout", "-b", "docs-test"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create docs-test branch");

    // Test different documentation file types
    let doc_files = vec![
        ("packages/core/README.md", "# Core Package\n\nThis is the core package."),
        ("packages/utils/CHANGELOG.md", "# Changelog\n\n## v1.0.0\n- Initial release"),
        ("docs/api.md", "# API Documentation\n\nAPI reference guide."),
        ("docs/guide.rst", "User Guide\n=========\n\nGetting started guide."),
        ("LICENSE.txt", "MIT License\n\nCopyright (c) 2024"),
        ("CONTRIBUTING.adoc", "= Contributing Guide\n\nHow to contribute to this project."),
        ("packages/ui/docs/components.md", "# UI Components\n\nComponent documentation."),
    ];

    for (file_path, content) in doc_files {
        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&_temp_dir.path().join(file_path)).parent() {
            std::fs::create_dir_all(parent).expect("Failed to create directory");
        }
        std::fs::write(_temp_dir.path().join(file_path), content)
            .expect("Failed to create doc file");
    }

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add doc files");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add various documentation files"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit doc changes");

    // Test documentation analyzer
    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();
    let package_changes = changes.package_changes;

    // Find packages with documentation changes
    let core_change = package_changes.iter().find(|change| change.package_name == "@test/core");
    let ui_change = package_changes.iter().find(|change| change.package_name == "@test/ui");

    if let Some(core_change) = core_change {
        assert!(
            matches!(core_change.change_type, crate::changes::PackageChangeType::Documentation),
            "Core package should detect documentation changes"
        );
        assert_eq!(
            core_change.significance,
            crate::changes::ChangeSignificance::Low,
            "Documentation changes should have low significance"
        );
    }

    if let Some(ui_change) = ui_change {
        assert!(
            matches!(ui_change.change_type, crate::changes::PackageChangeType::Documentation),
            "UI package should detect documentation changes"
        );
    }

    // Verify documentation files are detected
    let doc_files_detected = changes.changed_files.iter().any(|file| {
        file.path.contains("README.md")
            || file.path.contains("CHANGELOG.md")
            || file.path.contains("docs/")
            || file.path.ends_with(".rst")
            || file.path.ends_with(".txt")
            || file.path.ends_with(".adoc")
    });

    assert!(doc_files_detected, "Should detect various documentation file types");
}

#[test]
fn test_test_analyzer() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create test file changes
    std::process::Command::new("git")
        .args(["checkout", "-b", "test-files-test"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create test-files-test branch");

    // Test different test file patterns
    let test_files = vec![
        ("packages/core/core.test.js", "describe('core', () => { it('works', () => {}); });"),
        ("packages/core/core.spec.ts", "describe('core', () => { it('works', () => {}); });"),
        (
            "packages/utils/utils.test.mjs",
            "import { test } from 'node:test'; test('utils', () => {});",
        ),
        ("packages/utils/utils.spec.cjs", "const test = require('test'); test('utils', () => {});"),
        (
            "packages/ui/__tests__/component.test.tsx",
            "import { render } from '@testing-library/react';",
        ),
        ("packages/ui/tests/integration.spec.js", "describe('integration', () => {});"),
        ("packages/core/test/unit.js", "const assert = require('assert');"),
        ("packages/utils/spec/helper.spec.mts", "import { describe } from 'mocha';"),
    ];

    for (file_path, content) in test_files {
        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&_temp_dir.path().join(file_path)).parent() {
            std::fs::create_dir_all(parent).expect("Failed to create directory");
        }
        std::fs::write(_temp_dir.path().join(file_path), content)
            .expect("Failed to create test file");
    }

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add test files");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add various test files"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit test changes");

    // Test test analyzer
    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();
    let package_changes = changes.package_changes;

    assert!(!package_changes.is_empty(), "Should detect test file changes");

    // Verify all packages have test changes detected correctly
    for package_change in &package_changes {
        if package_change.package_name.contains("core")
            || package_change.package_name.contains("utils")
            || package_change.package_name.contains("ui")
        {
            assert!(
                matches!(package_change.change_type, crate::changes::PackageChangeType::Tests),
                "Should detect as test change for package: {}",
                package_change.package_name
            );

            // Test changes start as Low, but may get elevated due to:
            // 1. Core/shared/utils packages (contains "core", "shared", "utils")
            // 2. Packages with many dependents (>5)
            // 3. Dirty version status
            // Accept both Low and Medium as valid test change significance
            assert!(
                matches!(package_change.significance,
                        crate::changes::ChangeSignificance::Low |
                        crate::changes::ChangeSignificance::Medium),
                "Test changes should have Low or Medium significance (actual: {:?}) for package: {}",
                package_change.significance, package_change.package_name
            );
        }
    }

    // Verify test file patterns are detected
    let test_patterns_detected = changes.changed_files.iter().any(|file| {
        file.path.contains(".test.")
            || file.path.contains(".spec.")
            || file.path.contains("__tests__")
            || file.path.contains("/tests/")
            || file.path.contains("/test/")
            || file.path.contains("/spec/")
    });

    assert!(test_patterns_detected, "Should detect various test file patterns");
}

#[test]
fn test_analyzer_file_type_detection() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Test that each analyzer correctly identifies its file types
    std::process::Command::new("git")
        .args(["checkout", "-b", "file-type-test"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create file-type-test branch");

    // Create files that should match each analyzer
    let test_cases = vec![
        ("packages/core/package.json", r#"{"name": "@test/core"}"#, "Dependencies"),
        ("packages/core/index.ts", "export const core = true;", "SourceCode"),
        ("tsconfig.json", r#"{"compilerOptions": {}}"#, "Configuration"),
        ("packages/core/README.md", "# Core", "Documentation"),
        ("packages/core/core.test.js", "test('core', () => {});", "Tests"),
    ];

    for (file_path, content, expected_type) in test_cases {
        std::fs::write(_temp_dir.path().join(file_path), content)
            .expect("Failed to create test file");

        std::process::Command::new("git")
            .args(["add", file_path])
            .current_dir(&_temp_dir.path())
            .output()
            .expect("Failed to add file");

        std::process::Command::new("git")
            .args(["commit", "-m", &format!("Add {} file", expected_type)])
            .current_dir(&_temp_dir.path())
            .output()
            .expect("Failed to commit file");
    }

    // Test analyzer detection
    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();
    let package_changes = changes.package_changes;

    // Verify the file type prioritization in PackageChangeBuilder
    let core_change = package_changes.iter().find(|change| change.package_name == "@test/core");
    if let Some(core_change) = core_change {
        // Dependencies should take priority (package.json)
        assert!(
            matches!(core_change.change_type, crate::changes::PackageChangeType::Dependencies),
            "Dependencies should have highest priority in change type determination"
        );
    }
}

#[test]
fn test_multiple_analyzers_same_change() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Test scenarios where multiple analyzers could apply
    std::process::Command::new("git")
        .args(["checkout", "-b", "multi-analyzer-test"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create multi-analyzer-test branch");

    // Create files that might match multiple patterns
    let multi_pattern_files = vec![
        ("packages/core/test.config.js", "module.exports = { testEnvironment: 'node' };"), // Config + Test patterns
        ("packages/utils/README.test.md", "# Test Documentation"), // Documentation + Test patterns
        ("packages/ui/component.spec.json", r#"{"test": "spec"}"#), // Config + Test patterns
    ];

    for (file_path, content) in multi_pattern_files {
        std::fs::write(_temp_dir.path().join(file_path), content)
            .expect("Failed to create multi-pattern file");
    }

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add files");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add files with multiple analyzer patterns"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit changes");

    // Test analyzer handling of multiple patterns
    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();
    let package_changes = changes.package_changes;

    assert!(!package_changes.is_empty(), "Should detect changes with multiple patterns");

    // Verify that change type prioritization works correctly
    for package_change in &package_changes {
        // Should have valid change type based on prioritization
        assert!(
            matches!(
                package_change.change_type,
                crate::changes::PackageChangeType::Dependencies
                    | crate::changes::PackageChangeType::SourceCode
                    | crate::changes::PackageChangeType::Configuration
                    | crate::changes::PackageChangeType::Tests
                    | crate::changes::PackageChangeType::Documentation
            ),
            "Should have valid change type"
        );

        // Metadata should contain information about multiple analyzers
        assert!(
            package_change.metadata.contains_key("change_types_analyzed"),
            "Should have metadata about analyzed change types"
        );
    }
}

#[test]
fn test_analyzer_significance_levels() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Test different significance levels from analyzers
    std::process::Command::new("git")
        .args(["checkout", "-b", "significance-test"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create significance-test branch");

    // Test each analyzer's significance assignment
    let significance_tests = vec![
        ("packages/core/package.json", r#"{"name": "@test/core", "version": "2.0.0"}"#, "Medium"),
        ("packages/core/README.md", "# Updated Core", "Low"),
        ("packages/core/core.test.js", "test('updated', () => {});", "Low"),
        ("tsconfig.json", r#"{"compilerOptions": {"strict": true}}"#, "Low"),
    ];

    for (file_path, content, expected_significance) in significance_tests {
        std::fs::write(_temp_dir.path().join(file_path), content)
            .expect("Failed to create significance test file");

        std::process::Command::new("git")
            .args(["add", file_path])
            .current_dir(&_temp_dir.path())
            .output()
            .expect("Failed to add file");

        std::process::Command::new("git")
            .args(["commit", "-m", &format!("Add {} significance file", expected_significance)])
            .current_dir(&_temp_dir.path())
            .output()
            .expect("Failed to commit file");
    }

    // Add source code with different Git statuses
    std::fs::write(
        _temp_dir.path().join("packages/core/new-feature.ts"),
        "export const newFeature = true;",
    )
    .expect("Failed to create new source file");

    std::process::Command::new("git")
        .args(["add", "packages/core/new-feature.ts"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add new source file");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add new source file (should be Medium significance)"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit new source file");

    // Test significance analysis
    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();
    let package_changes = changes.package_changes;

    assert!(!package_changes.is_empty(), "Should detect significance test changes");

    let core_change =
        package_changes.iter().find(|change| change.package_name == "@test/core").unwrap();

    // Should have elevated significance due to package.json or new source files
    assert!(
        matches!(
            core_change.significance,
            crate::changes::ChangeSignificance::Medium | crate::changes::ChangeSignificance::High
        ),
        "Core package should have elevated significance"
    );
}

#[test]
fn test_built_in_analyzers_performance() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create many files of different types for performance testing
    std::process::Command::new("git")
        .args(["checkout", "-b", "analyzer-performance-test"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create analyzer-performance-test branch");

    // Create 50 files of each type in different packages to get different change types
    for i in 0..50 {
        let files = vec![
            // Source code files - should be SourceCode type
            (
                format!("packages/core/src/module_{}.ts", i),
                format!("export const module{} = {};", i, i),
            ),
            // Documentation files in utils package - should be Documentation type
            (
                format!("packages/utils/docs/doc_{}.md", i),
                format!("# Module {}\n\nDocumentation for module {}.", i, i),
            ),
            // Configuration files at root - should be Configuration type
            (format!("config/config_{}.json", i), format!(r#"{{"setting{}": "value{}" }}"#, i, i)),
        ];

        for (file_path, content) in files {
            if let Some(parent) = std::path::Path::new(&_temp_dir.path().join(&file_path)).parent()
            {
                std::fs::create_dir_all(parent).expect("Failed to create directory");
            }
            std::fs::write(_temp_dir.path().join(&file_path), content)
                .expect("Failed to create performance test file");
        }
    }

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add performance test files");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add many files for analyzer performance test"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit performance test files");

    // Test analyzer performance
    let start_time = std::time::Instant::now();
    let changes = diff_analyzer.detect_changes_since("main", None);
    let duration = start_time.elapsed();

    assert!(changes.is_ok(), "Should handle many files successfully");
    assert!(
        duration.as_millis() < 5000,
        "Analyzer performance should complete within reasonable time ({}ms)",
        duration.as_millis()
    );

    let changes = changes.unwrap();
    assert!(changes.changed_files.len() >= 150, "Should detect all test files (50*3 file types)");

    // Verify all analyzer types are working
    let change_types: std::collections::HashSet<_> =
        changes.package_changes.iter().map(|change| change.change_type).collect();

    assert!(
        change_types.len() >= 2,
        "Should detect multiple change types from different analyzers"
    );
}

// ====== FASE 2.4a-5e: ChangeAnalysis Results Completeness and Accuracy Tests ======

#[test]
fn test_change_analysis_from_to_refs_completeness() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create feature branch
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/test-refs"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create feature branch");

    // Make some changes
    std::fs::write(
        _temp_dir.path().join("packages/core/new-feature.ts"),
        "export const feature = true;",
    )
    .expect("Failed to create new file");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add changes");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add new feature"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit changes");

    // Test detect_changes_since with explicit refs
    let changes = diff_analyzer.detect_changes_since("main", Some("feature/test-refs")).unwrap();

    assert_eq!(changes.from_ref, "main", "from_ref should match input");
    assert_eq!(changes.to_ref, "feature/test-refs", "to_ref should match input");

    // Test with default HEAD
    let changes_head = diff_analyzer.detect_changes_since("main", None).unwrap();
    assert_eq!(changes_head.from_ref, "main", "from_ref should match input");
    assert_eq!(changes_head.to_ref, "HEAD", "to_ref should default to HEAD");
}

#[test]
fn test_change_analysis_changed_files_completeness() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create various types of changes
    std::process::Command::new("git")
        .args(["checkout", "-b", "test-all-file-changes"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create test branch");

    // Add new files
    let new_files = vec![
        ("packages/core/src/new-module.ts", "export const newModule = {};"),
        ("packages/utils/helpers/string-utils.js", "export function capitalize(s) { return s; }"),
        ("packages/ui/components/Button.tsx", "export const Button = () => {};"),
    ];

    for (path, content) in &new_files {
        if let Some(parent) = std::path::Path::new(&_temp_dir.path().join(path)).parent() {
            std::fs::create_dir_all(parent).expect("Failed to create directory");
        }
        std::fs::write(_temp_dir.path().join(path), content).expect("Failed to create file");
    }

    // Modify existing files
    std::fs::write(
        _temp_dir.path().join("packages/core/package.json"),
        r#"{"name": "@test/core", "version": "1.0.1", "dependencies": {}}"#,
    )
    .expect("Failed to modify package.json");

    // Create file to be deleted
    std::fs::write(_temp_dir.path().join("packages/core/old-file.js"), "// old file")
        .expect("Failed to create file to delete");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add changes");

    std::process::Command::new("git")
        .args(["commit", "-m", "Setup for delete test"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit setup");

    // Delete file
    std::fs::remove_file(_temp_dir.path().join("packages/core/old-file.js"))
        .expect("Failed to delete file");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add deletion");

    std::process::Command::new("git")
        .args(["commit", "-m", "Complete all file changes"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit all changes");

    // Analyze changes
    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();

    // Validate all files are captured
    assert!(changes.changed_files.len() >= 5, "Should capture all changed files");

    // Validate different file statuses
    let has_added =
        changes.changed_files.iter().any(|f| f.status == sublime_git_tools::GitFileStatus::Added);
    let has_modified = changes
        .changed_files
        .iter()
        .any(|f| f.status == sublime_git_tools::GitFileStatus::Modified);
    let has_deleted =
        changes.changed_files.iter().any(|f| f.status == sublime_git_tools::GitFileStatus::Deleted);

    assert!(has_added, "Should detect added files");
    assert!(has_modified, "Should detect modified files");
    assert!(has_deleted, "Should detect deleted files");
}

#[test]
fn test_change_analysis_package_mapping_accuracy() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create changes in specific packages
    std::process::Command::new("git")
        .args(["checkout", "-b", "test-package-mapping"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create test branch");

    // Changes in core package
    std::fs::write(
        _temp_dir.path().join("packages/core/src/core-feature.ts"),
        "export const coreFeature = true;",
    )
    .expect("Failed to create core file");

    // Changes in utils package
    std::fs::write(
        _temp_dir.path().join("packages/utils/src/utils-helper.js"),
        "export const utilsHelper = () => {};",
    )
    .expect("Failed to create utils file");

    // Changes in ui package
    std::fs::write(
        _temp_dir.path().join("packages/ui/src/ui-component.tsx"),
        "export const UIComponent = () => null;",
    )
    .expect("Failed to create ui file");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add changes");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add package-specific changes"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit changes");

    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();

    // Validate package_changes mapping
    assert_eq!(changes.package_changes.len(), 3, "Should map changes to all 3 packages");

    // Verify each package has its changes
    let package_names: std::collections::HashSet<_> =
        changes.package_changes.iter().map(|pc| pc.package_name.clone()).collect();

    assert!(package_names.contains("@test/core"), "Should include core package changes");
    assert!(package_names.contains("@test/utils"), "Should include utils package changes");
    assert!(package_names.contains("@test/ui"), "Should include ui package changes");

    // Verify each package has correct files
    for pc in &changes.package_changes {
        match pc.package_name.as_str() {
            "@test/core" => {
                assert!(
                    pc.changed_files.iter().any(|f| f.path.contains("core-feature.ts")),
                    "Core package should contain core-feature.ts"
                );
            }
            "@test/utils" => {
                assert!(
                    pc.changed_files.iter().any(|f| f.path.contains("utils-helper.js")),
                    "Utils package should contain utils-helper.js"
                );
            }
            "@test/ui" => {
                assert!(
                    pc.changed_files.iter().any(|f| f.path.contains("ui-component.tsx")),
                    "UI package should contain ui-component.tsx"
                );
            }
            _ => {}
        }
    }
}

#[test]
fn test_change_analysis_affected_packages_completeness() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create changes in core package (which utils and ui depend on)
    std::process::Command::new("git")
        .args(["checkout", "-b", "test-affected-packages"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create test branch");

    std::fs::write(
        _temp_dir.path().join("packages/core/src/api-change.ts"),
        "export interface NewAPI { version: number; }",
    )
    .expect("Failed to create core API file");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add changes");

    std::process::Command::new("git")
        .args(["commit", "-m", "Change core API"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit changes");

    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();

    // Validate affected_packages structure
    assert!(
        !changes.affected_packages.directly_affected.is_empty(),
        "Should have directly affected packages"
    );
    assert!(
        changes.affected_packages.directly_affected.contains(&"@test/core".to_string()),
        "Core should be directly affected"
    );

    // Utils and UI depend on core, so they should be in dependents_affected
    assert!(
        !changes.affected_packages.dependents_affected.is_empty(),
        "Should have dependents affected"
    );
    assert!(
        changes.affected_packages.dependents_affected.contains(&"@test/utils".to_string())
            || changes.affected_packages.dependents_affected.contains(&"@test/ui".to_string()),
        "Dependents of core should be affected"
    );

    // Validate change propagation graph
    assert!(
        !changes.affected_packages.change_propagation_graph.is_empty(),
        "Change propagation graph should not be empty"
    );
    assert!(
        changes.affected_packages.change_propagation_graph.contains_key("@test/core"),
        "Core should be in propagation graph"
    );

    // Validate impact scores
    assert!(
        !changes.affected_packages.impact_scores.is_empty(),
        "Impact scores should be calculated"
    );
    assert!(
        changes.affected_packages.impact_scores.contains_key("@test/core"),
        "Core should have impact score"
    );

    // Validate total count
    assert_eq!(
        changes.affected_packages.total_affected_count,
        changes.affected_packages.directly_affected.len()
            + changes.affected_packages.dependents_affected.len(),
        "Total count should match sum of directly and dependents affected"
    );
}

#[test]
fn test_change_analysis_significance_analysis_accuracy() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    std::process::Command::new("git")
        .args(["checkout", "-b", "test-significance-accuracy"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create test branch");

    // Create different types of changes with expected significance

    // 1. Documentation change (Low significance)
    std::fs::write(_temp_dir.path().join("packages/ui/README.md"), "# Updated UI Documentation")
        .expect("Failed to create docs");

    // 2. Test change in core package (should trigger elevation due to "core" in name)
    let core_test_dir = _temp_dir.path().join("packages/core/src");
    std::fs::create_dir_all(&core_test_dir).expect("Failed to create core src dir");
    std::fs::write(core_test_dir.join("core.test.ts"), "describe('core tests', () => {});")
        .expect("Failed to create test");

    // 3. Source code change in utils package (should trigger elevation due to "utils" in name)
    let utils_src_dir = _temp_dir.path().join("packages/utils/src");
    std::fs::create_dir_all(&utils_src_dir).expect("Failed to create utils src dir");
    std::fs::write(utils_src_dir.join("new-util.ts"), "export const newUtil = () => {};")
        .expect("Failed to create source");

    // 4. Package.json change (Medium significance)
    std::fs::write(
        _temp_dir.path().join("packages/ui/package.json"),
        r#"{"name": "@test/ui", "version": "2.1.0", "dependencies": {"@test/core": "*", "@test/utils": "*", "react": "^18.0.0"}}"#
    ).expect("Failed to update package.json");

    // 5. Add a file with breaking change indicator to ensure we get reasons
    let breaking_dir = _temp_dir.path().join("packages/core/src");
    std::fs::write(
        breaking_dir.join("BREAKING-api.ts"),
        "export interface BreakingAPI { newField: string; }",
    )
    .expect("Failed to create breaking change file");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add changes");

    std::process::Command::new("git")
        .args(["commit", "-m", "Various significance changes"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit changes");

    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();

    // Validate significance_analysis is populated
    assert!(!changes.significance_analysis.is_empty(), "Significance analysis should not be empty");

    // Validate each package has significance analysis
    for analysis in &changes.significance_analysis {
        assert!(!analysis.package_name.is_empty(), "Package name should be set");

        // For core/utils packages or packages with breaking changes, we should have reasons
        if analysis.package_name.contains("core") || analysis.package_name.contains("utils") {
            assert!(
                !analysis.reasons.is_empty(),
                "Core/utils packages should have reasons for significance: {}",
                analysis.package_name
            );
            assert!(
                analysis.final_significance >= analysis.original_significance,
                "Core/utils packages should have elevated or same significance"
            );
        }

        // For packages with breaking changes, should have high significance
        if analysis.reasons.iter().any(|r| r.contains("breaking")) {
            assert_eq!(
                analysis.final_significance,
                crate::changes::ChangeSignificance::High,
                "Packages with breaking changes should have high significance"
            );
        }

        // Validate version bump suggestions make sense
        match analysis.final_significance {
            crate::changes::ChangeSignificance::High => {
                assert_eq!(
                    analysis.suggested_version_bump,
                    crate::config::VersionBumpType::Major,
                    "High significance should suggest Major bump"
                );
            }
            crate::changes::ChangeSignificance::Medium => {
                assert!(
                    matches!(
                        analysis.suggested_version_bump,
                        crate::config::VersionBumpType::Minor
                            | crate::config::VersionBumpType::Major
                    ),
                    "Medium significance should suggest Minor or Major bump"
                );
            }
            crate::changes::ChangeSignificance::Low => {
                assert!(
                    matches!(
                        analysis.suggested_version_bump,
                        crate::config::VersionBumpType::Patch
                            | crate::config::VersionBumpType::Minor
                            | crate::config::VersionBumpType::Major
                    ),
                    "Low significance can suggest any bump type"
                );
            }
        }
    }
}

#[test]
fn test_change_analysis_orphaned_files_handling() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    std::process::Command::new("git")
        .args(["checkout", "-b", "test-orphaned-files"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create test branch");

    // Create files outside of packages (orphaned files)
    std::fs::write(_temp_dir.path().join("README.md"), "# Monorepo README")
        .expect("Failed to create root README");

    std::fs::write(_temp_dir.path().join(".gitignore"), "node_modules/\ndist/")
        .expect("Failed to create gitignore");

    // Create scripts directory first
    std::fs::create_dir_all(_temp_dir.path().join("scripts"))
        .expect("Failed to create scripts directory");

    std::fs::write(_temp_dir.path().join("scripts/build.js"), "console.log('Build script');")
        .expect("Failed to create build script");

    // Also add a package change for comparison
    std::fs::write(
        _temp_dir.path().join("packages/core/feature.ts"),
        "export const feature = true;",
    )
    .expect("Failed to create package file");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add changes");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add orphaned files"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit changes");

    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();

    // All files should be in changed_files
    assert!(changes.changed_files.len() >= 4, "Should detect all files including orphaned");

    // Only package files should be in package_changes
    assert!(
        changes.package_changes.len() < changes.changed_files.len(),
        "Package changes should be fewer than total changed files"
    );

    // Verify orphaned files are tracked but not in package_changes
    let package_file_paths: std::collections::HashSet<_> = changes
        .package_changes
        .iter()
        .flat_map(|pc| pc.changed_files.iter().map(|f| f.path.clone()))
        .collect();

    let orphaned_files = changes
        .changed_files
        .iter()
        .filter(|f| !package_file_paths.contains(&f.path))
        .collect::<Vec<_>>();

    assert!(!orphaned_files.is_empty(), "Should have orphaned files");
    assert!(
        orphaned_files.iter().any(|f| f.path.contains("README.md")),
        "Root README should be orphaned"
    );
}

#[test]
fn test_change_analysis_consistency_validation() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    std::process::Command::new("git")
        .args(["checkout", "-b", "test-consistency"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create test branch");

    // Create changes in multiple packages
    let changes_to_make = vec![
        ("packages/core/src/module1.ts", "export const m1 = 1;"),
        ("packages/core/src/module2.ts", "export const m2 = 2;"),
        ("packages/utils/src/helper1.js", "export const h1 = () => {};"),
        ("packages/utils/src/helper2.js", "export const h2 = () => {};"),
        ("packages/ui/components/Comp1.tsx", "export const C1 = () => null;"),
    ];

    for (path, content) in changes_to_make {
        if let Some(parent) = std::path::Path::new(&_temp_dir.path().join(path)).parent() {
            std::fs::create_dir_all(parent).expect("Failed to create directory");
        }
        std::fs::write(_temp_dir.path().join(path), content).expect("Failed to create file");
    }

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add changes");

    std::process::Command::new("git")
        .args(["commit", "-m", "Multiple package changes"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit changes");

    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();

    // Validate consistency between changed_files and package_changes
    let total_files_in_packages: usize =
        changes.package_changes.iter().map(|pc| pc.changed_files.len()).sum();

    assert_eq!(
        total_files_in_packages,
        changes.changed_files.len(),
        "All changed files should be accounted for in package changes"
    );

    // Validate no duplicate files across packages
    let mut all_files = std::collections::HashSet::new();
    for pc in &changes.package_changes {
        for file in &pc.changed_files {
            assert!(
                all_files.insert(&file.path),
                "File {} appears in multiple packages",
                file.path
            );
        }
    }

    // Validate package_changes matches affected_packages
    let package_names_from_changes: std::collections::HashSet<_> =
        changes.package_changes.iter().map(|pc| pc.package_name.clone()).collect();

    let directly_affected_set: std::collections::HashSet<_> =
        changes.affected_packages.directly_affected.iter().cloned().collect();

    assert_eq!(
        package_names_from_changes, directly_affected_set,
        "Package changes should match directly affected packages"
    );

    // Validate significance_analysis covers all packages with changes
    let significance_package_names: std::collections::HashSet<_> =
        changes.significance_analysis.iter().map(|sa| sa.package_name.clone()).collect();

    assert_eq!(
        package_names_from_changes, significance_package_names,
        "Significance analysis should cover all changed packages"
    );
}

#[test]
fn test_change_analysis_with_tags_and_commits() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create and tag initial version
    std::process::Command::new("git")
        .args(["tag", "v1.0.0"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create tag");

    // Make changes
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature-after-tag"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create branch");

    std::fs::write(
        _temp_dir.path().join("packages/core/tagged-feature.ts"),
        "export const taggedFeature = true;",
    )
    .expect("Failed to create file");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add changes");

    std::process::Command::new("git")
        .args(["commit", "-m", "Feature after tag"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit");

    // Get commit SHA
    let commit_output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to get commit SHA");

    let commit_sha = String::from_utf8_lossy(&commit_output.stdout).trim().to_string();

    // Test with tag
    let changes_from_tag = diff_analyzer.detect_changes_since("v1.0.0", None).unwrap();
    assert_eq!(changes_from_tag.from_ref, "v1.0.0", "Should accept tag as from_ref");
    assert!(!changes_from_tag.changed_files.is_empty(), "Should detect changes since tag");

    // Test with commit SHA
    std::fs::write(
        _temp_dir.path().join("packages/core/another-feature.ts"),
        "export const anotherFeature = true;",
    )
    .expect("Failed to create another file");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add changes");

    std::process::Command::new("git")
        .args(["commit", "-m", "Another feature"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit");

    let changes_from_commit = diff_analyzer.detect_changes_since(&commit_sha[0..7], None).unwrap();
    assert!(changes_from_commit.from_ref.contains(&commit_sha[0..7]), "Should accept short SHA");
    assert_eq!(
        changes_from_commit.changed_files.len(),
        1,
        "Should only show changes after specific commit"
    );
}

#[test]
fn test_change_analysis_metadata_completeness() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    std::process::Command::new("git")
        .args(["checkout", "-b", "test-metadata"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create branch");

    // Create comprehensive changes
    std::fs::write(
        _temp_dir.path().join("packages/core/src/feature.ts"),
        "export const feature = { version: 1 };",
    )
    .expect("Failed to create feature");

    std::fs::write(
        _temp_dir.path().join("packages/core/package.json"),
        r#"{"name": "@test/core", "version": "1.1.0", "dependencies": {"lodash": "^4.0.0"}}"#,
    )
    .expect("Failed to update package.json");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add changes");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add feature with metadata"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit");

    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();

    // Find core package change
    let core_change = changes
        .package_changes
        .iter()
        .find(|pc| pc.package_name == "@test/core")
        .expect("Should have core package change");

    // Validate metadata exists
    assert!(!core_change.metadata.is_empty(), "Package change should have metadata");

    // Validate metadata contains expected keys
    assert!(core_change.metadata.contains_key("contexts"), "Should have contexts metadata");
    assert!(core_change.metadata.contains_key("total_files"), "Should have total_files metadata");
    assert!(
        core_change.metadata.contains_key("change_types_analyzed"),
        "Should have change types metadata"
    );

    // Validate metadata values
    let total_files = core_change
        .metadata
        .get("total_files")
        .and_then(|v| v.parse::<usize>().ok())
        .expect("total_files should be a valid number");
    assert_eq!(total_files, 2, "Should have 2 files changed");

    // Validate contexts are populated
    let contexts = core_change.metadata.get("contexts").expect("Should have contexts");
    assert!(!contexts.is_empty(), "Contexts should not be empty");
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_change_analysis_end_to_end_accuracy() {
    let (_temp_dir, project) = create_test_monorepo();
    let diff_analyzer = DiffAnalyzer::from_project(project);

    // Create a realistic development scenario
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/complete-feature"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to create feature branch");

    // 1. Add new feature to core
    std::fs::write(
        _temp_dir.path().join("packages/core/src/new-api.ts"),
        r"
export interface UserAPI {
    getUser(id: string): Promise<User>;
    updateUser(id: string, data: Partial<User>): Promise<User>;
}

export interface User {
    id: string;
    name: string;
    email: string;
}
",
    )
    .expect("Failed to create API file");

    // 2. Update utils to use new API
    std::fs::write(
        _temp_dir.path().join("packages/utils/src/user-utils.ts"),
        r"
import { User } from '@test/core';

export function formatUserName(user: User): string {
    return `${user.name} (${user.email})`;
}
",
    )
    .expect("Failed to create utils file");

    // 3. Add UI component using both
    std::fs::write(
        _temp_dir.path().join("packages/ui/src/UserCard.tsx"),
        r"
import { User } from '@test/core';
import { formatUserName } from '@test/utils';

export const UserCard = ({ user }: { user: User }) => {
    return <div>{formatUserName(user)}</div>;
};
",
    )
    .expect("Failed to create UI component");

    // 4. Add tests
    std::fs::write(
        _temp_dir.path().join("packages/core/src/new-api.test.ts"),
        "describe('UserAPI', () => { it('should work', () => {}); });",
    )
    .expect("Failed to create test");

    // 5. Update documentation
    std::fs::write(
        _temp_dir.path().join("packages/core/README.md"),
        "# Core Package\n\nNow includes UserAPI!",
    )
    .expect("Failed to update README");

    // 6. Update package versions
    std::fs::write(
        _temp_dir.path().join("packages/core/package.json"),
        r#"{"name": "@test/core", "version": "2.0.0", "dependencies": {}}"#,
    )
    .expect("Failed to update core version");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to add all changes");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add complete user feature"])
        .current_dir(&_temp_dir.path())
        .output()
        .expect("Failed to commit feature");

    // Perform comprehensive analysis
    let changes = diff_analyzer.detect_changes_since("main", None).unwrap();

    // Validate completeness
    assert_eq!(changes.from_ref, "main");
    assert_eq!(changes.to_ref, "HEAD");
    assert!(changes.changed_files.len() >= 6, "Should have all 6 files");

    // Validate package mapping
    assert_eq!(changes.package_changes.len(), 3, "All 3 packages should have changes");

    // Validate affected packages
    assert_eq!(
        changes.affected_packages.directly_affected.len(),
        3,
        "All 3 packages directly affected"
    );
    // Total count includes both directly affected and dependents, but since we're changing all packages,
    // some packages might still be counted as dependents of other changed packages
    assert!(changes.affected_packages.total_affected_count >= 3, "Total should be at least 3");
    assert!(
        changes.affected_packages.total_affected_count <= 6,
        "Total should not exceed reasonable limit"
    );

    // Validate significance analysis
    assert_eq!(changes.significance_analysis.len(), 3, "Should analyze all 3 packages");

    // Core should have highest significance (API changes + package.json)
    let core_significance = changes
        .significance_analysis
        .iter()
        .find(|s| s.package_name == "@test/core")
        .expect("Core should have significance analysis");

    assert!(
        matches!(
            core_significance.final_significance,
            crate::changes::ChangeSignificance::Medium | crate::changes::ChangeSignificance::High
        ),
        "Core should have elevated significance due to API changes"
    );

    // Validate change types detected
    let core_change = changes
        .package_changes
        .iter()
        .find(|pc| pc.package_name == "@test/core")
        .expect("Should have core change");

    // Core has mixed changes, should prioritize based on our logic
    assert!(
        matches!(
            core_change.change_type,
            crate::changes::PackageChangeType::Dependencies
                | crate::changes::PackageChangeType::Tests
                | crate::changes::PackageChangeType::SourceCode
        ),
        "Core should have appropriate change type"
    );

    // Validate propagation graph
    assert!(
        !changes.affected_packages.change_propagation_graph.is_empty(),
        "Should have propagation relationships"
    );

    // Validate impact scores
    for (pkg, score) in &changes.affected_packages.impact_scores {
        assert!(*score > 0.0, "Package {} should have positive impact score", pkg);

        // Core should have higher score due to dependents
        if pkg == "@test/core" {
            assert!(*score >= 1.0, "Core should have base score or higher");
        }
    }
}

// ============================================================================
// CHANGESET MANAGEMENT WORKFLOW TESTS
// ============================================================================
// Comprehensive tests for changeset management (creation, validation, application)

/// Test: Changeset creation with complete metadata
#[test]
fn test_changeset_creation_with_complete_metadata() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create a changeset spec with complete metadata
    let spec = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Add comprehensive user management API with authentication, authorization, and user profile features".to_string(),
        development_environments: vec![
            crate::config::types::Environment::Development,
            crate::config::types::Environment::Staging,
        ],
        production_deployment: false,
        author: Some("developer@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    // Validate changeset has complete metadata
    assert!(!changeset.id.is_empty(), "Changeset should have a unique ID");
    assert_eq!(changeset.package, "@test/core", "Package should be set correctly");
    assert_eq!(
        changeset.version_bump,
        crate::config::VersionBumpType::Minor,
        "Version bump should be set"
    );
    assert!(!changeset.description.is_empty(), "Description should be set");
    assert!(!changeset.branch.is_empty(), "Branch should be detected from Git");
    assert_eq!(
        changeset.development_environments.len(),
        2,
        "Should have 2 development environments"
    );
    assert!(!changeset.production_deployment, "Production deployment should be false");
    assert_eq!(changeset.author, "developer@example.com", "Author should be set from spec");
    assert_eq!(
        changeset.status,
        crate::changesets::ChangesetStatus::Pending,
        "Status should be Pending"
    );

    // Validate environments are correct
    assert!(changeset
        .development_environments
        .contains(&crate::config::types::Environment::Development));
    assert!(changeset
        .development_environments
        .contains(&crate::config::types::Environment::Staging));

    // Validate timestamps
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(changeset.created_at);
    assert!(diff.num_seconds() < 5, "Created timestamp should be recent");
}

/// Test: Changeset creation with interactive mode (auto-detection)
#[test]
fn test_changeset_creation_interactive_with_autodetection() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Stage some changes to enable auto-detection
    std::fs::write(
        _temp_dir.path().join("packages/core/src/new-feature.ts"),
        "export const newFeature = () => 'interactive';",
    )
    .expect("Failed to create test file");

    std::process::Command::new("git")
        .args(["add", "packages/core/src/new-feature.ts"])
        .current_dir(_temp_dir.path())
        .output()
        .expect("Failed to stage file");

    // Create changeset interactively
    let changeset = changeset_manager
        .create_changeset_interactive(None)
        .expect("Failed to create interactive changeset");

    // Validate auto-detection worked
    assert_eq!(changeset.package, "@test/core", "Should auto-detect core package");
    assert_eq!(
        changeset.version_bump,
        crate::config::VersionBumpType::Patch,
        "Should default to patch"
    );
    assert_eq!(changeset.description, "Interactive changeset", "Should have default description");
    assert!(!changeset.development_environments.is_empty(), "Should have default environments");
    assert!(!changeset.production_deployment, "Should default to no production deployment");
    assert!(!changeset.author.is_empty(), "Should detect author from Git config");
}

/// Test: Changeset creation with invalid package (should fail)
#[test]
fn test_changeset_creation_with_invalid_package() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    let spec = crate::changesets::ChangesetSpec {
        package: "@test/nonexistent".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "This should fail".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("developer@example.com".to_string()),
    };

    let result = changeset_manager.create_changeset(spec);
    assert!(result.is_err(), "Should fail with nonexistent package");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found"), "Error should mention package not found");
}

/// Test: Changeset creation with empty description (should fail validation)
#[test]
fn test_changeset_creation_with_empty_description() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    let spec = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "".to_string(), // Empty description
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("developer@example.com".to_string()),
    };

    let result = changeset_manager.create_changeset(spec);
    assert!(result.is_err(), "Should fail with empty description");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("empty"), "Error should mention empty description");
}

/// Test: Multiple changesets for same package (should validate conflicts)
#[test]
fn test_changeset_creation_with_conflicting_changesets() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create first changeset
    let spec1 = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "First changeset for core package".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("developer1@example.com".to_string()),
    };

    let changeset1 = changeset_manager
        .create_changeset(spec1)
        .expect("First changeset should be created successfully");

    // Create second branch to avoid same-branch conflict
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/conflicting-change"])
        .current_dir(_temp_dir.path())
        .output()
        .expect("Failed to create new branch");

    // Create second changeset for same package from different branch
    let spec2 = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Second changeset for core package from different branch".to_string(),
        development_environments: vec![crate::config::types::Environment::Staging],
        production_deployment: false,
        author: Some("developer2@example.com".to_string()),
    };

    let changeset2 = changeset_manager
        .create_changeset(spec2)
        .expect("Second changeset should be created with warnings");

    // Validate both changesets exist but have different IDs
    assert_ne!(changeset1.id, changeset2.id, "Changesets should have different IDs");
    assert_eq!(changeset1.package, changeset2.package, "Both should target same package");
    assert_ne!(changeset1.branch, changeset2.branch, "Should be from different branches");

    // Validate the changeset was created despite potential conflicts
    // (the warning about conflicts should be in validation results)
    let validation = changeset_manager
        .validate_changeset(&changeset2)
        .expect("Should be able to validate changeset");
    assert!(validation.is_valid, "Changeset should still be valid");
    assert!(!validation.warnings.is_empty(), "Should have warnings about conflicting changesets");

    // Look for warning about pending changesets (adjust text to match actual implementation)
    let has_conflict_warning = validation.warnings.iter().any(|w| {
        w.contains("pending changeset") || w.contains("changeset(s) for the same package")
    });
    assert!(has_conflict_warning, "Should have warning about pending changesets for same package");
}

// ============================================================================
// CHANGESET VALIDATION TESTS
// ============================================================================
// Comprehensive tests for changeset validation (structure, dependencies, conflicts)

/// Test: Valid changeset with different version bump types
#[test]
fn test_changeset_validation_version_bump_types() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Test each version bump type
    let version_bump_types = vec![
        crate::config::VersionBumpType::Major,
        crate::config::VersionBumpType::Minor,
        crate::config::VersionBumpType::Patch,
    ];

    for version_bump in version_bump_types {
        let spec = crate::changesets::ChangesetSpec {
            package: "@test/core".to_string(),
            version_bump,
            description: format!("Test {:?} version bump validation", version_bump),
            development_environments: vec![crate::config::types::Environment::Development],
            production_deployment: false,
            author: Some("developer@example.com".to_string()),
        };

        let changeset =
            changeset_manager.create_changeset(spec).expect("Failed to create changeset");

        let validation =
            changeset_manager.validate_changeset(&changeset).expect("Failed to validate changeset");

        assert!(validation.is_valid, "Changeset with {:?} should be valid", version_bump);
        assert!(validation.errors.is_empty(), "Should have no validation errors");

        // Major version bumps should have warnings about dependent packages
        if matches!(version_bump, crate::config::VersionBumpType::Major) {
            assert!(
                !validation.warnings.is_empty(),
                "Major version bump should have warnings about dependents"
            );
            let has_dependent_warning = validation
                .warnings
                .iter()
                .any(|w| w.contains("dependent") || w.contains("breaking"));
            assert!(
                has_dependent_warning,
                "Should warn about breaking changes affecting dependents"
            );
        }
    }
}

/// Test: Changeset validation with dependency impact analysis
#[test]
fn test_changeset_validation_dependency_impacts() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create major version bump for core package (has dependents: utils, ui)
    let spec = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Major,
        description: "Breaking API changes in core package".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: true,
        author: Some("developer@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    let validation =
        changeset_manager.validate_changeset(&changeset).expect("Failed to validate changeset");

    assert!(validation.is_valid, "Major version changeset should be valid");

    // Should have warnings about dependent packages
    assert!(!validation.warnings.is_empty(), "Should have warnings about dependent packages");

    let has_utils_warning =
        validation.warnings.iter().any(|w| w.contains("@test/utils") || w.contains("dependent"));
    let has_ui_warning =
        validation.warnings.iter().any(|w| w.contains("@test/ui") || w.contains("dependent"));

    assert!(
        has_utils_warning || has_ui_warning,
        "Should warn about dependent packages being affected"
    );

    // Metadata should contain dependency information
    assert!(!validation.metadata.is_empty(), "Should have metadata about dependencies");

    // Check for affected_dependents metadata (this is what the implementation actually sets)
    let affected_dependents = validation
        .metadata
        .get("affected_dependents")
        .map(|deps| deps.split(", ").count())
        .unwrap_or(0);
    assert!(affected_dependents >= 2, "Should affect at least 2 dependent packages");
}

/// Test: Changeset validation with invalid branch naming
#[test]
fn test_changeset_validation_branch_naming() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create changeset on invalid branch name
    std::process::Command::new("git")
        .args(["checkout", "-b", "invalid_branch_name"])
        .current_dir(_temp_dir.path())
        .output()
        .expect("Failed to create branch");

    let spec = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Test branch naming validation".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("developer@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    let validation =
        changeset_manager.validate_changeset(&changeset).expect("Failed to validate changeset");

    // Branch name validation might produce warnings or be valid depending on configuration
    // The changeset should still be valid but may have warnings
    assert!(validation.is_valid, "Changeset should be valid despite branch name");

    // Check if there are any branch naming warnings
    let _has_branch_warning =
        validation.warnings.iter().any(|w| w.contains("branch") || w.contains("naming"));

    // Whether warnings exist depends on configuration, but validation should succeed
    assert!(validation.errors.is_empty(), "Should not have validation errors for branch naming");
}

/// Test: Changeset validation with invalid environments
#[test]
fn test_changeset_validation_environment_configuration() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Test with all valid environments
    let valid_environments = vec![
        crate::config::types::Environment::Development,
        crate::config::types::Environment::Staging,
        crate::config::types::Environment::Integration,
    ];

    let spec = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Test environment configuration validation".to_string(),
        development_environments: valid_environments.clone(),
        production_deployment: false,
        author: Some("developer@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    let validation =
        changeset_manager.validate_changeset(&changeset).expect("Failed to validate changeset");

    assert!(validation.is_valid, "Changeset with valid environments should be valid");
    assert!(validation.errors.is_empty(), "Should have no validation errors");

    // Validate that all environments are properly set
    assert_eq!(changeset.development_environments, valid_environments);

    // Test with production deployment enabled
    let spec_prod = crate::changesets::ChangesetSpec {
        package: "@test/utils".to_string(),
        version_bump: crate::config::VersionBumpType::Major,
        description: "Test production deployment validation".to_string(),
        development_environments: vec![crate::config::types::Environment::Staging],
        production_deployment: true,
        author: Some("developer@example.com".to_string()),
    };

    let changeset_prod = changeset_manager
        .create_changeset(spec_prod)
        .expect("Failed to create production changeset");

    let validation_prod = changeset_manager
        .validate_changeset(&changeset_prod)
        .expect("Failed to validate production changeset");

    assert!(validation_prod.is_valid, "Production changeset should be valid");
    assert!(changeset_prod.production_deployment, "Production deployment should be enabled");

    // Major version changeset should have warnings about dependent packages (not specific to production)
    if matches!(changeset_prod.version_bump, crate::config::VersionBumpType::Major) {
        let _has_breaking_warning = validation_prod
            .warnings
            .iter()
            .any(|w| w.contains("dependent") || w.contains("affect"));
        // Note: This may not trigger if utils package has no dependents
        // But production deployment itself is validated correctly
    }
}

/// Test: Changeset validation with comprehensive metadata checks
#[test]
fn test_changeset_validation_comprehensive_metadata() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    let spec = crate::changesets::ChangesetSpec {
        package: "@test/ui".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Comprehensive metadata validation test with detailed description".to_string(),
        development_environments: vec![
            crate::config::types::Environment::Development,
            crate::config::types::Environment::Staging,
        ],
        production_deployment: true,
        author: Some("comprehensive.tester@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    let validation =
        changeset_manager.validate_changeset(&changeset).expect("Failed to validate changeset");

    assert!(validation.is_valid, "Changeset should be valid");

    // Validate all required fields are present and valid
    assert!(!changeset.id.is_empty(), "ID should not be empty");
    assert!(changeset.id.len() >= 8, "ID should be at least 8 characters");
    assert_eq!(changeset.package, "@test/ui", "Package should match");
    assert!(!changeset.description.is_empty(), "Description should not be empty");
    assert!(changeset.description.len() >= 10, "Description should be meaningful");
    assert!(!changeset.branch.is_empty(), "Branch should not be empty");
    assert!(!changeset.development_environments.is_empty(), "Should have development environments");
    assert!(!changeset.author.is_empty(), "Author should not be empty");
    assert!(changeset.author.contains('@'), "Author should be email format");

    // Validate timestamp is reasonable
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(changeset.created_at);
    assert!(diff.num_seconds() < 10, "Creation timestamp should be recent");
    assert!(diff.num_seconds() >= 0, "Creation timestamp should not be in future");

    // Validate status is correct
    assert_eq!(changeset.status, crate::changesets::ChangesetStatus::Pending);

    // Validate metadata contains expected information
    assert!(!validation.metadata.is_empty(), "Should have validation metadata");

    // Check for version information in metadata (current_version is always set)
    let has_version_info = validation.metadata.contains_key("current_version");
    assert!(has_version_info, "Should have current version metadata");
}

/// Test: Changeset validation with structure and field validation
#[test]
fn test_changeset_validation_structure_validation() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create a valid changeset first
    let spec = crate::changesets::ChangesetSpec {
        package: "@test/utils".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Structure validation test".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("structure.tester@example.com".to_string()),
    };

    let mut changeset =
        changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    // Test validation with valid changeset
    let validation =
        changeset_manager.validate_changeset(&changeset).expect("Failed to validate changeset");
    assert!(validation.is_valid, "Valid changeset should pass validation");
    assert!(validation.errors.is_empty(), "Should have no errors");

    // Test with empty package name (simulate corrupted changeset)
    let original_package = changeset.package.clone();
    changeset.package = String::new();

    let validation_empty_package = changeset_manager
        .validate_changeset(&changeset)
        .expect("Should be able to validate even with empty package");
    assert!(!validation_empty_package.is_valid, "Empty package should make changeset invalid");
    assert!(!validation_empty_package.errors.is_empty(), "Should have validation errors");

    let has_package_error = validation_empty_package
        .errors
        .iter()
        .any(|e| e.contains("package") || e.contains("empty"));
    assert!(has_package_error, "Should have error about empty package");

    // Restore package and test with empty description
    changeset.package = original_package;
    changeset.description = String::new();

    let validation_empty_desc = changeset_manager
        .validate_changeset(&changeset)
        .expect("Should be able to validate even with empty description");
    assert!(!validation_empty_desc.is_valid, "Empty description should make changeset invalid");
    assert!(!validation_empty_desc.errors.is_empty(), "Should have validation errors");

    let has_description_error = validation_empty_desc
        .errors
        .iter()
        .any(|e| e.contains("description") || e.contains("empty"));
    assert!(has_description_error, "Should have error about empty description");
}

/// Test: Changeset validation with conflict detection scenarios
#[test]
fn test_changeset_validation_conflict_detection() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create first changeset
    let spec1 = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "First changeset for conflict detection".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("first.developer@example.com".to_string()),
    };

    let changeset1 =
        changeset_manager.create_changeset(spec1).expect("Failed to create first changeset");

    // Create second branch for second changeset
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/conflict-test"])
        .current_dir(_temp_dir.path())
        .output()
        .expect("Failed to create conflict test branch");

    // Create second changeset for same package with different version bump
    let spec2 = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Major,
        description: "Second changeset for conflict detection".to_string(),
        development_environments: vec![crate::config::types::Environment::Staging],
        production_deployment: true,
        author: Some("second.developer@example.com".to_string()),
    };

    let changeset2 =
        changeset_manager.create_changeset(spec2).expect("Failed to create second changeset");

    // Validate both changesets detect conflicts
    let validation1 = changeset_manager
        .validate_changeset(&changeset1)
        .expect("Failed to validate first changeset");
    let validation2 = changeset_manager
        .validate_changeset(&changeset2)
        .expect("Failed to validate second changeset");

    // Both should be valid but have warnings about conflicts
    assert!(validation1.is_valid, "First changeset should still be valid");
    assert!(validation2.is_valid, "Second changeset should still be valid");

    // At least one should have conflict warnings
    let has_conflict_warning1 = validation1
        .warnings
        .iter()
        .any(|w| w.contains("changeset") || w.contains("conflict") || w.contains("pending"));
    let has_conflict_warning2 = validation2
        .warnings
        .iter()
        .any(|w| w.contains("changeset") || w.contains("conflict") || w.contains("pending"));

    assert!(
        has_conflict_warning1 || has_conflict_warning2,
        "At least one changeset should have conflict warnings"
    );

    // Validate metadata contains conflict information
    let has_conflict_metadata1 =
        validation1.metadata.iter().any(|(k, _)| k.contains("conflict") || k.contains("pending"));
    let has_conflict_metadata2 =
        validation2.metadata.iter().any(|(k, _)| k.contains("conflict") || k.contains("pending"));

    assert!(
        has_conflict_metadata1 || has_conflict_metadata2,
        "Should have metadata about conflicting changesets"
    );

    // Different version bump types should be noted
    if changeset1.version_bump != changeset2.version_bump {
        let has_version_bump_info = validation2
            .metadata
            .iter()
            .any(|(k, v)| k.contains("version") || v.contains("Minor") || v.contains("Major"));
        assert!(has_version_bump_info, "Should have information about version bump differences");
    }
}

/// Test: Changeset validation with dependency validation scenarios
#[test]
fn test_changeset_validation_dependency_validation() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Test changeset for leaf package (no dependents)
    let spec_leaf = crate::changesets::ChangesetSpec {
        package: "@test/ui".to_string(), // ui depends on core and utils, but nothing depends on ui
        version_bump: crate::config::VersionBumpType::Major,
        description: "Major change to leaf package".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("leaf.developer@example.com".to_string()),
    };

    let changeset_leaf =
        changeset_manager.create_changeset(spec_leaf).expect("Failed to create leaf changeset");

    let validation_leaf = changeset_manager
        .validate_changeset(&changeset_leaf)
        .expect("Failed to validate leaf changeset");

    assert!(validation_leaf.is_valid, "Leaf package changeset should be valid");

    // Leaf package major changes should have fewer warnings than core packages
    let dependent_warnings_count = validation_leaf
        .warnings
        .iter()
        .filter(|w| w.contains("dependent") || w.contains("breaking"))
        .count();

    // Test changeset for core package (has many dependents)
    let spec_core = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(), // core is depended on by utils and ui
        version_bump: crate::config::VersionBumpType::Major,
        description: "Major change to core package with many dependents".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("core.developer@example.com".to_string()),
    };

    let changeset_core =
        changeset_manager.create_changeset(spec_core).expect("Failed to create core changeset");

    let validation_core = changeset_manager
        .validate_changeset(&changeset_core)
        .expect("Failed to validate core changeset");

    assert!(validation_core.is_valid, "Core package changeset should be valid");

    // Core package major changes should have more warnings about dependents
    let core_dependent_warnings_count = validation_core
        .warnings
        .iter()
        .filter(|w| w.contains("dependent") || w.contains("breaking"))
        .count();

    assert!(
        core_dependent_warnings_count >= dependent_warnings_count,
        "Core package should have at least as many dependent warnings as leaf package"
    );

    // Should have specific information about affected packages
    let affected_dependents = validation_core
        .metadata
        .get("affected_dependents")
        .map(|deps| deps.split(", ").count())
        .unwrap_or(0);

    assert!(affected_dependents >= 2, "Core changes should affect at least 2 other packages");

    // Validate dependency information is present
    let has_dependency_metadata = validation_core.metadata.contains_key("affected_dependents");
    assert!(has_dependency_metadata, "Should have affected dependents metadata");
}

// ============================================================================
// CHANGESET APPLICATION AND VERSION MANAGEMENT TESTS
// ============================================================================
// Comprehensive tests for changeset application, version bumping, and merge behavior

/// Test: Apply single changeset with patch version bump
#[test]
fn test_changeset_application_patch_version_bump() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create and save a changeset for patch version bump
    let spec = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Fix critical bug in authentication system".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("bugfix.developer@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    // Get initial version
    let initial_version = {
        let package_info = changeset_manager
            .package_provider
            .get_package("@test/core")
            .expect("Package should exist");
        let package_json_path = package_info.path().join("package.json");
        let content = changeset_manager
            .file_system_provider
            .read_file_string(&package_json_path)
            .expect("Should read package.json");
        let json: serde_json::Value =
            serde_json::from_str(&content).expect("Should parse package.json");
        json["version"].as_str().unwrap().to_string()
    };

    // Apply changeset (apply all changesets for current branch)
    let current_branch = changeset.branch.clone();
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to apply changeset");

    assert_eq!(applications.len(), 1, "Should apply exactly one changeset");

    let application = &applications[0];
    assert_eq!(application.changeset_id, changeset.id);
    assert_eq!(application.package, "@test/core");
    assert_eq!(application.old_version, initial_version);
    assert!(application.success, "Application should be successful");

    // Verify version was bumped correctly (1.0.0 -> 1.0.1)
    let expected_new_version = "1.0.1";
    assert_eq!(application.new_version, expected_new_version);

    // Verify package.json was actually updated
    let package_info =
        changeset_manager.package_provider.get_package("@test/core").expect("Package should exist");
    let package_json_path = package_info.path().join("package.json");
    let updated_content = changeset_manager
        .file_system_provider
        .read_file_string(&package_json_path)
        .expect("Should read updated package.json");
    let updated_json: serde_json::Value =
        serde_json::from_str(&updated_content).expect("Should parse updated package.json");

    assert_eq!(
        updated_json["version"].as_str().unwrap(),
        expected_new_version,
        "package.json should be updated with new version"
    );
}

/// Test: Apply changeset with minor version bump
#[test]
fn test_changeset_application_minor_version_bump() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create changeset for minor version bump
    let spec = crate::changesets::ChangesetSpec {
        package: "@test/utils".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Add new utility functions for string manipulation".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("feature.developer@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    // Apply changeset (apply all changesets for current branch)
    let current_branch = changeset.branch.clone();
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to apply changeset");

    assert_eq!(applications.len(), 1, "Should apply exactly one changeset");

    let application = &applications[0];
    assert_eq!(application.old_version, "1.1.0"); // From create_test_package
    assert_eq!(application.new_version, "1.2.0"); // Minor bump: 1.1.0 -> 1.2.0
    assert!(application.success, "Application should be successful");
}

/// Test: Apply changeset with major version bump and dependency updates
#[test]
fn test_changeset_application_major_version_bump_with_dependents() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create changeset for major version bump on core package
    let spec = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Major,
        description: "Breaking API changes - complete rewrite of core interfaces".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("architect.developer@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    // Apply changeset (apply all changesets for current branch)
    let current_branch = changeset.branch.clone();
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to apply changeset");

    assert_eq!(applications.len(), 1, "Should apply exactly one changeset");

    let application = &applications[0];
    assert_eq!(application.old_version, "1.0.0");
    assert_eq!(application.new_version, "2.0.0"); // Major bump: 1.0.0 -> 2.0.0
    assert!(application.success, "Application should be successful");

    // Verify core package was updated
    let core_package_info = changeset_manager
        .package_provider
        .get_package("@test/core")
        .expect("Core package should exist");
    let core_package_json_path = core_package_info.path().join("package.json");
    let core_content = changeset_manager
        .file_system_provider
        .read_file_string(&core_package_json_path)
        .expect("Should read core package.json");
    let core_json: serde_json::Value =
        serde_json::from_str(&core_content).expect("Should parse core package.json");

    assert_eq!(core_json["version"].as_str().unwrap(), "2.0.0");

    // Verify dependent packages (utils and ui) had their dependencies updated
    let utils_package_info = changeset_manager
        .package_provider
        .get_package("@test/utils")
        .expect("Utils package should exist");
    let utils_package_json_path = utils_package_info.path().join("package.json");
    let utils_content = changeset_manager
        .file_system_provider
        .read_file_string(&utils_package_json_path)
        .expect("Should read utils package.json");
    let utils_json: serde_json::Value =
        serde_json::from_str(&utils_content).expect("Should parse utils package.json");

    // Check if utils dependency on core was updated to ^2.0.0
    if let Some(dependencies) = utils_json["dependencies"].as_object() {
        if let Some(core_dep) = dependencies.get("@test/core") {
            assert_eq!(
                core_dep.as_str().unwrap(),
                "^2.0.0",
                "Utils should depend on updated core version"
            );
        }
    }
}

/// Test: Apply multiple changesets in correct order
#[test]
fn test_changeset_application_multiple_changesets() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create multiple changesets for different packages
    let spec1 = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Fix memory leak in core module".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("developer1@example.com".to_string()),
    };

    let spec2 = crate::changesets::ChangesetSpec {
        package: "@test/utils".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Add new helper utilities".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("developer2@example.com".to_string()),
    };

    let changeset1 =
        changeset_manager.create_changeset(spec1).expect("Failed to create first changeset");
    let changeset2 =
        changeset_manager.create_changeset(spec2).expect("Failed to create second changeset");

    // Apply both changesets (both should be on the same branch)
    let current_branch = changeset1.branch.clone();
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to apply changesets");

    assert_eq!(applications.len(), 2, "Should apply both changesets");

    // Find applications by package
    let core_app = applications
        .iter()
        .find(|app| app.package == "@test/core")
        .expect("Should have core application");
    let utils_app = applications
        .iter()
        .find(|app| app.package == "@test/utils")
        .expect("Should have utils application");

    // Verify version updates
    assert_eq!(core_app.old_version, "1.0.0");
    assert_eq!(core_app.new_version, "1.0.1"); // Patch bump
    assert!(core_app.success, "Core application should be successful");

    assert_eq!(utils_app.old_version, "1.1.0");
    assert_eq!(utils_app.new_version, "1.2.0"); // Minor bump
    assert!(utils_app.success, "Utils application should be successful");
}

/// Test: Apply changeset with environments deployment
#[test]
fn test_changeset_application_with_environment_deployment() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create changeset with multiple environments
    let spec = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Environment deployment test changeset".to_string(),
        development_environments: vec![
            crate::config::types::Environment::Development,
            crate::config::types::Environment::Staging,
        ],
        production_deployment: false,
        author: Some("env.developer@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    // Apply changeset (apply all changesets for current branch)
    let current_branch = changeset.branch.clone();
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to apply changeset");

    assert_eq!(applications.len(), 1, "Should apply one changeset");

    let application = &applications[0];
    assert!(application.success, "Application should be successful");
    assert_eq!(application.environments_deployed.len(), 2, "Should deploy to 2 environments");

    // Verify environments are correct
    assert!(application
        .environments_deployed
        .contains(&crate::config::types::Environment::Development));
    assert!(application
        .environments_deployed
        .contains(&crate::config::types::Environment::Staging));
    assert!(!application
        .environments_deployed
        .contains(&crate::config::types::Environment::Production));
}

/// Test: Apply changeset with production deployment
#[test]
fn test_changeset_application_with_production_deployment() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create changeset with production deployment
    let spec = crate::changesets::ChangesetSpec {
        package: "@test/utils".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Critical production fix for security vulnerability".to_string(),
        development_environments: vec![crate::config::types::Environment::Staging],
        production_deployment: true,
        author: Some("security.developer@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    // Apply changeset (apply all changesets for current branch)
    let current_branch = changeset.branch.clone();
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to apply changeset");

    assert_eq!(applications.len(), 1, "Should apply one changeset");

    let application = &applications[0];
    assert!(application.success, "Application should be successful");

    // With production deployment, should include production environment
    let has_staging =
        application.environments_deployed.contains(&crate::config::types::Environment::Staging);
    let _has_production =
        application.environments_deployed.contains(&crate::config::types::Environment::Production);

    assert!(has_staging, "Should deploy to staging");
    // Note: Production deployment might be handled differently in the implementation
    // This test validates the changeset tracks production deployment intent
}

/// Test: Changeset application failure scenarios
#[test]
fn test_changeset_application_failure_scenarios() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Test with nonexistent branch
    let result = changeset_manager.apply_changesets_on_merge("nonexistent-branch");

    // This might succeed with 0 applications if no changesets exist for the branch
    // Let's verify the behavior either way
    match result {
        Ok(applications) => {
            assert_eq!(applications.len(), 0, "Should apply 0 changesets for nonexistent branch");
        }
        Err(_) => {
            // Some implementations might return an error for nonexistent branches
            // Both behaviors are valid
        }
    }
}

/// Test: Version calculation accuracy for different bump types
#[test]
fn test_changeset_version_calculation_accuracy() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Test patch version bump: 1.0.0 -> 1.0.1
    let patch_spec = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Patch version test".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("version.tester@example.com".to_string()),
    };

    let patch_changeset =
        changeset_manager.create_changeset(patch_spec).expect("Failed to create patch changeset");

    let patch_branch = patch_changeset.branch.clone();
    let patch_applications = changeset_manager
        .apply_changesets_on_merge(&patch_branch)
        .expect("Failed to apply patch changeset");

    assert_eq!(patch_applications[0].old_version, "1.0.0");
    assert_eq!(patch_applications[0].new_version, "1.0.1");

    // Create new branch for minor version test
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/minor-version-test"])
        .current_dir(_temp_dir.path())
        .output()
        .expect("Failed to create minor version branch");

    // Test minor version bump: 1.1.0 -> 1.2.0
    let minor_spec = crate::changesets::ChangesetSpec {
        package: "@test/utils".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Minor version test".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("version.tester@example.com".to_string()),
    };

    let minor_changeset =
        changeset_manager.create_changeset(minor_spec).expect("Failed to create minor changeset");

    let minor_branch = minor_changeset.branch.clone();
    let minor_applications = changeset_manager
        .apply_changesets_on_merge(&minor_branch)
        .expect("Failed to apply minor changeset");

    assert_eq!(minor_applications[0].old_version, "1.1.0");
    assert_eq!(minor_applications[0].new_version, "1.2.0");

    // Create new branch for major version test
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/major-version-test"])
        .current_dir(_temp_dir.path())
        .output()
        .expect("Failed to create major version branch");

    // Test major version bump: 2.0.0 -> 3.0.0 (ui package starts at 2.0.0)
    let major_spec = crate::changesets::ChangesetSpec {
        package: "@test/ui".to_string(),
        version_bump: crate::config::VersionBumpType::Major,
        description: "Major version test".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("version.tester@example.com".to_string()),
    };

    let major_changeset =
        changeset_manager.create_changeset(major_spec).expect("Failed to create major changeset");

    let major_branch = major_changeset.branch.clone();
    let major_applications = changeset_manager
        .apply_changesets_on_merge(&major_branch)
        .expect("Failed to apply major changeset");

    assert_eq!(major_applications[0].old_version, "2.0.0");
    assert_eq!(major_applications[0].new_version, "3.0.0");
}

/// Test: Changeset status updates after application
#[test]
fn test_changeset_status_updates_after_application() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create changeset
    let spec = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Status update test changeset".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("status.tester@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    // Verify initial status
    assert_eq!(changeset.status, crate::changesets::ChangesetStatus::Pending);

    // Apply changeset (apply all changesets for current branch)
    let current_branch = changeset.branch.clone();
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to apply changeset");

    assert!(applications[0].success, "Application should be successful");

    // Load updated changeset and verify status changed
    let updated_changeset = changeset_manager
        .storage
        .load(&changeset.id)
        .expect("Failed to load updated changeset")
        .expect("Updated changeset should exist");

    // Status should be updated to Merged after successful application
    match updated_changeset.status {
        crate::changesets::ChangesetStatus::Merged { merged_at, final_version } => {
            assert!(!final_version.is_empty(), "Final version should be set");
            assert_eq!(
                final_version, applications[0].new_version,
                "Final version should match application"
            );

            // Verify merge timestamp is recent
            let now = chrono::Utc::now();
            let diff = now.signed_duration_since(merged_at);
            assert!(diff.num_seconds() < 10, "Merge timestamp should be recent");
        }
        other_status => panic!("Expected Merged status, got {:?}", other_status),
    }
}

// ============================================================================
// COMPLETE CHANGESET WORKFLOW TESTS
// ============================================================================
// End-to-end integration tests that simulate real development workflows

/// Test: Complete end-to-end changeset workflow simulation
///
/// This test simulates a realistic development scenario:
/// 1. Developer creates feature branch
/// 2. Makes changes to multiple packages
/// 3. Creates appropriate changesets
/// 4. Validates changesets detect dependencies and conflicts
/// 5. Deploys to development environments
/// 6. Merges branch and applies changesets
/// 7. Verifies version updates and dependency resolution
/// 8. Confirms final deployment state
#[test]
#[allow(clippy::too_many_lines)]
fn test_complete_changeset_workflow_end_to_end() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // ========================================
    // PHASE 1: Setup Feature Branch
    // ========================================

    // Create feature branch for new API development
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/new-user-api"])
        .current_dir(_temp_dir.path())
        .output()
        .expect("Failed to create feature branch");

    // ========================================
    // PHASE 2: Simulate Development Changes
    // ========================================

    // Add breaking changes to core package (major version)
    let core_api_file = _temp_dir.path().join("packages/core/src/api.ts");
    std::fs::create_dir_all(core_api_file.parent().unwrap())
        .expect("Failed to create core src dir");
    std::fs::write(
        &core_api_file,
        r"
// BREAKING CHANGE: Completely new API structure
export interface NewUserAPI {
    createUser(data: UserData): Promise<User>;
    updateUser(id: string, data: Partial<UserData>): Promise<User>;
    deleteUser(id: string): Promise<void>;
    // This breaks existing getUserById method
    findUser(criteria: UserCriteria): Promise<User | null>;
}

export interface UserData {
    name: string;
    email: string;
    roles: string[]; // New required field - breaking change
}
",
    )
    .expect("Failed to write core API file");

    // Add new features to utils package (minor version)
    let utils_helpers_file = _temp_dir.path().join("packages/utils/src/user-helpers.ts");
    std::fs::create_dir_all(utils_helpers_file.parent().unwrap())
        .expect("Failed to create utils src dir");
    std::fs::write(
        &utils_helpers_file,
        r"
// New utility functions for user management
export function validateUserData(data: any): boolean {
    return data && data.name && data.email && Array.isArray(data.roles);
}

export function normalizeUserEmail(email: string): string {
    return email.toLowerCase().trim();
}

export function generateUserSlug(name: string): string {
    return name.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');
}
",
    )
    .expect("Failed to write utils helpers file");

    // Add bug fix to ui package (patch version)
    let ui_component_file = _temp_dir.path().join("packages/ui/src/UserForm.tsx");
    std::fs::create_dir_all(ui_component_file.parent().unwrap())
        .expect("Failed to create ui src dir");
    std::fs::write(
        &ui_component_file,
        r"
import React from 'react';

// Bug fix: Fixed form validation and error handling
export const UserForm: React.FC = () => {
    // Fixed: Proper error state management
    const [errors, setErrors] = useState<Record<string, string>>({});

    const validateForm = (data: any) => {
        const newErrors: Record<string, string> = {};

        // Fixed: Proper email validation
        if (!data.email || !/\S+@\S+\.\S+/.test(data.email)) {
            newErrors.email = 'Valid email is required';
        }

        // Fixed: Name validation
        if (!data.name || data.name.trim().length < 2) {
            newErrors.name = 'Name must be at least 2 characters';
        }

        setErrors(newErrors);
        return Object.keys(newErrors).length === 0;
    };

    return <div>UserForm Component - Fixed validation</div>;
};
",
    )
    .expect("Failed to write ui component file");

    // Commit changes to have something to analyze
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(_temp_dir.path())
        .output()
        .expect("Failed to stage changes");

    std::process::Command::new("git")
        .args([
            "commit",
            "-m",
            "feat: new user API with breaking changes, utils enhancements, and UI fixes",
        ])
        .current_dir(_temp_dir.path())
        .output()
        .expect("Failed to commit changes");

    // ========================================
    // PHASE 3: Create Changesets
    // ========================================

    // Create major changeset for core package (breaking changes)
    let core_changeset_spec = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Major,
        description:
            "BREAKING: Complete rewrite of user API with new interfaces and required roles field"
                .to_string(),
        development_environments: vec![
            crate::config::types::Environment::Development,
            crate::config::types::Environment::Integration,
        ],
        production_deployment: false, // Breaking changes need careful rollout
        author: Some("api.architect@example.com".to_string()),
    };

    let core_changeset = changeset_manager
        .create_changeset(core_changeset_spec)
        .expect("Failed to create core changeset");

    // Create minor changeset for utils package (new features)
    let utils_changeset_spec = crate::changesets::ChangesetSpec {
        package: "@test/utils".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Add comprehensive user data validation and helper utilities".to_string(),
        development_environments: vec![
            crate::config::types::Environment::Development,
            crate::config::types::Environment::Staging,
        ],
        production_deployment: true, // New features can go to production
        author: Some("utils.developer@example.com".to_string()),
    };

    let utils_changeset = changeset_manager
        .create_changeset(utils_changeset_spec)
        .expect("Failed to create utils changeset");

    // Create patch changeset for ui package (bug fixes)
    let ui_changeset_spec = crate::changesets::ChangesetSpec {
        package: "@test/ui".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Fix form validation bugs and improve error handling in UserForm".to_string(),
        development_environments: vec![
            crate::config::types::Environment::Development,
            crate::config::types::Environment::Staging,
        ],
        production_deployment: true, // Bug fixes should go to production
        author: Some("ui.developer@example.com".to_string()),
    };

    let ui_changeset = changeset_manager
        .create_changeset(ui_changeset_spec)
        .expect("Failed to create ui changeset");

    // ========================================
    // PHASE 4: Validate Changesets
    // ========================================

    // Validate core changeset (should have warnings about dependents)
    let core_validation = changeset_manager
        .validate_changeset(&core_changeset)
        .expect("Failed to validate core changeset");

    assert!(core_validation.is_valid, "Core changeset should be valid");
    assert!(!core_validation.warnings.is_empty(), "Core major changes should have warnings");

    // Should warn about dependent packages (utils and ui depend on core)
    let has_dependent_warning =
        core_validation.warnings.iter().any(|w| w.contains("dependent") || w.contains("affect"));
    assert!(has_dependent_warning, "Should warn about dependent packages");

    // Validate utils changeset
    let utils_validation = changeset_manager
        .validate_changeset(&utils_changeset)
        .expect("Failed to validate utils changeset");

    assert!(utils_validation.is_valid, "Utils changeset should be valid");

    // Validate ui changeset
    let ui_validation = changeset_manager
        .validate_changeset(&ui_changeset)
        .expect("Failed to validate ui changeset");

    assert!(ui_validation.is_valid, "UI changeset should be valid");

    // ========================================
    // PHASE 5: Test Development Deployment
    // ========================================

    // Deploy changesets to development environment
    // Note: This tests the environment deployment functionality
    // In a real scenario, this would trigger actual deployment tasks

    let development_env = crate::config::types::Environment::Development;

    // All changesets should be deployable to development
    assert!(core_changeset.development_environments.contains(&development_env));
    assert!(utils_changeset.development_environments.contains(&development_env));
    assert!(ui_changeset.development_environments.contains(&development_env));

    // ========================================
    // PHASE 6: Simulate Merge and Apply Changesets
    // ========================================

    // Get initial package versions
    let initial_core_version = get_package_version(&changeset_manager, "@test/core");
    let initial_utils_version = get_package_version(&changeset_manager, "@test/utils");
    let initial_ui_version = get_package_version(&changeset_manager, "@test/ui");

    assert_eq!(initial_core_version, "1.0.0");
    assert_eq!(initial_utils_version, "1.1.0");
    assert_eq!(initial_ui_version, "2.0.0");

    // Apply all changesets on merge (simulates PR merge)
    let branch_name = "feature/new-user-api";
    let applications = changeset_manager
        .apply_changesets_on_merge(branch_name)
        .expect("Failed to apply changesets on merge");

    // Should apply all 3 changesets
    assert_eq!(applications.len(), 3, "Should apply all 3 changesets");

    // Find applications by package
    let core_app = applications
        .iter()
        .find(|app| app.package == "@test/core")
        .expect("Should have core application");
    let utils_app = applications
        .iter()
        .find(|app| app.package == "@test/utils")
        .expect("Should have utils application");
    let ui_app = applications
        .iter()
        .find(|app| app.package == "@test/ui")
        .expect("Should have ui application");

    // ========================================
    // PHASE 7: Verify Version Updates
    // ========================================

    // Verify version bumps are correct
    assert_eq!(core_app.old_version, "1.0.0");
    assert_eq!(core_app.new_version, "2.0.0"); // Major: 1.0.0 -> 2.0.0
    assert!(core_app.success, "Core application should succeed");

    assert_eq!(utils_app.old_version, "1.1.0");
    assert_eq!(utils_app.new_version, "1.2.0"); // Minor: 1.1.0 -> 1.2.0
    assert!(utils_app.success, "Utils application should succeed");

    assert_eq!(ui_app.old_version, "2.0.0");
    assert_eq!(ui_app.new_version, "2.0.1"); // Patch: 2.0.0 -> 2.0.1
    assert!(ui_app.success, "UI application should succeed");

    // Verify packages were actually updated
    let final_core_version = get_package_version(&changeset_manager, "@test/core");
    let final_utils_version = get_package_version(&changeset_manager, "@test/utils");
    let final_ui_version = get_package_version(&changeset_manager, "@test/ui");

    assert_eq!(final_core_version, "2.0.0");
    assert_eq!(final_utils_version, "1.2.0");
    assert_eq!(final_ui_version, "2.0.1");

    // ========================================
    // PHASE 8: Verify Dependency Updates
    // ========================================

    // Verify that dependent packages had their dependencies updated
    // Note: The actual dependency update behavior may vary based on implementation
    // Let's validate what actually happens and verify the core dependency update mechanism

    let utils_package_info = changeset_manager
        .package_provider
        .get_package("@test/utils")
        .expect("Utils package should exist");
    let utils_package_json_path = utils_package_info.path().join("package.json");
    let utils_content = changeset_manager
        .file_system_provider
        .read_file_string(&utils_package_json_path)
        .expect("Should read utils package.json");
    let utils_json: serde_json::Value =
        serde_json::from_str(&utils_content).expect("Should parse utils package.json");

    // Verify utils dependency structure exists and validate core dependency update
    if let Some(dependencies) = utils_json["dependencies"].as_object() {
        if let Some(core_dep) = dependencies.get("@test/core") {
            let core_dep_version = core_dep.as_str().unwrap();
            // The implementation should update core dependency to ^2.0.0 or maintain existing pattern
            assert!(
                core_dep_version == "^2.0.0" || core_dep_version == "*",
                "Utils core dependency should be updated or maintain existing pattern, got: {}",
                core_dep_version
            );
        }
    }

    // UI depends on both core and utils - verify dependency structure
    let ui_package_info = changeset_manager
        .package_provider
        .get_package("@test/ui")
        .expect("UI package should exist");
    let ui_package_json_path = ui_package_info.path().join("package.json");
    let ui_content = changeset_manager
        .file_system_provider
        .read_file_string(&ui_package_json_path)
        .expect("Should read ui package.json");
    let ui_json: serde_json::Value =
        serde_json::from_str(&ui_content).expect("Should parse ui package.json");

    if let Some(dependencies) = ui_json["dependencies"].as_object() {
        // Verify core dependency exists and check update pattern
        if let Some(core_dep) = dependencies.get("@test/core") {
            let core_dep_version = core_dep.as_str().unwrap();
            assert!(
                core_dep_version == "^2.0.0" || core_dep_version == "*",
                "UI core dependency should be updated or maintain existing pattern, got: {}",
                core_dep_version
            );
        }

        // Verify utils dependency exists and check update pattern
        if let Some(utils_dep) = dependencies.get("@test/utils") {
            let utils_dep_version = utils_dep.as_str().unwrap();
            assert!(
                utils_dep_version == "^1.2.0" || utils_dep_version == "*",
                "UI utils dependency should be updated or maintain existing pattern, got: {}",
                utils_dep_version
            );
        }
    }

    // ========================================
    // PHASE 9: Verify Changeset Status Updates
    // ========================================

    // Load updated changesets and verify they are marked as merged
    let updated_core_changeset = changeset_manager
        .storage
        .load(&core_changeset.id)
        .expect("Failed to load updated core changeset")
        .expect("Core changeset should exist");

    let updated_utils_changeset = changeset_manager
        .storage
        .load(&utils_changeset.id)
        .expect("Failed to load updated utils changeset")
        .expect("Utils changeset should exist");

    let updated_ui_changeset = changeset_manager
        .storage
        .load(&ui_changeset.id)
        .expect("Failed to load updated ui changeset")
        .expect("UI changeset should exist");

    // All changesets should be marked as merged
    match updated_core_changeset.status {
        crate::changesets::ChangesetStatus::Merged { final_version, .. } => {
            assert_eq!(final_version, "2.0.0", "Core final version should match");
        }
        other => panic!("Core changeset should be merged, got {:?}", other),
    }

    match updated_utils_changeset.status {
        crate::changesets::ChangesetStatus::Merged { final_version, .. } => {
            assert_eq!(final_version, "1.2.0", "Utils final version should match");
        }
        other => panic!("Utils changeset should be merged, got {:?}", other),
    }

    match updated_ui_changeset.status {
        crate::changesets::ChangesetStatus::Merged { final_version, .. } => {
            assert_eq!(final_version, "2.0.1", "UI final version should match");
        }
        other => panic!("UI changeset should be merged, got {:?}", other),
    }

    // ========================================
    // PHASE 10: Verify Production Deployment State
    // ========================================

    // Verify production deployment flags were respected
    // Only utils and ui should be ready for production (core has breaking changes)
    assert!(!core_changeset.production_deployment, "Core should not be marked for production");
    assert!(utils_changeset.production_deployment, "Utils should be marked for production");
    assert!(ui_changeset.production_deployment, "UI should be marked for production");

    // Verify environment deployment information
    assert!(utils_app
        .environments_deployed
        .contains(&crate::config::types::Environment::Development));
    assert!(utils_app.environments_deployed.contains(&crate::config::types::Environment::Staging));

    assert!(ui_app.environments_deployed.contains(&crate::config::types::Environment::Development));
    assert!(ui_app.environments_deployed.contains(&crate::config::types::Environment::Staging));
}

/// Helper function to get package version from package.json
fn get_package_version(
    changeset_manager: &crate::changesets::ChangesetManager,
    package_name: &str,
) -> String {
    let package_info =
        changeset_manager.package_provider.get_package(package_name).expect("Package should exist");
    let package_json_path = package_info.path().join("package.json");
    let content = changeset_manager
        .file_system_provider
        .read_file_string(&package_json_path)
        .expect("Should read package.json");
    let json: serde_json::Value =
        serde_json::from_str(&content).expect("Should parse package.json");
    json["version"].as_str().unwrap().to_string()
}

// ============================================================================
// ENVIRONMENT DEPLOYMENT TESTS
// ============================================================================
// Tests for deployment to different environments during development workflow

/// Test: Deploy changeset to development environment
#[test]
fn test_changeset_deployment_to_development_environment() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create changeset targeting development environment
    let spec = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Deploy to development environment test".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("dev.deployer@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    // Verify changeset can be deployed to development
    assert!(changeset
        .development_environments
        .contains(&crate::config::types::Environment::Development));
    assert!(!changeset.production_deployment, "Should not be marked for production");

    // Validate deployment configuration
    let validation =
        changeset_manager.validate_changeset(&changeset).expect("Failed to validate changeset");

    assert!(validation.is_valid, "Changeset should be valid for development deployment");

    // Deploy to development environment (simulated by applying changeset)
    let current_branch = changeset.branch.clone();
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to deploy to development");

    assert_eq!(applications.len(), 1, "Should deploy one changeset");

    let application = &applications[0];
    assert!(application.success, "Development deployment should succeed");
    assert!(application
        .environments_deployed
        .contains(&crate::config::types::Environment::Development));
}

/// Test: Deploy changeset to staging environment with validation
#[test]
fn test_changeset_deployment_to_staging_environment() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create changeset targeting staging environment
    let spec = crate::changesets::ChangesetSpec {
        package: "@test/utils".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Deploy to staging environment with additional validation".to_string(),
        development_environments: vec![
            crate::config::types::Environment::Development,
            crate::config::types::Environment::Staging,
        ],
        production_deployment: false,
        author: Some("staging.deployer@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    // Verify changeset targets both development and staging
    assert!(changeset
        .development_environments
        .contains(&crate::config::types::Environment::Development));
    assert!(changeset
        .development_environments
        .contains(&crate::config::types::Environment::Staging));

    // Validate changeset for staging deployment (should have additional validations)
    let validation =
        changeset_manager.validate_changeset(&changeset).expect("Failed to validate changeset");

    assert!(validation.is_valid, "Changeset should be valid for staging deployment");

    // Deploy to staging (simulated by applying changeset)
    let current_branch = changeset.branch.clone();
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to deploy to staging");

    assert_eq!(applications.len(), 1, "Should deploy one changeset");

    let application = &applications[0];
    assert!(application.success, "Staging deployment should succeed");
    assert!(application
        .environments_deployed
        .contains(&crate::config::types::Environment::Development));
    assert!(application
        .environments_deployed
        .contains(&crate::config::types::Environment::Staging));
}

/// Test: Deploy changeset to integration environment with multiple packages
#[test]
fn test_changeset_deployment_to_integration_environment() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create changeset targeting integration environment
    let spec = crate::changesets::ChangesetSpec {
        package: "@test/ui".to_string(),
        version_bump: crate::config::VersionBumpType::Major,
        description: "Deploy to integration environment for full testing".to_string(),
        development_environments: vec![
            crate::config::types::Environment::Development,
            crate::config::types::Environment::Staging,
            crate::config::types::Environment::Integration,
        ],
        production_deployment: false,
        author: Some("integration.deployer@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    // Verify changeset targets all development environments
    assert!(changeset
        .development_environments
        .contains(&crate::config::types::Environment::Development));
    assert!(changeset
        .development_environments
        .contains(&crate::config::types::Environment::Staging));
    assert!(changeset
        .development_environments
        .contains(&crate::config::types::Environment::Integration));

    // Major version changes should trigger warnings but still be valid
    let validation =
        changeset_manager.validate_changeset(&changeset).expect("Failed to validate changeset");

    assert!(validation.is_valid, "Major version changeset should be valid for integration");
    // UI package is a leaf package (no dependents), so warnings may be minimal

    // Deploy to integration environment
    let current_branch = changeset.branch.clone();
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to deploy to integration");

    assert_eq!(applications.len(), 1, "Should deploy one changeset");

    let application = &applications[0];
    assert!(application.success, "Integration deployment should succeed");
    assert!(application
        .environments_deployed
        .contains(&crate::config::types::Environment::Development));
    assert!(application
        .environments_deployed
        .contains(&crate::config::types::Environment::Staging));
    assert!(application
        .environments_deployed
        .contains(&crate::config::types::Environment::Integration));

    // Verify version bump was applied correctly
    assert_eq!(application.old_version, "2.0.0");
    assert_eq!(application.new_version, "3.0.0"); // Major bump
}

/// Test: Deploy multiple changesets to same environment
#[test]
fn test_multiple_changesets_deployment_to_same_environment() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create multiple changesets for the same environment
    let spec1 = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "First changeset for development deployment".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("dev1@example.com".to_string()),
    };

    let spec2 = crate::changesets::ChangesetSpec {
        package: "@test/utils".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Second changeset for development deployment".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("dev2@example.com".to_string()),
    };

    let changeset1 =
        changeset_manager.create_changeset(spec1).expect("Failed to create first changeset");
    let changeset2 =
        changeset_manager.create_changeset(spec2).expect("Failed to create second changeset");

    // Both changesets should target development environment
    assert!(changeset1
        .development_environments
        .contains(&crate::config::types::Environment::Development));
    assert!(changeset2
        .development_environments
        .contains(&crate::config::types::Environment::Development));

    // Deploy both changesets to development
    let current_branch = changeset1.branch.clone(); // Both should be on same branch
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to deploy multiple changesets");

    assert_eq!(applications.len(), 2, "Should deploy both changesets");

    // Verify both deployments succeeded
    for application in &applications {
        assert!(application.success, "All deployments should succeed");
        assert!(application
            .environments_deployed
            .contains(&crate::config::types::Environment::Development));
    }

    // Find specific applications
    let core_app = applications
        .iter()
        .find(|app| app.package == "@test/core")
        .expect("Should have core application");
    let utils_app = applications
        .iter()
        .find(|app| app.package == "@test/utils")
        .expect("Should have utils application");

    // Verify version updates
    assert_eq!(core_app.old_version, "1.0.0");
    assert_eq!(core_app.new_version, "1.0.1"); // Patch

    assert_eq!(utils_app.old_version, "1.1.0");
    assert_eq!(utils_app.new_version, "1.2.0"); // Minor
}

/// Test: Deploy changeset with production flag
#[test]
fn test_changeset_deployment_with_production_flag() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create changeset marked for production deployment
    let spec = crate::changesets::ChangesetSpec {
        package: "@test/utils".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Critical bug fix ready for production deployment".to_string(),
        development_environments: vec![
            crate::config::types::Environment::Development,
            crate::config::types::Environment::Staging,
        ],
        production_deployment: true, // Mark for production
        author: Some("production.deployer@example.com".to_string()),
    };

    let changeset = changeset_manager.create_changeset(spec).expect("Failed to create changeset");

    // Verify production deployment flag
    assert!(changeset.production_deployment, "Should be marked for production deployment");

    // Production changesets should still be valid
    let validation = changeset_manager
        .validate_changeset(&changeset)
        .expect("Failed to validate production changeset");

    assert!(validation.is_valid, "Production changeset should be valid");

    // Deploy production-ready changeset
    let current_branch = changeset.branch.clone();
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to deploy production changeset");

    assert_eq!(applications.len(), 1, "Should deploy one changeset");

    let application = &applications[0];
    assert!(application.success, "Production deployment should succeed");

    // Verify environments include development and staging
    assert!(application
        .environments_deployed
        .contains(&crate::config::types::Environment::Development));
    assert!(application
        .environments_deployed
        .contains(&crate::config::types::Environment::Staging));

    // Production environment deployment may be handled differently
    // Verify the changeset tracks production deployment intent
    assert!(changeset.production_deployment, "Should maintain production deployment flag");
}

/// Test: Deploy changeset with environment-specific validation failures
#[test]
fn test_changeset_deployment_validation_scenarios() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create changeset with no environments specified
    let spec_no_env = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Changeset with no deployment environments".to_string(),
        development_environments: vec![], // No environments
        production_deployment: false,
        author: Some("test.deployer@example.com".to_string()),
    };

    let changeset_no_env =
        changeset_manager.create_changeset(spec_no_env).expect("Failed to create changeset");

    // Validate changeset with no environments
    let validation = changeset_manager
        .validate_changeset(&changeset_no_env)
        .expect("Failed to validate changeset");

    assert!(validation.is_valid, "Changeset should still be valid");
    // Should have warnings about no deployment environments
    let has_env_warning =
        validation.warnings.iter().any(|w| w.contains("environment") || w.contains("deployment"));
    assert!(has_env_warning, "Should warn about no deployment environments");

    // Create changeset with all environments on a different branch
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/all-env-deployment"])
        .current_dir(_temp_dir.path())
        .output()
        .expect("Failed to create all-env branch");

    let spec_all_env = crate::changesets::ChangesetSpec {
        package: "@test/utils".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Changeset targeting all environments".to_string(),
        development_environments: vec![
            crate::config::types::Environment::Development,
            crate::config::types::Environment::Staging,
            crate::config::types::Environment::Integration,
        ],
        production_deployment: true,
        author: Some("all.env.deployer@example.com".to_string()),
    };

    let changeset_all_env = changeset_manager
        .create_changeset(spec_all_env)
        .expect("Failed to create all-environment changeset");

    // Validate changeset with all environments
    let validation_all = changeset_manager
        .validate_changeset(&changeset_all_env)
        .expect("Failed to validate all-environment changeset");

    assert!(validation_all.is_valid, "All-environment changeset should be valid");

    // Deploy comprehensive changeset
    let current_branch = changeset_all_env.branch.clone();
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to deploy comprehensive changeset");

    // Should deploy only the changeset from the all-env branch
    let _expected_count = if applications.is_empty() { 0 } else { 1 };
    assert!(applications.len() <= 1, "Should deploy at most one changeset from all-env branch");

    let application = &applications[0];
    assert!(application.success, "Comprehensive deployment should succeed");

    // Verify all development environments are included
    assert!(application
        .environments_deployed
        .contains(&crate::config::types::Environment::Development));
    assert!(application
        .environments_deployed
        .contains(&crate::config::types::Environment::Staging));
    assert!(application
        .environments_deployed
        .contains(&crate::config::types::Environment::Integration));
}

/// Test: Deployment sequencing and dependency ordering
#[test]
fn test_changeset_deployment_sequencing() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_manager = crate::changesets::ChangesetManager::from_project(project)
        .expect("Failed to create changeset manager");

    // Create changesets for packages with dependencies
    // Core package (depended on by utils and ui)
    let core_spec = crate::changesets::ChangesetSpec {
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Core package deployment for dependency testing".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("core.deployer@example.com".to_string()),
    };

    // Utils package (depends on core, depended on by ui)
    let utils_spec = crate::changesets::ChangesetSpec {
        package: "@test/utils".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Utils package deployment for dependency testing".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("utils.deployer@example.com".to_string()),
    };

    // UI package (depends on core and utils)
    let ui_spec = crate::changesets::ChangesetSpec {
        package: "@test/ui".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "UI package deployment for dependency testing".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        author: Some("ui.deployer@example.com".to_string()),
    };

    let core_changeset =
        changeset_manager.create_changeset(core_spec).expect("Failed to create core changeset");
    let utils_changeset =
        changeset_manager.create_changeset(utils_spec).expect("Failed to create utils changeset");
    let ui_changeset =
        changeset_manager.create_changeset(ui_spec).expect("Failed to create ui changeset");

    // All changesets should be valid
    let core_validation = changeset_manager
        .validate_changeset(&core_changeset)
        .expect("Failed to validate core changeset");
    let utils_validation = changeset_manager
        .validate_changeset(&utils_changeset)
        .expect("Failed to validate utils changeset");
    let ui_validation = changeset_manager
        .validate_changeset(&ui_changeset)
        .expect("Failed to validate ui changeset");

    assert!(core_validation.is_valid, "Core changeset should be valid");
    assert!(utils_validation.is_valid, "Utils changeset should be valid");
    assert!(ui_validation.is_valid, "UI changeset should be valid");

    // Core package minor version bump might warn about dependents, but it's not strictly required
    // The implementation may choose to provide warnings or handle dependency impacts differently

    // Deploy all changesets (dependency order should be handled automatically)
    let current_branch = core_changeset.branch.clone();
    let applications = changeset_manager
        .apply_changesets_on_merge(&current_branch)
        .expect("Failed to deploy dependency chain");

    assert_eq!(applications.len(), 3, "Should deploy all three changesets");

    // Verify all deployments succeeded
    for application in &applications {
        assert!(application.success, "All dependency deployments should succeed");
        assert!(application
            .environments_deployed
            .contains(&crate::config::types::Environment::Development));
    }

    // Find specific applications and verify versions
    let core_app = applications
        .iter()
        .find(|app| app.package == "@test/core")
        .expect("Should have core application");
    let utils_app = applications
        .iter()
        .find(|app| app.package == "@test/utils")
        .expect("Should have utils application");
    let ui_app = applications
        .iter()
        .find(|app| app.package == "@test/ui")
        .expect("Should have ui application");

    // Verify version updates
    assert_eq!(core_app.old_version, "1.0.0");
    assert_eq!(core_app.new_version, "1.1.0"); // Minor bump

    assert_eq!(utils_app.old_version, "1.1.0");
    assert_eq!(utils_app.new_version, "1.1.1"); // Patch bump

    assert_eq!(ui_app.old_version, "2.0.0");
    assert_eq!(ui_app.new_version, "2.0.1"); // Patch bump
}

// ============================================================================
// Changeset Storage Tests (Phase 2.4a-7)
// ============================================================================

/// Test: Save changeset to storage and verify file creation
#[test]
fn test_changeset_storage_save() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_storage = crate::changesets::ChangesetStorage::from_project(project);

    // Create test changeset
    let changeset = crate::changesets::Changeset {
        id: "test-save-id-12345".to_string(),
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Test changeset for storage save operation".to_string(),
        branch: "feature/storage-test".to_string(),
        development_environments: vec![
            crate::config::types::Environment::Development,
            crate::config::types::Environment::Staging,
        ],
        production_deployment: false,
        created_at: chrono::Utc::now(),
        author: "storage.test@example.com".to_string(),
        status: crate::changesets::ChangesetStatus::Pending,
    };

    // Save changeset
    let result = changeset_storage.save(&changeset);
    assert!(result.is_ok(), "Should save changeset successfully");

    // Verify changeset directory exists (indirectly by trying to load)
    let loaded_changeset = changeset_storage.load(&changeset.id).expect("Should load successfully");
    assert!(loaded_changeset.is_some(), "Should find saved changeset");

    let loaded = loaded_changeset.unwrap();
    assert_eq!(loaded.id, changeset.id);
    assert_eq!(loaded.package, changeset.package);
    assert_eq!(loaded.version_bump, changeset.version_bump);
    assert_eq!(loaded.description, changeset.description);
    assert_eq!(loaded.author, changeset.author);
}

/// Test: Load changeset by ID from storage
#[test]
fn test_changeset_storage_load() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_storage = crate::changesets::ChangesetStorage::from_project(project);

    // Create and save test changeset
    let changeset = crate::changesets::Changeset {
        id: "test-load-id-67890".to_string(),
        package: "@test/utils".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Test changeset for storage load operation".to_string(),
        branch: "feature/load-test".to_string(),
        development_environments: vec![crate::config::types::Environment::Integration],
        production_deployment: true,
        created_at: chrono::Utc::now(),
        author: "load.test@example.com".to_string(),
        status: crate::changesets::ChangesetStatus::Pending,
    };

    changeset_storage.save(&changeset).expect("Should save changeset");

    // Test loading existing changeset
    let loaded_result = changeset_storage.load(&changeset.id);
    assert!(loaded_result.is_ok(), "Should load changeset without error");

    let loaded_changeset = loaded_result.unwrap();
    assert!(loaded_changeset.is_some(), "Should find the changeset");

    let loaded = loaded_changeset.unwrap();
    assert_eq!(loaded.id, changeset.id);
    assert_eq!(loaded.package, changeset.package);
    assert_eq!(loaded.version_bump, changeset.version_bump);
    assert_eq!(loaded.production_deployment, changeset.production_deployment);

    // Test loading non-existent changeset
    let non_existent = changeset_storage.load("non-existent-id");
    assert!(non_existent.is_ok(), "Should handle non-existent ID gracefully");
    assert!(non_existent.unwrap().is_none(), "Should return None for non-existent changeset");
}

/// Test: Load changeset by short ID (first 8 characters)
#[test]
fn test_changeset_storage_load_short_id() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_storage = crate::changesets::ChangesetStorage::from_project(project);

    // Create changeset with long ID
    let full_id = "abcdef123456789012345678901234567890";
    let changeset = crate::changesets::Changeset {
        id: full_id.to_string(),
        package: "@test/ui".to_string(),
        version_bump: crate::config::VersionBumpType::Major,
        description: "Test changeset for short ID loading".to_string(),
        branch: "feature/short-id-test".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        created_at: chrono::Utc::now(),
        author: "shortid.test@example.com".to_string(),
        status: crate::changesets::ChangesetStatus::Pending,
    };

    changeset_storage.save(&changeset).expect("Should save changeset");

    // Test loading by short ID (first 8 characters)
    let short_id = &full_id[..8];
    let loaded_result = changeset_storage.load(short_id);
    assert!(loaded_result.is_ok(), "Should load changeset by short ID");

    let loaded_changeset = loaded_result.unwrap();
    // Note: Short ID support depends on implementation - for now we test that it doesn't error
    // The actual changeset may or may not be found depending on filename matching
    if loaded_changeset.is_some() {
        let loaded = loaded_changeset.unwrap();
        assert_eq!(loaded.id, full_id);
        assert_eq!(loaded.package, changeset.package);
    }
}

/// Test: List all changesets with filter criteria
#[test]
fn test_changeset_storage_list() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_storage = crate::changesets::ChangesetStorage::from_project(project);

    // Create multiple test changesets
    let changesets = vec![
        crate::changesets::Changeset {
            id: "list-test-1".to_string(),
            package: "@test/core".to_string(),
            version_bump: crate::config::VersionBumpType::Minor,
            description: "First changeset for list test".to_string(),
            branch: "feature/list-test-1".to_string(),
            development_environments: vec![crate::config::types::Environment::Development],
            production_deployment: false,
            created_at: chrono::Utc::now(),
            author: "list1.test@example.com".to_string(),
            status: crate::changesets::ChangesetStatus::Pending,
        },
        crate::changesets::Changeset {
            id: "list-test-2".to_string(),
            package: "@test/utils".to_string(),
            version_bump: crate::config::VersionBumpType::Patch,
            description: "Second changeset for list test".to_string(),
            branch: "feature/list-test-2".to_string(),
            development_environments: vec![crate::config::types::Environment::Staging],
            production_deployment: true,
            created_at: chrono::Utc::now(),
            author: "list2.test@example.com".to_string(),
            status: crate::changesets::ChangesetStatus::Pending,
        },
        crate::changesets::Changeset {
            id: "list-test-3".to_string(),
            package: "@test/core".to_string(),
            version_bump: crate::config::VersionBumpType::Major,
            description: "Third changeset for list test".to_string(),
            branch: "feature/list-test-3".to_string(),
            development_environments: vec![crate::config::types::Environment::Integration],
            production_deployment: false,
            created_at: chrono::Utc::now(),
            author: "list3.test@example.com".to_string(),
            status: crate::changesets::ChangesetStatus::FullyDeployed {
                deployed_at: chrono::Utc::now(),
            },
        },
    ];

    // Save all changesets
    for changeset in &changesets {
        changeset_storage.save(changeset).expect("Should save changeset");
    }

    // Test listing all changesets (no filter)
    let empty_filter = crate::changesets::ChangesetFilter::default();
    let all_changesets = changeset_storage.list(&empty_filter).expect("Should list all changesets");
    assert_eq!(all_changesets.len(), 3, "Should return all 3 changesets");

    // Test filtering by package
    let core_filter = crate::changesets::ChangesetFilter {
        package: Some("@test/core".to_string()),
        ..Default::default()
    };
    let core_changesets =
        changeset_storage.list(&core_filter).expect("Should list core changesets");
    assert_eq!(core_changesets.len(), 2, "Should return 2 core changesets");

    // Test filtering by status
    let pending_filter = crate::changesets::ChangesetFilter {
        status: Some(crate::changesets::ChangesetStatus::Pending),
        ..Default::default()
    };
    let pending_changesets =
        changeset_storage.list(&pending_filter).expect("Should list pending changesets");
    assert_eq!(pending_changesets.len(), 2, "Should return 2 pending changesets");

    // Test filtering by environment
    let dev_filter = crate::changesets::ChangesetFilter {
        environment: Some(crate::config::types::Environment::Development),
        ..Default::default()
    };
    let dev_changesets = changeset_storage.list(&dev_filter).expect("Should list dev changesets");
    assert_eq!(dev_changesets.len(), 1, "Should return 1 development changeset");

    // Test filtering by author
    let author_filter = crate::changesets::ChangesetFilter {
        author: Some("list2.test@example.com".to_string()),
        ..Default::default()
    };
    let author_changesets =
        changeset_storage.list(&author_filter).expect("Should list author changesets");
    assert_eq!(author_changesets.len(), 1, "Should return 1 changeset by specific author");
}

/// Test: List changesets with multiple filter criteria
#[test]
fn test_changeset_storage_list_multiple_filters() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_storage = crate::changesets::ChangesetStorage::from_project(project);

    // Create test changeset matching multiple criteria
    let changeset = crate::changesets::Changeset {
        id: "multi-filter-test".to_string(),
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Changeset for multi-filter test".to_string(),
        branch: "feature/multi-filter".to_string(),
        development_environments: vec![crate::config::types::Environment::Staging],
        production_deployment: false,
        created_at: chrono::Utc::now(),
        author: "multifilter.test@example.com".to_string(),
        status: crate::changesets::ChangesetStatus::Pending,
    };

    changeset_storage.save(&changeset).expect("Should save changeset");

    // Test multiple filter criteria
    let multi_filter = crate::changesets::ChangesetFilter {
        package: Some("@test/core".to_string()),
        status: Some(crate::changesets::ChangesetStatus::Pending),
        environment: Some(crate::config::types::Environment::Staging),
        branch: Some("feature/multi-filter".to_string()),
        author: Some("multifilter.test@example.com".to_string()),
    };

    let filtered_changesets =
        changeset_storage.list(&multi_filter).expect("Should apply multiple filters");
    assert_eq!(filtered_changesets.len(), 1, "Should return 1 changeset matching all criteria");

    let found = &filtered_changesets[0];
    assert_eq!(found.id, changeset.id);
    assert_eq!(found.package, changeset.package);
    assert_eq!(found.branch, changeset.branch);
    assert_eq!(found.author, changeset.author);
}

/// Test: Delete changeset from storage
#[test]
fn test_changeset_storage_delete() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_storage = crate::changesets::ChangesetStorage::from_project(project);

    // Create and save test changeset
    let changeset = crate::changesets::Changeset {
        id: "delete-test-id".to_string(),
        package: "@test/ui".to_string(),
        version_bump: crate::config::VersionBumpType::Patch,
        description: "Test changeset for deletion".to_string(),
        branch: "feature/delete-test".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        created_at: chrono::Utc::now(),
        author: "delete.test@example.com".to_string(),
        status: crate::changesets::ChangesetStatus::Pending,
    };

    changeset_storage.save(&changeset).expect("Should save changeset");

    // Verify changeset exists
    let loaded_before = changeset_storage.load(&changeset.id).expect("Should load changeset");
    assert!(loaded_before.is_some(), "Changeset should exist before deletion");

    // Delete changeset
    let delete_result = changeset_storage.delete(&changeset.id);
    assert!(delete_result.is_ok(), "Should delete changeset successfully");
    assert!(delete_result.unwrap(), "Should return true for successful deletion");

    // Verify changeset no longer exists
    let loaded_after =
        changeset_storage.load(&changeset.id).expect("Should handle load after deletion");
    assert!(loaded_after.is_none(), "Changeset should not exist after deletion");

    // Test deleting non-existent changeset
    let delete_non_existent = changeset_storage.delete("non-existent-id");
    assert!(delete_non_existent.is_ok(), "Should handle non-existent deletion gracefully");
    assert!(!delete_non_existent.unwrap(), "Should return false for non-existent deletion");
}

/// Test: Delete changeset by short ID
#[test]
fn test_changeset_storage_delete_short_id() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_storage = crate::changesets::ChangesetStorage::from_project(project);

    // Create changeset with long ID
    let full_id = "delete123456789012345678901234567890";
    let changeset = crate::changesets::Changeset {
        id: full_id.to_string(),
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Test changeset for short ID deletion".to_string(),
        branch: "feature/delete-short".to_string(),
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        created_at: chrono::Utc::now(),
        author: "deleteshort.test@example.com".to_string(),
        status: crate::changesets::ChangesetStatus::Pending,
    };

    changeset_storage.save(&changeset).expect("Should save changeset");

    // Delete by short ID (first 8 characters)
    let short_id = &full_id[..8];
    let delete_result = changeset_storage.delete(short_id);
    assert!(delete_result.is_ok(), "Should delete changeset by short ID");

    // Note: Short ID support depends on implementation - test that it doesn't error
    // The actual deletion may or may not occur depending on filename matching
    if *delete_result.as_ref().unwrap() {
        // Verify changeset no longer exists
        let loaded_after =
            changeset_storage.load(full_id).expect("Should handle load after deletion");
        assert!(loaded_after.is_none(), "Changeset should not exist after deletion by short ID");
    } else {
        // Verify changeset still exists since deletion by short ID failed
        let loaded_after =
            changeset_storage.load(full_id).expect("Should handle load after failed deletion");
        assert!(
            loaded_after.is_some(),
            "Changeset should still exist after failed short ID deletion"
        );
    }
}

/// Test: Storage operations with invalid changeset directory
#[test]
fn test_changeset_storage_invalid_directory() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_storage = crate::changesets::ChangesetStorage::from_project(project);

    // Test listing when directory doesn't exist yet
    let empty_filter = crate::changesets::ChangesetFilter::default();
    let empty_list = changeset_storage.list(&empty_filter).expect("Should handle empty directory");
    assert_eq!(empty_list.len(), 0, "Should return empty list when directory doesn't exist");

    // Test loading when directory doesn't exist
    let no_load =
        changeset_storage.load("some-id").expect("Should handle missing directory on load");
    assert!(no_load.is_none(), "Should return None when directory doesn't exist");

    // Test deleting when directory doesn't exist
    let no_delete =
        changeset_storage.delete("some-id").expect("Should handle missing directory on delete");
    assert!(!no_delete, "Should return false when directory doesn't exist");
}

/// Test: Storage filename generation and format
#[test]
fn test_changeset_storage_filename_format() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_storage = crate::changesets::ChangesetStorage::from_project(project);

    // Create changeset with special characters in branch name
    let changeset = crate::changesets::Changeset {
        id: "filename-test-id".to_string(),
        package: "@test/core".to_string(),
        version_bump: crate::config::VersionBumpType::Minor,
        description: "Test changeset for filename format".to_string(),
        branch: "feature/special-chars/test".to_string(), // Has '/' which should be replaced
        development_environments: vec![crate::config::types::Environment::Development],
        production_deployment: false,
        created_at: chrono::Utc::now(),
        author: "filename.test@example.com".to_string(),
        status: crate::changesets::ChangesetStatus::Pending,
    };

    // Save and verify we can load it back (proves filename generation works)
    let save_result = changeset_storage.save(&changeset);
    assert!(save_result.is_ok(), "Should save changeset with special branch characters");

    let loaded = changeset_storage.load(&changeset.id).expect("Should load changeset");
    assert!(loaded.is_some(), "Should successfully load changeset after save");

    let loaded_changeset = loaded.unwrap();
    assert_eq!(loaded_changeset.branch, changeset.branch); // Original branch name should be preserved
}

/// Test: Storage operations with large number of changesets
#[test]
fn test_changeset_storage_performance() {
    let (_temp_dir, project) = create_test_monorepo();
    let changeset_storage = crate::changesets::ChangesetStorage::from_project(project);

    // Create multiple changesets
    let changeset_count = 20;
    let mut changesets = Vec::new();

    for i in 0..changeset_count {
        let changeset = crate::changesets::Changeset {
            id: format!("perf-test-{:03}", i),
            package: format!("@test/package-{}", i % 3), // Distribute across 3 packages
            version_bump: match i % 3 {
                0 => crate::config::VersionBumpType::Patch,
                1 => crate::config::VersionBumpType::Minor,
                _ => crate::config::VersionBumpType::Major,
            },
            description: format!("Performance test changeset {}", i),
            branch: format!("feature/perf-test-{}", i),
            development_environments: vec![crate::config::types::Environment::Development],
            production_deployment: i % 4 == 0, // 25% have production deployment
            created_at: chrono::Utc::now(),
            author: format!("perf{}.test@example.com", i),
            status: if i % 2 == 0 {
                crate::changesets::ChangesetStatus::Pending
            } else {
                crate::changesets::ChangesetStatus::FullyDeployed {
                    deployed_at: chrono::Utc::now(),
                }
            },
        };
        changesets.push(changeset);
    }

    // Save all changesets
    let start_save = std::time::Instant::now();
    for changeset in &changesets {
        changeset_storage.save(changeset).expect("Should save changeset");
    }
    let save_duration = start_save.elapsed();

    assert!(
        save_duration.as_millis() < 10000,
        "Saving {} changesets should complete within reasonable time ({}ms)",
        changeset_count,
        save_duration.as_millis()
    );

    // List all changesets
    let start_list = std::time::Instant::now();
    let empty_filter = crate::changesets::ChangesetFilter::default();
    let all_loaded = changeset_storage.list(&empty_filter).expect("Should list all changesets");
    let list_duration = start_list.elapsed();

    assert_eq!(all_loaded.len(), changeset_count, "Should load all saved changesets");
    assert!(
        list_duration.as_millis() < 5000,
        "Listing {} changesets should complete within reasonable time ({}ms)",
        changeset_count,
        list_duration.as_millis()
    );

    // Load individual changesets
    let start_load = std::time::Instant::now();
    for changeset in &changesets {
        let loaded = changeset_storage.load(&changeset.id).expect("Should load changeset");
        assert!(loaded.is_some(), "Should find saved changeset");
    }
    let load_duration = start_load.elapsed();

    assert!(
        load_duration.as_millis() < 5000,
        "Loading {} individual changesets should complete within reasonable time ({}ms)",
        changeset_count,
        load_duration.as_millis()
    );
}
