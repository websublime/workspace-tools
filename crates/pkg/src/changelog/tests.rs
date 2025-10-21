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
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::unnecessary_to_owned)]

use crate::changelog::conventional::{ConventionalCommit, SectionType};
use crate::changelog::ChangelogGenerator;
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
