//! Hook execution context implementation
//!
//! This module provides implementation methods for the `HookExecutionContext`,
//! following the project's pattern of separating declarations from implementations.

use super::{CommitInfo, GitOperationType, HookExecutionContext, RemoteInfo};
use glob::Pattern;
use std::collections::HashMap;
use std::path::PathBuf;

impl HookExecutionContext {
    /// Create a new hook execution context
    #[must_use]
    pub fn new(repository_root: PathBuf, current_branch: impl Into<String>) -> Self {
        Self {
            repository_root: repository_root.clone(),
            current_branch: current_branch.into(),
            previous_branch: None,
            current_commit: None,
            previous_commit: None,
            changed_files: Vec::new(),
            affected_packages: Vec::new(),
            environment: std::env::vars().collect(),
            operation_type: GitOperationType::Unknown,
            remote_info: None,
            commits: Vec::new(),
            is_merge: false,
            working_directory: repository_root,
            metadata: HashMap::new(),
        }
    }

    /// Set the previous branch
    #[must_use]
    pub fn with_previous_branch(mut self, branch: impl Into<String>) -> Self {
        self.previous_branch = Some(branch.into());
        self
    }

    /// Set current commit hash
    #[must_use]
    pub fn with_current_commit(mut self, commit: impl Into<String>) -> Self {
        self.current_commit = Some(commit.into());
        self
    }

    /// Set previous commit hash
    #[must_use]
    pub fn with_previous_commit(mut self, commit: impl Into<String>) -> Self {
        self.previous_commit = Some(commit.into());
        self
    }

    /// Set changed files
    #[must_use]
    pub fn with_changed_files(mut self, files: Vec<String>) -> Self {
        self.changed_files = files;
        self
    }

    /// Set affected packages
    #[must_use]
    pub fn with_affected_packages(mut self, packages: Vec<String>) -> Self {
        self.affected_packages = packages;
        self
    }

    /// Set Git operation type
    #[must_use]
    pub fn with_operation_type(mut self, operation: GitOperationType) -> Self {
        self.operation_type = operation;
        self
    }

    /// Set remote information
    #[must_use]
    pub fn with_remote_info(mut self, remote: RemoteInfo) -> Self {
        self.remote_info = Some(remote);
        self
    }

    /// Add commit information
    #[must_use]
    pub fn with_commit(mut self, commit: CommitInfo) -> Self {
        self.commits.push(commit);
        self
    }

    /// Set multiple commits
    #[must_use]
    pub fn with_commits(mut self, commits: Vec<CommitInfo>) -> Self {
        self.commits = commits;
        self
    }

    /// Mark as merge commit
    #[must_use]
    pub fn with_merge(mut self, is_merge: bool) -> Self {
        self.is_merge = is_merge;
        self
    }

    /// Set working directory
    #[must_use]
    pub fn with_working_directory(mut self, dir: PathBuf) -> Self {
        self.working_directory = dir;
        self
    }

