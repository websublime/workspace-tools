use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Conventional commit configuration.
///
/// Controls parsing and interpretation of conventional commits
/// for automatic version bump calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionalConfig {
    /// Mapping of commit types to version bump types
    pub types: HashMap<String, ConventionalCommitType>,

    /// Whether to parse breaking changes from commit body
    pub parse_breaking_changes: bool,

    /// Whether to require conventional commit format
    pub require_conventional_commits: bool,

    /// Custom breaking change patterns
    pub breaking_change_patterns: Vec<String>,

    /// Default bump type for unknown commit types
    pub default_bump_type: String,
}

/// Configuration for a conventional commit type.
///
/// Defines how a specific commit type should be interpreted
/// for version bump calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionalCommitType {
    /// Version bump type (patch/minor/major/none)
    pub bump: String,

    /// Whether this type should appear in changelog
    pub changelog: bool,

    /// Display name for changelog sections
    pub changelog_title: Option<String>,

    /// Whether this type indicates a breaking change
    pub breaking: bool,
}

impl Default for ConventionalConfig {
    fn default() -> Self {
        let mut types = HashMap::new();

        types.insert(
            "feat".to_string(),
            ConventionalCommitType {
                bump: "minor".to_string(),
                changelog: true,
                changelog_title: Some("Features".to_string()),
                breaking: false,
            },
        );

        types.insert(
            "fix".to_string(),
            ConventionalCommitType {
                bump: "patch".to_string(),
                changelog: true,
                changelog_title: Some("Bug Fixes".to_string()),
                breaking: false,
            },
        );

        types.insert(
            "perf".to_string(),
            ConventionalCommitType {
                bump: "patch".to_string(),
                changelog: true,
                changelog_title: Some("Performance Improvements".to_string()),
                breaking: false,
            },
        );

        types.insert(
            "breaking".to_string(),
            ConventionalCommitType {
                bump: "major".to_string(),
                changelog: true,
                changelog_title: Some("Breaking Changes".to_string()),
                breaking: true,
            },
        );

        types.insert(
            "docs".to_string(),
            ConventionalCommitType {
                bump: "none".to_string(),
                changelog: false,
                changelog_title: None,
                breaking: false,
            },
        );

        types.insert(
            "style".to_string(),
            ConventionalCommitType {
                bump: "none".to_string(),
                changelog: false,
                changelog_title: None,
                breaking: false,
            },
        );

        types.insert(
            "refactor".to_string(),
            ConventionalCommitType {
                bump: "none".to_string(),
                changelog: false,
                changelog_title: None,
                breaking: false,
            },
        );

        types.insert(
            "test".to_string(),
            ConventionalCommitType {
                bump: "none".to_string(),
                changelog: false,
                changelog_title: None,
                breaking: false,
            },
        );

        types.insert(
            "build".to_string(),
            ConventionalCommitType {
                bump: "none".to_string(),
                changelog: false,
                changelog_title: None,
                breaking: false,
            },
        );

        types.insert(
            "ci".to_string(),
            ConventionalCommitType {
                bump: "none".to_string(),
                changelog: false,
                changelog_title: None,
                breaking: false,
            },
        );

        types.insert(
            "chore".to_string(),
            ConventionalCommitType {
                bump: "none".to_string(),
                changelog: false,
                changelog_title: None,
                breaking: false,
            },
        );

        Self {
            types,
            parse_breaking_changes: true,
            require_conventional_commits: false,
            breaking_change_patterns: vec![
                "BREAKING CHANGE:".to_string(),
                "BREAKING-CHANGE:".to_string(),
            ],
            default_bump_type: "patch".to_string(),
        }
    }
}
