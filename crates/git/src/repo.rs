use std::boxed::Box;
use std::error::Error;
use std::fs::canonicalize;
use std::path::PathBuf;

use git2::Repository;

fn canonicalize_path(path: &str) -> Result<String, Box<dyn Error>> {
    let location = PathBuf::from(path);
    let path = canonicalize(location.as_os_str())?;
    Ok(path.display().to_string())
}

pub struct Repo {
    repo: Repository,
    local_path: PathBuf,
}

impl Repo {
    pub fn create(path: &str) -> Self {
        let location = canonicalize_path(path).unwrap();
        let location_buf = PathBuf::from(location);

        Self { repo: Repository::init(location_buf.as_path()).unwrap(), local_path: location_buf }
    }

    pub fn open(path: &str) -> Result<Self, Box<dyn Error>> {
        let local_path = canonicalize_path(path)?;
        let repo = Repository::open(path)?;

        Ok(Self { repo, local_path: PathBuf::from(local_path) })
    }

    pub fn clone(url: &str, path: &str) -> Result<Self, Box<dyn Error>> {
        let local_path = canonicalize_path(path)?;
        let repo = Repository::clone(url, path)?;

        Ok(Self { repo, local_path: PathBuf::from(local_path) })
    }

    pub fn config(&self, username: &str, email: &str) -> Result<(), Box<dyn Error>> {
        let mut config = self.repo.config()?;
        config.set_str("user.name", username)?;
        config.set_str("user.email", email)?;
        config.set_bool("core.safecrlf", true)?;
        config.set_str("core.autocrlf", "input")?;
        config.set_bool("core.filemode", false)?;
        Ok(())
    }

    pub fn create_branch(&self, branch_name: &str) -> Result<(), Box<dyn Error>> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;

        self.repo.branch(branch_name, &commit, false)?;
        Ok(())
    }
}
