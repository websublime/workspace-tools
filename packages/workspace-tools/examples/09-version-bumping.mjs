import chalk from 'chalk';
import boxen from 'boxen';
import Table from 'cli-table3';
import {
  Version,
  VersionUtils,
  bumpVersion,
  bumpSnapshotVersion,
  Package,
  Dependency
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

console.log(createBox('Version Bumping Example',
  chalk.bold('This example demonstrates version bumping capabilities for packages and dependencies.')
));

// Example 1: Basic Version Types
heading('1. Understanding Version Types');
info('The Version enum represents different types of version bumps:');

const versionTable = new Table({
  head: [
    chalk.white.bold('Version Type'),
    chalk.white.bold('Description'),
    chalk.white.bold('Example')
  ],
  colWidths: [15, 40, 20]
});

versionTable.push([
  'Major',
  'Increments the major version number (1.0.0 → 2.0.0)',
  '1.2.3 → 2.0.0'
]);

versionTable.push([
  'Minor',
  'Increments the minor version number (1.0.0 → 1.1.0)',
  '1.2.3 → 1.3.0'
]);

versionTable.push([
  'Patch',
  'Increments the patch version number (1.0.0 → 1.0.1)',
  '1.2.3 → 1.2.4'
]);

versionTable.push([
  'Snapshot',
  'Creates a snapshot version with a SHA',
  '1.2.3 → 1.2.3-snapshot.abc123'
]);

console.log(versionTable.toString());

// Example 2: Direct Version Bumping
heading('2. Direct Version Bumping');
info('The bumpVersion function lets you increment a version string according to a specified type:');

code(`
// Bump a version to the next major version
const majorBump = bumpVersion('1.2.3', Version.Major);
console.log(majorBump); // 2.0.0

// Bump a version to the next minor version
const minorBump = bumpVersion('1.2.3', Version.Minor);
console.log(minorBump); // 1.3.0

// Bump a version to the next patch version
const patchBump = bumpVersion('1.2.3', Version.Patch);
console.log(patchBump); // 1.2.4
`);

try {
  // Demonstrate version bumping
  const majorBump = bumpVersion('1.2.3', Version.Major);
  const minorBump = bumpVersion('1.2.3', Version.Minor);
  const patchBump = bumpVersion('1.2.3', Version.Patch);

  const bumpTable = new Table({
    head: [
      chalk.white.bold('Bump Type'),
      chalk.white.bold('Original'),
      chalk.white.bold('Result')
    ],
    colWidths: [15, 15, 15]
  });

  bumpTable.push(['Major', '1.2.3', majorBump]);
  bumpTable.push(['Minor', '1.2.3', minorBump]);
  bumpTable.push(['Patch', '1.2.3', patchBump]);

  console.log(bumpTable.toString());

  success('Version bumping performed successfully');
} catch (err) {
  warning(`Error during version bumping: ${err.message}`);
}

// Example 3: Snapshot Version Bumping
heading('3. Creating Snapshot Versions');
info('Snapshot versions are special prerelease versions often used for CI/CD builds:');

code(`
// Create a snapshot version using a specific SHA
const snapshotVersion = bumpSnapshotVersion('1.2.3', 'abc123');
console.log(snapshotVersion); // 1.2.3-alpha.abc123

// You can also use the Version enum with bumpVersion
const snapshotBump = bumpVersion('1.2.3', Version.Snapshot);
console.log(snapshotBump); // 1.2.3-alpha.HEAD
`);

try {
  // Demonstrate snapshot version creation
  const snapshotVersion = bumpSnapshotVersion('1.2.3', 'abc123');
  const snapshotBump = bumpVersion('1.2.3', Version.Snapshot);

  const snapshotTable = new Table({
    head: [
      chalk.white.bold('Method'),
      chalk.white.bold('Original'),
      chalk.white.bold('SHA'),
      chalk.white.bold('Result')
    ],
    colWidths: [25, 15, 15, 25]
  });

  snapshotTable.push(['bumpSnapshotVersion', '1.2.3', 'abc123', snapshotVersion]);
  snapshotTable.push(['bumpVersion', '1.2.3', 'HEAD (default)', snapshotBump]);

  console.log(snapshotTable.toString());

  info('Snapshot versions are useful for:');
  success('- Continuous Integration builds');
  success('- Pre-release testing');
  success('- Identifying specific commits in your versioning');
} catch (err) {
  warning(`Error during snapshot creation: ${err.message}`);
}

// Example 4: VersionUtils Static Methods
heading('4. Using VersionUtils');
info('The VersionUtils class provides additional version-related utilities:');

code(`
// Bump versions using VersionUtils static methods
const majorUtil = VersionUtils.bumpMajor('1.2.3');
const minorUtil = VersionUtils.bumpMinor('1.2.3');
const patchUtil = VersionUtils.bumpPatch('1.2.3');
const snapshotUtil = VersionUtils.bumpSnapshot('1.2.3', 'def456');

// Additional utilities
const isBreaking = VersionUtils.isBreakingChange('1.2.3', '2.0.0');
console.log(isBreaking); // true (major version change is breaking)

const comparison = VersionUtils.compareVersions('1.2.3', '1.3.0');
console.log(comparison); // VersionComparisonResult.MinorUpgrade
`);

try {
  // Demonstrate VersionUtils methods
  const majorUtil = VersionUtils.bumpMajor('1.2.3');
  const minorUtil = VersionUtils.bumpMinor('1.2.3');
  const patchUtil = VersionUtils.bumpPatch('1.2.3');
  const snapshotUtil = VersionUtils.bumpSnapshot('1.2.3', 'def456');

  const utilTable = new Table({
    head: [
      chalk.white.bold('Method'),
      chalk.white.bold('Original'),
      chalk.white.bold('Result')
    ],
    colWidths: [25, 15, 25]
  });

  utilTable.push(['VersionUtils.bumpMajor', '1.2.3', majorUtil]);
  utilTable.push(['VersionUtils.bumpMinor', '1.2.3', minorUtil]);
  utilTable.push(['VersionUtils.bumpPatch', '1.2.3', patchUtil]);
  utilTable.push(['VersionUtils.bumpSnapshot', '1.2.3', snapshotUtil]);

  console.log(utilTable.toString());

  // Breaking change check
  const isBreaking = VersionUtils.isBreakingChange('1.2.3', '2.0.0');
  success(`Is 1.2.3 → 2.0.0 a breaking change? ${isBreaking ? 'Yes' : 'No'}`);

  const isMinorBreaking = VersionUtils.isBreakingChange('1.2.3', '1.3.0');
  success(`Is 1.2.3 → 1.3.0 a breaking change? ${isMinorBreaking ? 'Yes' : 'No'}`);

  // Version comparison
  const comparison = VersionUtils.compareVersions('1.2.3', '1.3.0');
  success(`Comparison of 1.2.3 and 1.3.0: ${comparison}`);
} catch (err) {
  warning(`Error using VersionUtils: ${err.message}`);
}

// Example 5: Bumping Package Versions
heading('5. Bumping Package Versions');
info('You can update the version of Package objects:');

code(`
// Create a package
const pkg = new Package('example-package', '1.0.0');
console.log(pkg.version); // 1.0.0

// Update the version using bump functions
const newVersion = bumpVersion(pkg.version, Version.Minor);
pkg.updateVersion(newVersion);
console.log(pkg.version); // 1.1.0
`);

try {
  // Create a package
  const pkg = new Package('example-package', '1.0.0');

  // Bump version in multiple ways
  const bumpVersions = [];

  // Original
  bumpVersions.push(['Original', pkg.version]);

  // Bump to minor
  const newVersionMinor = bumpVersion(pkg.version, Version.Minor);
  pkg.updateVersion(newVersionMinor);
  bumpVersions.push(['After Minor Bump', pkg.version]);

  // Bump to patch
  const newVersionPatch = bumpVersion(pkg.version, Version.Patch);
  pkg.updateVersion(newVersionPatch);
  bumpVersions.push(['After Patch Bump', pkg.version]);

  // Create snapshot
  const snapshotVer = bumpSnapshotVersion(pkg.version, 'ci12345');
  pkg.updateVersion(snapshotVer);
  bumpVersions.push(['After Snapshot', pkg.version]);

  // Major bump
  const newVersionMajor = bumpVersion(pkg.version, Version.Major);
  pkg.updateVersion(newVersionMajor);
  bumpVersions.push(['After Major Bump', pkg.version]);

  // Display results
  const pkgTable = new Table({
    head: [
      chalk.white.bold('Stage'),
      chalk.white.bold('Version')
    ],
    colWidths: [20, 30]
  });

  bumpVersions.forEach(row => pkgTable.push(row));
  console.log(pkgTable.toString());

  success('Package version bumping demonstrated successfully');
} catch (err) {
  warning(`Error during package version bumping: ${err.message}`);
}

// Example 6: Real-World Scenario - Release Management
heading('6. Real-World Scenario: Release Management');
info('Let\'s simulate a release process for a product with multiple packages:');

code(`
// Create a release workflow function
function simulateReleaseProcess(packages, releaseType) {
  console.log(\`Starting \${releaseType} release process...\`);

  // First, simulate determining which type of release to perform
  packages.forEach(pkg => {
    console.log(\`Processing package: \${pkg.name} @ \${pkg.version}\`);

    // Bump the version according to the release type
    const newVersion = bumpVersion(pkg.version, releaseType);
    pkg.updateVersion(newVersion);

    console.log(\`Package \${pkg.name} version bumped to \${pkg.version}\`);

    // In a real system, you might:
    // 1. Update package.json
    // 2. Generate changelogs
    // 3. Commit changes
    // 4. Create git tags
    // 5. Publish to npm
  });

  console.log(\`\${releaseType} release completed!\`);
}

// Create packages for our release
const releasePackages = [
  new Package('@product/core', '1.0.0'),
  new Package('@product/ui', '0.5.0'),
  new Package('@product/api', '0.9.1')
];

// Simulate a minor release
simulateReleaseProcess(releasePackages, Version.Minor);
`);

try {
  // Create a release workflow function
  function simulateReleaseProcess(packages, releaseType, releaseTypeName) {
    console.log(chalk.magenta(`\nStarting ${releaseTypeName} release process...`));

    const results = [];

    // Process each package
    packages.forEach(pkg => {
      const oldVersion = pkg.version;

      // Bump the version according to the release type
      const newVersion = bumpVersion(pkg.version, releaseType);
      pkg.updateVersion(newVersion);

      results.push([pkg.name, oldVersion, pkg.version]);
    });

    console.log(chalk.magenta(`${releaseTypeName} release completed!\n`));
    return results;
  }

  // Create packages for our release
  const releasePackages = [
    new Package('@product/core', '1.0.0'),
    new Package('@product/ui', '0.5.0'),
    new Package('@product/api', '0.9.1')
  ];

  // Create backup for later
  const backupPackages = releasePackages.map(p => new Package(p.name, p.version));

  // Simulate a minor release
  const minorResults = simulateReleaseProcess(releasePackages, Version.Minor, 'Minor');

  // Display results
  const releaseTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Before'),
      chalk.white.bold('After Minor')
    ],
    colWidths: [20, 15, 15]
  });

  minorResults.forEach(row => releaseTable.push(row));
  console.log(releaseTable.toString());

  // Simulate a snapshot release for CI
  info('\nNow let\'s simulate a CI snapshot build:');

  // Use the backup packages
  const ciResults = backupPackages.map(pkg => {
    const oldVersion = pkg.version;
    const snapshotVer = bumpSnapshotVersion(pkg.version, 'ci9876');
    pkg.updateVersion(snapshotVer);
    return [pkg.name, oldVersion, pkg.version];
  });

  const ciTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Before'),
      chalk.white.bold('CI Snapshot')
    ],
    colWidths: [20, 15, 30]
  });

  ciResults.forEach(row => ciTable.push(row));
  console.log(ciTable.toString());

  success('\nRelease simulation completed successfully');
} catch (err) {
  warning(`Error during release simulation: ${err.message}`);
}

