//! # Property-Based Testing Generators
//!
//! This module provides property-based testing generators using proptest for
//! the `sublime_pkg_tools` crate.
//!
//! ## What
//!
//! Provides generators for:
//! - Semantic versions (valid and invalid)
//! - Conventional commit messages
//! - Package names
//! - File paths
//! - Changeset data
//!
//! ## How
//!
//! Uses proptest's strategy combinators to generate valid and invalid test data
//! that can be used in property-based tests.
//!
//! ## Why
//!
//! Property-based testing allows testing with a wide range of inputs to find
//! edge cases and ensure robustness of the implementation.

use proptest::prelude::*;

/// Generates valid semantic version strings
///
/// This generates versions in the format "major.minor.patch" where each
/// component is a number between 0 and 100.
///
/// # Returns
///
/// A proptest strategy that generates version strings
///
/// # Examples
///
/// ```rust,ignore
/// use proptest::prelude::*;
/// use crate::common::generators::semver_strategy;
///
/// proptest! {
///     #[test]
///     fn test_version_parsing(version in semver_strategy()) {
///         // Test that all generated versions can be parsed
///         assert!(Version::parse(&version).is_ok());
///     }
/// }
/// ```
pub fn semver_strategy() -> impl Strategy<Value = String> {
    (0u32..100, 0u32..100, 0u32..100)
        .prop_map(|(major, minor, patch)| format!("{}.{}.{}", major, minor, patch))
}

/// Generates semantic versions with pre-release tags
///
/// # Returns
///
/// A proptest strategy that generates versions with pre-release identifiers
///
/// # Examples
///
/// ```rust,ignore
/// proptest! {
///     #[test]
///     fn test_prerelease_versions(version in semver_with_prerelease_strategy()) {
///         // Generated versions like "1.2.3-alpha.1"
///     }
/// }
/// ```
pub fn semver_with_prerelease_strategy() -> impl Strategy<Value = String> {
    (0u32..100, 0u32..100, 0u32..100, prop::option::of("[a-z]{3,8}"), prop::option::of(0u32..100))
        .prop_map(|(major, minor, patch, pre, pre_num)| {
            let mut version = format!("{}.{}.{}", major, minor, patch);
            if let Some(pre_tag) = pre {
                version.push('-');
                version.push_str(&pre_tag);
                if let Some(num) = pre_num {
                    version.push('.');
                    version.push_str(&num.to_string());
                }
            }
            version
        })
}

/// Generates semantic versions with build metadata
///
/// # Returns
///
/// A proptest strategy that generates versions with build metadata
///
/// # Examples
///
/// ```rust,ignore
/// proptest! {
///     #[test]
///     fn test_build_metadata(version in semver_with_build_strategy()) {
///         // Generated versions like "1.2.3+build.123"
///     }
/// }
/// ```
pub fn semver_with_build_strategy() -> impl Strategy<Value = String> {
    (0u32..100, 0u32..100, 0u32..100, prop::option::of(0u32..1000)).prop_map(
        |(major, minor, patch, build)| {
            let mut version = format!("{}.{}.{}", major, minor, patch);
            if let Some(build_num) = build {
                version.push_str(&format!("+build.{}", build_num));
            }
            version
        },
    )
}

/// Generates valid NPM package names
///
/// Generates package names that follow NPM naming conventions:
/// - Lowercase letters, numbers, hyphens, and underscores
/// - May have a scope prefix (e.g., @scope/package)
///
/// # Returns
///
/// A proptest strategy that generates package names
///
/// # Examples
///
/// ```rust,ignore
/// proptest! {
///     #[test]
///     fn test_package_names(name in package_name_strategy()) {
///         // Generated names like "my-package", "@scope/package"
///     }
/// }
/// ```
pub fn package_name_strategy() -> impl Strategy<Value = String> {
    let simple_name = "[a-z][a-z0-9-_]{2,30}";
    let scoped_name = "@[a-z][a-z0-9-]{2,20}/[a-z][a-z0-9-_]{2,30}";

    prop::string::string_regex(&format!("({}|{})", simple_name, scoped_name)).expect("valid regex")
}

