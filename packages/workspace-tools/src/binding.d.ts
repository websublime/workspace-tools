import type { RepositoryCommit, RepositoryRemoteTags } from './types/repo';

import type { PackageManager } from './types/enums';

import type { WorkspaceConfig } from './types/config';

import type { Result } from './types/general';

import type { Changes, Change, ChangeMeta } from './types/changes';

import type { PackageScopeMetadata } from './types/package';

import type { PackageInfo, RecommendBumpPackage } from './types/package';

export declare class Dependency {
  constructor(name: string, version: string)
  get name(): string
  get version(): string
}

export declare class Package {
  constructor(name: string, version: string, deps?: Array<Dependency>)
  updateVersion(version: string): void
  updateDependencyVersion(name: string, version: string): void
  get name(): string
  get version(): string
  get dependencies(): Array<Dependency>
}

export declare class Workspace {
  constructor(root: string)
  getPackages(): Result<Array<PackageInfo>>
  getPackageInfo(packageName: string): Result<PackageInfo>
  getChangedPackages(sha?: string | undefined | null): Result<Array<PackageInfo>>
  getPackageRecommendBump(packageName: string, bumpOptions?: BumpOptions | undefined | null): Result<RecommendBumpPackage>
  getBumps(bumpOptions?: BumpOptions | undefined | null): Result<Array<RecommendBumpPackage>>
}

/**
 * Add a change to the changes file.
 * If the change already exists, it will return false.
 * If the change does not exist, it will add the change and return true.
 *
 * @param {Object} change - The change object.
 * @param {string[]} deploy_envs - The deploy environments.
 * @param {string} cwd - The current working directory.
 * @returns {boolean} - If the change was added successfully.
 * @throws {Error} - If it fails to get the package property.
 */
export declare function addChange(change: Change, deploy_envs?: string[], cwd?: string): Result<boolean>

/**
 * Bumps the version of the package to major.
 *
 * @param {string} version - The version of the package.
 * @returns {string} The new version of the package.
 */
export declare function bumpMajor(version: string): string

/**
 * Bumps the version of the package to minor.
 *
 * @param {string} version - The version of the package.
 * @returns {string} The new version of the package.
 */
export declare function bumpMinor(version: string): string

export interface BumpOptions {
  since?: string
  releaseAs?: string
  fetchAll?: boolean
  fetchTags?: boolean
  syncDeps?: boolean
  push?: boolean
}

/**
 * Bumps the version of the package to patch.
 *
 * @param {string} version - The version of the package.
 * @returns {string} The new version of the package.
 */
export declare function bumpPatch(version: string): string

/**
 * Bumps the version of the package to snapshot.
 *
 * @param {string} version - The version of the package.
 * @param {string} snapshot - The snapshot.
 * @returns {string} The new version of the package.
 */
export declare function bumpSnapshot(version: string, snapshot: string): string

/**
 * Check if a change exists in the changes file.
 * If the change exists, it will return true.
 * If the change does not exist, it will return false.
 *
 * @param {string} branch - The branch name.
 * @param {string} package - The package name.
 * @param {string} cwd - The current working directory.
 * @returns {boolean} - If the change exists.
 */
export declare function changeExists(branch: string, package: string, cwd?: string): boolean

/**
 * Detect the package manager.
 *
 * @param {string} cwd - The current working directory.
 * @returns {string} The package manager.
 */
export declare function detectPackageManager(cwd: string): Result<PackageManager>

/**
 * Execute a command.
 *
 * @param {string} cmd - The command to execute.
 * @param {string} cwd - The command working directory.
 * @param {string[]} args - The command arguments.
 * @returns {string} The command output.
 *
 * @throws {Error} The error description.
 */
export declare function executeCmd(cmd: string, cwd: string, args?: Array<string> | undefined | null): Result<string>

/**
 * Get all changes from the changes file.
 *
 * @param {string} cwd - The current working directory.
 * @returns {Object} - The changes object.
 * @throws {Error} - If it fails to create the object.
 */
export declare function getChanges(cwd?: string): Result<Changes>

/**
 * Get all changes by branch from the changes file.
 *
 * @param {string} branch - The branch name.
 * @param {string} cwd - The current working directory.
 * @returns {Object} - The changes object.
 * @throws {Error} - If it fails to create the object/parsing/invalid.
 */
export declare function getChangesByBranch(branch: string, cwd?: string): Result<{deploy: string[]; pkgs: Changes[]}|null>

/**
 * Get all changes by package from the changes file.
 *
 * @param {string} package - The package name.
 * @param {string} branch - The branch name.
 * @param {string} cwd - The current working directory.
 * @returns {Object} - The changes object.
 * @throws {Error} - If it fails to create the object/parsing/invalid.
 */
export declare function getChangesByPackage(package: string, branch: string, cwd?: string): Result<Change|null>

/**
 * Get all changes meta by package from the changes file.
 * It will return an empty array if no changes are found.
 *
 * @param {string} package - The package name.
 * @param {string} cwd - The current working directory.
 * @returns {Array<ChangeMeta>} - The changes meta object.
 * @throws {Error} - If it fails to create the object/parsing/invalid.
 */