// Example 7: Handling Dependencies in Version Bumps
heading('7. Handling Dependencies in Version Bumps');
info('When bumping a package version, you may also need to update its usage as a dependency:');

// Create packages with dependencies
const libA = new Package('lib-a', '1.0.0');
const libB = new Package('lib-b', '1.0.0');
const libC = new Package('lib-c', '1.0.0');

// Add dependencies
libB.addDependency(new Dependency('lib-a', '^1.0.0'));
libC.addDependency(new Dependency('lib-a', '^1.0.0'));
libC.addDependency(new Dependency('lib-b', '^1.0.0'));

// Now, when we bump lib-a, we might need to update the dependency version in lib-b and lib-c
const newLibAVersion = bumpVersion(libA.version, Version.Major);
libA.updateVersion(newLibAVersion);

// Here's how we might update dependencies to reflect this major version change
const dependentPackages = [libB, libC];
dependentPackages.forEach(pkg => {
  const dependency = pkg.getDependency('lib-a');
  if (dependency) {
    // In a real system, you might have different strategies for different types of bumps
    dependency.updateVersion('^2.0.0');
  }
});

try {
  // Create packages with dependencies
  const libA = new Package('lib-a', '1.0.0');
  const libB = new Package('lib-b', '1.0.0');
  const libC = new Package('lib-c', '1.0.0');

  // Add dependencies
  libB.addDependency(new Dependency('lib-a', '^1.0.0'));
  libC.addDependency(new Dependency('lib-a', '^1.0.0'));
  libC.addDependency(new Dependency('lib-b', '^1.0.0'));

  // Display the initial state
  const initialState = [
    ['lib-a', libA.version, 'N/A'],
    ['lib-b', libB.version, libB.getDependency('lib-a')?.version || 'N/A'],
    ['lib-c (lib-a)', libC.version, libC.getDependency('lib-a')?.version || 'N/A'],
    ['lib-c (lib-b)', libC.version, libC.getDependency('lib-b')?.version || 'N/A'],
  ];

  const initialTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Version'),
      chalk.white.bold('Dependency Version')
    ],
    colWidths: [15, 15, 25]
  });

  initialState.forEach(row => initialTable.push(row));

  console.log(chalk.cyan('\nInitial state:'));
  console.log(initialTable.toString());

  // Now, bump lib-a to a new major version
  const newLibAVersion = bumpVersion(libA.version, Version.Major);
  libA.updateVersion(newLibAVersion);

  info(`\nBumped lib - a from 1.0.0 to ${newLibAVersion} `);

  // Update dependencies
  const dependentPackages = [libB, libC];
  dependentPackages.forEach(pkg => {
    const dependency = pkg.getDependency('lib-a');
    if (dependency) {
      const oldVersion = dependency.version;
      // In a real system, you might have different strategies for different types of bumps
      dependency.updateVersion('^2.0.0');
      info(`Updated ${pkg.name} 's dependency on lib-a from ${oldVersion} to ${dependency.version}`);
    }
  });

  // Display the final state
  const finalState = [
    ['lib-a', libA.version, 'N/A'],
    ['lib-b', libB.version, libB.getDependency('lib-a')?.version || 'N/A'],
    ['lib-c (lib-a)', libC.version, libC.getDependency('lib-a')?.version || 'N/A'],
    ['lib-c (lib-b)', libC.version, libC.getDependency('lib-b')?.version || 'N/A'],
  ];

  const finalTable = new Table({
    head: [
      chalk.white.bold('Package'),
      chalk.white.bold('Version'),
      chalk.white.bold('Dependency Version')
    ],
    colWidths: [15, 15, 25]
  });

  finalState.forEach(row => finalTable.push(row));

  console.log(chalk.cyan('\nFinal state after updates:'));
  console.log(finalTable.toString());

  success('\nDependency version updates completed successfully');
} catch (err) {
  warning(`Error during dependency handling: ${err.message}`);
}

