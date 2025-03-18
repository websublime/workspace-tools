import chalk from 'chalk';
import boxen from 'boxen';
import Table from 'cli-table3';
import {
  Package,
  Dependency,
  DependencyUpgrader,
  createDefaultUpgradeConfig,
  createUpgradeConfigFromStrategy,
  createUpgradeConfigWithRegistries,
  ExecutionMode,
  VersionUpdateStrategy,
  VersionStability,
  UpgradeStatus,
  DependencyFilter,
  VersionUtils
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

console.log(createBox('Dependency Upgrading Example',
  chalk.bold('This example demonstrates how to find and apply dependency upgrades in your packages.')
));

// Example 1: Understanding UpgradeConfig Options
heading('1. Understanding UpgradeConfig');
info('UpgradeConfig allows you to customize how dependencies are upgraded:');

const upgradeConfigTable = new Table({
  head: [
    chalk.white.bold('Property'),
    chalk.white.bold('Type'),
    chalk.white.bold('Description')
  ],
  colWidths: [20, 25, 40]
});

upgradeConfigTable.push([
  'dependencyTypes',
  'DependencyFilter',
  'Which types of dependencies to include (production, development, optional)'
]);

upgradeConfigTable.push([
  'updateStrategy',
  'VersionUpdateStrategy',
  'Which types of version updates to consider (patch, minor, major)'
]);

upgradeConfigTable.push([
  'versionStability',
  'VersionStability',
  'Whether to include prerelease versions'
]);

upgradeConfigTable.push([
  'targetPackages',
  'string[]',
  'Specific packages to upgrade (empty = all)'
]);

upgradeConfigTable.push([
  'targetDependencies',
  'string[]',
  'Specific dependencies to upgrade (empty = all)'
]);

upgradeConfigTable.push([
  'registries',
  'string[]',
  'Additional registries to check for updates'
]);

upgradeConfigTable.push([
  'executionMode',
  'ExecutionMode',
  'Whether to apply upgrades or just report them (DryRun or Apply)'
]);

console.log(upgradeConfigTable.toString());

// Example 2: Creating Upgrade Configurations
heading('2. Creating Upgrade Configurations');
info('There are several ways to create an upgrade configuration:');

code(`
// Create a default upgrade configuration
const defaultConfig = createDefaultUpgradeConfig();

// Create an upgrade config with a specific update strategy
const minorConfig = createUpgradeConfigFromStrategy(VersionUpdateStrategy.MinorAndPatch);

// Create an upgrade config with custom registries
const registriesConfig = createUpgradeConfigWithRegistries([
  'https://registry.npmjs.org',
  'https://npm.pkg.github.com'
]);

// Create a fully customized config
const customConfig = {
  dependencyTypes: DependencyFilter.WithDevelopment,
  updateStrategy: VersionUpdateStrategy.PatchOnly,
  versionStability: VersionStability.StableOnly,
  targetPackages: ['package-a', 'package-b'], // Only these packages
  targetDependencies: ['react', 'lodash'], // Only these dependencies
  registries: ['https://registry.npmjs.org'],
  executionMode: ExecutionMode.DryRun
};
`);

try {
  // Create different configurations
  const defaultConfig = createDefaultUpgradeConfig();
  const minorConfig = createUpgradeConfigFromStrategy(VersionUpdateStrategy.MinorAndPatch);
  const registriesConfig = createUpgradeConfigWithRegistries([
    'https://registry.npmjs.org',
    'https://npm.pkg.github.com'
  ]);

  const customConfig = {
    dependencyTypes: DependencyFilter.WithDevelopment,
    updateStrategy: VersionUpdateStrategy.PatchOnly,
    versionStability: VersionStability.StableOnly,
    targetPackages: ['package-a', 'package-b'],
    targetDependencies: ['react', 'lodash'],
    registries: ['https://registry.npmjs.org'],
    executionMode: ExecutionMode.DryRun
  };

  // Display configurations
  const configsTable = new Table({
    head: [
      chalk.white.bold('Config Type'),
      chalk.white.bold('Update Strategy'),
      chalk.white.bold('Dependency Types'),
      chalk.white.bold('Execution Mode')
    ],
    colWidths: [20, 20, 25, 20]
  });

  configsTable.push([
    'Default',
    defaultConfig.updateStrategy,
    defaultConfig.dependencyTypes,
    defaultConfig.executionMode
  ]);

  configsTable.push([
    'Strategy-Based',
    minorConfig.updateStrategy,
    minorConfig.dependencyTypes,
    minorConfig.executionMode
  ]);

  configsTable.push([
    'Custom Registries',
    registriesConfig.updateStrategy,
    registriesConfig.dependencyTypes,
    registriesConfig.executionMode
  ]);

  configsTable.push([
    'Fully Custom',
    customConfig.updateStrategy,
    customConfig.dependencyTypes,
    customConfig.executionMode
  ]);

  console.log(configsTable.toString());

  // Show custom config details
  console.log(chalk.cyan('\nCustom Config Additional Properties:'));
  success(`Target Packages: ${customConfig.targetPackages.join(', ')}`);
  success(`Target Dependencies: ${customConfig.targetDependencies.join(', ')}`);
  success(`Registries: ${customConfig.registries.join(', ')}`);
  success(`Version Stability: ${customConfig.versionStability}`);
} catch (err) {
  warning(`Error creating configurations: ${err.message}`);
}

// Example 3: Creating a DependencyUpgrader
heading('3. Creating and Configuring a DependencyUpgrader');
info('A DependencyUpgrader helps you find and apply available dependency upgrades:');

code(`
// Create a dependency upgrader with default config
const defaultUpgrader = new DependencyUpgrader();

// Create with specific config
const config = createUpgradeConfigFromStrategy(VersionUpdateStrategy.MinorAndPatch);
const upgrader = DependencyUpgrader.withConfig(config);

// Update configuration later
upgrader.setConfig({
  ...config,
  executionMode: ExecutionMode.Apply, // Change to apply mode
  dependencyTypes: DependencyFilter.AllDependencies // Include optional dependencies
});
`);

try {
  // Create upgraders
  const defaultUpgrader = new DependencyUpgrader();

  const config = createUpgradeConfigFromStrategy(VersionUpdateStrategy.MinorAndPatch);
  const upgrader = DependencyUpgrader.withConfig(config);

  // Update configuration dynamically
  const updatedConfig = {
    ...config,
    executionMode: ExecutionMode.Apply,
    dependencyTypes: DependencyFilter.AllDependencies
  };

  upgrader.setConfig(updatedConfig);

  // Show upgrader configs
  info('Default Upgrader Configuration:');
  const defaultConfig = createDefaultUpgradeConfig(); // Since we can't access the instance's config
  code(JSON.stringify(defaultConfig, null, 2));

  info('\nCustom Upgrader Configuration (after update):');
  code(JSON.stringify(updatedConfig, null, 2));

  success('Dependency upgraders created and configured successfully');
} catch (err) {
  warning(`Error creating upgraders: ${err.message}`);
}

// Example 4: Understanding Upgrade Status
heading('4. Understanding Upgrade Status');
info('The UpgradeStatus enum indicates the availability of dependency upgrades:');

const upgradeStatusTable = new Table({
  head: [
    chalk.white.bold('Status'),
    chalk.white.bold('Description')
  ],
  colWidths: [20, 65]
});

upgradeStatusTable.push([
  'UpToDate',
  'The dependency is already at the latest version'
]);

upgradeStatusTable.push([
  'PatchAvailable',
  'A patch update is available (fixes only, e.g., 1.2.3 → 1.2.4)'
]);

upgradeStatusTable.push([
  'MinorAvailable',
  'A minor update is available (features, e.g., 1.2.3 → 1.3.0)'
]);

upgradeStatusTable.push([
  'MajorAvailable',
  'A major update is available (breaking changes, e.g., 1.2.3 → 2.0.0)'
]);

upgradeStatusTable.push([
  'Constrained',
  'Updates exist but are constrained by version requirements'
]);

upgradeStatusTable.push([
  'CheckFailed',
  'Failed to check for updates (network issue, etc.)'
]);

console.log(upgradeStatusTable.toString());

// Example 5: Checking for Dependency Upgrades
heading('5. Checking for Dependency Upgrades');
info('Let\'s check for available upgrades in a package:');

code(`
// Create a package with dependencies
const myPackage = new Package('my-app', '1.0.0');
myPackage.addDependency(new Dependency('react', '^16.8.0'));
myPackage.addDependency(new Dependency('lodash', '^4.17.15'));
myPackage.addDependency(new Dependency('express', '^4.17.1'));

// Create an upgrader
const upgrader = new DependencyUpgrader();

// Check for upgrades
const availableUpgrades = upgrader.checkPackageUpgrades(myPackage);
console.log(\`Found \${availableUpgrades.length} available upgrades\`);
`);

try {
  // Create a package with dependencies (using versions known to be outdated)
  const myPackage = new Package('my-app', '1.0.0');
  myPackage.addDependency(new Dependency('react', '^16.8.0'));
  myPackage.addDependency(new Dependency('lodash', '^4.17.15'));
  myPackage.addDependency(new Dependency('express', '^4.17.1'));

  // Show the package definition
  info('Package with dependencies:');
  const depTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Version'),
      chalk.white.bold('Dependencies')
    ],
    colWidths: [15, 15, 50]
  });

  const depNames = myPackage.dependencies().map(d => `${d.name}@${d.version}`);

  depTable.push([
    myPackage.name,
    myPackage.version,
    depNames.join(', ')
  ]);

  console.log(depTable.toString());

  // Create an upgrader and check for upgrades
  // In a real system, this would connect to npm registry
  // For this example, we'll mock the response
  info('\nIn a real system, the upgrader would check the npm registry for latest versions.');
  info('For this example, we\'ll simulate checking for upgrades...\n');

  // Simulate checking for upgrades
  // These are values that would typically come from the registry
  const mockUpgrades = [
    {
      packageName: 'my-app',
      dependencyName: 'react',
      currentVersion: '^16.8.0',
      compatibleVersion: '16.14.0',
      latestVersion: '18.2.0',
      status: UpgradeStatus.MajorAvailable
    },
    {
      packageName: 'my-app',
      dependencyName: 'lodash',
      currentVersion: '^4.17.15',
      compatibleVersion: '4.17.21',
      latestVersion: '4.17.21',
      status: UpgradeStatus.PatchAvailable
    },
    {
      packageName: 'my-app',
      dependencyName: 'express',
      currentVersion: '^4.17.1',
      compatibleVersion: '4.18.2',
      latestVersion: '4.18.2',
      status: UpgradeStatus.MinorAvailable
    }
  ];

  // Display the mock upgrade results
  const upgradesTable = new Table({
    head: [
      chalk.white.bold('Dependency'),
      chalk.white.bold('Current'),
      chalk.white.bold('Compatible'),
      chalk.white.bold('Latest'),
      chalk.white.bold('Status')
    ],
    colWidths: [15, 15, 15, 15, 20]
  });

  mockUpgrades.forEach(upgrade => {
    upgradesTable.push([
      upgrade.dependencyName,
      upgrade.currentVersion,
      upgrade.compatibleVersion || 'N/A',
      upgrade.latestVersion || 'N/A',
      upgrade.status
    ]);
  });

  console.log(upgradesTable.toString());

  success(`Found ${mockUpgrades.length} available upgrades`);
} catch (err) {
  warning(`Error checking for upgrades: ${err.message}`);
}

