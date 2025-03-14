import {
  Dependency,
  Package,
  DependencyRegistry,
  ResolutionErrorType,
  Version,
  VersionComparisonResult,
  VersionUtils,
  PackageInfo,
  ChangeType,
  PackageDiff,
  getVersion,
  parseScopedPackage,
} from './binding.js'
import util from 'node:util'
import chalk from 'chalk'
import boxen from 'boxen'
import Table from 'cli-table3'

// ===== Helper functions for enum mapping =====
function versionComparisonToString(result) {
  const mapping = {
    [VersionComparisonResult.MajorUpgrade]: 'Major Upgrade',
    [VersionComparisonResult.MinorUpgrade]: 'Minor Upgrade',
    [VersionComparisonResult.PatchUpgrade]: 'Patch Upgrade',
    [VersionComparisonResult.PrereleaseToStable]: 'Prerelease to Stable',
    [VersionComparisonResult.NewerPrerelease]: 'Newer Prerelease',
    [VersionComparisonResult.Identical]: 'Identical',
    [VersionComparisonResult.MajorDowngrade]: 'Major Downgrade',
    [VersionComparisonResult.MinorDowngrade]: 'Minor Downgrade',
    [VersionComparisonResult.PatchDowngrade]: 'Patch Downgrade',
    [VersionComparisonResult.StableToPrerelease]: 'Stable to Prerelease',
    [VersionComparisonResult.OlderPrerelease]: 'Older Prerelease',
    [VersionComparisonResult.Indeterminate]: 'Indeterminate',
  }
  return mapping[result] || `Unknown (${result})`
}

function changeTypeToString(changeType) {
  const mapping = {
    [ChangeType.Added]: 'Added',
    [ChangeType.Removed]: 'Removed',
    [ChangeType.Updated]: 'Updated',
    [ChangeType.Unchanged]: 'Unchanged',
  }
  return mapping[changeType] || `Unknown (${changeType})`
}

function versionTypeToString(versionType) {
  const mapping = {
    [Version.Major]: 'Major',
    [Version.Minor]: 'Minor',
    [Version.Patch]: 'Patch',
    [Version.Snapshot]: 'Snapshot',
  }
  return mapping[versionType] || `Unknown (${versionType})`
}

function resolutionErrorTypeToString(errorType) {
  const mapping = {
    [ResolutionErrorType.VersionParseError]: 'Version Parse Error',
    [ResolutionErrorType.IncompatibleVersions]: 'Incompatible Versions',
    [ResolutionErrorType.NoValidVersion]: 'No Valid Version',
  }
  return mapping[errorType] || `Unknown (${errorType})`
}

// ===== Pretty printing helpers =====
function printHeader(title) {
  console.log(
    '\n' +
      boxen(chalk.bold.yellow(`🚀 ${title} 🚀`), {
        padding: 1,
        margin: 1,
        borderStyle: 'double',
        borderColor: 'yellow',
      }),
  )
}

function printSubHeader(title, emoji = '📌') {
  console.log('\n' + chalk.cyan.bold(`${emoji} ${title}`))
  console.log(chalk.cyan('━'.repeat(title.length + 4)))
}

function printProperty(name, value, emoji = '🔹') {
  console.log(`${emoji} ${chalk.blue(name)}: ${chalk.green(value)}`)
}

function printSuccess(message, emoji = '✅') {
  console.log(`${emoji} ${chalk.green(message)}`)
}

function printWarning(message, emoji = '⚠️') {
  console.log(`${emoji} ${chalk.yellow(message)}`)
}

function printError(message, emoji = '❌') {
  console.log(`${emoji} ${chalk.red(message)}`)
}

function printCode(code) {
  console.log(chalk.gray(code))
}

function printTable(headers, data) {
  const table = new Table({
    head: headers.map((h) => chalk.bold.white(h)),
    chars: {
      top: '━',
      'top-mid': '┳',
      'top-left': '┏',
      'top-right': '┓',
      bottom: '━',
      'bottom-mid': '┻',
      'bottom-left': '┗',
      'bottom-right': '┛',
      left: '┃',
      'left-mid': '┣',
      mid: '━',
      'mid-mid': '╋',
      right: '┃',
      'right-mid': '┫',
      middle: '┃',
    },
  })

  data.forEach((row) => table.push(row))
  console.log(table.toString())
}

// Print main title and version
console.log(
  boxen(chalk.bold.magenta('✨✨✨ WORKSPACE TOOLS API SHOWCASE ✨✨✨\n') + chalk.cyan(`Version: ${getVersion()}`), {
    padding: 1,
    margin: { top: 1, bottom: 1 },
    borderStyle: 'double',
    borderColor: 'magenta',
    align: 'center',
  }),
)

