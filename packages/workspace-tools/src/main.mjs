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
  PackageRegistry,
  RegistryManager,
  RegistryType,
  DependencyFilter,
  DependencyGraph,
  ValidationIssueType,
  ValidationReport,
  DependencyUpgrader,
  ExecutionMode,
  UpgradeStatus,
  VersionStability,
  VersionUpdateStrategy,
  bumpSnapshotVersion,
  bumpVersion,
  createDefaultUpgradeConfig,
  createUpgradeConfigFromStrategy,
  createUpgradeConfigWithRegistries,
  buildDependencyGraphFromPackageInfos,
  buildDependencyGraphFromPackages,
  generateAscii,
  generateDot,
  saveDotToFile,
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

// ===== Example 12: Package Registry =====
printHeader('Package Registry')

const demoPackageRegistry = () => {
  printSubHeader('Creating Package Registries', '🏢')

  // Create different types of registries
  const npmRegistry = PackageRegistry.createNpmRegistry('https://registry.npmjs.org')
  const githubRegistry = PackageRegistry.createNpmRegistry('https://npm.pkg.github.com')
  const localRegistry = PackageRegistry.createLocalRegistry()

  const registriesTable = new Table({
    head: [chalk.bold.white('Registry Type'), chalk.bold.white('URL'), chalk.bold.white('Purpose')],
    colWidths: [20, 30, 35],
  })

  registriesTable.push(
    [chalk.blue('NPM'), chalk.green('https://registry.npmjs.org'), chalk.yellow('Public packages')],
    [chalk.blue('GitHub'), chalk.green('https://npm.pkg.github.com'), chalk.yellow('Private organization packages')],
    [chalk.blue('Local'), chalk.green('(in-memory)'), chalk.yellow('Testing and offline work')],
  )

  console.log(registriesTable.toString())

  printSubHeader('Setting Authentication', '🔑')

  // Set auth for the GitHub registry
  try {
    const auth = {
      token: 'github_pat_xxxxxxxxxxxx',
      tokenType: 'Bearer',
      always: true,
    }

    printCode(`githubRegistry.setAuth({
  token: 'github_pat_xxxxxxxxxxxx',
  tokenType: 'Bearer',
  always: true
})`)

    githubRegistry.setAuth(auth)
    printSuccess('Authentication set for GitHub registry')
  } catch (err) {
    printError(`Failed to set auth: ${err.message}`)
  }

  printSubHeader('Setting User Agent', '🤖')

  printCode(`npmRegistry.setUserAgent('My-Awesome-App/1.0.0')`)

  try {
    npmRegistry.setUserAgent('My-Awesome-App/1.0.0')
    printSuccess('User agent set for NPM registry')
  } catch (err) {
    printError(`Failed to set user agent: ${err.message}`)
  }

  printSubHeader('Working with Local Registry', '💾')

  // Add packages to local registry
  printCode(`localRegistry.addPackage('test-lib', ['1.0.0', '1.1.0', '2.0.0'])`)

  try {
    localRegistry.addPackage('test-lib', ['1.0.0', '1.1.0', '2.0.0'])
    printSuccess('Added test-lib with 3 versions to local registry')

    const dependencies = {
      lodash: '^4.17.21',
      chalk: '^4.1.2',
    }

    printCode(`localRegistry.setDependencies('test-lib', '2.0.0', {
  'lodash': '^4.17.21',
  'chalk': '^4.1.2'
})`)

    localRegistry.setDependencies('test-lib', '2.0.0', dependencies)
    printSuccess('Set dependencies for test-lib@2.0.0')
  } catch (err) {
    printError(`Failed to work with local registry: ${err.message}`)
  }

  printSubHeader('Querying Package Information', '🔍')

  // Fetch package data
  let latestVersion, allVersions, packageInfo

  try {
    printCode(`latestVersion = localRegistry.getLatestVersion('test-lib')`)
    latestVersion = localRegistry.getLatestVersion('test-lib')
    printSuccess(`Latest version of test-lib: ${chalk.green(latestVersion)}`)

    printCode(`allVersions = localRegistry.getAllVersions('test-lib')`)
    allVersions = localRegistry.getAllVersions('test-lib')

    const versionsTable = new Table({
      head: [chalk.bold.white('#'), chalk.bold.white('Version')],
      colWidths: [5, 15],
    })

    allVersions.forEach((version, i) => {
      versionsTable.push([i + 1, chalk.blue(version)])
    })

    console.log(versionsTable.toString())

    printCode(`packageInfo = localRegistry.getPackageInfo('test-lib', '2.0.0')`)
    packageInfo = localRegistry.getPackageInfo('test-lib', '2.0.0')

    printSubHeader('Package Metadata', '📋')
    console.log(chalk.gray(JSON.stringify(packageInfo, null, 2)))
  } catch (err) {
    printError(`Failed to query registry: ${err.message}`)
  }

  printSubHeader('Getting All Packages', '📚')

  try {
    printCode(`allPackages = localRegistry.getAllPackages()`)
    const allPackages = localRegistry.getAllPackages()
    printSuccess(`Registry contains ${allPackages.length} packages: ${chalk.green(allPackages.join(', '))}`)
  } catch (err) {
    printError(`Failed to get all packages: ${err.message}`)
  }

  printSubHeader('Clearing Registry Cache', '🧹')

  printCode(`npmRegistry.clearCache()`)
  try {
    npmRegistry.clearCache()
    printSuccess('Registry cache cleared')
  } catch (err) {
    printError(`Failed to clear cache: ${err.message}`)
  }

  return { npmRegistry, githubRegistry, localRegistry }
}

const { npmRegistry, githubRegistry, localRegistry } = demoPackageRegistry()

// ===== Example 13: Registry Manager =====
printHeader('Registry Manager')

const demoRegistryManager = () => {
  printSubHeader('Creating Registry Manager', '🏛️')

  const manager = new RegistryManager()
  printSuccess('Registry Manager created')

  printProperty('Default Registry', manager.defaultRegistry)

  printSubHeader('Adding Registries', '➕')

  // Add different registry types
  const registriesTable = new Table({
    head: [chalk.bold.white('Type'), chalk.bold.white('URL'), chalk.bold.white('Status')],
    colWidths: [15, 35, 25],
  })

  try {
    printCode(`manager.addRegistry('https://registry.npmjs.org', RegistryType.Npm)`)
    manager.addRegistry('https://registry.npmjs.org', RegistryType.Npm)
    registriesTable.push([chalk.blue('NPM'), chalk.green('https://registry.npmjs.org'), chalk.green('✓ Added')])

    printCode(`manager.addRegistry('https://npm.pkg.github.com', RegistryType.GitHub)`)
    manager.addRegistry('https://npm.pkg.github.com', RegistryType.GitHub)
    registriesTable.push([chalk.blue('GitHub'), chalk.green('https://npm.pkg.github.com'), chalk.green('✓ Added')])

    printCode(`manager.addRegistry('https://registry.custom.com', RegistryType.Custom, 'custom-client')`)
    manager.addRegistry('https://registry.custom.com', RegistryType.Custom, 'custom-client')
    registriesTable.push([chalk.blue('Custom'), chalk.green('https://registry.custom.com'), chalk.green('✓ Added')])
  } catch (err) {
    printError(`Failed to add registry: ${err.message}`)
  }

  console.log(registriesTable.toString())

  printSubHeader('Registry URLs', '🔗')

  printCode(`registryUrls = manager.registryUrls()`)
  const registryUrls = manager.registryUrls()

  const urlsTable = new Table({
    head: [chalk.bold.white('#'), chalk.bold.white('Registry URL')],
    colWidths: [5, 50],
  })

  registryUrls.forEach((url, i) => {
    urlsTable.push([i + 1, chalk.green(url)])
  })

  console.log(urlsTable.toString())

  printSubHeader('Setting Authentication', '🔑')

  try {
    const auth = {
      token: 'npm_xxxxxxxxxxxx',
      tokenType: 'Bearer',
      always: false,
    }

    printCode(`manager.setAuth('https://registry.npmjs.org', {
  token: 'npm_xxxxxxxxxxxx',
  tokenType: 'Bearer',
  always: false
})`)

    manager.setAuth('https://registry.npmjs.org', auth)
    printSuccess('Authentication set for npm registry')

    // GitHub auth
    const githubAuth = {
      token: 'github_pat_xxxxxxxxxxxx',
      tokenType: 'Bearer',
      always: true,
    }

    printCode(`manager.setAuth('https://npm.pkg.github.com', {
  token: 'github_pat_xxxxxxxxxxxx',
  tokenType: 'Bearer',
  always: true
})`)

    manager.setAuth('https://npm.pkg.github.com', githubAuth)
    printSuccess('Authentication set for GitHub registry')
  } catch (err) {
    printError(`Failed to set authentication: ${err.message}`)
  }

  printSubHeader('Associating Scopes', '🔄')

  const scopesTable = new Table({
    head: [chalk.bold.white('Scope'), chalk.bold.white('Registry'), chalk.bold.white('Status')],
    colWidths: [15, 35, 25],
  })

  try {
    printCode(`manager.associateScope('@myorg', 'https://npm.pkg.github.com')`)
    manager.associateScope('@myorg', 'https://npm.pkg.github.com')
    scopesTable.push([chalk.blue('@myorg'), chalk.green('https://npm.pkg.github.com'), chalk.green('✓ Associated')])

    printCode(`manager.associateScope('@custom', 'https://registry.custom.com')`)
    manager.associateScope('@custom', 'https://registry.custom.com')
    scopesTable.push([chalk.blue('@custom'), chalk.green('https://registry.custom.com'), chalk.green('✓ Associated')])
  } catch (err) {
    printError(`Failed to associate scope: ${err.message}`)
  }

  console.log(scopesTable.toString())

  printSubHeader('Working with Scopes', '📋')

  const scopeCheckTable = new Table({
    head: [chalk.bold.white('Action'), chalk.bold.white('Result')],
    colWidths: [40, 35],
  })

  printCode(`manager.hasScope('@myorg')`)
  const hasMyOrg = manager.hasScope('@myorg')
  scopeCheckTable.push([chalk.blue('Check if @myorg scope exists'), hasMyOrg ? chalk.green('Yes') : chalk.red('No')])

  printCode(`manager.getRegistryForScope('@myorg')`)
  const myorgRegistry = manager.getRegistryForScope('@myorg')
  scopeCheckTable.push([chalk.blue('Get registry for @myorg'), chalk.green(myorgRegistry || 'None')])

  printCode(`manager.hasScope('@nonexistent')`)
  const hasNonexistent = manager.hasScope('@nonexistent')
  scopeCheckTable.push([
    chalk.blue('Check if @nonexistent scope exists'),
    hasNonexistent ? chalk.green('Yes') : chalk.red('No'),
  ])

  console.log(scopeCheckTable.toString())

  printSubHeader('Setting Default Registry', '⭐')

  try {
    printCode(`manager.setDefaultRegistry('https://registry.custom.com')`)
    manager.setDefaultRegistry('https://registry.custom.com')
    printSuccess(`Default registry changed to: ${chalk.green('https://registry.custom.com')}`)
    printProperty('Current Default', manager.defaultRegistry)
  } catch (err) {
    printError(`Failed to set default registry: ${err.message}`)
  }

  printSubHeader('Loading from .npmrc', '📄')

  try {
    printCode(`manager.loadFromNpmrc()`)
    // Note: This won't actually find an .npmrc file in most test environments
    manager.loadFromNpmrc()
    printSuccess('Loaded configuration from .npmrc file')

    printCode(`manager.loadFromNpmrc('/custom/path/.npmrc')`)
    // This will likely fail with file not found, but we'll handle that
    try {
      manager.loadFromNpmrc('/custom/path/.npmrc')
      printSuccess('Loaded configuration from custom .npmrc file')
    } catch (err) {
      printWarning(`Custom .npmrc not found: ${err.message}`)
    }
  } catch (err) {
    printWarning(`Failed to load .npmrc: ${err.message}`)
  }

  printSubHeader('Querying Package Information', '🔍')

  // Example queries
  const queryExamples = [
    {
      package: 'react',
      action: 'getLatestVersion',
      code: `manager.getLatestVersion('react')`,
    },
    {
      package: '@myorg/private-pkg',
      action: 'getLatestVersion',
      code: `manager.getLatestVersion('@myorg/private-pkg')`,
    },
    {
      package: 'react',
      action: 'getAllVersions',
      code: `manager.getAllVersions('react')`,
    },
  ]

  for (const example of queryExamples) {
    printCode(example.code)
    try {
      let result

      if (example.action === 'getLatestVersion') {
        result = manager.getLatestVersion(example.package)
        if (result) {
          printSuccess(`Latest version of ${chalk.blue(example.package)}: ${chalk.green(result)}`)
        } else {
          printWarning(`No version found for ${chalk.blue(example.package)}`)
        }
      } else if (example.action === 'getAllVersions') {
        result = manager.getAllVersions(example.package)
        if (result && result.length > 0) {
          printSuccess(`Found ${result.length} versions for ${chalk.blue(example.package)}`)
          console.log(chalk.gray(`First 5 versions: ${result.slice(0, 5).join(', ')}...`))
        } else {
          printWarning(`No versions found for ${chalk.blue(example.package)}`)
        }
      }
    } catch (err) {
      printError(`Failed to query ${example.package}: ${err.message}`)
    }
  }

  printSubHeader('Real-world Registry Flow', '🌐')

  const realWorldFlow = `
// Initialize registry manager
const manager = new RegistryManager();

// Add company private registry
manager.addRegistry('https://npm.company.com', RegistryType.Custom, 'company-client');
manager.setAuth('https://npm.company.com', {
  token: process.env.NPM_TOKEN,
  tokenType: 'Bearer',
  always: true
});

// Associate organization scopes with registries
manager.associateScope('@company', 'https://npm.company.com');
manager.associateScope('@myteam', 'https://npm.company.com');

// Load any user registry configuration
manager.loadFromNpmrc();

// Get package info based on scope routing
const companyPkg = manager.getLatestVersion('@company/ui-components');
const publicPkg = manager.getLatestVersion('react');
const teamPkg = manager.getLatestVersion('@myteam/utils');

// All versions route to the appropriate registry automatically
console.log(\`Company package: \${companyPkg}\`);  // From private registry
console.log(\`Public package: \${publicPkg}\`);    // From npmjs.org
console.log(\`Team package: \${teamPkg}\`);        // From private registry
`

  console.log(
    boxen(chalk.cyan(realWorldFlow), {
      padding: 1,
      borderStyle: 'round',
      borderColor: 'blue',
      title: 'Registry Manager Example',
      titleAlignment: 'center',
    }),
  )

  return { manager }
}

