# sublime_pkg_tools API Specification

## Table of Contents

- [Overview](#overview)
- [Version Information](#version-information)
- [Config Module](#config-module)
  - [PackageToolsConfig](#packagetoolsconfig)
  - [ChangesetConfig](#changesetconfig)
  - [VersionConfig](#versionconfig)
  - [DependencyConfig](#dependencyconfig)
  - [UpgradeConfig](#upgradeconfig)
  - [ChangelogConfig](#changelogconfig)
  - [AuditConfig](#auditconfig)
  - [GitConfig](#gitconfig)
  - [Configuration Loader](#configuration-loader)
- [Types Module](#types-module)
  - [Version Types](#version-types)
  - [Package Types](#package-types)
  - [Changeset Types](#changeset-types)
  - [Dependency Types](#dependency-types)
  - [Common Traits](#common-traits)
  - [Type Aliases](#type-aliases)
- [Version Module](#version-module)
  - [VersionResolver](#versionresolver)
  - [DependencyGraph](#dependencygraph)
  - [DependencyPropagator](#dependencypropagator)
  - [SnapshotGenerator](#snapshotgenerator)
  - [Resolution Types](#resolution-types)
  - [Application Types](#application-types)
- [Changeset Module](#changeset-module)
  - [ChangesetManager](#changesetmanager)
  - [ChangesetStorage](#changesetstorage)
  - [FileBasedChangesetStorage](#filebasedchangesetstorage)
  - [ChangesetHistory](#changesethistory)
  - [PackageDetector](#packagedetector)
- [Changes Module](#changes-module)
  - [ChangesAnalyzer](#changesanalyzer)
  - [PackageMapper](#packagemapper)
  - [Report Types](#report-types)
- [Changelog Module](#changelog-module)
  - [ChangelogGenerator](#changeloggenerator)
  - [ChangelogCollector](#changelogcollector)
  - [ChangelogParser](#changelogparser)
  - [Formatters](#formatters)
  - [Conventional Commits](#conventional-commits)
  - [Merge Messages](#merge-messages)
- [Upgrade Module](#upgrade-module)
  - [UpgradeManager](#upgrademanager)
  - [RegistryClient](#registryclient)
  - [Detection Functions](#detection-functions)
  - [Application Functions](#application-functions)
  - [BackupManager](#backupmanager)
- [Audit Module](#audit-module)
  - [AuditManager](#auditmanager)
  - [Audit Functions](#audit-functions)
  - [Health Score](#health-score)
  - [Issue Types](#issue-types)
  - [Report Types](#report-types-1)
  - [Formatters](#formatters-1)
- [Error Module](#error-module)
  - [Error Types](#error-types)
  - [Result Types](#result-types)

## Overview

`sublime_pkg_tools` is a comprehensive package and version management toolkit for Node.js projects with changeset support. It provides a library-first approach to managing packages, versions, changesets, and dependencies in both single-package and monorepo configurations.

**Key Features:**
- Version resolution with independent and unified strategies
- Dependency propagation across workspace packages
- Changeset-based workflow management
- Changes analysis and package mapping
- Conventional commits and changelog generation
- External dependency upgrade detection and application
- Comprehensive audit and health scoring
- Full async/await support

**Core Philosophy:**
- Changeset as source of truth
- Library not CLI
- Simple, serializable data model
- No opinionated workflow enforcement

## Version Information

### `VERSION`

```rust
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
```

The version of the `sublime_pkg_tools` crate as defined in `Cargo.toml`.

### `version()`

```rust
pub fn version() -> &'static str
```

Returns the version of the `sublime_pkg_tools` crate.

**Returns:**
- A string slice containing the version number in semver format.

**Example:**
```rust
use sublime_pkg_tools::version;

let ver = version();
println!("sublime_pkg_tools version: {}", ver);
```

## Config Module

The `config` module provides comprehensive configuration management for all package tools functionality.

### PackageToolsConfig

Main configuration structure for package tools.

```rust
pub struct PackageToolsConfig {
    pub changeset: ChangesetConfig,
    pub version: VersionConfig,
    pub dependency: DependencyConfig,
    pub upgrade: UpgradeConfig,
    pub changelog: ChangelogConfig,
    pub audit: AuditConfig,
    pub git: GitConfig,
}
```

**Fields:**
- `changeset`: Changeset management configuration
- `version`: Version resolution configuration
- `dependency`: Dependency propagation configuration
- `upgrade`: Upgrade detection and application configuration
- `changelog`: Changelog generation configuration
- `audit`: Audit and health check configuration
- `git`: Git integration configuration

**Implements:**
- `Default`: Provides sensible defaults
- `Clone`: Can be cloned
- `Debug`: Debug formatting
- `Serialize`, `Deserialize`: Serialization support

### ChangesetConfig

Configuration for changeset management.

```rust
pub struct ChangesetConfig {
    pub path: String,
    pub history_path: String,
    pub available_environments: Vec<String>,
    pub default_environments: Vec<String>,
}
```

**Fields:**
- `path`: Path to store active changesets (default: `.changesets`)
- `history_path`: Path to store archived changesets (default: `.changesets/history`)
- `available_environments`: List of valid environment names
- `default_environments`: Default environments for new changesets

### VersionConfig

Configuration for version resolution.

```rust
pub struct VersionConfig {
    pub strategy: VersioningStrategy,
    pub default_bump: VersionBump,
    pub snapshot_format: String,
}
```

**Fields:**
- `strategy`: Versioning strategy (Independent or Unified)
- `default_bump`: Default version bump type (Patch, Minor, or Major)
- `snapshot_format`: Format template for snapshot versions

### DependencyConfig

Configuration for dependency propagation.

```rust
pub struct DependencyConfig {
    pub propagation_bump: VersionBump,
    pub propagate_dependencies: bool,
    pub propagate_dev_dependencies: bool,
    pub propagate_peer_dependencies: bool,
    pub max_depth: usize,
    pub fail_on_circular: bool,
    pub skip_workspace_protocol: bool,
    pub skip_file_protocol: bool,
    pub skip_link_protocol: bool,
    pub skip_portal_protocol: bool,
}
```

**Fields:**
- `propagation_bump`: Version bump type for dependency updates
- `propagate_dependencies`: Whether to propagate regular dependencies
- `propagate_dev_dependencies`: Whether to propagate dev dependencies
- `propagate_peer_dependencies`: Whether to propagate peer dependencies
- `max_depth`: Maximum propagation depth
- `fail_on_circular`: Whether to fail on circular dependencies
- `skip_workspace_protocol`: Skip workspace: protocol dependencies
- `skip_file_protocol`: Skip file: protocol dependencies
- `skip_link_protocol`: Skip link: protocol dependencies
- `skip_portal_protocol`: Skip portal: protocol dependencies

### UpgradeConfig

Configuration for upgrade detection and application.

```rust
pub struct UpgradeConfig {
    pub auto_changeset: bool,
    pub changeset_bump: VersionBump,
    pub registry: RegistryConfig,
    pub backup: BackupConfig,
}
```

**Fields:**
- `auto_changeset`: Automatically create changesets for upgrades
- `changeset_bump`: Version bump type for upgrade changesets
- `registry`: Registry configuration
- `backup`: Backup and rollback configuration

#### RegistryConfig

```rust
pub struct RegistryConfig {
    pub default_registry: String,
    pub scoped_registries: HashMap<String, String>,
    pub timeout_secs: u64,
    pub retry_attempts: usize,
    pub read_npmrc: bool,
}
```

**Fields:**
- `default_registry`: Default npm registry URL
- `scoped_registries`: Scoped package registries
- `timeout_secs`: Request timeout in seconds
- `retry_attempts`: Number of retry attempts
- `read_npmrc`: Whether to read .npmrc configuration

#### BackupConfig

```rust
pub struct BackupConfig {
    pub enabled: bool,
    pub path: String,
    pub keep_count: usize,
}
```

**Fields:**
- `enabled`: Whether backups are enabled
- `path`: Path to store backups
- `keep_count`: Number of backups to keep

### ChangelogConfig

Configuration for changelog generation.

```rust
pub struct ChangelogConfig {
    pub enabled: bool,
    pub format: ChangelogFormat,
    pub include_commit_links: bool,
    pub repository_url: Option<String>,
    pub conventional: ConventionalConfig,
    pub template: TemplateConfig,
    pub exclude: ExcludeConfig,
    pub monorepo_mode: MonorepoMode,
}
```

**Fields:**
- `enabled`: Whether changelog generation is enabled
- `format`: Changelog format (KeepAChangelog, ConventionalCommits, or Custom)
- `include_commit_links`: Include links to commits
- `repository_url`: Repository URL for links
- `conventional`: Conventional commits configuration
- `template`: Template configuration
- `exclude`: Exclusion patterns
- `monorepo_mode`: Monorepo changelog mode

#### ChangelogFormat

```rust
pub enum ChangelogFormat {
    KeepAChangelog,
    ConventionalCommits,
    Custom,
}
```

#### MonorepoMode

```rust
pub enum MonorepoMode {
    PerPackage,
    Root,
    Both,
}
```

### AuditConfig

Configuration for audit and health checks.

```rust
pub struct AuditConfig {
    pub enabled: bool,
    pub min_severity: IssueSeverity,
    pub sections: AuditSectionsConfig,
    pub health_score_weights: HealthScoreWeightsConfig,
}
```

**Fields:**
- `enabled`: Whether audits are enabled
- `min_severity`: Minimum severity to report
- `sections`: Configuration for audit sections
- `health_score_weights`: Weights for health score calculation

### GitConfig

Configuration for Git integration.

```rust
pub struct GitConfig {
    pub branch_base: String,
    pub detect_affected_packages: bool,
}
```

**Fields:**
- `branch_base`: Base branch for comparisons
- `detect_affected_packages`: Auto-detect affected packages from Git

### Configuration Loader

#### `load_config()`

```rust
pub async fn load_config(workspace_root: &Path) -> Result<PackageToolsConfig>
```

Loads package tools configuration from the workspace.

**Parameters:**
- `workspace_root`: Path to the workspace root directory

**Returns:**
- `Result<PackageToolsConfig>`: Loaded configuration or error

**Example:**
```rust
use sublime_pkg_tools::config::load_config;
use std::path::Path;

let config = load_config(Path::new(".")).await?;
println!("Changeset path: {}", config.changeset.path);
```

#### `load_config_from_file()`

```rust
pub async fn load_config_from_file(path: &Path) -> Result<PackageToolsConfig>
```

Loads configuration from a specific file.

**Parameters:**
- `path`: Path to the configuration file

**Returns:**
- `Result<PackageToolsConfig>`: Loaded configuration or error

#### `ConfigLoader`

```rust
pub struct ConfigLoader;

impl ConfigLoader {
    pub async fn load(workspace_root: &Path) -> Result<PackageToolsConfig>;
    pub async fn load_from_file(path: &Path) -> Result<PackageToolsConfig>;
    pub async fn load_with_defaults() -> PackageToolsConfig;
}
```

## Types Module

The `types` module provides fundamental data structures used throughout the package tools system.

### Version Types

#### `Version`

Represents a semantic version (major.minor.patch).

```rust
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}
```

**Methods:**

```rust
impl Version {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self;
    pub fn parse(s: &str) -> Result<Self>;
    pub fn bump(&self, bump: VersionBump) -> Result<Self>;
    pub fn to_string(&self) -> String;
    pub fn is_greater_than(&self, other: &Version) -> bool;
    pub fn is_compatible_with(&self, other: &Version) -> bool;
}
```

**Implements:**
- `Clone`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`
- `Display`: Formats as "major.minor.patch"
- `FromStr`: Parses from string
- `Serialize`, `Deserialize`

#### `VersionBump`

Version bump type.

```rust
pub enum VersionBump {
    Major,
    Minor,
    Patch,
    None,
}
```

**Methods:**

```rust
impl VersionBump {
    pub fn from_str(s: &str) -> Result<Self>;
    pub fn as_str(&self) -> &str;
}
```

#### `VersioningStrategy`

Versioning strategy for monorepos.

```rust
pub enum VersioningStrategy {
    Independent,
    Unified,
}
```

**Variants:**
- `Independent`: Each package has its own version
- `Unified`: All packages share the same version

### Package Types

#### `PackageInfo`

Information about a package.

```rust
pub struct PackageInfo {
    pub name: String,
    pub version: Version,
    pub path: PathBuf,
    pub package_json_path: PathBuf,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub peer_dependencies: HashMap<String, String>,
    pub optional_dependencies: HashMap<String, String>,
    pub is_workspace_package: bool,
}
```

**Methods:**

```rust
impl PackageInfo {
    pub async fn from_path(path: &Path, fs: &FileSystemManager) -> Result<Self>;
    pub async fn load_package_json(&self, fs: &FileSystemManager) -> Result<serde_json::Value>;
    pub fn all_dependencies(&self) -> HashMap<String, String>;
    pub fn has_dependency(&self, name: &str) -> bool;
    pub fn get_dependency_version(&self, name: &str) -> Option<&String>;
}
```

**Implements:**
- `Clone`, `Debug`
- `Named`: Provides `name()` method
- `Versionable`: Provides `version()` method
- `HasDependencies`: Provides dependency methods

#### `DependencyType`

Type of dependency.

```rust
pub enum DependencyType {
    Regular,
    Dev,
    Peer,
    Optional,
}
```

### Changeset Types

#### `Changeset`

The central data structure representing package changes.

```rust
pub struct Changeset {
    pub id: String,
    pub branch: String,
    pub bump: VersionBump,
    pub packages: Vec<String>,
    pub environments: Vec<String>,
    pub commits: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Methods:**

```rust
impl Changeset {
    pub fn new(branch: &str, bump: VersionBump, environments: Vec<String>) -> Self;
    pub fn add_package(&mut self, package: &str);
    pub fn add_commit(&mut self, commit: &str);
    pub fn remove_package(&mut self, package: &str);
    pub fn has_package(&self, package: &str) -> bool;
    pub fn update_bump(&mut self, bump: VersionBump);
    pub fn add_environment(&mut self, env: &str);
    pub fn remove_environment(&mut self, env: &str);
    pub fn validate(&self) -> Result<()>;
}
```

**Implements:**
- `Clone`, `Debug`
- `Serialize`, `Deserialize`
- `Identifiable`: Provides `id()` method

#### `ArchivedChangeset`

A changeset that has been released.

```rust
pub struct ArchivedChangeset {
    pub changeset: Changeset,
    pub release_info: ReleaseInfo,
}
```

**Methods:**

```rust
impl ArchivedChangeset {
    pub fn new(changeset: Changeset, release_info: ReleaseInfo) -> Self;
    pub fn changeset(&self) -> &Changeset;
    pub fn release_info(&self) -> &ReleaseInfo;
}
```

#### `ReleaseInfo`

Information about a release.

```rust
pub struct ReleaseInfo {
    pub released_at: DateTime<Utc>,
    pub released_by: String,
    pub release_commit: String,
    pub released_versions: HashMap<String, String>,
}
```

**Methods:**

```rust
impl ReleaseInfo {
    pub fn new(
        released_at: DateTime<Utc>,
        released_by: String,
        release_commit: String,
        released_versions: HashMap<String, String>,
    ) -> Self;
}
```

#### `UpdateSummary`

Summary of a changeset update operation.

```rust
pub struct UpdateSummary {
    pub commits_added: usize,
    pub new_packages: Vec<String>,
    pub existing_packages: Vec<String>,
}
```

### Dependency Types

#### `VersionProtocol`

Version specification protocol.

```rust
pub enum VersionProtocol {
    Workspace,
    File(String),
    Link(String),
    Portal(String),
    Semver(String),
}
```

**Functions:**

```rust
pub fn parse_protocol(version_spec: &str) -> VersionProtocol;
pub fn is_workspace_protocol(version_spec: &str) -> bool;
pub fn is_local_protocol(version_spec: &str) -> bool;
pub fn should_skip_protocol(version_spec: &str, config: &DependencyConfig) -> bool;
pub fn extract_protocol_path(version_spec: &str) -> Option<String>;
```

#### `LocalLinkType`

Type of local link.

```rust
pub enum LocalLinkType {
    File,
    Link,
    Portal,
}
```

#### `DependencyUpdate`

Represents a dependency version update.

```rust
pub struct DependencyUpdate {
    pub dependency_name: String,
    pub dependency_type: DependencyType,
    pub old_version_spec: String,
    pub new_version_spec: String,
    pub reason: UpdateReason,
}
```

#### `UpdateReason`

Reason for a dependency update.

```rust
pub enum UpdateReason {
    DirectChange,
    Propagation,
}
```

#### `CircularDependency`

Represents a circular dependency.

```rust
pub struct CircularDependency {
    pub cycle: Vec<String>,
}
```

### Common Traits

#### `Named`

```rust
pub trait Named {
    fn name(&self) -> &str;
}
```

#### `Versionable`

```rust
pub trait Versionable {
    fn version(&self) -> &Version;
}
```

#### `HasDependencies`

```rust
pub trait HasDependencies {
    fn dependencies(&self) -> &HashMap<String, String>;
    fn dev_dependencies(&self) -> &HashMap<String, String>;
    fn peer_dependencies(&self) -> &HashMap<String, String>;
    fn optional_dependencies(&self) -> &HashMap<String, String>;
}
```

#### `Identifiable`

```rust
pub trait Identifiable {
    fn id(&self) -> &str;
}
```

### Type Aliases

```rust
pub type PackageName = String;
pub type VersionSpec = String;
pub type CommitHash = String;
pub type BranchName = String;
```

### Prelude

```rust
pub mod prelude {
    pub use super::{
        Changeset, ArchivedChangeset, ReleaseInfo,
        Version, VersionBump, VersioningStrategy,
        PackageInfo, DependencyType,
        Named, Versionable, HasDependencies, Identifiable,
    };
}
```

## Version Module

The `version` module provides version resolution and dependency propagation.

### VersionResolver

Main version resolution orchestrator.

```rust
pub struct VersionResolver {
    // Private fields
}
```

**Methods:**

```rust
impl VersionResolver {
    pub async fn new(
        workspace_root: PathBuf,
        config: PackageToolsConfig,
    ) -> Result<Self>;
    
    pub async fn resolve_versions(
        &self,
        changeset: &Changeset,
    ) -> Result<VersionResolution>;
    
    pub async fn apply_versions(
        &self,
        changeset: &Changeset,
        dry_run: bool,
    ) -> Result<ApplyResult>;
    
    pub async fn preview_versions(
        &self,
        changeset: &Changeset,
    ) -> Result<VersionResolution>;
}
```

**Example:**
```rust
use sublime_pkg_tools::version::VersionResolver;
use sublime_pkg_tools::types::{Changeset, VersionBump};
use sublime_pkg_tools::config::PackageToolsConfig;
use std::path::PathBuf;

let workspace_root = PathBuf::from(".");
let config = PackageToolsConfig::default();

let resolver = VersionResolver::new(workspace_root, config).await?;

let mut changeset = Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
changeset.add_package("my-package");

let resolution = resolver.resolve_versions(&changeset).await?;
for update in &resolution.updates {
    println!("{}: {} -> {}", update.name, update.current_version, update.next_version);
}
```

### DependencyGraph

Dependency graph for analyzing package relationships.

```rust
pub struct DependencyGraph {
    // Private fields
}
```

**Methods:**

```rust
impl DependencyGraph {
    pub fn from_packages(packages: &[PackageInfo]) -> Result<Self>;
    pub fn dependents(&self, package_name: &str) -> Vec<&str>;
    pub fn dependencies(&self, package_name: &str) -> Vec<&str>;
    pub fn detect_cycles(&self) -> Vec<CircularDependency>;
    pub fn topological_sort(&self) -> Result<Vec<String>>;
}
```

### DependencyPropagator

Handles dependency propagation logic.

```rust
pub struct DependencyPropagator {
    // Private fields
}
```

**Methods:**

```rust
impl DependencyPropagator {
    pub fn new(config: DependencyConfig) -> Self;
    
    pub fn propagate(
        &self,
        graph: &DependencyGraph,
        initial_updates: &[PackageUpdate],
    ) -> Result<Vec<PackageUpdate>>;
}
```

### SnapshotGenerator

Generates snapshot versions for testing.

```rust
pub struct SnapshotGenerator {
    // Private fields
}
```

**Methods:**

```rust
impl SnapshotGenerator {
    pub fn new(format: &str) -> Result<Self>;
    pub fn generate(&self, context: &SnapshotContext) -> Result<String>;
}
```

#### `SnapshotContext`

```rust
pub struct SnapshotContext {
    pub version: Version,
    pub branch: String,
    pub commit: &'static str,
    pub timestamp: i64,
}
```

#### `SnapshotVariable`

```rust
pub enum SnapshotVariable {
    Version,
    Branch,
    Commit,
    ShortCommit,
    Timestamp,
}
```

### Resolution Types

#### `VersionResolution`

Result of version resolution.

```rust
pub struct VersionResolution {
    pub updates: Vec<PackageUpdate>,
    pub circular_dependencies: Vec<CircularDependency>,
}
```

#### `PackageUpdate`

Version update for a package.

```rust
pub struct PackageUpdate {
    pub name: String,
    pub path: PathBuf,
    pub current_version: Version,
    pub next_version: Version,
    pub bump: VersionBump,
    pub dependency_updates: Vec<DependencyUpdate>,
}
```

### Application Types

#### `ApplyResult`

Result of applying version updates.

```rust
pub struct ApplyResult {
    pub resolution: VersionResolution,
    pub summary: ApplySummary,
}
```

#### `ApplySummary`

```rust
pub struct ApplySummary {
    pub packages_updated: usize,
    pub dependencies_updated: usize,
    pub files_modified: Vec<PathBuf>,
}
```

## Changeset Module

The `changeset` module provides changeset management functionality.

### ChangesetManager

Main changeset management orchestrator.

```rust
pub struct ChangesetManager {
    // Private fields
}
```

**Methods:**

```rust
impl ChangesetManager {
    pub async fn new(
        workspace_root: PathBuf,
        fs: FileSystemManager,
        config: PackageToolsConfig,
    ) -> Result<Self>;
    
    pub async fn create(
        &self,
        branch: &str,
        bump: VersionBump,
        environments: Vec<String>,
    ) -> Result<Changeset>;
    
    pub async fn load(&self, branch: &str) -> Result<Changeset>;
    
    pub async fn update(&self, changeset: &Changeset) -> Result<()>;
    
    pub async fn delete(&self, branch: &str) -> Result<()>;
    
    pub async fn list_pending(&self) -> Result<Vec<String>>;
    
    pub async fn exists(&self, branch: &str) -> Result<bool>;
    
    pub async fn archive(
        &self,
        branch: &str,
        release_info: ReleaseInfo,
    ) -> Result<()>;
    
    pub async fn add_commits_from_git(
        &self,
        branch: &str,
    ) -> Result<UpdateSummary>;
}
```

**Example:**
```rust
use sublime_pkg_tools::changeset::ChangesetManager;
use sublime_pkg_tools::types::{VersionBump, ReleaseInfo};
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::PathBuf;
use std::collections::HashMap;
use chrono::Utc;

let workspace_root = PathBuf::from(".");
let fs = FileSystemManager::new();
let config = PackageToolsConfig::default();

let manager = ChangesetManager::new(workspace_root, fs, config).await?;

// Create changeset
let changeset = manager.create(
    "feature-branch",
    VersionBump::Minor,
    vec!["production".to_string()]
).await?;

// Load and update
let mut changeset = manager.load("feature-branch").await?;
changeset.add_package("my-package");
manager.update(&changeset).await?;

// Archive
let mut versions = HashMap::new();
versions.insert("my-package".to_string(), "1.2.0".to_string());

let release_info = ReleaseInfo::new(
    Utc::now(),
    "ci-bot".to_string(),
    "abc123".to_string(),
    versions,
);

manager.archive("feature-branch", release_info).await?;
```

### ChangesetStorage

Trait for changeset storage implementations.

```rust
#[async_trait]
pub trait ChangesetStorage: Send + Sync {
    async fn save(&self, changeset: &Changeset) -> Result<()>;
    async fn load(&self, branch: &str) -> Result<Changeset>;
    async fn exists(&self, branch: &str) -> Result<bool>;
    async fn delete(&self, branch: &str) -> Result<()>;
    async fn list_pending(&self) -> Result<Vec<String>>;
    async fn archive(&self, changeset: &Changeset, release_info: ReleaseInfo) -> Result<()>;
    async fn load_archived(&self, id: &str) -> Result<ArchivedChangeset>;
    async fn list_archived(&self) -> Result<Vec<String>>;
}
```

### FileBasedChangesetStorage

File-based implementation of changeset storage.

```rust
pub struct FileBasedChangesetStorage {
    // Private fields
}
```

**Methods:**

```rust
impl FileBasedChangesetStorage {
    pub fn new(
        workspace_root: PathBuf,
        changeset_path: PathBuf,
        history_path: PathBuf,
        fs: FileSystemManager,
    ) -> Self;
}
```

**Implements:**
- `ChangesetStorage`

### ChangesetHistory

Query interface for changeset history.

```rust
pub struct ChangesetHistory {
    // Private fields
}
```

**Methods:**

```rust
impl ChangesetHistory {
    pub fn new(storage: Box<dyn ChangesetStorage>) -> Self;
    
    pub async fn query_by_date(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<ArchivedChangeset>>;
    
    pub async fn query_by_package(
        &self,
        package: &str,
    ) -> Result<Vec<ArchivedChangeset>>;
    
    pub async fn query_by_environment(
        &self,
        environment: &str,
    ) -> Result<Vec<ArchivedChangeset>>;
    
    pub async fn query_by_bump(
        &self,
        bump: VersionBump,
    ) -> Result<Vec<ArchivedChangeset>>;
    
    pub async fn get_latest(&self, count: usize) -> Result<Vec<ArchivedChangeset>>;
}
```

### PackageDetector

Detects affected packages from Git changes.

```rust
pub struct PackageDetector {
    // Private fields
}
```

**Methods:**

```rust
impl PackageDetector {
    pub async fn new(
        workspace_root: PathBuf,
        fs: FileSystemManager,
    ) -> Result<Self>;
    
    pub async fn detect_from_commits(
        &self,
        repo: &Repo,
        commits: &[String],
    ) -> Result<Vec<String>>;
    
    pub async fn detect_from_branch(
        &self,
        repo: &Repo,
        branch: &str,
        base: &str,
    ) -> Result<Vec<String>>;
}
```

## Changes Module

The `changes` module provides changes analysis and package mapping.

### ChangesAnalyzer

Main changes analysis orchestrator.

```rust
pub struct ChangesAnalyzer {
    // Private fields
}
```

**Methods:**

```rust
impl ChangesAnalyzer {
    pub async fn new(
        workspace_root: PathBuf,
        repo: Repo,
        fs: FileSystemManager,
        config: PackageToolsConfig,
    ) -> Result<Self>;
    
    pub async fn analyze_working_directory(&self) -> Result<ChangesReport>;
    
    pub async fn analyze_commit_range(
        &self,
        from_ref: &str,
        to_ref: &str,
    ) -> Result<ChangesReport>;
    
    pub async fn analyze_with_versions(
        &self,
        from_ref: &str,
        to_ref: &str,
        changeset: &Changeset,
    ) -> Result<ChangesReport>;
}
```

**Example:**
```rust
use sublime_pkg_tools::changes::ChangesAnalyzer;
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::PathBuf;

let workspace_root = PathBuf::from(".");
let fs = FileSystemManager::new();
let config = PackageToolsConfig::default();
let git_repo = Repo::open(".")?;

let analyzer = ChangesAnalyzer::new(workspace_root, git_repo, fs, config).await?;

let changes = analyzer.analyze_working_directory().await?;
for package_change in changes.packages {
    println!("Package: {}", package_change.package_name());
    println!("  Files changed: {}", package_change.files.len());
}
```

### PackageMapper

Maps file paths to packages.

```rust
pub struct PackageMapper {
    // Private fields
}
```

**Methods:**

```rust
impl PackageMapper {
    pub async fn new(
        workspace_root: PathBuf,
        fs: FileSystemManager,
    ) -> Result<Self>;
    
    pub fn map_file_to_package(&self, file_path: &Path) -> Option<&PackageInfo>;
    
    pub fn map_files_to_packages(
        &self,
        file_paths: &[PathBuf],
    ) -> HashMap<String, Vec<PathBuf>>;
    
    pub fn get_package(&self, name: &str) -> Option<&PackageInfo>;
    
    pub fn all_packages(&self) -> &[PackageInfo];
}
```

### Report Types

#### `ChangesReport`

```rust
pub struct ChangesReport {
    pub packages: Vec<PackageChanges>,
    pub summary: ChangesSummary,
    pub mode: AnalysisMode,
}
```

#### `PackageChanges`

```rust
pub struct PackageChanges {
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<FileChange>,
    pub commits: Vec<CommitInfo>,
    pub has_changes: bool,
    pub current_version: Option<Version>,
    pub next_version: Option<Version>,
    pub stats: PackageChangeStats,
}
```

**Methods:**

```rust
impl PackageChanges {
    pub fn package_name(&self) -> &str;
    pub fn file_count(&self) -> usize;
    pub fn commit_count(&self) -> usize;
}
```

#### `FileChange`

```rust
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: FileChangeType,
    pub lines_added: usize,
    pub lines_deleted: usize,
}
```

#### `FileChangeType`

```rust
pub enum FileChangeType {
    Added,
    Modified,
    Deleted,
    Renamed { from: PathBuf },
    Copied { from: PathBuf },
}
```

#### `CommitInfo`

```rust
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: DateTime<Utc>,
}
```

#### `ChangesSummary`

```rust
pub struct ChangesSummary {
    pub total_packages: usize,
    pub packages_with_changes: usize,
    pub total_files_changed: usize,
    pub total_commits: usize,
}
```

#### `PackageChangeStats`

```rust
pub struct PackageChangeStats {
    pub files_added: usize,
    pub files_modified: usize,
    pub files_deleted: usize,
    pub lines_added: usize,
    pub lines_deleted: usize,
}
```

#### `AnalysisMode`

```rust
pub enum AnalysisMode {
    WorkingDirectory,
    CommitRange,
    WithVersions,
}
```

## Changelog Module

The `changelog` module provides changelog generation with multiple format support.

### ChangelogGenerator

Main changelog generation orchestrator.

```rust
pub struct ChangelogGenerator {
    // Private fields
}
```

**Methods:**

```rust
impl ChangelogGenerator {
    pub async fn new(
        workspace_root: PathBuf,
        repo: Repo,
        fs: FileSystemManager,
        config: PackageToolsConfig,
    ) -> Result<Self>;
    
    pub async fn generate_for_version(
        &self,
        package: &str,
        version: &str,
    ) -> Result<GeneratedChangelog>;
    
    pub async fn generate_for_changeset(
        &self,
        changeset: &Changeset,
    ) -> Result<Vec<GeneratedChangelog>>;
    
    pub async fn update_changelog(
        &self,
        package: &str,
        version: &str,
        dry_run: bool,
    ) -> Result<()>;
}
```

**Example:**
```rust
use sublime_pkg_tools::changelog::ChangelogGenerator;
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::PathBuf;

let workspace_root = PathBuf::from(".");
let fs = FileSystemManager::new();
let config = PackageToolsConfig::default();
let git_repo = Repo::open(".")?;

let generator = ChangelogGenerator::new(workspace_root, git_repo, fs, config).await?;

let changelog = generator.generate_for_version("my-package", "2.0.0").await?;
println!("{}", changelog.to_markdown());
```

### ChangelogCollector

Collects changelog data from Git commits.

```rust
pub struct ChangelogCollector {
    // Private fields
}
```

**Methods:**

```rust
impl ChangelogCollector {
    pub async fn new(
        workspace_root: PathBuf,
        repo: Repo,
        config: ChangelogConfig,
    ) -> Result<Self>;
    
    pub async fn collect_for_version(
        &self,
        package: &str,
        from_version: Option<&str>,
        to_version: &str,
    ) -> Result<Changelog>;
}
```

### ChangelogParser

Parses existing CHANGELOG.md files.

```rust
pub struct ChangelogParser {
    // Private fields
}
```

**Methods:**

```rust
impl ChangelogParser {
    pub fn new() -> Self;
    pub fn parse(&self, content: &str) -> Result<ParsedChangelog>;
    pub fn parse_file(&self, path: &Path) -> Result<ParsedChangelog>;
}
```

#### `ParsedChangelog`

```rust
pub struct ParsedChangelog {
    pub versions: Vec<ParsedVersion>,
}
```

#### `ParsedVersion`

```rust
pub struct ParsedVersion {
    pub version: String,
    pub date: Option<String>,
    pub sections: HashMap<String, Vec<String>>,
}
```

### Formatters

#### `KeepAChangelogFormatter`

```rust
pub struct KeepAChangelogFormatter;

impl KeepAChangelogFormatter {
    pub fn new() -> Self;
    pub fn format(&self, changelog: &Changelog) -> String;
}
```

#### `ConventionalCommitsFormatter`

```rust
pub struct ConventionalCommitsFormatter;

impl ConventionalCommitsFormatter {
    pub fn new() -> Self;
    pub fn format(&self, changelog: &Changelog) -> String;
}
```

#### `CustomTemplateFormatter`

```rust
pub struct CustomTemplateFormatter {
    // Private fields
}

impl CustomTemplateFormatter {
    pub fn new(template: String) -> Self;
    pub fn format(&self, changelog: &Changelog) -> String;
}
```

### Conventional Commits

#### `ConventionalCommit`

```rust
pub struct ConventionalCommit {
    pub commit_type: String,
    pub scope: Option<String>,
    pub description: String,
    pub body: Option<String>,
    pub footers: Vec<CommitFooter>,
    pub breaking: bool,
}
```

**Methods:**

```rust
impl ConventionalCommit {
    pub fn parse(message: &str) -> Result<Self>;
    pub fn is_breaking(&self) -> bool;
    pub fn section_type(&self) -> SectionType;
}
```

#### `CommitFooter`

```rust
pub struct CommitFooter {
    pub token: String,
    pub value: String,
}
```

#### `SectionType`

```rust
pub enum SectionType {
    Features,
    Fixes,
    Breaking,
    Other(String),
}
```

### Merge Messages

#### `generate_merge_commit_message()`

```rust
pub fn generate_merge_commit_message(
    context: &MergeMessageContext,
) -> Result<String>
```

Generates a merge commit message from changeset information.

#### `MergeMessageContext`

```rust
pub struct MergeMessageContext {
    pub changeset: Changeset,
    pub version_updates: Vec<PackageUpdate>,
    pub changelog_entries: HashMap<String, String>,
}
```

### Types

#### `Changelog`

```rust
pub struct Changelog {
    pub version: String,
    pub date: Option<String>,
    pub sections: Vec<ChangelogSection>,
    pub metadata: ChangelogMetadata,
}
```

#### `ChangelogSection`

```rust
pub struct ChangelogSection {
    pub title: String,
    pub entries: Vec<ChangelogEntry>,
}
```

#### `ChangelogEntry`

```rust
pub struct ChangelogEntry {
    pub message: String,
    pub commit_hash: Option<String>,
    pub author: Option<String>,
}
```

#### `ChangelogMetadata`

```rust
pub struct ChangelogMetadata {
    pub package_name: String,
    pub repository_url: Option<String>,
    pub compare_url: Option<String>,
}
```

#### `GeneratedChangelog`

```rust
pub struct GeneratedChangelog {
    pub package: String,
    pub changelog: Changelog,
    pub markdown: String,
}
```

**Methods:**

```rust
impl GeneratedChangelog {
    pub fn to_markdown(&self) -> &str;
}
```

#### `VersionTag`

```rust
pub struct VersionTag {
    pub tag: String,
    pub version: Version,
    pub commit_hash: String,
}
```

## Upgrade Module

The `upgrade` module provides dependency upgrade detection and application.

### UpgradeManager

Main upgrade management orchestrator.

```rust
pub struct UpgradeManager {
    // Private fields
}
```

**Methods:**

```rust
impl UpgradeManager {
    pub async fn new(
        workspace_root: PathBuf,
        fs: FileSystemManager,
        config: PackageToolsConfig,
    ) -> Result<Self>;
    
    pub async fn detect_upgrades(
        &self,
        options: DetectionOptions,
    ) -> Result<Vec<PackageUpgrades>>;
    
    pub async fn apply_upgrades(
        &self,
        selection: UpgradeSelection,
        dry_run: bool,
    ) -> Result<UpgradeResult>;
    
    pub async fn apply_with_changeset(
        &self,
        selection: UpgradeSelection,
        dry_run: bool,
        changeset_manager: Option<&ChangesetManager>,
    ) -> Result<UpgradeResult>;
    
    pub async fn rollback_last(&self) -> Result<()>;
}
```

**Example:**
```rust
use sublime_pkg_tools::upgrade::{UpgradeManager, DetectionOptions, UpgradeSelection};
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::PathBuf;

let workspace_root = PathBuf::from(".");
let fs = FileSystemManager::new();
let config = PackageToolsConfig::default();

let manager = UpgradeManager::new(workspace_root, fs, config).await?;

// Detect upgrades
let options = DetectionOptions::all();
let available = manager.detect_upgrades(options).await?;
println!("Found {} packages with upgrades", available.len());

// Apply patch upgrades
let selection = UpgradeSelection::patch_only();
let result = manager.apply_upgrades(selection, false).await?;
println!("Applied {} upgrades", result.applied.len());
```

### RegistryClient

Client for fetching package metadata from npm registries.

```rust
pub struct RegistryClient {
    // Private fields
}
```

**Methods:**

```rust
impl RegistryClient {
    pub fn new(config: RegistryConfig) -> Self;
    
    pub async fn get_package_metadata(
        &self,
        package_name: &str,
    ) -> Result<PackageMetadata>;
    
    pub async fn get_latest_version(
        &self,
        package_name: &str,
    ) -> Result<String>;
}
```

#### `PackageMetadata`

```rust
pub struct PackageMetadata {
    pub name: String,
    pub versions: HashMap<String, VersionInfo>,
    pub dist_tags: HashMap<String, String>,
    pub repository: Option<RepositoryInfo>,
    pub deprecated: Option<String>,
}
```

#### `VersionInfo`

```rust
pub struct VersionInfo {
    pub version: String,
    pub published_at: DateTime<Utc>,
    pub deprecated: Option<String>,
}
```

#### `RepositoryInfo`

```rust
pub struct RepositoryInfo {
    pub url: String,
    pub repository_type: String,
}
```

#### `UpgradeType`

```rust
pub enum UpgradeType {
    Major,
    Minor,
    Patch,
}
```

#### `NpmrcConfig`

```rust
pub mod npmrc {
    pub struct NpmrcConfig {
        pub registries: HashMap<String, String>,
        pub auth_tokens: HashMap<String, String>,
    }
    
    impl NpmrcConfig {
        pub fn parse_file(path: &Path) -> Result<Self>;
        pub fn parse(content: &str) -> Result<Self>;
    }
}
```

### Detection Functions

#### `detect_upgrades()`

```rust
pub async fn detect_upgrades(
    workspace_root: &Path,
    options: DetectionOptions,
    fs: &FileSystemManager,
) -> Result<Vec<PackageUpgrades>>
```

Detects available upgrades for packages in the workspace.

**Parameters:**
- `workspace_root`: Path to the workspace root
- `options`: Detection options
- `fs`: Filesystem manager

**Returns:**
- `Result<Vec<PackageUpgrades>>`: List of packages with available upgrades

#### `DetectionOptions`

```rust
pub struct DetectionOptions {
    pub include_major: bool,
    pub include_minor: bool,
    pub include_patch: bool,
    pub include_dev_dependencies: bool,
    pub include_peer_dependencies: bool,
    pub packages: Option<Vec<String>>,
}
```

**Methods:**

```rust
impl DetectionOptions {
    pub fn all() -> Self;
    pub fn patch_only() -> Self;
    pub fn minor_and_patch() -> Self;
    pub fn specific_packages(packages: Vec<String>) -> Self;
}
```

#### `PackageUpgrades`

```rust
pub struct PackageUpgrades {
    pub package_name: String,
    pub package_path: PathBuf,
    pub dependencies: Vec<DependencyUpgrade>,
}
```

#### `DependencyUpgrade`

```rust
pub struct DependencyUpgrade {
    pub name: String,
    pub current_version: String,
    pub latest_version: String,
    pub upgrade_type: UpgradeType,
    pub dependency_type: DependencyType,
}
```

#### `UpgradePreview`

```rust
pub struct UpgradePreview {
    pub total_upgrades: usize,
    pub major_upgrades: usize,
    pub minor_upgrades: usize,
    pub patch_upgrades: usize,
}
```

#### `UpgradeSummary`

```rust
pub struct UpgradeSummary {
    pub packages_analyzed: usize,
    pub upgrades_found: usize,
    pub preview: UpgradePreview,
}
```

### Application Functions

#### `apply_upgrades()`

```rust
pub async fn apply_upgrades(
    packages: Vec<PackageUpgrades>,
    selection: UpgradeSelection,
    dry_run: bool,
    fs: &FileSystemManager,
) -> Result<UpgradeResult>
```

Applies selected upgrades to packages.

**Parameters:**
- `packages`: Packages with available upgrades
- `selection`: Upgrade selection criteria
- `dry_run`: Whether to perform a dry run
- `fs`: Filesystem manager

**Returns:**
- `Result<UpgradeResult>`: Result of the upgrade operation

#### `apply_with_changeset()`

```rust
pub async fn apply_with_changeset(
    packages: Vec<PackageUpgrades>,
    selection: UpgradeSelection,
    dry_run: bool,
    workspace_root: &Path,
    config: &UpgradeConfig,
    changeset_manager: Option<&ChangesetManager>,
    fs: &FileSystemManager,
) -> Result<UpgradeResult>
```

Applies upgrades with automatic changeset creation.

#### `UpgradeSelection`

```rust
pub struct UpgradeSelection {
    pub major: bool,
    pub minor: bool,
    pub patch: bool,
    pub packages: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
}
```

**Methods:**

```rust
impl UpgradeSelection {
    pub fn all() -> Self;
    pub fn patch_only() -> Self;
    pub fn minor_and_patch() -> Self;
    pub fn packages(packages: Vec<String>) -> Self;
    pub fn dependencies(dependencies: Vec<String>) -> Self;
}
```

#### `UpgradeResult`

```rust
pub struct UpgradeResult {
    pub applied: Vec<AppliedUpgrade>,
    pub skipped: Vec<DependencyUpgrade>,
    pub failed: Vec<(DependencyUpgrade, String)>,
    pub summary: ApplySummary,
    pub changeset_id: Option<String>,
}
```

#### `AppliedUpgrade`

```rust
pub struct AppliedUpgrade {
    pub package_name: String,
    pub dependency_name: String,
    pub old_version: String,
    pub new_version: String,
    pub upgrade_type: UpgradeType,
}
```

#### `ApplySummary`

```rust
pub struct ApplySummary {
    pub total_applied: usize,
    pub total_skipped: usize,
    pub total_failed: usize,
    pub packages_modified: Vec<String>,
}
```

### BackupManager

Manages backups and rollback for upgrades.

```rust
pub struct BackupManager {
    // Private fields
}
```

**Methods:**

```rust
impl BackupManager {
    pub fn new(workspace_root: PathBuf, config: BackupConfig, fs: FileSystemManager) -> Self;
    
    pub async fn create_backup(&self) -> Result<String>;
    
    pub async fn restore_backup(&self, backup_id: &str) -> Result<()>;
    
    pub async fn list_backups(&self) -> Result<Vec<BackupMetadata>>;
    
    pub async fn cleanup_old_backups(&self) -> Result<usize>;
}
```

#### `BackupMetadata`

```rust
pub struct BackupMetadata {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub packages: Vec<String>,
    pub size_bytes: u64,
}
```

## Audit Module

The `audit` module provides comprehensive auditing and health scoring.

### AuditManager

Main audit orchestrator.

```rust
pub struct AuditManager {
    // Private fields
}
```

**Methods:**

```rust
impl AuditManager {
    pub async fn new(
        workspace_root: PathBuf,
        config: PackageToolsConfig,
    ) -> Result<Self>;
    
    pub async fn run_audit(&self) -> Result<AuditReport>;
    
    pub async fn run_section(
        &self,
        section: &str,
    ) -> Result<Box<dyn std::any::Any>>;
}
```

**Example:**
```rust
use sublime_pkg_tools::audit::AuditManager;
use sublime_pkg_tools::config::PackageToolsConfig;
use std::path::PathBuf;

let workspace_root = PathBuf::from(".");
let config = PackageToolsConfig::default();

let audit_manager = AuditManager::new(workspace_root, config).await?;
let audit_result = audit_manager.run_audit().await?;

println!("Health score: {:.2}", audit_result.summary.health_score);
println!("Total issues: {}", audit_result.summary.total_issues);
```

### Audit Functions

#### `audit_upgrades()`

```rust
pub async fn audit_upgrades(
    workspace_root: &Path,
    fs: &FileSystemManager,
    config: &AuditConfig,
) -> Result<UpgradeAuditSection>
```

Audits available package upgrades.

#### `audit_dependencies()`

```rust
pub async fn audit_dependencies(
    workspace_root: &Path,
    fs: &FileSystemManager,
    config: &AuditConfig,
) -> Result<DependencyAuditSection>
```

Audits dependency health and issues.

#### `audit_version_consistency()`

```rust
pub async fn audit_version_consistency(
    workspace_root: &Path,
    fs: &FileSystemManager,
    config: &AuditConfig,
) -> Result<VersionConsistencyAuditSection>
```

Audits version consistency across packages.

#### `audit_breaking_changes()`

```rust
pub async fn audit_breaking_changes(
    workspace_root: &Path,
    fs: &FileSystemManager,
    config: &AuditConfig,
) -> Result<BreakingChangesAuditSection>
```

Audits for potential breaking changes.

#### `categorize_dependencies()`

```rust
pub async fn categorize_dependencies(
    workspace_root: &Path,
    fs: &FileSystemManager,
) -> Result<DependencyCategorization>
```

Categorizes dependencies by type.

#### `generate_categorization_issues()`

```rust
pub fn generate_categorization_issues(
    categorization: &DependencyCategorization,
) -> Vec<AuditIssue>
```

Generates issues from dependency categorization.

### Health Score

#### `calculate_health_score()`

```rust
pub fn calculate_health_score(report: &AuditReport) -> f64
```

Calculates overall health score from audit report.

#### `calculate_health_score_detailed()`

```rust
pub fn calculate_health_score_detailed(
    report: &AuditReport,
    weights: &HealthScoreWeights,
) -> HealthScoreBreakdown
```

Calculates detailed health score breakdown.

#### `HealthScoreWeights`

```rust
pub struct HealthScoreWeights {
    pub upgrades_weight: f64,
    pub dependencies_weight: f64,
    pub version_consistency_weight: f64,
    pub breaking_changes_weight: f64,
}
```

**Methods:**

```rust
impl HealthScoreWeights {
    pub fn default() -> Self;
    pub fn balanced() -> Self;
}
```

#### `HealthScoreBreakdown`

```rust
pub struct HealthScoreBreakdown {
    pub overall_score: f64,
    pub upgrades_score: f64,
    pub dependencies_score: f64,
    pub version_consistency_score: f64,
    pub breaking_changes_score: f64,
}
```

#### `calculate_diminishing_factor()`

```rust
pub fn calculate_diminishing_factor(count: usize, severity_multiplier: f64) -> f64
```

Calculates diminishing factor for issue counts.

### Issue Types

#### `AuditIssue`

```rust
pub struct AuditIssue {
    pub category: IssueCategory,
    pub severity: IssueSeverity,
    pub title: String,
    pub description: String,
    pub affected_packages: Vec<String>,
}
```

#### `IssueCategory`

```rust
pub enum IssueCategory {
    Upgrade,
    Dependency,
    VersionConsistency,
    BreakingChange,
    Other(String),
}
```

#### `IssueSeverity`

```rust
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}
```

### Report Types

#### `AuditReport`

```rust
pub struct AuditReport {
    pub summary: AuditSummary,
    pub sections: AuditSections,
}
```

**Methods:**

```rust
impl AuditReport {
    pub fn total_issues(&self) -> usize;
    pub fn critical_issues(&self) -> Vec<&AuditIssue>;
    pub fn high_issues(&self) -> Vec<&AuditIssue>;
}
```

#### `AuditSummary`

```rust
pub struct AuditSummary {
    pub total_packages: usize,
    pub total_issues: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
    pub medium_issues: usize,
    pub low_issues: usize,
    pub info_issues: usize,
    pub health_score: f64,
}
```

#### `AuditSections`

```rust
pub struct AuditSections {
    pub upgrades: Option<UpgradeAuditSection>,
    pub dependencies: Option<DependencyAuditSection>,
    pub version_consistency: Option<VersionConsistencyAuditSection>,
    pub breaking_changes: Option<BreakingChangesAuditSection>,
}
```

#### `UpgradeAuditSection`

```rust
pub struct UpgradeAuditSection {
    pub total_upgrades: usize,
    pub major_upgrades: usize,
    pub minor_upgrades: usize,
    pub patch_upgrades: usize,
    pub issues: Vec<AuditIssue>,
}
```

#### `DependencyAuditSection`

```rust
pub struct DependencyAuditSection {
    pub total_dependencies: usize,
    pub circular_dependencies: Vec<Vec<String>>,
    pub missing_dependencies: Vec<String>,
    pub deprecated_packages: Vec<DeprecatedPackage>,
    pub categorization: DependencyCategorization,
    pub issues: Vec<AuditIssue>,
}
```

#### `VersionConsistencyAuditSection`

```rust
pub struct VersionConsistencyAuditSection {
    pub inconsistencies: Vec<VersionInconsistency>,
    pub conflicts: Vec<VersionConflict>,
    pub issues: Vec<AuditIssue>,
}
```

#### `BreakingChangesAuditSection`

```rust
pub struct BreakingChangesAuditSection {
    pub total_breaking_changes: usize,
    pub packages_with_breaking_changes: Vec<PackageBreakingChanges>,
    pub issues: Vec<AuditIssue>,
}
```

#### `DeprecatedPackage`

```rust
pub struct DeprecatedPackage {
    pub name: String,
    pub version: String,
    pub message: Option<String>,
    pub used_by: Vec<String>,
}
```

#### `VersionInconsistency`

```rust
pub struct VersionInconsistency {
    pub dependency_name: String,
    pub versions: Vec<VersionUsage>,
}
```

#### `VersionUsage`

```rust
pub struct VersionUsage {
    pub package_name: String,
    pub version_spec: String,
}
```

#### `VersionConflict`

```rust
pub struct VersionConflict {
    pub dependency_name: String,
    pub package1: String,
    pub version1: String,
    pub package2: String,
    pub version2: String,
}
```

#### `PackageBreakingChanges`

```rust
pub struct PackageBreakingChanges {
    pub package_name: String,
    pub changes: Vec<BreakingChange>,
}
```

#### `BreakingChange`

```rust
pub struct BreakingChange {
    pub description: String,
    pub source: BreakingChangeSource,
    pub from_version: String,
    pub to_version: String,
}
```

#### `BreakingChangeSource`

```rust
pub enum BreakingChangeSource {
    Commit(String),
    Changelog,
    SemverMajor,
}
```

#### `DependencyCategorization`

```rust
pub struct DependencyCategorization {
    pub internal: Vec<InternalPackage>,
    pub external: Vec<ExternalPackage>,
    pub workspace: Vec<WorkspaceLink>,
    pub local: Vec<LocalLink>,
    pub stats: CategorizationStats,
}
```

#### `InternalPackage`

```rust
pub struct InternalPackage {
    pub name: String,
    pub path: PathBuf,
    pub version: String,
}
```

#### `ExternalPackage`

```rust
pub struct ExternalPackage {
    pub name: String,
    pub version: String,
    pub used_by: Vec<String>,
}
```

#### `WorkspaceLink`

```rust
pub struct WorkspaceLink {
    pub name: String,
    pub path: PathBuf,
    pub version_spec: String,
}
```

#### `LocalLink`

```rust
pub struct LocalLink {
    pub name: String,
    pub path: String,
    pub link_type: LocalLinkType,
    pub used_by: Vec<String>,
}
```

#### `CategorizationStats`

```rust
pub struct CategorizationStats {
    pub total_internal: usize,
    pub total_external: usize,
    pub total_workspace: usize,
    pub total_local: usize,
}
```

### Formatters

#### `format_markdown()`

```rust
pub fn format_markdown(report: &AuditReport, options: FormatOptions) -> String
```

Formats audit report as Markdown.

#### `format_json()`

```rust
pub fn format_json(report: &AuditReport) -> Result<String>
```

Formats audit report as JSON.

#### `format_json_compact()`

```rust
pub fn format_json_compact(report: &AuditReport) -> Result<String>
```

Formats audit report as compact JSON.

#### `FormatOptions`

```rust
pub struct FormatOptions {
    pub verbosity: Verbosity,
    pub include_summary: bool,
    pub include_recommendations: bool,
}
```

#### `Verbosity`

```rust
pub enum Verbosity {
    Minimal,
    Normal,
    Detailed,
}
```

#### `AuditReportExt`

Extension trait for formatting audit reports.

```rust
pub trait AuditReportExt {
    fn to_markdown(&self, options: FormatOptions) -> String;
    fn to_json(&self) -> Result<String>;
    fn to_json_compact(&self) -> Result<String>;
}

impl AuditReportExt for AuditReport {
    // Implementation provided
}
```

## Error Module

The `error` module provides comprehensive error handling for all package tools operations.

### Error Types

#### `Error`

Main error type for package tools.

```rust
pub enum Error {
    Config(ConfigError),
    Version(VersionError),
    Changeset(ChangesetError),
    Changes(ChangesError),
    Changelog(ChangelogError),
    Upgrade(UpgradeError),
    Audit(AuditError),
    FileSystem(FileSystemError),
    Git(RepoError),
    IO(std::io::Error),
    Json(serde_json::Error),
}
```

**Methods:**

```rust
impl Error {
    pub fn is_transient(&self) -> bool;
    pub fn filesystem_error(err: FileSystemError) -> Self;
    pub fn git_error(err: RepoError) -> Self;
}
```

**Implements:**
- `Display`: Human-readable error messages
- `std::error::Error`: Standard error trait
- `From<sublime_standard_tools::error::FileSystemError>`
- `From<sublime_git_tools::RepoError>`

#### `ConfigError`

Configuration-related errors.

```rust
pub enum ConfigError {
    NotFound { path: PathBuf },
    ParseError { path: PathBuf, source: Box<dyn std::error::Error> },
    InvalidConfig { message: String },
    ValidationFailed { errors: Vec<String> },
    UnsupportedFormat { format: String },
    Io { source: std::io::Error },
    EnvVarError { var_name: String, reason: String },
    MergeConflict { field: String, reason: String },
}
```

#### `VersionError`

Version resolution errors.

```rust
pub enum VersionError {
    InvalidVersion { version: String },
    ParseError { input: String },
    CircularDependency { cycle: Vec<String> },
    DependencyNotFound { package: String, dependency: String },
    PropagationFailed { reason: String },
    SnapshotGenerationFailed { reason: String },
    ApplicationFailed { package: String, reason: String },
}
```

#### `ChangesetError`

Changeset management errors.

```rust
pub enum ChangesetError {
    NotFound { branch: String },
    AlreadyExists { branch: String },
    InvalidChangeset { reason: String },
    StorageError { operation: String, reason: String },
    GitIntegrationFailed { reason: String },
    ArchiveFailed { changeset_id: String, reason: String },
}
```

#### `ChangesError`

Changes analysis errors.

```rust
pub enum ChangesError {
    AnalysisFailed { reason: String },
    MappingFailed { file: PathBuf, reason: String },
    GitError { source: RepoError },
    PackageNotFound { path: PathBuf },
}
```

#### `ChangelogError`

Changelog generation errors.

```rust
pub enum ChangelogError {
    GenerationFailed { package: String, reason: String },
    ParseError { path: PathBuf, reason: String },
    FormatError { format: String, reason: String },
    ConventionalCommitParseError { commit: String, reason: String },
    GitError { source: RepoError },
}
```

#### `UpgradeError`

Upgrade detection and application errors.

```rust
pub enum UpgradeError {
    DetectionFailed { reason: String },
    RegistryError { package: String, reason: String },
    ApplicationFailed { package: String, dependency: String, reason: String },
    BackupFailed { reason: String },
    RollbackFailed { backup_id: String, reason: String },
    NpmrcParseError { path: PathBuf, reason: String },
}
```

#### `AuditError`

Audit and health check errors.

```rust
pub enum AuditError {
    AuditFailed { section: String, reason: String },
    HealthScoreCalculationFailed { reason: String },
    ReportGenerationFailed { reason: String },
}
```

### Result Types

Type aliases for results with specific error types.

```rust
pub type Result<T> = std::result::Result<T, Error>;
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;
pub type VersionResult<T> = std::result::Result<T, VersionError>;
pub type ChangesetResult<T> = std::result::Result<T, ChangesetError>;
pub type ChangesResult<T> = std::result::Result<T, ChangesError>;
pub type ChangelogResult<T> = std::result::Result<T, ChangelogError>;
pub type UpgradeResult<T> = std::result::Result<T, UpgradeError>;
pub type AuditResult<T> = std::result::Result<T, AuditError>;
```

### Context Extension Trait

The `context` submodule provides context extension for errors:

```rust
pub mod context {
    pub trait ErrorContext<T, E> {
        fn with_context<C>(self, context: C) -> Result<T>
        where
            C: std::fmt::Display + Send + Sync + 'static;
        
        fn with_context_f<C, F>(self, f: F) -> Result<T>
        where
            C: std::fmt::Display + Send + Sync + 'static,
            F: FnOnce() -> C;
    }
    
    impl<T, E> ErrorContext<T, E> for std::result::Result<T, E>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        // Implementation provided
    }
}
```

### Recovery Extension

The `recovery` submodule provides error recovery utilities:

```rust
pub mod recovery {
    pub trait ErrorRecovery<T> {
        fn or_recover<F>(self, f: F) -> Result<T>
        where
            F: FnOnce(Error) -> Result<T>;
    }
    
    impl<T> ErrorRecovery<T> for Result<T> {
        // Implementation provided
    }
}
```

## Examples

### Complete Workflow Example

```rust
use sublime_pkg_tools::{
    config::load_config,
    changeset::ChangesetManager,
    version::VersionResolver,
    changelog::ChangelogGenerator,
    types::{Changeset, VersionBump},
};
use sublime_standard_tools::filesystem::FileSystemManager;
use sublime_git_tools::Repo;
use std::path::PathBuf;

async fn complete_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = PathBuf::from(".");
    
    // Load configuration
    let config = load_config(&workspace_root).await?;
    
    // Initialize managers
    let fs = FileSystemManager::new();
    let git_repo = Repo::open(".")?;
    
    let changeset_manager = ChangesetManager::new(
        workspace_root.clone(),
        fs.clone(),
        config.clone(),
    ).await?;
    
    let version_resolver = VersionResolver::new(
        workspace_root.clone(),
        config.clone(),
    ).await?;
    
    let changelog_generator = ChangelogGenerator::new(
        workspace_root.clone(),
        git_repo,
        fs.clone(),
        config.clone(),
    ).await?;
    
    // Create changeset
    let changeset = changeset_manager.create(
        "feature/new-api",
        VersionBump::Minor,
        vec!["production".to_string()],
    ).await?;
    
    // Add commits from Git
    let summary = changeset_manager.add_commits_from_git("feature/new-api").await?;
    println!("Added {} commits affecting {} packages",
        summary.commits_added,
        summary.new_packages.len()
    );
    
    // Resolve versions
    let resolution = version_resolver.resolve_versions(&changeset).await?;
    for update in &resolution.updates {
        println!("{}: {} -> {}",
            update.name,
            update.current_version,
            update.next_version
        );
    }
    
    // Apply versions
    let result = version_resolver.apply_versions(&changeset, false).await?;
    println!("Updated {} packages", result.summary.packages_updated);
    
    // Generate changelogs
    let changelogs = changelog_generator.generate_for_changeset(&changeset).await?;
    for cl in changelogs {
        println!("Generated changelog for {}", cl.package);
    }
    
    Ok(())
}
```

### Upgrade Workflow Example

```rust
use sublime_pkg_tools::{
    upgrade::{UpgradeManager, DetectionOptions, UpgradeSelection},
    config::load_config,
};
use sublime_standard_tools::filesystem::FileSystemManager;
use std::path::PathBuf;

async fn upgrade_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = PathBuf::from(".");
    let config = load_config(&workspace_root).await?;
    let fs = FileSystemManager::new();
    
    let manager = UpgradeManager::new(
        workspace_root,
        fs,
        config,
    ).await?;
    
    // Detect all available upgrades
    let options = DetectionOptions::all();
    let available = manager.detect_upgrades(options).await?;
    
    println!("Found upgrades for {} packages", available.len());
    for pkg in &available {
        println!("  {}: {} dependencies can be upgraded",
            pkg.package_name,
            pkg.dependencies.len()
        );
    }
    
    // Apply only patch upgrades
    let selection = UpgradeSelection::patch_only();
    let result = manager.apply_upgrades(selection, false).await?;
    
    println!("Applied {} upgrades", result.applied.len());
    if !result.failed.is_empty() {
        println!("Failed to apply {} upgrades", result.failed.len());
    }
    
    Ok(())
}
```

### Audit Workflow Example

```rust
use sublime_pkg_tools::{
    audit::{AuditManager, format_markdown, FormatOptions, Verbosity},
    config::load_config,
};
use std::path::PathBuf;

async fn audit_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = PathBuf::from(".");
    let config = load_config(&workspace_root).await?;
    
    let manager = AuditManager::new(workspace_root, config).await?;
    
    // Run full audit
    let report = manager.run_audit().await?;
    
    println!("Health Score: {:.2}/100", report.summary.health_score);
    println!("Total Issues: {}", report.summary.total_issues);
    println!("  Critical: {}", report.summary.critical_issues);
    println!("  High: {}", report.summary.high_issues);
    println!("  Medium: {}", report.summary.medium_issues);
    
    // Format as markdown
    let options = FormatOptions {
        verbosity: Verbosity::Detailed,
        include_summary: true,
        include_recommendations: true,
    };
    let markdown = format_markdown(&report, options);
    println!("\n{}", markdown);
    
    Ok(())
}
```

## Version History

### Version 0.1.0 (Initial Release)

Initial release with core functionality:
- Configuration management
- Type system and data structures
- Version resolution and dependency propagation
- Changeset management and storage
- Changes analysis and package mapping
- Changelog generation with multiple formats
- Dependency upgrade detection and application
- Comprehensive audit and health scoring
- Full error handling and recovery
