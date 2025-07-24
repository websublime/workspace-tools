//! # Dependency Source Definitions
//!
//! ## What
//! This module defines comprehensive dependency source types that represent all
//! dependency protocols supported in the JavaScript ecosystem. It provides
//! parsing and validation capabilities for npm, jsr, git, file, workspace, and url protocols.
//!
//! ## How  
//! The module implements a DependencySource enum with context-aware parsing logic.
//! Different variants handle different protocols (registry, git, workspace, etc.) with
//! their specific parsing requirements and validation rules.
//!
//! ## Why
//! The JavaScript ecosystem supports many dependency protocols beyond simple semver.
//! This module provides enterprise-grade support for all protocols with context-aware
//! validation that adapts behavior for single repositories vs monorepos.

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    path::PathBuf,
    str::FromStr,
};
use crate::{
    context::{DependencyProtocol, ProjectContext},
    errors::PackageError,
};

/// Complete dependency source representation for all JavaScript ecosystem protocols
///
/// This enum represents every possible way to specify a dependency in package.json,
/// from simple registry versions to complex workspace and git references.
///
/// ## Protocol Coverage
///
/// - **Registry**: npm, jsr, scoped packages (`@scope/package@^1.0.0`)
/// - **Workspace**: workspace protocols (`workspace:*`, `workspace:^`, `workspace:../path`)
/// - **Git**: git repositories with branches/tags/commits (`git+https://...#branch`)
/// - **GitHub**: GitHub shorthand (`user/repo`, `github:user/repo`)
/// - **File**: local file paths (`file:../local-package`)
/// - **URL**: direct tarball URLs (`https://example.com/package.tgz`)
///
/// ## Context Awareness
///
/// Different project contexts support different protocols:
/// - **Single Repository**: All protocols except workspace
/// - **Monorepo**: All protocols including workspace variants
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::context::DependencySource;
/// use semver::VersionReq;
///
/// // Registry dependency
/// let registry = DependencySource::Registry {
///     name: "react".to_string(),
///     version_req: VersionReq::parse("^18.0.0").unwrap(),
/// };
///
/// // Workspace dependency (monorepo only)
/// let workspace = DependencySource::Workspace {
///     name: "my-package".to_string(),
///     constraint: WorkspaceConstraint::Any,
/// };
///
/// // Git dependency
/// let git = DependencySource::Git {
///     name: "my-lib".to_string(),
///     repo: "https://github.com/user/my-lib.git".to_string(),
///     reference: GitReference::Branch("main".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencySource {
    /// Registry dependency with standard semver (npm registry default)
    ///
    /// Used for standard packages from npm registry.
    /// Example: `"react": "^18.0.0"`
    Registry {
        /// Package name
        name: String,
        /// Version requirement (semver)
        version_req: VersionReq,
    },

    /// Scoped package dependency
    ///
    /// Used for scoped packages from any registry.
    /// Example: `"@types/node": "^20.0.0"`
    Scoped {
        /// Scope name (without @)
        scope: String,
        /// Package name
        name: String,
        /// Version requirement (semver)
        version_req: VersionReq,
    },

    /// Explicit npm registry protocol
    ///
    /// Used when explicitly specifying npm registry.
    /// Example: `"npm:@mui/styled-engine-sc@5.3.0"`
    Npm {
        /// Package name (can include scope)
        name: String,
        /// Version requirement (semver)
        version_req: VersionReq,
    },

    /// JSR (JavaScript Registry) protocol
    ///
    /// Used for packages from jsr.io registry.
    /// Example: `"jsr:@luca/cases@^1.0.1"`
    Jsr {
        /// Scope name (without @)
        scope: String,
        /// Package name
        name: String,
        /// Version requirement (semver)
        version_req: VersionReq,
    },

    /// Workspace protocol dependency (monorepo only)
    ///
    /// Used for internal packages within a monorepo.
    /// Example: `"workspace:*"`, `"workspace:^"`
    Workspace {
        /// Package name
        name: String,
        /// Workspace constraint type
        constraint: WorkspaceConstraint,
    },

    /// Workspace path dependency (monorepo only)
    ///
    /// Used for explicit path references within a workspace.
    /// Example: `"workspace:../packages/core"`
    WorkspacePath {
        /// Package name
        name: String,
        /// Relative path to package
        path: PathBuf,
    },

    /// Workspace alias dependency (monorepo only)
    ///
    /// Used for aliased workspace references.
    /// Example: `"workspace:foo@*"`
    WorkspaceAlias {
        /// Alias name
        alias: String,
        /// Actual package name
        name: String,
        /// Workspace constraint
        constraint: WorkspaceConstraint,
    },

    /// File system dependency
    ///
    /// Used for local file dependencies.
    /// Example: `"file:../local-package"`
    File {
        /// Package name
        name: String,
        /// Path to local package
        path: PathBuf,
    },

    /// Git repository dependency
    ///
    /// Used for dependencies from git repositories.
    /// Example: `"git+https://github.com/user/repo.git#branch"`
    Git {
        /// Package name
        name: String,
        /// Git repository URL
        repo: String,
        /// Git reference (branch, tag, commit, semver)
        reference: GitReference,
    },

    /// GitHub shorthand dependency
    ///
    /// Used for GitHub repositories with shorthand syntax.
    /// Example: `"user/repo"`, `"github:user/repo"`
    GitHub {
        /// Package name
        name: String,
        /// GitHub username
        user: String,
        /// Repository name
        repo: String,
        /// Optional reference (branch, tag, commit)
        reference: Option<String>,
    },

    /// Private GitHub dependency with token
    ///
    /// Used for private GitHub repositories requiring authentication.
    GitHubPrivate {
        /// Package name
        name: String,
        /// Authentication token
        token: String,
        /// GitHub username
        user: String,
        /// Repository name
        repo: String,
    },

    /// Direct URL dependency
    ///
    /// Used for dependencies from direct tarball URLs.
    /// Example: `"https://example.com/package.tgz"`
    Url {
        /// Package name
        name: String,
        /// Direct URL to package tarball
        url: String,
    },
}