const { manager } = demoRegistryManager()

// ===== Example 14: Integration Across Features =====
printHeader('Complete Package Management Workflow')

const demoCompleteWorkflow = () => {
  printSubHeader('Setting Up Environment', '🌱')

  // Create registries and manager
  const manager = new RegistryManager()
  const localRegistry = PackageRegistry.createLocalRegistry()

  printSuccess('Registry manager and local registry created')

  // Add packages to local registry with dependencies
  localRegistry.addPackage('shared-lib', ['1.0.0', '1.1.0', '2.0.0'])
  localRegistry.addPackage('ui-components', ['1.0.0', '1.2.0', '2.0.0'])
  localRegistry.addPackage('api-client', ['1.0.0', '1.5.0'])

  // Add dependencies to packages
  localRegistry.setDependencies('ui-components', '2.0.0', {
    'shared-lib': '^2.0.0',
    react: '^17.0.2',
  })

  localRegistry.setDependencies('api-client', '1.5.0', {
    'shared-lib': '^1.0.0',
    axios: '^0.24.0',
  })

  // Add local registry to manager
  manager.addRegistry('https://local-registry', RegistryType.Npm)
  manager.setDefaultRegistry('https://local-registry')

  printSuccess('Local registry populated with test packages and dependencies')

  printSubHeader('Dependency Resolution Workflow', '⚙️')

  // Step 1: Create a dependency registry
  printCode(`const depRegistry = new DependencyRegistry()`)
  const depRegistry = new DependencyRegistry()

  // Step 2: Create some packages using that registry
  printCode(`
// Create app package
const appPkg = Package.withRegistry(
  'my-app',
  '1.0.0',
  [
    ['shared-lib', '^1.0.0'],
    ['ui-components', '^1.0.0'],
    ['api-client', '^1.0.0']
  ],
  depRegistry
)

// Create another package with conflicting dependencies
const dashboardPkg = Package.withRegistry(
  'dashboard',
  '1.0.0',
  [
    ['shared-lib', '^2.0.0'],
    ['ui-components', '^2.0.0']
  ],
  depRegistry
)`)

  // Create packages
  const appPkg = Package.withRegistry(
    'my-app',
    '1.0.0',
    [
      ['shared-lib', '^1.0.0'],
      ['ui-components', '^1.0.0'],
      ['api-client', '^1.0.0'],
    ],
    depRegistry,
  )

  const dashboardPkg = Package.withRegistry(
    'dashboard',
    '1.0.0',
    [
      ['shared-lib', '^2.0.0'],
      ['ui-components', '^2.0.0'],
    ],
    depRegistry,
  )

  printSuccess(`Created packages: ${chalk.blue('my-app')} and ${chalk.blue('dashboard')}`)

  // Step 3: Generate combined dependency info from packages
  printCode(`const combinedInfo = Package.generateDependencyInfo([appPkg, dashboardPkg])`)
  const combinedInfo = Package.generateDependencyInfo([appPkg, dashboardPkg])

  printSubHeader('Dependency Analysis', '📊')

  // Show total dependencies
  printProperty('Total Dependencies', combinedInfo.totalDependencies)

  // List all dependencies
  const depsTable = new Table({
    head: [
      chalk.bold.white('Dependency'),
      chalk.bold.white('Versions'),
      chalk.bold.white('Used By'),
      chalk.bold.white('Conflict'),
    ],
    colWidths: [20, 25, 25, 10],
  })

  for (const [dep, info] of Object.entries(combinedInfo.dependencies)) {
    const hasConflict = info.versions.length > 1

    depsTable.push([
      chalk.blue(dep),
      info.versions.join(', '),
      info.packages.join(', '),
      hasConflict ? chalk.red('✗ Yes') : chalk.green('✓ No'),
    ])
  }

  console.log(depsTable.toString())

  // Step 4: Look for version conflicts
  printCode(`
// Check each package for version conflicts
const appConflicts = appPkg.findVersionConflicts()
const dashboardConflicts = dashboardPkg.findVersionConflicts()`)

  printSubHeader('Resolving Conflicts', '🛠️')

  printCode(`// Resolve version conflicts
const resolution = depRegistry.resolveVersionConflicts()`)

  // Resolve conflicts
  const resolution = depRegistry.resolveVersionConflicts()

  // Print resolved versions
  const resolvedTable = new Table({
    head: [chalk.bold.white('Dependency'), chalk.bold.white('Resolved Version')],
    colWidths: [30, 30],
  })

  for (const [dep, version] of Object.entries(resolution.resolvedVersions)) {
    resolvedTable.push([chalk.blue(dep), chalk.green(version)])
  }

  console.log(resolvedTable.toString())

  // Print updates required
  if (resolution.updatesRequired.length > 0) {
    printSubHeader('Updates Required', '🔄')

    const updatesTable = new Table({
      head: [
        chalk.bold.white('Package'),
        chalk.bold.white('Dependency'),
        chalk.bold.white('From'),
        chalk.bold.white('To'),
      ],
      colWidths: [15, 20, 20, 20],
    })

    for (const update of resolution.updatesRequired) {
      updatesTable.push([
        update.packageName || '(unknown)',
        chalk.blue(update.dependencyName),
        chalk.red(update.currentVersion),
        chalk.green(update.newVersion),
      ])
    }

    console.log(updatesTable.toString())
  }

  // Step 5: Upgrade with a specific strategy
  printSubHeader('Version Bump Workflow', '📈')

  // Define the workflow as a series of steps with outputs
  const workflowTable = new Table({
    head: [chalk.bold.white('Step'), chalk.bold.white('Action'), chalk.bold.white('Result')],
    colWidths: [5, 30, 45],
  })

  // Step 1: Analyze current versions
  workflowTable.push([
    '1',
    chalk.blue('Analyze current versions'),
    `${chalk.yellow('my-app')}: ${chalk.green('1.0.0')}, ${chalk.yellow('dashboard')}: ${chalk.green('1.0.0')}`,
  ])

  // Step 2: Determine required bump type
  const bumpType = resolution.updatesRequired.length > 0 ? Version.Minor : Version.Patch
  const bumpTypeStr = bumpType === Version.Minor ? 'MINOR' : 'PATCH'

  workflowTable.push([
    '2',
    chalk.blue(`Determine bump type based on changes`),
    `Required bump: ${chalk.yellow(bumpTypeStr)}`,
  ])

  // Step 3: Apply the calculated bump
  let newAppVersion, newDashboardVersion

  if (bumpType === Version.Minor) {
    newAppVersion = VersionUtils.bumpMinor(appPkg.version)
    newDashboardVersion = VersionUtils.bumpMinor(dashboardPkg.version)
  } else {
    newAppVersion = VersionUtils.bumpPatch(appPkg.version)
    newDashboardVersion = VersionUtils.bumpPatch(dashboardPkg.version)
  }

  workflowTable.push([
    '3',
    chalk.blue(`Apply ${bumpTypeStr} version bump`),
    `New versions: ${chalk.yellow('my-app')}: ${chalk.green(newAppVersion)}, ${chalk.yellow('dashboard')}: ${chalk.green(newDashboardVersion)}`,
  ])

  // Step 4: Apply dependency updates
  workflowTable.push([
    '4',
    chalk.blue('Apply dependency updates'),
    `Applied ${chalk.yellow(resolution.updatesRequired.length)} updates to dependencies`,
  ])

  // Step 5: Update package.json files
  workflowTable.push([
    '5',
    chalk.blue('Create package.json updates'),
    `Updated package.json files with new versions and dependencies`,
  ])

  console.log(workflowTable.toString())

  // Step 6: Show the complete process output
  printSubHeader('Complete Process Output', '📝')

  const processOutput = `
> workspace-tools bump --packages=my-app,dashboard

Workspace Tools v${getVersion()}

Analyzing dependencies...
Found ${resolution.updatesRequired.length} dependencies requiring updates

Updating packages:
- my-app: 1.0.0 → ${newAppVersion}
- dashboard: 1.0.0 → ${newDashboardVersion}

Updating dependencies:
${resolution.updatesRequired
  .map((update) => `- ${update.dependencyName}: ${update.currentVersion} → ${update.newVersion}`)
  .join('\n')}

Writing changes to disk...
✓ Successfully updated all package files
✓ Dependency graph is consistent

Run 'git diff' to see the changes
  `

  console.log(
    boxen(chalk.white(processOutput), {
      padding: 1,
      borderStyle: 'round',
      borderColor: 'green',
      title: 'Terminal Output',
      titleAlignment: 'center',
    }),
  )

  return { appPkg, dashboardPkg, resolution }
}