// Example 6: Checking Multiple Packages
heading('6. Checking Multiple Packages for Upgrades');
info('You can also check for upgrades across multiple packages at once:');

code(`
// Create multiple packages with dependencies
const packageA = new Package('package-a', '1.0.0');
packageA.addDependency(new Dependency('react', '^16.8.0'));
packageA.addDependency(new Dependency('redux', '^4.0.0'));

const packageB = new Package('package-b', '1.0.0');
packageB.addDependency(new Dependency('react', '^16.9.0')); // Different version
packageB.addDependency(new Dependency('lodash', '^4.17.15'));

// Check all packages at once
const upgrader = new DependencyUpgrader();
const allUpgrades = upgrader.checkAllUpgrades([packageA, packageB]);

// Generate a report
const report = upgrader.generateUpgradeReport(allUpgrades);
console.log(report);
`);

try {
  // Create multiple packages with dependencies
  const packageA = new Package('package-a', '1.0.0');
  packageA.addDependency(new Dependency('react', '^16.8.0'));
  packageA.addDependency(new Dependency('redux', '^4.0.0'));

  const packageB = new Package('package-b', '1.0.0');
  packageB.addDependency(new Dependency('react', '^16.9.0')); // Different version
  packageB.addDependency(new Dependency('lodash', '^4.17.15'));

  // Show the packages
  info('Multiple packages with dependencies:');
  const packagesTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Version'),
      chalk.white.bold('Dependencies')
    ],
    colWidths: [15, 15, 50]
  });

  const depNamesA = packageA.dependencies().map(d => `${d.name}@${d.version}`);
  const depNamesB = packageB.dependencies().map(d => `${d.name}@${d.version}`);

  packagesTable.push([
    packageA.name,
    packageA.version,
    depNamesA.join(', ')
  ]);

  packagesTable.push([
    packageB.name,
    packageB.version,
    depNamesB.join(', ')
  ]);

  console.log(packagesTable.toString());

  // Similar to previous example, we'll mock the upgrades
  info('\nSimulating checking for upgrades across all packages...\n');

  // Simulate checking for upgrades
  const mockMultiUpgrades = [
    {
      packageName: 'package-a',
      dependencyName: 'react',
      currentVersion: '^16.8.0',
      compatibleVersion: '16.14.0',
      latestVersion: '18.2.0',
      status: UpgradeStatus.MajorAvailable
    },
    {
      packageName: 'package-a',
      dependencyName: 'redux',
      currentVersion: '^4.0.0',
      compatibleVersion: '4.2.1',
      latestVersion: '4.2.1',
      status: UpgradeStatus.MinorAvailable
    },
    {
      packageName: 'package-b',
      dependencyName: 'react',
      currentVersion: '^16.9.0',
      compatibleVersion: '16.14.0',
      latestVersion: '18.2.0',
      status: UpgradeStatus.MajorAvailable
    },
    {
      packageName: 'package-b',
      dependencyName: 'lodash',
      currentVersion: '^4.17.15',
      compatibleVersion: '4.17.21',
      latestVersion: '4.17.21',
      status: UpgradeStatus.PatchAvailable
    }
  ];

  // Display the mock upgrade results
  const multiUpgradesTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Dependency'),
      chalk.white.bold('Current'),
      chalk.white.bold('Compatible'),
      chalk.white.bold('Status')
    ],
    colWidths: [15, 15, 15, 15, 20]
  });

  mockMultiUpgrades.forEach(upgrade => {
    multiUpgradesTable.push([
      upgrade.packageName,
      upgrade.dependencyName,
      upgrade.currentVersion,
      upgrade.compatibleVersion || 'N/A',
      upgrade.status
    ]);
  });

  console.log(multiUpgradesTable.toString());

  // Simulate generating a report
  const upgrader = new DependencyUpgrader(); // Just for the report generation
  // In a real system, this would analyze and format the upgrades

  // Mock report generation with a formatted text report
  const mockReport =
    `
  Dependency Upgrade Report
  =========================

  Found 4 available upgrades across 2 packages:

  Package: package-a
  -----------------
    react: ^16.8.0 → 16.14.0 (Major update to 18.2.0 available)
    redux: ^4.0.0 → 4.2.1 (Minor update)

  Package: package-b
  -----------------
    react: ^16.9.0 → 16.14.0 (Major update to 18.2.0 available)
    lodash: ^4.17.15 → 4.17.21 (Patch update)

  Summary:
  - Major updates: 2 (react)
  - Minor updates: 1 (redux)
  - Patch updates: 1 (lodash)
  `;

  console.log(boxen(mockReport, {
    padding: 1,
    margin: 1,
    borderStyle: 'round',
    borderColor: 'yellow',
    title: 'Upgrade Report',
    titleAlignment: 'center'
  }));

  success('Dependency analysis completed across all packages');
} catch (err) {
  warning(`Error in multi-package analysis: ${err.message}`);
}

