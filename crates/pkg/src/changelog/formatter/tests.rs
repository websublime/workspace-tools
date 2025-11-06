//! Tests for changelog formatter module.
//!
//! **What**: Comprehensive tests for the Keep a Changelog formatter implementation,
//! covering all formatting scenarios, section mapping, and configuration options.
//!
//! **How**: Tests verify correct output format, section ordering, entry formatting,
//! link generation, and handling of various configuration combinations.
//!
//! **Why**: To ensure the formatter produces correct Keep a Changelog format output
//! and handles all edge cases properly.

use super::conventional::ConventionalCommitsFormatter;
use super::custom::CustomTemplateFormatter;
use super::keep_a_changelog::{KeepAChangelogFormatter, KeepAChangelogSection};
use crate::changelog::{Changelog, ChangelogEntry, ChangelogSection, SectionType};
use crate::config::{ChangelogConfig, TemplateConfig};
use chrono::{TimeZone, Utc};

/// Creates a test entry with common defaults.
fn create_entry(
    description: &str,
    commit_type: &str,
    breaking: bool,
    references: Vec<String>,
) -> ChangelogEntry {
    ChangelogEntry {
        description: description.to_string(),
        commit_hash: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
        short_hash: "abcdef1".to_string(),
        commit_type: Some(commit_type.to_string()),
        scope: Some("core".to_string()),
        breaking,
        author: "John Doe".to_string(),
        references,
        date: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
    }
}

/// Creates a test changelog with predefined date.
fn create_changelog(package_name: &str, version: &str) -> Changelog {
    Changelog::new(
        Some(package_name),
        version,
        None,
        Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap(),
    )
}

/// Creates a simple test entry for basic tests.
fn create_test_entry(description: &str, breaking: bool) -> ChangelogEntry {
    ChangelogEntry {
        description: description.to_string(),
        commit_hash: "abcdef1234567890".to_string(),
        short_hash: "abcdef1".to_string(),
        commit_type: Some("feat".to_string()),
        scope: None,
        breaking,
        author: "Test Author".to_string(),
        references: vec![],
        date: Utc::now(),
    }
}

#[test]
fn test_section_mapping() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    assert_eq!(formatter.map_section_type(&SectionType::Features), KeepAChangelogSection::Added);
    assert_eq!(formatter.map_section_type(&SectionType::Fixes), KeepAChangelogSection::Fixed);
    assert_eq!(
        formatter.map_section_type(&SectionType::Deprecations),
        KeepAChangelogSection::Deprecated
    );
    assert_eq!(formatter.map_section_type(&SectionType::Breaking), KeepAChangelogSection::Changed);
    assert_eq!(
        formatter.map_section_type(&SectionType::Performance),
        KeepAChangelogSection::Changed
    );
}

#[test]
fn test_format_version_header() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let changelog = Changelog::new(Some("test-pkg"), "1.0.0", None, Utc::now());
    let header = formatter.format_version_header(&changelog);

    assert!(header.contains("1.0.0"));
    assert!(header.starts_with("##"));
}

#[test]
fn test_format_entry_without_links() {
    let config = ChangelogConfig {
        include_commit_links: false,
        include_issue_links: false,
        include_authors: false,
        ..ChangelogConfig::default()
    };

    let formatter = KeepAChangelogFormatter::new(&config);
    let entry = create_test_entry("Add new feature", false);

    let formatted = formatter.format_entry(&entry);
    assert_eq!(formatted, "- Add new feature");
}

#[test]
fn test_format_entry_with_breaking_marker() {
    let config = ChangelogConfig {
        include_commit_links: false,
        include_issue_links: false,
        include_authors: false,
        ..ChangelogConfig::default()
    };

    let formatter = KeepAChangelogFormatter::new(&config);
    let entry = create_test_entry("Change API behavior", true);

    let formatted = formatter.format_entry(&entry);
    assert_eq!(formatted, "- **BREAKING**: Change API behavior");
}

#[test]
fn test_format_entry_with_commit_link() {
    let config = ChangelogConfig {
        include_commit_links: true,
        repository_url: Some("https://github.com/test/repo".to_string()),
        include_issue_links: false,
        include_authors: false,
        ..ChangelogConfig::default()
    };

    let formatter = KeepAChangelogFormatter::new(&config);
    let entry = create_test_entry("Add feature", false);

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("Add feature"));
    assert!(formatted.contains("[abcdef1]"));
    assert!(formatted.contains("/commit/abcdef1234567890"));
}

#[test]
fn test_format_entry_with_issue_links() {
    let config = ChangelogConfig {
        include_commit_links: false,
        include_issue_links: true,
        repository_url: Some("https://github.com/test/repo".to_string()),
        include_authors: false,
        ..ChangelogConfig::default()
    };

    let formatter = KeepAChangelogFormatter::new(&config);
    let mut entry = create_test_entry("Fix bug", false);
    entry.references = vec!["#123".to_string(), "#456".to_string()];

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("Fix bug"));
    assert!(formatted.contains("[#123]"));
    assert!(formatted.contains("[#456]"));
    assert!(formatted.contains("/issues/123"));
    assert!(formatted.contains("/issues/456"));
}

#[test]
fn test_format_entry_with_author() {
    let config = ChangelogConfig {
        include_commit_links: false,
        include_issue_links: false,
        include_authors: true,
        ..ChangelogConfig::default()
    };

    let formatter = KeepAChangelogFormatter::new(&config);
    let entry = create_test_entry("Update docs", false);

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("Update docs"));
    assert!(formatted.contains("by Test Author"));
}

#[test]
fn test_format_section() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let entry1 = create_test_entry("First feature", false);
    let entry2 = create_test_entry("Second feature", false);
    let entries = vec![&entry1, &entry2];

    let formatted = formatter.format_section(&KeepAChangelogSection::Added, &entries);
    assert!(formatted.contains("### Added"));
    assert!(formatted.contains("First feature"));
    assert!(formatted.contains("Second feature"));
}

