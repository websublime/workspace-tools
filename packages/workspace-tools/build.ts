import nodePath from 'node:path'

import fsExtra from 'fs-extra'
import { globSync } from 'glob'

import { defineConfig, type OutputOptions, rolldown } from 'rolldown'
import pkgJson from './package.json' with { type: 'json' }

const outputDir = 'dist'

const IS_RELEASING_CI = !!process.env.RELEASING_CI

const shared = defineConfig({
  input: {
    index: './src/index',
  },
  platform: 'node',
  resolve: {
    extensions: ['.js', '.cjs', '.mjs', '.ts'],
  },
  external: [
    /workspace-tools\..*\.node/,
    /workspace-tools\..*\.wasm/,
    /@websublime\/workspace-tools-.*/,
    /\.\/workspace-tools\.wasi\.cjs/,
    ...Object.keys(pkgJson?.dependencies || {}),
  ],
})

/**
 * Generate the dist/package.json with correct relative paths for npm publishing.
 *
 * When publishing the dist/ folder, paths must be relative to dist/, not the root.
 * For example: "./cjs/index.cjs" instead of "./dist/cjs/index.cjs"
 */
function generateDistPackageJson() {
  // Convert workspace:* references to actual version for npm publish
  const optionalDeps: Record<string, string> = {}
  if (pkgJson.optionalDependencies) {
    for (const [name, version] of Object.entries(pkgJson.optionalDependencies)) {
      // Replace workspace:* with the actual package version
      optionalDeps[name] = (version as string) === 'workspace:*' ? pkgJson.version : (version as string)
    }
  }

  const distPkgJson = {
    name: pkgJson.name,
    version: pkgJson.version,
    description: pkgJson.description,
    // Paths are relative to dist/ folder
    main: './cjs/index.cjs',
    module: './esm/index.mjs',
    types: './types/index.d.ts',
    exports: {
      '.': {
        types: './types/index.d.ts',
        import: './esm/index.mjs',
        require: './cjs/index.cjs',
      },
      './package.json': './package.json',
    },
    repository: pkgJson.repository,
    license: pkgJson.license,
    keywords: pkgJson.keywords,
    engines: pkgJson.engines,
    publishConfig: pkgJson.publishConfig,
    // Include the optional dependencies for platform-specific binaries
    // These are converted from workspace:* to actual versions
    optionalDependencies: optionalDeps,
    // Files to include when publishing (relative to dist/)
    files: [
      'cjs',
      'esm',
      'types',
      'shared',
      'README.md'
    ],
    // napi configuration for runtime binary loading
    napi: pkgJson.napi,
  }

  const distPath = nodePath.resolve(outputDir, 'package.json')
  fsExtra.writeJsonSync(distPath, distPkgJson, { spaces: 2 })
  console.log('[build:done]', 'Generated dist/package.json with correct relative paths')
}

/**
 * Copy README.md to dist folder for npm publishing.
 *
 * The README is included in the published package to provide documentation
 * on npm and when users install the package.
 */
function copyReadme() {
  const readmeSrc = nodePath.resolve('README.md')
  const readmeDest = nodePath.resolve(outputDir, 'README.md')

  if (fsExtra.existsSync(readmeSrc)) {
    fsExtra.copyFileSync(readmeSrc, readmeDest)
    console.log('[build:done]', 'Copied README.md to dist/')
  } else {
    console.warn('[build:warn]', 'README.md not found, skipping copy')
  }
}

/**
 * Generate index.d.ts that re-exports from binding.d.ts
 */
function generateIndexTypes() {
  const distTypesDir = nodePath.resolve(outputDir, 'types')
  fsExtra.ensureDirSync(distTypesDir)

  const indexDtsContent = `// Auto-generated type definitions
export * from './binding';
`
  const indexDtsPath = nodePath.join(distTypesDir, 'index.d.ts')
  fsExtra.writeFileSync(indexDtsPath, indexDtsContent)
  console.log('[build:done]', 'Generated dist/types/index.d.ts')
}

