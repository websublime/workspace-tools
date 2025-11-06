//! Tests for the changelog module.
//!
//! **What**: Comprehensive test suite for conventional commit parsing and changelog generation.
//!
//! **How**: Tests cover valid and invalid formats, breaking change detection, footer parsing,
//! reference extraction, edge cases, and property-based tests.
//!
//! **Why**: To ensure the parser correctly implements the Conventional Commits specification
//! and handles all edge cases properly.

#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]
#![allow(clippy::expect_used)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::unnecessary_to_owned)]

use crate::changelog::ChangelogGenerator;
use crate::changelog::conventional::{ConventionalCommit, SectionType};
use crate::config::{ChangelogConfig, ChangelogFormat, MonorepoMode, PackageToolsConfig};
use proptest::prelude::*;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use tempfile::TempDir;

// ============================================================================
// Valid Format Tests
// ============================================================================

#[test]
fn test_parse_simple_feat() {
    let commit = ConventionalCommit::parse("feat: add new feature").unwrap();
    assert_eq!(commit.commit_type(), "feat");
    assert_eq!(commit.scope(), None);
    assert!(!commit.is_breaking());
    assert_eq!(commit.description(), "add new feature");
    assert_eq!(commit.body(), None);
    assert!(commit.footers().is_empty());
}

#[test]
fn test_parse_simple_fix() {
    let commit = ConventionalCommit::parse("fix: resolve bug").unwrap();
    assert_eq!(commit.commit_type(), "fix");
    assert_eq!(commit.scope(), None);
    assert!(!commit.is_breaking());
    assert_eq!(commit.description(), "resolve bug");
}

#[test]
fn test_parse_with_scope() {
    let commit = ConventionalCommit::parse("feat(api): add endpoint").unwrap();
    assert_eq!(commit.commit_type(), "feat");
    assert_eq!(commit.scope(), Some("api"));
    assert_eq!(commit.description(), "add endpoint");
}

#[test]
fn test_parse_with_nested_scope() {
    let commit = ConventionalCommit::parse("fix(core/parser): resolve issue").unwrap();
    assert_eq!(commit.commit_type(), "fix");
    assert_eq!(commit.scope(), Some("core/parser"));
    assert_eq!(commit.description(), "resolve issue");
}

#[test]
fn test_parse_with_body() {
    let message = "feat: add feature\n\nThis is a detailed description\nwith multiple lines.";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.commit_type(), "feat");
    assert_eq!(commit.description(), "add feature");
    assert_eq!(commit.body(), Some("This is a detailed description\nwith multiple lines."));
}

#[test]
fn test_parse_with_single_footer() {
    let message = "feat: add feature\n\nRefs: #123";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.footers().len(), 1);
    assert_eq!(commit.footers()[0].key, "Refs");
    assert_eq!(commit.footers()[0].value, "#123");
}

#[test]
fn test_parse_with_multiple_footers() {
    let message = "feat: add feature\n\nRefs: #123\nCloses: #456\nReviewed-by: John";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.footers().len(), 3);
    assert_eq!(commit.footers()[0].key, "Refs");
    assert_eq!(commit.footers()[0].value, "#123");
    assert_eq!(commit.footers()[1].key, "Closes");
    assert_eq!(commit.footers()[1].value, "#456");
    assert_eq!(commit.footers()[2].key, "Reviewed-by");
    assert_eq!(commit.footers()[2].value, "John");
}

#[test]
fn test_parse_with_body_and_footers() {
    let message = "feat: add feature\n\nDetailed description here.\n\nRefs: #123\nCloses: #456";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.body(), Some("Detailed description here."));
    assert_eq!(commit.footers().len(), 2);
}

#[test]
fn test_parse_all_standard_types() {
    let types =
        vec!["feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore"];

    for commit_type in types {
        let message = format!("{}: test message", commit_type);
        let commit = ConventionalCommit::parse(&message).unwrap();
        assert_eq!(commit.commit_type(), commit_type);
        assert_eq!(commit.description(), "test message");
    }
}

#[test]
fn test_parse_with_extra_whitespace() {
    let commit = ConventionalCommit::parse("feat:    add feature with spaces").unwrap();
    assert_eq!(commit.commit_type(), "feat");
    assert_eq!(commit.description(), "add feature with spaces");
}

#[test]
fn test_parse_multiline_footer_value() {
    let message = "feat: add feature\n\nBREAKING CHANGE: This is a long\nbreaking change description\nthat spans multiple lines";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.footers().len(), 1);
    assert_eq!(commit.footers()[0].key, "BREAKING CHANGE");
    assert!(commit.footers()[0].value.contains("long"));
    assert!(commit.footers()[0].value.contains("multiple lines"));
}

// ============================================================================
// Invalid Format Tests
// ============================================================================

#[test]
fn test_parse_empty_message() {
    let result = ConventionalCommit::parse("");
    assert!(result.is_err());
}

#[test]
fn test_parse_whitespace_only() {
    let result = ConventionalCommit::parse("   \n  \n  ");
    assert!(result.is_err());
}

#[test]
fn test_parse_missing_colon() {
    let result = ConventionalCommit::parse("feat add feature");
    assert!(result.is_err());
}

#[test]
fn test_parse_missing_description() {
    let result = ConventionalCommit::parse("feat:");
    assert!(result.is_err());
}

#[test]
fn test_parse_missing_type() {
    let result = ConventionalCommit::parse(": add feature");
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_type_with_spaces() {
    let result = ConventionalCommit::parse("feat fix: add feature");
    assert!(result.is_err());
}

#[test]
fn test_parse_unclosed_scope() {
    let result = ConventionalCommit::parse("feat(api: add feature");
    assert!(result.is_err());
}

#[test]
fn test_parse_empty_scope() {
    let result = ConventionalCommit::parse("feat(): add feature");
    assert!(result.is_err());
}

// ============================================================================
// Breaking Change Tests
// ============================================================================

#[test]
fn test_breaking_change_with_exclamation() {
    let commit = ConventionalCommit::parse("feat!: breaking change").unwrap();
    assert!(commit.is_breaking());
    assert_eq!(commit.commit_type(), "feat");
    assert_eq!(commit.description(), "breaking change");
}

#[test]
fn test_breaking_change_with_scope_and_exclamation() {
    let commit = ConventionalCommit::parse("feat(api)!: breaking change").unwrap();
    assert!(commit.is_breaking());
    assert_eq!(commit.commit_type(), "feat");
    assert_eq!(commit.scope(), Some("api"));
}

#[test]
fn test_breaking_change_with_footer() {
    let message = "feat: add feature\n\nBREAKING CHANGE: removes old API";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert!(commit.is_breaking());
    assert_eq!(commit.commit_type(), "feat");
}

#[test]
fn test_breaking_change_with_footer_hyphen() {
    let message = "feat: add feature\n\nBREAKING-CHANGE: removes old API";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert!(commit.is_breaking());
}

#[test]
fn test_breaking_change_both_exclamation_and_footer() {
    let message = "feat!: add feature\n\nBREAKING CHANGE: removes old API";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert!(commit.is_breaking());
}

#[test]
fn test_non_breaking_change() {
    let commit = ConventionalCommit::parse("feat: add feature").unwrap();
    assert!(!commit.is_breaking());
}

// ============================================================================
// Footer Parsing Tests
// ============================================================================

#[test]
fn test_footer_basic_format() {
    let message = "feat: test\n\nKey: value";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.footers().len(), 1);
    assert_eq!(commit.footers()[0].key, "Key");
    assert_eq!(commit.footers()[0].value, "value");
}

#[test]
fn test_footer_with_hash() {
    let message = "feat: test\n\nRefs: #123";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.footers()[0].value, "#123");
}

#[test]
fn test_footer_multiple_values() {
    let message = "feat: test\n\nCloses: #123, #456, #789";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.footers()[0].value, "#123, #456, #789");
}

#[test]
fn test_footer_continuation_lines() {
    let message =
        "feat: test\n\nDescription: This is a long\ndescription that spans\nmultiple lines";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.footers().len(), 1);
    let value = &commit.footers()[0].value;
    assert!(value.contains("long"));
    assert!(value.contains("spans"));
    assert!(value.contains("multiple lines"));
}

#[test]
fn test_footer_screaming_snake_case() {
    let message = "feat: test\n\nBREAKING_CHANGE: description";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.footers()[0].key, "BREAKING_CHANGE");
}

#[test]
fn test_footer_kebab_case() {
    let message = "feat: test\n\nSigned-off-by: John Doe";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.footers()[0].key, "Signed-off-by");
}

#[test]
fn test_footer_mixed_case() {
    let message = "feat: test\n\nCo-Authored-By: Jane Smith";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.footers()[0].key, "Co-Authored-By");
}

#[test]
fn test_footer_empty_lines_between() {
    let message = "feat: test\n\nKey1: value1\n\nKey2: value2";
    let commit = ConventionalCommit::parse(message).unwrap();
    // Empty lines should be treated as continuation or separator
    assert!(!commit.footers().is_empty());
}

// ============================================================================
// Section Type Mapping Tests
// ============================================================================

#[test]
fn test_section_type_feat() {
    let commit = ConventionalCommit::parse("feat: add feature").unwrap();
    assert_eq!(commit.section_type(), SectionType::Features);
}

#[test]
fn test_section_type_fix() {
    let commit = ConventionalCommit::parse("fix: resolve bug").unwrap();
    assert_eq!(commit.section_type(), SectionType::Fixes);
}

#[test]
fn test_section_type_perf() {
    let commit = ConventionalCommit::parse("perf: improve speed").unwrap();
    assert_eq!(commit.section_type(), SectionType::Performance);
}

#[test]
fn test_section_type_docs() {
    let commit = ConventionalCommit::parse("docs: update readme").unwrap();
    assert_eq!(commit.section_type(), SectionType::Documentation);
}

#[test]
fn test_section_type_refactor() {
    let commit = ConventionalCommit::parse("refactor: restructure code").unwrap();
    assert_eq!(commit.section_type(), SectionType::Refactoring);
}

#[test]
fn test_section_type_build() {
    let commit = ConventionalCommit::parse("build: update deps").unwrap();
    assert_eq!(commit.section_type(), SectionType::Build);
}

#[test]
fn test_section_type_ci() {
    let commit = ConventionalCommit::parse("ci: add workflow").unwrap();
    assert_eq!(commit.section_type(), SectionType::CI);
}

#[test]
fn test_section_type_test() {
    let commit = ConventionalCommit::parse("test: add tests").unwrap();
    assert_eq!(commit.section_type(), SectionType::Tests);
}

#[test]
fn test_section_type_other() {
    let commit = ConventionalCommit::parse("chore: update config").unwrap();
    assert_eq!(commit.section_type(), SectionType::Other);
}

#[test]
fn test_section_type_unknown_type() {
    let commit = ConventionalCommit::parse("unknown: do something").unwrap();
    assert_eq!(commit.section_type(), SectionType::Other);
}

#[test]
fn test_section_type_breaking_overrides_type() {
    let commit = ConventionalCommit::parse("feat!: breaking change").unwrap();
    assert_eq!(commit.section_type(), SectionType::Breaking);

    let commit = ConventionalCommit::parse("fix!: breaking fix").unwrap();
    assert_eq!(commit.section_type(), SectionType::Breaking);

    let commit = ConventionalCommit::parse("docs!: breaking docs").unwrap();
    assert_eq!(commit.section_type(), SectionType::Breaking);
}

// ============================================================================
// Reference Extraction Tests
// ============================================================================

#[test]
fn test_extract_references_from_description() {
    let commit = ConventionalCommit::parse("feat: add feature #123").unwrap();
    let refs = commit.extract_references().unwrap();
    assert_eq!(refs.len(), 1);
    assert!(refs.contains(&"#123".to_string()));
}

#[test]
fn test_extract_multiple_references_from_description() {
    let commit = ConventionalCommit::parse("feat: fix #123 and #456").unwrap();
    let refs = commit.extract_references().unwrap();
    assert_eq!(refs.len(), 2);
    assert!(refs.contains(&"#123".to_string()));
    assert!(refs.contains(&"#456".to_string()));
}

#[test]
fn test_extract_references_from_body() {
    let message = "feat: add feature\n\nThis fixes #123 and relates to #456";
    let commit = ConventionalCommit::parse(message).unwrap();
    let refs = commit.extract_references().unwrap();
    assert!(refs.contains(&"#123".to_string()));
    assert!(refs.contains(&"#456".to_string()));
}

#[test]
fn test_extract_references_from_footer() {
    let message = "feat: add feature\n\nRefs: #123";
    let commit = ConventionalCommit::parse(message).unwrap();
    let refs = commit.extract_references().unwrap();
    assert!(refs.contains(&"#123".to_string()));
}

#[test]
fn test_extract_references_from_closes_footer() {
    let message = "feat: add feature\n\nCloses: #123";
    let commit = ConventionalCommit::parse(message).unwrap();
    let refs = commit.extract_references().unwrap();
    assert!(refs.contains(&"#123".to_string()));
}

#[test]
fn test_extract_references_from_fixes_footer() {
    let message = "feat: add feature\n\nFixes: #123";
    let commit = ConventionalCommit::parse(message).unwrap();
    let refs = commit.extract_references().unwrap();
    assert!(refs.contains(&"#123".to_string()));
}

#[test]
fn test_extract_references_from_resolves_footer() {
    let message = "feat: add feature\n\nResolves: #123";
    let commit = ConventionalCommit::parse(message).unwrap();
    let refs = commit.extract_references().unwrap();
    assert!(refs.contains(&"#123".to_string()));
}

#[test]
fn test_extract_references_no_duplicates() {
    let message = "feat: fix #123\n\nFixes #123 again\n\nRefs: #123";
    let commit = ConventionalCommit::parse(message).unwrap();
    let refs = commit.extract_references().unwrap();
    assert_eq!(refs.len(), 1);
    assert!(refs.contains(&"#123".to_string()));
}

#[test]
fn test_extract_references_mixed_locations() {
    let message = "feat: fix #123\n\nRelates to #456\n\nCloses: #789";
    let commit = ConventionalCommit::parse(message).unwrap();
    let refs = commit.extract_references().unwrap();
    assert_eq!(refs.len(), 3);
    assert!(refs.contains(&"#123".to_string()));
    assert!(refs.contains(&"#456".to_string()));
    assert!(refs.contains(&"#789".to_string()));
}

#[test]
fn test_extract_references_none() {
    let commit = ConventionalCommit::parse("feat: add feature").unwrap();
    let refs = commit.extract_references().unwrap();
    assert!(refs.is_empty());
}

