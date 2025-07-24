//! # Dependency Protocol Support
//!
//! This module defines the dependency protocols supported by the package tools
//! and how they are used in different project contexts.
//!
//! ## Protocol Coverage
//!
//! The package tools support all major dependency protocols from the JavaScript ecosystem:
//!
//! - **Registry protocols**: npm, jsr, scoped packages
//! - **Version control**: git, GitHub shortcuts
//! - **File system**: file, workspace protocols
//! - **Network**: direct URLs
//!
//! ## Context-Aware Protocol Support
//!
//! Different project contexts support different sets of protocols:
//!
//! - **Single Repository**: All protocols except workspace
//! - **Monorepo**: All protocols including workspace

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Dependency protocol types supported by the package tools
///
/// This enum represents all the dependency protocols that can be used
/// to specify package dependencies in the JavaScript ecosystem.
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::context::DependencyProtocol;
///
/// // Check if a protocol is registry-based
/// assert!(DependencyProtocol::Npm.is_registry_based());
/// assert!(!DependencyProtocol::Git.is_registry_based());
///
/// // Check workspace protocol support
/// assert!(DependencyProtocol::Workspace.is_workspace_only());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyProtocol {
    /// NPM registry protocol (registry.npmjs.org)
    Npm,
    /// JSR registry protocol (jsr.io)
    Jsr,
    /// Scoped packages (@scope/package)
    Scoped,
    /// Generic registry (configurable registry URL)
    Registry,
    /// Git repository protocol
    Git,
    /// GitHub shorthand (user/repo)
    GitHub,
    /// File system path (file:../)
    File,
    /// Workspace protocol (workspace:*)
    Workspace,
    /// Direct URL download
    Url,
    /// Future/custom protocols
    Other(String),
}

