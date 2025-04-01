# Api Specs

API Specification: sublime\_standard\_tools Crate

## Overview

`sublime_standard_tools` is a Rust crate that provides utilities for executing commands, detecting project roots, and working with JavaScript/TypeScript package managers in development workflows.

## Core Modules

### 1. Command Execution (`command.rs`)

#### Types

```rust
pub type ComandResult<T> = Result<T, CommandError>;
```

#### Functions

```rust
pub fn execute<P, I, F, S, R>(
    cmd: S,               // Command to execute
    path: P,              // Path to execute in
    args: I,              // Command arguments
    process: F            // Output processor function
) -> Result<R, CommandError>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    F: Fn(&str, &Output) -> ComandResult<R>;
```

### 2. Error Handling (`error.rs`)

#### Types

```rust
pub enum CommandError {
    Run(std::io::Error),          // IO error when running command
    Execution,                    // Error during command execution
    Failure { stdout: String, stderr: String }  // Command failed with non-zero exit code
}
```

#### Trait Implementations

```rust
impl Clone for CommandError;
impl AsRef<str> for CommandError;
impl Display for CommandError;  // Via thiserror
```

### 3. Package Manager Detection (`manager.rs`)

#### Types

```rust
pub enum CorePackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun
}

pub enum CorePackageManagerError {
    ParsePackageManagerError(String)  // Error with parsing package manager string
}
```

#### Functions

```rust
pub fn detect_package_manager(path: &Path) -> Option<CorePackageManager>;
```

#### Trait Implementations

```rust
impl TryFrom<&str> for CorePackageManager;
impl TryFrom<String> for CorePackageManager;
impl Display for CorePackageManager;
impl AsRef<str> for CorePackageManagerError;
impl Clone for CorePackageManagerError;
```

### 4. Project Path Detection (`path.rs`)

#### Functions

```rust
pub fn get_project_root_path(root: Option<PathBuf>) -> Option<PathBuf>;
```

### 5. String Utilities (`utils.rs`)

#### Functions

```rust
pub fn strip_trailing_newline(input: &String) -> String;
```

## Public API Summary

```rust
// Command execution
pub fn execute<P, I, F, S, R>(cmd: S, path: P, args: I, process: F) -> Result<R, CommandError>;
pub type ComandResult<T> = Result<T, CommandError>;

// Error handling
pub enum CommandError {
    Run(std::io::Error),
    Execution,
    Failure { stdout: String, stderr: String },
}

// Package manager detection
pub enum CorePackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}
pub enum CorePackageManagerError {
    ParsePackageManagerError(String),
}
pub fn detect_package_manager(path: &Path) -> Option<CorePackageManager>;

// Project path utilities
pub fn get_project_root_path(root: Option<PathBuf>) -> Option<PathBuf>;

// String utilities
pub fn strip_trailing_newline(input: &String) -> String;
```

## Usage Patterns

1. **Command Execution**:
   ```rust
   let result = execute("git", ".", ["status"], |stdout, output| {
       Ok(stdout.to_string())
   });
   ```

&#x20;  \`\`\`

1. **Package Manager Detection**:
   ```rust
   let project_dir = Path::new("path/to/project");
   if let Some(manager) = detect_package_manager(project_dir) {
       println!("Using {}", manager); // Prints "Using npm" or similar
   }
   ```

&#x20;  \`\`\`

1. **Project Root Detection**:
   ```rust
   let root_path = get_project_root_path(None).unwrap();
   ```

&#x20;  \`\`\`

## Notes

* The crate focuses on core utilities needed for JavaScript/TypeScript tooling
* Error handling is structured with thiserror for better diagnostics
* It provides package manager detection based on lock files (package-lock.json, yarn.lock, etc.)
* Command execution is abstracted with a powerful execute function that handles output processing

This API specification represents the current functionality of the `sublime_standard_tools` crate, which serves as foundational utilities that the new `sublime_package_tools` crate will build upon.





***

# API Specification: sublime\_git\_tools Crate

## Overview

`sublime_git_tools` is a Rust crate that provides a high-level abstraction over Git operations using libgit2. It offers a robust API for interacting with Git repositories, including common operations like creating and manipulating branches, committing changes, and tracking file status.

## Core Types

### `Repo`

The main struct representing a Git repository with high-level operation methods.

```rust
pub struct Repo {
    repo: Rc<Repository>,
    local_path: PathBuf
}
```

#### Creation Methods

```rust
impl Repo {
    // Creates a new Git repository at the specified path
    pub fn create(path: &str) -> Result<Self, RepoError>;
    
    // Opens an existing Git repository at the specified path
    pub fn open(path: &str) -> Result<Self, RepoError>;
    
    // Clones a Git repository from a URL to a local path
    pub fn clone(url: &str, path: &str) -> Result<Self, RepoError>;
}
```

#### Repository Management

```rust
impl Repo {
    // Gets the local path of the repository
    pub fn get_repo_path(&self) -> &Path;
    
    // Configures the repository with user information and core settings
    pub fn config(&self, username: &str, email: &str) -> Result<&Self, RepoError>;
    
    // Lists all configuration entries for the repository
    pub fn list_config(&self) -> Result<HashMap<String, String>, RepoError>;
}
```

#### Branch Operations

