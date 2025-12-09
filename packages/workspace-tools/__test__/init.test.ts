/**
 * Integration tests for the init command.
 *
 * ## What
 *
 * This module contains comprehensive integration tests for the `init` NAPI function
 * that initializes a workspace with changeset-based version management configuration.
 *
 * ## How
 *
 * Tests are organized into logical groups:
 * - Success tests: Verify the init command creates configuration correctly
 * - Error tests: Verify proper error handling for invalid inputs
 * - Type verification tests: Ensure TypeScript types match the actual response structure
 * - Configuration tests: Verify different configuration options work correctly
 *
 * ## Why
 *
 * Integration tests validate that the Node.js bindings work correctly end-to-end,
 * ensuring the Rust code, NAPI bindings, and TypeScript types are all aligned.
 * The init command is critical as it sets up the workspace for version management.
 *
 * @packageDocumentation
 */

import test from 'ava';
import * as path from 'path';
import * as os from 'os';
import * as fs from 'fs';

import { init } from '../src/index';
import type {
  InitParams,
  InitApiResponse,
  InitData,
  ErrorInfo,
} from '../src/index';

// ============================================================================
// Test Fixtures and Helpers
// ============================================================================

/**
 * Creates a temporary directory for testing.
 * Returns the path to the created directory.
 */
