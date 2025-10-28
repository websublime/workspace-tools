//! Example: Snapshot Version Generation
//!
//! This example demonstrates how to use the `SnapshotGenerator` to create
//! snapshot versions for pre-release testing and branch deployments.
//!
//! Snapshot versions enable deploying branch builds to testing environments
//! before merging to main, with unique, identifiable version strings.
//!
//! Run with: cargo run --example snapshot_version

use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_pkg_tools::types::Version;
use sublime_pkg_tools::version::{SnapshotContext, SnapshotGenerator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Snapshot Version Generation Example ===\n");

    // Load configuration (or use defaults)
    let config = PackageToolsConfig::default();
    println!("📋 Configuration:");
    println!("   Snapshot format: {}\n", config.version.snapshot_format);

    // Create a snapshot generator using the configured format
    let generator = SnapshotGenerator::new(&config.version.snapshot_format)?;
    println!("✅ Created snapshot generator with format: {}\n", generator.format());

    // Example 1: Generate snapshot for a feature branch
    println!("📦 Example 1: Feature Branch Snapshot");
    let base_version = Version::parse("1.2.3")?;
    let context = SnapshotContext::new(
        base_version,
        "feat/oauth-integration".to_string(),
        "a1b2c3d4e5f6".to_string(),
    );

    let snapshot = generator.generate(&context)?;
    println!("   Base version: {}", context.version);
    println!("   Branch: {}", context.branch);
    println!("   Commit: {}", context.commit);
    println!("   ➡️  Snapshot: {}\n", snapshot);

    // Example 2: Generate snapshot for a hotfix branch
    println!("📦 Example 2: Hotfix Branch Snapshot");
    let hotfix_version = Version::parse("2.0.1")?;
    let hotfix_context = SnapshotContext::new(
        hotfix_version,
        "hotfix/critical-bug".to_string(),
        "def456abc123".to_string(),
    );

    let hotfix_snapshot = generator.generate(&hotfix_context)?;
    println!("   Base version: {}", hotfix_context.version);
    println!("   Branch: {}", hotfix_context.branch);
    println!("   Commit: {}", hotfix_context.commit);
    println!("   ➡️  Snapshot: {}\n", hotfix_snapshot);

    // Example 3: Different snapshot formats
    println!("📦 Example 3: Different Snapshot Formats");
    let formats = vec![
        "{version}-{branch}.{commit}",
        "{version}-snapshot.{timestamp}",
        "{version}.{commit}",
        "{version}-{branch}-{timestamp}",
    ];

    let test_context = SnapshotContext::with_timestamp(
        Version::parse("3.0.0")?,
        "develop".to_string(),
        "xyz789abc".to_string(),
        1640000000,
    );

    for format in formats {
        let generator = SnapshotGenerator::new(format)?;
        let snap = generator.generate(&test_context)?;
        println!("   Format: {}", format);
        println!("   Result: {}\n", snap);
    }

    // Example 4: Branch name sanitization
    println!("📦 Example 4: Branch Name Sanitization");
    let complex_branches = vec![
        "feature/PROJ-123-add-auth",
        "fix/bug_fix_v2",
        "release/2.0.0-beta",
        "feat/user@domain.com",
    ];

    let sanitization_generator = SnapshotGenerator::new("{version}-{branch}")?;
    for branch in complex_branches {
        let ctx = SnapshotContext::new(
            Version::parse("1.0.0")?,
            branch.to_string(),
            "abc123".to_string(),
        );
        let result = sanitization_generator.generate(&ctx)?;
        println!("   Original: {}", branch);
        println!("   Sanitized: {}\n", result);
    }

    // Example 5: CI/CD workflow simulation
    println!("📦 Example 5: CI/CD Workflow Simulation");
    println!("   Simulating a CI/CD pipeline deploying a branch build...\n");

    // Simulate getting git info (in real scenario, use sublime_git_tools)
    let package_version = Version::parse("1.5.2")?;
    let current_branch = "feat/new-api".to_string();
    let commit_hash = "f7e8d9c0b1a2".to_string();

    let ci_context = SnapshotContext::new(package_version, current_branch, commit_hash);

    let snapshot_version = generator.generate(&ci_context)?;

    println!("   🔧 Build Info:");
    println!("      Package version: {}", ci_context.version);
    println!("      Branch: {}", ci_context.branch);
    println!("      Commit: {}", ci_context.commit);
    println!("\n   📤 Publishing:");
    println!("      Snapshot version: {}", snapshot_version);
    println!("      NPM tag: {}", ci_context.branch.replace('/', "-"));
    println!("\n   ✅ Users can install with:");
    println!("      npm install @myorg/package@{}", snapshot_version);

    println!("\n=== Example Complete ===");
    Ok(())
}