// ===== Example 1: Version Enum =====
printHeader('Version Enum')
;(() => {
  printSubHeader('Version Enum Values', '🏷️')
  const versionTable = new Table({
    head: [chalk.bold.white('Type'), chalk.bold.white('Value'), chalk.bold.white('String Representation')],
    colWidths: [15, 10, 25],
  })

  versionTable.push(
    [chalk.blue('Major'), Version.Major, chalk.green(versionTypeToString(Version.Major))],
    [chalk.blue('Minor'), Version.Minor, chalk.green(versionTypeToString(Version.Minor))],
    [chalk.blue('Patch'), Version.Patch, chalk.green(versionTypeToString(Version.Patch))],
    [chalk.blue('Snapshot'), Version.Snapshot, chalk.green(versionTypeToString(Version.Snapshot))],
  )

  console.log(versionTable.toString())

  printSubHeader('Version Bumping Examples', '🔄')

  function simulateVersionBump(versionType, currentVersion) {
    let newVersion = currentVersion

    switch (versionType) {
      case Version.Major:
        newVersion = VersionUtils.bumpMajor(currentVersion)
        break
      case Version.Minor:
        newVersion = VersionUtils.bumpMinor(currentVersion)
        break
      case Version.Patch:
        newVersion = VersionUtils.bumpPatch(currentVersion)
        break
      case Version.Snapshot:
        newVersion = VersionUtils.bumpSnapshot(currentVersion, 'abc123')
        break
    }

    return {
      type: versionTypeToString(versionType),
      from: currentVersion,
      to: newVersion,
    }
  }

  const bumpTable = new Table({
    head: [chalk.bold.white('Bump Type'), chalk.bold.white('From'), chalk.bold.white('To')],
    colWidths: [15, 15, 20],
  })

  ;['1.2.3'].forEach((version) => {
    bumpTable.push(
      [chalk.blue('Major'), version, chalk.green(VersionUtils.bumpMajor(version))],
      [chalk.blue('Minor'), version, chalk.green(VersionUtils.bumpMinor(version))],
      [chalk.blue('Patch'), version, chalk.green(VersionUtils.bumpPatch(version))],
      [chalk.blue('Snapshot'), version, chalk.green(VersionUtils.bumpSnapshot(version, 'abc123'))],
    )
  })

  console.log(bumpTable.toString())
})()

// ===== Example 2: ResolutionErrorType =====
printHeader('ResolutionErrorType')
;(() => {
  printSubHeader('Resolution Error Types', '🐛')

  const errorTable = new Table({
    head: [chalk.bold.white('Error Type'), chalk.bold.white('Value'), chalk.bold.white('Description')],
    colWidths: [25, 10, 50],
  })

  errorTable.push(
    [chalk.red('VersionParseError'), ResolutionErrorType.VersionParseError, 'Failed to parse version string'],
    [
      chalk.red('IncompatibleVersions'),
      ResolutionErrorType.IncompatibleVersions,
      'Found multiple incompatible version requirements',
    ],
    [
      chalk.red('NoValidVersion'),
      ResolutionErrorType.NoValidVersion,
      'No version exists that satisfies all requirements',
    ],
  )

  console.log(errorTable.toString())

  printSubHeader('Error Handling Example', '🔍')

  function simulateResolutionError(errorType, packageName) {
    let errorMessage = `Error resolving dependencies for ${packageName}: `

    switch (errorType) {
      case ResolutionErrorType.VersionParseError:
        errorMessage += `Invalid version format in package.json`
        break
      case ResolutionErrorType.IncompatibleVersions:
        errorMessage += `Found incompatible version requirements`
        break
      case ResolutionErrorType.NoValidVersion:
        errorMessage += `No version exists that satisfies all requirements`
        break
    }

    return {
      success: false,
      errorType,
      errorMessage,
    }
  }

  const errorTypes = [
    ResolutionErrorType.VersionParseError,
    ResolutionErrorType.IncompatibleVersions,
    ResolutionErrorType.NoValidVersion,
  ]

  for (const errorType of errorTypes) {
    const result = simulateResolutionError(errorType, 'test-package')
    printError(`${result.errorMessage} (Type: ${resolutionErrorTypeToString(result.errorType)})`)
  }
})()

