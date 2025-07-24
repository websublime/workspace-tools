//! # Context-Aware Dependency Parser
//!
//! ## What
//! Enterprise-grade dependency parsing service that converts dependency specification
//! strings into structured DependencySource variants. Supports all JavaScript ecosystem
//! protocols with context-aware validation and error handling.
//!
//! ## How
//! The parser analyzes dependency strings using protocol-specific regex patterns and
//! parsing logic. Context awareness ensures workspace protocols are rejected in single
//! repositories while being fully supported in monorepos.
//!
//! ## Why
//! JavaScript dependencies come in many formats beyond simple semver. This parser
//! provides robust, context-aware handling of all dependency protocols with
//! enterprise-grade error reporting and validation.

use crate::{
    context::{
        DependencySource, ProjectContext, DependencyProtocol, GitReference, WorkspaceConstraint,
    },
    errors::PackageError,
};
use regex::Regex;
use semver::VersionReq;
use std::{path::PathBuf, str::FromStr, sync::OnceLock};

/// Context-aware dependency parser for all JavaScript ecosystem protocols
///
/// This parser converts dependency specification strings into structured DependencySource
/// variants. It adapts its behavior based on project context, rejecting workspace
/// protocols in single repositories while supporting all protocols in monorepos.
///
/// ## Protocol Support
///
/// - **Registry**: `"^1.0.0"`, `"~2.1.0"`, `"latest"`
/// - **Scoped**: `"@types/node@^20.0.0"`
/// - **NPM**: `"npm:package@^1.0.0"`
/// - **JSR**: `"jsr:@scope/package@^1.0.0"`
/// - **Workspace**: `"workspace:*"`, `"workspace:^"`, `"workspace:../path"` (monorepo only)
/// - **File**: `"file:../local-package"`
/// - **Git**: `"git+https://github.com/user/repo.git#branch"`
/// - **GitHub**: `"user/repo"`, `"github:user/repo#tag"`
/// - **URL**: `"https://example.com/package.tgz"`
///
/// ## Context Awareness
///
/// The parser adapts behavior based on ProjectContext:
/// - **Single Repository**: Rejects workspace protocols with clear error messages
/// - **Monorepo**: Supports all protocols including workspace variants
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::context::{DependencyParser, ProjectContext, SingleRepositoryContext};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let context = ProjectContext::Single(SingleRepositoryContext::default());
/// let parser = DependencyParser::new(context);
///
/// // Parse registry dependency
/// let source = parser.parse("react", "^18.0.0")?;
/// println!("Parsed: {}", source);
///
/// // Parse workspace dependency (will fail in single repository)
/// let result = parser.parse("internal-package", "workspace:*");
/// assert!(result.is_err());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct DependencyParser {
    /// Project context for context-aware parsing
    context: ProjectContext,
}

impl DependencyParser {
    /// Create a new dependency parser with the given project context
    ///
    /// # Arguments
    ///
    /// * `context` - Project context for context-aware parsing behavior
    ///
    /// # Returns
    ///
    /// A new DependencyParser instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{DependencyParser, ProjectContext, MonorepoContext};
    ///
    /// let context = ProjectContext::Monorepo(MonorepoContext::default());
    /// let parser = DependencyParser::new(context);
    /// ```
    #[must_use]
    pub fn new(context: ProjectContext) -> Self {
        Self { context }
    }

    /// Parse a dependency specification into a DependencySource
    ///
    /// This method analyzes the dependency specification string and converts it
    /// into a structured DependencySource variant based on the detected protocol.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `spec` - Dependency specification string
    ///
    /// # Returns
    ///
    /// A parsed DependencySource or an error if parsing fails
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - The specification format is invalid
    /// - The protocol is not supported in the current context
    /// - Version requirements are malformed
    /// - Git or URL formats are invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{DependencyParser, ProjectContext, MonorepoContext};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let context = ProjectContext::Monorepo(MonorepoContext::default());
    /// let parser = DependencyParser::new(context);
    ///
    /// // Registry dependency
    /// let source = parser.parse("react", "^18.0.0")?;
    ///
    /// // Workspace dependency
    /// let workspace = parser.parse("my-package", "workspace:*")?;
    ///
    /// // Git dependency
    /// let git = parser.parse("my-lib", "git+https://github.com/user/my-lib.git#main")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(&self, name: &str, spec: &str) -> Result<DependencySource, PackageError> {
        // Detect protocol first
        let protocol = DependencyProtocol::parse(spec);

