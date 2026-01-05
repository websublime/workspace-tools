# `@websublime/workspace-tools`

![https://github.com/websublime/workspace-tools/actions](https://github.com/websublime/workspace-tools/workflows/CI/badge.svg)

> Node.js bindings for workspace management tools - version bumping, changelogs, and monorepo automation.

## Install

```bash
pnpm add @websublime/workspace-tools
# or
npm install @websublime/workspace-tools
# or
yarn add @websublime/workspace-tools
```

## Overview

This package provides native Node.js bindings (via napi-rs) for workspace management operations. It supports all major package managers (npm, yarn, pnpm, bun) and provides a comprehensive API for:

- **Workspace Status** - Get information about your workspace, packages, and pending changesets
- **Changesets** - Create, update, list, and manage changesets for version tracking
- **Version Bumping** - Preview, apply, and snapshot version bumps with dependency propagation
- **Command Execution** - Run commands across workspace packages with filtering and timeout support
- **Configuration** - View and inspect workspace configuration settings
- **Initialization** - Set up changeset-based version management in your workspace

## Quick Start

```typescript
import {
  status,
  changesetAdd,
  bumpPreview,
  bumpApply,
  execute,
} from '@websublime/workspace-tools'

// Get workspace status
const statusResult = await status({ root: '.' })
if (statusResult.success) {
  console.log(`Found ${statusResult.data.packages.length} packages`)
  console.log(`Package manager: ${statusResult.data.packageManager.name}`)
}

// Add a changeset
const changesetResult = await changesetAdd({
  root: '.',
  packages: ['@scope/my-package'],
  bump: 'minor',
  message: 'Add new feature',
})

// Preview version bumps
const previewResult = await bumpPreview({ root: '.', showDiff: true })
if (previewResult.success) {
  for (const pkg of previewResult.data.packages) {
    console.log(`${pkg.name}: ${pkg.currentVersion} → ${pkg.nextVersion}`)
  }
}

// Apply version bumps with git integration
const applyResult = await bumpApply({
  root: '.',
  gitCommit: true,
  gitTag: true,
})

// Execute commands across packages
const execResult = await execute({
  root: '.',
  cmd: 'npm:test',
  parallel: true,
  timeoutSecs: 300,
})
```

## API Reference

All functions return a Promise with an `ApiResponse` structure:

```typescript
interface ApiResponse<T> {
  success: boolean
  data?: T // Present when success is true
  error?: ErrorInfo // Present when success is false
}

interface ErrorInfo {
  code: string // e.g., 'EVALIDATION', 'ENOENT', 'ETIMEOUT'
  message: string
  context?: string
  kind: string
}
```

### Core Functions

| Function | Description |
|----------|-------------|
| `getVersion()` | Returns the version of the native bindings |
| `status(params)` | Get comprehensive workspace status information |
| `init(params)` | Initialize changeset-based version management |
| `configShow(params)` | Show current workspace configuration |

### Changeset Functions

| Function | Description |
|----------|-------------|
| `changesetAdd(params)` | Create a new changeset |
| `changesetUpdate(params)` | Update an existing changeset |
| `changesetList(params)` | List pending changesets with filtering |
| `changesetShow(params)` | Show details of a specific changeset |
| `changesetRemove(params)` | Remove a changeset |
| `changesetHistory(params)` | Query archived changeset history |
| `changesetCheck(params)` | Check if a changeset exists for a branch |

### Bump Functions

| Function | Description |
|----------|-------------|
| `bumpPreview(params)` | Preview version bumps without applying changes |
| `bumpApply(params)` | Apply version bumps with optional Git integration |
| `bumpSnapshot(params)` | Generate snapshot versions for pre-release testing |

### Execute Function

| Function | Description |
|----------|-------------|
| `execute(params)` | Run commands across workspace packages with timeout support |

### Config Functions

| Function | Description |
|----------|-------------|
| `configShow(params)` | Show current workspace configuration settings |

---

## Detailed API

### `status(params: StatusParams): Promise<StatusApiResponse>`

Get comprehensive workspace status information.

```typescript
interface StatusParams {
  root: string // Workspace root directory
  configPath?: string // Optional custom config path
}

interface StatusData {
  repository: RepositoryInfo
  packageManager: PackageManagerInfo
  branch?: BranchInfo
  changesets: ChangesetInfo[]
  packages: PackageInfo[]
}
```

**Example:**

```typescript
const result = await status({ root: '.' })
if (result.success) {
  console.log(`Repository type: ${result.data.repository.kind}`)
  console.log(`Branch: ${result.data.branch?.name}`)
  console.log(`Pending changesets: ${result.data.changesets.length}`)
}
```

---

### `init(params: InitParams): Promise<InitApiResponse>`

Initialize changeset-based version management.

```typescript
interface InitParams {
  root: string
  changesetPath?: string // Default: '.changesets'
  environments?: string[] // Available environments
  defaultEnv?: string // Default environment
  strategy?: 'independent' | 'unified'
  configFormat?: 'toml' | 'json' | 'yaml'
  force?: boolean // Overwrite existing config
}
```

**Example:**

```typescript
const result = await init({
  root: '.',
  strategy: 'independent',
  configFormat: 'toml',
})
```

---

### `configShow(params: ConfigShowParams): Promise<ConfigShowApiResponse>`

Show current workspace configuration from `repo.config.{json,toml,yaml}`.

```typescript
interface ConfigShowParams {
  root: string // Workspace root directory
  configPath?: string // Optional custom config file path
}

interface ConfigShowData {
  configPath: string // Path to loaded config file
  configFormat: string // Format: 'json', 'toml', or 'yaml'
  config: ConfigData // Full configuration object
}

interface ConfigData {
  changeset: ChangesetConfigInfo
  version: VersionConfigInfo
  dependency: DependencyConfigInfo
  upgrade: UpgradeConfigInfo
  changelog: ChangelogConfigInfo
  audit: AuditConfigInfo
  git: GitConfigInfo
  execute: ExecuteConfigInfo
}
```

**Example:**

```typescript
const result = await configShow({ root: '.' })

if (result.success) {
  console.log(`Config loaded from: ${result.data.configPath}`)
  console.log(`Format: ${result.data.configFormat}`)

  const { config } = result.data

  // Version settings
  console.log(`Strategy: ${config.version.strategy}`)
  console.log(`Default bump: ${config.version.defaultBump}`)

  // Changeset settings
  console.log(`Changeset path: ${config.changeset.path}`)
  console.log(`History path: ${config.changeset.historyPath}`)

  // Dependency propagation
  console.log(`Propagate deps: ${config.dependency.propagateDependencies}`)
  console.log(`Max depth: ${config.dependency.maxDepth}`)

  // Execute settings
  console.log(`Timeout: ${config.execute.timeoutSecs}s`)
  console.log(`Max parallel: ${config.execute.maxParallel}`)
}
```

**With custom config path:**

```typescript
const result = await configShow({
  root: '.',
  configPath: 'custom/repo.config.json',
})
```

---

### `changesetAdd(params: ChangesetAddParams): Promise<ChangesetAddApiResponse>`

Create a new changeset.

```typescript
interface ChangesetAddParams {
  root: string
  packages: string[] // Package names to include
  bump: 'major' | 'minor' | 'patch' | 'none'
  message: string // Changeset message/description
  env?: string // Environment (e.g., 'production')
}
```

**Example:**

```typescript
const result = await changesetAdd({
  root: '.',
  packages: ['@scope/core', '@scope/utils'],
  bump: 'minor',
  message: 'Add new utility functions',
})

if (result.success) {
  console.log(`Created changeset: ${result.data.id}`)
}
```

---

### `changesetList(params: ChangesetListParams): Promise<ChangesetListApiResponse>`

List pending changesets with filtering and sorting.

```typescript
interface ChangesetListParams {
  root: string
  configPath?: string
  env?: string // Filter by environment
  bump?: string // Filter by bump type
  sort?: 'date' | 'bump' | 'branch' // Sort order
}
```

**Example:**

```typescript
const result = await changesetList({
  root: '.',
  bump: 'minor',
  sort: 'date',
})

if (result.success) {
  for (const changeset of result.data.changesets) {
    console.log(`${changeset.id}: ${changeset.bump} - ${changeset.packages.join(', ')}`)
  }
}
```

---

### `bumpPreview(params: BumpPreviewParams): Promise<BumpPreviewApiResponse>`

Preview version bumps without applying changes.

```typescript
interface BumpPreviewParams {
  root: string
  configPath?: string
  packages?: string[] // Filter to specific packages
  showDiff?: boolean // Show detailed diffs
}

interface BumpPreviewData {
  strategy: string
  packages: PackageVersionInfo[]
  summary: BumpSummaryInfo
  changesets: string[]
}

interface PackageVersionInfo {
  name: string
  path: string
  currentVersion: string
  nextVersion: string
  bump: string
  dependencyUpdates: DependencyUpdateInfo[]
}
```

**Example:**

```typescript
const result = await bumpPreview({ root: '.', showDiff: true })

if (result.success) {
  console.log(`Strategy: ${result.data.strategy}`)
  console.log(`Summary: ${result.data.summary.totalPackages} packages`)
  console.log(`  Major: ${result.data.summary.majorBumps}`)
  console.log(`  Minor: ${result.data.summary.minorBumps}`)
  console.log(`  Patch: ${result.data.summary.patchBumps}`)

  for (const pkg of result.data.packages) {
    console.log(`${pkg.name}: ${pkg.currentVersion} → ${pkg.nextVersion} (${pkg.bump})`)
  }
}
```

---

### `bumpApply(params: BumpApplyParams): Promise<BumpApplyApiResponse>`

Apply version bumps with optional Git integration and prerelease support.

```typescript
interface BumpApplyParams {
  root: string
  configPath?: string
  packages?: string[]
  gitCommit?: boolean // Create a commit
  gitTag?: boolean // Create version tags
  gitPush?: boolean // Push to remote
  prerelease?: string // Prerelease tag (e.g., 'alpha', 'beta', 'rc')
  noChangelog?: boolean // Skip changelog generation
  noArchive?: boolean // Skip archiving changesets
  alwaysArchive?: boolean // Archive even on failure
  force?: boolean // Skip confirmations
}

interface BumpApplyData {
  strategy: string
  packagesUpdated: number
  changesetsArchived: number
  filesModified: string[]
  tagsCreated: string[]
  commitSha?: string
}
```

**Example:**

```typescript
// Standard release
const result = await bumpApply({
  root: '.',
  gitCommit: true,
  gitTag: true,
  gitPush: true,
})

// Prerelease (beta)
const betaResult = await bumpApply({
  root: '.',
  prerelease: 'beta',
  gitCommit: true,
  gitTag: true,
})

if (result.success) {
  console.log(`Updated ${result.data.packagesUpdated} packages`)
  console.log(`Commit: ${result.data.commitSha}`)
  console.log(`Tags: ${result.data.tagsCreated.join(', ')}`)
}
```

---

### `bumpSnapshot(params: BumpSnapshotParams): Promise<BumpSnapshotApiResponse>`

Generate snapshot versions for pre-release testing and CI.

```typescript
interface BumpSnapshotParams {
  root: string
  configPath?: string
  packages?: string[]
  format?: string // Snapshot format template
}

interface BumpSnapshotData {
  strategy: string
  packages: SnapshotVersionInfo[]
  format: string
}
```

**Format Variables:**

- `{version}` - Current package version
- `{branch}` - Current Git branch (sanitized)
- `{commit}` - Full Git commit hash
- `{short_commit}` - Short Git commit hash (7 chars)
- `{timestamp}` - Unix timestamp

**Example:**

```typescript
const result = await bumpSnapshot({
  root: '.',
  format: '{version}-snapshot.{short_commit}',
})

if (result.success) {
  for (const pkg of result.data.packages) {
    console.log(`${pkg.name}: ${pkg.originalVersion} → ${pkg.snapshotVersion}`)
  }
}
```

---

### `execute(params: ExecuteParams): Promise<ExecuteApiResponse>`

Execute commands across workspace packages with filtering, parallelism, and timeout support.

```typescript
interface ExecuteParams {
  root: string
  cmd: string // Command to execute (e.g., 'npm:test' or 'ls -la')
  filterPackage?: string[] // Run only on specific packages
  affected?: boolean // Run only on affected packages
  since?: string // Since commit/branch/tag (with affected)
  until?: string // Until commit/branch/tag (with affected)
  branch?: string // Compare against branch (with affected)
  parallel?: boolean // Run commands in parallel
  args?: string[] // Additional arguments
  timeoutSecs?: number // Global timeout in seconds
  perPackageTimeoutSecs?: number // Per-package timeout
}

interface ExecuteData {
  command: string
  results: PackageExecutionResult[]
  summary: ExecuteSummary
}

interface PackageExecutionResult {
  package: string
  success: boolean
  exitCode: number
  durationMs: number
  error?: string
}

interface ExecuteSummary {
  total: number
  succeeded: number
  failed: number
  totalDurationMs: number
}
```

**Command Formats:**

- `npm:<script>` - Run an npm script (e.g., `npm:test`, `npm:build`)
- Plain command - Run a system command (e.g., `ls -la`, `node index.js`)

**Example:**

```typescript
// Run tests on all packages in parallel
const result = await execute({
  root: '.',
  cmd: 'npm:test',
  parallel: true,
  timeoutSecs: 300,
})

if (result.success) {
  console.log(`${result.data.summary.succeeded}/${result.data.summary.total} passed`)

  for (const pkg of result.data.results) {
    const icon = pkg.success ? '✓' : '✗'
    console.log(`${icon} ${pkg.package}: ${pkg.durationMs}ms`)
  }
}

// Run on affected packages only
const affectedResult = await execute({
  root: '.',
  cmd: 'npm:build',
  affected: true,
  branch: 'main',
  parallel: true,
})

// Run on specific packages
const filteredResult = await execute({
  root: '.',
  cmd: 'npm:lint',
  filterPackage: ['@scope/core', '@scope/utils'],
})
```

**Note:** `filterPackage` and `affected` are mutually exclusive.

---

## Error Handling

All functions return an `ApiResponse` with consistent error handling:

```typescript
const result = await bumpPreview({ root: '/invalid/path' })

if (!result.success) {
  switch (result.error.code) {
    case 'ENOENT':
      console.error('Path not found:', result.error.message)
      break
    case 'EVALIDATION':
      console.error('Invalid parameters:', result.error.message)
      break
    case 'ETIMEOUT':
      console.error('Operation timed out:', result.error.message)
      break
    case 'EGIT':
      console.error('Git error:', result.error.message)
      break
    default:
      console.error(`Error [${result.error.code}]:`, result.error.message)
  }
}
```

### Error Codes

| Code | Description |
|------|-------------|
| `EVALIDATION` | Invalid parameters |
| `ENOENT` | Path or resource not found |
| `ETIMEOUT` | Operation timed out |
| `EGIT` | Git operation failed |
| `EEXEC` | Execution error |
| `ECONFIG` | Configuration error |
| `EIO` | I/O error |

---

## TypeScript Support

Full TypeScript definitions are included. Import types directly:

```typescript
import type {
  StatusParams,
  StatusData,
  ChangesetAddParams,
  BumpPreviewParams,
  BumpPreviewData,
  ExecuteParams,
  ExecuteData,
  ConfigShowParams,
  ConfigShowData,
  ConfigData,
  ErrorInfo,
} from '@websublime/workspace-tools'
```

---

## Development

### Requirements

- Rust (latest stable)
- Node.js 16+
- pnpm

### Build

```bash
pnpm install
pnpm build-binding:release
pnpm build-types
```

### Test

```bash
pnpm test
```

---

## License

MIT