    /// Add environment variable
    #[must_use]
    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.environment.insert(key, value);
        self
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get environment variable
    #[must_use]
    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.environment.get(key)
    }

    /// Get metadata value
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Check if this is a commit operation
    #[must_use]
    pub fn is_commit(&self) -> bool {
        self.operation_type == GitOperationType::Commit
    }

    /// Check if this is a push operation
    #[must_use]
    pub fn is_push(&self) -> bool {
        self.operation_type == GitOperationType::Push
    }

    /// Check if this is a merge operation
    #[must_use]
    pub fn is_merge_operation(&self) -> bool {
        self.operation_type == GitOperationType::Merge
    }

    /// Get commit hashes as vector
    #[must_use]
    pub fn commit_hashes(&self) -> Vec<String> {
        self.commits.iter().map(|c| c.hash.clone()).collect()
    }

    /// Check if any packages are affected
    #[must_use]
    pub fn has_affected_packages(&self) -> bool {
        !self.affected_packages.is_empty()
    }

    /// Check if specific file was changed
    #[must_use]
    pub fn has_changed_file(&self, file_path: &str) -> bool {
        self.changed_files.iter().any(|f| f == file_path || f.ends_with(file_path))
    }

    /// Check if any file matching pattern was changed
    ///
    /// Supports comprehensive glob patterns including:
    /// - `*.rs` - Files with .rs extension
    /// - `src/**/*.ts` - TypeScript files in src directory and subdirectories
    /// - `tests/**/test_*.rs` - Test files in any subdirectory of tests
    /// - `{src,lib}/**/*.rs` - Rust files in src or lib directories
    /// - `!target/**` - Exclude target directory (when used with other patterns)
    #[must_use]
    pub fn has_changed_files_matching(&self, pattern: &str) -> bool {
        // Handle exact matches first for performance
        if !pattern.contains(['*', '?', '[', '{']) {
            return self.changed_files.iter().any(|f| f == pattern);
        }

        // Compile the glob pattern
        match Pattern::new(pattern) {
            Ok(glob_pattern) => {
                self.changed_files.iter().any(|file_path| {
                    // Try matching the full path
                    if glob_pattern.matches(file_path) {
                        return true;
                    }

                    // Also try matching just the filename for patterns like "*.rs"
                    if let Some(filename) = std::path::Path::new(file_path).file_name() {
                        if let Some(filename_str) = filename.to_str() {
                            return glob_pattern.matches(filename_str);
                        }
                    }

                    false
                })
            }
            Err(e) => {
                log::warn!(
                    "Invalid glob pattern '{}': {}. Falling back to simple string matching.",
                    pattern,
                    e
                );

                // Fallback to simple string matching for invalid patterns
                self.changed_files.iter().any(|f| f.contains(pattern) || f == pattern)
            }
        }
    }

    /// Check if any file matches multiple patterns with optional exclusions
    ///
    /// # Arguments
    ///
    /// * `include_patterns` - Patterns that files must match
    /// * `exclude_patterns` - Patterns that files must NOT match (optional)
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Match TypeScript files but exclude test files
    /// context.has_changed_files_matching_multiple(&["**/*.ts"], &["**/*.test.ts"]);
    ///
    /// // Match source files in multiple directories
    /// context.has_changed_files_matching_multiple(&["src/**/*.rs", "lib/**/*.rs"], &[]);
    /// ```
    #[must_use]
    pub fn has_changed_files_matching_multiple(
        &self,
        include_patterns: &[&str],
        exclude_patterns: &[&str],
    ) -> bool {
        // Compile include patterns
        let compiled_includes: Vec<Pattern> = include_patterns
            .iter()
            .filter_map(|pattern| {
                Pattern::new(pattern)
                    .map_err(|e| {
                        log::warn!("Invalid include pattern '{}': {}", pattern, e);
                        e
                    })
                    .ok()
            })
            .collect();

        // Compile exclude patterns
        let compiled_excludes: Vec<Pattern> = exclude_patterns
            .iter()
            .filter_map(|pattern| {
                Pattern::new(pattern)
                    .map_err(|e| {
                        log::warn!("Invalid exclude pattern '{}': {}", pattern, e);
                        e
                    })
                    .ok()
            })
            .collect();

        self.changed_files.iter().any(|file_path| {
            // Check if file matches any include pattern
            let matches_include = compiled_includes.is_empty()
                || compiled_includes.iter().any(|pattern| {
                    pattern.matches(file_path) || {
                        // Also try matching just the filename
                        std::path::Path::new(file_path)
                            .file_name()
                            .and_then(|f| f.to_str())
                            .is_some_and(|filename| pattern.matches(filename))
                    }
                });

            // Check if file matches any exclude pattern
            let matches_exclude = compiled_excludes.iter().any(|pattern| {
                pattern.matches(file_path) || {
                    // Also try matching just the filename
                    std::path::Path::new(file_path)
                        .file_name()
                        .and_then(|f| f.to_str())
                        .is_some_and(|filename| pattern.matches(filename))
                }
            });

            matches_include && !matches_exclude
        })
    }

    /// Get file changes by extension
    #[must_use]
    pub fn get_changed_files_by_extension(&self, extension: &str) -> Vec<&String> {
        let ext_with_dot = if extension.starts_with('.') {
            extension.to_string()
        } else {
            format!(".{extension}")
        };

        self.changed_files.iter().filter(|f| f.ends_with(&ext_with_dot)).collect()
    }
}