const { appPkg, dashboardPkg, resolution } = demoCompleteWorkflow()

// ===== Example 15: Package Registry Comparison =====
printHeader('Package Registry Comparison')

const demoRegistryComparison = () => {
  printSubHeader('Types of Package Registries', '🔍')

  const registryTypesTable = new Table({
    head: [
      chalk.bold.white('Registry Type'),
      chalk.bold.white('Use Case'),
      chalk.bold.white('Features'),
      chalk.bold.white('Example'),
    ],
    colWidths: [15, 25, 30, 30],
  })

  registryTypesTable.push(
    [
      chalk.blue('NPM'),
      'Public packages, default registry',
      '- Standard npm protocol\n- Public access by default\n- Rate limited for anonymous',
      chalk.green('const npmReg = PackageRegistry.createNpmRegistry("https://registry.npmjs.org")'),
    ],
    [
      chalk.blue('GitHub'),
      'Organization private packages',
      '- Scoped packages only\n- Private and public options\n- GitHub token auth',
      chalk.green('const githubReg = PackageRegistry.createNpmRegistry("https://npm.pkg.github.com")'),
    ],
    [
      chalk.blue('Local'),
      'Testing and offline development',
      '- In-memory storage\n- No network needed\n- Programmatic package creation',
      chalk.green('const localReg = PackageRegistry.createLocalRegistry()'),
    ],
  )

  console.log(registryTypesTable.toString())

  printSubHeader('Registry Performance Comparison', '⚡')

  // Simulated performance data
  const perfTable = new Table({
    head: [
      chalk.bold.white('Operation'),
      chalk.bold.white('NPM Registry'),
      chalk.bold.white('GitHub Registry'),
      chalk.bold.white('Local Registry'),
    ],
    colWidths: [25, 15, 20, 15],
  })

  perfTable.push(
    [chalk.blue('Get Latest Version'), chalk.yellow('~300ms'), chalk.yellow('~350ms'), chalk.green('<1ms')],
    [chalk.blue('Get All Versions'), chalk.yellow('~500ms'), chalk.yellow('~550ms'), chalk.green('<1ms')],
    [chalk.blue('Get Package Info'), chalk.yellow('~650ms'), chalk.yellow('~700ms'), chalk.green('<1ms')],
    [chalk.blue('Cache Hit Performance'), chalk.green('~10ms'), chalk.green('~15ms'), chalk.green('<1ms')],
  )

  console.log(perfTable.toString())

  printSubHeader('Working with Multiple Registries', '🌐')

  const multiRegistryCode = `
// Create Registry Manager
const manager = new RegistryManager();

// Add multiple registries
manager.addRegistry('https://registry.npmjs.org', RegistryType.Npm);
manager.addRegistry('https://npm.pkg.github.com', RegistryType.GitHub);
manager.addRegistry('https://custom-registry.mycompany.com', RegistryType.Custom, 'company-client');

// Configure scopes
manager.associateScope('@myorg', 'https://npm.pkg.github.com');
manager.associateScope('@company', 'https://custom-registry.mycompany.com');

// Set authentication
manager.setAuth('https://npm.pkg.github.com', {
  token: process.env.GITHUB_TOKEN,
  tokenType: 'Bearer',
  always: true
});

manager.setAuth('https://custom-registry.mycompany.com', {
  token: process.env.COMPANY_NPM_TOKEN,
  tokenType: 'Bearer',
  always: true
});

// Usage - automatic routing to correct registry
const reactVersion = await manager.getLatestVersion('react');                 // Uses npmjs.org
const orgPackage = await manager.getLatestVersion('@myorg/components');       // Uses GitHub
const companyPackage = await manager.getLatestVersion('@company/api-client'); // Uses custom registry
`

  console.log(
    boxen(chalk.cyan(multiRegistryCode), {
      padding: 1,
      borderStyle: 'round',
      borderColor: 'blue',
      title: 'Multiple Registry Configuration',
      titleAlignment: 'center',
    }),
  )

  printSubHeader('Registry Authentication Methods', '🔐')

  const authTable = new Table({
    head: [
      chalk.bold.white('Registry'),
      chalk.bold.white('Auth Method'),
      chalk.bold.white('Token Source'),
      chalk.bold.white('Code Example'),
    ],
    colWidths: [15, 20, 20, 45],
  })

  authTable.push(
    [
      chalk.blue('NPM'),
      'Bearer Token',
      '~/.npmrc or NPM_TOKEN',
      chalk.green(`registry.setAuth({
    token: process.env.NPM_TOKEN,
    tokenType: 'Bearer',
    always: false
  })`),
    ],
    [
      chalk.blue('GitHub'),
      'Bearer Token',
      'GitHub PAT',
      chalk.green(`registry.setAuth({
    token: process.env.GITHUB_TOKEN,
    tokenType: 'Bearer',
    always: true
  })`),
    ],
    [
      chalk.blue('Azure DevOps'),
      'Basic Auth',
      'Personal Access Token',
      chalk.green(`registry.setAuth({
    token: 'username:' + process.env.AZURE_PAT,
    tokenType: 'Basic',
    always: true
  })`),
    ],
    [
      chalk.blue('Custom'),
      'Various',
      'Env or Config Files',
      chalk.green(`registry.setAuth({
    token: process.env.CUSTOM_TOKEN,
    tokenType: auth_type,
    always: true
  })`),
    ],
  )

  console.log(authTable.toString())

  printSubHeader('Registry Selection Logic', '🧠')

  const flowchart = `
  ┌─────────────┐
  │ Package Name│
  └──────┬──────┘
         │
         ▼
  ┌─────────────┐     Yes      ┌─────────────┐
  │  Is Scoped? ├──────────────►  Scope has  │
  └──────┬──────┘              │  Registry?  │
         │ No                  └──────┬──────┘
         │                            │ Yes
         │                            ▼
         │                     ┌─────────────┐
         │                     │ Use Scoped  │
         │                     │  Registry   │
         │                     └─────────────┘
         ▼                            │
  ┌─────────────┐                     │
  │ Use Default │                     │
  │  Registry   │◄────────────────────┘
  └─────────────┘
    `

  console.log(
    boxen(chalk.cyan(flowchart), {
      padding: 1,
      margin: 1,
      borderStyle: 'round',
      borderColor: 'cyan',
      title: 'Registry Selection Process',
      titleAlignment: 'center',
    }),
  )

  printSubHeader('Local Registry Testing', '🧪')

  // Create a local registry for testing
  const testRegistry = PackageRegistry.createLocalRegistry()

  // Add test packages
  testRegistry.addPackage('test-lib', ['1.0.0', '1.0.1', '1.1.0', '2.0.0'])
  testRegistry.addPackage('test-ui', ['0.9.0', '1.0.0', '1.5.0'])

  // Add dependencies
  testRegistry.setDependencies('test-lib', '2.0.0', {
    lodash: '^4.17.21',
    'test-ui': '^1.0.0',
  })

  testRegistry.setDependencies('test-ui', '1.5.0', {
    react: '^17.0.2',
    'styled-components': '^5.3.5',
  })

  // Test Operations
  const testOpsTable = new Table({
    head: [chalk.bold.white('Test Operation'), chalk.bold.white('Code'), chalk.bold.white('Result')],
    colWidths: [20, 35, 25],
  })

  // Get latest version
  const latestLib = testRegistry.getLatestVersion('test-lib')
  testOpsTable.push([
    chalk.blue('Get Latest Version'),
    chalk.yellow(`testRegistry.getLatestVersion('test-lib')`),
    chalk.green(latestLib),
  ])

  // Get all versions
  const allVersions = testRegistry.getAllVersions('test-lib')
  testOpsTable.push([
    chalk.blue('Get All Versions'),
    chalk.yellow(`testRegistry.getAllVersions('test-lib')`),
    chalk.green(allVersions.join(', ')),
  ])

  // Get all packages
  const allPackages = testRegistry.getAllPackages()
  testOpsTable.push([
    chalk.blue('Get All Packages'),
    chalk.yellow(`testRegistry.getAllPackages()`),
    chalk.green(allPackages.join(', ')),
  ])

  console.log(testOpsTable.toString())

  return { testRegistry }
}

const { testRegistry } = demoRegistryComparison()

// ===== Example 16: Using Registry Manager with Multiple Registries =====
printHeader('Multi-Registry Dependency Resolution')

