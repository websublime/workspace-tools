# Migration Guide - Package API v2

**Version:** 2.0.0  
**Date:** 2025-01-15  
**Status:** Draft

## Overview

This guide helps you migrate from `sublime_pkg_tools` v1.x to v2.0, which removes redundant convenience functions in favor of using APIs from `sublime_standard_tools` directly.

## Summary of Changes

Version 2.0 removes convenience wrapper functions that duplicated functionality from `sublime_standard_tools`, making the API cleaner and more maintainable. The core types (`Package`, `PackageJson`, `PackageJsonEditor`, etc.) remain unchanged.

### Removed Functions

- `read_package_json()` - Use `PackageJson::read_from_path()` instead
- `create_package_from_directory()` - Use `Package::from_path()` instead
- `is_package_directory()` - Use `AsyncFileSystem::exists()` instead
- `find_package_directories()` - Use `MonorepoDetector::detect_packages()` instead

### Unchanged Functions

- `validate_package_json()` - ✅ Still available (package.json-specific validation)

## Quick Migration Checklist

- [ ] Replace `read_package_json()` calls with `PackageJson::read_from_path()`
- [ ] Replace `create_package_from_directory()` calls with `Package::from_path()`
- [ ] Replace `is_package_directory()` with filesystem checks
- [ ] Replace `find_package_directories()` with `MonorepoDetector` or `ProjectDetector`
- [ ] Update imports to remove removed functions
- [ ] Add `sublime_standard_tools` imports where needed
- [ ] Test all affected code paths

## Detailed Migration Instructions

### 1. Reading package.json Files

#### Before (v1.x)

```rust
use sublime_pkg_tools::package::read_package_json;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let path = Path::new("./package.json");
    
    // Old API
    let package_json = read_package_json(&fs, path).await?;
    
    println!("Package: {} v{}", package_json.name, package_json.version);
    Ok(())
}
```

#### After (v2.0)

```rust
use sublime_pkg_tools::package::PackageJson;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let path = Path::new("./package.json");
    
    // New API - direct method call
    let package_json = PackageJson::read_from_path(&fs, path).await?;
    
    println!("Package: {} v{}", package_json.name, package_json.version);
    Ok(())
}
```

**Changes:**
- Import `PackageJson` instead of `read_package_json`
- Call `PackageJson::read_from_path()` directly
- Same arguments, same behavior

**Benefits:**
- More discoverable through IDE autocomplete
- Follows standard Rust patterns (`Type::from_*()`)
- Clear that you're working with `PackageJson` type

---

### 2. Creating Package from Directory

#### Before (v1.x)

```rust
use sublime_pkg_tools::package::{create_package_from_directory, Package};
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let dir = Path::new("./packages/auth");
    
    // Old API
    let package = create_package_from_directory(&fs, dir).await?;
    
    println!("Package: {}", package.name());
    Ok(())
}
```

#### After (v2.0)

```rust
use sublime_pkg_tools::package::Package;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let dir = Path::new("./packages/auth");
    
    // New API - direct method call
    let package = Package::from_path(&fs, dir).await?;
    
    println!("Package: {}", package.name());
    Ok(())
}
```

**Changes:**
- Remove `create_package_from_directory` import
- Call `Package::from_path()` directly
- Same arguments, same behavior

**Benefits:**
- Standard Rust constructor pattern (`Type::from_*()`)
- Clearer intent - constructing a `Package`
- Less API surface to remember

---

### 3. Checking for package.json

#### Before (v1.x)

```rust
use sublime_pkg_tools::package::is_package_directory;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let dir = Path::new("./packages/auth");
    
    // Old API
    if is_package_directory(&fs, dir).await {
        println!("Found package.json");
    }
    
    Ok(())
}
```

#### After (v2.0)

```rust
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use std::path::Path;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let dir = Path::new("./packages/auth");
    
    // New API - direct filesystem check
    let package_json_path = dir.join("package.json");
    if fs.exists(&package_json_path).await {
        println!("Found package.json");
    }
    
    Ok(())
}
```