#[test]
fn test_format_header() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let header = formatter.format_header();
    assert!(header.contains("# Changelog"));
    assert!(header.contains("Keep a Changelog"));
    assert!(header.contains("Semantic Versioning"));
}

#[test]
fn test_format_complete_changelog() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = Changelog::new(Some("test"), "1.0.0", None, Utc::now());
    let mut section = ChangelogSection::new(SectionType::Features);
    section.add_entry(create_test_entry("Add feature X", false));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);
    assert!(formatted.contains("1.0.0"));
    assert!(formatted.contains("### Added"));
    assert!(formatted.contains("Add feature X"));
}

#[test]
fn test_group_sections() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut features_section = ChangelogSection::new(SectionType::Features);
    features_section.add_entry(create_test_entry("Feature 1", false));

    let mut fixes_section = ChangelogSection::new(SectionType::Fixes);
    fixes_section.add_entry(create_test_entry("Fix 1", false));

    let sections = vec![features_section, fixes_section];
    let grouped = formatter.group_sections(&sections);

    assert_eq!(grouped.len(), 2);
    assert!(grouped.contains_key(&KeepAChangelogSection::Added));
    assert!(grouped.contains_key(&KeepAChangelogSection::Fixed));
}

#[test]
fn test_section_priority_ordering() {
    assert!(KeepAChangelogSection::Added.priority() < KeepAChangelogSection::Changed.priority());
    assert!(
        KeepAChangelogSection::Changed.priority() < KeepAChangelogSection::Deprecated.priority()
    );
    assert!(
        KeepAChangelogSection::Deprecated.priority() < KeepAChangelogSection::Removed.priority()
    );
    assert!(KeepAChangelogSection::Removed.priority() < KeepAChangelogSection::Fixed.priority());
    assert!(KeepAChangelogSection::Fixed.priority() < KeepAChangelogSection::Security.priority());
}

#[test]
fn test_format_empty_changelog() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);
    let changelog = create_changelog("test-pkg", "1.0.0");

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("## [1.0.0] - 2024-01-15"));
    assert!(!formatted.contains("###")); // No sections
}

#[test]
fn test_format_changelog_with_features() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Features);
    section.add_entry(create_entry("Add new API endpoint", "feat", false, vec![]));
    section.add_entry(create_entry("Add user authentication", "feat", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("## [1.0.0] - 2024-01-15"));
    assert!(formatted.contains("### Added"));
    assert!(formatted.contains("Add new API endpoint"));
    assert!(formatted.contains("Add user authentication"));
}

#[test]
fn test_format_changelog_with_fixes() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.1");
    let mut section = ChangelogSection::new(SectionType::Fixes);
    section.add_entry(create_entry("Fix memory leak", "fix", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### Fixed"));
    assert!(formatted.contains("Fix memory leak"));
}

#[test]
fn test_format_changelog_with_breaking_changes() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "2.0.0");
    let mut section = ChangelogSection::new(SectionType::Breaking);
    section.add_entry(create_entry("Remove deprecated API", "feat", true, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### Changed"));
    assert!(formatted.contains("**BREAKING**: Remove deprecated API"));
}

#[test]
fn test_format_multiple_sections_ordered() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.1.0");

    // Add sections in non-standard order
    let mut fixes = ChangelogSection::new(SectionType::Fixes);
    fixes.add_entry(create_entry("Fix bug", "fix", false, vec![]));
    changelog.add_section(fixes);

    let mut features = ChangelogSection::new(SectionType::Features);
    features.add_entry(create_entry("Add feature", "feat", false, vec![]));
    changelog.add_section(features);

    let mut deprecations = ChangelogSection::new(SectionType::Deprecations);
    deprecations.add_entry(create_entry("Deprecate old API", "chore", false, vec![]));
    changelog.add_section(deprecations);

    let formatted = formatter.format(&changelog);

    // Verify ordering: Added, Changed, Deprecated, Removed, Fixed, Security
    let added_pos = formatted.find("### Added");
    let deprecated_pos = formatted.find("### Deprecated");
    let fixed_pos = formatted.find("### Fixed");

    assert!(added_pos.is_some(), "Added section not found");
    assert!(deprecated_pos.is_some(), "Deprecated section not found");
    assert!(fixed_pos.is_some(), "Fixed section not found");

    let added_pos = added_pos.unwrap_or(0);
    let deprecated_pos = deprecated_pos.unwrap_or(0);
    let fixed_pos = fixed_pos.unwrap_or(0);

    assert!(added_pos < deprecated_pos, "Added should come before Deprecated");
    assert!(deprecated_pos < fixed_pos, "Deprecated should come before Fixed");
}

#[test]
fn test_format_with_commit_links() {
    let config = ChangelogConfig {
        include_commit_links: true,
        repository_url: Some("https://github.com/user/repo".to_string()),
        include_issue_links: false,
        include_authors: false,
        ..ChangelogConfig::default()
    };

    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Features);
    section.add_entry(create_entry("Add feature", "feat", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains(
        "[abcdef1](https://github.com/user/repo/commit/abcdef1234567890abcdef1234567890abcdef12)"
    ));
}

#[test]
fn test_format_with_issue_links() {
    let config = ChangelogConfig {
        include_commit_links: false,
        include_issue_links: true,
        repository_url: Some("https://github.com/user/repo".to_string()),
        include_authors: false,
        ..ChangelogConfig::default()
    };

    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Fixes);
    section.add_entry(create_entry(
        "Fix critical bug",
        "fix",
        false,
        vec!["#123".to_string(), "#456".to_string()],
    ));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("[#123](https://github.com/user/repo/issues/123)"));
    assert!(formatted.contains("[#456](https://github.com/user/repo/issues/456)"));
}