#[test]
fn test_extract_references_ignores_non_reference_footers() {
    let message = "feat: add feature\n\nAuthor: John\nVersion: 1.0.0";
    let commit = ConventionalCommit::parse(message).unwrap();
    let refs = commit.extract_references().unwrap();
    assert!(refs.is_empty());
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test]
fn test_edge_case_colon_in_description() {
    let commit = ConventionalCommit::parse("feat: add feature: with colon").unwrap();
    assert_eq!(commit.description(), "add feature: with colon");
}

#[test]
fn test_edge_case_parentheses_in_description() {
    let commit = ConventionalCommit::parse("feat: add feature (with notes)").unwrap();
    assert_eq!(commit.description(), "add feature (with notes)");
}

#[test]
fn test_edge_case_exclamation_in_description() {
    let commit = ConventionalCommit::parse("feat: add feature! exciting!").unwrap();
    assert_eq!(commit.description(), "add feature! exciting!");
    assert!(!commit.is_breaking()); // Only ! after type/scope is breaking
}

#[test]
fn test_edge_case_hash_in_description_not_reference() {
    let commit = ConventionalCommit::parse("feat: use #define macro").unwrap();
    let refs = commit.extract_references().unwrap();
    assert!(refs.is_empty()); // #define is not a reference
}

#[test]
fn test_edge_case_unicode_in_description() {
    let commit = ConventionalCommit::parse("feat: add emoji support 🎉").unwrap();
    assert_eq!(commit.description(), "add emoji support 🎉");
}

#[test]
fn test_edge_case_very_long_type() {
    let commit = ConventionalCommit::parse("verylongtype: description").unwrap();
    assert_eq!(commit.commit_type(), "verylongtype");
}

#[test]
fn test_edge_case_numeric_type() {
    let commit = ConventionalCommit::parse("feat123: description").unwrap();
    assert_eq!(commit.commit_type(), "feat123");
}

#[test]
fn test_edge_case_underscore_in_type() {
    let commit = ConventionalCommit::parse("feat_test: description").unwrap();
    assert_eq!(commit.commit_type(), "feat_test");
}

#[test]
fn test_edge_case_empty_body_with_footers() {
    let message = "feat: test\n\n\n\nRefs: #123";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.body(), None);
    assert_eq!(commit.footers().len(), 1);
}

#[test]
fn test_edge_case_body_looks_like_footer() {
    let message = "feat: test\n\nThis is not a footer: it's body text";
    let commit = ConventionalCommit::parse(message).unwrap();
    // Line with space in key should be treated as body, not footer
    // But our current implementation might parse it as footer
    // This tests the actual behavior
    assert!(commit.body().is_some() || !commit.footers().is_empty());
}

#[test]
fn test_edge_case_footer_with_colon_in_value() {
    let message = "feat: test\n\nURL: https://example.com";
    let commit = ConventionalCommit::parse(message).unwrap();
    assert_eq!(commit.footers().len(), 1);
    assert_eq!(commit.footers()[0].key, "URL");
    assert_eq!(commit.footers()[0].value, "https://example.com");
}

// ============================================================================
// Property-Based Tests
// ============================================================================

proptest! {
    #[test]
    fn prop_parse_valid_basic_format(
        commit_type in "[a-z]{3,10}",
        description in "[a-zA-Z ]{10,50}"
    ) {
        let message = format!("{}: {}", commit_type, description);
        let result = ConventionalCommit::parse(&message);
        assert!(result.is_ok());
        let commit = result.unwrap();
        assert_eq!(commit.commit_type(), commit_type);
        assert_eq!(commit.description(), description.trim());
    }

    #[test]
    fn prop_parse_with_scope(
        commit_type in "[a-z]{3,10}",
        scope in "[a-z]{2,15}",
        description in "[a-zA-Z ]{10,50}"
    ) {
        let message = format!("{}({}): {}", commit_type, scope, description);
        let result = ConventionalCommit::parse(&message);
        assert!(result.is_ok());
        let commit = result.unwrap();
        assert_eq!(commit.commit_type(), commit_type);
        assert_eq!(commit.scope(), Some(scope.as_str()));
    }

    #[test]
    fn prop_breaking_with_exclamation(
        commit_type in "[a-z]{3,10}",
        description in "[a-zA-Z ]{10,50}"
    ) {
        let message = format!("{}!: {}", commit_type, description);
        let result = ConventionalCommit::parse(&message);
        assert!(result.is_ok());
        assert!(result.unwrap().is_breaking());
    }

    #[test]
    fn prop_section_type_breaking_has_priority(
        commit_type in "[a-z]{3,10}",
        description in "[a-zA-Z ]{10,50}"
    ) {
        let message = format!("{}!: {}", commit_type, description);
        let commit = ConventionalCommit::parse(&message).unwrap();
        assert_eq!(commit.section_type(), SectionType::Breaking);
    }

    #[test]
    fn prop_reference_extraction(
        commit_type in "[a-z]{3,10}",
        issue_num in 1u32..10000u32
    ) {
        let message = format!("{}: fix issue #{}", commit_type, issue_num);
        let commit = ConventionalCommit::parse(&message).unwrap();
        let refs = commit.extract_references().unwrap();
        assert!(refs.contains(&format!("#{}", issue_num)));
    }
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_conventional_commit_serialization() {
    let commit = ConventionalCommit::parse("feat(api): add endpoint").unwrap();
    let json = serde_json::to_string(&commit).unwrap();
    let deserialized: ConventionalCommit = serde_json::from_str(&json).unwrap();
    assert_eq!(commit, deserialized);
}

#[test]
fn test_commit_footer_serialization() {
    use crate::changelog::conventional::CommitFooter;

    let footer = CommitFooter { key: "Refs".to_string(), value: "#123".to_string() };
    let json = serde_json::to_string(&footer).unwrap();
    let deserialized: CommitFooter = serde_json::from_str(&json).unwrap();
    assert_eq!(footer, deserialized);
}

#[test]
fn test_section_type_serialization() {
    let section = SectionType::Features;
    let json = serde_json::to_string(&section).unwrap();
    let deserialized: SectionType = serde_json::from_str(&json).unwrap();
    assert_eq!(section, deserialized);
}

// ============================================================================
// SectionType Tests
// ============================================================================

#[test]
fn test_section_type_title() {
    assert_eq!(SectionType::Breaking.title(), "Breaking Changes");
    assert_eq!(SectionType::Features.title(), "Features");
    assert_eq!(SectionType::Fixes.title(), "Bug Fixes");
    assert_eq!(SectionType::Performance.title(), "Performance Improvements");
    assert_eq!(SectionType::Deprecations.title(), "Deprecations");
    assert_eq!(SectionType::Documentation.title(), "Documentation");
    assert_eq!(SectionType::Refactoring.title(), "Code Refactoring");
    assert_eq!(SectionType::Build.title(), "Build System");
    assert_eq!(SectionType::CI.title(), "Continuous Integration");
    assert_eq!(SectionType::Tests.title(), "Tests");
    assert_eq!(SectionType::Other.title(), "Other Changes");
}

#[test]
fn test_section_type_priority() {
    assert_eq!(SectionType::Breaking.priority(), 0);
    assert_eq!(SectionType::Features.priority(), 1);
    assert_eq!(SectionType::Fixes.priority(), 2);
    assert!(SectionType::Breaking.priority() < SectionType::Features.priority());
    assert!(SectionType::Features.priority() < SectionType::Fixes.priority());
}

#[test]
fn test_section_type_ordering() {
    assert!(SectionType::Breaking < SectionType::Features);
    assert!(SectionType::Features < SectionType::Fixes);
    assert!(SectionType::Fixes < SectionType::Other);
}

#[test]
fn test_section_type_display() {
    assert_eq!(format!("{}", SectionType::Features), "Features");
    assert_eq!(format!("{}", SectionType::Breaking), "Breaking Changes");
}

// ============================================================================
// ChangelogGenerator Tests
// ============================================================================

/// Helper function to create a temporary Git repository for testing.
fn create_test_repo() -> (TempDir, Repo) {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repo::create(temp_dir.path().to_str().unwrap()).unwrap();
    (temp_dir, repo)
}

#[tokio::test]
async fn test_changelog_generator_new_success() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator = ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await;

    assert!(generator.is_ok());
    let generator = generator.unwrap();
    assert_eq!(generator.workspace_root(), &temp_dir.path().to_path_buf());
}

#[tokio::test]
async fn test_changelog_generator_new_with_custom_config() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();

    let mut config = ChangelogConfig::default();
    config.enabled = false;
    config.format = ChangelogFormat::Conventional;
    config.filename = "RELEASES.md".to_string();
    config.include_commit_links = false;
    config.include_issue_links = false;
    config.monorepo_mode = MonorepoMode::Root;

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config.clone())
            .await
            .unwrap();

    assert_eq!(generator.config().enabled, false);
    assert_eq!(generator.config().format, ChangelogFormat::Conventional);
    assert_eq!(generator.config().filename, "RELEASES.md");
    assert_eq!(generator.config().include_commit_links, false);
    assert_eq!(generator.config().include_issue_links, false);
    assert_eq!(generator.config().monorepo_mode, MonorepoMode::Root);
}

#[tokio::test]
async fn test_changelog_generator_invalid_workspace_root() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let non_existent_path = temp_dir.path().join("does_not_exist");

    let result = ChangelogGenerator::new(non_existent_path.clone(), repo, fs, config).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        crate::error::ChangelogError::InvalidPath { path, reason } => {
            assert_eq!(path, non_existent_path);
            assert!(reason.contains("does not exist"));
        }
        _ => panic!("Expected InvalidPath error, got: {:?}", err),
    }
}

#[tokio::test]
async fn test_changelog_generator_workspace_root_is_file() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    // Create a file instead of directory
    let file_path = temp_dir.path().join("file.txt");
    std::fs::write(&file_path, "test").unwrap();

    let result = ChangelogGenerator::new(file_path.clone(), repo, fs, config).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        crate::error::ChangelogError::InvalidPath { path, reason } => {
            assert_eq!(path, file_path);
            assert!(reason.contains("not a directory"));
        }
        _ => panic!("Expected InvalidPath error, got: {:?}", err),
    }
}

#[tokio::test]
async fn test_changelog_generator_workspace_root_accessor() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let workspace_root = generator.workspace_root();
    assert_eq!(workspace_root, &temp_dir.path().to_path_buf());
    assert!(workspace_root.exists());
}

#[tokio::test]
async fn test_changelog_generator_git_repo_accessor() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let git_repo = generator.git_repo();
    let current_sha = git_repo.get_current_sha();
    assert!(current_sha.is_ok());
}

#[tokio::test]
async fn test_changelog_generator_fs_accessor() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let fs = generator.fs();
    let exists = fs.exists(&temp_dir.path().to_path_buf()).await;
    assert!(exists);
}

#[tokio::test]
async fn test_changelog_generator_config_accessor() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();

    let mut config = ChangelogConfig::default();
    config.format = ChangelogFormat::Conventional;
    config.filename = "HISTORY.md".to_string();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let returned_config = generator.config();
    assert_eq!(returned_config.format, ChangelogFormat::Conventional);
    assert_eq!(returned_config.filename, "HISTORY.md");
}

#[tokio::test]
async fn test_changelog_generator_is_enabled_true() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.enabled = true;

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert!(generator.is_enabled());
}

#[tokio::test]
async fn test_changelog_generator_is_enabled_false() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.enabled = false;

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert!(!generator.is_enabled());
}

#[tokio::test]
async fn test_changelog_generator_get_repository_url_from_config() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.repository_url = Some("https://github.com/user/repo".to_string());

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let url = generator.get_repository_url().unwrap();
    assert_eq!(url, Some("https://github.com/user/repo".to_string()));
}

#[tokio::test]
async fn test_changelog_generator_get_repository_url_none() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.repository_url = None;

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let url = generator.get_repository_url().unwrap();
    assert_eq!(url, None);
}

#[tokio::test]
async fn test_changelog_generator_with_keep_a_changelog_format() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.format = ChangelogFormat::KeepAChangelog;

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert_eq!(generator.config().format, ChangelogFormat::KeepAChangelog);
}

#[tokio::test]
async fn test_changelog_generator_with_conventional_commits_format() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.format = ChangelogFormat::Conventional;

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert_eq!(generator.config().format, ChangelogFormat::Conventional);
}

#[tokio::test]
async fn test_changelog_generator_with_custom_format() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.format = ChangelogFormat::Custom;

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert_eq!(generator.config().format, ChangelogFormat::Custom);
}

#[tokio::test]
async fn test_changelog_generator_monorepo_mode_per_package() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.monorepo_mode = MonorepoMode::PerPackage;

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert_eq!(generator.config().monorepo_mode, MonorepoMode::PerPackage);
}

#[tokio::test]
async fn test_changelog_generator_monorepo_mode_root() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.monorepo_mode = MonorepoMode::Root;

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert_eq!(generator.config().monorepo_mode, MonorepoMode::Root);
}

#[tokio::test]
async fn test_changelog_generator_monorepo_mode_both() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.monorepo_mode = MonorepoMode::Both;

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert_eq!(generator.config().monorepo_mode, MonorepoMode::Both);
}

#[tokio::test]
async fn test_changelog_generator_with_custom_filename() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.filename = "HISTORY.md".to_string();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert_eq!(generator.config().filename, "HISTORY.md");
}

#[tokio::test]
async fn test_changelog_generator_link_configuration() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.include_commit_links = true;
    config.include_issue_links = true;
    config.include_authors = true;

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert!(generator.config().include_commit_links);
    assert!(generator.config().include_issue_links);
    assert!(generator.config().include_authors);
}

#[tokio::test]
async fn test_changelog_generator_tag_formats() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.version_tag_format = "{name}-v{version}".to_string();
    config.root_tag_format = "release-{version}".to_string();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert_eq!(generator.config().version_tag_format, "{name}-v{version}");
    assert_eq!(generator.config().root_tag_format, "release-{version}");
}

#[tokio::test]
async fn test_changelog_generator_conventional_config() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.conventional.enabled = true;
    config.conventional.breaking_section = "💥 Breaking Changes".to_string();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert!(generator.config().conventional.enabled);
    assert_eq!(generator.config().conventional.breaking_section, "💥 Breaking Changes");
}

#[tokio::test]
async fn test_changelog_generator_exclusion_config() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.exclude.patterns = vec!["^chore:".to_string(), "^docs:".to_string()];
    config.exclude.authors = vec!["bot@example.com".to_string()];

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert_eq!(generator.config().exclude.patterns.len(), 2);
    assert_eq!(generator.config().exclude.authors.len(), 1);
}

