//! # Changeset Examples
//!
//! This module provides comprehensive examples demonstrating how to use
//! the changeset system for package version management.
//!
//! ## What
//!
//! Contains practical examples showing:
//! - Creating and configuring changesets
//! - Working with different change types
//! - Multi-environment releases
//! - Serialization and deserialization
//! - Validation and error handling
//!
//! ## How
//!
//! Each example is self-contained and demonstrates specific use cases
//! with complete setup and expected outcomes.
//!
//! ## Why
//!
//! Examples serve as documentation, testing, and reference for proper
//! usage of the changeset system in real-world scenarios.

#![allow(dead_code)]

use chrono::{TimeZone, Utc};
use sublime_pkg_tools::{
    changeset::{ChangeEntry, Changeset, ChangesetPackage, ReleaseInfo},
    Version, VersionBump,
};

/// Example: Creating a simple feature changeset
///
/// This example shows how to create a changeset for a new feature
/// that affects a single package with direct changes.
pub fn example_simple_feature_changeset() -> Changeset {
    let mut changeset =
        Changeset::new("feat/user-authentication".to_string(), "developer@example.com".to_string());

    // Add development and staging as target environments
    changeset.add_target_environment("staging".to_string());

    // Create a package with feature changes
    let auth_package = ChangesetPackage::new_direct_changes(
        "@myorg/auth-service".to_string(),
        VersionBump::Minor,
        Version::new(1, 2, 3).into(),
        Version::new(1, 3, 0).into(),
        vec!["abc123def456".to_string(), "def456abc789".to_string()],
    );

    // Add specific change entries
    let mut package_with_changes = auth_package;
    package_with_changes.add_change(ChangeEntry::feature(
        "Add OAuth2 authentication support",
        false,
        Some("abc123def456"),
    ));
    package_with_changes.add_change(ChangeEntry::feature(
        "Add JWT token validation",
        false,
        Some("def456abc789"),
    ));

    changeset.add_package(package_with_changes);

    changeset
}

/// Example: Creating a multi-package changeset with dependencies
///
/// This example demonstrates a changeset that affects multiple packages
/// due to dependency propagation.
pub fn example_multi_package_changeset() -> Changeset {
    let mut changeset = Changeset::new(
        "feat/shared-library-update".to_string(),
        "team-lead@example.com".to_string(),
    );

    // Target multiple environments
    changeset.add_target_environment("staging".to_string());
    changeset.add_target_environment("prod".to_string());

    // 1. Main package with direct changes (minor bump)
    let mut shared_lib = ChangesetPackage::new_direct_changes(
        "@myorg/shared-lib".to_string(),
        VersionBump::Minor,
        Version::new(2, 1, 0).into(),
        Version::new(2, 2, 0).into(),
        vec!["ghi789jkl012".to_string()],
    );
    shared_lib.add_change(ChangeEntry::feature(
        "Add new utility functions for data processing",
        false,
        Some("ghi789jkl012"),
    ));

    // 2. Dependent package (patch bump due to dependency update)
    let user_service = ChangesetPackage::new_dependency_update(
        "@myorg/user-service".to_string(),
        VersionBump::Patch,
        Version::new(3, 0, 1).into(),
        Version::new(3, 0, 2).into(),
        "@myorg/shared-lib".to_string(),
        "2.1.0".to_string(),
        "2.2.0".to_string(),
    );

    // 3. Another dependent package
    let notification_service = ChangesetPackage::new_dependency_update(
        "@myorg/notification-service".to_string(),
        VersionBump::Patch,
        Version::new(1, 5, 2).into(),
        Version::new(1, 5, 3).into(),
        "@myorg/shared-lib".to_string(),
        "2.1.0".to_string(),
        "2.2.0".to_string(),
    );

    changeset.add_package(shared_lib);
    changeset.add_package(user_service);
    changeset.add_package(notification_service);

    changeset
}

