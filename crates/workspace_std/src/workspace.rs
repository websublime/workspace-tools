use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::{fs::File, io::BufReader, path::PathBuf};
use wax::{CandidatePath, Glob, Pattern};

#[cfg(not(windows))]
use std::path::Path;

#[cfg(not(windows))]
use std::fs::canonicalize;

use crate::{
    config::{get_workspace_config, WorkspaceConfig},
    git::Repository,
    manager::CorePackageManager,
    package::{Dependency, Package, PackageInfo, PackageJson},
};

#[derive(Debug, Deserialize, Serialize)]
/// A struct that represents a pnpm workspace.
struct PnpmInfo {
    pub name: String,
    pub path: String,
    pub private: bool,
}

pub struct Workspace {
    pub config: WorkspaceConfig,
}

impl From<&str> for Workspace {
    fn from(root: &str) -> Self {
        let path_buff = PathBuf::from(root);

        #[cfg(not(windows))]
        let canonic_path = canonicalize(Path::new(path_buff.as_os_str())).expect("Invalid path");

        #[cfg(windows)]
        let canonic_path = path_buff;

        let config = get_workspace_config(Some(canonic_path));
        Workspace { config }
    }
}

impl From<WorkspaceConfig> for Workspace {
    fn from(config: WorkspaceConfig) -> Self {
        Workspace { config }
    }
}

impl Workspace {
    pub fn new(root: PathBuf) -> Self {
        let config = get_workspace_config(Some(root));
        Workspace { config }
    }

    pub fn get_packages(&self) -> Vec<PackageInfo> {
        let manager = self.config.package_manager;

        match manager {
            CorePackageManager::Npm | CorePackageManager::Yarn => self.get_packages_from_npm(),
            CorePackageManager::Bun => todo!("Bun is not yet supported"),
            CorePackageManager::Pnpm => self.get_packages_from_pnpm(),
        }
    }

    pub fn get_package_info(&self, package_name: &str) -> Option<PackageInfo> {
        let packages = self.get_packages();
        packages.into_iter().find(|p| p.package.name == package_name)
    }

    pub fn get_changed_packages(&self, sha: Option<String>) -> Vec<PackageInfo> {
        let packages = &self.get_packages();
        let root = &self.config.workspace_root.as_path();
        let since = sha.unwrap_or(String::from("main"));
        let packages_paths =
            packages.iter().map(|pkg| pkg.package_path.to_string()).collect::<Vec<String>>();

        let repo = Repository::new(root);
        let changed_files =
            repo.get_all_files_changed_since_branch(&packages_paths, since.as_str());

        packages
            .iter()
            .flat_map(|pkg| {
                let mut pkgs = changed_files
                    .iter()
                    .filter(|file| file.starts_with(&pkg.package_path))
                    .map(|_| pkg.to_owned())
                    .collect::<Vec<PackageInfo>>();

                pkgs.dedup_by(|a, b| a.package.name == b.package.name);

                pkgs
            })
            .collect::<Vec<PackageInfo>>()
    }

    fn get_root_package_json(&self) -> PackageJson {
        let package_json_path = self.config.workspace_root.join("package.json");

        let package_json_file = File::open(package_json_path.as_path()).expect("File not found");
        let package_json_buffer = BufReader::new(package_json_file);

        serde_json::from_reader(package_json_buffer).expect("Error parsing package.json")
    }

    #[allow(clippy::unused_self)]
    fn aggregate_dependencies(&self, package_json: &PackageJson) -> Vec<Dependency> {
        let mut package_dependencies = vec![];

        let dependencies = package_json.dependencies.clone().unwrap_or_default();
        let dev_dependencies = package_json.dev_dependencies.clone().unwrap_or_default();
        let peer_dependencies = package_json.peer_dependencies.clone().unwrap_or_default();
        let optional_dependencies = package_json.optional_dependencies.clone().unwrap_or_default();

        if dependencies.is_object() {
            dependencies.as_object().iter().for_each(|dep| {
                dep.keys().for_each(|key| {
                    let dependency = Dependency {
                        name: key.clone(),
                        version: dep
                            .get(key)
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string()
                            .parse()
                            .unwrap(),
                    };
                    package_dependencies.push(dependency);
                });
            });
        }

        if dev_dependencies.is_object() {
            dev_dependencies.as_object().iter().for_each(|dep| {
                dep.keys().for_each(|key| {
                    let dependency = Dependency {
                        name: key.clone(),
                        version: dep
                            .get(key)
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string()
                            .parse()
                            .unwrap(),
                    };
                    package_dependencies.push(dependency);
                });
            });
        }

        if peer_dependencies.is_object() {
            peer_dependencies.as_object().iter().for_each(|dep| {
                dep.keys().for_each(|key| {
                    let dependency = Dependency {
                        name: key.clone(),
                        version: dep
                            .get(key)
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string()
                            .parse()
                            .unwrap(),
                    };
                    package_dependencies.push(dependency);
                });
            });
        }

        if optional_dependencies.is_object() {
            optional_dependencies.as_object().iter().for_each(|dep| {
                dep.keys().for_each(|key| {
                    let dependency = Dependency {
                        name: key.clone(),
                        version: dep
                            .get(key)
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string()
                            .parse()
                            .unwrap(),
                    };
                    package_dependencies.push(dependency);
                });
            });
        }

        package_dependencies
    }