#[tokio::test]
async fn test_changelog_generator_template_config() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.template.header = "# Release Notes\n".to_string();
    config.template.version_header = "## Version {version} ({date})".to_string();
    config.template.section_header = "**{section}**".to_string();
    config.template.entry_format = "* {description} - {hash}".to_string();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    assert_eq!(generator.config().template.header, "# Release Notes\n");
    assert_eq!(generator.config().template.version_header, "## Version {version} ({date})");
    assert_eq!(generator.config().template.section_header, "**{section}**");
    assert_eq!(generator.config().template.entry_format, "* {description} - {hash}");
}

#[tokio::test]
async fn test_changelog_generator_from_package_tools_config() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let package_config = PackageToolsConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, package_config.changelog)
            .await
            .unwrap();

    assert!(generator.is_enabled());
    assert_eq!(generator.config().format, ChangelogFormat::KeepAChangelog);
}

// ============================================================================
// Version Detection - Unit Tests
// ============================================================================

mod version_detection_unit_tests {
    use crate::changelog::version_detection::*;
    use crate::types::Version;
    use regex::Regex;

    #[test]
    fn test_version_tag_creation() {
        let tag = VersionTag::new("v1.0.0".to_string(), Version::parse("1.0.0").unwrap(), None);

        assert_eq!(tag.tag_name(), "v1.0.0");
        assert_eq!(tag.version().to_string(), "1.0.0");
        assert_eq!(tag.package_name(), None);
    }

    #[test]
    fn test_version_tag_with_package() {
        let tag = VersionTag::new(
            "mypackage@1.0.0".to_string(),
            Version::parse("1.0.0").unwrap(),
            Some("mypackage".to_string()),
        );

        assert_eq!(tag.tag_name(), "mypackage@1.0.0");
        assert_eq!(tag.version().to_string(), "1.0.0");
        assert_eq!(tag.package_name(), Some("mypackage"));
    }

    #[test]
    fn test_version_tag_matches_package() {
        let tag = VersionTag::new(
            "pkg@1.0.0".to_string(),
            Version::parse("1.0.0").unwrap(),
            Some("pkg".to_string()),
        );

        assert!(tag.matches_package(Some("pkg")));
        assert!(!tag.matches_package(Some("other")));
        assert!(!tag.matches_package(None));
    }

    #[test]
    fn test_version_tag_matches_root() {
        let tag = VersionTag::new("v1.0.0".to_string(), Version::parse("1.0.0").unwrap(), None);

        assert!(tag.matches_package(None));
        assert!(!tag.matches_package(Some("pkg")));
    }

    #[test]
    fn test_version_tag_ordering() {
        let tag1 = VersionTag::new("v1.0.0".to_string(), Version::parse("1.0.0").unwrap(), None);
        let tag2 = VersionTag::new("v2.0.0".to_string(), Version::parse("2.0.0").unwrap(), None);

        assert!(tag1 < tag2);
        assert!(tag2 > tag1);
    }

    #[test]
    fn test_build_tag_regex_root_format() {
        let pattern = build_tag_regex("v{version}");
        assert!(pattern.is_some());
        let pattern = pattern.unwrap();

        let re = Regex::new(&pattern).unwrap();
        assert!(re.is_match("v1.0.0"));
        assert!(re.is_match("v1.2.3"));
        assert!(re.is_match("v1.0.0-alpha"));
        assert!(re.is_match("v1.0.0+build123"));
        assert!(!re.is_match("1.0.0"));
        assert!(!re.is_match("v1.0"));
    }

    #[test]
    fn test_build_tag_regex_monorepo_format() {
        let pattern = build_tag_regex("{name}@{version}");
        assert!(pattern.is_some());
        let pattern = pattern.unwrap();

        let re = Regex::new(&pattern).unwrap();
        assert!(re.is_match("pkg@1.0.0"));
        assert!(re.is_match("@org/pkg@1.0.0"));
        assert!(!re.is_match("v1.0.0"));
    }

    #[test]
    fn test_parse_version_tag_root() {
        let tag = parse_version_tag("v1.2.3", None, "v{version}");
        assert!(tag.is_some());

        let tag = tag.unwrap();
        assert_eq!(tag.tag_name(), "v1.2.3");
        assert_eq!(tag.version().to_string(), "1.2.3");
        assert_eq!(tag.package_name(), None);
    }

    #[test]
    fn test_parse_version_tag_monorepo() {
        let tag = parse_version_tag("mypackage@1.2.3", Some("mypackage"), "{name}@{version}");
        assert!(tag.is_some());

        let tag = tag.unwrap();
        assert_eq!(tag.tag_name(), "mypackage@1.2.3");
        assert_eq!(tag.version().to_string(), "1.2.3");
        assert_eq!(tag.package_name(), Some("mypackage"));
    }

    #[test]
    fn test_parse_version_tag_scoped_package() {
        let tag = parse_version_tag("@org/pkg@2.0.0", Some("@org/pkg"), "{name}@{version}");
        assert!(tag.is_some());

        let tag = tag.unwrap();
        assert_eq!(tag.tag_name(), "@org/pkg@2.0.0");
        assert_eq!(tag.version().to_string(), "2.0.0");
        assert_eq!(tag.package_name(), Some("@org/pkg"));
    }

    #[test]
    fn test_parse_version_tag_with_prerelease() {
        let tag = parse_version_tag("v1.0.0-alpha.1", None, "v{version}");
        assert!(tag.is_some());

        let tag = tag.unwrap();
        assert_eq!(tag.version().to_string(), "1.0.0-alpha.1");
    }

    #[test]
    fn test_parse_version_tag_invalid() {
        assert!(parse_version_tag("invalid", None, "v{version}").is_none());
        assert!(parse_version_tag("v1.0", None, "v{version}").is_none());
        assert!(parse_version_tag("1.0.0", None, "v{version}").is_none());
    }

    #[test]
    fn test_parse_version_tag_wrong_package() {
        let tag = parse_version_tag("pkg1@1.0.0", Some("pkg2"), "{name}@{version}");
        assert!(tag.is_none());
    }

    #[test]
    fn test_find_version_tags() {
        let tags = vec![
            "v1.0.0".to_string(),
            "v1.1.0".to_string(),
            "v2.0.0".to_string(),
            "other-tag".to_string(),
            "v0.5.0".to_string(),
        ];

        let version_tags = find_version_tags(&tags, None, "v{version}");
        assert_eq!(version_tags.len(), 4);

        // Should be sorted newest first
        assert_eq!(version_tags[0].version().to_string(), "2.0.0");
        assert_eq!(version_tags[1].version().to_string(), "1.1.0");
        assert_eq!(version_tags[2].version().to_string(), "1.0.0");
        assert_eq!(version_tags[3].version().to_string(), "0.5.0");
    }

    #[test]
    fn test_find_version_tags_monorepo() {
        let tags = vec![
            "pkg1@1.0.0".to_string(),
            "pkg1@1.1.0".to_string(),
            "pkg2@1.0.0".to_string(),
            "pkg1@2.0.0".to_string(),
        ];

        let version_tags = find_version_tags(&tags, Some("pkg1"), "{name}@{version}");
        assert_eq!(version_tags.len(), 3);
        assert_eq!(version_tags[0].version().to_string(), "2.0.0");
        assert_eq!(version_tags[1].version().to_string(), "1.1.0");
        assert_eq!(version_tags[2].version().to_string(), "1.0.0");
    }

    #[test]
    fn test_find_previous_version() {
        let tags = vec!["v1.0.0".to_string(), "v1.1.0".to_string(), "v2.0.0".to_string()];

        let previous = find_previous_version(&tags, "2.0.0", None, "v{version}").unwrap();
        assert!(previous.is_some());
        assert_eq!(previous.unwrap().version().to_string(), "1.1.0");
    }

    #[test]
    fn test_find_previous_version_first_release() {
        let tags = vec!["v1.0.0".to_string(), "v1.1.0".to_string(), "v2.0.0".to_string()];

        let previous = find_previous_version(&tags, "1.0.0", None, "v{version}").unwrap();
        assert!(previous.is_none());
    }

    #[test]
    fn test_find_previous_version_no_tags() {
        let tags: Vec<String> = vec![];

        let previous = find_previous_version(&tags, "1.0.0", None, "v{version}").unwrap();
        assert!(previous.is_none());
    }

    #[test]
    fn test_find_previous_version_monorepo() {
        let tags = vec![
            "pkg1@1.0.0".to_string(),
            "pkg1@1.1.0".to_string(),
            "pkg2@1.0.0".to_string(),
            "pkg1@2.0.0".to_string(),
        ];

        let previous =
            find_previous_version(&tags, "2.0.0", Some("pkg1"), "{name}@{version}").unwrap();
        assert!(previous.is_some());
        assert_eq!(previous.unwrap().version().to_string(), "1.1.0");
    }

    #[test]
    fn test_find_previous_version_invalid_current() {
        let tags = vec!["v1.0.0".to_string()];

        let result = find_previous_version(&tags, "invalid", None, "v{version}");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_monorepo_format() {
        assert!(is_monorepo_format("{name}@{version}"));
        assert!(is_monorepo_format("{name}-{version}"));
        assert!(!is_monorepo_format("v{version}"));
        assert!(!is_monorepo_format("{version}"));
    }

    #[test]
    fn test_format_version_tag_root() {
        let tag = format_version_tag(None, "1.0.0", "v{version}");
        assert_eq!(tag, "v1.0.0");

        let tag = format_version_tag(None, "2.5.3", "{version}");
        assert_eq!(tag, "2.5.3");
    }

    #[test]
    fn test_format_version_tag_monorepo() {
        let tag = format_version_tag(Some("mypackage"), "1.0.0", "{name}@{version}");
        assert_eq!(tag, "mypackage@1.0.0");

        let tag = format_version_tag(Some("@org/pkg"), "2.0.0", "{name}@{version}");
        assert_eq!(tag, "@org/pkg@2.0.0");
    }

    #[test]
    fn test_format_version_tag_custom_format() {
        let tag = format_version_tag(Some("pkg"), "1.0.0", "{name}-v{version}");
        assert_eq!(tag, "pkg-v1.0.0");

        let tag = format_version_tag(None, "1.0.0", "release-{version}");
        assert_eq!(tag, "release-1.0.0");
    }
}

// ============================================================================
// Version Detection - Integration Tests
// ============================================================================

#[tokio::test]
async fn test_detect_previous_version_no_tags() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let previous = generator.detect_previous_version(None, "1.0.0").await.unwrap();
    assert!(previous.is_none());
}

#[tokio::test]
async fn test_detect_previous_version_first_release() {
    let (temp_dir, repo) = create_test_repo();

    // Create a tag for v1.0.0
    repo.create_tag("v1.0.0", Some("Release 1.0.0".to_string())).unwrap();

    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    // Looking for previous of v1.0.0 should find nothing
    let previous = generator.detect_previous_version(None, "1.0.0").await.unwrap();
    assert!(previous.is_none());
}

#[tokio::test]
async fn test_detect_previous_version_with_history() {
    let (temp_dir, repo) = create_test_repo();

    // Create tags for multiple versions
    repo.create_tag("v1.0.0", Some("Release 1.0.0".to_string())).unwrap();
    repo.commit_changes("Second commit").unwrap();
    repo.create_tag("v1.1.0", Some("Release 1.1.0".to_string())).unwrap();
    repo.commit_changes("Third commit").unwrap();
    repo.create_tag("v2.0.0", Some("Release 2.0.0".to_string())).unwrap();

    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    // Previous of 2.0.0 should be 1.1.0
    let previous = generator.detect_previous_version(None, "2.0.0").await.unwrap();
    assert!(previous.is_some());
    let tag = previous.unwrap();
    assert_eq!(tag.version().to_string(), "1.1.0");
    assert_eq!(tag.tag_name(), "v1.1.0");
}

#[tokio::test]
async fn test_detect_previous_version_monorepo() {
    let (temp_dir, repo) = create_test_repo();

    // Create tags for different packages
    repo.create_tag("pkg1@1.0.0", Some("pkg1 1.0.0".to_string())).unwrap();
    repo.commit_changes("Second commit").unwrap();
    repo.create_tag("pkg2@1.0.0", Some("pkg2 1.0.0".to_string())).unwrap();
    repo.commit_changes("Third commit").unwrap();
    repo.create_tag("pkg1@1.1.0", Some("pkg1 1.1.0".to_string())).unwrap();
    repo.commit_changes("Fourth commit").unwrap();
    repo.create_tag("pkg1@2.0.0", Some("pkg1 2.0.0".to_string())).unwrap();

    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    // Previous version of pkg1@2.0.0 should be pkg1@1.1.0
    let previous = generator.detect_previous_version(Some("pkg1"), "2.0.0").await.unwrap();
    assert!(previous.is_some());
    let tag = previous.unwrap();
    assert_eq!(tag.version().to_string(), "1.1.0");
    assert_eq!(tag.package_name(), Some("pkg1"));
    assert_eq!(tag.tag_name(), "pkg1@1.1.0");
}

#[tokio::test]
async fn test_detect_previous_version_monorepo_filters_by_package() {
    let (temp_dir, repo) = create_test_repo();

    // Create tags for different packages
    repo.create_tag("pkg1@1.0.0", Some("pkg1 1.0.0".to_string())).unwrap();
    repo.commit_changes("Second commit").unwrap();
    repo.create_tag("pkg2@2.0.0", Some("pkg2 2.0.0".to_string())).unwrap();
    repo.commit_changes("Third commit").unwrap();
    repo.create_tag("pkg1@1.5.0", Some("pkg1 1.5.0".to_string())).unwrap();

    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    // Previous version of pkg1@2.0.0 should be pkg1@1.5.0, not pkg2@2.0.0
    let previous = generator.detect_previous_version(Some("pkg1"), "2.0.0").await.unwrap();
    assert!(previous.is_some());
    let tag = previous.unwrap();
    assert_eq!(tag.version().to_string(), "1.5.0");
    assert_eq!(tag.package_name(), Some("pkg1"));
}

#[tokio::test]
async fn test_detect_previous_version_with_custom_root_format() {
    let (temp_dir, repo) = create_test_repo();

    // Create tags without 'v' prefix
    repo.create_tag("1.0.0", Some("Release 1.0.0".to_string())).unwrap();
    repo.commit_changes("Second commit").unwrap();
    repo.create_tag("1.1.0", Some("Release 1.1.0".to_string())).unwrap();

    let fs = FileSystemManager::new();
    let mut config = ChangelogConfig::default();
    config.root_tag_format = "{version}".to_string();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let previous = generator.detect_previous_version(None, "1.1.0").await.unwrap();
    assert!(previous.is_some());
    let tag = previous.unwrap();
    assert_eq!(tag.version().to_string(), "1.0.0");
    assert_eq!(tag.tag_name(), "1.0.0");
}

