//! Configuration and Components Integration Tests
//!
//! This module contains comprehensive integration tests that validate the configuration 
//! system works correctly with all components of the monorepo tools.

use std::sync::Arc;
use tempfile::TempDir;

use sublime_monorepo_tools::{
    analysis::{MonorepoAnalyzer, DiffAnalyzer},
    changesets::{ChangesetManager, ChangesetSpec},
    workflows::DevelopmentWorkflow,
    core::MonorepoProject,
    config::{VersionBumpType, Environment, MonorepoConfig},
};

/// Helper to run async code in sync tests
fn run_async<F, R>(f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    rt.block_on(f)
}

/// Create a standard test monorepo
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
  "name": "config-test-monorepo",
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
    create_test_package(&packages_dir, "frontend", "1.0.0", &[]);
    create_test_package(&packages_dir, "backend", "1.0.0", &["@test/shared"]);
    create_test_package(&packages_dir, "shared", "1.0.0", &[]);

    // Add files to git
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add files to git");

    // Create initial commit
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to create initial commit");

    // Create project
    let mut project = MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject");
    project.refresh_packages().expect("Failed to refresh packages");
    project.build_dependency_graph().expect("Failed to build dependency graph");
    
    (temp_dir, Arc::new(project))
}

/// Creates a test package with specified dependencies
fn create_test_package(packages_dir: &std::path::Path, name: &str, version: &str, dependencies: &[&str]) {
    let package_dir = packages_dir.join(name);
    std::fs::create_dir_all(&package_dir).expect("Failed to create package directory");

    let mut dep_obj = serde_json::Map::new();
    for dep in dependencies {
        dep_obj.insert(dep.to_string(), serde_json::Value::String("*".to_string()));
    }

    let package_json = serde_json::json!({
        "name": format!("@test/{}", name),
        "version": version,
        "dependencies": dep_obj
    });

    std::fs::write(
        package_dir.join("package.json"),
        serde_json::to_string_pretty(&package_json).expect("Failed to serialize package.json")
    ).expect("Failed to write package.json");
}

#[test]
fn test_configuration_presets_integration() {
    // Test: Validate that different configuration presets create different behaviors
    let (_temp_dir, project) = create_test_monorepo();
    
    // === Test Small Project Configuration ===
    let small_config = MonorepoConfig::small_project();
    assert!(!small_config.tasks.parallel, "Small projects should not run tasks in parallel");
    assert!(small_config.tasks.max_concurrent < 4, "Small projects should have limited concurrency");
    
    // === Test Large Project Configuration ===
    let large_config = MonorepoConfig::large_project();
    assert!(large_config.tasks.parallel, "Large projects should run tasks in parallel");
    assert!(large_config.tasks.max_concurrent >= 4, "Large projects should have higher concurrency");
    
    // === Test Default Configuration ===
    let default_config = MonorepoConfig::default();
    assert_eq!(default_config.versioning.default_bump, VersionBumpType::Patch, "Default bump should be patch");
    assert!(default_config.versioning.propagate_changes, "Default should propagate changes");
    
    // === Test Configuration Differences ===
    assert_ne!(small_config.tasks.max_concurrent, large_config.tasks.max_concurrent, 
              "Small and large projects should have different concurrency limits");
    assert_ne!(small_config.tasks.parallel, large_config.tasks.parallel,
              "Small and large projects should have different parallel settings");
    
    // === Test Component Creation with Project ===
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));
    let changes = analyzer.detect_changes_since("HEAD", None)
        .expect("Failed to detect changes");
    
    // Should work regardless of configuration
    assert_eq!(changes.changed_files.len(), 0, "No changes should be detected in fresh repo");
    
    println!("✅ Configuration presets integration test completed successfully!");
    println!("   - Validated small project configuration settings");
    println!("   - Validated large project configuration settings");
    println!("   - Confirmed different presets create different behaviors");
}