impl DependencyProtocol {
    /// Get all supported dependency protocols
    ///
    /// # Returns
    ///
    /// A vector containing all supported dependency protocols
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::DependencyProtocol;
    ///
    /// let all_protocols = DependencyProtocol::all();
    /// assert!(all_protocols.contains(&DependencyProtocol::Npm));
    /// assert!(all_protocols.contains(&DependencyProtocol::Workspace));
    /// ```
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::Npm,
            Self::Jsr,
            Self::Scoped,
            Self::Registry,
            Self::Git,
            Self::GitHub,
            Self::File,
            Self::Workspace,
            Self::Url,
        ]
    }

    /// Get all protocols except workspace (for single repository contexts)
    ///
    /// Single repositories don't support workspace protocols since they
    /// don't have multiple packages.
    ///
    /// # Returns
    ///
    /// A vector containing all protocols except workspace
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::DependencyProtocol;
    ///
    /// let single_repo_protocols = DependencyProtocol::all_except_workspace();
    /// assert!(single_repo_protocols.contains(&DependencyProtocol::Npm));
    /// assert!(!single_repo_protocols.contains(&DependencyProtocol::Workspace));
    /// ```
    #[must_use]
    pub fn all_except_workspace() -> Vec<Self> {
        vec![
            Self::Npm,
            Self::Jsr,
            Self::Scoped,
            Self::Registry,
            Self::Git,
            Self::GitHub,
            Self::File,
            Self::Url,
        ]
    }

    /// Get registry-based protocols only
    ///
    /// # Returns
    ///
    /// A vector containing only registry-based protocols
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::DependencyProtocol;
    ///
    /// let registry_protocols = DependencyProtocol::registry_only();
    /// assert!(registry_protocols.contains(&DependencyProtocol::Npm));
    /// assert!(registry_protocols.contains(&DependencyProtocol::Jsr));
    /// assert!(!registry_protocols.contains(&DependencyProtocol::Git));
    /// ```
    #[must_use]
    pub fn registry_only() -> Vec<Self> {
        vec![
            Self::Npm,
            Self::Jsr,
            Self::Scoped,
            Self::Registry,
        ]
    }

    /// Check if this protocol is registry-based
    ///
    /// Registry-based protocols involve downloading packages from
    /// a package registry rather than from source control or file system.
    ///
    /// # Returns
    ///
    /// `true` if this is a registry-based protocol, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::DependencyProtocol;
    ///
    /// assert!(DependencyProtocol::Npm.is_registry_based());
    /// assert!(DependencyProtocol::Jsr.is_registry_based());
    /// assert!(!DependencyProtocol::Git.is_registry_based());
    /// assert!(!DependencyProtocol::File.is_registry_based());
    /// ```
    #[must_use]
    pub fn is_registry_based(&self) -> bool {
        matches!(self, Self::Npm | Self::Jsr | Self::Scoped | Self::Registry)
    }

    /// Check if this protocol is workspace-only
    ///
    /// Workspace-only protocols are only valid in monorepo contexts
    /// and should be rejected in single repository contexts.
    ///
    /// # Returns
    ///
    /// `true` if this is a workspace-only protocol, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::DependencyProtocol;
    ///
    /// assert!(DependencyProtocol::Workspace.is_workspace_only());
    /// assert!(!DependencyProtocol::Npm.is_workspace_only());
    /// assert!(!DependencyProtocol::Git.is_workspace_only());
    /// ```
    #[must_use]
    pub fn is_workspace_only(&self) -> bool {
        matches!(self, Self::Workspace)
    }

    /// Check if this protocol requires network access
    ///
    /// Network-requiring protocols need internet connectivity and
    /// should be optimized for network operations in single repositories.
    ///
    /// # Returns
    ///
    /// `true` if this protocol requires network access, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::DependencyProtocol;
    ///
    /// assert!(DependencyProtocol::Npm.requires_network());
    /// assert!(DependencyProtocol::Git.requires_network());
    /// assert!(!DependencyProtocol::File.requires_network());
    /// assert!(!DependencyProtocol::Workspace.requires_network());
    /// ```
    #[must_use]
    pub fn requires_network(&self) -> bool {
        matches!(
            self,
            Self::Npm | Self::Jsr | Self::Registry | Self::Git | Self::GitHub | Self::Url
        )
    }

    /// Check if this protocol is file system based
    ///
    /// File system based protocols operate on local files and should be
    /// optimized for filesystem operations in monorepos.
    ///
    /// # Returns
    ///
    /// `true` if this is a file system based protocol, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::DependencyProtocol;
    ///
    /// assert!(DependencyProtocol::File.is_filesystem_based());
    /// assert!(DependencyProtocol::Workspace.is_filesystem_based());
    /// assert!(!DependencyProtocol::Npm.is_filesystem_based());
    /// assert!(!DependencyProtocol::Git.is_filesystem_based());
    /// ```
    #[must_use]
    pub fn is_filesystem_based(&self) -> bool {
        matches!(self, Self::File | Self::Workspace)
    }

    /// Parse a dependency string to detect the protocol
    ///
    /// This method analyzes a dependency specification string and
    /// determines which protocol it uses.
    ///
    /// # Arguments
    ///
    /// * `dep_string` - The dependency specification string
    ///
    /// # Returns
    ///
    /// The detected dependency protocol
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::DependencyProtocol;
    ///
    /// assert_eq!(DependencyProtocol::parse("^1.0.0"), DependencyProtocol::Npm);
    /// assert_eq!(DependencyProtocol::parse("workspace:*"), DependencyProtocol::Workspace);
    /// assert_eq!(DependencyProtocol::parse("git+https://github.com/user/repo.git"), DependencyProtocol::Git);
    /// assert_eq!(DependencyProtocol::parse("file:../local-package"), DependencyProtocol::File);
    /// ```
    #[must_use]
    pub fn parse(dep_string: &str) -> Self {
        if dep_string.starts_with("workspace:") {
            Self::Workspace
        } else if dep_string.starts_with("jsr:") {
            Self::Jsr
        } else if dep_string.starts_with("git+") || dep_string.ends_with(".git") {
            Self::Git
        } else if dep_string.starts_with("file:") {
            Self::File
        } else if dep_string.starts_with("http://") || dep_string.starts_with("https://") {
            // Check if it's a GitHub shorthand
            if dep_string.contains("github.com") && dep_string.matches('/').count() == 2 {
                Self::GitHub
            } else {
                Self::Url
            }
        } else if dep_string.contains('/') && !dep_string.contains('@') {
            // GitHub shorthand: user/repo
            Self::GitHub
        } else if dep_string.starts_with('@') {
            Self::Scoped
        } else {
            // Default to NPM for semver-like strings
            Self::Npm
        }
    }
}

