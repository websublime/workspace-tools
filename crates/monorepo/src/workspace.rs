use icu::collator::{Collator, CollatorOptions, Numeric, Strength};
use serde::{Deserialize, Serialize};
use std::fs::canonicalize;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{fs::File, io::BufReader, path::PathBuf};
use version_compare::{Cmp, Version};
use wax::{CandidatePath, Glob, Pattern};
use ws_git::error::RepositoryError;
use ws_git::repo::{Repository, RepositoryPublishTagInfo, RepositoryTags};
use ws_pkg::bump::BumpOptions;
use ws_pkg::package::{package_scope_name_version, Dependency, Package, PackageInfo, PackageJson};
use ws_pkg::version::Version as BumpVersion;
use ws_std::manager::CorePackageManager;

use crate::changes::Changes;
use crate::config::{get_workspace_config, WorkspaceConfig};
use crate::conventional::{
    get_conventional_by_package, ConventionalPackageOptions, RecommendBumpPackage,
};

#[derive(Debug, Deserialize, Serialize)]
/// A struct that represents a pnpm workspace.
struct PnpmInfo {
    pub name: String,
    pub path: String,
    pub private: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Workspace {
    pub config: WorkspaceConfig,
    pub repo: Repository,
    pub changes: Changes,
}

impl From<&str> for Workspace {
    fn from(root: &str) -> Self {
        let path_buff = PathBuf::from(root);

        #[cfg(not(windows))]
        let canonic_path = canonicalize(Path::new(path_buff.as_os_str())).expect("Invalid path");

        #[cfg(windows)]
        let canonic_path = path_buff;

        let repo = Repository::new(&canonic_path);
        let changes = Changes::new(&canonic_path);
        let config = get_workspace_config(Some(canonic_path));

        Workspace { config, repo, changes }
    }
}

impl From<WorkspaceConfig> for Workspace {
    fn from(config: WorkspaceConfig) -> Self {
        let repo = Repository::new(&config.workspace_root);
        let changes = Changes::new(&config.workspace_root);
        Workspace { config, repo, changes }
    }
}

impl Workspace {
    pub fn new(root: PathBuf) -> Self {
        let config = get_workspace_config(Some(root));
        let repo = Repository::new(&config.workspace_root);
        let changes = Changes::new(&config.workspace_root);
        Workspace { config, repo, changes }
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
        let since = sha.unwrap_or(String::from("main"));
        let packages_paths =
            packages.iter().map(|pkg| pkg.package_path.to_string()).collect::<Vec<String>>();

        let changed_files = self
            .repo
            .get_all_files_changed_since_branch(&packages_paths, since.as_str())
            .expect("Fail to get changed files");

        packages
            .iter()
            .flat_map(|pkg| {
                let mut pkgs = changed_files
                    .iter()
                    .filter(|file| file.starts_with(pkg.package_path.as_str()))
                    .map(|_| pkg.to_owned())
                    .collect::<Vec<PackageInfo>>();

                pkgs.dedup_by(|a, b| a.package.name == b.package.name);

                pkgs
            })
            .collect::<Vec<PackageInfo>>()
    }

    pub fn get_package_recommend_bump(
        &self,
        package_info: &PackageInfo,
        options: Option<BumpOptions>,
    ) -> RecommendBumpPackage {
        let current_branch = match self.repo.get_current_branch().unwrap_or(None) {
            Some(branch) => branch,
            None => String::from("main"),
        };
        let package_name = &package_info.package.name;
        let package_version = &package_info.package.version.to_string();
        let package_changes =
            self.changes.changes_by_package(package_name, current_branch.as_str());
        let branch_changes = self.changes.changes_by_branch(current_branch.as_str());
        let sha = self.repo.get_current_sha().expect("Error getting current sha");

        let settings = options.unwrap_or(BumpOptions {
            since: None,
            release_as: None,
            fetch_all: None,
            fetch_tags: None,
            sync_deps: None,
            push: None,
        });

        let since = &settings.since.unwrap_or(String::from("origin/main"));
        let release_as = settings.release_as.unwrap_or_else(|| {
            if let Some(change) = package_changes {
                BumpVersion::from(change.release_as.as_str())
            } else {
                BumpVersion::Patch
            }
        });

        let deploy_to = match branch_changes {
            Some(changes) => changes.deploy,
            None => vec![],
        };

        let fetch_all = &settings.fetch_all.unwrap_or(false);
        let fetch_tags = &settings.fetch_tags.unwrap_or(false);

        if *fetch_all {
            self.repo.fetch_all(Some(*fetch_tags)).expect("Error fetching all");
        }

        let tag_info = self.get_last_known_publish_tag_info_for_package(package_info);
        let tag_hash = tag_info.map(|t| t.hash);

        let sem_version = &(match release_as {
            BumpVersion::Major => BumpVersion::bump_major(package_version.as_str()).to_string(),
            BumpVersion::Minor => BumpVersion::bump_minor(package_version.as_str()).to_string(),
            BumpVersion::Patch => BumpVersion::bump_patch(package_version.as_str()).to_string(),
            BumpVersion::Snapshot => {
                BumpVersion::bump_snapshot(package_version.as_str(), sha.as_str()).to_string()
            }
        });

        let changed_files = self
            .repo
            .get_all_files_changed_since_sha(since.as_str())
            .expect("Error getting changed files");

        let commits_since = self
            .repo
            .get_commits_since(
                tag_hash.clone(),
                Some(package_info.package_relative_path.to_string()),
            )
            .expect("Error getting commits since");

        let conventional_package = get_conventional_by_package(
            package_info,
            &ConventionalPackageOptions {
                config: self.config.cliff_config.clone(),
                commits: commits_since,
                version: Some(sem_version.to_string()),
                tag: tag_hash,
            },
        );

        RecommendBumpPackage {
            from: package_version.to_string(),
            to: sem_version.to_string(),
            package_info: package_info.to_owned(),
            conventional: conventional_package,
            changed_files,
            deploy_to,
        }
    }

