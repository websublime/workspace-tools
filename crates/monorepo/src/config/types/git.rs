//! Git configuration types
//!
//! This module defines configuration structures for Git operations,
//! including default references and branch configurations.

use serde::{Deserialize, Serialize};

/// Configuration for Git operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Default reference for change detection (e.g., "HEAD~1", "main")
    pub default_since_ref: String,

    /// Default target for comparisons (e.g., "HEAD")
    pub default_until_ref: String,

    /// Remote name for push operations (e.g., "origin")
    pub default_remote: String,

    /// Branch configuration
    pub branches: BranchConfig,

    /// Repository hosting configuration for URL generation
    pub repository: RepositoryHostConfig,
}

/// Configuration for branch operations and classifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchConfig {
    /// List of main/production branches
    pub main_branches: Vec<String>,

    /// List of development branches
    pub develop_branches: Vec<String>,

    /// List of release branch prefixes
    pub release_prefixes: Vec<String>,

    /// List of feature branch prefixes
    pub feature_prefixes: Vec<String>,

    /// List of hotfix branch prefixes
    pub hotfix_prefixes: Vec<String>,

    /// Default branch for new features
    pub default_base_branch: String,
}

/// Configuration for repository hosting providers
///
/// This configuration enables dynamic URL generation for different Git hosting providers
/// including GitHub, GitLab, Bitbucket, and custom enterprise instances.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::config::types::git::RepositoryHostConfig;
///
/// // GitHub Enterprise configuration
/// let config = RepositoryHostConfig::github_enterprise("github.company.com");
///
/// // GitLab custom instance
/// let config = RepositoryHostConfig::gitlab_custom("gitlab.company.com");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryHostConfig {
    /// Repository hosting provider type
    pub provider: RepositoryProvider,

    /// Base URL for the repository host (e.g., "github.com", "gitlab.example.com")
    pub base_url: String,

    /// URL patterns for different operations
    pub url_patterns: UrlPatterns,

    /// Auto-detect provider from git remote URL
    pub auto_detect: bool,

    /// Override repository URL (useful for testing or custom setups)
    pub url_override: Option<String>,
}

/// Repository hosting provider types
///
/// Defines supported Git hosting providers with their specific URL patterns
/// and SSH conversion rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepositoryProvider {
    /// GitHub (github.com)
    GitHub,
    /// GitHub Enterprise Server
    GitHubEnterprise,
    /// GitLab (gitlab.com or custom instance)
    GitLab,
    /// Bitbucket (bitbucket.org)
    Bitbucket,
    /// Azure DevOps / TFS
    AzureDevOps,
    /// Custom or self-hosted provider
    Custom,
}

/// URL patterns for repository operations
///
/// Configurable URL templates that support variable substitution for
/// generating links to commits, comparisons, and other repository views.
///
/// # Supported Variables
///
/// - `{base_url}` - Repository base URL (e.g., "github.com")
/// - `{owner}` - Repository owner/organization
/// - `{repo}` - Repository name
/// - `{hash}` - Commit hash
/// - `{from}` - Source reference for comparisons
/// - `{to}` - Target reference for comparisons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlPatterns {
    /// Pattern for commit URLs
    ///
    /// Default: `https://{base_url}/{owner}/{repo}/commit/{hash}`
    pub commit_url: String,

    /// Pattern for compare URLs  
    ///
    /// Default: `https://{base_url}/{owner}/{repo}/compare/{from}...{to}`
    pub compare_url: String,

    /// SSH to HTTPS conversion patterns
    pub ssh_conversions: Vec<SshConversion>,
}

/// SSH to HTTPS URL conversion configuration
///
/// Defines patterns for converting SSH URLs to HTTPS URLs for web links.
/// This is essential for generating clickable links in changelogs and reports.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::config::types::git::SshConversion;
///
/// // GitHub conversion
/// let github = SshConversion {
///     ssh_pattern: "git@github.com:".to_string(),
///     https_replacement: "https://github.com/".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConversion {
    /// SSH pattern to match (e.g., "git@github.com:")
    pub ssh_pattern: String,

    /// HTTPS replacement (e.g., "<https://github.com/>")
    pub https_replacement: String,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            default_since_ref: "HEAD~1".to_string(),
            default_until_ref: "HEAD".to_string(),
            default_remote: "origin".to_string(),
            branches: BranchConfig::default(),
            repository: RepositoryHostConfig::default(),
        }
    }
}

