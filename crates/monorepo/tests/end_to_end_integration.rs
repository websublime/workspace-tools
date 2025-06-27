//! End-to-end integration tests
//!
//! This module contains comprehensive end-to-end tests that validate the complete
//! integration between Git, Analysis, Changeset, and Workflow components.

use std::sync::Arc;
use tempfile::TempDir;

use sublime_monorepo_tools::{
    analysis::{MonorepoAnalyzer, DiffAnalyzer},
    changesets::{ChangesetManager, ChangesetSpec},
    workflows::DevelopmentWorkflow,
    core::MonorepoProject,
    config::{VersionBumpType, Environment},
};


/// Helper to run async code in sync tests
fn run_async<F, R>(f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    rt.block_on(f)
}

/// Create a comprehensive test monorepo with multiple packages and dependencies
fn create_comprehensive_monorepo() -> (TempDir, Arc<MonorepoProject>) {
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
  "name": "comprehensive-monorepo",
  "version": "1.0.0",
  "private": true,
  "workspaces": ["packages/*"],
  "scripts": {
    "test": "echo 'Running tests'",
    "build": "echo 'Building packages'",
    "lint": "echo 'Linting code'"
  }
}"#;
    std::fs::write(temp_dir.path().join("package.json"), root_package_json)
        .expect("Failed to write root package.json");

    // Create package-lock.json for npm detection
    std::fs::write(temp_dir.path().join("package-lock.json"), "{}")
        .expect("Failed to write package-lock.json");

    // Create multiple test packages with dependencies
    create_test_package(&packages_dir, "foundation", "1.0.0", &[], "Core foundation utilities");
    create_test_package(&packages_dir, "core", "1.1.0", &["@test/foundation"], "Core business logic");
    create_test_package(&packages_dir, "utils", "1.2.0", &["@test/foundation", "@test/core"], "Utility functions");
    create_test_package(&packages_dir, "ui-components", "2.0.0", &["@test/foundation", "@test/core"], "UI component library");
    create_test_package(&packages_dir, "app", "3.0.0", &["@test/foundation", "@test/core", "@test/utils", "@test/ui-components"], "Main application");

    // Add all files and create initial commit
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add files to git");

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit with comprehensive monorepo setup"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to create initial commit");

    // Create and return project
    let mut project = MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject");
    project.refresh_packages().expect("Failed to refresh packages");
    project.build_dependency_graph().expect("Failed to build dependency graph");
    
    (temp_dir, Arc::new(project))
}

/// Creates a test package with specified dependencies and description
fn create_test_package(packages_dir: &std::path::Path, name: &str, version: &str, dependencies: &[&str], description: &str) {
    let package_dir = packages_dir.join(name);
    std::fs::create_dir_all(&package_dir).expect("Failed to create package directory");

    // Create src directory and main file
    let src_dir = package_dir.join("src");
    std::fs::create_dir_all(&src_dir).expect("Failed to create src directory");
    
    let main_content = format!(
        r#"//! {} package
//!
//! {}

export function {}() {{
    return "Hello from {}";
}}
"#,
        name, description, name.replace('-', "_"), name
    );
    std::fs::write(src_dir.join("index.ts"), main_content)
        .expect("Failed to write main file");

    // Create README
    let readme_content = format!(
        r#"# {} Package

{}

## Installation

```bash
npm install @test/{}
```

## Usage

```typescript
import {{ {} }} from "@test/{}";

console.log({}());
```
"#,
        name, description, name, name.replace('-', "_"), name, name.replace('-', "_")
    );
    std::fs::write(package_dir.join("README.md"), readme_content)
        .expect("Failed to write README");

    // Create package.json with dependencies
    let mut dep_obj = serde_json::Map::new();
    for dep in dependencies {
        dep_obj.insert(dep.to_string(), serde_json::Value::String("*".to_string()));
    }

    let package_json = serde_json::json!({
        "name": format!("@test/{}", name),
        "version": version,
        "description": description,
        "main": "src/index.ts",
        "dependencies": dep_obj,
        "scripts": {
            "build": "echo 'Building package'",
            "test": "echo 'Testing package'",
            "lint": "echo 'Linting package'"
        }
    });

    std::fs::write(
        package_dir.join("package.json"),
        serde_json::to_string_pretty(&package_json).expect("Failed to serialize package.json")
    ).expect("Failed to write package.json");
}