const demoMultiRegistry = () => {
  printSubHeader('Setting Up Registry Environment', '🌍')

  // Create registry manager with multiple registries
  const regManager = new RegistryManager()

  // Create multiple registries
  const npmRegistry = PackageRegistry.createNpmRegistry('https://registry.npmjs.org')
  const localRegistry = PackageRegistry.createLocalRegistry()

  // Set up local registry with test data
  localRegistry.addPackage('org-lib', ['1.0.0', '1.1.0', '2.0.0'])
  localRegistry.setDependencies('org-lib', '2.0.0', {
    lodash: '^4.17.21',
  })

  // Add registries to manager
  regManager.addRegistry('https://registry.npmjs.org', RegistryType.Npm)
  regManager.addRegistry('https://local-registry', RegistryType.Custom, 'local-client')

  // Associate scopes
  regManager.associateScope('@org', 'https://local-registry')

  printSuccess('Registry manager configured with multiple registries and scope associations')

  printSubHeader('Registry Resolution Flow', '⚙️')

  const requestsTable = new Table({
    head: [
      chalk.bold.white('Package Request'),
      chalk.bold.white('Resolution Logic'),
      chalk.bold.white('Registry Used'),
    ],
    colWidths: [25, 35, 25],
  })

  requestsTable.push(
    [chalk.blue('react'), 'Unscoped, use default registry', chalk.green('https://registry.npmjs.org')],
    [chalk.blue('@org/lib'), 'Scoped @org, check scope associations', chalk.green('https://local-registry')],
    [chalk.blue('@unscoped/pkg'), 'Scoped but no association, use default', chalk.green('https://registry.npmjs.org')],
  )

  console.log(requestsTable.toString())

  printSubHeader('Practical Multi-Registry Usage', '🛠️')

  const scenarioTable = new Table({
    head: [chalk.bold.white('Scenario'), chalk.bold.white('Registry Configuration'), chalk.bold.white('Benefits')],
    colWidths: [20, 35, 35],
  })

  scenarioTable.push(
    [
      chalk.blue('Enterprise with\nPrivate Packages'),
      '- Public npm registry for open source\n- Private registry for @company scope\n- GitHub for @teams scope',
      '- Improved security for private code\n- Faster access to internal packages\n- Simplified authentication',
    ],
    [
      chalk.blue('Development with\nMocked Packages'),
      '- Public npm registry as default\n- Local registry for test packages\n- Associate test scopes with local',
      '- Work offline during development\n- Test with controlled versions\n- Simulate registry failures',
    ],
    [
      chalk.blue('Mirror/Proxy\nConfiguration'),
      '- Company proxy as default registry\n- Direct access for specific scopes\n- Authentication per registry',
      '- Reduce external bandwidth\n- Audit all package usage\n- Improved reliability',
    ],
  )

  console.log(scenarioTable.toString())

  printSubHeader('Multi-Registry Challenge: Dependency Resolution', '🧩')

  const depRegistry = new DependencyRegistry()

  // Create scenario where packages from different registries depend on each other
  printCode(`
  // Create packages referencing different registries
  const appPackage = Package.withRegistry(
    'my-app',
    '1.0.0',
    [
      ['react', '^17.0.2'],             // From npm registry
      ['@org/lib', '^2.0.0'],           // From local registry
      ['lodash', '^4.17.21']            // From npm registry
    ],
    depRegistry
  )`)

  const appPackage = Package.withRegistry(
    'my-app',
    '1.0.0',
    [
      ['react', '^17.0.2'],
      ['@org/lib', '^2.0.0'],
      ['lodash', '^4.17.21'],
    ],
    depRegistry,
  )

  printSuccess(`Created package that references dependencies from multiple registries`)

  printSubHeader('Registry Interaction Flow', '📊')

  const workflowDiagram = `
  ┌─────────────┐                ┌─────────────┐
  │ Application │                │  Registry   │
  │    Code     │                │   Manager   │
  └──────┬──────┘                └──────┬──────┘
         │                              │
         │  Import Dependency           │
         ├──────────────────────────────┤
         │                              │
         ▼                              ▼
  ┌─────────────┐                ┌─────────────┐
  │  Dependency │                │   Resolve   │
  │  Reference  │───────────────►│   Package   │
  └──────┬──────┘                └──────┬──────┘
         │                              │
         │                              │
         ▼                              ▼
  ┌─────────────┐                ┌─────────────┐
  │  Package    │◄───────────────┤ Check Scope │
  │  Resolution │                │ Association │
  └──────┬──────┘                └──────┬──────┘
         │                              │
         │                              │
         ▼                              ▼
  ┌─────────────┐                ┌─────────────┐
  │  Dependency │◄───────────────┤  Retrieve   │
  │   Loaded    │                │   Package   │
  └─────────────┘                └─────────────┘
  `

  console.log(
    boxen(chalk.cyan(workflowDiagram), {
      padding: 1,
      margin: 1,
      borderStyle: 'round',
      borderColor: 'cyan',
      title: 'Registry Resolution Flow',
      titleAlignment: 'center',
    }),
  )

  printSubHeader('Best Practices', '✅')

  const bestPracticesTable = new Table({
    head: [chalk.bold.white('Practice'), chalk.bold.white('Description'), chalk.bold.white('Implementation')],
    colWidths: [25, 35, 35],
  })

  bestPracticesTable.push(
    [
      chalk.blue('Use Scopes Consistently'),
      'Organize packages into logical scopes',
      'Associate each scope with appropriate registry',
    ],
    [
      chalk.blue('Cache Registry Results'),
      'Reduce redundant network requests',
      'Registry implementations automatically cache',
    ],
    [
      chalk.blue('Manage Auth Per Registry'),
      'Use different tokens for each registry',
      'setAuth() method with appropriate credentials',
    ],
    [
      chalk.blue('Support .npmrc Configuration'),
      'Honor user/system registry settings',
      'manager.loadFromNpmrc() at startup',
    ],
    [
      chalk.blue('Test with Local Registry'),
      'Use local registry for tests',
      'Replace remote registry with LocalRegistry',
    ],
  )

  console.log(bestPracticesTable.toString())

  return { regManager, appPackage }
}

const { regManager, appPackage } = demoMultiRegistry()

// Final completion message with a summary of registry features
console.log(
  '\n' +
    boxen(
      chalk.bold.white('Registry Features Summary:') +
        '\n\n' +
        chalk.blue('• PackageRegistry:') +
        ' Create and interact with individual package registries\n' +
        chalk.blue('• RegistryManager:') +
        ' Coordinate multiple registries with scope-based routing\n' +
        chalk.blue('• Authentication:') +
        ' Support for various auth methods across registries\n' +
        chalk.blue('• Local Registry:') +
        ' In-memory registry for testing and offline development\n' +
        chalk.blue('• Multi-Registry:') +
        ' Resolve packages across multiple distinct registries\n\n' +
        chalk.bold.green('Registry API bindings complete and demonstrated successfully!'),
      {
        padding: 1,
        margin: 1,
        borderStyle: 'double',
        borderColor: 'green',
        align: 'left',
        title: '🎉 Registry API Showcase Complete 🎉',
        titleAlignment: 'center',
      },
    ),
)

// ===== Example 17: Dependency Graphs =====
printHeader('Dependency Graphs')

const demoDependencyGraph = () => {
  printSubHeader('Exploring DependencyGraph Class', '📊')

  // Create packages with dependencies to build a graph
  printCode(`
// Create packages with dependencies
const pkg1 = new Package('app', '1.0.0')
const pkg2 = new Package('ui-lib', '2.0.0')
const pkg3 = new Package('utils', '1.5.0')
const pkg4 = new Package('config', '0.8.0')

// Add dependencies between packages
pkg1.addDependency(new Dependency('ui-lib', '^2.0.0'))
pkg1.addDependency(new Dependency('utils', '^1.0.0'))
pkg2.addDependency(new Dependency('utils', '^1.5.0'))
pkg3.addDependency(new Dependency('config', '^0.8.0'))

// Build the dependency graph
const graph = buildDependencyGraphFromPackages([pkg1, pkg2, pkg3, pkg4])
  `)

  // Create packages with dependencies
  const pkg1 = new Package('app', '1.0.0')
  const pkg2 = new Package('ui-lib', '2.0.0')
  const pkg3 = new Package('utils', '1.5.0')
  const pkg4 = new Package('config', '0.8.0')

  // Add dependencies between packages
  pkg1.addDependency(new Dependency('ui-lib', '^2.0.0'))
  pkg1.addDependency(new Dependency('utils', '^1.0.0'))
  pkg2.addDependency(new Dependency('utils', '^1.5.0'))
  pkg3.addDependency(new Dependency('config', '^0.8.0'))

  // Build graph from packages
  const graph = buildDependencyGraphFromPackages([pkg1, pkg2, pkg3, pkg4])

  printSuccess('DependencyGraph created successfully')
  printProperty('Graph Type', 'DependencyGraph')

  // Show that the result is a DependencyGraph instance
  printCode(`console.log(graph instanceof DependencyGraph) // true`)
  console.log(`Result is DependencyGraph instance: ${chalk.green(graph instanceof DependencyGraph ? 'Yes' : 'No')}`)

  printSubHeader('DependencyGraph Methods', '🧰')

  // Display methods available on DependencyGraph
  const methodsTable = new Table({
    head: [chalk.bold.white('Method'), chalk.bold.white('Purpose'), chalk.bold.white('Example')],
    colWidths: [25, 30, 35],
  })

  methodsTable.push(
    [
      chalk.blue('isInternallyResolvable()'),
      'Check if all dependencies can be resolved within the workspace',
      chalk.yellow(`graph.isInternallyResolvable()`),
    ],
    [
      chalk.blue('findMissingDependencies()'),
      "Find dependencies that aren't in the workspace",
      chalk.yellow(`graph.findMissingDependencies()`),
    ],
    [
      chalk.blue('findVersionConflicts()'),
      'Find version requirement conflicts',
      chalk.yellow(`graph.findVersionConflicts()`),
    ],
    [
      chalk.blue('detectCircularDependencies()'),
      'Find circular dependency paths',
      chalk.yellow(`graph.detectCircularDependencies()`),
    ],
    [chalk.blue('getNode(id)'), 'Get a package node by identifier', chalk.yellow(`graph.getNode("app")`)],
    [
      chalk.blue('getDependents(id)'),
      'Get packages that depend on a specific package',
      chalk.yellow(`graph.getDependents("utils")`),
    ],
    [
      chalk.blue('validatePackageDependencies()'),
      'Run full validation and get report',
      chalk.yellow(`graph.validatePackageDependencies()`),
    ],
  )

  console.log(methodsTable.toString())

  // Display the graph structure
  printSubHeader('Graph Structure', '🌳')

  const asciiGraph = generateAscii(graph)
  console.log(
    boxen(chalk.cyan(asciiGraph), {
      padding: 1,
      borderStyle: 'round',
      borderColor: 'blue',
      title: 'ASCII Graph Visualization',
      titleAlignment: 'center',
    }),
  )

  // Demo using the methods
  printSubHeader('Using DependencyGraph Methods', '💻')

  printCode(`// Check if internally resolvable
const resolvable = graph.isInternallyResolvable()
console.log("Internally Resolvable:", resolvable)

// Find missing dependencies
const missing = graph.findMissingDependencies()
console.log("Missing Dependencies:", missing)

// Get a node by ID
const utilsNode = graph.getNode("utils")
console.log("Utils Node:", utilsNode && utilsNode.name)

// Get dependents
const utilsDependents = graph.getDependents("utils")
console.log("Utils Dependents:", utilsDependents)`)

  console.log(chalk.gray('// Results:'))
  console.log(chalk.gray(`Internally Resolvable: ${graph.isInternallyResolvable()}`))
  console.log(chalk.gray(`Missing Dependencies: ${JSON.stringify(graph.findMissingDependencies())}`))

  const utilsNode = graph.getNode('utils')
  console.log(chalk.gray(`Utils Node: ${utilsNode ? utilsNode.name : 'Not found'}`))

  let utilsDependents
  try {
    utilsDependents = graph.getDependents('utils')
    console.log(chalk.gray(`Utils Dependents: ${JSON.stringify(utilsDependents)}`))
  } catch (e) {
    console.log(chalk.gray(`Error getting dependents: ${e.message}`))
  }

  return { graph, pkg1, pkg2, pkg3, pkg4 }
}

demoDependencyGraph()

// ===== Example 18: Dependency Graph Validation and Node =====
printHeader('ValidationReport and Node')

