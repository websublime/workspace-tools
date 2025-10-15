# ChangesetBuilder Architecture

## Problem Statement

The original implementation mixed responsibilities between Git operations and Conventional Commit parsing, leading to incorrect filtering of commits by package.

### Issues Found

1. **Line 535-537**: `filter_commits_for_files` parameter `_files` was unused
2. **Responsibility Mixing**: `ConventionalCommit` was being used as source for Git file information
3. **Incorrect Filtering**: Commits were not properly filtered by which files they touched

## Correct Architecture

### Separation of Concerns

```
┌─────────────────────────────────────────────────────────────────┐
│ sublime_git_tools::Repo                                          │
│ ✓ Get commits between refs                                      │
│ ✓ Get files changed in commit (get_files_changed_in_commit)     │
│ ✓ Get files changed between refs (get_files_changed_between)    │
│ ✓ Technical Git information                                     │
└─────────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────────┐
│ PackageChangeDetector                                            │
│ ✓ Map changed files → affected packages                         │
│ ✓ Reuse MonorepoDetector from sublime_standard_tools           │
│ ✓ Apply PackageToolsConfig customizations                      │
│ ✓ Support single-package and monorepo                          │
└─────────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────────┐
│ ConventionalCommitService                                        │
│ ✓ Parse commit messages (feat:, fix:, etc.)                    │
│ ✓ Calculate version bumps (major/minor/patch)                  │
│ ✓ Generate changelog entries                                    │
│ ✗ NOT for Git file operations                                   │
└─────────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────────┐
│ ChangesetBuilder (orchestrator)                                  │
│ 1. Get changed files from Git                                   │
│ 2. Detect affected packages                                     │
│ 3. For each package:                                            │
│    a. Get commits that touched package files (using Git)        │
│    b. Parse commits for semantic info (ConventionalCommit)      │
│    c. Calculate version bump                                    │
│ 4. Build changeset structure                                    │
└─────────────────────────────────────────────────────────────────┘
```

## Implementation Plan

### Phase 1: Git-First Approach

**File**: `src/changeset/builder.rs`

```rust
impl ChangesetBuilder {
    async fn build_changeset(
        &self,
        branch: &str,
        author: String,
        target_environments: Vec<String>,
        from_ref: &str,
        to_ref: Option<&str>,
    ) -> ChangesetResult<Changeset> {
        // 1. Get all changed files from Git (source of truth)
        let changed_files = self.get_changed_files_from_git(from_ref, to_ref)?;
        
        // 2. Detect affected packages using detector
        let affected_packages = self.package_detector
            .detect_affected_packages(&changed_files)
            .await?;
        
        // 3. Get all commit hashes from Git
        let all_commits = self.get_commits_from_git(from_ref, to_ref)?;
        
        // 4. For each affected package
        for (package_name, package_files) in affected_packages {
            // 4a. Filter commits that touched this package (using Git)
            let package_commits = self.filter_commits_by_files(
                &all_commits,
                &package_files
            )?;
            
            // 4b. Parse commits for semantic information
            let parsed_commits = self.parse_commits_for_semantics(
                &package_commits
            )?;
            
            // 4c. Calculate version bump
            let bump = self.commit_service.calculate_version_bump(&parsed_commits);
            
            // 4d. Build changeset package
            // ...
        }
    }
    
    /// Filter commits that touched specific package files
    /// Uses Git directly to check which files each commit modified
    fn filter_commits_by_files(
        &self,
        commit_hashes: &[String],
        package_files: &[PathBuf],
    ) -> ChangesetResult<Vec<String>> {
        let mut filtered = Vec::new();
        let repo_root = self.repo.get_repo_path();
        
        // Convert absolute paths to repo-relative
        let relative_files: Vec<PathBuf> = package_files
            .iter()
            .filter_map(|p| p.strip_prefix(repo_root).ok().map(|r| r.to_path_buf()))
            .collect();
        
        for commit_hash in commit_hashes {
            // Get files changed in this commit from Git
            let changed_files = self.repo
                .get_files_changed_in_commit(commit_hash)?;
            
            // Check if any changed file belongs to this package
            let touches_package = changed_files.iter().any(|changed| {
                let changed_path = PathBuf::from(&changed.path);
                relative_files.iter().any(|pkg_file| {
                    changed_path == *pkg_file 
                        || changed_path.starts_with(pkg_file)
                        || pkg_file.starts_with(&changed_path)
                })
            });
            
            if touches_package {
                filtered.push(commit_hash.clone());
            }
        }
        
        Ok(filtered)
    }
    
    /// Parse commit hashes into ConventionalCommit for semantic analysis
    fn parse_commits_for_semantics(
        &self,
        commit_hashes: &[String],
    ) -> ChangesetResult<Vec<ConventionalCommit>> {
        // Get full commit information from Git
        // Parse messages using ConventionalCommitService
        // Return parsed commits
    }
}
```

### Phase 2: PackageChangeDetector Enhancement

**File**: `src/changeset/detector.rs`

```rust
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};

impl PackageChangeDetector {
    async fn detect_affected_packages(
        &self,
        changed_files: &[PathBuf],
    ) -> ChangesetResult<HashMap<String, Vec<PathBuf>>> {
        // 1. Use MonorepoDetector from standard tools as base
        let monorepo_detector = MonorepoDetector::new();
        
        // 2. Check if monorepo or single-package
        let is_monorepo = monorepo_detector
            .is_monorepo_root(&self.workspace_root)
            .await?
            .is_some();
        
        if is_monorepo {
            // Use MonorepoDetector to get packages
            let packages = monorepo_detector
                .detect_packages(&self.workspace_root)
                .await?;
            
            // Apply custom logic + PackageToolsConfig
            self.map_files_to_packages(changed_files, &packages)
        } else {
            // Single package logic
            self.detect_single_package(changed_files).await
        }
    }
}
```

## Key Principles

1. **Git is Source of Truth**: All file and commit information comes from Git first
2. **ConventionalCommit for Semantics Only**: Only used to parse messages and calculate bumps
3. **Reuse Standard Tools**: Don't reimplement what exists in `sublime_standard_tools`
4. **Configuration Aware**: Respect `PackageToolsConfig` settings
5. **Clear Responsibilities**: Each component has one job

## Testing Strategy

1. **Unit Tests**: Mock Git responses, test filtering logic
2. **Integration Tests**: Real Git repos with commits
3. **Test Cases**:
   - Single package with multiple commits
   - Monorepo with commits touching different packages
   - Commits touching multiple packages
   - Commits with no package changes (root files only)

## Migration Path

1. ✅ Add `get_files_changed_in_commit` to Git crate (DONE)
2. ⏳ Implement `filter_commits_by_files` correctly
3. ⏳ Update `build_changeset` to use Git-first approach
4. ⏳ Enhance `PackageChangeDetector` to reuse `MonorepoDetector`
5. ⏳ Add comprehensive tests
6. ⏳ Remove old implementation

## Current Status

- **Git Support**: ✅ `get_files_changed_in_commit` added
- **Builder Implementation**: ❌ Needs rewrite following this architecture
- **Detector Enhancement**: ❌ Should reuse MonorepoDetector
- **Tests**: ⚠️ Some passing but architecture is incorrect

## Next Steps

1. Implement `ChangesetBuilder` following Phase 1 design
2. Remove dependency on ConventionalCommit for file operations
3. Enhance PackageChangeDetector to use standard tools
4. Add missing integration tests
5. Validate with real-world monorepo scenarios