function createTempDir(prefix: string = 'init-test-'): string {
  return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

/**
 * Removes a directory and all its contents recursively.
 */
function removeTempDir(dirPath: string): void {
  if (fs.existsSync(dirPath)) {
    fs.rmSync(dirPath, { recursive: true, force: true });
  }
}

/**
 * Creates a minimal package.json in the given directory.
 */
function createPackageJson(
  dirPath: string,
  name: string = 'test-package',
  version: string = '1.0.0'
): void {
  fs.writeFileSync(
    path.join(dirPath, 'package.json'),
    JSON.stringify({ name, version }, null, 2)
  );
}

/**
 * Initializes a git repository in the given directory.
 */
function initGitRepo(dirPath: string): void {
  const { execSync } = require('child_process');
  execSync('git init', { cwd: dirPath, stdio: 'ignore' });
  execSync('git config user.email "test@test.com"', {
    cwd: dirPath,
    stdio: 'ignore',
  });
  execSync('git config user.name "Test User"', {
    cwd: dirPath,
    stdio: 'ignore',
  });
}

/**
 * Checks if a config file exists with the given format.
 */
function configFileExists(dirPath: string, format: string): boolean {
  const configNames: Record<string, string> = {
    json: 'repo.config.json',
    yaml: 'repo.config.yaml',
    toml: 'repo.config.toml',
  };
  const configFile = configNames[format] || `repo.config.${format}`;
  return fs.existsSync(path.join(dirPath, configFile));
}

/**
 * Checks if the changesets directory exists.
 */
function changesetsDirectoryExists(
  dirPath: string,
  changesetPath: string = '.changesets'
): boolean {
  return fs.existsSync(path.join(dirPath, changesetPath));
}

// ============================================================================
// Success Tests
// ============================================================================

test('init - creates configuration file with default settings', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const params: InitParams = {
      root: tempDir,
    };

    const result: InitApiResponse = await init(params);

    t.true(result.success, 'Init command should succeed');
    t.truthy(result.data, 'Data should be present on success');
    t.is(result.error, undefined, 'Error should not be present on success');

    const data: InitData = result.data as InitData;

    // Verify config file was created
    t.is(
      typeof data.configFile,
      'string',
      'Config file name should be a string'
    );
    t.true(data.configFile.length > 0, 'Config file name should not be empty');
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - creates changesets directory', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({ root: tempDir });

    t.true(result.success);
    t.truthy(result.data);

    const data: InitData = result.data as InitData;

    // Verify changeset path is returned
    t.is(
      typeof data.changesetPath,
      'string',
      'Changeset path should be a string'
    );
    t.true(data.changesetPath.length > 0, 'Changeset path should not be empty');

    // Verify the directory was actually created
    t.true(
      changesetsDirectoryExists(tempDir, data.changesetPath),
      `Changesets directory should exist at ${data.changesetPath}`
    );
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - respects custom changeset path', async (t) => {
  const tempDir = createTempDir();
  const customPath = '.custom-changesets';

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({
      root: tempDir,
      changesetPath: customPath,
    });

    t.true(result.success);
    t.truthy(result.data);

    const data: InitData = result.data as InitData;

    t.is(
      data.changesetPath,
      customPath,
      'Changeset path should match custom value'
    );
    t.true(
      changesetsDirectoryExists(tempDir, customPath),
      `Custom changesets directory should exist at ${customPath}`
    );
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - supports independent strategy', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({
      root: tempDir,
      strategy: 'independent',
    });

    t.true(result.success);
    t.truthy(result.data);

    const data: InitData = result.data as InitData;

    t.is(
      data.strategy,
      'independent',
      'Strategy should be independent'
    );
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - supports unified strategy', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({
      root: tempDir,
      strategy: 'unified',
    });

    t.true(result.success);
    t.truthy(result.data);

    const data: InitData = result.data as InitData;

    t.is(data.strategy, 'unified', 'Strategy should be unified');
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - supports JSON config format', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({
      root: tempDir,
      configFormat: 'json',
    });

    t.true(result.success);
    t.truthy(result.data);

    const data: InitData = result.data as InitData;

    t.is(data.configFormat, 'json', 'Config format should be json');
    t.true(
      configFileExists(tempDir, 'json'),
      'JSON config file should exist'
    );
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - supports YAML config format', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({
      root: tempDir,
      configFormat: 'yaml',
    });

    t.true(result.success);
    t.truthy(result.data);

    const data: InitData = result.data as InitData;

    t.is(data.configFormat, 'yaml', 'Config format should be yaml');
    t.true(
      configFileExists(tempDir, 'yaml'),
      'YAML config file should exist'
    );
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - supports TOML config format', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({
      root: tempDir,
      configFormat: 'toml',
    });

    t.true(result.success);
    t.truthy(result.data);

    const data: InitData = result.data as InitData;

    t.is(data.configFormat, 'toml', 'Config format should be toml');
    t.true(
      configFileExists(tempDir, 'toml'),
      'TOML config file should exist'
    );
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - supports custom registry', async (t) => {
  const tempDir = createTempDir();
  const customRegistry = 'https://npm.pkg.github.com';

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({
      root: tempDir,
      registry: customRegistry,
    });

    t.true(result.success);
    t.truthy(result.data);

    const data: InitData = result.data as InitData;

    t.is(data.registry, customRegistry, 'Registry should match custom value');
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - supports environments configuration', async (t) => {
  const tempDir = createTempDir();
  // Use 'production' instead of 'prod' as that's the CLI default environment
  const environments = ['development', 'staging', 'production'];

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({
      root: tempDir,
      environments,
    });

    t.true(result.success);
    t.truthy(result.data);

    const data: InitData = result.data as InitData;

    t.true(
      Array.isArray(data.environments),
      'Environments should be an array'
    );
    t.deepEqual(
      data.environments,
      environments,
      'Environments should match configured values'
    );
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - supports default environments configuration', async (t) => {
  const tempDir = createTempDir();
  const environments = ['dev', 'staging', 'prod'];
  const defaultEnv = ['prod'];

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({
      root: tempDir,
      environments,
      defaultEnv,
    });

    t.true(result.success);
    t.truthy(result.data);

    const data: InitData = result.data as InitData;

    t.true(
      Array.isArray(data.defaultEnvironments),
      'Default environments should be an array'
    );
    t.deepEqual(
      data.defaultEnvironments,
      defaultEnv,
      'Default environments should match configured values'
    );
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - force flag overwrites existing config', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    // First init
    const firstResult = await init({
      root: tempDir,
      strategy: 'independent',
    });

    t.true(firstResult.success, 'First init should succeed');

    // Second init without force should fail
    const secondResult = await init({
      root: tempDir,
      strategy: 'unified',
    });

    t.false(
      secondResult.success,
      'Second init without force should fail'
    );

    // Third init with force should succeed
    const thirdResult = await init({
      root: tempDir,
      strategy: 'unified',
      force: true,
    });

    t.true(thirdResult.success, 'Third init with force should succeed');
    t.is(
      thirdResult.data?.strategy,
      'unified',
      'Strategy should be updated to unified'
    );
  } finally {
    removeTempDir(tempDir);
  }
});

// ============================================================================
// Error Tests
// ============================================================================

test('init - returns error for non-existent path', async (t) => {
  const nonExistentPath = '/this/path/definitely/does/not/exist/anywhere';

  const result = await init({ root: nonExistentPath });

  t.false(result.success, 'Init should fail for non-existent path');
  t.is(result.data, undefined, 'Data should not be present on failure');
  t.truthy(result.error, 'Error should be present on failure');

  const error: ErrorInfo = result.error as ErrorInfo;

  t.is(typeof error.code, 'string', 'Error code should be a string');
  t.is(typeof error.message, 'string', 'Error message should be a string');
  t.is(typeof error.kind, 'string', 'Error kind should be a string');

  // Error code should be ENOENT or EVALIDATION
  t.true(
    ['ENOENT', 'EVALIDATION'].includes(error.code),
    `Error code should be ENOENT or EVALIDATION, got: ${error.code}`
  );
});

