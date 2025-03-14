import type { Result } from './types/general';

import type { ScopedPackageInfo } from './types/package';

import type { DependencyInfo } from './types/dependency';

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
  /**
   * Create a new dependency with a name and version
   *
   * @param {string} name - The name of the dependency package.
   * @param {string} version - The version of the dependency.
   *
   * @returns {Dependency} The new dependency.
   */
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
  /**
   * Updates dependency version
   *
   * @param {string} version - The new version of the dependency.
   */
  updateVersion(version: string): void
}

/**
 * DependencyRegistry class
 * A registry to manage shared dependency instances
 *
 * @class DependencyRegistry - The DependencyRegistry class.
 * @example
 *
 * ```typescript
 * const registry = new DependencyRegistry();
 * const dep1 = registry.getOrCreate("foo", "^1.0.0");
 * const dep2 = registry.getOrCreate("bar", "^2.0.0");
 *
 * // Resolve version conflicts
 * const result = registry.resolveVersionConflicts();
 * console.log(result.resolvedVersions);
 * ```
 */
export declare class DependencyRegistry {
  /**
   * Create a new dependency registry
   *
   * @returns {DependencyRegistry} A new empty registry
   */
  constructor()
  /**
   * Get or create a dependency in the registry
   *
   * @param {string} name - The name of the dependency
   * @param {string} version - The version or version requirement
   * @returns {Dependency} The dependency instance
   */
  getOrCreate(name: string, version: string): Dependency
  /**
   * Get a dependency by name
   *
   * @param {string} name - The name of the dependency
   * @returns {Dependency | null} The dependency instance if found, null otherwise
   */
  get(name: string): Dependency | null
  /**
   * Resolve version conflicts between dependencies
   *
   * @returns {ResolutionResult} The result of dependency resolution
   */
  resolveVersionConflicts(): ResolutionResult
  /**
   * Apply a resolution result to update all dependencies
   *
   * @param {ResolutionResult} result - The resolution result to apply
   * @returns {void}
   */
  applyResolutionResult(result: ResolutionResult): void
  /**
   * Find highest version that is compatible with all requirements
   *
   * @param {string} name - The name of the dependency
   * @param {string[]} requirements - List of version requirements
   * @returns {string | null} The highest compatible version, if any
   */
  findHighestCompatibleVersion(name: string, requirements: Array<string>): string | null
}

/** JavaScript binding for ws_pkg::Package */
export declare class Package {
  /** Create a new package with a name and version */
  constructor(name: string, version: string)
  /**
   * Create a new package with dependencies using the dependency registry
   *
   * @param {string} name - The name of the package
   * @param {string} version - The version of the package
   * @param {Array<[string, string]>} dependencies - Array of [name, version] tuples for dependencies
   * @param {DependencyRegistry} registry - The dependency registry to use
   * @returns {Package} The new package
   */
  static withRegistry(name: string, version: string, dependencies: Array<[string, string]> | undefined | null, registry: DependencyRegistry): Package
  /**
   * Get the package name
   *
   * @returns {string} The package name
   */
  get name(): string
  /**
   * Get the package version
   *
   * @returns {string} The package version
   */
  get version(): string
  /**
   * Update the package version
   *
   * @param {string} version - The new version to set
   */
  updateVersion(version: string): void
  /**
   * Get all dependencies of this package
   *
   * This method returns an array of Dependency objects that can be used in JavaScript.
   * Note: Due to technical limitations, this method requires special handling in JavaScript.
   *
   * @returns {Array<Dependency>} An array of Dependency objects
   */
  dependencies(): Array<Dependency>
  /**
   * Add a dependency to this package
   *
   * @param {Dependency} dependency - The dependency to add
   */
  addDependency(dependency: Dependency): void
  /**
   * Update a dependency's version
   *
   * @param {string} name - The name of the dependency to update
   * @param {string} version - The new version of the dependency
   */
  updateDependencyVersion(name: string, version: string): void
  /**
   * Get a dependency by name
   *
   * @param {string} name - The name of the dependency to get
   * @returns {Dependency | null} A dependency or null if not found
   */
  getDependency(name: string): Dependency | null
  /**
   * Get the number of dependencies
   *
   * @returns {number} The number of dependencies
   */
  get dependencyCount(): number
  /**
   * Update dependencies based on a resolution result
   *
   * This method updates all dependencies in the package according to the
   * resolution result.
   *
   * @param {ResolutionResult} resolution - The resolution result to apply
   * @param {Env} env - The NAPI environment
   * @returns {Array<[string, string, string]>} Array of [name, oldVersion, newVersion] tuples for updated deps
   */
  updateDependenciesFromResolution(resolution: ResolutionResult): Array<[string, string, string]>
  /**
   * Check for dependency version conflicts in this package
   *
   * @param {DependencyRegistry} registry - The dependency registry to use
   * @returns {Array<[string, Array<string>]>} Map of dependency names to conflicting version requirements
   */
  findVersionConflicts(): Array<[string, Array<string>]>
  /**
   * Generate combined dependency information for all packages
   *
   * @param {Package[]} packages - Array of packages to analyze
   * @param {DependencyRegistry} registry - The dependency registry to use
   * @returns {DependencyInfo} Object with dependency information
   */
  static generateDependencyInfo(packages: Array<Package>): DependencyInfo
}

/**
 * JavaScript binding for ws_pkg::PackageInfo
 * Represents a package with its metadata
 *
 * @class PackageInfo - The PackageInfo class.
 * @example
 *
 * ```typescript
 * const pkgInfo = new PackageInfo(package, "/path/to/package.json", "/path/to/package", "./relative/path", packageJson);
 * console.log(pkgInfo.packageJsonPath); // /path/to/package.json
 * ```
 */
