import type { RepositoryCommit, RepositoryRemoteTags } from './types/repo';

import type { PackageManager } from './types/enums';

import type { WorkspaceConfig } from './types/config';

import type { Result } from './types/general';

import type { Changes, Change } from './types/changes';

export declare function addChange(change: Change, deploy_envs?: string[], cwd?: string): Result<boolean>

export declare function changeExists(branch: string, package: string, cwd?: string): boolean

export declare function detectPackageManager(cwd: string): Result<PackageManager>

export declare function getChanges(cwd?: string): Result<Changes>

export declare function getChangesByBranch(branch: string, cwd?: string): Result<{deploy: string[]; pkgs: Changes[]}|null>

export declare function getChangesByPackage(package: string, branch: string, cwd?: string): Result<Change|null>

export declare function getConfig(cwd?: string): Result<WorkspaceConfig>

export declare function getProjectRootPath(cwd?: string | undefined | null): string | null

export declare function initChanges(cwd?: string | undefined | null): Result<Changes>

export declare function isVcsRepository(cwd: string): Result<boolean>

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
