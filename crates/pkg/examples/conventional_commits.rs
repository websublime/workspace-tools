//! Example demonstrating conventional commit parsing and Git integration.
//!
//! This example shows how to use the ConventionalCommitService to:
//! - Retrieve commits from Git history
//! - Parse them according to conventional commit specification
//! - Calculate version bumps based on commit types
//! - Analyze commit patterns for changeset creation
//!
//! Run with: cargo run --example conventional_commits

use chrono::Utc;
use std::collections::HashMap;
use sublime_git_tools::Repo;
use sublime_pkg_tools::{
    config::{ConventionalCommitType, ConventionalConfig, PackageToolsConfig},
    conventional::{CommitAnalysis, ConventionalCommitService},
    VersionBump,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    // env_logger::init(); // Optional logging setup

    println!("🔍 Conventional Commit Integration Example");
    println!("==========================================\n");

    // Example 1: Basic usage with default configuration
    println!("📋 Example 1: Basic Usage");
    println!("-------------------------");
    basic_usage_example().await?;

    println!("\n📋 Example 2: Custom Configuration");
    println!("----------------------------------");
    custom_config_example().await?;

    println!("\n📋 Example 3: Commits Between References");
    println!("----------------------------------------");
    commits_between_example().await?;

    println!("\n📋 Example 4: Commit Analysis");
    println!("-----------------------------");
    commit_analysis_example().await?;

    println!("\n📋 Example 5: Version Bump Calculation");
    println!("--------------------------------------");
    version_bump_example().await?;

    println!("\n✅ All examples completed successfully!");
    Ok(())
}

/// Demonstrates basic conventional commit parsing with default configuration.
async fn basic_usage_example() -> Result<(), Box<dyn std::error::Error>> {
    // Try to open current directory as a Git repository
    match Repo::open(".") {
        Ok(repo) => {
            let config = PackageToolsConfig::default();
            let service = ConventionalCommitService::new(repo, config)?;

            println!("✅ Opened Git repository in current directory");

            // Try to get commits since last tag
            match service.get_commits_since_last_tag().await {
                Ok(commits) => {
                    println!("📦 Found {} conventional commits since last tag", commits.len());

                    for (i, commit) in commits.iter().take(3).enumerate() {
                        println!(
                            "  {}. {} ({}): {}",
                            i + 1,
                            commit.commit_type,
                            &commit.hash[..8],
                            commit.description
                        );
                        if commit.breaking {
                            println!("     ⚠️  Breaking change!");
                        }
                    }

                    if commits.len() > 3 {
                        println!("     ... and {} more commits", commits.len() - 3);
                    }

                    // Calculate suggested version bump
                    let suggested_bump = service.calculate_version_bump(&commits);
                    println!("🎯 Suggested version bump: {:?}", suggested_bump);
                }
                Err(e) => {
                    println!("⚠️  Could not retrieve commits: {}", e);
                    println!(
                        "   This might be because there are no tags or commits in the repository"
                    );
                }
            }
        }
        Err(_) => {
            println!("⚠️  Current directory is not a Git repository");
            println!("   This example works best when run from within a Git repository");
        }
    }

    Ok(())
}