#[test]
fn test_format_with_authors() {
    let config = ChangelogConfig {
        include_commit_links: false,
        include_issue_links: false,
        include_authors: true,
        ..ChangelogConfig::default()
    };

    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Features);
    section.add_entry(create_entry("Add feature", "feat", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("by John Doe"));
}

#[test]
fn test_format_without_links_or_authors() {
    let config = ChangelogConfig {
        include_commit_links: false,
        include_issue_links: false,
        include_authors: false,
        ..ChangelogConfig::default()
    };

    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Features);
    section.add_entry(create_entry("Add feature", "feat", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("- Add feature"));
    assert!(!formatted.contains("http"));
    assert!(!formatted.contains("by "));
    assert!(!formatted.contains("("));
}

#[test]
fn test_format_performance_as_changed() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.1.0");
    let mut section = ChangelogSection::new(SectionType::Performance);
    section.add_entry(create_entry("Optimize query performance", "perf", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### Changed"));
    assert!(formatted.contains("Optimize query performance"));
}

#[test]
fn test_format_refactoring_as_changed() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Refactoring);
    section.add_entry(create_entry("Refactor data layer", "refactor", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### Changed"));
    assert!(formatted.contains("Refactor data layer"));
}

#[test]
fn test_format_documentation_as_changed() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Documentation);
    section.add_entry(create_entry("Update API documentation", "docs", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### Changed"));
    assert!(formatted.contains("Update API documentation"));
}

#[test]
fn test_format_build_as_changed() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Build);
    section.add_entry(create_entry("Update build dependencies", "build", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### Changed"));
    assert!(formatted.contains("Update build dependencies"));
}

#[test]
fn test_format_ci_as_changed() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::CI);
    section.add_entry(create_entry("Add CI pipeline", "ci", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### Changed"));
    assert!(formatted.contains("Add CI pipeline"));
}

#[test]
fn test_format_tests_as_changed() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Tests);
    section.add_entry(create_entry("Add integration tests", "test", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### Changed"));
    assert!(formatted.contains("Add integration tests"));
}

#[test]
fn test_format_other_as_changed() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Other);
    section.add_entry(create_entry("Update configuration", "chore", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### Changed"));
    assert!(formatted.contains("Update configuration"));
}

#[test]
fn test_format_header_default() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let header = formatter.format_header();

    assert!(header.contains("# Changelog"));
    assert!(header.contains("All notable changes to this project will be documented in this file"));
    assert!(header.contains("Keep a Changelog"));
    assert!(header.contains("keepachangelog.com"));
    assert!(header.contains("Semantic Versioning"));
    assert!(header.contains("semver.org"));
}

#[test]
fn test_format_complete_with_multiple_versions() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog1 = create_changelog("test-pkg", "1.1.0");
    let mut section1 = ChangelogSection::new(SectionType::Features);
    section1.add_entry(create_entry("Add new feature", "feat", false, vec![]));
    changelog1.add_section(section1);

    let mut changelog2 = create_changelog("test-pkg", "1.0.0");
    let mut section2 = ChangelogSection::new(SectionType::Features);
    section2.add_entry(create_entry("Initial release", "feat", false, vec![]));
    changelog2.add_section(section2);

    let changelogs = vec![changelog1, changelog2];
    let formatted = formatter.format_complete(&changelogs);

    assert!(formatted.contains("# Changelog"));
    assert!(formatted.contains("## [Unreleased]"));
    assert!(formatted.contains("## [1.1.0] - 2024-01-15"));
    assert!(formatted.contains("## [1.0.0] - 2024-01-15"));
    assert!(formatted.contains("Add new feature"));
    assert!(formatted.contains("Initial release"));
}