const configs = defineConfig([
  {
    ...shared,
    output: {
      dir: outputDir,
      format: 'esm',
      entryFileNames: 'esm/[name].mjs',
      chunkFileNames: 'shared/[name]-[hash].mjs',
    },
    plugins: [
      {
        name: 'shim',
        buildEnd() {
          // wasm build rely on `.node` binaries. But we don't want to copy `.node` files
          // to the dist folder, so we need to distinguish between `.wasm` and `.node` files.
          const wasmFiles = globSync(['./src/workspace-tools.*.wasm'], {
            absolute: true,
          })

          const isWasmBuild = wasmFiles.length > 0

          const nodeFiles = globSync(['./src/workspace-tools.*.node'], {
            absolute: true,
          })

          const wasiShims = globSync(['./src/*.wasi.js', './src/*.wasi.cjs', './src/*.mjs'], {
            absolute: true,
          })

          // Binary build is on the separate step on CI - allow missing binaries in CI
          if (!process.env.CI && !IS_RELEASING_CI && nodeFiles.length === 0 && wasmFiles.length === 0) {
            throw new Error('No binary files found. Run `pnpm build-binding` first.')
          }

          const copyTo = nodePath.resolve(outputDir)
          fsExtra.ensureDirSync(copyTo)

          // In CI release mode, binaries are in npm/*/ directories, not in dist/
          // Only copy binaries to dist/ for local development
          if (!IS_RELEASING_CI) {
            if (isWasmBuild) {
              // Move the binary file to dist
              // biome-ignore lint/complexity/noForEach: <explanation>
              wasmFiles.forEach((file) => {
                const fileName = nodePath.basename(file)
                console.log('[build:done]', 'Copying', file, `to ${copyTo}`)
                fsExtra.copyFileSync(file, nodePath.join(copyTo, fileName))
                console.log('[build:done]', `Cleaning ${file}`)
                try {
                  // GitHub windows runner emits `operation not permitted` error, most likely because of the file is still in use.
                  // We could safely ignore the error.
                  fsExtra.rmSync(file)
                } catch {}
              })
            } else if (nodeFiles.length > 0) {
              // biome-ignore lint/complexity/noForEach: <explanation>
              nodeFiles.forEach((file) => {
                const fileName = nodePath.basename(file)
                console.log('[build:done]', 'Copying', file, `to ${copyTo}`)
                fsExtra.copyFileSync(file, nodePath.join(copyTo, fileName))
                console.log('[build:done]', `Cleaning ${file}`)
              })
            }

            // biome-ignore lint/complexity/noForEach: <explanation>
            wasiShims.forEach((file) => {
              const fileName = nodePath.basename(file)
              console.log('[build:done]', 'Copying', file, 'to ./dist/')
              fsExtra.copyFileSync(file, nodePath.join(copyTo, fileName))
            })
          }

          // Copy binding types to dist/types/
          const distTypesDir = nodePath.resolve(outputDir, 'types')
          fsExtra.ensureDirSync(distTypesDir)
          const types = globSync(['./src/*.d.ts'], {
            absolute: true,
          })
          // biome-ignore lint/complexity/noForEach: <explanation>
          types.forEach((file) => {
            const fileName = nodePath.basename(file)
            console.log('[build:done]', 'Copying', file, 'to ./dist/types/')
            fsExtra.copyFileSync(file, nodePath.join(distTypesDir, fileName))
          })

          // Generate index.d.ts that re-exports from binding.d.ts
          generateIndexTypes()

          // Generate dist/package.json with correct paths
          generateDistPackageJson()

          // Copy README.md to dist/
          copyReadme()
        },
      },

      {
        name: 'cleanup binding.js',
        transform: {
          filter: {
            code: {
              include: ['require = createRequire(__filename)'],
            },
          },
          handler(code, id) {
            if (id.endsWith('binding.js')) {
              const ret = code.replace('require = createRequire(__filename)', '')
              return ret
            }
          },
        },
      },
    ],
  },
  {
    ...shared,
    plugins: [
      {
        name: 'shim-import-meta',
        transform: {
          filter: {
            code: {
              include: ['import.meta.resolve'],
            },
          },
          handler(code, id) {
            if (id.endsWith('.ts') && code.includes('import.meta.resolve')) {
              return code.replace('import.meta.resolve', 'undefined')
            }
          },
        },
      },
    ],
    output: {
      dir: outputDir,
      format: 'cjs',
      entryFileNames: 'cjs/[name].cjs',
      chunkFileNames: 'shared/[name]-[hash].cjs',
    },
  },
])

;(async () => {
  for (const config of configs) {
    await (await rolldown(config)).write(config.output as OutputOptions)
  }
})()