/// Demonstrates custom configuration for commit types.
async fn custom_config_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom configuration with additional commit types
    let mut custom_types = HashMap::new();

    // Add standard types
    custom_types.insert(
        "feat".to_string(),
        ConventionalCommitType {
            bump: "minor".to_string(),
            changelog: true,
            changelog_title: Some("🚀 Features".to_string()),
            breaking: false,
        },
    );

    custom_types.insert(
        "fix".to_string(),
        ConventionalCommitType {
            bump: "patch".to_string(),
            changelog: true,
            changelog_title: Some("🐛 Bug Fixes".to_string()),
            breaking: false,
        },
    );

    // Add custom types
    custom_types.insert(
        "security".to_string(),
        ConventionalCommitType {
            bump: "patch".to_string(),
            changelog: true,
            changelog_title: Some("🔒 Security".to_string()),
            breaking: false,
        },
    );

    custom_types.insert(
        "deprecate".to_string(),
        ConventionalCommitType {
            bump: "minor".to_string(),
            changelog: true,
            changelog_title: Some("⚠️  Deprecations".to_string()),
            breaking: false,
        },
    );

    let conventional_config = ConventionalConfig {
        types: custom_types,
        parse_breaking_changes: true,
        require_conventional_commits: false,
        breaking_change_patterns: vec![
            "BREAKING CHANGE:".to_string(),
            "BREAKING-CHANGE:".to_string(),
            "CUSTOM BREAK:".to_string(),
        ],
        default_bump_type: "patch".to_string(),
    };

    let mut config = PackageToolsConfig::default();
    config.conventional = conventional_config;

    // Create service with custom configuration
    match Repo::open(".") {
        Ok(repo) => {
            let service = ConventionalCommitService::new(repo, config)?;
            println!("✅ Created service with custom configuration");
            println!("   • Added custom commit types: security, deprecate");
            println!("   • Custom breaking change patterns");
            println!("   • Enhanced changelog titles with emojis");

            // Demonstrate parsing with custom types
            let parser = sublime_pkg_tools::conventional::ConventionalCommitParser::with_config(
                service.config().conventional.clone(),
            )?;

            // Test custom commit types
            let test_commits = vec![
                ("security: fix XSS vulnerability", "security"),
                ("deprecate: mark old API as deprecated", "deprecate"),
                ("feat: add new user management", "feat"),
                ("fix: resolve memory leak", "fix"),
            ];

            println!("\n🧪 Testing custom commit type parsing:");
            for (message, expected_type) in test_commits {
                match parser.parse(
                    message,
                    "test123".to_string(),
                    "Test Author".to_string(),
                    Utc::now(),
                ) {
                    Ok(commit) => {
                        let bump = parser.get_version_bump(&commit.commit_type, commit.breaking);
                        let section =
                            parser.get_changelog_section(&commit.commit_type).unwrap_or("Other");

                        println!(
                            "  ✅ {}: {} → {:?} bump → {}",
                            expected_type, commit.description, bump, section
                        );
                    }
                    Err(e) => {
                        println!("  ❌ Failed to parse '{}': {}", message, e);
                    }
                }
            }
        }
        Err(_) => {
            println!("⚠️  Current directory is not a Git repository");
        }
    }

    Ok(())
}

