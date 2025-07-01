//! Changelog manager implementation
//!
//! Main interface for changelog generation, coordinating commit parsing,
//! grouping, template processing, and file output.

use super::generator::ChangelogGenerator;
use super::parser::ConventionalCommitParser;
use super::types::{ChangelogRequest, ChangelogResult, GroupedCommits, TemplateVariables};
use crate::config::MonorepoConfig;
use crate::core::{MonorepoPackageInfo, MonorepoProject};
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::path::Path;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};

/// Changelog manager
///
/// Main interface for generating changelogs from Git commit history.
/// Coordinates commit parsing, grouping, template processing, and file output.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::changelog::ChangelogManager;
/// use sublime_monorepo_tools::core::MonorepoProject;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let project = Arc::new(MonorepoProject::new(".")?);
/// let manager = ChangelogManager::from_project(project)?;
///
/// // Generate changelog for specific package
/// let request = ChangelogRequest {
///     package_name: Some("my-package".to_string()),
///     version: "1.0.0".to_string(),
///     since: Some("v0.9.0".to_string()),
///     ..Default::default()
/// };
///
/// let result = manager.generate_changelog(request).await?;
/// println!("Generated changelog with {} commits", result.commit_count);
/// # Ok(())
/// # }
/// ```
pub struct ChangelogManager<'a> {
    /// Direct reference to git repository
    repository: &'a Repo,
    /// Direct reference to packages
    packages: &'a [MonorepoPackageInfo],
    /// Direct reference to file system manager
    file_system: &'a FileSystemManager,
    /// Direct reference to configuration
    config: &'a MonorepoConfig,
    /// Direct reference to root path
    root_path: &'a Path,
    /// Conventional commit parser
    parser: ConventionalCommitParser,
    /// Changelog generator
    generator: ChangelogGenerator,
}

impl<'a> ChangelogManager<'a> {
    /// Create a new changelog manager with direct borrowing from project
    ///
    /// Uses borrowing instead of trait objects to eliminate Arc proliferation
    /// and work with Rust ownership principles.
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to monorepo project
    ///
    /// # Returns
    ///
    /// A new changelog manager instance
    pub fn new(project: &'a MonorepoProject) -> Self {
        Self {
            repository: project.repository(),
            packages: &project.packages,
            file_system: project.services.file_system_service().manager(),
            config: project.services.config_service().get_configuration(),
            root_path: project.root_path(),
            parser: ConventionalCommitParser::new(),
            generator: ChangelogGenerator::new(),
        }
    }

    /// Creates a new changelog manager from an existing MonorepoProject
    /// 
    /// Convenience method that wraps the `new` constructor for backward compatibility.
    /// Uses real direct borrowing following Rust ownership principles.
    /// 
    /// # Arguments
    /// 
    /// * `project` - Reference to the monorepo project
    /// 
    /// # Returns
    /// 
    /// A new ChangelogManager instance with direct borrowing
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use sublime_monorepo_tools::{ChangelogManager, MonorepoProject};
    /// 
    /// let project = MonorepoProject::new("/path/to/monorepo")?;
    /// let changelog_manager = ChangelogManager::from_project(&project);
    /// ```
    #[must_use]
    pub fn from_project(project: &'a MonorepoProject) -> Self {
        Self::new(project)
    }