/// Example: Creating a breaking change changeset
///
/// This example shows how to handle breaking changes that require
/// major version bumps.
pub fn example_breaking_change_changeset() -> Changeset {
    let mut changeset =
        Changeset::new("feat/api-v2".to_string(), "api-team@example.com".to_string());

    // Only target staging first for breaking changes
    changeset.add_target_environment("staging".to_string());

    // Create package with breaking changes
    let mut api_package = ChangesetPackage::new_direct_changes(
        "@myorg/api-server".to_string(),
        VersionBump::Major,
        Version::new(1, 5, 0).into(),
        Version::new(2, 0, 0).into(),
        vec!["mno345pqr678".to_string(), "pqr678stu901".to_string()],
    );

    // Add breaking change entries
    api_package.add_change(ChangeEntry::breaking(
        "feat",
        "Remove deprecated /v1/users endpoint",
        Some("mno345pqr678"),
    ));
    api_package.add_change(ChangeEntry::breaking(
        "feat",
        "Change authentication response format",
        Some("pqr678stu901"),
    ));

    // Also add some non-breaking improvements
    api_package.add_change(ChangeEntry::feature(
        "Add new /v2/users endpoint with pagination",
        false,
        Some("mno345pqr678"),
    ));

    changeset.add_package(api_package);

    changeset
}

/// Example: Creating a bug fix changeset
///
/// This example demonstrates a simple bug fix that requires patch bumps.
pub fn example_bug_fix_changeset() -> Changeset {
    let mut changeset =
        Changeset::new("fix/memory-leak".to_string(), "bugfix-team@example.com".to_string());

    // Quick fix - deploy to all environments
    changeset.add_target_environment("staging".to_string());
    changeset.add_target_environment("prod".to_string());

    // Create package with bug fix
    let mut service_package = ChangesetPackage::new_direct_changes(
        "@myorg/data-processor".to_string(),
        VersionBump::Patch,
        Version::new(2, 3, 1).into(),
        Version::new(2, 3, 2).into(),
        vec!["stu901vwx234".to_string()],
    );

    service_package
        .add_change(ChangeEntry::fix("Fix memory leak in batch processing", Some("stu901vwx234")));

    changeset.add_package(service_package);

    changeset
}

/// Example: Applied changeset with release information
///
/// This example shows what a changeset looks like after it has been
/// applied and released to multiple environments.
pub fn example_applied_changeset() -> Changeset {
    let mut changeset = example_simple_feature_changeset();

    // Create release information
    let mut release_info = ReleaseInfo::new_with_timestamp(
        Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 0).unwrap(),
        "ci-deploy-bot".to_string(),
        "abc123def456ghi789".to_string(),
    );

    // Add environment releases
    release_info.add_environment_release_with_timestamp(
        "dev".to_string(),
        "v1.3.0-dev".to_string(),
        Utc.with_ymd_and_hms(2024, 1, 15, 14, 32, 0).unwrap(),
    );

    release_info.add_environment_release_with_timestamp(
        "staging".to_string(),
        "v1.3.0-staging".to_string(),
        Utc.with_ymd_and_hms(2024, 1, 15, 14, 45, 0).unwrap(),
    );

    // Apply the release info
    changeset.apply_release_info(release_info);

    changeset
}

/// Example: Dev dependency update changeset
///
/// This example shows a changeset that updates development dependencies,
/// which typically only require patch bumps.
pub fn example_dev_dependency_changeset() -> Changeset {
    let mut changeset = Changeset::new(
        "chore/update-dev-deps".to_string(),
        "maintenance-bot@example.com".to_string(),
    );

    // Dev dependency updates usually only go to dev environment first
    // No need to add staging/prod since this is just dev tooling

    // Update TypeScript definitions
    let frontend_package = ChangesetPackage::new_dev_dependency_update(
        "@myorg/frontend-app".to_string(),
        VersionBump::Patch,
        Version::new(1, 0, 5).into(),
        Version::new(1, 0, 6).into(),
        "@types/node".to_string(),
        "18.15.0".to_string(),
        "20.10.0".to_string(),
    );

    // Update test framework
    let backend_package = ChangesetPackage::new_dev_dependency_update(
        "@myorg/backend-api".to_string(),
        VersionBump::Patch,
        Version::new(2, 1, 3).into(),
        Version::new(2, 1, 4).into(),
        "jest".to_string(),
        "29.0.0".to_string(),
        "29.5.0".to_string(),
    );

    changeset.add_package(frontend_package);
    changeset.add_package(backend_package);

    changeset
}

