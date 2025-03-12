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
  /** Create a new dependency with a name and version */
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
   * @returns {Promise<void>} A promise that resolves when the version is updated.
   */
  updateVersion(version: string): NapiResult<undefined>
}

/** JavaScript binding for ws_pkg::Package */
export declare class Package {
  /** Create a new package with a name and version */
  constructor(name: string, version: string)
  /** Get the package name */
  name(): string
  /** Get the package version */
  version(): string
  /** Update the package version */
  updateVersion(version: string): NapiResult<undefined>
  /**
   * Get all dependencies of this package
   *
   * This method returns an array of Dependency objects that can be used in JavaScript.
   * Note: Due to technical limitations, this method requires special handling in JavaScript.
   */
  dependencies(): Array<Dependency>
  /** Add a dependency to this package */
  addDependency(dependency: Dependency): void
  /** Update a dependency's version */
  updateDependencyVersion(name: string, version: string): NapiResult<undefined>
  /** Get a dependency by name */
  getDependency(name: string): Dependency | null
  /** Get the number of dependencies */
  dependencyCount(): number
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

export declare function getVersion(): string

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