impl DependencySource {
    /// Get the package name for this dependency source
    ///
    /// # Returns
    ///
    /// The package name as a string slice
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::DependencySource;
    /// use semver::VersionReq;
    ///
    /// let source = DependencySource::Registry {
    ///     name: "react".to_string(),
    ///     version_req: VersionReq::parse("^18.0.0").unwrap(),
    /// };
    /// assert_eq!(source.name(), "react");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Registry { name, .. }
            | Self::Scoped { name, .. }
            | Self::Npm { name, .. }
            | Self::Jsr { name, .. }
            | Self::Workspace { name, .. }
            | Self::WorkspacePath { name, .. }
            | Self::WorkspaceAlias { name, .. }
            | Self::File { name, .. }
            | Self::Git { name, .. }
            | Self::GitHub { name, .. }
            | Self::GitHubPrivate { name, .. }
            | Self::Url { name, .. } => name,
        }
    }

    /// Get the dependency protocol for this source
    ///
    /// # Returns
    ///
    /// The dependency protocol enum value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{DependencySource, DependencyProtocol};
    /// use semver::VersionReq;
    ///
    /// let source = DependencySource::Registry {
    ///     name: "react".to_string(),
    ///     version_req: VersionReq::parse("^18.0.0").unwrap(),
    /// };
    /// assert_eq!(source.protocol(), DependencyProtocol::Registry);
    /// ```
    #[must_use]
    pub fn protocol(&self) -> DependencyProtocol {
        match self {
            Self::Registry { .. } => DependencyProtocol::Registry,
            Self::Scoped { .. } => DependencyProtocol::Scoped,
            Self::Npm { .. } => DependencyProtocol::Npm,
            Self::Jsr { .. } => DependencyProtocol::Jsr,
            Self::Workspace { .. } | Self::WorkspacePath { .. } | Self::WorkspaceAlias { .. } => {
                DependencyProtocol::Workspace
            }
            Self::File { .. } => DependencyProtocol::File,
            Self::Git { .. } => DependencyProtocol::Git,
            Self::GitHub { .. } | Self::GitHubPrivate { .. } => DependencyProtocol::GitHub,
            Self::Url { .. } => DependencyProtocol::Url,
        }
    }

    /// Check if this dependency source is supported in the given project context
    ///
    /// # Arguments
    ///
    /// * `context` - The project context to check against
    ///
    /// # Returns
    ///
    /// `true` if the dependency source is supported in the context, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{DependencySource, ProjectContext, SingleRepositoryContext, WorkspaceConstraint};
    ///
    /// let context = ProjectContext::Single(SingleRepositoryContext::default());
    /// let workspace_dep = DependencySource::Workspace {
    ///     name: "internal".to_string(),
    ///     constraint: WorkspaceConstraint::Any,
    /// };
    /// assert!(!workspace_dep.is_supported_in_context(&context));
    /// ```
    #[must_use]
    pub fn is_supported_in_context(&self, context: &ProjectContext) -> bool {
        let protocol = self.protocol();
        context.supported_protocols().contains(&protocol)
    }

    /// Check if this dependency source requires network access
    ///
    /// # Returns
    ///
    /// `true` if network access is required, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::DependencySource;
    /// use semver::VersionReq;
    ///
    /// let registry = DependencySource::Registry {
    ///     name: "react".to_string(),
    ///     version_req: VersionReq::parse("^18.0.0").unwrap(),
    /// };
    /// assert!(registry.requires_network());
    ///
    /// let workspace = DependencySource::Workspace {
    ///     name: "internal".to_string(),
    ///     constraint: WorkspaceConstraint::Any,
    /// };
    /// assert!(!workspace.requires_network());
    /// ```
    #[must_use]
    pub fn requires_network(&self) -> bool {
        self.protocol().requires_network()
    }

    /// Check if this dependency source is filesystem-based
    ///
    /// # Returns
    ///
    /// `true` if filesystem-based, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::DependencySource;
    /// use std::path::PathBuf;
    ///
    /// let file_dep = DependencySource::File {
    ///     name: "local".to_string(),
    ///     path: PathBuf::from("../local-package"),
    /// };
    /// assert!(file_dep.is_filesystem_based());
    /// ```
    #[must_use]
    pub fn is_filesystem_based(&self) -> bool {
        self.protocol().is_filesystem_based()
    }
}