#[test]
fn test_format_breaking_and_non_breaking_in_same_section() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "2.0.0");
    let mut section = ChangelogSection::new(SectionType::Breaking);
    section.add_entry(create_entry("Remove old API", "feat", true, vec![]));
    section.add_entry(create_entry("Update implementation", "feat", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### Changed"));
    assert!(formatted.contains("**BREAKING**: Remove old API"));
    assert!(formatted.contains("Update implementation"));
    assert!(!formatted.contains("**BREAKING**: Update implementation"));
}

#[test]
fn test_format_with_all_link_types() {
    let config = ChangelogConfig {
        include_commit_links: true,
        include_issue_links: true,
        include_authors: true,
        repository_url: Some("https://github.com/user/repo".to_string()),
        ..ChangelogConfig::default()
    };

    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Fixes);
    section.add_entry(create_entry("Fix bug", "fix", false, vec!["#123".to_string()]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("Fix bug"));
    assert!(formatted.contains("[abcdef1]"));
    assert!(formatted.contains("/commit/"));
    assert!(formatted.contains("[#123]"));
    assert!(formatted.contains("/issues/123"));
    assert!(formatted.contains("by John Doe"));
}

#[test]
fn test_format_entry_without_repository_url() {
    let config = ChangelogConfig {
        include_commit_links: true,
        include_issue_links: true,
        repository_url: None,
        ..ChangelogConfig::default()
    };

    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Features);
    section.add_entry(create_entry("Add feature", "feat", false, vec!["#123".to_string()]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("Add feature"));
    assert!(formatted.contains("(abcdef1)"));
    assert!(formatted.contains("(#123)"));
    assert!(!formatted.contains("http"));
}

#[test]
fn test_multiple_entries_in_same_section() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Features);
    section.add_entry(create_entry("First feature", "feat", false, vec![]));
    section.add_entry(create_entry("Second feature", "feat", false, vec![]));
    section.add_entry(create_entry("Third feature", "feat", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### Added"));
    assert!(formatted.contains("First feature"));
    assert!(formatted.contains("Second feature"));
    assert!(formatted.contains("Third feature"));
}

#[test]
fn test_mixed_sections_merged_correctly() {
    let config = ChangelogConfig::default();
    let formatter = KeepAChangelogFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");

    // Add Performance section
    let mut perf = ChangelogSection::new(SectionType::Performance);
    perf.add_entry(create_entry("Optimize queries", "perf", false, vec![]));
    changelog.add_section(perf);

    // Add Refactoring section
    let mut refactor = ChangelogSection::new(SectionType::Refactoring);
    refactor.add_entry(create_entry("Refactor code", "refactor", false, vec![]));
    changelog.add_section(refactor);

    let formatted = formatter.format(&changelog);

    // Both should be in "Changed" section
    assert!(formatted.contains("### Changed"));
    assert!(formatted.contains("Optimize queries"));
    assert!(formatted.contains("Refactor code"));

    // Should only have one "Changed" section
    assert_eq!(formatted.matches("### Changed").count(), 1);
}

// ============================================================================
// Conventional Commits Formatter Tests
// ============================================================================

#[test]
fn test_conventional_format_version_header() {
    let config = ChangelogConfig::default();
    let formatter = ConventionalCommitsFormatter::new(&config);

    let changelog = Changelog::new(Some("test-pkg"), "1.0.0", None, Utc::now());
    let header = formatter.format_version_header(&changelog);

    assert!(header.contains("1.0.0"));
    assert!(header.starts_with("##"));
}

#[test]
fn test_conventional_grouping_by_type() {
    let config = ChangelogConfig::default();
    let formatter = ConventionalCommitsFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");

    // Add features section
    let mut features = ChangelogSection::new(SectionType::Features);
    features.add_entry(create_entry("Add new API", "feat", false, vec![]));
    changelog.add_section(features);

    // Add fixes section
    let mut fixes = ChangelogSection::new(SectionType::Fixes);
    fixes.add_entry(create_entry("Fix bug", "fix", false, vec![]));
    changelog.add_section(fixes);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### Features"));
    assert!(formatted.contains("### Bug Fixes"));
    assert!(formatted.contains("Add new API"));
    assert!(formatted.contains("Fix bug"));
}

#[test]
fn test_conventional_breaking_changes_first() {
    let config = ChangelogConfig::default();
    let formatter = ConventionalCommitsFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "2.0.0");

    // Add features section
    let mut features = ChangelogSection::new(SectionType::Features);
    features.add_entry(create_entry("Add new API", "feat", false, vec![]));
    changelog.add_section(features);

    // Add breaking changes section
    let mut breaking = ChangelogSection::new(SectionType::Breaking);
    breaking.add_entry(create_entry("Remove old API", "feat", true, vec![]));
    changelog.add_section(breaking);

    let formatted = formatter.format(&changelog);

    // Breaking changes should appear before features
    let breaking_pos = formatted.find("### Breaking Changes");
    let features_pos = formatted.find("### Features");

    assert!(breaking_pos.is_some(), "Breaking Changes section not found");
    assert!(features_pos.is_some(), "Features section not found");

    let breaking_pos = breaking_pos.unwrap_or(0);
    let features_pos = features_pos.unwrap_or(0);
    assert!(breaking_pos < features_pos, "Breaking Changes should appear before Features");
}

#[test]
fn test_conventional_section_ordering() {
    let config = ChangelogConfig::default();
    let formatter = ConventionalCommitsFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.1.0");

    // Add sections in non-priority order
    let mut docs = ChangelogSection::new(SectionType::Documentation);
    docs.add_entry(create_entry("Update docs", "docs", false, vec![]));
    changelog.add_section(docs);

    let mut features = ChangelogSection::new(SectionType::Features);
    features.add_entry(create_entry("Add feature", "feat", false, vec![]));
    changelog.add_section(features);

    let mut fixes = ChangelogSection::new(SectionType::Fixes);
    fixes.add_entry(create_entry("Fix bug", "fix", false, vec![]));
    changelog.add_section(fixes);

    let formatted = formatter.format(&changelog);

    // Verify ordering based on priority
    let features_pos = formatted.find("### Features");
    let fixes_pos = formatted.find("### Bug Fixes");
    let docs_pos = formatted.find("### Documentation");

    assert!(features_pos.is_some());
    assert!(fixes_pos.is_some());
    assert!(docs_pos.is_some());

    let features_pos = features_pos.unwrap_or(0);
    let fixes_pos = fixes_pos.unwrap_or(0);
    let docs_pos = docs_pos.unwrap_or(0);

    assert!(features_pos < fixes_pos);
    assert!(fixes_pos < docs_pos);
}

#[test]
fn test_conventional_entry_with_scope() {
    let config = ChangelogConfig {
        include_commit_links: false,
        include_issue_links: false,
        include_authors: false,
        ..ChangelogConfig::default()
    };

    let formatter = ConventionalCommitsFormatter::new(&config);

    let entry = ChangelogEntry {
        description: "Add authentication".to_string(),
        commit_hash: "abc123".to_string(),
        short_hash: "abc123".to_string(),
        commit_type: Some("feat".to_string()),
        scope: Some("auth".to_string()),
        breaking: false,
        author: "Test Author".to_string(),
        references: vec![],
        date: Utc::now(),
    };

    let formatted = formatter.format_entry(&entry);
    assert_eq!(formatted, "- **auth**: Add authentication");
}

#[test]
fn test_conventional_entry_without_scope() {
    let config = ChangelogConfig {
        include_commit_links: false,
        include_issue_links: false,
        include_authors: false,
        ..ChangelogConfig::default()
    };

    let formatter = ConventionalCommitsFormatter::new(&config);
    let entry = create_test_entry("Add new feature", false);

    let formatted = formatter.format_entry(&entry);
    assert_eq!(formatted, "- Add new feature");
}

#[test]
fn test_conventional_entry_with_commit_link() {
    let config = ChangelogConfig {
        include_commit_links: true,
        repository_url: Some("https://github.com/test/repo".to_string()),
        include_issue_links: false,
        include_authors: false,
        ..ChangelogConfig::default()
    };

    let formatter = ConventionalCommitsFormatter::new(&config);
    let entry = create_test_entry("Add feature", false);

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("Add feature"));
    assert!(formatted.contains("[abcdef1]"));
    assert!(formatted.contains("/commit/abcdef1234567890"));
}

