---
type: Definition
title: 'API Specification: `sublime_git_tools` Crate'
tags: [workspace-tools, rust]
---

# API Specification: `sublime_git_tools` Crate

[Workspace Tools](https://app.capacities.io/f8d3f900-cfe2-4c6d-b44c-906946be3e3c/52c8c019-b006-462b-8c5d-6b518afa0d64)

## Table of Contents

1. Overview (#overview)

2. Architecture (#architecture)

3. Dependencies (#dependencies)

4. API Reference (#api-reference)

    - Core Structures (#core-structures)

    - Repository Operations (#repository-operations)

    - Branch Operations (#branch-operations)

    - Commit Operations (#commit-operations)

    - Tag Operations (#tag-operations)

    - Status Operations (#status-operations)

    - Remote Operations (#remote-operations)

    - Diff and Change Operations (#diff-and-change-operations)

5. Error Handling (#error-handling)

6. Usage Examples (#usage-examples)

7. Known Limitations (#known-limitations)

## Overview

The `sublime_git_tools` crate provides a high-level, Rust-based interface for Git operations. It wraps the lower-level `git2` library (which binds to `libgit2`) to offer a more ergonomic API for common Git operations such as repository management, branch handling, commits, tagging, and remote synchronization.

This crate enables developers to integrate Git operations directly into Rust applications without spawning external Git processes, providing better performance and tighter integration with application logic.

## Architecture

The crate follows a simple architecture built around a central `Repo` struct that encapsulates all Git repository operations. The design emphasizes:

1. **Ergonomic API**: Methods are named intuitively and follow a fluent interface pattern where possible.

2. **Comprehensive Error Handling**: Detailed error types provide context on what operation failed.

3. **Abstraction of Complex Git Operations**: Common Git workflows are simplified into single method calls.

4. **Resource Safety**: All Git resources are properly managed through Rust's ownership system.

Core components include:

- `Repo`: Main struct representing a Git repository

- Helper structs: `GitChangedFile`, `RepoCommit`, `RepoTags`

- Status enum: `GitFileStatus`

- Error handling: `RepoError` enum with comprehensive error variants

## Dependencies

The crate relies on the following external dependencies:

| Dependency    | Version   | Features                                    | Purpose                                              |
| :------------ | :-------- | :------------------------------------------ | :--------------------------------------------------- |
| `thiserror`   | workspace | -                                           | Error handling and error type derivation             |
| `serde`       | workspace | derive                                      | Serialization/deserialization of Git data structures |
| `chrono`      | 0.4       | -                                           | Date and time handling for Git commits               |
| `git2`        | 0.20      | default, vendored-libgit2, vendored-openssl | Core Git functionality                               |
| `libgit2-sys` | *         | ssh, https, vendored, vendored-openssl      | Lower-level Git library bindings                     |
| `dirs`        | 5.0.1     | -                                           | Finding system directories (e.g., for SSH keys)      |

Dev dependencies:

- `tempfile` (3.19.1): For creating temporary directories in tests

- `sublime_standard_tools` (workspace): For utility functions used in tests

## API Reference

### Core Structures

#### `Repo`

The central structure representing a Git repository with high-level operation methods.

```rust
pub struct Repo {
    // Internal fields (private)
}
```

**Construction Methods**:

- `fn create(path: &str) -> Result<Self, RepoError>`

    - Creates a new Git repository at the specified path

    - Initializes with an initial commit on 'main' branch

- `fn open(path: &str) -> Result<Self, RepoError>`

    - Opens an existing Git repository at the specified path

- `fn clone(url: &str, path: &str) -> Result<Self, RepoError>`

    - Clones a Git repository from a URL to a local path

**Basic Methods**:

- `fn get_repo_path(&self) -> &Path`

    - Gets the local path of the repository

- `fn config(&self, username: &str, email: &str) -> Result<&Self, RepoError>`

    - Configures the repository with user information and core settings

#### `GitFileStatus`

Enum representing the status of a file in Git.

```rust
pub enum GitFileStatus {
    Added,    // File has been added to the repository
    Modified, // File has been modified
    Deleted,  // File has been deleted
}
```

#### `GitChangedFile`

Structure representing a changed file in the Git repository.

```rust
pub struct GitChangedFile {
    pub path: String,        // The path to the changed file
    pub status: GitFileStatus // The status of the file
}
```

#### `RepoCommit`

Structure representing a commit in the Git repository.

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

Structure representing a tag in the Git repository.

```rust
pub struct RepoTags {
    pub hash: String, // The hash of the commit that the tag points to
    pub tag: String,  // The name of the tag
}
```

### Repository Operations

Methods for basic repository operations:

- `fn get_repo_path(&self) -> &Path`

    - Gets the local path of the repository

- `fn config(&self, username: &str, email: &str) -> Result<&Self, RepoError>`

    - Configures the repository with user information and core settings

- `fn list_config(&self) -> Result<HashMap<String, String>, RepoError>`

    - Lists all configuration entries for the repository

### Branch Operations

Methods for handling Git branches:

- `fn create_branch(&self, branch_name: &str) -> Result<&Self, RepoError>`

    - Creates a new branch based on the current HEAD

- `fn list_branches(&self) -> Result<Vec<String>, RepoError>`

    - Lists all local branches in the repository

- `fn checkout(&self, branch_name: &str) -> Result<&Self, RepoError>`

    - Checks out a local branch

- `fn get_current_branch(&self) -> Result<String, RepoError>`

    - Gets the name of the currently checked out branch

- `fn get_branch_from_commit(&self, sha: &str) -> Result<Option<String>, RepoError>`

    - Finds the branch that contains a specific commit

- `fn get_branches_containing_commit(&self, sha: &str) -> Result<Vec<String>, RepoError>`

    - Finds all branches that contain a specific commit

- `fn merge(&self, branch_name: &str) -> Result<(), RepoError>`

    - Merges the specified branch into the current HEAD

### Commit Operations

Methods for handling Git commits:

- `fn add(&self, file_path: &str) -> Result<&Self, RepoError>`

    - Adds a file to the Git index

- `fn add_all(&self) -> Result<&Self, RepoError>`

    - Adds all changed files to the Git index

- `fn commit(&self, message: &str) -> Result<String, RepoError>`

    - Creates a new commit with the current index

    - Returns the new commit's SHA

- `fn commit_changes(&self, message: &str) -> Result<String, RepoError>`

    - Adds all changes and creates a new commit

    - Combination of add_all() and commit() in one step

- `fn get_current_sha(&self) -> Result<String, RepoError>`

    - Gets the SHA of the current HEAD commit

- `fn get_previous_sha(&self) -> Result<String, RepoError>`

    - Gets the SHA of the parent of the current HEAD commit

- `fn get_commits_since(&self, since: Option<String>, relative: &Option<String>) -> Result<Vec<RepoCommit>, RepoError>`

    - Gets commits made since a specific reference or from the beginning

    - Can filter by path with the relative parameter

### Tag Operations

Methods for handling Git tags:

- `fn create_tag(&self, tag: &str, message: Option<String>) -> Result<&Self, RepoError>`

    - Creates a new tag at the current HEAD

- `fn get_last_tag(&self) -> Result<String, RepoError>`

    - Gets the name of the last tag in the repository

- `fn get_remote_or_local_tags(&self, local: Option<bool>) -> Result<Vec<RepoTags>, RepoError>`

    - Gets tags from either local repository or remote

### Status Operations

Methods for checking repository status:

- `fn status_porcelain(&self) -> Result<Vec<String>, RepoError>`

    - Gets the status of the repository in porcelain format

    - Returns a list of changed file paths

### Remote Operations

Methods for interacting with remote repositories:

- `fn push(&self, remote_name: &str, follow_tags: Option<bool>) -> Result<bool, RepoError>`

    - Pushes the current branch to a remote repository

- `fn push_with_ssh_config(&self, remote_name: &str, follow_tags: Option<bool>, ssh_key_paths: Vec<PathBuf>) -> Result<bool, RepoError>`

    - Pushes the current branch with custom SSH key paths

- `fn fetch(&self, remote_name: &str, refspecs: Option<&[&str]>, prune: bool) -> Result<bool, RepoError>`

    - Fetches changes from a remote repository

- `fn pull(&self, remote_name: &str, branch_name: Option<&str>) -> Result<bool, RepoError>`

    - Pulls changes from a remote repository

### Diff and Change Operations

Methods for examining differences and changes:

- `fn get_diverged_commit(&self, git_ref: &str) -> Result<String, RepoError>`

    - Finds the common ancestor (merge base) between HEAD and another reference

- `fn get_all_files_changed_since_sha_with_status(&self, git_ref: &str) -> Result<Vec<GitChangedFile>, RepoError>`

    - Gets all files changed since a specific reference with their status

- `fn get_all_files_changed_since_sha(&self, git_ref: &str) -> Result<Vec<String>, RepoError>`

    - Gets all files changed since a specific reference

- `fn get_all_files_changed_since_branch(&self, packages_paths: &[String], branch: &str) -> Result<Vec<String>, RepoError>`

    - Gets all files changed since a specific branch within specified package paths

## Error Handling

The crate uses a comprehensive error handling approach through the `RepoError` enum:

```rust
pub enum RepoError {
    CanonicPathFailure(std::io::Error),
    GitFailure(Git2Error),
    CreateRepoFailure(Git2Error),
    OpenRepoFailure(Git2Error),
    CloneRepoFailure(Git2Error),
    ConfigError(Git2Error),
    ConfigEntriesError(Git2Error),
    HeadError(Git2Error),
    PeelError(Git2Error),
    BranchError(Git2Error),
    SignatureError(Git2Error),
    IndexError(Git2Error),
    AddFilesError(Git2Error),
    WriteIndexError(Git2Error),
    TreeError(Git2Error),
    CommitError(Git2Error),
    WriteTreeError(Git2Error),
    BranchListError(Git2Error),
    BranchNameError(Git2Error),
    CheckoutBranchError(Git2Error),
    CheckoutError(Git2Error),
    LastTagError(Git2Error),
    CreateTagError(Git2Error),
    StatusError(Git2Error),
    CommitOidError(Git2Error),
    GraphError(Git2Error),
    PushError(Git2Error),
    RemoteError(Git2Error),
    ReferenceError(Git2Error),
    DiffError(Git2Error),
    RevWalkError(Git2Error),
    TagError(Git2Error),
    MergeError(Git2Error),
    MergeConflictError(Git2Error),
}
```

Each error variant provides context about what specific Git operation failed, allowing for detailed error handling and reporting.

## Usage Examples

### Basic Repository Operations

```rust
use sublime_git_tools::Repo;
// Open an existing repository
let repo = Repo::open("./my-repo").expect("Failed to open repository");
// Configure the repository
repo.config("John Doe", "john@example.com").expect("Failed to configure repo");
// Create a new branch
repo.create_branch("feature/new-feature").expect("Failed to create branch");
// Checkout the new branch
repo.checkout("feature/new-feature").expect("Failed to checkout branch");
// Make some changes to files...
// Add all changes
repo.add_all().expect("Failed to add changes");
// Commit the changes
let commit_id = repo.commit("feat: add new feature").expect("Failed to commit");
println!("Created commit: {}", commit_id);
// Push to remote
repo.push("origin", Some(true)).expect("Failed to push to origin");
```

### Working with Commits and Diffs

```rust
use sublime_git_tools::Repo;
let repo = Repo::open("./my-repo").expect("Failed to open repository");
// Get the current commit SHA
let current_sha = repo.get_current_sha().expect("Failed to get current SHA");
println!("Current commit: {}", current_sha);
// Get commits since a specific tag
let commits = repo.get_commits_since(
    Some("v1.0.0".to_string()),
    &None
).expect("Failed to get commits");
for commit in commits {
    println!("{}: {} (by {} on {})",
        &commit.hash[0..8],
        commit.message,
        commit.author_name,
        commit.author_date
    );
}
// Get files changed since a commit
let changed_files = repo.get_all_files_changed_since_sha_with_status(&current_sha)
    .expect("Failed to get changed files");
for file in changed_files {
    let status_str = match file.status {
        GitFileStatus::Added => "added",
        GitFileStatus::Modified => "modified",
        GitFileStatus::Deleted => "deleted",
    };
    println!("{}: {}", file.path, status_str);
}
```

### Working with Remotes

```rust
use sublime_git_tools::Repo;
use std::path::PathBuf;
let repo = Repo::open("./my-repo").expect("Failed to open repository");
// Fetch from remote
repo.fetch("origin", None, true).expect("Failed to fetch from origin");
// Pull from remote
repo.pull("origin", None).expect("Failed to pull from origin");
// Push with custom SSH keys
let ssh_keys = vec![
    PathBuf::from("/path/to/custom/id_ed25519"),
    PathBuf::from("/path/to/custom/id_rsa"),
];
repo.push_with_ssh_config("origin", Some(true), ssh_keys)
    .expect("Failed to push with custom SSH config");
```

### Working with Monorepos

```rust
use sublime_git_tools::Repo;
let repo = Repo::open("./monorepo").expect("Failed to open repository");
// Get changed files in specific packages since a branch
let packages = vec![
    "packages/frontend".to_string(),
    "packages/api".to_string(),
];
let changed_files = repo.get_all_files_changed_since_branch(
    &packages,
    "main"
).expect("Failed to get changed files");
println!("Changed files in packages:");
for file in changed_files {
    println!("  {}", file);
}
```

## Known Limitations

1. **SSH Agent Limitations**: The crate attempts to use SSH keys in standard locations but may not work with all SSH agent configurations. Custom SSH key paths can be provided via `push_with_ssh_config`.

2. **Authentication Support**: Currently focuses primarily on SSH authentication. HTTPS authentication with username/password is not fully implemented.

3. **Merge Conflict Resolution**: While the crate can detect merge conflicts, it does not provide tools for resolving them automatically.

4. **Performance with Large Repositories**: For very large repositories, some operations that scan the entire history may be slow.

5. **Git Submodules**: Limited support for managing Git submodules.

6. **Windows Path Handling**: Path handling on Windows may need special care when working with mixed path separators.

7. **Remote Tags**: The `get_remote_tags` method only works with the 'origin' remote.


