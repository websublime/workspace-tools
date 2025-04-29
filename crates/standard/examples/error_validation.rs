//! # Error Handling and Validation Example
//!
//! This example demonstrates how to use the error handling and validation
//! systems to build robust applications with proper error reporting and
//! data validation.

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

use std::{
    error::Error,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};
use sublime_standard_tools::{
    error::{FileSystemError, StandardError, StandardResult},
    validation::{ValidationContext, ValidationResult, ValidationRule, Validator},
};

/// Demonstrates error handling for filesystem operations
fn file_operations_example(path: &Path) -> StandardResult<String> {
    println!("=== Error Handling Example ===");
    println!("Attempting to read file: {}", path.display());

    // Try to open the file
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            let fs_error = match err.kind() {
                io::ErrorKind::NotFound => FileSystemError::NotFound { path: path.to_path_buf() },
                io::ErrorKind::PermissionDenied => {
                    FileSystemError::PermissionDenied { path: path.to_path_buf() }
                }
                _ => FileSystemError::Io { path: path.to_path_buf(), source: err },
            };

            return Err(StandardError::FileSystem(fs_error));
        }
    };

    // Read file contents
    let mut contents = String::new();
    if let Err(err) = file.read_to_string(&mut contents) {
        return Err(StandardError::Io(err));
    }

    println!("File read successfully, content length: {} bytes", contents.len());

    Ok(contents)
}

/// Demonstrates error propagation and conversion
fn process_file(path: &Path) -> StandardResult<Vec<String>> {
    println!("\nProcessing file: {}", path.display());

    // Try to read the file, allowing errors to propagate
    let content = file_operations_example(path)?;

    // Process the content
    let lines: Vec<String> = content.lines().map(std::string::ToString::to_string).collect();

    println!("File processed successfully, found {} lines", lines.len());

    Ok(lines)
}

/// Defines a validation rule for paths
struct PathValidationRule {
    /// Maximum allowed path length
    max_length: usize,
    /// List of allowed extensions
    allowed_extensions: Vec<String>,
}

impl ValidationRule<PathBuf> for PathValidationRule {
    fn validate(&self, target: &PathBuf) -> ValidationResult {
        let mut context = ValidationContext::new();

        // Check path length
        let path_str = target.to_string_lossy();
        if path_str.len() > self.max_length {
            context.add_error(format!(
                "Path length exceeds maximum of {} characters",
                self.max_length
            ));
        }

        // Check file extension
        if let Some(ext) = target.extension() {
            let ext_str = ext.to_string_lossy().to_string();
            if !self.allowed_extensions.contains(&ext_str) {
                context.add_error(format!(
                    "File extension '{}' is not allowed. Allowed: {:?}",
                    ext_str, self.allowed_extensions
                ));
            }
        } else if !target.is_dir() {
            context.add_warning("Path has no file extension");
        }

        // Check if path exists
        if !target.exists() {
            context.add_warning(format!("Path does not exist: {}", path_str));
        }

        // Check for absolute path
        if target.is_absolute() {
            context.add_warning("Path is absolute, consider using relative paths");
        }

        context.result()
    }
}

/// Demonstrates using the validation system
fn validation_example() {
    println!("\n=== Validation Example ===");

    // Create a validator for package.json files
    println!("Creating path validator");
    let mut path_validator = Validator::<PathBuf>::new();

    // Add a validation rule
    path_validator.add_rule(PathValidationRule {
        max_length: 100,
        allowed_extensions: vec![
            "json".to_string(),
            "js".to_string(),
            "ts".to_string(),
            "md".to_string(),
        ],
    });

    // Test some paths
    let test_paths = [PathBuf::from("package.json"),
        PathBuf::from("src/index.js"),
        PathBuf::from("README.md"),
        PathBuf::from("very/long/path/that/might/exceed/the/maximum/length/requirements/for/our/validation/rule/and/will/cause/an/error.php"),
        PathBuf::from("invalid.exe"),
        PathBuf::from("/absolute/path/file.js")];

    println!("Validating paths:");

    for (i, path) in test_paths.iter().enumerate() {
        println!("\n{}. Validating: {}", i + 1, path.display());

        match path_validator.validate(path) {
            ValidationResult::Valid => {
                println!("   ✅ Path is valid");
            }
            ValidationResult::Warning(warnings) => {
                println!("   ⚠️ Path has warnings:");
                for (j, warning) in warnings.iter().enumerate() {
                    println!("     {}. {}", j + 1, warning);
                }
            }
            ValidationResult::Error(errors) => {
                println!("   ❌ Path is invalid:");
                for (j, error) in errors.iter().enumerate() {
                    println!("     {}. {}", j + 1, error);
                }
            }
        }
    }
}

