//! This module provides a simple interface to interact with Git repositories.
//! To use this module import it like this:
//! ```rust
//! use workspace_std::git::Repository;
//! ```
use crate::types::GitResult;
use crate::utils::strip_trailing_newline;
use crate::{errors::GitError, utils::adjust_canonicalization};

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env::temp_dir,
    ffi::OsStr,
    fs::{remove_file, File},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
    str,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    location: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepositoryCommit {
    hash: String,
    author_name: String,
    author_email: String,
    author_date: String,
    message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepositoryRemoteTags {
    tag: String,
    hash: String,
}

impl From<&PathBuf> for Repository {
    fn from(root: &PathBuf) -> Self {
        let repo_path = &std::fs::canonicalize(Path::new(root.as_os_str())).expect("Invalid path");

        Repository { location: repo_path.clone() }
    }
}

impl From<&str> for Repository {
    fn from(root: &str) -> Self {
        let path_buff = PathBuf::from(root);
        let repo_path =
            &std::fs::canonicalize(Path::new(path_buff.as_os_str())).expect("Invalid path");

        Repository { location: repo_path.clone() }
    }
}

impl Repository {
    pub fn new(location: &Path) -> Self {
        let root = std::fs::canonicalize(location.as_os_str()).expect("Invalid path");
        Self { location: root }
    }

    pub fn get_repo_path(&self) -> &Path {
        &self.location
    }

    pub fn init(&self, initial_branch: &str, username: &str, email: &str) -> GitResult<bool> {
        let init = execute_git(
            &self.location,
            ["init", "--initial-branch", initial_branch],
            |_, output| Ok(output.status.success()),
        );
        let config = self.config(username, email);

        Ok(init.is_ok() && config.is_ok())
    }

    pub fn is_vcs(&self) -> GitResult<bool> {
        execute_git(&self.location, ["rev-parse", "--is-inside-work-tree"], |stdout, _| {
            Ok(stdout.trim() == "true")
        })
    }

    pub fn config(&self, username: &str, email: &str) -> GitResult<bool> {
        let user_config =
            execute_git(&self.location, ["config", "user.name", username], |_, output| {
                Ok(output.status.success())
            });

        let email_config =
            execute_git(&self.location, ["config", "user.email", email], |_, output| {
                Ok(output.status.success())
            });

        let clrf_config =
            execute_git(&self.location, ["config", "core.safecrlf", "false"], |_, output| {
                Ok(output.status.success())
            });

        let autocrlf_config =
            execute_git(&self.location, ["config", "core.autocrlf", "input"], |_, output| {
                Ok(output.status.success())
            });

        let filemode_config =
            execute_git(&self.location, ["config", "core.filemode", "false"], |_, output| {
                Ok(output.status.success())
            });

        Ok(user_config.is_ok()
            && email_config.is_ok()
            && clrf_config.is_ok()
            && autocrlf_config.is_ok()
            && filemode_config.is_ok())
    }

    pub fn log(&self) -> GitResult<String> {
        execute_git(&self.location, ["--no-pager", "log", "main..HEAD"], |stdout, _| {
            Ok(stdout.trim().to_string())
        })
    }

    pub fn diff(&self, diff: Option<String>) -> GitResult<String> {
        let diff = match diff {
            Some(diff) => diff,
            None => ".".to_string(),
        };

        execute_git(&self.location, ["diff", diff.as_str()], |stdout, _| Ok(stdout.to_string()))
    }

    pub fn list_config(&self, config_type: &str) -> GitResult<String> {
        execute_git(
            &self.location,
            ["--no-pager", "config", "list", format!("--{config_type}").as_str()],
            |stdout, _| Ok(stdout.to_string()),
        )
    }

    pub fn create_branch(&self, branch_name: &str) -> GitResult<bool> {
        execute_git(&self.location, ["checkout", "-b", branch_name], |_, output| {
            Ok(output.status.success())
        })
    }

    pub fn checkout(&self, branch_name: &str) -> GitResult<bool> {
        execute_git(&self.location, ["checkout", branch_name], |_, output| {
            Ok(output.status.success())
        })
    }

    pub fn merge(&self, branch_name: &str) -> GitResult<bool> {
        execute_git(&self.location, ["merge", branch_name], |_, output| Ok(output.status.success()))
    }

    pub fn add_all(&self) -> GitResult<bool> {
        execute_git(&self.location, ["add", "--all", "--verbose"], |_, output| {
            Ok(output.status.success())
        })
    }

    pub fn add(&self, path: &Path) -> GitResult<bool> {
        if path.to_str().is_some() {
            execute_git(
                &self.location,
                ["add", path.to_str().expect("Failed to convert path to str")],
                |_, output| Ok(output.status.success()),
            )
        } else {
            Ok(false)
        }
    }

    pub fn fetch_all(&self, fetch_tags: Option<bool>) -> GitResult<bool> {
        let mut args = vec!["fetch", "origin"];

        if fetch_tags.unwrap_or(false) {
            args.push("--tags");
            args.push("--force");
        }

        execute_git(&self.location, &args, |_, output| Ok(output.status.success()))
    }

    pub fn get_diverged_commit(&self, sha: &str) -> GitResult<String> {
        execute_git(&self.location, ["merge-base", sha, "HEAD"], |stdout, _| Ok(stdout.to_string()))
    }

    pub fn get_current_sha(&self) -> GitResult<String> {
        execute_git(&self.location, ["rev-parse", "--short", "HEAD"], |stdout, _| {
            Ok(stdout.to_string())
        })
    }

    pub fn get_previous_sha(&self) -> GitResult<String> {
        execute_git(&self.location, ["rev-parse", "--short", "HEAD~1"], |stdout, _| {
            Ok(stdout.to_string())
        })
    }

    #[allow(clippy::uninlined_format_args)]
    pub fn get_first_sha(&self, branch: Option<String>) -> GitResult<String> {
        let branch = match branch {
            Some(branch) => branch,
            None => String::from("main"),
        };

        execute_git(
            &self.location,
            [
                "log",
                format!("{}..HEAD", branch).as_str(),
                "--online",
                "--pretty=format:%h",
                "|",
                "tail",
                "-1",
            ],
            |stdout, _| Ok(stdout.to_string()),
        )
    }

    pub fn is_workdir_unclean(&self) -> GitResult<bool> {
        execute_git(&self.location, ["status", "--porcelain"], |stdout, _| Ok(!stdout.is_empty()))
    }

    pub fn status(&self) -> GitResult<Option<String>> {
        execute_git(&self.location, ["status", "--porcelain"], |stdout, _| {
            if stdout.is_empty() {
                Ok(None)
            } else {
                Ok(Some(stdout.to_string()))
            }
        })
    }

    pub fn get_current_branch(&self) -> GitResult<Option<String>> {
        execute_git(&self.location, ["rev-parse", "--abbrev-ref", "HEAD"], |stdout, _| {
            if stdout.is_empty() {
                Ok(None)
            } else {
                Ok(Some(stdout.to_string()))
            }
        })
    }

    pub fn get_branch_from_commit(&self, sha: &str) -> GitResult<Option<String>> {
        execute_git(
            &self.location,
            [
                "--no-pager",
                "branch",
                "--no-color",
                "--no-column",
                "--format",
                r#""%(refname:lstrip=2)""#,
                "--contains",
                sha,
            ],
            |stdout, _| {
                if stdout.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(stdout.to_string()))
                }
            },
        )
    }

    pub fn tag(&self, tag: &str, message: Option<String>) -> GitResult<bool> {
        let msg = message.unwrap_or(tag.to_string());

        execute_git(&self.location, ["tag", "-a", tag, "-m", msg.as_str()], |_, output| {
            Ok(output.status.success())
        })
    }

    pub fn push(&self, follow_tags: Option<bool>) -> GitResult<bool> {
        let mut args = vec!["push", "--no-verify"];

        if follow_tags.unwrap_or(false) {
            args.push("--follow-tags");
        }

        execute_git(&self.location, &args, |_, output| Ok(output.status.success()))
    }

    pub fn commit(
        &self,
        message: &str,
        body: Option<String>,
        footer: Option<String>,
    ) -> GitResult<bool> {
        let mut msg = message.to_string();
        let root = &self.location.as_path();

        if let Some(body) = body {
            msg.push_str("\n\n");
            msg.push_str(body.as_str());
        }

        if let Some(footer) = footer {
            msg.push_str("\n\n");
            msg.push_str(footer.as_str());
        }

        let temp_dir = temp_dir();
        let temp_file_path = temp_dir.join("commit_message.txt");

        let mut file = File::create(temp_file_path.as_path()).expect("Failed to creat commit file");
        file.write_all(message.as_bytes()).expect("Failed to write commit message");

        let file_path = temp_file_path.as_path();

        execute_git(
            root,
            [
                "commit",
                "-F",
                file_path.as_os_str().to_str().expect("Failed to convert path to string"),
                "--no-verify",
            ],
            |_, output| {
                remove_file(file_path).expect("Commit file not deleted");

                Ok(output.status.success())
            },
        )
    }

    pub fn get_all_files_changed_since_sha(&self, sha: &str) -> GitResult<Vec<String>> {
        execute_git(
            &self.location,
            ["--no-pager", "diff", "--name-only", sha, "HEAD"],
            |stdout, output| {
                if !output.status.success() {
                    return Ok(vec![]);
                }

                Ok(stdout
                    .split('\n')
                    .filter(|item| !item.trim().is_empty())
                    .map(|item| self.location.join(item))
                    .filter(|item| item.exists())
                    .map(|item| {
                        item.to_str().expect("Failed to convert path to string").to_string()
                    })
                    .collect::<Vec<String>>())
            },
        )
    }

    #[allow(clippy::uninlined_format_args)]
    pub fn get_commits_since(
        &self,
        since: Option<String>,
        relative: Option<String>,
    ) -> GitResult<Vec<RepositoryCommit>> {
        const DELIMITER: &str = r"#=#";
        const BREAK_LINE: &str = r"#+#";

        let log_format = format!(
            "--format={}%H{}%an{}%ae{}%ad{}%B{}",
            DELIMITER, DELIMITER, DELIMITER, DELIMITER, DELIMITER, BREAK_LINE
        );

        let mut args = vec![
            "--no-pager".to_string(),
            "log".to_string(),
            log_format,
            "--date=rfc2822".to_string(),
        ];

        if let Some(since) = since {
            args.push(format!("{}..", since));
        }

        if let Some(relative) = relative {
            args.push("--".to_string());
            args.push(relative);
        }

        execute_git(&self.location, &args, |stdout, output| {
            if !output.status.success() {
                return Ok(vec![]);
            }

            Ok(stdout
                .split(BREAK_LINE)
                .filter(|item| !item.trim().is_empty())
                .map(|item| {
                    let item_trimmed = item.trim();
                    let items = item_trimmed.split(DELIMITER).collect::<Vec<&str>>();

                    RepositoryCommit {
                        hash: items[1].to_string(),
                        author_name: items[2].to_string(),
                        author_email: items[3].to_string(),
                        author_date: items[4].to_string(),
                        message: items[5].to_string(),
                    }
                })
                .collect::<Vec<RepositoryCommit>>())
        })
    }

    #[allow(clippy::items_after_statements)]
    pub fn get_remote_or_local_tags(
        &self,
        local: Option<bool>,
    ) -> GitResult<Vec<RepositoryRemoteTags>> {
        let mut args = vec![];
        let regex = Regex::new(r"\s+").expect("Failed to create regex");

        match local {
            Some(true) => {
                args.push("show-ref");
                args.push("--tags");
            }
            Some(false) | None => {
                args.push("ls-remote");
                args.push("--tags");
                args.push("origin");
            }
        }

        execute_git(&self.location, &args, |stdout, output| {
            if !output.status.success() {
                return Ok(vec![]);
            }

            #[cfg(windows)]
            const LINE_ENDING: &str = "\r\n";
            #[cfg(not(windows))]
            const LINE_ENDING: &str = "\n";

            Ok(stdout
                .trim()
                .split(LINE_ENDING)
                .filter(|tags| !tags.trim().is_empty())
                .map(|tags| {
                    let hash_tags = regex.split(tags).collect::<Vec<&str>>();

                    RepositoryRemoteTags {
                        hash: hash_tags[0].to_string(),
                        tag: hash_tags[1].to_string(),
                    }
                })
                .collect::<Vec<RepositoryRemoteTags>>())
        })
    }

    pub fn get_all_files_changed_since_branch(
        &self,
        packages_paths: &[String],
        branch: &str,
    ) -> Vec<String> {
        let mut all_files = vec![];

        packages_paths.iter().for_each(|item| {
            let files = self
                .get_all_files_changed_since_sha(branch)
                .expect("Failed to retrieve files changed since branch");

            let pkg_files = files
                .iter()
                .filter(|file| file.starts_with(item.as_str()))
                .collect::<Vec<&String>>();

            all_files.append(
                &mut pkg_files.iter().map(|file| (*file).to_string()).collect::<Vec<String>>(),
            );
        });

        all_files
    }
}

