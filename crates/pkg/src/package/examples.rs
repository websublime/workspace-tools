//! # PackageManager Examples and Best Practices
//!
//! Comprehensive examples demonstrating all PackageManager functionality with enterprise-grade
//! patterns, error handling, and performance considerations.
//!
//! ## What
//!
//! This module provides real-world examples of PackageManager usage covering:
//! - Basic read/write/validate operations
//! - Error handling patterns
//! - Performance optimization techniques
//! - Integration with async workflows
//! - Testing strategies
//!
//! ## How
//!
//! Each example is self-contained and demonstrates best practices for specific use cases.
//! Examples are organized by complexity and include detailed explanations of the rationale
//! behind each approach.
//!
//! ## Why
//!
//! Comprehensive examples ensure developers can quickly adopt PackageManager in their
//! projects with confidence, following enterprise-grade patterns from the start.

#![allow(dead_code)] // Examples module may have unused functions
#![allow(clippy::unwrap_used)] // Examples may use unwrap for simplicity
#![allow(clippy::expect_used)] // Examples may use expect for clarity

use crate::{
    package::{manager::PackageManager, package::Package},
    Dependency, Result,
};
use std::{path::Path, sync::Arc};
use sublime_standard_tools::filesystem::FileSystemManager;

/// # Basic PackageManager Usage
/// 
/// This example demonstrates the fundamental operations of PackageManager:
/// reading, writing, and validating package.json files.
/// 
/// ## Key Points:
/// - Always use proper error handling with Result types
/// - FileSystemManager provides the async filesystem implementation
/// - Validation reports provide detailed feedback about package structure
pub async fn basic_usage_example() -> Result<()> {
    // Create a PackageManager with the standard filesystem
    let filesystem = FileSystemManager::new();
    let manager = PackageManager::new(filesystem);

    // Read an existing package.json file
    let package_path = Path::new("package.json");
    
    match manager.read_package(package_path).await {
        Ok(package) => {
            println!("Successfully read package: {} v{}", package.name, package.version);
            println!("Dependencies count: {}", package.dependencies.len());
            
            // Validate the package
            let validation_report = manager.validate_package(&package).await?;
            
            if validation_report.has_errors() {
                println!("⚠️  Package has validation errors:");
                for error in validation_report.errors() {
                    println!("  - {}", error);
                }
            }
            
            if validation_report.has_warnings() {
                println!("ℹ️  Package has validation warnings:");
                for warning in validation_report.warnings() {
                    println!("  - {}", warning);
                }
            }
            
            if !validation_report.has_errors() && !validation_report.has_warnings() {
                println!("✅ Package validation passed without issues");
            }
        }
        Err(e) => {
            println!("Failed to read package: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// # Creating and Writing Packages
/// 
/// This example shows how to create new packages programmatically and write them to disk
/// with proper error handling and atomic operations.
/// 
/// ## Key Points:
/// - Package::new validates the basic structure during creation
/// - write_package creates backups automatically
/// - Operations are atomic (temp file + rename)
pub async fn create_and_write_example() -> Result<()> {
    let filesystem = FileSystemManager::new();
    let manager = PackageManager::new(filesystem);

    // Create dependencies
    let dependencies = vec![
        Dependency::new("react", "^18.0.0")?,
        Dependency::new("@types/node", "^20.0.0")?,
        Dependency::new("lodash", "~4.17.21")?,
    ];

    // Create a new package
    let package = Package::new(
        "my-awesome-project", 
        "1.0.0", 
        Some(dependencies)
    )?;

    // Validate before writing (good practice)
    let validation_report = manager.validate_package(&package).await?;
    
    if validation_report.has_errors() {
        println!("Cannot write package due to validation errors:");
        for error in validation_report.errors() {
            println!("  - {}", error);
        }
        return Ok(()); // Don't write invalid packages
    }

    // Write to different locations
    let output_paths = vec![
        Path::new("./new-project/package.json"),
        Path::new("./backup/package.json"),
    ];

    for path in output_paths {
        match manager.write_package(path, &package).await {
            Ok(()) => {
                println!("✅ Successfully wrote package to: {}", path.display());
                
                // Verify the write by reading back
                let read_back = manager.read_package(path).await?;
                assert_eq!(read_back.name, package.name);
                assert_eq!(read_back.version, package.version);
                println!("✅ Verified written package matches original");
            }
            Err(e) => {
                println!("❌ Failed to write package to {}: {}", path.display(), e);
            }
        }
    }

    Ok(())
}

/// # Error Handling Patterns
/// 
/// This example demonstrates robust error handling patterns for all PackageManager operations.
/// 
/// ## Key Points:
/// - Different error types require different handling strategies
/// - Always provide meaningful error messages to users
/// - Consider retry strategies for transient failures
pub async fn error_handling_example() -> Result<()> {
    let filesystem = FileSystemManager::new();
    let manager = PackageManager::new(filesystem);

    let problematic_paths = vec![
        Path::new("nonexistent/package.json"),
        Path::new("/root/package.json"), // Permission issues
        Path::new("corrupted-package.json"),
    ];

    for path in problematic_paths {
        println!("Attempting to read: {}", path.display());
        
        match manager.read_package(path).await {
            Ok(package) => {
                println!("✅ Successfully read: {}", package.name);
            }
            Err(e) => {
                // Handle different error types appropriately
                match &e {
                    crate::Error::Package(package_error) => {
                        match package_error {
                            crate::errors::PackageError::PackageNotFound(name) => {
                                println!("📦 Package not found: {}", name);
                                // Could trigger package initialization workflow
                            }
                            crate::errors::PackageError::PackageJsonIoFailure { path, error } => {
                                println!("💾 I/O error reading {}: {}", path, error);
                                // Could retry with exponential backoff
                            }
                            crate::errors::PackageError::PackageJsonParseFailure { path, error } => {
                                println!("📝 Parse error in {}: {}", path, error);
                                // Could offer to fix common parsing issues
                            }
                            _ => {
                                println!("❌ Other package error: {}", e);
                            }
                        }
                    }
                    _ => {
                        println!("❌ Unexpected error: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

/// # Performance Optimization Example
/// 
/// This example shows how to optimize PackageManager usage for high-performance scenarios
/// like processing many packages in monorepos.
/// 
/// ## Key Points:
/// - Concurrent processing with proper resource management
/// - Batch operations for better throughput
/// - Memory-efficient processing of large package sets
pub async fn performance_optimization_example() -> Result<()> {
    let filesystem = FileSystemManager::new();
    let manager = Arc::new(PackageManager::new(filesystem));

    // Simulate processing many packages (like in a monorepo)
    let package_paths: Vec<&Path> = vec![
        Path::new("packages/frontend/package.json"),
        Path::new("packages/backend/package.json"),
        Path::new("packages/shared/package.json"),
        Path::new("packages/mobile/package.json"),
        // ... many more packages
    ];

    // Process packages concurrently with controlled concurrency
    let max_concurrent = 10;
    let mut tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    for chunk in package_paths.chunks(max_concurrent) {
        let mut chunk_tasks = Vec::new();
        
        for &path in chunk {
            let manager_clone = Arc::clone(&manager);
            let path_owned = path.to_path_buf();
            
            let task = tokio::spawn(async move {
                let start = std::time::Instant::now();
                
                let result = manager_clone.read_package(&path_owned).await;
                
                match result {
                    Ok(package) => {
                        // Quick validation for performance-critical scenarios
                        let validation = manager_clone.validate_package(&package).await?;
                        
                        let duration = start.elapsed();
                        println!("✅ Processed {} in {:?}", package.name, duration);
                        
                        Ok((package, validation))
                    }
                    Err(e) => {
                        println!("❌ Failed to process {}: {}", path_owned.display(), e);
                        Err(e)
                    }
                }
            });
            
            chunk_tasks.push(task);
        }
        
        // Wait for chunk to complete before processing next chunk
        for task in chunk_tasks {
            match task.await {
                Ok(Ok((package, validation))) => {
                    if validation.has_errors() {
                        println!("⚠️  Package {} has validation issues", package.name);
                    }
                }
                Ok(Err(e)) => {
                    println!("Processing error: {}", e);
                }
                Err(e) => {
                    println!("Task error: {}", e);
                }
            }
        }
    }

    println!("🏁 Completed batch processing of {} packages", package_paths.len());
    Ok(())
}

/// # Integration with Custom Workflows
/// 
/// This example demonstrates how to integrate PackageManager into custom workflows
/// like CI/CD pipelines, package update automation, and validation gates.
/// 
/// ## Key Points:
/// - Structured data extraction for automation
/// - Custom validation rules for organizational policies
/// - Integration with external systems and tools
pub async fn workflow_integration_example() -> Result<()> {
    let filesystem = FileSystemManager::new();
    let manager = PackageManager::new(filesystem);

    // Example: CI/CD Pipeline Integration
    println!("🔄 Starting CI/CD Package Validation Pipeline");

    let package_path = Path::new("package.json");
    let package = manager.read_package(package_path).await?;

    // Extract package information for CI/CD systems
    let package_info = extract_package_info(&package).await;
    println!("📊 Package Info: {:#?}", package_info);

    // Perform comprehensive validation
    let validation_report = manager.validate_package(&package).await?;
    
    // Custom organizational validation rules
    let custom_validation = apply_custom_validation_rules(&package, &validation_report).await;
    
    // Generate CI/CD friendly output
    let ci_report = generate_ci_report(&package, &validation_report, &custom_validation);
    println!("📋 CI Report:\n{}", ci_report);

    // Example: Automated Package Updates
    if custom_validation.update_recommended {
        println!("🔄 Automated updates recommended");
        let updated_package = apply_recommended_updates(package).await?;
        
        // Write updated package
        manager.write_package(package_path, &updated_package).await?;
        println!("✅ Package updated successfully");
    }

    Ok(())
}

/// # Testing Strategies with PackageManager
/// 
/// This example shows how to effectively test code that uses PackageManager.
/// 
/// ## Key Points:
/// - Mock filesystem for isolated testing
/// - Test data setup strategies
/// - Assertion patterns for validation results
pub async fn testing_strategies_example() -> Result<()> {
    // Example of how to structure tests with PackageManager
    println!("🧪 Testing Strategies Example");

    // 1. Using real filesystem for integration tests
    integration_test_example().await?;
    
    // 2. Using mock filesystem for unit tests
    unit_test_example().await?;

    Ok(())
}

// Helper functions for examples

#[derive(Debug)]
struct PackageInfo {
    name: String,
    version: String,
    dependency_count: usize,
    has_dev_dependencies: bool,
    is_scoped: bool,
}

async fn extract_package_info(package: &Package) -> PackageInfo {
    PackageInfo {
        name: package.name.clone(),
        version: package.version.clone(),
        dependency_count: package.dependencies.len(),
        has_dev_dependencies: false, // Package struct doesn't track dev deps separately
        is_scoped: package.name.starts_with('@'),
    }
}

struct CustomValidationResult {
    update_recommended: bool,
    security_issues: Vec<String>,
    policy_violations: Vec<String>,
}

async fn apply_custom_validation_rules(
    package: &Package,
    _validation_report: &crate::package::manager::PackageValidationReport,
) -> CustomValidationResult {
    let mut result = CustomValidationResult {
        update_recommended: false,
        security_issues: Vec::new(),
        policy_violations: Vec::new(),
    };

    // Example custom rules
    if package.dependencies.len() > 100 {
        result.policy_violations.push("Too many dependencies".to_string());
    }

    if package.name.contains("test") || package.name.contains("demo") {
        result.policy_violations.push("Invalid package name for production".to_string());
    }

    // Check for outdated version patterns
    if package.version.starts_with("0.") {
        result.update_recommended = true;
    }

    result
}

fn generate_ci_report(
    package: &Package,
    validation_report: &crate::package::manager::PackageValidationReport,
    custom_validation: &CustomValidationResult,
) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("Package: {} v{}\n", package.name, package.version));
    report.push_str(&format!("Dependencies: {}\n", package.dependencies.len()));
    
    if validation_report.has_errors() {
        report.push_str("❌ VALIDATION FAILED\n");
        for error in validation_report.errors() {
            report.push_str(&format!("  - {}\n", error));
        }
    } else {
        report.push_str("✅ Validation passed\n");
    }
    
    if !custom_validation.policy_violations.is_empty() {
        report.push_str("⚠️  Policy violations:\n");
        for violation in &custom_validation.policy_violations {
            report.push_str(&format!("  - {}\n", violation));
        }
    }
    
    report
}

async fn apply_recommended_updates(mut package: Package) -> Result<Package> {
    // Example: Update version if it's pre-1.0
    if package.version.starts_with("0.") {
        package.version = "1.0.0".to_string();
    }
    
    // Example: Remove deprecated dependencies
    package.dependencies.retain(|dep| !dep.name.contains("deprecated"));
    
    Ok(package)
}

async fn integration_test_example() -> Result<()> {
    println!("Integration test example");
    // This would use real filesystem operations
    let filesystem = FileSystemManager::new();
    let manager = PackageManager::new(filesystem);
    
    // Create a temporary test package
    let test_package = Package::new("integration-test", "1.0.0", None)?;
    
    // Use a temporary file for testing
    let test_path = std::env::temp_dir().join("test-integration-package.json");
    
    // Test write and read cycle
    manager.write_package(&test_path, &test_package).await?;
    let read_package = manager.read_package(&test_path).await?;
    
    assert_eq!(test_package.name, read_package.name);
    assert_eq!(test_package.version, read_package.version);
    
    // Cleanup
    let _ = std::fs::remove_file(&test_path);
    
    println!("✅ Integration test passed");
    Ok(())
}

async fn unit_test_example() -> Result<()> {
    println!("Unit test example");
    
    // This would use a mock filesystem for controlled testing
    // For brevity, using FileSystemManager but in real tests you'd use a mock
    let filesystem = FileSystemManager::new();
    let manager = PackageManager::new(filesystem);
    
    // Test validation logic
    let valid_package = Package::new("valid-package", "1.0.0", None)?;
    let validation_report = manager.validate_package(&valid_package).await?;
    
    assert!(!validation_report.has_errors());
    
    println!("✅ Unit test passed");
    Ok(())
}

/// # Quick Start Guide
/// 
/// For developers who want to get started quickly with PackageManager.
pub async fn quick_start_guide() -> Result<()> {
    println!("🚀 PackageManager Quick Start Guide");
    println!("===================================");
    
    // Step 1: Create a PackageManager
    println!("\n1️⃣  Create a PackageManager:");
    println!("```rust");
    println!("use sublime_package_tools::package::manager::PackageManager;");
    println!("use sublime_standard_tools::filesystem::FileSystemManager;");
    println!();
    println!("let filesystem = FileSystemManager::new();");
    println!("let manager = PackageManager::new(filesystem);");
    println!("```");
    
    // Step 2: Read a package
    println!("\n2️⃣  Read a package.json file:");
    println!("```rust");
    println!("let package = manager.read_package(Path::new(\"package.json\")).await?;");
    println!("println!(\"Package: {{}} v{{}}\", package.name, package.version);");
    println!("```");
    
    // Step 3: Validate a package
    println!("\n3️⃣  Validate a package:");
    println!("```rust");
    println!("let report = manager.validate_package(&package).await?;");
    println!("if report.has_errors() {{");
    println!("    for error in report.errors() {{");
    println!("        println!(\"Error: {{}}\", error);");
    println!("    }}");
    println!("}}");
    println!("```");
    
    // Step 4: Write a package
    println!("\n4️⃣  Write a package to disk:");
    println!("```rust");
    println!("manager.write_package(Path::new(\"output/package.json\"), &package).await?;");
    println!("```");
    
    println!("\n✅ You're ready to use PackageManager in your project!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example_functions_compile() {
        // These tests ensure that all example functions compile correctly
        // In a real scenario, you'd run these with proper test data
        
        // Test that the functions are callable (but don't run them in tests)
        let _basic_usage = basic_usage_example;
        let _create_and_write = create_and_write_example;
        let _error_handling = error_handling_example;
        let _performance = performance_optimization_example;
        let _workflow = workflow_integration_example;
        let _testing = testing_strategies_example;
        let _quick_start = quick_start_guide;
        
        // If we reach here, all examples compile correctly
        assert!(true);
    }
}