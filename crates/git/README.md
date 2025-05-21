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

## Main Features

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
```

## Error Handling

The crate uses a comprehensive error type (`RepoError`) that provides detailed information about Git operation failures:

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

## Cross-Platform Support

The crate is designed to work on all major platforms:
- Windows
- macOS
- Linux

## License

This crate is licensed under the terms specified in the workspace configuration.