impl RepositoryCommit {
    pub fn new(
        hash: String,
        author_name: String,
        author_email: String,
        author_date: String,
        message: String,
    ) -> Self {
        Self { hash, author_name, author_email, author_date, message }
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }

    pub fn set_message(&mut self, message: &String) {
        self.message = message.to_string();
    }

    pub fn get_author_name(&self) -> &String {
        &self.author_name
    }

    pub fn set_author_name(&mut self, author_name: &String) {
        self.author_name = author_name.to_string();
    }

    pub fn get_author_email(&self) -> &String {
        &self.author_email
    }

    pub fn set_author_email(&mut self, author_email: &String) {
        self.author_email = author_email.to_string();
    }

    pub fn get_author_date(&self) -> &String {
        &self.author_date
    }

    pub fn set_author_date(&mut self, author_date: &String) {
        self.author_date = author_date.to_string();
    }

    pub fn get_hash(&self) -> &String {
        &self.hash
    }

    pub fn set_hash(&mut self, hash: &String) {
        self.hash = hash.to_string();
    }

    pub fn get_hash_map(&self) -> HashMap<String, String> {
        HashMap::from([
            ("hash".to_string(), self.hash.to_string()),
            ("author_name".to_string(), self.author_name.to_string()),
            ("author_email".to_string(), self.author_email.to_string()),
            ("author_date".to_string(), self.author_date.to_string()),
            ("message".to_string(), self.message.to_string()),
        ])
    }
}

