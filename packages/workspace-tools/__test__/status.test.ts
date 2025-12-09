/**
 * Integration tests for the status command.
 *
 * ## What
 *
 * This module contains comprehensive integration tests for the `status` NAPI function
 * that retrieves workspace information including repository type, package manager,
 * Git branch, pending changesets, and all workspace packages.
 *
 * ## How
 *
 * Tests are organized into logical groups:
 * - Success tests: Verify the status command works with valid workspaces
 * - Error tests: Verify proper error handling for invalid inputs
 * - Type verification tests: Ensure TypeScript types match the actual response structure
 *
 * ## Why
 *
 * Integration tests validate that the Node.js bindings work correctly end-to-end,
 * ensuring the Rust code, NAPI bindings, and TypeScript types are all aligned.
 * This provides confidence that consumers of the library will have a good experience.
 *
 * @packageDocumentation
 */

import test from 'ava';
import * as path from 'path';
import * as os from 'os';
import * as fs from 'fs';

import { status } from '../src/index';
import type {
  StatusParams,
  StatusApiResponse,
  StatusData,
  RepositoryInfo,
  PackageManagerInfo,
  BranchInfo,
  ChangesetInfo,
  PackageInfo,
  ErrorInfo,
} from '../src/index';

// ============================================================================
// Test Fixtures and Helpers
// ============================================================================

/**
 * Get the workspace root directory.
 * This assumes tests are run from the workspace root or package directory.
 */
function getWorkspaceRoot(): string {
  // Navigate up from packages/workspace-tools to the workspace root
  const currentDir = process.cwd();

  // If we're in the workspace root, it should have the root package.json
  if (fs.existsSync(path.join(currentDir, 'pnpm-workspace.yaml'))) {
    return currentDir;
  }

  // If we're in packages/workspace-tools, go up two levels
  const workspaceRoot = path.join(currentDir, '..', '..');
  if (fs.existsSync(path.join(workspaceRoot, 'pnpm-workspace.yaml'))) {
    return workspaceRoot;
  }

  // Last resort: try __dirname based navigation
  const fromDirname = path.join(__dirname, '..', '..', '..');
  if (fs.existsSync(path.join(fromDirname, 'pnpm-workspace.yaml'))) {
    return fromDirname;
  }

  // Fallback to current directory if we can't find workspace root
  return currentDir;
}

/**
 * Creates a temporary directory for testing.
 * Returns the path to the created directory.
 */
function createTempDir(prefix: string = 'status-test-'): string {
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

// ============================================================================
// Success Tests
// ============================================================================

test('status - returns workspace info with proper structure', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  const params: StatusParams = {
    root: workspaceRoot,
  };

  const result: StatusApiResponse = await status(params);

  // Verify success
  t.true(result.success, 'Status command should succeed for valid workspace');
  t.truthy(result.data, 'Data should be present on success');
  t.is(result.error, undefined, 'Error should not be present on success');
});

test('status - returns valid repository info', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  const result = await status({ root: workspaceRoot });

  t.true(result.success);
  t.truthy(result.data);

  const data: StatusData = result.data as StatusData;
  const repository: RepositoryInfo = data.repository;

  // Verify repository info structure
  t.truthy(repository, 'Repository info should be present');
  t.is(typeof repository.kind, 'string', 'Repository kind should be a string');
  t.true(
    ['simple', 'monorepo'].includes(repository.kind),
    `Repository kind should be 'simple' or 'monorepo', got: ${repository.kind}`
  );

  // If it's a monorepo, monorepoType should be present
  if (repository.kind === 'monorepo') {
    t.truthy(
      repository.monorepoType,
      'Monorepo type should be present for monorepo'
    );
  }
});

