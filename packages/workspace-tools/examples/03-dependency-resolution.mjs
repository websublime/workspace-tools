import chalk from 'chalk';
import boxen from 'boxen';
import Table from 'cli-table3';
import {
  DependencyRegistry,
  Dependency,
  Package,
  ResolutionErrorType,
  VersionUtils
} from '../dist/esm/index.mjs';

// Console formatting utilities
const heading = (text) => console.log(chalk.bold.blue('\n' + text));
const subHeading = (text) => console.log(chalk.cyan('\n' + text));
const success = (text) => console.log(chalk.green(text));
const info = (text) => console.log(chalk.yellow(text));
const error = (text) => console.log(chalk.red(text));
const code = (text) => console.log(chalk.gray(text));
const createBox = (title, content) =>
  boxen(content, {
    title,
    padding: 1,
    margin: 1,
    borderStyle: 'round',
    borderColor: 'blue'
  });

console.log(createBox('Dependency Resolution Example',
  chalk.bold('This example demonstrates managing dependencies and resolving conflicts.')
));

// Example 1: Setting up a DependencyRegistry
heading('1. Creating and Managing a Dependency Registry');
info('Creating a new dependency registry:');
code(`const registry = new DependencyRegistry();`);
const registry = new DependencyRegistry();
success('Dependency registry created successfully');

// Example 2: Adding Dependencies
heading('2. Adding Dependencies to the Registry');
info('Adding various dependencies with different version requirements:');

const dependencies = [
  { name: 'express', version: '^4.17.1' },
  { name: 'react', version: '^17.0.2' },
  { name: 'lodash', version: '^4.17.21' },
  { name: 'typescript', version: '^4.5.4' }
];

// Create a table for dependencies
const dependenciesTable = new Table({
  head: [
    chalk.white.bold('Dependency Name'),
    chalk.white.bold('Version Requirement')
  ],
  colWidths: [20, 25]
});

// Fill the registry and the table
const registryDeps = [];
dependencies.forEach(({ name, version }) => {
  code(`const ${name}Dep = registry.getOrCreate('${name}', '${version}');`);
  const dep = registry.getOrCreate(name, version);
  registryDeps.push(dep);
  dependenciesTable.push([name, version]);
});

console.log(dependenciesTable.toString());
success('Dependencies added to registry');

// Example 3: Creating Conflicting Dependencies
heading('3. Creating Conflicting Dependencies');
info('Adding dependencies with conflicting version requirements:');

const conflictingDeps = [
  { name: 'react', version: '^16.8.0' }, // Conflicts with previous react version
  { name: 'express', version: '^5.0.0-alpha.8' }, // Conflicts with previous express version
  { name: 'typescript', version: '~4.5.0' } // Stricter requirement, but compatible
];

const conflictTable = new Table({
  head: [
    chalk.white.bold('Dependency Name'),
    chalk.white.bold('New Version Req'),
    chalk.white.bold('Conflicts with')
  ],
  colWidths: [20, 20, 20]
});

conflictingDeps.forEach(({ name, version }) => {
  code(`const ${name}DepConflict = registry.getOrCreate('${name}', '${version}');`);
  const dep = registry.getOrCreate(name, version);

  // Find existing dependency with the same name
  const existing = dependencies.find(d => d.name === name);
  conflictTable.push([name, version, existing ? existing.version : 'N/A']);
});

console.log(conflictTable.toString());
info('Created potentially conflicting dependencies');

// Example 4: Creating Packages Using Dependencies
heading('4. Creating Packages that Use the Dependencies');

// Create packages with dependencies from registry
const packageDefs = [
  {
    name: 'api-server',
    version: '1.0.0',
    deps: ['express', 'lodash', 'typescript']
  },
  {
    name: 'web-frontend',
    version: '0.9.0',
    deps: ['react', 'lodash']
  },
  {
    name: 'admin-dashboard',
    version: '1.2.0',
    deps: ['react', 'express', 'typescript']
  }
];