test('init - returns error for empty root path', async (t) => {
  const result = await init({ root: '' });

  t.false(result.success, 'Init should fail for empty root path');
  t.truthy(result.error);

  const error: ErrorInfo = result.error as ErrorInfo;
  t.is(error.code, 'EVALIDATION', 'Error code should be EVALIDATION');
});

test('init - returns error for file path instead of directory', async (t) => {
  const tempDir = createTempDir();
  const tempFile = path.join(tempDir, 'test-file.txt');

  try {
    fs.writeFileSync(tempFile, 'test content');

    const result = await init({ root: tempFile });

    t.false(result.success, 'Init should fail for file path');
    t.truthy(result.error);

    const error: ErrorInfo = result.error as ErrorInfo;
    t.is(error.code, 'EVALIDATION', 'Error code should be EVALIDATION');
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - returns error for invalid strategy', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({
      root: tempDir,
      strategy: 'invalid-strategy',
    });

    t.false(result.success, 'Init should fail for invalid strategy');
    t.truthy(result.error);

    const error: ErrorInfo = result.error as ErrorInfo;
    t.is(error.code, 'EVALIDATION', 'Error code should be EVALIDATION');
    t.true(
      error.message.toLowerCase().includes('strategy') ||
        error.context?.toLowerCase().includes('strategy'),
      'Error should mention strategy'
    );
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - returns error for invalid config format', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({
      root: tempDir,
      configFormat: 'xml', // Invalid format
    });

    t.false(result.success, 'Init should fail for invalid config format');
    t.truthy(result.error);

    const error: ErrorInfo = result.error as ErrorInfo;
    t.is(error.code, 'EVALIDATION', 'Error code should be EVALIDATION');
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - returns error when config exists without force', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    // First init should succeed
    const firstResult = await init({ root: tempDir });
    t.true(firstResult.success);

    // Second init without force should fail
    const secondResult = await init({ root: tempDir });

    t.false(
      secondResult.success,
      'Init should fail when config exists without force'
    );
    t.truthy(secondResult.error);

    const error: ErrorInfo = secondResult.error as ErrorInfo;
    // Could be ECONFIG or EVALIDATION depending on implementation
    t.true(
      ['ECONFIG', 'EVALIDATION', 'EIO'].includes(error.code),
      `Expected ECONFIG, EVALIDATION, or EIO, got: ${error.code}`
    );
  } finally {
    removeTempDir(tempDir);
  }
});

// ============================================================================
// Type Verification Tests
// ============================================================================

test('init - response matches InitApiResponse interface', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result: InitApiResponse = await init({ root: tempDir });

    // Verify all required fields are present
    t.true('success' in result, 'Response should have success field');
    t.is(typeof result.success, 'boolean', 'success should be a boolean');

    if (result.success) {
      t.true('data' in result, 'Successful response should have data field');
      t.is(
        result.error,
        undefined,
        'Successful response should not have error'
      );
    } else {
      t.true('error' in result, 'Failed response should have error field');
      t.is(result.data, undefined, 'Failed response should not have data');
    }
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - data matches InitData interface structure', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({ root: tempDir });

    t.true(result.success);
    const data = result.data as InitData;

    // Verify all required fields are present
    t.true('configFile' in data, 'Data should have configFile field');
    t.true('configFormat' in data, 'Data should have configFormat field');
    t.true('strategy' in data, 'Data should have strategy field');
    t.true('changesetPath' in data, 'Data should have changesetPath field');
    t.true('environments' in data, 'Data should have environments field');
    t.true(
      'defaultEnvironments' in data,
      'Data should have defaultEnvironments field'
    );
    t.true('registry' in data, 'Data should have registry field');

    // Verify types
    t.is(typeof data.configFile, 'string');
    t.is(typeof data.configFormat, 'string');
    t.is(typeof data.strategy, 'string');
    t.is(typeof data.changesetPath, 'string');
    t.true(Array.isArray(data.environments));
    t.true(Array.isArray(data.defaultEnvironments));
    t.is(typeof data.registry, 'string');
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - error matches ErrorInfo interface structure', async (t) => {
  const result = await init({ root: '/invalid/path' });

  t.false(result.success);
  const error = result.error as ErrorInfo;

  // Verify all required fields are present
  t.true('code' in error, 'Error should have code field');
  t.true('message' in error, 'Error should have message field');
  t.true('kind' in error, 'Error should have kind field');

  // context is optional
  t.true(
    error.context === undefined || typeof error.context === 'string',
    'Context should be undefined or a string'
  );
});

