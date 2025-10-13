//! # Version Resolver Example
//!
//! This example demonstrates the usage of the VersionResolver service
//! for determining package versions in different Git branch contexts.
//!
//! ## What
//!
//! Shows how to:
//! - Create and configure a VersionResolver
//! - Resolve current versions for packages
//! - Handle snapshot versions on development branches
//! - Work with release versions on main branch
//! - Integrate with filesystem and Git operations
//!
//! ## How
//!
//! Uses the VersionResolver with FileSystemManager and Git repository
//! to demonstrate real-world version resolution scenarios.
//!
//! ## Why
//!
//! Version resolution is a core feature that enables continuous deployment
//! while maintaining clean release versioning. This example shows practical
//! usage patterns.

use std::path::Path;

use sublime_git_tools::Repo;
use sublime_pkg_tools::{
    config::{PackageToolsConfig, VersionConfig},
    error::PackageResult,
    version::{ResolvedVersion, VersionResolver},
};
use sublime_standard_tools::filesystem::FileSystemManager;

#[tokio::main]
async fn main() -> PackageResult<()> {
    println!("🔧 Version Resolver Example");
    println!("==========================\n");

    // Example 1: Basic VersionResolver creation
    example_basic_creation().await?;

    // Example 2: Version resolution on different branches
    example_branch_based_resolution().await?;

    // Example 3: Configuration-driven behavior
    example_configuration_options().await?;

    // Example 4: Package search in single repo and monorepo
    example_package_search().await?;

    println!("\n✅ All examples completed successfully!");

    Ok(())
}