#[test]
fn test_conventional_entry_with_issue_links() {
    let config = ChangelogConfig {
        include_commit_links: false,
        include_issue_links: true,
        repository_url: Some("https://github.com/test/repo".to_string()),
        include_authors: false,
        ..ChangelogConfig::default()
    };

    let formatter = ConventionalCommitsFormatter::new(&config);
    let mut entry = create_test_entry("Fix critical bug", false);
    entry.references = vec!["#123".to_string(), "#456".to_string()];

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("Fix critical bug"));
    assert!(formatted.contains("[#123]"));
    assert!(formatted.contains("[#456]"));
    assert!(formatted.contains("/issues/123"));
    assert!(formatted.contains("/issues/456"));
}

#[test]
fn test_conventional_entry_with_author() {
    let config = ChangelogConfig {
        include_commit_links: false,
        include_issue_links: false,
        include_authors: true,
        ..ChangelogConfig::default()
    };

    let formatter = ConventionalCommitsFormatter::new(&config);
    let entry = create_test_entry("Update documentation", false);

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("Update documentation"));
    assert!(formatted.contains("by Test Author"));
}

#[test]
fn test_conventional_custom_section_titles() {
    use std::collections::HashMap;

    let mut types = HashMap::new();
    types.insert("feat".to_string(), "New Features".to_string());
    types.insert("fix".to_string(), "Bug Fixes".to_string());
    types.insert("perf".to_string(), "Performance".to_string());

    let config = ChangelogConfig {
        conventional: crate::config::ConventionalConfig {
            enabled: true,
            types,
            breaking_section: "💥 Breaking Changes".to_string(),
        },
        ..ChangelogConfig::default()
    };

    let formatter = ConventionalCommitsFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");

    let mut features = ChangelogSection::new(SectionType::Features);
    features.add_entry(create_entry("Add API", "feat", false, vec![]));
    changelog.add_section(features);

    let mut performance = ChangelogSection::new(SectionType::Performance);
    performance.add_entry(create_entry("Optimize queries", "perf", false, vec![]));
    changelog.add_section(performance);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### New Features"));
    assert!(formatted.contains("### Performance"));
}

#[test]
fn test_conventional_custom_breaking_title() {
    let config = ChangelogConfig {
        conventional: crate::config::ConventionalConfig {
            enabled: true,
            types: std::collections::HashMap::new(),
            breaking_section: "💥 Breaking Changes".to_string(),
        },
        ..ChangelogConfig::default()
    };

    let formatter = ConventionalCommitsFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "2.0.0");

    let mut breaking = ChangelogSection::new(SectionType::Breaking);
    breaking.add_entry(create_entry("Remove API", "feat", true, vec![]));
    changelog.add_section(breaking);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### 💥 Breaking Changes"));
}

#[test]
fn test_conventional_format_header() {
    let config = ChangelogConfig::default();
    let formatter = ConventionalCommitsFormatter::new(&config);

    let header = formatter.format_header();
    assert!(header.contains("# Changelog"));
    assert!(header.contains("Conventional Commits"));
    assert!(header.contains("Semantic Versioning"));
}

#[test]
fn test_conventional_format_complete() {
    let config = ChangelogConfig::default();
    let formatter = ConventionalCommitsFormatter::new(&config);

    let mut changelog1 = create_changelog("test-pkg", "1.1.0");
    let mut features = ChangelogSection::new(SectionType::Features);
    features.add_entry(create_entry("Add feature X", "feat", false, vec![]));
    changelog1.add_section(features);

    let mut changelog2 = create_changelog("test-pkg", "1.0.0");
    let mut fixes = ChangelogSection::new(SectionType::Fixes);
    fixes.add_entry(create_entry("Fix bug Y", "fix", false, vec![]));
    changelog2.add_section(fixes);

    let changelogs = vec![changelog1, changelog2];
    let formatted = formatter.format_complete(&changelogs);

    assert!(formatted.contains("# Changelog"));
    assert!(formatted.contains("## [Unreleased]"));
    assert!(formatted.contains("## [1.1.0]"));
    assert!(formatted.contains("## [1.0.0]"));
    assert!(formatted.contains("### Features"));
    assert!(formatted.contains("### Bug Fixes"));
}

#[test]
fn test_conventional_all_section_types() {
    let config = ChangelogConfig::default();
    let formatter = ConventionalCommitsFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "2.0.0");

    // Add all section types
    let mut breaking = ChangelogSection::new(SectionType::Breaking);
    breaking.add_entry(create_entry("Breaking change", "feat", true, vec![]));
    changelog.add_section(breaking);

    let mut features = ChangelogSection::new(SectionType::Features);
    features.add_entry(create_entry("New feature", "feat", false, vec![]));
    changelog.add_section(features);

    let mut fixes = ChangelogSection::new(SectionType::Fixes);
    fixes.add_entry(create_entry("Bug fix", "fix", false, vec![]));
    changelog.add_section(fixes);

    let mut perf = ChangelogSection::new(SectionType::Performance);
    perf.add_entry(create_entry("Performance improvement", "perf", false, vec![]));
    changelog.add_section(perf);

    let mut deprecations = ChangelogSection::new(SectionType::Deprecations);
    deprecations.add_entry(create_entry("Deprecate old API", "deprecate", false, vec![]));
    changelog.add_section(deprecations);

    let mut docs = ChangelogSection::new(SectionType::Documentation);
    docs.add_entry(create_entry("Update docs", "docs", false, vec![]));
    changelog.add_section(docs);

    let mut refactor = ChangelogSection::new(SectionType::Refactoring);
    refactor.add_entry(create_entry("Refactor code", "refactor", false, vec![]));
    changelog.add_section(refactor);

    let mut build = ChangelogSection::new(SectionType::Build);
    build.add_entry(create_entry("Update build", "build", false, vec![]));
    changelog.add_section(build);

    let mut ci = ChangelogSection::new(SectionType::CI);
    ci.add_entry(create_entry("Update CI", "ci", false, vec![]));
    changelog.add_section(ci);

    let mut tests = ChangelogSection::new(SectionType::Tests);
    tests.add_entry(create_entry("Add tests", "test", false, vec![]));
    changelog.add_section(tests);

    let mut other = ChangelogSection::new(SectionType::Other);
    other.add_entry(create_entry("Other change", "chore", false, vec![]));
    changelog.add_section(other);

    let formatted = formatter.format(&changelog);

    // Verify all sections are present
    assert!(formatted.contains("### Breaking Changes"));
    assert!(formatted.contains("### Features"));
    assert!(formatted.contains("### Bug Fixes"));
    assert!(formatted.contains("### Performance Improvements"));
    assert!(formatted.contains("### Deprecations"));
    assert!(formatted.contains("### Documentation"));
    assert!(formatted.contains("### Code Refactoring"));
    assert!(formatted.contains("### Build System"));
    assert!(formatted.contains("### Continuous Integration"));
    assert!(formatted.contains("### Tests"));
    assert!(formatted.contains("### Chores"));
}