// Example 7: Applying Dependency Upgrades
heading('7. Applying Dependency Upgrades');
info('Once you\'ve identified upgrades, you can apply them to your packages:');

code(`
  // Create an upgrader with Apply mode
  const config = {
    ...createDefaultUpgradeConfig(),
    executionMode: ExecutionMode.Apply // Set to apply changes
  };

  const upgrader = DependencyUpgrader.withConfig(config);

  // Check for upgrades
  const availableUpgrades = upgrader.checkPackageUpgrades(myPackage);

  // Apply the upgrades
  const appliedUpgrades = upgrader.applyUpgrades([myPackage], availableUpgrades);
  console.log(\`Applied \${appliedUpgrades.length} upgrades\`);

  // Now the package has updated dependencies
  myPackage.dependencies().forEach(dep => {
    console.log(\`\${dep.name}: \${dep.version}\`);
  });
  `);

try {
  // Create a package with outdated dependencies
  const appPackage = new Package('my-webapp', '2.0.0');
  appPackage.addDependency(new Dependency('react', '^16.8.0'));
  appPackage.addDependency(new Dependency('lodash', '^4.17.15'));

  // Show the initial state
  info('Initial package dependencies:');
  const initialDepsTable = new Table({
    head: [
      chalk.white.bold('Dependency'),
      chalk.white.bold('Current Version')
    ],
    colWidths: [20, 20]
  });

  appPackage.dependencies().forEach(dep => {
    initialDepsTable.push([dep.name, dep.version]);
  });

  console.log(initialDepsTable.toString());

  // Simulate applying upgrades
  info('\nSimulating upgrade application...\n');

  // Mock available upgrades
  const mockAvailableUpgrades = [
    {
      packageName: 'my-webapp',
      dependencyName: 'react',
      currentVersion: '^16.8.0',
      compatibleVersion: '16.14.0',
      latestVersion: '18.2.0',
      status: UpgradeStatus.MajorAvailable
    },
    {
      packageName: 'my-webapp',
      dependencyName: 'lodash',
      currentVersion: '^4.17.15',
      compatibleVersion: '4.17.21',
      latestVersion: '4.17.21',
      status: UpgradeStatus.PatchAvailable
    }
  ];

  // Simulate applying upgrades by directly updating the dependencies
  appPackage.dependencies().forEach(dep => {
    const upgrade = mockAvailableUpgrades.find(u => u.dependencyName === dep.name);
    if (upgrade && upgrade.compatibleVersion) {
      const newVersion = `^${upgrade.compatibleVersion}`;
      dep.updateVersion(newVersion);
    }
  });

  // Show the result after applying upgrades
  info('Package dependencies after applying upgrades:');
  const updatedDepsTable = new Table({
    head: [
      chalk.white.bold('Dependency'),
      chalk.white.bold('Updated Version')
    ],
    colWidths: [20, 20]
  });

  appPackage.dependencies().forEach(dep => {
    updatedDepsTable.push([dep.name, dep.version]);
  });

  console.log(updatedDepsTable.toString());

  success(`Applied ${mockAvailableUpgrades.length} upgrades successfully`);
} catch (err) {
  warning(`Error applying upgrades: ${err.message}`);
}