// ===== Example 3: Version Utils =====
printHeader('Version Utilities')
;(() => {
  printSubHeader('Version Comparison', '⚖️')

  const compareExamples = [
    ['1.0.0', '2.0.0'], // Major upgrade
    ['1.0.0', '1.1.0'], // Minor upgrade
    ['1.0.0', '1.0.1'], // Patch upgrade
    ['1.0.0-alpha', '1.0.0'], // Prerelease to stable
    ['1.0.0', '1.0.0'], // Identical
    ['2.0.0', '1.0.0'], // Major downgrade
  ]

  const comparisonTable = new Table({
    head: [chalk.bold.white('Version 1'), chalk.bold.white('Version 2'), chalk.bold.white('Relationship')],
    colWidths: [15, 15, 30],
  })

  for (const [v1, v2] of compareExamples) {
    const result = VersionUtils.compareVersions(v1, v2)
    const relationship = versionComparisonToString(result)

    let relationshipColored
    if (relationship.includes('Upgrade')) {
      relationshipColored = chalk.green(relationship)
    } else if (relationship.includes('Downgrade')) {
      relationshipColored = chalk.red(relationship)
    } else if (relationship === 'Identical') {
      relationshipColored = chalk.blue(relationship)
    } else {
      relationshipColored = chalk.yellow(relationship)
    }

    comparisonTable.push([chalk.blue(v1), chalk.blue(v2), relationshipColored])
  }

  console.log(comparisonTable.toString())

  printSubHeader('Breaking Change Detection', '💥')

  const breakingChangeTable = new Table({
    head: [chalk.bold.white('Version 1'), chalk.bold.white('Version 2'), chalk.bold.white('Breaking?')],
    colWidths: [15, 15, 15],
  })

  breakingChangeTable.push(
    [
      chalk.blue('1.0.0'),
      chalk.blue('2.0.0'),
      VersionUtils.isBreakingChange('1.0.0', '2.0.0') ? chalk.red('Yes') : chalk.green('No'),
    ],
    [
      chalk.blue('1.0.0'),
      chalk.blue('1.1.0'),
      VersionUtils.isBreakingChange('1.0.0', '1.1.0') ? chalk.red('Yes') : chalk.green('No'),
    ],
    [
      chalk.blue('1.0.0'),
      chalk.blue('1.0.1'),
      VersionUtils.isBreakingChange('1.0.0', '1.0.1') ? chalk.red('Yes') : chalk.green('No'),
    ],
  )

  console.log(breakingChangeTable.toString())
})()

// ===== Example 4: Dependencies =====
printHeader('Dependencies')

const demoDependencies = () => {
  printSubHeader('Creating Dependencies', '📦')

  const dep1 = new Dependency('react', '^17.0.2')
  const dep2 = new Dependency('lodash', '~4.17.21')

  const depTable = new Table({
    head: [chalk.bold.white('Name'), chalk.bold.white('Version'), chalk.bold.white('Type')],
    colWidths: [15, 15, 15],
  })

  depTable.push(
    [chalk.blue(dep1.name), chalk.green(dep1.version), chalk.yellow('Caret')],
    [chalk.blue(dep2.name), chalk.green(dep2.version), chalk.yellow('Tilde')],
  )

  console.log(depTable.toString())

  printSubHeader('Updating Dependency Version', '🔄')

  printProperty('Before', `${dep1.name} @ ${dep1.version}`)
  dep1.updateVersion('^18.0.0')
  printSuccess(`Updated to ${dep1.name} @ ${dep1.version}`)

  return { dep1, dep2 }
}

const { dep1, dep2 } = demoDependencies()

// ===== Example 5: Packages =====
printHeader('Packages')

const demoPackages = () => {
  printSubHeader('Creating a Package', '📚')

  const pkg1 = new Package('my-app', '1.0.0')
  printSuccess(`Created package: ${pkg1.name} @ ${pkg1.version}`)

  printSubHeader('Adding Dependencies', '➕')

  printCode(`pkg1.addDependency(dep1); // react@^18.0.0`)
  printCode(`pkg1.addDependency(dep2); // lodash@~4.17.21`)

  pkg1.addDependency(dep1)
  pkg1.addDependency(dep2)

  printSuccess(`${pkg1.name} now has ${pkg1.dependencyCount} dependencies`)

  printSubHeader('Listing Dependencies', '📋')

  const deps = pkg1.dependencies()
  const depsTable = new Table({
    head: [chalk.bold.white('#'), chalk.bold.white('Name'), chalk.bold.white('Version')],
    colWidths: [5, 15, 15],
  })

  deps.forEach((dep, i) => {
    depsTable.push([i + 1, chalk.blue(dep.name), chalk.green(dep.version)])
  })

  console.log(depsTable.toString())

  printSubHeader('Getting a Specific Dependency', '🔍')

  const reactDep = pkg1.getDependency('react')
  if (reactDep) {
    printSuccess(`Found dependency: ${reactDep.name} @ ${reactDep.version}`)
  } else {
    printError(`Dependency not found!`)
  }

  printSubHeader('Updating Package Version', '📈')

  printProperty('Before', pkg1.version)
  pkg1.updateVersion('1.1.0')
  printSuccess(`Updated to ${pkg1.version}`)

  printSubHeader('Updating Dependency Version', '🔧')

  printProperty('Before', `lodash @ ${pkg1.getDependency('lodash').version}`)
  pkg1.updateDependencyVersion('lodash', '^4.18.0')
  const updatedDep = pkg1.getDependency('lodash')
  printSuccess(`Updated to ${updatedDep.name} @ ${updatedDep.version}`)

  return { pkg1 }
}

const { pkg1 } = demoPackages()

// ===== Example 6: Package Info =====
printHeader('Package Info')