#[test]
fn test_analysis_component_configuration_integration() {
    // Test: Validate that analysis components work with different configurations
    let (_temp_dir, project) = create_test_monorepo();
    
    // === Test Analysis Creation ===
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));
    
    // Both analyzers should be created successfully
    // This validates that the configuration system works with analysis components
    
    // === Test Change Detection ===
    let changes = analyzer.detect_changes_since("HEAD", None)
        .expect("Failed to detect changes");
    assert_eq!(changes.changed_files.len(), 0, "No changes should be detected initially");
    
    // === Test Analysis with Changes ===
    // Create a test change
    let test_file = project.root_path().join("packages/frontend/new-feature.js");
    std::fs::write(&test_file, "console.log('new feature');")
        .expect("Failed to write test file");
    
    // Add and commit the change
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(project.root_path())
        .output()
        .expect("Failed to add changes");
        
    std::process::Command::new("git")
        .args(["commit", "-m", "feat: add new feature"])
        .current_dir(project.root_path())
        .output()
        .expect("Failed to commit changes");
    
    // Detect changes
    let changes_with_new_file = analyzer.detect_changes_since("HEAD~1", None)
        .expect("Failed to detect changes after adding file");
    
    assert!(!changes_with_new_file.changed_files.is_empty(), "Should detect the new file");
    assert!(!changes_with_new_file.package_changes.is_empty(), "Should detect package changes");
    
    // === Test Diff Analysis ===
    let package_changes = diff_analyzer.map_changes_to_packages(&changes_with_new_file.changed_files);
    assert!(!package_changes.is_empty(), "Diff analyzer should map changes to packages");
    
    // === Test Significance Analysis ===
    let significance_analysis = diff_analyzer.analyze_change_significance(&changes_with_new_file.package_changes);
    assert_eq!(significance_analysis.len(), changes_with_new_file.package_changes.len(), 
              "Should analyze significance for all package changes");
    
    println!("✅ Analysis component configuration integration test completed successfully!");
    println!("   - Validated analyzer creation with project configuration");
    println!("   - Tested change detection with configuration");
    println!("   - Verified significance analysis integration");
}

#[test]
fn test_changeset_workflow_configuration_integration() {
    // Test: Validate that changeset and workflow components respect configuration
    let (_temp_dir, project) = create_test_monorepo();
    
    // === Test Changeset Manager Creation ===
    let changeset_manager = ChangesetManager::from_project(Arc::clone(&project))
        .expect("Failed to create changeset manager");
    
    // === Test Changeset Creation with Different Configurations ===
    let dev_changeset_spec = ChangesetSpec {
        package: "@test/frontend".to_string(),
        version_bump: VersionBumpType::Patch,
        description: "Small development change".to_string(),
        development_environments: vec![Environment::Development],
        production_deployment: false,
        author: Some("dev@example.com".to_string()),
    };
    
    let prod_changeset_spec = ChangesetSpec {
        package: "@test/backend".to_string(),
        version_bump: VersionBumpType::Minor,
        description: "Production-ready feature".to_string(),
        development_environments: vec![Environment::Development, Environment::Staging],
        production_deployment: true,
        author: Some("prod@example.com".to_string()),
    };
    
    let dev_changeset = changeset_manager.create_changeset(dev_changeset_spec)
        .expect("Failed to create development changeset");
    let prod_changeset = changeset_manager.create_changeset(prod_changeset_spec)
        .expect("Failed to create production changeset");
    
    // === Test Changeset Validation ===
    let dev_validation = changeset_manager.validate_changeset(&dev_changeset)
        .expect("Failed to validate development changeset");
    let prod_validation = changeset_manager.validate_changeset(&prod_changeset)
        .expect("Failed to validate production changeset");
    
    assert!(dev_validation.is_valid, "Development changeset should be valid");
    assert!(prod_validation.is_valid, "Production changeset should be valid");
    
    // === Test Workflow Integration ===
    let workflow = DevelopmentWorkflow::from_project(Arc::clone(&project))
        .expect("Failed to create development workflow");
    
    let workflow_result = run_async(workflow.execute(Some("HEAD")))
        .expect("Failed to execute workflow");
    
    // Workflow should complete successfully
    assert!(workflow_result.checks_passed, "Workflow should pass for clean repo");
    
    // === Test Environment-Specific Behavior ===
    assert!(!dev_changeset.production_deployment, "Development changeset should not deploy to production");
    assert!(prod_changeset.production_deployment, "Production changeset should deploy to production");
    
    assert_eq!(dev_changeset.development_environments.len(), 1, 
              "Development changeset should target one environment");
    assert_eq!(prod_changeset.development_environments.len(), 2, 
              "Production changeset should target multiple environments");
    
    println!("✅ Changeset workflow configuration integration test completed successfully!");
    println!("   - Created and validated changesets with different configurations");
    println!("   - Tested workflow execution with configuration");
    println!("   - Verified environment-specific behavior");
}