```rust
impl Repo {
    // Creates a new branch based on the current HEAD
    pub fn create_branch(&self, branch_name: &str) -> Result<&Self, RepoError>;
    
    // Lists all local branches in the repository
    pub fn list_branches(&self) -> Result<Vec<String>, RepoError>;
    
    // Checks out a local branch
    pub fn checkout(&self, branch_name: &str) -> Result<&Self, RepoError>;
    
    // Gets the name of the currently checked out branch
    pub fn get_current_branch(&self) -> Result<String, RepoError>;
    
    // Finds the branch that contains a specific commit
    pub fn get_branch_from_commit(&self, sha: &str) -> Result<Option<String>, RepoError>;
    
    // Finds all branches that contain a specific commit
    pub fn get_branches_containing_commit(&self, sha: &str) -> Result<Vec<String>, RepoError>;
}
```

#### Tag Operations

```rust
impl Repo {
    // Creates a new tag at the current HEAD
    pub fn create_tag(&self, tag: &str, message: Option<String>) -> Result<&Self, RepoError>;
    
    // Gets the name of the last tag in the repository
    pub fn get_last_tag(&self) -> Result<String, RepoError>;
    
    // Gets tags from either local repository or remote
    pub fn get_remote_or_local_tags(&self, local: Option<bool>) -> Result<Vec<RepoTags>, RepoError>;
}
```

#### Commit Operations

```rust
impl Repo {
    // Gets the SHA of the current HEAD commit
    pub fn get_current_sha(&self) -> Result<String, RepoError>;
    
    // Gets the SHA of the parent of the current HEAD commit
    pub fn get_previous_sha(&self) -> Result<String, RepoError>;
    
    // Creates a new commit with the current index
    pub fn commit(&self, message: &str) -> Result<String, RepoError>;
    
    // Adds all changes and creates a new commit
    pub fn commit_changes(&self, message: &str) -> Result<String, RepoError>;
    
    // Gets commits made since a specific reference or from the beginning
    pub fn get_commits_since(
        &self, 
        since: Option<String>, 
        relative: &Option<String>
    ) -> Result<Vec<RepoCommit>, RepoError>;
}
```

#### Index/Working Directory Operations

```rust
impl Repo {
    // Adds a file to the Git index
    pub fn add(&self, file_path: &str) -> Result<&Self, RepoError>;
    
    // Adds all changed files to the Git index
    pub fn add_all(&self) -> Result<&Self, RepoError>;
    
    // Gets the status of the repository in porcelain format
    pub fn status_porcelain(&self) -> Result<Vec<String>, RepoError>;
}
```

#### Remote Operations

```rust
impl Repo {
    // Pushes the current branch to a remote repository
    pub fn push(&self, remote_name: &str, follow_tags: Option<bool>) -> Result<bool, RepoError>;
    
    // Pushes the current branch to a remote repository with custom SSH key paths
    pub fn push_with_ssh_config(
        &self, 
        remote_name: &str, 
        follow_tags: Option<bool>, 
        ssh_key_paths: Vec<PathBuf>
    ) -> Result<bool, RepoError>;
    
    // Fetches changes from a remote repository
    pub fn fetch(
        &self, 
        remote_name: &str, 
        refspecs: Option<&[&str]>, 
        prune: bool
    ) -> Result<bool, RepoError>;
    
    // Pulls changes from a remote repository
    pub fn pull(
        &self, 
        remote_name: &str, 
        branch_name: Option<&str>
    ) -> Result<bool, RepoError>;
}
```

#### Diff/Change Operations

```rust
impl Repo {
    // Finds the common ancestor (merge base) between HEAD and another reference
    pub fn get_diverged_commit(&self, git_ref: &str) -> Result<String, RepoError>;
    
    // Gets all files changed since a specific reference
    pub fn get_all_files_changed_since_sha(&self, git_ref: &str) -> Result<Vec<String>, RepoError>;
    
    // Gets all files changed since a specific reference with their status
    pub fn get_all_files_changed_since_sha_with_status(
        &self, 
        git_ref: &str
    ) -> Result<Vec<GitChangedFile>, RepoError>;
    
    // Gets all files changed since a specific branch within specified package paths
    pub fn get_all_files_changed_since_branch(
        &self, 
        packages_paths: &[String], 
        branch: &str
    ) -> Result<Vec<String>, RepoError>;
}
```

### `RepoCommit`

Represents a commit in the Git repository.

```rust
pub struct RepoCommit {
    pub hash: String,           // The commit hash (SHA)
    pub author_name: String,    // The name of the commit author
    pub author_email: String,   // The email of the commit author
    pub author_date: String,    // The date of the commit in RFC2822 format
    pub message: String,        // The commit message
}
```

### `GitChangedFile`

Represents a changed file in the Git repository.

```rust
pub struct GitChangedFile {
    pub path: String,           // The path to the changed file
    pub status: GitFileStatus,  // The status of the file (Added, Modified, or Deleted)
}
```

### `GitFileStatus`

Represents the status of a file in Git.

```rust
pub enum GitFileStatus {
    Added,      // File has been added to the repository
    Modified,   // File has been modified
    Deleted,    // File has been deleted
}
```

### `RepoTags`

Represents a tag in the Git repository.

```rust
pub struct RepoTags {
    pub hash: String,   // The hash of the commit that the tag points to
    pub tag: String,    // The name of the tag
}
```

### `RepoError`

Errors that can occur when working with Git repositories.

