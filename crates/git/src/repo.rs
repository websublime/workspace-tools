use serde::{Deserialize, Serialize};
use std::{
    fs::canonicalize,
    path::{Path, PathBuf},
};
use ws_std::command::execute;

use super::error::RepositoryError;

impl From<&str> for Repository {
    fn from(root: &str) -> Self {
        let path_buff = PathBuf::from(root);
        let repo_path = &canonicalize(Path::new(path_buff.as_os_str())).expect("Invalid path");

        Repository { location: repo_path.clone() }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    location: PathBuf,
}

impl Repository {
    pub fn new(location: &Path) -> Self {
        let root = canonicalize(location.as_os_str()).expect("Invalid path");
        Self { location: root }
    }

    pub fn get_repo_path(&self) -> &Path {
        &self.location
    }

    pub fn init(
        &self,
        initial_branch: &str,
        username: &str,
        email: &str,
    ) -> Result<bool, RepositoryError> {
        let inited = execute(
            "git",
            self.location.as_path(),
            ["init", "--initial-branch", initial_branch],
            |_, output| Ok(output.status.success()),
        )?;

        if !inited {
            return Err(RepositoryError::InitializeFailure);
        }

        let configed = self.config(username, email)?;

        Ok(inited && configed)
    }

    pub fn config(&self, username: &str, email: &str) -> Result<bool, RepositoryError> {
        let user_config = execute(
            "git",
            self.location.as_path(),
            ["config", "user.name", username],
            |_, output| Ok(output.status.success()),
        )?;

        if !user_config {
            return Err(RepositoryError::ConfigUsernameFailure);
        }

        let email_config = execute(
            "git",
            self.location.as_path(),
            ["config", "user.email", email],
            |_, output| Ok(output.status.success()),
        )?;

        if !email_config {
            return Err(RepositoryError::ConfigEmailFailure);
        }

        let clrf_config = execute(
            "git",
            self.location.as_path(),
            ["config", "core.safecrlf", "true"],
            |_, output| Ok(output.status.success()),
        )?;

        let autocrlf_config = execute(
            "git",
            self.location.as_path(),
            ["config", "core.autocrlf", "input"],
            |_, output| Ok(output.status.success()),
        )?;

        let filemode_config = execute(
            "git",
            self.location.as_path(),
            ["config", "core.filemode", "false"],
            |_, output| Ok(output.status.success()),
        )?;

        Ok(user_config && email_config && clrf_config && autocrlf_config && filemode_config)
    }

    pub fn is_vcs(&self) -> Result<bool, RepositoryError> {
        Ok(execute(
            "git",
            self.location.as_path(),
            ["rev-parse", "--is-inside-work-tree"],
            |stdout, _| Ok(stdout.trim() == "true"),
        )?)
    }

    pub fn create_branch(&self, branch_name: &str) -> Result<bool, RepositoryError> {
        let branch_created = execute(
            "git",
            self.location.as_path(),
            ["checkout", "-b", branch_name],
            |_, output| Ok(output.status.success()),
        )?;

        if !branch_created {
            return Err(RepositoryError::BranchCreationFailure);
        }

        Ok(branch_created)
    }

    pub fn list_branches(&self) -> Result<String, RepositoryError> {
        let branches = execute(
            "git",
            self.location.as_path(),
            ["--no-pager", "branch", "-a"],
            |message, _| Ok(message.to_string()),
        )?;

        Ok(branches)
    }

    pub fn list_config(&self, config_type: &str) -> Result<String, RepositoryError> {
        let list = execute(
            "git",
            self.location.as_path(),
            ["--no-pager", "config", "--list", format!("--{config_type}").as_str()],
            |stdout, _| Ok(stdout.to_string()),
        )?;

        Ok(list)
    }

    pub fn checkout(&self, branch_name: &str) -> Result<bool, RepositoryError> {
        let branch_checkouted =
            execute("git", self.location.as_path(), ["checkout", branch_name], |_, output| {
                Ok(output.status.success())
            })?;

        if !branch_checkouted {
            return Err(RepositoryError::BranchCheckoutFailure);
        }

        Ok(branch_checkouted)
    }