    #[allow(clippy::default_trait_access)]
    pub fn get_last_known_publish_tag_info_for_package(
        &self,
        package_info: &PackageInfo,
    ) -> Option<RepositoryPublishTagInfo> {
        let mut remote_tags = self
            .repo
            .get_remote_or_local_tags(Some(false))
            .or_else(|_| Ok::<Vec<RepositoryTags>, RepositoryError>(vec![]))
            .expect("Error getting remote tags");

        let mut local_tags = self
            .repo
            .get_remote_or_local_tags(Some(true))
            .or_else(|_| Ok::<Vec<RepositoryTags>, RepositoryError>(vec![]))
            .expect("Error getting local tags");

        remote_tags.append(&mut local_tags);

        let mut options = CollatorOptions::new();
        options.strength = Some(Strength::Secondary);
        options.numeric = Some(Numeric::On);

        let collator = Collator::try_new(&Default::default(), options).unwrap();

        remote_tags.sort_by(|a, b| {
            let tag_a = a.tag.replace("refs/tags/", "");
            let tag_b = b.tag.replace("refs/tags/", "");

            collator.compare(&tag_b, &tag_a)
        });

        let package_name = &package_info.package.name;
        let package_version = &package_info.package.version.to_string();
        let package_tag = format!("{package_name}@{package_version}");

        let mut match_tag = remote_tags.iter().find(|item| {
            let tag = item.tag.replace("refs/tags/", "");
            let matches: Vec<&str> = tag.matches(&package_tag).collect();

            !matches.is_empty()
        });

        if match_tag.is_none() {
            let mut highest_tag = None;

            remote_tags.iter().for_each(|item| {
                let tag = &item.tag.replace("refs/tags/", "");

                if tag.contains(&package_info.package.name) {
                    if highest_tag.is_none() {
                        highest_tag = Some(String::from(tag));
                    }

                    let high_tag = highest_tag.as_ref().unwrap();
                    let current_tag_meta = package_scope_name_version(tag).unwrap();
                    let highest_tag_meta = package_scope_name_version(high_tag).unwrap();

                    let current_version = Version::from(&current_tag_meta.version).unwrap();
                    let highest_version = Version::from(&highest_tag_meta.version).unwrap();

                    if current_version.compare_to(&highest_version, Cmp::Gt) {
                        highest_tag = Some(String::from(tag));
                    }
                }
            });

            if highest_tag.is_some() {
                let highest_tag = highest_tag.unwrap();
                let highest_tag_meta = package_scope_name_version(&highest_tag).unwrap();

                match_tag = remote_tags.iter().find(|item| {
                    let tag = item.tag.replace("refs/tags/", "");
                    let matches: Vec<&str> = tag.matches(&highest_tag_meta.full).collect();

                    !matches.is_empty()
                });
            }
        }

        if match_tag.is_some() {
            let hash = &match_tag.unwrap().hash;
            let tag = &match_tag.unwrap().tag;
            let package = &package_info.package.name;

            return Some(RepositoryPublishTagInfo {
                hash: hash.to_string(),
                tag: tag.to_string(),
                package: package.to_string(),
            });
        }

        None
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
                let package_path =
                    canonicalize(pkgs.path.clone()).expect("Failed to canonic package path");
                //let package_path = PathBuf::from(pkgs.path.clone());
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