```rust
pub enum RepoError {
    CanonicPathFailure(std::io::Error),       // Failed to canonicalize a path
    GitFailure(Git2Error),                    // Generic Git operation failure
    CreateRepoFailure(Git2Error),             // Failed to create a new repository
    OpenRepoFailure(Git2Error),               // Failed to open an existing repository
    CloneRepoFailure(Git2Error),              // Failed to clone a repository
    ConfigError(Git2Error),                   // Git configuration error
    ConfigEntriesError(Git2Error),            // Failed to retrieve configuration entries
    HeadError(Git2Error),                     // Failed to get repository HEAD
    PeelError(Git2Error),                     // Failed to peel a reference to a commit
    BranchError(Git2Error),                   // Failed to create or manipulate a branch
    SignatureError(Git2Error),                // Failed to get repository signature
    IndexError(Git2Error),                    // Failed to get or manipulate the index
    AddFilesError(Git2Error),                 // Failed to add files to the index
    WriteIndexError(Git2Error),               // Failed to write the index
    TreeError(Git2Error),                     // Failed to find or manipulate a tree
    CommitError(Git2Error),                   // Failed to create a commit
    WriteTreeError(Git2Error),                // Failed to write a tree
    BranchListError(Git2Error),               // Failed to list branches
    BranchNameError(Git2Error),               // Failed to get a branch name
    CheckoutBranchError(Git2Error),           // Failed to checkout a branch
    CheckoutError(Git2Error),                 // Failed to checkout
    LastTagError(Git2Error),                  // Failed to get the last tag
    CreateTagError(Git2Error),                // Failed to create a tag
    StatusError(Git2Error),                   // Failed to get repository status
    CommitOidError(Git2Error),                // Failed to parse a commit SHA
    GraphError(Git2Error),                    // Failed on repository graph operations
    PushError(Git2Error),                     // Failed to push to a remote
    RemoteError(Git2Error),                   // Failed on remote operations
    ReferenceError(Git2Error),                // Failed on reference parsing
    DiffError(Git2Error),                     // Failed on diff operations
    RevWalkError(Git2Error),                  // Failed on revision walking
    TagError(Git2Error),                      // Failed on tag operations
    MergeError(Git2Error),                    // Failed on merge operations
    MergeConflictError(Git2Error),            // Failed due to merge conflicts
}
```

## Common Usage Patterns

### Creating and Configuring a Repository

```rust
// Create a new repository
let repo = Repo::create("/path/to/new/repo").expect("Failed to create repository");
repo.config("John Doe", "john@example.com").expect("Failed to configure repository");

// Open an existing repository
let repo = Repo::open("/path/to/existing/repo").expect("Failed to open repository");
```

### Working with Branches

```rust
// List all branches
let branches = repo.list_branches().expect("Failed to list branches");
for branch in branches {
    println!("Branch: {}", branch);
}

// Create and checkout a branch
repo.create_branch("feature-branch").expect("Failed to create branch");
repo.checkout("feature-branch").expect("Failed to checkout branch");

// Get current branch
let current_branch = repo.get_current_branch().expect("Failed to get current branch");
println!("Current branch: {}", current_branch);
```

### Making Commits

```rust
// Add specific file and commit
repo.add("src/main.rs").expect("Failed to add file");
let commit_id = repo.commit("feat: update function").expect("Failed to commit");

// Add all changes and commit in one step
let commit_id = repo.commit_changes("feat: implement new feature").expect("Failed to commit changes");
```

### Working with Tags

```rust
// Create a tag
repo.create_tag("v1.0.0", Some("Release version 1.0.0".to_string())).expect("Failed to create tag");

// Get the most recent tag
let last_tag = repo.get_last_tag().expect("Failed to get last tag");

// Get all local tags
let tags = repo.get_remote_or_local_tags(Some(true)).expect("Failed to get tags");
for tag in tags {
    println!("Tag {} points to commit {}", tag.tag, tag.hash);
}
```

### Analyzing Changes

```rust
// Get files changed since a reference
let changed_files = repo.get_all_files_changed_since_sha("v1.0.0")
    .expect("Failed to get changed files");

// Get files changed with their status
let files_with_status = repo.get_all_files_changed_since_sha_with_status("HEAD~5")
    .expect("Failed to get changed files with status");
for file in files_with_status {
    println!("{}: {:?}", file.path, file.status);
}

// Get changes in specific packages since a branch
let package_paths = vec!["packages/pkg1".to_string(), "packages/pkg2".to_string()];
let changed_in_packages = repo.get_all_files_changed_since_branch(&package_paths, "main")
    .expect("Failed to get changes in packages");
```

### Working with Remotes

```rust
// Push to remote
repo.push("origin", Some(true)).expect("Failed to push with tags");

// Fetch updates
repo.fetch("origin", None, false).expect("Failed to fetch");

// Pull changes
repo.pull("origin", None).expect("Failed to pull");
```

### Getting Commit History

```rust
// Get all commits
let all_commits = repo.get_commits_since(None, &None).expect("Failed to get commits");

// Get commits since a tag
let commits = repo.get_commits_since(
    Some("v1.0.0".to_string()), 
    &None
).expect("Failed to get commits since tag");

// Get commits that touched a specific file
let file_commits = repo.get_commits_since(
    None,
    &Some("src/main.rs".to_string())
).expect("Failed to get commits for file");

for commit in commits {
    println!("{}: {} ({})", commit.hash, commit.message, commit.author_name);
}
```