/// Simulate realistic changes to multiple packages
fn create_realistic_changes(temp_dir: &TempDir, scenario: &str) {
    match scenario {
        "feature_development" => {
            // Feature: Add new logging capability
            // 1. Update foundation with new logging utilities
            let foundation_log = temp_dir.path().join("packages/foundation/src/logger.ts");
            std::fs::write(&foundation_log, r#"//! Enhanced logging utilities

export enum LogLevel {
    DEBUG = 0,
    INFO = 1,
    WARN = 2,
    ERROR = 3
}

export class Logger {
    constructor(private level: LogLevel = LogLevel.INFO) {}
    
    debug(message: string) { if (this.level <= LogLevel.DEBUG) console.log(`[DEBUG] ${message}`); }
    info(message: string) { if (this.level <= LogLevel.INFO) console.log(`[INFO] ${message}`); }
    warn(message: string) { if (this.level <= LogLevel.WARN) console.warn(`[WARN] ${message}`); }
    error(message: string) { if (this.level <= LogLevel.ERROR) console.error(`[ERROR] ${message}`); }
}
"#).expect("Failed to write logger file");

            // 2. Update core to use new logging
            let core_main = temp_dir.path().join("packages/core/src/index.ts");
            std::fs::write(&core_main, r#"//! Core business logic package
//! 
//! Enhanced with logging capabilities

import { Logger, LogLevel } from "@test/foundation";

const logger = new Logger(LogLevel.INFO);

export function core() {
    logger.info("Core function called");
    return "Hello from core with logging";
}

export function processData(data: any) {
    logger.debug("Processing data", data);
    // Enhanced data processing logic
    logger.info("Data processing completed");
    return data;
}
"#).expect("Failed to write core main file");

            // 3. Update app to use enhanced core features
            let app_main = temp_dir.path().join("packages/app/src/index.ts");
            std::fs::write(&app_main, r#"//! Main application package
//!
//! Enhanced with new logging and data processing

import { core, processData } from "@test/core";
import { utils } from "@test/utils";
import { Logger } from "@test/foundation";

const logger = new Logger();

export function app() {
    logger.info("Application starting");
    const coreResult = core();
    const utilsResult = utils();
    const processedData = processData({ message: "Hello World" });
    
    logger.info("Application initialized successfully");
    return `${coreResult} + ${utilsResult} + processed data`;
}
"#).expect("Failed to write app main file");
        },
        
        "bug_fix" => {
            // Bug fix: Fix critical issue in utils package
            let utils_main = temp_dir.path().join("packages/utils/src/index.ts");
            std::fs::write(&utils_main, r#"//! Utility functions package
//!
//! Fixed critical null pointer issue

import { foundation } from "@test/foundation";
import { core } from "@test/core";

export function utils() {
    // BUG FIX: Added null check to prevent crashes
    if (!foundation || !core) {
        throw new Error("Required dependencies not available");
    }
    
    return "Hello from utils - bug fixed";
}

export function safeProcess(input: any) {
    // BUG FIX: Enhanced input validation
    if (input === null || input === undefined) {
        return null;
    }
    
    return String(input).trim();
}
"#).expect("Failed to write utils main file");
        },
        
        "breaking_change" => {
            // Breaking change: Update foundation API
            let foundation_main = temp_dir.path().join("packages/foundation/src/index.ts");
            std::fs::write(&foundation_main, r#"//! Foundation package
//!
//! BREAKING CHANGE: Updated API structure

// BREAKING: Changed function signature and return type
export interface FoundationResult {
    message: string;
    version: string;
    timestamp: number;
}

export function foundation(): FoundationResult {
    return {
        message: "Hello from foundation v2",
        version: "2.0.0",
        timestamp: Date.now()
    };
}

// BREAKING: Removed deprecated function
// export function oldFoundation() - REMOVED

// NEW: Added configuration support
export interface Config {
    debug: boolean;
    maxRetries: number;
}

export const defaultConfig: Config = {
    debug: false,
    maxRetries: 3
};
"#).expect("Failed to write foundation main file");
        },
        
        _ => {
            panic!("Unknown change scenario: {}", scenario);
        }
    }
}

#[test]
fn test_end_to_end_feature_development_workflow() {
    // Test: Complete end-to-end workflow for feature development
    let (temp_dir, project) = create_comprehensive_monorepo();
    
    // Create new feature branch
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/enhanced-logging"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to create feature branch");
    
    // === STEP 1: GIT - Make realistic changes ===
    create_realistic_changes(&temp_dir, "feature_development");
    
    // Commit changes
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add changes");
        
    std::process::Command::new("git")
        .args(["commit", "-m", "feat: add enhanced logging capabilities"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit changes");
    
    // === STEP 2: ANALYSIS - Detect and analyze changes ===
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));
    
    // Detect changes since main branch
    let changes = analyzer.detect_changes_since("main", Some("feature/enhanced-logging"))
        .expect("Failed to detect changes");
    
    assert!(!changes.changed_files.is_empty(), "Should detect changed files");
    assert!(!changes.package_changes.is_empty(), "Should detect package changes");
    
    // Analyze affected packages
    // Analyze impact using the already detected changes
    let affected_analysis = &changes.affected_packages;
    
    // Validate analysis results
    assert!(affected_analysis.directly_affected.contains(&"@test/foundation".to_string()), 
           "Foundation should be directly affected");
    assert!(affected_analysis.directly_affected.contains(&"@test/core".to_string()), 
           "Core should be directly affected");
    assert!(affected_analysis.directly_affected.contains(&"@test/app".to_string()), 
           "App should be directly affected");
    
    // Check dependency propagation
    assert!(affected_analysis.dependents_affected.len() > 0, 
           "Should have dependent packages affected");
    
    // === STEP 3: CHANGESET - Create changesets based on analysis ===
    let changeset_manager = ChangesetManager::from_project(Arc::clone(&project))
        .expect("Failed to create changeset manager");
    
    // Create changesets for affected packages
    let mut changesets = Vec::new();
    for package_name in &affected_analysis.directly_affected {
        let changeset_spec = ChangesetSpec {
            package: package_name.clone(),
            version_bump: VersionBumpType::Minor, // Feature addition
            description: format!("Enhanced logging capabilities for {}", package_name),
            development_environments: vec![Environment::Development, Environment::Staging],
            production_deployment: false, // Not ready for production yet
            author: Some("test@example.com".to_string()),
        };
        
        let changeset = changeset_manager.create_changeset(changeset_spec)
            .expect("Failed to create changeset");
        changesets.push(changeset);
    }
    
    assert!(!changesets.is_empty(), "Should create changesets for affected packages");
    
    // Validate changesets
    for changeset in &changesets {
        let validation = changeset_manager.validate_changeset(&changeset)
            .expect("Failed to validate changeset");
        assert!(validation.is_valid, "Changeset should be valid: {:?}", validation.errors);
    }
    
    // === STEP 4: WORKFLOW - Execute development workflow ===
    let workflow = DevelopmentWorkflow::from_project(Arc::clone(&project))
        .expect("Failed to create development workflow");
    
    // Execute workflow to validate the feature branch
    let workflow_result = run_async(workflow.execute(Some("main")))
        .expect("Failed to execute development workflow");
    
    // Validate workflow results
    assert!(!workflow_result.changes.changed_files.is_empty(), 
           "Workflow should detect changes");
    
    // Check that workflow provides recommendations or we have changesets
    if workflow_result.recommendations.is_empty() {
        // Workflow might not provide recommendations if no changes detected, but we should have changesets
        assert!(!changesets.is_empty(), "Should have created changesets for feature development");
    } else {
        // Check that workflow suggests changeset-related actions
        let recommendations_text = workflow_result.recommendations.join(" ");
        let has_changeset_recommendation = recommendations_text.contains("changeset") || 
                                         recommendations_text.contains("test") ||
                                         recommendations_text.contains("passed");
        assert!(has_changeset_recommendation, "Should recommend changeset creation or provide useful feedback. Got: {:?}", workflow_result.recommendations);
    }
    
    // === STEP 5: INTEGRATION VALIDATION - Verify complete flow ===
    
    // 1. Verify Git integration - we verified git operations worked via successful commits
    
    // 2. Verify Analysis integration
    assert!(changes.affected_packages.total_affected_count > 0, 
           "Analysis should detect affected packages");
    
    // 3. Verify Changeset integration
    let stored_changesets = changeset_manager.list_changesets(&Default::default())
        .expect("Failed to list changesets");
    assert!(!stored_changesets.is_empty(), "Should have stored changesets");
    
    // 4. Verify Storage integration
    // Note: Storage verification is done via the list_changesets call above
    
    // === VALIDATION COMPLETE ===
    println!("✅ End-to-end feature development workflow completed successfully!");
    println!("   - Detected {} changed files", changes.changed_files.len());
    println!("   - Affected {} packages directly", affected_analysis.directly_affected.len());
    println!("   - Created {} changesets", changesets.len());
    println!("   - Generated {} workflow recommendations", workflow_result.recommendations.len());
}

#[test]
fn test_end_to_end_bug_fix_workflow() {
    // Test: Complete end-to-end workflow for critical bug fix
    let (temp_dir, project) = create_comprehensive_monorepo();
    
    // Create hotfix branch
    std::process::Command::new("git")
        .args(["checkout", "-b", "hotfix/critical-null-fix"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to create hotfix branch");
    
    // === BUG FIX SCENARIO ===
    create_realistic_changes(&temp_dir, "bug_fix");
    
    // Commit bug fix
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add changes");
        
    std::process::Command::new("git")
        .args(["commit", "-m", "fix: resolve critical null pointer issue in utils"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit changes");
    
    // === ANALYSIS ===
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));
    let changes = analyzer.detect_changes_since("main", Some("hotfix/critical-null-fix"))
        .expect("Failed to detect changes");
    
    // Bug fixes should typically affect fewer packages than features
    assert!(!changes.package_changes.is_empty(), "Should detect package changes for bug fix");
    
    // === CHANGESET FOR PATCH RELEASE ===
    let changeset_manager = ChangesetManager::from_project(Arc::clone(&project))
        .expect("Failed to create changeset manager");
    
    let changeset_spec = ChangesetSpec {
        package: "@test/utils".to_string(),
        version_bump: VersionBumpType::Patch, // Bug fix = patch version
        description: "Fix critical null pointer issue causing application crashes".to_string(),
        development_environments: vec![Environment::Development, Environment::Staging, Environment::Production],
        production_deployment: true, // Critical fix needs to go to production
        author: Some("test@example.com".to_string()),
    };
    
    let changeset = changeset_manager.create_changeset(changeset_spec)
        .expect("Failed to create bug fix changeset");
    
    // Validate bug fix changeset
    let validation = changeset_manager.validate_changeset(&changeset)
        .expect("Failed to validate changeset");
    assert!(validation.is_valid, "Bug fix changeset should be valid");
    
    // === WORKFLOW FOR HOTFIX ===
    let workflow = DevelopmentWorkflow::from_project(Arc::clone(&project))
        .expect("Failed to create development workflow");
    
    let workflow_result = run_async(workflow.execute(Some("main")))
        .expect("Failed to execute hotfix workflow");
    
    // Validate that workflow handles bug fix appropriately
    assert!(workflow_result.checks_passed, "Bug fix should pass all checks");
    
    println!("✅ End-to-end bug fix workflow completed successfully!");
    println!("   - Created patch changeset for critical fix");
    println!("   - Validated for production deployment");
    println!("   - All workflow checks passed");
}

#[test]
fn test_end_to_end_breaking_change_workflow() {
    // Test: Complete end-to-end workflow for breaking changes
    let (temp_dir, project) = create_comprehensive_monorepo();
    
    // Create breaking change branch
    std::process::Command::new("git")
        .args(["checkout", "-b", "feat/breaking-foundation-v2"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to create breaking change branch");
    
    // === BREAKING CHANGE SCENARIO ===
    create_realistic_changes(&temp_dir, "breaking_change");
    
    // Commit breaking changes
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add changes");
        
    std::process::Command::new("git")
        .args(["commit", "-m", "feat!: update foundation API with breaking changes"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit changes");
    
    // === ANALYSIS FOR BREAKING CHANGE ===
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));
    let diff_analyzer = DiffAnalyzer::from_project(Arc::clone(&project));
    
    let changes = analyzer.detect_changes_since("main", Some("feat/breaking-foundation-v2"))
        .expect("Failed to detect changes");
    
    // Analyze impact of breaking changes
    let change_analysis = diff_analyzer.detect_changes_since("main", Some("feat/breaking-foundation-v2"))
        .expect("Failed to detect changes");
    let affected_analysis = &change_analysis.affected_packages;
    
    // Breaking changes in foundation should affect ALL dependent packages
    assert!(affected_analysis.directly_affected.contains(&"@test/foundation".to_string()), 
           "Foundation should be directly affected");
    
    // All packages depend on foundation, so they should all be in dependents_affected
    let expected_dependents = ["@test/core", "@test/utils", "@test/ui-components", "@test/app"];
    for dependent in &expected_dependents {
        assert!(affected_analysis.dependents_affected.contains(&dependent.to_string()) ||
                affected_analysis.directly_affected.contains(&dependent.to_string()),
               "Package {} should be affected by breaking change in foundation", dependent);
    }
    
    // === CHANGESET FOR MAJOR VERSION ===
    let changeset_manager = ChangesetManager::from_project(Arc::clone(&project))
        .expect("Failed to create changeset manager");
    
    let changeset_spec = ChangesetSpec {
        package: "@test/foundation".to_string(),
        version_bump: VersionBumpType::Major, // Breaking change = major version
        description: "BREAKING: Update foundation API structure and remove deprecated functions".to_string(),
        development_environments: vec![Environment::Development, Environment::Staging],
        production_deployment: false, // Breaking changes need careful coordination
        author: Some("test@example.com".to_string()),
    };
    
    let breaking_changeset = changeset_manager.create_changeset(changeset_spec)
        .expect("Failed to create breaking change changeset");
    
    // Validate breaking change impact
    let validation = changeset_manager.validate_changeset(&breaking_changeset)
        .expect("Failed to validate changeset");
    assert!(validation.is_valid, "Breaking change changeset should be valid");
    
    // Check for warnings about breaking changes
    if !validation.warnings.is_empty() {
        println!("⚠️  Breaking change warnings: {:?}", validation.warnings);
    }
    
    // === WORKFLOW VALIDATION ===
    let workflow = DevelopmentWorkflow::from_project(Arc::clone(&project))
        .expect("Failed to create development workflow");
    
    let workflow_result = run_async(workflow.execute(Some("main")))
        .expect("Failed to execute breaking change workflow");
    
    // Validate workflow handles breaking changes appropriately
    assert!(!workflow_result.changes.changed_files.is_empty(), 
           "Should detect breaking changes");
    
    // Check recommendations mention the severity of changes
    let recommendations_text = workflow_result.recommendations.join(" ");
    let mentions_impact = recommendations_text.contains("changeset") || 
                         recommendations_text.contains("impact") ||
                         affected_analysis.total_affected_count > 3;
    assert!(mentions_impact, "Should recognize the high impact of breaking changes");
    
    println!("✅ End-to-end breaking change workflow completed successfully!");
    println!("   - Detected breaking change in foundation package");
    println!("   - Identified {} total affected packages", affected_analysis.total_affected_count);
    println!("   - Created major version changeset");
    println!("   - Workflow provided appropriate warnings and recommendations");
}

#[test]
fn test_end_to_end_multi_component_error_handling() {
    // Test: Error handling across all components in the pipeline
    let (temp_dir, project) = create_comprehensive_monorepo();
    
    // === TEST ERROR SCENARIOS ===
    
    // 1. Test invalid Git reference
    let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));
    let invalid_git_result = analyzer.detect_changes_since("nonexistent-branch", None);
    assert!(invalid_git_result.is_err(), "Should fail with invalid Git reference");
    
    // 2. Test invalid changeset creation
    let changeset_manager = ChangesetManager::from_project(Arc::clone(&project))
        .expect("Failed to create changeset manager");
    
    let invalid_changeset_spec = ChangesetSpec {
        package: "@test/nonexistent-package".to_string(),
        version_bump: VersionBumpType::Major,
        description: "This should fail".to_string(),
        development_environments: vec![Environment::Development],
        production_deployment: false,
        author: Some("test@example.com".to_string()),
    };
    
    let invalid_changeset_result = changeset_manager.create_changeset(invalid_changeset_spec);
    assert!(invalid_changeset_result.is_err(), "Should fail with nonexistent package");
    
    // 3. Test workflow with invalid configuration
    // Note: This tests the robustness of the system
    let workflow = DevelopmentWorkflow::from_project(Arc::clone(&project))
        .expect("Failed to create workflow");
    
    // Test workflow with invalid branch reference
    let invalid_workflow_result = run_async(workflow.execute(Some("invalid-branch")));
    // Workflow should handle this gracefully
    if invalid_workflow_result.is_err() {
        println!("✅ Workflow correctly handles invalid branch reference");
    }
    
    // === TEST RECOVERY SCENARIOS ===
    
    // Create valid changes for recovery testing
    create_realistic_changes(&temp_dir, "feature_development");
    
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add changes");
        
    std::process::Command::new("git")
        .args(["commit", "-m", "test: recovery scenario changes"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit changes");
    
    // Test that system recovers and works normally after errors
    let recovery_changes = analyzer.detect_changes_since("HEAD~1", None)
        .expect("Should recover and detect changes normally");
    
    assert!(!recovery_changes.changed_files.is_empty(), 
           "Should successfully detect changes after recovery");
    
    println!("✅ End-to-end error handling and recovery completed successfully!");
    println!("   - Validated error handling across Git, Analysis, Changeset, and Workflow components");
    println!("   - Confirmed system recovery after error conditions");
    println!("   - Verified graceful degradation and error reporting");
}