use std::{fs::read_to_string, path::PathBuf};

use git_cliff_core::{
    changelog::Changelog,
    commit::{Commit as GitCommit, Signature},
    config::{Config, GitConfig},
    release::Release,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use ws_git::repo::RepositoryCommit;
use ws_pkg::package::PackageInfo;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConventionalPackage {
    pub package_info: PackageInfo,
    pub conventional_commits: Value,
    pub changelog_output: String,
}

#[derive(Debug, Clone)]
pub struct ConventionalPackageOptions {
    pub config: Config,
    pub commits: Vec<RepositoryCommit>,
    pub version: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Struct representing the bump package.
pub struct RecommendBumpPackage {
    pub from: String,
    pub to: String,
    pub package_info: PackageInfo,
    pub conventional: ConventionalPackage,
    pub changed_files: Vec<String>,
    pub deploy_to: Vec<String>,
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

pub fn get_conventional_by_package(
    package_info: &PackageInfo,
    options: &ConventionalPackageOptions,
) -> ConventionalPackage {
    let changelog_dir =
        PathBuf::from(package_info.package_path.to_string()).join(String::from("CHANGELOG.md"));

    let commits = process_commits(&options.commits, &options.config.git);

    let mut conventional_package = ConventionalPackage {
        package_info: package_info.to_owned(),
        conventional_commits: json!([]),
        changelog_output: String::new(),
    };

    let changelog_output = if changelog_dir.exists() {
        let changelog_content = read_to_string(&changelog_dir).unwrap();
        prepend_generate_changelog(
            &commits,
            &options.config,
            &changelog_content,
            options.version.clone(),
        )
    } else {
        generate_changelog(&commits, &options.config, options.version.clone())
    };

    conventional_package.changelog_output = changelog_output.to_string();
    conventional_package.conventional_commits =
        serde_json::to_value(&commits).expect("Error convert conventional commits to Json");

    conventional_package
}
