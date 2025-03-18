import chalk from 'chalk';
import boxen from 'boxen';
import Table from 'cli-table3';
import {
  VersionUtils,
  Version,
  VersionComparisonResult,
  bumpVersion,
  bumpSnapshotVersion
} from '../dist/esm/index.mjs';

// Console formatting utilities
const heading = (text) => console.log(chalk.bold.blue('\n' + text));
const subHeading = (text) => console.log(chalk.cyan('\n' + text));
const success = (text) => console.log(chalk.green(text));
const info = (text) => console.log(chalk.yellow(text));
const warning = (text) => console.log(chalk.red(text));
const code = (text) => console.log(chalk.gray(text));
const createBox = (title, content) =>
  boxen(content, {
    title,
    padding: 1,
    margin: 1,
    borderStyle: 'round',
    borderColor: 'blue'
  });

console.log(createBox('Version Management Example',
  chalk.bold('This example demonstrates working with versions, comparing versions, and using version utilities.')
));

// Example 1: Version Enum and Basic Version Bumping
heading('1. Basic Version Bumping');
info('Using Version enum for bumping versions:');

const versions = [
  { name: 'Major bump', version: '1.2.3', type: Version.Major },
  { name: 'Minor bump', version: '1.2.3', type: Version.Minor },
  { name: 'Patch bump', version: '1.2.3', type: Version.Patch }
];

// Create a table for version bumping
const versionTable = new Table({
  head: [
    chalk.white.bold('Operation'),
    chalk.white.bold('Original'),
    chalk.white.bold('Version Type'),
    chalk.white.bold('Result')
  ],
  colWidths: [20, 15, 15, 15]
});

versions.forEach(({ name, version, type }) => {
  const result = bumpVersion(version, type);
  versionTable.push([
    name,
    version,
    Object.keys(Version).find(key => Version[key] === type),
    result
  ]);
});

console.log(versionTable.toString());

// Example 2: Snapshot Versions
heading('2. Creating Snapshot Versions');
info('Bumping to snapshot versions with commit hash:');
code(`const snapshotVersion = bumpSnapshotVersion('1.2.3', 'a1b2c3d');`);

const snapshotVersion = bumpSnapshotVersion('1.2.3', 'a1b2c3d');
success(`Result: ${snapshotVersion}`);

info('\nUsing the Version.Snapshot enum:');
code(`const snapshotVersionAlt = bumpVersion('2.0.0', Version.Snapshot);`);
const snapshotVersionAlt = bumpVersion('2.0.0', Version.Snapshot);
success(`Result: ${snapshotVersionAlt}`);

// Example 3: Version Utilities - Bumping Versions
heading('3. Using VersionUtils for Version Bumping');

const versionsToBump = [
  { version: '1.0.0', operation: 'bumpMajor' },
  { version: '1.0.0', operation: 'bumpMinor' },
  { version: '1.0.0', operation: 'bumpPatch' },
  { version: '1.0.0', operation: 'bumpSnapshot', arg: 'abc123' },
  { version: 'invalid', operation: 'bumpMajor' } // Testing invalid version
];

const versionUtilsTable = new Table({
  head: [
    chalk.white.bold('Original'),
    chalk.white.bold('Operation'),
    chalk.white.bold('Result'),
    chalk.white.bold('Status')
  ],
  colWidths: [15, 20, 30, 15]
});

versionsToBump.forEach(({ version, operation, arg }) => {
  let result;
  if (operation === 'bumpSnapshot') {
    result = VersionUtils.bumpSnapshot(version, arg);
    code(`VersionUtils.bumpSnapshot('${version}', '${arg}')`);
  } else {
    result = VersionUtils[operation](version);
    code(`VersionUtils.${operation}('${version}')`);
  }

  versionUtilsTable.push([
    version,
    operation,
    result || 'null',
    result ? chalk.green('Success') : chalk.red('Failed')
  ]);
});

console.log(versionUtilsTable.toString());

// Example 4: Comparing Versions
heading('4. Comparing Versions');
info('Using VersionUtils.compareVersions to find the relationship between versions:');

const versionPairs = [
  { v1: '1.0.0', v2: '2.0.0' },
  { v1: '1.0.0', v2: '1.1.0' },
  { v1: '1.0.0', v2: '1.0.1' },
  { v1: '1.0.0', v2: '1.0.0' },
  { v1: '2.0.0', v2: '1.0.0' },
  { v1: '1.1.0', v2: '1.0.0' },
  { v1: '1.0.0-alpha', v2: '1.0.0' },
  { v1: '1.0.0', v2: '1.0.0-beta' },
  { v1: '1.0.0-alpha', v2: '1.0.0-beta' },
  { v1: '1.0.0-beta', v2: '1.0.0-alpha' },
  { v1: 'invalid', v2: '1.0.0' }
];

// Mapping relation type to human readable description and color
const relationDescriptions = {
  [VersionComparisonResult.MajorUpgrade]: { text: 'Major Upgrade', color: chalk.green },
  [VersionComparisonResult.MinorUpgrade]: { text: 'Minor Upgrade', color: chalk.green },
  [VersionComparisonResult.PatchUpgrade]: { text: 'Patch Upgrade', color: chalk.green },
  [VersionComparisonResult.PrereleaseToStable]: { text: 'Prerelease to Stable', color: chalk.green },
  [VersionComparisonResult.NewerPrerelease]: { text: 'Newer Prerelease', color: chalk.green },
  [VersionComparisonResult.Identical]: { text: 'Identical', color: chalk.blue },
  [VersionComparisonResult.MajorDowngrade]: { text: 'Major Downgrade', color: chalk.red },
  [VersionComparisonResult.MinorDowngrade]: { text: 'Minor Downgrade', color: chalk.red },
  [VersionComparisonResult.PatchDowngrade]: { text: 'Patch Downgrade', color: chalk.red },
  [VersionComparisonResult.StableToPrerelease]: { text: 'Stable to Prerelease', color: chalk.red },
  [VersionComparisonResult.OlderPrerelease]: { text: 'Older Prerelease', color: chalk.red },
  [VersionComparisonResult.Indeterminate]: { text: 'Indeterminate', color: chalk.yellow }
};

