# Single Repo & Monorepo Support Fix

**Issue**: Repository Type Detection and Package Resolution  
**Story**: 1.4 - Basic Version Types  
**Fix Date**: 2024-01-15  
**Status**: ✅ **RESOLVED**  

## Problem Identified

The `VersionResolver` implementation was making the assumption that all projects are monorepos, which is incorrect. A project can be either:

1. **Single Repository**: One package with `package.json` at the root
2. **Monorepo**: Multiple packages with workspace configuration

### Original Issue:
```rust
// ❌ BEFORE: Assumed all projects are monorepos
async fn find_package_path(...) -> PackageResult<PathBuf> {
    // Always tried monorepo detection
    let monorepo_kind = self.monorepo_detector.is_monorepo_root(workspace_root).await?;
    
    if monorepo_kind.is_none() {
        // ❌ Failed immediately if not a monorepo
        return Err("Workspace root is not a monorepo");
    }
    
    // Only handled monorepo case
}
```

## Solution Implemented

Implemented proper repository type detection with support for both scenarios:

### ✅ Fixed Implementation:

```rust
// ✅ AFTER: Handles both single repo and monorepo
async fn find_package_path(
    &self,
    package_name: &str,
    workspace_root: &Path,
) -> PackageResult<PathBuf> {
    // 1. Detect repository type
    let monorepo_kind = self.monorepo_detector
        .is_monorepo_root(workspace_root)
        .await?;

    match monorepo_kind {
        Some(_) => {
            // Handle monorepo case
            self.find_package_in_monorepo(package_name, workspace_root).await
        }
        None => {
            // Handle single repository case
            self.find_package_in_single_repo(package_name, workspace_root).await
        }
    }
}
```

## Implementation Details

### 1. Single Repository Handling

```rust
async fn find_package_in_single_repo(
    &self,
    package_name: &str,
    workspace_root: &Path,
) -> PackageResult<PathBuf> {
    // Check for package.json at repository root
    let package_json_path = workspace_root.join("package.json");
    
    if !self.filesystem.exists(&package_json_path).await {
        return Err(/* No package.json found at root */);
    }
    
    // Read and parse package.json
    let content = self.filesystem.read_file_string(&package_json_path).await?;
    let package_json: serde_json::Value = serde_json::from_str(&content)?;
    
    // Match package name
    let actual_name = package_json["name"].as_str().ok_or(/* No name field */)?;
    
    if actual_name == package_name {
        Ok(workspace_root.to_path_buf())
    } else {
        Err(/* Name mismatch with detailed error */)
    }
}
```

### 2. Monorepo Handling (Unchanged)

```rust
async fn find_package_in_monorepo(
    &self,
    package_name: &str,
    workspace_root: &Path,
) -> PackageResult<PathBuf> {
    // Use MonorepoDetector for workspace analysis
    let monorepo = self.monorepo_detector.detect_monorepo(workspace_root).await?;
    
    // Find package using workspace structure
    if let Some(workspace_package) = monorepo.get_package(package_name) {
        Ok(workspace_package.absolute_path.clone())
    } else {
        Err(/* Detailed error with available packages */)
    }
}
```

## Repository Type Detection Logic

### Decision Flow:

```
1. Call monorepo_detector.is_monorepo_root(path)
   ├─ Returns Some(MonorepoKind) → Monorepo detected
   │  └─ Use find_package_in_monorepo()
   └─ Returns None → Single repository
      └─ Use find_package_in_single_repo()
```

### Supported Repository Types:

#### Single Repository:
```
my-service/
├── package.json          # Contains: {"name": "@myorg/my-service"}
├── src/
├── tests/
└── README.md
```

#### Monorepo (Multiple Types):
```
# npm/yarn workspaces
my-workspace/
├── package.json          # Contains: {"workspaces": ["packages/*"]}
├── packages/
│   ├── service-a/
│   └── service-b/

# pnpm workspaces
my-workspace/
├── pnpm-workspace.yaml   # Contains: packages: ['packages/*']
├── packages/
│   ├── service-a/
│   └── service-b/

# lerna
my-workspace/
├── lerna.json            # Lerna configuration
├── packages/
│   ├── service-a/
│   └── service-b/
```

## Error Handling Improvements

### Single Repository Errors:
- `"No package.json found at repository root"`
- `"No name field found in package.json"`
- `"Package name mismatch: Expected 'X', Found 'Y'"`

### Monorepo Errors:
- `"Package 'X' not found in Y monorepo. Available packages: A, B, C"`
- `"Failed to analyze monorepo: <detailed reason>"`

### Repository Detection Errors:
- `"Failed to detect repository type: <detailed reason>"`

## API Usage Examples

