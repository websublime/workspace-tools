# `@websublime/workspace-tools`

![https://github.com/websublime/workspace-tools/actions](https://github.com/websublime/workspace-tools/workflows/CI/badge.svg)

> Tools to use on github actions for bumping version, changelogs on a monorepo.

## Install this package

```
pnpm add @websublime/workspace-tools
```

## Usage

This package offer a set of functions to retrieve information about the monorepo and the packages that contain. It support all package managers including Bun (WIP).

## API

| Function                                                                                                                                                            | Description                                                                                                      |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `getProjectRootPath(root?: string): string or undefined`                                                                                                            | Get the root path of the project.                                                                                |
| `getDefinedPackageManager(root?: string): string or undefined`                                                                                                      | Get the package manager defined in the project.                                                                  |
| `detectPackageManager(root: string): PackageManager or undefined`                                                                                                   | Detect the package manager defined in the project.                                                               |
| `getPackages(cwd?: string): Array<PackageInfo>`                                                                                                                     | Get the list of packages in the monorepo.                                                                        |
| `getPackageInfo(package_name: string, cwd?: string): PackageInfo`                                                                                                   | Get PackageInfo for a package.                                                                                   |
| `getChangedPackages(sha?: string, cwd: string): Array<PackageInfo>`                                                                                                 | Get the list of packages that have changed since the given sha ('main').                                         |
| `git_add(file: string, cwd?: string): boolean`                                                                                                                      | Stage a file.                                                                                                    |
| `git_add_all(cwd?: string): boolean`                                                                                                                                | Stage all files.                                                                                                 |
| `git_config(name: string, email: string, cwd?: string): boolean`                                                                                                    | Git config user name and email.                                                                                  |
| `gitFetchAll(cwd?: string, fetch_tags?: boolean): boolean`                                                                                                          | Execute a `fetch` command to get the latest changes from the remote repository. You can also retrieve tags       |
| `gitCommit(message: string, body?: string, footer?: string cwd?: string): boolean`                                                                                  | Commit the changes.                                                                                              |
| `gitTag(tag: string, message?: string, cwd?: string): boolean`                                                                                                      | Tag the repository with the given tag.                                                                           |
| `gitPush(cwd?: string, follow_tags?: boolean): boolean`                                                                                                             | Push the changes to the remote repository, including optional tags.                                              |
| `gitCurrentBranch(cwd?: string): string or undefined`                                                                                                               | Get the current branch name.                                                                                     |
| `gitCurrentSha(cwd?: string): string`                                                                                                                               | Get's the current commit id.                                                                                     |
| `gitPreviousSha(cwd?: string): string or undefined`                                                                                                                 | Get's the previous commit id.                                                                                    |
| `gitFirstSha(cwd?: string, branch?: string): string or undefined`                                                                                                   | Get's the first commit id in a branch. Compare is done between branch..Head, and it should be used as main..HEAD |
| `isWorkdirUnclean(cwd?: string): boolean`                                                                                                                           | Check if the workdir is unclean (uncommited changes).                                                            |
| `gitCommitBranchName(sha: string, cwd?: string): string or undefined`                                                                                               | Get the branch name for the commit id.                                                                           |
| `gitAllFilesChangedSinceSha(sha: string, cwd?: string): Array<String>`                                                                                              | Get all files changed sinc branch, commit id etc.                                                                |
| `getDivergedCommit(sha: string, cwd?: string): string or undefined`                                                                                                 | Get the diverged commit from the given sha (main).                                                               |
| `getCommitsSince(cwd?: string, since?: string, relative?: string): Array<Commit>`                                                                                   | Get the commits since the given sha (main) for a particular package.                                             |
| `getAllFilesChangedSinceBranch(package_info: Array<PackageInfo>, branch: string, cwd?: string): Array<String>`                                                      | Get all the files changed for a branch (main).                                                                   |
| `getLastKnownPublishTagInfoForPackage(package_info: PackageInfo, cwd?: string): Array<PublishTagInfo>`                                                              | Get the last known publish tag info for a particular package.                                                    |
| `getLastKnownPublishTagInfoForAllPackages(package_info: Array<PackageInfo>, cwd?: string): Array<PublishTagInfo>`                                                   | Get the last known publish tag info for all packages.                                                            |
| `getRemoteOrLocalTags(cwd?: string, local?: boolean): Array<RemoteTags>`                                                                                            | Get all the tags in the remote or local repository.                                                              |
| `getConventionalForPackage(package_info: PackageInfo, no_fetch_all?: boolean cwd?: string, conventional_options?: ConventionalPackageOptions): ConventionalPackage` | Get the conventional commits for a particular package, changelog output and package info.                        |
| `getBumps(options: BumpOptions): Array<BumpPackage>`                                                                                                                | Output bumps version for packages and it's dependencies                                                          |
| `initChanges(cwd?: string, change_options?: ChangesOptions): ChangesFileData`                                                                                       | Creat changes file or retrieve is data if already exist                                                          |
| `addChange(change: Change, cwd?: string): boolean`                                                                                                                  | Adds a new change to the change file                                                                             |
| `removeChange(branch_name: String, cwd?: string): boolean`                                                                                                          | Removes the change from the changes files.                                                                       |
| `changeExist(branch_name: string, packages_name: Array<string>, cwd?: string): boolean`                                                                             | Check if change already exist.                                                                                   |
| `getChange(branch_name: string, cwd?: string): Array<Change>`                                                                                                       | Get the list of changes for the branch.                                                                          |
| `getChanges(cwd?: string): Changes`                                                                                                                                 | Get all changes.                                                                                                 |
| `getPackageChange(package_name: string, branch: string, cwd?: string): Changes`                                                                                     | Get a change by package name.                                                                                    |
| `changesFileExist(cwd?: string): boolean`                                                                                                                           | Check if `.changes.json` file exist                                                                              |

## Develop requirements

- Install the latest `Rust`
- Install `Node.js@16+` which fully supported `Node-API`
- Run `corepack enable`

## Test in local

- pnpm
- pnpm build
- pnpm test

And you will see:

```bash
$ ava --verbose

  ✔ get defined package manager
  ─

  2 tests passed
✨  Done in 1.12s.
```

## Release package

Ensure you have set your **NPM_TOKEN** in the `GitHub` project setting.

In `Settings -> Secrets`, add **NPM_TOKEN** into it.

When you want to release the package:

```
npm run build
npm version [<newversion> | major | minor | patch | premajor | preminor | prepatch | prerelease [--preid=<prerelease-id>] | from-git]

git push
```

GitHub actions will do the rest job for you.
