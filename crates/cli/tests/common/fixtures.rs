//! Test fixtures for E2E CLI tests.
//!
//! **What**: Provides reusable workspace fixtures for creating test environments.
//!
//! **How**: Uses builder pattern with fluent API to create temporary workspaces
//! with various configurations (single package, monorepo, git, changesets, etc.).
//!
//! **Why**: Eliminates duplication in E2E tests and ensures consistent test setup.
//!
//! # Examples
//!
//! ```rust,ignore
//! // Simple single package
//! let workspace = WorkspaceFixture::single_package();
//!
//! // Monorepo with git and changeset
//! let workspace = WorkspaceFixture::monorepo_independent()
//!     .with_git()
//!     .add_changeset(ChangesetBuilder::minor().package("pkg-a"));
//!
//! // Complex setup
//! let workspace = WorkspaceFixture::monorepo_unified()
//!     .with_git()
//!     .with_commits(3)
//!     .with_branch("feature/test")
//!     .with_npmrc("registry=https://custom.registry.com")
//!     .add_changesets(vec![
//!         ChangesetBuilder::minor().package("pkg-a"),
//!         ChangesetBuilder::patch().package("pkg-b"),
//!     ]);
//! ```

use serde_json::json;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Workspace fixture builder for E2E tests.
///
/// Provides fluent API for creating test workspaces with various configurations.
/// Automatically cleans up on drop via TempDir.
pub struct WorkspaceFixture {
    /// Temporary directory (auto-cleanup on drop)
    temp_dir: TempDir,
    /// Root path of the workspace
    root: PathBuf,
    /// Whether git is initialized
    git_initialized: bool,
    /// List of packages in the workspace
    packages: Vec<PackageInfo>,
    /// Whether changesets directory is created
    changesets_dir_created: bool,
}

/// Package information
#[derive(Debug, Clone)]
pub struct PackageInfo {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Relative path from workspace root
    pub path: PathBuf,
    /// Package dependencies
    pub dependencies: Vec<(String, String)>,
}

impl WorkspaceFixture {
    // =========================================================================
    // Factory Methods
    // =========================================================================

    /// Creates a single-package workspace.
    ///
    /// Structure:
    /// ```text
    /// workspace/
    /// ├── package.json
    /// └── (no other packages)
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let workspace = WorkspaceFixture::single_package();
    /// assert_eq!(workspace.package_count(), 1);
    /// ```
    #[allow(clippy::expect_used)]
    pub fn single_package() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root = temp_dir.path().to_path_buf();

        let package = PackageInfo {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            path: root.clone(),
            dependencies: vec![],
        };

