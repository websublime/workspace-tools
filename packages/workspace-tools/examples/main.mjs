// @ts-check
import { getProjectRootPath, executeCommand, detectPackageManager, GitRepository } from '../dist/esm/index.mjs';

/**
 * @typedef {import('../dist/types').executeCommand} executeCommand
 * @typedef {import('../dist/types').getProjectRootPath} getProjectRootPath
 * @typedef {import('../dist/types').GitRepository} GitRepository
 */

const rootPath = getProjectRootPath();

console.log(rootPath);

console.log(executeCommand("git", ".", ["status"]));
console.log(detectPackageManager(rootPath));

const gitRepo = GitRepository.open(rootPath);
console.log(gitRepo.path);