impl Default for BranchConfig {
    fn default() -> Self {
        Self {
            main_branches: vec![
                "main".to_string(),
                "master".to_string(),
                "trunk".to_string(),
                "develop".to_string(), // Include develop as main branch for workflow purposes
            ],
            develop_branches: vec![
                "develop".to_string(),
                "dev".to_string(),
                "development".to_string(),
            ],
            release_prefixes: vec![
                "release/".to_string(),
                "releases/".to_string(),
                "rel/".to_string(), // Include rel/ prefix for release branches
            ],
            feature_prefixes: vec![
                "feature/".to_string(),
                "feat/".to_string(),
                "features/".to_string(),
            ],
            hotfix_prefixes: vec!["hotfix/".to_string(), "fix/".to_string(), "bugfix/".to_string()],
            default_base_branch: "main".to_string(),
        }
    }
}

impl GitConfig {
    /// Create a new GitConfig with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a GitConfig with custom default reference
    #[must_use]
    pub fn with_default_since_ref(mut self, since_ref: impl Into<String>) -> Self {
        self.default_since_ref = since_ref.into();
        self
    }

    /// Create a GitConfig with custom branch configuration
    #[must_use]
    pub fn with_branches(mut self, branches: BranchConfig) -> Self {
        self.branches = branches;
        self
    }
}

impl BranchConfig {
    /// Check if a branch is considered a main/production branch
    #[must_use]
    pub fn is_main_branch(&self, branch: &str) -> bool {
        self.main_branches.iter().any(|main_branch| branch == main_branch)
    }

    /// Check if a branch is considered a development branch
    #[must_use]
    pub fn is_develop_branch(&self, branch: &str) -> bool {
        self.develop_branches.iter().any(|dev_branch| branch == dev_branch)
    }

    /// Check if a branch is a release branch
    #[must_use]
    pub fn is_release_branch(&self, branch: &str) -> bool {
        self.release_prefixes.iter().any(|prefix| branch.starts_with(prefix))
    }

    /// Check if a branch is a feature branch
    #[must_use]
    pub fn is_feature_branch(&self, branch: &str) -> bool {
        self.feature_prefixes.iter().any(|prefix| branch.starts_with(prefix))
    }

    /// Check if a branch is a hotfix branch
    #[must_use]
    pub fn is_hotfix_branch(&self, branch: &str) -> bool {
        self.hotfix_prefixes.iter().any(|prefix| branch.starts_with(prefix))
    }

    /// Check if a branch is protected (main or develop)
    #[must_use]
    pub fn is_protected_branch(&self, branch: &str) -> bool {
        self.is_main_branch(branch) || self.is_develop_branch(branch)
    }

    /// Get the appropriate base branch for a new branch
    #[must_use]
    pub fn get_base_branch(&self, branch_type: BranchType) -> &str {
        match branch_type {
            BranchType::Feature | BranchType::Release => &self.default_base_branch,
            BranchType::Hotfix => {
                // Hotfixes typically branch from main
                self.main_branches.first().unwrap_or(&self.default_base_branch)
            }
            BranchType::Main | BranchType::Develop => branch_type.as_str(),
        }
    }
}

/// Types of branches in a Git workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchType {
    /// Main/production branch
    Main,
    /// Development branch
    Develop,
    /// Feature branch
    Feature,
    /// Release branch
    Release,
    /// Hotfix branch
    Hotfix,
}

impl BranchType {
    /// Get the branch type as a string
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Develop => "develop",
            Self::Feature => "feature",
            Self::Release => "release",
            Self::Hotfix => "hotfix",
        }
    }
}

