# Build Scripts Documentation

This document describes the build configuration for the `@websublime/workspace-tools` npm package and its native Rust bindings via napi-rs.

## Overview

The Node.js bindings are built using napi-rs, which compiles the Rust code in `crates/node` into a native Node.js addon. The build process generates:

1. **Native Binary** (`.node` file): Platform-specific compiled binary
2. **TypeScript Definitions** (`binding.d.ts`): Auto-generated type definitions
3. **JavaScript Wrapper** (`binding.js`): Platform detection and module loading

## Package Configuration

### Location

```
packages/workspace-tools/package.json
```

### Build Scripts

| Script | Description |
|--------|-------------|
| `build-binding` | Build native module for development (debug) |
| `build-binding:release` | Build native module for production (optimized) |
| `build-binding:wasi` | Build WebAssembly version |
| `build:node` | Bundle TypeScript/JavaScript and copy binaries to `dist/` |

### Build Command Breakdown

```bash
napi build -o=./src --manifest-path ../../crates/node/Cargo.toml --platform -p sublime_node_tools --js binding.js --dts binding.d.ts --no-const-enum --no-dts-cache
```

| Flag | Value | Description |
|------|-------|-------------|
| `-o` | `./src` | Output directory for generated files |
| `--manifest-path` | `../../crates/node/Cargo.toml` | Path to Rust crate manifest |
| `--platform` | (flag) | Build for current platform |
| `-p` | `sublime_node_tools` | Rust package name to build |
| `--js` | `binding.js` | Name of generated JavaScript loader |
| `--dts` | `binding.d.ts` | Name of generated TypeScript definitions |
| `--no-const-enum` | (flag) | Avoid TypeScript const enums for compatibility |
| `--no-dts-cache` | (flag) | Regenerate TypeScript definitions on each build |

## Generated Files

After running `pnpm build-binding`, the following files are generated in `packages/workspace-tools/src/`:

| File | Description |
|------|-------------|
| `workspace-tools.{platform}.node` | Native binary (e.g., `workspace-tools.darwin-arm64.node`) |
| `binding.d.ts` | TypeScript type definitions |
| `binding.js` | JavaScript module loader with platform detection |

### Platform Binary Names

| Platform | Binary Name |
|----------|-------------|
| macOS x64 | `workspace-tools.darwin-x64.node` |
| macOS ARM64 | `workspace-tools.darwin-arm64.node` |
| Windows x64 | `workspace-tools.win32-x64-msvc.node` |
| Windows ARM64 | `workspace-tools.win32-arm64-msvc.node` |
| Linux x64 (glibc) | `workspace-tools.linux-x64-gnu.node` |
| Linux x64 (musl) | `workspace-tools.linux-x64-musl.node` |
| Linux ARM64 (glibc) | `workspace-tools.linux-arm64-gnu.node` |
| Linux ARM64 (musl) | `workspace-tools.linux-arm64-musl.node` |

## NAPI Configuration

The napi-rs configuration is defined in `packages/workspace-tools/package.json`:

```json
{
  "napi": {
    "binaryName": "workspace-tools",
    "packageName": "@websublime/workspace-tools",
    "targets": [
      "x86_64-apple-darwin",
      "x86_64-pc-windows-msvc",
      "x86_64-unknown-linux-gnu",
      "x86_64-unknown-linux-musl",
      "i686-pc-windows-msvc",
      "armv7-unknown-linux-gnueabihf",
      "aarch64-unknown-linux-gnu",
      "aarch64-apple-darwin",
      "aarch64-unknown-linux-musl",
      "aarch64-pc-windows-msvc"
    ],
    "wasm": {
      "initialMemory": 16384,
      "browser": {
        "fs": true
      }
    }
  }
}
```

## NPM Configuration

The `.npmrc` file in `packages/workspace-tools/` contains:

```
package-manager-strict=false
```

This allows flexibility in package manager usage during development.

## Development Workflow

### Building for Development

```bash
cd packages/workspace-tools
pnpm build-binding
```

This creates a debug build with full debug symbols, suitable for development and testing.

### Building for Release

```bash
cd packages/workspace-tools
pnpm build-binding:release
```

This creates an optimized release build with smaller binary size.

### Building the Distribution Package

After building the native module, run the Node.js build to create the final distribution:

```bash
cd packages/workspace-tools
pnpm build:node
```