## Notes

* The crate is built on libgit2 via the git2 crate, providing a safer and more Rust-idiomatic interface
* Support for SSH authentication with both default and custom key paths
* Error handling uses a comprehensive error type system to provide detailed information about failures
* Methods typically return `Result<T, RepoError>` to enable proper error handling
* Repository operations gracefully handle path canonicalization across platforms
* Methods allow method chaining with `&self` returns for builder-pattern style usage

This API specification represents the current functionality of the `sublime_git_tools` crate, which provides Git operations for version control integration in the larger application ecosystem.



***

# API Specification: sublime\_package\_tools Crate

## Overview

`sublime_package_tools` is a comprehensive Rust library for working with JavaScript/TypeScript package dependencies. It provides tools for analyzing, visualizing, and managing dependencies in monorepos and individual packages.

## Core Modules

### 1. Package Management

#### Package

```rust
pub struct Package {
    // Core package data 
    // (Implementation uses name: String, version: Rc<RefCell<Version>>, dependencies: Vec<Rc<RefCell<Dependency>>>)
}

impl Package {
    /// Create a new package with name, version, and optional dependencies
    pub fn new(name: &str, version: &str, 
               dependencies: Option<Vec<Rc<RefCell<Dependency>>>>) -> Result<Self, VersionError>;
    
    /// Create a new package using the dependency registry for dependency management
    pub fn new_with_registry(name: &str, version: &str, 
                          dependencies: Option<Vec<(&str, &str)>>,
                          registry: &mut DependencyRegistry) -> Result<Self, VersionError>;
    
    /// Get the package name
    pub fn name(&self) -> &str;
    
    /// Get the package version as a semver Version
    pub fn version(&self) -> semver::Version;
    
    /// Get the package version as a string
    pub fn version_str(&self) -> String;
    
    /// Update the package version
    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError>;
    
    /// Get the package dependencies
    pub fn dependencies(&self) -> &[Rc<RefCell<Dependency>>];
    
    /// Update a dependency version
    pub fn update_dependency_version(&self, dep_name: &str, new_version: &str) 
                                  -> Result<(), DependencyResolutionError>;
    
    /// Add a dependency to the package
    pub fn add_dependency(&mut self, dependency: Rc<RefCell<Dependency>>);
    
    /// Update package dependencies based on resolution result
    pub fn update_dependencies_from_resolution(&self, resolution: &ResolutionResult) 
                                            -> Result<Vec<(String, String, String)>, VersionError>;
}
```

#### PackageInfo

```rust
pub struct PackageInfo {
    pub package: Rc<RefCell<Package>>,
    pub package_json_path: String,
    pub package_path: String,
    pub package_relative_path: String,
    pub pkg_json: Rc<RefCell<Value>>,
}

impl PackageInfo {
    /// Create a new package info
    pub fn new(package: Package, package_json_path: String, 
               package_path: String, package_relative_path: String, 
               pkg_json: Value) -> Self;
    
    /// Update the package version
    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError>;
    
    /// Update a dependency version
    pub fn update_dependency_version(&self, dep_name: &str, new_version: &str) 
                                  -> Result<(), DependencyResolutionError>;
    
    /// Apply dependency resolution across all packages
    pub fn apply_dependency_resolution(&self, resolution: &ResolutionResult) 
                                    -> Result<(), VersionError>;
    
    /// Write the package.json file to disk
    pub fn write_package_json(&self) -> Result<(), PackageError>;
}
```

#### PackageDiff

```rust
pub struct PackageDiff {
    pub package_name: String,
    pub previous_version: String,
    pub current_version: String,
    pub dependency_changes: Vec<DependencyChange>,
    pub breaking_change: bool,
}

impl PackageDiff {
    /// Generate a diff between two packages
    pub fn between(previous: &Package, current: &Package) -> Result<Self, PackageError>;
    
    /// Count the number of breaking changes in dependencies
    pub fn count_breaking_changes(&self) -> usize;
    
    /// Count the changes by type
    pub fn count_changes_by_type(&self) -> HashMap<ChangeType, usize>;
}
```

### 2. Dependency Management

#### Dependency

```rust
pub struct Dependency {
    // Core dependency data
    // (Implementation uses name: String, version: Rc<RefCell<VersionReq>>)
}

impl Dependency {
    /// Create a new dependency
    pub fn new(name: &str, version: &str) -> Result<Self, VersionError>;
    
    /// Get the dependency name
    pub fn name(&self) -> &str;
    
    /// Get the version requirement
    pub fn version(&self) -> semver::VersionReq;
    
    /// Get the fixed version without range operators
    pub fn fixed_version(&self) -> Result<semver::Version, VersionError>;
    
    /// Compare versions
    pub fn compare_versions(&self, other: &str) -> Result<std::cmp::Ordering, VersionError>;
    
    /// Update the dependency version
    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError>;
    
    /// Check if a version matches this dependency's requirements
    pub fn matches(&self, version: &str) -> Result<bool, VersionError>;
}
```

#### DependencyChange

```rust
pub struct DependencyChange {
    pub name: String,
    pub previous_version: Option<String>,
    pub current_version: Option<String>,
    pub change_type: ChangeType,
    pub breaking: bool,
}

impl DependencyChange {
    /// Create a new dependency change
    pub fn new(name: &str, previous_version: Option<&str>, 
               current_version: Option<&str>, change_type: ChangeType) -> Self;
}
```

