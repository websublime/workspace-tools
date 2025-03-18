import chalk from 'chalk';
import boxen from 'boxen';
import Table from 'cli-table3';
import path from 'node:path';
import os from 'node:os';
import {
  Package,
  Dependency,
  buildDependencyGraphFromPackages,
  generateAscii,
  generateDot,
  saveDotToFile
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

console.log(createBox('Graph Visualization Example',
  chalk.bold('This example demonstrates visualizing dependency graphs in different formats.')
));

// Example 1: Creating a Graph for Visualization
heading('1. Creating a Package Dependency Graph');
info('First, let\'s create a dependency graph to visualize:');

code(`
// Create a set of packages with dependencies
const packages = [
  { name: 'app', version: '1.0.0', deps: ['ui', 'api', 'util'] },
  { name: 'ui', version: '1.0.0', deps: ['util', 'theme'] },
  { name: 'api', version: '1.0.0', deps: ['util', 'db'] },
  { name: 'util', version: '1.0.0', deps: [] },
  { name: 'theme', version: '1.0.0', deps: ['util'] },
  { name: 'db', version: '1.0.0', deps: ['util'] }
];

// Create Package objects
const packageObjects = packages.map(pkg => {
  const p = new Package(pkg.name, pkg.version);
  return p;
});

// Add dependencies
packages.forEach((pkgDef, index) => {
  const pkg = packageObjects[index];
  pkgDef.deps.forEach(depName => {
    const depIndex = packages.findIndex(p => p.name === depName);
    if (depIndex >= 0) {
      const depPkg = packages[depIndex];
      pkg.addDependency(new Dependency(depName, \`^\${depPkg.version}\`));
    }
  });
});

// Build the dependency graph
const graph = buildDependencyGraphFromPackages(packageObjects);
`);

// Create a set of packages with dependencies
const packages = [
  { name: 'app', version: '1.0.0', deps: ['ui', 'api', 'util'] },
  { name: 'ui', version: '1.0.0', deps: ['util', 'theme'] },
  { name: 'api', version: '1.0.0', deps: ['util', 'db'] },
  { name: 'util', version: '1.0.0', deps: [] },
  { name: 'theme', version: '1.0.0', deps: ['util'] },
  { name: 'db', version: '1.0.0', deps: ['util'] }
];

// Create Package objects
const packageObjects = packages.map(pkg => {
  const p = new Package(pkg.name, pkg.version);
  return p;
});

// Add dependencies
packages.forEach((pkgDef, index) => {
  const pkg = packageObjects[index];
  pkgDef.deps.forEach(depName => {
    const depIndex = packages.findIndex(p => p.name === depName);
    if (depIndex >= 0) {
      const depPkg = packages[depIndex];
      pkg.addDependency(new Dependency(depName, `^${depPkg.version}`));
    }
  });
});

// Build the dependency graph
const graph = buildDependencyGraphFromPackages(packageObjects);
success('Dependency graph built successfully');

// Display the package structure
const structureTable = new Table({
  head: [chalk.white.bold('Package'), chalk.white.bold('Dependencies')],
  colWidths: [15, 50]
});

packages.forEach(pkg => {
  structureTable.push([
    pkg.name,
    pkg.deps.join(', ') || 'None'
  ]);
});

console.log(structureTable.toString());

// Example 2: Understanding DotOptions
heading('2. Understanding DotOptions');
info('The DotOptions object configures the DOT output generation:');

const dotOptionsTable = new Table({
  head: [
    chalk.white.bold('Option'),
    chalk.white.bold('Type'),
    chalk.white.bold('Description')
  ],
  colWidths: [20, 15, 50]
});

dotOptionsTable.push([
  'title',
  'string',
  'Title of the graph visualization'
]);

dotOptionsTable.push([
  'showExternal',
  'boolean',
  'Whether to include external (unresolved) dependencies'
]);

dotOptionsTable.push([
  'highlightCycles',
  'boolean',
  'Whether to highlight circular dependencies'
]);

console.log(dotOptionsTable.toString());

// Example 3: Generating ASCII Graph
heading('3. Generating ASCII Graph');
info('Creating a simple text-based visualization of the dependency graph:');

code(`
// Generate ASCII representation of the graph
const asciiGraph = generateAscii(graph);
console.log(asciiGraph);
`);

try {
  const asciiGraph = generateAscii(graph);

  console.log(boxen(asciiGraph, {
    padding: 1,
    borderColor: 'blue',
    title: 'ASCII Graph',
    titleAlignment: 'center'
  }));

  success('ASCII graph generated successfully');
} catch (err) {
  warning(`Failed to generate ASCII graph: ${err.message}`);
}

// Example 4: Generating DOT Graph
heading('4. Generating DOT Graph');
info('Creating a DOT format visualization of the dependency graph:');

code(`
// Create dot options
const dotOptions = {
  title: 'Project Dependencies',
  showExternal: true,
  highlightCycles: true
};

// Generate DOT representation
const dotGraph = generateDot(graph, dotOptions);
console.log(dotGraph);
`);

// Create dot options
const dotOptions = {
  title: 'Project Dependencies',
  showExternal: true,
  highlightCycles: true
};

try {
  const dotGraph = generateDot(graph, dotOptions);

  console.log(boxen(dotGraph.substring(0, 500) + '...\n(truncated for display)', {
    padding: 1,
    borderColor: 'blue',
    title: 'DOT Graph (Truncated)',
    titleAlignment: 'center'
  }));

  success('DOT graph generated successfully');
} catch (err) {
  warning(`Failed to generate DOT graph: ${err.message}`);
}

// Example 5: Saving DOT to File
heading('5. Saving DOT to File');
info('Saving the DOT representation to a file for use with external tools:');

code(`
// Save DOT representation to file
const tempFilePath = path.join(os.tmpdir(), 'dependency-graph.dot');
await saveDotToFile(dotGraph, tempFilePath);
console.log(\`DOT file saved to: \${tempFilePath}\`);
`);

// Create a temporary file path
const tempFilePath = path.join(os.tmpdir(), 'dependency-graph.dot');

try {
  // Generate the DOT
  const dotGraph = generateDot(graph, dotOptions);

  // Save to file
  await saveDotToFile(dotGraph, tempFilePath);
  success(`DOT file saved to: ${tempFilePath}`);

  info('\nTo visualize this file, you can use tools like Graphviz:');
  code(`dot -Tpng ${tempFilePath} -o dependency-graph.png`);
  code(`dot -Tsvg ${tempFilePath} -o dependency-graph.svg`);
} catch (err) {
  warning(`Failed to save DOT file: ${err.message}`);
}

// Example 6: Creating a Graph with Circular Dependencies
heading('6. Visualizing Circular Dependencies');
info('Generating visualizations that highlight circular dependencies:');

code(`
// Create packages with circular dependencies
const pkgA = new Package('package-a', '1.0.0');
const pkgB = new Package('package-b', '1.0.0');
const pkgC = new Package('package-c', '1.0.0');

// Create circular dependency: A -> B -> C -> A
pkgA.addDependency(new Dependency('package-b', '^1.0.0'));
pkgB.addDependency(new Dependency('package-c', '^1.0.0'));
pkgC.addDependency(new Dependency('package-a', '^1.0.0')); // Creates a cycle

// Build the graph
const circularGraph = buildDependencyGraphFromPackages([pkgA, pkgB, pkgC]);

// Configure options to highlight cycles
const cycleOptions = {
  title: 'Circular Dependency Example',
  showExternal: false,
  highlightCycles: true
};

// Generate DOT with highlighted cycles
const cycleDot = generateDot(circularGraph, cycleOptions);
`);

// Create packages with circular dependencies
const pkgA = new Package('package-a', '1.0.0');
const pkgB = new Package('package-b', '1.0.0');
const pkgC = new Package('package-c', '1.0.0');

// Create circular dependency: A -> B -> C -> A
pkgA.addDependency(new Dependency('package-b', '^1.0.0'));
pkgB.addDependency(new Dependency('package-c', '^1.0.0'));
pkgC.addDependency(new Dependency('package-a', '^1.0.0')); // Creates a cycle

// Build the graph
const circularGraph = buildDependencyGraphFromPackages([pkgA, pkgB, pkgC]);

// Configure options to highlight cycles
const cycleOptions = {
  title: 'Circular Dependency Example',
  showExternal: false,
  highlightCycles: true
};

try {
  // Generate visualizations
  const cycleAscii = generateAscii(circularGraph);
  const cycleDot = generateDot(circularGraph, cycleOptions);

  console.log(boxen(cycleAscii, {
    padding: 1,
    borderColor: 'red',
    title: 'ASCII Graph with Cycle',
    titleAlignment: 'center'
  }));

  // Save the circular dependency DOT file
  const cycleDotPath = path.join(os.tmpdir(), 'circular-dependency.dot');
  await saveDotToFile(cycleDot, cycleDotPath);
  success(`Circular dependency DOT file saved to: ${cycleDotPath}`);

  info('\nNotice how the circular dependency is represented in the visualization.');
} catch (err) {
  warning(`Failed to generate cycle visualization: ${err.message}`);
}

// Example 7: Real-World Scenario - Monorepo Visualization
heading('7. Real-World Scenario: Monorepo Visualization');
info('Visualizing a monorepo structure with internal and external dependencies:');

code(`
// Create a monorepo structure
const monorepoPackages = [
  { name: '@company/core', version: '1.0.0', deps: ['lodash'] },
  { name: '@company/ui', version: '1.0.0', deps: ['@company/core', 'react', 'styled-components'] },
  { name: '@company/api', version: '1.0.0', deps: ['@company/core', 'express', 'mongoose'] },
  { name: '@company/admin', version: '1.0.0', deps: ['@company/ui', '@company/api'] },
  { name: '@company/client', version: '1.0.0', deps: ['@company/ui', '@company/api'] }
];

// Create Package objects for the monorepo
const monorepoObjects = monorepoPackages.map(pkg => {
  const p = new Package(pkg.name, pkg.version);
  return p;
});

// Add dependencies
monorepoPackages.forEach((pkgDef, index) => {
  const pkg = monorepoObjects[index];
  pkgDef.deps.forEach(depName => {
    // Find the dependency in our monorepo
    const depIndex = monorepoPackages.findIndex(p => p.name === depName);
    if (depIndex >= 0) {
      // Internal dependency
      const depPkg = monorepoPackages[depIndex];
      pkg.addDependency(new Dependency(depName, \`^\${depPkg.version}\`));
    } else {
      // External dependency
      pkg.addDependency(new Dependency(depName, '^1.0.0'));
    }
  });
});
`);

// Create a monorepo structure
const monorepoPackages = [
  { name: '@company/core', version: '1.0.0', deps: ['lodash'] },
  { name: '@company/ui', version: '1.0.0', deps: ['@company/core', 'react', 'styled-components'] },
  { name: '@company/api', version: '1.0.0', deps: ['@company/core', 'express', 'mongoose'] },
  { name: '@company/admin', version: '1.0.0', deps: ['@company/ui', '@company/api'] },
  { name: '@company/client', version: '1.0.0', deps: ['@company/ui', '@company/api'] }
];

// Create Package objects for the monorepo
const monorepoObjects = monorepoPackages.map(pkg => {
  const p = new Package(pkg.name, pkg.version);
  return p;
});

// Add dependencies
monorepoPackages.forEach((pkgDef, index) => {
  const pkg = monorepoObjects[index];
  pkgDef.deps.forEach(depName => {
    // Find the dependency in our monorepo
    const depIndex = monorepoPackages.findIndex(p => p.name === depName);
    if (depIndex >= 0) {
      // Internal dependency
      const depPkg = monorepoPackages[depIndex];
      pkg.addDependency(new Dependency(depName, `^${depPkg.version}`));
    } else {
      // External dependency
      pkg.addDependency(new Dependency(depName, '^1.0.0'));
    }
  });
});

// Build the monorepo graph and continue with visualization
code(`
// Build the monorepo graph
const monorepoGraph = buildDependencyGraphFromPackages(monorepoObjects);

// Configure options to show external dependencies
const monorepoOptions = {
  title: 'Monorepo Structure',
  showExternal: true, // Show external dependencies
  highlightCycles: true
};

// Generate visualizations
const monorepoAscii = generateAscii(monorepoGraph);
const monorepoDot = generateDot(monorepoGraph, monorepoOptions);

// Save the DOT file
const monorepoDotPath = path.join(os.tmpdir(), 'monorepo-structure.dot');
await saveDotToFile(monorepoDot, monorepoDotPath);
`);

// Build the monorepo graph
const monorepoGraph = buildDependencyGraphFromPackages(monorepoObjects);

// Configure options to show external dependencies
const monorepoOptions = {
  title: 'Monorepo Structure',
  showExternal: true, // Show external dependencies
  highlightCycles: true
};

try {
  // Generate visualizations
  const monorepoAscii = generateAscii(monorepoGraph);
  const monorepoDot = generateDot(monorepoGraph, monorepoOptions);

  console.log(boxen(monorepoAscii, {
    padding: 1,
    borderColor: 'green',
    title: 'Monorepo ASCII Graph',
    titleAlignment: 'center'
  }));

  // Save the DOT file
  const monorepoDotPath = path.join(os.tmpdir(), 'monorepo-structure.dot');
  await saveDotToFile(monorepoDot, monorepoDotPath);
  success(`Monorepo visualization saved to: ${monorepoDotPath}`);

  // Note for users
  info('\nThe DOT file shows both internal (monorepo) dependencies and external dependencies');
  info('Internal dependencies: @company/... packages');
  info('External dependencies: react, lodash, etc.');
} catch (err) {
  warning(`Failed to generate monorepo visualization: ${err.message}`);
}

// Example 8: Understanding Graphviz Integration
heading('8. Integrating with Graphviz');
info('While our tools generate DOT format, you\'ll need Graphviz to render visual graphs:');

const graphvizTable = new Table({
  head: [
    chalk.white.bold('Command'),
    chalk.white.bold('Description')
  ],
  colWidths: [30, 60]
});

graphvizTable.push([
  'dot -Tpng input.dot -o output.png',
  'Render a DOT file to PNG format'
]);

graphvizTable.push([
  'dot -Tsvg input.dot -o output.svg',
  'Render a DOT file to SVG format'
]);

graphvizTable.push([
  'dot -Tpdf input.dot -o output.pdf',
  'Render a DOT file to PDF format'
]);

graphvizTable.push([
  'neato -Tpng input.dot -o output.png',
  'Use alternative layout algorithm for the visualization'
]);

console.log(graphvizTable.toString());

// Final overview and conclusion
heading('Conclusion');
info('In this example, you\'ve learned how to:');
success('✓ Build dependency graphs from packages');
success('✓ Understand and configure visualization options using DotOptions');
success('✓ Generate ASCII visualizations for quick terminal viewing');
success('✓ Generate DOT format visualizations for high-quality graphics');
success('✓ Save DOT files for use with external tools like Graphviz');
success('✓ Visualize and detect circular dependencies');
success('✓ Create real-world visualizations of complex project structures');

console.log(createBox('Next Steps',
  chalk.cyan('1. Install Graphviz to render the saved DOT files into images\n') +
  chalk.cyan('2. Integrate these visualizations into your dependency analysis workflows\n') +
  chalk.cyan('3. Use the generated diagrams to identify potential issues in your dependency structure')
));

console.log('\n');
