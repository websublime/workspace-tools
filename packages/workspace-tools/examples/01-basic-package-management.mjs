import chalk from 'chalk';
import boxen from 'boxen';
import Table from 'cli-table3';
import { Dependency, Package, DependencyRegistry } from '../dist/esm/index.mjs';

// Console formatting utilities
const heading = (text) => console.log(chalk.bold.blue('\n' + text));
const subHeading = (text) => console.log(chalk.cyan('\n' + text));
const success = (text) => console.log(chalk.green(text));
const info = (text) => console.log(chalk.yellow(text));
const code = (text) => console.log(chalk.gray(text));
const createBox = (title, content) =>
  boxen(content, {
    title,
    padding: 1,
    margin: 1,
    borderStyle: 'round',
    borderColor: 'blue'
  });

console.log(createBox('Basic Package Management Example',
  chalk.bold('This example demonstrates creating and managing packages and dependencies.')
));

// Example 1: Creating Dependencies
heading('1. Creating Dependencies');
info('Creating a new dependency:');
code(`const dep1 = new Dependency('lodash', '^4.17.21');`);
const dep1 = new Dependency('lodash', '^4.17.21');
console.log(dep1);

info('\nCreating another dependency:');
code(`const dep2 = new Dependency('chalk', '^5.0.0');`);
const dep2 = new Dependency('chalk', '^5.0.0');
console.log(dep2);

// Show dependency properties
subHeading('Accessing Dependency Properties:');
info('Name:');
code(`dep1.name`);
console.log(`  ${dep1.name}`);

info('Version:');
code(`dep1.version`);
console.log(`  ${dep1.version}`);

// Example 2: Creating Packages
heading('2. Creating Packages');
info('Creating a new package:');
code(`const pkg1 = new Package('my-app', '1.0.0');`);
const pkg1 = new Package('my-app', '1.0.0');
console.log(pkg1);

// Adding dependencies to a package
subHeading('Adding Dependencies to a Package:');
code(`pkg1.addDependency(dep1);
pkg1.addDependency(dep2);`);
pkg1.addDependency(dep1);
pkg1.addDependency(dep2);

// Display package dependencies
const depList = pkg1.dependencies();
subHeading('Package Dependencies:');

// Create a table to display dependencies
const depTable = new Table({
  head: [chalk.white.bold('Dependency'), chalk.white.bold('Version')],
  colWidths: [20, 20]
});

depList.forEach(dep => {
  depTable.push([dep.name, dep.version]);
});

console.log(depTable.toString());

// Example 3: Updating Versions
heading('3. Updating Package and Dependency Versions');
info('Update package version:');
code(`pkg1.updateVersion('2.0.0');`);
pkg1.updateVersion('2.0.0');
console.log(`  New version: ${pkg1.version}`);

info('\nUpdate dependency version:');
code(`pkg1.updateDependencyVersion('lodash', '^5.0.0');`);
pkg1.updateDependencyVersion('lodash', '^5.0.0');

// Display updated dependencies
subHeading('Updated Dependencies:');
const updatedDeps = pkg1.dependencies();
const updatedDepsTable = new Table({
  head: [chalk.white.bold('Dependency'), chalk.white.bold('Version')],
  colWidths: [20, 20]
});

updatedDeps.forEach(dep => {
  updatedDepsTable.push([dep.name, dep.version]);
});

console.log(updatedDepsTable.toString());

// Example 4: Using DependencyRegistry
heading('4. Working with DependencyRegistry');
info('Creating a dependency registry:');
code(`const registry = new DependencyRegistry();`);
const registry = new DependencyRegistry();

info('\nGetting dependencies from registry:');
code(`const expDep1 = registry.getOrCreate('express', '^4.18.2');
const mongoDep = registry.getOrCreate('mongoose', '^7.0.0');`);
const expDep1 = registry.getOrCreate('express', '^4.18.2');
const mongoDep = registry.getOrCreate('mongoose', '^7.0.0');

