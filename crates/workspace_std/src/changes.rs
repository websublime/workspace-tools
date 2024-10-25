use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use crate::{
    config::{get_workspace_config, WorkspaceConfig},
    git::Repository,
};

type ChangesData = BTreeMap<String, ChangeMeta>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Change {
    pub package: String,
    pub release_as: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangeMeta {
    pub deploy: Vec<String>,
    pub pkgs: Vec<Change>,
}

#[derive(Debug, Clone)]
pub struct Changes {
    root: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangesConfig {
    pub message: Option<String>,
    pub git_user_name: Option<String>,
    pub git_user_email: Option<String>,
    pub changes: ChangesData,
}

impl From<&WorkspaceConfig> for Changes {
    fn from(config: &WorkspaceConfig) -> Self {
        Changes { root: config.workspace_root.clone() }
    }
}

impl From<&PathBuf> for Changes {
    fn from(root: &PathBuf) -> Self {
        Changes { root: root.clone() }
    }
}

impl Changes {
    fn read_changes(&self) -> Option<ChangesConfig> {
        let root_path = Path::new(self.root.as_os_str());
        let changes_path = &root_path.join(String::from(".changes.json"));

        if self.file_exist() {
            let changes_file = File::open(changes_path).expect("Failed to open changes file");
            let changes_reader = BufReader::new(changes_file);

            let changes_config: ChangesConfig =
                serde_json::from_reader(changes_reader).expect("Failed to parse changes json file");

            return Some(changes_config);
        }

        None
    }

    fn write_changes(&self, changes: &ChangesConfig) {
        let root_path = Path::new(self.root.as_os_str());
        let changes_path = &root_path.join(String::from(".changes.json"));

        let changes_file = File::create(changes_path).expect("Failed to create changes file");
        let changes_writer = BufWriter::new(changes_file);

        serde_json::to_writer_pretty(changes_writer, changes)
            .expect("Failed to write changes file");
    }

    pub fn new(root: &Path) -> Self {
        Changes { root: root.to_path_buf() }
    }

    pub fn file_exist(&self) -> bool {
        let root_path = Path::new(self.root.as_os_str());
        let changes_path = &root_path.join(String::from(".changes.json"));

        changes_path.exists()
    }

    pub fn init(&self) -> ChangesConfig {
        if self.file_exist() {
            let changes_config = self.read_changes().expect("Failed to read changes file");

            return changes_config;
        }

        let config = get_workspace_config(Some(self.root.clone()));
        let message = config.changes_config.get("message").expect("Failed to get message changes");
        let git_user_name = config
            .changes_config
            .get("git_user_name")
            .expect("Failed to get git_user_name changes");
        let git_user_email = config
            .changes_config
            .get("git_user_email")
            .expect("Failed to get git_user_email changes");

        let changes = ChangesConfig {
            message: Some(message.to_string()),
            git_user_name: Some(git_user_name.to_string()),
            git_user_email: Some(git_user_email.to_string()),
            changes: ChangesData::new(),
        };

        self.write_changes(&changes);

        changes
    }

    pub fn add(&self, change: &Change, deploy_envs: Option<Vec<String>>) -> bool {
        if self.file_exist() {
            let mut changes_config = self.read_changes().expect("Failed to read changes file");
            let current_branch = Repository::new(&self.root)
                .get_current_branch()
                .expect("Failed to get current branch");

            let branch = match current_branch {
                Some(branch) => branch,
                None => String::from("main"),
            };

            let envs = &deploy_envs.unwrap_or_default();

            changes_config
                .changes
                .entry(branch)
                .and_modify(|entry| {
                    let pkg_exist = entry.pkgs.iter().any(|pkg| pkg.package == change.package);

                    if !pkg_exist {
                        entry.deploy.extend(envs.clone());
                        entry.deploy =
                            entry.deploy.clone().into_iter().unique().collect::<Vec<String>>();
                        entry.pkgs.push(change.clone());
                    }
                })
                .or_insert(ChangeMeta { deploy: envs.clone(), pkgs: vec![change.clone()] });

            self.write_changes(&changes_config);

            return true;
        }

        false
    }

    pub fn remove(&self, branch_name: &str) -> bool {
        if self.file_exist() {
            let mut changes_config = self.read_changes().expect("Failed to read changes file");

            if changes_config.changes.contains_key(branch_name) {
                changes_config.changes.remove(branch_name);

                self.write_changes(&changes_config);

                return true;
            }
        }

        false
    }

    pub fn changes(&self) -> ChangesData {
        if self.file_exist() {
            let changes_config = self.read_changes().expect("Failed to read changes file");

            return changes_config.changes;
        }

        ChangesData::new()
    }

    pub fn changes_by_branch(&self, branch: &str) -> Option<ChangeMeta> {
        if self.file_exist() {
            let changes_config = self.read_changes().expect("Failed to read changes file");

            if changes_config.changes.contains_key(branch) {
                return changes_config.changes.get(branch).cloned();
            }

            return None;
        }

        None
    }

    pub fn changes_by_package(&self, package_name: &str, branch: &str) -> Option<Change> {
        if self.file_exist() {
            let changes_config = self.read_changes().expect("Failed to read changes file");

            if changes_config.changes.contains_key(branch) {
                let branch_changes =
                    changes_config.changes.get(branch).expect("Failed to get branch changes");

                let package_change = branch_changes
                    .pkgs
                    .clone()
                    .into_iter()
                    .find(|change| change.package == package_name);

                if let Some(change) = package_change {
                    return Some(change);
                }

                return None;
            }

            return None;
        }

        None
    }

    pub fn exist(&self, branch: &str, package_name: &str) -> bool {
        if self.file_exist() {
            let changes_config = self.read_changes().expect("Failed to read changes file");

            if changes_config.changes.contains_key(branch) {
                let branch_changes =
                    changes_config.changes.get(branch).expect("Failed to get branch changes");

                let existing_packages_changes = branch_changes
                    .pkgs
                    .iter()
                    .map(|change| change.package.to_string())
                    .collect::<Vec<String>>();

                return existing_packages_changes.iter().any(|pkg| pkg == package_name);
            }

            return false;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::CorePackageManager;
    use crate::test::MonorepoWorkspace;
    use std::{fs::File, io::BufReader};

    #[test]
    fn test_init_changes() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;

        let changes = Changes::new(root.as_path());
        let changes_config = changes.init();

        assert_eq!(
            changes_config.message,
            Some("chore(release): |---| release new version".to_string())
        );

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_changes_file_not_exist() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;

        let changes = Changes::new(root.as_path());

        assert!(!changes.file_exist());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_changes_file_exist() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());

        assert!(changes.file_exist());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_add_new_change() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change = &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change, Some(vec!["production".to_string()]));

        let changes_path = root.join(".changes.json");
        let changes_file = File::open(changes_path.as_path())?;
        let changes_reader = BufReader::new(changes_file);
        let changes_config: ChangesConfig = serde_json::from_reader(changes_reader)?;
        let change_meta = changes_config.changes.get("main").expect("Failed to get main change");

        assert!(change_meta.deploy.contains(&"production".to_string()));
        assert_eq!(change_meta.pkgs.len(), 1);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_update_new_change_with_same_environment() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change_one =
            &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };
        let change_two =
            &Change { package: "@scope/foo".to_string(), release_as: "patch".to_string() };

        changes.add(change_one, Some(vec!["production".to_string()]));
        changes.add(change_two, Some(vec!["production".to_string()]));

        let changes_path = root.join(".changes.json");
        let changes_file = File::open(changes_path.as_path())?;
        let changes_reader = BufReader::new(changes_file);
        let changes_config: ChangesConfig = serde_json::from_reader(changes_reader)?;
        let change_meta = changes_config.changes.get("main").expect("Failed to get main change");

        assert!(change_meta.deploy.contains(&"production".to_string()));
        assert_eq!(change_meta.pkgs.len(), 2);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_update_new_change_with_diff_environment() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change_one =
            &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };
        let change_two =
            &Change { package: "@scope/foo".to_string(), release_as: "patch".to_string() };

        changes.add(change_one, Some(vec!["production".to_string()]));
        changes.add(change_two, Some(vec!["development".to_string()]));

        let changes_path = root.join(".changes.json");
        let changes_file = File::open(changes_path.as_path())?;
        let changes_reader = BufReader::new(changes_file);
        let changes_config: ChangesConfig = serde_json::from_reader(changes_reader)?;
        let change_meta = changes_config.changes.get("main").expect("Failed to get main change");

        assert!(change_meta.deploy.contains(&"production".to_string()));
        assert!(change_meta.deploy.contains(&"development".to_string()));
        assert_eq!(change_meta.pkgs.len(), 2);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_avoid_duplicate_new_change() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change_one =
            &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };
        let change_two =
            &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change_one, Some(vec!["production".to_string()]));
        changes.add(change_two, Some(vec!["development".to_string()]));

        let changes_path = root.join(".changes.json");
        let changes_file = File::open(changes_path.as_path())?;
        let changes_reader = BufReader::new(changes_file);
        let changes_config: ChangesConfig = serde_json::from_reader(changes_reader)?;
        let change_meta = changes_config.changes.get("main").expect("Failed to get main change");

        assert!(change_meta.deploy.contains(&"production".to_string()));
        assert_eq!(change_meta.pkgs.len(), 1);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_remove_change() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change = &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change, Some(vec!["production".to_string()]));
        changes.remove("main");

        let changes_path = root.join(".changes.json");
        let changes_file = File::open(changes_path.as_path())?;
        let changes_reader = BufReader::new(changes_file);
        let changes_config: ChangesConfig = serde_json::from_reader(changes_reader)?;

        assert!(changes_config.changes.is_empty());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_empty_changes() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());

        assert!(changes.changes().is_empty());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_current_changes() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change = &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change, Some(vec!["production".to_string()]));

        assert!(!changes.changes().is_empty());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_empty_changes_by_branch() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());

        assert!(changes.changes_by_branch("main").is_none());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_changes_by_branch() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change = &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change, Some(vec!["production".to_string()]));

        assert!(changes.changes_by_branch("main").is_some());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_empty_changes_by_package() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change = &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change, Some(vec!["production".to_string()]));

        assert!(changes.changes_by_package("@scope/foo", "main").is_none());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_changes_by_package() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change = &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change, Some(vec!["production".to_string()]));

        let change_by_package = &changes.changes_by_package("@scope/bar", "main");
        let package_change = change_by_package.as_ref().unwrap();

        assert!(change_by_package.is_some());
        assert_eq!(package_change.package, "@scope/bar".to_string());
        assert_eq!(package_change.release_as, "patch".to_string());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_package_change_exist() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change_bar =
            &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };
        let change_foo =
            &Change { package: "@scope/foo".to_string(), release_as: "patch".to_string() };

        changes.add(change_bar, Some(vec!["production".to_string()]));
        changes.add(change_foo, Some(vec!["production".to_string()]));

        let package_change_exist = changes.exist("main", "@scope/bar");

        assert!(package_change_exist);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_package_change_not_exist() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change_bar =
            &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };
        let change_foo =
            &Change { package: "@scope/foo".to_string(), release_as: "patch".to_string() };

        changes.add(change_bar, Some(vec!["production".to_string()]));
        changes.add(change_foo, Some(vec!["production".to_string()]));

        let package_change_exist = changes.exist("main", "@scope/baz");

        assert!(!package_change_exist);

        monorepo.delete_repository();

        Ok(())
    }
}
