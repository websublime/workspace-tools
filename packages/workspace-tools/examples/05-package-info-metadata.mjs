import chalk from 'chalk';
import boxen from 'boxen';
import Table from 'cli-table3';
import path from 'node:path';
import {
  Package,
  PackageInfo,
  DependencyRegistry,
  parseScopedPackage
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

console.log(createBox('Package Information & Metadata Example',
  chalk.bold('This example demonstrates working with package.json and metadata.')
));

// Example 1: Creating Packages and PackageInfo
heading('1. Creating Basic PackageInfo');

// First, create a package
info('Creating a package:');
code(`const pkg = new Package('my-app', '1.0.0');`);
const pkg = new Package('my-app', '1.0.0');

// Add some dependencies
const registry = new DependencyRegistry();
const reactDep = registry.getOrCreate('react', '^18.0.0');
const expressDep = registry.getOrCreate('express', '^4.18.2');

code(`
// Add dependencies
pkg.addDependency(registry.getOrCreate('react', '^18.0.0'));
pkg.addDependency(registry.getOrCreate('express', '^4.18.2'));
`);
pkg.addDependency(reactDep);
pkg.addDependency(expressDep);

// Create package.json content for the example
const packageJsonContent = {
  name: 'my-app',
  version: '1.0.0',
  description: 'My example application',
  main: 'index.js',
  scripts: {
    test: 'jest',
    start: 'node index.js'
  },
  dependencies: {
    react: '^18.0.0',
    express: '^4.18.2'
  },
  devDependencies: {
    jest: '^29.0.0',
    typescript: '^5.0.0'
  },
  keywords: ['example', 'demo'],
  author: 'Example Author',
  license: 'MIT'
};

// Create package info
info('\nCreating PackageInfo:');
code(`
const pkgInfo = new PackageInfo(
  pkg,                              // Package object
  '/path/to/my-app/package.json',   // Path to package.json file
  '/path/to/my-app',                // Path to package directory
  './my-app',                       // Relative path to package
  packageJsonContent                // package.json content
);
`);

const pkgInfo = new PackageInfo(
  pkg,
  '/path/to/my-app/package.json',
  '/path/to/my-app',
  './my-app',
  packageJsonContent
);

// Example 2: Accessing PackageInfo Properties
heading('2. Accessing PackageInfo Properties');

const packageInfoTable = new Table({
  chars: {
    'top': '═', 'top-mid': '╤', 'top-left': '╔', 'top-right': '╗',
    'bottom': '═', 'bottom-mid': '╧', 'bottom-left': '╚', 'bottom-right': '╝',
    'left': '║', 'left-mid': '╟', 'right': '║', 'right-mid': '╢',
    'mid': '─', 'mid-mid': '┼', 'middle': '│'
  },
  style: { 'padding-left': 1, 'padding-right': 1, head: ['cyan'] }
});

info('Accessing various properties from PackageInfo:');
code(`
console.log(pkgInfo.packageJsonPath);  // Path to package.json
console.log(pkgInfo.packagePath);      // Path to package directory
console.log(pkgInfo.packageRelativePath); // Relative path
console.log(pkgInfo.package.name);     // Access the Package object
`);

packageInfoTable.push(
  [chalk.bold('Property'), chalk.bold('Value')],
  ['packageJsonPath', pkgInfo.packageJsonPath],
  ['packagePath', pkgInfo.packagePath],
  ['packageRelativePath', pkgInfo.packageRelativePath],
  ['package.name', pkgInfo.package.name],
  ['package.version', pkgInfo.package.version]
);

console.log(packageInfoTable.toString());

// Example 3: Accessing and Modifying package.json Content
heading('3. Accessing and Modifying package.json Content');

// Access package.json content
info('Accessing package.json content:');
code(`const packageJson = pkgInfo.packageJson;`);

// Create a table to display some key package.json fields
const pkgJsonTable = new Table({
  head: [chalk.white.bold('Field'), chalk.white.bold('Value')],
  colWidths: [20, 50]
});

const packageJson = pkgInfo.packageJson;

// Show some key fields from package.json
pkgJsonTable.push(
  ['name', packageJson.name],
  ['version', packageJson.version],
  ['description', packageJson.description],
  ['main', packageJson.main],
  ['license', packageJson.license]
);

console.log(pkgJsonTable.toString());

// Dependencies from package.json
subHeading('Dependencies in package.json:');
const depsTable = new Table({
  head: [chalk.white.bold('Dependency'), chalk.white.bold('Version'), chalk.white.bold('Type')],
  colWidths: [20, 15, 15]
});

// Add dependencies
for (const [name, version] of Object.entries(packageJson.dependencies || {})) {
  depsTable.push([name, version, 'production']);
}

// Add dev dependencies
for (const [name, version] of Object.entries(packageJson.devDependencies || {})) {
  depsTable.push([name, version, 'development']);
}

console.log(depsTable.toString());

// Example 4: Updating Package Versions and Dependencies
heading('4. Updating Package Versions and Dependencies');

info('Updating the package version:');
code(`
// Update the version
pkgInfo.updateVersion('2.0.0');

// Check the new version
console.log(pkgInfo.package.version); // 2.0.0
`);

try {
  pkgInfo.updateVersion('2.0.0');
  success(`Package version updated to: ${pkgInfo.package.version}`);
} catch (err) {
  error(`Failed to update version: ${err.message}`);
}

info('\nUpdating a dependency version:');
code(`
// Update the react dependency
pkgInfo.updateDependencyVersion('react', '^18.2.0');

// Check the dependency version
console.log(pkgInfo.package.getDependency('react').version); // ^18.2.0
`);

try {
  pkgInfo.updateDependencyVersion('react', '^18.2.0');
  const reactDep = pkgInfo.package.getDependency('react');
  success(`React dependency updated to: ${reactDep ? reactDep.version : 'not found'}`);
} catch (err) {
  error(`Failed to update dependency: ${err.message}`);
}

// Example 5: Writing Package.json to Disk
heading('5. Writing Package.json to Disk');

info('Writing modified package.json to disk:');
code(`
// Write the updated package.json to disk
pkgInfo.writePackageJson();
`);

info('This would write the modified package.json to the specified path.');
success('(In this example, we\'re not actually writing to disk)');

// Example 6: Working with Dependency Resolution
heading('6. Applying Dependency Resolution');

info('Creating a resolution to apply to this package:');
code(`
// Create a dependency registry
const depRegistry = new DependencyRegistry();

// Add some dependencies with different versions
const reactDepA = depRegistry.getOrCreate('react', '^17.0.2');
const reactDepB = depRegistry.getOrCreate('react', '^18.0.0');

// Resolve version conflicts
const resolution = depRegistry.resolveVersionConflicts();

// Apply the resolution to our package info
pkgInfo.applyDependencyResolution(resolution);
`);

// Set up the scenario
const depRegistry = new DependencyRegistry();
const reactDepA = depRegistry.getOrCreate('react', '^17.0.2');
const reactDepB = depRegistry.getOrCreate('react', '^18.0.0');
const resolution = depRegistry.resolveVersionConflicts();

info('\nResolution result:');
if (resolution.updatesRequired.length > 0) {
  const resolutionTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Dependency'),
      chalk.white.bold('From'),
      chalk.white.bold('To')
    ],
    colWidths: [15, 15, 15, 15]
  });

  resolution.updatesRequired.forEach(update => {
    resolutionTable.push([
      update.packageName,
      update.dependencyName,
      update.currentVersion,
      update.newVersion
    ]);
  });

  console.log(resolutionTable.toString());
} else {
  info('No updates required in the resolution');
}