**Changes:**
- Use `AsyncFileSystem::exists()` directly
- Construct path to `package.json` explicitly
- Import from `sublime_standard_tools::filesystem`

**Benefits:**
- More explicit about what you're checking
- Uses standard filesystem operations
- More flexible (can check for other files too)

**Alternative (if you need to parse it anyway):**

```rust
use sublime_pkg_tools::package::PackageJson;
use sublime_standard_tools::filesystem::FileSystemManager;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let dir = Path::new("./packages/auth");
    
    // Try to read package.json - existence check is implicit
    let package_json_path = dir.join("package.json");
    match PackageJson::read_from_path(&fs, &package_json_path).await {
        Ok(package_json) => {
            println!("Found and parsed: {}", package_json.name);
        }
        Err(_) => {
            println!("No valid package.json");
        }
    }
    
    Ok(())
}
```

---

### 4. Finding Packages in Workspace

This is the most significant change, as package discovery should use `sublime_standard_tools` capabilities.

#### Before (v1.x)

```rust
use sublime_pkg_tools::package::find_package_directories;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let root = Path::new("./");
    
    // Old API
    let package_dirs = find_package_directories(&fs, root, Some(3)).await?;
    
    for dir in package_dirs {
        println!("Found package at: {}", dir.display());
    }
    
    Ok(())
}
```

#### After (v2.0) - For Monorepos

```rust
use sublime_standard_tools::filesystem::FileSystemManager;
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
use std::path::Path;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let root = Path::new("./");
    
    // New API - use MonorepoDetector
    let detector = MonorepoDetector::with_filesystem(fs);
    
    // Check if it's a monorepo
    match detector.is_monorepo_root(root).await? {
        Some(monorepo_kind) => {
            println!("Detected {} monorepo", monorepo_kind.name());
            
            // Get all packages
            let packages = detector.detect_packages(root).await?;
            
            for package in packages {
                println!("Found package: {} at {}", 
                    package.name, 
                    package.absolute_path.display()
                );
            }
        }
        None => {
            println!("Not a monorepo - single package at root");
        }
    }
    
    Ok(())
}
```

#### After (v2.0) - Unified Detection (Single or Monorepo)

```rust
use sublime_standard_tools::filesystem::FileSystemManager;
use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorTrait};
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
use sublime_pkg_tools::package::Package;
use std::path::Path;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let root = Path::new("./");
    
    // Unified detection for both single and monorepo
    let project_detector = ProjectDetector::with_filesystem(fs.clone());
    let project = project_detector.detect(root, None).await?;
    
    if project.as_project_info().kind().is_monorepo() {
        // It's a monorepo - get all packages
        let monorepo_detector = MonorepoDetector::with_filesystem(fs.clone());
        let packages = monorepo_detector.detect_packages(root).await?;
        
        println!("Monorepo with {} packages", packages.len());
        for pkg in packages {
            println!("  - {} at {}", pkg.name, pkg.absolute_path.display());
        }
    } else {
        // Single package project
        let package = Package::from_path(&fs, root).await?;
        println!("Single package: {}", package.name());
    }
    
    Ok(())
}
```

**Changes:**
- Use `MonorepoDetector` for monorepo-specific operations
- Use `ProjectDetector` for unified single/monorepo detection
- Get richer `WorkspacePackage` objects instead of just paths
- More explicit about what you're detecting

**Benefits:**
- Handles both monorepo and single-package projects correctly
- More robust detection (uses workspace patterns, config files, etc.)
- Returns structured data (`WorkspacePackage`) with dependencies info
- Consistent with standard crate patterns

---

### 5. Validating package.json (Unchanged)

#### v1.x and v2.0 (Same)

```rust
use sublime_pkg_tools::package::validate_package_json;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::Path;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystemManager::new();
    let path = Path::new("./package.json");
    
    // This API is unchanged
    let result = validate_package_json(&fs, path).await?;
    
    if result.has_errors() {
        for error in result.errors() {
            eprintln!("Error: {}", error.message);
        }
    }
    
    Ok(())
}
```

