// @ts-check
import { getProjectRootPath, executeCommand, detectPackageManager, GitRepository } from '../dist/esm/index.mjs';

/**
 * @typedef {import('../dist/types').executeCommand} executeCommand
 * @typedef {import('../dist/types').detectPackageManager} detectPackageManager
 * @typedef {import('../dist/types').getProjectRootPath} getProjectRootPath
 * @typedef {import('../dist/types').GitRepository} GitRepository
 */

/** @type {getProjectRootPath} */
const rootPath = getProjectRootPath();

console.log(rootPath);

console.log(executeCommand("git", ".", ["status"]));
console.log(detectPackageManager(rootPath));

/**
 * @type {GitRepository}
 */
const gitRepo = GitRepository.open(rootPath)
console.log(gitRepo.currentBranch);
