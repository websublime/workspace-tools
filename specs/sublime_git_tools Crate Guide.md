---
type: Page
title: '`sublime_git_tools` Crate Documentation'
description: '`sublime_git_tools` provides a user-friendly API for working with Git repositories, wrapping the powerful but complex libgit2 library with a more ergonomic interface for common Git operations.'
icon: 🔧
createdAt: '2025-04-09T22:54:34.253Z'
creationDate: 2025-04-09 23:54
modificationDate: 2025-04-10 00:19
tags: [workspace-tools, rust]
coverImage: null
---

# `sublime_git_tools` Crate Documentation

[Workspace Tools](https://app.capacities.io/f8d3f900-cfe2-4c6d-b44c-906946be3e3c/52c8c019-b006-462b-8c5d-6b518afa0d64)

### Repository Management

Create, open, or clone repositories with simple, intuitive methods:

```rust
use sublime_git_tools::Repo;
// Create a new repository
let repo = Repo::create("/path/to/new/repo")?;
// Open an existing repository
let repo = Repo::open("./my-project")?;
// Clone a remote repository
let repo = Repo::clone("https://github.com/example/repo.git", "./cloned-repo")?;
```

### Branch and Commit Operations

Manage branches and commits with straightforward methods:

```rust
// Create and checkout branches
repo.create_branch("feature/new-feature")?;
repo.checkout("feature/new-feature")?;
// List branches
let branches = repo.list_branches()?;
println!("Available branches: {:?}", branches);
// Add and commit changes
repo.add("README.md")?;
let commit_id = repo.commit("docs: update README")?;
// Or add all changes and commit in one step
let commit_id = repo.commit_changes("feat: implement new feature")?;
```

### Change Detection and History

Find changes between commits, branches, or specific files:

```rust
// Get all changed files since a tag
let changed_files = repo.get_all_files_changed_since_sha("v1.0.0")?;
// Get changed files with their status (Added, Modified, Deleted)
let files_with_status = repo.get_all_files_changed_since_sha_with_status("main")?;
for file in files_with_status {
    println!("File: {} - {:?}", file.path, file.status);
}
// Get changes in specific packages since a branch
let packages = vec!["packages/pkg1".to_string(), "packages/pkg2".to_string()];
let package_changes = repo.get_all_files_changed_since_branch(&packages, "main")?;
```

### Commit History and Tags

Access commit history and manage tags:

```rust
// Get all commits since a specific tag
let commits = repo.get_commits_since(Some("v1.0.0".to_string()), &None)?;
// Get commits affecting a specific file
let file_commits = repo.get_commits_since(None, &Some("src/main.rs".to_string()))?;
for commit in commits {
    println!("{}: {} ({})", 
        commit.hash.chars().take(7).collect::<String>(),
        commit.message.lines().next().unwrap_or(""),
        commit.author_name
    );
}
// Create a new tag
repo.create_tag("v1.1.0", Some("Version 1.1.0 release".to_string()))?;
// List tags
let tags = repo.get_remote_or_local_tags(Some(true))?; // local tags
for tag in tags {
    println!("Tag: {} ({})", tag.tag, tag.hash);
}
```

### Remote Operations

Work with remote repositories:

```rust
// Push to a remote
repo.push("origin", Some(true))?; // true to include tags
// Fetch from remote
repo.fetch("origin", None, false)?; // No specific refspecs, no prune
// Pull from remote
repo.pull("origin", None)?; // Pull from tracking branch
```