impl RepositoryRemoteTags {
    pub fn new(hash: String, tag: String) -> Self {
        Self { hash, tag }
    }

    pub fn get_hash(&self) -> &String {
        &self.hash
    }

    pub fn set_hash(&mut self, hash: &String) {
        self.hash = hash.to_string();
    }

    pub fn get_tag(&self) -> &String {
        &self.tag
    }

    pub fn set_tag(&mut self, tag: &String) {
        self.tag = tag.to_string();
    }

    pub fn get_hash_map(&self) -> HashMap<String, String> {
        HashMap::from([
            ("hash".to_string(), self.hash.to_string()),
            ("tag".to_string(), self.tag.to_string()),
        ])
    }
}

pub fn execute_git<P, I, F, S, R>(path: P, args: I, process: F) -> GitResult<R>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    F: Fn(&str, &Output) -> GitResult<R>,
{
    let root = adjust_canonicalization(path);
    let root = PathBuf::from(root);
    let output = Command::new("git").current_dir(root.as_path()).args(args).output();

    output.map_err(|_| GitError::Execution).and_then(|output| {
        if output.status.success() {
            if let Ok(message) = str::from_utf8(&output.stdout) {
                process(strip_trailing_newline(&message.to_string()).as_str(), &output)
            } else {
                Err(GitError::Execution)
            }
        } else if let Ok(message) = str::from_utf8(&output.stdout) {
            if let Ok(err) = str::from_utf8(&output.stderr) {
                Err(GitError::GitError { stdout: message.to_string(), stderr: err.to_string() })
            } else {
                Err(GitError::Execution)
            }
        } else {
            Err(GitError::Execution)
        }
    })
}

/*#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir, remove_dir_all};
    #[cfg(not(windows))]
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    fn create_monorepo() -> Result<PathBuf, std::io::Error> {
        let temp_dir = temp_dir();
        let monorepo_root_dir = temp_dir.join("monorepo-workspace");

        if monorepo_root_dir.exists() {
            remove_dir_all(&monorepo_root_dir)?;
        }

        create_dir(&monorepo_root_dir)?;

        #[cfg(not(windows))]
        std::fs::set_permissions(&monorepo_root_dir, std::fs::Permissions::from_mode(0o777))?;

        Ok(monorepo_root_dir)
    }

    #[test]
    fn test_create_repo() -> Result<(), std::io::Error> {
        let monorepo_root_dir = create_monorepo()?;
        let repo = Repository::new(&monorepo_root_dir);
        let result = repo.init("main", "Websublime Machine", "machine@websublime.com");

        assert!(result.is_ok_and(|ok| ok));
        assert!(repo.is_vcs().expect("Repo is not a vcs system"));

        remove_dir_all(&monorepo_root_dir)?;

        Ok(())
    }
}*/