#[test]
fn test_conventional_section_type_to_commit_type() {
    let config = ChangelogConfig::default();
    let formatter = ConventionalCommitsFormatter::new(&config);

    assert_eq!(formatter.section_type_to_commit_type(&SectionType::Breaking), "breaking");
    assert_eq!(formatter.section_type_to_commit_type(&SectionType::Features), "feat");
    assert_eq!(formatter.section_type_to_commit_type(&SectionType::Fixes), "fix");
    assert_eq!(formatter.section_type_to_commit_type(&SectionType::Performance), "perf");
    assert_eq!(formatter.section_type_to_commit_type(&SectionType::Deprecations), "deprecate");
    assert_eq!(formatter.section_type_to_commit_type(&SectionType::Documentation), "docs");
    assert_eq!(formatter.section_type_to_commit_type(&SectionType::Refactoring), "refactor");
    assert_eq!(formatter.section_type_to_commit_type(&SectionType::Build), "build");
    assert_eq!(formatter.section_type_to_commit_type(&SectionType::CI), "ci");
    assert_eq!(formatter.section_type_to_commit_type(&SectionType::Tests), "test");
    assert_eq!(formatter.section_type_to_commit_type(&SectionType::Other), "chore");
}

#[test]
fn test_conventional_empty_changelog() {
    let config = ChangelogConfig::default();
    let formatter = ConventionalCommitsFormatter::new(&config);
    let changelog = create_changelog("test-pkg", "1.0.0");

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("## [1.0.0] - 2024-01-15"));
    assert!(!formatted.contains("###")); // No sections
}

#[test]
fn test_conventional_multiple_entries_per_section() {
    let config = ChangelogConfig::default();
    let formatter = ConventionalCommitsFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");

    let mut features = ChangelogSection::new(SectionType::Features);
    features.add_entry(create_entry("Add feature A", "feat", false, vec![]));
    features.add_entry(create_entry("Add feature B", "feat", false, vec![]));
    features.add_entry(create_entry("Add feature C", "feat", false, vec![]));
    changelog.add_section(features);

    let formatted = formatter.format(&changelog);

    assert!(formatted.contains("### Features"));
    assert!(formatted.contains("Add feature A"));
    assert!(formatted.contains("Add feature B"));
    assert!(formatted.contains("Add feature C"));

    // Should only have one Features section
    assert_eq!(formatted.matches("### Features").count(), 1);
}

// ============================================================================
// Custom Template Formatter Tests
// ============================================================================

#[test]
fn test_custom_format_version_header() {
    let config = ChangelogConfig::default();
    let formatter = CustomTemplateFormatter::new(&config);
    let changelog = create_changelog("test-pkg", "1.0.0");

    let header = formatter.format_version_header(&changelog);
    assert!(header.contains("1.0.0"));
    assert!(header.contains("2024-01-15"));
}

#[test]
fn test_custom_format_version_header_with_package() {
    let mut config = ChangelogConfig::default();
    config.template.version_header = "## {package} v{version} ({date})".to_string();

    let formatter = CustomTemplateFormatter::new(&config);
    let changelog = create_changelog("my-package", "2.0.0");

    let header = formatter.format_version_header(&changelog);
    assert!(header.contains("my-package"));
    assert!(header.contains("v2.0.0"));
    assert!(header.contains("2024-01-15"));
}

#[test]
fn test_custom_format_section_header() {
    let config = ChangelogConfig::default();
    let formatter = CustomTemplateFormatter::new(&config);
    let section = ChangelogSection::new(SectionType::Features);

    let header = formatter.format_section_header(&section);
    assert!(header.contains("Features"));
}

#[test]
fn test_custom_format_section_header_custom_template() {
    let mut config = ChangelogConfig::default();
    config.template.section_header = "## {title}:".to_string();

    let formatter = CustomTemplateFormatter::new(&config);
    let section = ChangelogSection::new(SectionType::Fixes);

    let header = formatter.format_section_header(&section);
    assert!(header.contains("## Bug Fixes:"));
}

#[test]
fn test_custom_format_entry_basic() {
    let config = ChangelogConfig::default();
    let formatter = CustomTemplateFormatter::new(&config);
    let entry = create_entry("Add new feature", "feat", false, vec![]);

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("Add new feature"));
    assert!(formatted.contains("abcdef1"));
}