// ============================================================================
// Edge Case Tests
// ============================================================================

test('init - handles path with spaces', async (t) => {
  const tempDir = createTempDir('init test with spaces ');

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({ root: tempDir });

    t.true(result.success, 'Init should succeed with spaces in path');
    t.truthy(result.data);
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - handles path with unicode characters', async (t) => {
  const tempBase = os.tmpdir();
  const tempDir = path.join(
    tempBase,
    `init-test-unicode-日本語-émoji-🚀-${Date.now()}`
  );

  try {
    fs.mkdirSync(tempDir, { recursive: true });
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({ root: tempDir });

    t.true(result.success, 'Init should succeed with unicode in path');
    t.truthy(result.data);
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - handles absolute path', async (t) => {
  const tempDir = path.resolve(createTempDir());

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const result = await init({ root: tempDir });

    t.true(result.success, 'Init should succeed with absolute path');
    t.truthy(result.data);
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - handles relative path', async (t) => {
  const tempDir = createTempDir();
  const originalDir = process.cwd();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    // Change to temp dir parent and use relative path
    const parentDir = path.dirname(tempDir);
    const relativePath = path.basename(tempDir);

    process.chdir(parentDir);

    const result = await init({ root: relativePath });

    t.true(result.success, 'Init should succeed with relative path');
    t.truthy(result.data);
  } finally {
    process.chdir(originalDir);
    removeTempDir(tempDir);
  }
});

test('init - works with current directory', async (t) => {
  const tempDir = createTempDir();
  const originalDir = process.cwd();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    process.chdir(tempDir);

    const result = await init({ root: '.' });

    t.true(result.success, 'Init should succeed with "." as root');
    t.truthy(result.data);
  } finally {
    process.chdir(originalDir);
    removeTempDir(tempDir);
  }
});

// ============================================================================
// Full Configuration Tests
// ============================================================================

test('init - creates complete configuration with all options', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const params: InitParams = {
      root: tempDir,
      changesetPath: '.my-changesets',
      environments: ['development', 'staging', 'production'],
      defaultEnv: ['production'],
      strategy: 'independent',
      registry: 'https://registry.npmjs.org',
      configFormat: 'toml',
    };

    const result = await init(params);

    t.true(result.success);
    t.truthy(result.data);

    const data: InitData = result.data as InitData;

    t.is(data.changesetPath, '.my-changesets');
    t.deepEqual(data.environments, ['development', 'staging', 'production']);
    t.deepEqual(data.defaultEnvironments, ['production']);
    t.is(data.strategy, 'independent');
    t.is(data.registry, 'https://registry.npmjs.org');
    t.is(data.configFormat, 'toml');

    // Verify files exist
    t.true(
      configFileExists(tempDir, 'toml'),
      'TOML config file should exist'
    );
    t.true(
      changesetsDirectoryExists(tempDir, '.my-changesets'),
      'Custom changesets directory should exist'
    );
  } finally {
    removeTempDir(tempDir);
  }
});

// ============================================================================
// Performance Tests (basic sanity checks)
// ============================================================================

test('init - completes within reasonable time', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    const startTime = Date.now();

    await init({ root: tempDir });

    const duration = Date.now() - startTime;

    // Init command should complete within 5 seconds
    t.true(
      duration < 5000,
      `Init command took ${duration}ms, expected < 5000ms`
    );
  } finally {
    removeTempDir(tempDir);
  }
});

test('init - can initialize multiple separate directories', async (t) => {
  const tempDirs = [createTempDir(), createTempDir(), createTempDir()];

  try {
    // Set up all directories
    for (const dir of tempDirs) {
      createPackageJson(dir);
      initGitRepo(dir);
    }

    // Initialize all directories
    const results = await Promise.all(
      tempDirs.map((dir) => init({ root: dir }))
    );

    // All should succeed
    for (let i = 0; i < results.length; i++) {
      t.true(
        results[i].success,
        `Init should succeed for directory ${i}`
      );
    }
  } finally {
    for (const dir of tempDirs) {
      removeTempDir(dir);
    }
  }
});