const demoValidationAndNode = () => {
  printSubHeader('Understanding the Node Interface', '🧩')

  // Explain the Node interface concept
  printCode(`// Node is a conceptual interface that defines what a node in the dependency graph looks like
// In the Rust implementation, Package implements the Node trait
// In JavaScript, we use the Package class directly

// Creating a new Node (actually a Package instance)
const nodeInstance = new Package('example-node', '1.0.0')
console.log(nodeInstance.name)  // 'example-node'

// The Node interface defines requirements like:
// - Having a unique identifier
// - Managing dependencies
// - Matching against dependency requirements`)

  // Demonstrate with a table
  const nodeInterfaceTable = new Table({
    head: [chalk.bold.white('Node Requirement'), chalk.bold.white('Implementation in Package')],
    colWidths: [30, 60],
  })

  nodeInterfaceTable.push(
    [chalk.blue('Unique Identifier'), 'Package.name property'],
    [chalk.blue('Dependencies List'), 'Package.dependencies() method'],
    [chalk.blue('Version Information'), 'Package.version property'],
    [chalk.blue('Dependency Matching'), 'Internal semver compatibility checking'],
  )

  console.log(nodeInterfaceTable.toString())

  printSubHeader('Creating a Graph with Issues', '⚠️')

  // Create packages with various issues
  const validationPkg1 = new Package('service-a', '1.0.0')
  const validationPkg2 = new Package('service-b', '1.0.0')
  const validationPkg3 = new Package('service-c', '1.0.0')
  const validationPkg4 = new Package('service-d', '1.0.0')

  // Create circular dependency: service-a -> service-b -> service-a
  validationPkg1.addDependency(new Dependency('service-b', '^1.0.0'))
  validationPkg2.addDependency(new Dependency('service-a', '^1.0.0'))

  // Create conflicting version requirements
  validationPkg1.addDependency(new Dependency('shared-lib', '^1.0.0'))
  validationPkg3.addDependency(new Dependency('shared-lib', '^2.0.0'))

  // Create dependency to a non-existent package
  validationPkg4.addDependency(new Dependency('missing-pkg', '^1.0.0'))

  // Build the problematic graph
  const validationGraph = buildDependencyGraphFromPackages([
    validationPkg1,
    validationPkg2,
    validationPkg3,
    validationPkg4,
  ])

  printSuccess('Created a dependency graph with various issues for validation')

  printSubHeader('ValidationReport Class', '📋')

  // Run validation on the graph
  printCode(`// Get a validation report from the graph
const validationReport = validationGraph.validatePackageDependencies()
console.log(validationReport instanceof ValidationReport) // true`)

  // Validate the graph
  let validationReport
  try {
    validationReport = validationGraph.validatePackageDependencies()

    // Show that the result is a ValidationReport instance
    console.log(
      `Result is ValidationReport instance: ${chalk.green(validationReport instanceof ValidationReport ? 'Yes' : 'No')}`,
    )

    // Print ValidationReport properties
    printSubHeader('ValidationReport Properties', '📊')

    const reportPropsTable = new Table({
      head: [chalk.bold.white('Property'), chalk.bold.white('Value'), chalk.bold.white('Description')],
      colWidths: [25, 15, 50],
    })

    reportPropsTable.push(
      [
        chalk.blue('hasIssues'),
        validationReport.hasIssues ? chalk.red('true') : chalk.green('false'),
        'Whether the graph has any validation issues',
      ],
      [
        chalk.blue('hasCriticalIssues'),
        validationReport.hasCriticalIssues ? chalk.red('true') : chalk.green('false'),
        'Whether the graph has critical issues that must be fixed',
      ],
      [
        chalk.blue('hasWarnings'),
        validationReport.hasWarnings ? chalk.yellow('true') : chalk.green('false'),
        'Whether the graph has non-critical warning issues',
      ],
    )

    console.log(reportPropsTable.toString())

    // Print ValidationReport methods
    printSubHeader('ValidationReport Methods', '🔍')

    const reportMethodsTable = new Table({
      head: [chalk.bold.white('Method'), chalk.bold.white('Purpose'), chalk.bold.white('Example')],
      colWidths: [25, 30, 35],
    })

    reportMethodsTable.push(
      [chalk.blue('getIssues()'), 'Get all validation issues', chalk.yellow(`validationReport.getIssues()`)],
      [
        chalk.blue('getCriticalIssues()'),
        'Get only critical issues',
        chalk.yellow(`validationReport.getCriticalIssues()`),
      ],
      [chalk.blue('getWarnings()'), 'Get only warning issues', chalk.yellow(`validationReport.getWarnings()`)],
    )

    console.log(reportMethodsTable.toString())
  } catch (e) {
    printError(`Error during validation: ${e.message}`)
    return { validationGraph }
  }

  printSubHeader('Validation Issue Types', '❌')

  // Show ValidationIssueType enum
  printCode(`// ValidationIssueType enum defines the types of validation issues
console.log(ValidationIssueType.CircularDependency)  // 0
console.log(ValidationIssueType.UnresolvedDependency)  // 1
console.log(ValidationIssueType.VersionConflict)  // 2`)

  // Create a table showing ValidationIssueType enum values
  const issueTypesTable = new Table({
    head: [chalk.bold.white('Issue Type'), chalk.bold.white('Value'), chalk.bold.white('Description')],
    colWidths: [30, 10, 50],
  })

  issueTypesTable.push(
    [
      chalk.red('ValidationIssueType.CircularDependency'),
      ValidationIssueType.CircularDependency,
      'Circular dependency between packages',
    ],
    [
      chalk.yellow('ValidationIssueType.UnresolvedDependency'),
      ValidationIssueType.UnresolvedDependency,
      'Dependency that cannot be resolved in the workspace',
    ],
    [
      chalk.blue('ValidationIssueType.VersionConflict'),
      ValidationIssueType.VersionConflict,
      'Conflicting version requirements for a package',
    ],
  )

  console.log(issueTypesTable.toString())

  // Get all issues from the report
  const issues = validationReport.getIssues()

  if (issues.length === 0) {
    printWarning('No issues found during validation (unexpected!)')
  } else {
    printSubHeader('ValidationReport Issues', '📄')

    const issuesTable = new Table({
      head: [chalk.bold.white('Issue Type'), chalk.bold.white('Critical'), chalk.bold.white('Details')],
      colWidths: [20, 12, 60],
    })

    for (const issue of issues) {
      let details = issue.message
      let issueTypeStr

      switch (issue.issueType) {
        case ValidationIssueType.CircularDependency:
          issueTypeStr = chalk.red('Circular')
          details = `Cycle path: ${issue.path ? issue.path.join(' → ') : 'N/A'}`
          break
        case ValidationIssueType.UnresolvedDependency:
          issueTypeStr = chalk.yellow('Unresolved')
          details = `Missing: ${issue.dependencyName || 'unknown'} (${issue.versionReq || 'any'})`
          break
        case ValidationIssueType.VersionConflict:
          issueTypeStr = chalk.blue('Version Conflict')
          details = `For: ${issue.dependencyName || 'unknown'}\nVersions: ${issue.conflictingVersions ? issue.conflictingVersions.join(', ') : 'unknown'}`
          break
        default:
          issueTypeStr = chalk.gray('Unknown')
      }

      issuesTable.push([issueTypeStr, issue.critical ? chalk.red('Yes') : chalk.green('No'), details])
    }

    console.log(issuesTable.toString())
  }

  // Demonstrate getting critical issues vs warnings
  printSubHeader('Critical Issues vs Warnings', '⚠️')

  printCode(`// Get only critical issues
const criticalIssues = validationReport.getCriticalIssues()
console.log(\`Found \${criticalIssues.length} critical issues\`)

// Get only warnings (non-critical issues)
const warnings = validationReport.getWarnings()
console.log(\`Found \${warnings.length} warnings\`)`)

  const criticalIssues = validationReport.getCriticalIssues()
  const warnings = validationReport.getWarnings()

  const issueStatsTable = new Table({
    head: [chalk.bold.white('Issue Category'), chalk.bold.white('Count'), chalk.bold.white('Action Required')],
    colWidths: [20, 10, 60],
  })

  issueStatsTable.push(
    [chalk.red('Critical Issues'), criticalIssues.length, 'Must be fixed before proceeding'],
    [chalk.yellow('Warnings'), warnings.length, "Should be reviewed but don't block progress"],
  )

  console.log(issueStatsTable.toString())

  return { validationGraph, validationReport }
}

const { validationGraph, validationReport } = demoValidationAndNode()

// ===== Example 19: DependencyFilter Usage =====
printHeader('DependencyFilter Enum')

const demoDependencyFilter = () => {
  printSubHeader('Understanding DependencyFilter', '🔍')

  // Create a table explaining DependencyFilter enum
  const filterTypesTable = new Table({
    head: [chalk.bold.white('Filter Type'), chalk.bold.white('Value'), chalk.bold.white('Description')],
    colWidths: [30, 10, 50],
  })

  filterTypesTable.push(
    [
      chalk.blue('DependencyFilter.ProductionOnly'),
      DependencyFilter.ProductionOnly,
      'Includes only dependencies from the "dependencies" section',
    ],
    [
      chalk.blue('DependencyFilter.WithDevelopment'),
      DependencyFilter.WithDevelopment,
      'Includes both "dependencies" and "devDependencies"',
    ],
    [
      chalk.blue('DependencyFilter.AllDependencies'),
      DependencyFilter.AllDependencies,
      'Includes all dependency types including optional',
    ],
  )

  console.log(filterTypesTable.toString())

  printSubHeader('Using DependencyFilter', '⚙️')

  // Show example usage of DependencyFilter
  printCode(`// DependencyFilter is used when building dependency graphs
// from package.json files to control which dependencies are included

// In a real application, you would use this in options:
const options = {
  dependencyFilter: DependencyFilter.WithDevelopment
}

// Default is DependencyFilter.WithDevelopment if not specified

// Production-only example (conceptual)
scanWorkspace({
  dependencyFilter: DependencyFilter.ProductionOnly
})

// Include all dependencies example (conceptual)
scanWorkspace({
  dependencyFilter: DependencyFilter.AllDependencies
})`)

  printSubHeader('Filter Impact on Graph Size', '📏')

  // Show impact of different filters on graph size
  const impactTable = new Table({
    head: [chalk.bold.white('Filter'), chalk.bold.white('Dependencies Included'), chalk.bold.white('Typical Impact')],
    colWidths: [25, 40, 25],
  })

  impactTable.push(
    [chalk.blue('ProductionOnly'), '- Regular dependencies', chalk.green('Smaller graph\nFewer edges')],
    [
      chalk.blue('WithDevelopment'),
      '- Regular dependencies\n- Dev dependencies',
      chalk.yellow('Medium graph\nModerate edges'),
    ],
    [
      chalk.blue('AllDependencies'),
      '- Regular dependencies\n- Dev dependencies\n- Optional dependencies\n- Peer dependencies',
      chalk.red('Larger graph\nMany edges'),
    ],
  )

  console.log(impactTable.toString())

  printSubHeader('Practical Application', '🛠️')

  // Create a conceptual example showing how each filter might be used
  const useCaseTable = new Table({
    head: [chalk.bold.white('Filter'), chalk.bold.white('Use Case')],
    colWidths: [25, 65],
  })

  useCaseTable.push(
    [
      chalk.blue('ProductionOnly'),
      '- Analyzing production deployment size\n- Identifying runtime dependencies\n- Optimizing for production',
    ],
    [
      chalk.blue('WithDevelopment'),
      '- Normal development workflows\n- CI/CD pipeline analysis\n- Standard dependency validation',
    ],
    [
      chalk.blue('AllDependencies'),
      '- Complete dependency audits\n- Security vulnerability scanning\n- License compliance checks',
    ],
  )

  console.log(useCaseTable.toString())

  printSubHeader('Filter Selection Logic', '🧠')

  // Explain how to select the appropriate filter
  const selectionLogic = `
// Decision Tree for Selecting DependencyFilter

if (analyzing_for_production_deployment) {
  // Focus only on what will be deployed
  return DependencyFilter.ProductionOnly;
}

if (doing_standard_development_work) {
  // Include dev tools and testing frameworks
  return DependencyFilter.WithDevelopment;
}

if (performing_security_audit || license_check) {
  // Include absolutely everything
  return DependencyFilter.AllDependencies;
}

// Default case for most scenarios
return DependencyFilter.WithDevelopment;
`

  console.log(
    boxen(chalk.cyan(selectionLogic), {
      padding: 1,
      borderStyle: 'round',
      borderColor: 'blue',
      title: 'Filter Selection Logic',
      titleAlignment: 'center',
    }),
  )

  // Include an example of code that would use different filters
  return {
    filterTypes: [DependencyFilter.ProductionOnly, DependencyFilter.WithDevelopment, DependencyFilter.AllDependencies],
  }
}

