export interface DependencyInfo {
  dependencies: Record<string, { versions: string[], packages: string[] }>[];
}