const packages = [];
packageDefs.forEach(({ name, version, deps }) => {
  code(`const ${name.replace('-', '')} = new Package('${name}', '${version}');`);
  const pkg = new Package(name, version);

  deps.forEach(depName => {
    // Find all dependencies with this name in the registry
    const depVersions = [...dependencies, ...conflictingDeps]
      .filter(d => d.name === depName)
      .map(d => d.version);

    if (depVersions.length > 0) {
      const depVersion = depVersions[0]; // Just take the first one for now
      code(`${name.replace('-', '')}.addDependency(registry.getOrCreate('${depName}', '${depVersion}'));`);
      pkg.addDependency(registry.getOrCreate(depName, depVersion));
    }
  });

  packages.push(pkg);
});

// Display the packages and their dependencies
subHeading('Created Packages:');
const packagesTable = new Table({
  head: [
    chalk.white.bold('Package Name'),
    chalk.white.bold('Version'),
    chalk.white.bold('Dependencies')
  ],
  colWidths: [20, 15, 40]
});

packages.forEach(pkg => {
  const depNames = pkg.dependencies().map(d => `${d.name}@${d.version}`).join(', ');
  packagesTable.push([pkg.name, pkg.version, depNames]);
});

console.log(packagesTable.toString());

// Example 5: Resolving Version Conflicts
heading('5. Resolving Version Conflicts');
info('Attempting to resolve version conflicts in the registry:');
code(`const resolutionResult = registry.resolveVersionConflicts();`);

const resolutionResult = registry.resolveVersionConflicts();

// Display resolution results
subHeading('Resolution Results:');
if (Object.keys(resolutionResult.resolvedVersions).length > 0) {
  const resolvedVersionsTable = new Table({
    head: [
      chalk.white.bold('Dependency'),
      chalk.white.bold('Resolved Version')
    ],
    colWidths: [20, 25]
  });

  for (const [key, value] of Object.entries(resolutionResult.resolvedVersions)) {
    resolvedVersionsTable.push([key, value]);
  }

  console.log(resolvedVersionsTable.toString());
} else {
  info('No versions were resolved');
}

// Display updates required
subHeading('Updates Required:');
if (resolutionResult.updatesRequired.length > 0) {
  const updatesTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Dependency'),
      chalk.white.bold('Current'),
      chalk.white.bold('New')
    ],
    colWidths: [20, 20, 15, 15]
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
  info('No updates required');
}

// Example 6: Applying Resolution
heading('6. Applying Resolution Results');
info('Apply the resolved versions to update the dependencies:');
code(`registry.applyResolutionResult(resolutionResult);`);

try {
  registry.applyResolutionResult(resolutionResult);
  success('Resolution applied successfully');
} catch (err) {
  error(`Failed to apply resolution: ${err.message}`);
}

// Example 7: Finding Highest Compatible Version
heading('7. Finding Highest Compatible Versions');
info('Finding highest compatible versions for multiple requirements:');

const compatibilityTests = [
  {
    name: 'react',
    requirements: ['^16.8.0', '^17.0.0']
  },
  {
    name: 'typescript',
    requirements: ['^4.5.0', '~4.5.4']
  },
  {
    name: 'express',
    requirements: ['^4.0.0', '^5.0.0-alpha']
  },
  {
    name: 'non-existent',
    requirements: ['^1.0.0']
  }
];

const compatibilityTable = new Table({
  head: [
    chalk.white.bold('Dependency'),
    chalk.white.bold('Requirements'),
    chalk.white.bold('Highest Compatible')
  ],
  colWidths: [20, 30, 20]
});

compatibilityTests.forEach(({ name, requirements }) => {
  code(`const highestVer = registry.findHighestCompatibleVersion('${name}', ${JSON.stringify(requirements)});`);
  const highestVer = registry.findHighestCompatibleVersion(name, requirements);

  compatibilityTable.push([
    name,
    requirements.join(', '),
    highestVer || chalk.red('None')
  ]);
});

console.log(compatibilityTable.toString());

// Example 8: Handling Resolution Errors
heading('8. Understanding Resolution Error Types');
info('The ResolutionErrorType enum helps identify types of resolution errors:');