        Self {
            temp_dir,
            root,
            git_initialized: false,
            packages: vec![package],
            changesets_dir_created: false,
        }
    }

    /// Creates an independent strategy monorepo.
    ///
    /// Each package has its own version and is bumped independently.
    ///
    /// Structure:
    /// ```text
    /// monorepo/
    /// ├── package.json (workspace root)
    /// └── packages/
    ///     ├── pkg-a/
    ///     │   └── package.json
    ///     └── pkg-b/
    ///         └── package.json
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let workspace = WorkspaceFixture::monorepo_independent();
    /// assert_eq!(workspace.package_count(), 2);
    /// ```
    #[allow(clippy::expect_used)]
    pub fn monorepo_independent() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root = temp_dir.path().to_path_buf();

        let packages = vec![
            PackageInfo {
                name: "@test/pkg-a".to_string(),
                version: "1.0.0".to_string(),
                path: root.join("packages/pkg-a"),
                dependencies: vec![],
            },
            PackageInfo {
                name: "@test/pkg-b".to_string(),
                version: "1.0.0".to_string(),
                path: root.join("packages/pkg-b"),
                dependencies: vec![],
            },
        ];

        Self { temp_dir, root, git_initialized: false, packages, changesets_dir_created: false }
    }

    /// Creates a unified strategy monorepo.
    ///
    /// All packages share the same version and are bumped together.
    ///
    /// Structure: Same as independent monorepo
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let workspace = WorkspaceFixture::monorepo_unified();
    /// ```
    #[allow(clippy::expect_used)]
    pub fn monorepo_unified() -> Self {
        // Same structure as independent, but with unified config
        Self::monorepo_independent()
    }

    /// Creates a monorepo with internal dependencies.
    ///
    /// Package B depends on Package A.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let workspace = WorkspaceFixture::monorepo_with_internal_deps();
    /// ```
    #[allow(clippy::expect_used)]
    pub fn monorepo_with_internal_deps() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root = temp_dir.path().to_path_buf();

        let packages = vec![
            PackageInfo {
                name: "@test/pkg-a".to_string(),
                version: "1.0.0".to_string(),
                path: root.join("packages/pkg-a"),
                dependencies: vec![],
            },
            PackageInfo {
                name: "@test/pkg-b".to_string(),
                version: "1.0.0".to_string(),
                path: root.join("packages/pkg-b"),
                dependencies: vec![("@test/pkg-a".to_string(), "^1.0.0".to_string())],
            },
        ];

        Self { temp_dir, root, git_initialized: false, packages, changesets_dir_created: false }
    }

    // =========================================================================
    // Builder Methods - Configuration
    // =========================================================================

    /// Adds default configuration file to workspace.
    ///
    /// Creates a `repo.config.json` with sensible defaults.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let workspace = WorkspaceFixture::single_package()
    ///     .with_default_config();
    /// ```
    #[allow(clippy::expect_used)]
    pub fn with_default_config(self) -> Self {
        let is_monorepo = self.packages.len() > 1;
        let strategy = if is_monorepo { "independent" } else { "independent" };

        let config = json!({
            "changeset": {
                "path": ".changesets/"
            },
            "version": {
                "strategy": strategy,
                "defaultBump": "patch"
            },
            "changelog": {
                "enabled": true,
                "path": "CHANGELOG.md"
            },
            "upgrade": {
                "enabled": true,
                "backup_dir": ".workspace-backups"
            }
        });

        self.with_custom_config(
            &serde_json::to_string_pretty(&config).expect("Failed to serialize"),
        )
    }

    /// Adds custom configuration from JSON string.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = r#"{"version": {"strategy": "unified"}}"#;
    /// let workspace = WorkspaceFixture::single_package()
    ///     .with_custom_config(config);
    /// ```
    #[allow(clippy::expect_used)]
    pub fn with_custom_config(self, json: &str) -> Self {
        let config_path = self.root.join("repo.config.json");
        std::fs::write(&config_path, json).expect("Failed to write config");
        self
    }

    // =========================================================================
    // Builder Methods - Git
    // =========================================================================

    /// Initializes git repository.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let workspace = WorkspaceFixture::single_package()
    ///     .with_git();
    /// ```
    #[allow(clippy::expect_used)]
    pub fn with_git(mut self) -> Self {
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&self.root)
            .output()
            .expect("Failed to init git");

        // Configure git user for commits
        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&self.root)
            .output()
            .expect("Failed to configure git email");

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&self.root)
            .output()
            .expect("Failed to configure git name");

        self.git_initialized = true;
        self
    }

    /// Creates initial commits in git repository.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of commits to create
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let workspace = WorkspaceFixture::single_package()
    ///     .with_git()
    ///     .with_commits(3);
    /// ```
    #[allow(clippy::expect_used)]
    pub fn with_commits(self, count: usize) -> Self {
        assert!(self.git_initialized, "Git must be initialized before creating commits");

        for i in 0..count {
            let file_path = self.root.join(format!("file{i}.txt"));
            std::fs::write(&file_path, format!("content {i}")).expect("Failed to write file");

            std::process::Command::new("git")
                .args(["add", "."])
                .current_dir(&self.root)
                .output()
                .expect("Failed to git add");

            std::process::Command::new("git")
                .args(["commit", "-m", &format!("Commit {i}")])
                .current_dir(&self.root)
                .output()
                .expect("Failed to git commit");
        }

        self
    }

    /// Creates and checks out a new git branch.
    ///
    /// # Arguments
    ///
    /// * `name` - Branch name
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let workspace = WorkspaceFixture::single_package()
    ///     .with_git()
    ///     .with_branch("feature/test");
    /// ```
    #[allow(clippy::expect_used)]
    pub fn with_branch(self, name: &str) -> Self {
        assert!(self.git_initialized, "Git must be initialized before creating branches");

        std::process::Command::new("git")
            .args(["checkout", "-b", name])
            .current_dir(&self.root)
            .output()
            .expect("Failed to create branch");

        self
    }

    // =========================================================================
    // Builder Methods - Changesets
    // =========================================================================

    /// Adds a changeset to the workspace.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let workspace = WorkspaceFixture::single_package()
    ///     .add_changeset(ChangesetBuilder::minor());
    /// ```
    #[allow(clippy::expect_used)]
    pub fn add_changeset(mut self, changeset: ChangesetBuilder) -> Self {
        self.ensure_changesets_dir();

        let changeset_data = changeset.build(&self.packages);
        let branch = changeset_data["branch"].as_str().expect("Branch field missing");
        let filename = format!("{}.json", branch.replace('/', "-"));
        let changeset_path = self.root.join(".changesets").join(filename);

        std::fs::write(
            &changeset_path,
            serde_json::to_string_pretty(&changeset_data).expect("Failed to serialize"),
        )
        .expect("Failed to write changeset");

        self
    }

    /// Adds multiple changesets to the workspace.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let workspace = WorkspaceFixture::monorepo_independent()
    ///     .add_changesets(vec![
    ///         ChangesetBuilder::minor().package("pkg-a"),
    ///         ChangesetBuilder::patch().package("pkg-b"),
    ///     ]);
    /// ```
    pub fn add_changesets(mut self, changesets: Vec<ChangesetBuilder>) -> Self {
        for changeset in changesets {
            self = self.add_changeset(changeset);
        }
        self
    }

    /// Ensures .changesets directory exists.
    #[allow(clippy::expect_used)]
    fn ensure_changesets_dir(&mut self) {
        if !self.changesets_dir_created {
            let changesets_dir = self.root.join(".changesets");
            std::fs::create_dir_all(&changesets_dir).expect("Failed to create .changesets dir");
            self.changesets_dir_created = true;
        }
    }

    // =========================================================================
    // Builder Methods - NPM
    // =========================================================================

    /// Adds .npmrc file to workspace.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let workspace = WorkspaceFixture::single_package()
    ///     .with_npmrc("registry=https://custom.registry.com");
    /// ```
    #[allow(clippy::expect_used)]
    pub fn with_npmrc(self, content: &str) -> Self {
        let npmrc_path = self.root.join(".npmrc");
        std::fs::write(&npmrc_path, content).expect("Failed to write .npmrc");
        self
    }

    /// Adds package-lock.json to workspace.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let workspace = WorkspaceFixture::single_package()
    ///     .with_package_lock();
    /// ```
    #[allow(clippy::expect_used)]
    pub fn with_package_lock(self) -> Self {
        let lock = json!({
            "name": self.packages[0].name,
            "version": self.packages[0].version,
            "lockfileVersion": 3,
            "requires": true,
            "packages": {}
        });

        let lock_path = self.root.join("package-lock.json");
        std::fs::write(
            &lock_path,
            serde_json::to_string_pretty(&lock).expect("Failed to serialize"),
        )
        .expect("Failed to write package-lock.json");

        self
    }

    // =========================================================================
    // Finalization - Create Workspace Files
    // =========================================================================

    /// Finalizes the workspace by creating all package.json files.
    ///
    /// This must be called after all builder methods.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let workspace = WorkspaceFixture::single_package()
    ///     .with_git()
    ///     .finalize();
    /// ```
    #[allow(clippy::expect_used)]
    pub fn finalize(self) -> Self {
        // Create root package.json
        let is_monorepo = self.packages.len() > 1;

        if is_monorepo {
            // Root package.json for monorepo
            let workspace_patterns: Vec<String> = self
                .packages
                .iter()
                .map(|p| {
                    let rel_path = p.path.strip_prefix(&self.root).expect("Invalid path");
                    rel_path.to_string_lossy().to_string()
                })
                .collect();

            let root_package = json!({
                "name": "monorepo-root",
                "version": "1.0.0",
                "private": true,
                "workspaces": workspace_patterns
            });

            std::fs::write(
                self.root.join("package.json"),
                serde_json::to_string_pretty(&root_package).expect("Failed to serialize"),
            )
            .expect("Failed to write root package.json");
        }

        // Create package.json for each package
        for package in &self.packages {
            // Ensure parent directory exists
            if let Some(parent) = package.path.parent() {
                std::fs::create_dir_all(parent).expect("Failed to create package dir");
            }

            let mut package_json = json!({
                "name": package.name,
                "version": package.version,
            });

            // Add dependencies if any
            if !package.dependencies.is_empty() {
                let deps: serde_json::Map<String, serde_json::Value> = package
                    .dependencies
                    .iter()
                    .map(|(name, version)| (name.clone(), json!(version)))
                    .collect();
                package_json["dependencies"] = json!(deps);
            }

            let package_json_path = package.path.join("package.json");
            std::fs::write(
                &package_json_path,
                serde_json::to_string_pretty(&package_json).expect("Failed to serialize"),
            )
            .expect("Failed to write package.json");
        }

        self
    }

    // =========================================================================
    // Accessors
    // =========================================================================

    /// Returns the root path of the workspace.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the number of packages in the workspace.
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }

    /// Returns whether git is initialized.
    pub fn is_git_initialized(&self) -> bool {
        self.git_initialized
    }

    /// Returns package information by name.
    pub fn package(&self, name: &str) -> Option<&PackageInfo> {
        self.packages.iter().find(|p| p.name == name)
    }

    /// Returns all packages.
    pub fn packages(&self) -> &[PackageInfo] {
        &self.packages
    }

    // =========================================================================
    // Assertions
    // =========================================================================

    /// Asserts that config file exists.
    ///
    /// # Panics
    ///
    /// Panics if config file does not exist.
    pub fn assert_config_exists(&self) {
        let config_path = self.root.join("repo.config.json");
        assert!(config_path.exists(), "Config file does not exist: {}", config_path.display());
    }

    /// Asserts that a package has the expected version.
    ///
    /// # Panics
    ///
    /// Panics if package version does not match.
    #[allow(clippy::expect_used)]
    pub fn assert_package_version(&self, package_name: &str, expected_version: &str) {
        let package = self
            .package(package_name)
            .unwrap_or_else(|| panic!("Package {package_name} not found"));

        let package_json_path = package.path.join("package.json");
        let content =
            std::fs::read_to_string(&package_json_path).expect("Failed to read package.json");
        let json: serde_json::Value =
            serde_json::from_str(&content).expect("Failed to parse package.json");

        let actual_version = json["version"].as_str().expect("No version in package.json");
        assert_eq!(
            actual_version, expected_version,
            "Package {package_name} version mismatch. Expected: {expected_version}, Actual: {actual_version}"
        );
    }

    /// Asserts that changesets directory contains expected number of files.
    ///
    /// # Panics
    ///
    /// Panics if count does not match.
    #[allow(clippy::expect_used)]
    pub fn assert_changeset_count(&self, expected: usize) {
        let changesets_dir = self.root.join(".changesets");
        if !changesets_dir.exists() {
            assert_eq!(expected, 0, "Changesets directory does not exist");
            return;
        }

        let count = std::fs::read_dir(&changesets_dir)
            .expect("Failed to read changesets dir")
            .filter(|entry| {
                entry
                    .as_ref()
                    .map(|e| {
                        e.path().extension().and_then(|ext| ext.to_str()) == Some("json")
                            && !e.file_name().to_string_lossy().starts_with('.')
                    })
                    .unwrap_or(false)
            })
            .count();

        assert_eq!(count, expected, "Expected {expected} changesets, found {count}");
    }

    /// Asserts that changelog exists.
    ///
    /// # Panics
    ///
    /// Panics if changelog does not exist.
    pub fn assert_changelog_exists(&self) {
        let changelog_path = self.root.join("CHANGELOG.md");
        assert!(changelog_path.exists(), "Changelog does not exist");
    }
}