    #[allow(clippy::needless_borrows_for_generic_args)]
    fn get_packages_from_npm(&self) -> Vec<PackageInfo> {
        let path = self.config.workspace_root.as_path();
        let PackageJson { workspaces, .. } = self.get_root_package_json();
        let mut workspaces = workspaces.unwrap_or_default();
        let mut packages = vec![];

        let globs = workspaces
            .iter_mut()
            .map(|workspace| {
                if workspace.ends_with("/*") {
                    workspace.push_str("*/package.json");
                    Glob::new(workspace).expect("Error parsing glob")
                } else {
                    workspace.push_str("/package.json");
                    Glob::new(workspace).expect("Error parsing glob")
                }
            })
            .collect::<Vec<Glob>>();

        let patterns = wax::any(globs).expect("Error creating patterns");
        let glob = Glob::new("**/package.json").expect("Error parsing glob");

        for entry in glob
            .walk(self.config.workspace_root.as_path())
            .not([
                "**/node_modules/**",
                "**/src/**",
                "**/dist/**",
                "**/tests/**",
                "**/__tests__/**",
            ])
            .expect("Error walking glob")
        {
            let entry = entry.expect("Error reading entry");
            let rel_path = entry
                .path()
                .strip_prefix(&path)
                .expect("Error getting entry path")
                .display()
                .to_string();
            let entry_path = entry.path().strip_prefix(&path).expect("Error getting entry path");

            if patterns.is_match(CandidatePath::from(entry_path)) {
                let package_json_file = File::open(&entry.path()).expect("File not found");
                let package_json_reader = BufReader::new(package_json_file);
                let pkg_json: PackageJson = serde_json::from_reader(package_json_reader)
                    .expect("Failed to parse package json file");

                let package_dependencies = self.aggregate_dependencies(&pkg_json);

                let package = Package {
                    name: pkg_json.name.clone(),
                    version: pkg_json.version.parse().unwrap(),
                    dependencies: package_dependencies,
                };

                packages.push(PackageInfo {
                    package,
                    package_path: entry.path().to_str().unwrap().replace("/package.json", ""),
                    package_json_path: entry.path().to_str().unwrap().to_string(),
                    package_relative_path: rel_path.replace("/package.json", ""),
                    pkg_json: serde_json::to_value(pkg_json)
                        .expect("Error converting package json"),
                });
            }
        }

        packages
    }

    fn get_packages_from_pnpm(&self) -> Vec<PackageInfo> {
        let path = &self.config.workspace_root.as_path();

        let mut command = Command::new("pnpm");
        command.current_dir(path).arg("list").arg("-r").arg("--depth").arg("-1").arg("--json");

        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let output = command.output().expect("Failed to execute command");
        let output_slice = &output.stdout.as_slice();
        let pnpm_info = serde_json::from_slice::<Vec<PnpmInfo>>(output_slice)
            .expect("Failed to parse pnpm list");

        pnpm_info
            .iter()
            .filter(|pkgs| pkgs.path != path.display().to_string())
            .map(|pkgs| {
                let package_path = PathBuf::from(pkgs.path.clone());
                let package_json_path = package_path.join("package.json");

                let package_json_file = File::open(&package_json_path).expect("File not found");
                let package_json_reader = BufReader::new(package_json_file);
                let pkg_json: PackageJson = serde_json::from_reader(package_json_reader)
                    .expect("Failed to parse package json file");

                let package_dependencies = self.aggregate_dependencies(&pkg_json);

                let package = Package {
                    name: pkg_json.name.clone(),
                    version: pkg_json.version.parse().unwrap(),
                    dependencies: package_dependencies,
                };

                PackageInfo {
                    package,
                    package_path: package_path.to_str().unwrap().to_string(),
                    package_json_path: package_json_path.to_str().unwrap().to_string(),
                    package_relative_path: package_path
                        .strip_prefix(path)
                        .expect("Error getting entry path")
                        .display()
                        .to_string(),
                    pkg_json: serde_json::to_value(pkg_json)
                        .expect("Error converting package json"),
                }
            })
            .collect::<Vec<PackageInfo>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::Repository;
    use crate::manager::CorePackageManager;
    use crate::test::MonorepoWorkspace;
    use std::fs::File;
    use std::io::Write;
    //use std::process::Command;

    #[test]
    fn test_get_npm_packages() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_workspace(&CorePackageManager::Npm)?;

        let workspace = Workspace::new(root);
        let packages = workspace.get_packages();

        assert_eq!(packages.len(), 6);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_yarn_packages() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_workspace(&CorePackageManager::Yarn)?;

        let workspace = Workspace::new(root);
        let packages = workspace.get_packages();

        assert_eq!(packages.len(), 6);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_pnpm_packages() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_workspace(&CorePackageManager::Pnpm)?;

        let workspace = Workspace::new(root);
        let packages = workspace.get_packages();

        assert_eq!(packages.len(), 6);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_changed_packages() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        let js_path = root.join("packages/package-foo/main.mjs");
        monorepo.create_workspace(&CorePackageManager::Pnpm)?;
        let workspace = Workspace::new(root.clone());
        let repo = Repository::new(root.as_path());

        let _ = repo.create_branch("feat/message").expect("Failed to create branch");

        let mut js_file = File::create(js_path.as_path()).expect("Failed to create main.js file");
        js_file.write_all(r#"export const message = "hello";"#.as_bytes())?;

        let _ = repo.add_all().expect("Failed to add files");
        let _ = repo.commit("feat: message to the world", None, None).expect("Failed to commit");

        let packages = workspace.get_changed_packages(Some("main".to_string()));

        assert_eq!(packages.len(), 1);

        monorepo.delete_repository();

        Ok(())
    }
}