try {
  pkgInfo.applyDependencyResolution(resolution);
  success('Applied dependency resolution to package');
} catch (err) {
  error(`Failed to apply resolution: ${err.message}`);
}

// Example 7: Parsing Scoped Package Names
heading('7. Parsing Scoped Package Names');

info('Parsing different formats of scoped package names:');

const scopedExamples = [
  '@scope/name',
  '@scope/name@1.0.0',
  '@scope/name@1.0.0@/path/to/package',
  '@scope/name:1.0.0',
  'regular-package@1.0.0' // Non-scoped for comparison
];

const scopedTable = new Table({
  head: [
    chalk.white.bold('Input'),
    chalk.white.bold('Result'),
    chalk.white.bold('Name'),
    chalk.white.bold('Version'),
    chalk.white.bold('Path')
  ],
  colWidths: [25, 15, 20, 15, 20]
});

scopedExamples.forEach(input => {
  code(`const parsed = parseScopedPackage('${input}');`);
  const parsed = parseScopedPackage(input);

  scopedTable.push([
    input,
    parsed ? chalk.green('Parsed') : chalk.red('Not scoped'),
    parsed ? parsed.name : 'N/A',
    parsed ? parsed.version : 'N/A',
    parsed && parsed.path ? parsed.path : 'N/A'
  ]);
});

