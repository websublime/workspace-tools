import type { Result } from './types/general';

export declare class DependencyBindings {

}

export declare class DependencyRegistryBindings {

}

export declare class PackageBindings {
  constructor(name: string, version: string)
  name(): string
  version(): string
  updateVersion(newVersion: string): void
  updateDependencyVersion(depName: string, newVersion: string): void
  getDependencies(): Array<DependencyBindings>
  addDependency(dependency: DependencyBindings): void
}

export declare function getVersion(): string