#### DependencyRegistry

```rust
pub struct DependencyRegistry {
    // Implementation contains HashMap<String, Rc<RefCell<Dependency>>>
}

impl DependencyRegistry {
    /// Create a new empty registry
    pub fn new() -> Self;
    
    /// Get or create a dependency in the registry
    pub fn get_or_create(&mut self, name: &str, version: &str) 
                      -> Result<Rc<RefCell<Dependency>>, VersionError>;
    
    /// Get a dependency from the registry
    pub fn get(&self, name: &str) -> Option<Rc<RefCell<Dependency>>>;
    
    /// Resolve version conflicts between dependencies
    pub fn resolve_version_conflicts(&self) -> Result<ResolutionResult, VersionError>;
    
    /// Find highest version that is compatible with all requirements
    pub fn find_highest_compatible_version(&self, name: &str, 
                                          requirements: &[&semver::VersionReq]) -> String;
    
    /// Apply the resolution result to update all dependencies
    pub fn apply_resolution_result(&mut self, result: &ResolutionResult) 
                                -> Result<(), VersionError>;
}
```

#### DependencyFilter

```rust
pub enum DependencyFilter {
    ProductionOnly,
    WithDevelopment,
    AllDependencies,
}
```

### 3. Dependency Graph and Analysis

#### DependencyGraph

```rust
pub struct DependencyGraph<'a, N: Node> {
    pub graph: petgraph::stable_graph::StableDiGraph<Step<'a, N>, ()>,
    pub node_indices: HashMap<N::Identifier, petgraph::stable_graph::NodeIndex>,
    pub dependents: HashMap<N::Identifier, Vec<N::Identifier>>,
}

impl<'a, N: Node> DependencyGraph<'a, N> {
    /// Check if all dependencies are internally resolvable
    pub fn is_internally_resolvable(&self) -> bool;
    
    /// Get unresolved dependencies
    pub fn unresolved_dependencies(&self) -> impl Iterator<Item = &N::DependencyType>;
    
    /// Get resolved dependencies
    pub fn resolved_dependencies(&self) -> impl Iterator<Item = &N>;
    
    /// Get a node by its identifier
    pub fn get_node(&self, id: &N::Identifier) -> Option<&Step<'a, N>>;
    
    /// Get dependents of a node
    pub fn get_dependents(&self, id: &N::Identifier) -> Result<&Vec<N::Identifier>, PackageError>;
    
    /// Check for circular dependencies
    pub fn detect_circular_dependencies(&self) -> Result<&Self, DependencyResolutionError>;
    
    /// Find all missing dependencies in the workspace
    pub fn find_missing_dependencies(&self) -> Vec<String> where N: Node<DependencyType = Dependency>;
    
    /// Find version conflicts in the graph
    pub fn find_version_conflicts(&self) -> Option<HashMap<String, Vec<String>>>
        where N: Node<DependencyType = Dependency>;
        
    /// Find all version conflicts in the graph with details
    pub fn find_version_conflicts_for_package(&self) -> HashMap<String, Vec<String>>
        where N: Node<DependencyType = Dependency>;
    
    /// Validate the dependency graph
    pub fn validate_package_dependencies(&self) -> Result<ValidationReport, DependencyResolutionError>
        where N: Node<DependencyType = Dependency>;
        
    /// Validate with custom options
    pub fn validate_with_options(&self, options: &ValidationOptions) 
                              -> Result<ValidationReport, DependencyResolutionError>
        where N: Node<DependencyType = Dependency>;
}
```

#### Node Trait

```rust
pub trait Node {
    type DependencyType: std::fmt::Debug + Clone;
    type Identifier: std::hash::Hash + Eq + Clone + std::fmt::Debug + std::fmt::Display;
    
    fn dependencies(&self) -> Vec<&Self::DependencyType>;
    fn dependencies_vec(&self) -> Vec<Self::DependencyType>;
    fn matches(&self, dependency: &Self::DependencyType) -> bool;
    fn identifier(&self) -> Self::Identifier;
}
```

#### Step Enum

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
```

#### ValidationReport

```rust
pub struct ValidationReport {
    // Implementation contains Vec<ValidationIssue>
}

impl ValidationReport {
    /// Create a new validation report
    pub fn new() -> Self;
    
    /// Add an issue to the report
    pub fn add_issue(&mut self, issue: ValidationIssue);
    
    /// Check if there are any issues
    pub fn has_issues(&self) -> bool;
    
    /// Get all issues
    pub fn issues(&self) -> &[ValidationIssue];
    
    /// Check if there are any critical issues
    pub fn has_critical_issues(&self) -> bool;
    
    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool;
    
    /// Get critical issues
    pub fn critical_issues(&self) -> Vec<&ValidationIssue>;
    
    /// Get warnings
    pub fn warnings(&self) -> Vec<&ValidationIssue>;
}
```

#### ValidationIssue

```rust
pub enum ValidationIssue {
    CircularDependency { path: Vec<String> },
    UnresolvedDependency { name: String, version_req: String },
    VersionConflict { name: String, versions: Vec<String> },
}

impl ValidationIssue {
    /// Check if the issue is critical
    pub fn is_critical(&self) -> bool;
    