// Example 8: Version Bumping in CI/CD Pipeline
heading('8. Version Bumping in CI/CD Pipelines');
info('In CI/CD workflows, version bumping is often automated based on commit history or CI metadata:');

code(`
// Simulate a CI/CD version bumping workflow

// In real CI/CD, we might read the commit history
// to determine what type of release to do
function determineReleaseType(commitMessages) {
  // This is a simplified example
  if (commitMessages.some(msg => msg.includes('BREAKING CHANGE'))) {
    return Version.Major;
  } else if (commitMessages.some(msg => msg.startsWith('feat'))) {
    return Version.Minor;
  } else {
    return Version.Patch;
  }
}

// Mock commit messages for demo
const commitMessages = [
  'fix: resolve issue with login button',
  'feat: add new dashboard component',
  'chore: update dependencies',
  'test: add tests for user profile',
];

// Determine release type from commit messages
const releaseType = determineReleaseType(commitMessages);

// Get CI metadata
const ciRunId = 'ci-123456';
const ciCommitSha = 'abc7890';

// Create a package to version
const pkg = new Package('my-app', '1.5.0');

// Based on workflow, either do a standard version bump or a snapshot
const isRelease = process.env.CI_IS_RELEASE === 'true';

if (isRelease) {
  // For release builds, use standard version bump
  const newVersion = bumpVersion(pkg.version, releaseType);
  pkg.updateVersion(newVersion);
  console.log(\`Released version \${pkg.version}\`);
} else {
  // For non-release builds, use snapshot version with CI metadata
  const snapshotVer = bumpSnapshotVersion(pkg.version, ciCommitSha);
  pkg.updateVersion(snapshotVer);
  console.log(\`Created CI build \${pkg.version}\`);
}
`);

