// @ts-check
import { getProjectRootPath, executeCommand, detectPackageManager, GitRepository, Dependency, bumpMajor, bumpMinor, bumpPatch, bumpSnapshot } from '../dist/esm/index.mjs';

/**
 * @typedef {import('../dist/types').executeCommand} executeCommand
 * @typedef {import('../dist/types').detectPackageManager} detectPackageManager
 * @typedef {import('../dist/types').getProjectRootPath} getProjectRootPath
 * @typedef {import('../dist/types').GitRepository} GitRepository
 * @typedef {import('../dist/types').bumpMajor} bumpMajor
 * @typedef {import('../dist/types').bumpMinor} bumpMinor
 * @typedef {import('../dist/types').bumpPatch} bumpPatch
 * @typedef {import('../dist/types').bumpSnapshot} bumpSnapshot
 * @typedef {import('../dist/types').Dependency} Dependency
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

/**
 * @type {bumpSnapshot}
 */
const snapshotVersion = bumpSnapshot('0.0.1', 'ae45th67en09');
console.log(snapshotVersion);

/**
 * @type {bumpMinor}
 */
const minorVersion = bumpMinor('0.0.1');
console.log(minorVersion);

/**
 * @type {bumpPatch}
 */
const patchVersion = bumpPatch('0.0.1');
console.log(patchVersion);

/**
 * @type {bumpMajor}
 */
const majorVersion = bumpMajor('0.0.1');
console.log(majorVersion);

/**
 * @type {Dependency}
 */
const dependency = new Dependency('react', '1.0.0');
console.log(dependency.version, dependency.name, dependency.toString());

dependency.updateVersion("1.5");
console.log(dependency.matches(dependency.version));