impl Display for DependencySource {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Registry { name, version_req } => write!(f, "{}@{}", name, version_req),
            Self::Scoped { scope, name, version_req } => {
                write!(f, "@{}/{}@{}", scope, name, version_req)
            }
            Self::Npm { name, version_req } => write!(f, "npm:{}@{}", name, version_req),
            Self::Jsr { scope, name, version_req } => {
                write!(f, "jsr:@{}/{}@{}", scope, name, version_req)
            }
            Self::Workspace { name: _, constraint } => write!(f, "workspace:{}", constraint),
            Self::WorkspacePath { name: _, path } => write!(f, "workspace:{}", path.display()),
            Self::WorkspaceAlias { alias, name: _, constraint } => {
                write!(f, "workspace:{}@{}", alias, constraint)
            }
            Self::File { name: _, path } => write!(f, "file:{}", path.display()),
            Self::Git { name: _, repo, reference } => write!(f, "git+{}#{}", repo, reference),
            Self::GitHub { name: _, user, repo, reference } => {
                if let Some(ref_str) = reference {
                    write!(f, "{}/{}#{}", user, repo, ref_str)
                } else {
                    write!(f, "{}/{}", user, repo)
                }
            }
            Self::GitHubPrivate { name: _, user, repo, .. } => {
                write!(f, "github:{}/{}", user, repo)
            }
            Self::Url { name: _, url } => write!(f, "{}", url),
        }
    }
}

