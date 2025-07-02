//! Changelog template generator
//!
//! Generates formatted changelog content using templates and conventional commits.

use super::types::{ConventionalCommit, GroupedCommits, TemplateVariables};
use crate::config::types::changelog::{ChangelogConfig, ChangelogFormat, CommitGrouping};
use crate::error::{Error, Result};
use std::collections::HashMap;

/// Changelog template generator
///
/// Generates formatted changelog content from grouped commits using configurable templates.
/// Supports multiple output formats (Markdown, Plain Text, JSON) and grouping strategies.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::changelog::{ChangelogGenerator, GroupedCommits, TemplateVariables};
/// use sublime_monorepo_tools::config::types::changelog::ChangelogConfig;
///
/// let generator = ChangelogGenerator::new();
/// let config = ChangelogConfig::default();
/// let grouped_commits = GroupedCommits::new();
/// let variables = TemplateVariables::new("my-package".to_string(), "1.0.0".to_string());
///
/// let changelog = generator.generate_changelog(&config, &grouped_commits, &variables)?;
/// ```
pub struct ChangelogGenerator;

impl ChangelogGenerator {
    /// Create a new changelog generator
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Generate changelog content
    ///
    /// # Arguments
    ///
    /// * `config` - Changelog configuration
    /// * `grouped_commits` - Grouped commits to include
    /// * `variables` - Template variables for substitution
    ///
    /// # Returns
    ///
    /// Generated changelog content as string
    pub fn generate_changelog(
        &self,
        config: &ChangelogConfig,
        grouped_commits: &GroupedCommits,
        variables: &TemplateVariables,
    ) -> Result<String> {
        match config.output_format {
            ChangelogFormat::Markdown => self.generate_markdown(config, grouped_commits, variables),
            ChangelogFormat::Text => self.generate_text(config, grouped_commits, variables),
            ChangelogFormat::Json => self.generate_json(config, grouped_commits, variables),
        }
    }

    /// Generate Markdown format changelog
    fn generate_markdown(
        &self,
        config: &ChangelogConfig,
        grouped_commits: &GroupedCommits,
        variables: &TemplateVariables,
    ) -> Result<String> {
        let mut content = String::new();

        // Add header
        content.push_str(&self.substitute_variables(&config.template.header_template, variables));

        // Add version section header
        content.push_str(&self.substitute_variables(&config.template.section_template, variables));

        // Add breaking changes section if any exist and enabled
        if config.include_breaking_changes && !grouped_commits.breaking_changes.is_empty() {
            content.push_str("### BREAKING CHANGES\n\n");
            for commit in &grouped_commits.breaking_changes {
                content.push_str(&self.format_commit_markdown(commit, variables));
            }
            content.push('\n');
        }

        // Generate content based on grouping strategy
        match config.grouping {
            CommitGrouping::Type => {
                self.generate_by_type_markdown(&mut content, config, grouped_commits, variables)?;
            }
            CommitGrouping::Scope => {
                self.generate_by_scope_markdown(&mut content, config, grouped_commits, variables)?;
            }
            CommitGrouping::None => {
                self.generate_ungrouped_markdown(&mut content, config, grouped_commits, variables)?;
            }
        }

        // Add footer
        content.push_str(&self.substitute_variables(&config.template.footer_template, variables));

        Ok(content)
    }

    /// Generate content grouped by commit type
    #[allow(clippy::unnecessary_wraps)]
    fn generate_by_type_markdown(
        &self,
        content: &mut String,
        config: &ChangelogConfig,
        grouped_commits: &GroupedCommits,
        variables: &TemplateVariables,
    ) -> Result<()> {
        // Sort types by importance
        let type_order = [
            "feat", "fix", "perf", "refactor", "revert", "docs", "style", "test", "build", "ci",
            "chore",
        ];

        for &commit_type in &type_order {
            if let Some(commits) = grouped_commits.by_type.get(commit_type) {
                if commits.is_empty() {
                    continue;
                }

                // Skip breaking changes in type groups if they're already shown
                let filtered_commits: Vec<_> = if config.include_breaking_changes {
                    commits.iter().filter(|c| !c.breaking_change).collect()
                } else {
                    commits.iter().collect()
                };

                if filtered_commits.is_empty() {
                    continue;
                }

                // Get display name from config or default
                let display_name = config
                    .conventional_commit_types
                    .get(commit_type)
                    .cloned()
                    .unwrap_or_else(|| commit_type.to_string());

                content.push_str(&format!("### {display_name}\n\n"));

                for commit in filtered_commits {
                    content.push_str(&self.format_commit_markdown(commit, variables));
                }
                content.push('\n');
            }
        }

        // Handle any custom types not in the standard order
        for (commit_type, commits) in &grouped_commits.by_type {
            if !type_order.contains(&commit_type.as_str()) && !commits.is_empty() {
                let display_name = config
                    .conventional_commit_types
                    .get(commit_type)
                    .cloned()
                    .unwrap_or_else(|| commit_type.to_string());

                content.push_str(&format!("### {display_name}\n\n"));

                for commit in commits {
                    if !config.include_breaking_changes || !commit.breaking_change {
                        content.push_str(&self.format_commit_markdown(commit, variables));
                    }
                }
                content.push('\n');
            }
        }

        Ok(())
    }