export declare class PackageInfo {
  /**
   * Create a new package info object
   *
   * @param {Package} package - The package object
   * @param {string} packageJsonPath - Path to the package.json file
   * @param {string} packagePath - Path to the package directory
   * @param {string} packageRelativePath - Relative path to the package directory
   * @param {Object} packageJson - The package.json content
   * @returns {PackageInfo} The new package info
   */
  constructor(package: Package, packageJsonPath: string, packagePath: string, packageRelativePath: string, packageJson: object)
  /**
   * Get the package json path
   *
   * @returns {string} The path to package.json
   */
  get packageJsonPath(): string
  /**
   * Get the package path
   *
   * @returns {string} The path to the package
   */
  get packagePath(): string
  /**
   * Get the relative package path
   *
   * @returns {string} The relative path to the package
   */
  get packageRelativePath(): string
  /**
   * Get the package
   *
   * @returns {Package} The package
   */
  get package(): Package
  /**
   * Update the package version
   *
   * @param {string} newVersion - The new version to set
   * @returns {void}
   */
  updateVersion(newVersion: string): void
  /**
   * Update a dependency version
   *
   * @param {string} depName - The name of the dependency to update
   * @param {string} newVersion - The new version to set
   * @returns {void}
   */
  updateDependencyVersion(depName: string, newVersion: string): void
  /**
   * Apply dependency resolution across all packages
   *
   * @param {ResolutionResult} resolution - The resolution result to apply
   * @returns {void}
   */
  applyDependencyResolution(resolution: ResolutionResult): void
  /**
   * Write the package.json file to disk
   *
   * @returns {void}
   */
  writePackageJson(): void
  /**
   * Get the package.json content
   *
   * @returns {Object} The package.json content
   */
  get packageJson(): NapiResult<object>
}

/** JavaScript binding for version utilities */
export declare class VersionUtils {
  /** Bump a version to the next major version */
  static bumpMajor(version: string): string | null
  /** Bump a version to the next minor version */
  static bumpMinor(version: string): string | null
  /** Bump a version to the next patch version */
  static bumpPatch(version: string): string | null
  /** Bump a version to a snapshot version with the given SHA */
  static bumpSnapshot(version: string, sha: string): string | null
  /** Compare two version strings and return their relationship */
  static compareVersions(v1: string, v2: string): VersionComparisonResult
  /** Check if moving from v1 to v2 is a breaking change */
  static isBreakingChange(v1: string, v2: string): boolean
}

/** JavaScript binding for dependency update information */
export interface DependencyUpdateInfo {
  /** Package containing the dependency */
  packageName: string
  /** Dependency name */
  dependencyName: string
  /** Current version */
  currentVersion: string
  /** New version to update to */
  newVersion: string
}

export declare function getVersion(): string

/**
 * Parse a scoped package name with optional version and path
 *
 * Handles formats like:
 * - @scope/name
 * - @scope/name@version
 * - @scope/name@version@path
 * - @scope/name:version
 *
 * @param {string} pkg_name - The scoped package name to parse
 * @returns {Object | null} An object with parsed components or null if not a valid scoped package
 */
export declare function parseScopedPackage(pkgName: string): ScopedPackageInfo | null

/** JavaScript binding for DependencyResolutionError */
export declare enum ResolutionErrorType {
  /** Error parsing a version */
  VersionParseError = 0,
  /** Incompatible versions of the same dependency */
  IncompatibleVersions = 1,
  /** No valid version found */
  NoValidVersion = 2
}

/** JavaScript binding for the result of dependency resolution */
export interface ResolutionResult {
  /** Resolved versions for each package */
  resolvedVersions: object
  /** Dependencies that need updates */
  updatesRequired: Array<DependencyUpdateInfo>
}

/** JavaScript binding for ws_pkg::types::version::Version enum */
export declare enum Version {
  /** Major version bump */
  Major = 0,
  /** Minor version bump */
  Minor = 1,
  /** Patch version bump */
  Patch = 2,
  /** Snapshot version */
  Snapshot = 3
}

/** JavaScript binding for version relationship comparisons */
export declare enum VersionComparisonResult {
  /** Second version is a major upgrade (1.0.0 -> 2.0.0) */
  MajorUpgrade = 0,
  /** Second version is a minor upgrade (1.0.0 -> 1.1.0) */
  MinorUpgrade = 1,
  /** Second version is a patch upgrade (1.0.0 -> 1.0.1) */
  PatchUpgrade = 2,
  /** Moved from prerelease to stable (1.0.0-alpha -> 1.0.0) */
  PrereleaseToStable = 3,
  /** Newer prerelease version (1.0.0-alpha -> 1.0.0-beta) */
  NewerPrerelease = 4,
  /** Versions are identical (1.0.0 == 1.0.0) */
  Identical = 5,
  /** Second version is a major downgrade (2.0.0 -> 1.0.0) */
  MajorDowngrade = 6,
  /** Second version is a minor downgrade (1.1.0 -> 1.0.0) */
  MinorDowngrade = 7,
  /** Second version is a patch downgrade (1.0.1 -> 1.0.0) */
  PatchDowngrade = 8,
  /** Moved from stable to prerelease (1.0.0 -> 1.0.0-alpha) */
  StableToPrerelease = 9,
  /** Older prerelease version (1.0.0-beta -> 1.0.0-alpha) */
  OlderPrerelease = 10,
  /** Version comparison couldn't be determined (invalid versions) */
  Indeterminate = 11
}