    /// Get a descriptive message for this issue
    pub fn message(&self) -> String;
}
```

#### ValidationOptions

```rust
pub struct ValidationOptions {
    pub treat_unresolved_as_external: bool,
    pub internal_packages: Vec<String>,
}

impl ValidationOptions {
    /// Create new validation options with default settings
    pub fn new() -> Self;
    
    /// Treat unresolved dependencies as external
    pub fn treat_unresolved_as_external(self, value: bool) -> Self;
    
    /// Set list of packages that should be considered internal
    pub fn with_internal_packages<I, S>(self, packages: I) -> Self 
        where I: IntoIterator<Item = S>, S: Into<String>;
    
    /// Check if a dependency should be treated as internal
    pub fn is_internal_dependency(&self, name: &str) -> bool;
}
```

### 4. Graph Visualization

```rust
/// Options for generating DOT graph output
pub struct DotOptions {
    pub title: String,
    pub show_external: bool,
    pub highlight_cycles: bool,
}

/// Generate DOT format representation of a dependency graph
pub fn generate_dot<N: Node>(graph: &DependencyGraph<N>, options: &DotOptions) -> Result<String, std::fmt::Error>;

/// Save DOT output to a file
pub fn save_dot_to_file(dot_content: &str, file_path: &str) -> std::io::Result<()>;

/// Generate an ASCII representation of the dependency graph
pub fn generate_ascii<N: Node>(graph: &DependencyGraph<N>) -> Result<String, std::fmt::Error>;
```

### 5. Registry Management

#### PackageRegistry

```rust
pub trait PackageRegistry {
    /// Get the latest version of a package
    fn get_latest_version(&self, package_name: &str) -> Result<Option<String>, PackageRegistryError>;

    /// Get all available versions of a package
    fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError>;

    /// Get metadata about a package
    fn get_package_info(&self, package_name: &str, version: &str) 
                    -> Result<serde_json::Value, PackageRegistryError>;

    /// Get the registry as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get the registry as mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

#### NpmRegistry

```rust
pub struct NpmRegistry {
    // Implementation contains client configuration and caches
}

impl NpmRegistry {
    /// Create a new npm registry client with the given base URL
    pub fn new(base_url: &str) -> Self;
    
    /// Set the user agent string
    pub fn set_user_agent(&mut self, user_agent: &str) -> &mut Self;
    
    /// Set authentication
    pub fn set_auth(&mut self, token: &str, auth_type: &str) -> &mut Self;
    
    /// Set cache TTL
    pub fn set_cache_ttl(&mut self, ttl: std::time::Duration) -> &mut Self;
    
    /// Clear all caches
    pub fn clear_cache(&mut self);
}

impl PackageRegistry for NpmRegistry {
    // Implementation of PackageRegistry trait
}
```

#### LocalRegistry

```rust
pub struct LocalRegistry {
    // Implementation contains local package cache
}

impl Default for LocalRegistry {
    fn default() -> Self;
}

impl PackageRegistry for LocalRegistry {
    // Implementation of PackageRegistry trait
}
```

#### RegistryManager

```rust
pub struct RegistryManager {
    // Implementation contains registries, scopes, and configuration
}

impl RegistryManager {
    /// Create a new registry manager with default npm registry
    pub fn new() -> Self;
    
    /// Add a registry
    pub fn add_registry(&mut self, url: &str, registry_type: RegistryType) -> &Self;
    
    /// Add a registry instance
    pub fn add_registry_instance(&mut self, url: &str, 
                             registry: Arc<dyn PackageRegistry + Send + Sync>) -> &Self;
    
    /// Set authentication for a registry
    pub fn set_auth(&mut self, registry_url: &str, auth: RegistryAuth) -> Result<&Self, RegistryError>;
    
    /// Associate a scope with a specific registry
    pub fn associate_scope(&mut self, scope: &str, registry_url: &str) -> Result<&Self, RegistryError>;
    
    /// Set the default registry
    pub fn set_default_registry(&mut self, registry_url: &str) -> Result<&Self, RegistryError>;
    
    /// Get the appropriate registry for a package
    pub fn get_registry_for_package(&self, package_name: &str) 
                                -> Arc<dyn PackageRegistry + Send + Sync>;
    
    /// Get the latest version of a package
    pub fn get_latest_version(&self, package_name: &str) -> Result<Option<String>, PackageRegistryError>;
    
    /// Get all available versions of a package
    pub fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError>;
    
    /// Get metadata about a package
    pub fn get_package_info(&self, package_name: &str, version: &str) 
                        -> Result<serde_json::Value, PackageRegistryError>;
    
    /// Load configuration from .npmrc file
    pub fn load_from_npmrc(&mut self, npmrc_path: Option<&str>) -> Result<&Self, RegistryError>;
    
    /// Get the default registry URL
    pub fn default_registry(&self) -> &str;
    
    /// Check if a scope is associated with a registry
    pub fn has_scope(&self, scope: &str) -> bool;
    
    /// Get the registry URL associated with a scope
    pub fn get_registry_for_scope(&self, scope: &str) -> Option<&str>;
    
    /// Get all registry URLs
    pub fn registry_urls(&self) -> Vec<&str>;
}
```

#### Registry Types and Auth

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
```

### 6. Version Handling

#### Version Utilities

```rust
pub enum Version {
    Major,
    Minor,
    Patch,
    Snapshot,
}

impl Version {
    /// Bumps the version of the package to major
    pub fn bump_major(version: &str) -> Result<semver::Version, VersionError>;
    