/// Generates conventional commit types
///
/// # Returns
///
/// A proptest strategy that generates commit types
pub fn commit_type_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "feat".to_string(),
        "fix".to_string(),
        "docs".to_string(),
        "style".to_string(),
        "refactor".to_string(),
        "perf".to_string(),
        "test".to_string(),
        "build".to_string(),
        "ci".to_string(),
        "chore".to_string(),
        "revert".to_string(),
    ])
}

/// Generates optional scope strings for conventional commits
///
/// # Returns
///
/// A proptest strategy that generates optional scopes
pub fn commit_scope_strategy() -> impl Strategy<Value = Option<String>> {
    prop::option::of(prop::string::string_regex("[a-z][a-z0-9-]{2,15}").expect("valid regex"))
}

/// Generates commit message descriptions
///
/// # Returns
///
/// A proptest strategy that generates commit descriptions
pub fn commit_description_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9 ,.-]{10,80}").expect("valid regex")
}

/// Generates conventional commit messages
///
/// Generates commit messages that follow the conventional commit format:
/// - type(scope): description
/// - May include breaking change marker
///
/// # Returns
///
/// A proptest strategy that generates conventional commit messages
///
/// # Examples
///
/// ```rust,ignore
/// proptest! {
///     #[test]
///     fn test_conventional_commits(message in conventional_commit_strategy()) {
///         // Generated messages like "feat(api): add new endpoint"
///     }
/// }
/// ```
pub fn conventional_commit_strategy() -> impl Strategy<Value = String> {
    (
        commit_type_strategy(),
        commit_scope_strategy(),
        commit_description_strategy(),
        prop::bool::ANY,
    )
        .prop_map(|(commit_type, scope, description, breaking)| {
            let breaking_marker = if breaking { "!" } else { "" };
            match scope {
                Some(s) => {
                    format!("{}({}){}: {}", commit_type, s, breaking_marker, description)
                }
                None => format!("{}{}: {}", commit_type, breaking_marker, description),
            }
        })
}

/// Generates file paths
///
/// Generates Unix-style file paths with various depths
///
/// # Returns
///
/// A proptest strategy that generates file paths
///
/// # Examples
///
/// ```rust,ignore
/// proptest! {
///     #[test]
///     fn test_file_paths(path in file_path_strategy()) {
///         // Generated paths like "src/module/file.rs"
///     }
/// }
/// ```
pub fn file_path_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-z][a-z0-9-]{2,15}", 1..5).prop_map(|parts| {
        let mut path = parts.join("/");
        path.push_str(".txt");
        path
    })
}

/// Generates version bump types
///
/// # Returns
///
/// A proptest strategy that generates version bump types
pub fn version_bump_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "major".to_string(),
        "minor".to_string(),
        "patch".to_string(),
        "none".to_string(),
    ])
}

/// Generates environment names
///
/// # Returns
///
/// A proptest strategy that generates environment names
pub fn environment_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "development".to_string(),
        "staging".to_string(),
        "production".to_string(),
        "test".to_string(),
        "qa".to_string(),
    ])
}

/// Generates lists of environments
///
/// # Returns
///
/// A proptest strategy that generates lists of environment names
pub fn environment_list_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(environment_strategy(), 1..4)
}

/// Generates commit hashes (SHA-1 style)
///
/// # Returns
///
/// A proptest strategy that generates 40-character hexadecimal strings
///
/// # Examples
///
/// ```rust,ignore
/// proptest! {
///     #[test]
///     fn test_commit_hashes(hash in commit_hash_strategy()) {
///         assert_eq!(hash.len(), 40);
///     }
/// }
/// ```
pub fn commit_hash_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-f0-9]{40}").expect("valid regex")
}

