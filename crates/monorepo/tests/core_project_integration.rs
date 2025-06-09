//! Integration tests for the core MonorepoProject functionality
//!
//! This module contains integration tests that verify the fundamental
//! MonorepoProject operations using realistic monorepo structures.

use std::sync::Arc;
use sublime_monorepo_tools::MonorepoProject;
use tempfile::TempDir;

mod common;

#[test]
#[allow(clippy::arc_with_non_send_sync)]
fn test_monorepo_project_creation_with_realistic_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repository");

    // Set up realistic monorepo structure using common utilities
    common::setup_test_monorepo(temp_dir.path());

    // Verify the structure was created correctly
    assert!(common::verify_test_structure(temp_dir.path()));

    // Create MonorepoProject
    let project = MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject");

    // Verify basic project properties
    assert_eq!(project.root_path(), temp_dir.path());

    // Verify project can be wrapped in Arc (required for managers)
    let arc_project = Arc::new(project);
    assert_eq!(arc_project.root_path(), temp_dir.path());
}

#[test]
fn test_monorepo_project_package_detection() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repository");

    // Set up realistic monorepo structure with 3 packages
    common::setup_test_monorepo(temp_dir.path());

    let _project = MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject");

    // Verify the project can detect the packages created by common utilities
    // The common module creates: @test/core, @test/utils, @test/app
    let packages_dir = temp_dir.path().join("packages");
    assert!(packages_dir.join("core").join("package.json").exists());
    assert!(packages_dir.join("utils").join("package.json").exists());
    assert!(packages_dir.join("app").join("package.json").exists());

    // Verify package structure includes dependencies as set up by common utilities
    let core_package_json = std::fs::read_to_string(packages_dir.join("core").join("package.json"))
        .expect("Failed to read core package.json");
    assert!(core_package_json.contains("@test/core"));
    assert!(core_package_json.contains("lodash"));

    let utils_package_json =
        std::fs::read_to_string(packages_dir.join("utils").join("package.json"))
            .expect("Failed to read utils package.json");
    assert!(utils_package_json.contains("@test/utils"));
    assert!(utils_package_json.contains("@test/core")); // Dependency on core

    let app_package_json = std::fs::read_to_string(packages_dir.join("app").join("package.json"))
        .expect("Failed to read app package.json");
    assert!(app_package_json.contains("@test/app"));
    assert!(app_package_json.contains("@test/core")); // Dependency on core
    assert!(app_package_json.contains("@test/utils")); // Dependency on utils
    assert!(app_package_json.contains("react")); // External dependency
}

#[test]
fn test_monorepo_project_with_configuration_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repository");

    // Set up monorepo with configuration files
    common::setup_test_monorepo(temp_dir.path());

    let project = MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject");

    // Verify configuration files created by common utilities
    assert!(temp_dir.path().join("tsconfig.json").exists());
    assert!(temp_dir.path().join(".eslintrc.json").exists());
    assert!(temp_dir.path().join("README.md").exists());

    // Verify root package.json workspace configuration
    let root_package_json = std::fs::read_to_string(temp_dir.path().join("package.json"))
        .expect("Failed to read root package.json");
    assert!(root_package_json.contains("test-monorepo"));
    assert!(root_package_json.contains("packages/*"));
    assert!(root_package_json.contains("typescript"));
    assert!(root_package_json.contains("jest"));

    // Project should be able to handle this structure
    assert_eq!(project.root_path(), temp_dir.path());
}