/// Demonstrates complex validation with context
#[allow(clippy::too_many_lines)]
#[allow(clippy::single_char_pattern)]
#[allow(clippy::ignored_unit_patterns)]
fn package_json_validation() -> StandardResult<()> {
    println!("\n=== Package.json Validation Example ===");

    // Define some test package.json content
    let valid_package = r#"{
        "name": "valid-package",
        "version": "1.0.0",
        "description": "A valid package.json example",
        "main": "index.js",
        "scripts": {
            "test": "jest",
            "start": "node index.js"
        },
        "engines": {
            "node": ">=14"
        },
        "dependencies": {
            "express": "^4.17.1"
        }
    }"#;

    let invalid_package = r#"{
        "name": "in valid name",
        "version": "1.0",
        "main": "index.js",
        "scripts": {}
    }"#;

    // Create a temporary file
    let temp_dir = tempfile::tempdir()
        .map_err(|e| StandardError::operation(format!("Failed to create temp dir: {}", e)))?;

    let valid_path = temp_dir.path().join("valid-package.json");
    let invalid_path = temp_dir.path().join("invalid-package.json");

    // Write test content to files
    std::fs::write(&valid_path, valid_package)
        .map_err(|e| StandardError::operation(format!("Failed to write test file: {}", e)))?;

    std::fs::write(&invalid_path, invalid_package)
        .map_err(|e| StandardError::operation(format!("Failed to write test file: {}", e)))?;

    // Function to validate package.json
    let validate_package = |path: &Path| -> StandardResult<()> {
        println!("Validating package.json at: {}", path.display());

        // Read the file
        let content =
            std::fs::read_to_string(path).map_err(|e| FileSystemError::from_io(e, path))?;

        // Parse the JSON
        let json: serde_json::Value = serde_json::from_str(&content)?;

        // Create validation context
        let mut context = ValidationContext::new();
        context.add_data("file_path", path.to_string_lossy().to_string());

        // Validate the package.json

        // 1. Validate name (required, valid npm package name)
        if let Some(name) = json.get("name") {
            if let Some(name_str) = name.as_str() {
                if name_str.contains(" ") || name_str.contains("/") {
                    context.add_error(format!(
                        "Invalid package name: '{}' (contains spaces or slashes)",
                        name_str
                    ));
                }
            } else {
                context.add_error("Package name must be a string");
            }
        } else {
            context.add_error("Missing required field: name");
        }

        // 2. Validate version (semver format)
        if let Some(version) = json.get("version") {
            if let Some(version_str) = version.as_str() {
                // Simple semver check (x.y.z)
                if !version_str.split('.').all(|part| part.parse::<u32>().is_ok()) {
                    context.add_error(format!(
                        "Invalid version format: '{}' (should be semver)",
                        version_str
                    ));
                }
            } else {
                context.add_error("Version must be a string");
            }
        } else {
            context.add_error("Missing required field: version");
        }

        // 3. Check for description
        if json.get("description").is_none() {
            context.add_warning("Missing recommended field: description");
        }

        // 4. Check scripts
        if let Some(scripts) = json.get("scripts") {
            if let Some(scripts_obj) = scripts.as_object() {
                if scripts_obj.is_empty() {
                    context.add_warning("Scripts object is empty");
                }

                // Check for test script
                if !scripts_obj.contains_key("test") {
                    context.add_warning("No test script defined");
                }
            } else {
                context.add_error("Scripts must be an object");
            }
        }

        // Get the validation result
        let result = context.result();

        match &result {
            ValidationResult::Valid => {
                println!("✅ Package.json is valid");
            }
            ValidationResult::Warning(warnings) => {
                println!("⚠️ Package.json has warnings:");
                for (i, warning) in warnings.iter().enumerate() {
                    println!("  {}. {}", i + 1, warning);
                }
            }
            ValidationResult::Error(errors) => {
                println!("❌ Package.json is invalid:");
                for (i, error) in errors.iter().enumerate() {
                    println!("  {}. {}", i + 1, error);
                }
            }
        }

        // Return error if validation failed with errors
        if result.has_errors() {
            if let ValidationResult::Error(errors) = result {
                return Err(StandardError::operation(format!(
                    "Package.json validation failed: {}",
                    errors.join(", ")
                )));
            }
        }

        Ok(())
    };

    // Validate both package.json files
    println!("\nValidating valid package.json:");
    if let Err(e) = validate_package(&valid_path) {
        println!("Unexpected error: {}", e);

        // Print error chain for debugging
        let mut source = e.source();
        let mut i = 1;
        while let Some(err) = source {
            println!("Cause {}: {}", i, err);
            source = err.source();
            i += 1;
        }
    }

    println!("\nValidating invalid package.json:");
    match validate_package(&invalid_path) {
        Ok(_) => println!("Validation should have failed!"),
        Err(e) => println!("Expected error: {}", e),
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Create a path that may or may not exist
    let example_path = Path::new("Cargo.toml");

    // Run the examples
    match file_operations_example(example_path) {
        Ok(content) => {
            println!("File content preview: {} characters", content.len());
        }
        Err(e) => {
            println!("Error reading file: {}", e);

            // Demonstrate error source chain
            let mut source = e.source();
            let mut depth = 1;
            while let Some(err) = source {
                println!("Cause {}: {}", depth, err);
                source = err.source();
                depth += 1;
            }
        }
    }

    // Try processing a file (using error propagation)
    match process_file(example_path) {
        Ok(lines) => {
            println!("Processed {} lines from the file", lines.len());
        }
        Err(e) => {
            println!("Error processing file: {}", e);
        }
    }

    // Run validation examples
    validation_example();
    package_json_validation()?;

    Ok(())
}
