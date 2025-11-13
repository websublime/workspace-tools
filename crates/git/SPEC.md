# sublime_git_tools API Specification

## Table of Contents

- [Overview](#overview)
- [Repository Management](#repository-management)
  - [Creating and Opening Repositories](#creating-and-opening-repositories)
  - [Repository Information](#repository-information)
  - [Configuration](#configuration)
- [Branch Operations](#branch-operations)
  - [Creating and Managing Branches](#creating-and-managing-branches)
  - [Branch Information](#branch-information)
- [Commit Operations](#commit-operations)
  - [Staging and Committing](#staging-and-committing)
  - [Commit Information](#commit-information)
  - [Commit History](#commit-history)
- [Tag Operations](#tag-operations)
  - [Creating Tags](#creating-tags)
  - [Tag Information](#tag-information)
- [File Status and Change Detection](#file-status-and-change-detection)
  - [Repository Status](#repository-status)
  - [Changed Files](#changed-files)
  - [Package-specific Changes](#package-specific-changes)
- [Remote Operations](#remote-operations)
  - [Pushing and Pulling](#pushing-and-pulling)
  - [Fetching](#fetching)
  - [SSH Operations](#ssh-operations)
- [Advanced Git Operations](#advanced-git-operations)
  - [Merging](#merging)
  - [Repository Analysis](#repository-analysis)
- [Types Reference](#types-reference)
  - [Repository Types](#repository-types)
  - [File Status Types](#file-status-types)
  - [Commit and Tag Types](#commit-and-tag-types)
  - [Error Types](#error-types)
- [Error Handling](#error-handling)
- [Examples](#examples)
  - [Basic Workflow](#basic-workflow)
  - [Branch Management](#branch-management)
  - [Tracking Changes](#tracking-changes)
  - [Remote Operations](#remote-operations-examples)

## Overview

`sublime_git_tools` is a high-level Rust interface to Git operations with robust error handling, built on libgit2. It provides a user-friendly API for working with Git repositories, wrapping the powerful but complex libgit2 library to offer a more ergonomic interface for common Git operations.

This crate is designed for Rust applications that need to:

- Create, clone, or manipulate Git repositories
- Manage branches, commits, and tags
- Track file changes between commits or branches
- Push/pull with remote repositories
- Get detailed commit histories
- Detect changes in specific parts of a repository
- Handle SSH authentication for remote operations
- Perform advanced Git operations like merging and status checking

## Repository Management

### Creating and Opening Repositories

#### `Repo::create`

Creates a new Git repository at the specified path with an initial commit on the 'main' branch.

```rust
pub fn create(path: &str) -> Result<Self, RepoError>
```

**Parameters:**
- `path`: The path where the repository should be created

**Returns:**
- `Result<Self, RepoError>`: A new `Repo` instance or an error

**Example:**
```rust
let repo = Repo::create("/path/to/new/repo")?;
println!("Repository created at: {}", repo.get_repo_path().display());
```

**Possible errors:**
- `CanonicalPathFailure`: Failed to canonicalize the provided path
- `CreateRepoFailure`: Failed to initialize the Git repository

#### `Repo::open`

Opens an existing Git repository at the specified path.

```rust
pub fn open(path: &str) -> Result<Self, RepoError>
```

**Parameters:**
- `path`: The path to the existing repository

**Returns:**
- `Result<Self, RepoError>`: A `Repo` instance or an error

**Example:**
```rust
let repo = Repo::open("./my-project")?;
let branch = repo.get_current_branch()?;
println!("Current branch: {}", branch);
```

**Possible errors:**
- `CanonicalPathFailure`: Failed to canonicalize the provided path
- `OpenRepoFailure`: Failed to open the Git repository

#### `Repo::clone`

Clones a Git repository from a URL to a local path.

```rust
pub fn clone(url: &str, path: &str) -> Result<Self, RepoError>
```

**Parameters:**
- `url`: The URL of the repository to clone
- `path`: The local path where the repository should be cloned

**Returns:**
- `Result<Self, RepoError>`: A `Repo` instance or an error

**Example:**
```rust
let repo = Repo::clone("https://github.com/example/repo.git", "./cloned-repo")?;
```

**Possible errors:**
- `CanonicalPathFailure`: Failed to canonicalize the provided path
- `CloneRepoFailure`: Failed to clone the Git repository

#### `Repo::clone_with_options`

Clones a Git repository from a URL to a local path with advanced options including shallow clone support.

```rust
pub fn clone_with_options(
    url: &str,
    path: &str,
    depth: Option<i32>
) -> Result<Self, RepoError>
```

**Parameters:**
- `url`: The URL of the repository to clone
- `path`: The local path where the repository should be cloned
- `depth`: Optional depth for shallow clone (e.g., `Some(1)` for only the latest commit, `None` for full clone)

**Returns:**
- `Result<Self, RepoError>`: A `Repo` instance or an error

**Examples:**
```rust
// Full clone (equivalent to Repo::clone)
let repo = Repo::clone_with_options(
    "https://github.com/example/repo.git",
    "./cloned-repo",
    None
)?;

// Shallow clone with depth 1 (only latest commit)
let repo = Repo::clone_with_options(
    "https://github.com/example/large-repo.git",
    "./shallow-clone",
    Some(1)
)?;

// Shallow clone with depth 10 (last 10 commits)
let repo = Repo::clone_with_options(
    "https://github.com/example/repo.git",
    "./partial-clone",
    Some(10)
)?;
```

**Performance Benefits:**
Shallow clones can significantly reduce:
- Clone time (especially for repositories with extensive history)
- Disk space usage
- Network bandwidth consumption

This is particularly useful for:
- CI/CD pipelines that only need the latest code
- Deployment scenarios
- Quick repository inspection
- Limited disk space environments

**Limitations:**
- Cannot push from a shallow clone without converting to full clone first
- Some operations requiring full history may fail
- Can be converted to full clone later using `git fetch --unshallow`

**Possible errors:**
- `CanonicalPathFailure`: Failed to canonicalize the provided path
- `CloneRepoFailure`: Failed to clone the Git repository
- Network connection failures
- Authentication failures
- Insufficient disk space

#### `Repo::clone_with_progress`

Clones a Git repository with real-time progress tracking callbacks.

```rust
pub fn clone_with_progress<F>(
    url: &str,
    path: &str,
    depth: Option<i32>,
    progress_cb: F
) -> Result<Self, RepoError>
where
    F: FnMut(usize, usize) + 'static
```

**Parameters:**
- `url`: The URL of the repository to clone
- `path`: The local path where the repository should be cloned
- `depth`: Optional depth for shallow clone
- `progress_cb`: Callback function that receives `(current_objects, total_objects)` for progress updates

**Returns:**
- `Result<Self, RepoError>`: A `Repo` instance or an error

**Examples:**
```rust
use sublime_git_tools::Repo;

// Clone with progress updates
let repo = Repo::clone_with_progress(
    "https://github.com/example/repo.git",
    "./cloned-repo",
    None,
    |current, total| {
        println!("Progress: {}/{} objects ({:.1}%)", 
                 current, total, 
                 (current as f64 / total as f64) * 100.0);
    }
)?;

// Shallow clone with progress
let repo = Repo::clone_with_progress(
    "https://github.com/example/large-repo.git",
    "./shallow",
    Some(1),
    |current, total| {
        if total > 0 {
            let percent = (current as f64 / total as f64) * 100.0;
            print!("\rReceiving objects: {:.0}%", percent);
        }
    }
)?;
```

**Progress Callback Details:**
The callback is invoked periodically during the clone operation with:
- `current`: Number of objects received so far
- `total`: Total number of objects to receive (may be 0 initially)

The callback is called during:
1. Receiving objects from the remote
2. Resolving deltas
3. Indexing objects

**Possible errors:**
- `CanonicalPathFailure`: Failed to canonicalize the provided path
- `CloneRepoFailure`: Failed to clone the Git repository
- Network connection failures
- Authentication failures
- Insufficient disk space

### Repository Information

#### `Repo::get_repo_path`

Returns the path to the repository root directory.

```rust
pub fn get_repo_path(&self) -> &Path
```

**Returns:**
- `&Path`: Reference to the repository path

**Example:**
```rust
let repo = Repo::open("./my-project")?;
let path = repo.get_repo_path();
println!("Repository path: {}", path.display());
```

### Configuration

#### `Repo::config`

Sets the Git configuration for username and email in the repository.

```rust
pub fn config(&self, username: &str, email: &str) -> Result<&Self, RepoError>
```

**Parameters:**
- `username`: The Git username to set
- `email`: The Git email to set

**Returns:**
- `Result<&Self, RepoError>`: Reference to self for chaining or an error

**Example:**
```rust
repo.config("Jane Doe", "jane@example.com")?;
```

**Possible errors:**
- `ConfigError`: Failed to access or modify Git configuration

#### `Repo::list_config`

Lists all configuration entries in the repository.

```rust
pub fn list_config(&self) -> Result<HashMap<String, String>, RepoError>
```

**Returns:**
- `Result<HashMap<String, String>, RepoError>`: Map of configuration key-value pairs or an error

**Example:**
```rust
let config_entries = repo.list_config()?;
for (key, value) in config_entries {
    println!("{}: {}", key, value);
}
```

**Possible errors:**
- `ConfigError`: Failed to access Git configuration
- `ConfigEntriesError`: Failed to retrieve configuration entries

## Branch Operations

### Creating and Managing Branches

#### `Repo::create_branch`

Creates a new branch from the current HEAD commit.

```rust
pub fn create_branch(&self, branch_name: &str) -> Result<&Self, RepoError>
```

**Parameters:**
- `branch_name`: The name of the new branch to create

**Returns:**
- `Result<&Self, RepoError>`: Reference to self for chaining or an error

**Example:**
```rust
repo.create_branch("feature/new-feature")?;
```

**Possible errors:**
- `HeadError`: Failed to get repository HEAD
- `PeelError`: Failed to peel HEAD to commit
- `BranchError`: Failed to create the branch

#### `Repo::checkout`

Switches to the specified branch.

```rust
pub fn checkout(&self, branch_name: &str) -> Result<&Self, RepoError>
```

**Parameters:**
- `branch_name`: The name of the branch to checkout

**Returns:**
- `Result<&Self, RepoError>`: Reference to self for chaining or an error

**Example:**
```rust
repo.checkout("feature-branch")?;
```

**Possible errors:**
- `BranchError`: Failed to find the branch
- `CheckoutBranchError`: Failed to checkout the branch
- `CheckoutError`: Failed to perform checkout

### Branch Information

#### `Repo::list_branches`

Lists all local branches in the repository.

```rust
pub fn list_branches(&self) -> Result<Vec<String>, RepoError>
```

**Returns:**
- `Result<Vec<String>, RepoError>`: Vector of branch names or an error

**Example:**
```rust
let branches = repo.list_branches()?;
for branch in branches {
    println!("Branch: {}", branch);
}
```

**Possible errors:**
- `BranchListError`: Failed to list branches
- `BranchNameError`: Failed to get branch name

#### `Repo::get_current_branch`

Gets the name of the currently checked out branch.

```rust
pub fn get_current_branch(&self) -> Result<String, RepoError>
```

**Returns:**
- `Result<String, RepoError>`: The current branch name or an error

**Example:**
```rust
let current_branch = repo.get_current_branch()?;
println!("Current branch: {}", current_branch);
```

**Possible errors:**
- `HeadError`: Failed to get repository HEAD
- `BranchNameError`: Failed to get branch name from HEAD

#### `Repo::branch_exists`

Checks if a branch exists in the repository.

```rust
pub fn branch_exists(&self, branch_name: &str) -> Result<bool, RepoError>
```

**Parameters:**
- `branch_name`: The name of the branch to check

**Returns:**
- `Result<bool, RepoError>`: True if branch exists, false otherwise, or an error

**Example:**
```rust
if repo.branch_exists("main")? {
    println!("Branch 'main' exists");
}
```

**Possible errors:**
- `BranchError`: Failed to access branch information

#### `Repo::get_branch_from_commit`

Gets the branch name that contains a specific commit.

```rust
pub fn get_branch_from_commit(&self, sha: &str) -> Result<Option<String>, RepoError>
```

**Parameters:**
- `sha`: The commit SHA to search for

**Returns:**
- `Result<Option<String>, RepoError>`: Branch name if found, None if not found, or an error

**Example:**
```rust
if let Some(branch) = repo.get_branch_from_commit("abcdef123456")? {
    println!("Commit is on branch: {}", branch);
}
```

**Possible errors:**
- `CommitOidError`: Failed to parse commit SHA
- `BranchListError`: Failed to list branches
- `BranchNameError`: Failed to get branch name

#### `Repo::get_branches_containing_commit`

Gets all branches that contain a specific commit.

```rust
pub fn get_branches_containing_commit(&self, sha: &str) -> Result<Vec<String>, RepoError>
```

**Parameters:**
- `sha`: The commit SHA to search for

**Returns:**
- `Result<Vec<String>, RepoError>`: Vector of branch names containing the commit or an error

**Example:**
```rust
let branches = repo.get_branches_containing_commit("abcdef123456")?;
for branch in branches {
    println!("Branch {} contains the commit", branch);
}
```

**Possible errors:**
- `CommitOidError`: Failed to parse commit SHA
- `BranchListError`: Failed to list branches
- `GraphError`: Failed to perform graph operations

## Commit Operations

### Staging and Committing

#### `Repo::add`

Adds a specific file to the staging area.

```rust
pub fn add(&self, file_path: &str) -> Result<&Self, RepoError>
```

**Parameters:**
- `file_path`: The path to the file to add

**Returns:**
- `Result<&Self, RepoError>`: Reference to self for chaining or an error

**Example:**
```rust
repo.add("src/main.rs")?;
```

**Possible errors:**
- `IndexError`: Failed to access the index
- `AddFilesError`: Failed to add file to index
- `WriteIndexError`: Failed to write index

#### `Repo::add_all`

Adds all changed files to the staging area.

```rust
pub fn add_all(&self) -> Result<&Self, RepoError>
```

**Returns:**
- `Result<&Self, RepoError>`: Reference to self for chaining or an error

**Example:**
```rust
repo.add_all()?;
```

**Possible errors:**
- `IndexError`: Failed to access the index
- `AddFilesError`: Failed to add files to index
- `WriteIndexError`: Failed to write index

#### `Repo::commit`

Creates a commit with the currently staged changes.

```rust
pub fn commit(&self, message: &str) -> Result<String, RepoError>
```

**Parameters:**
- `message`: The commit message

**Returns:**
- `Result<String, RepoError>`: The commit SHA or an error

**Example:**
```rust
let commit_id = repo.commit("feat: add new functionality")?;
println!("Created commit: {}", commit_id);
```

**Possible errors:**
- `SignatureError`: Failed to get repository signature
- `HeadError`: Failed to get repository HEAD
- `IndexError`: Failed to access the index
- `TreeError`: Failed to create tree from index
- `CommitError`: Failed to create commit

#### `Repo::commit_changes`

Adds all changes and creates a commit in one operation.

```rust
pub fn commit_changes(&self, message: &str) -> Result<String, RepoError>
```

**Parameters:**
- `message`: The commit message

**Returns:**
- `Result<String, RepoError>`: The commit SHA or an error

**Example:**
```rust
let commit_id = repo.commit_changes("fix: resolve authentication issue")?;
println!("Created commit: {}", commit_id);
```

**Possible errors:**
- `SignatureError`: Failed to get repository signature
- `HeadError`: Failed to get repository HEAD
- `IndexError`: Failed to access the index
- `AddFilesError`: Failed to add files to index
- `WriteIndexError`: Failed to write index
- `TreeError`: Failed to create tree from index
- `CommitError`: Failed to create commit

### Commit Information

#### `Repo::get_current_sha`

Gets the SHA of the current HEAD commit.

```rust
pub fn get_current_sha(&self) -> Result<String, RepoError>
```

**Returns:**
- `Result<String, RepoError>`: The current commit SHA or an error

**Example:**
```rust
let current_sha = repo.get_current_sha()?;
println!("Current commit: {}", current_sha);
```

**Possible errors:**
- `HeadError`: Failed to get repository HEAD
- `CommitOidError`: Failed to get commit OID

#### `Repo::get_previous_sha`

Gets the SHA of the previous commit (parent of HEAD).

```rust
pub fn get_previous_sha(&self) -> Result<String, RepoError>
```

**Returns:**
- `Result<String, RepoError>`: The previous commit SHA or an error

**Example:**
```rust
let previous_sha = repo.get_previous_sha()?;
println!("Previous commit: {}", previous_sha);
```

**Possible errors:**
- `HeadError`: Failed to get repository HEAD
- `PeelError`: Failed to peel HEAD to commit
- `CommitOidError`: Failed to get commit OID

### Commit History

#### `Repo::get_commits_since`

Gets commits since a specific reference, optionally filtered by file path.

```rust
pub fn get_commits_since(
    &self,
    since: Option<String>,
    path: &Option<String>
) -> Result<Vec<RepoCommit>, RepoError>
```

**Parameters:**
- `since`: Optional reference (tag, branch, or commit SHA) to start from
- `path`: Optional file path to filter commits

**Returns:**
- `Result<Vec<RepoCommit>, RepoError>`: Vector of commits or an error

**Example:**
```rust
// Get all commits since a tag
let commits = repo.get_commits_since(Some("v1.0.0".to_string()), &None)?;

// Get commits affecting a specific file
let file_commits = repo.get_commits_since(None, &Some("src/main.rs".to_string()))?;
```

**Possible errors:**
- `RevWalkError`: Failed to perform revision walk
- `ReferenceError`: Failed to resolve reference
- `CommitOidError`: Failed to parse commit

#### `Repo::get_commits_between`

Gets commits between two references (commits in `to_ref` but not in `from_ref`).

```rust
pub fn get_commits_between(
    &self,
    from_ref: &str,
    to_ref: &str,
    relative: &Option<String>
) -> Result<Vec<RepoCommit>, RepoError>
```

**Parameters:**
- `from_ref`: Starting reference (commits after this point)
- `to_ref`: Ending reference (commits up to this point)
- `relative`: Optional file path to filter commits

**Returns:**
- `Result<Vec<RepoCommit>, RepoError>`: Vector of commits between references

**Example:**
```rust
// Get commits between two tags
let commits = repo.get_commits_between("v1.0.0", "v1.1.0", &None)?;

// Get commits between branches affecting specific path
let path_commits = repo.get_commits_between("main", "feature-branch", &Some("src/".to_string()))?;
```

**Possible errors:**
- `RevWalkError`: Failed to perform revision walk
- `ReferenceError`: Failed to resolve reference
- `CommitOidError`: Failed to parse commit
- `PeelError`: Failed to peel reference to commit

## Tag Operations

### Creating Tags

#### `Repo::create_tag`

Creates a new tag at the current HEAD commit.

```rust
pub fn create_tag(&self, tag: &str, message: Option<String>) -> Result<&Self, RepoError>
```

**Parameters:**
- `tag`: The name of the tag to create
- `message`: Optional message for annotated tag (None for lightweight tag)

**Returns:**
- `Result<&Self, RepoError>`: Reference to self for chaining or an error

**Example:**
```rust
// Create annotated tag
repo.create_tag("v1.0.0", Some("Release version 1.0.0".to_string()))?;

// Create lightweight tag
repo.create_tag("v1.0.1", None)?;
```

**Possible errors:**
- `SignatureError`: Failed to get repository signature
- `HeadError`: Failed to get repository HEAD
- `PeelError`: Failed to peel HEAD to commit
- `CreateTagError`: Failed to create tag

### Tag Information

#### `Repo::get_last_tag`

Gets the most recent tag in the repository.

```rust
pub fn get_last_tag(&self) -> Result<String, RepoError>
```

**Returns:**
- `Result<String, RepoError>`: The most recent tag name or an error

**Example:**
```rust
let last_tag = repo.get_last_tag()?;
println!("Last tag: {}", last_tag);
```

**Possible errors:**
- `LastTagError`: Failed to get tags or no tags found

#### `Repo::get_remote_or_local_tags`

Gets tags from local or remote repositories.

```rust
pub fn get_remote_or_local_tags(
    &self,
    local: Option<bool>
) -> Result<Vec<RepoTags>, RepoError>
```

**Parameters:**
- `local`: Optional flag to specify local (true) or remote (false) tags. None for both.

**Returns:**
- `Result<Vec<RepoTags>, RepoError>`: Vector of tags or an error

**Example:**
```rust
// Get local tags
let local_tags = repo.get_remote_or_local_tags(Some(true))?;

// Get remote tags
let remote_tags = repo.get_remote_or_local_tags(Some(false))?;

// Get all tags
let all_tags = repo.get_remote_or_local_tags(None)?;
```

**Possible errors:**
- `TagError`: Failed to retrieve tags
- `RemoteError`: Failed to access remote (for remote tags)

## File Status and Change Detection

### Repository Status

#### `Repo::status_porcelain`

Gets the repository status in porcelain format (simple, parseable format).

```rust
pub fn status_porcelain(&self) -> Result<Vec<String>, RepoError>
```

**Returns:**
- `Result<Vec<String>, RepoError>`: Vector of status lines or an error

**Example:**
```rust
let status_lines = repo.status_porcelain()?;
for line in status_lines {
    println!("{}", line);
}
```

**Possible errors:**
- `StatusError`: Failed to get repository status

#### `Repo::get_status_detailed`

Gets detailed status information for all files in the repository.

```rust
pub fn get_status_detailed(&self) -> Result<Vec<GitChangedFile>, RepoError>
```

**Returns:**
- `Result<Vec<GitChangedFile>, RepoError>`: Vector of file status information or an error

**Example:**
```rust
let detailed_status = repo.get_status_detailed()?;
for file in detailed_status {
    println!("File: {}, Status: {:?}, Staged: {}, Workdir: {}",
        file.path, file.status, file.staged, file.workdir);
}
```

**Possible errors:**
- `StatusError`: Failed to get repository status

#### `Repo::get_staged_files`

Gets a list of files that are currently staged for commit.

```rust
pub fn get_staged_files(&self) -> Result<Vec<String>, RepoError>
```

**Returns:**
- `Result<Vec<String>, RepoError>`: Vector of staged file paths or an error

**Example:**
```rust
let staged_files = repo.get_staged_files()?;
println!("Files ready for commit: {:?}", staged_files);
```

**Possible errors:**
- `StatusError`: Failed to get repository status

### Changed Files

#### `Repo::get_all_files_changed_since_sha_with_status`

Gets all files that changed since a specific commit/reference with their status information.

```rust
pub fn get_all_files_changed_since_sha_with_status(
    &self,
    git_ref: &str
) -> Result<Vec<GitChangedFile>, RepoError>
```

**Parameters:**
- `git_ref`: The reference (commit SHA, tag, or branch) to compare against

**Returns:**
- `Result<Vec<GitChangedFile>, RepoError>`: Vector of changed files with status or an error

**Example:**
```rust
let changed_files = repo.get_all_files_changed_since_sha_with_status("v1.0.0")?;
for file in changed_files {
    println!("Changed file: {} ({:?})", file.path, file.status);
}
```

**Possible errors:**
- `ReferenceError`: Failed to resolve reference
- `DiffError`: Failed to perform diff operation
- `CommitOidError`: Failed to parse commit

#### `Repo::get_all_files_changed_since_sha`

Gets a simple list of files that changed since a specific commit/reference.

```rust
pub fn get_all_files_changed_since_sha(&self, git_ref: &str) -> Result<Vec<String>, RepoError>
```

**Parameters:**
- `git_ref`: The reference (commit SHA, tag, or branch) to compare against

**Returns:**
- `Result<Vec<String>, RepoError>`: Vector of changed file paths or an error

**Example:**
```rust
let changed_files = repo.get_all_files_changed_since_sha("v1.0.0")?;
for file in changed_files {
    println!("Changed: {}", file);
}
```

**Possible errors:**
- `ReferenceError`: Failed to resolve reference
- `DiffError`: Failed to perform diff operation

#### `Repo::get_files_changed_between`

Gets files changed between two references.

```rust
pub fn get_files_changed_between(&self, from_ref: &str, to_ref: &str) -> Result<Vec<GitChangedFile>, RepoError>
```

**Parameters:**
- `from_ref`: The starting reference
- `to_ref`: The ending reference

**Returns:**
- `Result<Vec<GitChangedFile>, RepoError>`: Vector of changed files with status or an error

**Example:**
```rust
let files = repo.get_files_changed_between("main", "feature-branch")?;
for file in files {
    println!("File: {}, Status: {:?}", file.path, file.status);
}
```

**Possible errors:**
- `ReferenceError`: Failed to resolve references
- `DiffError`: Failed to perform diff operation

#### `Repo::get_files_changed_in_commit`

Gets files changed in a specific commit.

```rust
pub fn get_files_changed_in_commit(&self, commit_hash: &str) -> Result<Vec<GitChangedFile>, RepoError>
```

**Parameters:**
- `commit_hash`: The commit hash to analyze

**Returns:**
- `Result<Vec<GitChangedFile>, RepoError>`: Vector of changed files with status or an error

**Example:**
```rust
let files = repo.get_files_changed_in_commit("abc123")?;
for file in files {
    println!("File: {}, Status: {:?}", file.path, file.status);
}
```

**Possible errors:**
- `ReferenceError`: Failed to resolve commit reference
- `CommitError`: Failed to access commit
- `TreeError`: Failed to access tree
- `DiffError`: Failed to perform diff operation

### Package-specific Changes

#### `Repo::get_all_files_changed_since_branch`

Gets files changed in specific package directories since a branch.

```rust
pub fn get_all_files_changed_since_branch(
    &self,
    packages_paths: &[String],
    branch: &str
) -> Result<Vec<String>, RepoError>
```

**Parameters:**
- `packages_paths`: Vector of package directory paths to check
- `branch`: The branch to compare against

**Returns:**
- `Result<Vec<String>, RepoError>`: Vector of changed files in the specified packages or an error

**Example:**
```rust
let packages = vec!["packages/pkg1".to_string(), "packages/pkg2".to_string()];
let package_changes = repo.get_all_files_changed_since_branch(&packages, "main")?;
for file in package_changes {
    println!("Package change: {}", file);
}
```

**Possible errors:**
- `ReferenceError`: Failed to resolve branch reference
- `DiffError`: Failed to perform diff operation

## Remote Operations

### Pushing and Pulling

#### `Repo::push`

Pushes the current branch to a remote repository.

```rust
pub fn push(&self, remote_name: &str, follow_tags: Option<bool>) -> Result<bool, RepoError>
```

**Parameters:**
- `remote_name`: The name of the remote (e.g., "origin")
- `follow_tags`: Optional flag to push tags along with commits

**Returns:**
- `Result<bool, RepoError>`: True if push was successful, or an error

**Example:**
```rust
// Push without tags
let success = repo.push("origin", Some(false))?;

// Push with tags
repo.push("origin", Some(true))?;
```

**Possible errors:**
- `HeadError`: Failed to get repository HEAD
- `BranchNameError`: Failed to get branch name
- `RemoteError`: Failed to find or access remote
- `PushError`: Failed to push to remote

#### `Repo::pull`

Pulls changes from a remote repository.

```rust
pub fn pull(&self, remote_name: &str, branch_name: Option<&str>) -> Result<bool, RepoError>
```

**Parameters:**
- `remote_name`: The name of the remote (e.g., "origin")
- `branch_name`: Optional specific branch to pull (current branch if None)

**Returns:**
- `Result<bool, RepoError>`: True if pull was successful, or an error

**Example:**
```rust
// Pull current branch
let success = repo.pull("origin", None)?;

// Pull specific branch
repo.pull("origin", Some("feature-branch"))?;
```

**Possible errors:**
- `RemoteError`: Failed to find or access remote
- `HeadError`: Failed to get repository HEAD
- `MergeError`: Failed to merge pulled changes

### Fetching

#### `Repo::fetch`

Fetches changes from a remote repository without merging.

```rust
pub fn fetch(
    &self,
    remote_name: &str,
    refspecs: Option<&[&str]>,
    prune: bool
) -> Result<bool, RepoError>
```

**Parameters:**
- `remote_name`: The name of the remote (e.g., "origin")
- `refspecs`: Optional reference specs to fetch (all refs if None)
- `prune`: Whether to prune deleted remote branches

**Returns:**
- `Result<bool, RepoError>`: True if fetch was successful, or an error

**Example:**
```rust
// Fetch all refs
repo.fetch("origin", None, false)?;

// Fetch specific refspecs with pruning
repo.fetch("origin", Some(&["refs/heads/main:refs/remotes/origin/main"]), true)?;
```

**Possible errors:**
- `RemoteError`: Failed to find or access remote
- `GitFailure`: Failed to fetch from remote

### SSH Operations

#### `Repo::push_with_ssh_config`

Pushes to a remote repository using SSH authentication with custom key paths.

```rust
pub fn push_with_ssh_config(
    &self,
    remote_name: &str,
    follow_tags: Option<bool>,
    ssh_key_paths: Vec<PathBuf>
) -> Result<bool, RepoError>
```

**Parameters:**
- `remote_name`: The name of the remote (e.g., "origin")
- `follow_tags`: Optional flag to push tags along with commits
- `ssh_key_paths`: Vector of SSH key file paths to try in order

**Returns:**
- `Result<bool, RepoError>`: True if push was successful, or an error

**Example:**
```rust
use std::path::PathBuf;

let key_paths = vec![
    PathBuf::from("/home/user/.ssh/id_ed25519"),
    PathBuf::from("/home/user/.ssh/id_rsa"),
];
let success = repo.push_with_ssh_config("origin", Some(true), key_paths)?;
```

**Possible errors:**
- `HeadError`: Failed to get repository HEAD
- `BranchNameError`: Failed to get branch name
- `RemoteError`: Failed to find or access remote
- `PushError`: Failed to push to remote

## Advanced Git Operations

### Merging

#### `Repo::merge`

Merges a branch into the current branch.

```rust
pub fn merge(&self, branch_name: &str) -> Result<(), RepoError>
```

**Parameters:**
- `branch_name`: The name of the branch to merge

**Returns:**
- `Result<(), RepoError>`: Success or an error

**Example:**
```rust
match repo.merge("feature-branch") {
    Ok(_) => println!("Merge completed successfully"),
    Err(RepoError::MergeConflictError(_)) => {
        println!("Merge conflicts detected");
    },
    Err(e) => println!("Merge failed: {}", e),
}
```

**Possible errors:**
- `HeadError`: Failed to get repository HEAD
- `BranchError`: Failed to find branch
- `MergeError`: Failed to perform merge
- `MergeConflictError`: Merge conflicts detected

### Repository Analysis

#### `Repo::get_merge_base`

Finds the merge base (common ancestor) between two branches.

```rust
pub fn get_merge_base(&self, branch1: &str, branch2: &str) -> Result<String, RepoError>
```

**Parameters:**
- `branch1`: The first branch name
- `branch2`: The second branch name

**Returns:**
- `Result<String, RepoError>`: The merge base commit SHA or an error

**Example:**
```rust
let merge_base = repo.get_merge_base("main", "feature-branch")?;
println!("Merge base: {}", merge_base);
```

**Possible errors:**
- `ReferenceError`: Failed to resolve branch references
- `GraphError`: Failed to find merge base
- `CommitOidError`: Failed to get commit OID

#### `Repo::get_diverged_commit`

Finds the common ancestor commit between the current branch and a reference.

```rust
pub fn get_diverged_commit(&self, git_ref: &str) -> Result<String, RepoError>
```

**Parameters:**
- `git_ref`: The reference to compare against (branch, tag, or commit SHA)

**Returns:**
- `Result<String, RepoError>`: The common ancestor commit SHA or an error

**Example:**
```rust
let diverged_commit = repo.get_diverged_commit("feature-branch")?;
println!("Common ancestor: {}", diverged_commit);
```

**Possible errors:**
- `ReferenceError`: Failed to resolve reference
- `HeadError`: Failed to get repository HEAD
- `GraphError`: Failed to find common ancestor
- `CommitOidError`: Failed to get commit OID

## Types Reference

### Repository Types

#### `Repo`

The main repository struct that wraps libgit2 functionality.

```rust
pub struct Repo {
    repo: Repository,
    local_path: PathBuf,
}
```

**Fields:**
- `repo`: Internal libgit2 repository handle
- `local_path`: Path to the repository root directory

### File Status Types

#### `GitFileStatus`

Represents the status of a file in Git.

```rust
pub enum GitFileStatus {
    Added,
    Modified,
    Deleted,
    Untracked,
}
```

**Variants:**
- `Added`: File has been added to the repository
- `Modified`: File has been modified
- `Deleted`: File has been deleted
- `Untracked`: File is not tracked by Git

#### `GitChangedFile`

Represents a changed file with detailed status information.

```rust
pub struct GitChangedFile {
    pub path: String,
    pub status: GitFileStatus,
    pub staged: bool,
    pub workdir: bool,
}
```

**Fields:**
- `path`: The path to the changed file
- `status`: The status of the file (Added, Modified, Deleted, or Untracked)
- `staged`: Whether the file is staged in the index
- `workdir`: Whether the file has changes in the working directory

### Commit and Tag Types

#### `RepoCommit`

Represents a commit in the Git repository.

```rust
pub struct RepoCommit {
    pub hash: String,
    pub author_name: String,
    pub author_email: String,
    pub author_date: String,
    pub message: String,
}
```

**Fields:**
- `hash`: The commit hash (SHA)
- `author_name`: The name of the commit author
- `author_email`: The email of the commit author
- `author_date`: The date of the commit in RFC2822 format
- `message`: The commit message

#### `RepoTags`

Represents a tag in the Git repository.

```rust
pub struct RepoTags {
    pub hash: String,
    pub tag: String,
}
```

**Fields:**
- `hash`: The hash of the commit that the tag points to
- `tag`: The name of the tag

### Error Types

#### `RepoError`

Comprehensive error type for all Git operations.

```rust
pub enum RepoError {
    CanonicalPathFailure(std::io::Error),
    GitFailure(git2::Error),
    CreateRepoFailure(git2::Error),
    OpenRepoFailure(git2::Error),
    CloneRepoFailure(git2::Error),
    ConfigError(git2::Error),
    ConfigEntriesError(git2::Error),
    HeadError(git2::Error),
    PeelError(git2::Error),
    BranchError(git2::Error),
    SignatureError(git2::Error),
    IndexError(git2::Error),
    AddFilesError(git2::Error),
    WriteIndexError(git2::Error),
    TreeError(git2::Error),
    CommitError(git2::Error),
    WriteTreeError(git2::Error),
    BranchListError(git2::Error),
    BranchNameError(git2::Error),
    CheckoutBranchError(git2::Error),
    CheckoutError(git2::Error),
    LastTagError(git2::Error),
    CreateTagError(git2::Error),
    StatusError(git2::Error),
    CommitOidError(git2::Error),
    GraphError(git2::Error),
    PushError(git2::Error),
    RemoteError(git2::Error),
    ReferenceError(git2::Error),
    DiffError(git2::Error),
    RevWalkError(git2::Error),
    TagError(git2::Error),
    MergeError(git2::Error),
    MergeConflictError(git2::Error),
}
```

## Error Handling

All operations return `Result<T, RepoError>` to provide comprehensive error information. The error types are specific to the operation that failed, allowing for targeted error handling:

```rust
use sublime_git_tools::{Repo, RepoError};

match repo.checkout("feature-branch") {
    Ok(_) => println!("Switched to feature-branch"),
    Err(RepoError::BranchNameError(_)) => println!("Branch does not exist"),
    Err(RepoError::CheckoutBranchError(_)) => println!("Failed to checkout branch"),
    Err(e) => println!("Other error: {}", e),
}
```

## Examples

### Basic Workflow

```rust
use sublime_git_tools::Repo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create or open repository
    let repo = Repo::create("/tmp/example-repo")?;
    
    // Configure repository
    repo.config("John Doe", "john@example.com")?;
    
    // Create and commit changes
    std::fs::write("/tmp/example-repo/README.md", "# Example Project")?;
    repo.add("README.md")?;
    let commit_id = repo.commit("Initial commit")?;
    
    println!("Created commit: {}", commit_id);
    Ok(())
}
```

### Branch Management

```rust
use sublime_git_tools::Repo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repo::open("./my-project")?;
    
    // Create and switch to feature branch
    repo.create_branch("feature/new-feature")?;
    repo.checkout("feature/new-feature")?;
    
    // Make changes and commit
    repo.add_all()?;
    repo.commit("Add new feature")?;
    
    // Switch back to main and merge
    repo.checkout("main")?;
    repo.merge("feature/new-feature")?;
    
    // List all branches
    let branches = repo.list_branches()?;
    for branch in branches {
        println!("Branch: {}", branch);
    }
    
    Ok(())
}
```

### Tracking Changes

```rust
use sublime_git_tools::Repo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repo::open("./my-project")?;
    
    // Get last tag
    let last_tag = repo.get_last_tag()?;
    println!("Last release: {}", last_tag);
    
    // Get changes since last release
    let changed_files = repo.get_all_files_changed_since_sha_with_status(&last_tag)?;
    println!("Changes since {}:", last_tag);
    
    for file in changed_files {
        let status_char = match file.status {
            GitFileStatus::Added => "+",
            GitFileStatus::Modified => "M",
            GitFileStatus::Deleted => "-",
            GitFileStatus::Untracked => "?",
        };
        println!("{} {}", status_char, file.path);
    }
    
    // Get commit history
    let commits = repo.get_commits_since(Some(last_tag), &None)?;
    println!("\nCommits since last release:");
    for commit in commits {
        println!("- {} ({})", commit.message, &commit.hash[0..7]);
    }
    
    Ok(())
}
```

### Remote Operations Examples

```rust
use sublime_git_tools::Repo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Clone a repository
    let repo = Repo::clone("https://github.com/example/repo.git", "/tmp/cloned-repo")?;
    
    // Configure the repository
    repo.config("Jane Doe", "jane@example.com")?;
    
    // Create a new branch with changes
    repo.create_branch("feature/remote-example")?;
    repo.checkout("feature/remote-example")?;
    
    // Make and commit changes
    std::fs::write("/tmp/cloned-repo/example.txt", "Example content")?;
    repo.add_all()?;
    repo.commit("feat: add example file")?;
    
    // Push the branch to remote
    repo.push("origin", None)?;
    
    // Fetch and pull from remote
    repo.fetch("origin", None, false)?;
    repo.checkout("main")?;
    repo.pull("origin", None)?;
    
    // List remote tags
    let remote_tags = repo.get_remote_or_local_tags(Some(false))?;
    println!("Remote tags:");
    for tag in remote_tags {
        println!("- {} ({})", tag.tag, &tag.hash[0..7]);
    }
    
    Ok(())
}
```