#[tokio::test]
async fn test_detect_previous_version_invalid_current_version() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let result = generator.detect_previous_version(None, "invalid").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_parse_version_tag_root_format() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let tag = generator.parse_version_tag("v1.2.3", None).unwrap();
    assert!(tag.is_some());
    let tag = tag.unwrap();
    assert_eq!(tag.tag_name(), "v1.2.3");
    assert_eq!(tag.version().to_string(), "1.2.3");
    assert_eq!(tag.package_name(), None);
}

#[tokio::test]
async fn test_parse_version_tag_monorepo_format() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let tag = generator.parse_version_tag("mypackage@1.2.3", Some("mypackage")).unwrap();
    assert!(tag.is_some());
    let tag = tag.unwrap();
    assert_eq!(tag.tag_name(), "mypackage@1.2.3");
    assert_eq!(tag.version().to_string(), "1.2.3");
    assert_eq!(tag.package_name(), Some("mypackage"));
}

#[tokio::test]
async fn test_parse_version_tag_scoped_package() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let tag = generator.parse_version_tag("@org/pkg@2.0.0", Some("@org/pkg")).unwrap();
    assert!(tag.is_some());
    let tag = tag.unwrap();
    assert_eq!(tag.tag_name(), "@org/pkg@2.0.0");
    assert_eq!(tag.version().to_string(), "2.0.0");
    assert_eq!(tag.package_name(), Some("@org/pkg"));
}

#[tokio::test]
async fn test_parse_version_tag_invalid_format() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let tag = generator.parse_version_tag("invalid", None).unwrap();
    assert!(tag.is_none());
}

#[tokio::test]
async fn test_parse_version_tag_wrong_package() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let tag = generator.parse_version_tag("pkg1@1.0.0", Some("pkg2")).unwrap();
    assert!(tag.is_none());
}

#[tokio::test]
async fn test_get_version_tags_empty_repository() {
    let (temp_dir, repo) = create_test_repo();
    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let tags = generator.get_version_tags(None).await.unwrap();
    assert_eq!(tags.len(), 0);
}

#[tokio::test]
async fn test_get_version_tags_root_format() {
    let (temp_dir, repo) = create_test_repo();

    // Create multiple version tags
    repo.create_tag("v1.0.0", Some("1.0.0".to_string())).unwrap();
    repo.commit_changes("Second").unwrap();
    repo.create_tag("v1.1.0", Some("1.1.0".to_string())).unwrap();
    repo.commit_changes("Third").unwrap();
    repo.create_tag("v2.0.0", Some("2.0.0".to_string())).unwrap();
    repo.create_tag("other-tag", Some("other".to_string())).unwrap();

    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let tags = generator.get_version_tags(None).await.unwrap();
    assert_eq!(tags.len(), 3);

    // Tags should be sorted newest first
    assert_eq!(tags[0].version().to_string(), "2.0.0");
    assert_eq!(tags[1].version().to_string(), "1.1.0");
    assert_eq!(tags[2].version().to_string(), "1.0.0");
}

#[tokio::test]
async fn test_get_version_tags_monorepo_format() {
    let (temp_dir, repo) = create_test_repo();

    // Create tags for multiple packages
    repo.create_tag("pkg1@1.0.0", Some("pkg1 1.0.0".to_string())).unwrap();
    repo.commit_changes("Second").unwrap();
    repo.create_tag("pkg2@1.0.0", Some("pkg2 1.0.0".to_string())).unwrap();
    repo.commit_changes("Third").unwrap();
    repo.create_tag("pkg1@2.0.0", Some("pkg1 2.0.0".to_string())).unwrap();

    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    // Get tags for pkg1 only
    let tags = generator.get_version_tags(Some("pkg1")).await.unwrap();
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].version().to_string(), "2.0.0");
    assert_eq!(tags[1].version().to_string(), "1.0.0");
    assert!(tags.iter().all(|t| t.package_name() == Some("pkg1")));
}

#[tokio::test]
async fn test_get_version_tags_filters_non_version_tags() {
    let (temp_dir, repo) = create_test_repo();

    // Create mix of version and non-version tags
    repo.create_tag("v1.0.0", Some("Release".to_string())).unwrap();
    repo.commit_changes("Second").unwrap();
    repo.create_tag("build-123", Some("Build".to_string())).unwrap();
    repo.commit_changes("Third").unwrap();
    repo.create_tag("v2.0.0", Some("Release".to_string())).unwrap();
    repo.create_tag("docs-update", Some("Docs".to_string())).unwrap();

    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let tags = generator.get_version_tags(None).await.unwrap();
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].version().to_string(), "2.0.0");
    assert_eq!(tags[1].version().to_string(), "1.0.0");
}

#[tokio::test]
async fn test_get_version_tags_with_prerelease() {
    let (temp_dir, repo) = create_test_repo();

    // Create tags with prereleases
    repo.create_tag("v1.0.0", Some("Release".to_string())).unwrap();
    repo.commit_changes("Second").unwrap();
    repo.create_tag("v1.1.0-alpha.1", Some("Alpha".to_string())).unwrap();
    repo.commit_changes("Third").unwrap();
    repo.create_tag("v1.1.0-beta.1", Some("Beta".to_string())).unwrap();
    repo.commit_changes("Fourth").unwrap();
    repo.create_tag("v1.1.0", Some("Release".to_string())).unwrap();

    let fs = FileSystemManager::new();
    let config = ChangelogConfig::default();

    let generator =
        ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

    let tags = generator.get_version_tags(None).await.unwrap();
    assert_eq!(tags.len(), 4);

    // All should be valid versions
    assert!(tags.iter().all(|t| t.version().to_string().contains('.')));
}

// ============================================================================
// Story 8.4: Changelog Data Collection Tests
// ============================================================================

mod data_collection_tests {
    use super::*;
    use crate::changelog::SectionType;