impl BranchConfig {
    /// Determine the type of a branch based on its name
    #[must_use]
    pub fn get_branch_type(&self, branch: &str) -> BranchType {
        if self.is_main_branch(branch) {
            BranchType::Main
        } else if self.is_develop_branch(branch) {
            BranchType::Develop
        } else if self.is_release_branch(branch) {
            BranchType::Release
        } else if self.is_hotfix_branch(branch) {
            BranchType::Hotfix
        } else {
            BranchType::Feature // Default to feature branch
        }
    }

    /// Get all valid branch prefixes for validation
    #[must_use]
    pub fn get_all_valid_prefixes(&self) -> Vec<String> {
        let mut prefixes = Vec::new();
        prefixes.extend(self.feature_prefixes.clone());
        prefixes.extend(self.hotfix_prefixes.clone());
        prefixes.extend(self.release_prefixes.clone());
        prefixes
    }
}

impl Default for RepositoryHostConfig {
    fn default() -> Self {
        Self::github()
    }
}

impl Default for UrlPatterns {
    fn default() -> Self {
        Self::github()
    }
}

impl RepositoryHostConfig {
    /// Create configuration for GitHub (github.com)
    ///
    /// # Returns
    ///
    /// A `RepositoryHostConfig` configured for standard GitHub.com usage
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::config::types::git::RepositoryHostConfig;
    ///
    /// let config = RepositoryHostConfig::github();
    /// assert_eq!(config.base_url, "github.com");
    /// ```
    #[must_use]
    pub fn github() -> Self {
        Self {
            provider: RepositoryProvider::GitHub,
            base_url: "github.com".to_string(),
            url_patterns: UrlPatterns::github(),
            auto_detect: true,
            url_override: None,
        }
    }

    /// Create configuration for GitHub Enterprise
    ///
    /// # Arguments
    ///
    /// * `enterprise_url` - The base URL of the GitHub Enterprise instance (e.g., "github.company.com")
    ///
    /// # Returns
    ///
    /// A `RepositoryHostConfig` configured for GitHub Enterprise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::config::types::git::RepositoryHostConfig;
    ///
    /// let config = RepositoryHostConfig::github_enterprise("github.company.com");
    /// assert_eq!(config.base_url, "github.company.com");
    /// ```
    #[must_use]
    pub fn github_enterprise(enterprise_url: &str) -> Self {
        Self {
            provider: RepositoryProvider::GitHubEnterprise,
            base_url: enterprise_url.to_string(),
            url_patterns: UrlPatterns::github_enterprise(enterprise_url),
            auto_detect: true,
            url_override: None,
        }
    }

    /// Create configuration for GitLab (gitlab.com)
    ///
    /// # Returns
    ///
    /// A `RepositoryHostConfig` configured for standard GitLab.com usage
    #[must_use]
    pub fn gitlab() -> Self {
        Self {
            provider: RepositoryProvider::GitLab,
            base_url: "gitlab.com".to_string(),
            url_patterns: UrlPatterns::gitlab(),
            auto_detect: true,
            url_override: None,
        }
    }

    /// Create configuration for custom GitLab instance
    ///
    /// # Arguments
    ///
    /// * `gitlab_url` - The base URL of the GitLab instance (e.g., "gitlab.company.com")
    ///
    /// # Returns
    ///
    /// A `RepositoryHostConfig` configured for custom GitLab instance
    #[must_use]
    pub fn gitlab_custom(gitlab_url: &str) -> Self {
        Self {
            provider: RepositoryProvider::GitLab,
            base_url: gitlab_url.to_string(),
            url_patterns: UrlPatterns::gitlab_custom(gitlab_url),
            auto_detect: true,
            url_override: None,
        }
    }

    /// Create configuration for Bitbucket (bitbucket.org)
    ///
    /// # Returns
    ///
    /// A `RepositoryHostConfig` configured for Bitbucket.org usage
    #[must_use]
    pub fn bitbucket() -> Self {
        Self {
            provider: RepositoryProvider::Bitbucket,
            base_url: "bitbucket.org".to_string(),
            url_patterns: UrlPatterns::bitbucket(),
            auto_detect: true,
            url_override: None,
        }
    }