/// Generates short commit hashes (7 characters)
///
/// # Returns
///
/// A proptest strategy that generates 7-character hexadecimal strings
pub fn short_commit_hash_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-f0-9]{7}").expect("valid regex")
}

/// Generates author names
///
/// # Returns
///
/// A proptest strategy that generates author names
pub fn author_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Z][a-z]+ [A-Z][a-z]+").expect("valid regex")
}

/// Generates author emails
///
/// # Returns
///
/// A proptest strategy that generates email addresses
pub fn author_email_strategy() -> impl Strategy<Value = String> {
    (
        prop::string::string_regex("[a-z][a-z0-9]{2,10}").expect("valid regex"),
        prop::string::string_regex("[a-z]{3,10}").expect("valid regex"),
        prop::string::string_regex("[a-z]{2,5}").expect("valid regex"),
    )
        .prop_map(|(user, domain, tld)| format!("{}@{}.{}", user, domain, tld))
}

/// Generates Git branch names
///
/// # Returns
///
/// A proptest strategy that generates Git branch names
pub fn branch_name_strategy() -> impl Strategy<Value = String> {
    proptest::prop_oneof![
        prop::sample::select(vec!["main".to_string(), "develop".to_string(), "master".to_string()]),
        prop::string::string_regex("(feature|bugfix|hotfix)/[a-z][a-z0-9-]{5,20}")
            .expect("valid regex"),
    ]
}

/// Generates changeset IDs
///
/// Generates UUID-like strings for changeset identifiers
///
/// # Returns
///
/// A proptest strategy that generates changeset IDs
pub fn changeset_id_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}")
        .expect("valid regex")
}

/// Generates dependency version specs
///
/// Generates version specifications as used in package.json
///
/// # Returns
///
/// A proptest strategy that generates version specs
///
/// # Examples
///
/// ```rust,ignore
/// proptest! {
///     #[test]
///     fn test_version_specs(spec in version_spec_strategy()) {
///         // Generated specs like "^1.2.3", "~2.0.0", ">=3.0.0"
///     }
/// }
/// ```
pub fn version_spec_strategy() -> impl Strategy<Value = String> {
    (prop::sample::select(vec!["^", "~", ">=", "<=", "="]), 0u32..100, 0u32..100, 0u32..100)
        .prop_map(|(prefix, major, minor, patch)| {
            format!("{}{}.{}.{}", prefix, major, minor, patch)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    proptest! {
        #[test]
        fn test_semver_strategy_generates_valid_versions(version in semver_strategy()) {
            assert!(Version::parse(&version).is_ok());
        }

        #[test]
        fn test_semver_with_prerelease_is_parseable(version in semver_with_prerelease_strategy()) {
            assert!(Version::parse(&version).is_ok());
        }

        #[test]
        fn test_package_names_are_valid(name in package_name_strategy()) {
            assert!(!name.is_empty());
            assert!(name.len() >= 3);
            // Package names should be lowercase
            assert_eq!(name.to_lowercase(), name);
        }

        #[test]
        fn test_conventional_commits_have_colon(message in conventional_commit_strategy()) {
            assert!(message.contains(':'));
        }

        #[test]
        fn test_commit_hashes_are_40_chars(hash in commit_hash_strategy()) {
            assert_eq!(hash.len(), 40);
            assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        }

        #[test]
        fn test_short_hashes_are_7_chars(hash in short_commit_hash_strategy()) {
            assert_eq!(hash.len(), 7);
            assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        }

        #[test]
        fn test_author_emails_have_at_sign(email in author_email_strategy()) {
            assert!(email.contains('@'));
            assert!(email.contains('.'));
        }

        #[test]
        fn test_file_paths_have_extension(path in file_path_strategy()) {
            assert!(path.ends_with(".txt"));
        }

        #[test]
        fn test_environment_list_not_empty(envs in environment_list_strategy()) {
            assert!(!envs.is_empty());
        }
    }
}
