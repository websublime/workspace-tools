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
- [File Change Detection](#file-change-detection)
  - [Changed Files](#changed-files)
  - [Package-specific Changes](#package-specific-changes)
- [Remote Operations](#remote-operations)
  - [Pushing and Pulling](#pushing-and-pulling)
  - [Fetching](#fetching)
  - [Merging](#merging)
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
let repo = Repo::create("/path/to/new/repo").expect("Failed to create repository");
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
let repo = Repo::open("./my-project").expect("Failed to open repository");
let branch = repo.get_current_branch().expect("Failed to get current branch");
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
let repo = Repo::clone("https://github.com/example/repo.git", "./cloned-repo")
    .expect("Failed to clone repository");
```

**Possible errors:**
- `CanonicalPathFailure`: Failed to canonicalize the provided path
- `CloneRepoFailure`: Failed to clone the Git repository

### Repository Information

#### `Repo::get_repo_path`

Gets the local path of the repository.

```rust
pub fn get_repo_path(&self) -> &Path
```

**Returns:**
- `&Path`: The path to the repository

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
println!("Repository path: {}", repo.get_repo_path().display());
```

### Configuration

#### `Repo::config`

Configures the repository with user information and core settings.

```rust
pub fn config(&self, username: &str, email: &str) -> Result<&Self, RepoError>
```

**Parameters:**
- `username`: The Git user name
- `email`: The Git user email

**Returns:**
- `Result<&Self, RepoError>`: A reference to self for method chaining, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
repo.config("Jane Doe", "jane@example.com").expect("Failed to configure repository");
```

**Possible errors:**
- `ConfigError`: Failed to access or update the Git configuration

#### `Repo::list_config`

Lists all configuration entries for the repository.

```rust
pub fn list_config(&self) -> Result<HashMap<String, String>, RepoError>
```

**Returns:**
- `Result<HashMap<String, String>, RepoError>`: A map of config keys to values, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let config = repo.list_config().expect("Failed to list config");
for (key, value) in config {
    println!("{} = {}", key, value);
}
```

**Possible errors:**
- `ConfigError`: Failed to access Git configuration
- `ConfigEntriesError`: Failed to retrieve configuration entries

## Branch Operations

### Creating and Managing Branches

#### `Repo::create_branch`

Creates a new branch based on the current HEAD.

```rust
pub fn create_branch(&self, branch_name: &str) -> Result<&Self, RepoError>
```

**Parameters:**
- `branch_name`: The name for the new branch

**Returns:**
- `Result<&Self, RepoError>`: A reference to self for method chaining, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
repo.create_branch("feature/new-feature").expect("Failed to create branch");
```

**Possible errors:**
- `HeadError`: Failed to get the current HEAD
- `PeelError`: Failed to peel reference to a commit
- `BranchError`: Failed to create branch

#### `Repo::checkout`

Checks out a local branch.

```rust
pub fn checkout(&self, branch_name: &str) -> Result<&Self, RepoError>
```

**Parameters:**
- `branch_name`: The name of the branch to checkout

**Returns:**
- `Result<&Self, RepoError>`: A reference to self for method chaining, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
repo.checkout("feature-branch").expect("Failed to checkout branch");
```

**Possible errors:**
- `CheckoutBranchError`: Failed to find or checkout the branch
- `BranchNameError`: Invalid branch name

### Branch Information

#### `Repo::list_branches`

Lists all local branches in the repository.

```rust
pub fn list_branches(&self) -> Result<Vec<String>, RepoError>
```

**Returns:**
- `Result<Vec<String>, RepoError>`: A list of branch names, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let branches = repo.list_branches().expect("Failed to list branches");
for branch in branches {
    println!("Branch: {}", branch);
}
```

**Possible errors:**
- `BranchListError`: Failed to list branches

#### `Repo::get_current_branch`

Gets the name of the currently checked out branch.

```rust
pub fn get_current_branch(&self) -> Result<String, RepoError>
```

**Returns:**
- `Result<String, RepoError>`: The current branch name, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let branch = repo.get_current_branch().expect("Failed to get current branch");
println!("Current branch: {}", branch);
```

**Possible errors:**
- `HeadError`: Failed to get repository HEAD
- `BranchNameError`: Failed to get branch name

#### `Repo::get_branch_from_commit`

Finds the branch that contains a specific commit.

```rust
pub fn get_branch_from_commit(&self, sha: &str) -> Result<Option<String>, RepoError>
```

**Parameters:**
- `sha`: The commit SHA to find

**Returns:**
- `Result<Option<String>, RepoError>`: The branch name if found, None if not found, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let commit_sha = repo.get_current_sha().expect("Failed to get current SHA");

match repo.get_branch_from_commit(&commit_sha) {
    Ok(Some(branch)) => println!("Commit {} is in branch: {}", commit_sha, branch),
    Ok(None) => println!("Commit {} is not in any branch", commit_sha),
    Err(e) => println!("Error: {}", e),
}
```

**Possible errors:**
- `CommitOidError`: Failed to parse commit SHA
- `BranchListError`: Failed to list branches
- `BranchNameError`: Failed to get branch name
- `PeelError`: Failed to peel reference to commit
- `GraphError`: Failed on repository graph operations

#### `Repo::get_branches_containing_commit`

Finds all branches that contain a specific commit.

```rust
pub fn get_branches_containing_commit(&self, sha: &str) -> Result<Vec<String>, RepoError>
```

**Parameters:**
- `sha`: The commit SHA to find

**Returns:**
- `Result<Vec<String>, RepoError>`: List of branch names containing the commit, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let commit_sha = repo.get_current_sha().expect("Failed to get current SHA");

let branches = repo.get_branches_containing_commit(&commit_sha)
    .expect("Failed to find branches");
for branch in branches {
    println!("Branch contains commit: {}", branch);
}
```

**Possible errors:**
- Similar to `get_branch_from_commit`

#### `Repo::get_diverged_commit`

Finds the common ancestor (merge base) between HEAD and another reference.

```rust
pub fn get_diverged_commit(&self, git_ref: &str) -> Result<String, RepoError>
```

**Parameters:**
- `git_ref`: The reference to compare with HEAD (branch name, tag, or commit SHA)

**Returns:**
- `Result<String, RepoError>`: The SHA of the common ancestor commit, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let merge_base = repo.get_diverged_commit("feature-branch")
    .expect("Failed to find common ancestor");
println!("Common ancestor commit: {}", merge_base);
```

**Possible errors:**
- `ReferenceError`: Failed to parse reference
- `PeelError`: Failed to peel reference to commit
- `HeadError`: Failed to get repository HEAD
- `MergeError`: Failed to find merge base

## Commit Operations

### Staging and Committing

#### `Repo::add`

Adds a file to the Git index.

```rust
pub fn add(&self, file_path: &str) -> Result<&Self, RepoError>
```

**Parameters:**
- `file_path`: The path to the file to add

**Returns:**
- `Result<&Self, RepoError>`: A reference to self for method chaining, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
repo.add("src/main.rs").expect("Failed to add file");
```

**Possible errors:**
- `IndexError`: Failed to get or manipulate the index

#### `Repo::add_all`

Adds all changed files to the Git index.

```rust
pub fn add_all(&self) -> Result<&Self, RepoError>
```

**Returns:**
- `Result<&Self, RepoError>`: A reference to self for method chaining, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
repo.add_all().expect("Failed to add all changes");
```

**Possible errors:**
- `IndexError`: Failed to get or manipulate the index

#### `Repo::commit`

Creates a new commit with the current index.

```rust
pub fn commit(&self, message: &str) -> Result<String, RepoError>
```

**Parameters:**
- `message`: The commit message

**Returns:**
- `Result<String, RepoError>`: The new commit's SHA, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
// First add some files
repo.add("src/main.rs").expect("Failed to add file");
// Then commit
let commit_id = repo.commit("fix: update main.rs").expect("Failed to commit");
println!("Created commit: {}", commit_id);
```

**Possible errors:**
- `SignatureError`: Failed to get signature
- `HeadError`: Failed to get HEAD
- `PeelError`: Failed to peel reference to commit
- `IndexError`: Failed to get index
- `WriteTreeError`: Failed to write tree
- `TreeError`: Failed to find tree
- `CommitError`: Failed to create commit

#### `Repo::commit_changes`

Adds all changes and creates a new commit in one step.

```rust
pub fn commit_changes(&self, message: &str) -> Result<String, RepoError>
```

**Parameters:**
- `message`: The commit message

**Returns:**
- `Result<String, RepoError>`: The new commit's SHA, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let commit_id = repo.commit_changes("feat: add new feature").expect("Failed to commit changes");
println!("Created commit: {}", commit_id);
```

**Possible errors:**
- Similar to `commit` plus `AddFilesError`

### Commit Information

#### `Repo::get_current_sha`

Gets the SHA of the current HEAD commit.

```rust
pub fn get_current_sha(&self) -> Result<String, RepoError>
```

**Returns:**
- `Result<String, RepoError>`: The current commit SHA, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let sha = repo.get_current_sha().expect("Failed to get current SHA");
println!("Current commit: {}", sha);
```

**Possible errors:**
- `HeadError`: Failed to get repository HEAD

#### `Repo::get_previous_sha`

Gets the SHA of the parent of the current HEAD commit.

```rust
pub fn get_previous_sha(&self) -> Result<String, RepoError>
```

**Returns:**
- `Result<String, RepoError>`: The previous commit SHA, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let prev_sha = repo.get_previous_sha().expect("Failed to get previous SHA");
println!("Previous commit: {}", prev_sha);
```

**Possible errors:**
- `HeadError`: Failed to get repository HEAD
- `PeelError`: Failed to peel reference to commit
- `GitFailure`: Failed to get parent commit

#### `Repo::status_porcelain`

Gets the status of the repository in porcelain format (a list of changed file paths).

```rust
pub fn status_porcelain(&self) -> Result<Vec<String>, RepoError>
```

**Returns:**
- `Result<Vec<String>, RepoError>`: List of changed file paths, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let status = repo.status_porcelain().expect("Failed to get status");
for file in status {
    println!("Changed file: {}", file);
}
```

**Possible errors:**
- `StatusError`: Failed to get repository status

### Commit History

#### `Repo::get_commits_since`

Gets commits made since a specific reference or from the beginning.

```rust
pub fn get_commits_since(&self, since: Option<String>, relative: &Option<String>) -> Result<Vec<RepoCommit>, RepoError>
```

**Parameters:**
- `since`: Optional reference to start from (branch, tag, or commit SHA)
- `relative`: Optional path to filter commits by (only commits touching this path)

**Returns:**
- `Result<Vec<RepoCommit>, RepoError>`: List of commits, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");

// Get all commits since v1.0.0
let commits = repo.get_commits_since(
    Some("v1.0.0".to_string()),
    &None
).expect("Failed to get commits");

// Get all commits that touched a specific file
let file_commits = repo.get_commits_since(
    None,
    &Some("src/main.rs".to_string())
).expect("Failed to get commits");

for commit in commits {
    println!("{}: {} ({})",
        commit.hash,
        commit.message,
        commit.author_name
    );
}
```

**Possible errors:**
- `GitFailure`: General Git operation failure
- `RevWalkError`: Failed on revision walking
- `ReferenceError`: Failed to parse reference
- `PeelError`: Failed to peel reference to commit
- `CommitOidError`: Failed to parse commit SHA
- `CommitError`: Failed to find commit

## Tag Operations

### Creating Tags

#### `Repo::create_tag`

Creates a new tag at the current HEAD.

```rust
pub fn create_tag(&self, tag: &str, message: Option<String>) -> Result<&Self, RepoError>
```

**Parameters:**
- `tag`: The name for the new tag
- `message`: Optional message for the tag

**Returns:**
- `Result<&Self, RepoError>`: A reference to self for method chaining, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
repo.create_tag("v1.0.0", Some("Version 1.0.0 release".to_string()))
    .expect("Failed to create tag");
```

**Possible errors:**
- `SignatureError`: Failed to get signature
- `HeadError`: Failed to get HEAD
- `CreateTagError`: Failed to create tag

### Tag Information

#### `Repo::get_last_tag`

Gets the name of the last tag in the repository.

```rust
pub fn get_last_tag(&self) -> Result<String, RepoError>
```

**Returns:**
- `Result<String, RepoError>`: The last tag name, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
match repo.get_last_tag() {
    Ok(tag) => println!("Last tag: {}", tag),
    Err(e) => println!("No tags found or error: {}", e),
}
```

**Possible errors:**
- `LastTagError`: Failed to get or find tags

#### `Repo::get_remote_or_local_tags`

Gets tags from either local repository or remote.

```rust
pub fn get_remote_or_local_tags(&self, local: Option<bool>) -> Result<Vec<RepoTags>, RepoError>
```

**Parameters:**
- `local`: If Some(true), gets local tags; if Some(false) or None, gets remote tags

**Returns:**
- `Result<Vec<RepoTags>, RepoError>`: List of tags, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");

// Get local tags
let local_tags = repo.get_remote_or_local_tags(Some(true))
    .expect("Failed to get local tags");

// Get remote tags (default)
let remote_tags = repo.get_remote_or_local_tags(None)
    .expect("Failed to get remote tags");

for tag in local_tags {
    println!("Tag: {} ({})", tag.tag, tag.hash);
}
```

**Possible errors:**
- `ReferenceError`: Failed on reference parsing
- `TagError`: Failed on tag operations
- `RemoteError`: Failed on remote operations

## File Change Detection

### Changed Files

#### `Repo::get_all_files_changed_since_sha_with_status`

Gets all files changed since a specific reference with their status.

```rust
pub fn get_all_files_changed_since_sha_with_status(&self, git_ref: &str) -> Result<Vec<GitChangedFile>, RepoError>
```

**Parameters:**
- `git_ref`: The reference to compare with HEAD (branch name, tag, or commit SHA)

**Returns:**
- `Result<Vec<GitChangedFile>, RepoError>`: List of changed files with status, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let changed_files = repo.get_all_files_changed_since_sha_with_status("v1.0.0")
    .expect("Failed to get changed files");

for file in changed_files {
    println!("File: {} - {:?}", file.path, file.status);
}
```

**Possible errors:**
- `ReferenceError`: Failed to parse reference
- `PeelError`: Failed to peel reference to commit
- `GitFailure`: General Git operation failure
- `DiffError`: Failed on diff operations

#### `Repo::get_all_files_changed_since_sha`

Gets all files changed since a specific reference (paths only, no status).

```rust
pub fn get_all_files_changed_since_sha(&self, git_ref: &str) -> Result<Vec<String>, RepoError>
```

**Parameters:**
- `git_ref`: The reference to compare with HEAD (branch name, tag, or commit SHA)

**Returns:**
- `Result<Vec<String>, RepoError>`: List of changed file paths, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let changed_files = repo.get_all_files_changed_since_sha("v1.0.0")
    .expect("Failed to get changed files");

for file in changed_files {
    println!("Changed file: {}", file);
}
```

**Possible errors:**
- Same as `get_all_files_changed_since_sha_with_status`

### Package-specific Changes

#### `Repo::get_all_files_changed_since_branch`

Gets all files changed since a specific branch within specified package paths.

```rust
pub fn get_all_files_changed_since_branch(&self, packages_paths: &[String], branch: &str) -> Result<Vec<String>, RepoError>
```

**Parameters:**
- `packages_paths`: List of package paths to filter by
- `branch`: The branch to compare against

**Returns:**
- `Result<Vec<String>, RepoError>`: List of changed file paths within the packages, or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let packages = vec!["packages/pkg1".to_string(), "packages/pkg2".to_string()];
let changed_files = repo.get_all_files_changed_since_branch(&packages, "main")
    .expect("Failed to get changed files");

for file in changed_files {
    println!("Changed file: {}", file);
}
```

**Possible errors:**
- Same as `get_all_files_changed_since_sha`

## Remote Operations

### Pushing and Pulling

#### `Repo::push`

Pushes the current branch to a remote repository.

```rust
pub fn push(&self, remote_name: &str, follow_tags: Option<bool>) -> Result<bool, RepoError>
```

**Parameters:**
- `remote_name`: The name of the remote (e.g., "origin")
- `follow_tags`: Whether to also push tags

**Returns:**
- `Result<bool, RepoError>`: Success indicator or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
// Push current branch with tags
repo.push("origin", Some(true)).expect("Failed to push");
```

**Possible errors:**
- `HeadError`: Failed to get repository HEAD
- `BranchNameError`: Failed to get branch name
- `RemoteError`: Failed on remote operations
- `PushError`: Failed to push to remote

#### `Repo::push_with_ssh_config`

Pushes the current branch to a remote repository with custom SSH key paths.

```rust
pub fn push_with_ssh_config(&self, remote_name: &str, follow_tags: Option<bool>, ssh_key_paths: Vec<PathBuf>) -> Result<bool, RepoError>
```

**Parameters:**
- `remote_name`: The name of the remote (e.g., "origin")
- `follow_tags`: Whether to also push tags
- `ssh_key_paths`: Paths to SSH keys to try for authentication

**Returns:**
- `Result<bool, RepoError>`: Success indicator or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
let key_paths = vec![
    PathBuf::from("/custom/path/to/id_ed25519"),
    PathBuf::from("/custom/path/to/id_rsa"),
];
repo.push_with_ssh_config("origin", Some(true), key_paths).expect("Failed to push");
```

**Possible errors:**
- Same as `push`

#### `Repo::pull`

Pulls changes from a remote repository. This fetches from the remote and merges the changes into the current branch.

```rust
pub fn pull(&self, remote_name: &str, branch_name: Option<&str>) -> Result<bool, RepoError>
```

**Parameters:**
- `remote_name`: The name of the remote (e.g., "origin")
- `branch_name`: Optional branch name to pull from (defaults to tracking branch)

**Returns:**
- `Result<bool, RepoError>`: Success indicator or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
// Pull from the tracking branch
repo.pull("origin", None).expect("Failed to pull");

// Pull from a specific branch
repo.pull("origin", Some("feature-branch")).expect("Failed to pull from feature branch");
```

**Possible errors:**
- `RemoteError`: Failed on remote operations
- `BranchNameError`: Failed to get branch name
- `PeelError`: Failed to peel reference to commit
- `MergeError`: Failed on merge operations
- `IndexError`: Failed to get or manipulate the index
- `WriteTreeError`: Failed to write tree
- `TreeError`: Failed to find tree
- `SignatureError`: Failed to get signature
- `CommitError`: Failed to create commit
- `MergeConflictError`: Merge conflicts detected

### Fetching

#### `Repo::fetch`

Fetches changes from a remote repository.

```rust
pub fn fetch(&self, remote_name: &str, refspecs: Option<&[&str]>, prune: bool) -> Result<bool, RepoError>
```

**Parameters:**
- `remote_name`: The name of the remote (e.g., "origin")
- `refspecs`: Optional reference specs to fetch
- `prune`: Whether to prune deleted references

**Returns:**
- `Result<bool, RepoError>`: Success indicator or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
// Fetch with default refspecs and no pruning
repo.fetch("origin", None, false).expect("Failed to fetch");

// Fetch a specific branch and prune
repo.fetch("origin", Some(&["refs/heads/main:refs/remotes/origin/main"]), true)
    .expect("Failed to fetch specific branch");
```

**Possible errors:**
- `RemoteError`: Failed on remote operations

### Merging

#### `Repo::merge`

Merges the specified branch into the current HEAD.

```rust
pub fn merge(&self, branch_name: &str) -> Result<(), RepoError>
```

**Parameters:**
- `branch_name`: The name of the branch to merge into the current branch

**Returns:**
- `Result<(), RepoError>`: Success or an error

**Example:**
```rust
let repo = Repo::open("./my-repo").expect("Failed to open repository");
repo.checkout("main").expect("Failed to checkout main");
repo.merge("feature-branch").expect("Failed to merge feature branch");
```

**Possible errors:**
- `HeadError`: Failed to get repository HEAD
- `PeelError`: Failed to peel reference to commit
- `ReferenceError`: Failed on reference parsing
- `CommitError`: Failed to create or find commit
- `MergeError`: Failed on merge operations
- `CheckoutError`: Failed on checkout
- `IndexError`: Failed to get or manipulate the index
- `WriteTreeError`: Failed to write tree
- `TreeError`: Failed to find tree
- `SignatureError`: Failed to get signature
- `MergeConflictError`: Merge conflicts detected

## Types Reference

### Repository Types

#### `Repo`

Represents a Git repository with high-level operation methods.

```rust
pub struct Repo {
    repo: Rc<Repository>,
    local_path: PathBuf,
}
```

**Description:**
- `repo`: The internal libgit2 repository (wrapped in an `Rc` for reference counting)
- `local_path`: The path to the repository on the local filesystem

### File Status Types

#### `GitFileStatus`

Represents the status of a file in Git.

```rust
pub enum GitFileStatus {
    Added,    // File has been added to the repository
    Modified, // File has been modified
    Deleted,  // File has been deleted
}
```

#### `GitChangedFile`

Represents a changed file in the Git repository.

```rust
pub struct GitChangedFile {
    pub path: String,         // The path to the changed file
    pub status: GitFileStatus, // The status of the file (Added, Modified, or Deleted)
}
```

### Commit and Tag Types

#### `RepoCommit`

Represents a commit in the Git repository.

```rust
pub struct RepoCommit {
    pub hash: String,         // The commit hash (SHA)
    pub author_name: String,  // The name of the commit author
    pub author_email: String, // The email of the commit author
    pub author_date: String,  // The date of the commit in RFC2822 format
    pub message: String,      // The commit message
}
```

#### `RepoTags`

Represents a tag in the Git repository.

```rust
pub struct RepoTags {
    pub hash: String, // The hash of the commit that the tag points to
    pub tag: String,  // The name of the tag
}
```

### Error Types

#### `RepoError`

Errors that can occur when working with Git repositories.

```rust
pub enum RepoError {
    CanonicalPathFailure(std::io::Error),     // Failed to canonicalize a path
    GitFailure(Git2Error),                     // Generic Git operation failure
    CreateRepoFailure(Git2Error),              // Failed to create a new repository
    OpenRepoFailure(Git2Error),                // Failed to open an existing repository
    CloneRepoFailure(Git2Error),               // Failed to clone a repository
    ConfigError(Git2Error),                    // Git configuration error
    ConfigEntriesError(Git2Error),             // Failed to retrieve configuration entries
    HeadError(Git2Error),                      // Failed to get repository HEAD
    PeelError(Git2Error),                      // Failed to peel a reference to a commit
    BranchError(Git2Error),                    // Failed to create or manipulate a branch
    SignatureError(Git2Error),                 // Failed to get repository signature
    IndexError(Git2Error),                     // Failed to get or manipulate the index
    AddFilesError(Git2Error),                  // Failed to add files to the index
    WriteIndexError(Git2Error),                // Failed to write the index
    TreeError(Git2Error),                      // Failed to find or manipulate a tree
    CommitError(Git2Error),                    // Failed to create a commit
    WriteTreeError(Git2Error),                 // Failed to write a tree
    BranchListError(Git2Error),                // Failed to list branches
    BranchNameError(Git2Error),                // Failed to get a branch name
    CheckoutBranchError(Git2Error),            // Failed to checkout a branch
    CheckoutError(Git2Error),                  // Failed to checkout
    LastTagError(Git2Error),                   // Failed to get the last tag
    CreateTagError(Git2Error),                 // Failed to create a tag
    StatusError(Git2Error),                    // Failed to get repository status
    CommitOidError(Git2Error),                 // Failed to parse a commit SHA
    GraphError(Git2Error),                     // Failed on repository graph operations
    PushError(Git2Error),                      // Failed to push to a remote
    RemoteError(Git2Error),                    // Failed on remote operations
    ReferenceError(Git2Error),                 // Failed on reference parsing
    DiffError(Git2Error),                      // Failed on diff operations
    RevWalkError(Git2Error),                   // Failed on revision walking
    TagError(Git2Error),                       // Failed on tag operations
    MergeError(Git2Error),                     // Failed on merge operations
    MergeConflictError(Git2Error),             // Failed due to merge conflicts
}
```

## Error Handling

The crate provides detailed error types through `RepoError` which wraps the underlying libgit2 errors. This allows for precise error handling in your application code.

```rust
match repo.checkout("feature-branch") {
    Ok(_) => println!("Switched to feature-branch"),
    Err(e) => match e {
        RepoError::BranchNameError(_) => eprintln!("Branch does not exist"),
        RepoError::CheckoutBranchError(_) => eprintln!("Failed to checkout branch"),
        _ => eprintln!("Error: {}", e),
    }
}
```

## Examples

### Basic Workflow

```rust
use sublime_git_tools::Repo;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new repository
    let repo = Repo::create("/tmp/example-repo")?;
    repo.config("John Doe", "john@example.com")?;
    
    // Create a file and make an initial commit
    let mut file = File::create("/tmp/example-repo/README.md")?;
    writeln!(file, "# Example Repository\n\nThis is an example repository.")?;
    
    repo.add_all()?;
    let commit_id = repo.commit("docs: add README")?;
    println!("Created commit: {}", commit_id);
    
    // Create a tag for this initial version
    repo.create_tag("v0.1.0", Some("Initial release".to_string()))?;
    
    Ok(())
}
```

### Branch Management

```rust
use sublime_git_tools::Repo;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open an existing repository
    let repo = Repo::open("/tmp/example-repo")?;
    
    // Create a feature branch
    repo.create_branch("feature/new-feature")?;
    repo.checkout("feature/new-feature")?;
    
    // Make changes on the feature branch
    let mut file = File::create("/tmp/example-repo/feature.txt")?;
    writeln!(file, "This is a new feature")?;
    
    repo.add_all()?;
    repo.commit("feat: add new feature")?;
    
    // Go back to main branch
    repo.checkout("main")?;
    
    // Merge the feature
    repo.merge("feature/new-feature")?;
    
    // List all branches
    let branches = repo.list_branches()?;
    println!("Branches in repository:");
    for branch in branches {
        println!("- {}", branch);
    }
    
    Ok(())
}
```

### Tracking Changes

```rust
use sublime_git_tools::Repo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open an existing repository
    let repo = Repo::open("/tmp/example-repo")?;
    
    // Get the last tag
    let last_tag = repo.get_last_tag()?;
    
    // Get all files changed since that tag
    let changed_files = repo.get_all_files_changed_since_sha(&last_tag)?;
    
    println!("Files changed since {}:", last_tag);
    for file in &changed_files {
        println!("- {}", file);
    }
    
    // Get detailed changes with status
    let changes_with_status = repo.get_all_files_changed_since_sha_with_status(&last_tag)?;
    
    println!("\nDetailed changes:");
    for change in changes_with_status {
        let status = match change.status {
            sublime_git_tools::GitFileStatus::Added => "Added",
            sublime_git_tools::GitFileStatus::Modified => "Modified",
            sublime_git_tools::GitFileStatus::Deleted => "Deleted",
        };
        println!("- {} ({})", change.path, status);
    }
    
    // Get commit history
    let commits = repo.get_commits_since(Some(last_tag), &None)?;
    
    println!("\nCommits since {}:", last_tag);
    for commit in commits {
        println!("{}: {} (by {} on {})",
            &commit.hash[0..7],
            commit.message.lines().next().unwrap_or(""),
            commit.author_name,
            commit.author_date
        );
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
    let remote_tags = repo.get_remote_or_local_tags(None)?;
    println!("Remote tags:");
    for tag in remote_tags {
        println!("- {} ({})", tag.tag, &tag.hash[0..7]);
    }
    
    Ok(())
}
```