This script:
1. Bundles the TypeScript/JavaScript source into ESM (`dist/esm/index.mjs`) and CJS (`dist/cjs/index.cjs`) formats
2. Copies the native `.node` binary from `src/` to `dist/`
3. Copies TypeScript definitions to `dist/types/`
4. Copies WASI shim files if present

### Full Build Sequence

For a complete build from scratch:

```bash
cd packages/workspace-tools
pnpm build-binding:release  # Build native module (release)
pnpm build:node             # Bundle and package for distribution
```

### Typical Binary Sizes

| Build Type | Approximate Size |
|------------|------------------|
| Debug | ~3 MB |
| Release | ~600 KB |

## Distribution Structure

After running `pnpm build:node`, the `dist/` folder contains:

```
dist/
├── cjs/
│   └── index.cjs              # CommonJS entry point
├── esm/
│   └── index.mjs              # ES Module entry point
├── types/
│   ├── binding.d.ts           # Native binding types
│   └── index.d.ts             # Main TypeScript definitions
├── workspace-tools.*.node     # Native binary
├── workspace-tools.wasi.cjs   # WASI fallback
├── wasi-worker.mjs            # WASI worker (Node.js)
└── wasi-worker-browser.mjs    # WASI worker (browser)
```

## Verification Checklist

### After `pnpm build-binding`

Verify the following files exist in `packages/workspace-tools/src/`:

- [ ] Native binary: `workspace-tools.{platform}.node`
- [ ] TypeScript definitions: `binding.d.ts`
- [ ] JavaScript wrapper: `binding.js`
- [ ] Module loads: `node -e "const { getVersion } = require('./src/binding.js'); console.log(getVersion());"`

### After `pnpm build:node`

Verify the following files exist in `packages/workspace-tools/dist/`:

- [ ] CJS bundle: `cjs/index.cjs`
- [ ] ESM bundle: `esm/index.mjs`
- [ ] TypeScript types: `types/binding.d.ts`
- [ ] Native binary: `workspace-tools.{platform}.node`
- [ ] CJS loads: `node -e "const { getVersion } = require('./dist/cjs/index.cjs'); console.log(getVersion());"`
- [ ] ESM loads: `node --experimental-vm-modules -e "import('./dist/esm/index.mjs').then(m => console.log(m.getVersion()));"`

## Troubleshooting

### Build Fails with "cannot find crate"

Ensure you're in the correct directory and the manifest path is correct:

```bash
cd packages/workspace-tools
ls ../../crates/node/Cargo.toml  # Should exist
```

### TypeScript Types Not Generating

Try clearing the cache and rebuilding:

```bash
rm packages/workspace-tools/src/binding.d.ts
pnpm build-binding --no-dts-cache
```

### Module Cannot Be Loaded

Check that the binary was built for your current platform:

```bash
ls packages/workspace-tools/src/*.node
```

The filename should match your platform (e.g., `darwin-arm64` for Apple Silicon Mac).

## CI/CD Integration

The build scripts are validated in the CI pipeline via GitHub Actions.

### Pull Request Workflow

The `build-node-bindings` job in `.github/workflows/pull-request.yml` runs on every PR that modifies Node-related files:

```yaml
build-node-bindings:
  name: Build Node.js Bindings
  strategy:
    matrix:
      settings:
        - host: macos-latest
          target: aarch64-apple-darwin
        - host: macos-13
          target: x86_64-apple-darwin
        - host: ubuntu-latest
          target: x86_64-unknown-linux-gnu
        - host: windows-latest
          target: x86_64-pc-windows-msvc
```

This job:
1. Builds the native module with `pnpm build-binding:release`
2. Verifies generated files exist (`binding.d.ts`, `binding.js`, `.node`)
3. Builds the distribution package with `pnpm build:node`
4. Verifies distribution files exist
5. Tests CJS and ESM module loading

### Platforms Tested

| Platform | Runner | Target |
|----------|--------|--------|
| macOS ARM64 | `macos-latest` | `aarch64-apple-darwin` |
| macOS x64 | `macos-13` | `x86_64-apple-darwin` |
| Linux x64 | `ubuntu-latest` | `x86_64-unknown-linux-gnu` |
| Windows x64 | `windows-latest` | `x86_64-pc-windows-msvc` |

## Related Documentation

- [NAPI-RS Documentation](https://napi.rs/)
- [crates/node/README.md](../README.md) - Crate documentation
- [PLAN_NODE.md](./PLAN_NODE.md) - Implementation plan
- [NAPI_RESEARCH.md](./NAPI_RESEARCH.md) - Research notes