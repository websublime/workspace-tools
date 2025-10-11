# sublime_git_tools

A high-level Rust interface to Git operations with robust error handling, built on libgit2.

## Overview

`sublime_git_tools` provides a user-friendly API for working with Git repositories. It wraps the 
powerful but complex libgit2 library to offer a more ergonomic interface for common Git operations.

This crate is designed for Rust applications that need to:

- Create, clone, or manipulate Git repositories
- Manage branches, commits, and tags
- Track file changes between commits or branches
- Push/pull with remote repositories
- Get detailed commit histories
- Detect changes in specific parts of a repository
- Handle SSH authentication for remote operations
- Perform advanced Git operations like merging and status checking

## Table of Contents

- [Repository Management](#repository-management)
- [Configuration](#configuration)
- [Branch Operations](#branch-operations)
- [Commit Operations](#commit-operations)
- [Tag Operations](#tag-operations)
- [File Status and Change Detection](#file-status-and-change-detection)
- [Remote Operations](#remote-operations)
- [Advanced Git Operations](#advanced-git-operations)
- [Error Handling](#error-handling)
- [Cross-Platform Support](#cross-platform-support)

## API Reference

For complete API documentation with detailed method signatures, parameters, return types, and comprehensive examples, see the [SPEC.md](SPEC.md) file.

## Repository Management

### Creating, Opening, and Cloning Repositories

```rust
use sublime_git_tools::Repo;

// Create a new repository
let repo = Repo::create("/path/to/new/repo")?;
println!("Repository created at: {}", repo.get_repo_path().display());

// Open an existing repository
let repo = Repo::open("./my-project")?;

// Clone a remote repository
let repo = Repo::clone("https://github.com/example/repo.git", "./cloned-repo")?;

// Get repository path
let repo_path = repo.get_repo_path();
println!("Repository is located at: {}", repo_path.display());
```

## Configuration

### Setting and Viewing Git Configuration

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project")?;

// Configure user name and email
repo.config("Jane Doe", "jane@example.com")?;

// List all configuration entries
let config_entries = repo.list_config()?;
for (key, value) in config_entries {
    println!("{}: {}", key, value);
}
```

## Branch Operations

### Creating and Managing Branches

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project")?;

// Create a new branch
repo.create_branch("feature/new-feature")?;

// Check if a branch exists
if repo.branch_exists("feature/new-feature")? {
    println!("Branch exists!");
}

// Checkout a branch
repo.checkout("feature/new-feature")?;

// Get current branch name
let current_branch = repo.get_current_branch()?;
println!("Currently on branch: {}", current_branch);

// List all branches
let branches = repo.list_branches()?;
for branch in branches {
    println!("Branch: {}", branch);
}
```

### Advanced Branch Information

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project")?;

// Get branch from a specific commit
let commit_sha = "abcdef123456";
if let Some(branch) = repo.get_branch_from_commit(commit_sha)? {
    println!("Commit {} is on branch: {}", commit_sha, branch);
}

// Get all branches containing a commit
let branches = repo.get_branches_containing_commit(commit_sha)?;
for branch in branches {
    println!("Branch {} contains commit {}", branch, commit_sha);
}

// Find merge base between branches
let merge_base = repo.get_merge_base("main", "feature/new-feature")?;
println!("Merge base: {}", merge_base);
```

## Commit Operations

### Staging and Committing

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project")?;

// Add a specific file
repo.add("src/main.rs")?;

// Add all changes
repo.add_all()?;

// Commit staged changes
let commit_id = repo.commit("feat: add new functionality")?;
println!("Created commit: {}", commit_id);

// Add all changes and commit in one step
let commit_id = repo.commit_changes("fix: resolve issue with authentication")?;
println!("Created commit: {}", commit_id);
```

### Commit Information

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project")?;

// Get current commit SHA
let current_sha = repo.get_current_sha()?;
println!("Current commit: {}", current_sha);

// Get previous commit SHA
let previous_sha = repo.get_previous_sha()?;
println!("Previous commit: {}", previous_sha);

// Get commit history since a specific reference
let commits = repo.get_commits_since(Some("v1.0.0".to_string()), &None)?;
for commit in commits {
    println!("{}: {} (by {} on {})",
        commit.hash,
        commit.message,
        commit.author_name,
        commit.author_date
    );
}

// Get commits affecting a specific file
let file_commits = repo.get_commits_since(None, &Some("src/main.rs".to_string()))?;
for commit in file_commits {
    println!("Commit {} modified src/main.rs: {}", commit.hash, commit.message);
}
```

## Tag Operations

### Creating and Managing Tags

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project")?;

// Create an annotated tag
repo.create_tag("v1.0.0", Some("Release version 1.0.0".to_string()))?;

// Create a lightweight tag
repo.create_tag("v1.0.1", None)?;

// Get the last tag
let last_tag = repo.get_last_tag()?;
println!("Last tag: {}", last_tag);

// Get all local tags
let local_tags = repo.get_remote_or_local_tags(Some(true))?;
for tag in local_tags {
    println!("Local tag: {} -> {}", tag.tag, tag.hash);
}

// Get all remote tags
let remote_tags = repo.get_remote_or_local_tags(Some(false))?;
for tag in remote_tags {
    println!("Remote tag: {} -> {}", tag.tag, tag.hash);
}
```

## File Status and Change Detection

### Repository Status

```rust
use sublime_git_tools::{Repo, GitFileStatus};

let repo = Repo::open("./my-project")?;

// Get porcelain status (simple format)
let status_lines = repo.status_porcelain()?;
for line in status_lines {
    println!("{}", line);
}

// Get detailed status with file information
let detailed_status = repo.get_status_detailed()?;
for file in detailed_status {
    println!("File: {}, Status: {:?}, Staged: {}, Workdir: {}",
        file.path,
        file.status,
        file.staged,
        file.workdir
    );
}

// Get only staged files
let staged_files = repo.get_staged_files()?;
for file in staged_files {
    println!("Staged: {}", file);
}
```

### Change Detection Between References

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project")?;

// Get all changed files since a commit/tag with status
let changed_files = repo.get_all_files_changed_since_sha_with_status("v1.0.0")?;
for file in changed_files {
    println!("Changed file: {} ({:?})", file.path, file.status);
}

// Get simple list of changed files since a commit/tag
let changed_files = repo.get_all_files_changed_since_sha("v1.0.0")?;
for file in changed_files {
    println!("Changed: {}", file);
}

// Get files changed between two references
let files = repo.get_files_changed_between("main", "feature-branch")?;
for file in files {
    println!("File: {}, Status: {:?}, Staged: {}, Workdir: {}",
        file.path, file.status, file.staged, file.workdir);
}

// Get changes in specific packages/directories since a branch
let packages = vec!["packages/pkg1".to_string(), "packages/pkg2".to_string()];
let package_changes = repo.get_all_files_changed_since_branch(&packages, "main")?;
for file in package_changes {
    println!("Package change: {}", file);
}
```

## Remote Operations

### Basic Remote Operations

```rust
use sublime_git_tools::Repo;
use std::path::PathBuf;

let repo = Repo::open("./my-project")?;

// Push to remote (without tags)
let success = repo.push("origin", Some(false))?;
if success {
    println!("Push completed successfully");
}

// Push with tags
repo.push("origin", Some(true))?;

// Fetch from remote
repo.fetch("origin", None, false)?;

// Fetch a specific branch
repo.fetch("origin", Some("feature-branch"), false)?;

// Pull from remote (current branch)
let success = repo.pull("origin", None)?;
if success {
    println!("Pull completed successfully");
}

// Pull a specific branch
repo.pull("origin", Some("feature-branch"))?;
```

### SSH Authentication

```rust
use sublime_git_tools::Repo;
use std::path::PathBuf;

let repo = Repo::open("./my-project")?;

// Push using SSH key paths (tries multiple keys in order)
let ssh_key_paths = vec![
    PathBuf::from("/home/user/.ssh/id_ed25519"),
    PathBuf::from("/home/user/.ssh/id_rsa"),
];

let success = repo.push_with_ssh_config("origin", Some(true), ssh_key_paths)?;
if success {
    println!("SSH push completed successfully");
}
```

## Advanced Git Operations

### Merging

```rust
use sublime_git_tools::{Repo, RepoError};

let repo = Repo::open("./my-project")?;

// Merge a branch
match repo.merge("feature-branch") {
    Ok(_) => println!("Merge completed successfully"),
    Err(RepoError::MergeConflictError(_)) => {
        println!("Merge conflicts detected - manual resolution required");
        // Handle conflicts...
    },
    Err(e) => println!("Merge failed: {}", e),
}

// Find diverged commit (common ancestor)
let diverged_commit = repo.get_diverged_commit("feature-branch")?;
println!("Common ancestor: {}", diverged_commit);
```

## Error Handling

The crate uses a comprehensive error type (`RepoError`) that provides detailed information about Git operation failures:

```rust
use sublime_git_tools::{Repo, RepoError};

match Repo::open("/non/existent/path") {
    Ok(repo) => {
        // Repository operations...
        match repo.checkout("non-existent-branch") {
            Ok(_) => println!("Switched to branch successfully"),
            Err(RepoError::BranchNameError(_)) => {
                println!("Branch does not exist");
            },
            Err(RepoError::CheckoutBranchError(e)) => {
                println!("Failed to checkout branch: {}", e);
            },
            Err(e) => println!("Other checkout error: {}", e),
        }
    },
    Err(RepoError::OpenRepoFailure(e)) => {
        println!("Failed to open repository: {}", e);
    },
    Err(e) => println!("Other error: {}", e),
}

// Handle merge conflicts
match repo.merge("feature-branch") {
    Ok(_) => println!("Merge successful"),
    Err(RepoError::MergeConflictError(_)) => {
        println!("Merge conflicts require manual resolution");
        // Get conflicted files
        let status = repo.get_status_detailed()?;
        for file in status {
            if matches!(file.status, GitFileStatus::Modified) && file.staged && file.workdir {
                println!("Conflict in: {}", file.path);
            }
        }
    },
    Err(RepoError::MergeError(e)) => {
        println!("Merge failed: {}", e);
    },
    Err(e) => println!("Unexpected error: {}", e),
}
```

### Available Error Types

The `RepoError` enum provides specific error variants for different Git operations:

- `CreateRepoFailure` - Repository creation errors
- `OpenRepoFailure` - Repository opening errors  
- `CloneRepoFailure` - Repository cloning errors
- `BranchError` / `BranchNameError` / `CheckoutBranchError` - Branch operation errors
- `CommitError` / `CommitOidError` - Commit operation errors
- `MergeError` / `MergeConflictError` - Merge operation errors
- `PushError` / `RemoteError` - Remote operation errors
- `TagError` / `CreateTagError` / `LastTagError` - Tag operation errors
- `ConfigError` / `ConfigEntriesError` - Configuration errors
- `StatusError` / `DiffError` - Status and diff errors
- And many more for comprehensive error coverage

## Data Types

### Repository Types

```rust
use sublime_git_tools::{Repo, RepoCommit, RepoTags, GitChangedFile, GitFileStatus};

// RepoCommit - represents a commit
let commit = RepoCommit {
    hash: "abcdef123456".to_string(),
    author_name: "John Doe".to_string(),
    author_email: "john@example.com".to_string(),
    author_date: "Wed, 01 Jan 2023 12:00:00 +0000".to_string(),
    message: "feat: add new feature".to_string(),
};

// RepoTags - represents a tag
let tag = RepoTags {
    hash: "abcdef123456".to_string(),
    tag: "v1.0.0".to_string(),
};

// GitChangedFile - represents a file with change information
let changed_file = GitChangedFile {
    path: "src/main.rs".to_string(),
    status: GitFileStatus::Modified,
    staged: true,
    workdir: false,
};

// GitFileStatus - file status enumeration
match changed_file.status {
    GitFileStatus::Added => println!("File was added"),
    GitFileStatus::Modified => println!("File was modified"),
    GitFileStatus::Deleted => println!("File was deleted"),
    GitFileStatus::Untracked => println!("File is untracked"),
}
```

## Common Workflows

### Feature Branch Workflow

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project")?;

// 1. Create and switch to feature branch
repo.create_branch("feature/user-authentication")?;
repo.checkout("feature/user-authentication")?;

// 2. Make changes and commit
repo.add("src/auth.rs")?;
repo.commit("feat: add user authentication module")?;

// 3. Push feature branch
repo.push("origin", Some(false))?;

// 4. Switch back to main and merge
repo.checkout("main")?;
repo.merge("feature/user-authentication")?;

// 5. Tag the release
repo.create_tag("v1.1.0", Some("Add user authentication".to_string()))?;
repo.push("origin", Some(true))?; // Push with tags
```

### Change Detection Workflow

```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./my-project")?;

// Check what changed since last release
let last_tag = repo.get_last_tag()?;
let changed_files = repo.get_all_files_changed_since_sha_with_status(&last_tag)?;

println!("Changes since {}: ", last_tag);
for file in changed_files {
    match file.status {
        GitFileStatus::Added => println!("+ {}", file.path),
        GitFileStatus::Modified => println!("M {}", file.path),
        GitFileStatus::Deleted => println!("- {}", file.path),
        GitFileStatus::Untracked => println!("? {}", file.path),
    }
}

// Get commits for changelog
let commits = repo.get_commits_since(Some(last_tag), &None)?;
for commit in commits {
    println!("- {} ({})", commit.message, commit.hash[..8].to_string());
}
```

## Cross-Platform Support

The crate is designed to work on all major platforms:
- **Windows** - Full support with vendored OpenSSL and libgit2
- **macOS** - Full support with vendored dependencies
- **Linux** - Full support with vendored dependencies

SSH operations are supported across all platforms with the vendored SSH implementation.

## Performance Considerations

- The crate uses vendored libgit2 for consistent behavior across platforms
- Repository operations are optimized for common use cases
- Large repositories may benefit from specific Git configuration tuning
- File change detection operations scale with repository size

## License

This crate is licensed under the terms specified in the workspace configuration.

## Contributing

Please see the workspace-level contribution guidelines for information on how to contribute to this crate.