    pub fn log(&self, target: Option<String>) -> Result<String, RepositoryError> {
        let mut args: Vec<String> = vec!["--no-pager".to_string(), "log".to_string()];

        if let Some(target_branch) = target {
            args.push(target_branch);
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let log = execute("git", self.location.as_path(), args_ref, |stdout, _| {
            Ok(stdout.trim().to_string())
        })?;

        Ok(log)
    }

    pub fn diff(&self, diff: Option<String>) -> Result<String, RepositoryError> {
        let diff = match diff {
            Some(diff) => diff,
            None => ".".to_string(),
        };

        let diff = execute(
            "git",
            self.location.as_path(),
            ["--no-pager", "diff", diff.as_str()],
            |stdout, _| Ok(stdout.to_string()),
        )?;

        Ok(diff)
    }

    pub fn merge(&self, branch_name: &str) -> Result<bool, RepositoryError> {
        let merged =
            execute("git", self.location.as_path(), ["merge", branch_name], |_, output| {
                Ok(output.status.success())
            })?;

        if !merged {
            return Err(RepositoryError::BranchMergeFailure);
        }

        Ok(merged)
    }

    pub fn add_all(&self) -> Result<bool, RepositoryError> {
        let add_all =
            execute("git", self.location.as_path(), ["add", "--all", "--verbose"], |_, output| {
                Ok(output.status.success())
            })?;
        let renormalize = execute(
            "git",
            self.location.as_path(),
            ["add", "--all", "--renormalize"],
            |_, output| Ok(output.status.success()),
        )?;

        if !add_all || !renormalize {
            return Err(RepositoryError::AddAllFailure);
        }

        Ok(add_all && renormalize)
    }

    pub fn add(&self, path: &Path) -> Result<bool, RepositoryError> {
        if path.exists() {
            let add = execute(
                "git",
                self.location.as_path(),
                ["add", path.to_str().expect("Failed to convert path to str"), "--verbose"],
                |_, output| Ok(output.status.success()),
            )?;

            let renormalize = execute(
                "git",
                self.location.as_path(),
                ["add", path.to_str().expect("Failed to convert path to str"), "--renormalize"],
                |_, output| Ok(output.status.success()),
            )?;

            if !add || !renormalize {
                return Err(RepositoryError::AddFailure);
            }

            Ok(add && renormalize)
        } else {
            Ok(false)
        }
    }

    pub fn fetch_all(&self, fetch_tags: Option<bool>) -> Result<bool, RepositoryError> {
        let mut args = vec!["fetch", "origin"];

        if fetch_tags.unwrap_or(false) {
            args.push("--tags");
            args.push("--force");
        }

        let fetched =
            execute("git", self.location.as_path(), args, |_, output| Ok(output.status.success()))?;

        if !fetched {
            return Err(RepositoryError::FetchFailure);
        }

        Ok(fetched)
    }

    pub fn get_diverged_commit(&self, sha: &str) -> Result<String, RepositoryError> {
        let commit =
            execute("git", self.location.as_path(), ["merge-base", sha, "HEAD"], |stdout, _| {
                Ok(stdout.to_string())
            })?;

        Ok(commit)
    }

    pub fn get_current_sha(&self) -> Result<String, RepositoryError> {
        let commit = execute(
            "git",
            self.location.as_path(),
            ["rev-parse", "--short", "HEAD"],
            |stdout, _| Ok(stdout.to_string()),
        )?;

        Ok(commit)
    }

    pub fn get_previous_sha(&self) -> Result<String, RepositoryError> {
        let commit = execute(
            "git",
            self.location.as_path(),
            ["rev-parse", "--short", "HEAD~1"],
            |stdout, _| Ok(stdout.to_string()),
        )?;

        Ok(commit)
    }

    pub fn get_first_sha(&self, branch: Option<String>) -> Result<String, RepositoryError> {
        let branch = match branch {
            Some(branch) => branch,
            None => String::from("main"),
        };

        #[cfg(not(windows))]
        let commit = execute(
            "sh",
            self.location.as_path(),
            [
                "-c",
                format!("git log --oneline {}..HEAD --pretty=format:%h | tail -1", branch).as_str(),
            ],
            |stdout, _| Ok(stdout.to_string()),
        )?;

        #[cfg(windows)]
        let commit = execute(
            "cmd",
            self.location.as_path(),
            [
                "/C",
                format!("git log --oneline {}..HEAD --pretty=format:%h | findstr /R /C:^^", branch)
                    .as_str(),
            ],
            |stdout, _| Ok(stdout.to_string()),
        )?;

        Ok(commit)
    }
}
