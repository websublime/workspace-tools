//! Example of standardized logging implementation for condition checker
//!
//! This file demonstrates how to refactor existing logging to use the new
//! standardized logging utilities.

use crate::logging::{log_operation, log_operation_error, log_file_operation, ErrorContext};
use crate::error::{Error, Result};

/// Example: Refactored execute_custom_script method with standardized logging
async fn execute_custom_script_example(
    &self,
    script: &str,
    expected_output: Option<&str>,
) -> Result<bool> {
    // BEFORE:
    // log::debug!("Custom script executed with exit code: {}", result.status());
    
    // AFTER:
    log_operation("custom_script", format!("Executing script: {}", script), None);
    
    let executor = self.file_system_provider.command_executor();
    let command = self.file_system_provider.create_command(script);
    
    match executor.execute(command).await {
        Ok(result) => {
            // BEFORE:
            // log::debug!("Custom script executed with exit code: {}", result.status());
            
            // AFTER:
            log::debug!(
                "[custom_script] Executed with exit code: {} (success: {})", 
                result.status(), 
                result.success()
            );
            
            if let Some(expected) = expected_output {
                let stdout = result.stdout().trim();
                let matches = stdout == expected.trim();
                
                // BEFORE:
                // log::debug!("Expected: '{}', Got: '{}', Matches: {}", expected.trim(), stdout, matches);
                
                // AFTER:
                log::debug!(
                    "[custom_script] Output validation - Expected: '{}', Got: '{}', Matches: {}", 
                    expected.trim(), 
                    stdout, 
                    matches
                );
                
                Ok(matches)
            } else {
                let success = result.success();
                
                // BEFORE:
                // log::debug!("Script exit code check: {} (success: {})", result.status(), success);
                
                // AFTER:
                if success {
                    log_operation("custom_script", "Completed successfully", Some(script));
                } else {
                    log_operation("custom_script", 
                        format!("Failed with exit code: {}", result.status()), 
                        Some(script)
                    );
                }
                
                Ok(success)
            }
        }
        Err(e) => {
            // BEFORE:
            // log::warn!("Failed to execute custom script '{}': {}", script, e);
            
            // AFTER:
            ErrorContext::new("custom_script")
                .with_detail("script", script)
                .log_error(&e);
            
            Ok(false)
        }
    }
}

/// Example: File operation logging
fn check_file_exists_example(&self, file_path: &str) -> bool {
    let path = std::path::Path::new(file_path);
    let exists = path.exists();
    
    // AFTER: Using standardized file operation logging
    log_file_operation("check_exists", file_path, exists);
    
    exists
}

/// Example: Package-specific logging
fn analyze_package_changes_example(&self, package_name: &str) -> Result<()> {
    use crate::logging::log_package_operation;
    
    // BEFORE:
    // log::info!("Analyzing changes for package: {}", package_name);
    
    // AFTER:
    log_package_operation("analyze_changes", package_name, "Starting analysis");
    
    // ... do analysis ...
    
    log_package_operation("analyze_changes", package_name, "Analysis complete");
    
    Ok(())
}

/// Example: Performance logging
async fn execute_with_timing_example(&self) -> Result<()> {
    use crate::logging::log_performance;
    use std::time::Instant;
    
    let start = Instant::now();
    
    // ... do work ...
    
    let duration = start.elapsed();
    log_performance("task_execution", duration.as_millis() as u64, Some(10));
    
    Ok(())
}

/// Example: Using the time_operation macro
fn process_packages_example(&self, packages: &[String]) -> Result<()> {
    crate::time_operation!("process_packages", {
        for package in packages {
            // Process each package
            log::debug!("[package] Processing: {}", package);
        }
        Ok(())
    })
}