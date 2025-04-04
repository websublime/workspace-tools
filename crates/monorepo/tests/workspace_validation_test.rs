use rstest::*;
use tempfile::TempDir;

use sublime_monorepo_tools::{DiscoveryOptions, ValidationOptions, WorkspaceManager};

mod fixtures;

#[rstest]
#[allow(clippy::print_stdout)]
fn test_workspace_validation(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    // Setup workspace
    let workspace_manager = WorkspaceManager::new();
    let workspace = workspace_manager
        .discover_workspace(temp_dir.path(), &DiscoveryOptions::default().auto_detect_root(false))
        .expect("Failed to discover workspace");

    // Test basic validation
    let validation_result = workspace.validate().expect("Validation should succeed");

    // Check validation results
    println!("Basic validation issues: {}", validation_result.has_issues());

    if validation_result.has_critical_issues() {
        println!("Critical issues detected:");
        for issue in validation_result.critical_issues() {
            println!("  - {}", issue.message());
        }
    }

    if validation_result.has_warnings() {
        println!("Warnings detected:");
        for issue in validation_result.warnings() {
            println!("  - {}", issue.message());
        }
    }
}

// Add a more focused test for treat_unresolved_as_external option
#[rstest]
#[allow(clippy::print_stdout)]
fn test_external_dependencies_validation(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    // Setup workspace
    let workspace_manager = WorkspaceManager::new();
    let workspace = workspace_manager
        .discover_workspace(temp_dir.path(), &DiscoveryOptions::default().auto_detect_root(false))
        .expect("Failed to discover workspace");

    // Print out package-major and package-tom dependencies for reference
    let major_pkg =
        workspace.get_package("@scope/package-major").expect("package-major should exist");
    let tom_pkg = workspace.get_package("@scope/package-tom").expect("package-tom should exist");

    println!("package-major dependencies:");
    for dep in major_pkg.borrow().package.borrow().dependencies() {
        println!("  - {}", dep.borrow().name());
    }

    println!("package-tom dependencies:");
    for dep in tom_pkg.borrow().package.borrow().dependencies() {
        println!("  - {}", dep.borrow().name());
    }

    // External dependencies to test with
    let pulseio_core = "@websublime/pulseio-core";
    let typescript = "typescript";

    // 1. Test validation treating unresolved deps as errors (default)
    let strict_options = ValidationOptions::new().treat_unresolved_as_external(false);

    let strict_validation =
        workspace.validate_with_options(&strict_options).expect("Strict validation should succeed");

    println!("Strict validation (external deps as errors):");
    println!("  Has issues: {}", strict_validation.has_issues());
    println!("  Has critical issues: {}", strict_validation.has_critical_issues());

    // Count unresolved dependency issues for our test packages
    let unresolved_count = strict_validation
        .critical_issues()
        .iter()
        .filter(|issue| {
            issue.message().contains(pulseio_core) || issue.message().contains(typescript)
        })
        .count();

    println!("  Unresolved external deps counted as critical: {unresolved_count}");
    assert!(unresolved_count > 0, "Should have critical issues for external packages");

    // 2. Test validation treating unresolved deps as external (permissive)
    let permissive_options = ValidationOptions::new().treat_unresolved_as_external(true);

    let permissive_validation = workspace
        .validate_with_options(&permissive_options)
        .expect("Permissive validation should succeed");

    println!("Permissive validation (external deps ignored):");
    println!("  Has issues: {}", permissive_validation.has_issues());
    println!("  Has critical issues: {}", permissive_validation.has_critical_issues());

    // Count unresolved dependency issues - should be fewer or none
    let permissive_unresolved_count = permissive_validation
        .critical_issues()
        .iter()
        .filter(|issue| {
            issue.message().contains(pulseio_core) || issue.message().contains(typescript)
        })
        .count();

    println!("  Unresolved external deps counted as critical: {permissive_unresolved_count}");
    assert!(
        permissive_unresolved_count < unresolved_count,
        "Should have fewer critical issues when treating external deps as acceptable"
    );

    // 3. Test with specific internal packages defined
    let specific_options = ValidationOptions::new()
        .treat_unresolved_as_external(true)
        .with_internal_dependencies(vec![pulseio_core]);

    let specific_validation = workspace
        .validate_with_options(&specific_options)
        .expect("Specific validation should succeed");

    println!("Specific validation (selected internals):");
    println!("  Has issues: {}", specific_validation.has_issues());
    println!("  Has critical issues: {}", specific_validation.has_critical_issues());

    // Count issues related to pulseio_core (should be critical again since we specified it as internal)
    let pulseio_issues = specific_validation
        .critical_issues()
        .iter()
        .filter(|issue| issue.message().contains(pulseio_core))
        .count();

    // Count issues related to typescript (should remain non-critical)
    let typescript_issues = specific_validation
        .critical_issues()
        .iter()
        .filter(|issue| issue.message().contains(typescript))
        .count();

    println!("  {pulseio_core} issues (should be critical): {pulseio_issues}");
    println!("  {typescript} issues (should not be critical): {typescript_issues}");

    // pulseio_core should have issues since we marked it as internal
    assert!(
        pulseio_issues > 0,
        "{pulseio_core} should be treated as internal and raise critical issues"
    );

    // typescript should have no critical issues
    assert_eq!(
        typescript_issues, 0,
        "{typescript} should be treated as external and not raise critical issues",
    );
}