#[test]
fn test_custom_format_entry_with_all_variables() {
    let mut config = ChangelogConfig::default();
    config.template.entry_format =
        "{breaking}{type}({scope}): {description} - {author} [{short_hash}] {references} on {date}"
            .to_string();
    config.include_authors = true;

    let formatter = CustomTemplateFormatter::new(&config);
    let entry = create_entry("Fix critical bug", "fix", false, vec!["#123".to_string()]);

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("fix"));
    assert!(formatted.contains("core"));
    assert!(formatted.contains("Fix critical bug"));
    assert!(formatted.contains("John Doe"));
    assert!(formatted.contains("abcdef1"));
    assert!(formatted.contains("#123"));
    assert!(formatted.contains("2024-01-15"));
}

#[test]
fn test_custom_format_entry_breaking_marker() {
    let mut config = ChangelogConfig::default();
    config.template.entry_format = "{breaking}{description}".to_string();

    let formatter = CustomTemplateFormatter::new(&config);
    let entry = create_entry("Change API signature", "feat", true, vec![]);

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("BREAKING: Change API signature"));
}

#[test]
fn test_custom_format_entry_without_breaking_marker() {
    let mut config = ChangelogConfig::default();
    config.template.entry_format = "{breaking}{description}".to_string();

    let formatter = CustomTemplateFormatter::new(&config);
    let entry = create_entry("Add feature", "feat", false, vec![]);

    let formatted = formatter.format_entry(&entry);
    assert!(!formatted.contains("BREAKING"));
    assert!(formatted.contains("Add feature"));
}

#[test]
fn test_custom_format_entry_with_commit_link() {
    let config = ChangelogConfig {
        repository_url: Some("https://github.com/user/repo".to_string()),
        include_commit_links: true,
        ..Default::default()
    };

    let formatter = CustomTemplateFormatter::new(&config);
    let entry = create_entry("Fix bug", "fix", false, vec![]);

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("[abcdef1]"));
    assert!(
        formatted.contains(
            "https://github.com/user/repo/commit/abcdef1234567890abcdef1234567890abcdef12"
        )
    );
}

#[test]
fn test_custom_format_entry_without_commit_link() {
    let config =
        ChangelogConfig { repository_url: None, include_commit_links: false, ..Default::default() };

    let formatter = CustomTemplateFormatter::new(&config);
    let entry = create_entry("Fix bug", "fix", false, vec![]);

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("abcdef1"));
    assert!(!formatted.contains("]("));
}

#[test]
fn test_custom_format_entry_with_issue_links() {
    let config = ChangelogConfig {
        repository_url: Some("https://github.com/user/repo".to_string()),
        include_issue_links: true,
        template: TemplateConfig {
            entry_format: "- {description} {references}".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    let formatter = CustomTemplateFormatter::new(&config);
    let entry = create_entry("Fix bug", "fix", false, vec!["#123".to_string(), "#456".to_string()]);

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("[#123](https://github.com/user/repo/issues/123)"));
    assert!(formatted.contains("[#456](https://github.com/user/repo/issues/456)"));
}

#[test]
fn test_custom_format_entry_without_issue_links() {
    let config = ChangelogConfig {
        repository_url: None,
        include_issue_links: false,
        template: TemplateConfig {
            entry_format: "- {description} {references}".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    let formatter = CustomTemplateFormatter::new(&config);
    let entry = create_entry("Fix bug", "fix", false, vec!["#123".to_string()]);

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("#123"));
    assert!(!formatted.contains("]("));
}

#[test]
fn test_custom_format_entry_with_author() {
    let mut config = ChangelogConfig::default();
    config.template.entry_format = "- {description} by {author}".to_string();
    config.include_authors = true;

    let formatter = CustomTemplateFormatter::new(&config);
    let entry = create_entry("Add feature", "feat", false, vec![]);

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("by John Doe"));
}

#[test]
fn test_custom_format_entry_without_author() {
    let mut config = ChangelogConfig::default();
    config.template.entry_format = "- {description} by {author}".to_string();
    config.include_authors = false;

    let formatter = CustomTemplateFormatter::new(&config);
    let entry = create_entry("Add feature", "feat", false, vec![]);

    let formatted = formatter.format_entry(&entry);
    assert!(!formatted.contains("John Doe"));
    assert!(formatted.contains("by "));
}

#[test]
fn test_custom_format_section() {
    let config = ChangelogConfig::default();
    let formatter = CustomTemplateFormatter::new(&config);

    let mut section = ChangelogSection::new(SectionType::Features);
    section.add_entry(create_entry("Add feature A", "feat", false, vec![]));
    section.add_entry(create_entry("Add feature B", "feat", false, vec![]));

    let formatted = formatter.format_section(&section);
    assert!(formatted.contains("Features"));
    assert!(formatted.contains("Add feature A"));
    assert!(formatted.contains("Add feature B"));
}

#[test]
fn test_custom_format_empty_section() {
    let config = ChangelogConfig::default();
    let formatter = CustomTemplateFormatter::new(&config);
    let section = ChangelogSection::new(SectionType::Features);

    let formatted = formatter.format_section(&section);
    assert!(formatted.is_empty());
}

#[test]
fn test_custom_format_changelog() {
    let config = ChangelogConfig::default();
    let formatter = CustomTemplateFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Features);
    section.add_entry(create_entry("Add new feature", "feat", false, vec![]));
    changelog.add_section(section);

    let formatted = formatter.format(&changelog);
    assert!(formatted.contains("1.0.0"));
    assert!(formatted.contains("2024-01-15"));
    assert!(formatted.contains("Features"));
    assert!(formatted.contains("Add new feature"));
}

#[test]
fn test_custom_format_changelog_multiple_sections() {
    let config = ChangelogConfig::default();
    let formatter = CustomTemplateFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "2.0.0");

    let mut features = ChangelogSection::new(SectionType::Features);
    features.add_entry(create_entry("Add feature", "feat", false, vec![]));
    changelog.add_section(features);

    let mut fixes = ChangelogSection::new(SectionType::Fixes);
    fixes.add_entry(create_entry("Fix bug", "fix", false, vec![]));
    changelog.add_section(fixes);

    let formatted = formatter.format(&changelog);
    assert!(formatted.contains("Features"));
    assert!(formatted.contains("Bug Fixes"));
    assert!(formatted.contains("Add feature"));
    assert!(formatted.contains("Fix bug"));
}

