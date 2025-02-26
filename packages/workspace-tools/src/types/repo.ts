export interface RepositoryCommit {
  hash: string
  authorName: string
  authorEmail: string
  authorDate: string
  message: string
}

export interface RepositoryRemoteTags {
  hash: string
  tag: string
}

export interface RepositoryPublishTagInfo {
  tag: string
  hash: string
  package: string
}
