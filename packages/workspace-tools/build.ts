import nodePath from 'node:path'

import fsExtra from 'fs-extra'
import { globSync } from 'glob'

import { defineConfig, type OutputOptions, rolldown } from 'rolldown'
import pkgJson from './package.json' with { type: 'json' }

const outputDir = 'dist'

const IS_RELEASING_CI = !!process.env.RELEASING

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
          // Binary build is on the separate step on CI
          if (!process.env.CI && nodeFiles.length === 0) {
            throw new Error('No binary files found')
          }

          const copyTo = nodePath.resolve(outputDir)
          fsExtra.ensureDirSync(copyTo)

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
            } else {
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
              console.log('[build:done]', 'Copying', file, 'to ./dist/shared')
              fsExtra.copyFileSync(file, nodePath.join(copyTo, fileName))
            })
          }

          // Copy binding types to dist
          const distTypesDir = nodePath.resolve(outputDir, 'types')
          fsExtra.ensureDirSync(distTypesDir)
          const types = globSync(['./src/*.d.ts'], {
            absolute: true,
          })
          // biome-ignore lint/complexity/noForEach: <explanation>
          types.forEach((file) => {
            const fileName = nodePath.basename(file)
            console.log('[build:done]', 'Copying', file, 'to ./dist/shared')
            fsExtra.copyFileSync(file, nodePath.join(distTypesDir, fileName))
          })
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