/// Workspace constraint types for workspace protocol dependencies
///
/// Workspace constraints define how workspace dependencies are resolved
/// and what version constraints apply to them.
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::context::WorkspaceConstraint;
///
/// // Any version in workspace
/// let any = WorkspaceConstraint::Any;
/// assert_eq!(any.to_string(), "*");
///
/// // Compatible version
/// let compatible = WorkspaceConstraint::Compatible;
/// assert_eq!(compatible.to_string(), "^");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceConstraint {
    /// Any version (`workspace:*`)
    ///
    /// Accepts any version of the workspace package.
    Any,
    /// Compatible version (`workspace:^`)
    ///
    /// Accepts compatible versions using caret range.
    Compatible,
    /// Patch version (`workspace:~`)
    ///
    /// Accepts patch-level changes using tilde range.
    Patch,
    /// Exact version constraint (`workspace:^1.0.0`)
    ///
    /// Uses an exact semver constraint.
    Exact(VersionReq),
}

impl Display for WorkspaceConstraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Any => write!(f, "*"),
            Self::Compatible => write!(f, "^"),
            Self::Patch => write!(f, "~"),
            Self::Exact(version_req) => write!(f, "{}", version_req),
        }
    }
}

impl FromStr for WorkspaceConstraint {
    type Err = PackageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "*" => Ok(Self::Any),
            "^" => Ok(Self::Compatible),
            "~" => Ok(Self::Patch),
            version_str => {
                let version_req = VersionReq::parse(version_str)
                    .map_err(|e| PackageError::Configuration(format!("Invalid workspace constraint '{}': {}", s, e)))?;
                Ok(Self::Exact(version_req))
            }
        }
    }
}

/// Git reference types for git-based dependencies
///
/// Git references specify which version of a git repository to use
/// as a dependency.
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::context::GitReference;
///
/// // Branch reference
/// let branch = GitReference::Branch("main".to_string());
/// assert_eq!(branch.to_string(), "main");
///
/// // Tag reference
/// let tag = GitReference::Tag("v1.0.0".to_string());
/// assert_eq!(tag.to_string(), "v1.0.0");
///
/// // Commit reference
/// let commit = GitReference::Commit("abc123".to_string());
/// assert_eq!(commit.to_string(), "abc123");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitReference {
    /// Branch reference
    ///
    /// References a specific branch in the git repository.
    /// Example: `"main"`, `"development"`
    Branch(String),
    /// Tag reference
    ///
    /// References a specific tag in the git repository.
    /// Example: `"v1.0.0"`, `"release-2023"`
    Tag(String),
    /// Commit reference
    ///
    /// References a specific commit hash.
    /// Example: `"abc123456"`, `"1234567890abcdef"`
    Commit(String),
    /// Semver reference
    ///
    /// References using semver constraints.
    /// Example: `"semver:^1.0.0"`, `"semver:~2.1.0"`
    Semver(VersionReq),
}

impl Display for GitReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Branch(branch) => write!(f, "{}", branch),
            Self::Tag(tag) => write!(f, "{}", tag),
            Self::Commit(commit) => write!(f, "{}", commit),
            Self::Semver(version_req) => write!(f, "semver:{}", version_req),
        }
    }
}

impl FromStr for GitReference {
    type Err = PackageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("semver:") {
            let version_str = &s[7..]; // Remove "semver:" prefix
            let version_req = VersionReq::parse(version_str)
                .map_err(|e| PackageError::Configuration(format!("Invalid git semver reference '{}': {}", s, e)))?;
            Ok(Self::Semver(version_req))
        } else if s.len() >= 7 && s.chars().all(|c| c.is_ascii_hexdigit()) {
            // Likely a commit hash (7+ hex characters)
            Ok(Self::Commit(s.to_string()))
        } else if s.starts_with('v') && Version::parse(&s[1..]).is_ok() {
            // Likely a version tag
            Ok(Self::Tag(s.to_string()))
        } else {
            // Default to branch
            Ok(Self::Branch(s.to_string()))
        }
    }
}

impl Default for GitReference {
    fn default() -> Self {
        Self::Branch("main".to_string())
    }
}