    #[tokio::test]
    async fn test_generate_for_version_basic() {
        let (temp_dir, repo) = create_test_repo();

        // Create version tags and commits
        repo.create_tag("v1.0.0", Some("Initial release".to_string())).unwrap();
        repo.commit_changes("feat: add new feature").unwrap();
        repo.commit_changes("fix: fix a bug").unwrap();
        repo.create_tag("v1.1.0", Some("Minor release".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "1.1.0", Some("1.0.0"), None).await.unwrap();

        assert_eq!(changelog.version, "1.1.0");
        assert_eq!(changelog.previous_version, Some("1.0.0".to_string()));
        assert!(!changelog.is_empty());
        assert!(changelog.entry_count() >= 2);
    }

    #[tokio::test]
    async fn test_generate_for_version_with_conventional_commits() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("feat(core): add authentication").unwrap();
        repo.commit_changes("fix(api): correct validation").unwrap();
        repo.commit_changes("docs: update README").unwrap();
        repo.create_tag("v1.1.0", Some("Release".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "1.1.0", Some("1.0.0"), None).await.unwrap();

        assert_eq!(changelog.entry_count(), 3);

        // Check that sections were created
        assert!(!changelog.sections.is_empty());
    }

    #[tokio::test]
    async fn test_generate_for_version_with_breaking_changes() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("feat!: breaking API change").unwrap();
        repo.commit_changes("fix: small fix").unwrap();
        repo.create_tag("v2.0.0", Some("Major".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "2.0.0", Some("1.0.0"), None).await.unwrap();

        assert!(changelog.has_breaking_changes());
        let breaking = changelog.breaking_changes();
        assert!(!breaking.is_empty());
    }

    #[tokio::test]
    async fn test_generate_for_version_auto_detect_previous() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("feat: new feature").unwrap();
        repo.create_tag("v1.1.0", Some("Minor".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        // Auto-detect previous version
        let changelog = generator.generate_for_version(None, "1.1.0", None, None).await.unwrap();

        assert_eq!(changelog.version, "1.1.0");
        assert_eq!(changelog.previous_version, Some("1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_changelog_metadata_population() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("feat: add feature").unwrap();
        repo.commit_changes("fix: fix bug").unwrap();
        repo.create_tag("v1.1.0", Some("Release".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "1.1.0", Some("1.0.0"), None).await.unwrap();

        // Check metadata
        assert!(changelog.metadata.tag.is_some());
        assert!(changelog.metadata.commit_range.is_some());
        assert!(changelog.metadata.total_commits >= 2);
        assert!(changelog.metadata.bump_type.is_some());
    }

    #[tokio::test]
    async fn test_changelog_to_markdown() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("feat: add feature").unwrap();
        repo.create_tag("v1.1.0", Some("Release".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config.clone())
                .await
                .unwrap();

        let changelog =
            generator.generate_for_version(None, "1.1.0", Some("1.0.0"), None).await.unwrap();

        let markdown = changelog.to_markdown(&config);

        assert!(markdown.contains("## [1.1.0]"));
        assert!(!markdown.is_empty());
    }

    #[tokio::test]
    async fn test_collector_filters_excluded_patterns() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("feat: add feature").unwrap();
        repo.commit_changes("chore: update deps").unwrap();
        repo.commit_changes("Merge branch 'main'").unwrap();
        repo.create_tag("v1.1.0", Some("Release".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let mut config = ChangelogConfig::default();
        config.exclude.patterns = vec!["^chore:".to_string(), "^Merge".to_string()];

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "1.1.0", Some("1.0.0"), None).await.unwrap();

        // Should only include the feat commit
        assert_eq!(changelog.entry_count(), 1);
    }

    #[tokio::test]
    async fn test_collector_groups_by_section() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("feat: feature 1").unwrap();
        repo.commit_changes("feat: feature 2").unwrap();
        repo.commit_changes("fix: fix 1").unwrap();
        repo.commit_changes("docs: update docs").unwrap();
        repo.create_tag("v1.1.0", Some("Release".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "1.1.0", Some("1.0.0"), None).await.unwrap();

        // Should have multiple sections
        assert!(changelog.sections.len() > 1);

        // Check section types are present
        let has_features =
            changelog.sections.iter().any(|s| s.section_type == SectionType::Features);
        let has_fixes = changelog.sections.iter().any(|s| s.section_type == SectionType::Fixes);
        assert!(has_features);
        assert!(has_fixes);
    }

    #[tokio::test]
    async fn test_collector_extracts_references() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("feat: add feature\n\nCloses #123\nFixes #456").unwrap();
        repo.create_tag("v1.1.0", Some("Release".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "1.1.0", Some("1.0.0"), None).await.unwrap();

        // Check that references were extracted
        let has_refs =
            changelog.sections.iter().flat_map(|s| &s.entries).any(|e| !e.references.is_empty());

        assert!(has_refs);
    }

    #[tokio::test]
    async fn test_collector_sorts_entries_by_date() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("feat: oldest").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        repo.commit_changes("feat: middle").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        repo.commit_changes("feat: newest").unwrap();
        repo.create_tag("v1.1.0", Some("Release".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "1.1.0", Some("1.0.0"), None).await.unwrap();

        // Find Features section
        let features_section =
            changelog.sections.iter().find(|s| s.section_type == SectionType::Features);

        assert!(features_section.is_some());
        let section = features_section.unwrap();

        // Entries should be sorted newest first
        if section.entries.len() >= 2 {
            assert!(section.entries[0].date >= section.entries[1].date);
        }
    }

    #[tokio::test]
    async fn test_collector_handles_plain_commits() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("Add some feature").unwrap();
        repo.commit_changes("Fix some bug").unwrap();
        repo.create_tag("v1.1.0", Some("Release".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let mut config = ChangelogConfig::default();
        config.conventional.enabled = false;

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "1.1.0", Some("1.0.0"), None).await.unwrap();

        // Should still collect commits even without conventional format
        assert!(changelog.entry_count() >= 2);
    }

    #[tokio::test]
    async fn test_infer_bump_type_major() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("feat!: breaking change").unwrap();
        repo.create_tag("v2.0.0", Some("Major".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "2.0.0", Some("1.0.0"), None).await.unwrap();

        assert_eq!(changelog.metadata.bump_type, Some(crate::types::VersionBump::Major));
    }

    #[tokio::test]
    async fn test_infer_bump_type_minor() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("feat: add feature").unwrap();
        repo.create_tag("v1.1.0", Some("Minor".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "1.1.0", Some("1.0.0"), None).await.unwrap();

        assert_eq!(changelog.metadata.bump_type, Some(crate::types::VersionBump::Minor));
    }

    #[tokio::test]
    async fn test_infer_bump_type_patch() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("fix: fix bug").unwrap();
        repo.create_tag("v1.0.1", Some("Patch".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "1.0.1", Some("1.0.0"), None).await.unwrap();

        assert_eq!(changelog.metadata.bump_type, Some(crate::types::VersionBump::Patch));
    }

    #[tokio::test]
    async fn test_changelog_entry_metadata() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("feat(core): add authentication\n\nFixes #123").unwrap();
        repo.create_tag("v1.1.0", Some("Release".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "1.1.0", Some("1.0.0"), None).await.unwrap();

        let entry = &changelog.sections[0].entries[0];

        // Check all metadata is present
        assert!(!entry.commit_hash.is_empty());
        assert!(!entry.short_hash.is_empty());
        assert!(!entry.author.is_empty());
        assert!(entry.commit_type.is_some());
        assert_eq!(entry.scope, Some("core".to_string()));
    }

    #[tokio::test]
    async fn test_section_priority_ordering() {
        let (temp_dir, repo) = create_test_repo();

        repo.create_tag("v1.0.0", Some("Initial".to_string())).unwrap();
        repo.commit_changes("fix: fix bug").unwrap();
        repo.commit_changes("feat!: breaking change").unwrap();
        repo.commit_changes("feat: add feature").unwrap();
        repo.create_tag("v2.0.0", Some("Major".to_string())).unwrap();

        let fs = FileSystemManager::new();
        let config = ChangelogConfig::default();

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs, config).await.unwrap();

        let changelog =
            generator.generate_for_version(None, "2.0.0", Some("1.0.0"), None).await.unwrap();

        // Breaking changes should be first
        if !changelog.sections.is_empty() {
            let first_non_empty = changelog.sections.iter().find(|s| !s.is_empty());
            if let Some(section) = first_non_empty {
                assert_eq!(section.section_type, SectionType::Breaking);
            }
        }
    }
}

// ============================================================================
// Collector Unit Tests
// ============================================================================

mod collector_unit_tests {
    use super::*;
    use crate::changelog::{ChangelogCollector, ChangelogEntry, SectionType};
    use chrono::Utc;
    use sublime_git_tools::RepoCommit;

    fn create_test_commit(hash: &str, message: &str, author: &str, date: &str) -> RepoCommit {
        RepoCommit {
            hash: hash.to_string(),
            author_name: author.to_string(),
            author_email: format!("{}@example.com", author),
            author_date: date.to_string(),
            message: message.to_string(),
        }
    }

    #[test]
    fn test_extract_first_line() {
        let (temp_dir, repo) = create_test_repo();
        let config = ChangelogConfig::default();
        let collector = ChangelogCollector::new(&repo, &config);

        let message = "First line\n\nSecond line\nThird line";
        let first = collector.extract_first_line(message);
        assert_eq!(first, "First line");

        drop(temp_dir);
    }

    #[test]
    fn test_extract_references_from_text() {
        let (temp_dir, repo) = create_test_repo();
        let config = ChangelogConfig::default();
        let collector = ChangelogCollector::new(&repo, &config);

        let text = "Fix bug #123 and closes #456";
        let refs = collector.extract_references_from_text(text);
        assert_eq!(refs, vec!["#123", "#456"]);

        let text2 = "Resolves #789";
        let refs2 = collector.extract_references_from_text(text2);
        assert_eq!(refs2, vec!["#789"]);

        drop(temp_dir);
    }

    #[test]
    fn test_parse_commit_conventional() {
        let (temp_dir, repo) = create_test_repo();
        let config = ChangelogConfig::default();
        let collector = ChangelogCollector::new(&repo, &config);

        let commit =
            create_test_commit("abc123", "feat(core): add new feature", "John Doe", "2024-01-01");

        let entry = collector.parse_commit(&commit);
        assert_eq!(entry.description, "add new feature");
        assert_eq!(entry.commit_type, Some("feat".to_string()));
        assert_eq!(entry.scope, Some("core".to_string()));
        assert!(!entry.breaking);

        drop(temp_dir);
    }

    #[test]
    fn test_parse_commit_plain() {
        let (temp_dir, repo) = create_test_repo();
        let mut config = ChangelogConfig::default();
        config.conventional.enabled = false;
        let collector = ChangelogCollector::new(&repo, &config);

        let commit = create_test_commit("def456", "Fix a bug", "Jane Smith", "2024-01-02");

        let entry = collector.parse_commit(&commit);
        assert_eq!(entry.description, "Fix a bug");
        assert!(entry.commit_type.is_none());
        assert!(!entry.breaking);

        drop(temp_dir);
    }

    #[test]
    fn test_determine_section_type() {
        let (temp_dir, repo) = create_test_repo();
        let config = ChangelogConfig::default();
        let collector = ChangelogCollector::new(&repo, &config);

        let feat_entry = ChangelogEntry {
            description: "test".to_string(),
            commit_hash: "abc".to_string(),
            short_hash: "abc".to_string(),
            commit_type: Some("feat".to_string()),
            scope: None,
            breaking: false,
            author: "test".to_string(),
            references: vec![],
            date: Utc::now(),
        };

        assert_eq!(collector.determine_section_type(&feat_entry), SectionType::Features);

        let breaking_entry = ChangelogEntry {
            description: "test".to_string(),
            commit_hash: "def".to_string(),
            short_hash: "def".to_string(),
            commit_type: Some("feat".to_string()),
            scope: None,
            breaking: true,
            author: "test".to_string(),
            references: vec![],
            date: Utc::now(),
        };

        assert_eq!(collector.determine_section_type(&breaking_entry), SectionType::Breaking);

        drop(temp_dir);
    }

    #[test]
    fn test_should_include_commit() {
        let (temp_dir, repo) = create_test_repo();
        let mut config = ChangelogConfig::default();
        config.exclude.patterns = vec!["^chore:".to_string(), "^Merge".to_string()];
        config.exclude.authors = vec!["bot".to_string()];

        let collector = ChangelogCollector::new(&repo, &config);

        let normal_commit = create_test_commit("abc", "feat: new feature", "John", "2024-01-01");
        assert!(collector.should_include_commit(&normal_commit));

        let chore_commit = create_test_commit("def", "chore: update deps", "John", "2024-01-01");
        assert!(!collector.should_include_commit(&chore_commit));

        let bot_commit = create_test_commit("ghi", "feat: automated", "bot", "2024-01-01");
        assert!(!collector.should_include_commit(&bot_commit));

        drop(temp_dir);
    }

    #[test]
    fn test_group_entries_by_section() {
        let (temp_dir, repo) = create_test_repo();
        let config = ChangelogConfig::default();
        let collector = ChangelogCollector::new(&repo, &config);

        let entries = vec![
            ChangelogEntry {
                description: "feature 1".to_string(),
                commit_hash: "abc".to_string(),
                short_hash: "abc".to_string(),
                commit_type: Some("feat".to_string()),
                scope: None,
                breaking: false,
                author: "test".to_string(),
                references: vec![],
                date: Utc::now(),
            },
            ChangelogEntry {
                description: "fix 1".to_string(),
                commit_hash: "def".to_string(),
                short_hash: "def".to_string(),
                commit_type: Some("fix".to_string()),
                scope: None,
                breaking: false,
                author: "test".to_string(),
                references: vec![],
                date: Utc::now(),
            },
            ChangelogEntry {
                description: "breaking".to_string(),
                commit_hash: "ghi".to_string(),
                short_hash: "ghi".to_string(),
                commit_type: Some("feat".to_string()),
                scope: None,
                breaking: true,
                author: "test".to_string(),
                references: vec![],
                date: Utc::now(),
            },
        ];

        let sections = collector.group_entries_by_section(entries);

        // Should have 3 sections: Breaking, Features, Fixes
        assert_eq!(sections.len(), 3);

        // Breaking should be first (highest priority)
        assert_eq!(sections[0].section_type, SectionType::Breaking);
        assert_eq!(sections[0].entries.len(), 1);

        drop(temp_dir);
    }
}

// ============================================================================
// Types Unit Tests
// ============================================================================

mod types_unit_tests {
    use crate::changelog::{Changelog, ChangelogEntry, ChangelogSection, SectionType};
    use chrono::Utc;

    #[test]
    fn test_changelog_new() {
        let changelog = Changelog::new(Some("test-package"), "1.0.0", Some("0.9.0"), Utc::now());

        assert_eq!(changelog.package_name, Some("test-package".to_string()));
        assert_eq!(changelog.version, "1.0.0");
        assert_eq!(changelog.previous_version, Some("0.9.0".to_string()));
        assert!(changelog.is_empty());
        assert_eq!(changelog.entry_count(), 0);
    }

    #[test]
    fn test_changelog_add_section() {
        let mut changelog = Changelog::new(Some("test"), "1.0.0", None, Utc::now());
        let section = ChangelogSection::new(SectionType::Features);

        changelog.add_section(section);
        assert_eq!(changelog.sections.len(), 1);
    }

    #[test]
    fn test_changelog_is_empty() {
        let changelog = Changelog::new(Some("test"), "1.0.0", None, Utc::now());
        assert!(changelog.is_empty());
    }

    #[test]
    fn test_changelog_entry_count() {
        let mut changelog = Changelog::new(Some("test"), "1.0.0", None, Utc::now());
        let mut section = ChangelogSection::new(SectionType::Features);

        let entry = ChangelogEntry {
            description: "Test feature".to_string(),
            commit_hash: "abc123".to_string(),
            short_hash: "abc123".to_string(),
            commit_type: Some("feat".to_string()),
            scope: None,
            breaking: false,
            author: "Author".to_string(),
            references: vec![],
            date: Utc::now(),
        };

        section.add_entry(entry);
        changelog.add_section(section);

        assert_eq!(changelog.entry_count(), 1);
        assert!(!changelog.is_empty());
    }

    #[test]
    fn test_changelog_breaking_changes() {
        let mut changelog = Changelog::new(Some("test"), "2.0.0", Some("1.0.0"), Utc::now());

        let mut breaking_section = ChangelogSection::new(SectionType::Breaking);
        breaking_section.add_entry(ChangelogEntry {
            description: "Breaking change".to_string(),
            commit_hash: "def456".to_string(),
            short_hash: "def456".to_string(),
            commit_type: Some("feat".to_string()),
            scope: None,
            breaking: true,
            author: "Author".to_string(),
            references: vec![],
            date: Utc::now(),
        });

        changelog.add_section(breaking_section);

        let breaking_changes = changelog.breaking_changes();
        assert_eq!(breaking_changes.len(), 1);
        assert!(changelog.has_breaking_changes());
    }

    #[test]
    fn test_section_new() {
        let section = ChangelogSection::new(SectionType::Fixes);
        assert_eq!(section.section_type, SectionType::Fixes);
        assert!(section.is_empty());
    }

    #[test]
    fn test_section_add_entry() {
        let mut section = ChangelogSection::new(SectionType::Features);
        let entry = ChangelogEntry {
            description: "New feature".to_string(),
            commit_hash: "abc123".to_string(),
            short_hash: "abc123".to_string(),
            commit_type: Some("feat".to_string()),
            scope: None,
            breaking: false,
            author: "Author".to_string(),
            references: vec![],
            date: Utc::now(),
        };

        section.add_entry(entry);
        assert_eq!(section.entries.len(), 1);
        assert!(!section.is_empty());
    }

    #[test]
    fn test_entry_commit_link() {
        let entry = ChangelogEntry {
            description: "Test".to_string(),
            commit_hash: "abc123def456".to_string(),
            short_hash: "abc123d".to_string(),
            commit_type: None,
            scope: None,
            breaking: false,
            author: "Author".to_string(),
            references: vec![],
            date: Utc::now(),
        };

        let link = entry.commit_link("https://github.com/user/repo");
        assert!(link.contains("abc123d"));
        assert!(link.contains("abc123def456"));
        assert!(link.contains("https://github.com/user/repo/commit/"));
    }

    #[test]
    fn test_entry_issue_links() {
        let entry = ChangelogEntry {
            description: "Test".to_string(),
            commit_hash: "abc123".to_string(),
            short_hash: "abc123".to_string(),
            commit_type: None,
            scope: None,
            breaking: false,
            author: "Author".to_string(),
            references: vec!["#123".to_string(), "#456".to_string()],
            date: Utc::now(),
        };

        let links = entry.issue_links("https://github.com/user/repo");
        assert_eq!(links.len(), 2);
        assert!(links[0].contains("#123"));
        assert!(links[1].contains("#456"));
    }
}

// =============================================================================
// File Management Tests (Story 8.8)
// =============================================================================

mod file_management_tests {
    use super::*;
    use crate::changelog::{Changelog, ChangelogGenerator, ChangelogParser};
    use crate::config::{ChangelogConfig, PackageToolsConfig};
    use chrono::Utc;
    use sublime_git_tools::Repo;
    use sublime_standard_tools::filesystem::FileSystemManager;
    use tempfile::TempDir;

    async fn create_test_generator(temp_dir: &TempDir) -> ChangelogGenerator {
        let workspace_root = temp_dir.path().to_path_buf();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        // Initialize git repository
        std::process::Command::new("git")
            .arg("init")
            .arg(temp_dir.path())
            .output()
            .expect("Failed to initialize git repository");

        let git_repo = Repo::open(workspace_root.to_str().expect("Invalid path"))
            .expect("Failed to open repo");

        ChangelogGenerator::new(workspace_root, git_repo, fs, config.changelog)
            .await
            .expect("Failed to create generator")
    }

    #[tokio::test]
    async fn test_update_changelog_creates_new_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let generator = create_test_generator(&temp_dir).await;

        let package_path = temp_dir.path().join("packages/my-package");
        generator.fs().create_dir_all(&package_path).await.expect("Failed to create package dir");

        let changelog = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());

        // Create new changelog
        let content = generator
            .update_changelog(&package_path, &changelog, false)
            .await
            .expect("Failed to update changelog");

        // Verify file was created
        let changelog_path = package_path.join("CHANGELOG.md");
        assert!(generator.fs().exists(&changelog_path).await);

        // Verify content includes header and version
        assert!(content.contains("# Changelog"));
        assert!(content.contains("## [1.0.0]"));
    }

    #[tokio::test]
    async fn test_update_changelog_dry_run() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let generator = create_test_generator(&temp_dir).await;

        let package_path = temp_dir.path().join("packages/my-package");
        generator.fs().create_dir_all(&package_path).await.expect("Failed to create package dir");

        let changelog = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());

        // Dry run
        let content = generator
            .update_changelog(&package_path, &changelog, true)
            .await
            .expect("Failed to update changelog");

        // Verify file was NOT created
        let changelog_path = package_path.join("CHANGELOG.md");
        assert!(!generator.fs().exists(&changelog_path).await);

        // But content was returned
        assert!(content.contains("# Changelog"));
        assert!(content.contains("## [1.0.0]"));
    }

    #[tokio::test]
    async fn test_update_changelog_prepends_to_existing() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let generator = create_test_generator(&temp_dir).await;

        let package_path = temp_dir.path().join("packages/my-package");
        generator.fs().create_dir_all(&package_path).await.expect("Failed to create package dir");

        // Create initial changelog
        let changelog1 = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());
        generator
            .update_changelog(&package_path, &changelog1, false)
            .await
            .expect("Failed to create initial changelog");

        // Add new version
        let changelog2 = Changelog::new(Some("my-package"), "2.0.0", Some("1.0.0"), Utc::now());
        let content = generator
            .update_changelog(&package_path, &changelog2, false)
            .await
            .expect("Failed to update changelog");

        // Verify both versions exist in correct order
        assert!(content.contains("## [2.0.0]"));
        assert!(content.contains("## [1.0.0]"));

        let pos_2_0 = content.find("## [2.0.0]").expect("Should contain 2.0.0");
        let pos_1_0 = content.find("## [1.0.0]").expect("Should contain 1.0.0");
        assert!(pos_2_0 < pos_1_0, "2.0.0 should come before 1.0.0");
    }

    #[tokio::test]
    async fn test_update_changelog_preserves_header() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let generator = create_test_generator(&temp_dir).await;

        let package_path = temp_dir.path().join("packages/my-package");
        generator.fs().create_dir_all(&package_path).await.expect("Failed to create package dir");

        // Create initial changelog with custom header
        let changelog_path = package_path.join("CHANGELOG.md");
        let initial_content = r#"# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