    /// Generate changelog based on request
    ///
    /// # Arguments
    ///
    /// * `request` - Changelog generation request
    ///
    /// # Returns
    ///
    /// Generated changelog result
    /// FASE 2 ASYNC ELIMINATION: Synchronous changelog generation  
    /// 
    /// Removed artificial async behavior - this is a purely synchronous operation.
    pub fn generate_changelog_sync(&self, request: ChangelogRequest) -> Result<ChangelogResult> {
        log::info!(
            "Generating changelog for package: {:?}, version: {}",
            request.package_name,
            request.version
        );

        // Get commits since specified reference
        let commits = self.get_commits_since_reference(&request.since, &request.until)?;
        log::debug!("Found {} total commits", commits.len());

        // Parse commits to conventional format
        let conventional_commits = self.parser.parse_commits(&commits)?;
        log::debug!("Parsed {} conventional commits", conventional_commits.len());

        // Filter commits for specific package if requested
        let filtered_commits = if let Some(package_name) = &request.package_name {
            let package_path = self.get_package_path(package_name)?;
            let changed_files = self.get_changed_files_by_commit(&commits)?;

            self.parser.filter_commits_for_package(
                &conventional_commits,
                &package_path,
                Some(&changed_files),
            )
        } else {
            conventional_commits
        };

        log::debug!("Filtered to {} commits for target", filtered_commits.len());

        // Filter commits by importance if not including all
        let final_commits: Vec<_> = if request.include_all_commits {
            filtered_commits
        } else {
            filtered_commits
                .into_iter()
                .filter(|commit| self.parser.should_include_commit(&commit.commit_type, false))
                .collect()
        };

        log::debug!("Final commit count after filtering: {}", final_commits.len());

        // Group commits for generation
        let grouped_commits = self.group_commits(final_commits);

        // Create template variables
        let package_name = request.package_name.clone().unwrap_or_else(|| {
            self.root_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("monorepo")
                .to_string()
        });

        let variables = self.create_template_variables(&package_name, &request.version)?;

        // Get changelog configuration
        let config = &self.config.changelog;

        // Generate changelog content
        let content = self.generator.generate_changelog(config, &grouped_commits, &variables)?;

        // Write to file if requested
        let output_path = if request.write_to_file {
            let path = self.write_changelog_to_file(&content, &request)?;
            Some(path)
        } else {
            None
        };

        Ok(ChangelogResult {
            content,
            package_name,
            version: request.version,
            commit_count: grouped_commits.total_commits(),
            has_breaking_changes: grouped_commits.has_breaking_changes(),
            output_path,
        })
    }

    /// Generate changelog
    pub fn generate_changelog(&self, request: ChangelogRequest) -> Result<ChangelogResult> {
        self.generate_changelog_sync(request)
    }

    /// Parse conventional commits from commit range
    ///
    /// # Arguments
    ///
    /// * `package_path` - Optional path to filter commits (None for all)
    /// * `since` - Start reference (tag, commit, etc.)
    ///
    /// # Returns
    ///
    /// Vector of parsed conventional commits
    /// FASE 2 ASYNC ELIMINATION: Synchronous conventional commit parsing
    ///
    /// Removed artificial async behavior - this is a purely synchronous operation.
    pub fn parse_conventional_commits_sync(
        &self,
        package_path: Option<&str>,
        since: &str,
    ) -> Result<Vec<super::types::ConventionalCommit>> {
        let commits = self
            .repository
            .get_commits_since(Some(since.to_string()), &None)
            .map_err(|e| Error::changelog(format!("Failed to get commits since {since}: {e}")))?;

        let conventional_commits = self.parser.parse_commits(&commits)?;

        if let Some(path) = package_path {
            let changed_files = self.get_changed_files_by_commit(&commits)?;
            Ok(self.parser.filter_commits_for_package(
                &conventional_commits,
                path,
                Some(&changed_files),
            ))
        } else {
            Ok(conventional_commits)
        }
    }

    /// Update existing changelog with new version
    ///
    /// # Arguments
    ///
    /// * `package_name` - Package to update (None for root)
    /// * `new_content` - New changelog content to prepend
    ///
    /// # Returns
    ///
    /// Updated changelog content
    /// FASE 2 ASYNC ELIMINATION: Synchronous changelog updating
    ///
    /// Removed artificial async behavior - this is a purely synchronous operation.
    pub fn update_existing_changelog_sync(
        &self,
        package_name: Option<&str>,
        new_content: &str,
    ) -> Result<String> {
        let changelog_path = self.get_changelog_path(package_name)?;

        let existing_content = if changelog_path.exists() {
            self.file_system.read_file_string(&changelog_path).unwrap_or_default()
        } else {
            String::new()
        };

        // Find insertion point (after header but before first version)
        let lines: Vec<&str> = existing_content.lines().collect();
        let mut insert_index = 0;

        // Skip header lines (lines starting with # until we find the first version)
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("## [") || line.starts_with("## ") {
                insert_index = i;
                break;
            }
            if i > 10 {
                break;
            }
        }

