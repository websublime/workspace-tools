#!/usr/bin/env node
// @ts-check

/**
 * @typedef {import('../dist/types').executeCommand} executeCommand
 */

import { executeCommand } from '../dist/esm/index.mjs';
import chalk from 'chalk';
import boxen from 'boxen';
import Table from 'cli-table3';

/**
 * Demonstrates how to use the executeCommand function
 * with proper error handling and visual output.
 */
async function demonstrateCommandExecution() {
  console.clear();

  // Display header
  console.log(boxen(chalk.bold.magenta('Command Execution Tool'), {
    padding: 1,
    margin: 1,
    borderStyle: 'double',
    borderColor: 'magenta',
    title: 'Example 02',
    titleAlignment: 'center'
  }));

  console.log(chalk.yellow('This example demonstrates how to execute shell commands from Node.js.'));
  console.log(chalk.dim('The function runs commands and captures their output.\n'));

  // Create a table for our examples
  const examplesTable = new Table({
    head: [
      chalk.white.bold('Command'),
      chalk.white.bold('Working Directory'),
      chalk.white.bold('Arguments'),
      chalk.white.bold('Description')
    ],
    style: {
      head: [], // Disable color in header
      border: [], // Disable color for borders
    }
  });

  // Add our examples to the table
  examplesTable.push(
    ['git', '.', ['--version'], 'Get Git version'],
    ['ls', '.', ['-la'], 'List files in detail'],
    ['echo', '.', ['Hello World'], 'Simple echo command'],
    ['git', '.', ['status'], 'Show repository status'],
    ['nonexistent', '.', [], 'Command that does not exist'],
    ['git', '.', ['non-existent-command'], 'Invalid Git subcommand']
  );

  console.log(examplesTable.toString());
  console.log();

  // Run our examples
  for (const [cmd, dir, args, description] of examplesTable.slice(0).map(row => row)) {
    console.log(chalk.cyan(`▶ Running: ${chalk.bold(cmd + ' ' + args.join(' '))}`));
    console.log(chalk.dim(`  ${description}`));

    try {
      // Execute the command
      const output = executeCommand(cmd, dir, args);

      // Show successful output
      console.log(boxen(chalk.green(output.trim() || '(no output)'), {
        padding: 1,
        margin: { top: 0, bottom: 1, left: 2, right: 2 },
        borderStyle: 'round',
        borderColor: 'green',
        title: 'Output',
        titleAlignment: 'center'
      }));
    } catch (error) {
      console.log(boxen(chalk.red(error.message), {
        padding: 1,
        margin: { top: 0, bottom: 1, left: 2, right: 2 },
        borderStyle: 'round',
        borderColor: 'red',
        title: error.code,
        titleAlignment: 'center'
      }));
    }

    console.log(chalk.dim('─'.repeat(process.stdout.columns * 0.8)));
  }

  console.log(chalk.blue('\nAPI Usage:'));
  console.log(chalk.dim(`
  // Execute a command in a specific directory with arguments
  try {
    const output = executeCommand('git', '/path/to/repo', ['status']);
    console.log('Command output:', output);
  } catch (error) {
    console.error('Command failed:', error.message);
  }
  `));
}

// Execute the demonstration
demonstrateCommandExecution().catch(console.error);