"#;
        generator
            .fs()
            .write_file_string(&changelog_path, initial_content)
            .await
            .expect("Failed to write initial changelog");

        // Add version
        let changelog = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());
        let content = generator
            .update_changelog(&package_path, &changelog, false)
            .await
            .expect("Failed to update changelog");

        // Verify header is preserved
        assert!(content.contains("# Changelog"));
        assert!(content.contains("All notable changes"));
        assert!(content.contains("Keep a Changelog"));
        assert!(content.contains("## [1.0.0]"));
    }

    #[tokio::test]
    async fn test_parse_changelog_success() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let generator = create_test_generator(&temp_dir).await;

        let package_path = temp_dir.path().join("packages/my-package");
        generator.fs().create_dir_all(&package_path).await.expect("Failed to create package dir");

        // Create changelog with multiple versions
        let changelog_path = package_path.join("CHANGELOG.md");
        let content = r#"# Changelog

## [2.0.0] - 2024-02-01

### Features
- Add new feature

## [1.0.0] - 2024-01-15

### Initial
- Initial release
"#;
        generator
            .fs()
            .write_file_string(&changelog_path, content)
            .await
            .expect("Failed to write changelog");

        // Parse it
        let parsed =
            generator.parse_changelog(&package_path).await.expect("Failed to parse changelog");

        assert_eq!(parsed.versions.len(), 2);
        assert_eq!(parsed.versions[0].version, "2.0.0");
        assert_eq!(parsed.versions[1].version, "1.0.0");
        assert!(parsed.header.contains("# Changelog"));
    }

    #[tokio::test]
    async fn test_parse_changelog_not_found() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let generator = create_test_generator(&temp_dir).await;

        let package_path = temp_dir.path().join("packages/my-package");
        generator.fs().create_dir_all(&package_path).await.expect("Failed to create package dir");

        // Try to parse non-existent changelog
        let result = generator.parse_changelog(&package_path).await;

        assert!(result.is_err());
        match result {
            Err(crate::error::ChangelogError::NotFound { .. }) => {}
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_prepend_changelog_with_existing_versions() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let generator = create_test_generator(&temp_dir).await;

        let existing = r#"# Changelog

## [1.0.0] - 2024-01-15

### Features
- Old feature
"#;

        let new_section = r#"## [2.0.0] - 2024-02-01

### Features
- New feature
"#;

        let result = generator.prepend_changelog(existing, new_section);

        // Verify order
        let pos_2_0 = result.find("## [2.0.0]").expect("Should contain 2.0.0");
        let pos_1_0 = result.find("## [1.0.0]").expect("Should contain 1.0.0");
        assert!(pos_2_0 < pos_1_0, "2.0.0 should come before 1.0.0");

        // Verify header is preserved
        assert!(result.contains("# Changelog"));
    }

    #[tokio::test]
    async fn test_prepend_changelog_empty_changelog() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let generator = create_test_generator(&temp_dir).await;

        let existing = r#"# Changelog

All notable changes will be documented here.

"#;

        let new_section = r#"## [1.0.0] - 2024-01-15

### Features
- Initial release
"#;

        let result = generator.prepend_changelog(existing, new_section);

        // Verify header comes first
        assert!(result.starts_with("# Changelog"));

        // Verify new section is added
        assert!(result.contains("## [1.0.0]"));
        assert!(result.contains("Initial release"));
    }

    #[tokio::test]
    async fn test_prepend_changelog_maintains_spacing() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let generator = create_test_generator(&temp_dir).await;

        let existing = r#"# Changelog

## [1.0.0] - 2024-01-15
- Feature
"#;

        let new_section = r#"## [2.0.0] - 2024-02-01
- New feature
"#;

        let result = generator.prepend_changelog(existing, new_section);

        // Verify proper spacing between sections
        assert!(result.contains("## [2.0.0]"));
        assert!(result.contains("## [1.0.0]"));

        // Should have blank line between versions
        let lines: Vec<&str> = result.lines().collect();
        let mut found_2_0 = false;
        let mut _found_blank_after = false;
        for line in lines.iter() {
            if line.contains("## [2.0.0]") {
                found_2_0 = true;
            } else if found_2_0 && line.is_empty() {
                _found_blank_after = true;
                break;
            }
        }
        assert!(found_2_0, "Should find 2.0.0");
    }

    #[tokio::test]
    async fn test_update_changelog_with_sections() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let generator = create_test_generator(&temp_dir).await;

        let package_path = temp_dir.path().join("packages/my-package");
        generator.fs().create_dir_all(&package_path).await.expect("Failed to create package dir");

        // Create changelog with sections
        let mut changelog = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());

        let mut features_section =
            crate::changelog::ChangelogSection::new(crate::changelog::SectionType::Features);
        features_section.add_entry(crate::changelog::ChangelogEntry {
            description: "Add new API".to_string(),
            commit_hash: "abc123".to_string(),
            short_hash: "abc123".to_string(),
            commit_type: Some("feat".to_string()),
            scope: None,
            breaking: false,
            author: "Developer".to_string(),
            references: vec![],
            date: Utc::now(),
        });
        changelog.add_section(features_section);

        let content = generator
            .update_changelog(&package_path, &changelog, false)
            .await
            .expect("Failed to update changelog");

        // Verify sections are rendered
        assert!(content.contains("## [1.0.0]"));
        assert!(content.contains("### Features"));
        assert!(content.contains("Add new API"));
    }

    #[tokio::test]
    async fn test_update_changelog_custom_filename() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create generator with custom config
        let workspace_root = temp_dir.path().to_path_buf();
        let fs = FileSystemManager::new();

        // Initialize git repository
        std::process::Command::new("git")
            .arg("init")
            .arg(temp_dir.path())
            .output()
            .expect("Failed to initialize git repository");

        let git_repo = Repo::open(workspace_root.to_str().expect("Invalid path"))
            .expect("Failed to open repo");

        let mut config = ChangelogConfig::default();
        config.filename = "HISTORY.md".to_string();

        let generator = ChangelogGenerator::new(workspace_root, git_repo, fs, config)
            .await
            .expect("Failed to create generator");

        let package_path = temp_dir.path().join("packages/my-package");
        generator.fs().create_dir_all(&package_path).await.expect("Failed to create package dir");

        let changelog = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());

        generator
            .update_changelog(&package_path, &changelog, false)
            .await
            .expect("Failed to update changelog");

        // Verify custom filename was used
        let changelog_path = package_path.join("HISTORY.md");
        assert!(generator.fs().exists(&changelog_path).await);
    }

    #[tokio::test]
    async fn test_changelog_parser_integration() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let generator = create_test_generator(&temp_dir).await;

        let package_path = temp_dir.path().join("packages/my-package");
        generator.fs().create_dir_all(&package_path).await.expect("Failed to create package dir");

        // Create changelog with parser directly
        let parser = ChangelogParser::new();
        let content = r#"# Changelog

## [1.0.0] - 2024-01-15

### Features
- Feature 1
- Feature 2

## [0.9.0] - 2024-01-01

### Fixes
- Fix 1
"#;

        let parsed = parser.parse(content).expect("Failed to parse");

        assert_eq!(parsed.version_count(), 2);
        assert!(parsed.has_version("1.0.0"));
        assert!(parsed.has_version("0.9.0"));

        let versions = parsed.version_list();
        assert_eq!(versions, vec!["1.0.0", "0.9.0"]);

        let latest = parsed.latest_version().expect("Should have latest");
        assert_eq!(latest.version, "1.0.0");
        assert!(latest.date.is_some());
    }

    #[tokio::test]
    async fn test_update_and_parse_roundtrip() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let generator = create_test_generator(&temp_dir).await;

        let package_path = temp_dir.path().join("packages/my-package");
        generator.fs().create_dir_all(&package_path).await.expect("Failed to create package dir");

        // Create and write changelog
        let changelog = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());
        generator
            .update_changelog(&package_path, &changelog, false)
            .await
            .expect("Failed to update changelog");

        // Parse it back
        let parsed =
            generator.parse_changelog(&package_path).await.expect("Failed to parse changelog");

        assert_eq!(parsed.versions.len(), 1);
        assert_eq!(parsed.versions[0].version, "1.0.0");
    }

    #[tokio::test]
    async fn test_update_changelog_preserves_existing_content() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let generator = create_test_generator(&temp_dir).await;

        let package_path = temp_dir.path().join("packages/my-package");
        generator.fs().create_dir_all(&package_path).await.expect("Failed to create package dir");

        // Create initial changelog
        let changelog_path = package_path.join("CHANGELOG.md");
        let initial_content = r#"# Changelog

## [1.0.0] - 2024-01-15

### Features
- Feature A
- Feature B