console.log(scopedTable.toString());

// Example 8: Real-World Scenario - Working With Workspace Package Info
heading('8. Real-World Scenario: Managing a Workspace');
info('In this scenario, we load and manage packages in a monorepo workspace:');

// Create example package.json files for a monorepo
const workspacePackages = [
  {
    name: '@workspace/core',
    version: '1.0.0',
    dependencies: {
      'lodash': '^4.17.21'
    },
    path: 'packages/core'
  },
  {
    name: '@workspace/ui',
    version: '0.5.0',
    dependencies: {
      '@workspace/core': '^1.0.0',
      'react': '^17.0.2'
    },
    path: 'packages/ui'
  },
  {
    name: '@workspace/api',
    version: '0.8.0',
    dependencies: {
      '@workspace/core': '^1.0.0',
      'express': '^4.17.1'
    },
    path: 'packages/api'
  }
];

// Create a collection to hold the package infos
const workspacePkgInfos = [];
const workspacePkgs = [];

// Process each workspace package
code(`
// In a real scenario, we would find and load package.json files from disk
const workspaceRoot = '/path/to/workspace';

// For each package in the workspace:
workspacePackages.forEach(pkgDef => {
  // Create package object
  const pkg = new Package(pkgDef.name, pkgDef.version);

  // Add dependencies
  for (const [name, version] of Object.entries(pkgDef.dependencies)) {
    pkg.addDependency(registry.getOrCreate(name, version));
  }

  // Create package info
  const packageJsonPath = path.join(workspaceRoot, pkgDef.path, 'package.json');
  const pkgInfo = new PackageInfo(
    pkg,
    packageJsonPath,
    path.join(workspaceRoot, pkgDef.path),
    pkgDef.path,
    {
      name: pkgDef.name,
      version: pkgDef.version,
      dependencies: pkgDef.dependencies
    }
  );

  workspacePkgInfos.push(pkgInfo);
  workspacePkgs.push(pkg);
});
`);

// Simulate creating the package infos
workspacePackages.forEach(pkgDef => {
  // Create package object
  const pkg = new Package(pkgDef.name, pkgDef.version);
  workspacePkgs.push(pkg);

  // Add dependencies
  for (const [name, version] of Object.entries(pkgDef.dependencies)) {
    pkg.addDependency(registry.getOrCreate(name, version));
  }

  // Create package info
  const workspaceRoot = '/path/to/workspace';
  const packageJsonPath = path.join(workspaceRoot, pkgDef.path, 'package.json');
  const pkgInfo = new PackageInfo(
    pkg,
    packageJsonPath,
    path.join(workspaceRoot, pkgDef.path),
    pkgDef.path,
    {
      name: pkgDef.name,
      version: pkgDef.version,
      dependencies: pkgDef.dependencies
    }
  );

  workspacePkgInfos.push(pkgInfo);
});

// Display the workspace packages
subHeading('Workspace Packages:');
const workspaceTable = new Table({
  head: [
    chalk.white.bold('Package'),
    chalk.white.bold('Version'),
    chalk.white.bold('Path'),
    chalk.white.bold('Dependencies')
  ],
  colWidths: [20, 10, 20, 30]
});

workspacePkgInfos.forEach(pkgInfo => {
  const deps = pkgInfo.package.dependencies()
    .map(dep => `${dep.name}@${dep.version}`)
    .join(', ');

  workspaceTable.push([
    pkgInfo.package.name,
    pkgInfo.package.version,
    pkgInfo.packageRelativePath,
    deps
  ]);
});

console.log(workspaceTable.toString());

// Example 9: Batch Version Update in Workspace
heading('9. Batch Version Update in Workspace');
info('Updating the version of all workspace packages:');