export declare function getChangesMetaByPackage(package: string, cwd?: string): Result<Array<ChangeMeta>>

export declare function getConfig(cwd?: string): Result<WorkspaceConfig>

/**
 * Get package dependents
 *
 * @param {Array<Package>} packages - The packages to get dependents from.
 * @returns {Object} - The package dependents.
 */
export declare function getPackageDependents(packages: Array<Package>): Record<string, Array<string>>

/**
 * Get package scope name version and path
 *
 * @param {string} pk_name_scope_name_version - The package name, version and optional file path.
 * @returns {Object} - The package scope name version and path.
 */
export declare function getPackageScopeNameVersion(pkNameScopeNameVersion: string): Result<PackageScopeMetadata>

/**
 * Get the workspace root path.
 *
 * @param {string} cwd - The current working directory.
 * @returns {string} The project(workspace) root path.
 */
export declare function getProjectRootPath(cwd?: string | undefined | null): string | null

/**
 * Initialize the changes file. If the file does not exist, it will create it with the default message.
 * If the file exists, it will return the content of the file.
 *
 * @param {string} cwd - The current working directory.
 * @throws {Error} - If it fails to create the object.
 */
export declare function initChanges(cwd?: string | undefined | null): Result<Changes>

/**
 * Check if the current working directory is a VCS repository
 *
 * @param {string} cwd - The current working directory
 * @returns {boolean} - True if the current working directory is a VCS repository
 */
export declare function isVcsRepository(cwd: string): Result<boolean>

/**
 * Remove a change from the changes file.
 * If the change does not exist, it will return false.
 * If the change exists, it will remove the change and return true.
 *
 * @param {string} branch - The branch name.
 * @param {string} cwd - The current working directory.
 * @returns {boolean} - If the change was removed successfully.
 * @throws {Error} - If it fails to remove the change.
 */
export declare function removeChange(branch: string, cwd?: string): boolean

/**
 * Add a file in the repository
 *
 * @param {string} filepath - The file path to add
 * @param {string} cwd - The current working directory
 * @returns {boolean} - True if the file was added successfully
 */
export declare function repoAdd(filepath: string, cwd: string): Result<boolean>

/**
 * Add all files in the repository
 *
 * @param {string} cwd - The current working directory
 * @returns {boolean} - True if all files were added successfully
 */
export declare function repoAddAll(cwd: string): Result<boolean>

/**
 * Checkout a branch in the repository
 *
 * @param {string} branch - The branch to checkout
 * @param {string} cwd - The current working directory
 * @returns {boolean} - True if the branch was checked out successfully
 */
export declare function repoCheckout(branch: string, cwd: string): Result<boolean>

/**
 * Commit changes in the repository
 *
 * @param {string} cwd - The current working directory
 * @param {string} message - The message to use for the commit
 * @param {string} body - The body to use for the commit
 * @param {string} footer - The footer to use for the commit
 * @returns {boolean} - True if the changes were committed successfully
 */
export declare function repoCommit(cwd: string, message: string, body?: string | undefined | null, footer?: string | undefined | null): Result<boolean>

/**
 * Configure the repository with the given username and email
 *
 * @param {string} username - The username to use for the repository
 * @param {string} email - The email to use for the repository
 * @param {string} cwd - The current working directory
 * @returns {boolean} - True if the repository was configured successfully
 */
export declare function repoConfig(username: string, email: string, cwd: string): Result<boolean>

/**
 * Create a new branch in the repository
 *
 * @param {string} branch - The branch to create
 * @param {string} cwd - The current working directory
 * @returns {boolean} - True if the branch was created successfully
 */
export declare function repoCreateBranch(branch: string, cwd: string): Result<boolean>

/**
 * Create a tag in the repository
 *
 * @param {string} tag - The tag to create
 * @param {string} cwd - The current working directory
 * @param {string} message - The message to use for the tag
 * @returns {boolean} - True if the tag was created successfully
 */
export declare function repoCreateTag(tag: string, cwd: string, message?: string | undefined | null): Result<boolean>

/**
 * Diff all changes in the repository
 *
 * @param {string} cwd - The current working directory
 * @param {string[]} target - The target to diff
 * @returns {string} - The diff of changes in the repository
 */
export declare function repoDiff(cwd: string, target?: Array<string> | undefined | null): Result<string>

/**
 * Fetch all changes in the repository
 *
 * @param {string} cwd - The current working directory
 * @param {boolean} fetch_tags - The flag to fetch tags
 * @returns {boolean} - True if all changes were fetched successfully
 */
export declare function repoFetchAll(cwd: string, fetchTags?: boolean | undefined | null): Result<boolean>

/**
 * Get all files changed since the branch in the repository
 *
 * @param {string} cwd - The current working directory
 * @param {string[]} packages - The packages to get all files changed since the branch
 * @param {string} branch - The branch to get all files changed since
 * @returns {string[]} - The list of files changed since the branch in the repository
 */