const demoPackageInfo = () => {
  printSubHeader('Creating Package.json', '📝')

  const packageJsonContent = {
    name: 'my-app',
    version: '1.0.0',
    dependencies: {
      react: '^17.0.2',
      lodash: '^4.17.21',
    },
    devDependencies: {
      typescript: '^4.5.4',
    },
  }

  console.log(chalk.gray(JSON.stringify(packageJsonContent, null, 2)))

  printSubHeader('Creating PackageInfo', '📄')

  const pkgInfo = new PackageInfo(pkg1, '/path/to/package.json', '/path/to', './relative/path', packageJsonContent)

  const infoTable = new Table({
    head: [chalk.bold.white('Property'), chalk.bold.white('Value')],
    colWidths: [20, 50],
  })

  infoTable.push(
    [chalk.blue('packageJsonPath'), chalk.green(pkgInfo.packageJsonPath)],
    [chalk.blue('packagePath'), chalk.green(pkgInfo.packagePath)],
    [chalk.blue('packageRelativePath'), chalk.green(pkgInfo.packageRelativePath)],
  )

  console.log(infoTable.toString())

  printSubHeader('Accessing package.json', '📖')

  console.log(chalk.yellow('Dependencies:'))
  console.log(chalk.gray(JSON.stringify(pkgInfo.packageJson.dependencies, null, 2)))

  console.log(chalk.yellow('\nDev Dependencies:'))
  console.log(chalk.gray(JSON.stringify(pkgInfo.packageJson.devDependencies, null, 2)))

  printSubHeader('Updating Package Version', '🔄')

  printProperty('Before', `${pkgInfo.package.version}`)
  pkgInfo.updateVersion('1.1.0')
  printSuccess(`Updated package to ${pkgInfo.package.version}`)
  printSuccess(`Updated in package.json: ${pkgInfo.packageJson.version}`)

  printSubHeader('Updating Dependency Version', '🔧')

  printProperty('Before', `react @ ${pkgInfo.packageJson.dependencies.react}`)
  pkgInfo.updateDependencyVersion('react', '^18.0.0')
  printSuccess(`Updated in package.json: ${pkgInfo.packageJson.dependencies.react}`)

  return { pkgInfo }
}

const { pkgInfo } = demoPackageInfo()

// ===== Example 7: Dependency Registry =====
printHeader('Dependency Registry')

const demoDependencyRegistry = () => {
  printSubHeader('Creating Registry', '🗃️')

  const registry = new DependencyRegistry()
  printSuccess('Registry created')

  printSubHeader('Adding Dependencies', '➕')

  const reactDep = registry.getOrCreate('react', '^17.0.2')
  const lodashDep = registry.getOrCreate('lodash', '^4.17.21')
  const expressDep = registry.getOrCreate('express', '^4.17.1')

  const registryDepsTable = new Table({
    head: [chalk.bold.white('Name'), chalk.bold.white('Version')],
    colWidths: [15, 15],
  })

  registryDepsTable.push(
    [chalk.blue(reactDep.name), chalk.green(reactDep.version)],
    [chalk.blue(lodashDep.name), chalk.green(lodashDep.version)],
    [chalk.blue(expressDep.name), chalk.green(expressDep.version)],
  )

  console.log(registryDepsTable.toString())

  printSubHeader('Creating Package with Registry', '📦')

  const pkg = Package.withRegistry(
    'my-server-app',
    '1.0.0',
    [
      ['express', '^4.17.1'],
      ['lodash', '^4.17.21'],
    ],
    registry,
  )

  printSuccess(`Created ${pkg.name} @ ${pkg.version} with ${pkg.dependencyCount} dependencies`)

  printSubHeader('Resolving Version Conflicts', '⚠️')

  // Add conflicting versions
  registry.getOrCreate('react', '^18.0.0')
  printWarning('Added conflicting version for react: ^18.0.0')

  const resolution = registry.resolveVersionConflicts()

  printProperty('Resolved Versions:', '')
  const resolvedTable = new Table({
    head: [chalk.bold.white('Package'), chalk.bold.white('Resolved Version')],
    colWidths: [20, 20],
  })

  for (const prop in resolution.resolvedVersions) {
    resolvedTable.push([chalk.blue(prop), chalk.green(resolution.resolvedVersions[prop])])
  }

  console.log(resolvedTable.toString())

  if (resolution.updatesRequired.length > 0) {
    printSubHeader('Updates Required', '🔄')

    const updatesTable = new Table({
      head: [
        chalk.bold.white('Package'),
        chalk.bold.white('Dependency'),
        chalk.bold.white('From'),
        chalk.bold.white('To'),
      ],
      colWidths: [15, 15, 15, 15],
    })

    for (const update of resolution.updatesRequired) {
      updatesTable.push([
        chalk.blue(update.packageName || '(unknown)'),
        chalk.blue(update.dependencyName),
        chalk.red(update.currentVersion),
        chalk.green(update.newVersion),
      ])
    }

    console.log(updatesTable.toString())
  }

  printSubHeader('Applying Resolution', '✅')

  registry.applyResolutionResult(resolution)
  const updatedReactDep = registry.get('react')
  if (updatedReactDep) {
    printSuccess(`Updated react to ${updatedReactDep.version}`)
  }

  printSubHeader('Finding Highest Compatible Version', '📊')

  const highestVersion = registry.findHighestCompatibleVersion('react', ['^16.0.0', '^17.0.0'])
  if (highestVersion) {
    printSuccess(`Highest compatible version for react: ${highestVersion}`)
  } else {
    printWarning(`No compatible version found`)
  }

  return { registry }
}