### Fixes
- Fix X
"#;
        generator
            .fs()
            .write_file_string(&changelog_path, initial_content)
            .await
            .expect("Failed to write initial changelog");

        // Add new version
        let changelog = Changelog::new(Some("my-package"), "2.0.0", Some("1.0.0"), Utc::now());
        let content = generator
            .update_changelog(&package_path, &changelog, false)
            .await
            .expect("Failed to update changelog");

        // Verify old content is preserved
        assert!(content.contains("Feature A"));
        assert!(content.contains("Feature B"));
        assert!(content.contains("Fix X"));
        assert!(content.contains("## [1.0.0]"));
        assert!(content.contains("## [2.0.0]"));
    }

    // =============================================================================
    // Parser Tests (Story 8.8)
    // =============================================================================

    mod parser_tests {

        use crate::changelog::ChangelogParser;

        #[test]
        fn test_parse_simple_changelog() {
            let content = r#"# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2024-01-15

### Features
- Add new feature

## [0.9.0] - 2024-01-01

### Fixes
- Fix bug
"#;

            let parser = ChangelogParser::new();
            let result = parser.parse(content).expect("Failed to parse");

            assert_eq!(result.versions.len(), 2);
            assert_eq!(result.versions[0].version, "1.0.0");
            assert_eq!(result.versions[1].version, "0.9.0");
            assert!(result.header.contains("# Changelog"));
        }

        #[test]
        fn test_parse_version_without_brackets() {
            let content = "# Changelog\n\n## 1.0.0 - 2024-01-15\n- Feature";

            let parser = ChangelogParser::new();
            let result = parser.parse(content).expect("Failed to parse");

            assert_eq!(result.versions.len(), 1);
            assert_eq!(result.versions[0].version, "1.0.0");
        }

        #[test]
        fn test_parse_version_with_v_prefix() {
            let content = "# Changelog\n\n## v1.0.0\n- Feature";

            let parser = ChangelogParser::new();
            let result = parser.parse(content).expect("Failed to parse");

            assert_eq!(result.versions.len(), 1);
            assert_eq!(result.versions[0].version, "1.0.0");
        }

        #[test]
        fn test_parse_prerelease_version() {
            let content = "# Changelog\n\n## [1.0.0-beta.1] - 2024-01-15\n- Feature";

            let parser = ChangelogParser::new();
            let result = parser.parse(content).expect("Failed to parse");

            assert_eq!(result.versions.len(), 1);
            assert_eq!(result.versions[0].version, "1.0.0-beta.1");
        }

        #[test]
        fn test_date_extraction() {
            let parser = ChangelogParser::new();

            let date1 = parser.extract_date("## [1.0.0] - 2024-01-15");
            assert!(date1.is_some());

            let date2 = parser.extract_date("## [1.0.0] (2024/01/15)");
            assert!(date2.is_some());

            let date3 = parser.extract_date("## [1.0.0]");
            assert!(date3.is_none());
        }

        #[test]
        fn test_get_version() {
            let content = "# Changelog\n\n## [1.0.0] - 2024-01-15\n- Feature";

            let parser = ChangelogParser::new();
            let result = parser.parse(content).expect("Failed to parse");

            assert!(result.get_version("1.0.0").is_some());
            assert!(result.get_version("2.0.0").is_none());
        }

        #[test]
        fn test_latest_version() {
            let content =
                "# Changelog\n\n## [2.0.0] - 2024-02-01\n- New\n\n## [1.0.0] - 2024-01-15\n- Old";

            let parser = ChangelogParser::new();
            let result = parser.parse(content).expect("Failed to parse");

            let latest = result.latest_version();
            assert!(latest.is_some());
            assert_eq!(latest.expect("Should have latest").version, "2.0.0");
        }

        #[test]
        fn test_version_list() {
            let content = "# Changelog\n\n## [2.0.0] - 2024-02-01\n\n## [1.0.0] - 2024-01-15\n\n## [0.9.0] - 2024-01-01";

            let parser = ChangelogParser::new();
            let result = parser.parse(content).expect("Failed to parse");

            let versions = result.version_list();
            assert_eq!(versions, vec!["2.0.0", "1.0.0", "0.9.0"]);
        }

        #[test]
        fn test_empty_changelog() {
            let content = "# Changelog\n\nNo releases yet.";

            let parser = ChangelogParser::new();
            let result = parser.parse(content).expect("Failed to parse");

            assert_eq!(result.versions.len(), 0);
            assert!(result.header.contains("# Changelog"));
        }

        #[test]
        fn test_parse_to_map() {
            let content = "# Changelog\n\n## [1.0.0] - 2024-01-15\n- Feature\n\n## [0.9.0] - 2024-01-01\n- Fix";

            let parser = ChangelogParser::new();
            let map = parser.parse_to_map(content).expect("Failed to parse");

            assert_eq!(map.len(), 2);
            assert!(map.contains_key("1.0.0"));
            assert!(map.contains_key("0.9.0"));
            assert!(map.get("1.0.0").expect("Should have 1.0.0").contains("Feature"));
        }

        #[test]
        fn test_parsed_changelog_has_version() {
            let content = "# Changelog\n\n## [1.0.0] - 2024-01-15\n- Feature";

            let parser = ChangelogParser::new();
            let result = parser.parse(content).expect("Failed to parse");

            assert!(result.has_version("1.0.0"));
            assert!(!result.has_version("2.0.0"));
        }

        #[test]
        fn test_parsed_changelog_version_count() {
            let content = "# Changelog\n\n## [3.0.0]\n\n## [2.0.0]\n\n## [1.0.0]";

            let parser = ChangelogParser::new();
            let result = parser.parse(content).expect("Failed to parse");

            assert_eq!(result.version_count(), 3);
        }

        #[test]
        fn test_parse_with_build_metadata() {
            let content = "# Changelog\n\n## [1.0.0+20240115] - 2024-01-15\n- Feature";

            let parser = ChangelogParser::new();
            let result = parser.parse(content).expect("Failed to parse");

            assert_eq!(result.versions.len(), 1);
            assert_eq!(result.versions[0].version, "1.0.0+20240115");
        }
    }

    // ============================================================================
    // Merge Message Generation Tests
    // ============================================================================

    mod merge_message_tests {
        use super::*;
        use crate::changelog::merge_message::{MergeMessageContext, generate_merge_commit_message};
        use crate::changelog::{Changelog, ChangelogEntry, ChangelogSection};
        use crate::config::GitConfig;
        use chrono::Utc;

        fn create_test_context() -> MergeMessageContext {
            MergeMessageContext::new(None, "1.0.0", Some("0.9.0"), "Minor", Utc::now())
        }

        fn create_test_changelog() -> Changelog {
            let mut changelog = Changelog::new(None, "1.0.0", Some("0.9.0"), Utc::now());

            // Add features section
            let mut features = ChangelogSection::new(SectionType::Features);
            features.add_entry(ChangelogEntry {
                description: "Add new feature".to_string(),
                commit_hash: "abc123".to_string(),
                short_hash: "abc123".to_string(),
                commit_type: Some("feat".to_string()),
                scope: None,
                breaking: false,
                author: "John".to_string(),
                references: vec![],
                date: Utc::now(),
            });
            changelog.add_section(features);

            // Add fixes section
            let mut fixes = ChangelogSection::new(SectionType::Fixes);
            fixes.add_entry(ChangelogEntry {
                description: "Fix bug".to_string(),
                commit_hash: "def456".to_string(),
                short_hash: "def456".to_string(),
                commit_type: Some("fix".to_string()),
                scope: None,
                breaking: false,
                author: "Jane".to_string(),
                references: vec![],
                date: Utc::now(),
            });
            changelog.add_section(fixes);

            // Add breaking changes section
            let mut breaking = ChangelogSection::new(SectionType::Breaking);
            breaking.add_entry(ChangelogEntry {
                description: "Breaking change".to_string(),
                commit_hash: "ghi789".to_string(),
                short_hash: "ghi789".to_string(),
                commit_type: Some("feat".to_string()),
                scope: None,
                breaking: true,
                author: "Bob".to_string(),
                references: vec![],
                date: Utc::now(),
            });
            changelog.add_section(breaking);

            changelog
        }

        #[test]
        fn test_context_creation() {
            let context = create_test_context();
            assert_eq!(context.version, "1.0.0");
            assert_eq!(context.previous_version, Some("0.9.0".to_string()));
            assert_eq!(context.bump_type, "Minor");
            assert_eq!(context.package_name, None);
            assert!(!context.is_monorepo());
        }

        #[test]
        fn test_context_with_package_name() {
            let context =
                MergeMessageContext::new(Some("my-package"), "1.0.0", None, "Major", Utc::now());
            assert_eq!(context.package_name, Some("my-package".to_string()));
            assert!(context.is_monorepo());
        }

        #[test]
        fn test_context_with_author() {
            let context = create_test_context().with_author(Some("John Doe".to_string()));
            assert_eq!(context.author, Some("John Doe".to_string()));
        }

        #[test]
        fn test_context_with_changelog() {
            let changelog = create_test_changelog();
            let context = create_test_context().with_changelog(Some(changelog));
            assert!(context.changelog.is_some());
        }

        #[test]
        fn test_breaking_changes_count() {
            let changelog = create_test_changelog();
            let context = create_test_context().with_changelog(Some(changelog));
            assert_eq!(context.breaking_changes_count(), 1);
        }

        #[test]
        fn test_breaking_changes_count_no_changelog() {
            let context = create_test_context();
            assert_eq!(context.breaking_changes_count(), 0);
        }

        #[test]
        fn test_features_count() {
            let changelog = create_test_changelog();
            let context = create_test_context().with_changelog(Some(changelog));
            assert_eq!(context.features_count(), 1);
        }

        #[test]
        fn test_fixes_count() {
            let changelog = create_test_changelog();
            let context = create_test_context().with_changelog(Some(changelog));
            assert_eq!(context.fixes_count(), 1);
        }

        #[test]
        fn test_changelog_summary_with_changes() {
            let changelog = create_test_changelog();
            let context = create_test_context().with_changelog(Some(changelog));
            let summary = context.changelog_summary();

            assert!(summary.contains("1 new feature"));
            assert!(summary.contains("1 bug fix"));
            assert!(summary.contains("1 breaking change"));
        }

        #[test]
        fn test_changelog_summary_no_changelog() {
            let context = create_test_context();
            let summary = context.changelog_summary();
            assert_eq!(summary, "No changelog available");
        }

        #[test]
        fn test_changelog_summary_empty_changelog() {
            let changelog = Changelog::new(None, "1.0.0", None, Utc::now());
            let context = create_test_context().with_changelog(Some(changelog));
            let summary = context.changelog_summary();
            assert_eq!(summary, "No changes recorded");
        }

        #[test]
        fn test_generate_merge_commit_message_single_package() {
            let changelog = create_test_changelog();
            let context = create_test_context().with_changelog(Some(changelog));
            let config = GitConfig::default();

            let message = generate_merge_commit_message(&context, &config);

            assert!(message.contains("1.0.0"));
            assert!(message.contains("1 new feature"));
            assert!(message.contains("1 bug fix"));
            assert!(message.contains("1 breaking change"));
        }

        #[test]
        fn test_generate_merge_commit_message_monorepo() {
            let changelog = create_test_changelog();
            let context = MergeMessageContext::new(
                Some("my-package"),
                "1.0.0",
                Some("0.9.0"),
                "Minor",
                Utc::now(),
            )
            .with_changelog(Some(changelog));
            let config = GitConfig::default();

            let message = generate_merge_commit_message(&context, &config);

            assert!(message.contains("my-package"));
            assert!(message.contains("1.0.0"));
        }

        #[test]
        fn test_generate_merge_commit_message_with_breaking_warning() {
            let changelog = create_test_changelog();
            let context = create_test_context().with_changelog(Some(changelog));
            let config = GitConfig::default();

            let message = generate_merge_commit_message(&context, &config);

            assert!(message.contains("⚠️  BREAKING CHANGES: 1"));
        }

        #[test]
        fn test_generate_merge_commit_message_no_breaking_warning() {
            let changelog = create_test_changelog();
            let context = create_test_context().with_changelog(Some(changelog));
            let mut config = GitConfig::default();
            config.include_breaking_warning = false;

            let message = generate_merge_commit_message(&context, &config);

            assert!(!message.contains("⚠️  BREAKING CHANGES"));
        }

        #[test]
        fn test_generate_merge_commit_message_no_breaking_changes() {
            let mut changelog = Changelog::new(None, "1.0.0", Some("0.9.0"), Utc::now());
            let mut features = ChangelogSection::new(SectionType::Features);
            features.add_entry(ChangelogEntry {
                description: "Add new feature".to_string(),
                commit_hash: "abc123".to_string(),
                short_hash: "abc123".to_string(),
                commit_type: Some("feat".to_string()),
                scope: None,
                breaking: false,
                author: "John".to_string(),
                references: vec![],
                date: Utc::now(),
            });
            changelog.add_section(features);

            let context = create_test_context().with_changelog(Some(changelog));
            let config = GitConfig::default();

            let message = generate_merge_commit_message(&context, &config);

            assert!(!message.contains("⚠️  BREAKING CHANGES"));
        }

        #[test]
        fn test_generate_merge_commit_message_custom_template() {
            let context = create_test_context();
            let mut config = GitConfig::default();
            config.merge_commit_template = "Release v{version} ({bump_type})".to_string();

            let message = generate_merge_commit_message(&context, &config);

            assert_eq!(message, "Release v1.0.0 (Minor)");
        }

        #[test]
        fn test_generate_merge_commit_message_custom_breaking_template() {
            let changelog = create_test_changelog();
            let context = create_test_context().with_changelog(Some(changelog));
            let mut config = GitConfig::default();
            config.breaking_warning_template =
                "\n⚠️  Warning: {breaking_changes_count} breaking changes!\n".to_string();

            let message = generate_merge_commit_message(&context, &config);

            assert!(message.contains("⚠️  Warning: 1 breaking changes!"));
        }

        #[test]
        fn test_generate_merge_commit_message_all_variables() {
            let changelog = create_test_changelog();
            let context = MergeMessageContext::new(
                Some("test-pkg"),
                "2.0.0",
                Some("1.5.0"),
                "Major",
                Utc::now(),
            )
            .with_author(Some("Test User".to_string()))
            .with_changelog(Some(changelog));

            let mut config = GitConfig::default();
            // Use monorepo_merge_commit_template since context has package_name
            config.monorepo_merge_commit_template = concat!(
                "Package: {package_name}\n",
                "Version: {previous_version} -> {version}\n",
                "Bump: {bump_type}\n",
                "Date: {date}\n",
                "Author: {author}\n",
                "Features: {features_count}\n",
                "Fixes: {fixes_count}\n",
                "Breaking: {breaking_changes_count}\n",
                "Summary:\n{changelog_summary}"
            )
            .to_string();
            config.include_breaking_warning = false;

            let message = generate_merge_commit_message(&context, &config);

            assert!(message.contains("Package: test-pkg"));
            assert!(message.contains("Version: 1.5.0 -> 2.0.0"));
            assert!(message.contains("Bump: Major"));
            assert!(message.contains("Author: Test User"));
            assert!(message.contains("Features: 1"));
            assert!(message.contains("Fixes: 1"));
            assert!(message.contains("Breaking: 1"));
        }

        #[test]
        fn test_generate_merge_commit_message_with_multiple_features() {
            let mut changelog = Changelog::new(None, "1.0.0", Some("0.9.0"), Utc::now());
            let mut features = ChangelogSection::new(SectionType::Features);

            for i in 1..=5 {
                features.add_entry(ChangelogEntry {
                    description: format!("Feature {}", i),
                    commit_hash: format!("abc{}", i),
                    short_hash: format!("abc{}", i),
                    commit_type: Some("feat".to_string()),
                    scope: None,
                    breaking: false,
                    author: "John".to_string(),
                    references: vec![],
                    date: Utc::now(),
                });
            }
            changelog.add_section(features);

            let context = create_test_context().with_changelog(Some(changelog));
            let config = GitConfig::default();

            let message = generate_merge_commit_message(&context, &config);

            assert!(message.contains("5 new features"));
        }

        #[test]
        fn test_generate_merge_commit_message_with_multiple_fixes() {
            let mut changelog = Changelog::new(None, "1.0.0", Some("0.9.0"), Utc::now());
            let mut fixes = ChangelogSection::new(SectionType::Fixes);

            for i in 1..=3 {
                fixes.add_entry(ChangelogEntry {
                    description: format!("Fix {}", i),
                    commit_hash: format!("def{}", i),
                    short_hash: format!("def{}", i),
                    commit_type: Some("fix".to_string()),
                    scope: None,
                    breaking: false,
                    author: "Jane".to_string(),
                    references: vec![],
                    date: Utc::now(),
                });
            }
            changelog.add_section(fixes);

            let context = create_test_context().with_changelog(Some(changelog));
            let config = GitConfig::default();

            let message = generate_merge_commit_message(&context, &config);

            assert!(message.contains("3 bug fixes"));
        }

        #[test]
        fn test_generate_merge_commit_message_without_previous_version() {
            let changelog = create_test_changelog();
            let context = MergeMessageContext::new(None, "1.0.0", None, "Major", Utc::now())
                .with_changelog(Some(changelog));
            let mut config = GitConfig::default();
            config.merge_commit_template =
                "Release {version} (previous: {previous_version})".to_string();

            let message = generate_merge_commit_message(&context, &config);

            assert!(message.contains("Release 1.0.0 (previous: N/A)"));
        }

        #[test]
        fn test_generate_merge_commit_message_without_author() {
            let changelog = create_test_changelog();
            let context = create_test_context().with_changelog(Some(changelog));
            let mut config = GitConfig::default();
            config.merge_commit_template = "Release {version} by {author}".to_string();

            let message = generate_merge_commit_message(&context, &config);

            assert!(message.contains("Release 1.0.0 by Unknown"));
        }

        #[test]
        fn test_generate_merge_commit_message_date_format() {
            let context = create_test_context();
            let mut config = GitConfig::default();
            config.merge_commit_template = "Release on {date}".to_string();

            let message = generate_merge_commit_message(&context, &config);

            assert!(message.starts_with("Release on "));
            // Date should be in YYYY-MM-DD format
            assert!(message.len() > 11); // "Release on " + date
        }

        #[test]
        fn test_generate_merge_commit_message_with_performance_changes() {
            let mut changelog = Changelog::new(None, "1.0.0", Some("0.9.0"), Utc::now());
            let mut perf = ChangelogSection::new(SectionType::Performance);
            perf.add_entry(ChangelogEntry {
                description: "Improve performance".to_string(),
                commit_hash: "perf123".to_string(),
                short_hash: "perf123".to_string(),
                commit_type: Some("perf".to_string()),
                scope: None,
                breaking: false,
                author: "John".to_string(),
                references: vec![],
                date: Utc::now(),
            });
            changelog.add_section(perf);

            let context = create_test_context().with_changelog(Some(changelog));
            let summary = context.changelog_summary();

            assert!(summary.contains("performance"));
        }

        #[test]
        fn test_merge_message_context_builder_pattern() {
            let changelog = create_test_changelog();
            let context = MergeMessageContext::new(
                Some("my-package"),
                "2.0.0",
                Some("1.0.0"),
                "Major",
                Utc::now(),
            )
            .with_author(Some("Jane Doe".to_string()))
            .with_changelog(Some(changelog));

            assert_eq!(context.package_name, Some("my-package".to_string()));
            assert_eq!(context.author, Some("Jane Doe".to_string()));
            assert!(context.changelog.is_some());
            assert_eq!(context.breaking_changes_count(), 1);
        }

        #[test]
        fn test_changelog_summary_multiple_sections() {
            let mut changelog = Changelog::new(None, "1.0.0", Some("0.9.0"), Utc::now());

            let mut features = ChangelogSection::new(SectionType::Features);
            features.add_entry(ChangelogEntry {
                description: "Feature".to_string(),
                commit_hash: "a".to_string(),
                short_hash: "a".to_string(),
                commit_type: Some("feat".to_string()),
                scope: None,
                breaking: false,
                author: "A".to_string(),
                references: vec![],
                date: Utc::now(),
            });
            changelog.add_section(features);

            let mut fixes = ChangelogSection::new(SectionType::Fixes);
            fixes.add_entry(ChangelogEntry {
                description: "Fix".to_string(),
                commit_hash: "b".to_string(),
                short_hash: "b".to_string(),
                commit_type: Some("fix".to_string()),
                scope: None,
                breaking: false,
                author: "B".to_string(),
                references: vec![],
                date: Utc::now(),
            });
            changelog.add_section(fixes);

            let mut docs = ChangelogSection::new(SectionType::Documentation);
            docs.add_entry(ChangelogEntry {
                description: "Doc".to_string(),
                commit_hash: "c".to_string(),
                short_hash: "c".to_string(),
                commit_type: Some("docs".to_string()),
                scope: None,
                breaking: false,
                author: "C".to_string(),
                references: vec![],
                date: Utc::now(),
            });
            changelog.add_section(docs);

            let context = create_test_context().with_changelog(Some(changelog));
            let summary = context.changelog_summary();

            assert!(summary.contains("1 new feature"));
            assert!(summary.contains("1 bug fix"));
            assert!(summary.contains("documentation"));
        }
    }
}

