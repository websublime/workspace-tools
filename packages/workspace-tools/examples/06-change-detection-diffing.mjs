import chalk from 'chalk';
import boxen from 'boxen';
import Table from 'cli-table3';
import {
  Package,
  Dependency,
  PackageDiff,
  ChangeType,
  DependencyChange
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

console.log(createBox('Change Detection & Diffing Example',
  chalk.bold('This example demonstrates comparing package versions and detecting changes.')
));

// Example 1: Understanding the ChangeType Enum
heading('1. Understanding the ChangeType Enum');
info('The ChangeType enum represents different types of changes in dependencies:');

const changeTypesTable = new Table({
  head: [
    chalk.white.bold('ChangeType'),
    chalk.white.bold('Value'),
    chalk.white.bold('Description')
  ],
  colWidths: [20, 10, 50]
});

changeTypesTable.push([
  'ChangeType.Added',
  ChangeType.Added,
  'A new dependency was added to the package'
]);

changeTypesTable.push([
  'ChangeType.Removed',
  ChangeType.Removed,
  'A dependency was removed from the package'
]);

changeTypesTable.push([
  'ChangeType.Updated',
  ChangeType.Updated,
  'A dependency version was updated'
]);

changeTypesTable.push([
  'ChangeType.Unchanged',
  ChangeType.Unchanged,
  'A dependency remained the same'
]);

console.log(changeTypesTable.toString());

// Example 2: Creating Two Package Versions for Comparison
heading('2. Creating Two Package Versions for Comparison');
info('Creating previous and current versions of a package:');

code(`
// Create the previous version of the package
const previousPkg = new Package('my-app', '1.0.0');

// Add dependencies to the previous version
previousPkg.addDependency(new Dependency('express', '^4.17.1'));
previousPkg.addDependency(new Dependency('react', '^17.0.2'));
previousPkg.addDependency(new Dependency('lodash', '^4.17.20'));

// Create the current version of the package
const currentPkg = new Package('my-app', '2.0.0');

// Add dependencies to the current version with some changes
currentPkg.addDependency(new Dependency('express', '^4.18.2'));  // Updated
currentPkg.addDependency(new Dependency('react', '^18.0.0'));    // Updated with breaking change
currentPkg.addDependency(new Dependency('typescript', '^4.8.4')); // Added
// Note: lodash is removed in this version
`);

// Create previous package version
const previousPkg = new Package('my-app', '1.0.0');
previousPkg.addDependency(new Dependency('express', '^4.17.1'));
previousPkg.addDependency(new Dependency('react', '^17.0.2'));
previousPkg.addDependency(new Dependency('lodash', '^4.17.20'));

// Create current package version
const currentPkg = new Package('my-app', '2.0.0');
currentPkg.addDependency(new Dependency('express', '^4.18.2')); // Updated
currentPkg.addDependency(new Dependency('react', '^18.0.0'));   // Major update
currentPkg.addDependency(new Dependency('typescript', '^4.8.4')); // Added

success('Created two package versions for comparison');

// Example 3: Creating a Package Diff
heading('3. Creating and Examining a Package Diff');
info('Creating a diff between the two package versions:');

code(`
// Create a diff between previous and current package versions
const diff = PackageDiff.between(previousPkg, currentPkg);
`);

// Create the diff
const diff = PackageDiff.between(previousPkg, currentPkg);

// Display the diff overview
const diffOverviewTable = new Table({
  chars: {
    'top': '═', 'top-mid': '╤', 'top-left': '╔', 'top-right': '╗',
    'bottom': '═', 'bottom-mid': '╧', 'bottom-left': '╚', 'bottom-right': '╝',
    'left': '║', 'left-mid': '╟', 'right': '║', 'right-mid': '╢',
    'mid': '─', 'mid-mid': '┼', 'middle': '│'
  },
  style: { head: ['cyan'] }
});

diffOverviewTable.push(
  [chalk.bold('Package Name'), diff.packageName],
  [chalk.bold('Previous Version'), diff.previousVersion],
  [chalk.bold('Current Version'), diff.currentVersion],
  [chalk.bold('Breaking Change?'), diff.breakingChange ? chalk.red('Yes') : chalk.green('No')]
);

console.log(diffOverviewTable.toString());

// Example 4: Examining Dependency Changes
heading('4. Examining Dependency Changes');
info('Looking at individual dependency changes:');

code(`
// Get all dependency changes
const dependencyChanges = diff.dependencyChanges;

// Examine each change
dependencyChanges.forEach(change => {
  console.log(\`Dependency: \${change.name}\`);
  console.log(\`  Change type: \${getChangeTypeName(change.changeType)}\`);
  console.log(\`  Previous version: \${change.previousVersion || 'N/A'}\`);
  console.log(\`  Current version: \${change.currentVersion || 'N/A'}\`);
  console.log(\`  Breaking change: \${change.breaking ? 'Yes' : 'No'}\`);
});
`);

// Helper function to get change type name
function getChangeTypeName(type) {
  switch (type) {
    case ChangeType.Added: return 'Added';
    case ChangeType.Removed: return 'Removed';
    case ChangeType.Updated: return 'Updated';
    case ChangeType.Unchanged: return 'Unchanged';
    default: return 'Unknown';
  }
}

// Function to get appropriate color for change type
function getChangeTypeColor(type, text) {
  switch (type) {
    case ChangeType.Added: return chalk.green(text);
    case ChangeType.Removed: return chalk.red(text);
    case ChangeType.Updated: return chalk.yellow(text);
    case ChangeType.Unchanged: return chalk.blue(text);
    default: return text;
  }
}

// Display dependency changes in a table
const dependencyChangeTable = new Table({
  head: [
    chalk.white.bold('Dependency'),
    chalk.white.bold('Change Type'),
    chalk.white.bold('Previous'),
    chalk.white.bold('Current'),
    chalk.white.bold('Breaking?')
  ],
  colWidths: [15, 15, 15, 15, 10]
});

diff.dependencyChanges.forEach(change => {
  dependencyChangeTable.push([
    change.name,
    getChangeTypeColor(change.changeType, getChangeTypeName(change.changeType)),
    change.previousVersion || 'N/A',
    change.currentVersion || 'N/A',
    change.breaking ? chalk.red('Yes') : chalk.green('No')
  ]);
});

console.log(dependencyChangeTable.toString());

// Example 5: Counting Breaking Changes and Change Types
heading('5. Analyzing Diff Statistics');

info('Counting breaking changes:');
code(`const breakingChanges = diff.countBreakingChanges();`);
const breakingChanges = diff.countBreakingChanges();
success(`Total breaking changes: ${breakingChanges}`);

info('\nCounting changes by type:');
code(`const changesByType = diff.countChangesByType();`);
const changesByType = diff.countChangesByType();

const changeCountTable = new Table({
  head: [
    chalk.white.bold('Change Type'),
    chalk.white.bold('Count')
  ],
  colWidths: [20, 10]
});

if (changesByType.added) {
  changeCountTable.push(['Added', chalk.green(changesByType.added)]);
}
if (changesByType.removed) {
  changeCountTable.push(['Removed', chalk.red(changesByType.removed)]);
}
if (changesByType.updated) {
  changeCountTable.push(['Updated', chalk.yellow(changesByType.updated)]);
}
if (changesByType.unchanged) {
  changeCountTable.push(['Unchanged', chalk.blue(changesByType.unchanged)]);
}

console.log(changeCountTable.toString());

// Example 6: String Representation of the Diff
heading('6. String Representation of the Diff');
info('Getting a string representation of the package diff:');
code(`const diffString = diff.toString();`);

const diffString = diff.toString();
console.log(boxen(diffString, {
  padding: 1,
  borderColor: 'blue',
  title: 'Package Diff Summary',
  titleAlignment: 'center'
}));

// Example 7: Working with DependencyChange Objects
heading('7. Working with DependencyChange Objects');
heading('7. Working with DependencyChange Objects');
info('Creating a DependencyChange object to represent a specific change:');

code(`
// Create a DependencyChange object directly
const manualChange = new DependencyChange(
  'vue',          // name
  '2.6.14',       // previousVersion
  '3.2.37',       // currentVersion
  ChangeType.Updated,  // changeType
  true            // breaking
);

// Accessing properties of the DependencyChange object
console.log(\`Dependency: \${manualChange.name}\`);
console.log(\`Previous version: \${manualChange.previousVersion}\`);
console.log(\`Current version: \${manualChange.currentVersion}\`);
console.log(\`Change type: \${getChangeTypeName(manualChange.changeType)}\`);
console.log(\`Breaking change: \${manualChange.breaking ? 'Yes' : 'No'}\`);
`);

// Create a DependencyChange object
const manualChange = new DependencyChange(
  'vue',          // name
  '2.6.14',       // previousVersion
  '3.2.37',       // currentVersion
  ChangeType.Updated,  // changeType
  true            // breaking
);

// Display the manual change
const manualChangeTable = new Table({
  head: [
    chalk.white.bold('Property'),
    chalk.white.bold('Value')
  ],
  colWidths: [20, 30]
});

manualChangeTable.push(
  ['name', manualChange.name],
  ['previousVersion', manualChange.previousVersion],
  ['currentVersion', manualChange.currentVersion],
  ['changeType', getChangeTypeName(manualChange.changeType)],
  ['breaking', manualChange.breaking ? chalk.red('true') : chalk.green('false')]
);

console.log(manualChangeTable.toString());

// Demonstrate using the DependencyChange in a real scenario
subHeading('Using DependencyChange for Custom Analysis:');

code(`
// Create an array of DependencyChange objects for custom analysis
const customChanges = [
  new DependencyChange(
    'react',
    '16.14.0',
    '17.0.2',
    ChangeType.Updated,
    true
  ),
  new DependencyChange(
    'lodash',
    '4.17.20',
    '4.17.21',
    ChangeType.Updated,
    false
  ),
  new DependencyChange(
    'webpack',
    '4.46.0',
    '5.70.0',
    ChangeType.Updated,
    true
  ),
  new DependencyChange(
    'jest',
    null,
    '27.5.1',
    ChangeType.Added,
    false
  )
];

// Analyze these changes for high-risk dependencies
const highRiskChanges = customChanges.filter(change =>
  change.breaking && change.changeType === ChangeType.Updated
);

console.log(\`High-risk dependencies requiring careful testing: \${
  highRiskChanges.map(change => change.name).join(', ')
}\`);
`);

// Create an array of DependencyChange objects for custom analysis
const customChanges = [
  new DependencyChange(
    'react',
    '16.14.0',
    '17.0.2',
    ChangeType.Updated,
    true
  ),
  new DependencyChange(
    'lodash',
    '4.17.20',
    '4.17.21',
    ChangeType.Updated,
    false
  ),
  new DependencyChange(
    'webpack',
    '4.46.0',
    '5.70.0',
    ChangeType.Updated,
    true
  ),
  new DependencyChange(
    'jest',
    null,
    '27.5.1',
    ChangeType.Added,
    false
  )
];

// Analyze these changes for high-risk dependencies
const highRiskChanges = customChanges.filter(change =>
  change.breaking && change.changeType === ChangeType.Updated
);

console.log(`High-risk dependencies requiring careful testing: ${highRiskChanges.map(change => change.name).join(', ')
  }`);

// Group changes by type for reporting
const changesByCategory = customChanges.reduce((acc, change) => {
  const category = getChangeTypeName(change.changeType);
  if (!acc[category]) acc[category] = [];
  acc[category].push(change);
  return acc;
}, {});

const categorizedTable = new Table({
  head: [
    chalk.white.bold('Category'),
    chalk.white.bold('Dependencies'),
    chalk.white.bold('Breaking Count')
  ],
  colWidths: [15, 40, 15]
});

for (const [category, changes] of Object.entries(changesByCategory)) {
  const breakingCount = changes.filter(c => c.breaking).length;
  categorizedTable.push([
    getChangeTypeColor(changes[0].changeType, category),
    changes.map(c => c.name).join(', '),
    breakingCount > 0 ? chalk.red(breakingCount) : chalk.green('0')
  ]);
}

console.log(categorizedTable.toString());

// Example 8: Real-World Scenario - Package Version Migration Analysis
heading('8. Real-World Scenario: Package Version Migration Analysis');
info('Analyzing a major version upgrade to determine migration effort:');

// Create initial and target versions of a complex package
code(`
// Create the initial version of a complex package
const initialVersion = new Package('complex-app', '3.5.0');

// Add many dependencies with varying versions
initialVersion.addDependency(new Dependency('react', '^16.8.0'));
initialVersion.addDependency(new Dependency('react-dom', '^16.8.0'));
initialVersion.addDependency(new Dependency('redux', '^4.0.5'));
initialVersion.addDependency(new Dependency('react-redux', '^7.1.3'));
initialVersion.addDependency(new Dependency('express', '^4.17.1'));
initialVersion.addDependency(new Dependency('mongoose', '^5.9.7'));
initialVersion.addDependency(new Dependency('webpack', '^4.42.1'));
initialVersion.addDependency(new Dependency('babel-core', '^6.26.3'));
initialVersion.addDependency(new Dependency('typescript', '^3.8.3'));

// Create the target version with updated dependencies
const targetVersion = new Package('complex-app', '4.0.0');

// Add updated dependencies, some with breaking changes
targetVersion.addDependency(new Dependency('react', '^17.0.2'));
targetVersion.addDependency(new Dependency('react-dom', '^17.0.2'));
targetVersion.addDependency(new Dependency('redux', '^4.1.2'));
targetVersion.addDependency(new Dependency('react-redux', '^7.2.6'));
targetVersion.addDependency(new Dependency('express', '^4.17.3'));
targetVersion.addDependency(new Dependency('mongoose', '^6.2.4'));  // Major version change
targetVersion.addDependency(new Dependency('webpack', '^5.70.0'));  // Major version change
targetVersion.addDependency(new Dependency('@babel/core', '^7.17.5')); // Package name changed
targetVersion.addDependency(new Dependency('typescript', '^4.6.2')); // Major version change
targetVersion.addDependency(new Dependency('jest', '^27.5.1')); // Added
`);

// Create initial version of complex package
const initialVersion = new Package('complex-app', '3.5.0');

// Add many dependencies with varying versions
initialVersion.addDependency(new Dependency('react', '^16.8.0'));
initialVersion.addDependency(new Dependency('react-dom', '^16.8.0'));
initialVersion.addDependency(new Dependency('redux', '^4.0.5'));
initialVersion.addDependency(new Dependency('react-redux', '^7.1.3'));
initialVersion.addDependency(new Dependency('express', '^4.17.1'));
initialVersion.addDependency(new Dependency('mongoose', '^5.9.7'));
initialVersion.addDependency(new Dependency('webpack', '^4.42.1'));
initialVersion.addDependency(new Dependency('babel-core', '^6.26.3'));
initialVersion.addDependency(new Dependency('typescript', '^3.8.3'));

// Create target version with updated dependencies
const targetVersion = new Package('complex-app', '4.0.0');

// Add updated dependencies, some with breaking changes
targetVersion.addDependency(new Dependency('react', '^17.0.2'));
targetVersion.addDependency(new Dependency('react-dom', '^17.0.2'));
targetVersion.addDependency(new Dependency('redux', '^4.1.2'));
targetVersion.addDependency(new Dependency('react-redux', '^7.2.6'));
targetVersion.addDependency(new Dependency('express', '^4.17.3'));
targetVersion.addDependency(new Dependency('mongoose', '^6.2.4'));  // Major version change
targetVersion.addDependency(new Dependency('webpack', '^5.70.0'));  // Major version change
targetVersion.addDependency(new Dependency('@babel/core', '^7.17.5')); // Package name changed
targetVersion.addDependency(new Dependency('typescript', '^4.6.2')); // Major version change
targetVersion.addDependency(new Dependency('jest', '^27.5.1')); // Added

// Create the migration diff
subHeading('Migration Analysis:');
code(`const migrationDiff = PackageDiff.between(initialVersion, targetVersion);`);

const migrationDiff = PackageDiff.between(initialVersion, targetVersion);

// Display migration summary
info('Migration Summary:');
const migrationSummary = new Table({
  head: [
    chalk.white.bold('Metric'),
    chalk.white.bold('Value'),
    chalk.white.bold('Notes')
  ],
  colWidths: [20, 10, 50]
});

const changesByTypeComplex = migrationDiff.countChangesByType();
const breakingChangesComplex = migrationDiff.countBreakingChanges();

migrationSummary.push([
  'Version Bump',
  chalk.bold(migrationDiff.breakingChange ? chalk.red('Major') : chalk.yellow('Minor')),
  `From ${migrationDiff.previousVersion} to ${migrationDiff.currentVersion}`
]);

migrationSummary.push([
  'Total Changes',
  chalk.bold(migrationDiff.dependencyChanges.length),
  'Number of dependencies affected'
]);

migrationSummary.push([
  'Breaking Changes',
  chalk.bold(breakingChangesComplex > 0 ? chalk.red(breakingChangesComplex) : '0'),
  'Dependencies with breaking changes that need careful migration'
]);

migrationSummary.push([
  'Added',
  chalk.bold(chalk.green(changesByTypeComplex.added || 0)),
  'New dependencies added'
]);

migrationSummary.push([
  'Removed',
  chalk.bold(chalk.red(changesByTypeComplex.removed || 0)),
  'Dependencies that were removed'
]);

migrationSummary.push([
  'Updated',
  chalk.bold(chalk.yellow(changesByTypeComplex.updated || 0)),
  'Dependencies that were updated'
]);

console.log(migrationSummary.toString());

// Display detailed migration analysis
subHeading('Detailed Change Analysis:');
const migrationDetailsTable = new Table({
  head: [
    chalk.white.bold('Dependency'),
    chalk.white.bold('Change'),
    chalk.white.bold('From'),
    chalk.white.bold('To'),
    chalk.white.bold('Breaking?'),
    chalk.white.bold('Migration Effort')
  ],
  colWidths: [15, 15, 15, 15, 10, 15]
});

// Assign migration effort based on change type and breaking status
function getMigrationEffort(change) {
  if (change.changeType === ChangeType.Added) return chalk.green('Low');
  if (change.changeType === ChangeType.Removed) return chalk.red('High');
  if (change.changeType === ChangeType.Updated && change.breaking) return chalk.red('High');
  if (change.changeType === ChangeType.Updated && !change.breaking) return chalk.yellow('Medium');
  return chalk.blue('None');
}

// Sort changes by effort (breaking changes first)
const sortedChanges = [...migrationDiff.dependencyChanges].sort((a, b) => {
  // First sort by change type (Removed -> Updated -> Added)
  if (a.changeType !== b.changeType) {
    return a.changeType - b.changeType;
  }
  // Then by breaking status
  return b.breaking - a.breaking;
});

// Fill the table with sorted changes
sortedChanges.forEach(change => {
  migrationDetailsTable.push([
    change.name,
    getChangeTypeColor(change.changeType, getChangeTypeName(change.changeType)),
    change.previousVersion || 'N/A',
    change.currentVersion || 'N/A',
    change.breaking ? chalk.red('Yes') : chalk.green('No'),
    getMigrationEffort(change)
  ]);
});

console.log(migrationDetailsTable.toString());

// Calculate migration effort score
const effortScore = sortedChanges.reduce((score, change) => {
  if (change.changeType === ChangeType.Added) return score + 1;
  if (change.changeType === ChangeType.Removed) return score + 5;
  if (change.changeType === ChangeType.Updated && change.breaking) return score + 5;
  if (change.changeType === ChangeType.Updated && !change.breaking) return score + 2;
  return score;
}, 0);

// Determine overall migration difficulty
let difficultyLevel = 'Easy';
let difficultyColor = chalk.green;

if (effortScore > 30) {
  difficultyLevel = 'Very Hard';
  difficultyColor = chalk.red.bold;
} else if (effortScore > 20) {
  difficultyLevel = 'Hard';
  difficultyColor = chalk.red;
} else if (effortScore > 10) {
  difficultyLevel = 'Moderate';
  difficultyColor = chalk.yellow;
}

// Display overall assessment
subHeading('Migration Difficulty Assessment:');
info(`Based on the changes analyzed, this migration is rated as: ${difficultyColor(difficultyLevel)}`);
info(`Migration effort score: ${effortScore} points`);

// Migration recommendations
subHeading('Migration Recommendations:');

// Get breaking changes for special advice
const breakingDeps = sortedChanges.filter(change => change.breaking);
const removedDeps = sortedChanges.filter(change => change.changeType === ChangeType.Removed);
const nameMigrations = []; // For renamed packages like babel-core -> @babel/core

// Check for potential package renames (removed + added with similar names)
const removedNames = removedDeps.map(d => d.name);
const addedDeps = sortedChanges.filter(change => change.changeType === ChangeType.Added);

// Simple heuristic for detecting renames: check if a removed pkg has a similar name to an added one
removedDeps.forEach(removed => {
  // Find potential matches (e.g. babel-core -> @babel/core)
  const baseName = removed.name.replace(/[@\-\/]/g, '');
  const potentialMatches = addedDeps.filter(added =>
    added.name.replace(/[@\-\/]/g, '').includes(baseName) ||
    baseName.includes(added.name.replace(/[@\-\/]/g, ''))
  );

  if (potentialMatches.length > 0) {
    nameMigrations.push({
      from: removed.name,
      to: potentialMatches[0].name
    });
  }
});

// Create recommendations
const recommendations = [];

if (breakingDeps.length > 0) {
  recommendations.push(`Research the breaking changes in: ${breakingDeps.map(d => d.name).join(', ')}`);
}

if (nameMigrations.length > 0) {
  nameMigrations.forEach(migration => {
    recommendations.push(`Package rename detected: ${migration.from} → ${migration.to}`);
  });
}

if (effortScore > 15) {
  recommendations.push('Consider a phased migration approach instead of migrating all at once');
}

if (sortedChanges.some(c => c.name === 'webpack' && c.breaking)) {
  recommendations.push('Update webpack configuration files to match the new webpack version');
}

if (sortedChanges.some(c => c.name === 'typescript' && c.breaking)) {
  recommendations.push('Run TypeScript compiler in --noImplicitAny mode to identify typing issues');
}

// Display recommendations
const recommendationsTable = new Table({
  head: [chalk.white.bold('#'), chalk.white.bold('Recommendation')],
  colWidths: [5, 85]
});

recommendations.forEach((rec, i) => {
  recommendationsTable.push([i + 1, rec]);
});

console.log(recommendationsTable.toString());

// Example 9: Creating and Comparing Multiple Package Versions
heading('9. Creating and Comparing Multiple Package Versions');
info('Tracking changes across multiple versions of a package:');

code(`
// Create multiple versions of a package to track its evolution
const versions = [
  { version: '1.0.0', dependencies: [
    { name: 'express', version: '^4.16.0' },
    { name: 'lodash', version: '^4.17.10' }
  ]},
  { version: '1.1.0', dependencies: [
    { name: 'express', version: '^4.16.4' },
    { name: 'lodash', version: '^4.17.11' },
    { name: 'moment', version: '^2.22.2' }
  ]},
  { version: '2.0.0', dependencies: [
    { name: 'express', version: '^4.17.1' },
    { name: 'lodash', version: '^4.17.15' },
    { name: 'moment', version: '^2.24.0' },
    { name: 'axios', version: '^0.19.0' }
  ]},
  { version: '3.0.0', dependencies: [
    { name: 'express', version: '^4.17.3' },
    { name: 'lodash', version: '^4.17.21' },
    { name: 'moment', version: '^2.29.1' },
    { name: 'axios', version: '^0.26.1' },
    { name: 'react', version: '^17.0.2' }
  ]}
];

// Create package objects for each version
const packageVersions = versions.map(v => {
  const pkg = new Package('evolving-app', v.version);
  v.dependencies.forEach(d => {
    pkg.addDependency(new Dependency(d.name, d.version));
  });
  return pkg;
});

// Compare each version with the next one
for (let i = 0; i < packageVersions.length - 1; i++) {
  const versionDiff = PackageDiff.between(packageVersions[i], packageVersions[i+1]);
  console.log(\`Changes from \${versionDiff.previousVersion} to \${versionDiff.currentVersion}:\`);
  console.log(\`  - Breaking change: \${versionDiff.breakingChange ? 'Yes' : 'No'}\`);
  console.log(\`  - Changes: \${versionDiff.dependencyChanges.length}\`);
}
`);

// Create multiple versions of a package to track its evolution
const versions = [
  {
    version: '1.0.0', dependencies: [
      { name: 'express', version: '^4.16.0' },
      { name: 'lodash', version: '^4.17.10' }
    ]
  },
  {
    version: '1.1.0', dependencies: [
      { name: 'express', version: '^4.16.4' },
      { name: 'lodash', version: '^4.17.11' },
      { name: 'moment', version: '^2.22.2' }
    ]
  },
  {
    version: '2.0.0', dependencies: [
      { name: 'express', version: '^4.17.1' },
      { name: 'lodash', version: '^4.17.15' },
      { name: 'moment', version: '^2.24.0' },
      { name: 'axios', version: '^0.19.0' }
    ]
  },
  {
    version: '3.0.0', dependencies: [
      { name: 'express', version: '^4.17.3' },
      { name: 'lodash', version: '^4.17.21' },
      { name: 'moment', version: '^2.29.1' },
      { name: 'axios', version: '^0.26.1' },
      { name: 'react', version: '^17.0.2' }
    ]
  }
];

// Create package objects for each version
const packageVersions = versions.map(v => {
  const pkg = new Package('evolving-app', v.version);
  v.dependencies.forEach(d => {
    pkg.addDependency(new Dependency(d.name, d.version));
  });
  return pkg;
});

// Compare each version with the next one
const evolutionTable = new Table({
  head: [
    chalk.white.bold('From'),
    chalk.white.bold('To'),
    chalk.white.bold('Breaking'),
    chalk.white.bold('Added'),
    chalk.white.bold('Removed'),
    chalk.white.bold('Updated'),
    chalk.white.bold('Total Changes')
  ],
  colWidths: [10, 10, 10, 10, 10, 10, 15]
});

for (let i = 0; i < packageVersions.length - 1; i++) {
  const versionDiff = PackageDiff.between(packageVersions[i], packageVersions[i + 1]);
  const changesByType = versionDiff.countChangesByType();

  evolutionTable.push([
    versionDiff.previousVersion,
    versionDiff.currentVersion,
    versionDiff.breakingChange ? chalk.red('Yes') : chalk.green('No'),
    changesByType.added || 0,
    changesByType.removed || 0,
    changesByType.updated || 0,
    versionDiff.dependencyChanges.length
  ]);
}

console.log(evolutionTable.toString());

// Summary
console.log(createBox('Summary',
  chalk.bold('Key Concepts Demonstrated:') + '\n\n' +
  '✅ Using the ChangeType enum to classify dependency changes\n' +
  '✅ Creating and comparing package versions\n' +
  '✅ Examining detailed dependency changes with PackageDiff\n' +
  '✅ Counting breaking changes and analyzing by change type\n' +
  '✅ Getting string representations of diffs\n' +
  '✅ Working with DependencyChange objects\n' +
  '✅ Analyzing migration difficulty for package upgrades\n' +
  '✅ Tracking changes across multiple package versions'
));