    /// Generate content grouped by scope
    #[allow(clippy::unnecessary_wraps)]
    fn generate_by_scope_markdown(
        &self,
        content: &mut String,
        _config: &ChangelogConfig,
        grouped_commits: &GroupedCommits,
        variables: &TemplateVariables,
    ) -> Result<()> {
        // Sort scopes alphabetically
        let mut scopes: Vec<_> = grouped_commits.by_scope.keys().collect();
        scopes.sort();

        for scope in scopes {
            if let Some(commits) = grouped_commits.by_scope.get(scope) {
                if commits.is_empty() {
                    continue;
                }

                content.push_str(&format!("### {scope}\n\n"));

                for commit in commits {
                    content.push_str(&self.format_commit_markdown(commit, variables));
                }
                content.push('\n');
            }
        }

        // Handle commits without scope
        let no_scope_commits: Vec<_> =
            grouped_commits.all_commits.iter().filter(|c| c.scope.is_none()).collect();

        if !no_scope_commits.is_empty() {
            content.push_str("### Other Changes\n\n");
            for commit in no_scope_commits {
                content.push_str(&self.format_commit_markdown(commit, variables));
            }
            content.push('\n');
        }

        Ok(())
    }

    /// Generate ungrouped content
    #[allow(clippy::unnecessary_wraps)]
    fn generate_ungrouped_markdown(
        &self,
        content: &mut String,
        _config: &ChangelogConfig,
        grouped_commits: &GroupedCommits,
        variables: &TemplateVariables,
    ) -> Result<()> {
        content.push_str("### Changes\n\n");

        for commit in &grouped_commits.all_commits {
            content.push_str(&self.format_commit_markdown(commit, variables));
        }

        Ok(())
    }

    /// Format a single commit for Markdown output
    #[allow(clippy::unused_self)]
    fn format_commit_markdown(
        &self,
        commit: &ConventionalCommit,
        variables: &TemplateVariables,
    ) -> String {
        let scope_str = commit.scope.as_ref().map(|s| format!("**{s}**: ")).unwrap_or_default();

        let breaking_indicator = if commit.breaking_change { "⚠️ " } else { "" };

        let commit_link = if let Some(repo_url) = &variables.repository_url {
            format!("[{hash_short}]({repo_url}/commit/{commit_hash})", hash_short = &commit.hash[..8], commit_hash = commit.hash)
        } else {
            commit.hash[..8].to_string()
        };

        format!("- {breaking_indicator}{scope_str}{description} ({commit_link})\n", description = commit.description)
    }

    /// Generate plain text format changelog
    #[allow(clippy::unnecessary_wraps)]
    fn generate_text(
        &self,
        config: &ChangelogConfig,
        grouped_commits: &GroupedCommits,
        variables: &TemplateVariables,
    ) -> Result<String> {
        let mut content = String::new();

        // Add header (strip markdown formatting)
        let header = self.substitute_variables(&config.template.header_template, variables);
        content.push_str(&self.strip_markdown(&header));

        // Add version section
        let section = self.substitute_variables(&config.template.section_template, variables);
        content.push_str(&self.strip_markdown(&section));

        // Add commits (similar structure to markdown but without formatting)
        for commit in &grouped_commits.all_commits {
            let scope_str = commit.scope.as_ref().map(|s| format!("{s}: ")).unwrap_or_default();

            let breaking_indicator = if commit.breaking_change { "[BREAKING] " } else { "" };

            content.push_str(&format!(
                "- {breaking_indicator}{scope_str}{} ({})\n",
                commit.description,
                &commit.hash[..8]
            ));
        }

        // Add footer
        let footer = self.substitute_variables(&config.template.footer_template, variables);
        content.push_str(&self.strip_markdown(&footer));

        Ok(content)
    }

    /// Generate JSON format changelog
    #[allow(clippy::unused_self)]
    fn generate_json(
        &self,
        _config: &ChangelogConfig,
        grouped_commits: &GroupedCommits,
        variables: &TemplateVariables,
    ) -> Result<String> {
        let mut json_data = HashMap::new();

        json_data.insert("package", &variables.package_name);
        json_data.insert("version", &variables.version);
        json_data.insert("date", &variables.date);

        if let Some(url) = &variables.repository_url {
            json_data.insert("repository_url", url);
        }

        if let Some(url) = &variables.compare_url {
            json_data.insert("compare_url", url);
        }

        // Create structured data
        let changelog_data = serde_json::json!({
            "metadata": json_data,
            "commits": {
                "total": grouped_commits.total_commits(),
                "breaking_changes": grouped_commits.breaking_changes.len(),
                "by_type": grouped_commits.by_type,
                "by_scope": grouped_commits.by_scope,
                "all": grouped_commits.all_commits
            }
        });

        serde_json::to_string_pretty(&changelog_data)
            .map_err(|e| Error::changelog(format!("Failed to serialize to JSON: {e}")))
    }

    /// Substitute template variables in a string
    #[allow(clippy::unused_self)]
    fn substitute_variables(&self, template: &str, variables: &TemplateVariables) -> String {
        template
            .replace("{package_name}", &variables.package_name)
            .replace("{version}", &variables.version)
            .replace("{date}", &variables.date)
            .replace("{repository_url}", variables.repository_url.as_deref().unwrap_or(""))
            .replace("{compare_url}", variables.compare_url.as_deref().unwrap_or(""))
            .replace("{previous_version}", variables.previous_version.as_deref().unwrap_or(""))
    }

    /// Strip basic markdown formatting for plain text output
    #[allow(clippy::unused_self)]
    fn strip_markdown(&self, text: &str) -> String {
        text.replace("# ", "")
            .replace("## ", "")
            .replace("### ", "")
            .replace("**", "")
            .replace(['*', '`'], "")
    }
}

impl Default for ChangelogGenerator {
    fn default() -> Self {
        Self::new()
    }
}
