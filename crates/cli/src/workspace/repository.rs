use super::WorkspaceManager;
use crate::common::errors::{CliError, CliResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Repository {
    path: PathBuf,
    name: Option<String>,
    branch: Option<String>,
    active: bool,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepositoryConfig {
    pub path: String,
    pub name: Option<String>,
    pub active: Option<bool>,
    pub branch: Option<String>,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
}

impl Repository {
    pub fn new<P: AsRef<Path>>(path: P, name: Option<String>) -> CliResult<Self> {
        let path = path.as_ref().canonicalize().map_err(CliError::Io)?;

        if !path.exists() {
            return Err(CliError::Workspace(format!(
                "Repository path does not exist: {}",
                path.display()
            )));
        }

        Ok(Self {
            path,
            name,
            branch: None,
            active: true,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn identifier(&self) -> String {
        self.name.clone().unwrap_or_else(|| {
            self.path.file_name().and_then(|n| n.to_str()).unwrap_or("unnamed").to_string()
        })
    }

    pub fn set_patterns(&mut self, include: Vec<String>, exclude: Vec<String>) {
        self.include_patterns = include;
        self.exclude_patterns = exclude;
    }

    pub fn include_patterns(&self) -> &[String] {
        &self.include_patterns
    }

    pub fn exclude_patterns(&self) -> &[String] {
        &self.exclude_patterns
    }

    pub fn update_branch(&mut self) -> CliResult<()> {
        // Try to detect current branch if it's a git repository
        if let Ok(output) = std::process::Command::new("git")
            .arg("--git-dir")
            .arg(self.path.join(".git"))
            .arg("symbolic-ref")
            .arg("--short")
            .arg("HEAD")
            .output()
        {
            if output.status.success() {
                if let Ok(branch) = String::from_utf8(output.stdout) {
                    self.branch = Some(branch.trim().to_string());
                    return Ok(());
                }
            }
        }

        self.branch = None;
        Ok(())
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn to_config(&self) -> RepositoryConfig {
        RepositoryConfig {
            path: self.path.to_string_lossy().into_owned(),
            name: self.name.clone(),
            active: Some(self.active),
            branch: self.branch.clone(),
            include_patterns: Some(self.include_patterns.clone()),
            exclude_patterns: Some(self.exclude_patterns.clone()),
        }
    }

    pub fn watch_patterns(&self) -> (Vec<String>, Vec<String>) {
        (self.include_patterns.clone(), self.exclude_patterns.clone())
    }
}

#[derive(Debug, Clone, Default)]
pub struct RepositoryManager {
    repositories: HashMap<String, Repository>,
}

impl RepositoryManager {
    pub fn new() -> Self {
        Self { repositories: HashMap::new() }
    }

    pub fn add_repository<P: AsRef<Path>>(
        &mut self,
        path: P,
        name: Option<String>,
        patterns: Option<(Vec<String>, Vec<String>)>,
    ) -> CliResult<()> {
        let path = path.as_ref().canonicalize().map_err(CliError::Io)?;

        if !path.exists() {
            return Err(CliError::Workspace(format!(
                "Repository path does not exist: {}",
                path.display()
            )));
        }

        let identifier = name.clone().unwrap_or_else(|| {
            path.file_name().and_then(|n| n.to_str()).unwrap_or("unnamed").to_string()
        });

        if self.repositories.contains_key(&identifier) {
            return Err(CliError::Workspace(format!(
                "Repository already exists with identifier: {}",
                identifier
            )));
        }

        let (include_patterns, exclude_patterns) = patterns.unwrap_or_else(|| {
            (
                vec!["**/*.{rs,toml}".to_string()],
                vec!["**/target/**".to_string(), "**/.git/**".to_string()],
            )
        });

        let mut repo = Repository::new(&path, name)?;
        repo.set_patterns(include_patterns, exclude_patterns);

        if let Err(e) = repo.update_branch() {
            log::warn!("Failed to update branch information: {}", e);
        }

        self.repositories.insert(identifier, repo);
        Ok(())
    }

    pub fn remove_repository(&mut self, identifier: &str) -> CliResult<()> {
        if self.repositories.remove(identifier).is_none() {
            return Err(CliError::Workspace(format!("Repository not found: {}", identifier)));
        }
        Ok(())
    }

    pub fn get_repository(&self, identifier: &str) -> CliResult<Option<&Repository>> {
        Ok(self.repositories.get(identifier))
    }

    pub fn list_repositories(&self) -> CliResult<Vec<&Repository>> {
        Ok(self.repositories.values().collect())
    }

    pub fn update_patterns(
        &mut self,
        identifier: &str,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
    ) -> CliResult<()> {
        let repo = self
            .repositories
            .get_mut(identifier)
            .ok_or_else(|| CliError::Workspace(format!("Repository not found: {}", identifier)))?;

        if let Some(include_patterns) = include {
            repo.include_patterns = include_patterns;
        }

        if let Some(exclude_patterns) = exclude {
            repo.exclude_patterns = exclude_patterns;
        }

        Ok(())
    }

    pub fn set_active(&mut self, identifier: &str, active: bool) -> CliResult<()> {
        let repo = self
            .repositories
            .get_mut(identifier)
            .ok_or_else(|| CliError::Workspace(format!("Repository not found: {}", identifier)))?;

        repo.active = active;
        Ok(())
    }

    pub fn load_from_config(&mut self, config: &[RepositoryConfig]) -> CliResult<()> {
        for repo_config in config {
            let path = PathBuf::from(&repo_config.path);

            if !path.exists() {
                log::warn!("Skipping non-existent repository path: {}", path.display());
                continue;
            }

            let mut repo = Repository::new(&path, repo_config.name.clone())?;

            if let Some(patterns) = repo_config.include_patterns.clone() {
                repo.include_patterns = patterns;
            }

            if let Some(patterns) = repo_config.exclude_patterns.clone() {
                repo.exclude_patterns = patterns;
            }

            repo.active = repo_config.active.unwrap_or(true);

            if let Err(e) = repo.update_branch() {
                log::warn!("Failed to update branch information: {}", e);
            }

            let identifier = repo.identifier();
            self.repositories.insert(identifier, repo);
        }

        Ok(())
    }

    pub fn save_to_config(&self) -> Vec<RepositoryConfig> {
        self.repositories.values().map(|repo| repo.to_config()).collect()
    }
}

impl WorkspaceManager for RepositoryManager {
    fn add_repository<P: AsRef<Path>>(&mut self, path: P, name: Option<String>) -> CliResult<()> {
        self.add_repository(path, name, None)
    }

    fn remove_repository(&mut self, identifier: &str) -> CliResult<()> {
        self.remove_repository(identifier)
    }

    fn list_repositories(&self) -> CliResult<Vec<&Repository>> {
        self.list_repositories()
    }

    fn get_repository(&self, identifier: &str) -> CliResult<Option<&Repository>> {
        self.get_repository(identifier)
    }
}