code(`
// Update all workspace packages to a new minor version
workspacePkgInfos.forEach(pkgInfo => {
  // Get current version
  const currentVersion = pkgInfo.package.version;

  // Compute new version (increment minor)
  const parts = currentVersion.split('.');
  parts[1] = parseInt(parts[1]) + 1;
  parts[2] = 0; // Reset patch to 0
  const newVersion = parts.join('.');

  // Update the package version
  pkgInfo.updateVersion(newVersion);

  console.log(\`Updated \${pkgInfo.package.name} from \${currentVersion} to \${newVersion}\`);
});

// Update internal dependencies to match new versions
workspacePkgInfos.forEach(pkgInfo => {
  pkgInfo.package.dependencies().forEach(dep => {
    // Check if this is a workspace dependency
    const isWorkspaceDep = workspacePkgs.some(pkg => pkg.name === dep.name);

    if (isWorkspaceDep) {
      // Find the referenced workspace package
      const referencedPkg = workspacePkgs.find(pkg => pkg.name === dep.name);

      if (referencedPkg) {
        // Update to match the new version
        pkgInfo.updateDependencyVersion(dep.name, \`^\${referencedPkg.version}\`);
        console.log(
          \`Updated dependency in \${pkgInfo.package.name}: \${dep.name} -> ^\${referencedPkg.version}\`
        );
      }
    }
  });
});
`);

// Simulate the version update
const versionUpdatesTable = new Table({
  head: [
    chalk.white.bold('Package'),
    chalk.white.bold('Old Version'),
    chalk.white.bold('New Version')
  ],
  colWidths: [25, 15, 15]
});

workspacePkgInfos.forEach(pkgInfo => {
  // Get current version
  const currentVersion = pkgInfo.package.version;

  // Compute new version (increment minor)
  const parts = currentVersion.split('.');
  parts[1] = parseInt(parts[1]) + 1;
  parts[2] = 0; // Reset patch to 0
  const newVersion = parts.join('.');

  // Update the package version
  try {
    pkgInfo.updateVersion(newVersion);
    versionUpdatesTable.push([
      pkgInfo.package.name,
      currentVersion,
      newVersion
    ]);
  } catch (err) {
    error(`Failed to update ${pkgInfo.package.name}: ${err.message}`);
  }
});

console.log(versionUpdatesTable.toString());

// Update internal dependencies
subHeading('Updating Internal Dependencies:');
const depUpdatesTable = new Table({
  head: [
    chalk.white.bold('Package'),
    chalk.white.bold('Dependency'),
    chalk.white.bold('New Version')
  ],
  colWidths: [25, 25, 20]
});

workspacePkgInfos.forEach(pkgInfo => {
  pkgInfo.package.dependencies().forEach(dep => {
    // Check if this is a workspace dependency
    const isWorkspaceDep = workspacePkgs.some(pkg => pkg.name === dep.name);

    if (isWorkspaceDep) {
      // Find the referenced workspace package
      const referencedPkg = workspacePkgs.find(pkg => pkg.name === dep.name);

      if (referencedPkg) {
        // Update to match the new version
        try {
          pkgInfo.updateDependencyVersion(dep.name, `^${referencedPkg.version}`);
          depUpdatesTable.push([
            pkgInfo.package.name,
            dep.name,
            `^${referencedPkg.version}`
          ]);
        } catch (err) {
          error(`Failed to update dependency ${dep.name}: ${err.message}`);
        }
      }
    }
  });
});

console.log(depUpdatesTable.toString());

// Example 10: Writing All Package Changes
heading('10. Writing All Package Changes');
info('Writing all changes back to disk:');

code(`
// In a real scenario, we would write all package.json files back to disk
workspacePkgInfos.forEach(pkgInfo => {
  console.log(\`Writing changes to \${pkgInfo.packageJsonPath}\`);
  pkgInfo.writePackageJson();
});
`);

// Simulate the write operation
const writeTable = new Table({
  head: [
    chalk.white.bold('Package'),
    chalk.white.bold('Path'),
    chalk.white.bold('Status')
  ],
  colWidths: [25, 40, 15]
});

workspacePkgInfos.forEach(pkgInfo => {
  writeTable.push([
    pkgInfo.package.name,
    pkgInfo.packageJsonPath,
    chalk.green('✓ Written')
  ]);
});

console.log(writeTable.toString());
info('(In this example, we\'re not actually writing to disk)');

// Summary
console.log(createBox('Summary',
  chalk.bold('Key Concepts Demonstrated:') + '\n\n' +
  '✅ Creating and working with PackageInfo objects\n' +
  '✅ Accessing package.json metadata\n' +
  '✅ Updating package versions and dependencies\n' +
  '✅ Writing changes back to disk\n' +
  '✅ Working with dependency resolution\n' +
  '✅ Parsing scoped package names\n' +
  '✅ Managing packages in a workspace\n' +
  '✅ Batch updating versions across workspace\n' +
  '✅ Ensuring consistent internal dependencies'
));
