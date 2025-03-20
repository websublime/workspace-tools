#!/usr/bin/env node
// @ts-check

/**
 * @typedef {import('../dist/types').getProjectRootPath} getProjectRootPath
 */

import { getProjectRootPath } from '../dist/esm/index.mjs';
import chalk from 'chalk';
import boxen from 'boxen';
import path from 'node:path';
import fs from 'node:fs';

/**
 * Demonstrates how to use the getProjectRootPath function
 * with proper error handling and visual output.
 */
async function demonstrateProjectRoot() {
  console.clear();

  // Display header
  console.log(boxen(chalk.bold.blue('Project Root Path Detector'), {
    padding: 1,
    margin: 1,
    borderStyle: 'double',
    borderColor: 'blue',
    title: 'Example 01',
    titleAlignment: 'center'
  }));

  console.log(chalk.yellow('This example demonstrates how to find the root of your project.'));
  console.log(chalk.dim('The function walks up directories looking for repository markers.\n'));

  try {
    // Case 1: Default behavior (using current directory)
    console.log(chalk.green('✓ Finding project root from current directory:'));
    const rootPath = getProjectRootPath();
    console.log(boxen(chalk.white(rootPath), {
      padding: 1,
      margin: { top: 0, bottom: 1, left: 2, right: 2 },
      borderStyle: 'round',
      borderColor: 'green'
    }));

    // Show what files exist in the root
    const files = fs.readdirSync(rootPath)
      .filter(file => !file.startsWith('.'))
      .slice(0, 5);

    console.log(chalk.green('  Files in project root:'));
    // biome-ignore lint/complexity/noForEach: <explanation>
    files.forEach(file => {
      console.log(chalk.dim(`  • ${file}`));
    });
    if (files.length === 5) console.log(chalk.dim('  • ...'));
    console.log();

    // Case 2: Custom path
    const customPath = path.join(process.cwd(), 'node_modules');
    console.log(chalk.green('✓ Finding project root from a custom path:'));
    console.log(chalk.dim(`  Custom path: ${customPath}`));

    try {
      const customRootPath = getProjectRootPath(customPath);
      console.log(boxen(chalk.white(customRootPath), {
        padding: 1,
        margin: { top: 0, bottom: 1, left: 2, right: 2 },
        borderStyle: 'round',
        borderColor: 'green'
      }));
    } catch (error) {
      console.log(boxen(chalk.red(`Error: ${error.message}`), {
        padding: 1,
        margin: { top: 0, bottom: 1, left: 2, right: 2 },
        borderStyle: 'round',
        borderColor: 'red'
      }));
    }

    // Case 3: Handling errors with non-existent path
    console.log(chalk.yellow('✓ Error handling with invalid path:'));
    try {
      getProjectRootPath('/path/does/not/exist');
    } catch (error) {
      console.log(boxen(chalk.red(`Error: ${error.message}`), {
        padding: 1,
        margin: { top: 0, bottom: 1, left: 2, right: 2 },
        borderStyle: 'round',
        borderColor: 'red'
      }));
    }

  } catch (error) {
    console.log(boxen(chalk.red(`Error: ${error.message}`), {
      padding: 1,
      margin: 1,
      borderStyle: 'round',
      borderColor: 'red'
    }));
  }

  console.log(chalk.blue('API Usage:'));
  console.log(chalk.dim(`
  // Get project root from current directory
  const rootPath = getProjectRootPath();

  // Get project root from a specific path
  const customRootPath = getProjectRootPath('/custom/path');
  `));
}

// Execute the demonstration
demonstrateProjectRoot().catch(console.error);
