# sublime_package_tools API Specification

This document provides a comprehensive overview of the `sublime_package_tools` crate, designed for managing Node.js packages, dependency graphs, and version handling in Rust.

## Table of Contents

- [Overview](#overview)
- [Package Management](#package-management)
  - [Package](#package)
  - [PackageInfo](#packageinfo)
  - [PackageDiff](#packagediff)
  - [CacheEntry](#cacheentry)
  - [Package Scoping](#package-scoping)
- [Dependency Management](#dependency-management)
  - [Dependency](#dependency)
  - [DependencyRegistry](#dependencyregistry)
  - [DependencyChange](#dependencychange)
  - [DependencyFilter](#dependencyfilter)
  - [DependencyUpdate](#dependencyupdate)
  - [ResolutionResult](#resolutionresult)
- [Dependency Graph](#dependency-graph)
  - [DependencyGraph](#dependencygraph)
  - [Node Trait](#node-trait)
  - [Step Enum](#step-enum)
  - [Validation](#validation)
  - [Graph Visualization](#graph-visualization)
  - [Graph Building](#graph-building)
- [Registry Management](#registry-management)
  - [PackageRegistry](#packageregistry)
  - [NpmRegistry](#npmregistry)
  - [LocalRegistry](#localregistry)
  - [RegistryManager](#registrymanager)
- [Upgrader](#upgrader)
  - [Upgrader](#upgrader-1)
  - [UpgradeConfig](#upgradeconfig)
  - [UpgradeStatus](#upgradestatus)
  - [AvailableUpgrade](#availableupgrade)
  - [ExecutionMode](#executionmode)
- [Version Management](#version-management)
  - [Version](#version)
  - [VersionUpdateStrategy](#versionupdatestrategy)
  - [VersionStability](#versionstability)
  - [VersionRelationship](#versionrelationship)
- [Error Types](#error-types)
  - [VersionError](#versionerror)
  - [PackageError](#packageerror)
  - [DependencyResolutionError](#dependencyresolutionerror)
  - [PackageRegistryError](#packageregistryerror)
  - [RegistryError](#registryerror)

## Overview

`sublime_package_tools` is a robust library for managing Node.js packages, dependency graphs, and version handling in Rust. Key features include:

- **Dependency Management**: Parse, validate, and manipulate package dependencies
- **Version Handling**: Semantic versioning utilities, compatibility checking, and upgrade strategies
- **Dependency Graph**: Build and visualize dependency graphs with cycle detection
- **Package Registry**: Interface with npm and other package registries
- **Upgrader**: Find and apply dependency upgrades with various strategies

## Package Management

### Package

Represents a Node.js package with name, version, and dependencies.

```rust
pub struct Package {
    // Private fields
}

impl Package {
    // Create a new package with dependencies
    pub fn new(
        name: &str,
        version: &str,
        dependencies: Option<Vec<Rc<RefCell<Dependency>>>>
    ) -> Result<Self, VersionError>;
    
    // Create a new package using a dependency registry
    pub fn new_with_registry(
        name: &str,
        version: &str,
        dependencies: Option<Vec<(&str, &str)>>,
        registry: &mut DependencyRegistry
    ) -> Result<Self, VersionError>;
    
    // Get the package name
    pub fn name(&self) -> &str;
    
    // Get the package version as a semver::Version
    pub fn version(&self) -> semver::Version;
    
    // Get the package version as a string
    pub fn version_str(&self) -> String;
    
    // Update the package version
    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError>;
    
    // Get package dependencies
    pub fn dependencies(&self) -> &[Rc<RefCell<Dependency>>];
    
    // Update a dependency version
    pub fn update_dependency_version(
        &self,
        dep_name: &str,
        new_version: &str
    ) -> Result<(), DependencyResolutionError>;
    
    // Add a dependency to the package
    pub fn add_dependency(&mut self, dependency: Rc<RefCell<Dependency>>);
    
    // Update package dependencies based on resolution result
    pub fn update_dependencies_from_resolution(
        &self,
        resolution: &ResolutionResult
    ) -> Result<Vec<(String, String, String)>, VersionError>;
}

impl Node for Package {
    type DependencyType = Dependency;
    type Identifier = String;
    
    // Implement Node trait methods for dependency graph operations
    fn dependencies(&self) -> Vec<&Self::DependencyType>;
    fn dependencies_vec(&self) -> Vec<Self::DependencyType>;
    fn matches(&self, dependency: &Self::DependencyType) -> bool;
    fn identifier(&self) -> Self::Identifier;
}
```

### PackageInfo

Bridges structured `Package` representation with raw package.json content.

```rust
pub struct PackageInfo {
    pub package: Rc<RefCell<Package>>,
    pub package_json_path: String,
    pub package_path: String,
    pub package_relative_path: String,
    pub pkg_json: Rc<RefCell<serde_json::Value>>,
}

impl PackageInfo {
    // Create a new package info
    pub fn new(
        package: Package,
        package_json_path: String,
        package_path: String,
        package_relative_path: String,
        pkg_json: serde_json::Value
    ) -> Self;
    
    // Update the package version
    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError>;
    
    // Update a dependency version
    pub fn update_dependency_version(
        &self,
        dep_name: &str,
        new_version: &str
    ) -> Result<(), DependencyResolutionError>;
    
    // Apply dependency resolution across all packages
    pub fn apply_dependency_resolution(
        &self,
        resolution: &ResolutionResult
    ) -> Result<(), VersionError>;
    
    // Write the package.json file to disk
    pub fn write_package_json(&self) -> Result<(), PackageError>;
}
```

### PackageDiff

Represents differences between two versions of a package.

```rust
pub struct PackageDiff {
    pub package_name: String,
    pub previous_version: String,
    pub current_version: String,
    pub dependency_changes: Vec<DependencyChange>,
    pub breaking_change: bool,
}

impl PackageDiff {
    // Generate a diff between two packages
    pub fn between(previous: &Package, current: &Package) -> Result<Self, PackageError>;
    
    // Count the number of breaking changes in dependencies
    pub fn count_breaking_changes(&self) -> usize;
    
    // Count the changes by type
    pub fn count_changes_by_type(&self) -> HashMap<ChangeType, usize>;
}

impl fmt::Display for PackageDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}
```

### CacheEntry

Generic cache entry with time-based expiration.

```rust
pub struct CacheEntry<T> {
    // Private fields
}

impl<T: Clone> CacheEntry<T> {
    // Creates a new cache entry with the current timestamp
    pub fn new(data: T) -> Self;
    
    // Checks if the cache entry is still valid (not expired)
    pub fn is_valid(&self, ttl: Duration) -> bool;
    
    // Gets a clone of the cached data
    pub fn get(&self) -> T;
}
```

### Package Scoping

Utilities for parsing and manipulating scoped package names.

```rust
pub struct PackageScopeMetadata {
    pub full: String,
    pub name: String,
    pub version: String,
    pub path: Option<String>,
}

// Parse package scope, name, and version from a string
pub fn package_scope_name_version(pkg_name: &str) -> Option<PackageScopeMetadata>;
```

## Dependency Management

### Dependency

Represents a package dependency with name and version requirements.

```rust
pub struct Dependency {
    // Private fields
}

impl Dependency {
    // Creates a new dependency with the given name and version requirements
    pub fn new(name: &str, version: &str) -> Result<Self, VersionError>;
    
    // Returns the name of the dependency
    pub fn name(&self) -> &str;
    
    // Returns the version requirement of the dependency
    pub fn version(&self) -> semver::VersionReq;
    
    // Extracts the fixed version from the version requirement
    pub fn fixed_version(&self) -> Result<semver::Version, VersionError>;
    
    // Compares the dependency's version with another version string
    pub fn compare_versions(&self, other: &str) -> Result<std::cmp::Ordering, VersionError>;
    
    // Updates the version requirement to a new value
    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError>;
    
    // Checks if a specific version matches this dependency's requirements
    pub fn matches(&self, version: &str) -> Result<bool, VersionError>;
}

impl std::fmt::Display for Dependency {
    // Formats a dependency as "name@version"
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
```

### DependencyRegistry

Central repository for managing dependencies.

```rust
pub struct DependencyRegistry {
    // Private fields
}

impl DependencyRegistry {
    // Creates a new, empty dependency registry
    pub fn new() -> Self;
    
    // Gets an existing dependency or creates a new one
    pub fn get_or_create(
        &mut self,
        name: &str,
        version: &str
    ) -> Result<Rc<RefCell<Dependency>>, VersionError>;
    
    // Gets an existing dependency by name
    pub fn get(&self, name: &str) -> Option<Rc<RefCell<Dependency>>>;
    
    // Resolve version conflicts between dependencies
    pub fn resolve_version_conflicts(&self) -> Result<ResolutionResult, VersionError>;
    
    // Find highest version that is compatible with all requirements
    pub fn find_highest_compatible_version(
        &self,
        name: &str,
        requirements: &[&VersionReq]
    ) -> String;
    
    // Apply the resolution result to update all dependencies
    pub fn apply_resolution_result(
        &mut self,
        result: &ResolutionResult
    ) -> Result<(), VersionError>;
}
```

### DependencyChange

Represents a change to a dependency between package versions.

```rust
pub struct DependencyChange {
    pub name: String,
    pub previous_version: Option<String>,
    pub current_version: Option<String>,
    pub change_type: ChangeType,
    pub breaking: bool,
}

impl DependencyChange {
    // Creates a new dependency change record
    pub fn new(
        name: &str,
        previous_version: Option<&str>,
        current_version: Option<&str>,
        change_type: ChangeType
    ) -> Self;
}
```

### DependencyFilter

Filter to control which types of dependencies are included in operations.

```rust
pub enum DependencyFilter {
    // Include only production dependencies
    ProductionOnly,
    // Include production and development dependencies
    WithDevelopment,
    // Include production, development, and optional dependencies
    AllDependencies,
}

impl Default for DependencyFilter {
    // Returns the default filter configuration (WithDevelopment)
    fn default() -> Self;
}
```

### DependencyUpdate

Represents a required update to a dependency.

```rust
pub struct DependencyUpdate {
    pub package_name: String,
    pub dependency_name: String,
    pub current_version: String,
    pub new_version: String,
}
```

### ResolutionResult

Result of a dependency resolution operation.

```rust
pub struct ResolutionResult {
    pub resolved_versions: HashMap<String, String>,
    pub updates_required: Vec<DependencyUpdate>,
}
```

## Dependency Graph

### DependencyGraph

A graph representation of dependencies between packages.

```rust
pub struct DependencyGraph<'a, N: Node> {
    pub graph: petgraph::stable_graph::StableDiGraph<Step<'a, N>, ()>,
    pub node_indices: HashMap<N::Identifier, petgraph::stable_graph::NodeIndex>,
    pub dependents: HashMap<N::Identifier, Vec<N::Identifier>>,
    pub cycles: Vec<Vec<N::Identifier>>,
}

impl<'a, N> From<&'a [N]> for DependencyGraph<'a, N> where N: Node {
    // Creates a dependency graph from a slice of nodes
    fn from(nodes: &'a [N]) -> Self;
}

impl<'a, N> Iterator for DependencyGraph<'a, N> where N: Node {
    type Item = Step<'a, N>;

    // Returns the next resolved node in topological order (leaf nodes first)
    fn next(&mut self) -> Option<Self::Item>;
}

impl<'a, N> DependencyGraph<'a, N> where N: Node {
    // Checks if all dependencies in the graph can be resolved internally
    pub fn is_internally_resolvable(&self) -> bool;
    
    // Returns an iterator over unresolved dependencies in the graph
    pub fn unresolved_dependencies(&self) -> impl Iterator<Item = &N::DependencyType>;
    
    // Returns an iterator over resolved nodes in the graph
    pub fn resolved_dependencies(&self) -> impl Iterator<Item = &N>;
    
    // Gets the graph index for a node with the given identifier
    pub fn get_node_index(&self, id: &N::Identifier) -> Option<petgraph::stable_graph::NodeIndex>;
    
    // Gets the node with the given identifier
    pub fn get_node(&self, id: &N::Identifier) -> Option<&Step<'a, N>>;
    
    // Detects circular dependencies in the graph
    pub fn detect_circular_dependencies(&self) -> &Self;
    
    // Checks if the graph has any circular dependencies
    pub fn has_cycles(&self) -> bool;
    
    // Returns information about the cycles in the graph
    pub fn get_cycles(&self) -> &Vec<Vec<N::Identifier>>;
    
    // Get the cycle information as strings for easier reporting
    pub fn get_cycle_strings(&self) -> Vec<Vec<String>>;
    
    // Find all external dependencies in the workspace
    pub fn find_external_dependencies(&self) -> Vec<String> where N: Node<DependencyType = Dependency>;
    
    // Find all version conflicts in the graph for Package nodes
    pub fn find_version_conflicts_for_package(&self) -> HashMap<String, Vec<String>> 
        where N: Node<DependencyType = Dependency>;
    
    // Find all version conflicts in the dependency graph
    pub fn find_version_conflicts(&self) -> Option<HashMap<String, Vec<String>>> 
        where N: Node<DependencyType = Dependency>;
    
    // Validates the dependency graph for Package nodes, checking for various issues
    pub fn validate_package_dependencies(&self) -> Result<ValidationReport, DependencyResolutionError> 
        where N: Node<DependencyType = Dependency>;
    
    // Get dependents of a node, even if cycles exist
    pub fn get_dependents(&mut self, id: &N::Identifier) -> Result<&Vec<N::Identifier>, PackageError>;
    
    // Check if dependencies can be upgraded to newer compatible versions
    pub fn check_upgradable_dependencies(&self) -> HashMap<String, Vec<(String, String)>> 
        where N: Node<DependencyType = Dependency>;
    
    // Validates the dependency graph for Package nodes with custom options
    pub fn validate_with_options(&self, options: &ValidationOptions) 
        -> Result<ValidationReport, DependencyResolutionError> 
        where N: Node<DependencyType = Dependency>;
}
```

### Node Trait

Trait for nodes in the dependency graph.

```rust
pub trait Node {
    // Type representing a dependency relationship
    type DependencyType: std::fmt::Debug + Clone;
    // Type used to uniquely identify a node
    type Identifier: std::hash::Hash + Eq + Clone + std::fmt::Debug + std::fmt::Display;
    
    // Returns a slice of dependencies for this Node
    fn dependencies(&self) -> Vec<&Self::DependencyType>;
    // Returns dependencies as owned values
    fn dependencies_vec(&self) -> Vec<Self::DependencyType>;
    // Returns true if the `dependency` can be met by this node
    fn matches(&self, dependency: &Self::DependencyType) -> bool;
    // Returns the unique identifier for this node
    fn identifier(&self) -> Self::Identifier;
}
```

### Step Enum

Wrapper around dependency graph nodes.

```rust
pub enum Step<'a, N: Node> {
    Resolved(&'a N),
    Unresolved(N::DependencyType),
}

impl<'a, N: Node> Step<'a, N> {
    pub fn is_resolved(&self) -> bool;
    pub fn as_resolved(&self) -> Option<&N>;
    pub fn as_unresolved(&self) -> Option<&N::DependencyType>;
}

impl<'a, N: Node> std::fmt::Display for Step<'a, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
```

### Validation

Types and functions for validating dependency graphs.

```rust
pub enum ValidationIssue {
    // Circular dependency detected
    CircularDependency { path: Vec<String> },
    // Unresolved dependency
    UnresolvedDependency { name: String, version_req: String },
    // Version conflict
    VersionConflict { name: String, versions: Vec<String> },
}

impl ValidationIssue {
    // Returns true if this is a critical issue that should be fixed
    pub fn is_critical(&self) -> bool;
    // Returns a descriptive message for this issue
    pub fn message(&self) -> String;
}

pub struct ValidationReport {
    // Private fields
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
    pub treat_unresolved_as_external: bool,
    pub internal_packages: Vec<String>,
}

impl ValidationOptions {
    // Create new validation options with default settings
    pub fn new() -> Self;
    // Treat unresolved dependencies as external (don't flag them as errors)
    pub fn treat_unresolved_as_external(self, value: bool) -> Self;
    // Set list of packages that should be considered internal
    pub fn with_internal_packages<I, S>(self, packages: I) -> Self
        where I: IntoIterator<Item = S>, S: Into<String>;
    // Check if a dependency should be treated as internal
    pub fn is_internal_dependency(&self, name: &str) -> bool;
}
```

### Graph Visualization

Functions for visualizing dependency graphs.

```rust
pub struct DotOptions {
    pub title: String,
    pub show_external: bool,
    pub highlight_cycles: bool,
}

impl Default for DotOptions {
    fn default() -> Self;
}

// Generate DOT format representation of a dependency graph
pub fn generate_dot<N: Node>(
    graph: &DependencyGraph<N>,
    options: &DotOptions
) -> Result<String, std::fmt::Error>;

// Save DOT output to a file
pub fn save_dot_to_file(dot_content: &str, file_path: &str) -> std::io::Result<()>;

// Generate an ASCII representation of the dependency graph
pub fn generate_ascii<N: Node>(graph: &DependencyGraph<N>) -> Result<String, std::fmt::Error>;
```

### Graph Building

Functions for building dependency graphs.

```rust
// Build a dependency graph from packages
pub fn build_dependency_graph_from_packages(packages: &[Package]) -> DependencyGraph<'_, Package>;

// Build a dependency graph from package infos
pub fn build_dependency_graph_from_package_infos<'a>(
    package_infos: &[PackageInfo],
    packages: &'a mut Vec<Package>
) -> DependencyGraph<'a, Package>;
```

## Registry Management

### PackageRegistry

Interface for package registry operations.

```rust
pub trait PackageRegistry {
    // Get the latest version of a package
    fn get_latest_version(&self, package_name: &str) -> Result<Option<String>, PackageRegistryError>;
    
    // Get all available versions of a package
    fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError>;
    
    // Get metadata about a package
    fn get_package_info(&self, package_name: &str, version: &str) -> Result<serde_json::Value, PackageRegistryError>;
    
    // Get the registry as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
    
    // Get the registry as mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
```

### NpmRegistry

NPM registry client implementation.

```rust
pub struct NpmRegistry {
    // Private fields
}

impl Default for NpmRegistry {
    fn default() -> Self;
}

impl PackageRegistry for NpmRegistry {
    // Implements the PackageRegistry trait
}

impl NpmRegistry {
    // Create a new npm registry client with the given base URL
    pub fn new(base_url: &str) -> Self;
    
    // Set the user agent string
    pub fn set_user_agent(&mut self, user_agent: &str) -> &mut Self;
    
    // Set authentication
    pub fn set_auth(&mut self, token: &str, auth_type: &str) -> &mut Self;
    
    // Set cache TTL
    pub fn set_cache_ttl(&mut self, ttl: Duration) -> &mut Self;
    
    // Clear all caches
    pub fn clear_cache(&mut self);
}
```

### LocalRegistry

In-memory package registry implementation.

```rust
pub struct LocalRegistry {
    // Private fields
}

impl Default for LocalRegistry {
    fn default() -> Self;
}

impl PackageRegistry for LocalRegistry {
    // Implements the PackageRegistry trait
}
```

### RegistryManager

Registry manager to handle multiple registries.

```rust
pub enum RegistryType {
    Npm,
    GitHub,
    Custom(String),
}

pub struct RegistryAuth {
    pub token: String,
    pub token_type: String,
    pub always: bool,
}

pub struct RegistryManager {
    // Private fields
}

impl Default for RegistryManager {
    fn default() -> Self;
}

impl RegistryManager {
    // Create a new registry manager with default npm registry
    pub fn new() -> Self;
    
    // Add a registry
    pub fn add_registry(&mut self, url: &str, registry_type: RegistryType) -> &Self;
    
    // Add a custom registry instance
    pub fn add_registry_instance(
        &mut self,
        url: &str,
        registry: Arc<dyn PackageRegistry + Send + Sync>
    ) -> &Self;
    
    // Set authentication for a registry
    pub fn set_auth(&mut self, registry_url: &str, auth: RegistryAuth) -> Result<&Self, RegistryError>;
    
    // Associate a scope with a specific registry
    pub fn associate_scope(&mut self, scope: &str, registry_url: &str) -> Result<&Self, RegistryError>;
    
    // Set the default registry
    pub fn set_default_registry(&mut self, registry_url: &str) -> Result<&Self, RegistryError>;
    
    // Get the appropriate registry for a package
    pub fn get_registry_for_package(&self, package_name: &str) -> Arc<dyn PackageRegistry + Send + Sync>;
    
    // Get the latest version of a package
    pub fn get_latest_version(&self, package_name: &str) -> Result<Option<String>, PackageRegistryError>;
    
    // Get all available versions of a package
    pub fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError>;
    
    // Get metadata about a package
    pub fn get_package_info(
        &self,
        package_name: &str,
        version: &str
    ) -> Result<serde_json::Value, PackageRegistryError>;
    
    // Load configuration from .npmrc file
    pub fn load_from_npmrc(&mut self, npmrc_path: Option<&str>) -> Result<&Self, RegistryError>;
    
    // Get the default registry URL
    pub fn default_registry(&self) -> &str;
    
    // Check if a scope is associated with a registry
    pub fn has_scope(&self, scope: &str) -> bool;
    
    // Get the registry URL associated with a scope
    pub fn get_registry_for_scope(&self, scope: &str) -> Option<&str>;
    
    // Get all registry URLs
    pub fn registry_urls(&self) -> Vec<&str>;
}
```

## Upgrader

### Upgrader

Package dependency upgrader.

```rust
pub struct Upgrader {
    // Private fields
}

impl Upgrader {
    // Create a new dependency upgrader with the given registry and default configuration
    pub fn new() -> Self;
    
    // Create an upgrader with custom configuration and registry manager
    pub fn create(config: UpgradeConfig, registry_manager: RegistryManager) -> Self;
    
    // Create a new dependency upgrader with the given configuration
    pub fn with_config(config: UpgradeConfig) -> Self;
    
    // Create with a specific registry manager
    pub fn with_registry_manager(registry_manager: RegistryManager) -> Self;
    
    // Get the registry manager
    pub fn registry_manager(&self) -> &RegistryManager;
    
    // Get a mutable reference to the registry manager
    pub fn registry_manager_mut(&mut self) -> &mut RegistryManager;
    
    // Set the configuration for the upgrader
    pub fn set_config(&mut self, config: UpgradeConfig);
    
    // Get the current configuration
    pub fn config(&self) -> &UpgradeConfig;
    
    // Check for upgrades for a single dependency
    pub fn check_dependency_upgrade(
        &mut self,
        package_name: &str,
        dependency: &Dependency
    ) -> Result<AvailableUpgrade, PackageRegistryError>;
    
    // Check all dependencies in a package for available upgrades
    pub fn check_package_upgrades(
        &mut self,
        package: &Package
    ) -> Result<Vec<AvailableUpgrade>, PackageRegistryError>;
    
    // Check all packages in a collection for available upgrades
    pub fn check_all_upgrades(
        &mut self,
        packages: &[Package]
    ) -> Result<Vec<AvailableUpgrade>, PackageRegistryError>;
    
    // Apply upgrades to packages based on what was found
    pub fn apply_upgrades(
        &self,
        packages: &[Rc<RefCell<Package>>],
        upgrades: &[AvailableUpgrade]
    ) -> Result<Vec<AvailableUpgrade>, DependencyResolutionError>;
    
    // Generate a report of upgrades in a human-readable format
    pub fn generate_upgrade_report(upgrades: &[AvailableUpgrade]) -> String;
}
```

### UpgradeConfig

Configuration for the dependency upgrader.

```rust
pub struct UpgradeConfig {
    pub dependency_types: DependencyFilter,
    pub update_strategy: VersionUpdateStrategy,
    pub version_stability: VersionStability,
    pub target_packages: Vec<String>,
    pub target_dependencies: Vec<String>,
    pub registries: Vec<String>,
    pub execution_mode: ExecutionMode,
}

impl From<&VersionUpdateStrategy> for UpgradeConfig {
    fn from(update_strategy: &VersionUpdateStrategy) -> Self;
}

impl Default for UpgradeConfig {
    fn default() -> Self;
}

impl UpgradeConfig {
    // Create a configuration with custom registries
    pub fn with_registries(registries: Vec<String>) -> Self;
}
```

### UpgradeStatus

Status of a dependency in relation to available updates.

```rust
pub enum UpgradeStatus {
    // Dependency is up to date
    UpToDate,
    // Patch update available (0.0.x)
    PatchAvailable(String),
    // Minor update available (0.x.0)
    MinorAvailable(String),
    // Major update available (x.0.0)
    MajorAvailable(String),
    // Version requirements don't allow update
    Constrained(String),
    // Failed to check for updates
    CheckFailed(String),
}

impl Default for UpgradeStatus {
    fn default() -> Self;
}

impl fmt::Display for UpgradeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}
```

### AvailableUpgrade

Represents an available upgrade for a dependency.

```rust
pub struct AvailableUpgrade {
    pub package_name: String,
    pub dependency_name: String,
    pub current_version: String,
    pub compatible_version: Option<String>,
    pub latest_version: Option<String>,
    pub status: UpgradeStatus,
}

impl fmt::Display for AvailableUpgrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}
```

### ExecutionMode

Determines whether upgrades should be applied or only reported.

```rust
pub enum ExecutionMode {
    // Only report potential upgrades without applying them
    DryRun,
    // Apply upgrades to packages
    Apply,
}

impl Default for ExecutionMode {
    fn default() -> Self;
}
```

## Version Management

### Version

Type of version bump to perform.

```rust
pub enum Version {
    Major,
    Minor,
    Patch,
    Snapshot,
}

impl From<&str> for Version {
    fn from(version: &str) -> Self;
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> FmtResult;
}

impl Version {
    // Bumps the version of the package to major
    pub fn bump_major(version: &str) -> Result<semver::Version, VersionError>;
    
    // Bumps the version of the package to minor
    pub fn bump_minor(version: &str) -> Result<semver::Version, VersionError>;
    
    // Bumps the version of the package to patch
    pub fn bump_patch(version: &str) -> Result<semver::Version, VersionError>;
    
    // Bumps the version of the package to snapshot appending the sha to the version
    pub fn bump_snapshot(version: &str, sha: &str) -> Result<semver::Version, VersionError>;
    
    // Compare two version strings and return their relationship
    pub fn compare_versions(v1: &str, v2: &str) -> VersionRelationship;
    
    // Check if moving from v1 to v2 is a breaking change according to semver
    pub fn is_breaking_change(v1: &str, v2: &str) -> bool;
    
    // Parse a version string into a semantic version
    pub fn parse(version: &str) -> Result<semver::Version, VersionError>;
}
```

### VersionUpdateStrategy

Controls what types of version updates are allowed when upgrading dependencies.

```rust
pub enum VersionUpdateStrategy {
    // Only upgrade patch versions (0.0.x)
    PatchOnly,
    // Upgrade patch and minor versions (0.x.y)
    MinorAndPatch,
    // Upgrade all versions including major ones (x.y.z)
    AllUpdates,
}

impl Default for VersionUpdateStrategy {
    fn default() -> Self;
}
```

### VersionStability

Controls whether prerelease versions are included in upgrades.

```rust
pub enum VersionStability {
    // Only include stable versions
    StableOnly,
    // Include prereleases and stable versions
    IncludePrerelease,
}

impl Default for VersionStability {
    fn default() -> Self;
}
```

### VersionRelationship

Describes how two versions relate to each other.

```rust
pub enum VersionRelationship {
    // Second version is a major upgrade (1.0.0 -> 2.0.0)
    MajorUpgrade,
    // Second version is a minor upgrade (1.0.0 -> 1.1.0)
    MinorUpgrade,
    // Second version is a patch upgrade (1.0.0 -> 1.0.1)
    PatchUpgrade,
    // Moved from prerelease to stable (1.0.0-alpha -> 1.0.0)
    PrereleaseToStable,
    // Newer prerelease version (1.0.0-alpha -> 1.0.0-beta)
    NewerPrerelease,
    // Versions are identical (1.0.0 == 1.0.0)
    Identical,
    // Second version is a major downgrade (2.0.0 -> 1.0.0)
    MajorDowngrade,
    // Second version is a minor downgrade (1.1.0 -> 1.0.0)
    MinorDowngrade,
    // Second version is a patch downgrade (1.0.1 -> 1.0.0)
    PatchDowngrade,
    // Moved from stable to prerelease (1.0.0 -> 1.0.0-alpha)
    StableToPrerelease,
    // Older prerelease version (1.0.0-beta -> 1.0.0-alpha)
    OlderPrerelease,
    // Version comparison couldn't be determined (invalid versions)
    Indeterminate,
}

impl Display for VersionRelationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> FmtResult;
}
```

## Error Types

### VersionError

Errors that can occur when working with semantic versions.

```rust
pub enum VersionError {
    // Indicates that a version string couldn't be parsed according to semver rules
    Parse {
        error: semver::Error,
        message: String,
    },
    // Indicates that a version is invalid for business logic reasons
    InvalidVersion(String),
}

impl From<semver::Error> for VersionError {
    fn from(error: semver::Error) -> Self;
}

impl Clone for VersionError {
    fn clone(&self) -> Self;
}

impl AsRef<str> for VersionError {
    fn as_ref(&self) -> &str;
}
```

### PackageError

Errors that can occur when working with packages.

```rust
pub enum PackageError {
    PackageJsonParseFailure {
        path: String,
        error: serde_json::Error,
    },
    PackageJsonIoFailure {
        path: String,
        error: io::Error,
    },
    PackageBetweenFailure(String),
    PackageNotFound(String),
}

impl From<serde_json::Error> for PackageError {
    fn from(error: serde_json::Error) -> Self;
}

impl From<io::Error> for PackageError {
    fn from(error: io::Error) -> Self;
}

impl AsRef<str> for PackageError {
    fn as_ref(&self) -> &str;
}

impl Clone for PackageError {
    fn clone(&self) -> Self;
}

impl PackageError {
    pub fn into_parse_error(error: serde_json::Error, path: String) -> PackageError;
    pub fn into_io_error(error: io::Error, path: String) -> PackageError;
}
```

### DependencyResolutionError

Errors that can occur during dependency resolution.

```rust
pub enum DependencyResolutionError {
    VersionParseError(String),
    IncompatibleVersions { name: String, versions: Vec<String>, requirements: Vec<String> },
    NoValidVersion { name: String, requirements: Vec<String> },
    DependencyNotFound { name: String, package: String },
    CircularDependency { path: Vec<String> },
}

impl AsRef<str> for DependencyResolutionError {
    fn as_ref(&self) -> &str;
}
```

### PackageRegistryError

Errors that can occur when interacting with package registries.

```rust
pub enum PackageRegistryError {
    FetchFailure(#[source] reqwest::Error),
    JsonParseFailure(#[source] reqwest::Error),
    NotFound { package_name: String, version: String },
    LockFailure,
}

impl AsRef<str> for PackageRegistryError {
    fn as_ref(&self) -> &str;
}

impl From<reqwest::Error> for PackageRegistryError {
    fn from(error: reqwest::Error) -> Self;
}

impl<T> From<PoisonError<T>> for PackageRegistryError {
    fn from(_: PoisonError<T>) -> Self;
}
```

### RegistryError

Errors that can occur when working with registries.

```rust
pub enum RegistryError {
    UrlNotSupported(String),
    UrlNotFound(String),
    NpmRcFailure {
        path: String,
        error: io::Error,
    },
}

impl From<io::Error> for RegistryError {
    fn from(error: io::Error) -> Self;
}

impl AsRef<str> for RegistryError {
    fn as_ref(&self) -> &str;
}

impl Clone for RegistryError {
    fn clone(&self) -> Self;
}
```