try {
  // Simulate a CI/CD version bumping workflow

  // In real CI/CD, we might read the commit history
  // to determine what type of release to do
  function determineReleaseType(commitMessages) {
    // This is a simplified example
    if (commitMessages.some(msg => msg.includes('BREAKING CHANGE'))) {
      return { type: Version.Major, name: 'Major' };
    } else if (commitMessages.some(msg => msg.startsWith('feat'))) {
      return { type: Version.Minor, name: 'Minor' };
    } else {
      return { type: Version.Patch, name: 'Patch' };
    }
  }

  // Scenarios to demonstrate
  const scenarios = [
    {
      name: 'Feature Release',
      commits: [
        'fix: resolve issue with login button',
        'feat: add new dashboard component',
        'chore: update dependencies',
        'test: add tests for user profile',
      ],
      isRelease: true,
      ciCommitSha: 'abc7890'
    },
    {
      name: 'Bug Fix Release',
      commits: [
        'fix: resolve critical security issue',
        'docs: update README',
        'chore: cleanup old files',
      ],
      isRelease: true,
      ciCommitSha: 'def1234'
    },
    {
      name: 'Breaking Change Release',
      commits: [
        'feat: refactor API endpoints',
        'BREAKING CHANGE: remove support for legacy authentication',
        'docs: update API documentation',
      ],
      isRelease: true,
      ciCommitSha: 'ghi5678'
    },
    {
      name: 'CI Development Build',
      commits: [
        'feat: work in progress on new feature',
        'fix: address PR feedback',
      ],
      isRelease: false,
      ciCommitSha: 'jkl9012'
    }
  ];

  // Demonstrate each scenario
  const scenarioResults = [];

  scenarios.forEach(scenario => {
    // Create a package with the same starting version
    const pkg = new Package('my-app', '1.5.0');

    // Determine release type from commit messages
    const { type: releaseType, name: releaseTypeName } = determineReleaseType(scenario.commits);

    // Process version based on scenario
    let versioningMethod = '';

    if (scenario.isRelease) {
      // For release builds, use standard version bump
      const newVersion = bumpVersion(pkg.version, releaseType);
      pkg.updateVersion(newVersion);
      versioningMethod = `${releaseTypeName} bump`;
    } else {
      // For non-release builds, use snapshot version with CI metadata
      const snapshotVer = bumpSnapshotVersion(pkg.version, scenario.ciCommitSha);
      pkg.updateVersion(snapshotVer);
      versioningMethod = 'Snapshot';
    }

    // Save results
    scenarioResults.push([
      scenario.name,
      versioningMethod,
      pkg.version,
      scenario.isRelease ? 'Yes' : 'No'
    ]);
  });

  // Display results
  const scenarioTable = new Table({
    head: [
      chalk.white.bold('Scenario'),
      chalk.white.bold('Version Method'),
      chalk.white.bold('Result'),
      chalk.white.bold('Release Build')
    ],
    colWidths: [25, 15, 25, 15]
  });

  scenarioResults.forEach(row => scenarioTable.push(row));
  console.log(scenarioTable.toString());

  success('\nCI/CD version bumping simulation completed successfully');
} catch (err) {
  warning(`Error during CI/CD simulation: ${err.message}`);
}

// Final overview and conclusion
heading('Conclusion');
info('In this example, you\'ve learned how to:');
success('✓ Use different types of version bumps (major, minor, patch)');
success('✓ Create snapshot versions for CI/CD workflows');
success('✓ Use the VersionUtils class for version manipulation');
success('✓ Update package versions programmatically');
success('✓ Handle dependency version updates after bumping');
success('✓ Implement version bumping in release management and CI/CD pipelines');

console.log(createBox('Next Steps',
  chalk.cyan('1. Integrate version bumping into your release workflows\n') +
  chalk.cyan('2. Use snapshot versions for prerelease or CI builds\n') +
  chalk.cyan('3. Automate version bumping based on commit conventions')
));

console.log('\n');
