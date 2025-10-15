# API Comparison - v1.x vs v2.0

**Document Version:** 1.0  
**Date:** 2025-01-15  
**Status:** Reference

## Overview

This document provides a side-by-side comparison of the `sublime_pkg_tools` API changes between v1.x and v2.0, making it easy to understand exactly what changed and why.

---

## Table of Contents

- [Quick Reference](#quick-reference)
- [Function-by-Function Comparison](#function-by-function-comparison)
- [Import Statements](#import-statements)
- [Complete Code Examples](#complete-code-examples)
- [Type Changes](#type-changes)
- [Error Handling](#error-handling)
- [Best Practices](#best-practices)

---

## Quick Reference

### Functions Removed in v2.0

| v1.x Function | v2.0 Replacement | Reason |
|---------------|------------------|--------|
| `read_package_json()` | `PackageJson::read_from_path()` | Thin wrapper, use type method directly |
| `create_package_from_directory()` | `Package::from_path()` | Thin wrapper, use type method directly |
| `is_package_directory()` | `AsyncFileSystem::exists()` | Use standard filesystem operations |
| `find_package_directories()` | `MonorepoDetector::detect_packages()` | Use standard monorepo detection |
| `find_packages_recursive()` | (internal, removed) | Replaced by standard monorepo detection |

### Functions Unchanged in v2.0

| Function | Status | Notes |
|----------|--------|-------|
| `validate_package_json()` | ✅ Unchanged | Package.json-specific validation remains in pkg crate |

### New Recommended Patterns in v2.0

| Use Case | v2.0 Recommendation | From Crate |
|----------|---------------------|------------|
| Detect project type | `ProjectDetector::detect()` | `sublime_standard_tools` |
| Find packages in monorepo | `MonorepoDetector::detect_packages()` | `sublime_standard_tools` |
| Check monorepo status | `MonorepoDetector::is_monorepo_root()` | `sublime_standard_tools` |
| Read files | `AsyncFileSystem::read_file_string()` | `sublime_standard_tools` |

---

## Function-by-Function Comparison

### 1. Reading package.json

#### v1.x

```rust
use sublime_pkg_tools::package::read_package_json;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn read_pkg() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let path = Path::new("./package.json");
    
    // Free function call
    let pkg = read_package_json(&fs, path).await?;
    println!("{}", pkg.name);
    
    Ok(())
}
```

#### v2.0

```rust
use sublime_pkg_tools::package::PackageJson;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn read_pkg() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let path = Path::new("./package.json");
    
    // Type method call
    let pkg = PackageJson::read_from_path(&fs, path).await?;
    println!("{}", pkg.name);
    
    Ok(())
}
```

**Key Differences:**
- Import changes from function to type
- Call pattern: `function(&fs, path)` → `Type::method(&fs, path)`
- Functionality identical
- Better IDE support (type-based autocomplete)

---

### 2. Creating Package from Directory

#### v1.x

```rust
use sublime_pkg_tools::package::{create_package_from_directory, Package};
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn create_pkg() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let dir = Path::new("./packages/auth");
    
    // Free function call
    let package = create_package_from_directory(&fs, dir).await?;
    println!("{}", package.name());
    
    Ok(())
}
```

#### v2.0

```rust
use sublime_pkg_tools::package::Package;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn create_pkg() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let dir = Path::new("./packages/auth");
    
    // Constructor pattern
    let package = Package::from_path(&fs, dir).await?;
    println!("{}", package.name());
    
    Ok(())
}
```

**Key Differences:**
- Standard Rust constructor pattern (`Type::from_*()`)
- Clearer intent - constructing a `Package`
- Removes redundant import

---

### 3. Checking if Directory Contains package.json

#### v1.x

```rust
use sublime_pkg_tools::package::is_package_directory;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn check_dir() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let dir = Path::new("./packages/auth");
    
    // Wrapper function
    if is_package_directory(&fs, dir).await {
        println!("Is a package!");
    }
    
    Ok(())
}
```

#### v2.0 - Option A (Simple Check)

```rust
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use std::path::Path;

async fn check_dir() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let dir = Path::new("./packages/auth");
    
    // Direct filesystem operation
    let package_json = dir.join("package.json");
    if fs.exists(&package_json).await {
        println!("Is a package!");
    }
    
    Ok(())
}
```

#### v2.0 - Option B (Robust Check)

```rust
use sublime_pkg_tools::package::PackageJson;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn check_dir() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let dir = Path::new("./packages/auth");
    
    // Try to parse - validates it's actually valid JSON
    let package_json = dir.join("package.json");
    match PackageJson::read_from_path(&fs, &package_json).await {
        Ok(_) => println!("Is a valid package!"),
        Err(_) => println!("Not a valid package"),
    }
    
    Ok(())
}
```

**Key Differences:**
- v2.0 offers two approaches: simple existence check or validation
- More explicit about what's being checked
- Can reuse filesystem trait for other checks
- Option B provides stronger guarantees (valid JSON, not just file exists)

---

### 4. Finding Packages in Workspace

#### v1.x

```rust
use sublime_pkg_tools::package::find_package_directories;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::{Path, PathBuf};

async fn find_pkgs() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let root = Path::new("./");
    
    // Returns Vec<PathBuf>
    let dirs = find_package_directories(&fs, root, Some(3)).await?;
    
    for dir in dirs {
        println!("Found: {}", dir.display());
    }
    
    Ok(())
}
```

#### v2.0 - Monorepo Detection

```rust
use sublime_standard_tools::filesystem::FileSystemManager;
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
use std::path::Path;

async fn find_pkgs() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let root = Path::new("./");
    
    let detector = MonorepoDetector::with_filesystem(fs);
    
    // Returns Vec<WorkspacePackage> with rich metadata
    let packages = detector.detect_packages(root).await?;
    
    for pkg in packages {
        println!("Found: {} at {} (v{})", 
            pkg.name, 
            pkg.absolute_path.display(),
            pkg.version
        );
        
        // Access dependencies info
        println!("  Dependencies: {:?}", pkg.dependencies.keys());
    }
    
    Ok(())
}
```

#### v2.0 - Unified Detection (Single or Monorepo)

```rust
use sublime_standard_tools::filesystem::FileSystemManager;
use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorTrait};
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
use sublime_pkg_tools::package::Package;
use std::path::Path;

async fn find_pkgs() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let root = Path::new("./");
    
    // Detect project type first
    let project_detector = ProjectDetector::with_filesystem(fs.clone());
    let project = project_detector.detect(root, None).await?;
    
    if project.as_project_info().kind().is_monorepo() {
        // Handle monorepo
        let monorepo_detector = MonorepoDetector::with_filesystem(fs);
        let packages = monorepo_detector.detect_packages(root).await?;
        
        println!("Monorepo with {} packages:", packages.len());
        for pkg in packages {
            println!("  - {} at {}", pkg.name, pkg.absolute_path.display());
        }
    } else {
        // Handle single package
        let package = Package::from_path(&fs, root).await?;
        println!("Single package: {}", package.name());
    }
    
    Ok(())
}
```

**Key Differences:**
- v1.x returned only paths (`Vec<PathBuf>`)
- v2.0 returns rich `WorkspacePackage` objects with metadata
- v2.0 provides unified detection for single/monorepo scenarios
- More robust detection (uses workspace patterns, config files)
- Better separation: use standard tools for structure detection

---

### 5. Validating package.json (UNCHANGED)

#### v1.x and v2.0 (Identical)

```rust
use sublime_pkg_tools::package::validate_package_json;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn validate() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let path = Path::new("./package.json");
    
    // Same API in both versions
    let result = validate_package_json(&fs, path).await?;
    
    if result.has_errors() {
        eprintln!("Validation errors:");
        for error in result.errors() {
            eprintln!("  - {}", error.message);
        }
    }
    
    Ok(())
}
```

**Key Differences:**
- None - this function remains unchanged

---

## Import Statements

### Complete Import Comparison

#### v1.x Imports

```rust
// Package operations
use sublime_pkg_tools::package::{
    read_package_json,                  // ❌ Removed in v2.0
    create_package_from_directory,      // ❌ Removed in v2.0
    is_package_directory,               // ❌ Removed in v2.0
    find_package_directories,           // ❌ Removed in v2.0
    validate_package_json,              // ✅ Still available
    Package,
    PackageJson,
    PackageJsonEditor,
};

// Standard tools
use sublime_standard_tools::filesystem::FileSystemManager;
```

#### v2.0 Imports

```rust
// Package operations - types and validation only
use sublime_pkg_tools::package::{
    validate_package_json,              // ✅ Still available
    Package,                             // Use Package::from_path()
    PackageJson,                         // Use PackageJson::read_from_path()
    PackageJsonEditor,
};

// Standard tools - more imports needed
use sublime_standard_tools::filesystem::{
    AsyncFileSystem,                     // For exists() checks
    FileSystemManager,
};
use sublime_standard_tools::monorepo::{
    MonorepoDetector,                    // For finding packages
    MonorepoDetectorTrait,
};
use sublime_standard_tools::project::{
    ProjectDetector,                     // For unified detection
    ProjectDetectorTrait,
};
```

---

## Complete Code Examples

### Example 1: Process All Packages in Workspace

#### v1.x

```rust
use sublime_pkg_tools::package::{
    find_package_directories,
    read_package_json,
    Package,
};
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn process_all() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let root = Path::new("./");
    
    // Find all package directories
    let dirs = find_package_directories(&fs, root, Some(3)).await?;
    
    println!("Found {} packages", dirs.len());
    
    // Process each package
    for dir in dirs {
        let pkg_json_path = dir.join("package.json");
        let pkg_json = read_package_json(&fs, &pkg_json_path).await?;
        
        println!("Processing: {} v{}", pkg_json.name, pkg_json.version);
        
        // Do something with the package
        process_package(&pkg_json)?;
    }
    
    Ok(())
}

fn process_package(pkg: &sublime_pkg_tools::package::PackageJson) -> Result<(), Box<dyn std::error::Error>> {
    // Package processing logic
    Ok(())
}
```

#### v2.0

```rust
use sublime_pkg_tools::package::{Package, PackageJson};
use sublime_standard_tools::filesystem::FileSystemManager;
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorTrait};
use std::path::Path;

async fn process_all() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let root = Path::new("./");
    
    // Detect project type
    let project_detector = ProjectDetector::with_filesystem(fs.clone());
    let project = project_detector.detect(root, None).await?;
    
    if project.as_project_info().kind().is_monorepo() {
        // Monorepo - process all packages
        let monorepo_detector = MonorepoDetector::with_filesystem(fs.clone());
        let packages = monorepo_detector.detect_packages(root).await?;
        
        println!("Found {} packages", packages.len());
        
        for pkg in packages {
            println!("Processing: {} v{}", pkg.name, pkg.version);
            
            // WorkspacePackage has dependency info already
            println!("  Workspace deps: {:?}", pkg.workspace_dependencies);
            
            // Load full package if needed
            let package = Package::from_path(&fs, &pkg.absolute_path).await?;
            process_package(&package.package_json())?;
        }
    } else {
        // Single package
        println!("Found single package");
        let package = Package::from_path(&fs, root).await?;
        process_package(&package.package_json())?;
    }
    
    Ok(())
}

fn process_package(pkg: &sublime_pkg_tools::package::PackageJson) -> Result<(), Box<dyn std::error::Error>> {
    // Package processing logic
    Ok(())
}
```

**Benefits of v2.0:**
- Handles both single and monorepo projects correctly
- Access to richer metadata from `WorkspacePackage`
- More explicit about project structure
- Leverages standard tools for detection

---

### Example 2: Update Package Versions

#### v1.x

```rust
use sublime_pkg_tools::package::{
    find_package_directories,
    PackageJsonEditor,
};
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn update_versions(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let root = Path::new("./");
    
    let dirs = find_package_directories(&fs, root, None).await?;
    
    for dir in dirs {
        let pkg_json_path = dir.join("package.json");
        
        let mut editor = PackageJsonEditor::new(fs.clone(), &pkg_json_path).await?;
        editor.set_version(version)?;
        editor.save().await?;
        
        println!("Updated {} to version {}", dir.display(), version);
    }
    
    Ok(())
}
```

#### v2.0

```rust
use sublime_pkg_tools::package::PackageJsonEditor;
use sublime_standard_tools::filesystem::FileSystemManager;
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
use std::path::Path;

async fn update_versions(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let root = Path::new("./");
    
    let detector = MonorepoDetector::with_filesystem(fs.clone());
    let packages = detector.detect_packages(root).await?;
    
    for pkg in packages {
        let pkg_json_path = pkg.absolute_path.join("package.json");
        
        let mut editor = PackageJsonEditor::new(fs.clone(), &pkg_json_path).await?;
        editor.set_version(version)?;
        editor.save().await?;
        
        println!("Updated {} to version {}", pkg.name, version);
    }
    
    Ok(())
}
```

**Benefits of v2.0:**
- Already know package names (from `WorkspacePackage`)
- Better error messages (can reference package by name)
- Same editing API (unchanged)

---

### Example 3: Conditional Package Processing

#### v1.x

```rust
use sublime_pkg_tools::package::{
    is_package_directory,
    read_package_json,
};
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn process_if_package(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    
    if is_package_directory(&fs, dir).await {
        let pkg_json = read_package_json(&fs, &dir.join("package.json")).await?;
        println!("Found package: {}", pkg_json.name);
        
        // Process the package
    } else {
        println!("Not a package directory");
    }
    
    Ok(())
}
```

#### v2.0

```rust
use sublime_pkg_tools::package::PackageJson;
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use std::path::Path;

async fn process_if_package(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let pkg_json_path = dir.join("package.json");
    
    // Option 1: Simple check then read
    if fs.exists(&pkg_json_path).await {
        let pkg_json = PackageJson::read_from_path(&fs, &pkg_json_path).await?;
        println!("Found package: {}", pkg_json.name);
        
        // Process the package
    } else {
        println!("Not a package directory");
    }
    
    Ok(())
}

// Option 2: Try to read (more robust)
async fn process_if_package_v2(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let pkg_json_path = dir.join("package.json");
    
    match PackageJson::read_from_path(&fs, &pkg_json_path).await {
        Ok(pkg_json) => {
            println!("Found valid package: {}", pkg_json.name);
            // Process the package
        }
        Err(_) => {
            println!("Not a valid package directory");
        }
    }
    
    Ok(())
}
```

**Benefits of v2.0:**
- More explicit control flow
- Option to validate JSON structure (not just file existence)
- Standard error handling patterns

---

## Type Changes

### Return Type Changes

| Operation | v1.x Return Type | v2.0 Return Type | Notes |
|-----------|------------------|------------------|-------|
| Find packages | `Vec<PathBuf>` | `Vec<WorkspacePackage>` | Much richer data structure |
| Check directory | `bool` | `bool` (via `exists()`) | Same semantic, different call |
| Read package.json | `PackageJson` | `PackageJson` | Unchanged |
| Create Package | `Package` | `Package` | Unchanged |
| Validate | `ValidationResult` | `ValidationResult` | Unchanged |

### WorkspacePackage Structure (New in v2.0)

```rust
pub struct WorkspacePackage {
    pub name: String,
    pub version: String,
    pub location: PathBuf,                    // Relative to workspace root
    pub absolute_path: PathBuf,               // Absolute path
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub workspace_dependencies: Vec<String>,  // Internal dependencies
    pub workspace_dev_dependencies: Vec<String>,
}
```

---

## Error Handling

### Error Types (Unchanged)

Both v1.x and v2.0 use the same error types:

```rust
use sublime_pkg_tools::error::PackageError;

// Errors remain the same
match PackageJson::read_from_path(&fs, path).await {
    Ok(pkg) => { /* ... */ }
    Err(PackageError::FileNotFound { .. }) => { /* ... */ }
    Err(PackageError::ParseError { .. }) => { /* ... */ }
    Err(e) => { /* ... */ }
}
```

---

## Best Practices

### v1.x Best Practices

```rust
// ✅ Good in v1.x
use sublime_pkg_tools::package::{
    find_package_directories,
    read_package_json,
};

let dirs = find_package_directories(&fs, root, None).await?;
for dir in dirs {
    let pkg = read_package_json(&fs, &dir.join("package.json")).await?;
    // Process package
}
```

### v2.0 Best Practices

```rust
// ✅ Better in v2.0 - use unified detection
use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorTrait};
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
use sublime_pkg_tools::package::Package;

let project_detector = ProjectDetector::with_filesystem(fs.clone());
let project = project_detector.detect(root, None).await?;

if project.as_project_info().kind().is_monorepo() {
    let monorepo_detector = MonorepoDetector::with_filesystem(fs);
    let packages = monorepo_detector.detect_packages(root).await?;
    
    // WorkspacePackage has rich metadata
    for pkg in packages {
        println!("{} depends on: {:?}", pkg.name, pkg.workspace_dependencies);
    }
} else {
    let package = Package::from_path(&fs, root).await?;
    // Single package handling
}
```

```rust
// ✅ Good in v2.0 - direct type methods
use sublime_pkg_tools::package::{PackageJson, Package};

let pkg_json = PackageJson::read_from_path(&fs, path).await?;
let package = Package::from_path(&fs, dir).await?;
```

```rust
// ✅ Good in v2.0 - use standard filesystem
use sublime_standard_tools::filesystem::AsyncFileSystem;

if fs.exists(&path.join("package.json")).await {
    // Process package
}
```

---

## Summary

### Why the Changes?

1. **Eliminate Redundancy**: Stop duplicating functionality between crates
2. **Clear Responsibilities**: `pkg` = package.json specifics, `standard` = project structure
3. **Better APIs**: Type methods instead of free functions
4. **Richer Data**: `WorkspacePackage` vs just `PathBuf`
5. **Unified Detection**: Handle single and monorepo consistently

### Migration Effort

| Complexity | Estimated Time | Scope |
|------------|----------------|-------|
| Low | 15-30 min | Simple usage (read/write package.json) |
| Medium | 1-2 hours | Moderate usage (finding packages, multiple operations) |
| High | Half day | Complex usage (custom detection logic, many integrations) |

### Key Takeaways

- ✅ Core types (`Package`, `PackageJson`, `PackageJsonEditor`) unchanged
- ✅ Validation API unchanged
- ❌ Convenience wrappers removed
- ✅ Replaced with direct type methods and standard tools
- ✅ Richer metadata available in v2.0
- ✅ Better separation of concerns

---

**Document Version:** 1.0  
**Last Updated:** 2025-01-15  
**Related:** [Migration Guide](./MIGRATION_GUIDE_V2.md), [API Refactoring](./API_REFACTORING.md)