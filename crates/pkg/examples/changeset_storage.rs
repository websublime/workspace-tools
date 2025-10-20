//! Example: Using FileBasedChangesetStorage
//!
//! This example demonstrates how to use the FileBasedChangesetStorage to:
//! - Create and save changesets
//! - Load and update changesets
//! - List pending changesets
//! - Archive changesets with release information
//! - Query archived changesets
//!
//! Run this example with:
//! ```bash
//! cargo run --example changeset_storage
//! ```

use std::collections::HashMap;
use sublime_pkg_tools::changeset::{ChangesetStorage, FileBasedChangesetStorage};
use sublime_pkg_tools::types::{Changeset, ReleaseInfo, VersionBump};
use sublime_standard_tools::filesystem::FileSystemManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for this example
    let temp_dir = tempfile::tempdir()?;
    println!("Using temporary directory: {:?}\n", temp_dir.path());

    // Initialize the file-based storage
    let fs = FileSystemManager::new();
    let storage = FileBasedChangesetStorage::new(
        temp_dir.path().to_path_buf(),
        ".changesets".to_string(),
        ".changesets/history".to_string(),
        fs,
    );

    println!("=== Creating Changesets ===\n");

    // Create a changeset for a feature branch
    let mut feature_changeset = Changeset::new(
        "feature/oauth-integration",
        VersionBump::Minor,
        vec!["staging".to_string(), "production".to_string()],
    );
    feature_changeset.add_package("@myorg/auth");
    feature_changeset.add_package("@myorg/core");
    feature_changeset.add_commit("abc123def456");
    feature_changeset.add_commit("789ghi012jkl");

    println!("Created changeset for branch: {}", feature_changeset.branch);
    println!("  Version bump: {:?}", feature_changeset.bump);
    println!("  Packages: {:?}", feature_changeset.packages);
    println!("  Commits: {} commits", feature_changeset.changes.len());

    // Save the changeset
    storage.save(&feature_changeset).await?;
    println!("  ✓ Saved to storage\n");

    // Create another changeset for a bugfix
    let mut bugfix_changeset =
        Changeset::new("fix/memory-leak", VersionBump::Patch, vec!["production".to_string()]);
    bugfix_changeset.add_package("@myorg/utils");
    bugfix_changeset.add_commit("mno345pqr678");

    println!("Created changeset for branch: {}", bugfix_changeset.branch);
    println!("  Version bump: {:?}", bugfix_changeset.bump);
    println!("  Packages: {:?}", bugfix_changeset.packages);

    storage.save(&bugfix_changeset).await?;
    println!("  ✓ Saved to storage\n");

    // Check if a changeset exists
    println!("=== Checking Existence ===\n");
    let exists = storage.exists("feature/oauth-integration").await?;
    println!("Changeset 'feature/oauth-integration' exists: {}\n", exists);

    // Load and update a changeset
    println!("=== Loading and Updating ===\n");
    let mut loaded = storage.load("feature/oauth-integration").await?;
    println!("Loaded changeset: {}", loaded.branch);
    println!("  Before update - packages: {:?}", loaded.packages);

    loaded.add_package("@myorg/api");
    storage.save(&loaded).await?;
    println!("  After update - packages: {:?}", loaded.packages);
    println!("  ✓ Updated changeset\n");

    // List all pending changesets
    println!("=== Listing Pending Changesets ===\n");
    let pending = storage.list_pending().await?;
    println!("Found {} pending changeset(s):", pending.len());
    for changeset in &pending {
        println!(
            "  - {} ({}): {} package(s), {} commit(s)",
            changeset.branch,
            match changeset.bump {
                VersionBump::Major => "MAJOR",
                VersionBump::Minor => "MINOR",
                VersionBump::Patch => "PATCH",
                VersionBump::None => "NONE",
            },
            changeset.packages.len(),
            changeset.changes.len()
        );
    }
    println!();

    // Archive a changeset (simulate a release)
    println!("=== Archiving Changeset ===\n");
    let release_changeset = storage.load("fix/memory-leak").await?;

    let mut versions = HashMap::new();
    versions.insert("@myorg/utils".to_string(), "1.2.4".to_string());

    let release_info = ReleaseInfo::new(
        "ci-bot@example.com".to_string(),
        "release-commit-xyz789".to_string(),
        versions,
    );

    println!("Archiving changeset: {}", release_changeset.branch);
    println!("  Applied by: {}", release_info.applied_by);
    println!("  Git commit: {}", release_info.git_commit);
    println!("  Versions released:");
    for (pkg, version) in &release_info.versions {
        println!("    {} -> {}", pkg, version);
    }

    storage.archive(&release_changeset, release_info).await?;
    println!("  ✓ Archived successfully\n");

    // Verify the changeset is no longer pending
    let still_pending = storage.exists("fix/memory-leak").await?;
    println!("Changeset 'fix/memory-leak' still pending: {}\n", still_pending);

    // Load archived changeset
    println!("=== Loading Archived Changeset ===\n");
    let archived = storage.load_archived("fix/memory-leak").await?;
    println!("Archived changeset: {}", archived.changeset.branch);
    println!("  Released at: {}", archived.release_info.applied_at);
    println!("  Released by: {}", archived.release_info.applied_by);
    println!("  Git commit: {}", archived.release_info.git_commit);
    println!("  Versions in release:");
    for (pkg, version) in &archived.release_info.versions {
        println!("    {} -> {}", pkg, version);
    }
    println!();

    // List all archived changesets
    println!("=== Listing Archived Changesets ===\n");
    let archived_list = storage.list_archived().await?;
    println!("Found {} archived changeset(s):", archived_list.len());
    for archived in &archived_list {
        println!(
            "  - {} (released {})",
            archived.changeset.branch,
            archived.release_info.applied_at.format("%Y-%m-%d %H:%M:%S")
        );
        println!("    Packages released: {}", archived.release_info.versions.len());
    }
    println!();

    // Final summary
    println!("=== Summary ===\n");
    let final_pending = storage.list_pending().await?;
    let final_archived = storage.list_archived().await?;
    println!("Pending changesets: {}", final_pending.len());
    println!("Archived changesets: {}", final_archived.len());
    println!("\nExample completed successfully!");

    Ok(())
}
