//! # Monorepo Detection Example
//!
//! This example demonstrates how to use the monorepo detection functionality
//! to identify and analyze different types of monorepo structures.
//!
//! It shows how to:
//! - Detect different types of monorepos (Lerna, Yarn Workspaces, pnpm, etc.)
//! - Find the monorepo root from any subdirectory
//! - Get information about packages within a monorepo
//! - Analyze package dependencies and relationships
//! - Find packages by path or name

#![allow(clippy::print_stdout)]
#![allow(clippy::uninlined_format_args)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

use std::{env, path::Path};
use sublime_standard_tools::{
    error::StandardResult,
    project::{FileSystem, FileSystemManager, MonorepoDetector, MonorepoInfo, PathUtils},
};

/// Creates a temporary monorepo for demonstration purposes
fn setup_test_monorepo() -> StandardResult<tempfile::TempDir> {
    // Create a temporary directory
    let temp_dir = tempfile::tempdir().map_err(|e| {
        sublime_standard_tools::error::StandardError::operation(format!(
            "Failed to create temporary directory: {}",
            e
        ))
    })?;

    let fs = FileSystemManager::new();
    let root_path = temp_dir.path();

    // Create a simple Yarn Workspaces monorepo structure
    println!("Creating a test Yarn Workspaces monorepo at: {}", root_path.display());

    // Root package.json with workspaces config
    fs.write_file_string(
        &root_path.join("package.json"),
        r#"{
            "name": "example-monorepo",
            "private": true,
            "workspaces": ["packages/*"]
        }"#,
    )?;

    // Create yarn.lock file
    fs.write_file_string(&root_path.join("yarn.lock"), "")?;

    // Create packages directory
    fs.create_dir_all(&root_path.join("packages"))?;

    // Create some packages
    let packages = [
        (
            "core",
            r#"{
            "name": "core",
            "version": "1.0.0"
        }"#,
        ),
        (
            "utils",
            r#"{
            "name": "utils",
            "version": "1.0.0",
            "dependencies": {
                "core": "1.0.0"
            }
        }"#,
        ),
        (
            "app",
            r#"{
            "name": "app",
            "version": "1.0.0",
            "dependencies": {
                "core": "1.0.0",
                "utils": "1.0.0"
            }
        }"#,
        ),
    ];

    for (name, content) in packages {
        let pkg_dir = root_path.join("packages").join(name);
        fs.create_dir_all(&pkg_dir)?;
        fs.write_file_string(&pkg_dir.join("package.json"), content)?;
    }

    println!("Test monorepo structure created successfully");

    Ok(temp_dir)
}

/// Detects and analyzes a monorepo structure
fn detect_monorepo(path: &Path) -> StandardResult<MonorepoInfo> {
    let detector = MonorepoDetector::new();

    println!("\n=== Monorepo Detection ===");
    println!("Analyzing directory: {}", path.display());

    // Check if this directory is a monorepo root
    if let Some(kind) = detector.is_monorepo_root(path)? {
        println!("📂 Directory is a {} monorepo root", kind.name());
    } else {
        println!("📂 Directory is not a monorepo root");
    }

    // Find the closest monorepo root (works from subdirectories too)
    match detector.find_monorepo_root(path)? {
        Some((root, kind)) => {
            println!("📂 Found {} monorepo at {}", kind.name(), root.display());
        }
        None => {
            println!("❌ No monorepo found in or above this directory");
        }
    }

    // Perform full monorepo detection and analysis
    let monorepo = detector.detect_monorepo(path)?;

    println!("\n=== Monorepo Analysis Results ===");
    println!("Type: {} monorepo", monorepo.kind().name());
    println!("Root: {}", monorepo.root().display());
    println!("Found {} packages:", monorepo.packages().len());

    for package in monorepo.packages() {
        println!("  - {}: {}", package.name, package.location.display());
    }

    Ok(monorepo)
}

/// Analyzes package relationships within a monorepo
fn analyze_packages(monorepo: &MonorepoInfo) {
    println!("\n=== Package Relationships ===");

    // Find packages with dependencies on other packages
    for package in monorepo.packages() {
        if !package.workspace_dependencies.is_empty() {
            println!("📦 {} depends on:", package.name);
            for dep in &package.workspace_dependencies {
                println!("  - {}", dep);
            }
        }
    }

    // Find dependents (packages that depend on a specific package)
    if let Some(package) = monorepo.packages().first() {
        let dependents = monorepo.find_dependents(&package.name);

        println!("\n📦 Packages depending on {}:", package.name);
        if dependents.is_empty() {
            println!("  None");
        } else {
            for dep in dependents {
                println!("  - {}", dep.name);
            }
        }
    }

    // Find a package containing a specific path
    if let Some(package) = monorepo.packages().first() {
        let path = &package.absolute_path;
        println!("\n📂 Package containing path {}:", path.display());
        if let Some(pkg) = monorepo.find_package_for_path(path) {
            println!("  - {}", pkg.name);
        } else {
            println!("  None");
        }
    }

    // Get a specific package by name
    if let Some(package) = monorepo.packages().first() {
        println!("\n📦 Looking up package by name: {}", package.name);
        if let Some(pkg) = monorepo.get_package(&package.name) {
            println!("  Found at: {}", pkg.absolute_path.display());
        } else {
            println!("  Not found");
        }
    }
}

/// Demonstrates using the monorepo detection functionality with a real project
fn real_project_example() {
    println!("\n=== Real Project Detection ===");
    println!("Checking if current directory is in a monorepo...");

    // Get current directory
    let current_dir = match PathUtils::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            println!("❌ Error getting current directory: {}", e);
            return;
        }
    };

    println!("Current directory: {}", current_dir.display());

    match MonorepoDetector::new().find_monorepo_root(&current_dir) {
        Ok(Some((root, kind))) => {
            println!("✅ Found {} monorepo at {}", kind.name(), root.display());

            // Perform full detection
            match detect_monorepo(&root) {
                Ok(monorepo) => {
                    analyze_packages(&monorepo);
                }
                Err(e) => {
                    println!("❌ Error analyzing monorepo: {}", e);
                }
            }
        }
        Ok(None) => {
            println!("ℹ️ Current directory is not part of a monorepo");
        }
        Err(e) => {
            println!("❌ Error during monorepo detection: {}", e);
        }
    }
}

#[tokio::main]
async fn main() -> StandardResult<()> {
    println!("=== Monorepo Detection Example ===");

    let args: Vec<String> = env::args().collect();
    let use_real_project = args.len() > 1 && args[1] == "--real";

    if use_real_project {
        real_project_example();
    } else {
        // Create and test with our sample monorepo
        let temp_dir = setup_test_monorepo()?;

        // Detect and analyze the monorepo
        let monorepo = detect_monorepo(temp_dir.path())?;

        // Analyze package relationships
        analyze_packages(&monorepo);

        println!("\nTip: Run with --real to check the current directory");
    }

    Ok(())
}