/// Example: Serializing and deserializing a changeset
///
/// This example demonstrates how changesets are serialized to JSON
/// for storage and later deserialized for processing.
pub fn example_changeset_serialization() -> Result<String, serde_json::Error> {
    let changeset = example_multi_package_changeset();

    // Serialize to pretty JSON
    let json = serde_json::to_string_pretty(&changeset)?;

    // Verify we can deserialize it back
    let _deserialized: Changeset = serde_json::from_str(&json)?;

    Ok(json)
}

/// Example: Validating changesets with different configurations
///
/// This example shows how to validate changesets against different
/// environment configurations.
pub fn example_changeset_validation() -> Vec<Result<(), sublime_pkg_tools::error::ChangesetError>> {
    let mut results = Vec::new();

    // Valid changeset
    let valid_changeset = example_simple_feature_changeset();
    results.push(valid_changeset.validate(None).map_err(Into::into));

    // Validate against specific environments
    let available_envs = vec!["dev".to_string(), "staging".to_string(), "prod".to_string()];
    results.push(valid_changeset.validate(Some(&available_envs)).map_err(Into::into));

    // Invalid changeset - empty branch
    let mut invalid_changeset = Changeset::default();
    invalid_changeset.add_package(ChangesetPackage::new_direct_changes(
        "test-pkg".to_string(),
        VersionBump::Patch,
        Version::new(1, 0, 0).into(),
        Version::new(1, 0, 1).into(),
        vec!["abc123".to_string()],
    ));
    results.push(invalid_changeset.validate(None).map_err(Into::into));

    results
}

/// Example: Working with changeset metadata
///
/// This example demonstrates how to extract and work with various
/// metadata from changesets.
pub fn example_changeset_metadata() {
    let changeset = example_applied_changeset();

    // Basic metadata
    println!("Changeset ID: {}", changeset.generate_id());
    println!("Filename: {}", changeset.generate_filename());
    println!("Branch: {}", changeset.branch);
    println!("Author: {}", changeset.author);
    println!("Is Applied: {}", changeset.is_applied());

    // Package information
    println!("Affected packages: {:?}", changeset.get_package_names());
    println!("Bump summary: {:?}", changeset.get_bump_summary());

    // Release information
    if let Some(release_info) = &changeset.release_info {
        println!("Applied by: {}", release_info.applied_by);
        println!("Git commit: {}", release_info.git_commit);
        println!("Environment count: {}", release_info.get_environment_count());
        println!("Tags created: {:?}", release_info.get_tags_summary());
    }

    // Package details
    for package in &changeset.packages {
        println!("Package: {}", package.name);
        println!("  Bump: {:?}", package.bump);
        println!("  Has breaking changes: {}", package.has_breaking_changes());
        println!("  Change type summary: {:?}", package.get_change_type_summary());

        if let Some(dep_info) = package.get_dependency_info() {
            println!("  Dependency update: {} {} -> {}", dep_info.0, dep_info.1, dep_info.2);
        }

        for commit in package.get_commits() {
            println!("  Commit: {}", commit);
        }
    }
}