// Example 8: Real-World Scenario - Monorepo Dependency Management
heading('8. Real-World Scenario: Monorepo Dependency Management');
info('Manage dependencies across multiple packages in a monorepo:');

code(`
  // Define our monorepo packages
  const monorepoPackages = [
    { name: '@org/core', version: '1.0.0', deps: { lodash: '^4.17.15', axios: '^0.21.1' } },
    { name: '@org/ui', version: '1.0.0', deps: { '@org/core': '^1.0.0', react: '^16.8.0', 'styled-components': '^5.1.0' } },
    { name: '@org/api', version: '1.0.0', deps: { '@org/core': '^1.0.0', express: '^4.17.1', mongoose: '^5.9.7' } },
    { name: '@org/web', version: '1.0.0', deps: { '@org/ui': '^1.0.0', '@org/api': '^1.0.0', next: '^10.0.0' } }
  ];

  // Create Package objects
  const packages = monorepoPackages.map(pkg => {
    const p = new Package(pkg.name, pkg.version);
    Object.entries(pkg.deps).forEach(([name, version]) => {
      p.addDependency(new Dependency(name, version));
    });
    return p;
  });

  // Create an upgrader with a strategy that only allows patch and minor updates
  const safeConfig = createUpgradeConfigFromStrategy(VersionUpdateStrategy.MinorAndPatch);
  const upgrader = DependencyUpgrader.withConfig({
    ...safeConfig,
    // We want to upgrade everything except our own internal packages
    targetDependencies: [], // Empty means all non-excluded
    // We don't want to upgrade our internal dependencies automatically
    // as they might need special handling
    executionMode: ExecutionMode.DryRun
  });

  // Check for upgrades across all packages
  const allUpgrades = upgrader.checkAllUpgrades(packages);

  // Filter out internal dependencies
  const externalUpgrades = allUpgrades.filter(upgrade => !upgrade.dependencyName.startsWith('@org/'));

  // Generate a report for the team to review
  const report = upgrader.generateUpgradeReport(externalUpgrades);
  console.log(report);

  // After team review, apply the approved upgrades
  const approvedUpgrades = externalUpgrades.filter(upgrade =>
    // For example, approve all patch updates automatically
    upgrade.status === UpgradeStatus.PatchAvailable
  );

  // Apply only the approved upgrades
  const appliedUpgrades = upgrader.applyUpgrades(packages, approvedUpgrades);
  `);

