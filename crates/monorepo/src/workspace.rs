use icu::collator::{Collator, CollatorOptions, Numeric, Strength};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{canonicalize, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::{fs::File, io::BufReader, path::PathBuf};
use version_compare::{Cmp, Version};
use wax::{CandidatePath, Glob, Pattern};
use ws_git::error::RepositoryError;
use ws_git::repo::{Repository, RepositoryPublishTagInfo, RepositoryTags};
use ws_pkg::bump::BumpOptions;
use ws_pkg::dependency::{DependencyGraph, Node};
use ws_pkg::package::{
    build_dependency_graph_from_package_infos, package_scope_name_version, Dependency, Package,
    PackageInfo, PackageJson,
};
use ws_pkg::version::Version as BumpVersion;
use ws_std::manager::CorePackageManager;

use crate::changes::{Change, Changes};
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

    pub fn get_changed_packages(
        &self,
        sha: Option<String>,
    ) -> (Vec<PackageInfo>, HashMap<String, Vec<String>>) {
        let packages = &self.get_packages();
        let since = sha.unwrap_or(String::from("feature-branch"));
        let packages_paths =
            packages.iter().map(|pkg| pkg.package_path.to_string()).collect::<Vec<String>>();

        let changed_files = match Some(since.contains("main")) {
            Some(true) => {
                let last_tag = self.repo.get_last_tag().expect("Error getting last tag");

                let diff_files = self
                    .repo
                    .diff(Some(vec!["--name-only".to_string(), last_tag.to_string()]))
                    .expect("Error get files diff from latest tag");

                diff_files
                    .split('\n')
                    .filter(|item| !item.trim().is_empty())
                    .map(|item| self.config.workspace_root.join(item))
                    .filter(|item| item.exists())
                    .map(|item| {
                        item.to_str().expect("Failed to convert path to string").to_string()
                    })
                    .collect::<Vec<String>>()
            }
            Some(false) | None => self
                .repo
                .get_all_files_changed_since_branch(&packages_paths, "main")
                .expect("Fail to get changed files"),
        };

        let mut files_map = HashMap::new();

        let changed_packages = packages
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
            .collect::<Vec<PackageInfo>>();

        for pkg in packages {
            let pkg_files: Vec<String> = changed_files
                .iter()
                .filter(|file| file.starts_with(&pkg.package_path))
                .cloned()
                .collect();

            if !pkg_files.is_empty() {
                files_map.insert(pkg.package.name.clone(), pkg_files);
            }
        }

        (changed_packages, files_map)
    }

    pub fn get_package_recommend_bump(
        &self,
        package_info: &PackageInfo,
        options: Option<BumpOptions>,
    ) -> RecommendBumpPackage {
        let package_name = &package_info.package.name;
        let package_version = &package_info.package.version.to_string();
        let package_changes_list = self.changes.changes_by_package_name(package_name);
        let package_changes = package_changes_list.first().or(None);
        let branch_meta_changes = self.changes.get_changes_meta_by_package_name(package_name);
        let branch_changes = branch_meta_changes.first().or(None);
        let sha = self.repo.get_current_sha().expect("Error getting current sha");

        let settings = options.unwrap_or(BumpOptions {
            since: None,
            release_as: None,
            fetch_all: None,
            fetch_tags: None,
            sync_deps: None,
            push: None,
        });

        let since = settings.since;
        let release_as = settings.release_as.unwrap_or_else(|| {
            if let Some(change) = package_changes {
                BumpVersion::from(change.release_as.as_str())
            } else {
                BumpVersion::Patch
            }
        });

        let deploy_to = match branch_changes {
            Some(changes) => changes.deploy.clone(),
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

        // Create a mutable clone of package_info so we can update it
        let mut updated_package_info = package_info.clone();

        // Update the package version
        updated_package_info.package.update_version(sem_version.as_str());

        // Update the package.json version
        updated_package_info.update_version(sem_version.as_str());

        let (_, packages_changed_files) = self.get_changed_packages(since);
        let changed_files = packages_changed_files.get(package_name).unwrap_or(&vec![]).clone();

        let commits_since = self
            .repo
            .get_commits_since(
                tag_hash.clone(),
                Some(package_info.package_relative_path.to_string()),
            )
            .expect("Error getting commits since");

        let conventional_package = get_conventional_by_package(
            &updated_package_info,
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
            package_info: updated_package_info,
            conventional: conventional_package,
            changed_files,
            deploy_to,
        }
    }

    pub fn get_bumps(&self, options: &BumpOptions) -> Vec<RecommendBumpPackage> {
        if options.fetch_all.unwrap_or(false) {
            self.repo
                .fetch_all(Some(options.fetch_tags.unwrap_or(false)))
                .expect("Error fetching repository");
        }

        let since = options.since.clone();
        let current_branch = self.repo.get_current_branch().unwrap_or(None);
        let branch = current_branch.unwrap_or(String::from("default-feature-branch"));

        let (repo_changed_packages, _packages_changed_files) =
            self.get_changed_packages(since.clone());

        if repo_changed_packages.is_empty() {
            return vec![];
        }

        let changed_packages = repo_changed_packages
            .into_iter()
            .filter(|changed_package| {
                !self
                    .config
                    .tools_config
                    .tools
                    .internal_packages
                    .contains(&changed_package.package.name)
            })
            .collect::<Vec<PackageInfo>>();

        let mut packages = Vec::new();
        let all_packages = self.get_packages();
        let dependency_graph =
            build_dependency_graph_from_package_infos(&all_packages, &mut packages);
        let mut bumps = Vec::new();
        let mut dependency_updates = HashMap::new();

        // First pass: Process changed packages and collect their version updates
        for changed_package in &changed_packages {
            let package_name = &changed_package.package.name;
            let changes_vec = self.changes.changes_by_package_name(package_name.as_str());
            let changes = changes_vec.iter().find(|change| change.package == *package_name);

            let calculated_release_as = match Some(branch.contains("main")) {
                Some(true) => match changes {
                    Some(change) => BumpVersion::from(change.release_as.as_str()),
                    None => BumpVersion::Patch,
                },
                Some(false) | None => BumpVersion::Snapshot,
            };

            let override_release_as = options.release_as.or(Some(calculated_release_as));

            let bump = self.get_package_recommend_bump(
                changed_package,
                Some(BumpOptions {
                    sync_deps: Some(false),
                    since: since.clone(),
                    release_as: override_release_as,
                    fetch_all: options.fetch_all,
                    fetch_tags: options.fetch_tags,
                    push: options.push,
                }),
            );

            dependency_updates.insert(package_name.clone(), bump.to.clone());
            bumps.push(bump);
        }

        if options.sync_deps.unwrap_or(false) {
            let mut dependent_bumps = Vec::new();

            for changed_package in &changed_packages {
                let package_name = &changed_package.package.name;
                let dependents = dependency_graph
                    .get_dependents(package_name)
                    .expect("Error getting dependents");

                for dependent_name in dependents {
                    // Skip if we already processed this package
                    if bumps.iter().any(|b| b.package_info.package.name == *dependent_name) {
                        continue;
                    }

                    let dependent_package_info = self.get_package_info(dependent_name);
                    if let Some(mut dependent_package_info) = dependent_package_info {
                        // Update dependency version in package.json and package info
                        if let Some(new_version) = dependency_updates.get(package_name) {
                            dependent_package_info
                                .update_dependency_version(package_name, new_version);
                            dependent_package_info
                                .package
                                .update_dependency_version(package_name, new_version);
                        }

                        let calculated_dependent_release_as = match Some(branch.contains("main")) {
                            Some(true) => BumpVersion::Patch,
                            Some(false) | None => BumpVersion::Snapshot,
                        };

                        let dependent_bump = self.get_package_recommend_bump(
                            &dependent_package_info,
                            Some(BumpOptions {
                                since: since.clone(),
                                release_as: Some(calculated_dependent_release_as),
                                fetch_all: options.fetch_all,
                                fetch_tags: options.fetch_tags,
                                sync_deps: options.sync_deps,
                                push: options.push,
                            }),
                        );

                        dependent_bumps.push(dependent_bump);
                    }
                }
            }

            bumps.extend(dependent_bumps);
        }

        bumps
    }
    /*#[allow(clippy::too_many_lines)]
    pub fn get_bumps(&self, options: &BumpOptions, write: Option<bool>) -> Vec<RecommendBumpPackage> {
        if options.fetch_all.unwrap_or(false) {
            self.repo
                .fetch_all(Some(options.fetch_tags.unwrap_or(false)))
                .expect("Error fetching repository");
        }

        let since = &(match options.since {
            Some(ref since) => since.to_string(),
            None => String::from("origin/main"),
        });

        let current_branch = match self.repo.get_current_branch().unwrap_or(None) {
            Some(branch) => branch,
            None => String::from("main"),
        };

        let changed_packages = self.get_changed_packages(Some(since.to_string()));

        if changed_packages.is_empty() {
            return vec![];
        }

        let mut bump_changes = HashMap::new();
        let mut bump_dependencies = HashMap::new();
        let packages = self.get_packages();

        for changed_package in &changed_packages {
            let package_name = &changed_package.package.name;
            let override_release_as = options.release_as;
            // TODO: fix here
            let changes = self.changes.changes_by_package_name(package_name.as_str());
            let change = changes.first().or(None);

            if let Some(chg) = change {
                let calculated_release_as = match Some(current_branch.contains("main")) {
                    Some(true) => BumpVersion::from(chg.release_as.as_str()),
                    Some(false) | None => BumpVersion::Snapshot,
                };

                let release_as = override_release_as.unwrap_or(calculated_release_as);

                let package_change = Change {
                    package: package_name.to_string(),
                    release_as: release_as.to_string(),
                };

                bump_changes.insert(package_name.to_string(), package_change);
            }

            if options.sync_deps.unwrap_or(false) {
                packages.iter().for_each(|pkg| {
                    pkg.package.dependencies.iter().for_each(|dependency| {
                        let calculated_release_as = match Some(current_branch.contains("main")) {
                            Some(true) => BumpVersion::Patch,
                            Some(false) | None => BumpVersion::Snapshot,
                        };
                        let release_as = override_release_as.unwrap_or(calculated_release_as);

                        if dependency.name == changed_package.package.name
                            && change.is_some()
                            && !bump_changes.contains_key(&pkg.package.name)
                        {
                            bump_changes.insert(
                                pkg.package.name.to_string(),
                                Change {
                                    package: pkg.package.name.to_string(),
                                    release_as: release_as.to_string(),
                                },
                            );
                        }
                    });
                });
            }
        }

        let mut bumps = bump_changes
            .iter()
            .map(|(name, change)| {
                let override_release_as = options.release_as;
                let package =
                    self.get_package_info(name).expect("Error getting package info on bump.");

                let calculated_release_as = match Some(current_branch.contains("main")) {
                    Some(true) => BumpVersion::from(change.release_as.as_str()),
                    Some(false) | None => BumpVersion::Snapshot,
                };

                let release_as = override_release_as.unwrap_or(calculated_release_as);

                let bump = self.get_package_recommend_bump(
                    &package,
                    Some(BumpOptions {
                        since: Some(since.to_string()),
                        release_as: Some(release_as),
                        fetch_all: options.fetch_all,
                        fetch_tags: options.fetch_tags,
                        sync_deps: options.sync_deps,
                        push: options.push,
                    }),
                );

                if !bump.package_info.dependencies().is_empty() {
                    bump_dependencies.insert(
                        bump.package_info.package.name.to_string(),
                        bump.package_info.dependencies().to_owned(),
                    );
                }

                bump
            })
            .collect::<Vec<RecommendBumpPackage>>();

        bumps.iter_mut().for_each(|bump| {
            let version = bump.to.as_str();
            bump.package_info.package.update_dependency(version);
            bump.package_info.update_dependency(version);

            bump.conventional.package_info.update_dependency(version);
            bump.conventional.package_info.package.update_dependency(version);

            match write {
                Some(true) => {
                    bump.package_info.write_package_json();
                    let body = format!("Package: {} updated to version: {}", &bump.package_info.package.name, &bump.to);
                    self.repo.add(PathBuf::from(&bump.package_info.package_json_path).as_path()).expect("Failed to add package.json");
                    self.repo.commit("chore: update package.json version", Some(body), None).expect("Failed to commit chore");
                },
                Some(false) | None => {
                    // Nothing todo
                },
            }
        });

        if options.sync_deps.unwrap_or(false) {
            bump_dependencies.iter().for_each(|(package_name, dependencies)| {
                let temp_bumps = bumps.clone();
                let bump = bumps
                    .iter_mut()
                    .find(|b| b.package_info.package.name == *package_name)
                    .expect("Error finding bump dependency");

                for dep in dependencies {
                    let bump_dep =
                        temp_bumps.iter().find(|pkgs| pkgs.package_info.package.name == dep.name);

                    if let Some(dep_bump) = bump_dep {
                        bump.package_info.update_dependency_version(&dep.name, &dep_bump.to);
                        bump.conventional
                            .package_info
                            .update_dependency_version(&dep.name, &dep_bump.to);
                        bump.package_info
                            .package
                            .update_dependency_version(&dep.name, &dep_bump.to);
                        bump.conventional
                            .package_info
                            .package
                            .update_dependency_version(&dep.name, &dep_bump.to);

                        bump.package_info.update_dev_dependency_version(&dep.name, &dep_bump.to);
                        bump.conventional
                            .package_info
                            .update_dev_dependency_version(&dep.name, &dep_bump.to);
                        bump.package_info
                            .package
                            .update_dev_dependency_version(&dep.name, &dep_bump.to);
                        bump.conventional
                            .package_info
                            .package
                            .update_dev_dependency_version(&dep.name, &dep_bump.to);

                        match write {
                            Some(true) => {
                                bump.package_info.write_package_json();
                                let body = format!("Package: {} updated to version: {}", &bump.package_info.package.name, &bump.to);
                                self.repo.add(PathBuf::from(&bump.package_info.package_json_path).as_path()).expect("Failed to add package.json");
                                self.repo.commit("chore: update package.json version dependencies", Some(body), None).expect("Failed to commit chore");
                            },
                            Some(false) | None => {
                                // Nothing todo
                            },
                        }
                    }
                }
            });
        }

        bumps
    }

    pub fn apply_bumps(&self, options: &BumpOptions) -> Vec<RecommendBumpPackage> {
        let bumps = self.get_bumps(options, Some(true));

        if !bumps.is_empty() {
            let git_message =
                self.config.changes_config.get("message").expect("Error getting git message");
            let git_author = self
                .config
                .changes_config
                .get("git_user_name")
                .expect("Error getting git author");
            let git_email = self
                .config
                .changes_config
                .get("git_user_email")
                .expect("Error getting git email");

            for bump in &bumps {
                let package_json_file_path =
                    PathBuf::from(bump.package_info.package_json_path.to_string());
                let changelog_file_path =
                    PathBuf::from(bump.conventional.package_info.package_path.to_string())
                        .join("CHANGELOG.md");

                let package_json_file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&package_json_file_path)
                    .expect("Error opening package.json file");
                let package_json_writer = BufWriter::new(package_json_file);
                serde_json::to_writer_pretty(package_json_writer, &bump.package_info.pkg_json)
                    .expect("Error writing package.json file");

                let mut changelog_file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(&changelog_file_path)
                    .expect("Error opening CHANGELOG.md file");
                changelog_file
                    .write_all(bump.conventional.changelog_output.as_bytes())
                    .expect("Error writing CHANGELOG.md file");

                let package_tag = &format!("{}@{}", bump.package_info.package.name, bump.to);
                let commit_body = format!(
                    "Release of package {} with version: {}",
                    bump.package_info.package.name, bump.to
                );

                self.repo
                    .config(git_author.as_str(), git_email.as_str())
                    .expect("Error configuring author name and email");
                self.repo.add_all().expect("Error adding all files");
                self.repo
                    .commit(&git_message.replace("{tag}", package_tag), Some(commit_body), None)
                    .expect("Error committing changes");

                self.repo
                    .tag(
                        package_tag.as_str(),
                        Some(format!(
                            "chore: release {} to version {}",
                            bump.package_info.package.name, bump.to
                        )),
                    )
                    .expect("Error tagging package");

                if options.push.unwrap_or(false) {
                    self.repo.push(Some(true)).expect("Error pushing changes");
                }
            }
        }

        bumps
    }*/

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

                let package = Package::new(
                    pkg_json.name.as_str(),
                    pkg_json.version.as_str(),
                    Some(package_dependencies),
                );

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

                let package = Package::new(
                    pkg_json.name.as_str(),
                    pkg_json.version.as_str(),
                    Some(package_dependencies),
                );

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
