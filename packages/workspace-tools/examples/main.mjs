// @ts-check
import { getProjectRootPath, executeCommand, detectPackageManager } from '../dist/esm/index.mjs';

/**
 * @typedef {import('../dist/types').executeCommand} executeCommand
 * @typedef {import('../dist/types').getProjectRootPath} getProjectRootPath
 */

const rootPath = getProjectRootPath();

console.log(rootPath);

console.log(executeCommand("git", ".", ["status"]));
console.log(detectPackageManager(rootPath));