// Create another package using the registry
subHeading('Creating a package with Registry Dependencies:');
code(`const pkg2 = Package.withRegistry(
  'api-server',
  '1.0.0',
  [
    ['express', '^4.18.2'],
    ['mongoose', '^7.0.0']
  ],
  registry
);`);
const pkg2 = Package.withRegistry(
  'api-server',
  '1.0.0',
  [
    ['express', '^4.18.2'],
    ['mongoose', '^7.0.0']
  ],
  registry
);

// Display the package and its dependencies
const pkg2DepsTable = new Table({
  head: [chalk.white.bold('Dependency'), chalk.white.bold('Version')],
  colWidths: [20, 20]
});

pkg2.dependencies().forEach(dep => {
  pkg2DepsTable.push([dep.name, dep.version]);
});

subHeading('Package Details:');
const packageDetails = new Table({
  chars: {
    'top': '═', 'top-mid': '╤', 'top-left': '╔', 'top-right': '╗',
    'bottom': '═', 'bottom-mid': '╧', 'bottom-left': '╚', 'bottom-right': '╝',
    'left': '║', 'left-mid': '╟', 'right': '║', 'right-mid': '╢',
    'mid': '─', 'mid-mid': '┼', 'middle': '│'
  },
  style: { head: ['cyan'] }
});

packageDetails.push(
  [chalk.bold('Name'), pkg2.name],
  [chalk.bold('Version'), pkg2.version],
  [chalk.bold('Dependency Count'), pkg2.dependencyCount],
  [chalk.bold('Dependencies'), pkg2DepsTable.toString()]
);

console.log(packageDetails.toString());

// Example 5: Getting Dependencies and Resolving Conflicts
heading('5. Finding and Resolving Dependency Conflicts');

// Create a scenario with conflicting versions
info('Creating packages with conflicting dependencies:');
code(`const dep3 = registry.getOrCreate('lodash', '^4.17.21');
const dep4 = registry.getOrCreate('lodash', '^3.10.1'); // Different version of the same package`);
const dep3 = registry.getOrCreate('lodash', '^4.17.21');
const dep4 = registry.getOrCreate('lodash', '^3.10.1'); // Different version

subHeading('Resolving version conflicts:');
code(`const resolutionResult = registry.resolveVersionConflicts();`);
const resolutionResult = registry.resolveVersionConflicts();

// Display the resolution results
const resolvedVersionsTable = new Table({
  head: [chalk.white.bold('Package'), chalk.white.bold('Resolved Version')],
  colWidths: [20, 20]
});

for (const [key, value] of Object.entries(resolutionResult.resolvedVersions)) {
  resolvedVersionsTable.push([key, value]);
}

console.log(resolvedVersionsTable.toString());

subHeading('Updates Required:');
if (resolutionResult.updatesRequired.length > 0) {
  const updatesTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Dependency'),
      chalk.white.bold('Current'),
      chalk.white.bold('New')
    ],
    colWidths: [15, 15, 15, 15]
  });

  resolutionResult.updatesRequired.forEach(update => {
    updatesTable.push([
      update.packageName,
      update.dependencyName,
      update.currentVersion,
      update.newVersion
    ]);
  });

  console.log(updatesTable.toString());
} else {
  console.log(chalk.green('No updates required'));
}

// Example 6: Applying Resolution and Finding Highest Compatible Version
heading('6. Advanced Dependency Operations');

subHeading('Applying resolution results:');
code(`registry.applyResolutionResult(resolutionResult);`);
registry.applyResolutionResult(resolutionResult);
success('Resolution applied successfully');

subHeading('Finding highest compatible version:');
code(`const highest = registry.findHighestCompatibleVersion('lodash', ['^3.0.0', '^4.0.0']);`);
const highest = registry.findHighestCompatibleVersion('lodash', ['^3.0.0', '^4.0.0']);

console.log(`Highest compatible version for 'lodash': ${chalk.green(highest || 'None found')}`);

// Summary
console.log(createBox('Summary',
  chalk.bold('Key Concepts Demonstrated:') + '\n\n' +
  '✅ Creating and managing dependencies\n' +
  '✅ Creating packages and adding dependencies\n' +
  '✅ Managing dependency versions\n' +
  '✅ Using the DependencyRegistry to share dependencies\n' +
  '✅ Resolving version conflicts in dependencies'
));
