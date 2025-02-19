export type BumpType = 'major' | 'minor' | 'patch' | 'snapshot'

export interface Change {
  package: string
  releaseAs: BumpType
}

export interface Changes {
  [key: string]: ChangeMeta
}

export interface ChangeMeta {
  deploy: string[]
  pkgs: Change[]
}