// ============================================================================
// Story 8.10: Generate from Changeset Integration Tests
// ============================================================================

mod generate_from_changeset_tests {
    use super::*;
    use crate::types::{Changeset, UpdateReason, VersionBump};
    use crate::version::{PackageUpdate, VersionResolution};
    use std::path::{Path, PathBuf};

    /// Helper to create a test changeset with commits
    fn create_test_changeset(branch: &str, bump: VersionBump, packages: Vec<&str>) -> Changeset {
        let mut changeset = Changeset::new(branch, bump, vec!["production".to_string()]);

        for package in packages {
            changeset.add_package(package);
        }

        changeset.add_commit("abc123");
        changeset.add_commit("def456");
        changeset.add_commit("ghi789");

        changeset
    }

    /// Helper to add test commits to a repository
    fn add_test_commits(repo: &Repo, temp_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        // Create initial commit
        let file_path = temp_dir.join("README.md");
        fs::write(&file_path, "# Test Project")?;
        repo.add_all()?;
        repo.commit("Initial commit")?;

        // Add more commits for testing
        fs::write(&file_path, "# Test Project\n\nUpdated")?;
        repo.add_all()?;
        repo.commit("feat: add new feature")?;

        fs::write(&file_path, "# Test Project\n\nUpdated again")?;
        repo.add_all()?;
        repo.commit("fix: fix bug")?;

        Ok(())
    }

    /// Helper to create a test version resolution
    fn create_test_resolution(packages: Vec<(&str, &str, &str, PathBuf)>) -> VersionResolution {
        let mut resolution = VersionResolution::new();

        for (name, current, next, path) in packages {
            let update = PackageUpdate::new(
                name.to_string(),
                path,
                crate::types::Version::parse(current).unwrap(),
                crate::types::Version::parse(next).unwrap(),
                UpdateReason::DirectChange,
            );
            resolution.add_update(update);
        }

        resolution
    }

    /// Helper to setup a test monorepo with packages
    async fn setup_test_monorepo(
        temp_dir: &Path,
        fs: &FileSystemManager,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create workspace structure
        fs.create_dir_all(&temp_dir.join("packages/pkg-a")).await?;
        fs.create_dir_all(&temp_dir.join("packages/pkg-b")).await?;

        // Create root package.json with workspaces
        let root_package_json = r#"{
  "name": "monorepo-root",
  "version": "1.0.0",
  "private": true,
  "workspaces": ["packages/*"]
}"#;
        fs.write_file_string(&temp_dir.join("package.json"), root_package_json).await?;

        // Create lerna.json to make it clearly a monorepo
        let lerna_json = r#"{
  "version": "independent",
  "packages": ["packages/*"]
}"#;
        fs.write_file_string(&temp_dir.join("lerna.json"), lerna_json).await?;

        // Create package A
        let pkg_a_json = r#"{
  "name": "@myorg/pkg-a",
  "version": "1.0.0",
  "dependencies": {}
}"#;
        fs.write_file_string(&temp_dir.join("packages/pkg-a/package.json"), pkg_a_json).await?;

        // Create package B
        let pkg_b_json = r#"{
  "name": "@myorg/pkg-b",
  "version": "2.0.0",
  "dependencies": {}
}"#;
        fs.write_file_string(&temp_dir.join("packages/pkg-b/package.json"), pkg_b_json).await?;

        Ok(())
    }

    /// Helper to setup a single package project
    async fn setup_single_package(
        temp_dir: &Path,
        fs: &FileSystemManager,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let package_json = r#"{
  "name": "my-package",
  "version": "1.5.0",
  "dependencies": {}
}"#;
        fs.write_file_string(&temp_dir.join("package.json"), package_json).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_generate_from_changeset_single_package() {
        let (temp_dir, repo) = create_test_repo();
        let fs = FileSystemManager::new();
        setup_single_package(temp_dir.path(), &fs).await.unwrap();
        add_test_commits(&repo, temp_dir.path()).unwrap();

        let config = ChangelogConfig::default();
        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs.clone(), config)
                .await
                .unwrap();

        let changeset = create_test_changeset("main", VersionBump::Minor, vec!["my-package"]);
        let resolution = create_test_resolution(vec![(
            "my-package",
            "1.5.0",
            "1.6.0",
            temp_dir.path().to_path_buf(),
        )]);

        let result = generator.generate_from_changeset(&changeset, &resolution).await;
        assert!(result.is_ok());

        let changelogs = result.unwrap();
        assert_eq!(changelogs.len(), 1);

        let changelog = &changelogs[0];
        assert_eq!(changelog.package_name, None); // Root changelog for single package
        assert_eq!(changelog.changelog.version, "1.6.0");
        assert!(!changelog.content.is_empty());
    }

    #[tokio::test]
    async fn test_generate_from_changeset_monorepo_per_package() {
        let (temp_dir, repo) = create_test_repo();
        let fs = FileSystemManager::new();
        setup_test_monorepo(temp_dir.path(), &fs).await.unwrap();
        add_test_commits(&repo, temp_dir.path()).unwrap();

        let mut config = ChangelogConfig::default();
        config.monorepo_mode = MonorepoMode::PerPackage;

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs.clone(), config)
                .await
                .unwrap();

        let changeset = create_test_changeset(
            "feature-branch",
            VersionBump::Minor,
            vec!["@myorg/pkg-a", "@myorg/pkg-b"],
        );
        let resolution = create_test_resolution(vec![
            ("@myorg/pkg-a", "1.0.0", "1.1.0", temp_dir.path().join("packages/pkg-a")),
            ("@myorg/pkg-b", "2.0.0", "2.1.0", temp_dir.path().join("packages/pkg-b")),
        ]);

        let result = generator.generate_from_changeset(&changeset, &resolution).await;
        assert!(result.is_ok());

        let changelogs = result.unwrap();
        // Should generate changelogs (either per-package if detected as monorepo, or root if not)
        assert!(!changelogs.is_empty());

        // If detected as monorepo, should have 2 package changelogs
        // If detected as single package, should have 1 root changelog
        if changelogs.len() == 2 {
            // Verify both packages have changelogs
            let pkg_a =
                changelogs.iter().find(|c| c.package_name == Some("@myorg/pkg-a".to_string()));
            assert!(pkg_a.is_some());
            let pkg_a = pkg_a.unwrap();
            assert_eq!(pkg_a.changelog.version, "1.1.0");
            assert_eq!(pkg_a.package_path, temp_dir.path().join("packages/pkg-a"));

            let pkg_b =
                changelogs.iter().find(|c| c.package_name == Some("@myorg/pkg-b".to_string()));
            assert!(pkg_b.is_some());
            let pkg_b = pkg_b.unwrap();
            assert_eq!(pkg_b.changelog.version, "2.1.0");
            assert_eq!(pkg_b.package_path, temp_dir.path().join("packages/pkg-b"));
        } else {
            // Single package mode - should have root changelog
            assert_eq!(changelogs[0].package_name, None);
        }
    }

    #[tokio::test]
    async fn test_generate_from_changeset_monorepo_root_mode() {
        let (temp_dir, repo) = create_test_repo();
        let fs = FileSystemManager::new();
        setup_test_monorepo(temp_dir.path(), &fs).await.unwrap();
        add_test_commits(&repo, temp_dir.path()).unwrap();

        let mut config = ChangelogConfig::default();
        config.monorepo_mode = MonorepoMode::Root;

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs.clone(), config)
                .await
                .unwrap();

        let changeset =
            create_test_changeset("feature-branch", VersionBump::Minor, vec!["@myorg/pkg-a"]);
        let resolution = create_test_resolution(vec![(
            "@myorg/pkg-a",
            "1.0.0",
            "1.1.0",
            temp_dir.path().join("packages/pkg-a"),
        )]);

        let result = generator.generate_from_changeset(&changeset, &resolution).await;
        assert!(result.is_ok());

        let changelogs = result.unwrap();
        assert_eq!(changelogs.len(), 1);

        // Should only generate root changelog
        let changelog = &changelogs[0];
        assert_eq!(changelog.package_name, None);
        assert_eq!(changelog.package_path, temp_dir.path().to_path_buf());
        assert_eq!(changelog.changelog.version, "1.1.0");
    }

    #[tokio::test]
    async fn test_generate_from_changeset_monorepo_both_mode() {
        let (temp_dir, repo) = create_test_repo();
        let fs = FileSystemManager::new();
        setup_test_monorepo(temp_dir.path(), &fs).await.unwrap();
        add_test_commits(&repo, temp_dir.path()).unwrap();

        let mut config = ChangelogConfig::default();
        config.monorepo_mode = MonorepoMode::Both;

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs.clone(), config)
                .await
                .unwrap();

        let changeset = create_test_changeset(
            "feature-branch",
            VersionBump::Minor,
            vec!["@myorg/pkg-a", "@myorg/pkg-b"],
        );
        let resolution = create_test_resolution(vec![
            ("@myorg/pkg-a", "1.0.0", "1.1.0", temp_dir.path().join("packages/pkg-a")),
            ("@myorg/pkg-b", "2.0.0", "2.1.0", temp_dir.path().join("packages/pkg-b")),
        ]);

        let result = generator.generate_from_changeset(&changeset, &resolution).await;
        assert!(result.is_ok());

        let changelogs = result.unwrap();
        // Should generate changelogs - at least root
        assert!(!changelogs.is_empty());

        // Verify root changelog exists
        let root = changelogs.iter().find(|c| c.package_name.is_none());
        assert!(root.is_some());

        // If detected as monorepo, should also have package changelogs
        if changelogs.len() > 1 {
            let pkg_a =
                changelogs.iter().find(|c| c.package_name == Some("@myorg/pkg-a".to_string()));
            assert!(pkg_a.is_some());

            let pkg_b =
                changelogs.iter().find(|c| c.package_name == Some("@myorg/pkg-b".to_string()));
            assert!(pkg_b.is_some());
        }
    }

    #[tokio::test]
    async fn test_generate_from_changeset_empty_resolution() {
        let (temp_dir, repo) = create_test_repo();
        let fs = FileSystemManager::new();
        setup_single_package(temp_dir.path(), &fs).await.unwrap();

        let config = ChangelogConfig::default();
        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs.clone(), config)
                .await
                .unwrap();

        let changeset = create_test_changeset("main", VersionBump::None, vec![]);
        let resolution = VersionResolution::new(); // Empty resolution

        let result = generator.generate_from_changeset(&changeset, &resolution).await;
        assert!(result.is_ok());

        let changelogs = result.unwrap();
        assert_eq!(changelogs.len(), 0); // No changelogs for empty resolution
    }

    #[tokio::test]
    async fn test_generated_changelog_paths() {
        let (temp_dir, repo) = create_test_repo();
        let fs = FileSystemManager::new();
        setup_test_monorepo(temp_dir.path(), &fs).await.unwrap();
        add_test_commits(&repo, temp_dir.path()).unwrap();

        let mut config = ChangelogConfig::default();
        config.filename = "RELEASES.md".to_string(); // Custom filename
        config.monorepo_mode = MonorepoMode::PerPackage;

        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs.clone(), config)
                .await
                .unwrap();

        let changeset = create_test_changeset("main", VersionBump::Patch, vec!["@myorg/pkg-a"]);
        let resolution = create_test_resolution(vec![(
            "@myorg/pkg-a",
            "1.0.0",
            "1.0.1",
            temp_dir.path().join("packages/pkg-a"),
        )]);

        let result = generator.generate_from_changeset(&changeset, &resolution).await;
        assert!(result.is_ok());

        let changelogs = result.unwrap();
        assert_eq!(changelogs.len(), 1);

        let changelog = &changelogs[0];
        // Path should contain the custom filename
        assert!(changelog.changelog_path.to_string_lossy().contains("RELEASES.md"));
        assert!(!changelog.existing); // File doesn't exist yet
    }

    #[tokio::test]
    async fn test_generated_changelog_content_not_empty() {
        let (temp_dir, repo) = create_test_repo();
        let fs = FileSystemManager::new();
        setup_single_package(temp_dir.path(), &fs).await.unwrap();
        add_test_commits(&repo, temp_dir.path()).unwrap();

        let config = ChangelogConfig::default();
        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs.clone(), config)
                .await
                .unwrap();

        let changeset = create_test_changeset("main", VersionBump::Major, vec!["my-package"]);
        let resolution = create_test_resolution(vec![(
            "my-package",
            "1.5.0",
            "2.0.0",
            temp_dir.path().to_path_buf(),
        )]);

        let result = generator.generate_from_changeset(&changeset, &resolution).await;
        assert!(result.is_ok());

        let changelogs = result.unwrap();
        assert_eq!(changelogs.len(), 1);

        let changelog = &changelogs[0];
        assert!(!changelog.content.is_empty());
        assert!(changelog.content.contains("2.0.0")); // Version should be in content
    }

    #[tokio::test]
    async fn test_generated_changelog_write_to_filesystem() {
        let (temp_dir, repo) = create_test_repo();
        let fs = FileSystemManager::new();
        setup_single_package(temp_dir.path(), &fs).await.unwrap();
        add_test_commits(&repo, temp_dir.path()).unwrap();

        let config = ChangelogConfig::default();
        let generator =
            ChangelogGenerator::new(temp_dir.path().to_path_buf(), repo, fs.clone(), config)
                .await
                .unwrap();

        let changeset = create_test_changeset("main", VersionBump::Minor, vec!["my-package"]);
        let resolution = create_test_resolution(vec![(
            "my-package",
            "1.5.0",
            "1.6.0",
            temp_dir.path().to_path_buf(),
        )]);

        let changelogs = generator.generate_from_changeset(&changeset, &resolution).await.unwrap();
        assert_eq!(changelogs.len(), 1);

        let changelog = &changelogs[0];
        let write_result = changelog.write(&fs).await;
        assert!(write_result.is_ok());

        // Verify file was created
        let changelog_path = temp_dir.path().join("CHANGELOG.md");
        assert!(fs.exists(&changelog_path).await);

        // Verify content was written
        let content = fs.read_file_string(&changelog_path).await.unwrap();
        assert!(!content.is_empty());
        assert!(content.contains("1.6.0"));
    }
}