#[test]
fn test_custom_format_header() {
    let config = ChangelogConfig::default();
    let formatter = CustomTemplateFormatter::new(&config);

    let header = formatter.format_header();
    assert!(header.contains("Changelog"));
}

#[test]
fn test_custom_format_complete() {
    let config = ChangelogConfig::default();
    let formatter = CustomTemplateFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Features);
    section.add_entry(create_entry("Add feature", "feat", false, vec![]));
    changelog.add_section(section);

    let complete = formatter.format_complete(&changelog);
    assert!(complete.contains("Changelog"));
    assert!(complete.contains("1.0.0"));
    assert!(complete.contains("Features"));
    assert!(complete.contains("Add feature"));
}

#[test]
fn test_custom_template_simple_format() {
    let config = ChangelogConfig {
        template: TemplateConfig {
            header: "# Project Changelog\n".to_string(),
            version_header: "Version {version} - {date}".to_string(),
            section_header: "## {title}".to_string(),
            entry_format: "* {description}".to_string(),
        },
        ..Default::default()
    };

    let formatter = CustomTemplateFormatter::new(&config);
    let mut changelog = create_changelog("test-pkg", "1.0.0");
    let mut section = ChangelogSection::new(SectionType::Features);
    section.add_entry(create_entry("New feature", "feat", false, vec![]));
    changelog.add_section(section);

    let complete = formatter.format_complete(&changelog);
    assert!(complete.contains("Project Changelog"));
    assert!(complete.contains("Version 1.0.0"));
    assert!(complete.contains("## Features"));
    assert!(complete.contains("* New feature"));
}

#[test]
fn test_custom_template_detailed_format() {
    let config = ChangelogConfig {
        template: TemplateConfig {
            header: "# Release Notes\n\n".to_string(),
            version_header: "# Release {version} ({date})".to_string(),
            section_header: "## {title}".to_string(),
            entry_format: "- {breaking}{description} - @{author} in {short_hash}".to_string(),
        },
        include_authors: true,
        ..Default::default()
    };

    let formatter = CustomTemplateFormatter::new(&config);
    let mut changelog = create_changelog("test-pkg", "2.0.0");
    let mut section = ChangelogSection::new(SectionType::Breaking);
    section.add_entry(create_entry("Change API", "feat", true, vec![]));
    changelog.add_section(section);

    let complete = formatter.format_complete(&changelog);
    assert!(complete.contains("Release Notes"));
    assert!(complete.contains("Release 2.0.0"));
    assert!(complete.contains("BREAKING: Change API"));
    assert!(complete.contains("@John Doe"));
}

#[test]
fn test_custom_format_skip_empty_sections() {
    let config = ChangelogConfig::default();
    let formatter = CustomTemplateFormatter::new(&config);

    let mut changelog = create_changelog("test-pkg", "1.0.0");
    changelog.add_section(ChangelogSection::new(SectionType::Features));
    changelog.add_section(ChangelogSection::new(SectionType::Fixes));

    let formatted = formatter.format(&changelog);
    assert!(formatted.contains("1.0.0"));
    assert!(!formatted.contains("Features"));
    assert!(!formatted.contains("Bug Fixes"));
}

#[test]
fn test_custom_format_all_variable_types() {
    let mut config = ChangelogConfig::default();
    config.template.entry_format = "Type: {type}, Scope: {scope}, Breaking: {breaking}, Desc: {description}, Author: {author}, Hash: {hash}, Short: {short_hash}, Refs: {references}, Date: {date}".to_string();
    config.include_authors = true;
    config.repository_url = Some("https://github.com/test/repo".to_string());
    config.include_commit_links = true;
    config.include_issue_links = true;

    let formatter = CustomTemplateFormatter::new(&config);
    let entry = create_entry("Important fix", "fix", true, vec!["#999".to_string()]);

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("Type: fix"));
    assert!(formatted.contains("Scope: core"));
    assert!(formatted.contains("Breaking: BREAKING: "));
    assert!(formatted.contains("Desc: Important fix"));
    assert!(formatted.contains("Author: John Doe"));
    assert!(formatted.contains("Hash: [abcdef1]"));
    assert!(formatted.contains("Short: [abcdef1]"));
    assert!(formatted.contains("Refs: [#999]"));
    assert!(formatted.contains("Date: 2024-01-15"));
}

#[test]
fn test_custom_format_section_alias() {
    let mut config = ChangelogConfig::default();
    config.template.section_header = "### {section}".to_string();

    let formatter = CustomTemplateFormatter::new(&config);
    let section = ChangelogSection::new(SectionType::Deprecations);

    let header = formatter.format_section_header(&section);
    assert!(header.contains("### Deprecations"));
}

#[test]
fn test_custom_format_with_multiple_references() {
    let config = ChangelogConfig {
        repository_url: Some("https://github.com/test/repo".to_string()),
        include_issue_links: true,
        template: TemplateConfig {
            entry_format: "- {description} {references}".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    let formatter = CustomTemplateFormatter::new(&config);
    let entry = create_entry(
        "Fix multiple issues",
        "fix",
        false,
        vec!["#100".to_string(), "#200".to_string(), "#300".to_string()],
    );

    let formatted = formatter.format_entry(&entry);
    assert!(formatted.contains("[#100]"));
    assert!(formatted.contains("[#200]"));
    assert!(formatted.contains("[#300]"));
    assert!(formatted.contains("/issues/100"));
    assert!(formatted.contains("/issues/200"));
    assert!(formatted.contains("/issues/300"));
}
