use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env::temp_dir,
    fs::{canonicalize, remove_file, File},
    io::Write,
    path::{Path, PathBuf},
};
use ws_std::command::execute;

use super::error::RepositoryError;

#[allow(clippy::from_over_into)]
impl Into<HashMap<String, String>> for RepositoryTags {
    fn into(self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("hash".to_string(), self.hash);
        map.insert("tag".to_string(), self.tag);
        map
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepositoryPublishTagInfo {
    pub hash: String,
    pub tag: String,
    pub package: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepositoryTags {
    pub hash: String,
    pub tag: String,
}

#[allow(clippy::from_over_into)]
impl Into<HashMap<String, String>> for RepositoryCommit {
    fn into(self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("hash".to_string(), self.hash);
        map.insert("author_name".to_string(), self.author_name);
        map.insert("author_email".to_string(), self.author_email);
        map.insert("author_date".to_string(), self.author_date);
        map.insert("message".to_string(), self.message);
        map
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepositoryCommit {
    pub hash: String,
    pub author_name: String,
    pub author_email: String,
    pub author_date: String,
    pub message: String,
}

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

    pub fn diff(&self, diff: Option<Vec<String>>) -> Result<String, RepositoryError> {
        let mut args: Vec<String> = vec!["--no-pager".to_string(), "diff".to_string()];

        if let Some(diff) = diff {
            args.extend(diff);
        } else {
            args.push(".".to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let diff =
            execute("git", self.location.as_path(), args_ref, |stdout, _| Ok(stdout.to_string()))?;

        Ok(diff)
    }

    pub fn get_last_tag(&self) -> Result<String, RepositoryError> {
        let tag = execute(
            "git",
            self.location.as_path(),
            ["describe", "--tags", "--abbrev=0"],
            |stdout, _| Ok(stdout.to_string()),
        )?;

        Ok(tag)
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
            &self.location,
            [
                "/C",
                format!("git log --oneline {}..HEAD --pretty=format:%h | findstr /R /C:^^", branch)
                    .as_str(),
            ],
            |stdout, _| {
                let output = stdout.to_string();
                let cmd_out = output
                    .lines()
                    .filter_map(|line| Some(line))
                    .last()
                    .expect("Failed to get last line");
                Ok(cmd_out.to_string())
            },
        )?;

        Ok(commit)
    }

    pub fn status(&self) -> Result<Option<String>, RepositoryError> {
        let status =
            execute("git", self.location.as_path(), ["status", "--porcelain"], |stdout, _| {
                if stdout.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(stdout.to_string()))
                }
            })?;

        Ok(status)
    }

    pub fn get_current_branch(&self) -> Result<Option<String>, RepositoryError> {
        let current_branch = execute(
            "git",
            self.location.as_path(),
            ["rev-parse", "--abbrev-ref", "HEAD"],
            |stdout, _| {
                if stdout.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(stdout.to_string()))
                }
            },
        )?;

        Ok(current_branch)
    }

    pub fn get_branch_from_commit(&self, sha: &str) -> Result<Option<String>, RepositoryError> {
        let branch = execute(
            "git",
            self.location.as_path(),
            [
                "--no-pager",
                "branch",
                "--no-color",
                "--no-column",
                "--format",
                r#"%(refname:lstrip=2)"#,
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
        )?;

        Ok(branch)
    }

    pub fn tag(&self, tag: &str, message: Option<String>) -> Result<bool, RepositoryError> {
        let msg = message.unwrap_or(tag.to_string());

        Ok(execute(
            "git",
            self.location.as_path(),
            ["tag", "-a", tag, "-m", msg.as_str()],
            |_, output| Ok(output.status.success()),
        )?)
    }

    pub fn push(&self, follow_tags: Option<bool>) -> Result<bool, RepositoryError> {
        let mut args = vec!["push", "--no-verify"];

        if follow_tags.unwrap_or(false) {
            args.push("--follow-tags");
        }

        Ok(execute("git", self.location.as_path(), args, |_, output| Ok(output.status.success()))?)
    }

    pub fn commit(
        &self,
        message: &str,
        body: Option<String>,
        footer: Option<String>,
    ) -> Result<bool, RepositoryError> {
        let mut msg = message.to_string();
        let root = self.location.as_path();

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

        let mut file = File::create(temp_file_path.as_path())?;
        file.write_all(message.as_bytes())?;

        let file_path = temp_file_path.as_path();

        Ok(execute(
            "git",
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
        )?)
    }

    pub fn get_all_files_changed_since_sha(
        &self,
        sha: &str,
    ) -> Result<Vec<String>, RepositoryError> {
        Ok(execute(
            "git",
            self.location.as_path(),
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
        )?)
    }

    #[allow(clippy::uninlined_format_args)]
    pub fn get_commits_since(
        &self,
        since: Option<String>,
        relative: Option<String>,
    ) -> Result<Vec<RepositoryCommit>, RepositoryError> {
        const DELIMITER: &str = r"#=#";
        const BREAK_LINE: &str = r"#+#";

        let log_format = format!(
            "--format={}%H{}%an{}%ae{}%ad{}%B{}",
            DELIMITER, DELIMITER, DELIMITER, DELIMITER, DELIMITER, BREAK_LINE
        );

        let mut args = vec!["--no-pager", "log", log_format.as_str(), "--date=rfc2822"];
        let mut owned_strings = Vec::new();

        if let Some(since) = since.as_deref() {
            owned_strings.push(format!("{}..", since));
            args.push(owned_strings.last().unwrap().as_str());
        }

        if let Some(relative) = relative.as_deref() {
            args.push("--");
            args.push(relative);
        }

        Ok(execute("git", self.location.as_path(), args, |stdout, output| {
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
        })?)
    }

    #[allow(clippy::items_after_statements)]
    pub fn get_remote_or_local_tags(
        &self,
        local: Option<bool>,
    ) -> Result<Vec<RepositoryTags>, RepositoryError> {
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

        Ok(execute("git", self.location.as_path(), args, |stdout, output| {
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

                    RepositoryTags { hash: hash_tags[0].to_string(), tag: hash_tags[1].to_string() }
                })
                .collect::<Vec<RepositoryTags>>())
        })?)
    }

    pub fn get_all_files_changed_since_branch(
        &self,
        packages_paths: &[String],
        branch: &str,
    ) -> Result<Vec<String>, RepositoryError> {
        let mut all_files = vec![];

        for item in packages_paths {
            let files = self.get_all_files_changed_since_sha(branch)?;

            let pkg_files = files
                .iter()
                .filter(|file| {
                    let file_path_buf = PathBuf::from(file);
                    let file_canonic = &canonicalize(file_path_buf).expect("Invalid file path");
                    let file = file_canonic.to_str().expect("Failed to convert path to string");

                    let item_path_buf = PathBuf::from(item);
                    let item_canonic = &canonicalize(item_path_buf).expect("Invalid item path");
                    let item = item_canonic.to_str().expect("Failed to convert path to string");

                    file.starts_with(item)
                })
                .collect::<Vec<&String>>();

            all_files.append(
                &mut pkg_files.iter().map(|file| (*file).to_string()).collect::<Vec<String>>(),
            );
        }

        Ok(all_files)
    }
}