// Actually use the ResolutionErrorType enum
const errorTypesTable = new Table({
  head: [
    chalk.white.bold('Error Type'),
    chalk.white.bold('Description'),
    chalk.white.bold('Typical Scenario')
  ],
  colWidths: [25, 30, 30]
});

errorTypesTable.push([
  `ResolutionErrorType.VersionParseError (${ResolutionErrorType.VersionParseError})`,
  'Failed to parse a version string',
  'Invalid version format like "x.y.z"'
]);

errorTypesTable.push([
  `ResolutionErrorType.IncompatibleVersions (${ResolutionErrorType.IncompatibleVersions})`,
  'No single version satisfies all requirements',
  'One package needs ^1.0.0, another needs ^2.0.0'
]);

errorTypesTable.push([
  `ResolutionErrorType.NoValidVersion (${ResolutionErrorType.NoValidVersion})`,
  'No version found that meets requirements',
  'Requirement exists but the version isn\'t available'
]);

console.log(errorTypesTable.toString());

// Simulate error handling based on error type
subHeading('Error Handling Based on Error Type:');
code(`
function handleResolutionError(error) {
  switch (error.code) {
    case ResolutionErrorType.VersionParseError:
      console.error('Failed to parse version, please check version format');
      break;
    case ResolutionErrorType.IncompatibleVersions:
      console.error('Incompatible versions requested, manual resolution required');
      break;
    case ResolutionErrorType.NoValidVersion:
      console.error('No valid version found that satisfies all requirements');
      break;
    default:
      console.error('Unknown resolution error');
  }
}`);

// Example 8b: Using VersionUtils to Check Version Compatibility
heading('8b. Checking Version Compatibility with VersionUtils');
info('We can use VersionUtils to determine if dependencies are compatible:');

const dependencyPairs = [
  { name: 'react', current: '16.8.0', potential: '17.0.0' },
  { name: 'typescript', current: '4.3.5', potential: '4.5.4' },
  { name: 'express', current: '4.17.1', potential: '5.0.0-alpha.8' }
];

const compatibilityCheckTable = new Table({
  head: [
    chalk.white.bold('Dependency'),
    chalk.white.bold('Current'),
    chalk.white.bold('Potential'),
    chalk.white.bold('Relationship'),
    chalk.white.bold('Breaking Change?')
  ],
  colWidths: [15, 15, 15, 20, 15]
});

dependencyPairs.forEach(({ name, current, potential }) => {
  code(`const relationship = VersionUtils.compareVersions('${current}', '${potential}');`);
  const relationship = VersionUtils.compareVersions(current, potential);

  code(`const isBreaking = VersionUtils.isBreakingChange('${current}', '${potential}');`);
  const isBreaking = VersionUtils.isBreakingChange(current, potential);

  let relationshipText;
  switch (relationship) {
    case 0: relationshipText = 'Major Upgrade'; break;
    case 1: relationshipText = 'Minor Upgrade'; break;
    case 2: relationshipText = 'Patch Upgrade'; break;
    case 5: relationshipText = 'Identical'; break;
    default: relationshipText = `Other (${relationship})`;
  }

  compatibilityCheckTable.push([
    name,
    current,
    potential,
    relationshipText,
    isBreaking ? chalk.red('Yes') : chalk.green('No')
  ]);
});

console.log(compatibilityCheckTable.toString());

info('This information helps decide whether to update dependencies and predict potential issues.');

// Example 9: Real-World Scenario - Coordinated Version Update
heading('9. Real-World Scenario: Project-wide Dependency Update');
info('In this scenario, we\'ll update multiple packages to use consistent dependencies:');

// Create a more complex package structure with some shared dependencies
const projectPackages = [
  {
    name: 'core-lib',
    version: '2.1.0',
    deps: [
      { name: 'lodash', version: '^4.17.15' },
      { name: 'typescript', version: '^4.3.5' }
    ]
  },
  {
    name: 'api-service',
    version: '1.5.2',
    deps: [
      { name: 'express', version: '^4.17.1' },
      { name: 'lodash', version: '^4.17.21' },
      { name: 'typescript', version: '^4.5.4' }
    ]
  },
  {
    name: 'web-client',
    version: '3.0.0-beta.1',
    deps: [
      { name: 'react', version: '^17.0.2' },
      { name: 'lodash', version: '^4.17.20' },
      { name: 'typescript', version: '^4.4.0' }
    ]
  }
];

