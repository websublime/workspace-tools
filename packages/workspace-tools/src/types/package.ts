import type { PackageJson } from 'type-fest'
import type { Package } from '../binding'

export interface ScopedPackageInfo {
  full: string
  name: string
  version: string
  path?: string | null
}

export interface PackageInfo {
  package: Package
  packageJsonPath: string
  packagePath: string
  packageRelativePath: string
  packageJson: PackageJson
  changedFiles?: string[]
}

export interface ConventionalPackage {
  packageInfo: PackageInfo,
  conventionalCommits: Record<string, unknown>,
  changelogOutput: string,
}

export interface RecommendBumpPackage {
  from: string
  to: string
  packageInfo: PackageInfo
  deployTo: string[]
  changedFiles: string[]
  conventional: ConventionalPackage
}