    /// Bumps the version of the package to minor
    pub fn bump_minor(version: &str) -> Result<semver::Version, VersionError>;
    
    /// Bumps the version of the package to patch
    pub fn bump_patch(version: &str) -> Result<semver::Version, VersionError>;
    
    /// Bumps the version of the package to snapshot appending the sha
    pub fn bump_snapshot(version: &str, sha: &str) -> Result<semver::Version, VersionError>;
    
    /// Compare two version strings and return their relationship
    pub fn compare_versions(v1: &str, v2: &str) -> VersionRelationship;
    
    /// Check if moving from v1 to v2 is a breaking change according to semver
    pub fn is_breaking_change(v1: &str, v2: &str) -> bool;
    
    /// Parse a version string to semver Version
    pub fn parse(version: &str) -> Result<semver::Version, VersionError>;
}
```

#### Version Management Types

```rust
pub enum VersionRelationship {
    MajorUpgrade,
    MinorUpgrade,
    PatchUpgrade,
    PrereleaseToStable,
    NewerPrerelease,
    Identical,
    MajorDowngrade,
    MinorDowngrade,
    PatchDowngrade,
    StableToPrerelease,
    OlderPrerelease,
    Indeterminate,
}

pub enum VersionUpdateStrategy {
    PatchOnly,
    MinorAndPatch,
    AllUpdates,
}

pub enum VersionStability {
    StableOnly,
    IncludePrerelease,
}
```

### 7. Dependency Upgrading

#### Upgrader

```rust
pub struct Upgrader {
    // Implementation contains registry_manager, config, and cache
}

impl Upgrader {
    /// Create a new dependency upgrader with default configuration
    pub fn new() -> Self;
    
    /// Create a upgrader with specific configuration and registry
    pub fn create(config: UpgradeConfig, registry_manager: RegistryManager) -> Self;
    
    /// Create a new upgrader with custom configuration
    pub fn with_config(config: UpgradeConfig) -> Self;
    
    /// Create with a specific registry manager
    pub fn with_registry_manager(registry_manager: RegistryManager) -> Self;
    
    /// Get the registry manager
    pub fn registry_manager(&self) -> &RegistryManager;
    
    /// Get a mutable reference to the registry manager
    pub fn registry_manager_mut(&mut self) -> &mut RegistryManager;
    
    /// Set the configuration for the upgrader
    pub fn set_config(&mut self, config: UpgradeConfig);
    
    /// Get the current configuration
    pub fn config(&self) -> &UpgradeConfig;
    
    /// Check for upgrades for a single dependency
    pub fn check_dependency_upgrade(&mut self, package_name: &str, dependency: &Dependency) 
                                 -> Result<AvailableUpgrade, PackageRegistryError>;
    
    /// Check all dependencies in a package for available upgrades
    pub fn check_package_upgrades(&mut self, package: &Package) 
                               -> Result<Vec<AvailableUpgrade>, PackageRegistryError>;
    
    /// Check all packages in a collection for available upgrades
    pub fn check_all_upgrades(&mut self, packages: &[Package]) 
                           -> Result<Vec<AvailableUpgrade>, PackageRegistryError>;
    
    /// Apply upgrades to packages based on what was found
    pub fn apply_upgrades(&self, packages: &[Rc<RefCell<Package>>], 
                        upgrades: &[AvailableUpgrade]) 
                      -> Result<Vec<AvailableUpgrade>, DependencyResolutionError>;
    
