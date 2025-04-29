//! # Project Management Example
//!
//! This example demonstrates how to use the project management functionality
//! to detect and validate Node.js projects, analyze package.json contents,
//! and perform filesystem operations within a project.

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

use std::path::{Path, PathBuf};
use sublime_standard_tools::{
    error::StandardResult,
    project::{
        FileSystem, FileSystemManager, NodePathKind, PackageManagerKind, PathExt, PathUtils,
        ProjectConfig, ProjectManager, ValidationStatus,
    },
};

/// Detects and analyzes a project at the given path
fn analyze_project(path: &Path, project_manager: &ProjectManager) -> StandardResult<()> {
    println!("=== Project Analysis ===");
    println!("Analyzing project at: {}", path.display());

    // Create a project configuration
    let config = ProjectConfig::new().detect_package_manager(true).validate_structure(true);

    // Detect the project
    let project = project_manager.detect_project(path, &config)?;

    // Print basic project information
    println!("\nProject Information:");
    println!("  Root: {}", project.root().display());

    // Print package.json details
    if let Some(pkg) = project.package_json() {
        println!("\nPackage Details:");
        println!("  Name: {}", pkg.name);
        println!("  Version: {}", pkg.version);
        println!("  Dependencies: {}", pkg.dependencies.len());
        println!("  DevDependencies: {}", pkg.dev_dependencies.len());

        println!("\nScripts:");
        for (name, script) in &pkg.scripts {
            println!("  {}: {}", name, script);
        }
    } else {
        println!("\nNo package.json found or it could not be parsed.");
    }

    // Print package manager information
    if let Some(pm) = project.package_manager() {
        println!("\nPackage Manager:");
        println!("  Type: {}", pm.kind().command());
        println!("  Lock file: {}", pm.lock_file_path().display());
    } else {
        println!("\nNo package manager detected.");
    }

    // Print validation status
    println!("\nValidation Status:");
    match project.validation_status() {
        ValidationStatus::Valid => {
            println!("  Project structure is valid.");
        }
        ValidationStatus::Warning(warnings) => {
            println!("  Project has warnings:");
            for (i, warning) in warnings.iter().enumerate() {
                println!("    {}. {}", i + 1, warning);
            }
        }
        ValidationStatus::Error(errors) => {
            println!("  Project has errors:");
            for (i, error) in errors.iter().enumerate() {
                println!("    {}. {}", i + 1, error);
            }
        }
        ValidationStatus::NotValidated => {
            println!("  Project has not been validated.");
        }
    }

    Ok(())
}