/// Example 1: Basic VersionResolver Creation
///
/// Shows how to create a VersionResolver with filesystem and Git integration.
async fn example_basic_creation() -> PackageResult<()> {
    println!("📋 Example 1: Basic VersionResolver Creation");
    println!("---------------------------------------------");

    // Create filesystem manager for file operations
    let filesystem = FileSystemManager::new();

    // Open Git repository (in a real scenario, this would be the project root)
    // For this example, we assume we're in a Git repository
    let current_dir = std::env::current_dir().map_err(|e| {
        sublime_pkg_tools::error::PackageError::operation(
            "get_current_dir",
            format!("Failed to get current directory: {}", e),
        )
    })?;

    // Try to open Git repository (this might fail if not in a Git repo)
    match Repo::open(&current_dir.to_string_lossy()) {
        Ok(repo) => {
            // Create default configuration
            let config = PackageToolsConfig::default();

            // Create VersionResolver (now includes MonorepoDetector integration)
            let resolver = VersionResolver::new(filesystem, repo, config);

            println!("✓ VersionResolver created successfully");
            println!("  - Filesystem: FileSystemManager");
            println!("  - Git repository: {}", current_dir.display());
            println!("  - Configuration: Default settings");
            println!("  - MonorepoDetector: Integrated for workspace analysis");

            // Get current branch information
            match resolver.get_current_branch().await {
                Ok(branch) => {
                    println!("  - Current branch: {}", branch);

                    // Check if snapshots should be used
                    let use_snapshots = resolver.should_use_snapshot().await?;
                    println!("  - Use snapshots: {}", use_snapshots);
                }
                Err(e) => {
                    println!("  - Warning: Could not determine current branch: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️  Not in a Git repository: {}", e);
            println!("   This is expected when running outside a Git repository.");
        }
    }

    println!();
    Ok(())
}

/// Example 2: Version Resolution on Different Branches
///
/// Demonstrates how VersionResolver behaves differently based on Git branch.
async fn example_branch_based_resolution() -> PackageResult<()> {
    println!("🌳 Example 2: Branch-based Version Resolution");
    println!("---------------------------------------------");

    // This example shows the conceptual behavior
    // In a real scenario, you would be on different branches

    let filesystem = FileSystemManager::new();
    let current_dir = std::env::current_dir().unwrap();

    match Repo::open(&current_dir.to_string_lossy()) {
        Ok(repo) => {
            let config = PackageToolsConfig::default();
            let resolver = VersionResolver::new(filesystem, repo, config);

            // Get current branch
            match resolver.get_current_branch().await {
                Ok(branch) => {
                    println!("Current branch: {}", branch);

                    let use_snapshots = resolver.should_use_snapshot().await?;

                    match branch.as_str() {
                        "main" | "master" => {
                            println!("✓ On main branch - using release versions");
                            println!("  - Snapshots enabled: {}", use_snapshots);
                            println!("  - Version source: package.json");
                        }
                        _ => {
                            println!("✓ On development branch - using snapshot versions");
                            println!("  - Snapshots enabled: {}", use_snapshots);
                            println!("  - Version format: {{base}}-{{commit}}.snapshot");

                            // Get current commit hash
                            if let Ok(commit_hash) = resolver.get_current_commit_hash().await {
                                let short_hash: String = commit_hash.chars().take(7).collect();
                                println!(
                                    "  - Current commit: {} (short: {})",
                                    commit_hash, short_hash
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Could not determine branch: {}", e);
                }
            }
        }
        Err(_) => {
            println!("⚠️  Not in a Git repository - demonstrating conceptual behavior:");
            println!();
            println!("On main/master branch:");
            println!("  - Version resolution: package.json → 1.2.3");
            println!("  - Result: ResolvedVersion::Release(1.2.3)");
            println!();
            println!("On feature branch (feat/auth):");
            println!("  - Version resolution: package.json + commit → 1.2.3-abc123d.snapshot");
            println!("  - Result: ResolvedVersion::Snapshot(1.2.3-abc123d.snapshot)");
        }
    }

    println!();
    Ok(())
}

/// Example 3: Configuration-driven Behavior
///
/// Shows how different configuration options affect version resolution.
async fn example_configuration_options() -> PackageResult<()> {
    println!("⚙️  Example 3: Configuration Options");
    println!("----------------------------------");

    let _filesystem = FileSystemManager::new();
    let _current_dir = std::env::current_dir().unwrap();

    // Example configurations
    let configs = vec![
        (
            "Default Configuration",
            PackageToolsConfig {
                version: VersionConfig {
                    commit_hash_length: 7,
                    allow_snapshot_on_main: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "Long Commit Hashes",
            PackageToolsConfig {
                version: VersionConfig {
                    commit_hash_length: 12,
                    allow_snapshot_on_main: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "Snapshots on Main Allowed",
            PackageToolsConfig {
                version: VersionConfig {
                    commit_hash_length: 7,
                    allow_snapshot_on_main: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
    ];

    for (name, config) in configs {
        println!("{}: ", name);
        println!("  - Commit hash length: {}", config.version.commit_hash_length);
        println!("  - Allow snapshots on main: {}", config.version.allow_snapshot_on_main);
        println!("  - Snapshot format: {}", config.version.snapshot_format);

        // Demonstrate commit hash shortening
        let full_hash = "abcd1234567890123456";
        let short_hash: String =
            full_hash.chars().take(config.version.commit_hash_length as usize).collect();
        println!("  - Example hash: {} → {}", full_hash, short_hash);

        println!();
    }

    Ok(())
}

/// Example 4: Package Search in Single Repo and Monorepo
///
/// Demonstrates finding packages by name in both single repository and monorepo structures.
async fn example_package_search() -> PackageResult<()> {
    println!("📦 Example 4: Package Search - Single Repo & Monorepo Support");
    println!("=============================================================");

    // Single Repository Example
    println!("🏠 Single Repository Structure:");
    println!("my-single-package/");
    println!("├── package.json (name: \"@myorg/single-service\")");
    println!("├── src/");
    println!("└── tests/");
    println!();

    println!("Single repo search behavior:");
    println!("- Algorithm:");
    println!("  1. Check if workspace root is a monorepo (no)");
    println!("  2. Read package.json from repository root");
    println!("  3. Match name field with requested package name");
    println!("  4. Return workspace root path if match found");
    println!();

    let single_repo_examples = vec![
        ("@myorg/single-service", "✅ Found at repository root"),
        ("@myorg/other-service", "❌ Name mismatch (expected vs actual name shown)"),
    ];

    println!("Single repo search examples:");
    for (package_name, result) in single_repo_examples {
        println!("  {} → {}", package_name, result);
    }

    println!();
    println!("{}", "=".repeat(60));

    // Monorepo Example
    println!("🏗️  Monorepo Structure:");
    println!("my-monorepo/");
    println!("├── package.json (workspaces config)");
    println!("├── packages/");
    println!("│   ├── auth-service/");
    println!("│   │   └── package.json (name: \"@myorg/auth-service\")");
    println!("│   ├── user-service/");
    println!("│   │   └── package.json (name: \"@myorg/user-service\")");
    println!("│   └── shared-utils/");
    println!("│       └── package.json (name: \"@myorg/shared-utils\")");
    println!("└── apps/");
    println!("    └── web-app/");
    println!("        └── package.json (name: \"@myorg/web-app\")");
    println!();

    println!("Monorepo search behavior:");
    println!("- Algorithm:");
    println!("  1. Detect monorepo type (npm, yarn, pnpm workspaces, lerna, etc.)");
    println!("  2. Parse workspace configuration files");
    println!("  3. Analyze actual workspace patterns from config");
    println!("  4. Find packages using MonorepoDescriptor.get_package()");
    println!("  5. Return absolute path from WorkspacePackage");
    println!();

    let monorepo_examples = vec![
        ("@myorg/auth-service", "✅ Found via workspace config"),
        ("@myorg/user-service", "✅ Found via workspace config"),
        ("@myorg/web-app", "✅ Found via workspace config"),
        ("@myorg/nonexistent", "❌ Not found (with list of available packages)"),
    ];

    println!("Monorepo search examples:");
    for (package_name, expected_result) in monorepo_examples {
        println!("  {} → {}", package_name, expected_result);
    }

    println!();
    println!("🎯 Unified Benefits:");
    println!("  ✅ Automatic repository type detection");
    println!("  ✅ Single API for both single repo and monorepo");
    println!("  ✅ Respects actual workspace configuration");
    println!("  ✅ Supports all major monorepo tools");
    println!("  ✅ Handles complex workspace patterns");
    println!("  ✅ Provides detailed error messages with context");
    println!("  ✅ No hardcoded assumptions about directory structure");

    println!();
    Ok(())
}

/// Utility function to demonstrate version resolution behavior
///
/// This function shows what the version resolution process would look like
/// with actual package.json files and Git repository state.
#[allow(dead_code)]
async fn demonstrate_version_resolution(
    package_path: &Path,
    resolver: &VersionResolver<FileSystemManager>,
) -> PackageResult<()> {
    println!("Resolving version for: {}", package_path.display());

    match resolver.resolve_current_version(package_path).await {
        Ok(ResolvedVersion::Release(version)) => {
            println!("✓ Release version: {}", version);
            println!("  - Source: package.json");
            println!("  - Type: Stable release");
        }
        Ok(ResolvedVersion::Snapshot(snapshot)) => {
            println!("✓ Snapshot version: {}", snapshot);
            println!("  - Base version: {}", snapshot.base_version());
            println!("  - Commit: {}", snapshot.commit_id());
            println!("  - Created: {}", snapshot.created_at());
            println!("  - Type: Development snapshot");
        }
        Err(e) => {
            println!("❌ Resolution failed: {}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_examples_run_without_panic() {
        // Test that examples can run without panicking
        // (they may have warnings about not being in a Git repo)

        assert!(example_basic_creation().await.is_ok());
        assert!(example_branch_based_resolution().await.is_ok());
        assert!(example_configuration_options().await.is_ok());
        assert!(example_package_search().await.is_ok());
    }

    #[test]
    fn test_commit_hash_shortening_examples() {
        let full_hash = "abcd1234567890123456";

        // Test different configuration lengths
        let configs = [
            (7, "abcd123"),
            (12, "abcd12345678"),
            (5, "abcd1"),
            (20, "abcd1234567890123456"), // Full hash when requested length exceeds actual
        ];

        for (length, expected) in configs {
            let result: String = full_hash.chars().take(length).collect();
            assert_eq!(result, expected);
        }
    }
}
