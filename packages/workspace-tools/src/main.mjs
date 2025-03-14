import {
  Dependency,
  Package,
  DependencyRegistry,
  ResolutionErrorType,
  Version,
  VersionComparisonResult,
  VersionUtils,
  PackageInfo,
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
          showHidden: true,
          ...log.options,
        }),
      ),
    )
  }
  log.options = {}
  return log
})()

const root = process.cwd()

const pkgBar = new Package('@scope/bar', '1.0.0')
const pkgCharlie = new Package('@scope/charlie', '1.5.0')

const depFoo = new Dependency('@scope/foo', '^1.0.0')
const depBaz = new Dependency('@scope/baz', '1.2.0')

pkgBar.addDependency(depFoo)
pkgBar.addDependency(depBaz)

pkgCharlie.addDependency(depFoo)
pkgCharlie.addDependency(depBaz)

const [fooDep] = pkgBar.dependencies()

log('Dependencies', fooDep)

fooDep.updateVersion('2.0.0')

pkgBar.updateVersion('1.1.0')
pkgBar.updateDependencyVersion('@scope/baz', '1.3.0')

const dependencyInfo = Package.generateDependencyInfo([pkgBar, pkgCharlie])

log('Dependency Info', dependencyInfo)

log('Updated', pkgBar, fooDep, depBaz)

const registry = new DependencyRegistry()

const dep1 = registry.getOrCreate('foo', '^1.0.0')
const dep2 = registry.getOrCreate('bar', '^2.0.0')
const dep3 = registry.getOrCreate('baz', '^3.0.0')

const compatibleFooVersion = registry.findHighestCompatibleVersion('foo', ['>=1.0.0', '<=1.5.0'])
const pkgTom = Package.withRegistry(
  '@scope/tom',
  '0.0.1',
  [
    ['foo', '^1.0.0'],
    ['bar', '^2.0.0'],
    ['baz', '^3.0.0'],
  ],
  registry,
)

log('Compatible Foo Version:', compatibleFooVersion)
log('Registry', registry)
log('Registry Foo', dep1)
log('Registry Bar', dep2)
log('Registry Baz', dep3)
log('Package Tom', pkgTom, pkgTom.dependencies())

const pkgInfo = new PackageInfo(pkgTom, '/path/to/package.json', '/path/to/package', './relative/path', {
  name: '@scope/tom',
  version: '0.0.1',
})

log('Package Info', pkgInfo)