test('status - returns valid package manager info', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  const result = await status({ root: workspaceRoot });

  t.true(result.success);
  t.truthy(result.data);

  const data: StatusData = result.data as StatusData;
  const packageManager: PackageManagerInfo = data.packageManager;

  // Verify package manager info structure
  t.truthy(packageManager, 'Package manager info should be present');
  t.is(
    typeof packageManager.name,
    'string',
    'Package manager name should be a string'
  );
  t.is(
    typeof packageManager.lockFile,
    'string',
    'Lock file should be a string'
  );

  // Verify it's a known package manager
  const knownPackageManagers = ['npm', 'yarn', 'pnpm', 'bun', 'jsr', 'unknown'];
  t.true(
    knownPackageManagers.includes(packageManager.name),
    `Package manager should be one of ${knownPackageManagers.join(', ')}, got: ${packageManager.name}`
  );
});

test('status - returns valid branch info when in git repository', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  const result = await status({ root: workspaceRoot });

  t.true(result.success);
  t.truthy(result.data);

  const data: StatusData = result.data as StatusData;
  const branch: BranchInfo | undefined = data.branch;

  // Branch may be undefined if not in a git repo or in detached HEAD
  if (branch !== undefined) {
    t.is(typeof branch.name, 'string', 'Branch name should be a string');
    t.true(branch.name.length > 0, 'Branch name should not be empty');
  } else {
    t.pass('Branch info is optional and may be undefined');
  }
});

test('status - returns valid changesets array', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  const result = await status({ root: workspaceRoot });

  t.true(result.success);
  t.truthy(result.data);

  const data: StatusData = result.data as StatusData;
  const changesets: Array<ChangesetInfo> = data.changesets;

  // Verify changesets is an array
  t.true(Array.isArray(changesets), 'Changesets should be an array');

  // Verify each changeset has the expected structure
  for (const changeset of changesets) {
    t.is(typeof changeset.id, 'string', 'Changeset id should be a string');
    t.true(changeset.id.length > 0, 'Changeset id should not be empty');
  }
});

test('status - returns valid packages array', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  const result = await status({ root: workspaceRoot });

  t.true(result.success);
  t.truthy(result.data);

  const data: StatusData = result.data as StatusData;
  const packages: Array<PackageInfo> = data.packages;

  // Verify packages is an array with at least one package
  t.true(Array.isArray(packages), 'Packages should be an array');
  t.true(packages.length > 0, 'Packages array should not be empty');

  // Verify each package has the expected structure
  for (const pkg of packages) {
    t.is(typeof pkg.name, 'string', 'Package name should be a string');
    t.true(pkg.name.length > 0, 'Package name should not be empty');

    t.is(typeof pkg.version, 'string', 'Package version should be a string');
    t.true(pkg.version.length > 0, 'Package version should not be empty');

    t.is(typeof pkg.path, 'string', 'Package path should be a string');
  }
});

test('status - works with current directory as root', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  // Save current directory and change to workspace root
  const originalDir = process.cwd();
  process.chdir(workspaceRoot);

  try {
    const result = await status({ root: '.' });

    t.true(result.success, 'Status should succeed with "." as root');
    t.truthy(result.data);
  } finally {
    // Restore original directory
    process.chdir(originalDir);
  }
});

test('status - works with absolute path', async (t) => {
  const workspaceRoot = path.resolve(getWorkspaceRoot());

  const result = await status({ root: workspaceRoot });

  t.true(result.success, 'Status should succeed with absolute path');
  t.truthy(result.data);
});

test('status - returns pnpm as package manager for this workspace', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  const result = await status({ root: workspaceRoot });

  t.true(result.success);
  t.truthy(result.data);

  const data: StatusData = result.data as StatusData;

  // This workspace uses pnpm
  t.is(
    data.packageManager.name,
    'pnpm',
    'This workspace should use pnpm as package manager'
  );
  t.is(
    data.packageManager.lockFile,
    'pnpm-lock.yaml',
    'Lock file should be pnpm-lock.yaml'
  );
});

