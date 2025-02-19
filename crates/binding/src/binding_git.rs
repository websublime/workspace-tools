#![allow(clippy::bind_instead_of_map)]
#![allow(clippy::needless_pass_by_value)]
use napi::{bindgen_prelude::Array, Error, Result};
use napi::{Env, Status};
use std::path::Path;

use ws_git::repo::Repository;

pub enum RepositoryError {
    FailCreateObject,
    FailParsing,
    FailRepositoryInit,
    FailCreateBranch,
    FailCheckoutBranch,
    FailMergeBranch,
    FailAddAll,
    FailAddFile,
    FailFetch,
    FailMergeBase,
    FailRevParse,
    FailLog,
    FailTag,
    FailPush,
    FailCommit,
    FailDiff,
    InvalidVcsRepository,
    InvalidConfigRepository,
    NapiError(Error<Status>),
}

impl AsRef<str> for RepositoryError {
    fn as_ref(&self) -> &str {
        match self {
            Self::NapiError(e) => e.status.as_ref(),
            Self::FailCreateObject => "Failed to create object",
            Self::FailParsing => "Failed to parse struct",
            Self::FailRepositoryInit => "Failed to initialize repository",
            Self::InvalidVcsRepository => "Invalid VCS repository",
            Self::InvalidConfigRepository => "Failed to configure repository",
            Self::FailCreateBranch => "Failed to create branch",
            Self::FailCheckoutBranch => "Failed to checkout branch",
            Self::FailMergeBranch => "Failed to merge branch",
            Self::FailAddAll => "Failed to add all files",
            Self::FailAddFile => "Failed to add file",
            Self::FailFetch => "Failed to fetch",
            Self::FailMergeBase => "Failed to merge base",
            Self::FailRevParse => "Failed to rev parse",
            Self::FailLog => "Failed to log",
            Self::FailTag => "Failed to tag",
            Self::FailPush => "Failed to push",
            Self::FailCommit => "Failed to commit",
            Self::FailDiff => "Failed to diff",
        }
    }
}

#[napi(js_name = "repoInit", ts_return_type = "Result<boolean>")]
pub fn js_repo_init(
    initial_branch: String,
    username: String,
    email: String,
    cwd: String,
) -> Result<bool, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let init = repository
        .init(initial_branch.as_str(), username.as_str(), email.as_str())
        .or_else(|_| {
            Err(Error::new(
                RepositoryError::FailRepositoryInit,
                format!("Failed to initialize repository in {cwd}"),
            ))
        })?;

    Ok(init)
}

#[napi(js_name = "isVcsRepository", ts_return_type = "Result<boolean>")]
pub fn js_is_vcs_repository(cwd: String) -> Result<bool, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let is_vcs = repository.is_vcs().or_else(|_| {
        Err(Error::new(RepositoryError::InvalidVcsRepository, format!("Invalid VCS in {cwd}")))
    })?;

    Ok(is_vcs)
}

#[napi(js_name = "repoConfig", ts_return_type = "Result<boolean>")]
pub fn js_repo_config(
    username: String,
    email: String,
    cwd: String,
) -> Result<bool, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let config = repository.config(username.as_str(), email.as_str()).or_else(|_| {
        Err(Error::new(
            RepositoryError::InvalidConfigRepository,
            format!("Failed to configure: {username} and {email} on {cwd}"),
        ))
    })?;

    Ok(config)
}

#[napi(js_name = "repoCreateBranch", ts_return_type = "Result<boolean>")]
pub fn js_repo_create_branch(branch: String, cwd: String) -> Result<bool, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let create_branch = repository.create_branch(branch.as_str()).or_else(|_| {
        Err(Error::new(
            RepositoryError::FailCreateBranch,
            format!("Failed to create branch: {branch} in {cwd}"),
        ))
    })?;

    Ok(create_branch)
}

#[napi(js_name = "repoCheckout", ts_return_type = "Result<boolean>")]
pub fn js_repo_checkout(branch: String, cwd: String) -> Result<bool, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let checkout = repository.checkout(branch.as_str()).or_else(|_| {
        Err(Error::new(
            RepositoryError::FailCheckoutBranch,
            format!("Failed to checkout branch: {branch} in {cwd}"),
        ))
    })?;

    Ok(checkout)
}

