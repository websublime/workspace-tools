import type { PackageJson } from 'type-fest'
import type { Package } from '../binding'

export interface PackageScopeMetadata {
  full: string
  name: string
  version: string
  path?: string
}

export interface PackageInfo {
  package: Package
  packageJsonPath: string
  packagePath: string
  packageRelativePath: string
  packageJson: PackageJson
}