try {
  // Define our monorepo packages
  const monorepoPackages = [
    { name: '@org/core', version: '1.0.0', deps: { lodash: '^4.17.15', axios: '^0.21.1' } },
    { name: '@org/ui', version: '1.0.0', deps: { '@org/core': '^1.0.0', react: '^16.8.0', 'styled-components': '^5.1.0' } },
    { name: '@org/api', version: '1.0.0', deps: { '@org/core': '^1.0.0', express: '^4.17.1', mongoose: '^5.9.7' } },
    { name: '@org/web', version: '1.0.0', deps: { '@org/ui': '^1.0.0', '@org/api': '^1.0.0', next: '^10.0.0' } }
  ];

  // Create Package objects
  const packages = monorepoPackages.map(pkg => {
    const p = new Package(pkg.name, pkg.version);
    Object.entries(pkg.deps).forEach(([name, version]) => {
      p.addDependency(new Dependency(name, version));
    });
    return p;
  });

  // Show the monorepo structure
  info('Monorepo Package Structure:');
  const monorepoTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Version'),
      chalk.white.bold('Dependencies')
    ],
    colWidths: [15, 10, 55]
  });

  packages.forEach(pkg => {
    const depString = pkg.dependencies()
      .map(d => `${d.name}@${d.version}`)
      .join(', ');

    monorepoTable.push([
      pkg.name,
      pkg.version,
      depString
    ]);
  });

  console.log(monorepoTable.toString());

  // Simulate checking for upgrades
  info('\nSimulating dependency analysis across the monorepo...\n');

  // Mock upgrades for demonstration
  const mockMonorepoUpgrades = [
    {
      packageName: '@org/core',
      dependencyName: 'lodash',
      currentVersion: '^4.17.15',
      compatibleVersion: '4.17.21',
      latestVersion: '4.17.21',
      status: UpgradeStatus.PatchAvailable
    },
    {
      packageName: '@org/core',
      dependencyName: 'axios',
      currentVersion: '^0.21.1',
      compatibleVersion: '0.27.2',
      latestVersion: '1.3.4',
      status: UpgradeStatus.MajorAvailable
    },
    {
      packageName: '@org/ui',
      dependencyName: 'react',
      currentVersion: '^16.8.0',
      compatibleVersion: '16.14.0',
      latestVersion: '18.2.0',
      status: UpgradeStatus.MajorAvailable
    },
    {
      packageName: '@org/ui',
      dependencyName: 'styled-components',
      currentVersion: '^5.1.0',
      compatibleVersion: '5.3.9',
      latestVersion: '5.3.9',
      status: UpgradeStatus.MinorAvailable
    },
    {
      packageName: '@org/api',
      dependencyName: 'express',
      currentVersion: '^4.17.1',
      compatibleVersion: '4.18.2',
      latestVersion: '4.18.2',
      status: UpgradeStatus.MinorAvailable
    },
    {
      packageName: '@org/api',
      dependencyName: 'mongoose',
      currentVersion: '^5.9.7',
      compatibleVersion: '5.13.15',
      latestVersion: '7.1.0',
      status: UpgradeStatus.MajorAvailable
    },
    {
      packageName: '@org/web',
      dependencyName: 'next',
      currentVersion: '^10.0.0',
      compatibleVersion: '10.2.3',
      latestVersion: '13.4.3',
      status: UpgradeStatus.MajorAvailable
    }
  ];

  // Filter out internal dependencies (@org/* packages)
  const externalUpgrades = mockMonorepoUpgrades.filter(
    upgrade => !upgrade.dependencyName.startsWith('@org/')
  );

  // Display all available upgrades
  info('Available External Dependency Upgrades:');
  const externalUpgradesTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Dependency'),
      chalk.white.bold('Current'),
      chalk.white.bold('Compatible'),
      chalk.white.bold('Latest'),
      chalk.white.bold('Status'),
      chalk.white.bold('Auto-Approve')
    ],
    colWidths: [15, 20, 15, 15, 15, 15, 15]
  });

  externalUpgrades.forEach(upgrade => {
    const autoApprove = upgrade.status === UpgradeStatus.PatchAvailable ? 'Yes' : 'No';
    externalUpgradesTable.push([
      upgrade.packageName,
      upgrade.dependencyName,
      upgrade.currentVersion,
      upgrade.compatibleVersion || 'N/A',
      upgrade.latestVersion || 'N/A',
      upgrade.status.toString(),
      autoApprove
    ]);
  });

  console.log(externalUpgradesTable.toString());

  // Filter for auto-approved upgrades (only patch updates in this scenario)
  const approvedUpgrades = externalUpgrades.filter(
    upgrade => upgrade.status === UpgradeStatus.PatchAvailable
  );

  info(`\nAuto-approved ${approvedUpgrades.length} out of ${externalUpgrades.length} upgrades`);

  // Simulate applying the approved upgrades
  info('\nSimulating applying approved upgrades...');

  // Show which upgrades would be applied
  const approvedTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Dependency'),
      chalk.white.bold('From'),
      chalk.white.bold('To')
    ],
    colWidths: [15, 20, 15, 15]
  });

  approvedUpgrades.forEach(upgrade => {
    approvedTable.push([
      upgrade.packageName,
      upgrade.dependencyName,
      upgrade.currentVersion,
      `^${upgrade.compatibleVersion}`
    ]);
  });

  console.log(approvedTable.toString());

  // Update the package objects for approved upgrades
  approvedUpgrades.forEach(upgrade => {
    const pkg = packages.find(p => p.name === upgrade.packageName);
    if (pkg) {
      const dep = pkg.getDependency(upgrade.dependencyName);
      if (dep) {
        dep.updateVersion(`^${upgrade.compatibleVersion}`);
      }
    }
  });

  // Show the monorepo packages after applying upgrades
  info('\nMonorepo Package Structure After Updates:');
  const updatedMonorepoTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Version'),
      chalk.white.bold('Dependencies')
    ],
    colWidths: [15, 10, 55]
  });

  packages.forEach(pkg => {
    const depString = pkg.dependencies()
      .map(d => `${d.name}@${d.version}`)
      .join(', ');

    updatedMonorepoTable.push([
      pkg.name,
      pkg.version,
      depString
    ]);
  });

  console.log(updatedMonorepoTable.toString());

  success(`\nSuccessfully applied ${approvedUpgrades.length} approved dependency upgrades`);
} catch (err) {
  warning(`Error in monorepo dependency management: ${err.message}`);
}