const { registry } = demoDependencyRegistry()

// ===== Example 8: Package Diff =====
printHeader('Package Diff')

const demoPackageDiff = () => {
  printSubHeader('Creating Packages for Diff', '📦')

  // Create first version
  const oldPkg = new Package('my-app', '1.0.0')
  oldPkg.addDependency(new Dependency('react', '^17.0.2'))
  oldPkg.addDependency(new Dependency('lodash', '^4.17.21'))

  printSuccess(`Created old package: ${oldPkg.name} @ ${oldPkg.version} with ${oldPkg.dependencyCount} dependencies`)

  // Create second version with changes
  const newPkg = new Package('my-app', '2.0.0')
  newPkg.addDependency(new Dependency('react', '^18.0.0')) // Updated
  newPkg.addDependency(new Dependency('express', '^4.18.1')) // Added
  // lodash removed

  printSuccess(`Created new package: ${newPkg.name} @ ${newPkg.version} with ${newPkg.dependencyCount} dependencies`)

  printSubHeader('Generating Diff', '📊')

  const diff = PackageDiff.between(oldPkg, newPkg)

  // Basic diff info
  const diffInfoTable = new Table({
    head: [chalk.bold.white('Property'), chalk.bold.white('Value')],
    colWidths: [20, 40],
  })

  diffInfoTable.push(
    [chalk.blue('Package Name'), chalk.green(diff.packageName)],
    [chalk.blue('Previous Version'), chalk.yellow(diff.previousVersion)],
    [chalk.blue('Current Version'), chalk.yellow(diff.currentVersion)],
    [chalk.blue('Breaking Change'), diff.breakingChange ? chalk.red('Yes') : chalk.green('No')],
  )

  console.log(diffInfoTable.toString())

  printSubHeader('Dependency Changes', '🔄')

  if (diff.dependencyChanges.length === 0) {
    printWarning('No dependency changes')
  } else {
    const changesTable = new Table({
      head: [
        chalk.bold.white('Name'),
        chalk.bold.white('Change Type'),
        chalk.bold.white('Previous'),
        chalk.bold.white('Current'),
        chalk.bold.white('Breaking'),
      ],
      colWidths: [15, 15, 15, 15, 10],
    })

    for (const change of diff.dependencyChanges) {
      const changeTypeStr = changeTypeToString(change.changeType)

      let changeTypeColored
      if (changeTypeStr === 'Added') {
        changeTypeColored = chalk.green(changeTypeStr)
      } else if (changeTypeStr === 'Removed') {
        changeTypeColored = chalk.red(changeTypeStr)
      } else if (changeTypeStr === 'Updated') {
        changeTypeColored = chalk.yellow(changeTypeStr)
      } else {
        changeTypeColored = chalk.blue(changeTypeStr)
      }

      changesTable.push([
        chalk.blue(change.name),
        changeTypeColored,
        change.previousVersion ? chalk.yellow(change.previousVersion) : chalk.gray('none'),
        change.currentVersion ? chalk.yellow(change.currentVersion) : chalk.gray('none'),
        change.breaking ? chalk.red('Yes') : chalk.green('No'),
      ])
    }

    console.log(changesTable.toString())
  }

  printSubHeader('Change Statistics', '📈')

  const statsTable = new Table({
    head: [chalk.bold.white('Metric'), chalk.bold.white('Value')],
    colWidths: [25, 10],
  })

  statsTable.push([chalk.blue('Breaking Changes'), chalk.red(diff.countBreakingChanges())])

  // Changes by type count
  const counts = diff.countChangesByType()
  for (const type in counts) {
    let label
    if (type === 'added') {
      label = chalk.green('Added')
    } else if (type === 'removed') {
      label = chalk.red('Removed')
    } else if (type === 'updated') {
      label = chalk.yellow('Updated')
    } else {
      label = chalk.blue(type)
    }

    statsTable.push([label, counts[type]])
  }

  console.log(statsTable.toString())

  printSubHeader('Diff as String', '📝')

  console.log(
    boxen(diff.toString(), {
      padding: 1,
      borderStyle: 'round',
      borderColor: 'blue',
    }),
  )

  return { diff, oldPkg, newPkg }
}

const { diff, oldPkg, newPkg } = demoPackageDiff()

// ===== Example 9: Working with Dependency Changes =====
printHeader('Working with Dependency Changes')