// =============================================================================
// Changeset Builder
// =============================================================================

/// Builder for creating test changesets.
#[derive(Debug, Clone)]
pub struct ChangesetBuilder {
    bump: String,
    branch: Option<String>,
    packages: Vec<String>,
    environments: Vec<String>,
}

impl ChangesetBuilder {
    /// Creates a minor bump changeset.
    pub fn minor() -> Self {
        Self {
            bump: "minor".to_string(),
            branch: None,
            packages: vec![],
            environments: vec!["production".to_string()],
        }
    }

    /// Creates a patch bump changeset.
    pub fn patch() -> Self {
        Self {
            bump: "patch".to_string(),
            branch: None,
            packages: vec![],
            environments: vec!["production".to_string()],
        }
    }

    /// Creates a major bump changeset.
    pub fn major() -> Self {
        Self {
            bump: "major".to_string(),
            branch: None,
            packages: vec![],
            environments: vec!["production".to_string()],
        }
    }

    /// Sets the branch name.
    pub fn branch(mut self, branch: &str) -> Self {
        self.branch = Some(branch.to_string());
        self
    }

    /// Adds a package to the changeset.
    pub fn package(mut self, package: &str) -> Self {
        self.packages.push(package.to_string());
        self
    }

    /// Sets environments for the changeset.
    pub fn environments(mut self, envs: &[&str]) -> Self {
        self.environments = envs.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Builds the changeset data.
    fn build(self, workspace_packages: &[PackageInfo]) -> serde_json::Value {
        let packages = if self.packages.is_empty() {
            // Use all workspace packages
            workspace_packages.iter().map(|p| p.name.clone()).collect()
        } else {
            self.packages
        };

        let branch = self.branch.unwrap_or_else(|| "feature/test".to_string());

        json!({
            "branch": branch,
            "bump": self.bump,
            "environments": self.environments,
            "packages": packages,
            "changes": [],
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        })
    }
}