**No changes required** - validation is package.json-specific and remains in the pkg crate.

---

## Common Migration Patterns

### Pattern 1: Iterating Over Packages

#### Before (v1.x)

```rust
let package_dirs = find_package_directories(&fs, root, Some(3)).await?;

for dir in package_dirs {
    let package = create_package_from_directory(&fs, &dir).await?;
    println!("Package: {}", package.name());
}
```

#### After (v2.0)

```rust
let detector = MonorepoDetector::with_filesystem(fs.clone());
let packages = detector.detect_packages(root).await?;

for pkg in packages {
    let package = Package::from_path(&fs, &pkg.absolute_path).await?;
    println!("Package: {}", package.name());
}
```

**Or more efficiently:**

```rust
let detector = MonorepoDetector::with_filesystem(fs.clone());
let workspace_packages = detector.detect_packages(root).await?;

// WorkspacePackage already has name and path
for pkg in workspace_packages {
    println!("Package: {} at {}", pkg.name, pkg.absolute_path.display());
    
    // Only create Package if you need full metadata
    if need_full_metadata {
        let package = Package::from_path(&fs, &pkg.absolute_path).await?;
        // ...
    }
}
```

---

### Pattern 2: Conditional Package Operations

#### Before (v1.x)

```rust
if is_package_directory(&fs, dir).await {
    let package_json = read_package_json(&fs, &dir.join("package.json")).await?;
    // Process package
}
```

#### After (v2.0)

```rust
let package_json_path = dir.join("package.json");
if fs.exists(&package_json_path).await {
    let package_json = PackageJson::read_from_path(&fs, &package_json_path).await?;
    // Process package
}
```

**Or more robustly:**

```rust
let package_json_path = dir.join("package.json");
match PackageJson::read_from_path(&fs, &package_json_path).await {
    Ok(package_json) => {
        // Process package
    }
    Err(_) => {
        // Not a package or invalid package.json
    }
}
```

---

### Pattern 3: Building Package List

#### Before (v1.x)

```rust
let mut packages = Vec::new();
let dirs = find_package_directories(&fs, root, Some(3)).await?;

for dir in dirs {
    let package = create_package_from_directory(&fs, &dir).await?;
    packages.push(package);
}
```

#### After (v2.0)

```rust
use futures::future::try_join_all;

let detector = MonorepoDetector::with_filesystem(fs.clone());
let workspace_packages = detector.detect_packages(root).await?;

// Sequential
let mut packages = Vec::new();
for pkg in workspace_packages {
    let package = Package::from_path(&fs, &pkg.absolute_path).await?;
    packages.push(package);
}

// Or parallel (more efficient)
let package_futures = workspace_packages
    .into_iter()
    .map(|pkg| Package::from_path(&fs, &pkg.absolute_path));

let packages = try_join_all(package_futures).await?;
```

---

## Import Changes Checklist

### Remove These Imports

```rust
// ❌ Remove
use sublime_pkg_tools::package::{
    read_package_json,
    create_package_from_directory,
    is_package_directory,
    find_package_directories,
};
```

### Add These Imports (as needed)

```rust
// ✅ Add for reading package.json
use sublime_pkg_tools::package::PackageJson;

// ✅ Add for creating Package
use sublime_pkg_tools::package::Package;

// ✅ Add for filesystem checks
use sublime_standard_tools::filesystem::AsyncFileSystem;

// ✅ Add for finding packages in monorepo
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};

// ✅ Add for unified project detection
use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorTrait};
```

### Keep These Imports (unchanged)

```rust
// ✅ Keep - validation still in pkg crate
use sublime_pkg_tools::package::validate_package_json;

// ✅ Keep - core types unchanged
use sublime_pkg_tools::package::{
    Package, PackageInfo, PackageJson, PackageJsonEditor,
    PackageJsonModification, PackageJsonValidator,
    Dependencies, ValidationResult, ValidationIssue,
};
```

---

## Cargo.toml Updates

If you were only using the removed convenience functions, you might now need to explicitly depend on `sublime_standard_tools`:

```toml
[dependencies]
sublime_pkg_tools = "2.0"
sublime_standard_tools = "1.0"  # Add if not already present
```

---

## Troubleshooting

### Issue: "Cannot find function `read_package_json`"

**Solution:**
```rust
// Change this:
use sublime_pkg_tools::package::read_package_json;
let pkg = read_package_json(&fs, path).await?;

// To this:
use sublime_pkg_tools::package::PackageJson;
let pkg = PackageJson::read_from_path(&fs, path).await?;
```

---

### Issue: "Cannot find function `find_package_directories`"

**Solution:**
```rust
// Change this:
use sublime_pkg_tools::package::find_package_directories;
let dirs = find_package_directories(&fs, root, max_depth).await?;

// To this:
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
let detector = MonorepoDetector::with_filesystem(fs);
let packages = detector.detect_packages(root).await?;
let dirs: Vec<PathBuf> = packages.into_iter()
    .map(|p| p.absolute_path)
    .collect();
```

---

### Issue: "How do I check if a directory is a package?"

**Solution:**
```rust
// Simple existence check:
use sublime_standard_tools::filesystem::AsyncFileSystem;
let is_package = fs.exists(&dir.join("package.json")).await;

// Or try to parse (more robust):
use sublime_pkg_tools::package::PackageJson;
let is_package = PackageJson::read_from_path(&fs, &dir.join("package.json"))
    .await
    .is_ok();
```

---

### Issue: "I need to find packages but don't know if it's a monorepo"

**Solution:** Use `ProjectDetector` for unified detection:

```rust
use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorTrait};
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};

let project_detector = ProjectDetector::with_filesystem(fs.clone());
let project = project_detector.detect(root, None).await?;

if project.as_project_info().kind().is_monorepo() {
    let monorepo_detector = MonorepoDetector::with_filesystem(fs);
    let packages = monorepo_detector.detect_packages(root).await?;
    // Handle multiple packages
} else {
    // Handle single package
    let package = Package::from_path(&fs, root).await?;
}
```

---

## FAQ

### Q: Why were these functions removed?

**A:** They were thin wrappers that duplicated functionality from `sublime_standard_tools`, adding maintenance burden without providing value. Using the underlying APIs directly is clearer and more maintainable.

### Q: Is this a breaking change?

**A:** Yes, this is a breaking change requiring a major version bump (1.x → 2.0). However, migration is straightforward with clear replacements for all removed functions.

### Q: Will `validate_package_json()` be removed too?

**A:** No. Validation is package.json-specific and provides domain-specific rules that don't belong in the generic standard tools. It remains in the pkg crate.

### Q: What if I prefer the old convenience functions?

**A:** You can create your own wrappers in your codebase:

```rust
pub async fn read_package_json<F: AsyncFileSystem>(
    fs: &F,
    path: &Path,
) -> Result<PackageJson, Error> {
    PackageJson::read_from_path(fs, path).await
}
```

However, we recommend using the direct APIs for better discoverability and maintainability.

### Q: How does this affect performance?

**A:** No performance impact - the removed functions were just thin wrappers. The underlying operations are identical.

### Q: Are there any new features in v2.0?

**A:** V2.0 focuses on API cleanup. New features may be added in 2.1+. Check the CHANGELOG for details.

---

## Additional Resources

- [API Refactoring Documentation](./API_REFACTORING.md) - Detailed technical discussion
- [sublime_standard_tools Documentation](../../standard/SPEC.md) - Full standard tools API
- [sublime_pkg_tools SPEC](../SPEC.md) - Updated v2.0 specification
- [Examples Directory](../examples/) - Updated examples using v2.0 API

---

## Support

If you encounter issues during migration:

1. Check this guide for the specific function you're migrating
2. Review the examples in the `examples/` directory
3. Check the API documentation: `cargo doc --open`
4. Search existing GitHub issues
5. Open a new issue with a minimal reproduction case

---

**Document Version:** 1.0  
**Last Updated:** 2025-01-15  
**Applies to:** sublime_pkg_tools v2.0.0+