#[napi(js_name = "repoMerge", ts_return_type = "Result<boolean>")]
pub fn js_repo_merge(branch: String, cwd: String) -> Result<bool, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let merge = repository.merge(branch.as_str()).or_else(|_| {
        Err(Error::new(
            RepositoryError::FailMergeBranch,
            format!("Failed to merge branch: {branch} in {cwd}"),
        ))
    })?;

    Ok(merge)
}

#[napi(js_name = "repoAddAll", ts_return_type = "Result<boolean>")]
pub fn js_repo_add_all(cwd: String) -> Result<bool, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let add_all = repository.add_all().or_else(|_| {
        Err(Error::new(RepositoryError::FailAddAll, format!("Failed to add all files in {cwd}")))
    })?;

    Ok(add_all)
}

#[napi(js_name = "repoAdd", ts_return_type = "Result<boolean>")]
pub fn js_repo_add(filepath: String, cwd: String) -> Result<bool, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let add = repository.add(Path::new(filepath.as_str())).or_else(|_| {
        Err(Error::new(
            RepositoryError::FailAddFile,
            format!("Failed to add file: {filepath} in {cwd}"),
        ))
    })?;

    Ok(add)
}

#[napi(js_name = "repoFetchAll", ts_return_type = "Result<boolean>")]
pub fn js_repo_fetch_all(cwd: String, fetch_tags: Option<bool>) -> Result<bool, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let fetch = repository.fetch_all(fetch_tags).or_else(|_| {
        Err(Error::new(RepositoryError::FailFetch, format!("Failed to fetch all in {cwd}")))
    })?;

    Ok(fetch)
}

#[napi(js_name = "repoGetDivergedCommit", ts_return_type = "Result<string>")]
pub fn js_repo_get_diverged_commit(sha: String, cwd: String) -> Result<String, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let diverged_commit = repository.get_diverged_commit(sha.as_str()).or_else(|_| {
        Err(Error::new(RepositoryError::FailMergeBase, format!("Failed to merge-base in {cwd}")))
    })?;

    Ok(diverged_commit)
}

#[napi(js_name = "repoGetCurrentSha", ts_return_type = "Result<string>")]
pub fn js_repo_get_current_sha(cwd: String) -> Result<String, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let current_sha = repository.get_current_sha().or_else(|_| {
        Err(Error::new(
            RepositoryError::FailRevParse,
            format!("Failed to get current sha in {cwd}"),
        ))
    })?;

    Ok(current_sha)
}

#[napi(js_name = "repoGetPreviousSha", ts_return_type = "Result<string>")]
pub fn js_repo_get_previous_sha(cwd: String) -> Result<String, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let previous_sha = repository.get_previous_sha().or_else(|_| {
        Err(Error::new(
            RepositoryError::FailRevParse,
            format!("Failed to get previous sha in {cwd}"),
        ))
    })?;

    Ok(previous_sha)
}

#[napi(js_name = "repoGetFirstSha", ts_return_type = "Result<string>")]
pub fn js_repo_get_first_sha(
    cwd: String,
    branch: Option<String>,
) -> Result<String, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let first_sha = repository.get_first_sha(branch).or_else(|_| {
        Err(Error::new(RepositoryError::FailLog, format!("Failed to get first sha in {cwd}")))
    })?;

    Ok(first_sha)
}

#[napi(js_name = "repoIsVcs", ts_return_type = "Result<boolean>")]
pub fn js_repo_is_vcs(cwd: String) -> Result<bool, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let is_unclean = repository.is_vcs().or_else(|_| {
        Err(Error::new(
            RepositoryError::FailRevParse,
            format!("Failed to check if repository is unclean in {cwd}"),
        ))
    })?;

    Ok(is_unclean)
}

#[napi(js_name = "repoGetCurrentBranch", ts_return_type = "Result<string|null>")]
pub fn js_repo_get_current_branch(cwd: String) -> Result<Option<String>, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let current_branch = repository.get_current_branch().or_else(|_| {
        Err(Error::new(
            RepositoryError::FailRevParse,
            format!("Failed to get current branch in {cwd}"),
        ))
    })?;

    Ok(current_branch)
}

#[napi(js_name = "repoGetBranchFromCommit", ts_return_type = "Result<string|null>")]
pub fn js_repo_get_branch_from_commit(
    sha: String,
    cwd: String,
) -> Result<Option<String>, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let branch = repository.get_branch_from_commit(sha.as_str()).or_else(|_| {
        Err(Error::new(
            RepositoryError::FailRevParse,
            format!("Failed to get branch from commit in {cwd}"),
        ))
    })?;

    Ok(branch)
}

