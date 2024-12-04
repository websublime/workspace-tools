use git_cliff_core::{
    changelog::Changelog,
    commit::{Commit as GitCommit, Signature},
    config::{Config, GitConfig},
    release::Release,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ws_git::repo::RepositoryCommit;
use ws_pkg::package::PackageInfo;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConventionalPackage {
    pub package_info: PackageInfo,
    pub conventional_config: Value,
    pub conventional_commits: Value,
    pub changelog_output: String,
}

#[derive(Debug, Clone)]
pub struct ConventionalPackageOptions {
    pub version: Option<String>,
    pub title: Option<String>,
}

fn generate_changelog(commits: &[GitCommit], config: &Config, version: Option<String>) -> String {
    let releases = Release { version, commits: commits.to_vec(), ..Release::default() };

    let changelog = Changelog::new(vec![releases], config);
    let mut changelog_output = Vec::new();

    changelog
        .expect("Failed to init changelog")
        .generate(&mut changelog_output)
        .expect("Failed to generate changelog");

    String::from_utf8(changelog_output).unwrap_or_default()
}

/// Prepend changelog output
fn prepend_generate_changelog(
    commits: &[GitCommit],
    config: &Config,
    changelog_content: &String,
    version: Option<String>,
) -> String {
    let releases = Release { version, commits: commits.to_vec(), ..Release::default() };

    let changelog = Changelog::new(vec![releases], config);
    let mut changelog_output = Vec::new();

    changelog
        .expect("Failed to init changelog")
        .prepend(changelog_content.to_string(), &mut changelog_output)
        .expect("Failed to prepend to changelog");

    String::from_utf8(changelog_output).unwrap_or_default()
}

fn process_commits<'a>(commits: &[RepositoryCommit], config: &GitConfig) -> Vec<GitCommit<'a>> {
    commits
        .iter()
        .map(|commit| {
            let timestamp = chrono::DateTime::parse_from_rfc2822(&commit.author_date).unwrap();

            let git_commit = GitCommit {
                id: commit.hash.to_string(),
                message: commit.message.to_string(),
                author: Signature {
                    name: Some(commit.author_name.to_string()),
                    email: Some(commit.author_email.to_string()),
                    timestamp: timestamp.timestamp(),
                },
                ..GitCommit::default()
            };

            git_commit.process(config).unwrap()
        })
        .collect::<Vec<GitCommit>>()
}