demoDependencyFilter()

// ===== Example 21: Integrated Monorepo Management =====
printHeader('Integrated Monorepo Management')

const demoIntegratedMonorepo = () => {
  printSubHeader('Monorepo Management Scenario', '🏗️')

  // Set up the scenario
  console.log(
    boxen(
      chalk.white(`
In this comprehensive example, we'll simulate managing a monorepo with multiple packages.
We'll perform the following operations:
- Set up a registry environment
- Create packages and define dependencies
- Build and validate the dependency graph
- Detect and resolve dependency issues
- Generate visualizations
- Plan package updates based on analysis
  `),
      {
        padding: 1,
        borderStyle: 'round',
        borderColor: 'blue',
        title: 'Scenario Overview',
        titleAlignment: 'center',
      },
    ),
  )

  printSubHeader('Setting Up Registry Environment', '📚')

  // Initialize registry manager and local registry for testing
  printCode(`
// Set up registry manager and local registry
const registryManager = new RegistryManager()
const localRegistry = PackageRegistry.createLocalRegistry()

// Add packages to the local registry
localRegistry.addPackage('shared-ui', ['1.0.0', '1.5.0', '2.0.0'])
localRegistry.addPackage('shared-utils', ['1.0.0', '1.1.0'])
localRegistry.addPackage('api-client', ['0.9.0', '1.0.0'])
localRegistry.addPackage('logger', ['1.0.0'])
localRegistry.addPackage('config', ['0.5.0', '1.0.0'])
localRegistry.addPackage('react', ['16.8.0', '17.0.2', '18.0.0'])
localRegistry.addPackage('lodash', ['4.17.21'])

// Set up package dependencies in the registry
localRegistry.setDependencies('shared-ui', '2.0.0', {
  'react': '^17.0.2',
  'lodash': '^4.17.21'
})

localRegistry.setDependencies('shared-utils', '1.1.0', {
  'lodash': '^4.17.21'
})

localRegistry.setDependencies('api-client', '1.0.0', {
  'shared-utils': '^1.0.0',
  'logger': '^1.0.0'
})

// Add registry to manager and set as default
registryManager.addRegistry('https://local-registry', RegistryType.Custom, 'local-client')
registryManager.setDefaultRegistry('https://local-registry')
  `)

  // Initialize registry manager and local registry for testing
  const registryManager = new RegistryManager()
  const localRegistry = PackageRegistry.createLocalRegistry()

  // Add packages to the local registry
  localRegistry.addPackage('shared-ui', ['1.0.0', '1.5.0', '2.0.0'])
  localRegistry.addPackage('shared-utils', ['1.0.0', '1.1.0'])
  localRegistry.addPackage('api-client', ['0.9.0', '1.0.0'])
  localRegistry.addPackage('logger', ['1.0.0'])
  localRegistry.addPackage('config', ['0.5.0', '1.0.0'])
  localRegistry.addPackage('react', ['16.8.0', '17.0.2', '18.0.0'])
  localRegistry.addPackage('lodash', ['4.17.21'])

  // Set up package dependencies in the registry
  localRegistry.setDependencies('shared-ui', '2.0.0', {
    react: '^17.0.2',
    lodash: '^4.17.21',
  })

  localRegistry.setDependencies('shared-utils', '1.1.0', {
    lodash: '^4.17.21',
  })

  localRegistry.setDependencies('api-client', '1.0.0', {
    'shared-utils': '^1.0.0',
    logger: '^1.0.0',
  })

  // Add registry to manager and set as default
  registryManager.addRegistry('https://local-registry', RegistryType.Custom, 'local-client')
  registryManager.setDefaultRegistry('https://local-registry')

  // Show available packages in registry
  const packagesTable = new Table({
    head: [chalk.bold.white('Package'), chalk.bold.white('Available Versions')],
    colWidths: [20, 70],
  })

  const allPackages = localRegistry.getAllPackages()
  for (const pkg of allPackages) {
    const versions = localRegistry.getAllVersions(pkg)
    packagesTable.push([chalk.blue(pkg), chalk.green(versions.join(', '))])
  }

  console.log(packagesTable.toString())
  printSuccess('Registry environment set up successfully')

  printSubHeader('Creating Workspace Packages', '📦')

  // Create dependency registry for our workspace
  printCode(`
// Create dependency registry for our workspace
const depRegistry = new DependencyRegistry()

// Create packages for our monorepo workspace with appropriate dependencies
const mainApp = Package.withRegistry(
  'main-app',
  '1.0.0',
  [
    ['shared-ui', '^2.0.0'],
    ['api-client', '^1.0.0'],
    ['config', '^0.5.0']  // Note: Using older version
  ],
  depRegistry
)

const adminDashboard = Package.withRegistry(
  'admin-dashboard',
  '0.9.0',
  [
    ['shared-ui', '^1.5.0'],  // Note: Different version from main-app
    ['shared-utils', '^1.0.0'],
    ['config', '^1.0.0']  // Note: Newer version than main-app
  ],
  depRegistry
)

const landingPage = Package.withRegistry(
  'landing-page',
  '1.2.0',
  [
    ['shared-ui', '^2.0.0'],
    ['logger', '^1.0.0']
  ],
  depRegistry
)

// Package with circular dependency
const analytics = Package.withRegistry(
  'analytics',
  '0.5.0',
  [
    ['reports', '^0.1.0']  // Will create circular dependency
  ],
  depRegistry
)

// Second part of circular dependency
const reports = Package.withRegistry(
  'reports',
  '0.1.0',
  [
    ['analytics', '^0.5.0']  // Circular dependency with analytics
  ],
  depRegistry
)

// Create workspace packages array
const workspacePackages = [mainApp, adminDashboard, landingPage, analytics, reports]
  `)

  // Create dependency registry for our workspace
  const depRegistry = new DependencyRegistry()

  // Create packages for our monorepo workspace with appropriate dependencies
  const mainApp = Package.withRegistry(
    'main-app',
    '1.0.0',
    [
      ['shared-ui', '^2.0.0'],
      ['api-client', '^1.0.0'],
      ['config', '^0.5.0'], // Note: Using older version
    ],
    depRegistry,
  )

  const adminDashboard = Package.withRegistry(
    'admin-dashboard',
    '0.9.0',
    [
      ['shared-ui', '^1.5.0'], // Note: Different version from main-app
      ['shared-utils', '^1.0.0'],
      ['config', '^1.0.0'], // Note: Newer version than main-app
    ],
    depRegistry,
  )

  const landingPage = Package.withRegistry(
    'landing-page',
    '1.2.0',
    [
      ['shared-ui', '^2.0.0'],
      ['logger', '^1.0.0'],
    ],
    depRegistry,
  )

  // Package with circular dependency
  const analytics = Package.withRegistry(
    'analytics',
    '0.5.0',
    [
      ['reports', '^0.1.0'], // Will create circular dependency
    ],
    depRegistry,
  )

  // Second part of circular dependency
  const reports = Package.withRegistry(
    'reports',
    '0.1.0',
    [
      ['analytics', '^0.5.0'], // Circular dependency with analytics
    ],
    depRegistry,
  )

  // Create workspace packages array
  const workspacePackages = [mainApp, adminDashboard, landingPage, analytics, reports]

  // Display created packages
  const workspaceTable = new Table({
    head: [chalk.bold.white('Package'), chalk.bold.white('Version'), chalk.bold.white('Dependencies')],
    colWidths: [20, 12, 60],
  })

  for (const pkg of workspacePackages) {
    const deps = pkg
      .dependencies()
      .map((d) => `${d.name}@${d.version}`)
      .join(', ')
    workspaceTable.push([chalk.blue(pkg.name), chalk.green(pkg.version), chalk.yellow(deps)])
  }

  console.log(workspaceTable.toString())
  printSuccess('Workspace packages created successfully')

  printSubHeader('Building Dependency Graph', '🔍')

  // Build and analyze dependency graph
  printCode(`
// Build dependency graph from workspace packages
const dependencyGraph = buildDependencyGraphFromPackages(workspacePackages)

// Check if graph is internally resolvable
const resolvable = dependencyGraph.isInternallyResolvable()
console.log(\`Graph is internally resolvable: \${resolvable}\`)

// Find missing dependencies
const missingDeps = dependencyGraph.findMissingDependencies()
console.log(\`Missing dependencies: \${missingDeps.length > 0 ? missingDeps.join(', ') : 'None'}\`)

// Check for circular dependencies
const circularDeps = dependencyGraph.detectCircularDependencies()
console.log(\`Circular dependencies: \${circularDeps ? 'Yes' : 'No'}\`)
  `)

  // Build dependency graph from workspace packages
  const dependencyGraph = buildDependencyGraphFromPackages(workspacePackages)

  // Check if graph is internally resolvable
  const resolvable = dependencyGraph.isInternallyResolvable()

  // Find missing dependencies
  const missingDeps = dependencyGraph.findMissingDependencies()

  // Check for circular dependencies
  const circularDeps = dependencyGraph.detectCircularDependencies()

  // Display results in a table
  const graphAnalysisTable = new Table({
    head: [chalk.bold.white('Analysis'), chalk.bold.white('Result'), chalk.bold.white('Details')],
    colWidths: [20, 15, 55],
  })

  graphAnalysisTable.push(
    [
      chalk.blue('Internally Resolvable'),
      resolvable ? chalk.green('Yes') : chalk.red('No'),
      'Some dependencies may not be available in the workspace',
    ],
    [
      chalk.blue('Missing Dependencies'),
      missingDeps.length > 0 ? chalk.yellow(missingDeps.length.toString()) : chalk.green('None'),
      missingDeps.length > 0 ? chalk.yellow(missingDeps.join(', ')) : 'All dependencies found in workspace',
    ],
    [
      chalk.blue('Circular Dependencies'),
      circularDeps ? chalk.red('Yes') : chalk.green('No'),
      circularDeps ? chalk.red(circularDeps.join(' → ')) : 'No circular dependencies detected',
    ],
  )

  console.log(graphAnalysisTable.toString())

  printSubHeader('Visualizing Dependency Graph', '📊')

  // Generate graph visualizations
  printCode(`
// Generate ASCII visualization of the graph
const asciiGraph = generateAscii(dependencyGraph)
console.log(asciiGraph)

// Generate DOT graph for more detailed visualization
const dotOptions = {
  title: "Monorepo Dependency Graph",
  showExternal: true,
  highlightCycles: true
}
const dotGraph = generateDot(dependencyGraph, dotOptions)

// Save DOT graph to file
saveDotToFile(dotGraph, "./monorepo-dependencies.dot")
  `)

  // Generate ASCII visualization
  const asciiGraph = generateAscii(dependencyGraph)

  console.log(
    boxen(chalk.cyan(asciiGraph), {
      padding: 1,
      borderStyle: 'round',
      borderColor: 'blue',
      title: 'ASCII Dependency Graph',
      titleAlignment: 'center',
    }),
  )

  // Generate DOT graph
  const dotOptions = {
    title: 'Monorepo Dependency Graph',
    showExternal: true,
    highlightCycles: true,
  }

  try {
    const dotGraph = generateDot(dependencyGraph, dotOptions)
    printSuccess('DOT graph generated successfully')
    printProperty('DOT Output Size', `${dotGraph.length} characters`)

    // Show the first few lines
    const previewLines = dotGraph.split('\n').slice(0, 8).join('\n') + '\n...'
    console.log(
      boxen(chalk.gray(previewLines), {
        padding: 1,
        borderStyle: 'round',
        borderColor: 'gray',
        title: 'DOT Graph Preview',
        titleAlignment: 'center',
      }),
    )
  } catch (e) {
    printError(`Error generating DOT graph: ${e.message}`)
  }

  printSubHeader('Validating Dependencies', '✅')

  // Validate the dependency graph
  printCode(`
// Run complete validation on dependency graph
const validationReport = dependencyGraph.validatePackageDependencies()

// Check if there are any issues
if (validationReport.hasIssues) {
  console.log("Validation found issues in the dependency graph:")

  // Get all issues
  const allIssues = validationReport.getIssues()

  // Get critical issues
  const criticalIssues = validationReport.getCriticalIssues()

  // Get warnings
  const warnings = validationReport.getWarnings()

  console.log(\`Found \${criticalIssues.length} critical issues and \${warnings.length} warnings\`)

  // Display issues by type
  for (const issue of allIssues) {
    // Handle each issue based on type
    switch (issue.issueType) {
      case ValidationIssueType.CircularDependency:
        console.log(\`Circular dependency: \${issue.path.join(' → ')}\`)
        break
      case ValidationIssueType.UnresolvedDependency:
        console.log(\`Unresolved dependency: \${issue.dependencyName} \${issue.versionReq}\`)
        break
      case ValidationIssueType.VersionConflict:
        console.log(\`Version conflict: \${issue.dependencyName} - \${issue.conflictingVersions.join(', ')}\`)
        break
    }
  }
}
  `)

  // Run validation
  let validationReport
  try {
    validationReport = dependencyGraph.validatePackageDependencies()

    // Display validation summary
    const validationSummaryTable = new Table({
      head: [chalk.bold.white('Validation Status'), chalk.bold.white('Count'), chalk.bold.white('Status')],
      colWidths: [30, 10, 50],
    })

    const criticalIssues = validationReport.getCriticalIssues()
    const warnings = validationReport.getWarnings()

    validationSummaryTable.push(
      [
        chalk.blue('Total Issues'),
        validationReport.getIssues().length,
        validationReport.hasIssues ? chalk.yellow('⚠️ Issues Found') : chalk.green('✓ No Issues'),
      ],
      [
        chalk.red('Critical Issues'),
        criticalIssues.length,
        criticalIssues.length > 0 ? chalk.red('❌ Must Fix') : chalk.green('✓ None'),
      ],
      [
        chalk.yellow('Warnings'),
        warnings.length,
        warnings.length > 0 ? chalk.yellow('⚠️ Review') : chalk.green('✓ None'),
      ],
    )

    console.log(validationSummaryTable.toString())

    // If there are issues, display them in detail
    if (validationReport.hasIssues) {
      printSubHeader('Validation Issues Detail', '❌')

      const issuesTable = new Table({
        head: [chalk.bold.white('Type'), chalk.bold.white('Critical'), chalk.bold.white('Details')],
        colWidths: [15, 12, 65],
      })

      for (const issue of validationReport.getIssues()) {
        let issueDetails
        let issueType

        switch (issue.issueType) {
          case ValidationIssueType.CircularDependency:
            issueType = chalk.red('Circular')
            issueDetails = `Path: ${issue.path ? issue.path.join(' → ') : 'unknown'}`
            break
          case ValidationIssueType.UnresolvedDependency:
            issueType = chalk.yellow('Unresolved')
            issueDetails = `Missing: ${issue.dependencyName}@${issue.versionReq}`
            break
          case ValidationIssueType.VersionConflict:
            issueType = chalk.blue('Version')
            issueDetails = `Package: ${issue.dependencyName}\nVersions: ${issue.conflictingVersions ? issue.conflictingVersions.join(', ') : 'unknown'}`
            break
          default:
            issueType = chalk.gray('Unknown')
            issueDetails = issue.message
        }

        issuesTable.push([issueType, issue.critical ? chalk.red('Yes') : chalk.green('No'), chalk.white(issueDetails)])
      }

      console.log(issuesTable.toString())
    }
  } catch (e) {
    printError(`Error validating dependency graph: ${e.message}`)
  }

  printSubHeader('Resolving Dependency Conflicts', '🔧')

  // Resolve dependency conflicts
  printCode(`
// Resolve version conflicts through dependency registry
const resolutionResult = depRegistry.resolveVersionConflicts()

// Display resolved versions
console.log("Resolved dependency versions:")
for (const [dep, version] of Object.entries(resolutionResult.resolvedVersions)) {
  console.log(\`- \${dep}: \${version}\`)
}

// Apply resolution to all packages
console.log("\\nApplying updates to packages:")
for (const pkg of workspacePackages) {
  const updates = pkg.updateDependenciesFromResolution(resolutionResult)
  for (const [name, oldVersion, newVersion] of updates) {
    console.log(\`- \${pkg.name}: \${name} \${oldVersion} → \${newVersion}\`)
  }
}

// Apply resolution result to dependency registry
depRegistry.applyResolutionResult(resolutionResult)

// Re-validate the graph after resolution
const updatedGraph = buildDependencyGraphFromPackages(workspacePackages)
const updatedReport = updatedGraph.validatePackageDependencies()

console.log(\`\\nAfter resolution:\\n- Critical issues: \${updatedReport.getCriticalIssues().length}\`)
console.log(\`- Version conflicts: \${updatedReport.getWarnings().filter(i => i.issueType === ValidationIssueType.VersionConflict).length}\`)
  `)

  // Resolve version conflicts
  const resolutionResult = depRegistry.resolveVersionConflicts()

  // Display resolved versions
  const resolvedVersionsTable = new Table({
    head: [chalk.bold.white('Dependency'), chalk.bold.white('Resolved Version')],
    colWidths: [30, 30],
  })

  for (const [dep, version] of Object.entries(resolutionResult.resolvedVersions)) {
    resolvedVersionsTable.push([chalk.blue(dep), chalk.green(version)])
  }

  console.log(resolvedVersionsTable.toString())

  // Apply resolution to all packages
  const updatesTable = new Table({
    head: [
      chalk.bold.white('Package'),
      chalk.bold.white('Dependency'),
      chalk.bold.white('From'),
      chalk.bold.white('To'),
    ],
    colWidths: [20, 20, 20, 20],
  })

  for (const pkg of workspacePackages) {
    const updates = pkg.updateDependenciesFromResolution(resolutionResult)
    for (const [name, oldVersion, newVersion] of updates) {
      updatesTable.push([chalk.blue(pkg.name), chalk.yellow(name), chalk.red(oldVersion), chalk.green(newVersion)])
    }
  }

  if (updatesTable.length > 0) {
    console.log(updatesTable.toString())
  } else {
    printWarning('No dependency updates required')
  }

  // Apply resolution result to dependency registry
  depRegistry.applyResolutionResult(resolutionResult)

  // Re-validate the graph after resolution
  const updatedGraph = buildDependencyGraphFromPackages(workspacePackages)
  let updatedReport

  try {
    updatedReport = updatedGraph.validatePackageDependencies()

    const afterResolutionTable = new Table({
      head: [
        chalk.bold.white('Issue Type'),
        chalk.bold.white('Before'),
        chalk.bold.white('After'),
        chalk.bold.white('Status'),
      ],
      colWidths: [25, 15, 15, 35],
    })

    const versionConflictsBefore = validationReport
      .getWarnings()
      .filter((i) => i.issueType === ValidationIssueType.VersionConflict).length
    const versionConflictsAfter = updatedReport
      .getWarnings()
      .filter((i) => i.issueType === ValidationIssueType.VersionConflict).length

    const circularBefore = validationReport
      .getCriticalIssues()
      .filter((i) => i.issueType === ValidationIssueType.CircularDependency).length
    const circularAfter = updatedReport
      .getCriticalIssues()
      .filter((i) => i.issueType === ValidationIssueType.CircularDependency).length

    const unresolvedBefore = validationReport
      .getCriticalIssues()
      .filter((i) => i.issueType === ValidationIssueType.UnresolvedDependency).length
    const unresolvedAfter = updatedReport
      .getCriticalIssues()
      .filter((i) => i.issueType === ValidationIssueType.UnresolvedDependency).length

    afterResolutionTable.push(
      [
        chalk.yellow('Version Conflicts'),
        versionConflictsBefore,
        versionConflictsAfter,
        versionConflictsBefore > versionConflictsAfter
          ? chalk.green('✓ Improved')
          : versionConflictsBefore === versionConflictsAfter
            ? chalk.yellow('⚠️ No Change')
            : chalk.red('❌ Worse'),
      ],
      [
        chalk.red('Circular Dependencies'),
        circularBefore,
        circularAfter,
        circularBefore > circularAfter
          ? chalk.green('✓ Improved')
          : circularBefore === circularAfter
            ? chalk.yellow('⚠️ No Change')
            : chalk.red('❌ Worse'),
      ],
      [
        chalk.blue('Unresolved Dependencies'),
        unresolvedBefore,
        unresolvedAfter,
        unresolvedBefore > unresolvedAfter
          ? chalk.green('✓ Improved')
          : unresolvedBefore === unresolvedAfter
            ? chalk.yellow('⚠️ No Change')
            : chalk.red('❌ Worse'),
      ],
    )

    console.log(afterResolutionTable.toString())
  } catch (e) {
    printError(`Error validating updated dependency graph: ${e.message}`)
  }

  printSubHeader('Action Plan Based on Analysis', '📝')

  // Generate action plan based on validation results
  const actionPlanTable = new Table({
    head: [chalk.bold.white('Issue'), chalk.bold.white('Action'), chalk.bold.white('Priority')],
    colWidths: [25, 55, 12],
  })

  if (updatedReport) {
    // Check for remaining circular dependencies
    const circularIssues = updatedReport
      .getCriticalIssues()
      .filter((i) => i.issueType === ValidationIssueType.CircularDependency)
    if (circularIssues.length > 0) {
      for (const issue of circularIssues) {
        actionPlanTable.push([
          chalk.red('Circular Dependency'),
          `Refactor ${issue.path ? issue.path.join(' and ') : 'affected packages'} to break the cycle`,
          chalk.red('HIGH'),
        ])
      }
    }

    // Check for remaining unresolved dependencies
    const unresolvedIssues = updatedReport
      .getCriticalIssues()
      .filter((i) => i.issueType === ValidationIssueType.UnresolvedDependency)
    if (unresolvedIssues.length > 0) {
      for (const issue of unresolvedIssues) {
        actionPlanTable.push([
          chalk.yellow('Unresolved Dependency'),
          `Add ${issue.dependencyName}@${issue.versionReq} to workspace or update reference`,
          chalk.red('HIGH'),
        ])
      }
    }

    // Check for remaining version conflicts
    const versionIssues = updatedReport.getWarnings().filter((i) => i.issueType === ValidationIssueType.VersionConflict)
    if (versionIssues.length > 0) {
      for (const issue of versionIssues) {
        actionPlanTable.push([
          chalk.blue('Version Conflict'),
          `Standardize ${issue.dependencyName} to single version across packages`,
          chalk.yellow('MEDIUM'),
        ])
      }
    }
  }

  // Add generic recommendations if no specific issues
  if (actionPlanTable.length === 0) {
    actionPlanTable.push(
      [chalk.green('No Critical Issues'), 'Proceed with development', chalk.green('LOW')],
      [chalk.blue('Maintenance'), 'Keep dependencies up to date with regular audits', chalk.yellow('MEDIUM')],
    )
  }

  console.log(actionPlanTable.toString())

  printSubHeader('Simulating Package Upgrade', '🚀')

  // Simulate updating a package version
  printCode(`
// Simulate upgrading shared-ui to v3.0.0
// First, update the registry with new version
localRegistry.addPackage('shared-ui', ['3.0.0'])
localRegistry.setDependencies('shared-ui', '3.0.0', {
  'react': '^18.0.0',  // Note: Now requires React 18
  'lodash': '^4.17.21'
})

// Create packages with old and new versions for diff
const sharedUiOld = new Package('shared-ui', '2.0.0')
sharedUiOld.addDependency(new Dependency('react', '^17.0.2'))
sharedUiOld.addDependency(new Dependency('lodash', '^4.17.21'))

const sharedUiNew = new Package('shared-ui', '3.0.0')
sharedUiNew.addDependency(new Dependency('react', '^18.0.0'))
sharedUiNew.addDependency(new Dependency('lodash', '^4.17.21'))

// Generate diff between versions
const packageDiff = PackageDiff.between(sharedUiOld, sharedUiNew)

// Analyze impact of the upgrade
console.log(\`Upgrading shared-ui from \${packageDiff.previousVersion} to \${packageDiff.currentVersion}\`)
console.log(\`Is breaking change? \${packageDiff.breakingChange ? 'Yes' : 'No'}\`)

const changes = packageDiff.dependencyChanges
console.log(\`Found \${changes.length} dependency changes\`)
for (const change of changes) {
  console.log(\`- \${change.name}: \${change.previousVersion || 'none'} → \${change.currentVersion || 'none'} (\${changeTypeToString(change.changeType)})\`)
}

// Analyze impact on consuming packages
console.log("\\nImpact on consuming packages:")
let affectedPackages = 0
for (const pkg of workspacePackages) {
  const uiDep = pkg.getDependency('shared-ui')
  if (uiDep) {
    console.log(\`- \${pkg.name} requires shared-ui@\${uiDep.version}\`)
    affectedPackages++
  }
}
console.log(\`Total affected packages: \${affectedPackages}\`)
  `)

  // Simulate upgrading shared-ui to v3.0.0
  // First, update the registry with new version
  localRegistry.addPackage('shared-ui', ['3.0.0'])
  localRegistry.setDependencies('shared-ui', '3.0.0', {
    react: '^18.0.0', // Note: Now requires React 18
    lodash: '^4.17.21',
  })

  // Create packages with old and new versions for diff
  const sharedUiOld = new Package('shared-ui', '2.0.0')
  sharedUiOld.addDependency(new Dependency('react', '^17.0.2'))
  sharedUiOld.addDependency(new Dependency('lodash', '^4.17.21'))

  const sharedUiNew = new Package('shared-ui', '3.0.0')
  sharedUiNew.addDependency(new Dependency('react', '^18.0.0'))
  sharedUiNew.addDependency(new Dependency('lodash', '^4.17.21'))

  // Generate diff between versions
  let packageDiff
  try {
    packageDiff = PackageDiff.between(sharedUiOld, sharedUiNew)

    // Display diff info
    const diffTable = new Table({
      head: [chalk.bold.white('Property'), chalk.bold.white('Value')],
      colWidths: [30, 60],
    })

    diffTable.push(
      [chalk.blue('Package'), chalk.green(packageDiff.packageName)],
      [chalk.blue('Previous Version'), chalk.yellow(packageDiff.previousVersion)],
      [chalk.blue('New Version'), chalk.yellow(packageDiff.currentVersion)],
      [chalk.blue('Breaking Change'), packageDiff.breakingChange ? chalk.red('Yes') : chalk.green('No')],
      [chalk.blue('Total Changes'), packageDiff.dependencyChanges.length.toString()],
    )

    console.log(diffTable.toString())

    // If there are changes, show details
    if (packageDiff.dependencyChanges.length > 0) {
      const changesTable = new Table({
        head: [
          chalk.bold.white('Dependency'),
          chalk.bold.white('Change Type'),
          chalk.bold.white('From'),
          chalk.bold.white('To'),
          chalk.bold.white('Breaking'),
        ],
        colWidths: [15, 15, 15, 15, 10],
      })

      for (const change of packageDiff.dependencyChanges) {
        let changeTypeStr = changeTypeToString(change.changeType)
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
          change.currentVersion ? chalk.green(change.currentVersion) : chalk.gray('none'),
          change.breaking ? chalk.red('Yes') : chalk.green('No'),
        ])
      }

      console.log(changesTable.toString())
    }

    // Analyze impact on consuming packages
    const affectedTable = new Table({
      head: [
        chalk.bold.white('Affected Package'),
        chalk.bold.white('Current Requirement'),
        chalk.bold.white('Compatible?'),
      ],
      colWidths: [20, 25, 15],
    })

    let affectedCount = 0

    for (const pkg of workspacePackages) {
      const uiDep = pkg.getDependency('shared-ui')
      if (uiDep) {
        affectedCount++

        // Determine if the current requirement is compatible with the new version
        const requirement = uiDep.version
        const isCompatible =
          requirement.startsWith('^') && !requirement.startsWith('^1.') && !packageDiff.breakingChange

        affectedTable.push([
          chalk.blue(pkg.name),
          chalk.yellow(requirement),
          isCompatible ? chalk.green('Yes') : chalk.red('No'),
        ])
      }
    }

    if (affectedCount > 0) {
      console.log(affectedTable.toString())
      printProperty('Total Affected Packages', affectedCount)
    } else {
      printWarning('No packages in the workspace depend on shared-ui')
    }
  } catch (e) {
    printError(`Error generating package diff: ${e.message}`)
  }

  printSubHeader('Final Recommendations', '📋')

  // Generate final recommendations
  const recommendationsTable = new Table({
    head: [chalk.bold.white('Category'), chalk.bold.white('Recommendation')],
    colWidths: [20, 70],
  })

  // Add custom recommendations based on our analysis
  recommendationsTable.push(
    [
      chalk.red('Critical Issues'),
      '1. Fix circular dependency between analytics and reports packages\n' +
        '2. Consider extracting shared functionality to break the cycle',
    ],
    [
      chalk.yellow('Version Conflicts'),
      '1. Standardize shared-ui version across all packages\n' + '2. Upgrade config package to v1.0.0 in main-app',
    ],
    [
      chalk.blue('Upgrade Strategy'),
      '1. Upgrade React to v18 before updating to shared-ui v3.0.0\n' +
        '2. Plan for breaking changes when upgrading shared-ui',
    ],
    [
      chalk.green('Graph Management'),
      '1. Run dependency validation regularly as part of CI\n' +
        '2. Generate graph visualizations for team documentation\n' +
        '3. Use Version, DependencyGraph and ValidationReport APIs to automate dependency management',
    ],
  )

  console.log(recommendationsTable.toString())

  return {
    registryManager,
    localRegistry,
    dependencyGraph,
    workspacePackages,
    validationReport,
  }
}