// Example 9: Customizing Upgrade Strategies
heading('9. Customizing Upgrade Strategies');
info('You can create custom upgrade strategies for different use cases:');

code(`
    // Create different strategies for different situations

    // 1. Safe strategy for production packages (patch updates only)
    const safeStrategy = {
      dependencyTypes: DependencyFilter.ProductionOnly, // Only production dependencies
      updateStrategy: VersionUpdateStrategy.PatchOnly, // Only patch updates
      versionStability: VersionStability.StableOnly, // No prereleases
      executionMode: ExecutionMode.DryRun // Always manual review first
    };

    // 2. Development strategy for developer tooling (minor updates allowed)
    const devToolsStrategy = {
      dependencyTypes: DependencyFilter.WithDevelopment, // Include dev dependencies
      updateStrategy: VersionUpdateStrategy.MinorAndPatch, // Minor updates OK
      versionStability: VersionStability.StableOnly, // No prereleases
      executionMode: ExecutionMode.Apply // Can auto-apply
    };

    // 3. Edge strategy for experimental features (all updates, including prereleases)
    const edgeStrategy = {
      dependencyTypes: DependencyFilter.AllDependencies, // All dependency types
      updateStrategy: VersionUpdateStrategy.AllUpdates, // Major updates OK
      versionStability: VersionStability.IncludePrerelease, // Prereleases OK
      executionMode: ExecutionMode.DryRun // Review first
    };

    // Apply different strategies to different packages
    function upgradeWithStrategy(pkg, strategy) {
      const upgrader = DependencyUpgrader.withConfig(strategy);
      const upgrades = upgrader.checkPackageUpgrades(pkg);

      if (strategy.executionMode === ExecutionMode.Apply) {
        upgrader.applyUpgrades([pkg], upgrades);
      }

      return upgrades;
    }

    // Use in your workflow
    const coreLib = new Package('@product/core', '2.0.0');
    const devTools = new Package('@product/dev-tools', '1.5.0');
    const experimental = new Package('@product/experimental', '0.1.0');

    const coreUpgrades = upgradeWithStrategy(coreLib, safeStrategy);
    const devToolsUpgrades = upgradeWithStrategy(devTools, devToolsStrategy);
    const experimentalUpgrades = upgradeWithStrategy(experimental, edgeStrategy);
    `);

try {
  // Define various upgrade strategies
  const safeStrategy = {
    dependencyTypes: DependencyFilter.ProductionOnly, // Only production dependencies
    updateStrategy: VersionUpdateStrategy.PatchOnly, // Only patch updates
    versionStability: VersionStability.StableOnly, // No prereleases
    targetPackages: [],
    targetDependencies: [],
    registries: ['https://registry.npmjs.org'],
    executionMode: ExecutionMode.DryRun // Always manual review first
  };

  const devToolsStrategy = {
    dependencyTypes: DependencyFilter.WithDevelopment, // Include dev dependencies
    updateStrategy: VersionUpdateStrategy.MinorAndPatch, // Minor updates OK
    versionStability: VersionStability.StableOnly, // No prereleases
    targetPackages: [],
    targetDependencies: [],
    registries: ['https://registry.npmjs.org'],
    executionMode: ExecutionMode.Apply // Can auto-apply
  };

  const edgeStrategy = {
    dependencyTypes: DependencyFilter.AllDependencies, // All dependency types
    updateStrategy: VersionUpdateStrategy.AllUpdates, // Major updates OK
    versionStability: VersionStability.IncludePrerelease, // Prereleases OK
    targetPackages: [],
    targetDependencies: [],
    registries: ['https://registry.npmjs.org'],
    executionMode: ExecutionMode.DryRun // Review first
  };

  // Compare the strategies
  const strategyTable = new Table({
    head: [
      chalk.white.bold('Strategy'),
      chalk.white.bold('Dependencies'),
      chalk.white.bold('Updates'),
      chalk.white.bold('Prereleases'),
      chalk.white.bold('Mode')
    ],
    colWidths: [15, 20, 15, 15, 15]
  });

  strategyTable.push([
    'Safe',
    'Production Only',
    'Patch Only',
    'No',
    'Dry Run'
  ]);

  strategyTable.push([
    'Dev Tools',
    'With Development',
    'Minor & Patch',
    'No',
    'Apply'
  ]);

  strategyTable.push([
    'Edge',
    'All',
    'All Updates',
    'Yes',
    'Dry Run'
  ]);

  console.log(strategyTable.toString());

  // Demonstrate applying different strategies

  info('\nUse Case Examples:');

  success('Safe Strategy:');
  success('- Core libraries with many dependents');
  success('- Production services with high reliability requirements');
  success('- Shared infrastructure components');

  info('Dev Tools Strategy:');
  info('- Build tools (webpack, babel, etc.)');
  info('- Test frameworks (jest, mocha, etc.)');
  info('- Linters and formatters (eslint, prettier, etc.)');

  warning('Edge Strategy:');
  warning('- Experimental features');
  warning('- Technology evaluations');
  warning('- Proof of concept projects');

  success('\nBy applying different strategies to different packages,');
  success('you can balance stability and innovation across your projects.');
} catch (err) {
  warning(`Error with upgrade strategies: ${err.message}`);
}