### Single Repository:
```rust
let resolver = VersionResolver::new(fs, repo, config);

// In single repo, package name must match root package.json name
let version = resolver.resolve_package_version(
    "@myorg/my-service",  // Must match package.json["name"]
    Path::new(".")        // Repository root
).await?;
```

### Monorepo:
```rust
let resolver = VersionResolver::new(fs, repo, config);

// In monorepo, searches across all workspace packages
let version = resolver.resolve_package_version(
    "@myorg/auth-service", // Finds in packages/auth-service/
    Path::new(".")         // Monorepo root
).await?;
```

## Testing Coverage

### New Tests Added:

1. **Repository Type Detection Logic**:
   ```rust
   #[test]
   fn test_single_repo_vs_monorepo_detection()
   ```

2. **Package Name Matching**:
   ```rust
   #[test]
   fn test_package_name_matching_logic()
   ```

3. **Error Message Formatting**:
   ```rust
   #[test]
   fn test_error_message_formatting()
   ```

4. **Repository Scenarios**:
   ```rust
   #[test]
   fn test_repository_type_scenarios()
   ```

### Test Results:
```
running 19 tests
test version::tests::version_tests::test_repository_type_scenarios ... ok
test version::tests::version_tests::test_package_name_matching_logic ... ok
test version::tests::version_tests::test_single_repo_vs_monorepo_detection ... ok
test version::tests::version_tests::test_error_message_formatting ... ok
[... all other tests ...]

test result: ok. 19 passed; 0 failed; 0 ignored
```

## Documentation Updates

### Updated Method Documentation:
- `resolve_package_version()`: Now explains both single repo and monorepo behavior
- `find_package_path()`: Clarified to handle both repository types
- Added examples for both scenarios

### Updated Examples:
- `examples/version_resolver_example.rs`: Demonstrates both single repo and monorepo usage
- Clear separation between single repository and monorepo sections
- Visual structure diagrams for both types

## Real-World Impact

### Before Fix:
- ❌ Only worked with monorepos
- ❌ Failed silently or with generic errors on single repos
- ❌ Required workspace configuration even for simple projects

### After Fix:
- ✅ Works with both single repositories and monorepos
- ✅ Automatic repository type detection
- ✅ Appropriate search strategy for each type
- ✅ Detailed error messages with context
- ✅ Single API for all repository types

## Performance Considerations

### Single Repository:
- **Fast**: Only reads one `package.json` file
- **Efficient**: Direct path resolution without workspace analysis

### Monorepo:
- **Comprehensive**: Full workspace analysis for accurate results
- **Cached**: MonorepoDetector can cache workspace structure
- **Scalable**: Handles large workspaces efficiently

## Compatibility

### Backward Compatibility:
- ✅ Existing monorepo functionality unchanged
- ✅ All existing APIs work as before
- ✅ No breaking changes to public interface

### Forward Compatibility:
- ✅ Ready for future repository type support
- ✅ Extensible architecture for custom workspace types
- ✅ Prepared for advanced monorepo features

## Quality Verification

### Compilation:
```bash
$ cargo check --manifest-path crates/pkg/Cargo.toml
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.92s
```

### Clippy Compliance:
```bash
$ cargo clippy --manifest-path crates/pkg/Cargo.toml -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.22s
```

### Full Test Suite:
```bash
$ cargo test --manifest-path crates/pkg/Cargo.toml
test result: ok. 148 passed; 0 failed; 11 ignored
```

### Example Execution:
```bash
$ cargo run --example version_resolver_example
✅ All examples completed successfully!
```

## Files Modified

### Core Implementation:
- `src/version/resolver.rs`: Added repository type detection and single repo support
- Split `find_package_path` into separate methods for each repository type

### Testing:
- `src/version/tests.rs`: Added 4 new tests covering repository type scenarios
- Increased test coverage from 15 to 19 tests

### Documentation:
- `examples/version_resolver_example.rs`: Updated to demonstrate both scenarios
- Method documentation updated with single repo examples

## Conclusion

The `VersionResolver` now properly supports both single repositories and monorepos through:

1. ✅ **Automatic Detection**: Uses `MonorepoDetector` to determine repository type
2. ✅ **Appropriate Handling**: Different search strategies for each type
3. ✅ **Unified API**: Single interface works for both scenarios
4. ✅ **Better Errors**: Context-aware error messages for each case
5. ✅ **Complete Testing**: Comprehensive test coverage for all scenarios
6. ✅ **Clear Documentation**: Examples and documentation for both use cases

The implementation now correctly handles the reality that projects can be either single repositories or monorepos, without making assumptions about the repository structure.

**Status**: ✅ **SINGLE REPO & MONOREPO SUPPORT COMPLETE**