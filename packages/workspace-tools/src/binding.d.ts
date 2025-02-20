import type { RepositoryCommit, RepositoryRemoteTags } from './types/repo';

import type { PackageManager } from './types/enums';

import type { WorkspaceConfig } from './types/config';

import type { Result } from './types/general';

import type { Changes, Change, ChangeMeta } from './types/changes';

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
export declare function executeCmd(cmd: string, cwd: string, args?: Array<string> | undefined | null): Result<String>

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

export declare function repoAdd(filepath: string, cwd: string): Result<boolean>

export declare function repoAddAll(cwd: string): Result<boolean>

export declare function repoCheckout(branch: string, cwd: string): Result<boolean>

export declare function repoCommit(cwd: string, message: string, body?: string | undefined | null, footer?: string | undefined | null): Result<boolean>

export declare function repoConfig(username: string, email: string, cwd: string): Result<boolean>

export declare function repoCreateBranch(branch: string, cwd: string): Result<boolean>

export declare function repoCreateTag(tag: string, cwd: string, message?: string | undefined | null): Result<boolean>

export declare function repoFetchAll(cwd: string, fetchTags?: boolean | undefined | null): Result<boolean>

export declare function repoGetAllFilesChangedSinceBranch(cwd: string, packages: Array<string>, branch: string): Result<string[]>

export declare function repoGetAllFilesChangedSinceSha(cwd: string, sha: string): Result<string[]>

export declare function repoGetBranchFromCommit(sha: string, cwd: string): Result<string|null>

export declare function repoGetCommitsSince(cwd: string, since?: string | undefined | null, relative?: string | undefined | null): Result<RepositoryCommit[]>

export declare function repoGetCurrentBranch(cwd: string): Result<string|null>

export declare function repoGetCurrentSha(cwd: string): Result<string>

export declare function repoGetDivergedCommit(sha: string, cwd: string): Result<string>

export declare function repoGetFirstSha(cwd: string, branch?: string | undefined | null): Result<string>

export declare function repoGetPreviousSha(cwd: string): Result<string>

export declare function repoGetTags(cwd: string, local?: boolean | undefined | null): Result<RepositoryRemoteTags[]>

export declare function repoInit(initialBranch: string, username: string, email: string, cwd: string): Result<boolean>

export declare function repoIsVcs(cwd: string): Result<boolean>

export declare function repoMerge(branch: string, cwd: string): Result<boolean>

export declare function repoPush(cwd: string, followTags?: boolean | undefined | null): Result<boolean>
