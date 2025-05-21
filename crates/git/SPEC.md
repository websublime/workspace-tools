# SPEC.md for sublime_git_tools

## Table of Contents

- [Overview](#overview)
- [Repository Module](#repository-module)
  - [Repository Management](#repository-management)
  - [Branch Operations](#branch-operations)
  - [Commit Operations](#commit-operations)
  - [Tag Operations](#tag-operations)
  - [File Change Detection](#file-change-detection)
  - [Remote Operations](#remote-operations)
  - [Helper Methods](#helper-methods)
- [Types Module](#types-module)
  - [Repository Types](#repository-types)
  - [File Status Types](#file-status-types)
  - [Commit and Tag Types](#commit-and-tag-types)
  - [Error Types](#error-types)
- [Examples](#examples)

## Overview

A high-level Rust interface to Git operations with robust error handling, built on libgit2.

```rust
const VERSION = "0.1.0"

fn version() -> &'static str
```

## Repository Module

### Repository Management

#### Types

```rust
pub struct Repo {
    repo: Rc<Repository>,
    local_path: PathBuf,
}
```

#### Core Repository Methods

```rust
impl Repo {
    /// Creates a new Git repository at the specified path
    pub fn create(path: &str) -> Result<Self, RepoError>
    
    /// Opens an existing Git repository at the specified path
    pub fn open(path: &str) -> Result<Self, RepoError>
    
    /// Clones a Git repository from a URL to a local path
    pub fn clone(url: &str, path: &str) -> Result<Self, RepoError>
    
    /// Gets the local path of the repository
    pub fn get_repo_path(&self) -> &Path
    
    /// Configures the repository with user information and core settings
    pub fn config(&self, username: &str, email: &str) -> Result<&Self, RepoError>
    
    /// Lists all configuration entries for the repository
    pub fn list_config(&self) -> Result<HashMap<String, String>, RepoError>
}
```

### Branch Operations

```rust
impl Repo {
    /// Creates a new branch based on the current HEAD
    pub fn create_branch(&self, branch_name: &str) -> Result<&Self, RepoError>
    
    /// Lists all local branches in the repository
    pub fn list_branches(&self) -> Result<Vec<String>, RepoError>
    
    /// Checks out a local branch
    pub fn checkout(&self, branch_name: &str) -> Result<&Self, RepoError>
    
    /// Gets the name of the currently checked out branch
    pub fn get_current_branch(&self) -> Result<String, RepoError>
    
    /// Finds the branch that contains a specific commit
    pub fn get_branch_from_commit(&self, sha: &str) -> Result<Option<String>, RepoError>
    
    /// Finds all branches that contain a specific commit
    pub fn get_branches_containing_commit(&self, sha: &str) -> Result<Vec<String>, RepoError>
    
    /// Finds the common ancestor (merge base) between HEAD and another reference
    pub fn get_diverged_commit(&self, git_ref: &str) -> Result<String, RepoError>
}
```

### Commit Operations

```rust
impl Repo {
    /// Adds a file to the Git index
    pub fn add(&self, file_path: &str) -> Result<&Self, RepoError>
    
    /// Adds all changed files to the Git index
    pub fn add_all(&self) -> Result<&Self, RepoError>
    
    /// Creates a new commit with the current index
    pub fn commit(&self, message: &str) -> Result<String, RepoError>
    
    /// Adds all changes and creates a new commit
    pub fn commit_changes(&self, message: &str) -> Result<String, RepoError>
    
    /// Gets the SHA of the current HEAD commit
    pub fn get_current_sha(&self) -> Result<String, RepoError>
    
    /// Gets the SHA of the parent of the current HEAD commit
    pub fn get_previous_sha(&self) -> Result<String, RepoError>
    
    /// Gets commits made since a specific reference or from the beginning
    pub fn get_commits_since(&self, since: Option<String>, relative: &Option<String>) -> Result<Vec<RepoCommit>, RepoError>
    
    /// Gets the status of the repository in porcelain format
    pub fn status_porcelain(&self) -> Result<Vec<String>, RepoError>
}
```

### Tag Operations

```rust
impl Repo {
    /// Creates a new tag at the current HEAD
    pub fn create_tag(&self, tag: &str, message: Option<String>) -> Result<&Self, RepoError>
    
    /// Gets the name of the last tag in the repository
    pub fn get_last_tag(&self) -> Result<String, RepoError>
    
    /// Gets tags from either local repository or remote
    pub fn get_remote_or_local_tags(&self, local: Option<bool>) -> Result<Vec<RepoTags>, RepoError>
    
    /// Gets all local tags in the repository
    fn get_local_tags(&self) -> Result<Vec<RepoTags>, RepoError>
}
```

### File Change Detection

```rust
impl Repo {
    /// Gets all files changed since a specific reference with their status
    pub fn get_all_files_changed_since_sha_with_status(&self, git_ref: &str) -> Result<Vec<GitChangedFile>, RepoError>
    
    /// Gets all files changed since a specific reference
    pub fn get_all_files_changed_since_sha(&self, git_ref: &str) -> Result<Vec<String>, RepoError>
    
    /// Gets all files changed since a specific branch within specified package paths
    pub fn get_all_files_changed_since_branch(&self, packages_paths: &[String], branch: &str) -> Result<Vec<String>, RepoError>
}
```

### Remote Operations

```rust
impl Repo {
    /// Pushes the current branch to a remote repository
    pub fn push(&self, remote_name: &str, follow_tags: Option<bool>) -> Result<bool, RepoError>
    
    /// Pushes the current branch to a remote repository with custom SSH key paths
    pub fn push_with_ssh_config(&self, remote_name: &str, follow_tags: Option<bool>, ssh_key_paths: Vec<PathBuf>) -> Result<bool, RepoError>
    
    /// Fetches changes from a remote repository
    pub fn fetch(&self, remote_name: &str, refspecs: Option<&[&str]>, prune: bool) -> Result<bool, RepoError>
    
    /// Pulls changes from a remote repository
    pub fn pull(&self, remote_name: &str, branch_name: Option<&str>) -> Result<bool, RepoError>
    
    /// Merges the specified branch into the current HEAD
    pub fn merge(&self, branch_name: &str) -> Result<(), RepoError>
}
```

### Helper Methods

```rust
impl Repo {
    /// Checks if a commit touches a specific path
    fn commit_touches_path(&self, commit: &Commit, path: &PathBuf) -> Result<bool, RepoError>
    
    /// Creates SSH credentials for Git operations
    fn create_ssh_credentials(&self, _url: &str, username_from_url: Option<&str>, _allowed_types: CredentialType, custom_key_paths: Option<&Vec<PathBuf>>) -> Result<Cred, Git2Error>
    
    /// Gets all remote tags from the 'origin' remote
    fn get_remote_tags(&self) -> Result<Vec<RepoTags>, RepoError>
    
    /// Creates an initial commit in a new repository
    fn make_initial_commit(&self) -> Result<(), RepoError>
}
```

## Types Module

### Repository Types

```rust
/// Represents a Git repository with high-level operation methods
pub struct Repo {
    repo: Rc<Repository>,
    local_path: PathBuf,
}
```

### File Status Types

```rust
/// Represents the status of a file in Git
pub enum GitFileStatus {
    Added,    /// File has been added to the repository
    Modified, /// File has been modified
    Deleted,  /// File has been deleted
}

/// Represents a changed file in the Git repository
pub struct GitChangedFile {
    /// The path to the changed file
    pub path: String,
    /// The status of the file (Added, Modified, or Deleted)
    pub status: GitFileStatus,
}
```

### Commit and Tag Types

```rust
/// Represents a commit in the Git repository
pub struct RepoCommit {
    /// The commit hash (SHA)
    pub hash: String,
    /// The name of the commit author
    pub author_name: String,
    /// The email of the commit author
    pub author_email: String,
    /// The date of the commit in RFC2822 format
    pub author_date: String,
    /// The commit message
    pub message: String,
}

/// Represents a tag in the Git repository
pub struct RepoTags {
    /// The hash of the commit that the tag points to
    pub hash: String,
    /// The name of the tag
    pub tag: String,
}
```

### Error Types

```rust
/// Errors that can occur when working with Git repositories
pub enum RepoError {
    CanonicPathFailure(std::io::Error),     /// Failed to canonicalize a path
    GitFailure(Git2Error),                   /// Generic Git operation failure
    CreateRepoFailure(Git2Error),            /// Failed to create a new repository
    OpenRepoFailure(Git2Error),              /// Failed to open an existing repository
    CloneRepoFailure(Git2Error),             /// Failed to clone a repository
    ConfigError(Git2Error),                  /// Git configuration error
    ConfigEntriesError(Git2Error),           /// Failed to retrieve configuration entries
    HeadError(Git2Error),                    /// Failed to get repository HEAD
    PeelError(Git2Error),                    /// Failed to peel a reference to a commit
    BranchError(Git2Error),                  /// Failed to create or manipulate a branch
    SignatureError(Git2Error),               /// Failed to get repository signature
    IndexError(Git2Error),                   /// Failed to get or manipulate the index
    AddFilesError(Git2Error),                /// Failed to add files to the index
    WriteIndexError(Git2Error),              /// Failed to write the index
    TreeError(Git2Error),                    /// Failed to find or manipulate a tree
    CommitError(Git2Error),                  /// Failed to create a commit
    WriteTreeError(Git2Error),               /// Failed to write a tree
    BranchListError(Git2Error),              /// Failed to list branches
    BranchNameError(Git2Error),              /// Failed to get a branch name
    CheckoutBranchError(Git2Error),          /// Failed to checkout a branch
    CheckoutError(Git2Error),                /// Failed to checkout
    LastTagError(Git2Error),                 /// Failed to get the last tag
    CreateTagError(Git2Error),               /// Failed to create a tag
    StatusError(Git2Error),                  /// Failed to get repository status
    CommitOidError(Git2Error),               /// Failed to parse a commit SHA
    GraphError(Git2Error),                   /// Failed on repository graph operations
    PushError(Git2Error),                    /// Failed to push to a remote
    RemoteError(Git2Error),                  /// Failed on remote operations
    ReferenceError(Git2Error),               /// Failed on reference parsing
    DiffError(Git2Error),                    /// Failed on diff operations
    RevWalkError(Git2Error),                 /// Failed on revision walking
    TagError(Git2Error),                     /// Failed on tag operations
    MergeError(Git2Error),                   /// Failed on merge operations
    MergeConflictError(Git2Error),           /// Failed due to merge conflicts
}
```

## Examples

### Repository Management

```rust
use sublime_git_tools::Repo;

// Create a new repository
let repo = Repo::create("/path/to/new/repo").expect("Failed to create repository");

// Open an existing repository
let repo = Repo::open("./my-project").expect("Failed to open repository");

// Clone a remote repository
let repo = Repo::clone("https://github.com/example/repo.git", "./cloned-repo")
    .expect("Failed to clone repository");

// Configure user information
repo.config("John Doe", "john@example.com").expect("Failed to configure repository");
```

### Branch and Commit Operations

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project").expect("Failed to open repository");

// Create a new branch
repo.create_branch("feature/new-feature").expect("Failed to create branch");

// Checkout a branch
repo.checkout("feature/new-feature").expect("Failed to checkout branch");

// Add files and commit
repo.add("src/main.rs").expect("Failed to add file");
let commit_id = repo.commit("feat: update main.rs").expect("Failed to commit");

// Or add all changes and commit in one step
let commit_id = repo.commit_changes("feat: implement new feature")
    .expect("Failed to commit changes");
```

### Tag Operations

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project").expect("Failed to open repository");

// Create a tag
repo.create_tag("v1.0.0", Some("Version 1.0.0 release".to_string()))
    .expect("Failed to create tag");

// Get the last tag
let last_tag = repo.get_last_tag().expect("Failed to get last tag");
println!("Last tag: {}", last_tag);

// Get local tags
let local_tags = repo.get_remote_or_local_tags(Some(true))
    .expect("Failed to get local tags");

for tag in local_tags {
    println!("Tag {} points to commit {}", tag.tag, tag.hash);
}
```

### File Change Detection

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project").expect("Failed to open repository");

// Get all changed files since a tag or commit
let changed_files = repo.get_all_files_changed_since_sha("v1.0.0")
    .expect("Failed to get changed files");

// Get all changed files with their status (Added, Modified, Deleted)
let changed_files_with_status = repo
    .get_all_files_changed_since_sha_with_status("v1.0.0")
    .expect("Failed to get changed files");

// Get changes in specific packages since a branch
let packages = vec!["packages/pkg1".to_string(), "packages/pkg2".to_string()];
let package_changes = repo
    .get_all_files_changed_since_branch(&packages, "main")
    .expect("Failed to get package changes");
```

### Commit History

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project").expect("Failed to open repository");

// Get all commits since a specific tag
let commits = repo.get_commits_since(
    Some("v1.0.0".to_string()),
    &None
).expect("Failed to get commits");

// Get commits affecting a specific file
let file_commits = repo.get_commits_since(
    None,
    &Some("src/main.rs".to_string())
).expect("Failed to get file history");

for commit in commits {
    println!("{}: {} (by {} on {})",
        commit.hash,
        commit.message,
        commit.author_name,
        commit.author_date
    );
}
```

### Remote Operations

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project").expect("Failed to open repository");

// Push to remote
repo.push("origin", Some(true)).expect("Failed to push"); // true = push tags

// Fetch from remote
repo.fetch("origin", None, false).expect("Failed to fetch");

// Pull from remote
repo.pull("origin", None).expect("Failed to pull");

// Merge a branch
repo.checkout("main").expect("Failed to checkout main");
repo.merge("feature-branch").expect("Failed to merge feature branch");
```

### Error Handling

```rust
use sublime_git_tools::{Repo, RepoError};

match repo.checkout("feature-branch") {
    Ok(_) => println!("Switched to feature-branch"),
    Err(e) => match e {
        RepoError::BranchNameError(_) => eprintln!("Branch does not exist"),
        RepoError::CheckoutBranchError(_) => eprintln!("Failed to checkout branch"),
        _ => eprintln!("Error: {}", e),
    }
}
```