test('status - identifies this as a monorepo', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  const result = await status({ root: workspaceRoot });

  t.true(result.success);
  t.truthy(result.data);

  const data: StatusData = result.data as StatusData;

  // This workspace is a monorepo
  t.is(data.repository.kind, 'monorepo', 'This workspace should be a monorepo');
  t.is(
    data.repository.monorepoType,
    'pnpm',
    'Monorepo type should be pnpm'
  );
});

// ============================================================================
// Error Tests
// ============================================================================

test('status - returns error for non-existent path', async (t) => {
  const nonExistentPath = '/this/path/definitely/does/not/exist/anywhere';

  const result = await status({ root: nonExistentPath });

  t.false(result.success, 'Status should fail for non-existent path');
  t.is(result.data, undefined, 'Data should not be present on failure');
  t.truthy(result.error, 'Error should be present on failure');

  const error: ErrorInfo = result.error as ErrorInfo;

  // Verify error structure
  t.is(typeof error.code, 'string', 'Error code should be a string');
  t.is(typeof error.message, 'string', 'Error message should be a string');
  t.is(typeof error.kind, 'string', 'Error kind should be a string');

  // Error code should be ENOENT (not found) or EVALIDATION
  t.true(
    ['ENOENT', 'EVALIDATION'].includes(error.code),
    `Error code should be ENOENT or EVALIDATION, got: ${error.code}`
  );
});

test('status - returns error for empty root path', async (t) => {
  const result = await status({ root: '' });

  t.false(result.success, 'Status should fail for empty root path');
  t.truthy(result.error);

  const error: ErrorInfo = result.error as ErrorInfo;
  t.is(error.code, 'EVALIDATION', 'Error code should be EVALIDATION');
  t.true(
    error.message.length > 0,
    'Error message should not be empty'
  );
});

test('status - returns error for file path instead of directory', async (t) => {
  // Create a temp file
  const tempDir = createTempDir();
  const tempFile = path.join(tempDir, 'test-file.txt');

  try {
    fs.writeFileSync(tempFile, 'test content');

    const result = await status({ root: tempFile });

    t.false(result.success, 'Status should fail for file path');
    t.truthy(result.error);

    const error: ErrorInfo = result.error as ErrorInfo;
    t.is(error.code, 'EVALIDATION', 'Error code should be EVALIDATION');
    t.true(
      error.message.toLowerCase().includes('directory') ||
        error.message.toLowerCase().includes('not a directory'),
      'Error message should mention directory requirement'
    );
  } finally {
    removeTempDir(tempDir);
  }
});

test('status - error contains helpful message for validation errors', async (t) => {
  const result = await status({ root: '/definitely/not/a/valid/path' });

  t.false(result.success);
  t.truthy(result.error);

  const error: ErrorInfo = result.error as ErrorInfo;

  // Error message should be helpful
  t.true(error.message.length > 10, 'Error message should be descriptive');

  // Error kind should indicate the type of error
  t.true(
    ['Validation', 'Io'].includes(error.kind),
    `Error kind should be Validation or Io, got: ${error.kind}`
  );
});

test('status - handles directory without workspace config', async (t) => {
  // Create a temp directory without any workspace config
  const tempDir = createTempDir();

  try {
    // Create a minimal package.json to make it a valid node project
    fs.writeFileSync(
      path.join(tempDir, 'package.json'),
      JSON.stringify({ name: 'test-package', version: '1.0.0' })
    );

    const result = await status({ root: tempDir });

    // This should either succeed (if it detects a simple repo) or fail gracefully
    // The behavior depends on whether the CLI requires workspace config
    if (result.success) {
      t.truthy(result.data);
      t.is(
        result.data?.repository.kind,
        'simple',
        'Should be detected as simple repository'
      );
    } else {
      t.truthy(result.error);
      t.is(typeof result.error?.code, 'string');
    }
  } finally {
    removeTempDir(tempDir);
  }
});

// ============================================================================
// Type Verification Tests
// ============================================================================