impl Display for DependencyProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Npm => write!(f, "npm"),
            Self::Jsr => write!(f, "jsr"),
            Self::Scoped => write!(f, "scoped"),
            Self::Registry => write!(f, "registry"),
            Self::Git => write!(f, "git"),
            Self::GitHub => write!(f, "github"),
            Self::File => write!(f, "file"),
            Self::Workspace => write!(f, "workspace"),
            Self::Url => write!(f, "url"),
            Self::Other(name) => write!(f, "{name}"),
        }
    }
}

/// Protocol support configuration for different project contexts
///
/// This struct manages which protocols are supported and how they
/// should be handled in different project contexts.
#[derive(Debug, Clone)]
pub struct ProtocolSupport {
    /// List of supported protocols
    pub supported: Vec<DependencyProtocol>,
    /// Whether to reject unsupported protocols with an error
    pub strict_mode: bool,
    /// Whether to emit warnings for discouraged protocols
    pub emit_warnings: bool,
}

impl ProtocolSupport {
    /// Create protocol support for single repository context
    ///
    /// # Returns
    ///
    /// Protocol support configuration for single repositories
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{ProtocolSupport, DependencyProtocol};
    ///
    /// let support = ProtocolSupport::for_single_repository();
    /// assert!(support.is_supported(&DependencyProtocol::Npm));
    /// assert!(!support.is_supported(&DependencyProtocol::Workspace));
    /// ```
    #[must_use]
    pub fn for_single_repository() -> Self {
        Self {
            supported: DependencyProtocol::all_except_workspace(),
            strict_mode: true,  // Reject workspace protocols
            emit_warnings: true,
        }
    }

    /// Create protocol support for monorepo context
    ///
    /// # Returns
    ///
    /// Protocol support configuration for monorepos
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{ProtocolSupport, DependencyProtocol};
    ///
    /// let support = ProtocolSupport::for_monorepo();
    /// assert!(support.is_supported(&DependencyProtocol::Npm));
    /// assert!(support.is_supported(&DependencyProtocol::Workspace));
    /// ```
    #[must_use]
    pub fn for_monorepo() -> Self {
        Self {
            supported: DependencyProtocol::all(),
            strict_mode: false, // Allow all protocols with warnings
            emit_warnings: true,
        }
    }

    /// Check if a protocol is supported in this context
    ///
    /// # Arguments
    ///
    /// * `protocol` - The protocol to check
    ///
    /// # Returns
    ///
    /// `true` if the protocol is supported, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{ProtocolSupport, DependencyProtocol};
    ///
    /// let support = ProtocolSupport::for_single_repository();
    /// assert!(support.is_supported(&DependencyProtocol::Npm));
    /// assert!(!support.is_supported(&DependencyProtocol::Workspace));
    /// ```
    #[must_use]
    pub fn is_supported(&self, protocol: &DependencyProtocol) -> bool {
        self.supported.contains(protocol)
    }

    /// Validate a dependency string against supported protocols
    ///
    /// # Arguments
    ///
    /// * `dep_string` - The dependency specification to validate
    ///
    /// # Returns
    ///
    /// A result indicating if the dependency is valid, with optional warnings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::ProtocolSupport;
    ///
    /// let support = ProtocolSupport::for_single_repository();
    /// let result = support.validate("^1.0.0");
    /// assert!(result.is_valid);
    ///
    /// let result = support.validate("workspace:*");
    /// assert!(!result.is_valid);
    /// ```
    #[must_use]
    pub fn validate(&self, dep_string: &str) -> ValidationResult {
        let protocol = DependencyProtocol::parse(dep_string);
        let is_supported = self.is_supported(&protocol);

        let mut warnings = Vec::new();
        
        if !is_supported && self.emit_warnings {
            warnings.push(format!("Protocol '{protocol}' is not supported in this context"));
        }

        ValidationResult {
            is_valid: is_supported || !self.strict_mode,
            protocol,
            warnings,
        }
    }
}

/// Result of protocol validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the dependency is valid in this context
    pub is_valid: bool,
    /// The detected protocol
    pub protocol: DependencyProtocol,
    /// Any warnings generated during validation
    pub warnings: Vec<String>,
}