const demoWorkingWithDependencyChanges = () => {
  printSubHeader('Extracting and Analyzing Changes', '🧮')

  // Get dependency changes from the diff
  const depChanges = diff.dependencyChanges
  printSuccess(`Found ${depChanges.length} dependency changes`)

  // Group changes by type
  const addedChanges = depChanges.filter((c) => c.changeType === ChangeType.Added)
  const removedChanges = depChanges.filter((c) => c.changeType === ChangeType.Removed)
  const updatedChanges = depChanges.filter((c) => c.changeType === ChangeType.Updated)

  // Breaking changes
  const breakingChanges = depChanges.filter((c) => c.breaking)

  const summaryTable = new Table({
    head: [chalk.bold.white('Change Type'), chalk.bold.white('Count'), chalk.bold.white('Details')],
    colWidths: [15, 10, 50],
  })

  summaryTable.push(
    [chalk.green('Added'), addedChanges.length, addedChanges.map((c) => c.name).join(', ')],
    [chalk.red('Removed'), removedChanges.length, removedChanges.map((c) => c.name).join(', ')],
    [chalk.yellow('Updated'), updatedChanges.length, updatedChanges.map((c) => c.name).join(', ')],
    [chalk.red('Breaking'), breakingChanges.length, breakingChanges.map((c) => c.name).join(', ')],
  )

  console.log(summaryTable.toString())

  printSubHeader('Change Severity Analysis', '📊')

  // Analyze the severity of changes
  const severity = (() => {
    // Assess severity based on breaking changes and overall change pattern
    if (diff.breakingChange) {
      return { level: 'Major', description: 'Package version increment (MAJOR)' }
    }
    if (breakingChanges.length > 0) {
      return { level: 'Potentially Breaking', description: 'Dependency major version changes' }
    }
    if (updatedChanges.length > 0) {
      return { level: 'Minor', description: 'Dependency updates (MINOR)' }
    }
    if (addedChanges.length > 0 || removedChanges.length > 0) {
      return { level: 'Feature Change', description: 'Dependencies added or removed' }
    }
    return { level: 'Minimal', description: 'No significant changes' }
  })()

  let levelColor
  switch (severity.level) {
    case 'Major':
      levelColor = chalk.red.bold(severity.level)
      break
    case 'Potentially Breaking':
      levelColor = chalk.red(severity.level)
      break
    case 'Minor':
      levelColor = chalk.yellow(severity.level)
      break
    case 'Feature Change':
      levelColor = chalk.blue(severity.level)
      break
    default:
      levelColor = chalk.green(severity.level)
  }

  console.log(`${chalk.bold('Severity Assessment:')} ${levelColor} - ${chalk.white(severity.description)}`)

  return { oldPkg, newPkg, diff, depChanges }
}

demoWorkingWithDependencyChanges()

// ===== Example 10: Scoped Package Parsing =====
printHeader('Scoped Package Parsing')

const demoScopedPackageParsing = () => {
  // Parse different formats of scoped packages
  const examples = [
    '@scope/package',
    '@scope/package@1.0.0',
    '@scope/package@1.0.0@/some/path',
    '@scope/package:1.0.0',
    'non-scoped-package', // Should return null
  ]

  printSubHeader('Parsing Different Package Formats', '🔍')

  const resultsTable = new Table({
    head: [
      chalk.bold.white('Input'),
      chalk.bold.white('Full'),
      chalk.bold.white('Name'),
      chalk.bold.white('Version'),
      chalk.bold.white('Path'),
    ],
    colWidths: [25, 25, 20, 15, 20],
  })

  for (const example of examples) {
    const result = parseScopedPackage(example)

    if (result) {
      resultsTable.push([
        chalk.blue(example),
        chalk.green(result.full),
        chalk.yellow(result.name),
        chalk.cyan(result.version),
        result.path ? chalk.magenta(result.path) : chalk.gray('none'),
      ])
    } else {
      resultsTable.push([
        chalk.blue(example),
        chalk.red('Not a scoped package'),
        chalk.gray('-'),
        chalk.gray('-'),
        chalk.gray('-'),
      ])
    }
  }

  console.log(resultsTable.toString())
}

demoScopedPackageParsing()

// ===== Example 11: Real-world Example - Package Update =====
printHeader('Real-world Example: Package Update Workflow')