/// Lists important directories and files in the project
fn list_project_files(path: &Path, fs: &FileSystemManager) -> StandardResult<()> {
    println!("\n=== Project Files ===");

    // Check if we're in a project
    if !path.is_in_project() {
        println!("Not in a Node.js project (no package.json found in path or any parent)");
        return Ok(());
    }

    // Get project root
    let project_root = PathUtils::find_project_root(path).ok_or_else(|| {
        sublime_standard_tools::error::StandardError::operation(
            "Could not find project root".to_string(),
        )
    })?;

    println!("Project root: {}", project_root.display());

    // List source files
    let src_dir = project_root.node_path(NodePathKind::Src);
    if fs.exists(&src_dir) {
        println!("\nSource files:");
        if let Ok(entries) = fs.read_dir(&src_dir) {
            for (i, entry) in entries.iter().enumerate().take(10) {
                println!(
                    "  {}. {}",
                    i + 1,
                    entry.file_name().unwrap_or_default().to_string_lossy()
                );
            }

            if entries.len() > 10 {
                println!("  ... and {} more", entries.len() - 10);
            }
        } else {
            println!("  (Could not read source directory)");
        }
    } else {
        println!("\nNo 'src' directory found.");
    }

    // List node_modules status
    let node_modules = project_root.node_path(NodePathKind::NodeModules);
    println!("\nNode modules:");
    if fs.exists(&node_modules) {
        println!("  node_modules directory exists");

        // Count top-level packages
        if let Ok(entries) = fs.read_dir(&node_modules) {
            println!("  Contains {} top-level packages", entries.len());
        }
    } else {
        println!("  node_modules directory does not exist");
    }

    // List package.json location
    let package_json = project_root.node_path(NodePathKind::PackageJson);
    println!("\nConfiguration files:");
    println!("  package.json: {}", if fs.exists(&package_json) { "Found" } else { "Not found" });

    // Check for various lock files
    println!(
        "  package-lock.json: {}",
        if fs.exists(&project_root.join(PackageManagerKind::Npm.lock_file())) {
            "Found (npm)"
        } else {
            "Not found"
        }
    );

    println!(
        "  yarn.lock: {}",
        if fs.exists(&project_root.join(PackageManagerKind::Yarn.lock_file())) {
            "Found (yarn)"
        } else {
            "Not found"
        }
    );

    println!(
        "  pnpm-lock.yaml: {}",
        if fs.exists(&project_root.join(PackageManagerKind::Pnpm.lock_file())) {
            "Found (pnpm)"
        } else {
            "Not found"
        }
    );

    println!(
        "  bun.lockb: {}",
        if fs.exists(&project_root.join(PackageManagerKind::Bun.lock_file())) {
            "Found (bun)"
        } else {
            "Not found"
        }
    );

    Ok(())
}

/// Creates a temporary project structure for testing
fn create_test_project(fs: &FileSystemManager) -> StandardResult<PathBuf> {
    println!("\n=== Creating Test Project ===");

    // Use a temporary directory for our test project
    let temp_dir = tempfile::tempdir().map_err(|e| {
        sublime_standard_tools::error::StandardError::operation(format!(
            "Failed to create temporary directory: {}",
            e
        ))
    })?;

    let project_path = temp_dir.path().to_path_buf();
    println!("Creating project at: {}", project_path.display());

    // Create basic package.json
    let package_json = r#"{
        "name": "test-project",
        "version": "1.0.0",
        "description": "A test project for the example",
        "main": "index.js",
        "scripts": {
            "start": "node index.js",
            "test": "echo \"Error: no test specified\" && exit 1"
        },
        "dependencies": {
            "express": "^4.17.1"
        },
        "devDependencies": {
            "jest": "^27.0.0"
        }
    }"#;

    fs.write_file_string(&project_path.join("package.json"), package_json)?;

    // Create source directory and a file
    fs.create_dir_all(&project_path.join("src"))?;
    fs.write_file_string(
        &project_path.join("src/index.js"),
        "console.log('Hello from test project');",
    )?;

    // Create npm lock file
    fs.write_file_string(
        &project_path.join("package-lock.json"),
        "{\"name\":\"test-project\",\"version\":\"1.0.0\",\"lockfileVersion\":2}",
    )?;

    println!("Test project created successfully!");

    // Return the path, but note that the temp_dir will be deleted
    // when it goes out of scope, which means the caller should use
    // this path immediately.
    Ok(project_path)
}

fn main() -> StandardResult<()> {
    let fs_manager = FileSystemManager::new();
    let project_manager = ProjectManager::new();

    // Get current directory path
    let current_dir = PathUtils::current_dir()?;
    println!("Current directory: {}", current_dir.display());

    // Try to find project root
    if let Some(project_root) = PathUtils::find_project_root(&current_dir) {
        println!("Found project root: {}", project_root.display());

        // Analyze existing project
        analyze_project(&project_root, &project_manager)?;

        // List project files
        list_project_files(&project_root, &fs_manager)?;
    } else {
        println!("No project found in the current directory.");

        // Create a test project
        let test_project_path = create_test_project(&fs_manager)?;

        // Analyze the test project
        analyze_project(&test_project_path, &project_manager)?;

        // List project files
        list_project_files(&test_project_path, &fs_manager)?;
    }

    Ok(())
}
