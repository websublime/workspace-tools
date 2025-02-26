import type { Result } from './types/general';

/**
 * Dependency class.
 * Represents a package dependency.
 *
 * @class Dependency - The Dependency class.
 * @property {string} name - The name of the dependency.
 * @property {string} version - The version of the dependency.
 *
 * @example
 *
 * ```typescript
 * const dep = new Dependency("foo", "1.0.0");
 * console.log(dep.name); // foo
 * console.log(dep.version); // 1.0.0
 * ```
 */
export declare class Dependency {
  constructor(name: string, version: string)
  /**
   * Gets the name of the dependency.
   *
   * @returns {string} The name of the dependency.
   */
  get name(): string
  /**
   * Gets the version of the dependency.
   *
   * @returns {string} The version of the dependency.
   */
  get version(): string
}

/**
 * Package class.
 * Represents a package.
 *
 * @class Package - The Package class.
 * @property {string} name - The name of the package.
 * @property {string} version - The version of the package.
 * @property {Optional<Array<Dependency>>} dependencies - The dependencies of the package.
 *
 * @example
 *
 * ```typescript
 * const pkg = new Package("foo", "1.0.0", [new Dependency("bar", "2.0.0")]);
 * console.log(pkg.name); // foo
 * console.log(pkg.version); // 1.0.0
 * console.log(pkg.dependencies); // [Dependency { name: 'bar', version: '2.0.0' }]
 * ```
 */
export declare class Package {
  constructor(name: string, version: string, deps?: Array<Dependency>)
  /**
   * Update the package version.
   *
   * @param {string} version - The new version.
   * @returns {void}
   */
  updateVersion(version: string): void
  /**
   * Update the dependency version.
   *
   * @param {string} name - The dependency name.
   * @param {string} version - The new version.
   * @returns {void}
   */
  updateDependencyVersion(name: string, version: string): void
  /**
   * Get package name.
   *
   * @returns {string} The package name.
   */
  get name(): string
  /**
   * Get package version.
   *
   * @returns {string} The package version.
   */
  get version(): string
  /**
   * Get package dependencies.
   *
   * @returns {Array<Dependency>} The package dependencies.
   */
  get dependencies(): Array<Dependency>
}

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

export type PackageManager =  'Npm'|
'Yarn'|
'Pnpm'|
'Bun';