export declare function repoGetAllFilesChangedSinceBranch(cwd: string, packages: Array<string>, branch: string): Result<string[]>

/**
 * Get all files changed since the sha in the repository
 *
 * @param {string} cwd - The current working directory
 * @param {string} sha - The sha to get all files changed since
 * @returns {string[]} - The list of files changed since the sha in the repository
 */
export declare function repoGetAllFilesChangedSinceSha(cwd: string, sha: string): Result<string[]>

/**
 * Get the branch from the commit in the repository
 *
 * @param {string} sha - The commit sha to get the branch
 * @param {string} cwd - The current working directory
 * @returns {string} - The branch from the commit in the repository
 */
export declare function repoGetBranchFromCommit(sha: string, cwd: string): Result<string|null>

/**
 * Get all commits since the sha in the repository
 *
 * @param {string} cwd - The current working directory
 * @param {string} since - The sha to get all commits since
 * @param {string} relative - The relative path to get all commits since
 * @returns {RepositoryCommit[]} - The list of commits since the sha in the repository
 */
export declare function repoGetCommitsSince(cwd: string, since?: string | undefined | null, relative?: string | undefined | null): Result<RepositoryCommit[]>

/**
 * Get the current branch in the repository
 *
 * @param {string} cwd - The current working directory
 * @returns {string} - The current branch in the repository
 */
export declare function repoGetCurrentBranch(cwd: string): Result<string|null>

/**
 * Get the current sha in the repository
 *
 * @param {string} cwd - The current working directory
 * @returns {string} - The current sha in the repository
 */
export declare function repoGetCurrentSha(cwd: string): Result<string>

/**
 * Get the diverged commit in the repository
 *
 * @param {string} sha - The commit sha to get the diverged commit
 * @param {string} cwd - The current working directory
 * @returns {string} - The diverged commit in the repository
 */
export declare function repoGetDivergedCommit(sha: string, cwd: string): Result<string>

/**
 * Get the first sha in the repository
 *
 * @param {string} cwd - The current working directory
 * @param {string} branch - The branch to get the first sha
 * @returns {string} - The first sha in the repository
 */
export declare function repoGetFirstSha(cwd: string, branch?: string | undefined | null): Result<string>

/**
 * Get the last tag in the repository
 *
 * @param {string} cwd - The current working directory
 * @returns {string} - The last tag in the repository
 */
export declare function repoGetLastTag(cwd: string): Result<string>

/**
 * Get the previous sha in the repository
 *
 * @param {string} cwd - The current working directory
 * @returns {string} - The previous sha in the repository
 */
export declare function repoGetPreviousSha(cwd: string): Result<string>

/**
 * Get all local/remote tags in the repository
 *
 * @param {string} cwd - The current working directory
 * @param {boolean} local - The flag to get local tags
 * @returns {RepositoryRemoteTags[]} - The list of tags in the repository
 */
export declare function repoGetTags(cwd: string, local?: boolean | undefined | null): Result<RepositoryRemoteTags[]>

/**
 * Initialize a new repository
 *
 * @param {string} initial_branch - The initial branch to create
 * @param {string} username - The username to use for the repository
 * @param {string} email - The email to use for the repository
 * @param {string} cwd - The current working directory
 * @returns {boolean} - True if the repository was initialized successfully
 */
export declare function repoInit(initialBranch: string, username: string, email: string, cwd: string): Result<boolean>

/**
 * Check if the repository is a VCS repository
 *
 * @param {string} cwd - The current working directory
 * @returns {boolean} - True if the repository is a VCS repository
 */
export declare function repoIsVcs(cwd: string): Result<boolean>

/**
 * List all branches in the repository
 *
 * @param {string} cwd - The current working directory
 * @returns {string} - The list of branches in the repository
 */
export declare function repoListBranches(cwd: string): Result<string>

/**
 * List all configurations in the repository
 *
 * @param {string} config_type - The type of configuration to list
 * @param {string} cwd - The current working directory
 * @returns {string} - The list of configurations in the repository
 */
export declare function repoListConfig(configType: string, cwd: string): Result<string>

/**
 * Log all commits in the repository
 *
 * @param {string} cwd - The current working directory
 * @param {string} target - The target to log
 * @returns {string} - The log of commits in the repository
 */
export declare function repoLog(cwd: string, target?: string | undefined | null): Result<string>

/**
 * Merge a branch in the repository
 *
 * @param {string} branch - The branch to merge
 * @param {string} cwd - The current working directory
 * @returns {boolean} - True if the branch was merged successfully
 */
export declare function repoMerge(branch: string, cwd: string): Result<boolean>

/**
 * Push changes in the repository
 *
 * @param {string} cwd - The current working directory
 * @param {boolean} follow_tags - The flag to follow tags
 * @returns {boolean} - True if the changes were pushed successfully
 */
export declare function repoPush(cwd: string, followTags?: boolean | undefined | null): Result<boolean>

/**
 * Get the repository status
 *
 * @param {string} cwd - The current working directory
 * @returns {string} - The repository status
 */
export declare function repoStatus(cwd: string): Result<string|null>