// Example 10: Advanced Version Utilities
heading('10. Advanced Version Utilities');
info('The VersionUtils class provides helpful methods for version management:');

code(`
// Version comparison and manipulation with VersionUtils
const v1 = '1.2.3';
const v2 = '1.3.0';
const v3 = '2.0.0';
const v4 = '1.2.3-beta.1';

// Compare versions to understand their relationship
const comp1 = VersionUtils.compareVersions(v1, v2);
console.log(\`\${v1} to \${v2}: \${comp1}\`); // MinorUpgrade

const comp2 = VersionUtils.compareVersions(v1, v3);
console.log(\`\${v1} to \${v3}: \${comp2}\`); // MajorUpgrade

const comp3 = VersionUtils.compareVersions(v1, v4);
console.log(\`\${v1} to \${v4}: \${comp3}\`); // StableToPrerelease

// Check if updates are breaking changes
console.log(\`Is \${v1} to \${v2} breaking? \${VersionUtils.isBreakingChange(v1, v2)}\`); // false
console.log(\`Is \${v1} to \${v3} breaking? \${VersionUtils.isBreakingChange(v1, v3)}\`); // true

// Bump versions
console.log(\`Bump major of \${v1}: \${VersionUtils.bumpMajor(v1)}\`); // 2.0.0
console.log(\`Bump minor of \${v1}: \${VersionUtils.bumpMinor(v1)}\`); // 1.3.0
console.log(\`Bump patch of \${v1}: \${VersionUtils.bumpPatch(v1)}\`); // 1.2.4
console.log(\`Bump snapshot of \${v1}: \${VersionUtils.bumpSnapshot(v1, 'abc123')}\`); // 1.2.3-alpha.abc123
`);

