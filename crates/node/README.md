# sublime_node_tools

Node.js bindings for Sublime Workspace CLI Tools via napi-rs.

## What

This crate provides Node.js bindings via napi-rs for the workspace-tools CLI,
exposing 23 execute functions as native JavaScript/TypeScript functions. It enables
Node.js and TypeScript applications to use workspace-tools functionality
programmatically without spawning CLI processes.

## Features

- **Full type safety**: TypeScript definitions are auto-generated
- **Native performance**: Direct function calls instead of process spawning
- **Structured responses**: `JsonResponse<T>` with success/error handling
- **Node.js-style errors**: Familiar error codes like ENOENT, EVALIDATION

## Installation

This crate is used internally by the `@websublime/workspace-tools` npm package.
You don't need to install it directly.

```bash
npm install @websublime/workspace-tools
# or
pnpm add @websublime/workspace-tools
```

## Usage

```typescript
import { status, changesetAdd, bumpPreview } from '@websublime/workspace-tools';

// Get workspace status
const result = await status({ root: '.' });
if (result.success) {
  console.log(result.data.packages);
} else {
  console.error(`Error [${result.error.code}]: ${result.error.message}`);
}

// Add a changeset
const changesetResult = await changesetAdd({
  root: '.',
  packages: ['@scope/pkg1'],
  bumpType: 'minor',
  message: 'Add new feature'
});

// Preview version bumps
const previewResult = await bumpPreview({ root: '.', showDiff: true });
```

## API Overview

### Status & Init

- `status(params)`: Get workspace status
- `init(params)`: Initialize workspace configuration

### Changeset Commands

- `changesetAdd(params)`: Create a new changeset
- `changesetUpdate(params)`: Update an existing changeset
- `changesetList(params)`: List pending changesets
- `changesetShow(params)`: Show changeset details
- `changesetRemove(params)`: Remove a changeset
- `changesetHistory(params)`: Query changeset history
- `changesetCheck(params)`: Check if branch has a changeset

### Bump Commands

- `bumpPreview(params)`: Preview version bumps
- `bumpApply(params)`: Apply version bumps
- `bumpSnapshot(params)`: Generate snapshot versions

### Config Commands

- `configShow(params)`: Show configuration
- `configValidate(params)`: Validate configuration

### Upgrade Commands

- `upgradeCheck(params)`: Check for available upgrades
- `upgradeApply(params)`: Apply upgrades
- `backupCreate(params)`: Create backup
- `backupRestore(params)`: Restore from backup
- `backupList(params)`: List backups

### Other Commands

- `audit(params)`: Run workspace audit
- `changes(params)`: Analyze changes
- `clone(params)`: Clone repository
- `execute(params)`: Execute command across packages

## Response Format

All functions return a `JsonResponse<T>` with the following structure:

```typescript
interface JsonResponse<T> {
  success: boolean;
  data?: T;      // Present when success is true
  error?: string;  // Present when success is false
}
```

## Error Codes

Error codes follow Node.js conventions:

| Code | Description |
|------|-------------|
| `ECONFIG` | Configuration error |
| `EVALIDATION` | Validation error |
| `EEXEC` | Execution error |
| `EGIT` | Git error |
| `EPKG` | Package error |
| `ENOENT` | File/path not found |
| `EIO` | I/O error |
| `ENETWORK` | Network error |
| `EUSER` | User error |
| `ETIMEOUT` | Timeout error |

## Development

### Building

```bash
# Build the native module
pnpm build-binding

# Build for release
pnpm build-binding:release
```

### Testing

```bash
# Run Rust tests
cargo test -p sublime_node_tools

# Run Node.js integration tests
pnpm test
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         JavaScript/TypeScript                        │
│                                                                      │
│   const result = await status({ root: "/path/to/project" });        │
└────────────────────────────────────┬────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      NAPI Layer (crates/node/)                       │
│                                                                      │
│   1. Parameter validation                                           │
│   2. Conversion JS → Rust structs                                   │
│   3. Call execute_* functions from CLI                              │
│   4. Return JsonResponse<T>                                         │
└────────────────────────────────────┬────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      CLI Layer (crates/cli/)                         │
│                                                                      │
│   execute_status(), execute_init(), execute_changeset_add(), ...    │
└─────────────────────────────────────────────────────────────────────┘
```

## License

MIT