test('status - response matches StatusApiResponse interface', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  const result: StatusApiResponse = await status({ root: workspaceRoot });

  // Verify all required fields are present
  t.true('success' in result, 'Response should have success field');
  t.is(typeof result.success, 'boolean', 'success should be a boolean');

  if (result.success) {
    t.true('data' in result, 'Successful response should have data field');
    t.is(result.error, undefined, 'Successful response should not have error');
  } else {
    t.true('error' in result, 'Failed response should have error field');
    t.is(result.data, undefined, 'Failed response should not have data');
  }
});

test('status - data matches StatusData interface structure', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  const result = await status({ root: workspaceRoot });

  t.true(result.success);
  const data = result.data as StatusData;

  // Verify all required fields are present
  t.true('repository' in data, 'Data should have repository field');
  t.true('packageManager' in data, 'Data should have packageManager field');
  t.true('changesets' in data, 'Data should have changesets field');
  t.true('packages' in data, 'Data should have packages field');

  // branch is optional
  t.true(
    data.branch === undefined || 'name' in data.branch,
    'Branch should be undefined or have name field'
  );
});

test('status - error matches ErrorInfo interface structure', async (t) => {
  const result = await status({ root: '/invalid/path' });

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

test('status - handles path with spaces', async (t) => {
  const tempDir = createTempDir('status test with spaces ');

  try {
    // Create a minimal package.json
    fs.writeFileSync(
      path.join(tempDir, 'package.json'),
      JSON.stringify({ name: 'test-package', version: '1.0.0' })
    );

    const result = await status({ root: tempDir });

    // Should not throw and should return a valid response
    t.true('success' in result);
  } finally {
    removeTempDir(tempDir);
  }
});

test('status - handles path with unicode characters', async (t) => {
  const tempBase = os.tmpdir();
  const tempDir = path.join(tempBase, `status-test-unicode-日本語-émoji-🚀-${Date.now()}`);

  try {
    fs.mkdirSync(tempDir, { recursive: true });

    // Create a minimal package.json
    fs.writeFileSync(
      path.join(tempDir, 'package.json'),
      JSON.stringify({ name: 'test-package', version: '1.0.0' })
    );

    const result = await status({ root: tempDir });

    // Should not throw and should return a valid response
    t.true('success' in result);
  } finally {
    removeTempDir(tempDir);
  }
});

test('status - handles relative path correctly', async (t) => {
  const workspaceRoot = getWorkspaceRoot();
  const originalDir = process.cwd();

  try {
    // Change to a directory and use relative path
    process.chdir(workspaceRoot);

    // Use a relative path to a subdirectory
    const result = await status({ root: './packages/workspace-tools' });

    // This might succeed (if it's a valid package) or fail (if it needs workspace root)
    // The important thing is it doesn't throw
    t.true('success' in result);
  } finally {
    process.chdir(originalDir);
  }
});

// ============================================================================
// Performance Tests (basic sanity checks)
// ============================================================================

test('status - completes within reasonable time', async (t) => {
  const workspaceRoot = getWorkspaceRoot();
  const startTime = Date.now();

  await status({ root: workspaceRoot });

  const duration = Date.now() - startTime;

  // Status command should complete within 5 seconds for a reasonable workspace
  t.true(duration < 5000, `Status command took ${duration}ms, expected < 5000ms`);
});

test('status - can be called multiple times', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  // Call status multiple times to ensure no resource leaks or state issues
  const results = await Promise.all([
    status({ root: workspaceRoot }),
    status({ root: workspaceRoot }),
    status({ root: workspaceRoot }),
  ]);

  // All calls should succeed
  for (const result of results) {
    t.true(result.success, 'All parallel status calls should succeed');
  }

  // All results should be equivalent
  t.deepEqual(results[0].data, results[1].data, 'Results should be consistent');
  t.deepEqual(results[1].data, results[2].data, 'Results should be consistent');
});