        let mut updated_lines = Vec::new();

        // Add existing header
        updated_lines.extend_from_slice(&lines[..insert_index]);

        // Add new content
        for line in new_content.lines() {
            updated_lines.push(line);
        }

        // Add existing content (skip duplicate header if present)
        if insert_index < lines.len() {
            updated_lines.extend_from_slice(&lines[insert_index..]);
        }

        Ok(updated_lines.join("\n"))
    }

    /// Update existing changelog with new version
    pub fn update_existing_changelog(
        &self,
        package_name: Option<&str>,
        new_content: &str,
    ) -> Result<String> {
        self.update_existing_changelog_sync(package_name, new_content)
    }

    /// Get commits since a reference
    fn get_commits_since_reference(
        &self,
        since: &Option<String>,
        until: &Option<String>,
    ) -> Result<Vec<sublime_git_tools::RepoCommit>> {
        let repository = self.repository;

        match (since, until) {
            (Some(since_ref), Some(_until_ref)) => {
                // Get commits between two references
                // For now, we'll use get_commits_since and filter
                // In a more advanced implementation, we could add get_commits_between to git crate
                let all_commits =
                    repository.get_commits_since(Some(since_ref.clone()), &None).map_err(|e| {
                        Error::changelog(format!("Failed to get commits since {since_ref}: {e}"))
                    })?;

                // Filter commits until the until_ref
                // This is a simplified implementation - in practice you'd want to resolve refs to SHAs
                Ok(all_commits)
            }
            (Some(since_ref), None) => {
                repository.get_commits_since(Some(since_ref.clone()), &None).map_err(|e| {
                    Error::changelog(format!("Failed to get commits since {since_ref}: {e}"))
                })
            }
            (None, Some(_until_ref)) => {
                // Get recent commits (last 100 as fallback)
                repository
                    .get_commits_since(Some("100 commits ago".to_string()), &None)
                    .map_err(|e| Error::changelog(format!("Failed to get recent commits: {e}")))
            }
            (None, None) => {
                // Get commits since last tag or recent history
                if let Ok(last_tag) = self.get_last_tag() {
                    repository.get_commits_since(Some(last_tag), &None).map_err(|e| {
                        Error::changelog(format!("Failed to get commits since last tag: {e}"))
                    })
                } else {
                    // Fallback to recent commits
                    repository
                        .get_commits_since(Some("1 month ago".to_string()), &None)
                        .map_err(|e| Error::changelog(format!("Failed to get recent commits: {e}")))
                }
            }
        }
    }

    /// Get the last Git tag
    fn get_last_tag(&self) -> Result<String> {
        self.repository
            .get_last_tag()
            .map_err(|e| Error::changelog(format!("Failed to get last tag: {e}")))
    }

    /// Get package path relative to repository root
    fn get_package_path(&self, package_name: &str) -> Result<String> {
        let packages = self.packages;
        let package = packages
            .iter()
            .find(|p| p.name() == package_name)
            .ok_or_else(|| Error::changelog(format!("Package '{package_name}' not found")))?;

        let repo_root = self.root_path;
        let relative_path = package
            .workspace_package
            .location
            .strip_prefix(repo_root)
            .map_err(|e| Error::changelog(format!("Failed to get relative path: {e}")))?;

        Ok(relative_path.to_string_lossy().to_string())
    }

    /// Get changed files for each commit
    #[allow(clippy::unnecessary_wraps)]
    fn get_changed_files_by_commit(
        &self,
        commits: &[sublime_git_tools::RepoCommit],
    ) -> Result<HashMap<String, Vec<String>>> {
        let mut changed_files = HashMap::new();

        for commit in commits {
            match self.repository.get_all_files_changed_since_sha(&commit.hash) {
                Ok(files) => {
                    changed_files.insert(commit.hash.clone(), files);
                }
                Err(e) => {
                    log::warn!("Failed to get changed files for commit {}: {}", commit.hash, e);
                    // Continue with other commits
                }
            }
        }

        Ok(changed_files)
    }

    /// Group commits according to configuration
    #[allow(clippy::unused_self)]
    fn group_commits(&self, commits: Vec<super::types::ConventionalCommit>) -> GroupedCommits {
        let mut grouped = GroupedCommits::new();

        for commit in commits {
            grouped.add_commit(commit);
        }

        grouped
    }

    /// Create template variables for changelog generation
    ///
    /// This method creates template variables using configurable repository hosting settings
    /// that support multiple providers including GitHub, GitLab, Bitbucket, and enterprise instances.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The name of the package
    /// * `version` - The version for the changelog
    ///
    /// # Returns
    ///
    /// Template variables with repository URL and other metadata, or an error if creation fails
    ///
    /// # Examples
    ///
    /// The method automatically detects repository URLs from git remote and converts them
    /// based on the configured repository hosting provider:
    ///
    /// - GitHub: `git@github.com:owner/repo.git` → `https://github.com/owner/repo`
    /// - GitLab: `git@gitlab.com:owner/repo.git` → `https://gitlab.com/owner/repo`
    /// - Enterprise: `git@github.company.com:owner/repo.git` → `https://github.company.com/owner/repo`
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    fn create_template_variables(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<TemplateVariables> {
        let mut variables = TemplateVariables::new(package_name.to_string(), version.to_string());

        // Get repository configuration from project config
        let repo_config = &self.config.git.repository;

        // Try to detect repository URL from git remote
        if let Ok(repository) = self.repository.list_config() {
            let remote_name = &self.config.git.default_remote;
            let remote_key = format!("remote.{remote_name}.url");

            if let Some(remote_url) = repository.get(&remote_key) {
                log::debug!("Found git remote URL: {}", remote_url);

                // Use repository configuration to convert URL intelligently
                if let Some(web_url) = repo_config.detect_repository_url(remote_url) {
                    log::debug!("Converted repository URL: {}", web_url);
                    variables = variables.with_repository_url(web_url);
                } else {
                    log::warn!(
                        "Failed to convert repository URL '{}' using provider: {:?}",
                        remote_url,
                        repo_config.provider
                    );

                    // Fallback: attempt basic cleanup without provider-specific logic
                    let fallback_url = if remote_url.ends_with(".git") {
                        remote_url
                            .strip_suffix(".git")
                            .map_or_else(|| remote_url.clone(), std::string::ToString::to_string)
                    } else {
                        remote_url.clone()
                    };

                    if fallback_url.starts_with("http://") || fallback_url.starts_with("https://") {
                        variables = variables.with_repository_url(fallback_url);
                    }
                }
            } else {
                log::info!(
                    "No git remote '{}' found, repository URL will not be available for changelog links",
                    remote_name
                );
            }
        } else {
            log::warn!("Failed to read git configuration, repository URL will not be available");
        }

        Ok(variables)
    }

    /// Write changelog content to file
    fn write_changelog_to_file(&self, content: &str, request: &ChangelogRequest) -> Result<String> {
        let output_path = if let Some(custom_path) = &request.output_path {
            custom_path.clone()
        } else {
            self.get_default_changelog_path(&request.package_name)?
        };

        self.file_system.write_file_string(&Path::new(&output_path), content).map_err(
            |e| Error::changelog(format!("Failed to write changelog to {output_path}: {e}")),
        )?;

        log::info!("Wrote changelog to: {}", output_path);
        Ok(output_path)
    }

    /// Get default changelog file path
    fn get_default_changelog_path(&self, package_name: &Option<String>) -> Result<String> {
        if let Some(name) = package_name {
            let package_path = self.get_package_path(name)?;
            let full_path = self.root_path.join(package_path);
            Ok(full_path.join("CHANGELOG.md").to_string_lossy().to_string())
        } else {
            // Root changelog
            Ok(self.root_path.join("CHANGELOG.md").to_string_lossy().to_string())
        }
    }

    /// Get changelog file path for reading
    fn get_changelog_path(&self, package_name: Option<&str>) -> Result<std::path::PathBuf> {
        if let Some(name) = package_name {
            let package_path = self.get_package_path(name)?;
            let full_path = self.root_path.join(package_path);
            Ok(full_path.join("CHANGELOG.md"))
        } else {
            Ok(self.root_path.join("CHANGELOG.md"))
        }
    }
}