/// Example: Complex changeset with multiple change types
///
/// This example creates a comprehensive changeset that includes
/// features, fixes, documentation, and breaking changes.
pub fn example_comprehensive_changeset() -> Changeset {
    let mut changeset =
        Changeset::new("release/v2.0.0".to_string(), "release-manager@example.com".to_string());

    // Target all environments for major release
    changeset.add_target_environment("staging".to_string());
    changeset.add_target_environment("prod".to_string());

    // Main package with mixed changes
    let mut main_package = ChangesetPackage::new_direct_changes(
        "@myorg/core-service".to_string(),
        VersionBump::Major,
        Version::new(1, 9, 5).into(),
        Version::new(2, 0, 0).into(),
        vec![
            "feat123".to_string(),
            "fix456".to_string(),
            "docs789".to_string(),
            "breaking012".to_string(),
        ],
    );

    // Add various types of changes
    main_package.add_change(ChangeEntry::feature(
        "Add GraphQL API support",
        false,
        Some("feat123"),
    ));

    main_package
        .add_change(ChangeEntry::fix("Fix race condition in concurrent requests", Some("fix456")));

    main_package.add_change(ChangeEntry::new(
        "docs",
        "Add comprehensive API documentation",
        false,
        Some("docs789"),
    ));

    main_package.add_change(ChangeEntry::breaking(
        "feat",
        "Remove legacy REST endpoints",
        Some("breaking012"),
    ));

    main_package.add_change(ChangeEntry::new(
        "perf",
        "Optimize database query performance by 40%",
        false,
        Some("perf345"),
    ));

    changeset.add_package(main_package);

    // Supporting package updates
    let utils_package = ChangesetPackage::new_dependency_update(
        "@myorg/utils".to_string(),
        VersionBump::Minor,
        Version::new(0, 8, 2).into(),
        Version::new(0, 9, 0).into(),
        "@myorg/core-service".to_string(),
        "1.9.5".to_string(),
        "2.0.0".to_string(),
    );

    changeset.add_package(utils_package);

    changeset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_feature_changeset() {
        let changeset = example_simple_feature_changeset();

        assert_eq!(changeset.branch, "feat/user-authentication");
        assert_eq!(changeset.author, "developer@example.com");
        assert_eq!(changeset.packages.len(), 1);
        assert!(changeset.releases.contains(&"dev".to_string()));
        assert!(changeset.releases.contains(&"staging".to_string()));
        assert!(changeset.is_pending());

        let package = &changeset.packages[0];
        assert_eq!(package.name, "@myorg/auth-service");
        assert_eq!(package.bump, VersionBump::Minor);
        assert_eq!(package.changes.len(), 2);
        assert!(package.is_direct_changes());
        assert!(!package.has_breaking_changes());
    }

    #[test]
    fn test_multi_package_changeset() {
        let changeset = example_multi_package_changeset();

        assert_eq!(changeset.packages.len(), 3);

        // Check main package
        let shared_lib = changeset.find_package("@myorg/shared-lib").unwrap();
        assert_eq!(shared_lib.bump, VersionBump::Minor);
        assert!(shared_lib.is_direct_changes());

        // Check dependent packages
        let user_service = changeset.find_package("@myorg/user-service").unwrap();
        assert_eq!(user_service.bump, VersionBump::Patch);
        assert!(user_service.is_dependency_update());
        assert_eq!(user_service.dependency, Some("@myorg/shared-lib".to_string()));
    }

    #[test]
    fn test_breaking_change_changeset() {
        let changeset = example_breaking_change_changeset();

        assert_eq!(changeset.packages.len(), 1);

        let package = &changeset.packages[0];
        assert_eq!(package.bump, VersionBump::Major);
        assert!(package.has_breaking_changes());
        assert_eq!(package.changes.len(), 3);

        // Count breaking vs non-breaking changes
        let breaking_count = package.changes.iter().filter(|c| c.is_breaking()).count();
        let non_breaking_count = package.changes.iter().filter(|c| !c.is_breaking()).count();

        assert_eq!(breaking_count, 2);
        assert_eq!(non_breaking_count, 1);
    }

    #[test]
    fn test_applied_changeset() {
        let changeset = example_applied_changeset();

        assert!(changeset.is_applied());
        assert!(!changeset.is_pending());

        let release_info = changeset.release_info.as_ref().unwrap();
        assert_eq!(release_info.applied_by, "ci-deploy-bot");
        assert_eq!(release_info.get_environment_count(), 2);
        assert!(release_info.has_environment("dev"));
        assert!(release_info.has_environment("staging"));

        assert_eq!(release_info.get_tag_for_environment("dev"), Some("v1.3.0-dev"));
        assert_eq!(release_info.get_tag_for_environment("staging"), Some("v1.3.0-staging"));
    }

    #[test]
    fn test_dev_dependency_changeset() {
        let changeset = example_dev_dependency_changeset();

        assert_eq!(changeset.packages.len(), 2);

        for package in &changeset.packages {
            assert!(package.is_dev_dependency_update());
            assert_eq!(package.bump, VersionBump::Patch);
            assert!(package.dependency.is_some());
        }
    }

    #[test]
    fn test_changeset_serialization() {
        let result = example_changeset_serialization();
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(!json.is_empty());
        assert!(json.contains("feat/shared-library-update"));
        assert!(json.contains("@myorg/shared-lib"));
    }

    #[test]
    fn test_changeset_validation() {
        let results = example_changeset_validation();

        // First two should be valid
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());

        // Third should be invalid (empty branch)
        assert!(results[2].is_err());
    }

    #[test]
    fn test_comprehensive_changeset() {
        let changeset = example_comprehensive_changeset();

        assert_eq!(changeset.packages.len(), 2);

        let main_package = changeset.find_package("@myorg/core-service").unwrap();
        assert_eq!(main_package.bump, VersionBump::Major);
        assert!(main_package.has_breaking_changes());
        assert_eq!(main_package.changes.len(), 5);

        let change_summary = main_package.get_change_type_summary();
        assert_eq!(change_summary.get("feat"), Some(&2)); // 1 normal + 1 breaking
        assert_eq!(change_summary.get("fix"), Some(&1));
        assert_eq!(change_summary.get("docs"), Some(&1));
        assert_eq!(change_summary.get("perf"), Some(&1));
    }

    #[test]
    fn test_changeset_metadata_extraction() {
        let changeset = example_applied_changeset();

        // Test ID generation
        let id = changeset.generate_id();
        assert!(id.starts_with("feat-user-authentication-"));
        assert!(id.ends_with("Z"));

        // Test filename generation
        let filename = changeset.generate_filename();
        assert!(filename.ends_with(".json"));

        // Test package names
        let package_names = changeset.get_package_names();
        assert_eq!(package_names, vec!["@myorg/auth-service"]);

        // Test bump summary
        let bump_summary = changeset.get_bump_summary();
        assert_eq!(bump_summary.get(&VersionBump::Minor), Some(&1));
    }
}