// Run the integrated demo
demoIntegratedMonorepo()

// Final completion message with a summary of integrated functionality
console.log(
  '\n' +
    boxen(
      chalk.bold.white('Integrated Monorepo Management Summary:') +
        '\n\n' +
        chalk.blue('• Registry Management:') +
        ' Used PackageRegistry and RegistryManager to manage package sources\n' +
        chalk.blue('• Dependency Tracking:') +
        ' Used Dependency and Package classes to model workspace structure\n' +
        chalk.blue('• Graph Analysis:') +
        ' Used DependencyGraph to analyze relationships between packages\n' +
        chalk.blue('• Validation:') +
        ' Used ValidationReport to identify and report dependency issues\n' +
        chalk.blue('• Conflict Resolution:') +
        ' Resolved version conflicts using DependencyRegistry\n' +
        chalk.blue('• Impact Analysis:') +
        ' Analyzed upgrade impacts using PackageDiff\n' +
        chalk.blue('• Visualization:') +
        ' Generated ASCII and DOT graph visualizations\n\n' +
        chalk.bold.green('All major package management APIs successfully demonstrated in an integrated workflow!'),
      {
        padding: 1,
        margin: 1,
        borderStyle: 'double',
        borderColor: 'magenta',
        align: 'left',
        title: '🎉 Integrated Monorepo Management Complete 🎉',
        titleAlignment: 'center',
      },
    ),
)
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
