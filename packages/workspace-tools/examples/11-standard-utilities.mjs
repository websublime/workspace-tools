import chalk from 'chalk';
import boxen from 'boxen';
import Table from 'cli-table3';
import {
  CorePackageManager,
  detectPackageManager,
  execute,
  executeWithStatus,
  getProjectRootPath,
  stripTrailingNewline
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

console.log(createBox('Standard Utilities Example',
  chalk.bold('This example demonstrates the core functionality of the ws_std module.')
));

// Example 1: Command Execution
heading('1. Executing Commands');
info('The execute and executeWithStatus functions allow you to run shell commands:');

code(`
// Basic command execution
const output = await execute('echo', '.', ['Hello from the shell']);
console.log(\`Command output: \${output}\`);

// Execute with status information
const result = await executeWithStatus('ls', '.', ['-la']);
console.log(\`Exit code: \${result.exitCode}\`);
console.log(\`Output: \${result.stdout.substring(0, 100)}...\`);
console.log(\`Errors: \${result.stderr || 'None'}\`);
`);

try {
  // Run a simple echo command
  info('\nRunning a simple command:');
  const output = await execute('echo', '.', ['Hello from the shell']);
  success(`Command output: ${stripTrailingNewline(output)}`);

  // Run a command with status
  info('\nRunning a command with status information:');
  const result = await executeWithStatus('ls', '.', ['-la']);

  const statusTable = new Table({
    head: [chalk.white.bold('Property'), chalk.white.bold('Value')],
    colWidths: [15, 65]
  });

  statusTable.push(
    ['Exit Code', result.exitCode],
    ['stdout', result.stdout.substring(0, 100) + (result.stdout.length > 100 ? '...' : '')],
    ['stderr', result.stderr || 'None']
  );

  console.log(statusTable.toString());
} catch (err) {
  warning(`Error executing command: ${err.message}`);
}

// Example 2: Handling Command Errors
heading('2. Handling Command Errors');
info('When a command fails, you can get details about the error:');

code(`
try {
  // Execute a command that will fail
  const output = await execute('some-invalid-command', '.', []);
  console.log(output);
} catch (err) {
  console.error(\`Command failed: \${err.message}\`);
}

// Or use executeWithStatus
const result = await executeWithStatus('some-invalid-command', '.', []);
if (result.exitCode !== 0) {
  console.error(\`Command failed with exit code: \${result.exitCode}\`);
  console.error(\`Error output: \${result.stderr}\`);
}
`);

subHeading('Error Handling Demo:');
info('Attempting to run a non-existent command:');

try {
  // Try to execute an invalid command
  await execute('command-that-does-not-exist', '.', []);
} catch (err) {
  warning('Command execution failed as expected');
  warning(`Error message: ${err.message}`);
}

info('\nUsing executeWithStatus for error handling:');
try {
  const badResult = await executeWithStatus('command-that-does-not-exist', '.', []);

  const errorTable = new Table({
    head: [chalk.white.bold('Property'), chalk.white.bold('Value')],
    colWidths: [20, 60]
  });

  errorTable.push(
    ['Exit Code', badResult.exitCode],
    ['stdout', badResult.stdout || 'None'],
    ['stderr', badResult.stderr || 'None']
  );

  console.log(errorTable.toString());

  if (badResult.exitCode !== 0) {
    warning(`Command failed with exit code: ${badResult.exitCode}`);
  }
} catch (err) {
  warning(`Error executing command: ${err.message}`);
}

// Example 3: Working with Package Managers
heading('3. Working with Package Managers');
info('The CorePackageManager enum and detectPackageManager function help with package manager detection:');

code(`
// List all available package managers
console.log('Available package managers:');
for (const pm in CorePackageManager) {
  console.log(\`- \${pm}: \${CorePackageManager[pm]}\`);
}

// Detect the package manager used in the current project
const detectedPM = await detectPackageManager();
console.log(\`Detected package manager: \${detectedPM}\`);

// Check if a specific package manager is being used
if (detectedPM === CorePackageManager.NPM) {
  console.log('This project uses npm');
} else if (detectedPM === CorePackageManager.Yarn) {
  console.log('This project uses Yarn');
} else if (detectedPM === CorePackageManager.PNPM) {
  console.log('This project uses PNPM');
} else {
  console.log('Using another package manager or none detected');
}
`);

// Show all available package managers
info('Available package managers:');
const pmTable = new Table({
  head: [
    chalk.white.bold('Name'),
    chalk.white.bold('Value'),
    chalk.white.bold('Description')
  ],
  colWidths: [15, 15, 50]
});

for (const pm in CorePackageManager) {
  let description;
  switch (pm) {
    case 'NPM':
      description = 'The default package manager for Node.js';
      break;
    case 'Yarn':
      description = 'Fast, reliable, and secure dependency management';
      break;
    case 'PNPM':
      description = 'Fast, disk space efficient package manager';
      break;
    default:
      description = 'Other package manager';
  }

  pmTable.push([pm, CorePackageManager[pm], description]);
}

console.log(pmTable.toString());

// Detect the package manager in the current project
subHeading('Package Manager Detection:');
try {
  const detectedPM = await detectPackageManager();
  success(`Detected package manager: ${detectedPM !== null ? detectedPM : 'None detected'}`);

  if (detectedPM !== null) {
    // Show which package manager was detected
    switch (detectedPM) {
      case CorePackageManager.NPM:
        info('This project uses npm');
        break;
      case CorePackageManager.Yarn:
        info('This project uses Yarn');
        break;
      case CorePackageManager.PNPM:
        info('This project uses PNPM');
        break;
      default:
        info('Using another package manager');
    }

    // Demonstrate how to use the detected package manager
    info('\nWith the detected package manager, you could:');

    const installCmd = detectedPM === CorePackageManager.NPM ? 'npm' :
      detectedPM === CorePackageManager.Yarn ? 'yarn' :
        detectedPM === CorePackageManager.PNPM ? 'pnpm' : 'unknown';

    code(`
    // Install a package
    await execute('${installCmd}', '.', ['install', 'lodash']);

    // Run a script
    await execute('${installCmd}', '.', ['run', 'build']);
    `);
  } else {
    warning('No package manager detected for this project');
  }
} catch (err) {
  warning(`Error detecting package manager: ${err.message}`);
}

// Example 4: Project Root Path
heading('4. Finding Project Root Path');
info('The getProjectRootPath function helps locate the root of your project:');

code(`
// Get the project root path
const rootPath = await getProjectRootPath();
console.log(\`Project root: \${rootPath}\`);

// Use the root path in other operations
const packageJsonPath = \`\${rootPath}/package.json\`;
console.log(\`Path to package.json: \${packageJsonPath}\`);
`);

try {
  // Get the project root path
  const rootPath = await getProjectRootPath();

  const pathInfo = new Table({
    head: [chalk.white.bold('Path'), chalk.white.bold('Value')],
    colWidths: [20, 60]
  });

  pathInfo.push(
    ['Project Root', rootPath || 'Not found'],
    ['package.json', rootPath ? `${rootPath}/package.json` : 'N/A'],
    ['node_modules', rootPath ? `${rootPath}/node_modules` : 'N/A']
  );

  console.log(pathInfo.toString());

  if (rootPath) {
    success('Successfully located project root');

    info('\nCommon use cases for project root:');
    info('- Loading configuration files');
    info('- Resolving relative paths');
    info('- Finding package.json and other project files');

    // Show a practical example
    subHeading('Practical Example:');
    code(`
    const fs = require('fs');
    const path = require('path');

    // Load package.json
    const rootPath = await getProjectRootPath();
    const packageJsonPath = path.join(rootPath, 'package.json');
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));

    console.log(\`Project name: \${packageJson.name}\`);
    console.log(\`Project version: \${packageJson.version}\`);
    `);
  } else {
    warning('Could not find project root path');
  }
} catch (err) {
  warning(`Error finding project root: ${err.message}`);
}

// Example 5: String Utilities
heading('5. String Utilities');
info('The stripTrailingNewline function removes trailing newlines from a string:');

code(`
// Command outputs often end with newlines
const output = await execute('echo', '.', ['Hello World']);
console.log(\`Original output: "\${output}"\`);

// Strip trailing newline
const cleaned = stripTrailingNewline(output);
console.log(\`Cleaned output: "\${cleaned}"\`);
`);

try {
  // Generate some strings with trailing newlines for demonstration
  const examples = [
    { input: "Hello\n", output: stripTrailingNewline("Hello\n") },
    { input: "Hello\r\n", output: stripTrailingNewline("Hello\r\n") },
    { input: "Hello", output: stripTrailingNewline("Hello") },
    { input: "Hello\n\n", output: stripTrailingNewline("Hello\n\n") }
  ];

  const strTable = new Table({
    head: [chalk.white.bold('Original'), chalk.white.bold('Cleaned'), chalk.white.bold('Original Length'), chalk.white.bold('Cleaned Length')],
    colWidths: [20, 20, 15, 15]
  });

  examples.forEach(ex => {
    strTable.push([
      JSON.stringify(ex.input),
      JSON.stringify(ex.output),
      ex.input.length,
      ex.output.length
    ]);
  });

  console.log(strTable.toString());

  // Try with real command output
  try {
    const cmdOutput = await execute('echo', '.', ['Command Output Test']);
    success(`\nCommand output before: "${cmdOutput}" (${cmdOutput.length} chars)`);
    const cleanedOutput = stripTrailingNewline(cmdOutput);
    success(`Command output after: "${cleanedOutput}" (${cleanedOutput.length} chars)`);
  } catch (err) {
    warning(`Couldn't demonstrate with command: ${err.message}`);
  }

} catch (err) {
  warning(`Error in string utilities demo: ${err.message}`);
}

// Example 6: Real-World Scenario - Project Analysis Script
heading('6. Real-World Scenario: Project Analysis Script');
info('Let\'s combine these utilities to create a project analysis script:');

code(`
async function analyzeProject() {
  try {
    // Find the project root
    const rootPath = await getProjectRootPath();
    console.log(\`Project root: \${rootPath}\`);

    // Detect package manager
    const pm = await detectPackageManager();
    console.log(\`Package manager: \${pm}\`);

    // Get package dependencies
    let dependencies;
    if (pm === CorePackageManager.NPM) {
      dependencies = await execute('npm', rootPath, ['list', '--depth=0']);
    } else if (pm === CorePackageManager.Yarn) {
      dependencies = await execute('yarn', rootPath, ['list', '--depth=0']);
    } else if (pm === CorePackageManager.PNPM) {
      dependencies = await execute('pnpm', rootPath, ['list', '--depth=0']);
    }
    console.log(\`Dependencies:\\n\${dependencies}\`);

    // Check Node.js version
    const nodeVersion = await execute('node', rootPath, ['--version']);
    console.log(\`Node.js version: \${stripTrailingNewline(nodeVersion)}\`);

    // Check git status
    try {
      const gitStatus = await execute('git', rootPath, ['status', '--porcelain']);
      if (gitStatus.trim()) {
        console.log('Project has uncommitted changes');
      } else {
        console.log('Working directory is clean');
      }
    } catch (err) {
      console.log('Not a git repository or git not installed');
    }

    return {
      rootPath,
      packageManager: pm,
      nodeVersion: stripTrailingNewline(nodeVersion)
    };
  } catch (err) {
    console.error('Error analyzing project:', err);
    throw err;
  }
}
`);

// Show how this would work by demonstrating parts of it
subHeading('Project Analysis Demo:');
info('Running a partial project analysis...');

async function runPartialAnalysis() {
  try {
    // Create a results table
    const resultsTable = new Table({
      head: [chalk.white.bold('Information'), chalk.white.bold('Value')],
      colWidths: [25, 55]
    });

    // Find the project root
    const rootPath = await getProjectRootPath();
    resultsTable.push(['Project Root', rootPath || 'Not found']);

    // Detect package manager
    const pm = await detectPackageManager();
    resultsTable.push(['Package Manager', pm || 'Not detected']);

    // Check Node.js version
    try {
      const nodeVersion = await execute('node', '.', ['--version']);
      resultsTable.push(['Node.js Version', stripTrailingNewline(nodeVersion)]);
    } catch (err) {
      resultsTable.push(['Node.js Version', 'Error detecting version']);
    }

    // Check git status (if available)
    try {
      const gitStatus = await execute('git', '.', ['status', '--porcelain']);
      if (gitStatus.trim()) {
        resultsTable.push(['Git Status', 'Uncommitted changes']);
      } else {
        resultsTable.push(['Git Status', 'Working directory clean']);
      }
    } catch (err) {
      resultsTable.push(['Git Status', 'Not a git repository or git not installed']);
    }

    // Try to list directory contents
    try {
      const fileList = await execute('ls', '.', ['-la']);
      const fileCount = fileList.split('\n').length - 1;
      resultsTable.push(['Directory Files', `${fileCount} entries`]);
    } catch (err) {
      resultsTable.push(['Directory Files', 'Error listing files']);
    }

    console.log(resultsTable.toString());

    return {
      rootPath,
      packageManager: pm
    };
  } catch (err) {
    warning(`Error in analysis: ${err.message}`);
    return {};
  }
}

// Run the partial analysis
await runPartialAnalysis();
success('Analysis complete!');

// Example 7: Working with Different Package Managers
heading('7. Working with Different Package Managers');
info('Here\'s how to perform common operations with different package managers:');

const pmOperationsTable = new Table({
  head: [
    chalk.white.bold('Operation'),
    chalk.white.bold('NPM'),
    chalk.white.bold('Yarn'),
    chalk.white.bold('PNPM')
  ],
  colWidths: [15, 25, 25, 25]
});

// Add rows for different operations
pmOperationsTable.push(
  ['Install all', 'npm install', 'yarn', 'pnpm install'],
  ['Add a package', 'npm install lodash', 'yarn add lodash', 'pnpm add lodash'],
  ['Add dev package', 'npm install -D jest', 'yarn add -D jest', 'pnpm add -D jest'],
  ['Remove package', 'npm uninstall lodash', 'yarn remove lodash', 'pnpm remove lodash'],
  ['Update packages', 'npm update', 'yarn upgrade', 'pnpm update'],
  ['Run script', 'npm run build', 'yarn build', 'pnpm build']
);

console.log(pmOperationsTable.toString());

// Demonstrate how to use the detected package manager programmatically
subHeading('Programmatic package manager usage:');
const detectedPM = await detectPackageManager();

if (detectedPM) {
  info(`Detected package manager: ${detectedPM}`);

  // Show command examples
  const cmdExamples = new Table({
    head: [chalk.white.bold('Task'), chalk.white.bold('Command')],
    colWidths: [20, 60]
  });

  if (detectedPM === CorePackageManager.NPM) {
    cmdExamples.push(
      ['Install dependencies', 'execute("npm", ".", ["install"])'],
      ['Run build script', 'execute("npm", ".", ["run", "build"])'],
      ['Install package', 'execute("npm", ".", ["install", "lodash"])']
    );
  } else if (detectedPM === CorePackageManager.Yarn) {
    cmdExamples.push(
      ['Install dependencies', 'execute("yarn", ".", [])'],
      ['Run build script', 'execute("yarn", ".", ["build"])'],
      ['Install package', 'execute("yarn", ".", ["add", "lodash"])']
    );
  } else if (detectedPM === CorePackageManager.PNPM) {
    cmdExamples.push(
      ['Install dependencies', 'execute("pnpm", ".", ["install"])'],
      ['Run build script', 'execute("pnpm", ".", ["run", "build"])'],
      ['Install package', 'execute("pnpm", ".", ["add", "lodash"])']
    );
  }

  console.log(cmdExamples.toString());
} else {
  warning('No package manager detected');
}

// Example 8: Using getProjectRootPath with commands
heading('8. Using Project Root Path with Commands');
info('The getProjectRootPath function helps you run commands in the right directory:');

code(`
    async function runInProjectRoot(command, args) {
      const rootPath = await getProjectRootPath();
      if (!rootPath) {
        throw new Error('Could not find project root');
      }

      console.log(\`Running \${command} in \${rootPath}\`);
      return await execute(command, rootPath, args);
    }

    // Example usage
    const packageJson = await runInProjectRoot('cat', ['package.json']);
    const packageInfo = JSON.parse(packageJson);
    console.log(\`Project name: \${packageInfo.name}\`);
    console.log(\`Project version: \${packageInfo.version}\`);
    `);

// Example 9: String Processing with stripTrailingNewline
heading('9. String Processing with stripTrailingNewline');
info('When working with command outputs, stripTrailingNewline comes in handy:');

const stringExamples = new Table({
  head: [
    chalk.white.bold('Input String'),
    chalk.white.bold('After stripTrailingNewline'),
    chalk.white.bold('Length Change')
  ],
  colWidths: [25, 25, 20]
});

[
  "No newline",
  "One newline\n",
  "Carriage return\r\n",
  "Multiple newlines\n\n"
].forEach(input => {
  const output = stripTrailingNewline(input);
  stringExamples.push([
    JSON.stringify(input),
    JSON.stringify(output),
    `${input.length} → ${output.length}`
  ]);
});

console.log(stringExamples.toString());

// Example 10: Safer Command Execution Pattern
heading('10. Safer Command Execution Pattern');
info('A safer pattern for executing commands with proper error handling:');

code(`
    async function safeExecute(cmd, dir, args) {
      try {
        return {
          success: true,
          output: await execute(cmd, dir, args)
        };
      } catch (err) {
        return {
          success: false,
          error: err.message
        };
      }
    }

    // Using the safe execute pattern
    const { success, output, error } = await safeExecute('echo', '.', ['Hello, World!']);

    if (success) {
      console.log('Command output:', output);
    } else {
      console.error('Command failed:', error);
    }
    `);

// Demonstrate the pattern
subHeading('Demonstrating safe command execution:');

async function safeExecute(cmd, dir, args) {
  try {
    return {
      success: true,
      output: await execute(cmd, dir, args)
    };
  } catch (err) {
    return {
      success: false,
      error: err.message
    };
  }
}

// Try a successful command
const successResult = await safeExecute('echo', '.', ['This should work']);
info('Executing a command that should succeed:');
if (successResult.success) {
  success(`Command succeeded with output: ${stripTrailingNewline(successResult.output)}`);
} else {
  warning(`Command failed: ${successResult.error}`);
}

// Try a failing command
const failResult = await safeExecute('some-invalid-command', '.', []);
info('\nExecuting a command that should fail:');
if (failResult.success) {
  success(`Command succeeded with output: ${stripTrailingNewline(failResult.output)}`);
} else {
  warning(`Command failed: ${failResult.error}`);
}

// Final summary box
console.log(createBox('Summary',
  chalk.bold('Key Concepts Demonstrated:') + '\n\n' +
  '✅ Executing shell commands with execute and executeWithStatus\n' +
  '✅ Handling command execution errors properly\n' +
  '✅ Working with different package managers\n' +
  '✅ Finding project root paths\n' +
  '✅ Processing command output strings\n' +
  '✅ Building real-world project analysis tools\n' +
  '✅ Implementing safe command execution patterns'
));

console.log(createBox('Next Steps',
  chalk.cyan('1. Integrate these utilities into your build scripts\n') +
  chalk.cyan('2. Create custom project analysis tools\n') +
  chalk.cyan('3. Build cross-platform compatible command execution wrappers\n') +
  chalk.cyan('4. Develop package manager agnostic scripts')
));