/// Main function to run all changeset examples
///
/// This function demonstrates all the changeset functionality by running
/// each example and displaying the results in a structured format.
fn main() {
    println!("🔧 Changeset Examples - Sublime Package Tools");
    println!("============================================\n");

    // Example 1: Simple Feature Changeset
    println!("📦 Example 1: Simple Feature Changeset");
    println!("---------------------------------------");
    let simple_changeset = example_simple_feature_changeset();
    println!("Branch: {}", simple_changeset.branch);
    println!("Author: {}", simple_changeset.author);
    println!("Packages: {:?}", simple_changeset.get_package_names());
    println!("Is Applied: {}", simple_changeset.is_applied());
    println!("Is Pending: {}\n", simple_changeset.is_pending());

    // Example 2: Multi-package Changeset
    println!("📦 Example 2: Multi-package Changeset");
    println!("-------------------------------------");
    let multi_changeset = example_multi_package_changeset();
    println!("Branch: {}", multi_changeset.branch);
    println!("Packages: {:?}", multi_changeset.get_package_names());
    println!("Bump Summary: {:?}\n", multi_changeset.get_bump_summary());

    // Example 3: Breaking Change Changeset
    println!("📦 Example 3: Breaking Change Changeset");
    println!("---------------------------------------");
    let breaking_changeset = example_breaking_change_changeset();
    println!("Branch: {}", breaking_changeset.branch);
    println!("Packages: {:?}", breaking_changeset.get_package_names());
    let has_breaking = breaking_changeset.packages.iter().any(|p| p.has_breaking_changes());
    println!("Has breaking changes: {}\n", has_breaking);

    // Example 4: Bug Fix Changeset
    println!("📦 Example 4: Bug Fix Changeset");
    println!("-------------------------------");
    let bugfix_changeset = example_bug_fix_changeset();
    println!("Branch: {}", bugfix_changeset.branch);
    println!("Packages: {:?}", bugfix_changeset.get_package_names());
    println!("Is Applied: {}", bugfix_changeset.is_applied());
    println!("Is Pending: {}\n", bugfix_changeset.is_pending());

    // Example 5: Applied Changeset
    println!("📦 Example 5: Applied Changeset");
    println!("------------------------------");
    let applied_changeset = example_applied_changeset();
    println!("Branch: {}", applied_changeset.branch);
    println!("Is Applied: {}", applied_changeset.is_applied());
    if let Some(release_info) = &applied_changeset.release_info {
        println!("Applied at: {}", release_info.applied_at);
        println!("Applied by: {}\n", release_info.applied_by);
    }

    // Example 6: Dev Dependency Changeset
    println!("📦 Example 6: Dev Dependency Changeset");
    println!("--------------------------------------");
    let dev_changeset = example_dev_dependency_changeset();
    println!("Branch: {}", dev_changeset.branch);
    println!("Packages: {:?}", dev_changeset.get_package_names());
    println!("Is Applied: {}", dev_changeset.is_applied());
    println!("Is Pending: {}\n", dev_changeset.is_pending());

    // Example 7: Changeset Serialization
    println!("📦 Example 7: Changeset Serialization");
    println!("-------------------------------------");
    let serialization_result = example_changeset_serialization();
    match serialization_result {
        Ok(json_string) => {
            println!("✅ Serialization successful");
            println!("JSON length: {} characters", json_string.len());
            // Try to deserialize it back to verify
            if let Ok(deserialized) = serde_json::from_str::<Changeset>(&json_string) {
                println!("✅ Deserialization successful");
                println!("Branch: {}\n", deserialized.branch);
            }
        }
        Err(e) => {
            println!("❌ Serialization failed: {}\n", e);
        }
    }

    // Example 8: Changeset Validation
    println!("📦 Example 8: Changeset Validation");
    println!("----------------------------------");
    let validation_results = example_changeset_validation();
    let mut success_count = 0;
    let mut error_count = 0;

    for result in validation_results {
        match result {
            Ok(()) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    println!("✅ Successful validations: {}", success_count);
    println!("❌ Failed validations: {}\n", error_count);

    // Example 9: Changeset Metadata
    println!("📦 Example 9: Changeset Metadata");
    println!("--------------------------------");
    example_changeset_metadata();
    println!();

    // Example 10: Comprehensive Changeset
    println!("📦 Example 10: Comprehensive Changeset");
    println!("--------------------------------------");
    let comprehensive_changeset = example_comprehensive_changeset();
    println!("Branch: {}", comprehensive_changeset.branch);
    println!("Author: {}", comprehensive_changeset.author);
    println!("Packages: {:?}", comprehensive_changeset.get_package_names());
    let has_breaking = comprehensive_changeset.packages.iter().any(|p| p.has_breaking_changes());
    println!("Has breaking changes: {}", has_breaking);
    println!("Bump Summary: {:?}", comprehensive_changeset.get_bump_summary());
    println!("ID: {}", comprehensive_changeset.generate_id());
    println!("Filename: {}", comprehensive_changeset.generate_filename());

    println!("\n🎉 All changeset examples completed successfully!");
    println!("Check the source code for detailed implementation examples.");
}