try {
  // Version comparison and manipulation with VersionUtils
  const v1 = '1.2.3';
  const v2 = '1.3.0';
  const v3 = '2.0.0';
  const v4 = '1.2.3-beta.1';

  // Create a table for version comparisons
  const comparisonTable = new Table({
    head: [
      chalk.white.bold('From'),
      chalk.white.bold('To'),
      chalk.white.bold('Comparison Result'),
      chalk.white.bold('Breaking Change?')
    ],
    colWidths: [10, 15, 25, 20]
  });

  // Compare versions
  const comp1 = VersionUtils.compareVersions(v1, v2);
  const comp2 = VersionUtils.compareVersions(v1, v3);
  const comp3 = VersionUtils.compareVersions(v1, v4);
  const comp4 = VersionUtils.compareVersions(v3, v1);

  comparisonTable.push([
    v1, v2, comp1, VersionUtils.isBreakingChange(v1, v2) ? 'Yes' : 'No'
  ]);

  comparisonTable.push([
    v1, v3, comp2, VersionUtils.isBreakingChange(v1, v3) ? 'Yes' : 'No'
  ]);

  comparisonTable.push([
    v1, v4, comp3, VersionUtils.isBreakingChange(v1, v4) ? 'Yes' : 'No'
  ]);

  comparisonTable.push([
    v3, v1, comp4, VersionUtils.isBreakingChange(v3, v1) ? 'Yes' : 'No'
  ]);

  console.log(comparisonTable.toString());

  // Create a table for version bumping
  const bumpTable = new Table({
    head: [
      chalk.white.bold('Method'),
      chalk.white.bold('Original'),
      chalk.white.bold('Result')
    ],
    colWidths: [30, 15, 25]
  });

  // Bump versions
  bumpTable.push(['VersionUtils.bumpMajor', v1, VersionUtils.bumpMajor(v1)]);
  bumpTable.push(['VersionUtils.bumpMinor', v1, VersionUtils.bumpMinor(v1)]);
  bumpTable.push(['VersionUtils.bumpPatch', v1, VersionUtils.bumpPatch(v1)]);
  bumpTable.push(['VersionUtils.bumpSnapshot (abc123)', v1, VersionUtils.bumpSnapshot(v1, 'abc123')]);

  // Demonstrate bump with prerelease version
  bumpTable.push(['VersionUtils.bumpMajor (prerelease)', v4, VersionUtils.bumpMajor(v4)]);
  bumpTable.push(['VersionUtils.bumpMinor (prerelease)', v4, VersionUtils.bumpMinor(v4)]);

  console.log(bumpTable.toString());

  // Practical example with VersionUtils
  subHeading('Practical Example with VersionUtils');
  info('Let\'s use VersionUtils to help with dependency upgrades:');

  code(`
  // Determine if an upgrade should be automatically applied based on semver
  function shouldAutoApprove(currentVersion, newVersion) {
    const comparison = VersionUtils.compareVersions(currentVersion, newVersion);
    const strippedCurrent = currentVersion.replace(/^\\^|~/, '');
    const strippedNew = newVersion.replace(/^\\^|~/, '');

    // Auto-approve patch updates
    if (comparison === VersionComparisonResult.PatchUpgrade) {
      return true;
    }

    // Auto-approve minor updates for 0.x versions (0.x is considered unstable)
    if (strippedCurrent.startsWith('0.') && comparison === VersionComparisonResult.MinorUpgrade) {
      return true;
    }

    return false;
  }

  // Example dependencies
  const dependencies = [
    { name: 'stable-lib', current: '1.2.3', latest: '1.2.5' },
    { name: 'feature-lib', current: '2.1.0', latest: '2.2.0' },
    { name: 'major-change', current: '3.0.0', latest: '4.0.0' },
    { name: 'unstable-lib', current: '0.5.1', latest: '0.6.0' }
  ];

  // Process each dependency
  dependencies.forEach(dep => {
    const autoApprove = shouldAutoApprove(dep.current, dep.latest);
    console.log(
      \`\${dep.name}: \${dep.current} -> \${dep.latest} | \` +
      \`Type: \${VersionUtils.compareVersions(dep.current, dep.latest)} | \` +
      \`Auto-approve: \${autoApprove ? 'Yes' : 'No'}\`
    );
  });
  `);

  // Practical example implementation
  function shouldAutoApprove(currentVersion, newVersion) {
    const comparison = VersionUtils.compareVersions(currentVersion, newVersion);
    const strippedCurrent = currentVersion.replace(/^\^|~/, '');

    // Auto-approve patch updates
    if (comparison === 'PatchUpgrade') {
      return true;
    }

    // Auto-approve minor updates for 0.x versions (0.x is considered unstable)
    if (strippedCurrent.startsWith('0.') && comparison === 'MinorUpgrade') {
      return true;
    }

    return false;
  }

  // Example dependencies
  const dependencies = [
    { name: 'stable-lib', current: '1.2.3', latest: '1.2.5' },
    { name: 'feature-lib', current: '2.1.0', latest: '2.2.0' },
    { name: 'major-change', current: '3.0.0', latest: '4.0.0' },
    { name: 'unstable-lib', current: '0.5.1', latest: '0.6.0' }
  ];

  // Display results in a table
  const approvalTable = new Table({
    head: [
      chalk.white.bold('Dependency'),
      chalk.white.bold('Current'),
      chalk.white.bold('Latest'),
      chalk.white.bold('Change Type'),
      chalk.white.bold('Auto-approve')
    ],
    colWidths: [15, 15, 15, 20, 15]
  });

  // Process each dependency
  dependencies.forEach(dep => {
    const autoApprove = shouldAutoApprove(dep.current, dep.latest);
    const comparison = VersionUtils.compareVersions(dep.current, dep.latest);

    approvalTable.push([
      dep.name,
      dep.current,
      dep.latest,
      comparison,
      autoApprove ? 'Yes' : 'No'
    ]);
  });

  console.log(approvalTable.toString());

  // Additional useful VersionUtils example: Sort versions
  subHeading('Sorting Versions with VersionUtils');

  const unsortedVersions = ['1.0.0', '1.10.0', '1.2.0', '2.0.0', '0.9.1', '1.1.0-beta.1', '1.0.0-rc.1'];

  info('Unsorted versions:');
  console.log(unsortedVersions.join(', '));

  // Sort versions assuming compareVersions returns numeric enum values
  const sortedVersions = [...unsortedVersions].sort((a, b) => {
    // Get the comparison result
    const comparison = VersionUtils.compareVersions(a, b);

    // If comparison result is less than 5 (MajorUpgrade through NewerPrerelease),
    // version a is older than b
    if (comparison >= 0 && comparison <= 4) return -1;

    // If comparison result is greater than 5 (MajorDowngrade through OlderPrerelease),
    // version a is newer than b
    if (comparison >= 6 && comparison <= 10) return 1;

    // If comparison is Identical (5) or Indeterminate (11) or any other value
    return 0;
  });

  info('Sorted versions (oldest to newest):');
  console.log(sortedVersions.join(', '));

  success('\nVersionUtils provides powerful capabilities for semver comparison and manipulation');
} catch (err) {
  warning(`Error in VersionUtils examples: ${err.message}`);
}

// Final overview and conclusion
heading('Conclusion');
info('In this example, you\'ve learned how to:');
success('✓ Create and configure dependency upgraders');
success('✓ Understand different upgrade configuration options');
success('✓ Check for available dependency upgrades in packages');
success('✓ Analyze upgrades across multiple packages');
success('✓ Apply selected upgrades to your packages');
success('✓ Create custom upgrade strategies for different use cases');
success('✓ Manage dependencies in monorepo environments');

console.log(createBox('Next Steps',
  chalk.cyan('1. Integrate dependency upgrading into your CI/CD pipeline\n') +
  chalk.cyan('2. Create custom upgrade policies for different types of packages\n') +
  chalk.cyan('3. Set up automated pull requests for dependency updates')
));

console.log('\n');
