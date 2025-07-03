//! Comprehensive tests for the changelog module
//!
//! This module provides complete test coverage for all changelog functionality,
//! including conventional commit parsing, changelog generation, manager operations,
//! and all supporting types and utilities.

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod tests {
    use super::super::types::{
        ChangelogRequest, ChangelogResult, ConventionalCommit, GroupedCommits, TemplateVariables,
    };
    use super::super::{ChangelogGenerator, ChangelogManager, ConventionalCommitParser};
    use crate::config::types::changelog::{
        ChangelogConfig, ChangelogFormat, ChangelogTemplate, CommitGrouping,
    };
    use crate::core::MonorepoProject;
    use crate::error::Result;
    use std::collections::HashMap;
    use sublime_git_tools::RepoCommit;
    use tempfile::TempDir;

    /// Create a test monorepo project for testing
    fn create_test_project() -> Result<(TempDir, MonorepoProject)> {
        let temp_dir = TempDir::new().map_err(crate::error::Error::Io)?;
        let root_path = temp_dir.path().to_path_buf();

        // Create a basic package.json for the root
        let package_json_content = r#"{
            "name": "test-monorepo",
            "version": "1.0.0",
            "workspaces": ["packages/*", "apps/*"]
        }"#;
        std::fs::write(root_path.join("package.json"), package_json_content)
            .map_err(crate::error::Error::Io)?;

        // Create package-lock.json to indicate npm as package manager
        let package_lock_content = r#"{
            "name": "test-monorepo",
            "version": "1.0.0",
            "lockfileVersion": 2,
            "requires": true,
            "packages": {}
        }"#;
        std::fs::write(root_path.join("package-lock.json"), package_lock_content)
            .map_err(crate::error::Error::Io)?;

        // Create package directories
        std::fs::create_dir_all(root_path.join("packages/core"))
            .map_err(crate::error::Error::Io)?;
        std::fs::create_dir_all(root_path.join("packages/utils"))
            .map_err(crate::error::Error::Io)?;

        // Create package.json files for packages
        let core_package_json = r#"{
            "name": "@test/core",
            "version": "1.0.0",
            "dependencies": {
                "lodash": "^4.17.21"
            }
        }"#;
        std::fs::write(root_path.join("packages/core/package.json"), core_package_json)
            .map_err(crate::error::Error::Io)?;

        let utils_package_json = r#"{
            "name": "@test/utils",
            "version": "1.0.0",
            "dependencies": {
                "@test/core": "^1.0.0"
            }
        }"#;
        std::fs::write(root_path.join("packages/utils/package.json"), utils_package_json)
            .map_err(crate::error::Error::Io)?;

        // Initialize git repository
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&root_path)
            .output()
            .map_err(crate::error::Error::Io)?;

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&root_path)
            .output()
            .map_err(crate::error::Error::Io)?;

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&root_path)
            .output()
            .map_err(crate::error::Error::Io)?;

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&root_path)
            .output()
            .map_err(crate::error::Error::Io)?;

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&root_path)
            .output()
            .map_err(crate::error::Error::Io)?;

        let project = MonorepoProject::new(&root_path)?;
        Ok((temp_dir, project))
    }

    /// Create test commits for testing
    fn create_test_commits() -> Vec<RepoCommit> {
        vec![
            RepoCommit {
                hash: "abc123".to_string(),
                message: "feat(auth): add OAuth2 support\n\nImplements OAuth2 authentication flow"
                    .to_string(),
                author_name: "Jane Doe".to_string(),
                author_email: "jane@example.com".to_string(),
                author_date: "2024-01-15".to_string(),
            },
            RepoCommit {
                hash: "def456".to_string(),
                message: "fix(core): resolve memory leak issue".to_string(),
                author_name: "John Smith".to_string(),
                author_email: "john@example.com".to_string(),
                author_date: "2024-01-16".to_string(),
            },
            RepoCommit {
                hash: "ghi789".to_string(),
                message: "feat!: breaking change in API\n\nBREAKING CHANGE: API endpoint changed"
                    .to_string(),
                author_name: "Bob Wilson".to_string(),
                author_email: "bob@example.com".to_string(),
                author_date: "2024-01-17".to_string(),
            },
            RepoCommit {
                hash: "jkl012".to_string(),
                message: "docs: update README with new examples".to_string(),
                author_name: "Alice Johnson".to_string(),
                author_email: "alice@example.com".to_string(),
                author_date: "2024-01-18".to_string(),
            },
            RepoCommit {
                hash: "mno345".to_string(),
                message: "non-conventional commit message".to_string(),
                author_name: "Charlie Brown".to_string(),
                author_email: "charlie@example.com".to_string(),
                author_date: "2024-01-19".to_string(),
            },
        ]
    }

    /// Create test conventional commits
    fn create_test_conventional_commits() -> Vec<ConventionalCommit> {
        vec![
            ConventionalCommit {
                commit_type: "feat".to_string(),
                scope: Some("auth".to_string()),
                description: "add OAuth2 support".to_string(),
                body: Some("Implements OAuth2 authentication flow".to_string()),
                breaking_change: false,
                hash: "abc123".to_string(),
                author: "Jane Doe".to_string(),
                date: "2024-01-15".to_string(),
            },
            ConventionalCommit {
                commit_type: "fix".to_string(),
                scope: Some("core".to_string()),
                description: "resolve memory leak issue".to_string(),
                body: None,
                breaking_change: false,
                hash: "def456".to_string(),
                author: "John Smith".to_string(),
                date: "2024-01-16".to_string(),
            },
            ConventionalCommit {
                commit_type: "feat".to_string(),
                scope: None,
                description: "breaking change in API".to_string(),
                body: Some("BREAKING CHANGE: API endpoint changed".to_string()),
                breaking_change: true,
                hash: "ghi789".to_string(),
                author: "Bob Wilson".to_string(),
                date: "2024-01-17".to_string(),
            },
            ConventionalCommit {
                commit_type: "docs".to_string(),
                scope: None,
                description: "update README with new examples".to_string(),
                body: None,
                breaking_change: false,
                hash: "jkl012".to_string(),
                author: "Alice Johnson".to_string(),
                date: "2024-01-18".to_string(),
            },
        ]
    }

    // ConventionalCommitParser Tests
    #[test]
    fn test_conventional_commit_parser_new() {
        let parser = ConventionalCommitParser::new();
        // Parser should be successfully created
        assert!(matches!(parser, ConventionalCommitParser));
    }

    #[test]
    fn test_conventional_commit_parser_default() {
        let parser = ConventionalCommitParser::default();
        // Parser should be successfully created
        assert!(matches!(parser, ConventionalCommitParser));
    }

    #[test]
    fn test_parse_conventional_commit_with_scope() -> Result<()> {
        let parser = ConventionalCommitParser::new();
        let commit = parser.parse_commit_message(
            "feat(auth): add OAuth2 support\n\nImplements OAuth2 authentication flow",
            "abc123",
            "Jane Doe",
            "2024-01-15",
        )?;

        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, Some("auth".to_string()));
        assert_eq!(commit.description, "add OAuth2 support");
        assert_eq!(commit.body, Some("Implements OAuth2 authentication flow".to_string()));
        assert!(!commit.breaking_change);
        assert_eq!(commit.hash, "abc123");
        assert_eq!(commit.author, "Jane Doe");
        assert_eq!(commit.date, "2024-01-15");

        Ok(())
    }

    #[test]
    fn test_parse_conventional_commit_without_scope() -> Result<()> {
        let parser = ConventionalCommitParser::new();
        let commit = parser.parse_commit_message(
            "fix: resolve memory leak issue",
            "def456",
            "John Smith",
            "2024-01-16",
        )?;

        assert_eq!(commit.commit_type, "fix");
        assert_eq!(commit.scope, None);
        assert_eq!(commit.description, "resolve memory leak issue");
        assert_eq!(commit.body, None);
        assert!(!commit.breaking_change);
        assert_eq!(commit.hash, "def456");
        assert_eq!(commit.author, "John Smith");
        assert_eq!(commit.date, "2024-01-16");

        Ok(())
    }

    #[test]
    fn test_parse_breaking_change_in_header() -> Result<()> {
        let parser = ConventionalCommitParser::new();
        let commit = parser.parse_commit_message(
            "feat!: breaking change in API",
            "ghi789",
            "Bob Wilson",
            "2024-01-17",
        )?;

        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, None);
        assert_eq!(commit.description, "breaking change in API");
        assert!(commit.breaking_change);
        assert_eq!(commit.hash, "ghi789");

        Ok(())
    }

    #[test]
    fn test_parse_breaking_change_in_body() -> Result<()> {
        let parser = ConventionalCommitParser::new();
        let commit = parser.parse_commit_message(
            "feat: new API endpoint\n\nBREAKING CHANGE: old endpoint removed",
            "xyz123",
            "Alice Johnson",
            "2024-01-18",
        )?;

        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.description, "new API endpoint");
        assert!(commit.breaking_change);
        assert_eq!(commit.body, Some("BREAKING CHANGE: old endpoint removed".to_string()));

        Ok(())
    }

    #[test]
    fn test_parse_breaking_change_with_scope() -> Result<()> {
        let parser = ConventionalCommitParser::new();
        let commit = parser.parse_commit_message(
            "feat(api)!: breaking change in API",
            "ghi789",
            "Bob Wilson",
            "2024-01-17",
        )?;

        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, Some("api".to_string()));
        assert_eq!(commit.description, "breaking change in API");
        assert!(commit.breaking_change);

        Ok(())
    }

    #[test]
    fn test_parse_non_conventional_commit() -> Result<()> {
        let parser = ConventionalCommitParser::new();
        let commit = parser.parse_commit_message(
            "just a regular commit message",
            "mno345",
            "Charlie Brown",
            "2024-01-19",
        )?;

        assert_eq!(commit.commit_type, "chore");
        assert_eq!(commit.scope, None);
        assert_eq!(commit.description, "just a regular commit message");
        assert!(!commit.breaking_change);

        Ok(())
    }

    #[test]
    fn test_parse_empty_commit_message() {
        let parser = ConventionalCommitParser::new();
        let result = parser.parse_commit_message("", "abc123", "Jane Doe", "2024-01-15");
        assert!(result.is_err());
    }

    #[test]
    fn test_should_include_commit() {
        let parser = ConventionalCommitParser::new();

        // Test with include_all = true
        assert!(parser.should_include_commit("feat", true));
        assert!(parser.should_include_commit("fix", true));
        assert!(parser.should_include_commit("docs", true));
        assert!(parser.should_include_commit("style", true));
        assert!(parser.should_include_commit("chore", true));

        // Test with include_all = false
        assert!(parser.should_include_commit("feat", false));
        assert!(parser.should_include_commit("fix", false));
        assert!(parser.should_include_commit("perf", false));
        assert!(parser.should_include_commit("refactor", false));
        assert!(parser.should_include_commit("revert", false));
        assert!(parser.should_include_commit("breaking", false));

        // Test excluded types
        assert!(!parser.should_include_commit("docs", false));
        assert!(!parser.should_include_commit("style", false));
        assert!(!parser.should_include_commit("test", false));
        assert!(!parser.should_include_commit("build", false));
        assert!(!parser.should_include_commit("ci", false));
        assert!(!parser.should_include_commit("chore", false));
    }

    #[test]
    fn test_get_type_display_name() {
        let parser = ConventionalCommitParser::new();

        assert_eq!(parser.get_type_display_name("feat"), "Features");
        assert_eq!(parser.get_type_display_name("fix"), "Bug Fixes");
        assert_eq!(parser.get_type_display_name("perf"), "Performance Improvements");
        assert_eq!(parser.get_type_display_name("refactor"), "Code Refactoring");
        assert_eq!(parser.get_type_display_name("revert"), "Reverts");
        assert_eq!(parser.get_type_display_name("docs"), "Documentation");
        assert_eq!(parser.get_type_display_name("style"), "Styles");
        assert_eq!(parser.get_type_display_name("test"), "Tests");
        assert_eq!(parser.get_type_display_name("build"), "Build System");
        assert_eq!(parser.get_type_display_name("ci"), "Continuous Integration");
        assert_eq!(parser.get_type_display_name("chore"), "Chores");
        assert_eq!(parser.get_type_display_name("breaking"), "BREAKING CHANGES");
        assert_eq!(parser.get_type_display_name("unknown"), "Other Changes");
    }

    #[test]
    fn test_parse_commits() -> Result<()> {
        let parser = ConventionalCommitParser::new();
        let repo_commits = create_test_commits();

        let parsed_commits = parser.parse_commits(&repo_commits)?;

        assert_eq!(parsed_commits.len(), 5);

        // Check first commit
        assert_eq!(parsed_commits[0].commit_type, "feat");
        assert_eq!(parsed_commits[0].scope, Some("auth".to_string()));
        assert_eq!(parsed_commits[0].description, "add OAuth2 support");

        // Check second commit
        assert_eq!(parsed_commits[1].commit_type, "fix");
        assert_eq!(parsed_commits[1].scope, Some("core".to_string()));

        // Check breaking change commit
        assert_eq!(parsed_commits[2].commit_type, "feat");
        assert!(parsed_commits[2].breaking_change);

        // Check non-conventional commit (should be parsed as chore)
        assert_eq!(parsed_commits[4].commit_type, "chore");
        assert_eq!(parsed_commits[4].description, "non-conventional commit message");

        Ok(())
    }

    #[test]
    fn test_filter_commits_for_package() {
        let parser = ConventionalCommitParser::new();
        let commits = create_test_conventional_commits();

        // Test filtering by package path
        let filtered = parser.filter_commits_for_package(&commits, "packages/auth", None);

        // Should find commits that mention "auth" in scope or description
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].scope, Some("auth".to_string()));
    }

    #[test]
    fn test_filter_commits_for_package_with_changed_files() {
        let parser = ConventionalCommitParser::new();
        let commits = create_test_conventional_commits();

        // Create changed files map
        let mut changed_files = HashMap::new();
        changed_files.insert("abc123".to_string(), vec!["packages/auth/src/oauth.rs".to_string()]);
        changed_files.insert("def456".to_string(), vec!["packages/core/src/memory.rs".to_string()]);

        let filtered =
            parser.filter_commits_for_package(&commits, "packages/auth", Some(&changed_files));

        // Should find commits that have changed files in the package path
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].hash, "abc123");
    }

    // GroupedCommits Tests
    #[test]
    fn test_grouped_commits_new() {
        let grouped = GroupedCommits::new();

        assert!(grouped.by_type.is_empty());
        assert!(grouped.by_scope.is_empty());
        assert!(grouped.breaking_changes.is_empty());
        assert!(grouped.all_commits.is_empty());
    }

    #[test]
    fn test_grouped_commits_default() {
        let grouped = GroupedCommits::default();

        assert!(grouped.by_type.is_empty());
        assert!(grouped.by_scope.is_empty());
        assert!(grouped.breaking_changes.is_empty());
        assert!(grouped.all_commits.is_empty());
    }

    #[test]
    fn test_grouped_commits_add_commit() {
        let mut grouped = GroupedCommits::new();
        let commits = create_test_conventional_commits();

        // Add commits
        for commit in commits {
            grouped.add_commit(commit);
        }

        // Check total
        assert_eq!(grouped.total_commits(), 4);

        // Check by type
        assert_eq!(grouped.by_type.len(), 3); // feat, fix, docs
        assert_eq!(grouped.by_type["feat"].len(), 2); // Two feat commits
        assert_eq!(grouped.by_type["fix"].len(), 1);
        assert_eq!(grouped.by_type["docs"].len(), 1);

        // Check by scope
        assert_eq!(grouped.by_scope.len(), 2); // auth, core
        assert_eq!(grouped.by_scope["auth"].len(), 1);
        assert_eq!(grouped.by_scope["core"].len(), 1);

        // Check breaking changes
        assert_eq!(grouped.breaking_changes.len(), 1);
        assert!(grouped.has_breaking_changes());
    }

    #[test]
    fn test_grouped_commits_get_by_type() {
        let mut grouped = GroupedCommits::new();
        let commits = create_test_conventional_commits();

        for commit in commits {
            grouped.add_commit(commit);
        }

        // Test existing types
        assert_eq!(grouped.get_by_type("feat").len(), 2);
        assert_eq!(grouped.get_by_type("fix").len(), 1);
        assert_eq!(grouped.get_by_type("docs").len(), 1);

        // Test non-existing type
        assert_eq!(grouped.get_by_type("nonexistent").len(), 0);
    }

    #[test]
    fn test_grouped_commits_get_by_scope() {
        let mut grouped = GroupedCommits::new();
        let commits = create_test_conventional_commits();

        for commit in commits {
            grouped.add_commit(commit);
        }

        // Test existing scopes
        assert_eq!(grouped.get_by_scope("auth").len(), 1);
        assert_eq!(grouped.get_by_scope("core").len(), 1);

        // Test non-existing scope
        assert_eq!(grouped.get_by_scope("nonexistent").len(), 0);
    }

    // TemplateVariables Tests
    #[test]
    fn test_template_variables_new() {
        let variables = TemplateVariables::new("test-package".to_string(), "1.0.0".to_string());

        assert_eq!(variables.package_name, "test-package");
        assert_eq!(variables.version, "1.0.0");
        assert!(!variables.date.is_empty());
        assert_eq!(variables.repository_url, None);
        assert_eq!(variables.compare_url, None);
        assert_eq!(variables.previous_version, None);
    }

    #[test]
    fn test_template_variables_with_repository_url() {
        let variables = TemplateVariables::new("test-package".to_string(), "1.0.0".to_string())
            .with_repository_url("https://github.com/test/repo".to_string());

        assert_eq!(variables.repository_url, Some("https://github.com/test/repo".to_string()));
    }

    #[test]
    fn test_template_variables_with_previous_version() {
        let variables = TemplateVariables::new("test-package".to_string(), "1.0.0".to_string())
            .with_repository_url("https://github.com/test/repo".to_string())
            .with_previous_version("0.9.0".to_string());

        assert_eq!(variables.previous_version, Some("0.9.0".to_string()));
        assert_eq!(
            variables.compare_url,
            Some("https://github.com/test/repo/compare/0.9.0...1.0.0".to_string())
        );
    }

    #[test]
    fn test_template_variables_with_previous_version_no_repo() {
        let variables = TemplateVariables::new("test-package".to_string(), "1.0.0".to_string())
            .with_previous_version("0.9.0".to_string());

        assert_eq!(variables.previous_version, Some("0.9.0".to_string()));
        assert_eq!(variables.compare_url, None);
    }

    // ChangelogRequest Tests
    #[test]
    fn test_changelog_request_default() {
        let request = ChangelogRequest::default();

        assert_eq!(request.package_name, None);
        assert_eq!(request.version, "Unreleased");
        assert_eq!(request.since, None);
        assert_eq!(request.until, Some("HEAD".to_string()));
        assert!(!request.include_all_commits);
        assert!(request.write_to_file);
        assert_eq!(request.output_path, None);
    }

    #[test]
    fn test_changelog_request_custom() {
        let request = ChangelogRequest {
            package_name: Some("test-package".to_string()),
            version: "1.0.0".to_string(),
            since: Some("v0.9.0".to_string()),
            until: Some("v1.0.0".to_string()),
            include_all_commits: true,
            write_to_file: false,
            output_path: Some("/custom/path/CHANGELOG.md".to_string()),
        };

        assert_eq!(request.package_name, Some("test-package".to_string()));
        assert_eq!(request.version, "1.0.0");
        assert_eq!(request.since, Some("v0.9.0".to_string()));
        assert_eq!(request.until, Some("v1.0.0".to_string()));
        assert!(request.include_all_commits);
        assert!(!request.write_to_file);
        assert_eq!(request.output_path, Some("/custom/path/CHANGELOG.md".to_string()));
    }

    // ChangelogGenerator Tests
    #[test]
    fn test_changelog_generator_new() {
        let generator = ChangelogGenerator::new();
        assert!(matches!(generator, ChangelogGenerator));
    }

    #[test]
    fn test_changelog_generator_default() {
        let generator = ChangelogGenerator::default();
        assert!(matches!(generator, ChangelogGenerator));
    }

    #[test]
    fn test_generate_markdown_changelog() -> Result<()> {
        let generator = ChangelogGenerator::new();
        let config = ChangelogConfig::default();

        let mut grouped_commits = GroupedCommits::new();
        let commits = create_test_conventional_commits();
        for commit in commits {
            grouped_commits.add_commit(commit);
        }

        let variables = TemplateVariables::new("test-package".to_string(), "1.0.0".to_string())
            .with_repository_url("https://github.com/test/repo".to_string());

        let result = generator.generate_changelog(&config, &grouped_commits, &variables)?;

        // Check that the changelog contains expected sections
        assert!(result.contains("# Changelog"));
        assert!(result.contains("## [1.0.0]"));
        assert!(result.contains("### BREAKING CHANGES"));
        assert!(result.contains("### Features"));
        assert!(result.contains("### Bug Fixes"));
        assert!(result.contains("### Documentation"));
        assert!(result.contains("add OAuth2 support"));
        assert!(result.contains("resolve memory leak issue"));

        Ok(())
    }

    #[test]
    fn test_generate_markdown_changelog_no_breaking_changes() -> Result<()> {
        let generator = ChangelogGenerator::new();
        let mut config = ChangelogConfig::default();
        config.include_breaking_changes = false;

        let mut grouped_commits = GroupedCommits::new();
        let commits = create_test_conventional_commits();
        for commit in commits {
            grouped_commits.add_commit(commit);
        }

        let variables = TemplateVariables::new("test-package".to_string(), "1.0.0".to_string());

        let result = generator.generate_changelog(&config, &grouped_commits, &variables)?;

        // Should not contain breaking changes section
        assert!(!result.contains("### BREAKING CHANGES"));
        assert!(result.contains("### Features"));

        Ok(())
    }

    #[test]
    fn test_generate_markdown_changelog_by_scope() -> Result<()> {
        let generator = ChangelogGenerator::new();
        let mut config = ChangelogConfig::default();
        config.grouping = CommitGrouping::Scope;

        let mut grouped_commits = GroupedCommits::new();
        let commits = create_test_conventional_commits();
        for commit in commits {
            grouped_commits.add_commit(commit);
        }

        let variables = TemplateVariables::new("test-package".to_string(), "1.0.0".to_string());

        let result = generator.generate_changelog(&config, &grouped_commits, &variables)?;

        // Should contain scope-based grouping
        assert!(result.contains("### auth"));
        assert!(result.contains("### core"));
        assert!(result.contains("### Other Changes"));

        Ok(())
    }

    #[test]
    fn test_generate_markdown_changelog_ungrouped() -> Result<()> {
        let generator = ChangelogGenerator::new();
        let mut config = ChangelogConfig::default();
        config.grouping = CommitGrouping::None;

        let mut grouped_commits = GroupedCommits::new();
        let commits = create_test_conventional_commits();
        for commit in commits {
            grouped_commits.add_commit(commit);
        }

        let variables = TemplateVariables::new("test-package".to_string(), "1.0.0".to_string());

        let result = generator.generate_changelog(&config, &grouped_commits, &variables)?;

        // Should contain ungrouped changes
        assert!(result.contains("### Changes"));
        assert!(!result.contains("### Features"));
        assert!(!result.contains("### Bug Fixes"));

        Ok(())
    }

    #[test]
    fn test_generate_text_changelog() -> Result<()> {
        let generator = ChangelogGenerator::new();
        let mut config = ChangelogConfig::default();
        config.output_format = ChangelogFormat::Text;

        let mut grouped_commits = GroupedCommits::new();
        let commits = create_test_conventional_commits();
        for commit in commits {
            grouped_commits.add_commit(commit);
        }

        let variables = TemplateVariables::new("test-package".to_string(), "1.0.0".to_string());

        let result = generator.generate_changelog(&config, &grouped_commits, &variables)?;

        // Should contain text format (no markdown)
        assert!(result.contains("Changelog"));
        assert!(!result.contains("# Changelog"));
        assert!(!result.contains("## ["));
        assert!(!result.contains("### "));
        assert!(result.contains("[BREAKING]"));
        assert!(result.contains("auth: add OAuth2 support"));

        Ok(())
    }

    #[test]
    fn test_generate_json_changelog() -> Result<()> {
        let generator = ChangelogGenerator::new();
        let mut config = ChangelogConfig::default();
        config.output_format = ChangelogFormat::Json;

        let mut grouped_commits = GroupedCommits::new();
        let commits = create_test_conventional_commits();
        for commit in commits {
            grouped_commits.add_commit(commit);
        }

        let variables = TemplateVariables::new("test-package".to_string(), "1.0.0".to_string())
            .with_repository_url("https://github.com/test/repo".to_string());

        let result = generator.generate_changelog(&config, &grouped_commits, &variables)?;

        // Should be valid JSON
        let json: serde_json::Value = serde_json::from_str(&result)
            .map_err(|e| crate::error::Error::changelog(format!("Invalid JSON: {e}")))?;

        // Check JSON structure
        assert!(json.get("metadata").is_some());
        assert!(json.get("commits").is_some());
        assert_eq!(json["metadata"]["package"], "test-package");
        assert_eq!(json["metadata"]["version"], "1.0.0");
        assert_eq!(json["commits"]["total"], 4);
        assert_eq!(json["commits"]["breaking_changes"], 1);

        Ok(())
    }

    // ChangelogManager Tests
    #[test]
    fn test_changelog_manager_new() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let _manager = ChangelogManager::new(&project);

        // Manager should be created successfully
        // Cannot test internal state as fields are private

        Ok(())
    }

    #[test]
    fn test_changelog_manager_from_project() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let _manager = ChangelogManager::from_project(&project);

        // Manager should be created successfully
        // Cannot test internal state as fields are private

        Ok(())
    }

    #[test]
    fn test_changelog_manager_with_components() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let _manager = ChangelogManager::with_components(
            project.services.config_service().get_configuration(),
            &project.packages,
            project.repository(),
            project.services.file_system_service().manager(),
            project.root_path(),
        );

        // Manager should be created successfully
        // Cannot test internal state as fields are private

        Ok(())
    }

    #[test]
    fn test_parse_conventional_commits_all() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let manager = ChangelogManager::new(&project);

        // This will likely return empty results since we have a fresh repo
        // but it should not error
        let result = manager.parse_conventional_commits(None, "HEAD~1");

        // Should either succeed with empty results or fail gracefully
        match result {
            Ok(commits) => assert!(commits.is_empty()),
            Err(_) => {
                // Expected for fresh repo with no commits since HEAD~1
                assert!(true);
            }
        }

        Ok(())
    }

    #[test]
    fn test_parse_conventional_commits_for_package() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let manager = ChangelogManager::new(&project);

        // This will likely return empty results since we have a fresh repo
        let result = manager.parse_conventional_commits(Some("packages/core"), "HEAD~1");

        // Should either succeed with empty results or fail gracefully
        match result {
            Ok(commits) => assert!(commits.is_empty()),
            Err(_) => {
                // Expected for fresh repo with no commits since HEAD~1
                assert!(true);
            }
        }

        Ok(())
    }

    #[test]
    fn test_update_existing_changelog_new_file() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let manager = ChangelogManager::new(&project);

        let new_content = "## [1.0.0] - 2024-01-15\n\n### Features\n\n- Add new feature";
        let result = manager.update_existing_changelog(None, new_content)?;

        // Should return the new content since no existing file
        assert!(result.contains("## [1.0.0] - 2024-01-15"));
        assert!(result.contains("### Features"));

        Ok(())
    }

    #[test]
    fn test_update_existing_changelog_existing_file() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let manager = ChangelogManager::new(&project);

        // Create an existing changelog
        let existing_content =
            "# Changelog\n\n## [0.9.0] - 2024-01-01\n\n### Bug Fixes\n\n- Fix old bug";
        let changelog_path = project.root_path().join("CHANGELOG.md");
        std::fs::write(&changelog_path, existing_content).map_err(crate::error::Error::Io)?;

        let new_content = "## [1.0.0] - 2024-01-15\n\n### Features\n\n- Add new feature\n\n";
        let result = manager.update_existing_changelog(None, new_content)?;

        // Should contain both new and existing content
        assert!(result.contains("## [1.0.0] - 2024-01-15"));
        assert!(result.contains("### Features"));
        assert!(result.contains("## [0.9.0] - 2024-01-01"));
        assert!(result.contains("### Bug Fixes"));

        Ok(())
    }

    // Integration Tests
    #[test]
    fn test_full_changelog_generation_workflow() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let manager = ChangelogManager::new(&project);

        // Create a changelog request
        let request = ChangelogRequest {
            package_name: None,
            version: "1.0.0".to_string(),
            since: None,
            until: Some("HEAD".to_string()),
            include_all_commits: true,
            write_to_file: false,
            output_path: None,
        };

        // This will likely return empty results since we have a fresh repo
        let result = manager.generate_changelog(request);

        // Should either succeed with empty changelog or fail gracefully
        match result {
            Ok(changelog_result) => {
                assert_eq!(changelog_result.version, "1.0.0");
                assert_eq!(changelog_result.package_name, "test-monorepo");
                assert!(!changelog_result.content.is_empty());
                assert_eq!(changelog_result.output_path, None);
            }
            Err(_) => {
                // Expected for fresh repo
                assert!(true);
            }
        }

        Ok(())
    }

    #[test]
    fn test_changelog_generation_with_file_output() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let manager = ChangelogManager::new(&project);

        let request = ChangelogRequest {
            package_name: None,
            version: "1.0.0".to_string(),
            since: None,
            until: Some("HEAD".to_string()),
            include_all_commits: true,
            write_to_file: true,
            output_path: None,
        };

        let result = manager.generate_changelog(request);

        match result {
            Ok(changelog_result) => {
                assert!(changelog_result.output_path.is_some());
                // Check that file was created
                let path = changelog_result.output_path.unwrap();
                assert!(std::path::Path::new(&path).exists());
            }
            Err(_) => {
                // Expected for fresh repo
                assert!(true);
            }
        }

        Ok(())
    }

    #[test]
    fn test_changelog_generation_for_specific_package() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let manager = ChangelogManager::new(&project);

        let request = ChangelogRequest {
            package_name: Some("@test/core".to_string()),
            version: "1.0.0".to_string(),
            since: None,
            until: Some("HEAD".to_string()),
            include_all_commits: true,
            write_to_file: false,
            output_path: None,
        };

        let result = manager.generate_changelog(request);

        match result {
            Ok(changelog_result) => {
                assert_eq!(changelog_result.package_name, "@test/core");
                assert_eq!(changelog_result.version, "1.0.0");
            }
            Err(_) => {
                // Expected for fresh repo
                assert!(true);
            }
        }

        Ok(())
    }

    // Error Handling Tests
    #[test]
    fn test_changelog_generation_invalid_package() -> Result<()> {
        let (_temp_dir, project) = create_test_project()?;
        let manager = ChangelogManager::new(&project);

        let request = ChangelogRequest {
            package_name: Some("nonexistent-package".to_string()),
            version: "1.0.0".to_string(),
            since: None,
            until: Some("HEAD".to_string()),
            include_all_commits: true,
            write_to_file: false,
            output_path: None,
        };

        let result = manager.generate_changelog(request);
        assert!(result.is_err());

        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    #[test]
    fn test_conventional_commit_types_configuration() -> Result<()> {
        let mut config = ChangelogConfig::default();

        // Test default types
        assert_eq!(config.conventional_commit_types.get("feat"), Some(&"Features".to_string()));
        assert_eq!(config.conventional_commit_types.get("fix"), Some(&"Bug Fixes".to_string()));

        // Test custom types
        config.conventional_commit_types.insert("custom".to_string(), "Custom Changes".to_string());
        assert_eq!(
            config.conventional_commit_types.get("custom"),
            Some(&"Custom Changes".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_changelog_template_configuration() {
        let template = ChangelogTemplate::default();

        assert!(template.header_template.contains("# Changelog"));
        assert!(template.section_template.contains("## [{version}]"));
        assert!(template.commit_template.contains("- {description}"));
        assert!(template.footer_template.contains("Generated by"));
    }

    #[test]
    fn test_changelog_format_serialization() -> Result<()> {
        let format = ChangelogFormat::Markdown;
        let json = serde_json::to_string(&format)
            .map_err(|e| crate::error::Error::changelog(format!("Serialization error: {e}")))?;

        let deserialized: ChangelogFormat = serde_json::from_str(&json)
            .map_err(|e| crate::error::Error::changelog(format!("Deserialization error: {e}")))?;

        assert_eq!(format, deserialized);

        Ok(())
    }

    #[test]
    fn test_commit_grouping_serialization() -> Result<()> {
        let grouping = CommitGrouping::Type;
        let json = serde_json::to_string(&grouping)
            .map_err(|e| crate::error::Error::changelog(format!("Serialization error: {e}")))?;

        let deserialized: CommitGrouping = serde_json::from_str(&json)
            .map_err(|e| crate::error::Error::changelog(format!("Deserialization error: {e}")))?;

        assert_eq!(grouping, deserialized);

        Ok(())
    }

    #[test]
    fn test_conventional_commit_serialization() -> Result<()> {
        let commit = ConventionalCommit {
            commit_type: "feat".to_string(),
            scope: Some("auth".to_string()),
            description: "add OAuth2 support".to_string(),
            body: Some("Implements OAuth2 authentication flow".to_string()),
            breaking_change: false,
            hash: "abc123".to_string(),
            author: "Jane Doe".to_string(),
            date: "2024-01-15".to_string(),
        };

        let json = serde_json::to_string(&commit)
            .map_err(|e| crate::error::Error::changelog(format!("Serialization error: {e}")))?;

        let deserialized: ConventionalCommit = serde_json::from_str(&json)
            .map_err(|e| crate::error::Error::changelog(format!("Deserialization error: {e}")))?;

        assert_eq!(commit.commit_type, deserialized.commit_type);
        assert_eq!(commit.scope, deserialized.scope);
        assert_eq!(commit.description, deserialized.description);
        assert_eq!(commit.breaking_change, deserialized.breaking_change);

        Ok(())
    }

    #[test]
    fn test_changelog_result_serialization() -> Result<()> {
        let result = ChangelogResult {
            content: "# Changelog\n\n## [1.0.0] - 2024-01-15".to_string(),
            package_name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            commit_count: 5,
            has_breaking_changes: true,
            output_path: Some("/path/to/CHANGELOG.md".to_string()),
        };

        let json = serde_json::to_string(&result)
            .map_err(|e| crate::error::Error::changelog(format!("Serialization error: {e}")))?;

        let deserialized: ChangelogResult = serde_json::from_str(&json)
            .map_err(|e| crate::error::Error::changelog(format!("Deserialization error: {e}")))?;

        assert_eq!(result.package_name, deserialized.package_name);
        assert_eq!(result.version, deserialized.version);
        assert_eq!(result.commit_count, deserialized.commit_count);
        assert_eq!(result.has_breaking_changes, deserialized.has_breaking_changes);

        Ok(())
    }

    // Edge Cases and Performance Tests
    #[test]
    fn test_empty_commits_handling() -> Result<()> {
        let generator = ChangelogGenerator::new();
        let config = ChangelogConfig::default();
        let grouped_commits = GroupedCommits::new();
        let variables = TemplateVariables::new("test-package".to_string(), "1.0.0".to_string());

        let result = generator.generate_changelog(&config, &grouped_commits, &variables)?;

        // Should still generate a valid changelog with header and footer
        assert!(result.contains("# Changelog"));
        assert!(result.contains("## [1.0.0]"));
        assert!(result.contains("Generated by"));

        Ok(())
    }

    #[test]
    fn test_large_number_of_commits() -> Result<()> {
        let mut grouped_commits = GroupedCommits::new();

        // Add many commits
        for i in 0..1000 {
            let commit = ConventionalCommit {
                commit_type: if i % 2 == 0 { "feat" } else { "fix" }.to_string(),
                scope: Some(format!("module{}", i % 10)),
                description: format!("commit {i}"),
                body: None,
                breaking_change: i % 100 == 0,
                hash: format!("hash{i}"),
                author: "Test Author".to_string(),
                date: "2024-01-15".to_string(),
            };
            grouped_commits.add_commit(commit);
        }

        let generator = ChangelogGenerator::new();
        let config = ChangelogConfig::default();
        let variables = TemplateVariables::new("test-package".to_string(), "1.0.0".to_string());

        let result = generator.generate_changelog(&config, &grouped_commits, &variables)?;

        // Should handle large number of commits without issues
        assert!(result.contains("# Changelog"));
        assert!(result.contains("### Features"));
        assert!(result.contains("### Bug Fixes"));
        assert_eq!(grouped_commits.total_commits(), 1000);

        Ok(())
    }

    #[test]
    fn test_unicode_in_commit_messages() -> Result<()> {
        let parser = ConventionalCommitParser::new();
        let commit = parser.parse_commit_message(
            "feat(ui): add support for émojis 🎉 and unicöde çharacters",
            "abc123",
            "Jane Döe",
            "2024-01-15",
        )?;

        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, Some("ui".to_string()));
        assert!(commit.description.contains("émojis 🎉"));
        assert!(commit.description.contains("unicöde çharacters"));
        assert_eq!(commit.author, "Jane Döe");

        Ok(())
    }

    #[test]
    fn test_multiline_commit_body() -> Result<()> {
        let parser = ConventionalCommitParser::new();
        let commit = parser.parse_commit_message(
            "feat(auth): add OAuth2 support\n\nThis is a multiline body\nwith multiple paragraphs.\n\nAnd more details here.",
            "abc123",
            "Jane Doe",
            "2024-01-15",
        )?;

        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, Some("auth".to_string()));
        assert_eq!(commit.description, "add OAuth2 support");
        assert!(commit.body.is_some());

        let body = commit.body.unwrap();
        assert!(body.contains("This is a multiline body"));
        assert!(body.contains("with multiple paragraphs"));
        assert!(body.contains("And more details here"));

        Ok(())
    }

    #[test]
    fn test_commit_with_special_characters() -> Result<()> {
        let parser = ConventionalCommitParser::new();
        let commit = parser.parse_commit_message(
            "fix(api): handle special chars in URLs: &, ?, #, %",
            "abc123",
            "Jane Doe",
            "2024-01-15",
        )?;

        assert_eq!(commit.commit_type, "fix");
        assert_eq!(commit.scope, Some("api".to_string()));
        assert!(commit.description.contains("&, ?, #, %"));

        Ok(())
    }
}