const compareTable = new Table({
  head: [
    chalk.white.bold('Version 1'),
    chalk.white.bold('Version 2'),
    chalk.white.bold('Relation'),
    chalk.white.bold('Breaking Change?')
  ],
  colWidths: [15, 15, 30, 20]
});

versionPairs.forEach(({ v1, v2 }) => {
  code(`VersionUtils.compareVersions('${v1}', '${v2}')`);
  const relation = VersionUtils.compareVersions(v1, v2);
  const isBreaking = VersionUtils.isBreakingChange(v1, v2);

  const relationDesc = relationDescriptions[relation];

  compareTable.push([
    v1,
    v2,
    relationDesc ? relationDesc.color(relationDesc.text) : chalk.red('Unknown'),
    isBreaking ? chalk.red('Yes') : chalk.green('No')
  ]);
});

console.log(compareTable.toString());

// Example 5: Real-World Scenario - Analyzing Project Updates
heading('5. Real-World Scenario: Analyzing Project Dependencies');
info('In this scenario, we analyze a set of dependencies to determine update types:');

const projectDependencies = [
  { name: 'react', currentVersion: '17.0.2', newVersion: '18.0.0' },
  { name: 'express', currentVersion: '4.17.1', newVersion: '4.18.2' },
  { name: 'lodash', currentVersion: '4.17.20', newVersion: '4.17.21' },
  { name: 'typescript', currentVersion: '4.5.0', newVersion: '5.0.0' },
  { name: 'eslint', currentVersion: '7.32.0', newVersion: '8.0.0' },
  { name: 'jest', currentVersion: '27.0.0', newVersion: '27.5.1' }
];

subHeading('Dependency Update Analysis:');
const dependencyTable = new Table({
  head: [
    chalk.white.bold('Package'),
    chalk.white.bold('Current'),
    chalk.white.bold('New'),
    chalk.white.bold('Update Type'),
    chalk.white.bold('Breaking?')
  ],
  colWidths: [15, 15, 15, 20, 12]
});

let breakingChanges = 0;
let safeUpdates = 0;

projectDependencies.forEach(({ name, currentVersion, newVersion }) => {
  const relation = VersionUtils.compareVersions(currentVersion, newVersion);
  const isBreaking = VersionUtils.isBreakingChange(currentVersion, newVersion);

  const relationDesc = relationDescriptions[relation];

  if (isBreaking) {
    breakingChanges++;
  } else {
    safeUpdates++;
  }

  dependencyTable.push([
    name,
    currentVersion,
    newVersion,
    relationDesc ? relationDesc.color(relationDesc.text) : chalk.red('Unknown'),
    isBreaking ? chalk.red('Yes') : chalk.green('No')
  ]);
});

console.log(dependencyTable.toString());

subHeading('Update Summary:');
console.log(`Total Dependencies: ${chalk.bold(projectDependencies.length)}`);
console.log(`Breaking Changes: ${chalk.bold.red(breakingChanges)}`);
console.log(`Safe Updates: ${chalk.bold.green(safeUpdates)}`);

// Example 6: Version Recommendation System
heading('6. Version Recommendation System');
info('Building a simple version recommendation system:');

function recommendVersionUpdate(currentVersion, updateImportance = 'minor') {
  try {
    switch (updateImportance) {
      case 'major':
        return VersionUtils.bumpMajor(currentVersion);
      case 'minor':
        return VersionUtils.bumpMinor(currentVersion);
      case 'patch':
        return VersionUtils.bumpPatch(currentVersion);
      default:
        return VersionUtils.bumpPatch(currentVersion);
    }
  } catch (error) {
    return null;
  }
}

const currentProjectVersion = '1.4.7';
subHeading(`Current Project Version: ${chalk.bold(currentProjectVersion)}`);
info('Recommending next versions based on different importance levels:');

const recommendationTable = new Table({
  head: [
    chalk.white.bold('Update Type'),
    chalk.white.bold('Current'),
    chalk.white.bold('Recommended'),
    chalk.white.bold('Description')
  ],
  colWidths: [15, 15, 15, 50]
});

recommendationTable.push([
  chalk.red('Major'),
  currentProjectVersion,
  recommendVersionUpdate(currentProjectVersion, 'major'),
  'Major version bump for breaking changes'
]);

recommendationTable.push([
  chalk.yellow('Minor'),
  currentProjectVersion,
  recommendVersionUpdate(currentProjectVersion, 'minor'),
  'Minor version bump for new features (no breaking changes)'
]);

recommendationTable.push([
  chalk.green('Patch'),
  currentProjectVersion,
  recommendVersionUpdate(currentProjectVersion, 'patch'),
  'Patch version bump for bug fixes and minor improvements'
]);

console.log(recommendationTable.toString());

// Summary
console.log(createBox('Summary',
  chalk.bold('Key Concepts Demonstrated:') + '\n\n' +
  '✅ Using Version enum for version bumping\n' +
  '✅ Creating snapshot versions\n' +
  '✅ Using VersionUtils for various version operations\n' +
  '✅ Comparing versions with VersionComparisonResult\n' +
  '✅ Detecting breaking changes\n' +
  '✅ Building a version recommendation system'
));