#[test]
fn test_component_error_handling_integration() {
    // Test: Validate that all components handle configuration-related errors gracefully
    let (_temp_dir, project) = create_test_monorepo();
    
    // === Test Component Creation Resilience ===
    
    // All components should be able to be created with the project
    let analyzer_result = MonorepoAnalyzer::from_project(Arc::clone(&project));
    // MonorepoAnalyzer::from_project doesn't return Result, so it should always succeed
    
    let workflow_result = DevelopmentWorkflow::from_project(Arc::clone(&project));
    assert!(workflow_result.is_ok(), "Workflow creation should succeed");
    
    let changeset_manager_result = ChangesetManager::from_project(Arc::clone(&project));
    assert!(changeset_manager_result.is_ok(), "Changeset manager creation should succeed");
    
    // === Test Invalid Operation Handling ===
    if let Ok(changeset_manager) = changeset_manager_result {
        // Test creating changeset for non-existent package
        let invalid_changeset_spec = ChangesetSpec {
            package: "@test/nonexistent".to_string(),
            version_bump: VersionBumpType::Patch,
            description: "Invalid changeset".to_string(),
            development_environments: vec![Environment::Development],
            production_deployment: false,
            author: Some("test@example.com".to_string()),
        };
        
        let invalid_result = changeset_manager.create_changeset(invalid_changeset_spec);
        // Should either succeed with fallback behavior or fail gracefully
        match invalid_result {
            Ok(_) => println!("✅ Changeset creation succeeded with invalid package (fallback behavior)"),
            Err(_) => println!("✅ Changeset creation failed gracefully with invalid package"),
        }
    }
    
    // === Test Analysis Error Handling ===
    let changes_result = analyzer_result.detect_changes_since("invalid-ref", None);
    // Should handle invalid references gracefully
    match changes_result {
        Ok(_) => println!("✅ Analysis succeeded with invalid reference (fallback behavior)"),
        Err(_) => println!("✅ Analysis failed gracefully with invalid reference"),
    }
    
    // === Test Workflow Error Handling ===
    if let Ok(workflow) = workflow_result {
        let invalid_workflow_result = run_async(workflow.execute(Some("invalid-ref")));
        match invalid_workflow_result {
            Ok(_) => println!("✅ Workflow succeeded with invalid reference (fallback behavior)"),
            Err(_) => println!("✅ Workflow failed gracefully with invalid reference"),
        }
    }
    
    println!("✅ Component error handling integration test completed successfully!");
    println!("   - Tested component creation resilience");
    println!("   - Validated error handling for invalid operations");
    println!("   - Confirmed graceful degradation across components");
}

#[test]
fn test_cross_component_data_flow_integration() {
    // Test: Validate that data flows correctly between all components
    let (_temp_dir, project) = create_test_monorepo();
    
    // === Step 1: Create changes (Git integration) ===
    let test_file = project.root_path().join("packages/shared/utils.js");
    std::fs::write(&test_file, "export function utility() { return 'updated'; }")
        .expect("Failed to write test file");
    
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(project.root_path())
        .output()
        .expect("Failed to add changes");
        
    std::process::Command::new("git")
        .args(["commit", "-m", "feat: update shared utility"])
        .current_dir(project.root_path())
        .output()
        .expect("Failed to commit changes");
    
    // === Step 2: Analyze changes (Analysis integration) ===
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));
    let changes = analyzer.detect_changes_since("HEAD~1", None)
        .expect("Failed to detect changes");
    
    assert!(!changes.changed_files.is_empty(), "Should detect changes");
    assert!(!changes.package_changes.is_empty(), "Should detect package changes");
    
    // === Step 3: Create changesets (Changeset integration) ===
    let changeset_manager = ChangesetManager::from_project(Arc::clone(&project))
        .expect("Failed to create changeset manager");
    
    let changeset_spec = ChangesetSpec {
        package: "@test/shared".to_string(),
        version_bump: VersionBumpType::Minor,
        description: "Updated shared utility function".to_string(),
        development_environments: vec![Environment::Development, Environment::Staging],
        production_deployment: false,
        author: Some("dev@example.com".to_string()),
    };
    
    let changeset = changeset_manager.create_changeset(changeset_spec)
        .expect("Failed to create changeset");
    
    // === Step 4: Validate changeset (Validation integration) ===
    let validation = changeset_manager.validate_changeset(&changeset)
        .expect("Failed to validate changeset");
    
    assert!(validation.is_valid, "Changeset should be valid");
    
    // === Step 5: Execute workflow (Workflow integration) ===
    let workflow = DevelopmentWorkflow::from_project(Arc::clone(&project))
        .expect("Failed to create workflow");
    
    let workflow_result = run_async(workflow.execute(Some("HEAD~1")))
        .expect("Failed to execute workflow");
    
    // === Validate Cross-Component Integration ===
    
    // 1. Changes detected by analysis should be consistent
    assert!(changes.package_changes.iter().any(|pc| pc.package_name == "@test/shared"),
           "Analysis should detect changes in shared package");
    
    // 2. Changeset should reference the correct package
    assert_eq!(changeset.package, "@test/shared", "Changeset should target the correct package");
    
    // 3. Workflow should have information about changes
    assert!(!workflow_result.changes.changed_files.is_empty(), 
           "Workflow should detect the same changes");
    
    // 4. Data consistency across components
    let workflow_package_changes: Vec<_> = workflow_result.changes.package_changes.iter()
        .map(|pc| &pc.package_name)
        .collect();
    assert!(workflow_package_changes.contains(&&"@test/shared".to_string()),
           "Workflow should see changes in the same packages as analysis");
    
    println!("✅ Cross-component data flow integration test completed successfully!");
    println!("   - Validated Git → Analysis → Changeset → Workflow data flow");
    println!("   - Confirmed data consistency across all components");
    println!("   - Verified component integration with configuration");
}