    /// Generate a report of upgrades in a human-readable format
    pub fn generate_upgrade_report(upgrades: &[AvailableUpgrade]) -> String;
}
```

#### Upgrade Configuration and Status

```rust
pub enum ExecutionMode {
    DryRun,
    Apply,
}

pub struct UpgradeConfig {
    pub dependency_types: DependencyFilter,
    pub update_strategy: VersionUpdateStrategy,
    pub version_stability: VersionStability,
    pub target_packages: Vec<String>,
    pub target_dependencies: Vec<String>,
    pub registries: Vec<String>,
    pub execution_mode: ExecutionMode,
}

pub enum UpgradeStatus {
    UpToDate,
    PatchAvailable(String),
    MinorAvailable(String),
    MajorAvailable(String),
    Constrained(String),
    CheckFailed(String),
}

pub struct AvailableUpgrade {
    pub package_name: String,
    pub dependency_name: String,
    pub current_version: String,
    pub compatible_version: Option<String>,
    pub latest_version: Option<String>,
    pub status: UpgradeStatus,
}
```

### 8. Resolution and Utilities

#### Resolution Results

```rust
pub struct ResolutionResult {
    pub resolved_versions: HashMap<String, String>,
    pub updates_required: Vec<DependencyUpdate>,
}

pub struct DependencyUpdate {
    pub package_name: String,
    pub dependency_name: String,
    pub current_version: String,
    pub new_version: String,
}
```

#### Package Scope Utilities

```rust
pub struct PackageScopeMetadata {
    pub full: String,
    pub name: String,
    pub version: String,
    pub path: Option<String>,
}

/// Parse package scope, name, and version from a string
pub fn package_scope_name_version(pkg_name: &str) -> Option<PackageScopeMetadata>;
```

#### Helper Functions

```rust
/// Build a dependency graph from Package objects
pub fn build_dependency_graph_from_packages(packages: &[Package]) -> DependencyGraph<'_, Package>;

/// Build a dependency graph from PackageInfo objects
pub fn build_dependency_graph_from_package_infos<'a>(
    package_infos: &[PackageInfo],
    packages: &'a mut Vec<Package>,
) -> DependencyGraph<'a, Package>;
```

### 9. Error Types

#### Package Errors

```rust
pub enum PackageError {
    PackageJsonParseFailure { path: String, error: serde_json::Error },
    PackageJsonIoFailure { path: String, error: io::Error },
    PackageBetweenFailure(String),
    PackageNotFound(String),
}
```

#### Registry Errors

```rust
pub enum PackageRegistryError {
    FetchFailure(#[source] reqwest::Error),
    JsonParseFailure(#[source] reqwest::Error),
    NotFound { package_name: String, version: String },
    LockFailure,
}

pub enum RegistryError {
    UrlNotSupported(String),
    UrlNotFound(String),
    NpmRcFailure { path: String, error: io::Error },
}
```

#### Dependency Errors

```rust
pub enum DependencyResolutionError {
    VersionParseError(String),
    IncompatibleVersions { name: String, versions: Vec<String>, requirements: Vec<String> },
    NoValidVersion { name: String, requirements: Vec<String> },
    DependencyNotFound { name: String, package: String },
    CircularDependency { path: Vec<String> },
}
```

#### Version Errors

```rust
pub enum VersionError {
    Parse { error: semver::Error, message: String },
    InvalidVersion(String),
}
```

### 10. Miscellaneous

#### Change Types

```rust
pub enum ChangeType {
    Added,
    Removed,
    Updated,
    Unchanged,
}
```

#### Cache Utilities

```rust
pub struct CacheEntry<T> {
    data: T,
    timestamp: Instant,
}

impl<T: Clone> CacheEntry<T> {
    pub fn new(data: T) -> Self;
    pub fn is_valid(&self, ttl: Duration) -> bool;
    pub fn get(&self) -> T;
}
```

## Common Usage Patterns

### 1. Creating and Working with Packages

```rust
// Create a new package with dependencies
let dep1 = Rc::new(RefCell::new(Dependency::new("react", "^17.0.0").unwrap()));
let dep2 = Rc::new(RefCell::new(Dependency::new("lodash", "^4.0.0").unwrap()));
let pkg = Package::new("my-app", "1.0.0", Some(vec![dep1, dep2])).unwrap();

// Update a package version
pkg.update_version("1.1.0").unwrap();

// Update a dependency version
pkg.update_dependency_version("react", "^18.0.0").unwrap();
```

### 2. Building and Analyzing Dependency Graphs

```rust
// Build a graph from packages
let packages = vec![/* list of Package objects */];
let graph = build_dependency_graph_from_packages(&packages);

// Check for validation issues
let validation = graph.validate_package_dependencies().unwrap();
if validation.has_critical_issues() {
    for issue in validation.critical_issues() {
        println!("Critical issue: {}", issue.message());
    }
}

// Check for version conflicts
if let Some(conflicts) = graph.find_version_conflicts() {
    for (name, versions) in conflicts {
        println!("Conflict for {}: {}", name, versions.join(", "));
    }
}

// Generate visualization
let ascii = generate_ascii(&graph).unwrap();
println!("{}", ascii);
```

### 3. Using the Registry to Get Package Information

```rust
// Create a registry manager
let mut manager = RegistryManager::new();

// Add a custom registry
manager.add_registry("https://custom-registry.example.com", RegistryType::Npm);

// Associate a scope
manager.associate_scope("@my-scope", "https://custom-registry.example.com").unwrap();

// Get package versions
let versions = manager.get_all_versions("react").unwrap();
println!("Available versions: {}", versions.join(", "));
```

### 4. Upgrading Dependencies

```rust
// Create an upgrader
let config = UpgradeConfig {
    update_strategy: VersionUpdateStrategy::MinorAndPatch,
    execution_mode: ExecutionMode::DryRun,
    ..UpgradeConfig::default()
};
let mut upgrader = Upgrader::with_config(config);

// Check for upgrades
let upgrades = upgrader.check_all_upgrades(&packages).unwrap();

// Generate report
let report = Upgrader::generate_upgrade_report(&upgrades);
println!("{}", report);

// Apply upgrades
let packages_rc: Vec<Rc<RefCell<Package>>> = packages.iter()
    .map(|p| Rc::new(RefCell::new(p.clone())))
    .collect();

upgrader.set_config(UpgradeConfig {
    execution_mode: ExecutionMode::Apply,
    ..config
});

let applied = upgrader.apply_upgrades(&packages_rc, &upgrades).unwrap();
```

### 5. Creating Package Diffs

```rust
// Create package diff between versions
let old_package = /* get previous package version */;
let new_package = /* get current package version */;

let diff = PackageDiff::between(&old_package, &new_package).unwrap();

// Display changes
println!("{}", diff);

// Count breaking changes
let breaking_changes = diff.count_breaking_changes();
println!("Breaking changes: {}", breaking_changes);
```

This API specification provides a comprehensive overview of the `sublime_package_tools` crate's functionality for managing JavaScript/TypeScript package dependencies, analyzing dependency graphs, resolving version conflicts, and upgrading dependencies.