    /// Create configuration for Azure DevOps
    ///
    /// # Arguments
    ///
    /// * `organization` - The Azure DevOps organization name
    ///
    /// # Returns
    ///
    /// A `RepositoryHostConfig` configured for Azure DevOps
    #[must_use]
    pub fn azure_devops(organization: &str) -> Self {
        Self {
            provider: RepositoryProvider::AzureDevOps,
            base_url: format!("dev.azure.com/{organization}"),
            url_patterns: UrlPatterns::azure_devops(organization),
            auto_detect: true,
            url_override: None,
        }
    }

    /// Detect and convert repository URL from git remote
    ///
    /// This method intelligently converts SSH URLs to HTTPS URLs and handles
    /// various Git hosting providers based on the configured patterns.
    ///
    /// # Arguments
    ///
    /// * `remote_url` - The git remote URL (SSH or HTTPS)
    ///
    /// # Returns
    ///
    /// The converted HTTPS URL suitable for web links, or None if conversion fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::config::types::git::RepositoryHostConfig;
    ///
    /// let config = RepositoryHostConfig::github();
    /// let url = config.detect_repository_url("git@github.com:owner/repo.git");
    /// assert_eq!(url, Some("https://github.com/owner/repo".to_string()));
    /// ```
    pub fn detect_repository_url(&self, remote_url: &str) -> Option<String> {
        // Check for URL override first
        if let Some(override_url) = &self.url_override {
            return Some(override_url.clone());
        }

        let mut converted_url = remote_url.to_string();

        // Apply SSH to HTTPS conversions
        for conversion in &self.url_patterns.ssh_conversions {
            if converted_url.starts_with(&conversion.ssh_pattern) {
                converted_url =
                    converted_url.replace(&conversion.ssh_pattern, &conversion.https_replacement);
                break;
            }
        }

        // Remove .git suffix if present
        if converted_url.to_lowercase().ends_with(".git") {
            converted_url = converted_url.strip_suffix(".git")?.to_string();
        }

        // Validate that we have a valid HTTP(S) URL
        if converted_url.starts_with("http://") || converted_url.starts_with("https://") {
            Some(converted_url)
        } else {
            None
        }
    }

    /// Generate commit URL for the given commit hash
    ///
    /// # Arguments
    ///
    /// * `repository_url` - The base repository URL
    /// * `commit_hash` - The commit hash
    ///
    /// # Returns
    ///
    /// The URL to view the commit in the web interface
    pub fn generate_commit_url(&self, repository_url: &str, commit_hash: &str) -> Option<String> {
        let (owner, repo) = Self::parse_repository_parts(repository_url)?;

        let url = self
            .url_patterns
            .commit_url
            .replace("{base_url}", &self.base_url)
            .replace("{owner}", &owner)
            .replace("{repo}", &repo)
            .replace("{hash}", commit_hash);

        Some(url)
    }

    /// Generate comparison URL between two references
    ///
    /// # Arguments
    ///
    /// * `repository_url` - The base repository URL
    /// * `from_ref` - Source reference
    /// * `to_ref` - Target reference
    ///
    /// # Returns
    ///
    /// The URL to view the comparison in the web interface
    pub fn generate_compare_url(
        &self,
        repository_url: &str,
        from_ref: &str,
        to_ref: &str,
    ) -> Option<String> {
        let (owner, repo) = Self::parse_repository_parts(repository_url)?;

        let url = self
            .url_patterns
            .compare_url
            .replace("{base_url}", &self.base_url)
            .replace("{owner}", &owner)
            .replace("{repo}", &repo)
            .replace("{from}", from_ref)
            .replace("{to}", to_ref);

        Some(url)
    }

