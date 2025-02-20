import {
  initChanges,
  addChange,
  removeChange,
  getChanges,
  getChangesByBranch,
  getChangesByPackage,
  getChangesMetaByPackage,
  getConfig,
  detectPackageManager,
  getProjectRootPath,
  executeCmd,
  bumpMajor,
  bumpMinor,
  bumpPatch,
  bumpSnapshot,
} from './binding.js'
import util from 'node:util'

const log = (() => {
  const log = (...values) => {
    console.log(
      ...values.map((value) =>
        util.inspect(value, {
          colors: true,
          depth: null,
          getters: true,
          showHidden: false,
          ...log.options,
        }),
      ),
    )
  }
  log.options = {}
  return log
})()

const root = process.cwd()

log('Init changes', initChanges(root))

log('Add Change', addChange({ package: '@scope/foo', releaseAs: 'patch' }, ['int'], root))

log('Get changes', getChanges(root))

log('Get changes by branch', getChangesByBranch('feature/next', root))

log('Get an inexisting change', getChangesByBranch('unknown', root))

log('Get changes by package', getChangesByPackage('@scope/foo', 'feature/next', root))

log('Get changes metadata by package', getChangesMetaByPackage('@scope/foo', root))

log('Detect package manager', detectPackageManager(root))

log('Get workspace config', getConfig(root))

log('Get project root path', getProjectRootPath(root))

log('Get project root path without parameters', getProjectRootPath())

log('Execute git command', executeCmd('git', '.', ['--version']))

log('Bump version to major', bumpMajor('1.0.0'))

log('Bump version to minor', bumpMinor('1.0.0'))

log('Bump version to patch', bumpPatch('1.0.0'))

log('Bump version to snapshot', bumpSnapshot('1.0.0', 'a23d456h'))

log('Delete the change from changes file', removeChange('feature/next', root))