const realWorldExample = () => {
  printSubHeader('Scenario: Updating Dependencies Across Packages', '🔄')

  // Create a registry
  const registry = new DependencyRegistry()

  // Create multiple packages with ACTUAL conflicting dependencies
  // Notice we're using different version specifiers that are truly incompatible
  const serverPkg = Package.withRegistry(
    'my-server',
    '1.0.0',
    [
      ['express', '^4.17.1'],
      ['lodash', '^4.17.21'], // Wants 4.17.x
      ['common-lib', '2.x'], // Server wants v2.x
    ],
    registry,
  )

  const clientPkg = Package.withRegistry(
    'my-client',
    '1.0.0',
    [
      ['react', '^17.0.2'],
      ['lodash', '~4.16.0'], // Wants 4.16.x only
      ['common-lib', '2.3.0'], // Client wants exactly 2.3.0
    ],
    registry,
  )

  const sharedPkg = Package.withRegistry(
    'my-shared',
    '1.0.0',
    [
      ['lodash', '4.15.0'], // Wants exactly 4.15.0
      ['common-lib', '^1.9.0'], // Shared wants v1.x - this is a REAL conflict
    ],
    registry,
  )

  const packagesTable = new Table({
    head: [chalk.bold.white('Package'), chalk.bold.white('Version'), chalk.bold.white('Dependencies')],
    colWidths: [15, 10, 50],
  })

  packagesTable.push(
    [chalk.blue(serverPkg.name), chalk.green(serverPkg.version), 'express@^4.17.1, lodash@^4.17.21, common-lib@2.x'],
    [chalk.blue(clientPkg.name), chalk.green(clientPkg.version), 'react@^17.0.2, lodash@~4.16.0, common-lib@2.3.0'],
    [chalk.blue(sharedPkg.name), chalk.green(sharedPkg.version), 'lodash@4.15.0, common-lib@^1.9.0'],
  )

  console.log(packagesTable.toString())

  // Add more versions to the registry to simulate available versions
  registry.getOrCreate('common-lib', '1.9.5')
  registry.getOrCreate('common-lib', '2.0.0')
  registry.getOrCreate('common-lib', '2.3.0')
  registry.getOrCreate('common-lib', '2.5.0')

  registry.getOrCreate('lodash', '4.15.0')
  registry.getOrCreate('lodash', '4.16.6')
  registry.getOrCreate('lodash', '4.17.0')
  registry.getOrCreate('lodash', '4.17.21')

  printSubHeader('Detecting and Resolving Version Conflicts', '⚠️')

  printWarning('Examining dependencies for conflicts...')

  // Log conflicting dependencies before resolution
  const beforeTable = new Table({
    head: [chalk.bold.white('Dependency'), chalk.bold.white('Versions Required')],
    colWidths: [15, 60],
  })

  beforeTable.push(
    [chalk.red('lodash'), chalk.yellow('^4.17.21, ~4.16.0, 4.15.0')],
    [chalk.red('common-lib'), chalk.yellow('2.x, 2.3.0, ^1.9.0')],
  )

  console.log(beforeTable.toString())
  printWarning('These conflicting requirements cannot all be satisfied simultaneously!')

  const resolution = registry.resolveVersionConflicts()
  printSuccess(`Found ${resolution.updatesRequired.length} conflicting versions to resolve`)

  // Show the resolution result with warnings
  printSubHeader('Resolution Result', '🔍')

  const resolvedTable = new Table({
    head: [chalk.bold.white('Package'), chalk.bold.white('Resolved Version'), chalk.bold.white('Notes')],
    colWidths: [15, 20, 40],
  })

  for (const [pkg, version] of Object.entries(resolution.resolvedVersions)) {
    let notes = ''
    let versionDisplay = chalk.green(version)

    if (pkg === 'common-lib') {
      notes = chalk.yellow('⚠️ May break my-shared which wants v1.x')
      versionDisplay = chalk.yellow(version)
    } else if (pkg === 'lodash' && version === '4.17.21') {
      notes = chalk.yellow('⚠️ Higher than what my-client and my-shared specified')
      versionDisplay = chalk.yellow(version)
    }

    resolvedTable.push([chalk.blue(pkg), versionDisplay, notes])
  }

  console.log(resolvedTable.toString())

  // Apply resolution to all packages
  printSubHeader('Applying Resolution to All Packages', '✅')

  const updatesTable = new Table({
    head: [
      chalk.bold.white('Package'),
      chalk.bold.white('Dependency'),
      chalk.bold.white('From'),
      chalk.bold.white('To'),
    ],
    colWidths: [15, 15, 15, 15],
  })

  for (const pkg of [serverPkg, clientPkg, sharedPkg]) {
    const updates = pkg.updateDependenciesFromResolution(resolution)

    for (const [name, oldVersion, newVersion] of updates) {
      updatesTable.push([chalk.blue(pkg.name), chalk.yellow(name), chalk.red(oldVersion), chalk.green(newVersion)])
    }
  }

  if (updatesTable.length === 0) {
    console.log(chalk.yellow("No updates applied - this shouldn't happen with our conflicting versions!"))
  } else {
    console.log(updatesTable.toString())
  }

  printSubHeader('Final State After Resolution', '🏁')

  const finalStateTable = new Table({
    head: [chalk.bold.white('Package'), chalk.bold.white('lodash Version'), chalk.bold.white('common-lib Version')],
    colWidths: [15, 20, 20],
  })

  for (const pkg of [serverPkg, clientPkg, sharedPkg]) {
    const lodashDep = pkg.getDependency('lodash')
    const commonLibDep = pkg.getDependency('common-lib')

    finalStateTable.push([
      chalk.blue(pkg.name),
      lodashDep ? chalk.green(lodashDep.version) : chalk.gray('N/A'),
      commonLibDep ? chalk.green(commonLibDep.version) : chalk.gray('N/A'),
    ])
  }

  console.log(finalStateTable.toString())

  printSubHeader('Analysis of Resolution Impact', '🔬')

  // Create a copy of the original package to compare
  const oldClientPkg = new Package('my-client', '1.0.0')
  oldClientPkg.addDependency(new Dependency('react', '^17.0.2'))
  oldClientPkg.addDependency(new Dependency('lodash', '~4.16.0'))
  oldClientPkg.addDependency(new Dependency('common-lib', '2.3.0'))

  const clientDiff = PackageDiff.between(oldClientPkg, clientPkg)

  console.log(
    boxen(clientDiff.toString(), {
      padding: 1,
      borderStyle: 'round',
      borderColor: 'cyan',
    }),
  )

  printWarning('Potential Risk Analysis:')

  const riskTable = new Table({
    head: [chalk.bold.white('Package'), chalk.bold.white('Risk Level'), chalk.bold.white('Description')],
    colWidths: [15, 15, 60],
  })

  riskTable.push(
    [chalk.blue('my-server'), chalk.green('Low'), 'All dependencies upgraded to compatible versions'],
    [chalk.blue('my-client'), chalk.yellow('Medium'), 'lodash upgraded beyond specified range (~4.16.0 → ^4.17.21)'],
    [chalk.blue('my-shared'), chalk.red('High'), 'common-lib potentially breaking: ^1.9.0 → 2.x'],
  )

  console.log(riskTable.toString())

  // Add recommendation
  printSubHeader('Recommended Actions', '📋')
  console.log(chalk.cyan('Based on the dependency resolution:'))
  console.log(chalk.green('✓ Proceed with lodash updates - likely backward compatible'))
  console.log(chalk.yellow('⚠️ Test my-shared extensively after common-lib update from v1 to v2'))
  console.log(chalk.red('❗ Consider updating my-shared to support common-lib v2'))

  printSubHeader('Demonstrating Version Bumps', '🚀')

  function performVersionBump(pkg, versionType, sha = null) {
    const oldVersion = pkg.version
    let newVersion
    let emoji

    switch (versionType) {
      case Version.Major:
        newVersion = VersionUtils.bumpMajor(oldVersion)
        emoji = '💥'
        break
      case Version.Minor:
        newVersion = VersionUtils.bumpMinor(oldVersion)
        emoji = '✨'
        break
      case Version.Patch:
        newVersion = VersionUtils.bumpPatch(oldVersion)
        emoji = '🔧'
        break
      case Version.Snapshot:
        newVersion = VersionUtils.bumpSnapshot(oldVersion, sha || 'abcdef')
        emoji = '📸'
        break
    }

    pkg.updateVersion(newVersion)
    return {
      name: pkg.name,
      from: oldVersion,
      to: newVersion,
      type: versionTypeToString(versionType),
      emoji,
    }
  }

  const bumpResultsTable = new Table({
    head: [chalk.bold.white('Package'), chalk.bold.white('Type'), chalk.bold.white('From'), chalk.bold.white('To')],
    colWidths: [15, 15, 15, 20],
  })

  const majorBump = performVersionBump(serverPkg, Version.Major)
  const minorBump = performVersionBump(clientPkg, Version.Minor)
  const patchBump = performVersionBump(sharedPkg, Version.Patch)
  const snapshotPkg = new Package('snapshot-pkg', '1.0.0')
  const snapshotBump = performVersionBump(snapshotPkg, Version.Snapshot, 'abc123')

  bumpResultsTable.push(
    [
      chalk.blue(majorBump.name),
      `${majorBump.emoji} ${chalk.red(majorBump.type)}`,
      chalk.yellow(majorBump.from),
      chalk.green(majorBump.to),
    ],
    [
      chalk.blue(minorBump.name),
      `${minorBump.emoji} ${chalk.yellow(minorBump.type)}`,
      chalk.yellow(minorBump.from),
      chalk.green(minorBump.to),
    ],
    [
      chalk.blue(patchBump.name),
      `${patchBump.emoji} ${chalk.blue(patchBump.type)}`,
      chalk.yellow(patchBump.from),
      chalk.green(patchBump.to),
    ],
    [
      chalk.blue(snapshotBump.name),
      `${snapshotBump.emoji} ${chalk.cyan(snapshotBump.type)}`,
      chalk.yellow(snapshotBump.from),
      chalk.green(snapshotBump.to),
    ],
  )

  console.log(bumpResultsTable.toString())
}

realWorldExample()

// Final completion message
console.log(
  '\n' +
    boxen(
      chalk.bold.green('🎉 ') +
        chalk.bold.green('All ') +
        chalk.bold.yellow('Examples ') +
        chalk.bold.blue('Completed ') +
        chalk.bold.magenta('Successfully! ') +
        chalk.bold.green('🎉'),
      {
        padding: 1,
        margin: 1,
        borderStyle: 'round',
        align: 'center',
      },
    ),
)