/// Demonstrates getting commits between two references.
async fn commits_between_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Demonstrating commits between references functionality:");

    match Repo::open(".") {
        Ok(repo) => {
            let config = PackageToolsConfig::default();
            let service = ConventionalCommitService::new(repo, config)?;

            println!("✅ Successfully opened Git repository");

            // Try to get commits between different references
            let test_cases = vec![
                ("HEAD~5", "HEAD", "last 5 commits"),
                ("HEAD~10", "HEAD~5", "commits 5-10 back"),
                ("main", "HEAD", "commits ahead of main"),
            ];

            for (from_ref, to_ref, description) in test_cases {
                println!("\n🔄 Testing {}: {} → {}", description, from_ref, to_ref);

                match service.get_commits_between(from_ref, to_ref).await {
                    Ok(commits) => {
                        println!("   📦 Found {} conventional commits", commits.len());

                        for (i, commit) in commits.iter().take(3).enumerate() {
                            println!(
                                "     {}. {} ({}): {}",
                                i + 1,
                                commit.commit_type,
                                &commit.hash[..8],
                                commit.description
                            );
                            if commit.breaking {
                                println!("        ⚠️  Breaking change detected!");
                            }
                        }

                        if commits.len() > 3 {
                            println!("        ... and {} more commits", commits.len() - 3);
                        }

                        if !commits.is_empty() {
                            let suggested_bump = service.calculate_version_bump(&commits);
                            println!("     🎯 Suggested version bump: {:?}", suggested_bump);
                        }
                    }
                    Err(e) => {
                        println!("   ⚠️  Could not get commits: {}", e);
                        println!(
                            "      This might be because the references don't exist or are invalid"
                        );
                    }
                }
            }

            // Demonstrate practical use case: feature branch analysis
            println!("\n🌟 Practical Example: Feature Branch Analysis");
            println!("   (This would typically be used in CI/CD to analyze a PR)");

            match service.get_commits_between("origin/main", "HEAD").await {
                Ok(commits) => {
                    if commits.is_empty() {
                        println!("   📭 No commits ahead of origin/main");
                    } else {
                        println!("   📈 Found {} commits in this branch/PR", commits.len());

                        let analysis = service.analyze_commits(&commits);
                        println!("   📊 Analysis Results:");
                        println!("      • Suggested bump: {:?}", analysis.suggested_bump);
                        println!(
                            "      • Breaking changes: {}",
                            if analysis.has_breaking_changes { "Yes" } else { "No" }
                        );

                        if !analysis.type_distribution.is_empty() {
                            println!("      • Commit types:");
                            for (commit_type, count) in &analysis.type_distribution {
                                println!("        - {}: {} commits", commit_type, count);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("   ⚠️  Could not analyze branch: {}", e);
                    println!("      (This is normal if not in a Git repository with origin/main)");
                }
            }
        }
        Err(_) => {
            println!("⚠️  Current directory is not a Git repository");
            println!("   The get_commits_between functionality works with real Git repositories");
            println!("   and can compare any two valid Git references (branches, tags, commits)");

            println!("\n💡 Example use cases:");
            println!("   • Compare feature branch against main");
            println!("   • Get commits between two release tags");
            println!("   • Analyze changes in a specific commit range");
            println!("   • Calculate version bumps for pull requests");
        }
    }

    Ok(())
}

/// Demonstrates comprehensive commit analysis.
async fn commit_analysis_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create some example commits for analysis
    let example_commits = vec![
        sublime_pkg_tools::conventional::ConventionalCommit {
            commit_type: sublime_pkg_tools::conventional::CommitType::Feat,
            scope: Some("auth".to_string()),
            breaking: false,
            description: "add OAuth2 authentication".to_string(),
            body: Some("Implemented OAuth2 flow with refresh tokens".to_string()),
            footer: None,
            hash: "abc123def".to_string(),
            author: "alice@example.com".to_string(),
            date: Utc::now(),
        },
        sublime_pkg_tools::conventional::ConventionalCommit {
            commit_type: sublime_pkg_tools::conventional::CommitType::Fix,
            scope: Some("ui".to_string()),
            breaking: true,
            description: "resolve layout issues".to_string(),
            body: Some(
                "BREAKING CHANGE: Changed CSS class names for better consistency".to_string(),
            ),
            footer: None,
            hash: "def456ghi".to_string(),
            author: "bob@example.com".to_string(),
            date: Utc::now(),
        },
        sublime_pkg_tools::conventional::ConventionalCommit {
            commit_type: sublime_pkg_tools::conventional::CommitType::Perf,
            scope: Some("api".to_string()),
            breaking: false,
            description: "optimize database queries".to_string(),
            body: Some("Reduced query time by 40% using proper indexing".to_string()),
            footer: None,
            hash: "ghi789jkl".to_string(),
            author: "charlie@example.com".to_string(),
            date: Utc::now(),
        },
        sublime_pkg_tools::conventional::ConventionalCommit {
            commit_type: sublime_pkg_tools::conventional::CommitType::Docs,
            scope: None,
            breaking: false,
            description: "update API documentation".to_string(),
            body: None,
            footer: None,
            hash: "jkl012mno".to_string(),
            author: "diana@example.com".to_string(),
            date: Utc::now(),
        },
    ];

    // Analyze commits
    match Repo::open(".") {
        Ok(repo) => {
            let config = PackageToolsConfig::default();
            let service = ConventionalCommitService::new(repo, config)?;
            let analysis = service.analyze_commits(&example_commits);

            print_commit_analysis(&analysis);

            // Group commits by type
            let grouped = service.group_commits_by_type(&example_commits);
            println!("\n📊 Commits grouped by type (changelog-worthy only):");
            for (commit_type, commits) in &grouped {
                println!("  {} ({} commits):", commit_type, commits.len());
                for commit in commits {
                    let scope_str =
                        commit.scope.as_ref().map(|s| format!("({})", s)).unwrap_or_default();
                    println!("    • {}{}: {}", commit_type, scope_str, commit.description);
                    if commit.breaking {
                        println!("      ⚠️  Breaking change");
                    }
                }
            }
        }
        Err(_) => {
            println!("⚠️  Current directory is not a Git repository");
            println!("   Showing analysis of example commits anyway:");

            // Create a mock service for demonstration
            let temp_dir = tempfile::TempDir::new()?;
            let repo = Repo::create(temp_dir.path().to_str().unwrap())?;
            let config = PackageToolsConfig::default();
            let service = ConventionalCommitService::new(repo, config)?;
            let analysis = service.analyze_commits(&example_commits);
            print_commit_analysis(&analysis);
        }
    }

    Ok(())
}

/// Demonstrates version bump calculation logic.
async fn version_bump_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create different scenarios for version bump calculation
    let scenarios = vec![
        (
            "Only patch fixes",
            vec![
                sublime_pkg_tools::conventional::CommitType::Fix,
                sublime_pkg_tools::conventional::CommitType::Fix,
            ],
            VersionBump::Patch,
        ),
        (
            "Mix of fixes and features",
            vec![
                sublime_pkg_tools::conventional::CommitType::Fix,
                sublime_pkg_tools::conventional::CommitType::Feat,
                sublime_pkg_tools::conventional::CommitType::Fix,
            ],
            VersionBump::Minor,
        ),
        (
            "Breaking change present",
            vec![
                sublime_pkg_tools::conventional::CommitType::Feat,
                sublime_pkg_tools::conventional::CommitType::Fix,
            ],
            VersionBump::Major, // We'll mark one as breaking
        ),
        (
            "Non-versioning commits only",
            vec![
                sublime_pkg_tools::conventional::CommitType::Docs,
                sublime_pkg_tools::conventional::CommitType::Style,
                sublime_pkg_tools::conventional::CommitType::Test,
            ],
            VersionBump::None,
        ),
    ];

    // Create a service for testing
    let temp_dir = tempfile::TempDir::new()?;
    let repo = Repo::create(temp_dir.path().to_str().unwrap())?;
    let config = PackageToolsConfig::default();
    let service = ConventionalCommitService::new(repo, config)?;

    println!("🎯 Version Bump Calculation Scenarios:");
    println!("=====================================");

    for (scenario_name, commit_types, expected_bump) in scenarios {
        let mut commits = Vec::new();
        for (i, commit_type) in commit_types.iter().enumerate() {
            let breaking = scenario_name == "Breaking change present" && i == 0;
            commits.push(sublime_pkg_tools::conventional::ConventionalCommit {
                commit_type: commit_type.clone(),
                scope: None,
                breaking,
                description: format!("commit {}", i + 1),
                body: None,
                footer: None,
                hash: format!("hash{}", i),
                author: "test@example.com".to_string(),
                date: Utc::now(),
            });
        }

        let calculated_bump = service.calculate_version_bump(&commits);
        let status = if calculated_bump == expected_bump { "✅" } else { "❌" };

        println!("\n{} Scenario: {}", status, scenario_name);
        println!("   Commits: {:?}", commit_types);
        println!("   Expected: {:?}", expected_bump);
        println!("   Calculated: {:?}", calculated_bump);

        if calculated_bump != expected_bump {
            println!("   ⚠️  Mismatch detected!");
        }
    }

    Ok(())
}

/// Helper function to print commit analysis results.
fn print_commit_analysis(analysis: &CommitAnalysis) {
    println!("📈 Commit Analysis Results:");
    println!("==========================");
    println!("📊 Total commits: {}", analysis.total_commits);
    println!("✅ Conventional commits: {}", analysis.conventional_commits);
    println!("🎯 Suggested version bump: {:?}", analysis.suggested_bump);
    println!("⚠️  Breaking changes: {}", if analysis.has_breaking_changes { "Yes" } else { "No" });

    if !analysis.breaking_changes.is_empty() {
        println!("💥 Breaking change commits:");
        for breaking_commit in &analysis.breaking_changes {
            println!(
                "   • {} ({}): {}",
                breaking_commit.commit_type,
                &breaking_commit.hash[..8],
                breaking_commit.description
            );
        }
    }

    if !analysis.type_distribution.is_empty() {
        println!("📋 Commit type distribution:");
        let mut sorted_types: Vec<_> = analysis.type_distribution.iter().collect();
        sorted_types.sort_by_key(|(_, count)| *count);
        sorted_types.reverse();

        for (commit_type, count) in sorted_types {
            println!("   • {}: {} commits", commit_type, count);
        }
    }
}