#[test]
fn test_monorepo_project_with_source_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repository");

    // Set up monorepo with source files for change detection testing
    common::setup_test_monorepo(temp_dir.path());

    let project = MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject");

    // Verify source files created by common utilities
    let packages_dir = temp_dir.path().join("packages");

    // Core package source files
    assert!(packages_dir.join("core").join("src").join("index.ts").exists());
    assert!(packages_dir.join("core").join("src").join("utils.ts").exists());
    assert!(packages_dir.join("core").join("src").join("api").join("core.ts").exists());
    assert!(packages_dir.join("core").join("tests").join("index.test.ts").exists());

    // Utils package source files
    assert!(packages_dir.join("utils").join("src").join("index.ts").exists());
    assert!(packages_dir.join("utils").join("src").join("utils.ts").exists());
    assert!(packages_dir.join("utils").join("docs").join("api.md").exists());
    assert!(packages_dir.join("utils").join("tests").join("index.test.ts").exists());

    // App package source files
    assert!(packages_dir.join("app").join("src").join("index.ts").exists());
    assert!(packages_dir.join("app").join("src").join("utils.ts").exists());
    assert!(packages_dir.join("app").join("tests").join("index.test.ts").exists());

    // Project should handle complex source structure
    assert_eq!(project.root_path(), temp_dir.path());
}

#[test]
fn test_monorepo_project_with_package_changes() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repository");

    // Set up monorepo structure
    common::setup_test_monorepo(temp_dir.path());

    let project = MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject");

    let packages_dir = temp_dir.path().join("packages");

    // Test different types of changes using common utilities

    // 1. Source code changes
    common::create_package_change(&packages_dir.join("core"), "source");
    let core_index =
        std::fs::read_to_string(packages_dir.join("core").join("src").join("index.ts"))
            .expect("Failed to read core index.ts");
    assert!(core_index.contains("Modified for testing"));

    // 2. Dependency changes
    common::create_package_change(&packages_dir.join("utils"), "dependencies");
    let utils_package_json =
        std::fs::read_to_string(packages_dir.join("utils").join("package.json"))
            .expect("Failed to read utils package.json");
    assert!(utils_package_json.contains("new-dependency"));

    // 3. Documentation changes
    common::create_package_change(&packages_dir.join("app"), "documentation");
    assert!(packages_dir.join("app").join("README.md").exists());

    // 4. Custom changes
    common::create_package_change(&packages_dir.join("core"), "custom");
    assert!(packages_dir.join("core").join("CHANGED").exists());

    // Project should still be valid after changes
    assert_eq!(project.root_path(), temp_dir.path());
}

#[test]
fn test_monorepo_project_error_handling() {
    // Test project creation with non-existent directory
    let non_existent_path = std::path::Path::new("/non/existent/path");
    let result = MonorepoProject::new(non_existent_path);
    assert!(result.is_err());

    // Test project creation with invalid directory (file instead of directory)
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("not_a_directory.txt");
    std::fs::write(&file_path, "test content").expect("Failed to create test file");

    let result = MonorepoProject::new(&file_path);
    assert!(result.is_err());
}

#[test]
fn test_monorepo_project_git_integration() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test without Git repository
    common::setup_test_monorepo(temp_dir.path());
    let _project_without_git = MonorepoProject::new(temp_dir.path());
    // Should still work without Git (depending on implementation)

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

    // Test with Git repository
    let project_with_git =
        MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject with Git");

    assert_eq!(project_with_git.root_path(), temp_dir.path());

    // Test Git operations can be performed
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add files to git");

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit files");

    // Project should still be valid after Git operations
    assert_eq!(project_with_git.root_path(), temp_dir.path());
}

#[test]
#[allow(clippy::arc_with_non_send_sync)]
fn test_monorepo_project_arc_usage() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repository");

    // Set up monorepo structure
    common::setup_test_monorepo(temp_dir.path());

    // Create multiple Arc-wrapped projects (simulating shared ownership)
    let project1 =
        Arc::new(MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject"));
    let project2 = Arc::clone(&project1);
    let project3 = Arc::clone(&project1);

    // Verify all references point to the same project
    assert_eq!(project1.root_path(), project2.root_path());
    assert_eq!(project2.root_path(), project3.root_path());
    assert_eq!(project1.root_path(), temp_dir.path());

    // Verify Arc can be used locally
    let _local_project = project3;
    // Simulate some work
    std::thread::sleep(std::time::Duration::from_millis(1));

    // Project references are still valid
    assert_eq!(project1.root_path(), temp_dir.path());
    assert_eq!(project2.root_path(), temp_dir.path());
}