// Create a new registry for the project
const projectRegistry = new DependencyRegistry();

// Create packages and add to registry
const projectPkgs = projectPackages.map(pkgDef => {
  const pkg = new Package(pkgDef.name, pkgDef.version);

  pkgDef.deps.forEach(dep => {
    const depInstance = projectRegistry.getOrCreate(dep.name, dep.version);
    pkg.addDependency(depInstance);
  });

  return pkg;
});

// Show initial state of the project
subHeading('Initial Project Dependencies:');
const initialProjectTable = new Table({
  head: [
    chalk.white.bold('Package'),
    chalk.white.bold('Dependency'),
    chalk.white.bold('Version')
  ],
  colWidths: [20, 20, 20]
});

projectPkgs.forEach(pkg => {
  pkg.dependencies().forEach(dep => {
    initialProjectTable.push([pkg.name, dep.name, dep.version]);
  });
});

console.log(initialProjectTable.toString());

// Resolve conflicts
info('Resolving project-wide dependency conflicts:');
code(`const projectResolution = projectRegistry.resolveVersionConflicts();`);
const projectResolution = projectRegistry.resolveVersionConflicts();

// Apply resolution to all packages
code(`projectRegistry.applyResolutionResult(projectResolution);

// Now update each package to use the resolved dependencies
projectPkgs.forEach(pkg => {
  pkg.updateDependenciesFromResolution(projectResolution);
});`);

projectRegistry.applyResolutionResult(projectResolution);

// Update each package to use the resolved dependencies
projectPkgs.forEach(pkg => {
  pkg.updateDependenciesFromResolution(projectResolution);
});

// Show final state
subHeading('Final Project Dependencies (after resolution):');
const finalProjectTable = new Table({
  head: [
    chalk.white.bold('Package'),
    chalk.white.bold('Dependency'),
    chalk.white.bold('Version')
  ],
  colWidths: [20, 20, 20]
});

projectPkgs.forEach(pkg => {
  pkg.dependencies().forEach(dep => {
    finalProjectTable.push([pkg.name, dep.name, dep.version]);
  });
});

console.log(finalProjectTable.toString());

// Highlight the differences
if (projectResolution.updatesRequired.length > 0) {
  subHeading('Applied Updates:');

  const updatesTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Dependency'),
      chalk.white.bold('Before'),
      chalk.white.bold('After')
    ],
    colWidths: [20, 20, 20, 20]
  });

  projectResolution.updatesRequired.forEach(update => {
    updatesTable.push([
      update.packageName,
      update.dependencyName,
      update.currentVersion,
      chalk.green(update.newVersion)
    ]);
  });

  console.log(updatesTable.toString());

  // Provide a summary
  const uniqueDeps = [...new Set(projectResolution.updatesRequired.map(u => u.dependencyName))];
  const uniquePkgs = [...new Set(projectResolution.updatesRequired.map(u => u.packageName))];

  console.log(`Updated ${chalk.bold(uniqueDeps.length)} unique dependencies across ${chalk.bold(uniquePkgs.length)} packages.`);
  console.log(`This ensures all packages now use consistent dependency versions.`);
} else {
  success('No updates were required. All packages already use consistent versions.');
}

// Summary
console.log(createBox('Summary',
  chalk.bold('Key Concepts Demonstrated:') + '\n\n' +
  '✅ Creating and managing a dependency registry\n' +
  '✅ Adding dependencies with different version requirements\n' +
  '✅ Creating packages using shared dependencies\n' +
  '✅ Resolving version conflicts\n' +
  '✅ Finding highest compatible versions\n' +
  '✅ Understanding resolution error types\n' +
  '✅ Implementing project-wide dependency resolution'
));
