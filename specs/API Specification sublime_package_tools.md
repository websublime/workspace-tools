---
type: Definition
title: 'API Specification: `sublime_package_tools` Crate'
tags: [workspace-tools, rust]
---

# API Specification: `sublime_package_tools` Crate

[Workspace Tools](https://app.capacities.io/f8d3f900-cfe2-4c6d-b44c-906946be3e3c/52c8c019-b006-462b-8c5d-6b518afa0d64)

## Overview

The `sublime_package_tools` crate provides a comprehensive toolkit for package dependency management, primarily focused on Node.js ecosystem projects. It enables dependency graph analysis, version management, conflict detection, visualization, and package upgrading capabilities.

## Architecture

The crate is organized around several key concepts:

1. **Packages**: Represents software packages with version information and dependencies

2. **Dependencies**: Models relationships between packages with version requirements

3. **Graphs**: Provides dependency graph construction and analysis tools

4. **Registries**: Interfaces with package registries (npm, local)

5. **Upgrading**: Finds and applies available package upgrades

6. **Visualization**: Generates visual representations of dependency graphs

The architecture follows a modular approach with clear separation of concerns. Core functionality is built around the `Package`, `Dependency`, and `DependencyGraph` types, with additional services like registries and the upgrader building on these foundations.

## Package Crate API

### Core Types

#### `Dependency`

Represents a package dependency with version requirements.

```rust
pub struct Dependency {
    // Name of the dependency
    name: String,
    // Version requirement
    version: Rc<RefCell<VersionReq>>,
}
impl Dependency {
    // Create a new dependency with name and version requirement
    pub fn new(name: &str, version: &str) -> Result<Self, VersionError>;
    
    // Get the dependency name
    pub fn name(&self) -> &str;
    
    // Get the version requirement
    pub fn version(&self) -> VersionReq;
    
    // Get fixed version (without operators like ^ or ~)
    pub fn fixed_version(&self) -> Result<Version, VersionError>;
    
    // Compare versions
    pub fn compare_versions(&self, other: &str) -> Result<Ordering, VersionError>;
    
    // Update version requirement
    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError>;
    
    // Check if a version matches this dependency's requirement
    pub fn matches(&self, version: &str) -> Result<bool, VersionError>;
}
```

**Example Usage:**

```rust
use sublime_package_tools::Dependency;
// Create a dependency
let dependency = Dependency::new("react", "^17.0.2")?;
// Check if a version satisfies the requirement
let satisfies = dependency.matches("17.0.5")?;
assert!(satisfies);
// Update version requirement
dependency.update_version("^18.0.0")?;
```

#### `DependencyChange`

Represents changes to dependencies between package versions.

```rust
pub struct DependencyChange {
    // Name of the dependency
    pub name: String,
    // Previous version (None if newly added)
    pub previous_version: Option<String>,
    // Current version (None if removed)
    pub current_version: Option<String>,
    // Type of change
    pub change_type: ChangeType,
    // Whether this is a breaking change based on semver
    pub breaking: bool,
}
impl DependencyChange {
    // Creates a new dependency change
    pub fn new(
        name: &str,
        previous_version: Option<&str>,
        current_version: Option<&str>,
        change_type: ChangeType,
    ) -> Self;
}
```

**Example Usage:**

```rust
use sublime_package_tools::{ChangeType, DependencyChange};
// Record a dependency that was updated
let change = DependencyChange::new(
    "express",
    Some("^4.17.1"),
    Some("^4.18.1"),
    ChangeType::Updated
);
println!("Breaking change: {}", change.breaking); // false
```

#### `DependencyFilter`

Controls which types of dependencies are included in operations.

```rust
pub enum DependencyFilter {
    // Include only production dependencies
    ProductionOnly,
    // Include production and development dependencies
    WithDevelopment,
    // Include production, development, and optional dependencies
    AllDependencies,
}
```

#### `Package`

Represents a software package with name, version, and dependencies.

```rust
pub struct Package {
    name: String,
    version: Rc<RefCell<Version>>,
    dependencies: Vec<Rc<RefCell<Dependency>>>,
}
impl Package {
    // Create a new package with name, version, and optional dependencies
    pub fn new(
        name: &str,
        version: &str,
        dependencies: Option<Vec<Rc<RefCell<Dependency>>>>,
    ) -> Result<Self, VersionError>;
    
    // Create a new package using the dependency registry
    pub fn new_with_registry(
        name: &str,
        version: &str,
        dependencies: Option<Vec<(&str, &str)>>,
        registry: &mut DependencyRegistry,
    ) -> Result<Self, VersionError>;
    
    // Get the package name
    pub fn name(&self) -> &str;
    
    // Get the package version
    pub fn version(&self) -> Version;
    
    // Get the package version as a string
    pub fn version_str(&self) -> String;
    
    // Update the package version
    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError>;
    
    // Get the package dependencies
    pub fn dependencies(&self) -> &[Rc<RefCell<Dependency>>];
    
    // Update a dependency version
    pub fn update_dependency_version(
        &self,
        dep_name: &str,
        new_version: &str,
    ) -> Result<(), DependencyResolutionError>;
    
    // Add a dependency to the package
    pub fn add_dependency(&mut self, dependency: Rc<RefCell<Dependency>>);
    
    // Update package dependencies based on resolution result
    pub fn update_dependencies_from_resolution(
        &self,
        resolution: &ResolutionResult,
    ) -> Result<Vec<(String, String, String)>, VersionError>;
}
```

**Example Usage:**

```rust
use sublime_package_tools::{Dependency, DependencyRegistry, Package};
use std::rc::Rc;
use std::cell::RefCell;
// Create dependencies
let dep1 = Rc::new(RefCell::new(Dependency::new("express", "^4.17.1")?));
let dep2 = Rc::new(RefCell::new(Dependency::new("lodash", "^4.17.21")?));
// Create package with dependencies
let package = Package::new(
    "my-server",
    "1.0.0",
    Some(vec![Rc::clone(&dep1), Rc::clone(&dep2)])
)?;
// Alternatively, use the registry
let mut registry = DependencyRegistry::new();
let package = Package::new_with_registry(
    "my-server",
    "1.0.0",
    Some(vec![("express", "^4.17.1"), ("lodash", "^4.17.21")]),
    &mut registry
)?;
```

#### `PackageInfo`

Wraps a package with additional file system metadata.

```rust
pub struct PackageInfo {
    pub package: Rc<RefCell<Package>>,
    pub package_json_path: String,
    pub package_path: String, 
    pub package_relative_path: String,
    pub pkg_json: Rc<RefCell<Value>>,
}
impl PackageInfo {
    // Create a new package info
    pub fn new(
        package: Package,
        package_json_path: String,
        package_path: String,
        package_relative_path: String,
        pkg_json: Value,
    ) -> Self;
    
    // Update the package version
    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError>;
    
    // Update a dependency version
    pub fn update_dependency_version(
        &self, 
        dep_name: &str,
        new_version: &str,
    ) -> Result<(), DependencyResolutionError>;
    
    // Apply dependency resolution across all packages
    pub fn apply_dependency_resolution(
        &self,
        resolution: &ResolutionResult,
    ) -> Result<(), VersionError>;
    
    // Write the package.json file to disk
    pub fn write_package_json(&self) -> Result<(), PackageError>;
}
```

**Example Usage:**

```rust
use sublime_package_tools::PackageInfo;
use serde_json::json;
// Create package info from existing Package and JSON data
let package_info = PackageInfo::new(
    package,
    "/path/to/package.json",
    "/path/to",
    "relative/path",
    json!({
        "name": "my-server",
        "version": "1.0.0",
        "dependencies": {
            "express": "^4.17.1",
            "lodash": "^4.17.21"
        }
    })
);
// Update a dependency version
package_info.update_dependency_version("express", "^4.18.0")?;
// Write changes to disk
package_info.write_package_json()?;
```

#### `PackageDiff`

Compares two versions of a package to identify changes.

```rust
pub struct PackageDiff {
    // Name of the package
    pub package_name: String,
    // Version before the change
    pub previous_version: String,
    // Version after the change
    pub current_version: String,
    // Changes to the dependencies
    pub dependency_changes: Vec<DependencyChange>,
    // Whether the package version change is breaking
    pub breaking_change: bool,
}
impl PackageDiff {
    // Generate a diff between two packages
    pub fn between(previous: &Package, current: &Package) -> Result<Self, PackageError>;
    
    // Count the number of breaking changes in dependencies
    pub fn count_breaking_changes(&self) -> usize;
    
    // Count changes by type
    pub fn count_changes_by_type(&self) -> HashMap<ChangeType, usize>;
}
```

**Example Usage:**

```rust
use sublime_package_tools::{Package, PackageDiff};
// Generate diff between package versions
let diff = PackageDiff::between(&old_package, &new_package)?;
println!("Package: {} ({}→{})", 
    diff.package_name, 
    diff.previous_version, 
    diff.current_version
);
println!("Breaking changes: {}", diff.count_breaking_changes());
// Print all dependency changes
for change in &diff.dependency_changes {
    println!("{} {}: {} → {}", 
        change.change_type,
        change.name,
        change.previous_version.as_deref().unwrap_or("none"),
        change.current_version.as_deref().unwrap_or("none")
    );
}
```

### Graph Module

#### `DependencyGraph`

Models the relationships between packages and their dependencies.

```rust
pub struct DependencyGraph<'a, N: Node> {
    pub graph: StableDiGraph<Step<'a, N>, ()>,
    pub node_indices: HashMap<N::Identifier, NodeIndex>,
    pub dependents: HashMap<N::Identifier, Vec<N::Identifier>>,
    pub cycles: Vec<Vec<N::Identifier>>,
}
impl<'a, N> DependencyGraph<'a, N>
where
    N: Node,
{
    // Check for circular dependencies
    pub fn detect_circular_dependencies(&self) -> &Self;
    
    // Returns whether the graph has any circular dependencies
    pub fn has_cycles(&self) -> bool;
    
    // Returns information about the cycles in the graph
    pub fn get_cycles(&self) -> &Vec<Vec<N::Identifier>>;
    
    // Get the cycle groups as strings for easier reporting
    pub fn get_cycle_strings(&self) -> Vec<Vec<String>>;
    
    // Find all external dependencies in the workspace
    pub fn find_external_dependencies(&self) -> Vec<String>
    where N: Node<DependencyType = Dependency>;
    
    // Find all version conflicts in the graph
    pub fn find_version_conflicts(&self) -> Option<HashMap<String, Vec<String>>>
    where N: Node<DependencyType = Dependency>;
    
    // Validates the dependency graph
    pub fn validate_package_dependencies(&self) -> Result<ValidationReport, DependencyResolutionError>
    where N: Node<DependencyType = Dependency>;
    
    // Get dependents of a node, even if cycles exist
    pub fn get_dependents(&self, id: &N::Identifier) -> Result<&Vec<N::Identifier>, PackageError>;
    
    // Validates the dependency graph with custom options
    pub fn validate_with_options(
        &self,
        options: &ValidationOptions,
    ) -> Result<ValidationReport, DependencyResolutionError>
    where N: Node<DependencyType = Dependency>;
}
```

**Example Usage:**

```rust
use sublime_package_tools::{build_dependency_graph_from_packages, ValidationOptions};
// Create dependency graph from packages
let graph = build_dependency_graph_from_packages(&packages);
// Check for circular dependencies
if graph.has_cycles() {
    println!("Found circular dependencies:");
    for cycle in graph.get_cycle_strings() {
        println!("  {}", cycle.join(" -> "));
    }
}
// Check for version conflicts
if let Some(conflicts) = graph.find_version_conflicts() {
    println!("Found version conflicts:");
    for (package, versions) in conflicts {
        println!("  {} has conflicting versions: {}", 
            package, versions.join(", "));
    }
}
// Validate with custom options
let options = ValidationOptions::new()
    .treat_unresolved_as_external(true)
    .with_internal_packages(vec!["@my-org/ui", "@my-org/core"]);
let validation = graph.validate_with_options(&options)?;
if validation.has_critical_issues() {
    println!("Critical issues found:");
    for issue in validation.critical_issues() {
        println!("  {}", issue.message());
    }
}
```

#### `ValidationReport` and `ValidationIssue`

Provide feedback on dependency validation issues.

```rust
pub enum ValidationIssue {
    // Circular dependency detected
    CircularDependency { path: Vec<String> },
    // Unresolved dependency
    UnresolvedDependency { name: String, version_req: String },
    // Version conflict
    VersionConflict { name: String, versions: Vec<String> },
}
pub struct ValidationReport {
    issues: Vec<ValidationIssue>,
}
impl ValidationReport {
    pub fn new() -> Self;
    pub fn add_issue(&mut self, issue: ValidationIssue);
    pub fn has_issues(&self) -> bool;
    pub fn issues(&self) -> &[ValidationIssue];
    pub fn has_critical_issues(&self) -> bool;
    pub fn has_warnings(&self) -> bool;
    pub fn critical_issues(&self) -> Vec<&ValidationIssue>;
    pub fn warnings(&self) -> Vec<&ValidationIssue>;
}
pub struct ValidationOptions {
    // If true, unresolved dependencies are treated as external and not flagged as errors
    pub treat_unresolved_as_external: bool,
    // Optional list of specific packages to consider internal
    pub internal_packages: Vec<String>,
}
impl ValidationOptions {
    pub fn new() -> Self;
    pub fn treat_unresolved_as_external(self, value: bool) -> Self;
    pub fn with_internal_packages<I, S>(self, packages: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>;
    pub fn is_internal_dependency(&self, name: &str) -> bool;
}
```

### Visualization Module

```rust
// Generate DOT format representation of a dependency graph
pub fn generate_dot<N: Node>(
    graph: &DependencyGraph<N>,
    options: &DotOptions,
) -> Result<String, std::fmt::Error>;
// Save DOT output to a file
pub fn save_dot_to_file(
    dot_content: &str, 
    file_path: &str
) -> std::io::Result<()>;
// Generate an ASCII representation of the dependency graph
pub fn generate_ascii<N: Node>(
    graph: &DependencyGraph<N>
) -> Result<String, std::fmt::Error>;
pub struct DotOptions {
    // Title of the graph
    pub title: String,
    // Whether to include external (unresolved) dependencies
    pub show_external: bool,
    // Whether to highlight circular dependencies
    pub highlight_cycles: bool,
}
```

**Example Usage:**

```rust
use sublime_package_tools::{generate_ascii, generate_dot, save_dot_to_file, DotOptions};
// Generate ASCII visualization for terminal output
let ascii = generate_ascii(&graph)?;
println!("{}", ascii);
// Generate DOT format for GraphViz
let dot_options = DotOptions {
    title: "My Project Dependencies".to_string(),
    show_external: true,
    highlight_cycles: true,
};
let dot = generate_dot(&graph, &dot_options)?;
save_dot_to_file(&dot, "dependencies.dot")?;
```

### Registry Module

#### `DependencyRegistry`

Registry for dependency instances.

```rust
pub struct DependencyRegistry {
    dependencies: HashMap<String, Rc<RefCell<Dependency>>>,
}
impl DependencyRegistry {
    pub fn new() -> Self;
    
    pub fn get_or_create(
        &mut self,
        name: &str,
        version: &str,
    ) -> Result<Rc<RefCell<Dependency>>, VersionError>;
    
    pub fn get(&self, name: &str) -> Option<Rc<RefCell<Dependency>>>;
    
    // Resolve version conflicts between dependencies
    pub fn resolve_version_conflicts(&self) -> Result<ResolutionResult, VersionError>;
    
    // Apply the resolution result to update all dependencies
    pub fn apply_resolution_result(
        &mut self,
        result: &ResolutionResult,
    ) -> Result<(), VersionError>;
}
pub struct ResolutionResult {
    /// Resolved versions for each package
    pub resolved_versions: HashMap<String, String>,
    /// Packages that need version updates
    pub updates_required: Vec<DependencyUpdate>,
}
pub struct DependencyUpdate {
    /// Package name
    pub package_name: String,
    /// Dependency name
    pub dependency_name: String,
    /// Current version
    pub current_version: String,
    /// New version to update to
    pub new_version: String,
}
```

#### `PackageRegistry`

Interface for interacting with package registries.

```rust
pub trait PackageRegistry {
    /// Get the latest version of a package
    fn get_latest_version(
        &self,
        package_name: &str,
    ) -> Result<Option<String>, PackageRegistryError>;
    /// Get all available versions of a package
    fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError>;
    /// Get metadata about a package
    fn get_package_info(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Value, PackageRegistryError>;
}
pub struct NpmRegistry {
    base_url: String,
    client: Client,
    user_agent: String,
    cache_ttl: Duration,
    // ... other fields
}
impl NpmRegistry {
    pub fn new(base_url: &str) -> Self;
    pub fn set_user_agent(&mut self, user_agent: &str) -> &mut Self;
    pub fn set_auth(&mut self, token: &str, auth_type: &str) -> &mut Self;
    pub fn set_cache_ttl(&mut self, ttl: Duration) -> &mut Self;
    pub fn clear_cache(&mut self);
}
pub struct LocalRegistry {
    packages: Arc<Mutex<HashMap<String, HashMap<String, Value>>>>,
}
```

#### `RegistryManager`

Manages multiple registries with scope associations.

```rust
pub struct RegistryManager {
    registries: HashMap<String, Arc<dyn PackageRegistry + Send + Sync>>,
    scopes: HashMap<String, String>,
    default_registry: String,
    auth_configs: HashMap<String, RegistryAuth>,
}
pub struct RegistryAuth {
    /// Auth token
    pub token: String,
    /// Token type (bearer, basic, etc)
    pub token_type: String,
    /// Whether to always use this auth
    pub always: bool,
}
pub enum RegistryType {
    /// npm registry
    Npm,
    /// GitHub packages registry
    GitHub,
    /// Custom registry
    Custom(String),
}
impl RegistryManager {
    pub fn new() -> Self;
    
    pub fn add_registry(&mut self, url: &str, registry_type: RegistryType) -> &Self;
    
    pub fn add_registry_instance(
        &mut self,
        url: &str,
        registry: Arc<dyn PackageRegistry + Send + Sync>,
    ) -> &Self;
    
    pub fn set_auth(
        &mut self,
        registry_url: &str,
        auth: RegistryAuth,
    ) -> Result<&Self, RegistryError>;
    
    pub fn associate_scope(
        &mut self,
        scope: &str,
        registry_url: &str,
    ) -> Result<&Self, RegistryError>;
    
    pub fn set_default_registry(&mut self, registry_url: &str) -> Result<&Self, RegistryError>;
    
    pub fn get_registry_for_package(
        &self,
        package_name: &str,
    ) -> Arc<dyn PackageRegistry + Send + Sync>;
    
    pub fn get_latest_version(
        &self,
        package_name: &str,
    ) -> Result<Option<String>, PackageRegistryError>;
    
    pub fn get_all_versions(
        &self,
        package_name: &str,
    ) -> Result<Vec<String>, PackageRegistryError>;
    
    pub fn get_package_info(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<serde_json::Value, PackageRegistryError>;
    
    pub fn load_from_npmrc(&mut self, npmrc_path: Option<&str>) -> Result<&Self, RegistryError>;
}
```

**Example Usage:**

```rust
use sublime_package_tools::{RegistryManager, RegistryType, RegistryAuth};
use std::time::Duration;
// Create registry manager
let mut manager = RegistryManager::new();
// Add GitHub registry
manager.add_registry("https://npm.pkg.github.com", RegistryType::GitHub);
// Add authentication for GitHub registry
let auth = RegistryAuth {
    token: "ghp_token123".to_string(),
    token_type: "Bearer".to_string(),
    always: true,
};
manager.set_auth("https://npm.pkg.github.com", auth)?;
// Associate scope with registry
manager.associate_scope("@my-org", "https://npm.pkg.github.com")?;
// Get latest version of a package
let latest = manager.get_latest_version("@my-org/ui")?;
println!("Latest version: {}", latest.unwrap_or_else(|| "not found".to_string()));
// Load configuration from npmrc
manager.load_from_npmrc(Some("/path/to/.npmrc"))?;
```

### Upgrader Module

#### `Upgrader`

Finds and applies package upgrades.

```rust
pub struct Upgrader {
    registry_manager: RegistryManager,
    config: UpgradeConfig,
    cache: HashMap<String, Vec<String>>,
}
pub struct UpgradeConfig {
    /// Which types of dependencies to include
    pub dependency_types: DependencyFilter,
    /// Which types of version updates to include
    pub update_strategy: VersionUpdateStrategy,
    /// Whether to include prerelease versions
    pub version_stability: VersionStability,
    /// Specific packages to upgrade (if empty, upgrade all)
    pub target_packages: Vec<String>,
    /// Specific dependencies to upgrade (if empty, upgrade all)
    pub target_dependencies: Vec<String>,
    /// Additional registries to check for updates
    pub registries: Vec<String>,
    /// Whether to actually apply the upgrades or just report them
    pub execution_mode: ExecutionMode,
}
pub enum ExecutionMode {
    /// Only report potential upgrades without applying them
    DryRun,
    /// Apply upgrades to packages
    Apply,
}
pub enum VersionUpdateStrategy {
    /// Only upgrade patch versions (0.0.x)
    PatchOnly,
    /// Upgrade patch and minor versions (0.x.y)
    MinorAndPatch,
    /// Upgrade all versions including major ones (x.y.z)
    AllUpdates,
}
pub enum VersionStability {
    /// Only include stable versions
    StableOnly,
    /// Include prereleases and stable versions
    IncludePrerelease,
}
pub struct AvailableUpgrade {
    /// Package name containing the dependency
    pub package_name: String,
    /// Dependency name
    pub dependency_name: String,
    /// Current version of the dependency
    pub current_version: String,
    /// Latest available version that's compatible with requirements
    pub compatible_version: Option<String>,
    /// Latest overall version (may not be compatible with current requirements)
    pub latest_version: Option<String>,
    /// Status of this dependency's upgradability
    pub status: UpgradeStatus,
}
pub enum UpgradeStatus {
    /// Dependency is up to date
    UpToDate,
    /// Patch update available (0.0.x)
    PatchAvailable(String),
    /// Minor update available (0.x.0)
    MinorAvailable(String),
    /// Major update available (x.0.0)
    MajorAvailable(String),
    /// Version requirements don't allow update
    Constrained(String),
    /// Failed to check for updates
    CheckFailed(String),
}
impl Upgrader {
    pub fn new() -> Self;
    
    pub fn create(config: UpgradeConfig, registry_manager: RegistryManager) -> Self;
    
    pub fn with_config(config: UpgradeConfig) -> Self;
    
    pub fn with_registry_manager(registry_manager: RegistryManager) -> Self;
    
    pub fn check_dependency_upgrade(
        &mut self,
        package_name: &str,
        dependency: &Dependency,
    ) -> Result<AvailableUpgrade, PackageRegistryError>;
    
    pub fn check_package_upgrades(
        &mut self,
        package: &Package,
    ) -> Result<Vec<AvailableUpgrade>, PackageRegistryError>;
    
    pub fn check_all_upgrades(
        &mut self,
        packages: &[Package],
    ) -> Result<Vec<AvailableUpgrade>, PackageRegistryError>;
    
    pub fn apply_upgrades(
        &self,
        packages: &[Rc<RefCell<Package>>],
        upgrades: &[AvailableUpgrade],
    ) -> Result<Vec<AvailableUpgrade>, DependencyResolutionError>;
    
    pub fn generate_upgrade_report(upgrades: &[AvailableUpgrade]) -> String;
}
```

**Example Usage:**

```rust
use sublime_package_tools::{
    ExecutionMode, Package, RegistryManager, UpgradeConfig, 
    Upgrader, VersionUpdateStrategy
};
use std::cell::RefCell;
use std::rc::Rc;
// Create upgrader with custom configuration
let config = UpgradeConfig {
    update_strategy: VersionUpdateStrategy::MinorAndPatch,
    execution_mode: ExecutionMode::DryRun,
    ..UpgradeConfig::default()
};
let registry_manager = RegistryManager::new();
let mut upgrader = Upgrader::create(config, registry_manager);
// Check for available upgrades
let available_upgrades = upgrader.check_all_upgrades(&packages)?;
// Generate report
let report = Upgrader::generate_upgrade_report(&available_upgrades);
println!("{}", report);
// Convert packages for applying updates
let rc_packages: Vec<Rc<RefCell<Package>>> = 
    packages.iter()
            .map(|pkg| Rc::new(RefCell::new(pkg.clone())))
            .collect();
// Apply the upgrades (in dry-run mode this won't actually change anything)
let applied = upgrader.apply_upgrades(&rc_packages, &available_upgrades)?;
```

### Version Utilities

```rust
pub enum VersionRelationship {
    /// Second version is a major upgrade (1.0.0 -> 2.0.0)
    MajorUpgrade,
    /// Second version is a minor upgrade (1.0.0 -> 1.1.0)
    MinorUpgrade,
    /// Second version is a patch upgrade (1.0.0 -> 1.0.1)
    PatchUpgrade,
    /// Moved from prerelease to stable (1.0.0-alpha -> 1.0.0)
    PrereleaseToStable,
    /// Newer prerelease version (1.0.0-alpha -> 1.0.0-beta)
    NewerPrerelease,
    // ... and other relationships
}
pub enum Version {
    Major,
    Minor,
    Patch,
    Snapshot,
}
impl Version {
    /// Bumps the version of the package to major.
    pub fn bump_major(version: &str) -> Result<semver::Version, VersionError>;
    /// Bumps the version of the package to minor.
    pub fn bump_minor(version: &str) -> Result<semver::Version, VersionError>;
    /// Bumps the version of the package to patch.
    pub fn bump_patch(version: &str) -> Result<semver::Version, VersionError>;
    /// Bumps the version of the package to snapshot appending the sha to the version.
    pub fn bump_snapshot(version: &str, sha: &str) -> Result<semver::Version, VersionError>;
    /// Compare two version strings and return their relationship
    pub fn compare_versions(v1: &str, v2: &str) -> VersionRelationship;
    /// Check if moving from v1 to v2 is a breaking change according to semver
    pub fn is_breaking_change(v1: &str, v2: &str) -> bool;
    pub fn parse(version: &str) -> Result<semver::Version, VersionError>;
}
```

### Error Types

```rust
pub enum DependencyResolutionError {
    #[error("Failed to parse version: {0}")]
    VersionParseError(String),
    
    #[error("Incompatible version: {name}. Versions: {versions:?}. Requirements: {requirements:?}")]
    IncompatibleVersions { name: String, versions: Vec<String>, requirements: Vec<String> },
    
    #[error("No valid version found for {name} with requirements {requirements:?}")]
    NoValidVersion { name: String, requirements: Vec<String> },
    
    #[error("Dependency {name} not found in package {package}")]
    DependencyNotFound { name: String, package: String },
    
    #[error("Circular dependencies found: {path:?}")]
    CircularDependency { path: Vec<String> },
}
pub enum PackageError {
    #[error("Failed to parse package json: {path}")]
    PackageJsonParseFailure {
        path: String,
        #[source]
        error: serde_json::Error,
    },
    
    #[error("Failed to read/write package json: {path}")]
    PackageJsonIoFailure {
        path: String,
        #[source]
        error: io::Error,
    },
    
    #[error("Failed to diff package between: {0}")]
    PackageBetweenFailure(String),
    
    #[error("Failed to found package: {0}")]
    PackageNotFound(String),
}
pub enum VersionError {
    #[error("Failed to parse version: {message}")]
    Parse {
        #[source]
        error: semver::Error,
        message: String,
    },
    
    #[error("Invalid version: {0}")]
    InvalidVersion(String),
}
```

## Summary of Interactions Between Crates

The `sublime_package_tools` crate is designed as a standalone utility for package dependency management. Its key components interact as follows:

1. `Package` objects represent software packages with their dependencies

2. `DependencyGraph` provides analysis of package dependency relationships

3. `PackageRegistry` and its implementations like `NpmRegistry` provide access to package metadata

4. `Upgrader` uses the registry information to identify and apply package upgrades

5. Visualization tools like `generate_dot` and `generate_ascii` help visualize dependency graphs

The crate has no external dependencies on other workspace crates, making it a self-contained module for package management.

## Known Limitations

1. The current implementation is focused on Node.js-style package management, though the abstractions could support other ecosystems

2. Network operations in `NpmRegistry` are synchronous, which could impact performance for large-scale operations

3. Some graph traversal algorithms may not scale efficiently for very large dependency graphs

4. The upgrader implementation currently cannot fully simulate upgrade impacts before applying them

This API specification provides a comprehensive overview of the `sublime_package_tools` crate, including its core components, their interactions, and usage examples. The crate offers rich functionality for package dependency management, version control, and visualization.

