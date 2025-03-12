import { Dependency, Package } from './binding.js'
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

const pkg = new Package('my-pkg', '1.0.0')
const dep = new Dependency('dep1', '^1.0.0')
pkg.addDependency(dep)

const deps = pkg.dependencies()

log('Dependencies', deps)

deps[0].updateVersion('2.0.0')

log('Updated', pkg, deps)