    /// Parse repository URL to extract owner and repository name
    ///
    /// # Arguments
    ///
    /// * `repository_url` - The repository URL to parse
    ///
    /// # Returns
    ///
    /// A tuple of (owner, repo) or None if parsing fails
    fn parse_repository_parts(repository_url: &str) -> Option<(String, String)> {
        // Handle different URL formats
        let url = if repository_url.starts_with("http://") || repository_url.starts_with("https://")
        {
            repository_url
                .strip_prefix("http://")
                .or_else(|| repository_url.strip_prefix("https://"))?
        } else {
            repository_url
        };

        // Split by '/' and get the last two parts (owner/repo)
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 2 {
            let owner = parts[parts.len() - 2].to_string();
            let repo = parts[parts.len() - 1].to_string();
            Some((owner, repo))
        } else {
            None
        }
    }
}

impl UrlPatterns {
    /// Create URL patterns for GitHub
    #[must_use]
    pub fn github() -> Self {
        Self {
            commit_url: "https://{base_url}/{owner}/{repo}/commit/{hash}".to_string(),
            compare_url: "https://{base_url}/{owner}/{repo}/compare/{from}...{to}".to_string(),
            ssh_conversions: vec![SshConversion {
                ssh_pattern: "git@github.com:".to_string(),
                https_replacement: "https://github.com/".to_string(),
            }],
        }
    }

    /// Create URL patterns for GitHub Enterprise
    #[must_use]
    pub fn github_enterprise(enterprise_url: &str) -> Self {
        Self {
            commit_url: format!("https://{enterprise_url}/{{owner}}/{{repo}}/commit/{{hash}}"),
            compare_url: format!(
                "https://{enterprise_url}/{{owner}}/{{repo}}/compare/{{from}}...{{to}}"
            ),
            ssh_conversions: vec![SshConversion {
                ssh_pattern: format!("git@{enterprise_url}:"),
                https_replacement: format!("https://{enterprise_url}/"),
            }],
        }
    }

    /// Create URL patterns for GitLab
    #[must_use]
    pub fn gitlab() -> Self {
        Self {
            commit_url: "https://{base_url}/{owner}/{repo}/-/commit/{hash}".to_string(),
            compare_url: "https://{base_url}/{owner}/{repo}/-/compare/{from}...{to}".to_string(),
            ssh_conversions: vec![SshConversion {
                ssh_pattern: "git@gitlab.com:".to_string(),
                https_replacement: "https://gitlab.com/".to_string(),
            }],
        }
    }

    /// Create URL patterns for custom GitLab instance
    #[must_use]
    pub fn gitlab_custom(gitlab_url: &str) -> Self {
        Self {
            commit_url: format!("https://{gitlab_url}/{{owner}}/{{repo}}/-/commit/{{hash}}"),
            compare_url: format!(
                "https://{gitlab_url}/{{owner}}/{{repo}}/-/compare/{{from}}...{{to}}"
            ),
            ssh_conversions: vec![SshConversion {
                ssh_pattern: format!("git@{gitlab_url}:"),
                https_replacement: format!("https://{gitlab_url}/"),
            }],
        }
    }

    /// Create URL patterns for Bitbucket
    #[must_use]
    pub fn bitbucket() -> Self {
        Self {
            commit_url: "https://{base_url}/{owner}/{repo}/commits/{hash}".to_string(),
            compare_url: "https://{base_url}/{owner}/{repo}/branches/compare/{to}..{from}"
                .to_string(),
            ssh_conversions: vec![SshConversion {
                ssh_pattern: "git@bitbucket.org:".to_string(),
                https_replacement: "https://bitbucket.org/".to_string(),
            }],
        }
    }

    /// Create URL patterns for Azure DevOps
    #[must_use]
    pub fn azure_devops(organization: &str) -> Self {
        Self {
            commit_url: format!("https://dev.azure.com/{organization}/{{owner}}/_git/{{repo}}/commit/{{hash}}"),
            compare_url: format!("https://dev.azure.com/{organization}/{{owner}}/_git/{{repo}}/branchCompare?baseVersion=GB{{from}}&targetVersion=GB{{to}}"),
            ssh_conversions: vec![
                SshConversion {
                    ssh_pattern: format!("git@ssh.dev.azure.com:v3/{organization}/"),
                    https_replacement: format!("https://dev.azure.com/{organization}/"),
                },
            ],
        }
    }
}
