import chalk from 'chalk';
import boxen from 'boxen';
import Table from 'cli-table3';
import {
  Package,
  Dependency,
  DependencyGraph,
  DependencyFilter,
  ValidationReport,
  ValidationIssueType,
  buildDependencyGraphFromPackages
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

console.log(createBox('Dependency Graph Analysis Example',
  chalk.bold('This example demonstrates building and analyzing dependency graphs.')
));

// Example 1: Understanding the DependencyFilter Enum
heading('1. Understanding the DependencyFilter Enum');
info('The DependencyFilter enum controls which types of dependencies to include in a graph:');

const filterTable = new Table({
  head: [
    chalk.white.bold('Filter'),
    chalk.white.bold('Value'),
    chalk.white.bold('Description')
  ],
  colWidths: [25, 10, 50]
});

filterTable.push([
  'DependencyFilter.ProductionOnly',
  DependencyFilter.ProductionOnly,
  'Only include regular production dependencies'
]);

filterTable.push([
  'DependencyFilter.WithDevelopment',
  DependencyFilter.WithDevelopment,
  'Include both production and development dependencies'
]);

filterTable.push([
  'DependencyFilter.AllDependencies',
  DependencyFilter.AllDependencies,
  'Include production, development, and optional dependencies'
]);

console.log(filterTable.toString());

// Example 2: Creating a Simple Dependency Graph
heading('2. Creating a Simple Dependency Graph');
info('Building a dependency graph for a small set of packages:');

code(`
// Create packages
const packageA = new Package('package-a', '1.0.0');
const packageB = new Package('package-b', '1.0.0');
const packageC = new Package('package-c', '1.0.0');

// Create dependencies
packageA.addDependency(new Dependency('package-b', '^1.0.0'));
packageB.addDependency(new Dependency('package-c', '^1.0.0'));

// Build a dependency graph
const simpleGraph = buildDependencyGraphFromPackages([packageA, packageB, packageC]);
`);

// Create packages
const packageA = new Package('package-a', '1.0.0');
const packageB = new Package('package-b', '1.0.0');
const packageC = new Package('package-c', '1.0.0');

// Create dependencies
packageA.addDependency(new Dependency('package-b', '^1.0.0'));
packageB.addDependency(new Dependency('package-c', '^1.0.0'));

// Build dependency graph
const simpleGraph = buildDependencyGraphFromPackages([packageA, packageB, packageC]);
success('Dependency graph created successfully');

// Example 3: Internal Resolvability
heading('3. Checking Internal Resolvability');
info('Checking if all dependencies in the graph can be resolved internally:');

code(`
const isResolvable = simpleGraph.isInternallyResolvable();
console.log(\`Can all dependencies be resolved within the workspace? \${isResolvable ? 'Yes' : 'No'}\`);
`);

const isResolvable = simpleGraph.isInternallyResolvable();
success(`Can all dependencies be resolved within the workspace? ${isResolvable ? 'Yes' : 'No'}`);

info('Finding any missing dependencies:');
code(`
const missingDeps = simpleGraph.findMissingDependencies();
console.log(\`Missing dependencies: \${missingDeps.length > 0 ? missingDeps.join(', ') : 'None'}\`);
`);

const missingDeps = simpleGraph.findMissingDependencies();
if (missingDeps.length > 0) {
  warning(`Missing dependencies: ${missingDeps.join(', ')}`);
} else {
  success('No missing dependencies');
}

// Example 4: Finding Version Conflicts
heading('4. Finding Version Conflicts');
info('Creating a graph with version conflicts:');

code(`
// Create packages with conflicting dependency versions
const app = new Package('my-app', '1.0.0');
const libA = new Package('lib-a', '1.0.0');
const libB = new Package('lib-b', '1.0.0');
const shared = new Package('shared-lib', '1.0.0');

// Add dependencies with different version requirements
app.addDependency(new Dependency('lib-a', '^1.0.0'));
app.addDependency(new Dependency('lib-b', '^1.0.0'));

libA.addDependency(new Dependency('shared-lib', '^1.0.0'));
libB.addDependency(new Dependency('shared-lib', '^2.0.0')); // Conflict!

// Build the graph
const conflictGraph = buildDependencyGraphFromPackages([app, libA, libB, shared]);

// Find version conflicts
const conflicts = conflictGraph.findVersionConflicts();
`);

// Create packages with conflicting dependency versions
const app = new Package('my-app', '1.0.0');
const libA = new Package('lib-a', '1.0.0');
const libB = new Package('lib-b', '1.0.0');
const shared = new Package('shared-lib', '1.0.0');

// Add dependencies with different version requirements
app.addDependency(new Dependency('lib-a', '^1.0.0'));
app.addDependency(new Dependency('lib-b', '^1.0.0'));

libA.addDependency(new Dependency('shared-lib', '^1.0.0'));
libB.addDependency(new Dependency('shared-lib', '^2.0.0')); // Conflict!

// Build the graph with conflicts
const conflictGraph = buildDependencyGraphFromPackages([app, libA, libB, shared]);

// Find version conflicts
const conflicts = conflictGraph.findVersionConflicts();

if (conflicts) {
  warning('Version conflicts detected:');

  const conflictsTable = new Table({
    head: [
      chalk.white.bold('Dependency'),
      chalk.white.bold('Conflicting Versions')
    ],
    colWidths: [20, 60]
  });

  for (const [depName, versions] of Object.entries(conflicts)) {
    conflictsTable.push([
      depName,
      versions.join(', ')
    ]);
  }

  console.log(conflictsTable.toString());
} else {
  success('No version conflicts found');
}

// Example 4b: Creating a DependencyGraph Directly
heading('4b. Creating a DependencyGraph Directly');
info('Creating a DependencyGraph instance directly:');

code(`
// Import the DependencyGraph class
import { DependencyGraph } from '../src/index.js';

// Create a new DependencyGraph instance
const directGraph = new DependencyGraph();

// Add nodes and connections to the graph
// Note: The actual API might differ - this is conceptual
directGraph.addNode(packageA);
directGraph.addNode(packageB);
directGraph.addConnection(packageA, packageB);
`);

info('Note: The DependencyGraph class in this library is not designed to be instantiated directly.');
info('Instead, use the buildDependencyGraphFromPackages or buildDependencyGraphFromPackageInfos functions.');
info('This is a design decision to ensure graphs are built consistently.');

// Example 5: Detecting Circular Dependencies
heading('5. Detecting Circular Dependencies');
info('Creating a graph with circular dependencies:');

code(`
// Create packages with circular dependencies
const moduleA = new Package('module-a', '1.0.0');
const moduleB = new Package('module-b', '1.0.0');
const moduleC = new Package('module-c', '1.0.0');

// Create circular dependency: A -> B -> C -> A
moduleA.addDependency(new Dependency('module-b', '^1.0.0'));
moduleB.addDependency(new Dependency('module-c', '^1.0.0'));
moduleC.addDependency(new Dependency('module-a', '^1.0.0')); // Creates a cycle

// Build the graph
const circularGraph = buildDependencyGraphFromPackages([moduleA, moduleB, moduleC]);

// Detect circular dependencies
const cycle = circularGraph.detectCircularDependencies();
`);

// Create packages with circular dependencies
const moduleA = new Package('module-a', '1.0.0');
const moduleB = new Package('module-b', '1.0.0');
const moduleC = new Package('module-c', '1.0.0');

// Create circular dependency: A -> B -> C -> A
moduleA.addDependency(new Dependency('module-b', '^1.0.0'));
moduleB.addDependency(new Dependency('module-c', '^1.0.0'));
moduleC.addDependency(new Dependency('module-a', '^1.0.0')); // Creates a cycle

// Build graph with cycle
const circularGraph = buildDependencyGraphFromPackages([moduleA, moduleB, moduleC]);

// Detect circular dependencies
const cycle = circularGraph.detectCircularDependencies();
if (cycle) {
  warning('Circular dependency detected:');
  warning(`  ${cycle.join(' → ')} → ${cycle[0]}`);

  // Visualize the cycle
  const cycleTable = new Table({
    head: [chalk.white.bold('Node'), chalk.white.bold('Depends On')],
    colWidths: [20, 20]
  });

  for (let i = 0; i < cycle.length; i++) {
    const nextIdx = (i + 1) % cycle.length;
    cycleTable.push([
      cycle[i],
      cycle[nextIdx]
    ]);
  }

  console.log(cycleTable.toString());
} else {
  success('No circular dependencies found');
}

// Example 6: Finding Dependents
heading('6. Finding Dependents');
info('For each package, finding what other packages depend on it:');

code(`
// Get dependents for each node in the graph
const packages = [moduleA, moduleB, moduleC];
for (const pkg of packages) {
  const dependents = circularGraph.getDependents(pkg.name);
  console.log(\`\${pkg.name} is depended on by: \${dependents.join(', ')}\`);
}
`);

// Display dependents for each package
const dependentsTable = new Table({
  head: [chalk.white.bold('Package'), chalk.white.bold('Depended On By')],
  colWidths: [20, 50]
});

const packages = [moduleA, moduleB, moduleC];
for (const pkg of packages) {
  const dependents = circularGraph.getDependents(pkg.name);
  dependentsTable.push([
    pkg.name,
    dependents.length > 0 ? dependents.join(', ') : 'None'
  ]);
}

console.log(dependentsTable.toString());

// Example 7: Getting Nodes from the Graph
heading('7. Getting Nodes from the Graph');
info('Accessing nodes in the graph by their identifier:');

code(`
// Get a node by ID
const nodeA = circularGraph.getNode('module-a');
console.log(\`Node found: \${nodeA ? nodeA.name : 'Not found'}\`);
`);

const nodeA = circularGraph.getNode('module-a');
if (nodeA) {
  success(`Node found: ${nodeA.name} (version ${nodeA.version})`);

  // Show dependencies
  const nodeDeps = nodeA.dependencies();

  if (nodeDeps.length > 0) {
    const nodeTable = new Table({
      head: [chalk.white.bold('Dependency'), chalk.white.bold('Version')],
      colWidths: [20, 20]
    });

    nodeDeps.forEach(dep => {
      nodeTable.push([dep.name, dep.version]);
    });

    console.log('Dependencies:');
    console.log(nodeTable.toString());
  } else {
    info('No dependencies');
  }
} else {
  warning('Node not found');
}

// Example 8: Understanding ValidationIssueType
heading('8. Understanding ValidationIssueType');
info('The ValidationIssueType enum represents different types of validation issues:');

const validationTypeTable = new Table({
  head: [
    chalk.white.bold('Issue Type'),
    chalk.white.bold('Value'),
    chalk.white.bold('Description')
  ],
  colWidths: [25, 10, 50]
});

validationTypeTable.push([
  'ValidationIssueType.CircularDependency',
  ValidationIssueType.CircularDependency,
  'A circular dependency was detected in the graph'
]);

validationTypeTable.push([
  'ValidationIssueType.UnresolvedDependency',
  ValidationIssueType.UnresolvedDependency,
  'A dependency could not be resolved in the workspace'
]);

validationTypeTable.push([
  'ValidationIssueType.VersionConflict',
  ValidationIssueType.VersionConflict,
  'Different version requirements exist for the same dependency'
]);

console.log(validationTypeTable.toString());

// Example 9: Validating Package Dependencies
heading('9. Validating Package Dependencies');
info('Running a complete validation on the dependency graph:');

code(`
// Validate the graph with circular dependencies
const validationReport = circularGraph.validatePackageDependencies();

// Check if there are any issues
console.log(\`Has issues: \${validationReport.hasIssues}\`);
console.log(\`Has critical issues: \${validationReport.hasCriticalIssues}\`);
console.log(\`Has warnings: \${validationReport.hasWarnings}\`);

// Get all issues
const issues = validationReport.getIssues();
for (const issue of issues) {
  console.log(\`- \${issue.message} (Critical: \${issue.critical})\`);
}
`);

// Validate the graph
const validationReport = circularGraph.validatePackageDependencies();

// Check validation status
const validationStatusTable = new Table({
  chars: {
    'top': '═', 'top-mid': '╤', 'top-left': '╔', 'top-right': '╗',
    'bottom': '═', 'bottom-mid': '╧', 'bottom-left': '╚', 'bottom-right': '╝',
    'left': '║', 'left-mid': '╟', 'right': '║', 'right-mid': '╢',
    'mid': '─', 'mid-mid': '┼', 'middle': '│'
  },
  style: { head: ['cyan'] }
});

validationStatusTable.push(
  [chalk.bold('Has Issues'), validationReport.hasIssues ? chalk.red('Yes') : chalk.green('No')],
  [chalk.bold('Has Critical Issues'), validationReport.hasCriticalIssues ? chalk.red('Yes') : chalk.green('No')],
  [chalk.bold('Has Warnings'), validationReport.hasWarnings ? chalk.yellow('Yes') : chalk.green('No')]
);

console.log(validationStatusTable.toString());

// Display all validation issues
const issues = validationReport.getIssues();

if (issues.length > 0) {
  const issuesTable = new Table({
    head: [
      chalk.white.bold('Type'),
      chalk.white.bold('Severity'),
      chalk.white.bold('Message'),
      chalk.white.bold('Details')
    ],
    colWidths: [20, 10, 30, 30]
  });

  issues.forEach(issue => {
    let issueTypeText;
    let detailsText = '';

    // Format based on issue type
    switch (issue.issueType) {
      case ValidationIssueType.CircularDependency:
        issueTypeText = 'Circular Dependency';
        detailsText = issue.path ? `Cycle: ${issue.path.join(' → ')}` : '';
        break;
      case ValidationIssueType.UnresolvedDependency:
        issueTypeText = 'Unresolved Dependency';
        detailsText = issue.dependencyName ?
          `${issue.dependencyName}${issue.versionReq ? '@' + issue.versionReq : ''}` : '';
        break;
      case ValidationIssueType.VersionConflict:
        issueTypeText = 'Version Conflict';
        detailsText = issue.conflictingVersions ?
          `Versions: ${issue.conflictingVersions.join(', ')}` : '';
        break;
      default:
        issueTypeText = 'Unknown';
    }

    issuesTable.push([
      issueTypeText,
      issue.critical ? chalk.red('Critical') : chalk.yellow('Warning'),
      issue.message,
      detailsText
    ]);
  });

  console.log(issuesTable.toString());
} else {
  success('No validation issues found');
}

// Example 9b: Creating a ValidationReport Directly
heading('9b. Working with ValidationReport Directly');
info('Working with ValidationReport objects directly:');

code(`
// Import ValidationReport class
import { ValidationReport, ValidationIssueType } from '../src/index.js';

// Create a new ValidationReport
// Note: The actual API might differ - this is conceptual
const report = new ValidationReport();

// Add issues to the report
report.addIssue({
  issueType: ValidationIssueType.CircularDependency,
  message: "Circular dependency detected: A → B → C → A",
  critical: true,
  path: ["A", "B", "C"]
});
`);

info('Note: The ValidationReport class is typically created by the graph validation methods');
info('rather than instantiated directly. This ensures consistent validation logic.');

// Example 10: Real-World Scenario - Complex Project Analysis
heading('10. Real-World Scenario: Complex Project Analysis');
info('Analyzing a complex project structure with multiple dependencies:');

code(`
    // Create a complex project structure with multiple packages and dependencies
    const packages = [
      { name: 'app', version: '1.0.0', deps: ['ui-lib', 'api-client', 'util'] },
      { name: 'ui-lib', version: '2.0.0', deps: ['util', 'css-lib'] },
      { name: 'api-client', version: '1.5.0', deps: ['util', 'http-lib'] },
      { name: 'util', version: '3.1.0', deps: [] },
      { name: 'css-lib', version: '1.2.0', deps: [] },
      { name: 'http-lib', version: '2.0.0', deps: ['util'] },
      { name: 'test-lib', version: '1.0.0', deps: ['mocha', 'chai'] } // External deps
    ];

    // Create the package objects
    const projectPackages = packages.map(pkg => {
      const p = new Package(pkg.name, pkg.version);
      pkg.deps.forEach(dep => {
        const depVersion = packages.find(p => p.name === dep)?.version || '1.0.0';
        p.addDependency(new Dependency(dep, \`^\${depVersion}\`));
      });
      return p;
    });

    // Build the project graph
    const projectGraph = buildDependencyGraphFromPackages(projectPackages);

    // Analyze the project
    console.log('Project Analysis:');
    console.log(\`- Internally resolvable: \${projectGraph.isInternallyResolvable() ? 'Yes' : 'No'}\`);
    console.log(\`- Missing dependencies: \${projectGraph.findMissingDependencies().join(', ') || 'None'}\`);

    // Validate the project
    const projectValidation = projectGraph.validatePackageDependencies();
    console.log(\`- Has issues: \${projectValidation.hasIssues ? 'Yes' : 'No'}\`);
    `);

// Create a complex project structure
const projectDefs = [
  { name: 'app', version: '1.0.0', deps: ['ui-lib', 'api-client', 'util'] },
  { name: 'ui-lib', version: '2.0.0', deps: ['util', 'css-lib'] },
  { name: 'api-client', version: '1.5.0', deps: ['util', 'http-lib'] },
  { name: 'util', version: '3.1.0', deps: [] },
  { name: 'css-lib', version: '1.2.0', deps: [] },
  { name: 'http-lib', version: '2.0.0', deps: ['util'] },
  { name: 'test-lib', version: '1.0.0', deps: ['mocha', 'chai'] } // External deps
];

// Create packages
const projectPackages = projectDefs.map(pkg => {
  const p = new Package(pkg.name, pkg.version);
  pkg.deps.forEach(dep => {
    const depPkg = projectDefs.find(p => p.name === dep);
    const depVersion = depPkg ? depPkg.version : '1.0.0';
    p.addDependency(new Dependency(dep, `^${depVersion}`));
  });
  return p;
});

// Build project graph
const projectGraph = buildDependencyGraphFromPackages(projectPackages);

// Display project structure
subHeading('Project Structure:');
const projectStructureTable = new Table({
  head: [
    chalk.white.bold('Package'),
    chalk.white.bold('Version'),
    chalk.white.bold('Dependencies')
  ],
  colWidths: [15, 10, 50]
});

projectPackages.forEach(pkg => {
  const depsText = pkg.dependencies()
    .map(d => `${d.name}@${d.version}`)
    .join(', ');

  projectStructureTable.push([
    pkg.name,
    pkg.version,
    depsText || 'None'
  ]);
});

console.log(projectStructureTable.toString());

// Analyze the project
subHeading('Project Analysis:');
const projectAnalysisTable = new Table({
  chars: {
    'top': '═', 'top-mid': '╤', 'top-left': '╔', 'top-right': '╗',
    'bottom': '═', 'bottom-mid': '╧', 'bottom-left': '╚', 'bottom-right': '╝',
    'left': '║', 'left-mid': '╟', 'right': '║', 'right-mid': '╢',
    'mid': '─', 'mid-mid': '┼', 'middle': '│'
  },
  style: { head: ['cyan'] }
});

const missingDepsProject = projectGraph.findMissingDependencies();
projectAnalysisTable.push(
  [chalk.bold('Internally Resolvable'), projectGraph.isInternallyResolvable() ?
    chalk.green('Yes') : chalk.red('No')],
  [chalk.bold('Missing Dependencies'), missingDepsProject.length > 0 ?
    chalk.red(missingDepsProject.join(', ')) : chalk.green('None')],
  [chalk.bold('Has Cycles'), projectGraph.detectCircularDependencies() ?
    chalk.red('Yes') : chalk.green('No')]
);

console.log(projectAnalysisTable.toString());

// Validate the project
const projectValidation = projectGraph.validatePackageDependencies();

// Show issues breakdown
if (projectValidation.hasIssues) {
  // Group issues by type
  const criticalIssues = projectValidation.getCriticalIssues();
  const warnings = projectValidation.getWarnings();

  subHeading('Validation Issues:');

  if (criticalIssues.length > 0) {
    warning(`${criticalIssues.length} Critical Issues:`);
    criticalIssues.forEach((issue, i) => {
      warning(`  ${i + 1}. ${issue.message}`);
    });
  }

  if (warnings.length > 0) {
    info(`${warnings.length} Warnings:`);
    warnings.forEach((issue, i) => {
      info(`  ${i + 1}. ${issue.message}`);
    });
  }

  // Provide recommendations
  subHeading('Recommendations:');
  if (missingDepsProject.length > 0) {
    console.log(chalk.yellow('1. Add missing packages to the workspace:'));
    missingDepsProject.forEach(dep => {
      console.log(`   - ${dep}`);
    });
  }

  if (projectGraph.detectCircularDependencies()) {
    console.log(chalk.red('2. Resolve circular dependencies:'));
    const cycle = projectGraph.detectCircularDependencies();
    if (cycle) {
      console.log(`   - Break the dependency cycle: ${cycle.join(' → ')}`);
    }
  }
} else {
  success('Project dependency graph is valid with no issues.');
}

// Summary
console.log(createBox('Summary',
  chalk.bold('Key Concepts Demonstrated:') + '\n\n' +
  '✅ Using DependencyFilter to control which dependencies are included\n' +
  '✅ Building dependency graphs from packages\n' +
  '✅ Checking internal resolvability of dependencies\n' +
  '✅ Finding version conflicts in dependencies\n' +
  '✅ Detecting circular dependencies in the graph\n' +
  '✅ Finding dependents of packages\n' +
  '✅ Getting nodes from the graph\n' +
  '✅ Understanding validation issue types\n' +
  '✅ Validating package dependencies\n' +
  '✅ Performing complex project analysis'
));