#[napi(js_name = "repoCreateTag", ts_return_type = "Result<boolean>")]
pub fn js_repo_tag(
    tag: String,
    cwd: String,
    message: Option<String>,
) -> Result<bool, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let tag = repository.tag(tag.as_str(), message).or_else(|_| {
        Err(Error::new(RepositoryError::FailTag, format!("Failed to tag in {cwd}")))
    })?;

    Ok(tag)
}

#[napi(js_name = "repoPush", ts_return_type = "Result<boolean>")]
pub fn js_repo_push(cwd: String, follow_tags: Option<bool>) -> Result<bool, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let push = repository.push(follow_tags).or_else(|_| {
        Err(Error::new(RepositoryError::FailPush, format!("Failed to push in {cwd}")))
    })?;

    Ok(push)
}

#[napi(js_name = "repoCommit", ts_return_type = "Result<boolean>")]
pub fn js_repo_commit(
    cwd: String,
    message: String,
    body: Option<String>,
    footer: Option<String>,
) -> Result<bool, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let commit = repository.commit(message.as_str(), body, footer).or_else(|_| {
        Err(Error::new(RepositoryError::FailCommit, format!("Failed to commit in {cwd}")))
    })?;

    Ok(commit)
}

#[napi(js_name = "repoGetAllFilesChangedSinceSha", ts_return_type = "Result<string[]>")]
pub fn js_get_all_files_changed_since_sha(
    cwd: String,
    sha: String,
) -> Result<Vec<String>, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let files = repository.get_all_files_changed_since_sha(sha.as_str()).or_else(|_| {
        Err(Error::new(
            RepositoryError::FailDiff,
            format!("Failed to get all files changed since sha in {cwd}"),
        ))
    })?;

    Ok(files)
}

#[napi(js_name = "repoGetCommitsSince", ts_return_type = "Result<RepositoryCommit[]>")]
pub fn js_repo_get_commits_since(
    env: Env,
    cwd: String,
    since: Option<String>,
    relative: Option<String>,
) -> Result<Array, RepositoryError> {
    let mut array = env.create_array(0).or_else(|_| {
        Err(Error::new(RepositoryError::FailCreateObject, "Failed to create commits array object"))
    })?;
    let repository = Repository::from(cwd.as_str());
    let commits = repository.get_commits_since(since, relative).or_else(|_| {
        Err(Error::new(RepositoryError::FailLog, format!("Failed to get commits since in {cwd}")))
    })?;

    commits.iter().for_each(|commit| {
        let commit_value = serde_json::to_value(commit)
            .or_else(|_| {
                Err(Error::new(RepositoryError::FailParsing, "Failed to parse commits value"))
            })
            .unwrap();

        array.insert(commit_value).expect("Failed to insert commit value");
    });

    Ok(array)
}

#[napi(js_name = "repoGetTags", ts_return_type = "Result<RepositoryRemoteTags[]>")]
pub fn js_repo_get_tags(
    env: Env,
    cwd: String,
    local: Option<bool>,
) -> Result<Array, RepositoryError> {
    let mut array = env.create_array(0).or_else(|_| {
        Err(Error::new(RepositoryError::FailCreateObject, "Failed to create tags array object"))
    })?;
    let repository = Repository::from(cwd.as_str());
    let tags = repository.get_remote_or_local_tags(local).or_else(|_| {
        Err(Error::new(RepositoryError::FailTag, format!("Failed to get tags in {cwd}")))
    })?;

    tags.iter().for_each(|tag| {
        let tag_value = serde_json::to_value(tag)
            .or_else(|_| {
                Err(Error::new(RepositoryError::FailParsing, "Failed to parse tags value"))
            })
            .unwrap();

        array.insert(tag_value).expect("Failed to insert tag value");
    });

    Ok(array)
}

#[napi(js_name = "repoGetAllFilesChangedSinceBranch", ts_return_type = "Result<string[]>")]
pub fn js_repo_get_all_files_changed_since_branch(
    cwd: String,
    packages: Vec<String>,
    branch: String,
) -> Result<Vec<String>, RepositoryError> {
    let repository = Repository::from(cwd.as_str());
    let files = repository
        .get_all_files_changed_since_branch(&packages, branch.as_str())
        .or_else(|_| {
            Err(Error::new(
                RepositoryError::FailDiff,
                format!("Failed to get all files changed since branch in {cwd}"),
            ))
        })
        .unwrap();

    Ok(files)
}