        // Context-aware protocol validation
        if !self.context.supported_protocols().contains(&protocol) {
            return Err(PackageError::UnsupportedOperation(format!(
                "Protocol '{}' is not supported in {} context. Dependency: {}@{}",
                protocol,
                if self.context.is_single() { "single repository" } else { "monorepo" },
                name,
                spec
            )));
        }

        // Parse based on detected protocol, but check if name is scoped for Registry protocol
        match protocol {
            DependencyProtocol::Registry => {
                // Check if this is actually a scoped package based on the name
                if name.starts_with('@') {
                    self.parse_scoped_dependency(name, spec)
                } else {
                    self.parse_registry_dependency(name, spec)
                }
            }
            DependencyProtocol::Scoped => self.parse_scoped_dependency(name, spec),
            DependencyProtocol::Npm => self.parse_npm_dependency(name, spec),
            DependencyProtocol::Jsr => self.parse_jsr_dependency(name, spec),
            DependencyProtocol::Workspace => self.parse_workspace_dependency(name, spec),
            DependencyProtocol::File => self.parse_file_dependency(name, spec),
            DependencyProtocol::Git => self.parse_git_dependency(name, spec),
            DependencyProtocol::GitHub => self.parse_github_dependency(name, spec),
            DependencyProtocol::Url => self.parse_url_dependency(name, spec),
            DependencyProtocol::Other(custom) => Err(PackageError::UnsupportedOperation(format!(
                "Custom protocol '{}' is not yet supported",
                custom
            ))),
        }
    }

    /// Parse registry dependency (standard npm registry)
    fn parse_registry_dependency(&self, name: &str, spec: &str) -> Result<DependencySource, PackageError> {
        let version_req = VersionReq::parse(spec)
            .map_err(|e| PackageError::Configuration(format!("Invalid version requirement '{}': {}", spec, e)))?;

        Ok(DependencySource::Registry {
            name: name.to_string(),
            version_req,
        })
    }

    /// Parse scoped package dependency
    fn parse_scoped_dependency(&self, name: &str, spec: &str) -> Result<DependencySource, PackageError> {
        // Scoped package name should be like @scope/package
        if !name.starts_with('@') {
            return Err(PackageError::Configuration(format!(
                "Scoped package name must start with '@': {}",
                name
            )));
        }

        let parts: Vec<&str> = name[1..].split('/').collect(); // Remove @ and split
        if parts.len() != 2 {
            return Err(PackageError::Configuration(format!(
                "Invalid scoped package name format: {}",
                name
            )));
        }

        let version_req = VersionReq::parse(spec)
            .map_err(|e| PackageError::Configuration(format!("Invalid version requirement '{}': {}", spec, e)))?;

        Ok(DependencySource::Scoped {
            scope: parts[0].to_string(),
            name: parts[1].to_string(),
            version_req,
        })
    }

    /// Parse explicit npm dependency
    fn parse_npm_dependency(&self, name: &str, spec: &str) -> Result<DependencySource, PackageError> {
        // Format: npm:package@version or npm:@scope/package@version
        if !spec.starts_with("npm:") {
            return Err(PackageError::Configuration(format!(
                "NPM dependency must start with 'npm:': {}",
                spec
            )));
        }

        let npm_spec = &spec[4..]; // Remove "npm:" prefix
        let (package_name, version_str) = Self::split_package_version(npm_spec)?;

        let version_req = VersionReq::parse(version_str)
            .map_err(|e| PackageError::Configuration(format!("Invalid version requirement '{}': {}", version_str, e)))?;

        Ok(DependencySource::Npm {
            name: package_name.to_string(),
            version_req,
        })
    }

    /// Parse JSR dependency
    fn parse_jsr_dependency(&self, name: &str, spec: &str) -> Result<DependencySource, PackageError> {
        // Format: jsr:@scope/package@version
        if !spec.starts_with("jsr:@") {
            return Err(PackageError::Configuration(format!(
                "JSR dependency must start with 'jsr:@': {}",
                spec
            )));
        }

        let jsr_spec = &spec[4..]; // Remove "jsr:" prefix
        let (package_name, version_str) = Self::split_package_version(jsr_spec)?;

        // JSR packages must be scoped
        if !package_name.starts_with('@') {
            return Err(PackageError::Configuration(format!(
                "JSR packages must be scoped: {}",
                package_name
            )));
        }

        let parts: Vec<&str> = package_name[1..].split('/').collect(); // Remove @ and split
        if parts.len() != 2 {
            return Err(PackageError::Configuration(format!(
                "Invalid JSR package name format: {}",
                package_name
            )));
        }

        let version_req = VersionReq::parse(version_str)
            .map_err(|e| PackageError::Configuration(format!("Invalid version requirement '{}': {}", version_str, e)))?;

        Ok(DependencySource::Jsr {
            scope: parts[0].to_string(),
            name: parts[1].to_string(),
            version_req,
        })
    }

    /// Parse workspace dependency
    fn parse_workspace_dependency(&self, name: &str, spec: &str) -> Result<DependencySource, PackageError> {
        if !spec.starts_with("workspace:") {
            return Err(PackageError::Configuration(format!(
                "Workspace dependency must start with 'workspace:': {}",
                spec
            )));
        }

        let workspace_spec = &spec[10..]; // Remove "workspace:" prefix

        // Handle different workspace formats
        if workspace_spec.starts_with("../") || workspace_spec.starts_with("./") {
            // Workspace path: workspace:../path
            Ok(DependencySource::WorkspacePath {
                name: name.to_string(),
                path: PathBuf::from(workspace_spec),
            })
        } else if workspace_spec.contains('@') {
            // Workspace alias: workspace:alias@constraint
            let (alias, constraint_str) = Self::split_package_version(workspace_spec)?;
            let constraint = WorkspaceConstraint::from_str(constraint_str)?;
            
            Ok(DependencySource::WorkspaceAlias {
                alias: alias.to_string(),
                name: name.to_string(),
                constraint,
            })
        } else {
            // Workspace constraint: workspace:*, workspace:^, workspace:~, workspace:^1.0.0
            let constraint = WorkspaceConstraint::from_str(workspace_spec)?;
            
            Ok(DependencySource::Workspace {
                name: name.to_string(),
                constraint,
            })
        }
    }

    /// Parse file dependency
    fn parse_file_dependency(&self, name: &str, spec: &str) -> Result<DependencySource, PackageError> {
        if !spec.starts_with("file:") {
            return Err(PackageError::Configuration(format!(
                "File dependency must start with 'file:': {}",
                spec
            )));
        }

        let file_path = &spec[5..]; // Remove "file:" prefix
        
        Ok(DependencySource::File {
            name: name.to_string(),
            path: PathBuf::from(file_path),
        })
    }

    /// Parse git dependency
    fn parse_git_dependency(&self, name: &str, spec: &str) -> Result<DependencySource, PackageError> {
        // Format: git+https://github.com/user/repo.git#reference
        let git_regex = Self::git_regex();
        
        if let Some(captures) = git_regex.captures(spec) {
            let repo = captures.get(1)
                .ok_or_else(|| PackageError::Configuration(format!("Invalid git URL format: {}", spec)))?
                .as_str();
            
            let reference_str = captures.get(2).map(|m| m.as_str()).unwrap_or("main");
            let reference = GitReference::from_str(reference_str)?;
            
            Ok(DependencySource::Git {
                name: name.to_string(),
                repo: repo.to_string(),
                reference,
            })
        } else {
            Err(PackageError::Configuration(format!(
                "Invalid git dependency format: {}",
                spec
            )))
        }
    }

    /// Parse GitHub dependency
    fn parse_github_dependency(&self, name: &str, spec: &str) -> Result<DependencySource, PackageError> {
        // Formats: user/repo, user/repo#reference, github:user/repo, github:user/repo#reference
        let github_spec = if spec.starts_with("github:") {
            &spec[7..] // Remove "github:" prefix
        } else {
            spec
        };

        let (repo_part, reference) = if github_spec.contains('#') {
            let parts: Vec<&str> = github_spec.split('#').collect();
            if parts.len() != 2 {
                return Err(PackageError::Configuration(format!(
                    "Invalid GitHub reference format: {}",
                    spec
                )));
            }
            (parts[0], Some(parts[1].to_string()))
        } else {
            (github_spec, None)
        };

        let repo_parts: Vec<&str> = repo_part.split('/').collect();
        if repo_parts.len() != 2 {
            return Err(PackageError::Configuration(format!(
                "Invalid GitHub repository format: {}",
                spec
            )));
        }

        Ok(DependencySource::GitHub {
            name: name.to_string(),
            user: repo_parts[0].to_string(),
            repo: repo_parts[1].to_string(),
            reference,
        })
    }

    /// Parse URL dependency
    fn parse_url_dependency(&self, name: &str, spec: &str) -> Result<DependencySource, PackageError> {
        // Validate URL format
        if !spec.starts_with("http://") && !spec.starts_with("https://") {
            return Err(PackageError::Configuration(format!(
                "URL dependency must start with http:// or https://: {}",
                spec
            )));
        }

        Ok(DependencySource::Url {
            name: name.to_string(),
            url: spec.to_string(),
        })
    }

    /// Split package@version format
    fn split_package_version(spec: &str) -> Result<(&str, &str), PackageError> {
        let last_at = spec.rfind('@');
        if let Some(at_pos) = last_at {
            // Handle scoped packages: @scope/package@version
            if spec.starts_with('@') && spec[1..at_pos].contains('/') {
                Ok((&spec[..at_pos], &spec[at_pos + 1..]))
            } else if spec.starts_with('@') {
                // This is a scoped package without version separator
                Err(PackageError::Configuration(format!(
                    "Scoped package missing version: {}",
                    spec
                )))
            } else {
                // Regular package@version
                Ok((&spec[..at_pos], &spec[at_pos + 1..]))
            }
        } else {
            Err(PackageError::Configuration(format!(
                "Missing version separator '@' in: {}",
                spec
            )))
        }
    }

    /// Get compiled git regex
    fn git_regex() -> &'static Regex {
        static GIT_REGEX: OnceLock<Regex> = OnceLock::new();
        GIT_REGEX.get_or_init(|| {
            Regex::new(r"^git\+(.+?)(?:#(.+))?$")
                .expect("Invalid git regex pattern")
        })
    }

    /// Get the project context
    ///
    /// # Returns
    ///
    /// Reference to the project context
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{DependencyParser, ProjectContext, SingleRepositoryContext};
    ///
    /// let context = ProjectContext::Single(SingleRepositoryContext::default());
    /// let parser = DependencyParser::new(context.clone());
    /// 
    /// assert_eq!(parser.context(), &context);
    /// ```
    #[must_use]
    pub fn context(&self) -> &ProjectContext {
        &self.context
    }

    /// Check if workspace protocols are supported in the current context
    ///
    /// # Returns
    ///
    /// `true` if workspace protocols are supported, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{DependencyParser, ProjectContext, MonorepoContext, SingleRepositoryContext};
    ///
    /// let monorepo_context = ProjectContext::Monorepo(MonorepoContext::default());
    /// let monorepo_parser = DependencyParser::new(monorepo_context);
    /// assert!(monorepo_parser.supports_workspace_protocols());
    ///
    /// let single_context = ProjectContext::Single(SingleRepositoryContext::default());
    /// let single_parser = DependencyParser::new(single_context);
    /// assert!(!single_parser.supports_workspace_protocols());
    /// ```
    #[must_use]
    pub fn supports_workspace_protocols(&self) -> bool {
        self.context.supported_protocols().contains(&DependencyProtocol::Workspace)
    }

    /// Validate a dependency specification without parsing
    ///
    /// This method performs lightweight validation to check if a dependency
    /// specification is potentially valid in the current context without
    /// performing full parsing.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `spec` - Dependency specification string
    ///
    /// # Returns
    ///
    /// `true` if the specification is potentially valid, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::context::{DependencyParser, ProjectContext, SingleRepositoryContext};
    ///
    /// let context = ProjectContext::Single(SingleRepositoryContext::default());
    /// let parser = DependencyParser::new(context);
    ///
    /// assert!(parser.validate("react", "^18.0.0"));
    /// assert!(!parser.validate("internal", "workspace:*"));
    /// ```
    #[must_use]
    pub fn validate(&self, name: &str, spec: &str) -> bool {
        let protocol = DependencyProtocol::parse(spec);
        self.context.supported_protocols().contains(&protocol)
    }
}